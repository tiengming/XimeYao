# 进度（PROGRESS）

> 写法与上限见 `WORKFLOW.md §5`；本文件合计 ≤150 行。历史原文见 [completed-legacy-progress.md](docs/verify/completed-legacy-progress.md)。

## 接手指南

- 当前阶段：审计整改已全部落地（P1a/P1b/P2/P3a/P3b 本地提交未 push；剪贴板 P2P 直连已实机通过）；候选栏几何与面板四项（U-9/U-11/U-12/U-13/U-14/U-15/U-16）、首轮实机三修复、品牌栏移除、二级菜单定位修复、U-10 日志治理、U-28 上屏通知修复**全部实机确认通过**（2026-11-11 用户复测）；版本推进 0.5.0；仓库地址与关于页入口已统一到 `ximeiorg/XimeYao`。
- 下一个功能点：输入功能批次第一批（U-25 简繁转换 → U-18 整句开关 → U-23 反查，schema 配置批；见未决表时机）或 U-8/U-17 关于页（等用户文案）。设置页外观两项已在上游 libximecore 落地，待上游 PR。开工前按 `WORKFLOW.md §2` 写 S0。
- 阻断项：本机 cargo 直接编译 C 依赖会报 `D8050`，构建时需 `CC=CXX=C:\Program Files\LLVM\bin\clang-cl.exe` 且 `PATH` 前置 LLVM（见审计文档附录）；CI 不受影响。
- 30 秒读法：本文件 → `DECISIONS.md` 最近 3 条 → `WORKFLOW.md §1`。

## 当前状态

- 版本：0.5.0（`Cargo.toml` 的 `workspace.package.version`；2026-11-11 由 0.4.0 升，本批含候选栏几何 + 面板四项 + 三修复 + 品牌栏移除）
- 分支：`main` 与 `origin/main`（fork `tiengming/XimeYao`）同步于 `f0fa784`；本轮 17 个提交（`1bf9913` 起）已推送。上游 `ximeiorg/XimeYao`（旧名 `ximeiorg/winxime`，同一 HEAD `09ab350`）需走 PR：`tiengming` 对 `ximeiorg/winxime` 无 push 权限（`git push --dry-run upstream main` 返回 403）
- 环境：Windows 11 专业版 10.0.26100 / rustc 1.98.0-nightly / node v24.22.1
- 基线：build / clippy / test 三项 2026-11-11 实测通过（192 passed、1 ignored、0 failed，含 tsf 6；0 warning 含上游）
- 依赖：libximecore 合入上游 `04d6ca9`（PR #6：`xime-speech` 摘要字段 + 词典回调 Result），8 个 core 依赖已钉 `rev`；本地 `[patch]` 仍指向 `D:\project\libximecore` 同提交克隆；新仓 libximep2p 克隆在 `D:\project\libximep2p`（HEAD `0865b35`，本项目钉 `b19ecde`）；`winxime-setup` 已开 `pair-page`（`D-008`）；librime 子模块仍 `d71168e`
- 焦点：候选栏几何与面板四项（U-9/U-11/U-12/U-13/U-14/U-15/U-16）、实机三修复、二级菜单定位、品牌栏移除、U-10 日志治理、U-28 上屏通知——全部编码+机检+**实机确认通过**（2026-11-11）。日志治理后每键收敛为 1 条 trace、无大对象转储；上屏成功不再弹通知。设置页外观两项（候选栏横纵开关、数学计算开关）已在上游 libximecore 落地（`CalculatorConfig` + 外观页两个 switch），随本地 `[patch]` 编译，待上游 PR。P1a、P2、P3a、P3b、P5 已本地提交，全部待实机验证；P3b 的 rev pin 卡在 `b222de2` 尚未推送到 `ximeiorg/libximecore`。

## 进行中

- 无。候选栏批次（几何+面板四项+三修复+二级菜单定位+品牌栏移除）与 U-10 日志治理、U-28 上屏通知已于 2026-11-11 全部实机确认通过。
- 构建警告已清零（上游 aes-gcm deprecated ×2 改 `Nonce` copy 填充、TSF 链接 LNK4104 ×4 以 `#![allow(linker_messages)]` 说明缘由关闭）；上游 `xime-plugin/src/cipher.rs` 改动随下次上游 PR 一并提交。

## 已完成

历史记录（含 2026-11-09 审计整改批次、2026-11-10 P5/P1b/P2P 链路、2026-11-11 候选栏几何与面板四项、U-10/U-28）不再在本文件逐条罗列，唯一记录见 [completed-legacy-progress.md](docs/verify/completed-legacy-progress.md) 与 [docs/verify/](docs/verify/) 各档案。

## 未决

| 编号 | 问题 | 状态 | 处置 | 下一步 |
| --- | --- | --- | --- | --- |
| U-2 | CI 只有构建，test/clippy 未进流水线 | 已登记 | 评估构建时长后决定 | 先让 docs job 跑通 |
| U-3 | 历史验证证据只在旧 PROGRESS 与提交信息里 | 已登记 | 不追溯回填 | 新功能点按 `WORKFLOW.md §3` 落档 |
| U-4 | DECISIONS 旧条目无 D-nnn | 已裁定 | 不溯改（`D-001`） | 引用旧条目用日期加标题 |
| U-5a | IPC 管道 ACL 为 Everyone（审计 F1） | 已修（待实机；`2b57fac`） | 当前用户 SID + SYSTEM | 用户跑 `rebuild.ps1` 后按档案 4 步验证 |
| U-5b | 导出/导入/恢复词典接受客户端任意路径（审计 F2） | 已修（待实机） | 服务端生成导出路径；导入/恢复只收白名单内真实文件（`D-007`） | 设置页导出/导入/恢复各走一次 |
| U-6 | 插件备份恢复解压无体积上限、无回滚、目录错误被吞（审计 F3） | 已修（未接线；`8284a4b`） | 三重限额 + 暂存目录 + 失败回滚（`D-003`） | 接线后拿真实备份包做端到端恢复 |
| U-7a | 语音模型包无体积上限、失败留半成品（审计 F4） | 已修（`eac49f0`） | 下载 512 MiB、解压单文件 512 MiB / 合计 1 GiB、失败清理 | 发布前用真实模型包下载一次 |
| U-7b | 语音模型包无 sha256 校验（审计 F4） | 已修（`eac49f0` + core `b222de2`；rev pin 待推送） | 注册表加 `ModelChecksum`，下载流式校验、暂存后替换（`D-005`） | 合入 `ximeiorg/libximecore` 后钉 `rev`，再用真实包下载一次 |
| U-8 | 产品关于页信息需修改（改哪几行尚未定） | 已登记 | 只改文案/字段动本仓 `set_app_metadata`（`winxime-setup`、`winxime-server` 各一处）；改版式必须动上游 `xime-setup` 的 `about.rs`，该仓只读，只能走上游 PR。版本号已随 `CARGO_PKG_VERSION`（0.5.0）、主页已指向 `ximeiorg/XimeYao`，面板内版权栏已按用户要求移除 | **时机：用户给出具体要改的文字/字段那一轮**；不阻塞候选栏工作 |
| U-16b | 设置页修改面板快捷键（`panel.hotkey`） | 已登记 | 属上游 `xime-config`/外观页 PR；本仓热键链路已按 Ctrl+` 落地 | 与 U-8/U-17 上游 PR 一并处理 |
| U-17 | 关于页信息与项目匹配（具体文案待定） | 已登记 | 元数据改本仓 `set_app_metadata`（`winxime-setup`、`winxime-server` 各一处）；版式动上游 `about.rs` 走 PR | **时机：用户给出要改的具体文字/字段那一轮** |
| U-18 | 五笔整句输入开关（用户 2026-11-11 建议 1） | 已登记 | 引擎层：先确认 librime 五笔 schema 是否原生支持整句（未验证），再定设置页开关形态 | **时机：输入功能批次 ①——候选栏几何实机确认后** |
| U-19 | 支持码表导入（建议 2） | 已登记 | 需先定码表格式与校验规则；导入路径安全复用 `D-007` 白名单思路 | **时机：输入功能批次 ②——U-18 之后单独一轮** |
| U-20 | 造词快捷键（如 Ctrl+=，建议 3） | 已登记 | 快捷键复用 U-16 TSF 保留键链路；rime 造词能力未验证，先查证 | **时机：与 U-16b 设置页改键同批（上游 PR）** |
| U-21 | 多功能按键：分号/反引号触发计算、日期输入、重复上屏（建议 4） | 已登记 | 重复上屏已有 F24 自注入通道（paste.rs）、计算已有 calculator 可复用；日期需新增；先出交互方案再实现 | **时机：U-10 打字链路优化之后** |
| U-22 | emoji 输入编码：打「开心」出表情候选（建议 5） | 已登记 | 需词库/schema 层支持，数据源（emoji 表）待查证 | **时机：词库批次——U-19 码表导入落地后** |
| U-23 | 拼音与五笔反查（建议 6） | 已登记 | schema 反查可能 librime 原生支持（未验证），主在 schema 配置与部署 | **时机：与 U-18 同批（schema 配置批）** |
| U-24 | 字词调频，可设置 emoji/符号不参与调频（建议 7） | 已登记 | 调频走 librime 用户词典；排除规则需自研配置项 | **时机：输入功能批次** |
| U-25 | 简繁转换（建议 8） | 已登记 | librime opencc 大概率原生支持（未验证），加设置开关，十项中成本最低 | **时机：输入功能批次第一批** |
| U-26 | 候选右键置顶字词、删除词条（建议 9） | 已登记 | 候选栏右键菜单 + IPC 新命令 + librime 用户词典操作，横跨 UI 与引擎 | **时机：候选栏实机确认后的下一个候选栏功能点** |
| U-27 | 候选注释：拼音/英文翻译/五笔拆字编码（建议 10） | 已登记 | librime comment 字段是否随候选返回未验证；英文翻译需额外数据源 | **时机：与 U-26 同批** |

输入功能批次（U-18~U-27 十条用户建议）的前置（候选栏几何实机确认、U-10 日志治理）均已完成，按上表时机分批：schema 配置批（U-18/U-23/U-25）→ 码表与词库批（U-19/U-22）→ 候选栏交互批（U-26/U-27）→ 快捷键批（U-20 随上游 PR）。开工前每条先查证 librime 原生能力，避免自研重复造轮子。

候选栏几何与面板四项（U-9/U-11/U-12/U-13/U-14/U-15/U-16）、首轮实机三修复、二级菜单定位、品牌栏移除、U-10 日志治理、U-28 上屏通知：2026-11-11 用户实机复测**全部通过**（见 [2026-11-11-candidate-geometry.md](docs/verify/2026-11-11-candidate-geometry.md)），不再逐条占表。

## 验证记录

| 日期 | 主题 | 文件 | 结论 |
| --- | --- | --- | --- |
| 2026-11-11 | 候选栏窗口几何与面板四项（U-9/U-11/U-12/U-13/U-14/U-15/U-16）+ 实机反馈修复、二级菜单定位与品牌栏移除 | [2026-11-11-candidate-geometry.md](docs/verify/2026-11-11-candidate-geometry.md) | 机检通过（192 passed、0 warning）；U-13 与首轮三修复已实机确认，其余待实机 |
| 2026-11-11 | U-10 日志治理 + U-28 上屏通知修复 | [2026-11-11-candidate-geometry.md](docs/verify/2026-11-11-candidate-geometry.md) 第 11 节 | 机检通过（192 passed、0 warning）；待实机验证日志增量与通知行为 |
| 2026-11-11 | 候选栏批次全项（几何+面板四项+三修复+二级菜单+品牌栏移除）+ U-10 + U-28 | 同上档案实机一节 | **用户实机复测全部通过** |

## 下一步

- 输入功能批次第一批：U-25 简繁转换（librime opencc 大概率原生）→ U-18 五笔整句开关 → U-23 反查（schema 配置批）；或 U-8/U-17 关于页（等用户文案）。
- U-18~U-27 十条输入功能建议已登记（见未决表批次时机），前置（候选栏实机、U-10 日志治理）已完成，开工前先查证 librime 原生能力。
- U-10 剩余项：绘制重建与 506ms 离群值随后单独一轮；`DECISIONS.md` 2026-09-30 条「每键重读三份 yaml」的说法已过时（日志治理本身已实机通过并从未决表移除）。
- 已开 PR：`ximeiorg/XimeYao#26`（草稿，`tiengming:main` → `main`）；core 侧 `ximeiorg/libximecore#6` 已合并为 `04d6ca9`，等 CI 转绿后把 #26 转 ready。上游同步同理走 PR（本机账号对 `ximeiorg` 无 push 权限）。
- P3b 收尾（暂缓：按要求不推远程）：把 libximecore 的 `b222de2` 推到 `ximeiorg/libximecore` 后再钉 `rev`，并用真实模型包走一次下载。
- 本机构建先设 `CC=CXX=clang-cl` 与 LLVM `PATH`，再跑 `cargo build --quiet`；P2P 实机步骤见 [2026-11-10-clipboard-p2p-host.md](docs/verify/2026-11-10-clipboard-p2p-host.md) 末节。
