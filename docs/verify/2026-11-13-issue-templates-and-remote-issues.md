# 2026-11-13 远程 issue 模板与远程 issue 本地登记

> 主题：① 给 `ximeiorg/XimeYao` 写 issue 模板（缺陷 / 功能建议 / 提问），供他人反馈；② 把远程已有 issue 逐条登记为本地未决并排期。

## 1. 平台（含版本）

| 项 | 值 |
| --- | --- |
| 系统 | Windows 11 专业版 10.0.26100 |
| 本仓 | XimeYao 0.5.0，`main`（`D:\project\XimeYao`） |
| 上游 | `ximeiorg/XimeYao` `main` = `3473fef`（PR #28 已合并）；本机账号 `tiengming` 对上游无 push 权限，走 PR |
| 本轮提交 | fork `tiengming/XimeYao`：`main` → `fd6747b`（本地文档登记）；`feat/issue-templates` → `d9c7570`（仅 `.github/ISSUE_TEMPLATE/` 四份文件） |
| 上游 PR | `ximeiorg/XimeYao#31`（`tiengming:feat/issue-templates` → `main`），2026-11-13 开、Docs + Build 双绿后**已合并为 `cb111f4`**；模板已在仓库 Issues → New issue 页生效（用户截图确认：功能建议 / 使用问题 / Blank issue / 两个 contact link 均出现） |
| 取数方式 | GitHub REST API（`api.github.com`），经本机代理 `http://127.0.0.1:10809`，脚本 `target/tmp/gh-fetch.mjs`（取数）与 `target/tmp/gh-api.mjs`（带方法/头/体，用于开 PR 与查 CI）（均为一次性产物） |
| 工具链 | node v24.22.1；Python（仅用于 YAML 解析自检） |

## 2. 远程 issue 快照（取数时间 2026-11-13）

远程共 5 个 issue、25 个 PR（PR 全部已关闭，其中 #28 已合并为 `3473fef`）。仓库既有 label：`bug`、`documentation`、`duplicate`、`enhancement`、`good first issue`、`help wanted`、`invalid`、`question`、`wontfix`。

| 编号 | 标题 | 提交人 | 日期 | 评论数 | 原文摘要 | 本地编号 |
| --- | --- | --- | --- | --- | --- | --- |
| #30 | 数字选字问题 | ztff | 2026-11-12 | 0 | 「正常逻辑为字母区的数字选字，数字小键盘区的混合输入，现在本输入法正好相反」 | U-33 |
| #29 | 万象拼音不能切双拼 | ztff | 2026-11-12 | 0 | 「1. 输入 /zrm 后无切换选项；2. 改 wanxiang.custom.yaml 重新部署后无效果」 | U-34 |
| #27 | 0.4 版本的万象无法上屏 | illidanplus | 2026-11-11 | 1 | 「已打出来，点击空格键码后加 1，按回车则是源码上屏，只能鼠标直接点击上屏」（附 2 张截图） | U-35 |
| #12 | 还不能正常用吗 | dadadafafafa | 2026-09-08 | 2 | 「计划什么时候出正式版啊？」（维护者回复「等待ing」「最近都没时间使用 Windows」） | U-36 |
| #11 | 好期待呀，快出一版我们用用吧！ | chenxin0397 | 2026-08-27 | 0 | 「如题」 | U-37 |

## 3. 模板设计（`.github/ISSUE_TEMPLATE/`）

| 文件 | 形态 | 必填字段 |
| --- | --- | --- |
| `bug_report.yml` | issue form | 客户端版本、系统（类型 + 版本号）、所用输入方案、问题模块、预期实现、实际问题、复现路径、复现频率 |
| `feature_request.yml` | issue form | 想解决的问题、期望的使用方式、涉及模块 |
| `question.yml` | issue form | 客户端版本、问题 |
| `config.yml` | 模板选择器 | `blank_issues_enabled: true`；两个 `contact_links` 指向 README 与本仓 `PROGRESS.md` |

设计要点：

- **缺陷模板收齐复现必需信息**：版本号、系统版本号、方案名、预期/实际、逐步复现路径——对应远程 #27「描述不清晰」的反馈（维护者已在该 issue 下要求补「预期 / 动作 / 实际」）。
- **日志字段指向 `%APPDATA%\Xime\logs\server.log`**（与本仓日志落盘位置一致），并提示先删个人信息。
- **模块下拉项**取自本仓实际模块划分（候选栏 / 设置程序 / 托盘 / 方案市场 / 词典管理 / 语音 / 剪贴板 / 安装升级），便于后续给 issue 打 label 与归类。
- **label 用仓库已有值**（`bug` / `enhancement` / `question`），不新造标签。

## 4. 验证

| 平台 | 命令或步骤 | 实际输出 | 覆盖范围 |
| --- | --- | --- | --- |
| Windows 11 | `python -c "import yaml; [yaml.safe_load(open(f'.github/ISSUE_TEMPLATE/{f}',encoding='utf-8')) for f in ['bug_report.yml','feature_request.yml','question.yml','config.yml']]"` | 四份文件均解析为 dict，无异常 | YAML 语法合法 |
| Windows 11 | `node scripts/doccheck.mjs` | `doccheck: 通过` | 文档结构与链接 |
| Windows 11 | `node scripts/refcheck.mjs` | `refcheck: 通过（N 个文件）` | 章节引用与 D-nnn 引用 |
| Windows 11 | `node target/tmp/gh-fetch.mjs "https://api.github.com/repos/ximeiorg/XimeYao/issues?state=all&per_page=100"` | `200 113568`，解析出 issues=5 / prs=25 | 远程 issue 快照 |
| Windows 11 | `node target/tmp/gh-api.mjs POST "https://api.github.com/repos/ximeiorg/XimeYao/pulls" target/tmp/pr-resp.json target/tmp/pr-body.json` | `201 16284`，PR 编号 31，head `tiengming:feat/issue-templates` @ `d9c7570`，base `main` @ `3473fef`，additions=254 / changed_files=4 | 上游 PR 已开 |
| Windows 11 | `node target/tmp/gh-api.mjs GET ".../commits/d9c7570.../check-runs"` | `200 6111`；`Docs` = completed/success，`Build` = completed/success | PR #31 的 CI 状态 |
| GitHub 网页 | 打开 `ximeiorg/XimeYao` → Issues → New issue | 出现「功能建议（Feature）」「使用问题（Question）」「Blank issue」与两个 contact link（用户截图） | 模板选择器渲染 |

## 5. 未验证项

- 模板在 GitHub 网页上的渲染已由用户截图确认（模板选择器可用）；**三个表单各自的必填项校验与下拉提交**尚未逐个走查。
- 远程 issue 的行为根因（U-33 ~ U-35）尚未在本机复现，表中「处置」列写的是**假设方向**，不是已证结论。
- #27 附带的两张截图（`github.com/user-attachments`）本轮未下载细看。

## 6. 关联

- 决策：`D-012`。
- 未决映射见 `PROGRESS.md` 未决表 U-33 ~ U-37。
