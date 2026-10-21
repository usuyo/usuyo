# 歇存 Xiecun

开源的注意力回收工具。把散落在各平台收藏夹里的内容，变成你自己的、可搜索、可交给 Agent 处理的文本资料库。

一句话：先收好，心思回到当下。

## 文档地图

- `docs/PRD.md` — 产品定义与边界（为什么做、做什么、不做什么）
- `docs/SPEC.md` — 技术规格：内容包契约、目录结构、模块划分、接口
- `docs/features/` — 每个功能一份 feature doc（见下方开发流程）
- `docs/decisions/` — 架构决策记录（ADR），一个决定一个文件

## 技术栈（已定，不要擅自更换）

- 语言：Go 1.23+（内核、CLI、MCP 服务、守护进程）
- 桌面壳：MyGo（https://mygo.egoist.dev），系统 webview 跑 web 前端，前端 TypeScript
- 前端：TypeScript + Vite；包管理 pnpm；画布库待定（见 docs/features）
- 存储：文件为真相源（Markdown + 媒体 + JSON），索引用可重建的本地索引（bleve，纯 Go）
- 重计算：ffmpeg、OCR、ASR 作为独立 sidecar 进程，永不 import 进内核
- 适配器：独立进程（CLI/HTTP/MCP），按协议对接，永不 import 进内核

## 不可动摇的原则

1. **内核平台中立。** 内核仓库里不出现任何针对具体平台（抖音、小红书、B站、公众号、X…）的抓取逻辑。平台适配器放社区仓库。
2. **raw 永远保留，原文层机器永不改写。** 规范化、识别、AI 产物全部放 `derived/`，可以重算；`raw/` 只增不改。
3. **文件是真相，索引可丢。** 删掉 `.xiecun/index/` 必须能完整重建。
4. **不制造压力。** 没有未读数、没有待办队列、没有打卡。整理是可选的，不存在"未整理"状态。
5. **入库内容是不可信输入。** 通过 MCP/API 返回给 Agent 的内容带 `untrusted: true` 和来源；写操作只接受结构化参数；MCP 不提供删除和外发。
6. **Agent 写入先提议后生效。** 任何来自 Agent/模型的修改先生成变更集，用户确认后应用，并写入操作日志，可整批撤销。

## 开发流程：feature-dev

每个功能走同一条流程，不跳步：

1. **写 feature doc**：在 `docs/features/NNN-<slug>.md` 新建，用模板 `docs/features/TEMPLATE.md`。写清：目标、用户故事、范围内/外、数据契约变更、接口变更、验收标准、测试方案。
2. **评审**：把 feature doc 的要点贴给用户确认（或用户自己改）。没确认不写代码。
3. **实现**：按 feature doc 实现，附带测试。契约变更要同步改 `docs/SPEC.md`。
4. **验证**：跑 `make check`（lint + test），手动跑验收标准里的命令。
5. **收尾**：feature doc 顶部状态改为 `done`，有架构级决定就补一条 ADR。

一个 feature 一个分支，一个 PR。PR 描述引用 feature doc。

## 代码约定

- Go：标准布局 `cmd/`、`internal/`、`pkg/`；错误用 `fmt.Errorf("...: %w", err)` 包装；对外接口先写 interface。
- 测试：`go test ./...`；适配器契约用 `testdata/` 里的固定样例校验。
- 前端：`apps/desktop/web`，pnpm workspace；组件按功能分目录。
- 提交信息：`<scope>: <summary>`，例如 `core: add content package writer`。
- 文案：中文为主；产品内用"收下 / 存着 / 回头看"，不用"收藏 / 待办 / 未读"。
- 不写"破解 / 去水印 / 绕过"这类字眼；统一说"个人备份 / 离线阅读 / 注意力整理"。

## 常用命令

```
make init     # 安装依赖（go mod、pnpm install、sidecar 检查）
make dev      # 启动内核守护进程 + 桌面壳开发模式
make check    # lint + test
make build    # 打包
xiecun --help # CLI
```

## 当前里程碑

M0：无 GUI 的最小闭环。粘贴链接 → 内容包 → `xiecun search` 能搜到 → Claude Code 通过 MCP 能读。详见 `docs/SPEC.md` 的 M0 范围。
