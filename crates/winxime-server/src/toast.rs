//! Windows 系统通知（WinRT ToastNotification）。
//!
//! 仅在 MSIX 包内运行时可用（包进程有 package identity，AUMID =
//! `{PackageFullName}!{AppId}`）；非打包环境（开发直跑）静默跳过，
//! 页面底部消息不受影响。

#![cfg(windows)]

use windows::Data::Xml::Dom::XmlDocument;
use windows::UI::Notifications::{ToastNotification, ToastNotificationManager};
use windows::Win32::Storage::Packaging::Appx::GetCurrentPackageFullName;
use windows::Win32::System::Com::{CoInitializeEx, COINIT_MULTITHREADED};
use windows_core::{HSTRING, PWSTR};

/// MSIX 清单里的应用入口 Id（crates/winxime-server/msix/AppxManifest.xml）。
const PACKAGE_APP_ID: &str = "XimeServer";

/// 非打包进程用的固定 AUMID。
///
/// Win32（MSI 安装 / 开发直跑）进程没有包身份，WinRT toast 靠**开始菜单快捷方式上的
/// `System.AppUserModel.ID`** 认领应用；这里创建的快捷方式与它必须一致。
const APP_AUMID: &str = "XimeYao.InputMethod";

/// 弹系统通知的 AUMID：优先包身份，其次（并确保）开始菜单快捷方式。
fn toast_aumid() -> Option<String> {
    if let Some(package) = package_aumid() {
        return Some(package);
    }
    ensure_start_menu_shortcut().then(|| APP_AUMID.to_string())
}

/// 确保开始菜单里有带 AUMID 的快捷方式；每个进程只做一次。
fn ensure_start_menu_shortcut() -> bool {
    static OK: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *OK.get_or_init(|| match create_start_menu_shortcut() {
        Ok(()) => {
            tracing::info!("已创建通知用快捷方式（AUMID={APP_AUMID}）");
            true
        }
        Err(e) => {
            tracing::warn!("创建通知快捷方式失败（将退回托盘气泡）: {e}");
            false
        }
    })
}

/// 在开始菜单建 `曦码·曜.lnk`，指向当前 exe，并写入 `System.AppUserModel.ID`。
///
/// 这是「无包身份应用发 toast」的官方做法：Windows 按 AUMID 在开始菜单里找到快捷方式，
/// 才允许该应用发通知（也才有名字/图标）。每次启动覆盖写一次，exe 路径变了也能自愈。
fn create_start_menu_shortcut() -> Result<(), String> {
    use windows::core::{HSTRING, Interface, GUID, PWSTR};
    use windows::Win32::Foundation::PROPERTYKEY;
    use windows::Win32::System::Com::StructuredStorage::{
        PropVariantClear, PROPVARIANT, PROPVARIANT_0, PROPVARIANT_0_0, PROPVARIANT_0_0_0,
    };
    use windows::Win32::System::Com::{
        CoCreateInstance, CoTaskMemAlloc, IPersistFile, CLSCTX_INPROC_SERVER,
    };
    use windows::Win32::System::Variant::VT_LPWSTR;
    use windows::Win32::UI::Shell::PropertiesSystem::IPropertyStore;
    use windows::Win32::UI::Shell::{IShellLinkW, ShellLink};

    /// `PKEY_AppUserModel_ID` = {9F4C2855-9F79-4B39-A8D0-E1D42DE1D5F3}, pid 5。
    /// windows-rs 没生成这个常量，按 propkey.h 手写（值不会变）。
    const PKEY_APP_USER_MODEL_ID: PROPERTYKEY = PROPERTYKEY {
        fmtid: GUID::from_u128(0x9F4C2855_9F79_4B39_A8D0_E1D42DE1D5F3),
        pid: 5,
    };

    let exe = std::env::current_exe().map_err(|e| format!("取 exe 路径失败: {e}"))?;
    // 快捷方式指向**设置程序**（更符合直觉）：点开会进设置，而不是重启输入法服务。
    // 拿不到设置程序（异常布局）时退回自己。
    let target = exe
        .parent()
        .map(|dir| dir.join("winxime-setup.exe"))
        .filter(|path| path.is_file())
        .unwrap_or_else(|| exe.clone());
    let appdata = std::env::var_os("APPDATA").ok_or("没有 APPDATA")?;
    let dir = std::path::PathBuf::from(appdata)
        .join("Microsoft")
        .join("Windows")
        .join("Start Menu")
        .join("Programs");
    std::fs::create_dir_all(&dir).map_err(|e| format!("建开始菜单目录失败: {e}"))?;
    let link = dir.join("曦码·曜.lnk");

    let shell_link: IShellLinkW =
        unsafe { CoCreateInstance(&ShellLink, None, CLSCTX_INPROC_SERVER) }
            .map_err(|e| format!("CoCreateInstance(ShellLink) 失败: {e}"))?;
    unsafe {
        shell_link
            .SetPath(&HSTRING::from(target.to_string_lossy().as_ref()))
            .map_err(|e| format!("SetPath 失败: {e}"))?;
        shell_link
            .SetDescription(&HSTRING::from("曦码·曜 设置"))
            .map_err(|e| format!("SetDescription 失败: {e}"))?;

        let store: IPropertyStore = shell_link
            .cast()
            .map_err(|e| format!("取 IPropertyStore 失败: {e}"))?;

        // PROPVARIANT(VT_LPWSTR)：内存必须由 COM 分配器给（PropVariantClear 负责释放）。
        let wide: Vec<u16> = APP_AUMID
            .encode_utf16()
            .chain(std::iter::once(0))
            .collect();
        let bytes = wide.len() * std::mem::size_of::<u16>();
        let raw = CoTaskMemAlloc(bytes) as *mut u16;
        if raw.is_null() {
            return Err("CoTaskMemAlloc 失败".to_string());
        }
        std::ptr::copy_nonoverlapping(wide.as_ptr(), raw, wide.len());
        // 逐字段构造（`ManuallyDrop` 的联合体字段不能直接赋值）。
        let inner = PROPVARIANT_0_0 {
            vt: VT_LPWSTR,
            Anonymous: PROPVARIANT_0_0_0 {
                pwszVal: PWSTR(raw),
            },
            ..Default::default()
        };
        let mut value = PROPVARIANT::default();
        value.Anonymous = PROPVARIANT_0 {
            Anonymous: std::mem::ManuallyDrop::new(inner),
        };

        let set_result = store.SetValue(&PKEY_APP_USER_MODEL_ID, &value);
        let _ = PropVariantClear(&mut value);
        set_result.map_err(|e| format!("写 AppUserModel.ID 失败: {e}"))?;
        store.Commit().map_err(|e| format!("Commit 失败: {e}"))?;

        let persist: IPersistFile = shell_link
            .cast()
            .map_err(|e| format!("取 IPersistFile 失败: {e}"))?;
        persist
            .Save(&HSTRING::from(link.to_string_lossy().as_ref()), true)
            .map_err(|e| format!("保存快捷方式失败: {e}"))?;
    }
    Ok(())
}


/// 弹出系统通知；失败静默（通知属尽力而为的提示，不阻塞设置操作）。
pub fn show_toast(title: &str, body: &str) {
    let title = title.to_string();
    let body = body.to_string();
    let _ = std::thread::Builder::new()
        .name("toast".into())
        .spawn(move || show_toast_blocking(title, body));
}

fn show_toast_blocking(title: String, body: String) {
    // 后台线程调 WinRT 需先初始化 COM 单元（已初始化则忽略）。
    unsafe {
        let _ = CoInitializeEx(None, COINIT_MULTITHREADED);
    }
    match toast_aumid() {
        Some(aumid) => {
            if let Err(e) = show_winrt(&aumid, &title, &body) {
                tracing::warn!("系统通知(WinRT)显示失败: {e}，改用托盘气泡");
                if !crate::tray::show_balloon(&title, &body) {
                    tracing::warn!("托盘气泡也不可用（托盘图标未创建），通知未显示: {body}");
                }
            }
        }
        None => {
            // 连快捷方式都建不出来（受限环境）：退回托盘气泡，仍不依赖包身份。
            if crate::tray::show_balloon(&title, &body) {
                tracing::debug!("系统通知走托盘气泡（AUMID 不可用）: {body}");
            } else {
                tracing::warn!("通知未显示：AUMID 与托盘气泡都不可用: {body}");
            }
        }
    }
}

/// WinRT toast（需要包身份 AUMID）。
fn show_winrt(aumid: &str, title: &str, body: &str) -> windows_core::Result<()> {
    let xml = format!(
        "<toast activationType=\"system\" launch=\"\">\
<visual><binding template=\"ToastGeneric\">\
<text>{}</text><text>{}</text>\
</binding></visual></toast>",
        xml_escape(title),
        xml_escape(body),
    );
    let doc = XmlDocument::new()?;
    doc.LoadXml(&HSTRING::from(xml))?;
    let toast = ToastNotification::CreateToastNotification(&doc)?;
    let notifier = ToastNotificationManager::CreateToastNotifierWithId(&HSTRING::from(aumid))?;
    notifier.Show(&toast)
}

/// 当前包身份的 AUMID；非打包进程返回 None。
fn package_aumid() -> Option<String> {
    use windows::Win32::Foundation::{APPMODEL_ERROR_NO_PACKAGE, ERROR_INSUFFICIENT_BUFFER};
    unsafe {
        // Win32 两段式：首传空缓冲取长度（打包进程返回 INSUFFICIENT_BUFFER，
        // 非打包进程返回 NO_PACKAGE），再传缓冲取全名。
        let mut len = 0u32;
        let err = GetCurrentPackageFullName(&mut len, None);
        if err == APPMODEL_ERROR_NO_PACKAGE {
            return None;
        }
        if err != ERROR_INSUFFICIENT_BUFFER && err.0 != 0 {
            return None;
        }
        let mut buf = vec![0u16; len as usize];
        let err = GetCurrentPackageFullName(&mut len, Some(PWSTR(buf.as_mut_ptr())));
        if err.0 != 0 {
            return None;
        }
        let pfn = String::from_utf16_lossy(&buf[..len as usize]);
        Some(format!("{pfn}!{PACKAGE_APP_ID}"))
    }
}

/// XML 文本转义（& < > " '）。
fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}
