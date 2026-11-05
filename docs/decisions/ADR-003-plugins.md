<a id="adr-003-插件架构与分发"></a>
# ADR-003 插件架构与分发

日期：2026-10-28 · 状态：proposed

本条定插件的种类、进程与协议、清单、权限与隔离、仓库拆分、分发与校验、导出与读回。依据是调研 [05 §6](../research/05-note-apps-and-plugins.md#6-对歇存插件化架构的启示)，需要拍板的 P1–P9 按 05 的建议定（见 [05 §6.8](../research/05-note-apps-and-plugins.md#68-需要拍板的事)），只有分发方式按用户 2026-10-28 的决定改成 GitHub 组织加 GitHub App，不做索引仓库。清单字段与契约的改动留到实现时再做。

<a id="背景"></a>
## 背景

usuyo 要接三种外部能力：从平台取回内容，把音视频和图片变成文字，把资料库导出到笔记软件。[PRD §4](../PRD.md) 原则 1 要求内核平台中立，原则 4 要求一切可插拔；[ADR-001](ADR-001-stack.md) 要求重计算与适配器都是独立进程，不进内核。

调研 05 的发现：

- 平台抓取正在从笔记插件里搬出去。24 个 Obsidian 社区插件里有 7 个把抓取和转写放进本机的独立后端或命令行工具，做的就是 usuyo「获取方式」的事（[05 §1.4](../research/05-note-apps-and-plugins.md#14-共性与对歇存的含义)）。
- 别人的插件依赖宿主 API，拿不出来用；能复用的是背后的协议和命令行工具，转写基本都是 OpenAI 转写接口的形状（[05 §2.5](../research/05-note-apps-and-plugins.md#25-运行方式统计与可复用协议)）。
- GitHub App 能做而 Actions 做不了的有四件事，其中一件是不需要作者配合就收到插件仓库的 release 事件（[05 §3.9](../research/05-note-apps-and-plugins.md#39-github-app-与-actions-的能力差别)）。
- 主流笔记宿主都没有运行时强制隔离；ffmpeg、ASR、OCR、本地视觉模型只能跑原生进程（[05 §4.1](../research/05-note-apps-and-plugins.md#41-宿主插件的安全模型)、[05 §4.4](../research/05-note-apps-and-plugins.md#44-原生进程还是-wasm)）。
- 没有工具从笔记软件读回改动（[05 §5.9](../research/05-note-apps-and-plugins.md#59-读回双向的先例与歇存的做法)）。

2026-10-28 用户定了分发方式：主仓库和官方插件都放在 GitHub 组织 `usuyo` 里，进组织就是上架；第三方插件放在作者自己的仓库，用户按仓库直接安装；不做靠 PR 收录的索引仓库；GitHub App 从一开始就做。获取方式不进组织。

<a id="决定"></a>
## 决定

<a id="1-插件分三类"></a>
### 1. 插件分三类

| 类 | 做什么 | 输入 | 输出 | 官方插件 |
|---|---|---|---|---|
| 获取 fetch | 从平台或外部文件取回内容 | URL、订阅源、导出包 | `raw/` 快照、文本、媒体 | 没有，只能是第三方仓库 |
| 转换 transform | 把条目里已有的文件变成新的一层 | 条目文件的路径，只读 | `derived/` 下的约定文件、`images/keyframes/` | 有 |
| 导出 export | 把条目和主题页写到笔记软件，可选读回用户改动 | 条目与主题页的只读视图（JSON） | 交给宿主写的文件，或经宿主代发的云 API 调用；读回时返回结构化改动 | 有；云端目标标「离开本机」 |

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
| `usuyo/usuyo` | 内核、CLI、MCP、桌面壳；插件运行器、权限落实、provider 代理、导出记录与读回合并；屏蔽清单 `security/plugin-blocklist.json`；不含任何平台抓取逻辑 | AGPL-3.0 |
| `usuyo/plugin-kit` | 清单与消息的 JSON Schema、`derived/` 文件格式、契约测试套件与 `testdata/`、Go / TypeScript / Python SDK、插件模板、本地校验命令 | MIT |
| `usuyo/<kind>-<name>` | 官方插件，只有转换与导出，一个插件一个仓库，例如 `transform-ffmpeg`、`transform-asr-sherpa`、`transform-ocr-rapid`、`export-obsidian`、`export-notion`；release 附各平台二进制，清单列出各资产的 sha256 | MIT |
| `usuyo` 下的 GitHub App 服务 | 接收 release 事件，跑校验，写 check run；仓库名实现时定 | 实现时定 |
| 第三方插件 | 作者自己的仓库，三类都可以；获取方式只能以这种形式存在 | 作者自定 |

- **命名**：官方插件在 `usuyo` 组织下不加 `usuyo-` 前缀；第三方插件仓库名用前缀 `usuyo-transform-`、`usuyo-export-`、`usuyo-fetch-`。两者都加 GitHub topic `usuyo-plugin`。
- **官方插件也走插件协议。** Obsidian 导出、ffmpeg、默认 ASR、默认 OCR 都做成组织里独立仓库的官方插件，随安装包预装，不写进内核。第三方插件拿到的是同一套能力。
- **按法律风险分组织内外。** 获取方式是唯一涉及平台的一类，一律不进 `usuyo` 组织，客户端也不预置任何获取方式。这样即使某个获取方式出事，组织和官方插件不受连累。

<a id="6-分发与校验"></a>
### 6. 分发与校验

- **官方插件列表就是组织本身。** 客户端列出 `usuyo` 组织里带 topic `usuyo-plugin` 的仓库，版本和资产取各自的 GitHub Release。不另做索引文件，不走 PR 收录。仓库进组织就是上架，由组织成员审核。
- **第三方插件不经过 usuyo。** 作者在自己的仓库发布；用户输入 `owner/repo` 直接安装；浏览靠搜索 topic `usuyo-plugin`。客户端清楚标出「官方」和「第三方 · 未经 usuyo 审核」两种来源。
- **GitHub App 从一开始就做**，归 `usuyo` 组织所有，装在组织上：
  - 每次 release 自动校验清单 schema、各平台资产的 sha256、跑契约测试，结果写成 check run；
  - 第三方作者也可以把它装到自己的仓库，拿到同样的校验；
  - 客户端显示每个版本的校验结果：通过、失败、没有校验。
- **安装与更新以 release 资产的 sha256 为准。** 客户端下载后核对清单里列出的 sha256，不一致就拒装。
- **屏蔽清单。** 不做下架用的索引，只在主仓库放一份 `security/plugin-blocklist.json`，列出恶意仓库和恶意版本（仓库、版本范围、原因、日期）。客户端在启动、安装和更新前读取它，命中的拒装，已装的停用。

<a id="7-导出与读回"></a>
### 7. 导出与读回

- **只读回用户写的层**：`notes.md` 的 `takeaway` 与正文、主题页、用户标签。原文、图里的字、口播文字、画面描述、AI 摘要每次导出都重写，界面上写明。
- **身份写在文件里**：每个导出文件带 `usuyo_id`（Obsidian frontmatter、Logseq `usuyo-id::`、思源 `custom-usuyo-id`、Notion 属性）。导出记录放 `.usuyo/exports/<target>.json`，只是缓存，丢了可以按 `usuyo_id` 扫描重建；用户重命名、移动导出文件后照样认得回来。
- **用户区用正文标记划出**：Obsidian 用 `%% usuyo:begin notes %%` 与 `%% usuyo:end notes %%`，通用 Markdown 与 Logseq 用 HTML 注释；不用 YAML 注释，因为 Obsidian 改属性时会删掉它。标记缺失、重复或嵌套时不覆盖，只报告。
- **变更检测用上次导出的哈希。** 读回的改动作为「用户自己的改动」写进改动记录，可以整批撤销，不走 Agent 的改动提议。usuyo 和笔记软件两边都改过同一个用户区时不自动合并，两版并列让用户选。
- **导出插件不直接写目标目录**，结果交给宿主写，宿主记哈希。
- **frontmatter 沿用 Web Clipper 默认键** `title`、`source`、`author`、`published`、`created`、`description`、`tags`，再加 `usuyo_id`、`platform`、`kind`。
- **目标顺序**：Obsidian（同时产出通用 Markdown 文件夹）、思源、Notion、Logseq OG、飞书、语雀。云端目标的读回只做轮询；飞书、语雀先只做单向导出。
- **主题页**：默认导出一份副本到已有 vault，按上面的规则读回；也支持「资料库即 vault」，`topics/` 直接作为 vault 里的文件夹，用户在 Obsidian 里改的就是真相文件，usuyo 监听变化后重新读取。

<a id="8-gpl-工具"></a>
### 8. GPL 工具

取得法律意见前，GPL、AGPL 授权的工具只做「用户自己安装、usuyo 以独立进程调用」，不随安装包分发。官方插件本身用 MIT；它调用的第三方二进制只从上游官方 release 下载，并锁定 sha256。

<a id="放弃的"></a>
## 放弃的

| 方案 | 原因 |
|---|---|
| 靠 PR 收录的索引仓库：`plugins` 仓库、`index/<id>.yaml`、生成的 `index.json`，即 [05 §6.4](../research/05-note-apps-and-plugins.md#64-索引用-github-仓库还是-github-app) 的建议 | 用户认为丑陋，也多一层维护；组织本身就能当官方列表，topic 就能用来浏览 |
| 中心化的插件商店网站 | 多一个要运行和审核的服务；仓库加 topic 已经能浏览和安装 |
| 获取方式进组织，或客户端预置任何获取方式 | Tachiyomi 的应用、扩展、网站同在一个组织，一起被下线（[02 §1](../research/02-adapter-ecosystems.md#1-tachiyomi-与-mihon-扩展生态)） |
| 三类插件各一份清单 | 字段大半重合，三套 schema 会各自漂移 |
| 现在就做 `wasm-runner`，或只用 Wasm | ffmpeg、ASR、OCR、本地视觉模型做不了；眼下没有纯变换插件需要它 |
| wazero 或 Extism 嵌进内核 | [ADR-001](ADR-001-stack.md) 要求重计算与插件外置；Extism 的 Go SDK 停滞 |
| 在 usuyo 里运行 Obsidian 插件 | 依赖 Obsidian API、跑在渲染进程、没有隔离。改为导出格式对齐、协议兼容、包装命令行工具、把别的工具的产物当导入来源（[05 §6.5](../research/05-note-apps-and-plugins.md#65-怎么复用现有插件与工具)） |
| 插件直接写资料库或目标目录 | 宿主是唯一写方，才能校验、记哈希、整批撤销 |
| 插件自己持有模型密钥 | 一个插件就能读走全部密钥；改为宿主注入或代发 |
| 通过管道传大块数据 | 只传文件路径 |
| 所有平台一律标「已沙箱」 | Windows 上文件与网络现在做不到强制 |

<a id="影响"></a>
## 影响

文档。用户说这些到设计阶段再一起改，这次只列出：

- [PRD §4](../PRD.md) 原则 4 的「一切可插拔」加「导出」，和获取方式、识别器并列。
- [SPEC §8](../SPEC.md#8-适配器协议-v0) 扩展清单：加 `kind`、转换与导出的字段、`permissions`；补转换插件的 JSON-RPC 协议。顺带把 [SPEC §2](../SPEC.md#2-资料库目录真相源) 的 `threads/`、`thoughts/` 改成 product-model 的 `topics/`，导出读的是 `topics/`。
- [product-model §8.1](../product-model.md#81-设置清单) 设置清单加「导出」分组；「自动更新获取方式」保留，但扩大了权限的新版本要等用户同意；「获取方式来源列表」改为按仓库安装的插件列表，标出「官方」与「第三方 · 未经 usuyo 审核」。
- CLAUDE.md 重写时，技术栈里「适配器」一行改为三类插件，并指向本条。

代码，实现时做：

- `pkg/contract`：`adapter.yaml` 的 Go 类型与 schema 加 `kind` 等字段；新增转换、导出消息的 schema。插件清单与消息的 schema 以内核的 `pkg/contract` 还是 `plugin-kit` 为准，实现时定。
- 内核：插件运行器、各平台沙箱 profile 生成（每个 macOS 大版本回归）、provider 代理、导出记录、读回合并。
- 新仓库 `plugin-kit`、GitHub App 服务和首批官方插件，开始实现时再建。

运维：

- GitHub App 要持续运行一个接收 webhook 的服务，包括托管、私钥保管、webhook 签名校验、排队与重试。
- 客户端直接调 GitHub API 列组织仓库、读 release 和 check run。未登录时每个 IP 每小时只有 60 次 REST 请求，客户端要缓存结果、按需刷新。

[ADR-001](ADR-001-stack.md) 的结论不变，本条是其中「独立进程」一条的展开。

<a id="修订-2026-11-01"></a>
## 修订 2026-11-01

按用户 2026-11-01 的方向修订插件范围的几处决定。上面的正文不改，有冲突的以本节为准；字段和调用细节见 [usuyo 插件协议 v1（草案）](../plugin-protocol.md)。

1. **名字跟着改名走。** 产品名 2026-11-05 定为 usuyo，命令 `usuyo`（[open-decisions §1.8](../open-decisions.md#18-grill-回答)）。官方组织是 `usuyo`，获取插件组织是 `usuyo-fetch`，client-hook 插件组织是 `usuyo-fetch-hook`（11-05）；topic 是 `usuyo-plugin`；第三方仓库前缀是 `usuyo-fetch-`、`usuyo-transform-`、`usuyo-export-`，两个组织内的仓库不加前缀；导出文件里的身份键是 `usuyo_id`，记录目录是 `.usuyo/`。之前用过的暂名 `xiecun`（歇存）、`seelu`（思鹭）只留在调研和 open-decisions 里。
2. **获取方式组织单独设**（改 [§5](#5-仓库拆分)、[§6](#6-分发与校验)）。自己维护的获取插件放进单独的 GitHub 组织 `usuyo-fetch`：照 Tachiyomi 拆成 Mihon（应用）与 Keiyoushi（扩展）的分法，应用和获取插件分属两个组织；和 Keiyoushi 的单一大仓库 extensions-source 不同，`usuyo-fetch` 里每个插件一个仓库。官方组织放应用、plugin-kit、转换和导出插件，外加两类不含平台知识、随安装包预装的获取插件（见第 5 条）；除此之外客户端不预置任何获取来源，用户手动添加组织名或 `owner/repo`。来源分三种显示：官方、第三方来源 · 组织名、第三方 · 未经 usuyo 审核。理由和仓库结构的取舍见 [plugin-protocol §8.8](../plugin-protocol.md#88-获取方式组织的仓库结构)。
3. **风险的处理方式。**
   - 获取插件的清单加两个描述性的必填字段：`access`（七层之一，含义照 [调研 06 §1.1](../research/06-platform-access-and-account-risk.md#11-七个层级)）和 `login`（`none`、`optional`、`required`）；能力之间不同时按能力分别声明，顶层取最高一档。另用 `sites` 写明会用到哪个站点的账号。这些是给用户挑选用的标签，不是闸门。
   - 不由宿主推导风险档，不做平台修正表；运行器按用户排的顺序依次试，失败换下一个，不限制往哪一层换；不对 `private-api`、`client-hook`、微信的视觉层单独弹确认。
   - 新版本调高 `access` 或 `login` 只在更新说明里标出，不停用。[§4](#4-权限与隔离) 的「扩大权限要用户重新同意」照旧，这是防经自动更新下发恶意代码的安全规则；`permissions` 加了设备类；登录站点的 (`site`, `credential`, `domains`) 出现新值、新增密钥设置或改了密钥的注入目标，也算扩大权限（[plugin-protocol §8.5](../plugin-protocol.md#85-添加来源更新与权限扩大)）。
   - 责任：用户添加第三方插件来源（组织名或 `owner/repo`）时确认一次声明：用途是个人备份、离线阅读、注意力整理，平台账号和法律上的后果由用户自己承担，链到 [调研 06 §16](../research/06-platform-access-and-account-risk.md#16-账号风险的证据) 的证据。主仓库 README、获取方式组织的主页、插件 README 模板里也写声明，模板见 [plugin-protocol 附录 A](../plugin-protocol.md#附录-a-声明模板)。不做逐个插件的弹窗。
4. **清单与协议在新文档定稿**（改 [§3](#3-清单)）。清单文件名定为 `usuyo-plugin.yaml`，不再沿用 `adapter.yaml`；字段、获取插件的子命令与错误码、转换与导出的 JSON-RPC 方法、投递接口、凭据、设备类权限、发版与来源证明都写在 [plugin-protocol](../plugin-protocol.md)。`pkg/contract` 的 Go 类型、JSON Schema 和 `testdata/` 仍在实现插件运行器时改。
5. **两类预装的获取插件**（2026-11-01 用户同意，[open-decisions P1、P3](../open-decisions.md#7-获取与插件)）。通用网页获取（只做 `public-web`、不登录、没有站点专用规则）和读别的工具导出文件的导入（书签 HTML、Pocket、Readwise 一类，不联网）进官方组织，随安装包预装，`id` 用 `builtin/` 前缀；内核仍不含获取逻辑，只调插件。平台官方导出包（抖音个人信息下载、Google Takeout、X 归档一类）的导入插件放 `usuyo-fetch`。本机文件、截图、粘贴文字的收下是宿主自己的功能，不是插件。见 [plugin-protocol §8.4](../plugin-protocol.md#84-三种来源)。
6. **schema 以 plugin-kit 为准。** 插件清单、信封、JSON-RPC 消息的 schema 以 plugin-kit 为准，内核经 Go 模块引用；`pkg/contract` 只留资料库自己的类型，实现插件运行器时迁移（[plugin-protocol §10](../plugin-protocol.md#10-契约测试与-plugin-kit)）。取代 [影响](#影响) 里「以内核的 `pkg/contract` 还是 `plugin-kit` 为准，实现时定」那一句。
7. **插件身份与拦截条件**（改 [§6](#6-分发与校验)）。插件 `id` 等于 `owner/repo`，客户端另按 GitHub 的数字仓库 id 认仓库，改名、转移或旧名被重新注册时停止更新。官方组织的插件缺来源证明、或 GitHub App 的校验没通过就拒装，不再只是显示结果；用户添加的来源（包括 `usuyo-fetch`）在添加时固定签发者，以后签发者变了就停止更新；第三方插件校验没通过时拦下还是只标出，仍待定（[open-decisions P12](../open-decisions.md#7-获取与插件)）。细节见 [plugin-protocol §8.3](../plugin-protocol.md#83-校验与来源证明)。
8. **投递接口。** 浏览器扩展、油猴脚本、手机、Agent 经本机 HTTP 接口或收件文件夹投递；令牌按投递端分别发，`api` 类令牌只交链接，身份由宿主重新算。见 [plugin-protocol §5](../plugin-protocol.md#5-投递接口)。

以上改动带来的设置项变化列在 [plugin-protocol §0.4](../plugin-protocol.md#04-需要同步的文档)，[product-model §8.1](../product-model.md#81-设置清单) 照那张表改。

<a id="待核实"></a>
## 待核实

完整清单见 [05 §7](../research/05-note-apps-and-plugins.md#7-待核实清单)。和本条直接相关的：

- GPL、AGPL 工具作为独立进程被调用并随 usuyo 分发的合规边界，需要法律意见；随包预装的 ffmpeg 用哪种许可证的构建。
- Seatbelt 放行 Metal、VideoToolbox 需要的最少规则；macOS 上 `RLIMIT_AS` 是否生效。
- Linux 桌面会话里 cgroup v2 委派是否普遍可用；GPU 设备最少要 bind 哪些；Ubuntu 限制 userns 时 bubblewrap 的降级与报错。
- Windows AppContainer 对 GPU 与 Media Foundation 的影响；MXC 执行器能否用于生产。
- wazero 有无按指令计数的配额。
- 飞书、语雀是否保留 HTML 注释，决定它们以后能不能读回。
- 「资料库即 vault」时，Obsidian 改属性会重写 YAML、重命名会改写链接，对真相文件的影响。
- 思源对同一 `path` 重复调用 `createDocWithMd` 的实际结果。
- 客户端不登录时能否读到公开仓库的 check run；check run 挂在 release tag 指向的提交上，客户端按 tag 查找是否可靠。
- 组织成员审核的具体流程：谁能把仓库建进组织或转入组织，上架前要过哪些检查。
