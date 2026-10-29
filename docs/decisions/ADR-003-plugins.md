<a id="adr-003-插件架构与分发"></a>
# ADR-003 插件架构与分发

日期：2026-10-28 · 状态：proposed

本条定插件的种类、进程与协议、清单、权限与隔离、仓库拆分、索引的运作、导出与读回。依据是调研 [05 §6](../research/05-note-apps-and-plugins.md#6-对歇存插件化架构的启示)，需要拍板的 P1–P9 都按 05 的建议定，见 [05 §6.8](../research/05-note-apps-and-plugins.md#68-需要拍板的事)。清单字段与契约的改动留到实现时再做。

<a id="背景"></a>
## 背景

歇存要接三种外部能力：从平台取回内容，把音视频和图片变成文字，把资料库导出到笔记软件。[PRD §4](../PRD.md#4-原则) 原则 1 要求内核平台中立，原则 4 要求一切可插拔；[ADR-001](ADR-001-stack.md) 要求重计算与适配器都是独立进程，不进内核。

调研 05 的发现：

- 平台抓取正在从笔记插件里搬出去。24 个 Obsidian 社区插件里有 7 个把抓取和转写放进本机的独立后端或命令行工具，做的就是歇存「获取方式」的事（[05 §1.4](../research/05-note-apps-and-plugins.md#14-共性与对歇存的含义)）。
- 别人的插件依赖宿主 API，拿不出来用；能复用的是背后的协议和命令行工具，转写基本都是 OpenAI 转写接口的形状（[05 §2.5](../research/05-note-apps-and-plugins.md#25-运行方式统计与可复用协议)）。
- 纯 GitHub Actions 的仓库索引就能收 PR、校验、给外部提交评论和打标签；GitHub App 只在四件事上不可替代（[05 §3.9](../research/05-note-apps-and-plugins.md#39-github-app-与-actions-的能力差别)）。
- 主流笔记宿主都没有运行时强制隔离；ffmpeg、ASR、OCR、本地视觉模型只能跑原生进程（[05 §4.1](../research/05-note-apps-and-plugins.md#41-宿主插件的安全模型)、[05 §4.4](../research/05-note-apps-and-plugins.md#44-原生进程还是-wasm)）。
- 没有工具从笔记软件读回改动（[05 §5.9](../research/05-note-apps-and-plugins.md#59-读回双向的先例与歇存的做法)）。

2026-10-28 用户认可了方向：先做 GitHub 仓库索引，托管 GitHub App 以后再加，官方索引不收获取方式。

<a id="决定"></a>
## 决定

<a id="1-插件分三类"></a>
### 1. 插件分三类

| 类 | 做什么 | 输入 | 输出 | 官方索引 |
|---|---|---|---|---|
| 获取 fetch | 从平台或外部文件取回内容 | URL、订阅源、导出包 | `raw/` 快照、文本、媒体 | 不收，只进社区 store |
| 转换 transform | 把条目里已有的文件变成新的一层 | 条目文件的路径，只读 | `derived/` 下的约定文件、`images/keyframes/` | 收 |
| 导出 export | 把条目和主题页写到笔记软件，可选读回用户改动 | 条目与主题页的只读视图（JSON） | 交给宿主写的文件，或经宿主代发的云 API 调用；读回时返回结构化改动 | 收；云端目标标「离开本机」 |

- **导入是获取的一种**，来源是文件而不是 URL，沿用获取方式的协议，`origin.entry` 记 `import`。
- **模型 provider 不是插件**，是内核里的一份配置：名称、OpenAI 兼容端点、模型、密钥。密钥存系统钥匙串。转换插件只引用 provider 名称，请求由内核代签代发，插件拿不到密钥明文。
- **留在内核的**：编排（按转换插件声明的输入输出，把抽音轨、转写、摘要串起来）、权限落实、provider 代理、导出记录与读回合并、写资料库。

<a id="2-进程与协议"></a>
### 2. 进程与协议

所有插件都是独立进程。内核只认一种协议：独立进程加 JSON。

| 类 | 控制协议 | 数据 |
|---|---|---|
| 获取 | 沿用 [SPEC §8](../SPEC.md#8-适配器协议-v0) 的子命令 `canonicalize`、`fetch`、`list`、`health`，stdout 输出 JSON；错误信封与退出码按 [02 §15.3](../research/02-adapter-ecosystems.md#153-调用错误与产物) | `--out` 指向宿主分配的临时目录 |
| 转换 | stdio 上换行分隔的 JSON-RPC 2.0，分帧和 MCP stdio 相同。方法 `describe`、`health`、`run`；`progress` 通知；支持取消；stdin 关闭后退出。可以常驻，模型加载一次处理多条 | 只传文件路径，大块数据不走管道 |
| 导出 | 同转换；另有 `readback` 调用：宿主给出目标侧当前内容与上次导出记录，插件只返回用户区的结构化改动 | 同转换 |

- **只有宿主写资料库。** 插件只写宿主分配的临时目录，宿主校验后再移入 `derived/` 或导出目录。
- **重计算只做原生进程。** ffmpeg、ASR、OCR、本地视觉模型的 Wasm 版太慢、输入有 2 GB 上限、用不了 GPU（[05 §4.4](../research/05-note-apps-and-plugins.md#44-原生进程还是-wasm)）。
- **Wasm 以后再做。** 出现纯变换插件（文本规范化、字幕互转、导出格式映射）时，再做官方的 `wasm-runner` 进程，内嵌 wazero。wazero 不进内核；对内核来说 runner 只是又一个进程插件；macOS 上需要 JIT 的 entitlement 只给 runner。
- **转换插件可以做成薄客户端**，后端是用户自己跑的服务，兼容 OpenAI `/v1/audio/transcriptions`、whisper-asr-webservice `/asr`、OpenAI chat、Umi-OCR `/api/ocr`。

<a id="3-清单"></a>
### 3. 清单

- 三类插件共用一份清单：在现有 `adapter.yaml` 上加 `kind: fetch | transform | export`，共用一份 schema。
- 字段方向按 [05 §6.2](../research/05-note-apps-and-plugins.md#62-进程模型与协议)：
  - 共有 `id`、`name`、`version`、`protocol`、`kind`、`license`、`homepage`、`maintainers`、`install`、`requires.runtime`、`permissions`；
  - 转换另有 `consumes`、`produces`、`provider`；
  - 导出另有 `target`、`writes`、`readback`。
- 这次不改 `pkg/contract` 和 SPEC。字段名在实现插件运行器时定稿，届时同步改 [SPEC §8](../SPEC.md#8-适配器协议-v0)、JSON Schema 和 `testdata/`。

<a id="4-权限与隔离"></a>
### 4. 权限与隔离

清单里的 `permissions` 声明 `fs.read`、`fs.write`、`network`（`none` 或 `allowlist`）、`gpu`、`exec`、`env`、`limits`（内存、超时），原生插件与将来的 Wasm 插件用同一个结构（[05 §4.5](../research/05-note-apps-and-plugins.md#45-权限声明与各平台落实程度)）。路径用宿主展开的占位符，插件拿不到资料库根目录。

各平台的落实：

- **macOS**：Seatbelt profile 强制文件、网络与再起子进程；内存靠宿主轮询，超限即杀。
- **Linux**：bubblewrap 强制，不可用时降级 Landlock，都不行就仅告知；内存用 cgroup v2 或 prlimit。
- **Windows**：先只强制 Job Object（内存、进程数、超时）与环境变量，文件和网络先仅告知。
- **所有平台**：环境变量显式传入；超时到期杀整个进程组；`allowlist` 网络只能经宿主代理出站，密钥由宿主在代发时注入。

界面按平台、按声明项显示「强制 / 部分 / 仅告知」三档，不一律写「已沙箱」。

用户同意：

- 安装时展示权限，用户按资料库同意。
- 新版本扩大权限时，这个版本先停用，等用户重新同意；不静默更新扩大了权限的版本。做法照 GitHub App 改权限需安装方批准，防的是经自动更新下发恶意代码。
- 插件的启用状态、配置与密钥放本机配置目录，不放进会同步的资料库，避免同步过来的插件配置直接执行（[05 §4.1](../research/05-note-apps-and-plugins.md#41-宿主插件的安全模型)）。

<a id="5-仓库拆分"></a>
### 5. 仓库拆分

| 仓库 | 内容 | 许可证 |
|---|---|---|
| `xiecun-app/xiecun` | 内核、CLI、MCP、桌面壳；插件运行器、权限落实、provider 代理、导出记录与读回合并；不含任何平台抓取逻辑 | AGPL-3.0 |
| `xiecun-app/plugin-kit` | 清单与消息的 JSON Schema、`derived/` 文件格式、契约测试套件与 `testdata/`、Go / TypeScript / Python SDK、插件模板、本地校验命令 | MIT |
| `xiecun-app/plugins` | 官方索引，只收转换与导出插件：`index/<id>.yaml`、生成的 `index.json`、`removed.yaml`、`deprecated.yaml`、校验与定时任务的 workflow | MIT |
| `xiecun-app/<kind>-<name>` | 官方插件，一个插件一个仓库，例如 `transform-ffmpeg`、`transform-asr-sherpa`、`transform-ocr-rapid`、`export-obsidian`、`export-notion`；release 附各平台二进制与 sha256 | MIT |
| 社区获取方式 store | 按 [02 §15.7](../research/02-adapter-ecosystems.md#157-适配器源信任与治理) 的 `store.json`、`index.json` 分层；官方客户端不预置任何 store | 各 store 自定 |
| 社区转换、导出插件 | 提 PR 进官方索引，或自建 store | 作者自定 |

- **命名**：官方插件在 `xiecun-app` 组织下不加 `xiecun-` 前缀；社区插件仓库名用前缀 `xiecun-transform-`、`xiecun-export-`、`xiecun-fetch-`。两者都加 GitHub topic `xiecun-plugin`。
- **官方插件也走插件协议。** Obsidian 导出、ffmpeg、默认 ASR、默认 OCR 都做成独立仓库的官方插件，随安装包预装，不写进内核。社区插件拿到的是同一套能力。
- **按法律风险分索引。** 获取方式是唯一涉及平台的一类，官方索引不收，客户端也不预置任何获取方式 store。这样即使某个获取方式出事，官方索引和插件生态不受连累。

<a id="6-索引的运作"></a>
### 6. 索引的运作

- **提交**：PR 只加一个 `index/<id>.yaml`（仓库地址、类别），一个 PR 一个插件。
- **校验分两段**：
  - `pull_request` 任务只有读权限，在临时环境里下载插件 release，校验清单 schema、sha256、权限声明，跑契约测试，结果上传为 artifact；
  - `workflow_run` 任务读取结果，以 `github-actions[bot]` 评论、打标签；
  - `pull_request_target` 里只把 PR 内容当数据读，不签出、不执行 PR 带来的任何东西。
- **更新**：定时任务轮询各插件的 Latest Release，校验通过后把版本、各平台资产的 sha256、权限摘要写进生成的 `index.json`。作者不用再提 PR，但每个新版本都过同一套校验；校验失败就在索引仓库开 issue 并 @ 作者。
- **下架**：`removed.yaml`（原因、日期）、`deprecated.yaml`、按版本禁用、恶意清单，都是数据文件，由客户端读取后执行。
- **需要 `[bot]` 身份或跨仓库写入时**：注册一个只装在索引仓库的 GitHub App，私钥放仓库 secrets，workflow 里用 `actions/create-github-app-token` 换 1 小时的 token。不托管服务。
- **托管 GitHub App** 在下面任一情况出现时再加：插件多到轮询不经济；要发版后即时收录；要读私有源码核对构建；要逐版本的扫描服务。加了 App 之后，它也只给索引仓库开 PR 或提交，索引始终以仓库里的文件为准，每次变更都是一条可审计的提交。

<a id="7-导出与读回"></a>
### 7. 导出与读回

- **只读回用户写的层**：`notes.md` 的 `takeaway` 与正文、主题页、用户标签。原文、图里的字、口播文字、画面描述、AI 摘要每次导出都重写，界面上写明。
- **身份写在文件里**：每个导出文件带 `xiecun_id`（Obsidian frontmatter、Logseq `xiecun-id::`、思源 `custom-xiecun-id`、Notion 属性）。导出记录放 `.xiecun/exports/<target>.json`，只是缓存，丢了可以按 `xiecun_id` 扫描重建；用户重命名、移动导出文件后照样认得回来。
- **用户区用正文标记划出**：Obsidian 用 `%% xiecun:begin notes %%` 与 `%% xiecun:end notes %%`，通用 Markdown 与 Logseq 用 HTML 注释；不用 YAML 注释，因为 Obsidian 改属性时会删掉它。标记缺失、重复或嵌套时不覆盖，只报告。
- **变更检测用上次导出的哈希。** 读回的改动作为「用户自己的改动」写进改动记录，可以整批撤销，不走 Agent 的改动提议。歇存和笔记软件两边都改过同一个用户区时不自动合并，两版并列让用户选。
- **导出插件不直接写目标目录**，结果交给宿主写，宿主记哈希。
- **frontmatter 沿用 Web Clipper 默认键** `title`、`source`、`author`、`published`、`created`、`description`、`tags`，再加 `xiecun_id`、`platform`、`kind`。
- **目标顺序**：Obsidian（同时产出通用 Markdown 文件夹）、思源、Notion、Logseq OG、飞书、语雀。云端目标的读回只做轮询；飞书、语雀先只做单向导出。
- **主题页**：默认导出一份副本到已有 vault，按上面的规则读回；也支持「资料库即 vault」，`topics/` 直接作为 vault 里的文件夹，用户在 Obsidian 里改的就是真相文件，歇存监听变化后重新读取。

<a id="8-gpl-工具"></a>
### 8. GPL 工具

取得法律意见前，GPL、AGPL 授权的工具只做「用户自己安装、歇存以独立进程调用」，不随安装包分发。官方插件本身用 MIT；它调用的第三方二进制只从上游官方 release 下载，并锁定 sha256。

<a id="放弃的"></a>
## 放弃的

| 方案 | 原因 |
|---|---|
| 第一步就做托管 GitHub App | 要持久运行的 webhook 服务、私钥托管、签名校验、排队与重试；纯 Actions 已经能收 PR、校验、评论。保留为后续选项，触发条件见 [决定 6](#6-索引的运作) |
| 官方索引收获取方式，或客户端预置获取方式 store | Tachiyomi 的应用、扩展、网站同在一个组织，一起被下线（[02 §1](../research/02-adapter-ecosystems.md#1-tachiyomi-与-mihon-扩展生态)） |
| 三类插件各一份清单 | 字段大半重合，三套 schema 会各自漂移 |
| 现在就做 `wasm-runner`，或只用 Wasm | ffmpeg、ASR、OCR、本地视觉模型做不了；眼下没有纯变换插件需要它 |
| wazero 或 Extism 嵌进内核 | [ADR-001](ADR-001-stack.md) 要求重计算与插件外置；Extism 的 Go SDK 停滞 |
| 在歇存里运行 Obsidian 插件 | 依赖 Obsidian API、跑在渲染进程、没有隔离。改为导出格式对齐、协议兼容、包装命令行工具、把别的工具的产物当导入来源（[05 §6.5](../research/05-note-apps-and-plugins.md#65-怎么复用现有插件与工具)） |
| 插件直接写资料库或目标目录 | 宿主是唯一写方，才能校验、记哈希、整批撤销 |
| 插件自己持有模型密钥 | 一个插件就能读走全部密钥；改为宿主注入或代发 |
| 通过管道传大块数据 | 只传文件路径 |
| 所有平台一律标「已沙箱」 | Windows 上文件与网络现在做不到强制 |

<a id="影响"></a>
## 影响

文档。用户说这些到设计阶段再一起改，这次只列出：

- [PRD §4](../PRD.md#4-原则) 原则 4 的「一切可插拔」加「导出」，和获取方式、识别器并列。
- [SPEC §8](../SPEC.md#8-适配器协议-v0) 扩展清单：加 `kind`、转换与导出的字段、`permissions`；补转换插件的 JSON-RPC 协议。顺带把 [SPEC §2](../SPEC.md#2-资料库目录真相源) 的 `threads/`、`thoughts/` 改成 product-model 的 `topics/`，导出读的是 `topics/`。
- [product-model §8.1](../product-model.md#81-设置清单) 设置清单加「导出」分组；「自动更新获取方式」保留，但扩大了权限的新版本要等用户同意。
- CLAUDE.md 重写时，技术栈里「适配器」一行改为三类插件，并指向本条。

代码，实现时做：

- `pkg/contract`：`adapter.yaml` 的 Go 类型与 schema 加 `kind` 等字段；新增转换、导出消息的 schema。插件清单与消息的 schema 以内核的 `pkg/contract` 还是 `plugin-kit` 为准，实现时定。
- 内核：插件运行器、各平台沙箱 profile 生成（每个 macOS 大版本回归）、provider 代理、导出记录、读回合并。
- 新仓库 `plugin-kit`、`plugins` 和首批官方插件，开始实现时再建。

[ADR-001](ADR-001-stack.md) 的结论不变，本条是其中「独立进程」一条的展开。

<a id="待核实"></a>
## 待核实

完整清单见 [05 §7](../research/05-note-apps-and-plugins.md#7-待核实清单)。和本条直接相关的：

- GPL、AGPL 工具作为独立进程被调用并随歇存分发的合规边界，需要法律意见；随包预装的 ffmpeg 用哪种许可证的构建。
- Seatbelt 放行 Metal、VideoToolbox 需要的最少规则；macOS 上 `RLIMIT_AS` 是否生效。
- Linux 桌面会话里 cgroup v2 委派是否普遍可用；GPU 设备最少要 bind 哪些；Ubuntu 限制 userns 时 bubblewrap 的降级与报错。
- Windows AppContainer 对 GPU 与 Media Foundation 的影响；MXC 执行器能否用于生产。
- wazero 有无按指令计数的配额。
- 飞书、语雀是否保留 HTML 注释，决定它们以后能不能读回。
- 「资料库即 vault」时，Obsidian 改属性会重写 YAML、重命名会改写链接，对真相文件的影响。
- 思源对同一 `path` 重复调用 `createDocWithMd` 的实际结果。
