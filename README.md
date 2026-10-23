# 曦码·曜 (Xime Yao) 输入法 (Windows)

基于 RIME 引擎的 Windows 五笔输入法，使用 Rust + TSF 构建。

> ⚠️ 注意：本项目还在开发中，请勿用于生产环境。

## 快速开始

```powershell
# 开发构建（构建+注册+启动服务器）
powershell -ExecutionPolicy Bypass -File .\rebuild.ps1

# 打包 MSI 安装包
.\msi-build.ps1

# 打包 MSIX 应用包
.\msix-bundle.ps1

# 打包并测试安装（无需签名）
.\msix-bundle.ps1 -InstallUnsigned
```

## 安装

### MSI 安装（管理员权限）

```powershell
msiexec /i target\wix\ximeyao-{version}-x86_64.msi
```

### MSIX 安装

```powershell
# 打包并直接安装（开发测试，无需签名）
.\msix-bundle.ps1 -InstallUnsigned

# 打包、签名并安装
.\msix-bundle.ps1 -Sign

# 生成未签名 MSIX（用于商店提交）
.\msix-bundle.ps1
```

安装后按 `Win+Space` 切换到曦码·曜（Xime Yao）输入法。

## GPU 加速包（可选）

默认 MSI 是 **CPU 包**。GPU 版用 `.\msi-build.ps1 -Gpu` 单独构建、单独分发（文件名带
`-cuda` 后缀），**两个包不能混装**：它们的 `onnxruntime.dll` 同名不同体，装到同一目录会
互相覆盖。

| | CPU 包 | GPU 包 |
|---|---|---|
| 语音识别后端 | CPU | 优先 CUDA，缺运行库时自动回退 CPU |
| 额外文件 | — | `onnxruntime_providers_cuda.dll`（约 255 MB） |
| 目标机要求 | — | **CUDA 13 运行库**（`cublas64_13.dll`、`cublasLt64_13.dll`）**+ cuDNN 9**（`cudnn64_9.dll`） |

CUDA 运行库加起来约 1 GB，**不随安装包分发**，由目标机自己装（NVIDIA 驱动附带的运行时
或 CUDA 13 Toolkit）。装没装成，看 `%APPDATA%\Xime\logs\server.log`：

- `语音识别使用 CUDA（GPU）` → 走 GPU
- `未找到 CUDA 运行库（cuBLAS）` 或 `未找到 cuDNN 9` → 回退 CPU（输入法本身不受影响）

缺库为什么不当成报错：`onnxruntime` 建 CUDA 会话时缺这些 DLL 会**直接 abort 进程**
（C++ 异常穿过 FFI 边界，Rust 兜不住），所以 server 在开会话前先用
`speech::cuda_runtime_ready()` 探测，缺了就老老实实走 CPU。探测发生在 server 进程内、
按 exe 目录 → System32 → PATH 查 DLL，因此**不放在 MSI 里**（FileSearch 探不到 PATH 上的
CUDA bin 目录）。GPU 包的安装完成页会带一行运行库要求说明。

## 开发构建

### 前置要求

- Rust 工具链（nightly，项目自带 `rust-toolchain.toml`）
- Visual Studio 构建工具（C++ 支持）
- CMake
- **LLVM（clang）**：`libquickjs-ng-sys` 的 build.rs 在 Windows 上硬编码用 `clang` 编
  quickjs；`rebuild.ps1` / `msi-build.ps1` 会自动把 `C:\Program Files\LLVM\bin` 加进 PATH
- **librime-lua 插件**（可选但强烈建议）：RIME 只从 `librime/plugins/<name>` 发现插件，
  仓库不携带该源码。缺了不会编译失败，但 `rime.dll` 不含 lua，依赖 lua 的方案
  （注释格式/农历/计算器/日期等）会缺功能：
  ```powershell
  git clone --depth 1 https://github.com/hchunhui/librime-lua.git D:\project\third_party\librime-lua
  git clone --depth 1 -b thirdparty https://github.com/hchunhui/librime-lua.git D:\project\third_party\librime-lua\thirdparty
  cmd /c mklink /J "D:\project\libximecore\librime\plugins\lua" "D:\project\third_party\librime-lua"
  ```
  回退：删掉这个目录联接后重新构建即可。
- WiX Toolset v3.14（用于 MSI 打包）
- Windows SDK（用于 MSIX 打包）

### 数据目录结构

```
Program Files\Xime\
├── winxime-server.exe        # IPC 服务器进程
├── winxime-setup.exe         # 设置界面
├── winxime_tsf.dll           # TSF 输入法 DLL
├── winxime-tsf-register.exe  # TSF 注册工具
├── rime.dll                  # RIME 引擎
├── data/                     # RIME 基础数据（key_bindings 等）
├── user-data/                # 方案文件（首次启动部署到 %APPDATA%）
└── resources/                # 应用资源

%APPDATA%\Xime\rime\          # 用户数据目录（方案文件、用户配置）
```

## 项目结构

| 目录 | 说明 |
|------|------|
| `crates/winxime-server/` | IPC 服务器（候选栏渲染 + RIME 引擎 + 托盘图标 + 系统通知） |
| `crates/winxime-setup/` | 设置界面（iced） |
| `crates/winxime-tsf/` | TSF 输入法 DLL |
| `crates/winxime-tsf-register/` | TSF 注册/卸载工具 |
| `crates/winxime-ipc/` | 命名管道 IPC 协议 |
| `rime-wubi/` | 五笔方案文件（.schema.yaml, .dict.yaml 等） |

外部依赖：
| 仓库 | 用途 |
|------|------|
| `libximecore/` | RIME 引擎绑定 + 配置管理 + 共享库（**各客户端共用**，改动需顾及 Android/Linux/macOS） |
| `librime/` | RIME 引擎 C 库（子模块，源码只读；插件目录用联接挂载，见「前置要求」） |

## 用户配置（`xime.custom.yaml`）

优先级：**内置 `xime.yaml`** → **系统级 `data/xime.yaml`** → **用户级
`%APPDATA%\Xime\rime\xime.custom.yaml`**。

- 用户文件**只记录与默认值不同的部分**，加载时按 YAML 树深合并；程序保存时
  只改自己负责的顶层键（`style` / `color_schemes` / `wubi_radicals` / `pair_secret` /
  `log_level`），**其余键（如移动端的 `keyboard:`）原样保留**。
- 每次保存前留快照：`xime.custom.yaml.bak`（撤销上一次修改）、
  `xime.custom.yaml.orig`（第一次被程序改写前的那份，用于「恢复为用户提供的配置」）。
- 设置页「外观 → 操作」提供：撤销上一次修改 / 恢复为用户提供的配置 / 恢复默认配置
  （恢复默认 = 删除用户文件，快照保留）。
- 已知取舍：结构化读写**不保留 YAML 注释**。

## 打包

### MSI

```powershell
.\msi-build.ps1
# 输出: target\wix\ximeyao-{version}-x86_64.msi
```

### MSIX

```powershell
# 开发测试（直接注册）
.\msix-bundle.ps1 -Register

# 生成未签名包（商店提交）
.\msix-bundle.ps1

# 生成并安装未签名包
.\msix-bundle.ps1 -InstallUnsigned

# 签名包（自签名证书自动安装到受信任根）
.\msix-bundle.ps1 -Sign

# 输出: target\wix\ximeyao-{version}-x86_64.msix
```

## 许可证

GPL-3.0-or-later

本项目链接 librime (BSD-3-Clause)、opencc (Apache-2.0)、yaml-cpp (MIT)、leveldb (BSD-3-Clause) 等库。
