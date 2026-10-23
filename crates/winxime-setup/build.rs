fn main() {
    #[cfg(windows)]
    {
        let mut res = winres::WindowsResource::new();
        res.set_icon("../../resources/icon.ico");
        if let Err(e) = res.compile() {
            // 资源编译失败不应静默：exe 缺图标/清单会导致安装后外观不对，
            // 直接在构建期失败并说明原因。
            println!("cargo:warning=嵌入 Windows 资源失败: {e}");
            std::process::exit(1);
        }
    }
}
