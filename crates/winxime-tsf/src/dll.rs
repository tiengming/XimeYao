use crate::class_factory::CLSID_XIME;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Once;
use tracing::{error, info};
use windows::Win32::Foundation::*;
use windows::Win32::System::LibraryLoader::{GetModuleFileNameW, GetProcAddress, LoadLibraryW};
use windows_core::*;

static DLL_MODULE: AtomicUsize = AtomicUsize::new(0);

/// 初始化 TSF 的日志（进程内只做一次）。
///
/// **不在 `DllMain` 里做**：那里持有加载器锁，建目录 / 起 tracing 的后台写线程
/// 都有死锁风险（历史上这类 bug 极难查）。改在真正被调用的三个入口做——
/// COM 激活、注册、反注册。
///
/// 也**不要在 `Activate` 里直接调 `xime_config::init_logging`**：那个函数会覆盖
/// `LOG_GUARD`，旧 guard 的 Drop 要等后台线程收尾（最坏 ~1.1 秒），而且发生在
/// 输入法激活的宿主 UI 线程上。统一走这里。
pub(crate) fn init_logging_once() {
    static INIT: Once = Once::new();
    INIT.call_once(|| xime_config::init_logging("tsf"));
}

fn get_dll_module() -> HINSTANCE {
    HINSTANCE(DLL_MODULE.load(Ordering::Acquire) as *mut _)
}

fn clsid_str() -> String {
    format!(
        "{{{:08X}-{:04X}-{:04X}-{:02X}{:02X}-{:02X}{:02X}{:02X}{:02X}{:02X}{:02X}}}",
        CLSID_XIME.data1,
        CLSID_XIME.data2,
        CLSID_XIME.data3,
        CLSID_XIME.data4[0],
        CLSID_XIME.data4[1],
        CLSID_XIME.data4[2],
        CLSID_XIME.data4[3],
        CLSID_XIME.data4[4],
        CLSID_XIME.data4[5],
        CLSID_XIME.data4[6],
        CLSID_XIME.data4[7],
    )
}

fn get_module_path() -> String {
    unsafe {
        let mut buf = [0u16; 260];
        let len = GetModuleFileNameW(Some(HMODULE(get_dll_module().0)), &mut buf);
        if len > 0 {
            String::from_utf16_lossy(&buf[..len as usize])
        } else {
            String::new()
        }
    }
}

#[no_mangle]
pub unsafe extern "system" fn DllGetClassObject(
    rclsid: *const GUID,
    riid: *const GUID,
    ppv: *mut *mut core::ffi::c_void,
) -> HRESULT {
    if rclsid.is_null() || riid.is_null() || ppv.is_null() {
        return E_POINTER;
    }
    init_logging_once();
    if *rclsid != CLSID_XIME {
        return HRESULT(-2147221231);
    }
    let factory = crate::ClassFactory;
    let unknown: IUnknown = factory.into();
    Interface::query(&unknown, riid, ppv)
}

static ACTIVE_INSTANCES: AtomicUsize = AtomicUsize::new(0);

pub fn increment_instance_count() {
    ACTIVE_INSTANCES.fetch_add(1, Ordering::AcqRel);
}

pub fn decrement_instance_count() {
    ACTIVE_INSTANCES.fetch_sub(1, Ordering::AcqRel);
}

#[no_mangle]
pub unsafe extern "system" fn DllCanUnloadNow() -> HRESULT {
    if ACTIVE_INSTANCES.load(Ordering::Acquire) > 0 {
        S_FALSE
    } else {
        S_OK
    }
}

#[no_mangle]
pub unsafe extern "system" fn DllRegisterServer() -> HRESULT {
    init_logging_once();
    match do_register() {
        Ok(_) => S_OK,
        Err(e) => {
            // regsvr32 自己会弹失败对话框；这里补上写入日志的失败原因，
            // 因为 TSF 宿主进程通常没有控制台，stderr 谁看不见。
            error!("DllRegisterServer 失败: {}", e);
            HRESULT(-1i32)
        }
    }
}

#[no_mangle]
pub unsafe extern "system" fn DllUnregisterServer() -> HRESULT {
    init_logging_once();
    do_unregister();
    S_OK
}

#[no_mangle]
pub unsafe extern "system" fn DllMain(
    hinst: HINSTANCE,
    reason: u32,
    _reserved: *mut core::ffi::c_void,
) -> BOOL {
    if reason == DLL_PROCESS_ATTACH {
        DLL_MODULE.store(hinst.0 as usize, Ordering::Release);
        crate::language_bar::set_instance(hinst);
        // 日志初始化在 DllGetClassObject / DllRegisterServer 里做（见 init_logging_once）。
    }
    BOOL(1)
}

const DLL_PROCESS_ATTACH: u32 = 1;

fn uninstall_layout() {
    let install_str = format!("0804:{}{{{}}}", clsid_str(), clsid_str(),);
    let wide: Vec<u16> = install_str
        .encode_utf16()
        .chain(std::iter::once(0))
        .collect();
    unsafe {
        if let Ok(module) = LoadLibraryW(w!("input.dll")) {
            // FARPROC 是不带参数签名的函数指针，按已知导出签名重新解释。
            let proc: FARPROC = GetProcAddress(module, s!("InstallLayoutOrTip"));
            if let Some(proc) = proc {
                let install: unsafe extern "system" fn(*const u16, u32) -> BOOL =
                    std::mem::transmute(proc);
                let _ = install(wide.as_ptr(), 1);
            }
        }
    }
}

fn do_register() -> Result<()> {
    use winreg::enums::*;
    use winreg::RegKey;

    let cs = clsid_str();
    let module_path = get_module_path();
    let name = "曦码·曜输入法";

    info!(
        "DllRegisterServer: CLSID={}, module={}",
        cs, module_path
    );

    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let software = hklm
        .create_subkey("Software")
        .map_err(|e| {
            error!("打开 HKLM\\Software 失败: {}", e);
            e
        })?
        .0;

    let classes = software
        .create_subkey("Classes")
        .map_err(|e| {
            error!("打开 HKLM\\Software\\Classes 失败: {}", e);
            e
        })?
        .0;

    info!("Step 1: 创建 CLSID 注册表键");
    let (clsid_key, _) = classes
        .create_subkey(format!("CLSID\\{}", cs))
        .map_err(|e| {
            error!("创建 CLSID 键失败: {}", e);
            e
        })?;

    info!("Step 2: 写入默认值");
    clsid_key.set_value("", &name).map_err(|e| {
        error!("写入默认值失败: {}", e);
        e
    })?;

    info!("Step 3: 创建 InprocServer32");
    let (inproc, _) = clsid_key.create_subkey("InprocServer32").map_err(|e| {
        error!("创建 InprocServer32 失败: {}", e);
        e
    })?;
    inproc.set_value("", &module_path).map_err(|e| {
        error!("写入模块路径失败: {}", e);
        e
    })?;
    inproc
        .set_value("ThreadingModel", &"Apartment")
        .map_err(|e| {
            error!("写入 ThreadingModel 失败: {}", e);
            e
        })?;

    let _ = classes.create_subkey(format!(
        "CLSID\\{}\\Implemented Categories\\{{34745C63-B2F0-4784-8B67-5E12C8701A31}}",
        cs
    ));

    info!("DllRegisterServer 完成（TSF profile / 图标需另跑 winxime-tsf-register）");
    Ok(())
}

fn do_unregister() {
    use winreg::enums::*;
    use winreg::RegKey;

    let cs = clsid_str();

    uninstall_layout();

    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let _ = hklm.delete_subkey_all(format!("Software\\Classes\\CLSID\\{}", cs));
    let _ = hklm.delete_subkey_all(format!("Software\\Microsoft\\CTF\\TIP\\{}", cs));
}
