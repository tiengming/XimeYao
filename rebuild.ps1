# Rebuild Xime and test with the real install effect (MSIX development registration).
#
# Flow mirrors msix-bundle.ps1 -Register, so testing happens against the exact
# layout that gets installed:
#   1. build release (windows_subsystem -> no console window)
#   2. stage install layout to target\msix-pkg (binaries + rime.dll + data + user-data + resources)
#   3. Add-AppxPackage -Register (loose-file development registration = install effect)
#   4. start winxime-server.exe from the staged layout (same as MSI's StartServer action)
#
# Notes:
#   - Auto-elevates via UAC (the server self-registers the TSF DLL, which writes HKLM).
#   - target\msix-pkg must stay on disk: the registered package points at that folder
#     (it plays the role of C:\Program Files\WindowsApps for a real install).
#   - User data lives in %APPDATA%\Xime and persists across rebuilds, like a real install.
#   - Logs: %TEMP%\winxime\*.log
#
# 用法：
#   .\rebuild.ps1                        # CPU 包（默认，语音识别走 CPU 推理）
#   .\rebuild.ps1 -Gpu                   # CUDA 包（GPU 加速；需 CUDA 13 运行库的 cuBLAS）
#   .\rebuild.ps1 -Gpu -SherpaLibDir <path>
# CUDA 包额外要求：机器上有 CUDA 13 运行库（cublas64_13.dll + cublasLt64_13.dll）
# 与 cuDNN 9；缺库时 server 会**自动回退 CPU**（日志里有 warn），不会崩。
# 详见 DECISIONS.md「CUDA 包」相关条目。

param(
    # 构建 CUDA 加速包（语音识别走 GPU）。与 CPU 包**分目标目录**构建：
    # 两种 sherpa-onnx 预编译包里的 onnxruntime.dll 同名不同物
    # （CPU 16.97MB / CUDA 15.53MB），同一目录会互相覆盖、链错运行库。
    [switch]$Gpu,
    # CUDA 预编译库目录（内含 onnxruntime_providers_cuda.dll）。
    # 留空则在 $HOME\.cargo\sherpa-archives 下自动找带 cuda 的那个包。
    [string]$SherpaLibDir = ""
)

$ErrorActionPreference = "Stop"

$isAdmin = ([Security.Principal.WindowsPrincipal][Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
if (-not $isAdmin) {
    # 自动以管理员重启本脚本：UAC 确认后在新窗口继续执行（-NoExit 保持窗口显示输出）。
    # **参数要原样带过去**：只传 -File 的话提权后 $Gpu 变回默认值，`-Gpu` 会被
    # 静默丢掉（打出来的其实是 CPU 包，而用户以为在验 GPU）。
    try {
        $argList = @("-NoExit", "-ExecutionPolicy", "Bypass", "-File", "$PSCommandPath")
        if ($Gpu) { $argList += "-Gpu" }
        if ($SherpaLibDir -ne "") { $argList += @("-SherpaLibDir", $SherpaLibDir) }
        Start-Process powershell.exe -Verb RunAs -ArgumentList $argList | Out-Null
    } catch {
        Write-Host "UAC cancelled; run again to retry." -ForegroundColor Red
        exit 1
    }
    exit
}

# cargo 与 msix-bundle.ps1 都按仓库根目录解析相对路径；提权重启后 cwd 是 System32，统一锚定。
Set-Location -LiteralPath $PSScriptRoot

# 文件沙箱探测：server 必须能写 %APPDATA%\Xime（日志/方案注册表/用户词典/剪贴板库）。
# Agent/DSH 的终端常把命令限制在工作区内（workspace-write）：cargo 能写 target/，
# 但 server 一启动就因「拒绝访问」在日志初始化阶段失败，表现为「Server did not start」。
# 提前用探针文件判定，避免把沙箱限制误判成程序 bug。
$probe = Join-Path $env:APPDATA ".xime-rebuild-probe"
try {
    Set-Content -LiteralPath $probe -Value "probe" -Encoding ascii -ErrorAction Stop
    Remove-Item -LiteralPath $probe -Force -ErrorAction Stop
} catch {
    Write-Host "ERROR: 当前终端不能写 $env:APPDATA（文件沙箱或权限限制）。" -ForegroundColor Red
    Write-Host "       server 需要写 $env:APPDATA\Xime（日志/方案注册表/用户词典）；" -ForegroundColor Yellow
    Write-Host "       在受限终端里它会因「拒绝访问」启动失败，这不是程序 bug。" -ForegroundColor Yellow
    Write-Host "       请改用普通管理员 PowerShell / Windows Terminal，或从资源管理器运行本脚本。" -ForegroundColor Yellow
    exit 1
}

# libquickjs-ng-sys 的 build.rs 在非 Android 平台硬编码 `cc.compiler("clang")`：
# clang.exe 必须在 PATH 上，否则 release 构建在 quickjs 这一步直接失败
# （`failed to find tool "clang"`）。`.cargo/config.toml` 里的 LIBCLANG_PATH 只解决
# bindgen 找 libclang.dll，不能让 cc 找到编译器——两件事，别混。
$llvmBin = $env:LIBCLANG_PATH
if ([string]::IsNullOrEmpty($llvmBin)) { $llvmBin = "C:\Program Files\LLVM\bin" }
if ((Test-Path (Join-Path $llvmBin "clang.exe")) -and -not (($env:PATH -split ';') -contains $llvmBin)) {
    $env:PATH = "$llvmBin;$env:PATH"
    Write-Host "LLVM added to PATH: $llvmBin" -ForegroundColor DarkGray
} elseif (-not (Get-Command clang.exe -ErrorAction SilentlyContinue)) {
    Write-Host "WARNING: clang.exe not found (libquickjs-ng-sys needs it to build quickjs)." -ForegroundColor Yellow
}

# librime 只从 `librime/plugins/<name>` 发现插件（上游 action-install-plugins-windows.bat
# 的落点也是这里）。我们用目录联接指向仓库外的 clone，避免改动 librime 源码。
# 缺了不会编译失败，但 rime.dll 里就没有 lua 组件——依赖 lua 的方案会缺功能，所以先提示。
$luaPlugin = Join-Path $PSScriptRoot "..\libximecore\librime\plugins\lua"
if (-not (Test-Path (Join-Path $luaPlugin "CMakeLists.txt"))) {
    Write-Host "WARNING: librime-lua 插件未就绪（$luaPlugin 不存在）" -ForegroundColor Yellow
    Write-Host "  rime.dll 将不含 lua 组件；依赖 lua 的方案（注释格式/农历/计算器等）会缺功能。" -ForegroundColor Yellow
}

$packageDir = "$PSScriptRoot\target\msix-pkg"

Write-Host "Step 0: Clearing old logs..." -ForegroundColor Yellow
Remove-Item "$env:TEMP\winxime\*.log" -Force -ErrorAction SilentlyContinue

Write-Host "Step 1: Stopping server and setup..." -ForegroundColor Yellow
# 优雅退出：用上一次构建的 server /q（IPC shutdown）；都没有时跳过，稍后强制结束兜底。
foreach ($exe in @(
    "$packageDir\winxime-server.exe",
    "$PSScriptRoot\target\release\winxime-server.exe",
    "$PSScriptRoot\target\debug\winxime-server.exe"
)) {
    if (Test-Path $exe) {
        Start-Process -FilePath $exe -ArgumentList "/q" -Wait -ErrorAction SilentlyContinue
        break
    }
}
# setup 也要停：暂存目录被占用会导致复制失败。
# 注意：不能直接 Get-Process | Stop-Process——无匹配进程时管道为空，
# Stop-Process 的必选参数 Id 会进入交互式提示（-ErrorAction 压不住）。
$staleProcesses = Get-Process -Name "winxime-server", "winxime-setup" -ErrorAction SilentlyContinue
if ($staleProcesses) {
    $staleProcesses | Stop-Process -Force -ErrorAction SilentlyContinue
}
Start-Sleep -Seconds 2

# CUDA 包：定位预编译库目录 → 独立目标目录 → 带 feature 构建。
$stageDir = "target\release"
if ($Gpu) {
    if ($SherpaLibDir -eq "") {
        # 与 .cargo\config.toml 的 SHERPA_ONNX_ARCHIVE_DIR 保持同一约定；
        # [env] 只对 cargo 生效，这里自己找（accel 包目录名里带 cuda）。
        $archiveRoot = "$HOME\.cargo\sherpa-archives"
        $candidate = Get-ChildItem $archiveRoot -Directory -Filter "*cuda*" -ErrorAction SilentlyContinue |
            ForEach-Object { Join-Path $_.FullName "lib" } |
            Where-Object { Test-Path (Join-Path $_ "onnxruntime_providers_cuda.dll") } |
            Select-Object -First 1
        $SherpaLibDir = $candidate
    }
    if ([string]::IsNullOrEmpty($SherpaLibDir) -or -not (Test-Path $SherpaLibDir)) {
        Write-Host "找不到 CUDA 版 sherpa-onnx 预编译库（含 onnxruntime_providers_cuda.dll 的 lib 目录）。" -ForegroundColor Red
        Write-Host "  显式指定： .\rebuild.ps1 -Gpu -SherpaLibDir <解压后的 ...-win-x64-cuda\lib>" -ForegroundColor Yellow
        Write-Host "  下载：https://github.com/k2-fsa/sherpa-onnx/releases (sherpa-onnx-v*-cuda-*-win-x64-cuda.tar.bz2)" -ForegroundColor Yellow
        exit 1
    }
    $SherpaLibDir = (Resolve-Path $SherpaLibDir).Path
    $env:SHERPA_ONNX_LIB_DIR = $SherpaLibDir
    $env:CARGO_TARGET_DIR = "target-cuda"
    $stageDir = "target-cuda\release"
    Write-Host "  CUDA 库目录: $SherpaLibDir" -ForegroundColor Gray
    Write-Host "  目标目录:    target-cuda（与 CPU 包隔离，onnxruntime.dll 同名）" -ForegroundColor Gray
}

Write-Host "Step 2: Building release$(if ($Gpu) { ' (CUDA / speech-cuda)' })..." -ForegroundColor Yellow
if ($Gpu) {
    cargo build --release --quiet --features speech-cuda
} else {
    cargo build --release --quiet
}
if ($LASTEXITCODE -ne 0) {
    Write-Host "Build failed!" -ForegroundColor Red
    exit 1
}

Write-Host "Step 3: Staging install layout + MSIX registration..." -ForegroundColor Yellow
& "$PSScriptRoot\msix-bundle.ps1" -Register -TargetDir $stageDir
if ($LASTEXITCODE -ne 0) {
    Write-Host "Staging/registration failed!" -ForegroundColor Red
    exit 1
}

# server 以 Medium 运行（工作区 Low 标签会连带降 exe，必须抬回来），写不了 HKLM；
# TSF 的 DLL 复制与注册由 rebuild（管理员）在这里显式完成。tsf-register 的
# 完整性标签在 msix-bundle 里抬到 High。
$tsfRegister = Join-Path $packageDir "winxime-tsf-register.exe"
$tsfDll = Join-Path $packageDir "winxime_tsf.dll"
$tsfIcon = Join-Path $packageDir "resources\icon.ico"
if ((Test-Path $tsfRegister) -and (Test-Path $tsfDll) -and (Test-Path $tsfIcon)) {
    Write-Host "Step 3b: Registering TSF DLL..." -ForegroundColor Yellow
    & $tsfRegister -copy-and-register $tsfDll $tsfIcon
    if ($LASTEXITCODE -ne 0) {
        Write-Host "TSF registration failed (exit $LASTEXITCODE); server will retry at startup." -ForegroundColor Red
    }
}

Write-Host "Step 4: Starting server from the staged layout..." -ForegroundColor Yellow
$server = Start-Process -FilePath "$packageDir\winxime-server.exe" -PassThru
Start-Sleep -Seconds 3

if ($server -and -not $server.HasExited) {
    Write-Host ""
    Write-Host "Done! Server is running from the installed layout (no console window)." -ForegroundColor Green
    Write-Host "  Package layout: $packageDir" -ForegroundColor White
    Write-Host "  Logs:           $env:APPDATA\Xime\logs\*.log" -ForegroundColor White
    Write-Host "Test input in Notepad or any application." -ForegroundColor White
    Write-Host "NOTE: TSF DLL 是进程内加载的，请关掉重开要测试的应用（旧进程仍用旧 DLL）" -ForegroundColor Yellow
} else {
    # 进程启动后立刻退出时不打印退出码的话，只会看到一句「Server did not start」，
    # 无法区分 loader / panic / 业务早期退出。这里把退出码和日志候选路径都打出来。
    if ($server) {
        Write-Host ("Server exited immediately (exit code 0x{0:X8} / {0})." -f $server.ExitCode) -ForegroundColor Red
    } else {
        Write-Host "Server could not be started." -ForegroundColor Red
    }
    Write-Host "  Logs: $env:APPDATA\Xime\logs\server.log" -ForegroundColor White
    Write-Host "  （若日志初始化失败，server 会把原因写到 stderr，或回退到 $env:TEMP\xime\logs）" -ForegroundColor White
    exit 1
}
