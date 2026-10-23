use std::sync::{Arc, Mutex, OnceLock};
use std::time::SystemTime;
use windows::Win32::Graphics::Direct2D::Common::D2D1_COLOR_F;
use xime_config::XimeConfig;

/// 进程级配置缓存。
///
/// `XimeConfig::load()` 每次都要解析内嵌 yaml + 系统/用户两份文件 + 两轮 merge，
/// 而候选栏**每次按键刷新**都会取配置（`RootModel` / `CandidateModel` 各一次，
/// 五笔根字提示 `ShowRoot` 还有一次）。
///
/// 有效性靠**用户配置文件的修改时间**判断，而不是等人来通知：
/// - 设置程序所有保存都走 `XimeConfig::save()` → 原子写 `user_config_path()`，
///   mtime 必变；而 `set_notify_reload_style` 这个钩子在 Windows 侧根本没注册，
///   等通知的话样式改了要重启才生效。
/// - 系统配置只在安装时变，那时进程本来就会重启；`ReloadConfig` 另走 [`invalidate`]。
struct Entry {
    config: Arc<XimeConfig>,
    /// 解析时用户配置的 mtime（文件不存在则 None）。
    stamp: Option<SystemTime>,
}

static CACHE: OnceLock<Mutex<Option<Entry>>> = OnceLock::new();

fn cache() -> &'static Mutex<Option<Entry>> {
    CACHE.get_or_init(|| Mutex::new(None))
}

fn user_config_stamp() -> Option<SystemTime> {
    std::fs::metadata(XimeConfig::user_config_path())
        .and_then(|meta| meta.modified())
        .ok()
}

/// 取当前配置：文件没变就返回缓存，变了（或首次）重新解析。
pub fn get() -> Arc<XimeConfig> {
    let stamp = user_config_stamp();
    let mut guard = match cache().lock() {
        Ok(g) => g,
        Err(poisoned) => poisoned.into_inner(),
    };
    if let Some(entry) = guard.as_ref() {
        if entry.stamp == stamp {
            return Arc::clone(&entry.config);
        }
    }
    let config = Arc::new(XimeConfig::load());
    *guard = Some(Entry {
        config: Arc::clone(&config),
        stamp,
    });
    config
}

/// 强制丢弃缓存，下次 [`get`] 重新读盘（`ReloadConfig` 后调用）。
pub fn invalidate() {
    let mut guard = match cache().lock() {
        Ok(g) => g,
        Err(poisoned) => poisoned.into_inner(),
    };
    *guard = None;
}

pub fn hex_to_rgb(hex: u32) -> (f32, f32, f32) {
    let r = ((hex >> 16) & 0xFF) as f32 / 255.0;
    let g = ((hex >> 8) & 0xFF) as f32 / 255.0;
    let b = (hex & 0xFF) as f32 / 255.0;
    (r, g, b)
}

pub fn get_colors(
    primary_color: u32,
) -> (
    D2D1_COLOR_F,
    D2D1_COLOR_F,
    D2D1_COLOR_F,
    D2D1_COLOR_F,
    D2D1_COLOR_F,
    D2D1_COLOR_F,
    D2D1_COLOR_F,
) {
    let primary = hex_to_rgb(primary_color);

    let bg_color = D2D1_COLOR_F {
        r: 0.95,
        g: 0.95,
        b: 0.95,
        a: 0.85,
    };
    let border_color = D2D1_COLOR_F {
        r: 0.0,
        g: 0.0,
        b: 0.0,
        a: 0.05,
    };
    let fg_color = D2D1_COLOR_F {
        r: 0.2,
        g: 0.2,
        b: 0.2,
        a: 1.0,
    };
    let selkey_color = D2D1_COLOR_F {
        r: 0.4,
        g: 0.4,
        b: 0.4,
        a: 1.0,
    };
    let comment_color = D2D1_COLOR_F {
        r: 0.5,
        g: 0.5,
        b: 0.5,
        a: 0.7,
    };
    let highlight_bg_color = D2D1_COLOR_F {
        r: primary.0,
        g: primary.1,
        b: primary.2,
        a: 0.9,
    };
    let highlight_fg_color = D2D1_COLOR_F {
        r: 1.0,
        g: 1.0,
        b: 1.0,
        a: 1.0,
    };

    (
        bg_color,
        border_color,
        fg_color,
        selkey_color,
        comment_color,
        highlight_bg_color,
        highlight_fg_color,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 缓存要"命中同一份"，`invalidate` 之后要真的重新解析。
    /// 两件事写在同一个测试里：缓存是进程级全局状态，拆成两个测试会互相干扰。
    #[test]
    fn config_cache_hits_and_invalidates() {
        let first = get();
        let second = get();
        assert!(Arc::ptr_eq(&first, &second), "配置未变时应命中缓存");

        invalidate();
        let third = get();
        assert!(!Arc::ptr_eq(&first, &third), "invalidate 之后应重新解析");
    }
}