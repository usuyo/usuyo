<a id="调研-12--桌面壳与内核语言rust-还是-go"></a>
# 调研 12 — 桌面壳与内核语言：Rust 还是 Go

状态：草案 · 2026-11-04

回答你 11-04 的一句：「然后软件的底层我感觉用rust/go？或许会更适合我们的架构设计？」要你回答的放在待讨论的 [6. 桌面壳](../answers.md#6-桌面壳) 和 [7. 内核语言](../answers.md#7-内核语言)。

<a id="约定"></a>
## 约定

- **查证日期**：2026-11-04。事实来自这一轮的三份笔记：一份替 Go 辩护，一份替 Rust 辩护，一份裁定。说法不一时以裁定为准。你的 GitHub 仓库是我当天用 `gh api` 重查的。
- **选项字母**（下文通用）：

  | 字母 | 内核 | 壳 |
  |---|---|---|
  | A | Go | MyGo（[ADR-001](../decisions/ADR-004-rust-tauri.md) 现状） |
  | A′ | Go | Wails v3 |
  | B | Go，作 Tauri 的 sidecar | Tauri 2 |
  | C | Rust | Tauri 2 |
  | D | Rust | 别的壳：D1 Electron，D2 GPUI 或 Dioxus |
  | E | Go 编排，Rust 库经 cgo 链进来 | 看壳 |

- **推断**是我们的判断，没有直接出处；**待核实**是没查到，或者不构建就确认不了。这一轮不许构建第三方代码，所以编译时间都没量。
- **名字**：正文写「本产品」。代码里的 `xiecun`（`cmd/xiecun`、`.xiecun/index/`）和插件协议里的 `seelu` 都是旧占位名，名字还没定（[待讨论 5](../answers.md#5-名字)）。
- **本篇不改别的文档。** 换语言要动哪些地方，列在 [§6](#6-迁移要改什么)。
- **ADR-001 仍然有效。** 你回答之前，它是 accepted，其他文档照它写。

<a id="0-结论"></a>
## 0. 结论

**2026-11-04 用户定了**：只做桌面，手机以后原生重写，选 A（Go + MyGo），退路 A′（Wails v3）；下面推荐的 C 不采用（「其实就桌面端吧，手机端其实可以考虑native去重写，不用考虑手机上的跨平台，所以go感觉也是够的」，[open-decisions §1.8](../answers.md)）。

1. **先定壳，再定内核。** 插件都是独立进程，所以语言不由插件设计决定。它决定的是壳、索引、文件监听、插件隔离和手机端（[§1](#1-决定语言的到底是什么)）。
2. **壳：推荐 Tauri 2。**
   - MyGo 0.7.x 只有一个维护者，10-28 还发了破坏性变更。它不满足 [04 §6.6](04-tech-options.md#66-推荐与风险画布与壳) 自己定的退出条件（[§2.5](#25-壳的成熟度)）。
   - 备选 Wails v3：10-26 出了稳定版，macOS 和 Linux 上默认走 cgo。
   - Electron 只在要 120fps 或各平台一致的 WebGPU 时才选（[§2.1](#21-迷雾画布)）。
3. **内核：推荐 C，Rust 内核加 Tauri 2。**
   - 壳是 Tauri 2，内核也用 Rust，就只有一套工具链。
   - Rust 眼下就给：FSEvents 递归监听、tantivy 检索、rusqlite 能加载扩展。以后还给：Windows AppContainer 隔离、手机上跑内核。
   - Go 的长处照实写：编译快；MCP 的 go-sdk 和 sigstore-go 更稳。
   - 中文分词不算理由。gse 是纯 Go 的 jieba，Go 内核也能做词典分词。
4. **你不想用 Rust，就选 A′（Go + Wails v3）。** B 只在 A′ 的壳原型失败时作后备。A、D、E 出局（[§4](#4-每个选项)）。
5. **要你回答的只有一句：内核改用 Rust，你愿意吗？**
   - 愿意 → C；不愿意 → A′。
   - 两种回答都会换掉 MyGo。你想留 MyGo，在 [待讨论 6](../answers.md#6-桌面壳) 里说。
6. **退出条件。** 你答「愿意」后，第一件事是搭一个空骨架：Tauri 2 + tantivy + rusqlite + rmcp + tokio，量增量重编。改一行要等超过约 10 秒，就退回 A′。这一条已列进 [TODO](../BUILD.md#5-功能线)（[§5](#5-什么会改变推荐)）。
7. **现在换最便宜。** Go 代码共 794 行，大半是占位（[§6](#6-迁移要改什么)）。
8. **你回答之前，ADR-001（Go + MyGo）仍是 accepted，其他文档照它写。**

<a id="1-决定语言的到底是什么"></a>
## 1. 决定语言的到底是什么

插件、重计算和前端都在内核进程外面，跟内核语言无关。

| 部分 | 跟内核语言有没有关系 | 依据 |
|---|---|---|
| 获取、转换、导出插件 | 无关：独立进程，按 JSON 协议对接，插件作者用什么语言都行 | [ADR-003 §2](../decisions/ADR-003-plugins.md#2-进程与协议) |
| ffmpeg、OCR、ASR、embedding | 无关：sidecar 或插件进程 | [CLAUDE.md](../../CLAUDE.md)「重计算」 |
| 前端 | 无关：只走本机 HTTP 和事件，不调壳生成的绑定 | [ADR-002 和内核的接口](../decisions/ADR-002-frontend-stack.md#和内核的接口) |
| 接口（HTTP、MCP、CLI、插件协议） | 无关：用 JSON Schema、OpenAPI 写下来，换语言不改接口（推断） | [overview §5](../BUILD.md#3-现在做到哪) |
| 壳 | 弱相关：壳和 daemon 本来就是两个进程；但和内核同一种语言，就少一套工具链 | [SPEC §9](../SPEC.md#9-clim0) 的 `daemon` |
| 全文索引、向量 | 相关：进程内的库 | [04 §1](04-tech-options.md#1-搜索索引与向量检索) |
| 文件监听 | 相关：资料库一开就要监听 | [§3](#3-对比表) 表下说明 |
| 插件运行器与隔离 | 相关；但 macOS、Linux 上两种语言都是起外部程序 | [§2.3](#23-插件隔离) |
| 安装校验（sigstore） | 相关 | [插件协议 §8.3](../plugin-protocol.md#83-校验与来源证明) |
| 手机端 | 相关 | [§2.6](#26-手机端) |

所以要比的是这几样：壳、索引、文件监听、隔离、手机端，再加编译循环。

**中文分词不在这张单子上。** [gse](https://github.com/go-ego/gse) v1.3.1（2026-11-01）自述是 Go 写的 jieba，带内嵌词典，`go.mod` 只依赖两个纯 Go 库。Go 内核自己写几十行 bleve tokenizer 包装，就能在进程内做词典分词（[04 §1.1](04-tech-options.md#11-全文索引候选)）。Rust 那边是 [tantivy-jieba](https://crates.io/crates/tantivy-jieba) 0.21.0（2026-10-06）。两边都能做，所以它不决定语言。

**接口设计不等这一问。** 只有两处要等你回答：
- 界面那个端口认哪个 `Origin`，要看用哪个壳（[§7](#7-待核实清单)）；
- 内核作独立 daemon，还是跑在壳的进程里（[§4.4](#44-选项-c)）。

<a id="2-六个问题"></a>
## 2. 六个问题

| 问题 | 答 | 偏向 |
|---|---|---|
| [迷雾画布](#21-迷雾画布)要不要系统 webview 以外的东西 | 不要，瞄准 WebGL2。要 120fps 或一致的 WebGPU，只能用 Chromium | 语言中立。WKWebView 系的壳打平 |
| [模型](#22-模型放哪)放内核里还是外面 | 外面：sidecar 或插件进程 | 中立。想要单个二进制自带小模型时略偏 C |
| [插件怎么隔离](#23-插件隔离) | macOS、Linux 都起 `sandbox-exec`、`bwrap`；Windows 现在只要 Job Object | 现在中立。以后 Windows 要强制隔离时偏 C |
| [调原生库](#24-调原生库)（agent-desktop、wx-cli 一类） | 插件是进程，跟内核无关 | 中立 |
| [壳的成熟度](#25-壳的成熟度) | Tauri 2 最成熟；Wails v3 能用；MyGo 不满足退出条件 | Tauri 2（C 或 B）。A′ 可用。A 出局 |
| [手机端](#26-手机端) | Tauri 2 正式支持；gomobile 自称 experimental | 手机上跑内核时偏 C；只做薄客户端时中立。B 最差 |

<a id="21-迷雾画布"></a>
### 2.1 迷雾画布

- 迷雾是点阵像素场。用 WebGL2 的片元着色器或实例化绘制就能画，不需要 WebGPU（推断）。
- 10-29 定了所有变化走同一个时钟。全部由 `requestAnimationFrame` 驱动，在 60fps 下是一致的（推断）。
- WebKit 主干源码（查于 2026-11-04）确认了三件事：
  - WKWebView 的 rAF 默认贴近 60fps：`PreferPageRenderingUpdatesNear60FPSEnabled` 默认开，只有 visionOS 关（[UnifiedWebPreferences.yaml](https://github.com/WebKit/WebKit/blob/main/Source/WTF/Scripts/Preferences/UnifiedWebPreferences.yaml)）。
  - WebGPU 只在按 macOS 26 构建时默认开（[PlatformEnable.h](https://github.com/WebKit/WebKit/blob/main/Source/WTF/wtf/PlatformEnable.h)）。你这台是 macOS 15.6.1，系统 WebKit 按系统版本构建，所以默认没有（推断）。
  - WebKitGTK 默认不编 WebGPU：[WebKitFeatures.cmake](https://github.com/WebKit/WebKit/blob/main/Source/cmake/WebKitFeatures.cmake) 第 341 行 `ENABLE_WEBGPU` 默认 OFF，GTK 端口没改它。
- 所以 WKWebView 系的壳（MyGo、Wails、Tauri）帧率一样。各自的情况：
  - Tauri：[#13978](https://github.com/tauri-apps/tauri/issues/13978)（只有 60fps）以 not_planned 关闭，维护者让等 CEF 运行时。
  - Wails：有一份解除 60fps 的私有 SPI 补丁，[#6056](https://github.com/wailsapp/wails/issues/6056) 和 [#6060](https://github.com/wailsapp/wails/pull/6060) 都还开着。用私有 API 就上不了 Mac App Store。
  - MyGo：能用 Go 写原生着色器，但画布就要离开 [ADR-002](../decisions/ADR-002-frontend-stack.md) 的 React 前端，原生内容也不能垫在网页下面。
- 要 120fps 或三个平台一致的 WebGPU，只能用 Chromium：[Electron](https://www.npmjs.com/package/electron) 45.2.2（2026-11-03），或 Tauri 还没发布的 CEF 运行时。这跟内核语言无关。

<a id="22-模型放哪"></a>
### 2.2 模型放哪

- CLAUDE.md 和 [ADR-003 §2](../decisions/ADR-003-plugins.md#2-进程与协议) 已经定了：embedding、OCR、ASR、rerank 都是 sidecar 或插件进程。
- 地图按内容相似度排布，所以 embedding 是必需的。但它由预装的官方转换插件产出，插件用什么语言都行。
- Rust 的进程内选项更多：[fastembed](https://crates.io/crates/fastembed) 7.5.0（2026-10-23）、[candle-core](https://crates.io/crates/candle-core) 0.11.1（2026-10-31）、[ort](https://crates.io/crates/ort) 2.0.0-rc.15（2026-10-19，还是 RC）。这些用在 Rust 写的插件里就行，不用换内核。
- 内核只存、查向量：Go 有 modernc 自带的 sqlite-vec（纯 Go），Rust 有 `sqlite-vec` crate。个人资料库的规模下都够（推断）。

<a id="23-插件隔离"></a>
### 2.3 插件隔离

- **macOS、Linux 打平。** 两种语言都是起 `/usr/bin/sandbox-exec` 或 `bwrap`。Codex CLI 也这么做（[seatbelt.rs](https://github.com/openai/codex/blob/main/codex-rs/sandboxing/src/seatbelt.rs) 第 61 行）。Landlock 两边都有官方库：[go-landlock](https://github.com/landlock-lsm/go-landlock) v0.11.0（2026-10-23）、[rust-landlock](https://github.com/landlock-lsm/rust-landlock) 0.4.7（2026-07-27）。
- **App Sandbox** 是签名时给整个应用的 entitlement，跟语言无关。
- **Windows。**
  - [ADR-003 §4](../decisions/ADR-003-plugins.md#4-权限与隔离) 现在只强制 Job Object。Go 的 `x/sys/windows` 就有 `CreateJobObjectW`、`AssignProcessToJobObject`。
  - 以后要 AppContainer：windows-rs 现成（[CreateAppContainerProfile](https://microsoft.github.io/windows-docs-rs/doc/windows/Win32/Security/Isolation/fn.CreateAppContainerProfile.html)）；[zsyscall_windows.go](https://github.com/golang/sys/blob/master/windows/zsyscall_windows.go) 里一处都没有，Go 要自己调 userenv.dll（查于 2026-11-04）。
  - OpenAI 说 Codex CLI 改用 Rust 的理由之一是「Native Security Bindings」（[discussion #1174](https://github.com/openai/codex/discussions/1174)，2025-05-30）。
- B 已经带着 Rust 工具链，以后可以加一个 Rust 写的 launcher 进程。这是进程边界，不算 E。

<a id="24-调原生库"></a>
### 2.4 调原生库

- agent-desktop 按 [插件协议 §2.8](../plugin-protocol.md#28-安装与启动) 打进视觉层插件的资产，按命令行调用，不进内核（[10 §1.1](10-desktop-and-android-agents.md#11-是什么)）。
- wx-cli 一系是 Rust 写的，但没有 C ABI，原仓库 2026-07-15 起被屏蔽（[11 §1](11-subscriptions-compat-and-wx-cli.md#1-wx-cli)）。平台逻辑本来也不能进内核（[CLAUDE.md](../../CLAUDE.md) 原则 1）。
- cgo 的代价（失去一台机器交叉编译、每个平台要 C 工具链）只在内核要链 C 库时才付。Go 还有 [purego](https://github.com/ebitengine/purego)，不开 cgo 也能调 C 库。

<a id="25-壳的成熟度"></a>
### 2.5 壳的成熟度

| | MyGo | Wails v3 | Tauri 2 | Electron |
|---|---|---|---|---|
| 版本 | v0.7.4（11-01），0.x | v3.0.0（10-26）稳定；v3.1.0（11-01） | 2.12.5（10-21）；3.0.0-alpha.5（10-14） | 45.2.2（11-03） |
| 维护 | egoist 708 次提交，`claude` 86 次，人类第二名 14 次 | 团队，36,944★ | 组织，112,476★ | 组织，123,753★ |
| cgo 与交叉编译 | 纯 Go，一台机器出全平台 | macOS、Linux 默认 cgo；不用 cgo 的 purego 后端是实验性的 | 每个平台各自构建 | 每个平台各自构建 |
| 签名与更新 | 有，0.x | 有：各平台签名、NSIS、MSIX、DMG、AppImage，带回滚的应用内更新 | 有；打包工具先签 sidecar 再签整个 .app，然后公证 | 有 |
| 手机端 | 没有（[PR #940](https://github.com/egoist/mygo/pull/940) 开着） | 实验性，不在稳定承诺里 | 正式支持 | 没有 |

来源（查于 2026-11-04）：[MyGo 发布页](https://github.com/egoist/mygo/releases)与[贡献者](https://github.com/egoist/mygo/graphs/contributors)；[Wails v3.0.0 发布说明](https://github.com/wailsapp/wails/releases/tag/v3.0.0)，cgo 见 [application_darwin.go](https://github.com/wailsapp/wails/blob/master/v3/pkg/application/application_darwin.go) 第 1 行的构建标签和第 245 行的 `import "C"`；[Tauri 发布页](https://github.com/tauri-apps/tauri/releases)，sidecar 签名见 [tauri-bundler 的 app.rs](https://github.com/tauri-apps/tauri/blob/dev/crates/tauri-bundler/src/bundle/macos/app.rs) 第 98–140 行；[Electron](https://www.npmjs.com/package/electron)。

拿 [04 §6.6](04-tech-options.md#66-推荐与风险画布与壳) 给 MyGo 定的三条退出条件对照：

| 条件 | 今天 |
|---|---|
| (a) 连续 60 天无破坏性变更，或进入 1.x | 没满足：v0.7.2（10-28）还有破坏性变更 |
| (b) Windows、Linux 各有一个第三方 webview 应用公开发布 | 待核实 |
| (c) 作者以外至少一名持续贡献者 | 没满足：第二名是 `claude`，人类第二名 14 次提交 |

04 的规则是开工（原 M3）前三条都要满足；现在两条没满足，所以开工前就该换壳。

一条两边都要守的规则：Wails v3.0.0 的发布说明写明 GHSA-2p5f-2gx8-mh8m 只修了一部分，Linux 上同源 iframe 还能碰到原生消息通道。这是浏览器的同源行为，不是 Wails 独有，不拿来比壳。抓来的 HTML 只放进不带 `allow-same-origin` 的 sandbox iframe。

<a id="26-手机端"></a>
### 2.6 手机端

- Tauri 2 正式支持 iOS、Android；[UniFFI](https://crates.io/crates/uniffi) 0.32.2（2026-09-23）能给 Rust 库生成 Swift、Kotlin 绑定。
- [gomobile](https://github.com/golang/mobile) 没有任何 tag，最新只有 2026-10-11 的伪版本，README 自称 experimental。Wails v3 的手机端是实验性的。MyGo 没有。
- [App Store 审核指南](https://developer.apple.com/app-store/review/guidelines/) 2.5.2 不许下载并执行代码。所以「下载插件、起进程」这一套在 iOS 上本来就不成立，B 的 Go sidecar 也起不来。
- 手机只做薄客户端（浏览、收下、交给电脑）时，跟语言无关。要在手机上读写资料库、建索引、搜索时，C 最顺：同一个 Rust crate，同一份前端。

<a id="3-对比表"></a>
## 3. 对比表

✓ 好　△ 能用，有代价　✗ 差或缺　— 不适用

| 维度 | A Go + MyGo | A′ Go + Wails v3 | B Go 内核 + Tauri 2 壳 | C Rust + Tauri 2 | D Rust + 别的壳 | E Rust 库经 cgo 进 Go |
|---|---|---|---|---|---|---|
| 壳的成熟度 | ✗ 0.7.x，单人维护，10-28 还有破坏性变更 | △ 稳定版才 9 天，团队维护 | ✓ | ✓ | D1 ✓；D2 ✗ 都在 1.0 前 | 看壳 |
| 迷雾画布 | △ WebGL2，60fps | △ WebGL2，60fps；补丁没合并 | △ WebGL2，60fps；以后有 CEF | △ 同 B | D1 ✓ 120fps，WebGPU 一致 | 看壳 |
| 中文全文检索 | △ bleve + gse | △ 同 A | △ 同 A | ✓ tantivy + jieba，或 FTS5 + libsimple | ✓ 同 C | ✗ tantivy 经 cgo |
| 文件监听 | ✓ MyGo 自带 FSEvents | △ fsnotify 用 kqueue，要自己补 FSEvents | △ 同 A′ | ✓ notify | ✓ | — |
| MCP SDK | ✓ go-sdk v1 | ✓ | ✓ | △ rmcp 大版本换得勤 | △ | ✓ |
| 来源证明校验 | ✓ sigstore-go | ✓ | ✓ | △ sigstore-verify 在 1.0 前 | △ | ✓ |
| Windows 强制隔离（以后） | △ 要自己调 API | △ | △ 可加 Rust launcher | ✓ windows-rs | ✓ | △ |
| 进程内模型（可选） | ✗ 要 cgo | ✗ | ✗ | ✓ | ✓ | △ 经 cgo |
| 手机端 | ✗ | △ 实验性 | ✗ iOS 起不了 sidecar | ✓ | D1 ✗ | ✗ |
| 打包、签名、更新 | △ 有，0.x | ✓ | ✓ sidecar 也签 | ✓ | ✓ | ✗ 两套 |
| 一台机器出全平台 | ✓ | △ 默认 cgo | △ 各平台构建 | △ 同 B | △ | ✗ |
| 构建等待 | ✓ | ✓ | ✓ 内核快，壳少改 | △ 没量过，最大风险 | △ | ✗ 最慢 |
| 工具链 | Go + TS | Go + C + TS | Go + Rust + TS | Rust + TS | Rust + Node | Go + Rust + C |
| 你的经验 | △ | △ | △ | ✓ | ✓ | ✗ |
| 迁移成本 | ✓ 零 | ✓ 小 | △ 小到中 | △ 中 | ✗ 大 | ✗ 大 |
| **总评** | 出局 | 不要 Rust 时选它 | 被 C 压过，作 A′ 的后备 | **推荐** | 出局；定了 Electron 时改比 Go + Electron | 不做 |

表里几格的依据（都查于 2026-11-04）：

- **文件监听。** [fsnotify](https://github.com/fsnotify/fsnotify) v1.10.1（2026-05-04）在 macOS 用 kqueue。README 写每个被监听的文件都要开一个 fd，不能递归；FSEvents 的 [#11](https://github.com/fsnotify/fsnotify/issues/11) 还开着。本机 `kern.maxfilesperproc` 是 61440。资料库有上万个条目、每个几份文件，用 kqueue 会顶到 fd 上限（推断）。Rust 的 [notify](https://crates.io/crates/notify) 8.2.1（2026-10-27）在 macOS 默认用 FSEvents，能递归。[ADR-003 §7](../decisions/ADR-003-plugins.md#7-导出与读回) 要求资料库改动后重新读取，所以 daemon 一上线就要监听。
- **全文检索。** Go：[bleve](https://github.com/blevesearch/bleve) v2.6.2（2026-11-03）+ gse，要自写 tokenizer 包装。Rust：[tantivy](https://crates.io/crates/tantivy) 0.27.0（2026-10-17）+ tantivy-jieba；或 [rusqlite](https://crates.io/crates/rusqlite) 0.41.1（2026-10-28）加载 [libsimple](https://crates.io/crates/libsimple) 0.10.0（2026-10-25），做中文加拼音的 FTS5。modernc 的纯 Go SQLite 不能加载 C 扩展。没有同代的公平基准。
- **MCP SDK。** [go-sdk](https://github.com/modelcontextprotocol/go-sdk/releases) v1.8.0（2026-09-14），一直在 v1.x。[rmcp](https://crates.io/crates/rmcp/versions) 2026 年发了 1.0.0（03-03）、2.0.0（06-29）、3.0.0（07-28），现在 3.8.0（10-26）。我们用到的接口不多，迁移面小（推断）。
- **来源证明校验。** [sigstore-go](https://github.com/sigstore/sigstore-go/releases) v1.4.0（2026-10-29）。[sigstore-verify](https://crates.io/crates/sigstore-verify) 0.15.1（2026-10-23）出自 sigstore 官方组织，约 200 万次下载，mise、rattler 在用，还没到 1.0。老的 `sigstore` crate 才自称 experimental。
- **构建等待。** [Elanis 的基准](https://github.com/Elanis/web-to-desktop-framework-comparison)里空应用干净构建：Wails 约 6–25 秒，Tauri 约 134–237 秒（测的 Wails 大概率是 v2，待核实）。[Rust 2025 编译性能调查](https://blog.rust-lang.org/2025/09/10/rust-compiler-performance-survey-2025-results/)：55% 的人改一处要等 10 秒以上。Agent 每轮都要编译、跑测试，等待会乘以轮数。
- **你的经验。** 你在 GitHub 上公开的 [SteamCloudFileManager](https://github.com/Fldicoahkiin/SteamCloudFileManager) 是 Rust + egui 写的跨平台应用，241★，2025-09-24 建，2026-10-24 还在推送（查于 2026-11-04）。

<a id="4-每个选项"></a>
## 4. 每个选项

<a id="41-选项-a"></a>
### 4.1 选项 A

**Go 内核加 MyGo 壳，ADR-001 现状。出局。**

- 好处：全纯 Go，一台机器出全平台；MyGo 自带 FSEvents 监听和 Go 写的原生着色器；代码和文档都不用改。
- 坏处：
  - 0.x，单人维护，10-28 还有破坏性变更；04 的三条退出条件有两条没满足（[§2.5](#25-壳的成熟度)）。
  - 网页里卡在 60fps，没有现成的解法。
  - 没有手机端。
- 迁移成本：代码零改。文档只改版本号：MyGo 0.7.4 要求 Go 1.27.3+，而 `go.mod` 写的是 1.26.0；再补上退出条件。
- 为什么出局：ADR-002 定的选型原则是每层选最新的稳定版、最合需求的方案（[ADR-002 背景](../decisions/ADR-002-frontend-stack.md#背景)）。0.x 加单人维护过不了这一条。

<a id="42-选项-a"></a>
### 4.2 选项 A′

**Go 内核加 Wails v3 壳。你不要 Rust 时选它。**

- 好处：
  - 只有一套语言；内核一行不改。
  - Wails v3 是稳定版，团队维护；签名、安装包、带回滚的应用内更新都齐。
  - 编译快，Agent 的改、编、测循环短。
  - go-sdk v1、sigstore-go v1.4.0 都稳。
- 坏处：
  - 稳定版才发 9 天。
  - macOS、Linux 默认走 cgo，不能一台机器出全平台；purego 后端是实验性的。
  - 文件监听要自己补：从 MyGo 抄 FSEvents 实现（MIT），或者自己写。
  - 以后 Windows 要 AppContainer，得自己调系统 API。
  - 手机端实验性；60fps 的补丁还没合并。
  - 分词用 gse，gse 也是单人维护（vcaesar 758 次提交，第二名 3 次）。分词放单独的字段，出问题可以退回 bigram（[04 §1.4](04-tech-options.md#14-推荐与风险搜索)）。
- 迁移成本：小。内核零改，壳还没写。改 ADR-001 的壳那一行、[04 §6.4](04-tech-options.md#64-桌面壳mygo-与回退路径) 的结论、ADR-002 的 `shell/` 说明（[§6](#6-迁移要改什么)）。

<a id="43-选项-b"></a>
### 4.3 选项 B

**Go daemon 作 Tauri 2 的 sidecar，Rust 只写一层薄壳。被 C 压过，作 A′ 的后备。**

- 先例：[Clash Verge Rev](https://github.com/clash-verge-rev/clash-verge-rev) 就是 Tauri 2 壳加 Go 写的 mihomo sidecar。
- 好处：拿到 Tauri 壳的成熟度，打包工具会签 sidecar；内核不动，编译快。
- 坏处：
  - 两套工具链、两套 CI、两套依赖审计。
  - 换来的只有编译快和两个更稳的 Go 库，而这两个库在 Rust 里都有能用的对应物（[§3](#3-对比表) 表下说明）。
  - 文件监听、Windows 隔离的问题和 A′ 一样。
  - iOS 起不了 sidecar，手机端最差。
- 迁移成本：小到中。内核零改，新建一个 Tauri 壳工程，Go 二进制作 `externalBin`。
- 什么时候用：你选了 A′，而 Wails v3 在迷雾原型里出问题，例如 60fps 补丁迟迟不合并、手机端成了刚需、Linux 上的 webview 有问题。

<a id="44-选项-c"></a>
### 4.4 选项 C

**Rust 写内核、CLI、MCP、daemon，加 Tauri 2 壳，前端照 ADR-002 不变。推荐。**

- 进程怎么分：桌面上 daemon 照 [SPEC §9](../SPEC.md#9-clim0) 作独立进程，由 Tauri 当 `externalBin` 起它（推断）。「同一个 crate 在进程内跑」只指手机端。这样 ADR-002 和 HTTP 接口都不用改。Tauri 的 `invoke` 也是壳生成的绑定，照样不用。
- 好处：
  - 和壳同一套工具链。
  - 眼下就有：FSEvents 递归监听；tantivy 检索；rusqlite 能加载 libsimple，做中文加拼音的 FTS5。
  - 以后不用换语言：Windows AppContainer、手机上跑内核、单个二进制自带小模型。
  - ADR-001 当初放弃 Rust 的理由是「单人速度不占优」。现在代码主要由 Agent 写，你也发过 Rust + egui 的跨平台应用（[§3](#3-对比表) 表下说明），「人写 Rust 慢」这一半不成立。剩下的「等编译」写成退出条件。
- 坏处，照实写：
  - 编译循环最慢，而且没量过。这是最大的风险。
  - rmcp 大版本换得勤；sigstore-verify 还没到 1.0。
  - Tauri 3 已在 alpha，上 2.x 以后还要再迁一次。
  - 依赖树大，审计工作多。
  - 每个平台各自构建。macOS 签名和公证本来就要 macOS 机器，CI 本来就得按平台跑。
- 迁移成本：中。
  - 约 800 行 Go 类型与 CLI 骨架，用 serde + clap 重写，大约 Agent 一次会话（推断）。
  - `pkg/contract` 的 JSON Schema 和 `testdata/` 原样保留，契约测试照旧跑。
  - plugin-kit 的 Go SDK 不动，插件作者照样能用 Go 写插件。
  - 文档的改动见 [§6](#6-迁移要改什么)。这次正在整理文档，一起改最省；以后每多写一份，成本就涨一点。

<a id="45-选项-d"></a>
### 4.5 选项 D

**Rust 内核加别的壳。出局。** 为迷雾定了 Electron 时，更该比的是 Go 内核加 Electron（[§5](#5-什么会改变推荐)）。

- D1 Electron：三个平台渲染一致，120fps，WebGPU 一致。体积和内存最大（Elanis：约 268MB / 370MB），没有手机端。
- D2 [GPUI](https://crates.io/crates/gpui) 0.2.2 或 [Dioxus](https://crates.io/crates/dioxus) 0.7.10：都在 1.0 前；迷雾能原生画，但要丢掉 ADR-002 定的 React 前端。
- 迁移成本：大。C 的成本，加上 Electron 打包；D2 还要重写 ADR-002。

<a id="46-选项-e"></a>
### 4.6 选项 E

**Rust 写索引、模型、沙箱库，Go 做编排，进程内经 cgo 连起来。不做。**

- 每个接缝都是 cgo，Go 不开 cgo 的好处没了。
- 两套运行时（Go 调度器和 tokio）、两套内存管理，panic 不能跨 FFI 传。
- 工具链是 Go + Rust + C，构建最慢，签名和发版两套。
- 改成进程间连接，它就不是 E，而是「A′ 加几个 Rust 写的插件」，插件协议本来就允许。

<a id="5-什么会改变推荐"></a>
## 5. 什么会改变推荐

| 情况 | 改成什么 | 为什么 |
|---|---|---|
| 你不愿意让内核用 Rust | A′，不选 B | A′ 只有一套工具链，Wails v3 也已是稳定版。B 留作 A′ 的后备 |
| 骨架的编译循环太慢 | A′ | 你同意 C 以后先搭空骨架：Tauri 2 + tantivy + rusqlite + rmcp + tokio。在本机改一行内核代码，量 `cargo build` 的增量重编，顺手记一轮 `cargo test` 的时间。增量重编超过约 10 秒就退回 A′。阈值是推断，你可以改（[TODO](../BUILD.md#5-功能线)） |
| 为迷雾选了 Electron（要 120fps 或一致的 WebGPU） | Go 内核 + Electron | Tauri 生态这条理由没了，只按内核本身比，编译快的分量变重。这是 ADR-001 原来列的回退 |
| sigstore-verify 做不到 [插件协议 §8.3](../plugin-protocol.md#83-校验与来源证明) 的要求（按 TUF 更新信任根、离线验证 GitHub attestation bundle、按工作流核对签发者） | C 里带一个 Go 写的小校验进程，或重新比 C 和 A′ | 校验是装插件的门，不能打折扣 |
| 手机端确定只做薄客户端，Windows 也确定只做 Job Object | 单凭这一项不改；和编译偏慢叠在一起时改 A′ | C「以后」的两条好处没了，「眼下」的几条还在 |

**不会改变推荐的：**

- 中文分词：gse 已经能做（[§1](#1-决定语言的到底是什么)）。
- sidecar 签名：Tauri 的打包工具会签（[§2.5](#25-壳的成熟度)）。
- 公开的 Agent 编码基准：[SWE-bench Multilingual](https://www.swebench.com/multilingual.html) 每种语言的题目不同，各语言的解决率不能直接比，所以不拿来作理由。

<a id="6-迁移要改什么"></a>
## 6. 迁移要改什么

**ADR-001 在你回答前一直有效。** 下表只列要动的地方，本篇一处都没改。代码现状：`find . -name '*.go' | xargs wc -l` 共 794 行；写死 Go 栈的文档 17 份（`grep -rl` 计数，2026-11-04）。

| 位置 | 现在写的 | 选 C 要改 | 选 A′ 要改 |
|---|---|---|---|
| [ADR-001](../decisions/ADR-004-rust-tauri.md) | Go + MyGo；「MyGo 仍在 0.5.x」；放弃 Rust + Tauri 是因为「单人速度不占优」 | 写新 ADR 取代它。理由换成「编译循环对壳和库的成熟度」这笔账 | 改壳那一行，更新版本号，补壳的退出条件 |
| [ADR-002](../decisions/ADR-002-frontend-stack.md#和内核的接口) | 前端只走本机接口；壳能力收在 `shell/` | `shell/` 的说明按 Tauri 改；前端不变 | `shell/` 的说明按 Wails 改 |
| [CLAUDE.md](../../CLAUDE.md) 技术栈、代码约定、常用命令 | Go 1.23+、MyGo、bleve；`cmd/`、`internal/`、`pkg/`；`go test`、`make` | Rust + Tauri 2、tantivy 或 FTS5；cargo 工作区；`cargo test` | 壳改 Wails v3，Go 版本号更新 |
| [04 §6.4](04-tech-options.md#64-桌面壳mygo-与回退路径) | MyGo ★★；「Wails v3 仍 beta」；Tauri ★ | 正文不改，加一行注：已过时，见本篇 | 同左 |
| `go.mod` | `go 1.26.0`，三个直接依赖 | 换成 `Cargo.toml` | Go 版本按 Wails v3 的要求改 |
| `cmd/`、`internal/`、`pkg/` | `cmd/xiecun` 11 个文件；`internal/` 下 7 个占位文件；`pkg/contract` 4 个 `.go` 加 `schema/`、`testdata/` | 用 serde + clap 重写；`schema/`、`testdata/` 原样保留 | 不改 |
| [SPEC §1](../SPEC.md#1-仓库布局monorepo) 仓库布局 | Go 标准布局 | 按 cargo 工作区改 | 不改 |
| [插件协议 §8.3](../plugin-protocol.md#83-校验与来源证明) | 点名 sigstore-go | 改成 sigstore-verify，或带 Go 校验进程 | 不改 |
| feature 001 | bleve 方案 | 改索引方案 | 不改 |
| CI | Go | cargo + pnpm，按平台跑矩阵 | 加 cgo 的各平台构建 |
| 调研 01–10 | 历史记录 | 正文不改，只加一行「语言已改，见新 ADR」 | 只在提到 MyGo 的地方加同样一行 |

和语言一起定下的两件接口上的事：
- 界面端口放行哪个 `Origin`（看壳，[§7](#7-待核实清单)）；
- daemon 作独立进程（[§4.4](#44-选项-c) 的推断）。

<a id="7-待核实清单"></a>
## 7. 待核实清单

1. **本机 Rust 骨架的编译循环。** Tauri 2 + tantivy + rusqlite + rmcp + tokio 的增量重编和一轮测试各要多久。这一轮不许构建，没量。这是 C 的第一条退出条件（[§5](#5-什么会改变推荐)）。
2. Tauri 的应用代码能否拿到 WKWebView 句柄、调私有 SPI 解除 60fps（推断能，没找到先例）。
3. WebView2 从哪个运行时版本起默认开 WebGPU。
4. sigstore-verify 能否按 [插件协议 §8.3](../plugin-protocol.md#83-校验与来源证明) 离线验证 GitHub attestation bundle。
5. launchd 启动的 GUI 应用默认软 fd 上限是多少；fsnotify 在多大的资料库上会顶到。
6. bleve + gse 对 tantivy + jieba：在真实中英混排样本上的召回和延迟（[04 §1.4](04-tech-options.md#14-推荐与风险搜索) 要的实测）。
7. Tauri 3 正式版的日期；CEF 运行时在 Linux 上的问题（[#16189](https://github.com/tauri-apps/tauri/issues/16189)、[#16251](https://github.com/tauri-apps/tauri/issues/16251)）。
8. Elanis 基准里的 Wails 是 v2 还是 v3（影响构建时间和包体那几格）。
9. MyGo 退出条件 (b)：Windows、Linux 上有没有第三方正式发布的 webview 应用。A 已出局，只作记录。
10. Tauri 默认配置下，同源 iframe 能否调到 IPC。
11. libsimple 在 Windows 上能否编译；tantivy-jieba 的词典让二进制大多少。
12. Tauri、Wails 的 webview 发请求时带什么 `Origin`。Tauri 在 macOS、Linux 上是 `tauri://localhost`、Windows 上是 `http://tauri.localhost`，这是凭记忆写的，没查。
13. Rust 侧有没有支持 OpenAPI 3.2 的生成工具。
14. 用现在的模型从零写桌面 daemon，Go 和 Rust 差多少：没有公开数据。
