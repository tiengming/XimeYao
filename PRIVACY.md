# 隐私政策 / Privacy Policy

生效日期 / Effective Date: 2026年8月15日 / August 15, 2026

## 中文版

### 曦码·曜输入法 隐私政策

感谢您使用曦码·曜输入法（以下简称"本应用"）。我们非常重视您的隐私。本政策说明了本应用如何处理您的信息。

#### 1. 数据收集

本应用**不收集**任何个人信息、使用数据或诊断数据：不上传、不存储、不传输您的任何输入内容、词库、设置或个人数据，也不包含任何遥测、崩溃上报或统计上报。

本应用**不主动联网**。只有在您**主动使用**「扩展商店」或「下载语音模型」等功能时，才会向对应的索引/下载地址（`https://index.ximei.me/…`、GitHub Releases 等）发起 HTTPS 请求以获取清单和安装包；这些请求**不携带**您的输入内容、词库或任何身份信息。

#### 2. 数据存储

本应用完全在您的设备本地运行。以下数据仅存储于您的设备，用于提供输入功能，不会离开设备：

- 用户数据目录 `%APPDATA%\Xime\`：RIME 方案与词库、`rime\xime.custom.yaml`（用户配置）及其快照 `.bak` / `.orig`、剪贴板历史数据库、日志（`logs\server.log` / `tsf.log` / `setup.log`）；
- 语音模型目录 `%APPDATA%\Xime\models\`（仅在您主动下载后存在）；
- 开始菜单快捷方式 `曦码·曜.lnk`（用于系统通知归属，首次发通知时创建）。

日志仅记录本机运行信息（错误、方案名、通知文案等），不会外发。

#### 3. 第三方服务

本应用不集成任何第三方分析、广告或追踪服务，不包含任何跟踪器。

#### 4. 权限说明

本应用作为文本服务（TSF）运行于 Windows 系统，仅获得提供文字输入所需的系统权限。应用内所有处理均在本机完成。

#### 5. 数据删除

- 运行 `full-uninstall.ps1` 会删除本机上的全部应用数据（含用户目录、开始菜单快捷方式与注册表项）。
- 通过 MSI 卸载默认**保留** `%APPDATA%\Xime\`（用户词库/配置属于您的数据）；如需彻底清除，卸载后手动删除该目录即可。

#### 6. 儿童隐私

本应用不面向儿童提供专门服务，也不收集任何用户的个人信息，包括儿童信息。

#### 7. 政策更新

本政策如有更新，将通过应用更新页面公布。

#### 8. 联系我们

如有任何关于隐私的问题，请联系：kingzcheung@gmail.com

---

## English Version

### Xime Yao Input Method — Privacy Policy

Thank you for using Xime Yao Input Method ("the App"). We value your privacy. This policy explains how the App handles your information.

#### 1. Data Collection

The App does **not** collect any personal information, usage data, or diagnostic data: it does not upload, store, or transmit your input content, dictionaries, settings, or personal data, and contains no telemetry, crash reporting, or analytics.

The App does **not** access the network on its own. Only when you **explicitly use** features such as the Extension Store or voice-model download does it send HTTPS requests to the corresponding index/download endpoints (e.g. `https://index.ximei.me/…`, GitHub Releases) to fetch manifests and packages. These requests carry **none** of your input content, dictionaries, or identity information.

#### 2. Data Storage

The App operates entirely on your device. The following data stays local and never leaves your device:

- `%APPDATA%\Xime\`: RIME schemas and dictionaries, `rime\xime.custom.yaml` (user configuration) with its `.bak` / `.orig` snapshots, clipboard history database, and logs (`logs\server.log` / `tsf.log` / `setup.log`);
- `%APPDATA%\Xime\models\` (only if you download voice models);
- Start-menu shortcut `曦码·曜.lnk` (used for toast attribution, created on first notification).

Logs contain only local runtime information (errors, schema names, notification text) and are never uploaded.

#### 3. Third-Party Services

The App integrates no third-party analytics, advertising, or tracking services and contains no trackers.

#### 4. Permissions

The App runs as a Text Services Framework (TSF) service on Windows and only uses the system permissions required to provide text input. All processing occurs on your device.

#### 5. Data Deletion

- Running `full-uninstall.ps1` removes all local application data (user data directory, Start-menu shortcut, registry entries).
- Uninstalling via MSI keeps `%APPDATA%\Xime\` by default (user dictionaries/configuration belong to you). Delete that directory manually for a complete wipe.

#### 6. Children's Privacy

The App does not provide services targeted at children and collects no personal information from any user, including children.

#### 7. Policy Updates

Any updates to this policy will be announced through the App's update page.

#### 8. Contact

For privacy questions, contact: kingzcheung@gmail.com
