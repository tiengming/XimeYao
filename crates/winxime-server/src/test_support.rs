//! 单元测试共用的夹具目录。
//!
//! 测试数据根统一落在 workspace 的 `target/test-tmp/`：受限宿主里 `%TEMP%`
//! 不保证可写（技术债基线中 models / schema_switches 失败的根因），而
//! `target/` 本身已被 `.gitignore` 忽略，不会污染工作树。

use std::path::{Path, PathBuf};

/// 取一个干净的测试数据根 `target/test-tmp/<label>_<pid>/`，
/// 并预建模拟 rime 用户目录的 `rime/` 子目录。
///
/// - `label` 要在同一个测试二进制内唯一，惯例是 `模块_场景`（如 `models_layout`）；
/// - 带进程号是因为 `cargo test` 会并行跑多个测试二进制，光是 label 不够隔离。
pub(crate) fn test_root(label: &str) -> PathBuf {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("target")
        .join("test-tmp")
        .join(format!("{label}_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    let _ = std::fs::create_dir_all(root.join("rime"));
    root
}
