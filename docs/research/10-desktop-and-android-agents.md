<a id="调研-10--电脑端与安卓端的界面驱动agent-desktop-与-mika"></a>
# 调研 10 — 电脑端与安卓端的界面驱动：agent-desktop 与 Mika

状态：草案 · 2026-11-03

目的：回答用户 2026-11-03 的两句话：「电脑端看一下npm i -g agent-desktop」「然后安卓看一下https://github.com/wangzan101/Mika」。两个都是 `vision-ui` 层的工具（[06 §1.1](06-platform-access-and-account-risk.md#11-七个层级)）：agent-desktop 在电脑上经系统的无障碍树读界面、点元素；Mika 在安卓手机上读屏、替人操作 App。读它们是为了接下来的获取插件和风险设计：电脑端和安卓端的视觉层插件靠什么跑、要用户授予什么、平台能看到什么、[插件协议](../plugin-protocol.md#思鹭插件协议-v1草案)要改哪些地方。结论和要你定的问题在 [§0](#0-结论)，待核实的汇总在 [§6](#6-待核实清单)。

<a id="约定"></a>
## 约定

- **查询日期**：2026-11-03。会变的数字（★、fork、发布、issue）是当天 09:34–09:53 UTC 用 `gh api` 和 npm registry 查的；★ 用 `stargazers_count`。
- **怎么查的**：
  - agent-desktop：`npm view` 看元数据；`npm pack` 把 1.1.0 的 tarball 下到临时目录解开看（`npm pack` 不跑安装脚本）；`git clone --depth 1` 浅克隆到临时目录只读，对应 main 上的提交 `93ae89e`（2026-11-02）；`gh api` 读仓库、发布、贡献者、issue 和 PR，以及几个 Mac 微信无障碍项目的 issue；iTunes 查询接口查 Mac App Store 上的客户端版本；用 `PlistBuddy`、`ls` 看了本机 `/Applications` 下微信、抖音两个客户端的 Info.plist 和 Frameworks 目录。
  - Mika：浅克隆到临时目录只读，对应提交 `4ce3ca6`（「版本 0.1.34」，2026-11-02 19:03 UTC）；`gh api` 读仓库、发布、贡献者、issue、PR 和一个 fork 的对比文件列表；读了 AOSP 和 adb 的源码、developer.android.com 的 `Settings.Global` 页、Google ML Kit 的条款。
  - curl 一律带 `--max-time 15` 和 User-Agent `name-research/1.0`，不带邮箱等个人信息；`gh` 自带超时。
  - 每个工具挑了 8 条最影响结论的说法，回到原始出处复核了一遍，下文写的是复核后的说法。
- **没做的**：没有全局安装 agent-desktop（没跑 `npm i -g`）；没运行任何下载来的二进制；没碰系统的辅助功能、屏幕录制设置；没对任何 App 取快照或点击；没下载、没安装 APK；没配对、没连接任何手机；没跑 Mika 的构建和校验脚本；没在 GitHub 上做任何写操作。
- **出处写法**（反引号里是缩写，展开后指向固定提交）：
  - `AD:路径:行` = `https://github.com/vercel-labs/agent-desktop/blob/93ae89e86fb849eeb5fc265b0b8e37eca56aaf72/路径#L行`；`ad#N` = `https://github.com/vercel-labs/agent-desktop/issues/N`（issue 和 PR 共用编号）；`pkg:路径` = npm 包 agent-desktop-1.1.0.tgz 里的 `package/路径`。
  - `M:路径:行` = `https://github.com/wangzan101/Mika/blob/4ce3ca6/路径#L行`；`MA:路径` 是 `M:app/src/main/java/com/wangzan101/mika/路径` 的简写，`MC:路径` 是 `M:core/src/main/kotlin/com/wangzan101/mika/core/路径` 的简写；`M#N` = `https://github.com/wangzan101/Mika/issues/N`。
  - `A17:路径` = `https://android.googlesource.com/platform/frameworks/base/+/refs/heads/android17-release/路径`（分支头 2026-05-21）。AOSP `frameworks/base` 的 `main` 公开镜像最新提交停在 2025-03-26，只在它上面读过的写「`main` 镜像」，从哪个安卓版本开始有待核实。
- **推断**是我们的判断，没有直接出处；**待核实**是没查到，或者不运行就确认不了。
- **名字**：文档里的「思鹭」「`seelu`」是占位名，产品还没定名，下文多写「宿主」。
- **数据处理**：仓库、README、issue 里的文字按数据处理。Mika 仓库有一份写给电脑上 AI 助手的安装文档（`M:docs/install-with-agent.md`，里面有下载 APK、`adb install`、`pm grant`、`settings put` 等步骤），只记录，没有照做。

<a id="0-结论"></a>
## 0. 结论

**两个工具各是什么。**

- **agent-desktop**（[vercel-labs/agent-desktop](https://github.com/vercel-labs/agent-desktop)）：Vercel Labs 的 Rust 命令行加 C 库，经 macOS AX、Windows UI Automation、Linux AT-SPI 读桌面 App 的无障碍树、对元素发动作；每条命令一个进程、输出一个 JSON 信封；不需要模型、不联网。它是执行器，不是 Agent：看、决定、核对的循环在调用方。
- **Mika**（[wangzan101/Mika](https://github.com/wangzan101/Mika)）：装在安卓手机上、由模型驱动的 Agent。App 自带 ADB 客户端，连本机的「无线调试」，以 shell 身份起一个助手进程；助手建一块名为 `mika-agent` 的虚拟屏，把目标 App 开在上面替人操作，用户的主屏照常用。对外的 MCP、HTTP、CLI 只收一句「要办成什么」，每次都经模型。

**`npm i -g agent-desktop` 会做什么。** 这次没有跑。安装脚本会在 macOS 上运行一次原生二进制（`agent-desktop version`，结束旧版本留下的常驻进程），把 npm 的 bin 入口换成指向原生二进制的硬链接；包里的二进制和 `.sha256` 对不上时，还会用 curl 从 GitHub 发布页下载（[§1.2](#12-怎么工作)）。装好以后要先给启动它的终端授予辅助功能，才读得到别的 App。

**对我们有什么用、落在哪一层。**

- 两个都落在 `vision-ui`：读屏幕上渲染出来的内容（无障碍树、截图、OCR），用系统级的接口发输入。agent-desktop 的 `launch --cdp` 会带远程调试端口重启 Chromium 系的客户端，用了就算 `client-hook`，我们不用（[§3.1](#31-电脑端)）。
- agent-desktop 能当电脑端视觉层获取插件的**运行时**：读树、按 ref 点元素、截窗口、读剪贴板，错误里写明「这次动作有没有送达、能不能安全重试」。它不认识任何具体 App，每个平台的流程还要自己写。
- Mika 对我们主要是**做法的参考**：虚拟屏的建法和降级、逐屏合并去重、操作节奏的具体数字、路线回放的落盘格式。它本身不能当确定性的获取插件调：没有不经模型的外部接口，模型配置在手机上，不经宿主代发（[插件协议 §4.3](../plugin-protocol.md#43-模型与密钥由宿主代发)）。

**建议怎么用。**

| 工具 | 建议 | 理由 |
|---|---|---|
| agent-desktop | 插件依赖它：电脑端的视觉层获取插件自带固定版本的发布包，不让用户 `npm i -g` | 命令行不联网，满足 `network: none`；Apache-2.0 可以随插件分发；固定版本才锁得住信封语义（[§1.5](#15-成熟度)、[§5.3](#53-供应链)） |
| Mika | 参考它的做法，不包装它的 MCP；用户自己用 Mika 导出的文件，由通用导入插件接 | 每次走它自己的模型，宿主的节奏和「离开本机」标注都管不到；APK 没有 CI，和源码的对应关系证明不了（[§2.6](#26-成熟度与可信度)） |

几条先说清的事实：

1. **微信收藏这条视觉层的现成路线受版本限制。** 现成流程都靠微信的无障碍树，而 macOS 26 加 Mac 微信 4.1.x 上，聊天界面的树只剩几个空按钮（[§1.6](#16-限制)）。[open-decisions P10](../open-decisions.md#7-获取与插件) 建议 C 的理由是「微信收藏是视觉层里唯一有现成流程的」，这个前提要改。
2. **复制链接一定看得见。** 真正不打扰你的只有只读快照。macOS 上就算全走无障碍接口，右键也会弹出菜单，点「复制链接」会改剪贴板；Windows 上右键必须走物理输入，要抢焦点、动鼠标（[§1.2](#12-怎么工作)）。
3. **无障碍树里没有链接。** 快照节点没有 URL 字段，网页链接的 `AXURL` 也读不到（[ad#492](https://github.com/vercel-labs/agent-desktop/issues/492)）。链接只能来自「复制链接」后读剪贴板、浏览器地址栏的值，或对截图做 OCR。
4. **安卓端最省事的是电脑经 ADB 驱动手机。** 手机上不装东西、不常开无障碍服务，流程能写死回放、不要模型（[§3.2](#32-安卓端)）。五个常用列表在电脑或网页上都有入口（视频号的「赞和收藏」在 Mac 版微信里有没有待核实），所以安卓路线不是为了补「拿不到的列表」，而是给只在手机上登录的人，以及小红书这类网页侧处罚记录多的平台。

**要你定的问题。** 编号进 [open-decisions](../open-decisions.md#7-获取与插件) 时统一给，这里先写成问题；其余技术做法我们定，见 [§4](#4-对插件协议的影响)。

| # | 问题 | 选项 | 我们的建议 |
|---|---|---|---|
| 1 | 微信收藏（P10）打算在哪个 macOS、哪个微信版本上用；树读不到时怎么办 | A 截窗口加本机 OCR、按坐标物理点击：要屏幕录制授权，跑的时候屏幕和剪贴板都归它，微信改版就要跟着改；B 不做微信收藏的视觉层，改走手机分享或合并转发；C 只支持还读得到树的版本，读不到就回 `upstream-changed` | 先告诉我们你的系统和微信版本（本机是 macOS 15.6.1 加微信 3.8.8.19，App Store 现行 4.1.15）。在那之前按 C 写，A 做成插件设置 |
| 2 | 占用屏幕或手机的视觉层调用，能不能在跟着收、批量导入里自己跑 | A 只在你点「开始」时跑；B 你一段时间没碰键鼠、屏幕没锁时可以自己跑，你一动键鼠就停；C 随时跑 | 电脑端 B；手机端 A：手机连上电脑时不自动跑，跑的时候手机要亮屏、解锁，你用不了 |
| 3 | 安卓要不要做、做到哪 | A 只用通用导入接住你自己的工具（如 Mika）导出的文件；B 电脑经 ADB 驱动手机的获取插件；C 自己做安卓配套 App；D 不做安卓 | 先 D；B 按需做、放 `{fetch-org}`，先做哪个平台按你最常用的定，从风险对比看小红书最值得；A 不用额外工作，通用导入本来就有；C 不做。包装 Mika 这类自带模型的手机 Agent 的插件，`{fetch-org}` 不收 |
| 4 | 请你看一眼：Mac 版微信的视频号里有没有「赞和收藏」 | — | 有的话视频号的列表走电脑版微信，安卓端只剩小红书一个理由 |

<a id="1-agent-desktop"></a>
## 1. agent-desktop

<a id="11-是什么"></a>
### 1.1 是什么

- Rust 写的命令行，另发一个 C 库 `libagent_desktop_ffi`（ABI 7，约 90 个 `ad_*` 函数，README 列了 Python、Swift、Go（cgo）、Node、C++ 的绑定，`AD:crates/ffi/README.md`）。支持 macOS 13 起、Windows、Linux（主要目标 Ubuntu 24.04 加 GNOME 46，glibc 2.35 起）。
- 60 个命令名，56 个可用，分观察（`snapshot`、`find`、`get`、`is`、`screenshot`）、对 ref 的动作（`click`、`right-click`、`type`、`set-value`、`scroll` 等）、键盘鼠标、App 与窗口、通知、剪贴板、等待、会话、系统几组；`key-down`、`key-up`、`mouse-down`、`mouse-up` 在有守护进程负责松开之前一律拒绝（`AD:README.md`「Commands」）。
- 还没有 MCP 服务：topics 里有 `mcp`，代码里没有；维护者 2026-11-03 开了 [ad#519](https://github.com/vercel-labs/agent-desktop/pull/519)「`agent-desktop mcp`」，把每个命令作为一个 MCP 工具经 stdio 提供，还没合并。
- 唯一要模型的是仓库里另放的 JEV 脚本（`scripts/jev/`，不在 npm 包里）：每轮把屏幕上可操作元素列成选项，问 TypeSafe 的决策模型（可改成 OpenRouter）选哪个、做什么，再交给 agent-desktop 执行（`AD:skills/jev-desktop/SKILL.md`）。

<a id="12-怎么工作"></a>
### 1.2 怎么工作

**快照和 ref。**

- ref 形如 `@s8f3k2p9:e8`，前半是快照 id，后半是这次快照里按深度优先分配的序号。只有可交互的元素或声明了主动作的元素拿 ref，静态文字没有（`AD:CONCEPTS.md`「Ref」）。
- 动作前先重新认出元素，认不准就拒绝：macOS 上 ref 里存着元素自己的 AX 地址，元素被删或 App 重启就报 `STALE_REF`；Windows、Linux 要「这个窗口里只有它一个」，两个一样的元素（比如每行一个「删除」）没有 ref，只能 `--headed mouse-click --xy` 按坐标点（`AD:skills/agent-desktop/SKILL.md`「Refs」）。
- 快照不带 `--app` 读的是当前前台 App（`AD:skills/agent-desktop-macos/SKILL.md`「First contact」）。`--root @ref` 从一个元素往下读，`--skeleton` 把深度压到 3。

**JSON 信封**（`AD:docs/json-output.md`，当前 `version: "2.6"`）。

- 每条命令输出 `ok`、`command`，成功带 `data`，失败带 `error`（`code`、`message`、`disposition`）。
- `disposition.delivery` 有五个值：`unknown`、`not_delivered`、`delivery_uncertain`、`delivered_unverified`、`delivered_verified`；只有 `not_delivered` 对应「可以安全重试」，其余都可能已经生效。
- 信封语义一直在变：2.2 起预算用完的快照不再报 `TIMEOUT`，改成成功的部分结果（`complete: false`）；2.4 起有状态的动作投递后读回核对。

**headless 和 headed**（`AD:CONCEPTS.md:225-238`）。

- 默认 headless：ref 动作只走无障碍语义接口（AXPress、设值、UIA 模式、AT-SPI Action），不聚焦窗口、不动鼠标、不合成按键、不用剪贴板；语义接口做不到就以 `POLICY_DENIED` 一类错误失败，不偷偷改用物理输入。全局 `--headed` 才允许物理输入：macOS `CGEvent`、Windows `SendInput`、X11 XTest、Wayland 的 Mutter RemoteDesktop。skill 文档要求「Never switch to headed mode to recover from a headless failure」。
- macOS：headless 右键用 `AXShowMenu`（`AD:skills/agent-desktop-macos/SKILL.md:45`），菜单会弹到屏幕上（推断，这是这个动作本来的效果）；剪贴板变化来自接着点的「复制链接」，不是右键本身。`double-click`、`triple-click`、`hover`、`drag` 必须 `--headed`。
- Windows：`right-click`、`double-click`、`triple-click`、`type`、`focus` 都要 `--headed`；headless 写字只能用 `set-value`（`AD:skills/agent-desktop-windows/SKILL.md:35,93`）。
- Linux：`scroll` 是物理滚轮，要 `--headed` 且只在 X11。

**对 Chromium、Electron 的快照不是纯被动读取。** 读不到网页那部分树时，agent-desktop 往目标 App 写 `AXManualAccessibility` 或 `AXEnhancedUserInterface` 为 true，写后者要等 2 秒；属性原来就是 true 时直接返回、不写（`AD:crates/core/src/tree/renderer_accessibility.rs:20-47`、`AD:crates/macos/src/system/renderer_activation.rs:5,21-29,33-54`）。平台能不能据此察觉见 [§5.1](#51-平台能不能察觉)。

**安装与进程。**

- npm 包 15.0 MB、解开 30.2 MB、26 个文件：六个平台的原生可执行文件，macOS 的剪贴板 helper、Linux 的剪贴板持有进程各两份，每个配一个 `.sha256`；另有 Node 启动器和 `scripts/postinstall.js`。
- 安装脚本（`pkg:scripts/postinstall.js:188-230,345-346,389`）：包里的二进制和 `.sha256` 一致就视为已安装；不一致或缺失时用 curl 从 GitHub 发布页下载、校验 sha256 后解开；把 bin 入口换成指向原生二进制的硬链接；在 macOS 上运行一次 `agent-desktop version`（超时 5 秒），用来结束旧版本留下的常驻进程。安装脚本被拦时（npm 12 默认拦）走 Node 启动器，每次调用多 125–146 ms。
- 1.1.0 删掉了 macOS 上的常驻 command host，每次调用一个进程（`AD:CHANGELOG.md` 第 24 行，ad#414）。`agent-desktop-macos-helper` 只管剪贴板，由主程序带一次性 token 拉起；请求权限时另有一个自我重启的 helper 进程。
- 不联网：`Cargo.lock` 里没有 reqwest、ureq、hyper、tokio、rustls、openssl、native-tls、curl 这类网络库；唯一的 HTTP 代码是 `launch --cdp` 对 `127.0.0.1` 发 `GET /json/version`（`AD:crates/core/src/system/cdp_endpoint.rs:69-78`）。Linux 的 X11 后端按 `DISPLAY` 连 X 服务器，`DISPLAY` 是 TCP 形式时，`network: none` 的沙箱会拦它（推断）。
- 状态目录默认 `~/.agent-desktop`，`AGENT_DESKTOP_HOME` 可改；ref 映射存着元素的 `name`、`value`、`description`，最多留 128 个快照（`AD:crates/core/src/refs/refs_store.rs` 的 `MAX_SAVED_SNAPSHOTS`）。

<a id="13-要授予什么"></a>
### 1.3 要授予什么

| 系统 | 要什么 | 出处 |
|---|---|---|
| macOS | 辅助功能（macOS 27 起这个列表改名「Device Control and Data Access」），授给**启动 agent-desktop 的 App**（终端、编辑器一类）；系统设置里如果单独列出了 `agent-desktop` 二进制，也要打开。屏幕录制只有截图要；自动化（控制 System Events）只有打开通知中心一类命令要 | `AD:skills/agent-desktop-macos/SKILL.md:23`；`AD:skills/agent-desktop-macos/references/troubleshooting.md:9,11` |
| macOS 的归属 | 权限引导页的代码从自己往上找第一个可执行文件路径里带 `.app` 的祖先进程（最多 64 层），取这条路径里最外层的 `.app`；注释写「TCC charges a command-line tool to the app that launched its process tree」。这是第三方工具的代码注释和故障文档，不是 Apple 文档 | `AD:crates/macos/src/system/permissions/permission_guide.rs:18,130-159` |
| macOS 的子命令 | `permissions` 只报告、不弹窗；`permissions --request` 弹系统对话框；`permissions --guide` 打开系统设置对应页 | `AD:crates/macos/src/system/permissions/mod.rs` |
| Windows | 不用授权，同完整性级别的 App 可读可写；以管理员运行的 App 能读不能写，先比较两个进程令牌的完整性级别 | `AD:skills/agent-desktop-windows/references/permissions-and-elevation.md` |
| Linux | 在用户自己的图形会话里运行，装了 `at-spi2-core`；在总线启动之前就开着的 App 要重启；X11 按键要 XTest，Wayland 要 Mutter RemoteDesktop 和 `wl-clipboard` | `AD:docs/install.md`「Permissions」 |

对我们的意思：获取插件是宿主拉起的子进程，按上面的旁证，辅助功能和屏幕录制会记在宿主上，没声明的插件也用得上；这和 [插件协议 §7](../plugin-protocol.md#7-权限与隔离) 现在的推断一致，只是多了第三方旁证。

<a id="14-确定性"></a>
### 1.4 确定性

- 命令行和 C 库都不需要模型（`AD:docs/faq.md:78`「The native CLI does not require a remote model service」）。快照、查找、ref 动作、剪贴板、截图都是确定的系统调用；一次快照里的 ref 编号也是确定的。
- 不确定来自界面本身：同一个 App 两次快照可能不同（Chromium 冷启动只露出一部分、预算用完返回部分树）；AX 返回成功不代表生效，文档举的例子是 Finder 对 `AXConfirm` 返回成功却什么也没做，所以要再快照或用 `get`、`is` 核对。
- 能被固定脚本按顺序调用，不需要 Agent（推断：命令行加 JSON 信封够写固定流程）。`batch` 里后一条用不了同一个 batch 里前一条快照新给的 ref（推断：batch 的参数在调用前就写死了，快照 id 是随机的；文档没写这句），所以「右键 → 读菜单 → 点菜单项」每一步要单独起一次进程。

<a id="15-成熟度"></a>
### 1.5 成熟度

- 11,299 ★、774 fork、22 个 subscriber，未归档，最近推送 2026-11-03（09:53 UTC 查）。仓库建于 2026-02-19；npm 上 47 个版本，第一个发于 2026-02-23；1.0.0 发于 10-28，1.1.0 发于 11-03（npm 04:15 UTC，GitHub release 04:13 UTC，13 个资产）。
- 贡献者：lahfir 221 次、github-actions[bot] 43、detail-app[bot] 26、ctate 15、dependabot[bot] 14，其余个位数；lahfir、ctate 的 GitHub 资料都写 Vercel。实际上是一位主维护者加一位同事。issue 91 个（开着 50），PR 415 个（开着 84、合入 248）。
- 每月提交（按提交者日期）：2 月 85、3 月 11、4 月 14、5 月 21、6 月 34、7 月 11、8 月 15、9 月 69、10 月 63，11 月前 3 天 9。1.0 之后新 issue 很密（推断：用户量上来了）。
- Homebrew、Scoop 的官方包「正在提交」，还不能用。

<a id="16-限制"></a>
### 1.6 限制

**树不全的界面**（issue 证据）：

- Electron、Chromium：Arc 报激活失败（[ad#465](https://github.com/vercel-labs/agent-desktop/issues/465)）；密集网页的 ref 映射超过 1 MB 上限直接报错（[ad#463](https://github.com/vercel-labs/agent-desktop/issues/463)）；默认 `--max-depth 10` 常常看不到网页里的按钮。
- 自绘界面：wxWidgets 写的 Bambu Studio 只露出 8 个 ref（[ad#375](https://github.com/vercel-labs/agent-desktop/issues/375)）；Word 正文读不到（[ad#479](https://github.com/vercel-labs/agent-desktop/issues/479)）；Tauri 在 Linux 上只有一个空的 `webarea`（[ad#446](https://github.com/vercel-labs/agent-desktop/issues/446)）。故障文档承认自绘界面、游戏、部分网页视图「expose little accessibility data」，退路是截图。
- Qt：没有树完整度的报告。

**长列表慢。** [ad#356](https://github.com/vercel-labs/agent-desktop/issues/356)：Apple Notes 一个约 700 条的文件夹，默认 3 秒预算直接超时；放宽到 90 秒，完整快照 85 秒、1581 个 ref、192 KB；`--skeleton` 也要 19 秒，因为一个浅层的表格底下挂着 736 行，屏上只有约 10 行。[ad#416](https://github.com/vercel-labs/agent-desktop/issues/416)：每条命令先扫一遍全机窗口清单，约占 70% 的时间。[ad#424](https://github.com/vercel-labs/agent-desktop/issues/424)：窗口清单不稳定时观察命令超时，错误提示只建议 `--headed`，照做会抢走键盘焦点。

**中国客户端没人报过。** issue 里没有关于中国客户端的报告；唯一提到微信的是外部 PR [ad#477](https://github.com/vercel-labs/agent-desktop/pull/477) 测试说明里的一句：前置检查要求前台 App 报告焦点窗口，「which WeChat and Claude do not」。仓库全文里没有任何平台名。

**Mac 微信的树**（别的项目的 issue）：

- [BiboyQG/WeChat-MCP #15](https://github.com/BiboyQG/WeChat-MCP/issues/15)（2026-04-03，已关）：Mac 微信 4.1.5、macOS 26.3.1，`AXWindow` 下只有三个 `AXButton` 和一个空的 `AXGroup`。仓库主 2026-04-12 在 #15 和 [#13](https://github.com/BiboyQG/WeChat-MCP/issues/13) 都回了「works fine on macOS 15」，问题出在 macOS 26。
- [dustin573/wechat-mcp #1](https://github.com/dustin573/wechat-mcp/issues/1)（2026-10-20，开着）：4.1.15、macOS 26.6.2，只有浅层窗口，应用级命中测试返回 `kAXErrorNotImplemented`，`AXEnhancedUserInterface` 标为可设但写不进去。
- [BiboyQG/WeChat-MCP #17](https://github.com/BiboyQG/WeChat-MCP/issues/17)（macOS 15、国际版 4.1.7）：消息列表和输入框要聊天窗口在前台才出现。
- Mac App Store 中国区现行 `com.tencent.xinWeChat` 4.1.15，2026-10-14 发布，最低 macOS 12.0（iTunes 查询）。
- 两份报告指向 4.x 是 Qt 写的（#15 的报告人推测；dustin573 #1 的报告人在二进制里看到 `QAccessible`、`QCocoaAccessibility` 一类标识，但他自己说这证明不了聊天区用的是标准 Qt 无障碍），未确认。
- 这些报告测的都是聊天界面；收藏页没人测过，收藏页同样读不到是推断。本机的 3.8.8.19 是 AppKit 的界面，大概率还读得到（推断）。
- PC 微信 4.1 起，从没开过无障碍模式的新账号拿不到 UI Automation 树（[06 §1.2](06-platform-access-and-account-risk.md#12-视觉层的风险从哪里来)）；agent-desktop 在 Windows 上走的就是 UIA，同样受限（推断）。

**拿不到链接。** 快照节点只有 `role`、`name`、`value`、`description`、`native_id`、`states`、`available_actions`、`bounds` 等字段，没有 URL（`AD:crates/core/src/tree/node_identity.rs` 等三个文件）；请求暴露 `AXURL` 的 [ad#492](https://github.com/vercel-labs/agent-desktop/issues/492) 还开着。

**签名。** macOS 的两个可执行文件是 ad-hoc 签名，没有 Team ID（`codesign -dvvv` 读 npm 包里的文件）；Windows 的 exe 没签名，SmartScreen、杀毒软件、全新 Windows 11 的智能应用控制可能拦（`AD:docs/install.md`「Permissions」）。

<a id="17-隐私"></a>
### 1.7 隐私

- 它自己的 FAQ 写明：快照树、窗口标题、剪贴板、通知、截图、命令结果都含你让它看的桌面内容；会话 trace 即使结构化字段被脱敏，也可能留着证据文件；「redaction does not make a screenshot safe to publish」（`AD:docs/faq.md`「What data can contain private information?」）。
- 落盘：`~/.agent-desktop` 下的 ref 映射存着元素的名字和值，最多 128 个快照；`session start --screenshots` 存前后截图，不脱敏（`AD:CONCEPTS.md`「State Root」「Replay Artifacts」）。
- 密码框：读的一侧不给值，写的一侧不回显（`AD:CONCEPTS.md`「Secure Field」）。
- JEV 会把每个元素的角色、名字、最多 60 字的值、窗口标题和最近的动作发给 TypeSafe 或 OpenRouter，文档自己承认「enough to send the contents of a private document or a filled form」。命令行本身不发任何东西。
- 文档里没有关于平台风控、账号风险的说明。

<a id="18-许可证"></a>
### 1.8 许可证

- Apache-2.0（`AD:LICENSE`、`AD:Cargo.toml:9`、npm 元数据），仓库根目录没有 NOTICE。`LICENSE` 末尾的 `Copyright [yyyy] [name of copyright owner]` 在 Apache-2.0 全文的附录「How to apply」里，是标准文本自带的示例，不是漏填（`AD:LICENSE:178-190`）。
- npm 包（26 个文件）和 release 压缩包（只有 `agent-desktop`、`agent-desktop-macos-helper`）里都没有 LICENSE 文件，插件随包分发时要从仓库把 LICENSE 带上。
- Apache-2.0 第 6 条不授予商标权：不用 Vercel 的名字和标识，不暗示官方合作（推断，按第 6 条）。

<a id="2-mika"></a>
## 2. Mika

<a id="21-是什么"></a>
### 2.1 是什么

- 装在手机上的 Android Agent，Kotlin 写的，`app`、`core` 下 1,350 个 `.kt` 文件约 19.5 万行（含测试）。安装要安卓 11（`minSdk = 30`），操作 App 要 12，推荐 14（`M:app/build.gradle.kts:19`、`M:README.md:125-131`）。配对完不需要电脑。
- 你给一句目标（「把小红书某博主最近发的整理一下」），主模型调工具读屏、点按、打字，办完给结果；另有内置浏览器、Linux 沙箱（PRoot 加 Alpine，随 APK 带）、记忆、定时任务。对外是 MCP 服务端、HTTP `/api/v1` 和 Python 命令行，都只收目标（[§2.5](#25-模型依赖与确定性)）。
- 模型：OpenAI 兼容、OpenAI Responses、Anthropic、Gemini 四种接口，国内外预设一长串，也能接 Ollama、LM Studio；主模型必须会调工具（`M:docs/models.md:5`）。账号登录支持 ChatGPT、xAI、OpenRouter，明确不做 Claude 订阅、Google 账号登录（`M:docs/dev/account-signin.md`）。

<a id="22-后台屏幕的原理"></a>
### 2.2 后台屏幕的原理

用户说的「隐藏的后台屏」是这样做出来的：

1. **自己当 ADB 客户端。** 不用 Shizuku、不要 root（源码里搜不到 Shizuku；root 版是单独的预发布 `v0.1.33-root`）。App 里有一套纯 Kotlin 的 ADB 实现（配对用 SPAKE2，连接走 TLS，`MC:adb/`）。用户在开发者选项里开「无线调试」、配对一次；之后按「上次的端口 → 系统属性 `service.adb.tls.port` → mDNS `_adb-tls-connect._tcp` → 扫本机临时端口 → 用户手填」的顺序找端口。
2. **以 shell 身份起助手。** 经 adbd 的 `exec:` 服务跑 `CLASSPATH=<base.apk> app_process /system/bin --nice-name=<包名>:helper com.wangzan101.mika.shell.HelperMain <包名> <uid> <签名指纹>`，一次性口令从标准输入读（`MA:cap/phone/shell/HelperProcess.kt:29`）。助手是 uid 2000 的 shell 进程，能做 `adb shell` 能做的事。取系统上下文的写法照 scrcpy，源文件头部保留了出处。
3. **助手建虚拟屏。** 调 `DisplayManager.createVirtualDisplay("mika-agent", 宽, 高, dpi, surface, flags)`，尺寸和主屏相同，第一次要打开 App 时才建（`MA:cap/phone/shell/ShellService.kt:91-120,1603-1632`）：
   - 标志：`PUBLIC`、`OWN_CONTENT_ONLY`、`SUPPORTS_TOUCH`、`ROTATES_WITH_CONTENT`、`DESTROY_CONTENT_ON_REMOVAL`；安卓 13 起加 `TOUCH_FEEDBACK_DISABLED`。
   - shell 有建可信屏的权限时再加 `TRUSTED`、`OWN_DISPLAY_GROUP`、`ALWAYS_UNLOCKED`；安卓 14 起再加 `OWN_FOCUS`、`STEAL_TOP_FOCUS_DISABLED`，这块屏有自己的焦点，不抢主屏的输入。
   - 没有这个权限的系统（例如 MuMu 模拟器）退回不可信屏（[M#55](https://github.com/wangzan101/Mika/issues/55)）。
4. **画面进 App 自己的 `ImageReader`**，截图不走无障碍截图接口（`MA:cap/phone/display/AgentDisplay.kt:19,32`）。
5. **收尾和杂项**：目标 App 在虚拟屏上静音（收尾还原）；横屏 App 锁这块屏的方向；任务结束时 App 挪到主屏最后面留着；冒到主屏的页面挪回虚拟屏，登录、付款一类留给用户（`M:docs/dev/architecture.md` 第 6 节）。

已知问题：vivo 上虚拟屏几秒后停止出帧（[M#15](https://github.com/wangzan101/Mika/issues/15)，后来加了保持唤醒）；鸿蒙 NEXT 加卓易通没有无线调试，做不了（[M#150](https://github.com/wangzan101/Mika/issues/150)）。scrcpy 3.x 的 `--new-display` 也能开一块虚拟屏，名字叫 `scrcpy`（[scrcpy 文档](https://github.com/Genymobile/scrcpy/blob/master/doc/virtual-display.md)）。

<a id="23-读屏与输入"></a>
### 2.3 读屏与输入

- **读屏**：Mika 自己的无障碍服务，事件类型 `typeAllMask`，标志 `flagRetrieveInteractiveWindows`、`flagReportViewIds`、`flagIncludeNotImportantViews`，能取窗口内容、派发手势、截图（`M:app/src/main/res/xml/accessibility_service.xml`）；按屏取窗口用 `windowsOnAllDisplays`（`MA:cap/phone/a11y/ScreenAccessibility.kt:93`）。
- **树不够用时补字**：先按有名字的元素够不够多、有没有大块空白判断树的质量；拿不准或缺块时，用随 APK 带的 Google ML Kit 文字识别（中文、拉丁两种模型打进 APK，`com.google.mlkit:text-recognition` 16.0.1，不经 Play 服务下载，`M:gradle/libs.versions.toml:18,45-46`）补出屏上的字。防截屏的页面不识别，交给用户。配了看图模型时还能让模型在截图上框元素，框完再回树里核对。
- **点按**：无障碍 `dispatchGesture`，手势先 `setDisplayId` 指到虚拟屏（`MA:cap/phone/a11y/ScreenAccessibility.kt:112,125`）。**变速滑动**由助手逐点注入，走隐藏的 `InputManager.injectInputEvent`（`MA:cap/phone/shell/ShellService.kt:1129,1492-1494`）。**打字**：找得到输入框就 `ACTION_SET_TEXT` 分段写、读回核对；找不到就走剪贴板粘贴，粘完把用户原来的剪贴板还回去。
- **节奏**（作者说是为了不被 App 当成机器人，[M#10](https://github.com/wangzan101/Mika/issues/10) 的回复）：落指前停 30–120 ms、按住 50–120 ms、落点随机偏、滑动走小弧线；中文每字 160–380 ms。App 知识里标了 `pace: careful` 的（定义是「这个 App 限制一个账号操作的快慢」）：同一屏动作之间 0.9–2.2 s，换新一屏停 2.5–5 s，再按屏上字数每字加 15 ms 的阅读时间（至多 8 s），单次停留至多 15 s，一个任务至多换 24 屏。网站：careful 站第二页起停 10–16 s，之后每多一页上下限各加 0.6–1.0 s，至多 30 s、12 页（`M:docs/dev/knowledge-format.md:37,83-85`、`MC:cap/phone/AppPacing.kt:35-55`、`MC:cap/web/PagePacing.kt:28-40`）。
- **标了 careful 的**：App 是微信、抖音、小红书、快手、微博、闲鱼、BOSS直聘，网站是 zhipin.com；B站的 App 和网站知识都有，没标；视频号没有条目（`M:core/src/main/resources/knowledge/apps/`）。
- **读长列表** `phone_collect`：每屏读一次文字行（可用 `fromY`、`toY` 去掉顶栏底栏），滚 3 格；新一屏的头和已收的尾按整行文字找最长重叠来去重；连着两屏没新行、到上限（30 屏、1000 行、5 万字）、切了 App、屏读不了就停。结果是纯文本行，只回给模型，不写文件（`MC:cap/phone/PhoneAutomationTools.kt:46-140`、`MC:cap/phone/PhoneAutomationEngine.kt:77-87`）。
- **内置浏览器**：WebView 画在 App 自建的私有虚拟显示上，经 DevTools 协议发真实的鼠标键盘事件，刻意不开 `Runtime.enable`，免得页面察觉调试器；为此全局打开了 WebView 调试（`MA:cap/web/cdp/WebDevtools.kt:305`）。

<a id="24-要授予什么"></a>
### 2.4 要授予什么

| 项 | 怎么来 | 出处 |
|---|---|---|
| 开发者选项、USB 调试 | 用户手动开，要一直开着 | `M:docs/setup-guide.md` 第 1–2 步 |
| 无线调试 | 用户手动开、配对一次；配对码可由无障碍从设置页读出，或在通知里填 | 同上第 4–5 步 |
| 无障碍服务 | 设置指南建议先跳过：旁加载的 App 让用户自己去系统设置里开，会撞上「受限制的设置」；配对后由助手写 `enabled_accessibility_services` 和 `accessibility_enabled 1` 打开，被系统停掉时自动重开 | `M:docs/setup-guide.md:77-84`；`MC:shell/RecoveryPermissions.kt:78-86` |
| `WRITE_SECURE_SETTINGS` | 助手用 `pm grant` 授给 App 自己，用来自动开无线调试、修无障碍 | 同上；`M:app/src/main/AndroidManifest.xml:30-32` |
| 电池白名单 | 助手跑 `cmd deviceidle whitelist +<包名>` | `MC:shell/RecoveryPermissions.kt` |
| 通知 | 安卓 13 起要用户允许 | 清单 `POST_NOTIFICATIONS` |
| 悬浮窗 | 不要 `SYSTEM_ALERT_WINDOW`，悬浮胶囊画在 `TYPE_ACCESSIBILITY_OVERLAY` 上 | `MA:cap/phone/a11y/OverlayHost.kt:139` |
| 其他 | 精确闹钟、录音、读图片、装包、开机启动；可选存 PIN 让电脑远程亮屏解锁（13 起） | 清单；`M:docs/mcp.md`「远程亮屏与解锁」 |

开发者选项里有能清数据的开关（小米的 16 KB 页面模式），有人踩到过（[M#39](https://github.com/wangzan101/Mika/issues/39)），设置指南加了警告。

<a id="25-模型依赖与确定性"></a>
### 2.5 模型依赖与确定性

- **对外入口都收目标、都经模型。** MCP 共十个工具：七个管任务（`mika_run_task`、`mika_task_status`、`mika_wait_task`、`mika_answer`、`mika_steer_task`、`mika_cancel_task`、`mika_list_tasks`），三个管设备（`mika_device_state`、`mika_wake_device`、`mika_unlock_device`）（`MC:mcp/McpTaskTools.kt:176-182`、`MC:mcp/McpDeviceTools.kt:36-38`）。HTTP 和命令行调的是同一套任务接口。局域网监听要 Bearer 令牌，经 USB 时只绑 127.0.0.1；从电脑来的任务，付款和确认永远不放行（`M:docs/mcp.md`「安全」）。
- **不经模型的工具会话只是桩。** `McpToolSession` 的注释是「一个无模型的会话型 Run」，但除了测试没有调用方（全仓库 `openToolSession` 只出现在 `MC:mcp/McpServer.kt:88` 和对应的测试里）。
- **路线回放也要模型选路线。** 办成过的任务按步骤记下工具、参数、目标元素的选择器（角色、名字、资源 ID、位置、同类序号）和前后锚点（前台包名加至多 3 段稳定文字，排除列表内容和带数字的字）；下次 `Replayer` 每步核对锚点、要求选择器唯一命中后不调模型地执行，对不上就交还模型（`MC:replay/Route.kt`、`MC:replay/Replayer.kt`）。但回放只在任务开头由 Mika 自己触发：`RouteMatcher` 先按二元组相似度 ≥0.3、或包名相同挑候选，没有候选不调模型，有候选就调一次模型选定（超时 20 秒，`MC:replay/RouteMatcher.kt:29-56`）。外部不能点名回放某条路线。
- **没有脚本入口。** 广播、intent、脚本接口的请求（[M#113](https://github.com/wangzan101/Mika/issues/113)）、「录制操作沉淀成 skill」（[M#147](https://github.com/wangzan101/Mika/issues/147)）都还开着。社区的插件 PR（#170、#176、#178、#184，都没合并）是给模型加工具，不是给外部程序调 Mika。
- **模型多半拿不到链接（推断）。** `prompts/tools/` 下 79 个工具说明里没有读剪贴板的工具；助手的 AIDL 接口里和剪贴板有关的只有写入和还原（`M:app/src/main/aidl/com/wangzan101/mika/shell/IMikaShell.aidl:33-44`）。模型可以把剪贴板粘进某个输入框再读出来，但代码里没有专门的路。
- 小结：Mika 不能被当成确定性的获取插件调用；要么接受每次走它的模型，要么只借做法自己写固定流程。

<a id="26-成熟度与可信度"></a>
### 2.6 成熟度与可信度

- **数字**（09:53 UTC 查）：5,795 ★、1,126 fork、9 个 subscriber；仓库建于 2026-10-29；430 个提交；36 个发布（含 1 个 root 预发布），38 个 APK 附件都带 GitHub 给的 sha256 摘要，下载合计 27,350（09:38 UTC）。贡献者 40 个，含 5 个匿名；作者 312 次，其余 34 人里 26 人只有 1–2 次。
- **公开历史从一个大提交开始。** 首提交 `093965f`「Mika 0.1.0:第一个公开测试版」没有父提交，git tree 里 1,370 个文件、156,953 行。所以 5 天 430 个提交是闭门开发之后接着写的（推断）。
- **作者**：WangZan（`wangzan101`），资料写深圳，账号建于 2022-03，此前的仓库是论文配图、标点修正一类小工具，最高 40 星。v0.1.34 的发布说明写「移除个人联系方式和群聊入口」。`.gitignore` 列了 `.claude/`、`CLAUDE.md`、`.omc/`，明显用 AI 编程助手开发。免责声明自述「目前主要在一台安卓 14 手机上测试过」（`M:docs/disclaimer.md`）。
- **没有 CI。** Actions 工作流 0 个，加 CI 的 PR #275 还开着；APK 是作者本机构建上传的，公开流程证明不了 APK 和源码对应。
- **可疑迹象检查**：
  - 预编译文件：`libproot.so` 等三个 `.so` 和 31 MB 的 `rootfs.tar.xz`，附来源锁文件、补丁和声称逐字节可重现的构建脚本（`M:tools/sandbox/README.md`），本次没跑校验。
  - 源码里没有 `DexClassLoader`、`PathClassLoader`、`loadLibrary` 一类动态加载；连的外部主机只有各家模型服务商、`api.github.com`（检查更新）、`www.bing.com`（浏览器搜索）和几个登录地址；清单和构建脚本里没有 Firebase、Crashlytics、Sentry、Bugly、友盟。更新包下载后验大小、sha256、包名、版本和签名证书。
  - 第三方静态审查 [M#241](https://github.com/wangzan101/Mika/issues/241) 的结论是没有后门、隐蔽外传、动态加载或遥测 SDK，另列了 5 个可加固点（局域网 MCP 明文传令牌、WebView 调试套接字、运行截图不加密落盘、ADB TLS 不校验设备证书、本机端口扫描）。这是别人的说法，我们只核了「没有动态加载」「WebView 调试常开」两条。
  - **恶意 fork**：[M#250](https://github.com/wangzan101/Mika/issues/250) 报告 fork `zhaikong/mika` 在技能脚本里加了从区块链交易取载荷再执行的代码。`gh api` 对比结果：领先 1 个提交、落后 347 个，改动文件里确有技能脚本目录下的 `transport.py`、`session.py`、`extract_pdf.py`，还有 `.vscode/tasks.json` 和一批放在奇怪目录下的字体文件；文件内容没读。
- **star 的走势**：逐个 star 的时间接口拿不到；star-history 的曲线上日增约 1.2k–1.5k，看不出单日跳变（推断，曲线经过平滑）。issue 大多是带自检结果和系统版本的设备报告，作者一两天内回复、修复、标版本。
- 判断：真在快速迭代、有真实用户反馈的个人项目（推断）；证明不了的是 APK 与源码对应、root 分支的内容。

<a id="27-限制"></a>
### 2.7 限制

- **每次走模型，输出由模型决定。** 同一个请求两次写出来的文件格式可能不一样；`phone_collect` 按整行文字精确去重，行内的时间、计数一变就对不上（推断）。
- **微信在虚拟屏上不配合。** [M#237](https://github.com/wangzan101/Mika/issues/237)（vivo，开着微信应用分身，2026-11-02）：在虚拟屏上会话列表看得见，整屏无障碍节点为空（`treeQuality=unusable`）；按识别出的文字坐标点、长按、滚动都「画面没变」，`input -d 21 tap` 也没反应；同一块屏上的设置 App、系统分享面板正常，返回键有效。报告人只验证了分享面板能弹出、能关闭，系统分享是他提的修复建议，没实测发出消息；他也只测了虚拟屏，没在主屏对照，所以「微信在主屏上可以」是推断。[M#92](https://github.com/wangzan101/Mika/issues/92)（一加 Ace 5 Pro，Android 15）只说自检全过、给文件传输助手发消息失败，没说树是空的。
- **支付类 App**：Flyme 的「支付保护」打开支付 App 时会关掉无障碍（[M#1](https://github.com/wangzan101/Mika/issues/1)）。
- **机型**：vivo 出帧停止、鸿蒙没有无线调试（[§2.2](#22-后台屏幕的原理)）；vivo 上读剪贴板曾报 `SecurityException`（[M#74](https://github.com/wangzan101/Mika/issues/74)，0.1.27 修了）。
- **安全默认偏松**：默认「询问模式」只在花钱前问；开发阶段关掉了最终核查；开了自主模式连付款也不拦（`M:docs/dev/roadmap.md` 已知问题 1）。网页任务不进路线回放。

<a id="28-隐私"></a>
### 2.8 隐私

- 没有 Mika 自己的服务器；API Key 和 ChatGPT 会话用安卓密钥库加密存（`M:docs/disclaimer.md`「数据与隐私」）。屏上的字会发给你在 Mika 里配的模型服务商，多半在云端。
- **ML Kit 会给 Google 发使用指标。** ML Kit 条款写明：识别在本机完成，图片和结果不上传；但会把 API 的性能和使用量指标发给 Google，也会不时连 Google 服务器取修复、模型更新和硬件兼容信息（[ML Kit terms](https://developers.google.com/ml-kit/terms)「Privacy」段，2026-11-03 取）。README 里「只发给你配的模型服务商」漏了这一项（`M:README.md:135-137`）。
- WebView 调试全局开着，同一台手机上的别的 App 也许能连这个调试套接字（M#241 的说法，要按系统版本实测）；运行截图不加密落盘（同上）。
- 记忆只从用户自己说的话里记，不从屏幕和网页里记，过滤密码、验证码、卡号，用密钥库加密（`M:docs/dev/architecture.md` 第 12 节）。

<a id="29-许可证"></a>
### 2.9 许可证

- Mika 本体 Apache-2.0，NOTICE 只有「Mika / Copyright 2026 wangzan101」（`M:LICENSE`、`M:NOTICE`）。借代码要附许可证、保留 NOTICE 和源文件里的出处（包括 scrcpy 那段的头部）、标明改动。
- 随 APK 带的 PRoot 是 GPL-2.0-or-later，talloc 是 LGPL-3.0-or-later，都以独立程序运行（`M:licenses/THIRD_PARTY_NOTICES.md:146-170`）。项目自己的规矩要求每次发布附 `mika-<版本>-gpl-sources.tar.gz`（`M:tools/sandbox/README.md:60-67`），但 38 个附件全是 APK。和自己定的规矩不符是事实；算不算违反 GPL-2.0 第 3 条要看源码的提供方式（仓库里有来源锁文件、补丁和构建脚本），这是法律判断，标推断。
- ML Kit 是 Google 的闭源 SDK，没有列进第三方声明，也没列进 App 的「开源许可」页（`MA:ui/LicensesScreen.kt:52-71`）。
- 对我们：Apache 部分可以借；PRoot 照 [插件协议 §9.6](../plugin-protocol.md#96-许可证) 不打进插件资产；ML Kit 不打包。
- 能借的块：

| 块 | 位置 | 用处 |
|---|---|---|
| 虚拟屏的标志组合与降级 | `MA:cap/phone/shell/ShellService.kt:91-120,1603-1632` | 安卓视觉层插件开后台屏时参考；电脑端也可以直接用 scrcpy 的 `--new-display` |
| 逐屏合并去重 | `MC:cap/phone/PhoneAutomationEngine.kt:77-87` | `list` 一页处理几屏时的去重；我们要按身份比，不按整行文字比 |
| 节奏数字 | `MC:cap/phone/AppPacing.kt`、`MC:cap/web/PagePacing.kt`、`M:docs/dev/knowledge-format.md:83-85` | `pacing` 默认值的出处 |
| 路线格式 | `MC:replay/Route.kt`、`MC:replay/Replayer.kt` | 视觉层流程「固定下来」时的落盘格式 |
| 读剪贴板 | `MA:cap/phone/shell/ShellClipboard.kt:14-44` | 说明 shell 身份读得到手机剪贴板 |
| ADB 客户端 | `MC:adb/`（纯 JVM） | 只有自己做手机 App 时用得上；电脑端插件直接用 adb |

- 限制：助手进程只认 Mika 自己的 APK、签名指纹和一次性口令（`MA:cap/phone/shell/HelperProcess.kt:29-50`），别的程序借不了它；复用这些代码等于自己做一个安卓伴侣 App。

<a id="3-怎么接进获取插件"></a>
## 3. 怎么接进获取插件

两端的插件都照 [06 §1.3](06-platform-access-and-account-risk.md#13-混合路径视觉层拿链接公开页面取内容) 的混合路径：视觉层的 `list` 一页只返回 `url`（加上界面上看得到的标题、作者），内容由别的插件按链接去取（[插件协议 §3.8](../plugin-protocol.md#38-list-与跟着收)）。

<a id="31-电脑端"></a>
### 3.1 电脑端

**几种做法。**

| 做法 | 怎么做 | 拿得到什么 | access / login | 用不用 |
|---|---|---|---|---|
| 电脑-1 读树加无障碍动作 | 插件自带 agent-desktop：`snapshot --app` 找到列表容器，之后每次都带 `--root` 限在容器上；每一行 `right-click`（macOS headless 走 `AXShowMenu`）→ `snapshot --surface menu` → 点「复制链接」→ `clipboard-get`；`scroll` 列表容器，前后两屏有重叠 | 行上的文字；链接来自剪贴板 | `vision-ui` / `required`，`credential: app` | 树在的时候用；macOS 上不抢焦点、不动指针，但菜单照样弹、剪贴板照样变；Windows 上右键要 `--headed` |
| 电脑-2 截图、OCR、物理输入 | `screenshot --app` 截窗口 → 本机 OCR 找行 → `--headed` 按坐标右键 → 再截图 OCR 找「复制链接」→ 点 → `clipboard-get` | 同上 | 同上，多声明 `screen-capture` | 树没了时的退路，即问题 1 的 A；agent-desktop 只剩统一信封的用处 |
| 电脑-3 读日常浏览器的网页树 | 用 agent-desktop 读 Chrome 里平台网页的无障碍树，不开 CDP、不装扩展 | 卡片文字；链接要打开后读地址栏，或逐条复制 | `vision-ui` / `required`，`credential: app` | 不建议：每条都要在登录着的页面里点一次，不如 `browser-session` 拿得全 |
| 不用：`launch --cdp` | 带远程调试端口重启官方客户端，在它的渲染进程里执行脚本 | — | `client-hook` | 不用 |

**按平台。**

| 平台 | 电脑端客户端 | 列表在哪 | 链接或 ID 从哪来 | 建议 | access / login |
|---|---|---|---|---|---|
| 公众号（微信收藏里的文章） | Mac 微信（现行 4.1.15）、Windows 微信 4.x | 左栏「收藏」→ 链接类。树在时 Mac 上每行一个 `AXStaticText`，文字以「链接」开头（[pengyulong/wechat-favorites-exporter](https://github.com/pengyulong/wechat-favorites-exporter) README）；Windows 上是 UIA 的 `ListItem`（pywechat 的 `cardLink_to_url`）。Mac 上拿不到可靠的收藏时间 | 右键「复制链接」，剪贴板里是 `mp.weixin.qq.com/s?__biz=…`，可能带分享 token | 电脑-1；树没了按问题 1 | `vision-ui` / `required`，`sites: [{site: weixin.qq.com, credential: app}]`；正文交给公众号的 `public-web` 插件 |
| 视频号「赞和收藏」 | PC 微信里的视频号窗口：Windows 版有「我的」收藏、点赞（[nobiyou/wx_channel](https://github.com/nobiyou/wx_channel) README，这是 `client-hook` 工具的说法）；Mac 版有没有待核实 | 视频号 →「我的」 | 页面由 `WeChatAppEx` 承载，agent-desktop 能不能打开它的网页无障碍、分享菜单里有没有「复制链接」都待核实 | 先不排首批，等问题 4 | `vision-ui` / `required`，`credential: app`；内容另由借元宝登录态的插件取（D-g） |
| 抖音 | 抖音电脑版 8.7.0（Mac App Store `com.bytedance.douyin.desktop`，2026-10-15）；本机这一版是 Electron 36.4.0（字节自己的构建），带 `libLynx.dylib` 和字节安全 SDK 的 `mssdk/libMetaSecML.dylib`（凭文件名推断） | 「我的」→ 喜欢、收藏（推断） | 树里没有；分享 →「复制链接」得到 `v.douyin.com` 短链（推断）；打开详情会算一次观看（推断） | 不做电脑端视觉层，照 [06 §17.3](06-platform-access-and-account-risk.md#173-首批自己适配的插件) 走 `browser-session` 的页内请求 | 若做：`vision-ui` / `required`，`credential: app` |
| B站 | 官方客户端 Mac、Windows 都是 Electron（下载地址路径带 `pc_electron_mac`；[msojocs/bilibili-linux](https://github.com/msojocs/bilibili-linux) 移植的是它的 `app.asar`） | 收藏夹、稍后再看（推断） | 同抖音 | 不做；扩展导出收藏夹、yutto 都直接给 BV 号 | 同上 |
| 小红书、快手、X | 没有自己的桌面客户端（Mac App Store 查不到；小红书、快手在 Mac 上只能经应用宝跑手机版） | 网页版 | — | 不做 agent-desktop 插件；小红书走油猴导出，X 走 twitter-web-exporter 或官方归档 | — |

<a id="32-安卓端"></a>
### 3.2 安卓端

**几种做法。**

| 做法 | 怎么做 | 手机上要开的 | 模型 | 拿得到什么 | access / login | 建议 |
|---|---|---|---|---|---|---|
| 安卓-A 手机上跑 Mika，导出再导入 | 让 Mika 把收藏列表写成文件（沙箱的 `write_file`、`share_file`），传到电脑由通用导入插件读 | 开发者选项、USB 调试常开、无线调试；无障碍由 Mika 打开 | 每次，用 Mika 自己的配置 | 屏上的整行文字；多半没有链接（推断），没有链接就是无平台 ID 的条目，去重和取内容都接不上 | 导入插件照 [插件协议 §9.4](../plugin-protocol.md#94-浏览器扩展与油猴脚本) 记 `official-export` / `none`，`description` 写明清单来历 | 不做专门支持 |
| 安卓-B 电脑经 ADB 驱动手机 | 获取插件在电脑上，经宿主管的 adb 服务按固定流程：打开收藏页 → `uiautomator dump` 读树 → 逐条点开 → 分享 →「复制链接」→ 经 adb shell 读手机剪贴板 → 返回 → 滚动；读不到树时截图走本机 OCR | 开发者选项、USB 调试或无线调试；第一次连电脑点「允许 USB 调试」；不装 App、不开无障碍服务 | 默认不要 | 卡片上的标题、作者，分享链接；没有收藏时间 | `vision-ui` / `required`，`credential: app`，`devices: [adb]` | 按需做，放 `{fetch-org}` |
| 安卓-C 自己做安卓 App | C1 无障碍服务 App（AutoJs6、gkd 一类）；C2 照 Mika 自带 ADB 客户端、起 shell 助手、开虚拟屏 | 装不在应用商店的 APK；C1 开无障碍、先「允许受限制的设置」；C2 同 A | 可以不要 | 同 B | `vision-ui` / `required` | 不做：成本按月计，检测面更大，平台流程在手机上跑还要另起一套运行时 |
| 包装 Mika 的 MCP | 插件调 `mika_run_task` 交一句目标 | 同 A | 每次，在手机上发生，宿主代发不了、标不了「离开本机」 | 由模型决定 | `vision-ui` / `required` | `{fetch-org}` 不收；用户自己装的第三方插件照常能用（D-b 没有闸门） |
| 安卓-D 不做安卓 | 只走电脑和网页 | 无 | 不要 | 最全：平台 ID、标题、作者、收藏夹归属 | 按平台 | 先行，已经在 [06 §17.3](06-platform-access-and-account-risk.md#173-首批自己适配的插件) 的首批里 |

**安卓-B 的默认做法**（技术上我们定）：

- **主屏，不开虚拟屏。** 用的是你自己那块屏，跑的时候手机要亮屏、解锁，你用不了。虚拟屏做成插件设置：scrcpy 的 `--new-display --start-app`，或者照 Mika 的标志组合自己写一个 `app_process` 助手；读树要 `uiautomator dump --windows`，点按要 `input -d <屏号> tap`（`main` 镜像）。现成工具不直接支持（Android-MCP 只调不带参数的 `uiautomator dump`，uiautomator2 只有截图带屏号），要自己写一层薄的 adb 封装。微信、视频号不用虚拟屏（[§2.7](#27-限制)）。
- **读树用 `uiautomator dump`。** 它要等屏幕安静 1 秒、10 秒内等不到就失败（`main` 镜像里的 `waitForIdle(1000, 1000 * 10)`），自动播放视频的页面改用截图加本机 OCR。
- **剪贴板在手机侧读。** shell 身份读得到手机剪贴板（Mika 的 `ShellClipboard.kt` 就这样读）。不用 scrcpy 的剪贴板同步，它会把手机剪贴板同步到电脑、盖掉你电脑上的内容（[scrcpy control.md](https://github.com/Genymobile/scrcpy/blob/master/doc/control.md)）。
- **流程写死回放**，落盘格式参考 Mika 的选择器和锚点（[§2.5](#25-模型依赖与确定性)）；对不上就停，回 `upstream-changed`，碰到验证或风控提示回 `blocked`。
- **代价**：每条要点开、分享、复制、返回，四五次换屏；按 Mika 给 careful App 的节奏（换一屏停 2.5–5 秒），一条十几到二十几秒（推断）。适合「插上手机，列一次」的批量导入，不适合后台常驻的跟着收。

**按平台**（包名出自 Mika 的 App 知识，例如 `M:core/src/main/resources/knowledge/apps/xiaohongshu/APP.md:4`）：

| 列表 | App 包名 | 电脑或网页上有没有 | 分享出来的链接 | B 的价值 | Mika 标的节奏 |
|---|---|---|---|---|---|
| 小红书收藏、专辑 | `com.xingin.xhs` | 网页有：油猴脚本 xhs-vault 导出（[06 §1.3](06-platform-access-and-account-risk.md#13-混合路径视觉层拿链接公开页面取内容)） | `xhslink` 短链，带 `xsec_token`，只在会话内有效、几小时后 404，拿到要尽快取 | 最高：网页侧处罚记录多（[06 §16.2](06-platform-access-and-account-risk.md#162-issue-里的账号问题)），App 侧暂时没有报告（没有报告不等于没有风险） | careful |
| 抖音收藏、喜欢 | `com.ss.android.ugc.aweme` | 网页有：在真实页面里取「喜欢与收藏」（browser-session） | `v.douyin.com` 短链 | 低：网页侧页内请求已经是最稳的一层 | careful |
| B站收藏夹、稍后再看 | `tv.danmaku.bili` | 网页有：扩展导出、yutto | `b23.tv` 短链，展开是 BV 号 | 低 | 没标 |
| 视频号「赞和收藏」 | 在微信 `com.tencent.mm` 里 | Windows 微信有；Mac 待核实（问题 4） | `weixin.qq.com/sph/<短码>` | 只在 Mac 微信没有这两个列表时才需要；主屏上能不能读是推断 | careful（微信） |
| 微信收藏 | `com.tencent.mm` | 电脑版微信有（[§3.1](#31-电脑端)） | 文章是 `mp.weixin.qq.com` 链接；笔记、图片、聊天记录没有公开链接 | 和电脑版微信视觉层同一档风险，微信协议明文禁止读界面（[06 §1.2](06-platform-access-and-account-risk.md#12-视觉层的风险从哪里来)） | careful |

<a id="4-对插件协议的影响"></a>
## 4. 对插件协议的影响

这一节只是建议，[插件协议](../plugin-protocol.md#思鹭插件协议-v1草案)本身由别的写入者改。「谁定」一列：**我们定**是技术做法，照 [插件协议 §11](../plugin-protocol.md#11-设计取舍) 的口径写进设计取舍；**问你**的对应 [§0](#0-结论) 的问题。

| 协议小节 | 现在 | 建议 | 谁定 |
|---|---|---|---|
| [§2.3](../plugin-protocol.md#23-获取插件的字段) 能力声明 | 没有说明这次调用会不会干扰用户 | 加 `interaction`：`background`（默认，只读，不弹菜单、不改剪贴板、不抢焦点、不动指针）或 `foreground`（上面任何一样会发生）。宿主只拿它排程和加锁，不拦截、不弹确认。`devices` 说的是要什么权限，说不出这次调用会不会干扰你 | 我们定 |
| [§2.3](../plugin-protocol.md#23-获取插件的字段) 能力声明 | 获取插件没有模型字段 | 加 `provider`：`none`（默认）、`optional`、`required`，含义同 [§2.9](../plugin-protocol.md#29-转换插件的字段)；视觉层插件流程对不上时可以让看图模型找一个控件，找到后回树里核对 | 我们定 |
| [§2.3](../plugin-protocol.md#23-获取插件的字段) `pacing` | 只有 `batch`、`interval`、`daily` | 说明里加视觉层的参考值，出处是 Mika（[§2.3](#23-读屏与输入)）：换一屏停 2.5–5 秒、按字数每字加 15 ms、一次至多换 24 屏；这些是一次调用内部的节奏，`interval`、`daily` 另按平台给 | 我们定 |
| [§2.4](../plugin-protocol.md#24-access-与-login) access 与 login | 没说浏览器、Electron 目标和安卓的几种做法算哪层 | 规则里加两条。一，目标是浏览器或 Electron 客户端时：只读无障碍树或截图、动作走无障碍接口或系统输入，记 `vision-ui`，读日常浏览器的网页也算；经 CDP、远程调试、扩展或页面脚本读 DOM 或请求，记 `browser-session`；带调试端口重启官方客户端、或在它的渲染进程里执行脚本，记 `client-hook`。二，经 ADB 或手机上的程序读控件树、截图、OCR 再发输入，记 `vision-ui`；开后台虚拟屏、以 shell 身份起助手不改变层级；只有注入或 Hook 目标 App 的进程、读它的私有数据才是 `client-hook` | 我们定 |
| [§2.5](../plugin-protocol.md#25-登录站点-sites) 登录站点 | `credential: app` 表示驱动已登录的客户端或手机 | 写明 `site` 填这个平台账号的站点（`weixin.qq.com`、`www.douyin.com`），和浏览器插件写同一个 `site` 没关系，`credential` 不同；登录状态由插件在 `health` 的 `login[]` 里报。可选加 `apps`（例如 `[{os: android, id: com.xingin.xhs}]`），宿主连上设备后先用 `pm path` 查装没装，没装就给 `needs: ["app:com.xingin.xhs"]`；加不加在实现时定 | 我们定 |
| [§2.7](../plugin-protocol.md#27-权限-permissions) `devices` | `adb`：「经 ADB 驱动 Android 手机」 | 不加新值，会不会动指针由 `interaction` 说清。`adb` 的说明改成「经宿主管的 adb 服务操作你选定的一台 Android 设备；adb shell 能做的都能做：读屏幕和控件树、截图、模拟点按、读剪贴板、开虚拟屏、推送并运行程序、装卸 App」，权限页照这句写。安卓上的读屏、截图、剪贴板都经 `adb`，不另声明 `accessibility`、`screen-capture`、`clipboard`（这三项指电脑本机）；插件推到手机上的文件、装的东西、改的设置，要在 README 列出并在退出前还原 | 我们定 |
| [§2.7](../plugin-protocol.md#27-权限-permissions) `exec` | 缺省等于 `requires.runtime` 里的名字 | 加一句：`${PLUGIN_DIR}` 里的可执行文件（插件自带的工具和它们再拉起的 helper）总是可以执行，不用写进 `exec`；`exec` 只管安装目录以外的程序。原因：agent-desktop 会自己起剪贴板 helper，请求权限时还会重启自己 | 我们定 |
| [§2.8](../plugin-protocol.md#28-安装与启动) 安装与启动 | 资产是插件自己的可执行文件 | 视觉层插件自带固定版本的 agent-desktop 发布包，只打包对应平台的那一份（v1.1.0 每个平台 1.65–3.43 MB，npm 包是 15 MB），不用 `requires.runtime`、不让用户 `npm i -g`。插件仓库的 CI 先核 `checksums.txt`，再跑 `gh attestation verify <archive> --repo vercel-labs/agent-desktop`，通过才打进资产；`upstream` 写版本和 Apache-2.0，资产里带上从仓库取的 LICENSE。升级 agent-desktop 就发一个新的插件版本。推到手机上的文件（scrcpy-server、自己的助手）放在电脑端资产里一起校验，不在运行时另下载；GPL 的手机端组件只让用户自装。运行时表加 `adb`、`scrcpy`。三个以上插件都带 agent-desktop 时，再考虑做成宿主管的共用运行时 | 我们定 |
| [§3.1](../plugin-protocol.md#31-运行器与节奏) 运行器与节奏 | 每个平台一把单调用锁 | 加三把锁：`interaction: foreground` 的调用全机同一时间只跑一个；声明了 `clipboard` 的调用全机同一时间只跑一个；同一台安卓设备同一时间只跑一个调用，不论哪个插件。设备没连、没解锁、没授权时插件回 `dependency-missing`，跟着收跳过这一轮、不算失败。宿主不存手机的解锁密码，不替你解锁。占屏幕的调用什么时候能自己跑见问题 2 | 锁我们定；自动跑问你 |
| [§3.2](../plugin-protocol.md#32-调用约定) 调用约定 | `waiting` 的原因有 `user-action`、`device-locked`、`device-disconnected` | `waiting` 加 `user-active`（插件在 `foreground` 的步骤之间发现你在用键鼠，停下等你停手，等不到回 `blocked`）和 `device-unauthorized`（手机上还没点「允许 USB 调试」）。环境变量加 `SEELU_PROVIDER`（`provider` 不是 `none` 时）、`ADB_SERVER_SOCKET`、`ANDROID_SERIAL`（声明了 `adb` 时）。包 agent-desktop 的插件自己给子进程设 `AGENT_DESKTOP_HOME`：一次一进程设 `${TMP}/agent-desktop`，常驻设 `${HOME}/agent-desktop`，不用 `${CACHE}`，因为 ref 映射里有界面上的文字 | 我们定 |
| [§3.3](../plugin-protocol.md#33-信封与错误码) 信封与错误码 | 没有「已投递 / 没投递」这一维 | v1 不加：`list`、`fetch` 不改平台状态，`unfavorite` 幂等；以后加了不幂等的写操作再加。插件内部只在 agent-desktop 报 `not_delivered` 时自己重试。agent-desktop 的错误码怎么翻成我们的码写进 plugin-kit 的视觉层 SDK：`PERM_DENIED` → `dependency-missing`（`permission:accessibility` 或 `permission:screen-capture`）；找不到客户端窗口 → `dependency-missing`（`app:wechat`）；窗口是登录或扫码页 → `login-required`；树空或很浅 → `upstream-changed`（有 OCR 退路的先自己退）；界面出现验证、风控、下线提示 → `blocked`；`STALE_REF` 先自己重新快照再试；`POLICY_DENIED` → `internal`（插件的 bug）；快照 `complete: false` 不算错。`needs` 的 `state:` 加 `debugging`（开发者选项或 USB 调试没开）和 `authorized`（这台电脑没被手机授权） | 我们定 |
| [§3.4](../plugin-protocol.md#34-超时与取消) 超时 | `list` 每页默认 60 秒 | 视觉层插件在能力里写 `timeout: 10m`，每处理一条发一次 `progress`，靠 120 秒「无进度」发现卡死；先发 `progress` 再取快照，一次快照可能要几十秒 | 我们定 |
| [§3.8](../plugin-protocol.md#38-list-与跟着收) list | 条目要有 `url` 或 `ref` | 视觉层的条目带 `url`（从「复制链接」来），不用屏上位置当 `ref`，新收藏会把位置往后挤；翻屏去重按 `canonicalize` 后的身份比，不按整行文字比；游标记最后一条的链接和已读条数，下一页先在可见的几行里找它，找不到就回顶部重新滚；`actedAt` 留空；一页宜少（推断：一页 5–10 条）。可选加条目字段 `fetchSoon: true`，小红书这类会话内有效的链接让宿主尽快取 | 我们定 |
| [§4.3](../plugin-protocol.md#43-模型与密钥由宿主代发) 模型代发 | 只写转换、导出插件 `run` 参数里的 `provider` | 改成获取插件也适用：一次一进程时经 `SEELU_PROVIDER` 拿到 `{baseUrl, token, model}`，常驻时在 `context.provider`；token 规则不变。[§9.5](../plugin-protocol.md#95-视觉层流程) 里「用户选云端视觉模型时走代发」那句这样才有落点。另写一句：包装自带模型的第三方 Agent（Mika 的 MCP 一类）不在这一条之内，它的模型调用在手机上发生，宿主标不了「离开本机」 | 前半我们定；这类插件收不收见问题 3 |
| [§7](../plugin-protocol.md#7-权限与隔离) 权限与隔离 | 「推断子进程会按负责进程继承这些权限」 | 改成：有第三方旁证（agent-desktop 的代码注释和故障文档说授权记在启动它的 App 上），没有 Apple 文档；系统设置有时也会单独列出子程序。不做按插件分开的 macOS 授权：要让插件进程自己当负责进程得用私有接口，插件二进制多是 ad-hoc 签名，每次升级可能都要重新授权（推断）。实现时先在 Seatbelt profile 里跑一次 `agent-desktop snapshot`，跑不通就照现在的写法，声明了 `accessibility` 的插件在 macOS 上不进 Seatbelt。剪贴板：Apple 给剪贴板加了「程序读之前先问」（[NSPasteboard.AccessBehavior](https://developer.apple.com/documentation/appkit/nspasteboard/accessbehavior-swift.enum)，macOS 26 上还是开发者预览），开了以后弹窗大概率记在宿主上（推断），宿主第一次用到 `clipboard` 时先跟你说清楚 | 我们定 |
| [§7](../plugin-protocol.md#7-权限与隔离) `adb` 一行 | 「部分：经 `exec` 放行 adb，经网络许可放行本机 adb 服务端口」 | 宿主自己起一个 adb 服务：私有端口，这个服务进程的 `HOME` 指向本机配置目录下的子目录，让 `adbkey` 固定（adb 的密钥在 `$HOME/.android/adbkey`，adb 源码 `client/auth.cpp`）。不这样的话，插件的私有 `${HOME}` 每次调用都会生成新密钥，手机每次都要重新点授权（推断）。插件只拿到 `ADB_SERVER_SOCKET=tcp:127.0.0.1:<端口>` 和 `ANDROID_SERIAL=<你选的设备>`，沙箱只放行这个端口；无线调试的配对和连接在宿主界面里做。连着几台设备时插件经同一个服务仍能点名别的设备，所以 macOS、Linux 仍是「部分」。adb 密钥算凭据，0600，不放 `${CACHE}` | 我们定 |
| [§8.3](../plugin-protocol.md#83-校验与来源证明)、[§8.6](../plugin-protocol.md#86-屏蔽清单与下架) | — | 拿 Mika 的恶意 fork（[M#250](https://github.com/wangzan101/Mika/issues/250)）当例子：从山寨仓库装了就中招，说明「只认来源证明」有实际意义 | 我们定 |
| [§9.1](../plugin-protocol.md#91-包装命令行) 包装命令行 | — | 视觉层 SDK 包 agent-desktop 时守这几条：只调 `${PLUGIN_DIR}` 里那份固定版本；检查信封 `version`，主版本不是 2 就回 `internal`；只有能力声明了 `interaction: foreground` 才允许 `--headed`；不用 `launch --cdp`、`session start`、`trace`、`--screenshots`、`skills`，不用 JEV；每条命令都带 `--app`，最好也带 `--window-id` | 我们定 |
| [§9.4](../plugin-protocol.md#94-浏览器扩展与油猴脚本) 扩展与油猴脚本 | 导入插件按自己做的事填 `official-export`，文件来历写进 `description` | 例子里加一句：手机上的 Agent（Mika 一类）导出的清单同样处理，`description` 写「清单由你手机上的某某导出，导出时用的是它自己的模型」 | 我们定 |
| [§9.5](../plugin-protocol.md#95-视觉层流程) 视觉层流程 | 以 Mac 微信收藏为例，`exec: [osascript]` | 例子改成自带 agent-desktop（见下），去掉 `osascript`，加 `interaction: foreground`、`timeout: 10m`，`daily` 改成按每页 10 条算的 30（推断，没有微信那边的阈值）；写明受微信和 macOS 版本限制。再加一个安卓例子（经 ADB 读小红书收藏）：默认主屏，虚拟屏是插件设置，微信和视频号不用虚拟屏；手机剪贴板经 adb shell 读。运行时默认不用模型兜底，设置里可以打开（本机或云端，云端经 [§4.3](../plugin-protocol.md#43-模型与密钥由宿主代发) 代发并标「离开本机」）。P10 的建议要按 [§0](#0-结论) 第 1 条重写 | 例子我们定；P10 问你（问题 1） |
| [§9.6](../plugin-protocol.md#96-许可证) 许可证 | MIT、Apache、BSD 可以打进资产 | 加一句：上游的发布包里没有 LICENSE 文件时（agent-desktop 就是），从上游仓库取一份放进资产 | 我们定 |
| [§11](../plugin-protocol.md#11-设计取舍) 设计取舍 | — | 加这几行：视觉层插件自带固定版本的 agent-desktop；能力加 `interaction`，加全机的屏幕锁、剪贴板锁和设备锁；不做按插件分开的 macOS 授权；信封不加「投递」字段；安卓走电脑经 ADB、不做手机配套 App；宿主自己管 adb 服务和密钥；安卓视觉层默认主屏；获取插件也能经宿主代发用模型。理由和放弃了什么照上表 | 我们定 |
| [§13](../plugin-protocol.md#13-待定) 待定 | 16 项 | 加四项，分别链到 open-decisions 里对应问题 1–3 的条目（P10 改写，P19、P18 新开），和问题 3 末句「包装 Mika 这类插件收不收」新开的 R11 | 问你 |

[插件协议 §9.5](../plugin-protocol.md#95-视觉层流程) 的两个例子可以写成这样（`interval`、`daily`、`adb>=35` 都是示意值）。第一个是 Mac 微信收藏里的公众号文章，`screen-capture` 只给 OCR 退路用：

```yaml
platforms: [wechat-mp]
access: vision-ui
login: required
sites:
  - {site: weixin.qq.com, name: 微信, credential: app}
upstream:
  - {name: agent-desktop, url: https://github.com/vercel-labs/agent-desktop, license: Apache-2.0}
capabilities:
  list:
    access: vision-ui
    login: required
    sites: [weixin.qq.com]
    order: newest-first
    interaction: foreground
    timeout: 10m
    sources:
      - {source: "wechat:favorites:articles", title: 微信收藏里的文章, action: favorite}
pacing:
  wechat-mp: {batch: true, interval: 6s, daily: 30}
permissions:
  devices: [accessibility, clipboard, screen-capture]
  network: {mode: none}
```

第二个是经 ADB 读手机上的小红书收藏：

```yaml
platforms: [xiaohongshu]
access: vision-ui
login: required
sites:
  - {site: www.xiaohongshu.com, name: 小红书, credential: app}
capabilities:
  list:
    access: vision-ui
    login: required
    sites: [www.xiaohongshu.com]
    order: newest-first
    interaction: foreground
    timeout: 10m
    sources:
      - {source: "xiaohongshu:favorites", title: 我在手机上收藏的笔记, action: favorite}
pacing:
  xiaohongshu: {batch: true, interval: 10s, daily: 60}
settings:
  - {key: display, label: 在哪块屏上跑, type: enum, options: [main, virtual], default: main}
permissions:
  devices: [adb]
  network: {mode: none}
requires:
  runtime: ["adb>=35"]
run:
  command: ["${PLUGIN_DIR}/seelu-fetch-xiaohongshu-android"]
  serve: true
```

<a id="5-风险"></a>
## 5. 风险

<a id="51-平台能不能察觉"></a>
### 5.1 平台能不能察觉

下面都是从文档和源码读出来的「App 在技术上能看到什么」，没有实机验证，也没有找到国内 App 据此处罚的报告。证据的总表在 [06 §16](06-platform-access-and-account-risk.md#16-账号风险的证据)，视觉层的风险来源在 [06 §1.2](06-platform-access-and-account-risk.md#12-视觉层的风险从哪里来)。

**电脑端（agent-desktop）。**

| 动作 | macOS | Windows |
|---|---|---|
| 只读快照，原生客户端（微信） | AX 查询由目标 App 自己的进程来答，App 在原理上能知道自己被查了（推断）；没找到 Mac 微信据此处理的资料。macOS 26 加 4.1.x 上聊天界面先不给树，和察不察觉无关 | 目标窗口会收到 UIA 的查询（推断）；PC 微信 4.1 按账号决定给不给树，说明它知道一个账号用没用过无障碍模式（推断，依据 [06 §1.2](06-platform-access-and-account-risk.md#12-视觉层的风险从哪里来)） |
| 只读快照，Electron 客户端（抖音、B站） | 这个 App 的网页无障碍没开过时，第一次快照会把它打开；Electron 有 `accessibility-support-changed` 事件和 `app.isAccessibilitySupportEnabled()`，App 能收到事件、查得到状态（[Electron app 文档](https://github.com/electron/electron/blob/main/docs/api/app.md)）。原来就开着（例如你开着 VoiceOver）时不会多一次变化。写 `AXManualAccessibility` 一定触发这个事件是推断；抖音、B站有没有监听，待核实 | Chromium 发现 UIA 客户端后自己打开无障碍（推断，依据 agent-desktop 的 Chromium 文档） |
| 只读快照，日常浏览器的网页 | 浏览器那一层能知道（同 Electron）；网页脚本能不能直接读到有没有辅助技术，没找到公开接口（推断） | 同左 |
| 滚动 | headless 改滚动条的值或发翻页动作，App 看到位置变了却没有滚轮事件（推断）；`--headed` 发 `CGEvent`，事件里带来源进程 PID，没找到客户端读它的资料（[06 §16.1](06-platform-access-and-account-risk.md#161-平台规则与可检测性)） | `--headed` 才能物理滚，`SendInput` 的事件在低级钩子里带 `LLMHF_INJECTED` |
| 点击、右键 | headless 走 `AXPress`、`AXShowMenu`；在网页里，前面没有 pointerdown、mousemove 就直接来一个 click，网页的风控脚本也许看得出不像人点的（推断，待核实） | 右键只能 `--headed`，带注入标志 |
| 节奏 | 按频率判断，和用哪一层无关（微信个人账号使用规范 3.4） | 同左 |

公开报告：Mac 微信的几个无障碍工具 issue 里搜「下线」「封」「ban」「logout」都是 0；agent-desktop 自己的 issue 里没有账号相关的报告；PC 微信用 pywinauto 读界面有强制下线、被禁止登录电脑的记录（[06 §16.2](06-platform-access-and-account-risk.md#162-issue-里的账号问题)）。没有报告不等于没有风险：微信协议 8.2.1.6 明文禁止读取界面，按 11-01 定的方向，风险由你自己担。

**安卓端。**

| 手段 | App 能看到什么 | 出处 | 用在 |
|---|---|---|---|
| USB 调试、开发者选项的开关 | developer.android.com 写这两项「always return 0 for all third-party apps」，但 AOSP 里这句和 `@Readable(redactedValue = "0")` 只出现在 `android17-release`，`android16-release`、`android16-qpr2-release` 都没有。遮蔽在调用方 App 自己的进程里做，还要功能开关 `enableRedactedValueForReadable` 打开。所以：Android 17 起、开关打开时读到 0；Android 16 及以前读得到真值；App 直接查 settings provider 能不能拿到真值，待核实 | [Settings.Global](https://developer.android.com/reference/android/provider/Settings.Global)（2026-11-03 取）；`A17:core/java/android/provider/Settings.java`（第 3846–3858、3896–3925 行等） | A、B、C2 |
| 无线调试开关 `adb_wifi_enabled` | Android 16、17 里都是 `@hide` 加 `@Readable`，没有遮蔽值；`@hide` 只表示没有 SDK 常量，App 用字符串键能读到真值（按源码，没实机验证） | `A17:core/java/android/provider/Settings.java` | A、C2；B 用无线调试时 |
| UiAutomation（`uiautomator dump`、uiautomator2） | 连着的时候，系统给所有 App 的状态里置「无障碍已开启」，同时 `getEnabledAccessibilityServiceList` 返回空列表；`uiautomator dump` 连接时不带 `FLAG_DONT_SUPPRESS_ACCESSIBILITY_SERVICES`，就是压制模式。App 看到的是「无障碍开着、却列不出任何服务」，你自己开着的无障碍服务也会被暂时压住。`dump` 只连一下，uiautomator2 的常驻服务整段会话都连着 | `A17:services/accessibility/java/com/android/server/accessibility/UiAutomationManager.java:169-185`、同目录 `AccessibilityManagerService.java:1824-1848`、`AccessibilityUserState.java:473-479` | B |
| `input tap` 注入的事件 | 构造时 `flags` 为 0，不带无障碍的 0x800；派发链路上会不会再加标志没核实；压力和接触面积固定 1.0，可能成为统计特征（推断） | `main` 镜像的 `services/core/java/com/android/server/input/InputShellCommand.java` | B |
| 无障碍服务派发的手势 | 带 0x800 标志；任何 App 都能列出已开启的无障碍服务 | [06 §16.1](06-platform-access-and-account-risk.md#161-平台规则与可检测性) | A、C1 |
| 后台虚拟屏 | Mika 的屏名写死 `mika-agent`、带 `PUBLIC`；scrcpy 的叫 `scrcpy`。公开的 `DisplayManager.getDisplays()` 列得出公开屏，App 也能用 `Activity.getDisplay()` 知道自己不在 0 号屏（推断） | [§2.2](#22-后台屏幕的原理)；[DisplayManager](https://developer.android.com/reference/android/hardware/display/DisplayManager) | A、B（可选）、C2 |
| 微信在虚拟屏上 | 整屏无障碍节点为空，注入的点按不起作用 | [M#237](https://github.com/wangzan101/Mika/issues/237) | A、B（虚拟屏）、C2 |
| 支付类 App | Flyme 的支付保护打开支付 App 时关掉无障碍 | [M#1](https://github.com/wangzan101/Mika/issues/1) | A、C1 |

公开报告：Mika 的 issue 里搜「封号」「封禁」各 3 条，都是设计提案或 PR 里的字，不是账号处罚；「风控」4 条里只有 [M#10](https://github.com/wangzan101/Mika/issues/10)（「AI 太快操作容易导致 APP 被判为机器人操作，从而退出 APP 的登录状态」）算间接证据；「抖音」0 条。抖音、小红书、B站在 Mika 上没有风控或封号报告，这只说明没有报告。

**给 06 的更正**（06 由别的写入者维护，这里只记下）：

- [06 §16.1](06-platform-access-and-account-risk.md#161-平台规则与可检测性) 和 [06 §1.2](06-platform-access-and-account-risk.md#12-视觉层的风险从哪里来) 写「USB 调试开关现在对第三方 App 恒返回 0」，要加限定：只对 Android 17 起、功能开关打开时成立，Android 16 及以前读得到。
- [06 §16.2](06-platform-access-and-account-risk.md#162-issue-里的账号问题) 可以补 Mika 的 #237、#92（微信）和 #1（Flyme 支付保护）。

<a id="52-隐私"></a>
### 5.2 隐私

**电脑端插件这边**（写进 plugin-kit 的视觉层 SDK，契约测试也查）：

- **不留痕**：`AGENT_DESKTOP_HOME` 设在 `${TMP}` 或 `${HOME}` 下，调用结束或进程退出时由宿主删掉；不开会话、不开 trace、不用 `--screenshots`，这些回放产物不脱敏。
- **缩小快照范围**：微信主窗口的快照会带上左边的会话列表，里面有联系人名字和消息预览。进了收藏页以后每次快照都用 `--root` 限在列表容器上，整窗快照只在导航那一步做一次，用完就丢。截图用 `screenshot --app` 截窗口，不用 `--screen`，免得截到别的 App 和通知。
- **截图和 OCR**：截图写在 `${TMP}`，在插件进程里用本机 OCR，用完马上删，不进 `${OUT}`。用云端视觉模型就走宿主代发，界面标「离开本机」。
- **剪贴板**：一页开始前把你原来的剪贴板内容读进内存，读完每条链接就丢，一页结束时写回；全程不写日志、不写盘。剪贴板管理工具（Raycast、Maccy 一类）会把微信复制出来的每条链接都记进历史，写剪贴板的是微信，我们拦不住，界面上要说清楚。
- **日志**：stderr 不写屏上的文字、标题、链接，只写计数和 agent-desktop 的错误码。

**安卓端（安卓-B）**：手机剪贴板经 adb shell 在手机侧读，不经电脑剪贴板；adb 密钥是凭据，0600 放在本机配置目录；插件推到手机上的东西退出前删掉。

**用 Mika（安卓-A）**：收藏页上的字会发给你在 Mika 里配的模型服务商；ML Kit 会给 Google 发使用指标；WebView 调试常开、运行截图不加密落盘（[§2.8](#28-隐私)）。这些都在宿主之外，宿主标不了。

**宿主这边**：诊断日志只放本机配置目录，导出诊断、提交 bug 之前先给你看；插件崩溃时也照样删掉 `${TMP}`；第一次用到 `clipboard` 或 `screen-capture` 时，先说清楚跑的时候会动剪贴板、会截微信窗口、截图只在本机识别并马上删，再请你在系统设置里授权。

<a id="53-供应链"></a>
### 5.3 供应链

**agent-desktop。**

- npm 的安装脚本会执行原生二进制一次、改写 bin 入口，包里的二进制对不上时还会从网上下载；所以不走 npm，插件自带发布包。
- 同一个 release 的 `checksums.txt` 只防损坏、不证明来源；来源要用 `gh attestation verify`（`AD:docs/install.md`「Release archives」）。
- macOS 二进制是 ad-hoc 签名、没有 Team ID：如果系统把授权单独记在二进制上，每次升级哈希都变，可能要重新授权（推断）。Windows 的 exe 没签名，智能应用控制会不会拦宿主启动的它，待核实。
- 提交几乎全出自一位维护者；信封语义还在变（2.2、2.4 都改过），所以锁版本，升级时重跑契约测试。
- npm 包和发布压缩包里没有 LICENSE，分发时要自己带上（[§1.8](#18-许可证)）。

**Mika。**

- 没有 CI，APK 和源码的对应关系证明不了；APK 的签名证书、release 有没有开混淆，本次都没看（没下载 APK）。
- 5 天 36 个发布，几小时一版；公开历史从一个 1,370 个文件的首提交开始；明显用 AI 编程助手开发；作者是个人。
- 预编译的 PRoot 和 31 MB 的根文件系统附了可重现构建脚本，本次没跑校验；发布里缺 GPL 源码包；ML Kit 没写进许可声明。
- 已经有人 fork 后夹带远程执行代码（[M#250](https://github.com/wangzan101/Mika/issues/250)）。用户要用，只从原仓库的 Releases 装。
- 我们的文档要是推荐它，等于替一个不能验证的 APK 背书，所以只写「你自己的工具导出的文件可以导入」，不点名推荐。

<a id="6-待核实清单"></a>
## 6. 待核实清单

**agent-desktop 与电脑端：**

1. Mac 微信 4.1.x 在 macOS 15 上收藏页的树还在不在，每行是什么角色、有没有 ref，headless 的 `AXShowMenu` 和滚动能不能用；本机的 3.8.8.19 能读到什么；macOS 27 加 4.1.15 的情况。下面几条能回答前几问，要先给启动它的终端授予辅助功能（你在系统设置里做），读的是你登录着的微信。第四条会在微信里弹出右键菜单，看完按 Esc 关掉，不会复制东西；其余只读。

   ```bash
   agent-desktop permissions                                   # 只报告，不弹窗
   agent-desktop snapshot --app 微信 --skeleton -i --compact    # 树深不深、有没有「收藏」按钮的 ref
   # 手动点开收藏页后：
   agent-desktop snapshot --app 微信 -i --compact --max-depth 14
   agent-desktop right-click @<snapshot_id>:eN                 # 行有 ref 时再试，应弹出菜单
   agent-desktop snapshot --app 微信 --surface menu -i          # 有没有「复制链接」的 ref
   ```

2. 「4.x 是 Qt 写的」：两份报告指向，未确认。
3. 系统把子进程的授权记在负责的 App 上：只有 agent-desktop 的注释和故障文档作旁证，没查 Apple 文档；ad-hoc 签名的二进制升级后要不要重新授权。
4. 写 `AXManualAccessibility` 是否一定触发 Electron 的 `accessibility-support-changed`；抖音、B站客户端有没有监听。
5. 视频号：Mac 微信有没有「赞和收藏」（问题 4）；agent-desktop 能不能打开 `WeChatAppEx` 的网页无障碍；分享菜单有没有「复制链接」；复制出来的链接元宝认不认。
6. 抖音电脑版：有哪几个列表，Lynx 画的部分进不进树，MetaSec SDK 查哪些信号，有没有关掉远程调试。B站客户端有哪几个列表。
7. 网页里 headless 的 `AXPress` 是不是只发一个 click、前面没有 pointer 事件；网页脚本能不能察觉辅助技术开着；只按滚动条值滚动时，App 看不看得出没有滚轮事件。
8. Seatbelt 下 AX 调用和剪贴板要放行哪些系统服务。
9. 智能应用控制会不会拦未签名的 `agent-desktop.exe`；宿主下载的文件带不带隔离属性。
10. macOS 27 的剪贴板读取提示是不是默认开着，提示记在谁头上。
11. 1.1.0 去掉常驻进程以后，[ad#356](https://github.com/vercel-labs/agent-desktop/issues/356) 里快照超时后锁一直占着的问题还在不在。
12. 截窗口时窗口被挡住，拍到的是不是窗口本身；`--headed` 合成的事件会不会被当成你的输入，影响 `user-active` 的判断。
13. 节奏数字（每条 1–3 秒、`daily: 30`）是推断，没有微信那边的阈值。

**Mika 与安卓端：**

14. Android 17 的 `enableRedactedValueForReadable` 在出厂设备上默认开没开，国内 ROM 有没有跟进；App 直接查 settings provider 能不能拿到真值。
15. 微信在主屏上能不能被 adb 或无障碍正常读、正常点（#237 只测了虚拟屏）。
16. `input` 注入的事件在派发链路上会不会被加上 0x800 一类标志；USB 状态广播里的 adb 位、`init.svc.adbd` 一类系统属性第三方 App 能不能读到。
17. UiAutomation 的「无障碍开着却没有服务」和 `uiautomator dump --windows` 从哪个安卓版本开始有（只读了 `main` 镜像和 `android17-release`）。
18. 虚拟屏名 `mika-agent`、`scrcpy` 能不能被第三方 App 经公开接口看到；国内 App 是否据此判断。
19. 小红书、抖音、B站 App 的控件树能读出多少（资源 ID 是否混淆、有没有自绘或跨端框架的页面）；收藏页是否按收藏时间从新到旧排、屏上有没有收藏时间。
20. 抖音「个人信息下载」包不包含收藏、喜欢；B站收藏夹接口有没有收藏时间。
21. scrcpy 的发行包是否自带 adb，两个版本的 adb 同时用会不会互相重启服务端。
22. 安卓 10 起读剪贴板的限制对无障碍服务 App（安卓-C1）的具体影响。
23. Mika 的 APK 和源码是否对应、签名证书、有没有混淆；PRoot、根文件系统能不能逐字节重现；root 分支的内容；fork 里还有没有别的恶意版本。
24. Mika 用 ChatGPT 账号登录时的 `dynamic_agent_client` 是不是 OpenAI 给第三方本机 App 的正式入口（作者这么说）。
25. 抖音、小红书、B站在 Mika 上的实际成功率和账号后果（没有公开评测结果，issue 里没有报告）。
