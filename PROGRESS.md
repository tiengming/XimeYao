# 进度（PROGRESS）

> 写法与上限见 `WORKFLOW.md §5`；本文件合计 ≤150 行。历史原文见 [completed-legacy-progress.md](docs/verify/completed-legacy-progress.md)。

## 接手指南

- 当前阶段：候选栏几何与面板四项（U-9/U-11/U-12/U-13/U-14/U-15/U-16）、首轮实机三修复、品牌栏移除、二级菜单定位修复、U-10 日志治理、U-28 上屏通知修复**全部实机确认通过**（2026-11-11 用户复测）；版本推进 0.5.0；上游 PR #28 已合并（`3473fef`）。2026-11-12 又修好「市场方案装不上/卸不掉」（U-32）与「商店失败提示 4 秒消失、通知与页面文案不一致」（U-31），改动在上游 libximecore，已拆成 PR #7 提交（待合并），实机复核待用户。2026-11-13 新增远程 issue 表单模板（缺陷/功能建议/提问），并把远程 5 个 issue 登记为 U-33~U-37。
- 下一个功能点：输入功能批次第一批（U-25 简繁转换 → U-18 整句开关 → U-23 反查，schema 配置批；见未决表时机）或 U-8/U-17 关于页（等用户文案）。设置页外观两项已在上游 libximecore 落地，待上游 PR。远程 issue 的取证（U-33~U-35）可插队。开工前按 `WORKFLOW.md §2` 写 S0。
- 阻断项：本机 cargo 直接编译 C 依赖会报 `D8050`，构建时需 `CC=CXX=C:\Program Files\LLVM\bin\clang-cl.exe` 且 `PATH` 前置 LLVM（见审计文档附录）；CI 不受影响。上游 libximecore 必须用 `cargo +nightly-2026-07-03`（stable 1.89 低于 `ordered-float` 1.90 / `rfd` 1.92 要求）。
- 30 秒读法：本文件 → `DECISIONS.md` 最近 3 条 → `WORKFLOW.md §1`。

## 当前状态

- 版本：0.5.0（`Cargo.toml` 的 `workspace.package.version`；2026-11-11 由 0.4.0 升，本批含候选栏几何 + 面板四项 + 三修复 + 品牌栏移除）
- 分支：`main` 与 `origin/main`（fork `tiengming/XimeYao`）同步于 `a2e5604`；上游 `ximeiorg/XimeYao` 的 `main` 已是 `3473fef`（PR #28 已合并，含候选栏几何等一批与版本 0.5.0）；`tiengming` 对 `ximeiorg/XimeYao` 无 push 权限（只能走 PR）
- 环境：Windows 11 专业版 10.0.26100 / rustc 1.98.0-nightly / node v24.22.1
- 基线：build / clippy / test 三项 2026-11-11 实测通过（192 passed、1 ignored、0 failed，含 tsf 6；0 warning 含上游）
- 依赖：libximecore 合入上游 `04d6ca9`（PR #6：`xime-speech` 摘要字段 + 词典回调 Result），8 个 core 依赖已钉 `rev`；本地 `[patch]` 仍指向 `D:\project\libximecore` 同提交克隆；新仓 libximep2p 克隆在 `D:\project\libximep2p`（HEAD `0865b35`，本项目钉 `b19ecde`）；`winxime-setup` 已开 `pair-page`（`D-008`）；librime 子模块仍 `d71168e`
- 焦点：候选栏几何与面板四项（U-9/U-11/U-12/U-13/U-14/U-15/U-16）、实机三修复、二级菜单定位、品牌栏移除、U-10 日志治理、U-28 上屏通知——全部编码+机检+**实机确认通过**（2026-11-11），已整理成 PR #28 待上游合并。日志治理后每键收敛为 1 条 trace、无大对象转储；上屏成功不再弹通知。设置页外观两项（候选栏横纵开关、数学计算开关）已在上游 libximecore 落地（`CalculatorConfig` + 外观页两个 switch），随本地 `[patch]` 编译，**尚未提交上游**。P1a、P2、P3a、P3b、P5 已本地提交，全部待实机验证；P3b 的 rev pin 卡在 `b222de2` 尚未推送到 `ximeiorg/libximecore`。

## 进行中

- 2026-11-12 方案包安装/卸载 + 通知-UI 统一（U-32 / U-31）：代码与机检已完成，**脚本探针 18 项断言全通过**（`target/tmp/schema-probe/`，一次性产物），改动主体在上游 `libximecore`（`xime-config` + `xime-setup`），本仓经 `[patch]` 引入；**上游 PR #7 已提交（待合并）、实机验证待用户**；顺带把 aes-gcm 弃用与无 feature 编译失败也各开一个 PR（#8 / #9）。见 [2026-11-12-schema-package-install-notify.md](docs/verify/2026-11-12-schema-package-install-notify.md)。
- 候选栏批次（几何+面板四项+三修复+二级菜单定位+品牌栏移除）与 U-10 日志治理、U-28 上屏通知已于 2026-11-11 全部实机确认通过。
- 构建警告已清零（上游 aes-gcm deprecated ×2 改 `Nonce` copy 填充、TSF 链接 LNK4104 ×4 以 `#![allow(linker_messages)]` 说明缘由关闭）；上游 `xime-plugin/src/cipher.rs` 改动已随上游 PR #8 提交。
- 2026-11-13 issue 模板与远程 issue 登记（`D-012`）：`.github/ISSUE_TEMPLATE/` 四份文件已写好（`bug_report.yml` / `feature_request.yml` / `question.yml` / `config.yml`），YAML 解析自检通过；远程 5 个 issue 已映射为 U-33~U-37 进未决表；决策与证据档案已写。待走 PR 提交上游。

## 已完成

历史记录（含 2026-11-09 审计整改批次、2026-11-10 P5/P1b/P2P 链路、2026-11-11 候选栏几何与面板四项、U-10/U-28）不再在本文件逐条罗列，唯一记录见 [completed-legacy-progress.md](docs/verify/completed-legacy-progress.md) 与 [docs/verify/](docs/verify/) 各档案。

## 未决

| 编号 | 问题 | 状态 | 处置 | 下一步 |
| --- | --- | --- | --- | --- |
| U-2 | CI 只有构建，test/clippy 未进流水线 | 已登记 | 评估构建时长后决定 | 先让 docs job 跑通 |
| U-3 | 历史验证证据只在旧 PROGRESS 与提交信息里 | 已登记 | 不追溯回填 | 新功能点按 `WORKFLOW.md §3` 落档 |
| U-4 | DECISIONS 旧条目无 D-nnn | 已裁定 | 不溯改（`D-001`） | 引用旧条目用日期加标题 |
| U-5b | 导出/导入/恢复词典接受客户端任意路径（审计 F2） | 已修（部分实机通过） | 服务端生成导出路径；导入/恢复只收白名单内真实文件（`D-007`）。2026-11-11 实机：备份通过；导出因引擎锁占用失败（未走到路径校验）；导入/恢复未走。导出失败文案「导出用户词典失败：ExportUserDict」不说原因（U-29） | 择机重走导出（空闲时）与导入/恢复；U-29 文案随下一轮词典功能修复 |
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
| U-29 | 词典操作失败文案不说明原因：导出报「导出用户词典失败：ExportUserDict」（Debug 格式直出），引擎锁占用时也不提示「正在输入，请稍后重试」 | 已修（自动验证通过） | `ipc_server.rs`：librime 失败经 `dict_op_reason` 转人话（发生了什么/为什么/下一步，附原始 Display 供排查）；引擎锁 `WouldBlock` 对词典命令回「输入法正在处理其它任务…请稍后重试」；备份失败此前连 error 字段都没带，已补。六条失败/成功路径探测脚本实测通过（`target/tmp/u29-probe/`） | 日常使用中各走一次导出/导入/恢复补 U-5b 证据；引擎锁占用分支留待出现时确认 |
| U-30 | 模型下载 403 失败后留空目录，且「已下载」判定只查目录存在（`scan_dir_ids`），空目录被判为已下载、之后安装永远失败无处自愈（模型侧未对齐方案侧的压缩包过滤） | 已登记 | 上游 `xime-setup`：`get_cached_model_ids` 改为过滤目录内确有文件的项（对齐 `downloaded_schema_ids` 的做法）；403 源头是 modelscope CDN auth_key 过期，属模型源问题 | 随上游 PR 一并修；模型源链接由模型发布方更新 |
| U-32 | 市场方案「装不上 / 卸不掉」：缓存目录名（历史三种写法）被当方案 id，界面 id ≠ 索引 id ≠ 注册表 key ⇒ 卸载报「方案包未安装」（文件其实已删）；`local` 被当冲突占用方 ⇒ `rime-wanxiang` 因 README.md 冲突装不上；冲突预检把所有其它包都算冲突 ⇒ 装第二个市场包必被要求卸载第一个 | 已修（上游 `xime-config` + `xime-setup`，PR #7 已提交待合并 + 实机） | id 归一化（`normalize_package_id` / `same_package_id` / `resolve_package_key`）+ 缓存目录解析层 `package_cache_dir`；`local` 与内置包不算冲突；冲突只列真正占用文件的第三方包（`D-011`）。脚本探针 18 项断言全通过（含遗留目录名归一、别名卸载、local 不阻塞） | 等上游合并 #7；实机按档案第 9 节六步复核 |
| U-31 | 商店下载失败提示 4 秒自动消失（`expire_install_messages`），用户错过即「无提示」（墨奇方案下载实测无提示） | 已修（上游 `xime-setup`，PR #7 已提交待合并 + 实机） | `expire_install_messages` 只回收模型/插件消息，方案结果常驻页面；通知与页面文案改由同一处 `MarketNotice` 构造（`D-011`）。脚本探针已验证页面文案与 toast 同文、方案结果不自动消失、失败文案含「下一步」 | 等上游合并 #7；实机确认失败提示不消失且与系统通知同文 |
| U-33 | 远程 #30「数字选字问题」（ztff，2026-11-12）：数字键应只在小键盘区做混合输入，字母区数字应选候选字词；本输入法行为相反 | 已登记（远程 #30） | 假设方向（未复现）：内置计算器「数字开头影子缓冲」默认开启（`calculator::auto_detect_enabled` 默认 true），组词串为空时按数字被吞进算式缓冲，候选选字键被抢；字母区与小键盘区未做区分 | 先在 issue 下追问「哪个应用、按哪个数字键、期望选中第几个」；本机复现后按 `WORKFLOW.md §4` 三档处置（疑似需按「是否组词中」区分数字键归属，动手前写 `D-nnn`） |
| U-34 | 远程 #29「万象拼音不能切双拼」（ztff，2026-11-12）：输入 `/zrm` 无切换选项；改 `wanxiang.custom.yaml` 重新部署后无效果 | 已登记（远程 #29） | 假设方向（未复现）：方案 `switches` 的多选一 option 未在托盘菜单暴露（`schema_switches.rs` 只解析有 `states` 的项）；`/zrm` 选方案属 rime 侧能力，与本仓托管逻辑是否相关待查 | 追问用户所用万象版本与 `/zrm` 期望行为；本机用万象方案复现「托盘菜单是否列出双拼选项」，确认是引擎不支持还是菜单没暴露 |
| U-35 | 远程 #27「0.4 版本的万象无法上屏」（illidanplus，2026-11-11）：已打出候选，按空格变成码后加 1，按回车把源码上屏，只能鼠标点击上屏 | 已登记（远程 #27） | 假设方向（未复现）：空格与回车在候选态的提交路径与鼠标点选不一致（点选走 `ui/view.rs` 的选中→提交，键盘路径走 librime 提交 + 计算器影子缓冲回放） | 已有维护者回复要求补「预期 / 动作 / 实际」；等用户补充后本机用万象方案复现（0.5.0 与 0.4.0 行为可能已不同，需先确认版本） |
| U-36 | 远程 #12「还不能正常用吗」（dadadafafafa，2026-09-08）：询问何时出正式版 | 已登记（远程 #12） | 非缺陷，是发布节奏提问；已有维护者回复「等待ing」 | 版本推进到 0.6.0 或发版公告时回帖说明计划；不单独立功能点 |
| U-37 | 远程 #11「好期待呀，快出一版我们用用吧！」（chenxin0397，2026-08-27） | 已登记（远程 #11） | 期待类反馈，无具体问题 | 与 U-36 合并到同一次发布答复 |

输入功能批次（U-18~U-27 十条用户建议）的前置（候选栏几何实机确认、U-10 日志治理）均已完成，按上表时机分批：schema 配置批（U-18/U-23/U-25）→ 码表与词库批（U-19/U-22）→ 候选栏交互批（U-26/U-27）→ 快捷键批（U-20 随上游 PR）。开工前每条先查证 librime 原生能力，避免自研重复造轮子。

候选栏几何与面板四项（U-9/U-11/U-12/U-13/U-14/U-15/U-16）、首轮实机三修复、二级菜单定位、品牌栏移除、U-10 日志治理、U-28 上屏通知：2026-11-11 用户实机复测**全部通过**（见 [2026-11-11-candidate-geometry.md](docs/verify/2026-11-11-candidate-geometry.md)），不再逐条占表。

## 验证记录

| 日期 | 主题 | 文件 | 结论 |
| --- | --- | --- | --- |
| 2026-11-11 | 候选栏窗口几何与面板四项（U-9/U-11/U-12/U-13/U-14/U-15/U-16）+ 实机反馈修复、二级菜单定位与品牌栏移除 | [2026-11-11-candidate-geometry.md](docs/verify/2026-11-11-candidate-geometry.md) | 机检通过（192 passed、0 warning）；U-13 与首轮三修复已实机确认，其余待实机 |
| 2026-11-11 | U-10 日志治理 + U-28 上屏通知修复 | [2026-11-11-candidate-geometry.md](docs/verify/2026-11-11-candidate-geometry.md) 第 11 节 | 机检通过（192 passed、0 warning）；待实机验证日志增量与通知行为 |
| 2026-11-11 | 候选栏批次全项（几何+面板四项+三修复+二级菜单+品牌栏移除）+ U-10 + U-28 | 同上档案实机一节 | **用户实机复测全部通过** |
| 2026-11-11 | U-5a IPC 管道 ACL（P1a）实机验证补记 | [2026-11-09-ipc-acl.md](docs/verify/2026-11-09-ipc-acl.md) | **通过**（server.log 三次启动均含「用户 SID + SYSTEM」ACL 行；TSF 打字与设置页词典列表均走新管道正常） |
| 2026-11-11 | U-29 词典操作失败文案修复（`dict_op_reason` 人话 + 引擎锁占用提示 + 备份失败补 error） | [2026-11-11-dict-error-messages.md](docs/verify/2026-11-11-dict-error-messages.md) | 机检通过（192 passed、0 warning）；**自动验证通过**（探测脚本六条命令：两处人话文案、三处 dict_paths 拒绝回归、成功路径正常） |
| 2026-11-12 | 方案包安装/卸载失败与通知-UI 不统一（U-32 + U-31） | [2026-11-12-schema-package-install-notify.md](docs/verify/2026-11-12-schema-package-install-notify.md) | 机检通过（本仓 build/clippy 0 warning/test 全绿；上游 xime-setup-lib 62 passed、xime-config 44 passed + 2 预存失败）；**脚本探针 18 项断言全通过**；**上游 PR #7/#8/#9 已提交**；实机待用户按档案第 9 节六步复核 |
| 2026-11-13 | 远程 issue 表单模板 + 远程 5 个 issue 本地登记（U-33~U-37） | [2026-11-13-issue-templates-and-remote-issues.md](docs/verify/2026-11-13-issue-templates-and-remote-issues.md) | 四份 YAML 解析通过；doccheck/refcheck 双绿；远程快照 issues=5 / prs=25；**模板网页渲染效果与 U-33~U-35 根因未验证** |

## 下一步

- 输入功能批次第一批：U-25 简繁转换（librime opencc 大概率原生）→ U-18 五笔整句开关 → U-23 反查（schema 配置批）；或 U-8/U-17 关于页（等用户文案）。
- U-18~U-27 十条输入功能建议已登记（见未决表批次时机），前置（候选栏实机、U-10 日志治理）已完成，开工前先查证 librime 原生能力。
- 2026-11-11 实机新遗留：U-29 词典失败文案（本轮已修，待实机）、U-30 模型下载 403 后空目录误判已下载（属上游 `xime-setup`，随上游 PR）。U-31（商店失败提示 4 秒消失）与 U-32（方案装不上/卸不掉）已在上游 `libximecore` 修好并提交 PR #7（待合并），实机复核待用户（`D-011`）。
- U-10 剩余项：绘制重建与 506ms 离群值随后单独一轮；`DECISIONS.md` 2026-09-30 条「每键重读三份 yaml」的说法已过时（日志治理本身已实机通过并从未决表移除）。
- 已开 PR：`ximeiorg/XimeYao#28`（`tiengming:feat/candidate-geometry-and-dict-msgs` → `main`，候选栏几何与面板四项 + U-10 日志治理 + U-28 上屏通知 + U-29 词典文案 + 版本 0.5.0；**按约定不含 docs/**，证据档案只留本地）**已于 2026-11-13 合并为 `3473fef`**；core 侧 `ximeiorg/libximecore#6` 已合并为 `04d6ca9`，等 CI 转绿后把 #26 转 ready；本轮又提交 core 侧 `ximeiorg/libximecore#7`（方案包 id 归一化 + 通知/UI 同源）、`#8`（aes-gcm 弃用）、`#9`（clipboard-page 门控修复无 feature 编译失败），三者均待上游合并。上游同步同理走 PR（本机账号对 `ximeiorg` 无 push 权限）。
- 本轮待办：把 `.github/ISSUE_TEMPLATE/` 四份模板 + `D-012` + `PROGRESS.md` 未决表 U-33~U-37 走 PR 提交上游（`ximeiorg/XimeYao`），合并后到仓库「Issues → New issue」页人工确认三个表单的渲染与下拉可提交；再按 U-33→U-34→U-35 顺序在远程 issue 下追问缺项并本机取证。
- P3b 收尾（暂缓：按要求不推远程）：把 libximecore 的 `b222de2` 推到 `ximeiorg/libximecore` 后再钉 `rev`，并用真实模型包走一次下载。
- 本机构建先设 `CC=CXX=clang-cl` 与 LLVM `PATH`，再跑 `cargo build --quiet`；P2P 实机步骤见 [2026-11-10-clipboard-p2p-host.md](docs/verify/2026-11-10-clipboard-p2p-host.md) 末节。
