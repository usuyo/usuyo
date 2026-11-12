<a id="手机那条路"></a>
# 手机那条路

手机上的 App 替你翻自己存过的列表，一条条点「分享 → 复制链接」，链接交给 usuyo，内容由同一平台不要账号的取法去取（[调研 06 §1.3](../research/06-platform-access-and-account-risk.md#13-混合路径视觉层拿链接公开页面取内容)）。先做 Mac 上的 iPhone 镜像，再做安卓经 adb。各平台用到它的取法列在 [routes.md](routes.md#2-先做的十个)。

依据：2026-11-12 读的 Apple 支持文档 [120421](https://support.apple.com/en-us/120421)、mirroir-mcp v0.47.0 的 README 和 `docs/`、iphone-mirror-mcp 和 phone-harness 的 README，[调研 10](../research/10-desktop-and-android-agents.md)，[协议 §9.5](../protocol/wrapping.md#95-视觉层流程)。这台 Mac 是 macOS 15.6.1，有 `/System/Applications/iPhone Mirroring.app`。没有连过手机，没有运行 mirroir-mcp。App 里的界面文字和层次没有截图核对过，[第 7 节](#7-第一批流程)的锚点都是推断，靠演练模式（[第 6 节](#6-改版以后怎么维护)）定下来。

<a id="1-做什么不做什么"></a>
## 1. 做什么，不做什么

- 只做 `list`：一页处理几条，每条返回一个 `url`，加界面上读到的标题、作者；不返回 `ref`（[协议 §3.8](../protocol/fetch.md#38-list-与跟着收)）。返回的链接和 `/v1/batches` 交来的一样进导入预览。
- 内容不在手机上取。视频号的链接交 `wechat-channels` 的 `feed-info`，小红书交 `note-page`，公众号文章交 `wechat-mp` 的 `article`（[routes.md](routes.md#3-小红书)）。
- 层级 `vision-ui`，`login: required`，`sites[].credential: app`：账号在你手机的 App 里，usuyo 什么凭据都不拿（[协议 §2.5](../protocol/manifest.md#25-登录站点-sites)）。

<a id="2-iphone-镜像能做的和不能做的"></a>
## 2. iPhone 镜像能做的和不能做的

| 方面 | 情况 | 出处 |
|---|---|---|
| 条件 | Mac 是 macOS 15 以上、Apple 芯片或 T2；iPhone 是 iOS 18 以上、设了密码；两边同一个 Apple 账户、开双重认证；蓝牙、Wi-Fi 都开；iPhone 锁着、在 Mac 旁边；Mac 不能在共享网络、AirPlay 或随航；一次只连一台 Mac 和一台 iPhone | Apple 120421 |
| 地区 | 欧盟不可用；文档没有提中国大陆 | Apple 120421 |
| 拿起手机 | 解锁 iPhone 就断开镜像；之后 iPhone 上显示被哪台 Mac 用过，设置里有记录 | Apple 120421 |
| 读屏幕 | 只有画面：镜像窗口的无障碍树里没有子元素，iOS 的界面读不到也点不到；要截窗口再认字 | mirroir-mcp `docs/limitations.md` |
| 截窗口 | ScreenCaptureKit，要「屏幕录制」授权；受保护的视频截出来是黑的 | mirroir-mcp `docs/install.md`；iphone-mirror-mcp README；Apple 120421 |
| 输入 | `CGEvent`，要「辅助功能」授权；macOS 26.5 上点按、打字、滑动不抢前台、不动指针，拖动和双指手势会把镜像临时拉到前台 | mirroir-mcp `docs/limitations.md`（26.5 上测的，15.x 没测） |
| 剪贴板 | 手机和 Mac 之间只有通用剪贴板（接力，同一账户，蓝牙加 Wi-Fi，异步）；mirroir-mcp 的 FAQ 写程序触发的粘贴过不去，限制页写经通用剪贴板能过去，两处说法不一致；iphone-mirror-mcp 的 `read_clipboard` 先在手机上按 ⌘C，再读 Mac 的剪贴板 | mirroir-mcp `docs/faq.md`、`docs/limitations.md`；iphone-mirror-mcp `docs/tools.md` |
| 声音 | 手机的声音从 Mac 放出来；mirroir-mcp 的 `listen` 只录镜像这一个 App 的声音、在 Mac 上转写，要 macOS 26 | Apple 120421；mirroir-mcp `docs/tools.md` |
| 能不能被 App 认出来 | iOS 的 App 能知道自己被录屏、投屏（`UIScreen.isCaptured`）；iPhone 镜像没有公开的识别接口（Apple DTS 2024-07） | [06 §1.2](../research/06-platform-access-and-account-risk.md#12-视觉层的风险从哪里来) |

<a id="3-现有的上游"></a>
## 3. 现有的上游

| 项目 | 许可证 · 版本 | 怎么装 | 能做的 | 用法 |
|---|---|---|---|---|
| [jfarcand/mirroir-mcp](https://github.com/jfarcand/mirroir-mcp) | Apache-2.0 · v0.47.0（2026-11-12） | `brew tap jfarcand/tap && brew install mirroir-mcp`（`install-agent` 不接受 npx 装的副本，所以要 Homebrew）；`mirroir-mcp doctor` 查缺什么；官方 release 有 `darwin-arm64` 包和 `SHA256SUMS` | 42 个工具：截图、`describe_screen`（Apple Vision 认字，默认语言 `en-US`）、按字点（`tap` 带 `text`）、按坐标点、滑动、`press_back`、`launch_app`（在 Spotlight 里打 App 名，没有按键的字经剪贴板粘贴）、`scroll_to`；没有配置文件时只开 14 个只读工具，改东西的工具要写进 `~/.mirroir-mcp/permissions.json`；`mirroir-mcp serve` 只在 127.0.0.1 上讲 HTTP MCP、带 bearer 令牌，`install-agent` 让它以自己签名的「mirroir serve」常驻，授权记在它身上；默认每天向 npm 查一次新版本，`updateCheck: false` 关掉；录下的流程能不经模型回放（`mirroir test`） | 驱动的第一个后端 |
| [nickatnight96/iphone-mirror-mcp](https://github.com/nickatnight96/iphone-mirror-mcp) | MIT · v1.0.1（2026-08-14），9 星 | 见它的 README | 63 个工具，有 `read_clipboard`；没有无障碍树，按认字和模板找元素 | 参考它读剪贴板的做法 |
| [phone-harness/phone-harness](https://github.com/phone-harness/phone-harness) | MIT · 2026-11-09 有推送 | `uv`，Python 3.12 | iPhone 经镜像窗口、安卓经 adb，同一套命令；本机手机走它的命令行，MCP 只给它的云手机 | 参考：iPhone 和安卓共一套接口的写法；不用它的云手机 |
| [vercel-labs/agent-desktop](https://github.com/vercel-labs/agent-desktop) | Apache-2.0 · v1.2.0 | 插件资产 | 读 macOS App 的无障碍树 | 镜像窗口没有树，用不上；Mac 微信那几种取法用它（[routes.md §4](routes.md#4-视频号)） |

<a id="4-驱动和流程分开"></a>
## 4. 驱动和流程分开

**驱动**不认识任何平台，放 plugin-kit 的视觉层 SDK（[协议 §9.5](../protocol/wrapping.md#95-视觉层流程) 说它在那）。**流程**是每个平台仓库里的一种取法（`usuyo-fetch/<平台>` 的 `iphone`、`android`），只做「翻这一个列表、复制链接」一件事。

驱动对流程只给这几样：

| 操作 | iPhone 镜像怎么做 | 安卓怎么做（[第 8 节](#8-安卓经-adb)） |
|---|---|---|
| `open(app)` | `press_home`，在主屏上按图标下的名字 `tap`；不用 `launch_app`，中文名要经剪贴板粘贴 | `am start` 包名 |
| `read()` → 屏上的字和位置 | `describe_screen`，`text_only`；中文要你在 mirroir-mcp 的配置里把 `ocrLanguages` 加上简体中文（写法待核），按字点（`tap` 带 `text`）也靠它 | `uiautomator dump` 的控件树；树拿不到时截图认字 |
| `tap(锚点)` | `tap` 带 `text`；没有字的图标按旁边的字算位置，点 `x`、`y` | `input tap` |
| `swipe(方向)` | `swipe` | `input swipe` |
| `back()` | `press_back` | `input keyevent BACK` |
| `clipboard()` → 新链接 | 读 Mac 的剪贴板（下面） | `adb shell` 读手机的剪贴板 |

**认字用哪一份。** 预装的 `builtin/ocr-vision` 是转换插件（图进、`ocr.json` 出），协议里获取插件调不了转换插件。iPhone 后端用 mirroir-mcp 的 `describe_screen`，它用的也是 Apple Vision；要和 `builtin/ocr-vision` 认出一样的字（同一个 revision、同一组语言），驱动另带一个小的 Swift 程序，用 `builtin/ocr-vision` 的识别代码（MIT，同一个作者），对 `screenshot` 截下的图再认一次。截图写 `${TMP}`，认完就删（[协议 §9.5](../protocol/wrapping.md#95-视觉层流程) 的隐私规则）。

**装和管**（用户 11-12；做到的见[第 12 节](#12-手机页做到的11-12-代码)）。mirroir 不要你到终端里装。设置里有「手机」一页，做成设置那样：装（照[协议的资产](../protocol/distribution.md)取官方 release 的 `darwin-arm64` 包，钉版本、对 `SHA256SUMS`，放在按机器目录里）、屏幕录制和辅助功能各自给没给（给的那一下系统只认你自己点，页上有打开系统设置对应那一栏的按钮）、iPhone 镜像连没连上、试跑一次（演练模式，只读屏不点）、装的版本和有没有新的、卸掉。`install-agent` 不接受 npx 装的副本；接不接受 release 包里的那一份，还没核。

**怎么连 mirroir-mcp。** 装好以后「mirroir serve」常驻在 127.0.0.1 上（`mirroir-mcp install-agent`，由那一页替你跑）。插件按[协议 §9.2](../protocol/wrapping.md#92-包装本机-http-服务)、[§9.3](../protocol/wrapping.md#93-包装-mcp-服务) 当它的 MCP 客户端，地址和令牌是插件设置，令牌 `secret: true`。屏幕录制和辅助功能授给「mirroir serve」，不授给 usuyo。`health` 读它的 `tools/list`：`type_text`、`press_key`、`open_url`、`set_network`、`shake`、`generate_skill`、`run_steps`、`drag`、`long_press` 里有任何一个露出来，回 `dependency-missing`，`hint` 写要你在 `permissions.json` 里只留 `tap`、`swipe`、`press_back`、`press_home`、`scroll_to`；`launch_app` 也关掉。

**链接怎么到 usuyo。** 点「复制链接」之前，驱动记下 Mac 剪贴板的 `changeCount`；点完以后每 200 毫秒看一次，变了就读字符串，取第一个链接；8 秒没变，这一条记「没拿到链接」，接着下一条，连着三条没拿到，回 `upstream-changed`。不往剪贴板写任何东西（写进去的会经通用剪贴板同步到手机）。剪贴板的原内容由宿主在调用前后存、放回（整机锁和这一步还没做，[协议 §3.13](../protocol/fetch.md#313-宿主和-plugin-kit-现在做到的11-11)）（[协议 §3.1](../protocol/fetch.md#31-运行器与节奏)）。读剪贴板的是插件进程，macOS 26 的「程序读剪贴板先问」弹窗记在 usuyo 上（推断，[协议 §7](../protocol/isolation.md#7-权限与隔离)）。

通用剪贴板在你那里过不来时的另一条路：iPhone 上建一个快捷指令，把剪贴板里的链接追加到 iCloud 云盘里的一个文本文件，快捷指令用拉丁字母起名，流程在「复制链接」后用 `launch_app` 打开它（全是有按键的字，不经剪贴板）；这时 `launch_app` 要开；电脑上由收件文件夹接（[协议 §5.2](../protocol/delivery.md#52-监听文件夹)）。收件文件夹现在固定在按机器的 `inbox/`，选不了 iCloud 云盘里的文件夹，这条路等它能选。

**清单片段**（小红书的 `iphone`；`interval`、`daily` 是示意值）：

```yaml
sites:
  - {site: www.xiaohongshu.com, name: 小红书, credential: app}
upstream:
  - {name: mirroir-mcp, url: https://github.com/jfarcand/mirroir-mcp, license: Apache-2.0}
settings:
  - {key: mirroir-endpoint, label: mirroir serve 的地址, type: string, default: "http://127.0.0.1:<端口>"}
  - {key: mirroir-token, label: mirroir serve 的令牌, type: string, secret: true}
  - {key: dry-run, label: 只看不点, type: bool, default: false}
methods:
  - id: iphone
    name: iPhone 镜像
    access: vision-ui
    login: required
    account: own-phone
    sites: [www.xiaohongshu.com]
    interaction: foreground
    capabilities:
      list:
        order: newest-first
        timeout: 10m
        sources:
          - {source: "xiaohongshu:favorites", title: 我在 iPhone 上收藏的笔记, action: favorite}
          - {source: "xiaohongshu:likes", title: 我在 iPhone 上赞过的笔记, action: like}
pacing:
  xiaohongshu: {batch: true, interval: 60s, daily: 20}
permissions:
  devices: [clipboard]
  network: {mode: allowlist, hosts: ["${setting.mirroir-endpoint}"]}
```

屏幕录制和辅助功能在 mirroir serve 身上，清单只写插件自己读的 `clipboard`；声明了 `screen-capture`、`accessibility` 的插件，宿主会请你把授权给 usuyo（[协议 §7](../protocol/isolation.md#7-权限与隔离)）。权限页上要写出「经 mirroir serve 看屏幕、替你点」，`devices` 现在没有对应的写法。一个平台仓库里这一种取法和 `userscript`、`note-page` 并排，各是清单里的一种方法（[协议 §2.3.1](../protocol/manifest.md#231-方法-methods)）。

<a id="5-流程永远不做的事"></a>
## 5. 流程永远不做的事

- 不发帖、不评论、不点赞、不收藏、不取消收藏、不关注、不私信、不转发给人。
- 只点白名单里的字：每一步写明可以点哪几个字（「我」「收藏」「赞过」「分享」「复制链接」一类）和列表里的卡片；要点的位置上读到「赞」「喜欢」「关注」「评论」「发送」「私信」「取消收藏」「删除」「举报」「转发给朋友」，这一步停，回 `upstream-changed`。列表页标题里的「赞和收藏」这类整串字在白名单里写死，不按单字匹配。
- 不打字。mirroir-mcp 那边 `type_text`、`press_key` 不开（[第 4 节](#4-驱动和流程分开)）。
- 不翻推荐流：进 App 以后不停在任何推荐、发现、关注的流上，那一屏只用来找下一步的字；一屏上找不到下一步的字，停。
- 不点开列表以外的东西：不点头像、话题、商品、广告、链接卡片。
- 截图、读到的字不出本机，不交模型。`provider: none`。

<a id="6-改版以后怎么维护"></a>
## 6. 改版以后怎么维护

**流程是数据。** 每个流程是仓库里的一个文件，一步一项：

| 字段 | 例子 |
|---|---|
| `expect` | 这一屏一定有的几段字：`["收藏", "赞过"]` |
| `tap` | 要点的字，或「某段字右边 40 点」这样相对一段字的位置 |
| `allow` | 这一步白名单里的字 |
| `next` | 点完以后下一屏要看到的字，等 3 秒，看不到就停 |
| `variants` | 同一步在不同版本上的写法，按 `expect` 先对上的那个走 |

定位只用字，不用写死的屏幕坐标；没有字的图标用离它最近的那段字定位置，点完以后用 `next` 核对。对不上回 `upstream-changed`，`hint` 写停在哪一步、读到了哪几段字；屏上出现验证、风控、下线一类的字回 `blocked`（[协议 §9.5](../protocol/wrapping.md#95-视觉层流程) 的错误对照）。

**演练模式**（设置 `dry-run`）：不点任何东西。你自己把手机翻到某一屏，插件读这一屏，回报它认为这是流程的哪一步、`expect` 对上了几段、每个 `tap` 的目标找没找到、列表里读到几张卡片。App 改版以后先用它看，改流程文件，不改代码。回报只写在这次调用的结果里，不进日志。

**测试。** 每一步的截图和 `describe_screen` 回答存成样例（截自你的手机，去掉别人的名字和头像），插件的测试对着假的 mirroir 服务回放；mirroir-mcp 自己也有假镜像的测试做法（`docs/testing.md`）。

<a id="7-第一批流程"></a>
## 7. 第一批流程

**节奏。** 每换一屏停 2.5–5 秒，随机；每屏读完再加按字数每字 15 毫秒；一页最多 8 条（出自 Mika 给 careful 一类 App 的数，[调研 10 §3.2](../research/10-desktop-and-android-agents.md#32-安卓端)）。一条约四屏，十五到二十几秒。

**什么时候跑。** `interaction: foreground`。只在你点「开始」时跑：协议默认给 `foreground` 的是 `idle`，手机这几种取法默认 `manual`，要协议补一句「占着手机的调用默认 `manual`」（[协议 §3.1](../protocol/fetch.md#31-运行器与节奏) 现在只对安卓设备这样写）。每一步之间看一次你最近有没有动键盘鼠标，动了就发 `waiting`（`user-active`），等你停手，30 秒等不到回 `blocked`；mirroir-mcp 发的事件算不算进去，待核。你拿起或解锁 iPhone，镜像断开，插件回 `dependency-missing`；`needs` 的 `state:` 现在只有 `unlocked` 这类值（[协议 §3.3](../protocol/fetch.md#33-信封与错误码)），「手机要锁着」要加一个值。

<a id="71-视频号赞和收藏"></a>
### 7.1 视频号「赞和收藏」

| 步 | 这一屏找什么（推断） | 做什么 | 下一屏要看到 |
|---|---|---|---|
| 1 | 底栏「微信」「通讯录」「发现」「我」 | 点「发现」 | 「视频号」 |
| 2 | 「视频号」 | 点「视频号」 | 视频号首页顶栏（会自动播放，不在这一屏停） |
| 3 | 顶栏右侧的人像图标（没有字，按顶栏的字定位置） | 点 | 你的视频号主页，有「赞和收藏」或「赞过」「收藏」 |
| 4 | 「赞和收藏」 | 点 | 卡片网格 |
| 5 | 卡片网格 | 点下一张没处理过的卡片 | 视频页 |
| 6 | 分享图标（没有字，按右侧的数字和字定位置） | 点 | 分享面板，有「复制链接」 |
| 7 | 「复制链接」 | 点，等剪贴板变 | 面板收起 |
| 8 | — | `back()` 回网格；处理完这一屏的卡片就向上滑一屏，前后两屏有重叠，按链接去重 | 卡片网格 |

链接应是 `weixin.qq.com/sph/<短码>`；分享菜单里有没有「复制链接」，是你还没答的那一问（[BUILD §7](../BUILD.md#7-只有用户能答的和能验的)）。卡片上没有标题，`title` 留空，由 `feed-info` 取回作者和描述。

<a id="72-小红书收藏"></a>
### 7.2 小红书收藏

| 步 | 这一屏找什么（推断） | 做什么 | 下一屏要看到 |
|---|---|---|---|
| 1 | 底栏「首页」「消息」「我」 | 点「我」 | 「笔记」「收藏」「赞过」 |
| 2 | 「收藏」 | 点「收藏」；里面再有「笔记」「专辑」时点「笔记」，照 yichen-skills 的收藏页规则要两级都选中才开始（[06 §3](../research/06-platform-access-and-account-risk.md#3-用户点名的项目)） | 卡片网格，卡片下有标题 |
| 3 | 卡片网格 | 记下这张卡的标题和作者，点开 | 笔记页 |
| 4 | 右上角的分享图标（没有字，按作者名那一行定位置） | 点 | 分享面板，有「复制链接」 |
| 5 | 「复制链接」 | 点，等剪贴板变 | 面板收起 |
| 6 | — | `back()`；这一屏处理完向上滑一屏，按链接去重；连着两屏没有新链接就停 | 卡片网格 |

剪贴板里是一段分享文字加 `xhslink` 短链，宿主从文字里取第一个链接（[协议 §5.1](../protocol/delivery.md#51-本机-http-接口)）。短链展开后的 `xsec_token` 几小时失效，这一页的链接返回后宿主当天取完；条目字段 `fetchSoon` 在[调研 10 §4](../research/10-desktop-and-android-agents.md#4-对插件协议的影响) 提过，协议还没收。「赞过」同一个流程，第 2 步点「赞过」。

<a id="73-微信收藏里的文章"></a>
### 7.3 微信收藏里的文章

| 步 | 这一屏找什么（推断） | 做什么 | 下一屏要看到 |
|---|---|---|---|
| 1 | 底栏「我」 | 点「我」 | 「收藏」 |
| 2 | 「收藏」 | 点 | 收藏列表，顶上有搜索或分类 |
| 3 | 分类里的「链接」 | 点 | 只有链接类的列表 |
| 4 | 列表的一行 | 记下这一行的标题，点开 | 文章页，右上角「…」 |
| 5 | 「…」 | 点 | 菜单，有「复制链接」 |
| 6 | 「复制链接」 | 点，等剪贴板变 | 菜单收起 |
| 7 | — | `back()`；滑一屏，按链接去重 | 收藏列表 |

只留 `mp.weixin.qq.com` 的链接，交 `wechat-mp` 的 `article`；别的链接照原样交，由运行器按 `match` 分给对应的仓库。

<a id="8-安卓经-adb"></a>
## 8. 安卓经 adb

同一套驱动操作，换一个后端，做法照[协议 §9.5](../protocol/wrapping.md#95-视觉层流程) 例子二和[调研 10 §3.2](../research/10-desktop-and-android-agents.md#32-安卓端)：宿主管 adb 服务和密钥（[协议 §7](../protocol/isolation.md#7-权限与隔离)），插件拿 `ADB_SERVER_SOCKET` 和 `ANDROID_SERIAL`，`devices: [adb]`，手机上不装 App、不开无障碍服务，在主屏上跑。

和 iPhone 不同的地方：

- **读屏用控件树。** `uiautomator dump` 给出每个控件的字、资源 id 和位置，锚点可以写资源 id，比认字准；树要等屏幕安静 1 秒，10 秒等不到就失败，自动播放的视频页改用截图认字。流程文件里同一步按系统分 `variants`。
- **剪贴板在手机上读**，经 `adb shell`，不用 scrcpy 的剪贴板同步（它会盖掉 Mac 上的剪贴板）。
- **App 看得到的东西更多。** USB 调试开关在 Android 16 及以前读得到真值；`uiautomator dump` 连着时，系统对所有 App 报「无障碍开着、却列不出服务」；`input tap` 的压力和接触面积固定是 1.0（推断）（[调研 10 §5.1](../research/10-desktop-and-android-agents.md#51-平台能不能察觉)）。
- **要在第二台设备上登录。** 小红书、抖音、快手、B站在安卓手机上要另登录一次，算一台新设备。调研里记下的是新注册的号比老号更容易被拦、IP 跳变在各层都加风险（[06 §1.1](../research/06-platform-access-and-account-risk.md#11-七个层级)）；新设备登录本身的处置没有找到记录。
- **微信一次只登一台手机。** 安卓手机上登录微信，会把 iPhone 上的微信挤下线（推断，微信的设备规则没有在调研里核过）；安卓平板能不能以「平板」身份和 iPhone 同时登录，待核。所以视频号、微信收藏的安卓取法只给把微信装在安卓手机上的人。微信在 Mika 的虚拟屏上整屏读不到（[Mika #237](https://github.com/wangzan101/Mika/issues/237)），微信只在主屏上跑。
- 默认 `manual`，连着电脑时不自己跑（[协议 §3.1](../protocol/fetch.md#31-运行器与节奏)）。

<a id="9-各平台在视觉层上记录到的情况"></a>
## 9. 各平台在视觉层上记录到的情况

| 平台 | 记录 | 出处 |
|---|---|---|
| 微信（视频号、收藏） | 《微信软件许可及服务协议》8.2.1.6 禁止借助第三方软件读取界面、自动化操作；2026-01 点名录屏、截图、辅助服务读取、模拟按键；豆包手机助手以系统级注入操作微信，用户被退出、无法登录；PC 微信用 pywinauto 读群消息 5–10 次被强制下线，2026-08 两个账号被禁止登录电脑；iPhone 镜像操作微信没有报告 | [06 §1.2](../research/06-platform-access-and-account-risk.md#12-视觉层的风险从哪里来)、[pywechat #267](https://github.com/Hello-Mr-Crab/pywechat/issues/267)、[#296](https://github.com/Hello-Mr-Crab/pywechat/issues/296) |
| 小红书 | 用户协议 4.1 禁止自动化收集，没有据此处罚视觉层的案例；网页侧有低频只读被警告、封号；App 侧没有报告；不登录批量取分享链接后，复制链接的账号被下线 | [06 §16.2](../research/06-platform-access-and-account-risk.md#162-issue-里的账号问题)、[XHS-Downloader #305](https://github.com/JoeanAmier/XHS-Downloader/issues/305) |
| 抖音、快手、B站、知乎、贴吧 | 协议字面上禁止自动化收集（抖音 5.1、B站 5.6、知乎二.9），没有找到处罚视觉层的案例 | [06 §1.2](../research/06-platform-access-and-account-risk.md#12-视觉层的风险从哪里来) |
| 手机 Agent 工具本身 | mirroir-mcp 的 22 条 issue、Open-AutoGLM、mobilerun、mobile-use 里没有账号后果的报告 | [06 §16.2](../research/06-platform-access-and-account-risk.md#162-issue-里的账号问题) |

<a id="10-工作包"></a>
## 10. 工作包

| # | 包 | 放哪 | 交付 | 依赖 |
|---|---|---|---|---|
| 1 | 流程运行器 | plugin-kit 视觉层 SDK | 读流程文件；`expect`、`tap`、`allow`、`next`、`variants`；白名单和禁点字的检查；节奏；演练模式；用户在动键鼠时的 `waiting`；错误码照[协议 §9.5](../protocol/wrapping.md#95-视觉层流程)；对着假驱动的测试 | — |
| 2 | iPhone 后端 | plugin-kit 视觉层 SDK | mirroir serve 的 HTTP MCP 客户端（地址、令牌、超时 10 秒）；`tools/list` 检查（[第 4 节](#4-驱动和流程分开)）；`describe_screen` 转成统一的「字和位置」 | 1 |
| 3 | Mac 剪贴板和认字的小程序 | plugin-kit 视觉层 SDK 的资产 | Swift：读 `changeCount` 和字符串；用 `builtin/ocr-vision` 的识别代码认一张图；都只读，输出 JSON | — |
| 4 | 假 mirroir 服务和样例 | plugin-kit `testdata/` | 按步回放截图和 `describe_screen` 回答；样例从你手机截，去掉别人的名字和头像 | 2 |
| 5 | 三个流程 | `usuyo-fetch/wechat-channels`、`xiaohongshu`、`wechat-mp` 的 `iphone` 取法 | [第 7 节](#7-第一批流程)的三个流程文件、清单片段、对着 4 的测试 | 1–4；演练模式跑过一次你的手机 |
| 6 | 安卓后端 | plugin-kit 视觉层 SDK | 宿主 adb 服务的客户端；`uiautomator dump` 解析；手机侧读剪贴板 | 1；宿主的 adb 服务 |
| 7 | 协议补项 | `docs/protocol/` | 一个仓库几种取法的清单写法；占手机的调用默认 `manual`；`fetchSoon` | — |

<a id="11-只有你能验的"></a>
## 11. 只有你能验的

- iPhone 镜像在你那里能不能用（中国大陆的可用性 Apple 文档没写）；接力和通用剪贴板开着。
- 主屏上微信、小红书的图标名能不能按字点到；mirroir-mcp 的 `ocrLanguages` 加上简体中文以后认不认得。
- macOS 15.6.1 上 mirroir-mcp 点按时抢不抢前台、动不动指针（它测的是 26.5）。
- 小红书笔记页的分享面板里「复制链接」的位置和字样；收藏页「笔记」「专辑」两级的字样。
- 微信收藏的分类里有没有「链接」，文章页「…」菜单里的字样。
- 每个流程的演练模式回报；真跑五条后第二天账号有没有提示。
- 有安卓手机的话：USB 调试的授权弹窗，`uiautomator dump` 在小红书收藏页拿不拿得到树。

<a id="12-手机页做到的11-12-代码"></a>
## 12. 手机页做到的（11-12 代码）

守护进程管「手机」那一页的七条路由（[本机接口](../spec/local-api.md#手机)），代码在 `crates/usuyo-core/src/phone/`。只在 macOS 上开（命令行的 `daemon` 配上 `PhoneConfig::system`）；别的系统和没配的时候，这几条回 404 `not-found`，页面说内核不管手机。这台机器上没有下过、跑过真的 mirroir，也没有连过手机：下面每一条只对着假的 release 服务器、假的「mirroir serve」、假的 `adb` 和假的剪贴板计数测过。

- **钉的那一版**：`crates/usuyo-core/data/phone-driver.json`，mirroir-mcp v0.47.0，只有 `mirroir-mcp-darwin-arm64.tar.gz` 一个包（官方 release 没出 Intel 的），2518002 字节，sha256 抄自同一个 release 的 `SHA256SUMS`（11-12 读的 release 接口给的 `digest` 和它一样）。Intel 的 Mac 点「装上」回 422，一句「mirroir 只给 Apple 芯片的 Mac 出了包，这台电脑装不了」。
- **装**：和下模型同一套下载（只走 https、只连公网、照设置里的代理、60 秒没收到字节算断、不超过钉的大小），每次从头下，不续传；按钉的 sha256 核对；包里要正好一个叫 `mirroir-mcp` 的普通文件（`._*` 和 pax 头跳过，别的一律拒），写成 0755，和 `driver.json`（`{version, asset, sha256}`）一起先放进暂存文件夹，再整个换成按机器目录的 `drivers/mirroir-mcp/`（[按机器的目录](../spec/machine.md#按机器的目录)）。哪一步不成都什么也不留下，原来装着的那份不动，页面上「没装上：」后面是一句原因。换新同样；卸掉是整个目录挪开再删。
- **常驻那一步要你做**：`install-agent` 会在登录钥匙串里放一张自签名的代码签名证书（第一次签名钥匙串要你点「始终允许」），再在 `~/Library/LaunchAgents/` 登记登录时启动的后台项；这两样改的是这台 Mac 的设置，守护进程不做。装好以后「mirroir serve」不回话时，页面的 `driver.step` 给一句话和两条命令（`'<驱动目录>/mirroir-mcp' install-agent`、`launchctl bootstrap gui/$(id -u) ~/Library/LaunchAgents/dev.mirroir.serve.plist`）；在跑的版本（`initialize` 报的）和装着的不一样时给重新登记、`launchctl kickstart -k` 的那两条；卸掉以后它还在跑时给 `launchctl bootout` 那一条。`install-agent` 只拒 npx 装的副本（v0.47.0 `UpdateCommand.swift` 的 `InstallChannel`），按机器目录里的这一份它认作 `unknown`，照理会接受，没跑过。
- **授权**：只经「mirroir serve」自己问。`mirroir-mcp doctor` 不用：它查的是启动它的那个进程的授权，由守护进程起就成了 usuyo 的。读 `status` 那句话的开头：`Connected` 时两项都算给了（mirroir 的说明：两项都给了以前 `status` 只说「No window」）；`Paused`（窗口经辅助功能找得到）辅助功能算给了，屏幕录制看 `check_health` 截没截到图；`No window` 而截得到图时，屏幕录制给了、辅助功能没给；别的都是 `unknown`。`check_health` 十秒里最多问一次（它要真截一张）。这几条是照 mirroir-mcp 的源码和说明推的，没有在真的授权上核过。
- **镜像**：`status` 是 `Connected` 回 `connected`，`Not running` 回 `not-set-up`，别的回 `away`；「mirroir serve」不回话时看 `iPhone Mirroring` 这个进程在不在（`pgrep -x`），不在算 `not-set-up`（页面给「打开 iPhone 镜像」），在算 `away`。契约里没有「没开」「看不出来」这两个值，所以「从没设置过」和「这会儿没开」都报成 `not-set-up`。
- **试剪贴板**：每 500 毫秒用 `osascript -l JavaScript` 读一次 `NSPasteboard.generalPasteboard.changeCount`，等 10 秒；只读这个数，不读剪贴板里的字。这样不用另编 Swift 小程序（[第 10 节](#10-工作包)第 3 包里读 `changeCount` 的那一半就不要了）。读不出这个数时回 500 和一句「看不了这台 Mac 的剪贴板变没变」；等的中途读不出时这次试验作废（`clipboard` 省略）并记进日志。
- **演练**：调一次 `describe_screen`（`text_only`、`omit_screenshot`），只取每段字，对着装着的获取插件里 `drives: iphone` 的方法所在插件的 `flows/*.json` 比（流程文件的写法在 `crates/usuyo-core/src/phone/flows.rs` 开头）；屏上一段字里含着要找的字就算对上，至少对上一段、过半才算像。读到的字不存、不进日志，工具报错时也不往外带它的原话。宿主不认识平台，所以[第 7 节](#7-第一批流程)的三个流程不进这个仓库，只在测试样例 `crates/usuyo-core/testdata/phone/flows/` 里；真的流程要由各平台仓库的 `iphone` 取法带上。`cards`（列表里读到几张卡片）还没有。mirroir 的 `screenDescriberMode` 默认 `auto`，v0.47.0 的源码里 `auto` 走本机 OCR，不走看图模型。
- **安卓**：在 `PATH`、`/opt/homebrew/bin`、`/usr/local/bin`、`~/Library/Android/sdk/platform-tools` 里找 `adb`，`adb version` 5 秒、`adb devices -l` 10 秒；序列号长于 10 位的只留头尾四位。第一次 `adb devices` 会让 adb 起它自己的服务（5037 端口）并留着；宿主自己管的 adb 服务（[协议 §7](../protocol/isolation.md#7-权限与隔离)）还没做。
- **打开系统设置**：守护进程用 `open` 打开屏幕录制、辅助功能那一栏（`x-apple.systempreferences:` 的两个地址）或 `/System/Applications/iPhone Mirroring.app`，只有这三样。壳里另有同样三样的 `open_settings_pane` 命令，页面现在走的是守护进程这条。

第一次真装时页面会是这样（推断）：点「装上」→「在装 mirroir v0.47.0」→ 装好，「驱动」那一行多出要你在终端里跑的两条命令；跑完、钥匙串点了「始终允许」以后，「mirroir serve」第一次被 `status` 问到时，系统会弹辅助功能、第一次截图时弹屏幕录制的框，要点「打开系统设置」、不要点「拒绝」；给了屏幕录制以后要重启它（`launchctl kickstart -k gui/$(id -u)/dev.mirroir.serve`）。没写 `~/.mirroir-mcp/permissions.json`，它只开只读的工具，演练够用；它默认每天问一次 npm 有没有新版本，`updateCheck: false` 关掉，也没替你写。

