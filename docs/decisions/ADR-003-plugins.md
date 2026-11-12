<a id="adr-003-插件架构与分发"></a>
# ADR-003 插件架构与分发

日期：2026-10-28 · 状态：accepted

本条定插件的种类、进程与协议、清单、权限与隔离、仓库拆分、分发与校验、导出与读回。字段和调用细节在[插件协议](../protocol/README.md)。依据是调研 [05 §6](../research/05-note-apps-and-plugins.md#6-对歇存插件化架构的启示)；分发方式按用户的决定用 GitHub 组织加 GitHub App，不做索引仓库。

<a id="背景"></a>
## 背景

usuyo 要接三种外部能力：从平台取回内容，把音视频和图片变成文字，把资料库导出到笔记软件。[PRD 原则](../PRD.md#4-原则) 1 要求内核平台中立，原则 7 要求能换的都能换；[ADR-004](ADR-004-rust-tauri.md) 要求重计算与插件都是独立进程，不进内核。

调研 05 的发现：

- 平台抓取正在从笔记插件里搬出去。24 个 Obsidian 社区插件里有 7 个把抓取和转写放进本机的独立后端或命令行工具，做的就是 usuyo 获取插件的事（[05 §1.4](../research/05-note-apps-and-plugins.md#14-共性与对歇存的含义)）。
- 别人的插件依赖宿主 API，拿不出来用；能复用的是背后的协议和命令行工具，转写基本都是 OpenAI 转写接口的形状（[05 §2.5](../research/05-note-apps-and-plugins.md#25-运行方式统计与可复用协议)）。
- GitHub App 能做而 Actions 做不了的有四件事，其中一件是不需要作者配合就收到插件仓库的 release 事件（[05 §3.9](../research/05-note-apps-and-plugins.md#39-github-app-与-actions-的能力差别)）。
- 主流笔记宿主都没有运行时强制隔离；ffmpeg、ASR、OCR、本地视觉模型只能跑原生进程（[05 §4.1](../research/05-note-apps-and-plugins.md#41-宿主插件的安全模型)、[05 §4.4](../research/05-note-apps-and-plugins.md#44-原生进程还是-wasm)）。
- 没有工具从笔记软件读回改动（[05 §5.9](../research/05-note-apps-and-plugins.md#59-读回双向的先例与歇存的做法)）。

用户定的分发方式：主仓库和官方插件放在 GitHub 组织 `usuyo` 里，进组织就是上架；自己维护的获取插件放另一个组织；第三方插件放在作者自己的仓库，用户按仓库直接安装；不做靠 PR 收录的索引仓库；GitHub App 从一开始就做。

<a id="决定"></a>
## 决定

<a id="1-插件分三类"></a>
### 1. 插件分三类

| 类 | 做什么 | 输入 | 输出 | 官方插件 |
|---|---|---|---|---|
| 获取 fetch | 从平台或外部文件取回内容 | URL、订阅源、导出包 | `raw/` 快照、文本、媒体 | 只有两类不含平台知识的预装插件（见 [§5](#5-仓库拆分)） |
| 转换 transform | 把条目里已有的文件变成新的一层 | 条目文件的路径，只读 | `derived/` 下的约定文件，关键帧在 `derived/keyframes/` | 有 |
| 导出 export | 把条目、主题页、卡片、成稿写到笔记软件，读回用户在那边改的 | 用户层和机器层的只读视图（JSON） | 交给宿主写的文件，或经宿主代发的云 API 调用；读回时交回用户层的值 | 有；云端目标标「离开本机」 |

- **导入是获取的一种**，来源是文件而不是 URL，沿用获取插件的协议，`origins[].entry` 记 `import`。
- **模型 provider 不是插件**，是内核里的一份配置：名称、OpenAI 兼容端点、模型、密钥。密钥存系统钥匙串。转换插件只引用 provider 名称，请求由宿主代签代发，插件拿不到密钥明文。
- **留在内核的**：编排（按转换插件声明的输入输出，把抽音轨、转写、摘要串起来）、权限落实、provider 代理、导出记录与读回合并、写资料库。

<a id="2-进程与协议"></a>
### 2. 进程与协议

所有插件都是独立进程。内核只认一种协议：独立进程加 JSON。

| 类 | 控制协议 | 数据 |
|---|---|---|
| 获取 | 子命令 `canonicalize`、`fetch`、`list`、`health`，stdout 输出 JSON 信封（[插件协议 §3](../protocol/fetch.md#3-获取插件协议)） | `--out` 指向宿主分配的临时目录 |
| 转换 | stdio 上换行分隔的 JSON-RPC 2.0，分帧和 MCP stdio 相同。方法 `describe`、`health`、`run`；`progress` 通知；支持取消；stdin 关闭后退出。可以常驻，模型加载一次处理多条 | 只传文件路径，大块数据不走管道 |
| 导出 | 同转换；另有 `readback`：插件只读目标文件夹，把目标文件转回用户层的值交给宿主；底本留在宿主，不交给插件 | 同转换 |

- **只有宿主写资料库。** 插件只写宿主分配的临时目录，宿主校验后再移入 `derived/` 或导出目录。
- **重计算只做原生进程。** ffmpeg、ASR、OCR、本地视觉模型的 Wasm 版太慢、输入有 2 GB 上限、用不了 GPU（[05 §4.4](../research/05-note-apps-and-plugins.md#44-原生进程还是-wasm)）。
- **Wasm 要等出现纯变换插件**（文本规范化、字幕互转、导出格式映射）时再做：官方的 `wasm-runner` 进程，Wasm 运行时不进内核；对内核来说 runner 只是又一个进程插件；macOS 上需要 JIT 的 entitlement 只给 runner。
- **转换插件可以做成薄客户端**，后端是用户自己跑的服务，兼容 OpenAI `/v1/audio/transcriptions`、whisper-asr-webservice `/asr`、OpenAI chat、Umi-OCR `/api/ocr`。

<a id="3-清单"></a>
### 3. 清单

- 三类插件共用一份清单 `usuyo-plugin.yaml`，用 `kind: fetch | transform | export` 区分，共用一份 schema；清单、信封、JSON-RPC 消息的 schema 以 plugin-kit 为准，内核的 `crates/usuyo-contract` 只放资料库自己的类型。
- 字段：
  - 共有 `id`、`name`、`version`、`protocol`、`kind`、`license`、`homepage`、`maintainers`、`install`、`requires.runtime`、`permissions`；
  - 获取另有 `access`（七层之一，含义照[调研 06 §1.1](../research/06-platform-access-and-account-risk.md#11-七个层级)）、`login`（`none`、`optional`、`required`）和 `sites`（会用到哪个站点的账号）。能力之间不同时按能力分别声明，顶层取最高一档。这些是给用户挑选用的标签，不是闸门；
  - 转换另有 `consumes`、`produces`、`provider`；
  - 导出另有 `target`、`writes`、`readback`。
- 完整的字段表在[插件协议 §2](../protocol/manifest.md#2-清单)。

<a id="4-权限与隔离"></a>
### 4. 权限与隔离

清单里的 `permissions` 声明 `fs.read`、`fs.write`、`network`（`none` 或 `allowlist`）、`gpu`、`exec`、`env`、`limits`（内存、超时）和设备类，原生插件与将来的 Wasm 插件用同一个结构（[05 §4.5](../research/05-note-apps-and-plugins.md#45-权限声明与各平台落实程度)）。路径用宿主展开的占位符，插件拿不到资料库根目录。导出插件读回时要的目标文件夹只读权限也在这里声明。

各平台的落实：

- **macOS**：Seatbelt profile 强制文件、网络与再起子进程；内存靠宿主轮询，超限即杀。
- **Linux**：bubblewrap 强制，不可用时降级 Landlock，都不行就仅告知；内存用 cgroup v2 或 prlimit。
- **Windows**：先只强制 Job Object（内存、进程数、超时）与环境变量，文件和网络先仅告知。
- **所有平台**：环境变量显式传入；超时到期杀整个进程组；`allowlist` 网络只能经宿主代理出站，密钥由宿主在代发时注入。

界面按平台、按声明项显示「强制 / 部分 / 仅告知」三档，不一律写「已沙箱」。宿主不代装根证书，也不向别的进程注入。

用户同意：

- 安装时展示权限，用户按资料库同意。
- 新版本扩大权限时，这个版本先停用，等用户重新同意；不静默更新扩大了权限的版本。登录站点的 (`site`, `credential`, `domains`) 出现新值、新增密钥设置或改了密钥的注入目标，也算扩大权限（[插件协议 §8.5](../protocol/distribution.md#85-添加来源更新与权限扩大)）。做法照 GitHub App 改权限需安装方批准，防的是经自动更新下发恶意代码。
- 新版本调高 `access` 或 `login` 只在更新说明里标出，不停用。
- 插件的启用状态、配置与密钥放本机配置目录，不放进会同步的资料库，避免同步过来的插件配置直接执行（[05 §4.1](../research/05-note-apps-and-plugins.md#41-宿主插件的安全模型)）。

<a id="5-仓库拆分"></a>
### 5. 仓库拆分

照 Tachiyomi 拆成 Mihon（应用）与 Keiyoushi（扩展）的分法，应用和获取插件分属不同的组织；和 Keiyoushi 的单一大仓库不同，获取插件一个插件一个仓库（取舍见[插件协议 §8.8](../protocol/distribution.md#88-获取方式组织的仓库结构)）。

| 仓库 | 内容 | 许可证 |
|---|---|---|
| `usuyo/usuyo` | 内核、CLI、MCP、桌面壳；插件运行器、权限落实、provider 代理、导出记录与读回合并；屏蔽清单 `security/plugin-blocklist.json`；不含任何平台抓取逻辑 | AGPL-3.0 |
| `usuyo/plugin-kit` | 清单与消息的 JSON Schema、`derived/` 文件格式、契约测试套件与 `testdata/`、SDK、插件模板、本地校验 | MIT |
| `usuyo/<kind>-<name>` | 官方插件，一个插件一个仓库：转换、导出（例如 `transform-ffmpeg`、`transform-asr-sherpa`、`export-obsidian`），外加两类预装的获取插件：通用网页获取（只做 `public-web`、不登录、没有站点专用规则）和别的工具导出文件的导入（书签 HTML、Pocket、Readwise 一类，不联网），`id` 用 `builtin/` 前缀。release 附各平台二进制，清单列出各资产的 sha256 | MIT |
| `usuyo` 下的 GitHub App 服务 | 接收 release 事件，跑校验，写 check run；仓库名实现时定 | 实现时定 |
| `usuyo-fetch/<name>` | 自己维护的平台获取插件，一个插件一个仓库；平台官方导出包（个人信息下载、Google Takeout、X 归档一类）的导入插件也在这里 | MIT |
| `usuyo-fetch-hook/<name>` | 读本机客户端数据的 client-hook 插件，单独一个组织，一次通知最多波及这一组 | MIT |
| 第三方插件 | 作者自己的仓库，三类都可以 | 作者自定 |

- **命名**：三个组织内的仓库不加 `usuyo-` 前缀；第三方插件仓库名用前缀 `usuyo-fetch-`、`usuyo-transform-`、`usuyo-export-`。都加 GitHub topic `usuyo-plugin`。导出文件里的身份键是 `usuyo_id`。
- **官方插件也走插件协议。** Obsidian 导出、ffmpeg、默认 ASR、默认 OCR 都做成组织里独立仓库的官方插件，随安装包预装，不写进内核。第三方插件拿到的是同一套能力。
- **按法律风险分组织。** 平台获取插件不进 `usuyo` 组织，客户端也不预置任何获取来源，用户手动添加组织名或 `owner/repo`，添加时可以一次加 `usuyo-fetch` 和 `usuyo-fetch-hook` 两个。本机文件、截图、粘贴文字的收下是宿主自己的功能，不是插件。

<a id="6-分发与校验"></a>
### 6. 分发与校验

- **官方插件列表就是组织本身。** 客户端列出 `usuyo` 组织里带 topic `usuyo-plugin` 的仓库，版本和资产取各自的 GitHub Release。不另做索引文件，不走 PR 收录。仓库进组织就是上架，由组织成员审核。
- **来源分三种显示**：官方、第三方来源 · 组织名、第三方 · 未经 usuyo 审核。第三方插件不经过 usuyo，作者在自己的仓库发布，用户输入 `owner/repo` 直接安装，浏览靠搜索 topic `usuyo-plugin`。
- **责任**：用户添加第三方插件来源时确认一次声明：用途是个人备份、离线阅读、注意力整理，平台账号和法律上的后果由用户自己承担，链到[调研 06 §16](../research/06-platform-access-and-account-risk.md#16-账号风险的证据) 的证据。主仓库 README、获取插件组织的主页、插件 README 模板里也写，模板见[插件协议附录 A](../protocol/distribution.md#附录-a-声明模板)。不做逐个插件的弹窗。
- **GitHub App 从一开始就做**，归 `usuyo` 组织所有，装在组织上；webhook 由无服务器中转接收、验签、排队，再触发官方组织的 GitHub Actions 拉插件、校验清单 schema 和各平台资产的 sha256、跑契约测试，结果写回 check run。没有自己的服务器，App 私钥放在中转平台上。第三方作者也可以把它装到自己的仓库，拿到同样的校验；客户端显示每个版本的校验结果：通过、失败、没有校验。
- **身份与拦截。** 插件 `id` 等于 `owner/repo`，客户端另按 GitHub 的数字仓库 id 认仓库，改名、转移或旧名被重新注册时停止更新。官方组织的插件缺来源证明、或校验没通过就拒装；用户添加的来源（包括 `usuyo-fetch`）在添加时固定签发者，以后签发者变了就停止更新；第三方插件校验没通过时只标出，安装前多一步确认（[插件协议 §8.3](../protocol/distribution.md#83-校验与来源证明)）。
- **安装与更新以 release 资产的 sha256 为准。** 客户端下载后核对清单里列出的 sha256，不一致就拒装。
- **屏蔽清单。** 不做下架用的索引，只在主仓库放一份 `security/plugin-blocklist.json`，列出恶意仓库和恶意版本（仓库、版本范围、原因、日期）。客户端在启动、安装和更新前读取它，命中的拒装，已装的停用。

<a id="7-导出与读回"></a>
### 7. 导出与读回

- **资料库不当 vault 用。** 资料库的形状按 usuyo 自己的需要定（[SPEC 第 1 层](../spec/library.md#1-资料库-v1第-1-层)）；在 Obsidian 里改的是导出的那份，再读回 usuyo。导出只在用户点「现在导出」时跑，先读回再重写。
- **读回的是用户层**：笔记、记一句、标签、主题，以及主题页、卡片、成稿的正文。划线、标题、日期和分隔线以下的各层只出不进，每次导出重写，界面上写明。主题成员走条目和卡片的 `usuyo_topics`。
- **用户区是一条分隔线。** 条目笔记里只有一行分隔线，上面是你的笔记，下面每次导出重写；主题、卡片、成稿不加标记，属性之后的整篇是正文。通用 Markdown 用同一条线，写成 HTML 注释。
- **身份写在文件里**：每个导出文件带 `usuyo_id`（Obsidian frontmatter、Logseq `usuyo-id::`、思源 `custom-usuyo-id`、Notion 属性）；插件按它认挪过、改过名的文件。
- **底本加三方合并。** 导出记录放按机器目录的 `libraries/<libraryId>/exports/<target>.json`（[SPEC §1.1](../spec/library.md#11-目录树)），每个对象是 `{path, hash, base}`，`base` 是上次交给 `run` 的那份用户层。每次导出先读回，宿主拿它和底本、库里现在的那份照 [SPEC §1.8](../spec/library.md#18-同步) 三方合并，经 [SPEC §1.4](../spec/library.md#14-你写的那一层)「读进来」写进库，记 `actor: external`、`via: readback:<插件 id>`。一次导出算一批，能整批撤销，不走改动提议。合不上的，库里那份留下，目标那份进回收站（`reason: conflict`），出「两边都改过的」卡片。路径丢了能按 `usuyo_id` 扫描重建，底本丢了照 [SPEC §1.8](../spec/library.md#18-同步)「云盘的冲突副本」那条没底本的做法。
- **导出插件不直接写目标目录**，结果交给宿主写，写之前再比一次哈希。读回时沙箱放行目标文件夹，只读，协议规定只有 `readback` 读它（[插件协议 §7.1](../protocol/isolation.md#71-常驻进程的路径)）。
- **删掉以后**：usuyo 里删掉一条，已经导出去的副本不管；在 Obsidian 里删掉的笔记，下次导出重新生成。
- **frontmatter**：导出文件沿用 Web Clipper 默认键 `title`、`source`、`author`、`published`、`created`、`description`、`tags`，再加 `usuyo_id`、`platform`、`kind`；资料库里的 front matter 用 camelCase、不加前缀（[SPEC §0.1](../spec/conventions.md#01-命名)），Obsidian 要的样子由导出插件转换（[插件协议 §4.4](../protocol/transform-export.md#44-导出的-run-与-readback)）。
- **目标顺序**：Obsidian（同时产出通用 Markdown 文件夹）、思源、Notion、Logseq OG、飞书、语雀。云端目标的读回只做轮询；飞书、语雀只做单向导出。

<a id="8-gpl-工具"></a>
### 8. GPL 工具

取得法律意见前，GPL、AGPL 授权的工具只做「用户自己安装、usuyo 以独立进程调用」，不随安装包分发。官方插件本身用 MIT；它调用的第三方二进制只从上游官方 release 下载，并锁定 sha256。

<a id="放弃的"></a>
## 放弃的

| 方案 | 原因 |
|---|---|
| 靠 PR 收录的索引仓库：`plugins` 仓库、`index/<id>.yaml`、生成的 `index.json`，即 [05 §6.4](../research/05-note-apps-and-plugins.md#64-索引用-github-仓库还是-github-app) 的建议 | 用户认为丑陋，也多一层维护；组织本身就能当官方列表，topic 就能用来浏览 |
| 中心化的插件商店网站 | 多一个要运行和审核的服务；仓库加 topic 已经能浏览和安装 |
| 平台获取插件进官方组织，或客户端预置任何获取来源 | Tachiyomi 的应用、扩展、网站同在一个组织，一起被下线（[02 §1](../research/02-adapter-ecosystems.md#1-tachiyomi-与-mihon-扩展生态)） |
| 宿主按 `access` 推导风险档、平台修正表，失败后不许往高层换，对 `private-api`、`client-hook`、微信视觉层单独弹确认 | 层级是给用户挑的标签，不是闸门；运行器按用户排的顺序依次试，责任靠添加来源时的一次声明 |
| 三类插件各一份清单 | 字段大半重合，三套 schema 会各自漂移 |
| 现在就做 `wasm-runner`，或只用 Wasm | ffmpeg、ASR、OCR、本地视觉模型做不了；眼下没有纯变换插件需要它 |
| Wasm 运行时嵌进内核 | [ADR-004](ADR-004-rust-tauri.md) 要求重计算与插件外置 |
| 在 usuyo 里运行 Obsidian 插件 | 依赖 Obsidian API、跑在渲染进程、没有隔离。改为导出格式对齐、协议兼容、包装命令行工具、把别的工具的产物当导入来源（[05 §6.5](../research/05-note-apps-and-plugins.md#65-怎么复用现有插件与工具)） |
| 资料库即 vault，在 Obsidian 里直接改真相文件 | 资料库的形状要按同步合并、引用、Agent 读写来定，不迁就笔记软件 |
| 用正文标记划用户区，两边都改过时两版并列让用户选 | 规则多、易坏；改成一条分隔线加三方合并，和同步用同一套 |
| 插件直接写资料库或目标目录 | 宿主是唯一写方，才能校验、记哈希、整批撤销 |
| 插件自己持有模型密钥 | 一个插件就能读走全部密钥；改为宿主注入或代发 |
| 通过管道传大块数据 | 只传文件路径 |
| 所有平台一律标「已沙箱」 | Windows 上文件与网络现在做不到强制 |

<a id="影响"></a>
## 影响

代码：

- 内核：插件运行器、各平台沙箱 profile 生成（每个 macOS 大版本回归）、provider 代理、导出记录、读回合并；做到哪见[总纲](../BUILD.md#3-现在做到哪)。
- `plugin-kit` 仓库、GitHub App 的中转和工作流、官方插件，各自一个仓库（[总纲 012](../BUILD.md#012-插件的安装和分发)）。

运维：

- 中转要验 webhook 签名、排队与重试，App 私钥放在中转平台上；受 Actions 免费额度限制。
- 客户端直接调 GitHub API 列组织仓库、读 release 和 check run。未登录时每个 IP 每小时只有 60 次 REST 请求，客户端要缓存结果、按需刷新。

[ADR-004](ADR-004-rust-tauri.md) 的结论不变，本条是其中「独立进程」一条的展开。

<a id="待核实"></a>
## 待核实

完整清单见 [05 §7](../research/05-note-apps-and-plugins.md#7-待核实清单)。和本条直接相关的：

- GPL、AGPL 工具作为独立进程被调用并随 usuyo 分发的合规边界，需要法律意见；随包预装的 ffmpeg 用哪种许可证的构建。
- Seatbelt 放行 Metal、VideoToolbox 需要的最少规则；macOS 上 `RLIMIT_AS` 是否生效。
- Linux 桌面会话里 cgroup v2 委派是否普遍可用；GPU 设备最少要 bind 哪些；Ubuntu 限制 userns 时 bubblewrap 的降级与报错。
- Windows AppContainer 对 GPU 与 Media Foundation 的影响；MXC 执行器能否用于生产。
- Wasm 运行时有无按指令计数的配额。
- 飞书、语雀是否保留 HTML 注释，决定它们以后能不能读回。
- 思源对同一 `path` 重复调用 `createDocWithMd` 的实际结果。
- 客户端不登录时能否读到公开仓库的 check run；check run 挂在 release tag 指向的提交上，客户端按 tag 查找是否可靠。
- 组织成员审核的具体流程：谁能把仓库建进组织或转入组织，上架前要过哪些检查。
