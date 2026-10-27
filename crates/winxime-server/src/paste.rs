//! 候选栏/面板「点击即上屏」的注入通道。
//!
//! server 不能直接把文本塞进前台应用的编辑会话：宿主 TSF 是**请求驱动**的，
//! 每条 `ProcessKeyEvent` 请求的回包里才能带 `commit`。所以这里走「自注入触发键」：
//!
//! 1. 把待处理载荷登记到 [`PENDING`]（文本）/ [`PENDING_SELECT`]（rime 候选下标）；
//! 2. `SendInput` 一个 [`VK_PASTE_TRIGGER`]（VK_F24，实体键盘上基本不存在）；
//! 3. 宿主 TSF 照常把这个键上报给 server（它不区分硬件键和注入键）；
//! 4. server 在 `ProcessKeyEvent` 里认出触发键，按载荷分两路：
//!    文本载荷直接当 `commit` 回包；下标载荷交给 `eng.select_candidate(index)`
//!    让 rime 选中（保留词频/学习），再把 rime 产出的 commit 回包
//!    （见 `ipc_server::select_rime_candidate`）。
//!
//! 这样两种点击都走**和打字/选词完全同一条 TSF 编辑会话**上屏，不需要扩 IPC 协议、
//! 不需要改宿主，也不像模拟 Ctrl+V 那样会和「半成品编码串」打架。

use std::sync::Mutex;
use std::time::{Duration, Instant};

use windows::Win32::UI::Input::KeyboardAndMouse::{
    SendInput, VkKeyScanW, INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYBD_EVENT_FLAGS,
    KEYEVENTF_KEYUP, VIRTUAL_KEY,
};

/// 触发键：VK_F24。宿主会把它吃掉（server 回 `success: true`）；
/// 万一宿主没接住（前台应用不是 TSF 应用）漏给应用，F24 也没有任何副作用。
pub const VK_PASTE_TRIGGER: u16 = 0x87;

/// `librime::vk_to_xk(VK_F24)`：0x87 不在 librime 的映射表里，原样返回，
/// 所以 server 收到的是 keycode == 0x87。
pub const XK_PASTE_TRIGGER: i32 = 0x87;

/// 待上屏文本的有效期。触发键注入失败（前台应用完整性更高被 UIPI 拦下、
/// 宿主没在跑、用户此刻在英文态）时，这份文本不该在很久以后被别的触发键取走。
const PENDING_TTL: Duration = Duration::from_millis(1500);

/// 待上屏文本 + 登记时刻。
static PENDING: Mutex<Option<(String, Instant)>> = Mutex::new(None);

/// 待选中的 rime 候选下标 + 登记时刻（点击普通候选时用）。
///
/// 与文本共用同一个触发键，但载荷不同：文本是「直接上屏这段字」，
/// 下标是「让 rime 选中这一条」（保留词频/学习语义）。两者互斥：登记一种就清掉另一种，
/// 避免迟到的旧载荷截胡下一次点击。
static PENDING_SELECT: Mutex<Option<(usize, Instant)>> = Mutex::new(None);

fn lock_pending() -> std::sync::MutexGuard<'static, Option<(String, Instant)>> {
    match PENDING.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    }
}

fn lock_pending_select() -> std::sync::MutexGuard<'static, Option<(usize, Instant)>> {
    match PENDING_SELECT.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    }
}

/// 请求把 `text` 上屏到前台应用的光标处。
///
/// 返回 `false` 表示注入触发键被系统拒绝（典型场景：前台应用以更高完整性级别
/// 运行，UIPI 不允许本进程注入输入）——调用方据此回退到「已复制到剪贴板」提示。
pub fn request_commit(text: &str) -> bool {
    queue_pending(text);
    inject_trigger()
}

/// 请求让 rime 选中当前页的第 `index` 个候选（点击普通候选）。
pub fn request_select(index: usize) -> bool {
    queue_pending_select(index);
    inject_trigger()
}

/// 取走未过期的待上屏文本（`ProcessKeyEvent` 认出触发键时调用）。
pub fn take_pending() -> Option<String> {
    let taken = lock_pending().take();
    match taken {
        Some((text, at)) if at.elapsed() <= PENDING_TTL => Some(text),
        Some((text, _)) => {
            tracing::warn!("待上屏文本已过期（{} 字），丢弃", text.chars().count());
            None
        }
        None => None,
    }
}

/// 取走未过期的待选中候选下标。
pub fn take_pending_select() -> Option<usize> {
    let taken = lock_pending_select().take();
    match taken {
        Some((index, at)) if at.elapsed() <= PENDING_TTL => Some(index),
        Some((index, _)) => {
            tracing::warn!("待选中候选 #{index} 已过期，丢弃");
            None
        }
        None => None,
    }
}

/// 登记待上屏文本（不注入按键，便于单测），并清掉另一种载荷。
fn queue_pending(text: &str) {
    *lock_pending() = Some((text.to_string(), Instant::now()));
    let _ = lock_pending_select().take();
}

/// 登记待选中候选（不注入按键，便于单测），并清掉另一种载荷。
fn queue_pending_select(index: usize) {
    *lock_pending() = None;
    *lock_pending_select() = Some((index, Instant::now()));
}

/// 注入触发键（按下 + 抬起）。
fn inject_trigger() -> bool {
    let inputs = [key_input(VK_PASTE_TRIGGER, false), key_input(VK_PASTE_TRIGGER, true)];
    let sent = unsafe { SendInput(&inputs, std::mem::size_of::<INPUT>() as i32) };
    if sent != inputs.len() as u32 {
        tracing::warn!(
            "上屏触发键注入失败（{}/{} 个事件被接受，可能被 UIPI 拦下）",
            sent,
            inputs.len()
        );
        return false;
    }
    true
}

fn key_input(vk: u16, up: bool) -> INPUT {
    INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT {
                wVk: VIRTUAL_KEY(vk),
                wScan: 0,
                dwFlags: if up {
                    KEYEVENTF_KEYUP
                } else {
                    KEYBD_EVENT_FLAGS(0)
                },
                time: 0,
                dwExtraInfo: 0,
            },
        },
    }
}

/// B2 回放：把一颗按键重新注入输入流（当前键已被影子缓冲吞掉）。
///
/// `keysym` 是宿主换算后的 X11 keysym：可打印 ASCII 用 `VkKeyScanW` 反查 VK 与
/// Shift 状态；控制键（空格/回车/Tab/Esc）和方向键单独映射。
/// 返回 false = 无法映射或 SendInput 被拒（调用方负责兜底提示）。
pub fn inject_key(keysym: i32, modifiers: i32) -> bool {
    let (vk, needs_shift) = match keysym {
        0x20 => (0x20u16, false), // space
        0x0D => (0x0Du16, false), // enter
        0x09 => (0x09u16, false), // tab
        0x1B => (0x1Bu16, false), // esc
        0x08 => (0x08u16, false), // backspace
        0x41..=0x5A => (keysym as u16, false),
        0x61..=0x7A => ((keysym as u16) - 0x20, false),
        0x21..=0x7E => {
            let ch = keysym as u8 as u16;
            let scan = unsafe { VkKeyScanW(ch) };
            if scan == -1 {
                tracing::warn!("回放：字符 0x{ch:02X} 无法映射到 VK");
                return false;
            }
            let scan = scan as u16;
            ((scan & 0xFF) as u16, (scan & 0x100) != 0)
        }
        0xFF51 => (0x25u16, false), // Left
        0xFF52 => (0x26u16, false), // Up
        0xFF53 => (0x27u16, false), // Right
        0xFF54 => (0x28u16, false), // Down
        0xFF50 => (0x24u16, false), // Home
        0xFF57 => (0x23u16, false), // End
        _ => {
            tracing::warn!("回放：keysym 0x{keysym:X} 暂不支持");
            return false;
        }
    };

    let shift_physically_down = modifiers & librime::K_SHIFT_MASK as i32 != 0;
    let inject_shift = needs_shift && !shift_physically_down;
    let mut inputs: Vec<INPUT> = Vec::with_capacity(4);
    if inject_shift {
        inputs.push(key_input(0x10, false)); // Shift down
    }
    inputs.push(key_input(vk, false));
    inputs.push(key_input(vk, true));
    if inject_shift {
        inputs.push(key_input(0x10, true)); // Shift up
    }
    let sent = unsafe { SendInput(&inputs, std::mem::size_of::<INPUT>() as i32) };
    if sent != inputs.len() as u32 {
        tracing::warn!(
            "回放按键注入失败（{}/{} 个事件被接受，可能被 UIPI 拦下）",
            sent,
            inputs.len()
        );
        return false;
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 两种载荷共用静态槽，测试必须串行（cargo test 默认多线程）。
    static TEST_LOCK: Mutex<()> = Mutex::new(());

    fn lock_state() -> std::sync::MutexGuard<'static, ()> {
        match TEST_LOCK.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        }
    }

    /// 待上屏文本「取一次就没了」且**过期作废**。
    ///
    /// 刻意不走 `request_commit()`——那会真的往用户桌面注入按键。
    #[test]
    fn pending_commit_is_taken_once_and_expires() {
        let _guard = lock_state();
        queue_pending("第一段文本");
        assert_eq!(take_pending().as_deref(), Some("第一段文本"));
        assert_eq!(take_pending(), None, "取过一次之后不能再被第二次取走");

        // 手工塞一条过期条目：过期的待上屏文本不该再被触发键取走。
        let expired = Instant::now() - PENDING_TTL - Duration::from_millis(1);
        *lock_pending() = Some(("过期文本".to_string(), expired));
        assert_eq!(take_pending(), None);
    }

    /// 待选中候选：取一次、过期作废，且登记新的会清掉另一种载荷。
    #[test]
    fn pending_select_is_taken_once_and_expires() {
        let _guard = lock_state();
        queue_pending("旧文本");
        queue_pending_select(3);
        assert_eq!(take_pending(), None, "登记 select 应清掉旧文本");
        assert_eq!(take_pending_select(), Some(3));
        assert_eq!(take_pending_select(), None);

        queue_pending_select(7);
        queue_pending("新文本");
        assert_eq!(take_pending_select(), None, "登记文本应清掉旧 select");
        assert_eq!(take_pending().as_deref(), Some("新文本"));

        let expired = Instant::now() - PENDING_TTL - Duration::from_millis(1);
        *lock_pending_select() = Some((9, expired));
        assert_eq!(take_pending_select(), None);
    }

    /// 触发键的 keycode 必须和宿主 `vk_to_xk` 的换算结果一致，否则 server 认不出来。
    #[test]
    fn trigger_keycode_matches_host_conversion() {
        assert_eq!(librime::vk_to_xk(VK_PASTE_TRIGGER), XK_PASTE_TRIGGER);
        // 不能落在字母/数字 keysym 区间：万一触发键漏进 rime，也不会被当成一次输入。
        assert!(
            XK_PASTE_TRIGGER > 'z' as i32,
            "触发键 keycode 不能与可打印字符重叠"
        );
    }
}