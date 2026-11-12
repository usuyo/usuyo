<a id="调研-11--订阅兼容与-wx-cli"></a>
# 调研 11 — 订阅、兼容与 wx-cli

状态：草案 · 2026-11-04

回答 11-04 的三句话：「然后再看看wx-cli」「然后我希望兼容性和开创性都需要具备，后续甚至可以支持那种，rss，Thysrael/Horizon这种」「然后之前调研的所有的方案都可以作为一种插件去弄」。要你定的事在 discussion [第 1 条](../answers.md#1-什么东西进资料库)、[第 9 条](../answers.md#9-自己的获取插件组织收什么)、[第 17 条](../answers.md#17-兼容和开创各指什么)；11-04 的回复记在 [open-decisions §1.7](../answers.md)。

<a id="约定"></a>
## 约定

- **查询日期**：全部是 2026-11-04。仓库状态、★、fork、issue 用 `gh api` 查；DMCA 通知原文用 `gh api repos/github/dmca/contents/<路径>` 取；npm 用 `npm view`。★ 会变，写的是当天的数。
- **源码**：下面几个提交浅克隆到临时目录，只读。没有构建，没有运行任何程序，也没有安装任何东西。
  - jackwener/wx-cli-again `077a54c`；pandorafuture/wx-cli `2abe708`；hum0r23/wx-cli `6dc57e6`；r266-tech/wx-cli `d833d68`；Thysrael/Horizon `d7406f7`。
  - `仓库:路径:行` 指这些提交里的文件，`#N` 指该仓库的 issue。
- **没做的**：没读微信或任何 App 的数据目录，没碰系统设置，没在 GitHub 上做写操作。r266-tech/wxkey 的 README 第 7 行有一句写给 AI 助手的话，按数据处理，没有照做。
- **取密钥那一步只写用户要做什么、要授予什么**，不写做法，同 [06 约定](06-platform-access-and-account-risk.md#约定)。
- **只列事实，不是法律意见。**
- **推断**是我们的判断，没有直接出处；**待核实**是没查到，或者不运行就确认不了。
- **名字**：正文写「本产品」，标识写 `{name}`，例如 `{name} add`。插件协议里的 `seelu`、代码里的 `xiecun` 都是旧占位。
- **用词**：本产品的动作叫收下、跟着收；「订阅」只指阅读器里的订阅和 feed 本身。「路过」是这次拟的词，没定。

<a id="0-结论"></a>
## 0. 结论

1. **wx-cli 推断指 jackwener/wx-cli。** 原仓库 2026-07-15 起因腾讯的 DMCA 通知返回 451。通知以反规避条款（17 U.S.C. 1201）为主、服务条款为辅，没有主张商标；只点名这一个仓库，GitHub 按 1,464 个仓库的派生网络处理。2026 年针对微信本地数据的通知共 37 份。同作者的 wx-cli-again 是真代码，但只是 0.6.3 的单提交快照。见 [§1.1](#11-是哪个-wx-cli)、[§1.2](#12-dmca-通知)。
2. **你的电脑现在用不上。** 这类工具都只支持微信 4.x，本机是 3.8.8.19。层级是 `client-hook`，登录必须。见 [§1.4](#14-收藏能拿到什么)、[§1.5](#15-层级)。
3. **建议：你的 `{fetch-org}` 不放微信专用的 client-hook 插件**（[R5](../protocol/distribution.md#85-添加来源更新与权限扩大) 的 B）。
   - 理由有三条：用户要改的系统设置多；上游不稳；账号会被处理（WeFlow #411）。
   - 「包装也会被点名」不是理由：只调用用户自装工具的仓库，到今天一个都没被点名。
   - 官方组织按 P3 放一个和平台无关的「链接清单导入」插件，读 JSON、CSV 链接清单。
   - client-hook 工具导出的文件按 `official-export` 导入。这是协议写定的，不是待定问题。
   - 见 [§1.8](#18-建议)。
4. **Horizon 是跑一次、出一份日报的批处理工具。** 它不存条目状态，维护者两次拒绝把它变成服务。接它就当一个 feed：读它发布的 Atom，或它写出的 Markdown 日报。包装它的 MCP 不划算。它的条目是模型挑的，收下时要标出来。见 [§2](#2-horizon)。
5. **订阅和资料库的边界，建议 A。** 收下是你拿的，feed 条目是自己来的。
   - feed 在你已经在用的阅读器里读，在那里加星就算收下；本产品跟着收阅读器的星标，不做订阅列表。
   - 外加一小条：你点名的低频个人来源（一个博客、一个 Mastodon 或 Bluesky 账号）可以一个一个跟着收。第一次拉只记下已有的、不收。
   - 你自己平台收藏夹的跟着收照旧。
   - 见 [§3](#3-订阅和资料库的边界)。
6. **兼容靠标准，不靠自己做阅读器**：RSS、Atom、JSON Feed、OPML、Google Reader API、MCP、带 front matter 的 Markdown、Obsidian vault。见 [§4](#4-兼容矩阵)。
7. **开创性，今天站得住的有五条**：Agent 的改动先提议、可以整批撤销；raw 和 derived 分开、索引可以重建；导出以后能读回；清单里用七个获取层级当标签；Keiyoushi 式的仓库分法。迷雾地图和注意力模型只是方向，要原型。见 [§5](#5-开创性)。
8. **之前调研的方案都能做成插件，这在协议上成立**：七层都能写成插件，client-hook 也能。按建议走要补 9 处协议；wx-cli 那种包装另缺 3 处，只有别人写第三方插件时才用得到。见 [§6](#6-对插件协议的影响)。

---

<a id="1-wx-cli"></a>
## 1. wx-cli

<a id="11-是哪个-wx-cli"></a>
### 1.1 是哪个 wx-cli

你没给链接。**推断是 jackwener 的 wx-cli**：同名的里面只有它被 DMCA 点名，也在 06 里出现过；作者也写了 06 里的 OpenCLI 和 wechat-article-to-markdown。

| 仓库 | 状态（11-04） | ★ / fork | 语言 / 许可 | 是什么 |
|---|---|---|---|---|
| [jackwener/wx-cli](https://github.com/jackwener/wx-cli) | 451，2026-07-15T15:39:40Z 起按 DMCA 屏蔽 | — | — | 原仓库。npm 上的 `@jackwener/wx-cli` 0.3.0（2026-05-15）和 5 个平台包仍能装，没标 deprecated |
| [jackwener/wx-cli-again](https://github.com/jackwener/wx-cli-again) | 200 | 897 / 1,793 | Rust / Apache-2.0 | 自称从 wx-cli 重新开始。有真代码：76 个文件，`.rs` 合计 16,973 行，`Cargo.toml` 是 0.6.3。只有 1 个提交（2026-09-14），0 个 release。README 说后续代码在私有仓库 botiverse/wx-cli，查是 404 |
| [pandorafuture/wx-cli](https://github.com/pandorafuture/wx-cli) | 200 | 1,027 / 501 | Rust / MIT | 同名不同源。Mac 微信 4.1.7 以上，带本机 REST 和 SSE，不读收藏 |
| [r266-tech/wx-cli](https://github.com/r266-tech/wx-cli) | 200 | 16 / 13 | Go / MIT | 程序名 `wechat-cli`。同作者的 r266-tech/wechat-cli 2026-08-04 被点名，现在 451；这个仓库 2026-09-11 新建 |
| 不挂 fork 关系的拷贝 | 200 | — | Rust | hum0r23/wx-cli（163 个提交，release v0.3.0 带 macOS arm64 和 Windows 二进制）、Steady775/wx-cli、RealMnzn/wx-cli、Markjinli/wx-cli-gui 都带到 0.3.0 的全部历史；dvonascjona/wx-cli 带早期历史。都是 `fork=false`，所以不在被屏蔽的派生网络里 |

xiaotianhu/wx-cli 只是同名：一个扫码登录的远程 Shell 机器人，和收藏无关。

<a id="12-dmca-通知"></a>
### 1.2 DMCA 通知

**2026-07-13 点名 wx-cli 的那份**（[通知原文](https://github.com/github/dmca/blob/master/2026/07/2026-07-13-wechat.md)）：

- **代表谁**：WeChat International Pte. Ltd.、深圳市腾讯计算机系统有限公司、腾讯科技（深圳）有限公司。落款写 MSK 获授权代理。
  - MSK 推断是 Mitchell Silberberg & Knupp：06-29、07-07 两份通知的信头写着 WWW.MSK.COM。待核实。
- **依据有三条**：
  1. **DMCA 反规避条款**：17 U.S.C. 1201 的 (a)(1)(A)、(a)(2)、(b)(1)。主张微信的库结构、密钥派生、内存布局合称「Protected Database Design」，受版权保护；本地库加密是控制访问它的技术措施；提取密钥就是规避，仓库也算提供规避工具。
  2. **服务条款**：诱使用户违反微信、Weixin 的条款，构成故意干扰合同关系，引了 Blizzard v. Ceiling Fan Software。
  3. **GitHub 自己的条款**。
- **没有主张**：没有主张商标，也没说仓库复制了腾讯的代码。
- **用户的数据**：信里承认用户对自己的数据保有合法权利，但仍主张工具违法。
- **范围**：只点名 1 个仓库，要求连 fork、release、附件一起处理，GitHub 按 1,464 个仓库的派生网络处理；还请 GitHub 防止同一人或其他人再上传类似内容。
- **日期**：通知日期是 07-13。屏蔽在 07-15T15:39:40Z，github/dmca 收录这份通知的提交晚 37 秒。

**2026 年全部相关通知**：

- 2026/01–11 共 2,523 个通知文件，按 `wechat|weixin|tencent` 过滤得 41 个：
  - 4 份针对《洛克王国：世界》，和微信无关；
  - **37 份针对微信本地数据**：01-08 一份点名 29 个仓库，按 4,195 个仓库处理；06-29 到 10-13 共 36 份，每份点名 1 个仓库，标题和理由都是同一套。
- 36 份里有 9 份按派生网络处理，每份 104 到 3.2k 个仓库；07-13 一天就有 4 份。
- 36 个点名仓库今天的状态：25 个 451，5 个 404，6 个 200。
  - 200 的 6 个里，4 个只剩 0–15 KB（推断作者已清空）；WeFlow（26 MB）和 caigee-cmd/wechat-insight（9.9 MB）还有内容。
- 和 wx-cli 直接相关的三份：
  - **07-13-3** ylytdeng/wechat-decrypt：wx-cli-again 的致谢说受它启发。
  - **08-04** r266-tech/wechat-cli：理由里引了它自述的「本地解密」。
  - **09-25** bjdr69/docker-wxcli，现在 404：通知引它的自述，它一键部署的整条链里**含取密钥工具本身**。
- **反通知**：2026 年文件名带 counter、retraction 的有 40 个，没有一个和微信、腾讯有关。
- **「防止再上传」看不出在执行**：wx-cli-again 上线 7 周、1,793 个 fork，没被点名；[§1.1](#11-是哪个-wx-cli) 那几个拷贝也不在后续 35 份通知里。这只说明「还没」。
- **npm 上没有对应动作**：`@jackwener/wx-cli` 还能装。

<a id="13-三个工具怎么工作"></a>
### 1.3 三个工具怎么工作

三个工具都是先附加到微信进程、读进程内存拿到密钥，再解密本地库。下表只写你要做什么、它能给什么。

| | wx-cli-again | pandorafuture | r266-tech |
|---|---|---|---|
| 微信版本 | 4.x | 4.1.7+；发版的 v0.7.4 只放行 4.1.7–4.1.8，挡掉 4.1.15（#21） | 4.x |
| 系统 | macOS、Linux、Windows | macOS arm64 | macOS arm64、Windows |
| 你要做的 | 本机终端里用 sudo；「开发者工具」里通常要勾上终端；首次取密钥时微信开着并已登录。不用关 SIP，默认不动微信本体 | 关 SIP，或按 #20 重签微信；一次性设置要 sudo；取密钥时会重启微信 | 首次输一次管理员密码，**密码本身存进钥匙串**，以后缺密钥时无人值守刷新；会在本机另放一份改过签名的微信副本；建议给两个程序完全磁盘访问权限 |
| 密钥放哪 | 明文 `~/.wx-cli/all_keys.json`，解密后的库副本在 `~/.wx-cli/cache/` | — | `~/.config/wxcli/config.json` |
| 收藏 | 有，只有文章类带 url | 没有 | 有，网页类带标题、描述、url |
| 公众号推送 | `biz-articles` | 没有 | 没写 |
| 对外接口 | 命令行，加 Unix socket 上的常驻进程 | 命令行，加本机 REST、SSE（`127.0.0.1:9100`） | 命令行（`{ok, data, error}` JSON 信封），加本机窗口（`127.0.0.1:18789`，带随机令牌） |

出处：wx-cli-again `README.md:127-157`；pandorafuture `README.md:43-51,113-123`；r266-tech `README.md:39`；wxkey `README.md:17,19,65-66`。

<a id="14-收藏能拿到什么"></a>
### 1.4 收藏能拿到什么

**结论：在微信 4.x 上，能拿到收藏里公众号文章和网页的链接；视频号条目的链接拿不到（待核实）。** 视觉层拿不到链接，因为无障碍树里没有 URL 字段（[调研 10 §0](10-desktop-and-android-agents.md#0-结论)）。

- **wx-cli-again**（`src/daemon/query.rs:3867-3963`）：
  - 读 `favorite/favorite.db` 的 `fav_db_item` 表，按更新时间从新到旧排。
  - 只有类型 5（文章）会从内容 XML 的 `<link>` 里取 http(s) 链接。
  - 没有标题字段，只给内容 XML 的前 100 个字。标题要等公众号插件取正文时再拿。
  - 默认 50 条，只有条数上限，没有游标。
- **r266-tech**（`internal/wxparse/wxparse.go:95-109`）：
  - 读同一张表。网页条目只要有 `pagetitle` 就取 `clean_url`，有 URL、没标题的会丢 URL（推断）。
  - 有 `after`、`before` 和分页。
- **视频的类型号两家不一样**：r266 记 4 是视频、20 落到 unknown；wx-cli-again 记 20 是视频、没有 4。两家都没单独标视频号。
- **`biz-articles`** 读公众号推送，给公众号名、标题、url、摘要、封面、时间。
  - 它是「订阅的号推过什么」，不是「你收下了什么」，按 [§3](#3-订阅和资料库的边界) 的 feed 处理。
  - 它有 `--unread` 选项。包装它的插件不该用，免得把未读带进来（推断，原则 4）。
- **版本卡点**：本机是 macOS 15.6.1 加微信 3.8.8.19（[调研 10 §0](10-desktop-and-android-agents.md#0-结论)），三个工具一个都用不上。升到 4.1.x 又会碰到三件事：
  - 调研 10 记的是 Mac 微信 4.1.x 在 macOS 26 上几乎不给界面树（[调研 10 §1.6](10-desktop-and-android-agents.md#16-限制)）；
  - 4.1.15 以后新装的读不到消息（wx-cli-again #3）；
  - pandorafuture 发版挡掉 4.1.15（#21）。

<a id="15-层级"></a>
### 1.5 层级

**`client-hook`，登录 `required`。**

- **为什么是这一层**：附加或读微信进程内存拿密钥、解密本地库，正是 [插件协议 §2.4](../protocol/manifest.md#24-access-与-login) 里 client-hook 的定义。
- **r266 的「只读」不改变层级**：它指的是不写库、不发消息。取密钥那一步照样附加进程；协议规定按实际默认路径填。
- **为什么要登录**：取密钥时微信要开着并已登录，读的也是这个账号的数据。`sites` 的 `credential` 写 `app`（[§2.5](../protocol/manifest.md#25-登录站点-sites)）。
- **和 06 一致**：[06 §8](06-platform-access-and-account-risk.md#8-微信生态) 现有的 wx-cli-again 一行也是 client-hook、必须。

<a id="16-做成插件"></a>
### 1.6 做成插件

| 做法 | 插件做什么 | 仓库里有没有取密钥代码 | 层级 | 协议能不能表达 |
|---|---|---|---|---|
| A 包装你自装的 wx-cli | `list` 调 `wx favorites --json`，把文章类转成只带 `url` 的条目，正文交给公众号插件（[06 §1.3](06-platform-access-and-account-risk.md#13-混合路径视觉层拿链接公开页面取内容) 的混合路径） | 没有，取密钥由你自己跑 | client-hook | 部分能，缺三处，见下 |
| B 导入 wx-cli 导出的文件 | 你自己导出 JSON，拖进来；插件按 [§3.9](../protocol/fetch.md#39-从文件导入) 读 | 没有，只解析 JSON | `official-export` | 能 |
| C 插件自己取密钥 | — | 有，这正是 37 份通知点名的行为 | client-hook | 不能：`devices` 里没有「附加调试器、读别的进程内存」这一项；要 sudo，宿主管不住 |

**B 的层级已经写定。**
- [插件协议 §9.4](../protocol/wrapping.md#94-浏览器扩展与油猴脚本) 规定：导入插件按自己做的事填 `official-export`；文件怎么来的，写进 `description` 和插件 README。
- 所以 B 的 `description` 写：清单由你用自己装的工具导出，导出那一步属于 client-hook。
- 要改成 client-hook，就是一条改协议的提案。而且要把 xhs-vault（browser-session）、手机 Agent（vision-ui）导出的清单一起改，不能只改微信。

**A 卡在三处。** 插件的 `HOME` 是宿主给的私有目录。wx 的配置可以放在工作目录，密钥路径可以写绝对路径，这两样不是问题；被钉死在 `$HOME/.wx-cli` 的只有 socket、pid 和日志（`wx-cli-again:src/config.rs:37,93-123,146-195`）。所以：

1. 插件连不到你在沙箱外起好的常驻进程，会在沙箱里另起一个；
2. 另起的后台进程会脱离进程组，宿主「一次一进程」结束时收不收得掉，待核实；
3. 协议里没有「连本机 Unix socket」的权限项；`fs.read` 还要覆盖你的密钥文件和微信的容器目录，macOS 读别的 App 的容器可能另弹授权（推断）。

<a id="17-风险证据"></a>
### 1.7 风险证据

**下架：被点名的是带取密钥代码的仓库。**

- bjdr69/docker-wxcli 被点名，是因为它部署的东西里含取密钥工具。
- 只调用你自装的 wx-cli、本身不带取密钥代码的仓库，到 11-04 一个都没被点名：
  - JimLiu/baoyu-skills，27,246 ★，11-02 还有推送；
  - godisgod3/mac-wechat-text-exporter-docx；
  - Ethan9123/wx-life-analysis。
- 这只说明「还没」。WeFlow 07-07 被点名，按 3.2k 个仓库处理，现在仍可访问（[06 §16.3](06-platform-access-and-account-risk.md#163-下架停更与法律风险) 已记）。

**账号：取密钥那一步就可能被处理。**

- **[hicccc77/WeFlow #411](https://github.com/hicccc77/WeFlow/issues/411)**，2026-03-11 开，这条 06 里还没有。
  - API 记 109 条评论（分页取到 110 条），65 条提到警告、风险、封、限制或异常。
  - 协作者当天说，经确认是获取密钥时被记录；仓库主 03-14 说自己被警告，是因为 Hook 过微信。
  - 这些都是开发者的判断，腾讯没有公开说法。
  - 一位用户描述的流程：Windows、WeFlow 3.0.3、导出一个联系人的聊天为 JSON。推断：只读本地库、不发消息，也会因为取密钥那一步被处理。
- **[pandorafuture/wx-cli #16](https://github.com/pandorafuture/wx-cli/issues/16)**（2026-08-23）：
  - 用户说用了一两个小时就被临时限制，按「学习规则」解封。
  - 贴的日志和 jackwener wx-cli 0.3.0 的输出字符串逐字一致，是代码比对的结果（`hum0r23-wx-cli:src/cli/init.rs:48,77`）。
  - 那次取到 0/2 个密钥，账号仍被限制。单个案例；推断受限出在附加和读内存那一步，和解没解出库无关。
- **平台规则**：微信 2026-01 点名的违规类型里有「解密本地数据库」（[06 §16.1](06-platform-access-and-account-risk.md#161-平台规则与可检测性)）。
- **反规避的先例帮不上**（推断）：youtube-dl 当年恢复，是因为证明了没有规避技术措施；这类工具的 README 直接写着「解密」。国内法见 [06 §16.3](06-platform-access-and-account-risk.md#163-下架停更与法律风险) 的法律表。
- **本机安全**（推断，和原则 5 有关）：密钥明文放在一个文件里，socket 对同一用户的任何进程开放，r266 把管理员密码存进钥匙串。装了这类工具，同一用户下的任何程序都能读全部聊天。

<a id="18-建议"></a>
### 1.8 建议

1. **`{fetch-org}` 不放微信专用的 client-hook 插件**：A、C 不放，B 也不做成微信专用的。和 [R5](../protocol/distribution.md#85-添加来源更新与权限扩大) 的建议 B 一致。理由有三条：
   - **系统改动大**：sudo；开发者工具授权；关 SIP 或重签微信；管理员密码进钥匙串。
   - **上游不稳**：
     - 公开代码是单提交快照，后续在私有仓库；
     - 有版本白名单；
     - 密钥在内存里的位置会变（Linux 4.1.x 取密钥 0 命中，wx-cli-again #7）；
     - 收藏 XML 的字段名一改，url 就没了（推断）；
     - 0.3 到 0.6 改过输出外壳，插件得锁版本，B 也一样。
   - **账号风险**：WeFlow #411、pandorafuture #16。
2. **官方组织放一个和平台无关的「链接清单导入」插件**，按 [P3](../answers.md)：
   - 读任何工具导出的 JSON、CSV 链接清单，`official-export`、`login: none`、`network: none`，现行 [§3.9](../protocol/fetch.md#39-从文件导入) 就够。
   - 文档写「微信收藏可以用你自己装的工具导出成清单再导入」，不点名、不链接 wx-cli。这样官方组织里没有微信专用的东西（推断）。
3. **你自己装第三方插件照常能装**：A 这类别人写的插件，按 [D-b](../answers.md) 和 [R-i](../answers.md) 能装。你 11-04 那句话也是这个意思：协议对七层都开放，`{fetch-org}` 收什么另定。
4. **先要升微信**：不论 A 还是 B，都要先把微信升到 4.x；升级会影响视觉层那条路（[P10](../PRD.md#72-获取插件)）。

<a id="19-给-06-的补充"></a>
### 1.9 给 06 的补充

**[06 §16.3](06-platform-access-and-account-risk.md#163-下架停更与法律风险) 的一条 bullet**，放在「fork 保存不可靠」之后，可以直接粘贴：

```markdown
- **点名的是取密钥工具，重开的和拷贝还在。** 腾讯 2026-07-13 点名 jackwener/wx-cli 的通知以反规避条款（17 U.S.C. 1201）为主、服务条款为辅，主张本地库的「Protected Database Design」受版权保护、提取密钥即规避，没有主张商标，并请 GitHub 防止同一人或他人再上传类似内容；GitHub 2026-07-15 按 1,464 个仓库的派生网络屏蔽（[通知](https://github.com/github/dmca/blob/master/2026/07/2026-07-13-wechat.md)）。一键部署的整条链里含取密钥工具的 bjdr69/docker-wxcli 也被点名并删除（[通知](https://github.com/github/dmca/blob/master/2026/09/2026-09-25-wechat.md)）；只调用用户自装 wx-cli 的仓库（如 JimLiu/baoyu-skills）到 2026-11-04 没有被点名。r266-tech/wechat-cli 被屏蔽后同作者另起 r266-tech/wx-cli（[通知](https://github.com/github/dmca/blob/master/2026/08/2026-08-04-wechat.md)）。同一作者 2026-09-14 重开的 jackwener/wx-cli-again 到 2026-11-04 仍可访问、有 1,793 个 fork；另有至少 4 个不挂 fork 关系的全历史拷贝仍可访问（hum0r23/wx-cli、Steady775/wx-cli、RealMnzn/wx-cli、Markjinli/wx-cli-gui），hum0r23 那个还带 v0.3.0 二进制；npm 上的 @jackwener/wx-cli 0.3.0 也还能装。36 份通知点名的仓库 2026-11-04 有 25 个返回 451、5 个已删、6 个仍可访问（其中 4 个只剩 0–15 KB，推断已清空）。账号一侧：WeFlow 用户 2026-03-11 成批收到微信警告，协作者说是取密钥时被记录（[hicccc77/WeFlow #411](https://github.com/hicccc77/WeFlow/issues/411)）。
```

**[06 §8](06-platform-access-and-account-risk.md#8-微信生态) 的两行**：用第二行替换现有的 wx-cli-again 一行，第一行加在它前面（按 ★ 排）。r266-tech 不加；要加的话，写「管理员密码存进钥匙串」，不写「sudo 凭据」。

```markdown
| [pandorafuture/wx-cli](https://github.com/pandorafuture/wx-cli) | 1027 / 2026-08-26 / v0.7.4（2026-07-22） / MIT | 微信聊天 | Mac 微信 4.1.7 以上的本地数据库解密与查询命令行，带本机 REST 接口和 SSE 推送；要关 SIP，不读收藏；和 jackwener 的 wx-cli 同名不同源 | client-hook | 必须 | 命令行、HTTP 接口、Agent Skill | 不接入。MIT。★ 等为 2026-11-04 查询。 |
| [jackwener/wx-cli-again](https://github.com/jackwener/wx-cli-again) | 897 / 2026-09-14 / none / Apache-2.0 | 微信聊天、公众号 | 读微信 4.x 本机加密数据库的命令行加常驻进程：聊天、联系人、收藏（文章类带链接）、公众号推送、朋友圈；wx-cli 被 DMCA 按 1,464 个仓库的网络屏蔽后重开，只有一个提交，作者说后续代码在私有仓库 | client-hook | 必须 | 命令行、Agent Skill | 不接入。Apache-2.0；只参考收藏字段。★ 等为 2026-11-04 查询。 |
```

---

<a id="2-horizon"></a>
## 2. Horizon

<a id="21-基本情况"></a>
### 2.1 基本情况

| 项 | 值 |
|---|---|
| 仓库 | [Thysrael/Horizon](https://github.com/Thysrael/Horizon)，自称「你自己的 AI 新闻雷达」，生成中英文每日简报 |
| ★ / fork | 12,275 / 1,795 |
| 许可、语言 | MIT · Python ≥ 3.11 |
| 时间 | 2026-02-20 建，最后推送 2026-11-03；近 30 天 31 次提交 |
| 发版 | 没有 release，也没有 tag；README 把发版、发 PyPI 列为计划 |
| 贡献者 | 41 |

<a id="22-怎么挑出重要的"></a>
### 2.2 怎么挑出重要的

管线在 `src/orchestrator.py:241-300`，按顺序：

1. **时间窗口**：默认最近 24 小时。
2. **抓取**：并发抓所有来源。
   - 来源有 RSS、Hacker News、Reddit、Telegram、X（Apify，或 cookie 加 Playwright）、GitHub 等。
3. **跨来源去重**：同一个 URL 合并。比较前 host 转小写，去掉默认端口、末尾斜杠、`utm_*` 和一组跟踪参数（`orchestrator.py:59-85`）。
4. **打分**：按 Profile 的 `analysis.md` 打 0–10 分，给理由、摘要和标签。
5. **筛选和补充**：按阈值过滤，讲同一件事的由模型合并；再写背景、影响，可以用网页搜索。
6. **渲染**：拼成 Markdown 日报，这一步不调模型。

所以「重要」是**模型按你写的评分标准打出来的**。[07 §1](07-positioning-and-naming.md#1-这是什么东西) 记过你的话，AI 总结出来的东西也有大量垃圾；07 还说，在库里再造一个更聪明的挑选器，等于把推荐流请回家。Horizon 正是这样一个挑选器，本产品不在自己这边重做它（[§2.6](#26-怎么接)）。

<a id="23-输出与存储"></a>
### 2.3 输出与存储

| 输出 | 怎么做的 |
|---|---|
| Markdown 日报 | 每种语言一份，`data/summaries/horizon-<日期>-<语言>.md` |
| GitHub Pages 加 Atom | `docs/feed-zh.xml`、`feed-en.xml`，每种语言放最近 7 篇，`<content>` 是整篇日报的 HTML |
| 邮件、Webhook、微信 | SMTP；飞书、钉钉、Slack、Discord 或通用 Webhook，请求体是模板；微信经 iLink Bot，扫码登录 |
| MCP | stdio 服务 `horizon-mcp`。代码里 13 个工具，README 只列了 12 个，漏的是 `hz_send_webhook`，它会**往外发** |
| Agent Skill | `skills/horizon/SKILL.md` |

- **存储**：
  - 没有数据库，也不记哪些条目处理过；
  - 唯一的「记忆」是对过去日报做 BM25 检索；
  - 配置是一个 `data/config.json`，密钥放 `.env`。
- **不可信内容**：提示词里有一条共用规则，把条目字段当数据、不当指令（`src/ai/prompting/common.py:3-6`）。但 MCP 的返回里没有不可信标记，也没有来源信封。
- **HorizonHub**：设计文档说客户端会上报使用数据，源码里没找到（推断还没实现）。

<a id="24-维护者的取向"></a>
### 2.4 维护者的取向

- **#69**（2026-05-24 开，06-05 关）有人要一个 HTTP API。维护者说现在的 RSS 已经够拉取，又担心 HTTP 服务污染 CLI。
- **#75**（05-27 开，08-01 关）有人要「只爬取、存 SQLite」的模式。维护者说只爬取不是这个项目的愿景，也没想清楚要不要让它有状态。
- **推断**：Horizon 会一直是「跑一次、出一份日报」的工具，不会变成能拉取条目的服务。要接它，就接它的产物。

<a id="25-同类项目"></a>
### 2.5 同类项目

| 项目 | 情况 | 怎么挑 | 和我们的关系 |
|---|---|---|---|
| [sansan0/TrendRadar](https://github.com/sansan0/TrendRadar) | 63,400 ★ · GPL-3.0 | 各平台热榜加 RSS；关键词文件，或用日常语言写关注方向让模型打标签；推到 9 个渠道；本地 SQLite；有 MCP | 有状态、定时跑，是另一种 AI 阅读器 |
| [RSSNext/Folo](https://github.com/RSSNext/Folo) 的 AI | 39,220 ★ · AGPL-3.0 | 阅读器里做时间线摘要和定时简报（AI Tasks） | 简报做在阅读器里面 |
| [yinan-c/RSSbrew](https://github.com/yinan-c/RSSbrew) | 297 ★ · AGPL-3.0 | 你写的规则过滤，加 AI 摘要，输出还是 feed | 筛选靠你写的规则 |
| [Tiendil/feeds.fun](https://github.com/Tiendil/feeds.fun) | 397 ★ · NOASSERTION | 模型只打标签，分数来自你写的规则 | 和 07「判断留在自己手里」最接近，是选项 C 的参照 |

<a id="26-怎么接"></a>
### 2.6 怎么接

| 路 | 走哪 | 成本 | 建议 |
|---|---|---|---|
| 读它发布的 Atom | 通用 feed 插件；一篇日报是一条 entry | 小（feed 插件做好以后） | 建议 |
| 读 `horizon-*.md` | 投递文件夹，或一个导入插件（[§3.9](../protocol/fetch.md#39-从文件导入)、[§5.2](../protocol/delivery.md#52-监听文件夹)） | 小 | 建议 |
| Agent Skill | 你的 Agent 跑 Horizon、读日报，把你想要的链接逐条 `{name} add`（[§5.3](../protocol/delivery.md#53-各种入口怎么接)，能不能直接收下等 [discussion 第 15 条](../answers.md#15-agent-能不能直接收下)） | 小 | 可以 |
| 包装它的 MCP（[§9.3](../protocol/wrapping.md#93-包装-mcp-服务)） | 获取插件启动 `horizon-mcp` | 大 | **不建议**：要在 `requires.runtime` 里带一整套 Python；它用自己配的密钥调模型，[§4.3](../protocol/transform-export.md#43-模型与密钥由宿主代发) 的宿主代发管不到；`hz_send_webhook` 会往外发，插件得写明不调 |
| 它的 Webhook 推到投递接口 | [§5.1](../protocol/delivery.md#51-本机-http-接口) | — | **不合适**：`api` 令牌只收链接，一段 `text` 只取第一个链接，整篇日报会变成一个链接 |

- **要标出「机器挑的」**：
  - 日报的原文本身是模型写的；里面每条为什么在那，是模型打的分。
  - [M3](../PRD.md#76-地图) 的建议是：远看浮出来的字、条目第一行，只能来自你的话和平台原带的标签。
  - 所以从 Horizon 来的条目要一眼看得出是机器挑的。现在没有字段能表达，见 [§6.2 第 11 条](#62-要补的地方)。
- **我们自己不出跨条目简报**：
  - 现有插件都是按条目处理的，没有一种吃一批条目、产出一份简报（[§6.2 第 10 条](#62-要补的地方)）；
  - 定时出简报违反 [product-model §5.1 第 1 条](../PRD.md#4-原则)（从不定时）；
  - 「今日回顾」也是 07 点名不做的。
- **借什么**：它的 Atom 和 Markdown 当输入格式。通用网页条目去重时，可以参考它去掉跟踪参数的做法（推断）。所以 Horizon 记作[致谢](../acknowledgements.md#致谢候选)的参考设计。

---

<a id="3-订阅和资料库的边界"></a>
## 3. 订阅和资料库的边界

收下是你拿的，feed 条目是自己来的。这一节比五个选项，[discussion 第 1 条](../answers.md#1-什么东西进资料库) 由你定。

<a id="31-现状"></a>
### 3.1 现状

- **自己的收藏夹已经是 B 的样子**：跟着收的新条目直接收下、不逐次预览，记 `origin.entry: sync`（[插件协议 §3.8](../protocol/fetch.md#38-list-与跟着收)）。量由你自己的动作决定。
- **PRD 写过 RSS**：PRD v0.3 的同步一行写「订阅收藏夹/账号/公众号/RSS」。
- **[C6](../PRD.md#71-收下) 的建议是 B**：别人的公开账号只收订阅之后的新内容，不回溯。
- **第一次拉没有定义**：[§3.8](../protocol/fetch.md#38-list-与跟着收) 只写了增量什么时候停，没写第一次拉时窗口里已有的条目收不收。
- **一处说法打架**：07 写跟着收新来的「只是地图上稍亮的一个点」，[M4](../PRD.md#76-地图) 写「点更淡，或单独一层」。等地图原型定，这里不定。

<a id="32-用什么来比"></a>
### 3.2 用什么来比

- **原则 4**：不制造压力，没有未读数、要处理的队列和打卡，不存在「未整理」状态。
- **原则 5**：入库内容是不可信输入；MCP 不删、不外发。
- **[product-model §5.1](../PRD.md#4-原则) 的四条检验**：
  - 只由你的动作打开，从不定时；
  - 不计数剩下的；
  - 按钮写清后果；
  - 没有「没处理」的状态。

  原文是为「放下」写的，这里借来检验订阅。
- **量的比例**：订 30 个 feed、每个每天 2 条，就是每天 60 条、一年约 2.2 万条。你亲手收下的很可能低一两个数量级（推断，数字是假设）。
- **变成阅读器的风险**：界面的重心会不会变成「新来了什么」。

<a id="33-五个选项"></a>
### 3.3 五个选项

<a id="a-订阅留在阅读器里"></a>
#### A 订阅留在阅读器里

- **是什么**：
  - feed 在你已经在用的阅读器里读：Miniflux、FreshRSS、Folo、NetNewsWire、Readwise Reader。
  - 在那里加星就算收下。本产品跟着收「阅读器的星标」，这是你自己的一个列表。
  - 本产品不做订阅列表，也不存没加星的条目。
- **怎么接**：
  - [Miniflux](https://github.com/miniflux/v2)（Apache-2.0）和 [FreshRSS](https://github.com/FreshRSS/FreshRSS)（AGPL-3.0）都兼容 Google Reader API。一个获取插件用 `list` 读星标流就行，来源比如 `greader:starred`，`order: newest-first`。
  - Readwise Reader 走它自己的 API v3；Folo 的 `webhooks` 动作能不能在加星时触发、NetNewsWire 的星标外部能不能读，都待核实。
- **原则 4**：本产品没有任何计数。阅读器里的未读，是你原来就在用的工具的事。
- **原则 5**：只有你加星的进库，量最小。
- **数据模型**：
  - 身份由条目链接经平台插件 `canonicalize` 得出；
  - raw 是平台插件或通用网页插件取回的原页面，feed 里的正文摘录可以另存一份快照；
  - `action: favorite`。
- **协议**：现有的 `list` 和跟着收够用。只缺一处：阅读器是你自己跑的服务，`sites[].site` 要能引用一个设置项（[§6.2 第 7 条](#62-要补的地方)）。
- **界面**：设置里多一种跟着收的来源「阅读器的星标」，别的不加。
- **变成阅读器的风险**：最低。
- **缺点**：没在用阅读器的人要先装一个。
- **插件放哪**：读 Google Reader API 不含平台知识，推断可以像 P1 那样进官方组织。但 D-f 原则上官方组织不放获取插件，要你点头。

<a id="a2-在本产品里读只留你拿的"></a>
#### A2 在本产品里读，只留你拿的

这一项是这次新拟的，下面的判断都是推断。

- **是什么**：feed 在本产品里显示成一条不计数的流；你点了收下的才进资料库，其余的不留。
- **原则 4**：
  - 字面守得住：不计数，也不留；
  - 但一块阅读界面本身，就会把界面的重心移到「新来了什么」；
  - 流不能自己弹出来，否则违反 [product-model §5.1 第 1 条](../PRD.md#4-原则)。
- **原则 5**：进库的量同 A；流里的内容不进 MCP。
- **数据模型**：
  - 资料库不变；
  - 要另存订阅列表，和每个订阅看过的 `guid`。这是配置状态，不是真相文件；
  - 流本身只是缓存。
- **协议**：
  - 要 B 的用户填地址、首拉、条件请求（[§6.2 第 1、3、6 条](#62-要补的地方)）；
  - 要显示正文，还得有一种不落库的 `fetch`，协议里没有。
- **界面**：多一个阅读视图，加订阅管理和 OPML 导入。这就是一个不计数的 RSS 阅读器。
- **变成阅读器的风险**：高。功能上和 Folo、Miniflux 正面重叠。

<a id="b-全部进雾"></a>
#### B 全部进雾

- **是什么**：选中的 feed 的每一条都自动收下，在雾里画成一层淡的「路过」。
- **原则 4**：字面守得住；但满地没经你挑的点，本身就是一种「还没看」的压力（推断）。
- **原则 5**：大量不可信内容进 MCP 的搜索结果，Agent 搜到的一大半可能是 feed 来的，提示注入的面跟着变大（推断）。
- **数据模型**：
  - 每条一个条目；raw 是 feed 里的正文，或取回的原页面；
  - 身份经 `canonicalize` 得出。feed 自己的 `guid`、`atom:id`、JSON Feed `id`，只在这个 feed 内部去重；
  - 记 `origin.subscription`；
  - 存储按订阅数线性涨。
- **协议**：[§6.2 第 1–6、8、9 条](#62-要补的地方)，其中 8 是 OPML。
- **界面**：M4 的分层一开始就要做；还得有「只看亲手收下的」开关，否则地图读不出你看重什么。
- **变成阅读器的风险**：高，等于给 RSS 阅读器换一层地图皮。

<a id="c-只进命中关注的"></a>
#### C 只进命中关注的

- **是什么**：只有命中你「一直清楚的那一片」的条目才收下，其余不存。这一片是什么还在 [M1](../PRD.md#76-地图) 讨论，建议是主题。
- **原则 4**：命中就直接收下，守得住；命中的先放进「等你确认」的列表，那就是队列，违反。
- **原则 5**：进库的量小。
- **数据模型**：同 B，另记 `origin.matchedBy {topic, rule}`。没命中的不进资料库，只在订阅记录里留看过的 `guid`。
- **谁在判断**：
  - 规则是你写的关键词时，判断在你手里（RSSbrew、Feeds Fun 的路子）；
  - 用语义相近去匹配时，挑的是模型。那正是 07 说的「更聪明的挑选器」。
- **协议**：同 B，外加宿主在收下前匹配一次，插件不用改。
- **界面**：每个订阅多一行「只收和这些主题有关的」。
- **变成阅读器的风险**：中。它就是「智能订阅」。
- **前提**：M1 先定。

<a id="d-v1-不做订阅"></a>
#### D v1 不做订阅

- **是什么**：v1 不接任何 feed，RSS 和别人的账号都放到以后。你自己平台收藏夹的跟着收照做。
- **原则 4、5**：都不受影响。
- **数据模型、协议**：不变，[§6.2 第 1–9 条](#62-要补的地方)都可以晚点。
- **界面**：不变。
- **变成阅读器的风险**：无。
- **代价**（推断）：
  - 等于收回 PRD v0.3 那句 RSS；
  - C6 B 那条「点名的低频个人来源」也一起推后；
  - RSS 只能经浏览器扩展、分享菜单一条一条收下。
- **和 A 的关系**：读阅读器的星标，跟的是你自己的列表，不算订阅。所以 D 下也可以做。真那样做，D 就只比 A 少那一小条（推断）。

<a id="34-对比"></a>
### 3.4 对比

| | A 阅读器星标 | A2 本产品里读 | B 全部进雾 | C 只进命中的 | D v1 不做 |
|---|---|---|---|---|---|
| 原则 4 | 守得住 | 字面守得住，重心会移 | 字面守得住，地图有压力 | 守得住，前提是没有待确认 | 守得住 |
| 进库量（原则 5） | 最小 | 同 A | 最大 | 小 | 最小 |
| 量由谁定 | 你 | 你 | feed | feed 加你的规则 | 你 |
| 协议改动 | `sites` 引用设置项 | 地址、首拉、条件请求、不落库的取正文 | 地址、首拉、条件请求、身份、订阅 id、OPML | 同 B，加匹配 | 无 |
| 界面改动 | 设置里一项 | 阅读视图、订阅管理 | 分层、过滤开关 | 订阅里的主题过滤 | 无 |
| 变成阅读器的风险 | 最低 | 高 | 高 | 中 | 无 |
| 依赖 | 你在用阅读器 | 无 | 无 | M1 | 无 |

<a id="35-建议"></a>
### 3.5 建议

**A，加一小条。**

1. **RSS 默认走 A**：
   - 阅读、未读、发现都留给现成的阅读器；加星就是收下；
   - 本产品跟着收阅读器的星标，不做订阅列表，也不做 OPML 整包订阅。
   - 有一处要你定：[§4.1](#41-输入) 那行 OPML 导入，先写「可以不做」。
2. **加一小条**：你点名的低频个人来源可以按 C6 B 一个一个跟着收，比如某人的博客、一个 Mastodon 或 Bluesky 账号。
   - 第一次拉，只记下当时已有的条目，不收；以后新出现的才收下。
   - 地图上分层画（M4）。
3. **你自己平台收藏夹的跟着收照旧**：
   - 你跟着收的收藏夹和列表，新来的就收下，现在已经是 B 的样子；
   - 量由你自己的动作决定，所以不改。
4. **Horizon 当一个 feed**：读 Atom 或 Markdown；收下的条目标「机器挑的」（[§2.6](#26-怎么接)）。
5. **不在本产品里出简报。** C 等 M1 定了再说，到时先只用你写的关键词。

**理由**：
- 兼容靠标准接口，不用再做一个阅读器；
- 原则 4、5 和 07 的定位都不用让步；
- 在「AI 阅读器」这条路上，Folo（39k ★）和 TrendRadar（63k ★）已经走得很远。

**要你答的**，并在 [discussion 第 1 条](../answers.md#1-什么东西进资料库) 里：
- 你现在用哪个阅读器？这决定先接 Google Reader API，还是 Readwise、Folo。
- 「加星就是收下」能不能当 RSS 的唯一入口？
- 低频个人来源那一小条要不要？

---

<a id="4-兼容矩阵"></a>
## 4. 兼容矩阵

**工作量**按要动的接口算，不按小时：
- **小**：用现有能力就能做，只写一个插件。现有能力指 `list`、`fetch`、[§3.9](../protocol/fetch.md#39-从文件导入) 导入、[§5.1](../protocol/delivery.md#51-本机-http-接口) 投递、[§5.2](../protocol/delivery.md#52-监听文件夹) 投递文件夹；
- **中**：加一个字段或子命令，或者宿主加一个小功能；
- **大**：SPEC 里加新概念，或者加一种新的插件类别。

「在设计里」指 11-03 版的 PRD v0.3、SPEC、插件协议、open-decisions 写到了。

<a id="41-输入"></a>
### 4.1 输入

| 入口 | 在设计里吗 | 走哪 | 工作量 | 备注 |
|---|---|---|---|---|
| 粘贴链接 | 在 | `{name} add`、`POST /v1/deliver`；通用网页获取随安装包预装（P1） | 小 | — |
| 分享菜单 | 一部分 | iOS 快捷指令写投递文件夹；Android 经 HTTP Shortcuts 局域网直传；macOS 分享扩展没写 | 中 | [§5.3](../protocol/delivery.md#53-各种入口怎么接) |
| 浏览器扩展 | 在 | `extension` 令牌 `POST /v1/deliver`，带正文和文件 | 中（单独的扩展项目） | — |
| 阅读器星标（Google Reader API、Fever） | 不在 | `list` 读星标流 | 小到中：插件小，`sites` 要能填你的主机 | 选项 A；Miniflux 的 Webhook 用 HMAC 签名，接不上投递接口，用拉取 |
| RSS、Atom、JSON Feed | PRD 写了，协议没接上 | `list`，来源 `feed:<url>` | 中：协议缺 [§6.2 第 1–6 条](#62-要补的地方)，插件本身小 | 解析的是格式不是平台，推断照 P1 进官方组织，要你点头 |
| OPML 导入 | 不在 | 导入插件产出的是条目，没有路能把文件导成订阅 | 中（宿主读 `outline type="rss"` 的 `xmlUrl`） | 按 [§3.5](#35-建议) 可以不做；要不要做待你定 |
| Pocket、Omnivore、Readwise、Instapaper、Wallabag、Netscape HTML、Raindrop、Cubox、flomo | 在（P3 预装） | 导入插件 | 每种小 | Omnivore、Readwise 的 ZIP 带正文 |
| 链接清单（JSON、CSV） | 本篇新提 | 导入插件，官方组织 | 小 | 接住 client-hook 工具导出的清单，[§1.8](#18-建议) |
| Obsidian vault、Web Clipper 的 Markdown | Web Clipper 在 P3 列表里，vault 没写 | 导入插件读带 front matter 的 Markdown | 小到中 | front matter 里的来源字段因插件而异（推断） |
| Notion 导出包 | 不在 | 导入插件 | 中 | 包结构待核实 |
| 平台获取插件 | 在 | `fetch`、`list` | 按平台 | `{fetch-org}`，见 [06 §17.3](06-platform-access-and-account-risk.md#173-首批自己适配的插件) |
| RSSHub 实例 | 在 | 通用 feed 插件指向你自己的实例；带 cookie 的路由单独做插件，放 `{fetch-org}` | 小到中 | 带 cookie 轮询有账号风险 |
| newsletter | 不在 | Kill the Newsletter 转成 Atom，再走 feed 插件；或 Folo 的收件邮箱 | 小；自己收 IMAP 是大 | 经第三方的要标「离开本机」 |
| Mastodon、Bluesky 账号 | 不在 | 通用 feed 插件读 `/@<用户名>.rss`、`/profile/<DID>/rss`；`public-web`，`login: none` | 小（feed 插件做好以后） | 别人的账号受 C6 约束；推断走通用插件，不各做平台插件 |
| Horizon | 不在 | Atom 或 Markdown，见 [§2.6](#26-怎么接) | 小 | MCP 包装大，不建议 |
| Agent Skill 交链接 | 在 | `{name} add`、`api` 令牌 | 小 | 等 [A3](../PRD.md#78-agent) |

Mastodon、Bluesky 的路径出处：
- Mastodon：`mastodon/mastodon` 的 `app/controllers/accounts_controller.rb`；
- Bluesky：`bluesky-social/social-app` 的 `bskyweb/cmd/bskyweb/server.go` 第 391–392 行。

两处都在 11-04 经 `gh api` 读过。

<a id="42-标准"></a>
### 4.2 标准

| 标准 | 在设计里吗 | 用在哪 | 和身份、去重有关的条款 |
|---|---|---|---|
| RSS 2.0 | 只在 PRD 提过 | 读：通用 feed 插件；写：导出成 feed | `<guid isPermaLink="true">` 可以当永久链接（[RSS Board](https://www.rssboard.org/rss-specification)） |
| Atom（RFC 4287） | 同上 | 同上；Horizon 的产物 | feed 和 entry 都必须恰好有一个 `atom:id`（[RFC 4287](https://www.rfc-editor.org/rfc/rfc4287.txt)） |
| JSON Feed 1.1 | 不在 | 读同上；导出成 feed 时首选 | `id` 必填、长期不变、不能复用；扩展用 `_` 开头的对象（[规范](https://www.jsonfeed.org/version/1.1/)） |
| OPML 2.0 | 不在（只有播客导入提过） | 导入订阅（可以不做）、导出订阅列表 | `type="rss"` 的 outline 用 `xmlUrl`（[规范](http://opml.org/spec2.opml)） |
| Google Reader API、Fever | 不在 | 读阅读器的星标（A） | Miniflux README、FreshRSS 文档都写了兼容 |
| MCP 2026-07-28 | 在 | 作服务端：[SPEC §10](../BUILD.md#006-agent)；作客户端：[§9.3](../protocol/wrapping.md#93-包装-mcp-服务) 包装 | — |
| Markdown 加 front matter | 在 | 条目的 `content.md`、导出 | — |
| Obsidian vault | 在 | 导出和读回；资料库直接当 vault 等 [E2](../PRD.md#79-导出与互通) | — |
| Netscape 书签 HTML | 在 | 导入（P3）、导出（E1） | — |
| WebDAV | 在 | 备份或同步盘，等 [K6](../PRD.md#710-存储删除与联网) | — |
| ActivityPub（W3C 2018-01-23）、AT Protocol | 不在 | 只经 Mastodon、Bluesky 的 RSS 读；做原生客户端是大，不建议 | — |

<a id="43-输出"></a>
### 4.3 输出

| 输出 | 在设计里吗 | 走哪 | 工作量 |
|---|---|---|---|
| 资料库本身就是 Markdown 文件夹 | 在 | [SPEC §2](../spec/library.md#2-资料库目录真相源) | 已有 |
| 导出到笔记软件并读回 | 在 | 导出插件 | 已有设计 |
| MCP 读取 | 在 | [SPEC §10](../BUILD.md#006-agent) | 已有设计 |
| 迁出包：Netscape HTML、CSV | 在（[E1](../PRD.md#79-导出与互通) 建议 B） | 命令行 | 小 |
| 迁出包：带正文、多家能导入的格式 | 等（E1 的 C） | 命令行或导出插件 | 中 |
| 本机 HTTP 读接口 | 不在：投递接口只有 `ping`、`deliver`、`batches` | 和 MCP 同源生成 | 中 |
| **资料库导出成 feed** | 不在 | 导出插件 `target: feed`、`writes: files` | 中 |

**导出成 feed 的两种看法**（都是推断，要你定，在 [discussion 第 17 条](../answers.md#17-兼容和开创各指什么)）：
- **不冲突**：阅读器里的未读是你自己选的，不在本产品里，和原则 4 不冲突。
- **冲突**：阅读器大多会显示未读数，等于本产品亲手造了一个未读数，和原则 4 冲突。

做法见 [§6.2 第 12 条](#62-要补的地方)。先写文件，不开读端点：多数阅读器加不了 Bearer 请求头，令牌只能放进 URL，会留在日志里。价值不大，排在 E1 后面。

---

<a id="5-开创性"></a>
## 5. 开创性

比较对象：
- 01 的 24 个收藏、稍后读、档案和画布产品；
- 05 的笔记软件插件，02 的适配器生态；
- 这次看的 Horizon、Folo、TrendRadar、Miniflux、RSSbrew、Feeds Fun。

<a id="51-今天站得住的"></a>
### 5.1 今天站得住的

| 设计 | 为什么算新 | 可靠程度 |
|---|---|---|
| Agent 的改动先提议，确认后生效，可以整批撤销 | 01 查过的产品都没有「提议、确认、应用、整批撤销」，最近的 Eagle 只在执行前确认；Horizon 的 MCP 也没有（读过源码）；TrendRadar 只读了 README | 较高，但只看了文档，没实测。这在写代码的 Agent 里很常见，新只新在收藏这一类产品 |
| raw 和 derived 分开，索引整个删了能重建 | 01 里没有产品三条都满足，最近的是 ArchiveBox | 中。K5 允许你删 raw 以后，「永远保留」会变弱 |
| 导出到笔记软件以后读回你在那边的改动 | 05 查过的工具都不读回 | 中，没实现过 |
| 清单里写七个获取层级和登录要求，当挑选用的标签 | Mihon 的 `contentWarning`、RSSHub 的 `antiCrawler` 只是开关 | 中。值是作者自己填的，宿主核实不了 |
| Keiyoushi 式的仓库分法：客户端不预置获取来源，你自己的获取插件放单独的组织 | 收藏类产品里少见 | 中。模式借自 Mihon 和 Keiyoushi，新只新在用到这一类 |

还有两条小的，不单独算：
- MCP 返回带「不可信、来源」信封：只是一个标记，管不管用要看 Agent 认不认；
- 模型密钥由宿主代发：没专门查别的生态有没有先例。

<a id="52-方向要原型"></a>
### 5.2 方向，要原型

- **迷雾地图**：
  - 按相似度摆东西的图早就有：Are.na 一个 block 能在多个 channel 里，Recall 有知识图谱。
  - 新的是三件合在一起：点阵的雾；你打开或选定的地方清楚；从不承诺把雾驱散。
  - 没有原型（M2、M4、M7）。说它是开创，要等做出来。
- **注意力模型**：
  - 只记发生过的事，加你选定的「一直清楚的一片」（[M6](../PRD.md#76-地图)、M1），两个都没定；
  - 单看「没有未读」，mymind 已经这样做了。
- **站不住的说法**：
  - 「RSS 也能订阅，还没有未读」：走 B 时地图本身就成了未读。
  - 「AI 摘要默认关」：只是一个默认值。
  - 「一切可插拔」「本地优先」：这类产品都这么说。

<a id="53-持平"></a>
### 5.3 持平

这些别人都有，做好只是不落后：
- MCP 读取（Karakeep、Readwise、Raindrop、Anytype）；本机 REST、命令行加 SKILL.md；
- 浏览器扩展、导入 Pocket 和 Omnivore；OCR、ASR 以后能搜；导出到 Obsidian；
- 文件就是数据（Obsidian、ArchiveBox）；没有未读、不做社交（mymind、Pinboard）；
- 订阅 RSS、AI 摘要、每日简报：每个阅读器都有。

**兼容和开创的分工**：
- 兼容靠 [§4.2](#42-标准) 那张表里的标准；
- 开创放在收下以后：你的判断怎么留下，Agent 怎么只提议。

不放在「又一个 AI 阅读器」上。这一问在 [discussion 第 17 条](../answers.md#17-兼容和开创各指什么)。

---

<a id="6-对插件协议的影响"></a>
## 6. 对插件协议的影响

11-04 那句「之前调研的所有的方案都可以作为一种插件去弄」，按窄读处理：协议对七个获取层级都开放，client-hook 也能写成插件。你自己的 `{fetch-org}` 收哪些，是 [R5](../protocol/distribution.md#85-添加来源更新与权限扩大)、[R11](../protocol/distribution.md#85-添加来源更新与权限扩大) 的事，另问（[discussion 第 9 条](../answers.md#9-自己的获取插件组织收什么)）。

<a id="61-插件化覆盖表"></a>
### 6.1 插件化覆盖表

「卡在哪」一列的编号指 [§6.2](#62-要补的地方) 的条目。

| 方案 | 层级 | 协议现在能不能表达 | 卡在哪 |
|---|---|---|---|
| 包装你自装的 wx-cli（[§1.6](#16-做成插件) 的 A） | client-hook | 部分能 | 14–16 |
| 导入 client-hook 工具导出的清单（B） | `official-export` | 能 | 无 |
| 插件自己取密钥（C） | client-hook | 不能 | `devices` 没有「附加调试器、读别的进程内存」；要 sudo |
| RSS、Atom、JSON Feed 跟着收 | `public-web` | 部分能 | 1–6、13 |
| 阅读器的星标 | 跟阅读器的账号，`login: required` | 部分能 | 7 |
| OPML 导入订阅 | — | 不能 | 8：OPML 里是订阅不是条目，订阅对象没有文件格式 |
| 读 Horizon 的 Atom 或 Markdown | `public-web` 或导入 | 能 | 11：标不出原文是机器写的 |
| 带进 Horizon 的打分和摘要 | — | 不能 | 11：获取插件写不了 `derived/` |
| 跨条目简报 | — | 不能 | 10，建议不做 |
| 装证书做中间人的抓包 | client-hook | 没逐项核 | 宿主不代装根证书（R-i）；插件让你自己装证书算不算允许，没写（推断） |
| 视觉层（agent-desktop、手机 Agent） | `vision-ui` | 能 | [P19](../PRD.md#72-获取插件)（能不能自己跑）、P17 没定 |
| 其余层级（public-web、browser-session、session-api、private-api） | 各层 | 能，没逐项核 | — |

<a id="62-要补的地方"></a>
### 6.2 要补的地方

**按 [§3.5](#35-建议) 的建议要补 9 处：下面的 1、2、3、4、5、7、9、11、13。**
- 选 B 还要 6、8；
- 导出成 feed 要 12；
- 10 建议不做；
- 14–16 只在别人写 wx-cli 包装插件时才用得到。

1. **你自己填的来源。**
   - 现状：`sources` 只能列出插件自己知道的来源，`settings` 也没有列表类型（[§2.11](../protocol/manifest.md#211-插件设置-settings)）。
   - 提议：加一个子命令 `resolve-source <输入>`。输入一个链接或账号，返回 `{match, source, title, order, owner}`，宿主存成一个跟着收。
   - 它能接住「贴一个页面链接，自动找出 feed」，也带上了下一条要的 `owner`。另一种提议「`sources` 返回带参数的模板」只当设置表单的提示。先例是 RSSHub 的路由参数和 Radar 规则（[02 §7](02-adapter-ecosystems.md#7-rsshub-与-rsshub-radar)）。
2. **来源归属**：`owner: self | other`。C6（别人的不回溯）和 M4（分开画）都要用。
3. **第一次拉**：`owner: other` 的来源，第一次拉时把已有条目的身份都记下、不收，以后只收新出现的；`owner: self` 照旧全量。[§3.8](../protocol/fetch.md#38-list-与跟着收) 要补这一条。
4. **feed 条目的身份。**
   - plugin-kit 要求每个平台有 `contentId` 的正则，可 `guid`、`atom:id`、JSON Feed `id` 是任意字符串。
   - 提议：不设 `feed` 平台。条目身份由条目链接经能接住它的平台插件 `canonicalize` 得出，接不住就按通用网页。
   - feed 自己的 id 写进 `list` 条目的新字段 `sourceItemId`，只在这个来源内部去重。
5. **同时带 `url` 和 `ref` 时先用哪个**：先按 `url` 找平台插件取全文；取不到或只取到一部分，再回到 feed 插件 `fetch <ref>`，用 feed 里的正文。两份快照都留，和 P5 补全的规则一致。
6. **条件请求**：feed 轮询要 ETag 和 Last-Modified 才不浪费。提议 `list` 返回一个不透明的 `state`，宿主按跟着收保存，下次用 `--state` 传回，和游标一个思路。
7. **你自己跑的服务当登录站点**：阅读器的星标要登录，可 `sites[].site` 是写死的主机名（[§2.5](../protocol/manifest.md#25-登录站点-sites)）。提议允许 `site` 引用一个设置项，像 [§9.2](../protocol/wrapping.md#92-包装本机-http-服务) 的 `${setting.endpoint}`。
8. **OPML 导入**（只有选 B 才要）：做成宿主自己的功能，把每个 `xmlUrl` 依次交给 `resolve-source`。解析 OPML 是格式层面的事，照 P1、P3 的先例。
9. **记下是哪个跟着收收下的**（D37）：
   - `origin.entry` 照旧记入口的种类：现在写 `sync`，词表提议改成 `subscription`；
   - 另加 `origin.subscription` 记是哪一个。
   - 删除跟着收时不删已收下的、M4 分层、C 的 `matchedBy`，都要用它。
10. **跨条目简报**：现有三类插件表达不了。建议不做；外部工具出的简报，用 feed 插件或导入插件读。
11. **上游的机器产物**（两件事，一节提案）：
    - 原文本身是模型写的：fetch 输出加 `textOrigin: platform | machine`，并把 `evidence.upstream`（例如 `{name: horizon}`）设成必填，界面按 M3 标出「由 Horizon 挑出」；
    - 上游算好的打分和摘要：fetch 输出加可选的 `derived: [{produces, path, by}]`，宿主照转换插件的规矩移进 `derived/`。
12. **导出成 feed**：导出插件 `target: feed`、`writes: files`。默认排除私密，正文只给纯文本，身份放 `_{name}`。读端点等本机 HTTP 读接口一起定。
13. **feed 的节奏**：节奏按平台算（D-e），可 feed 散在很多主机上。提议 `feed:` 来源按主机算，默认沿用跟着收的频率：每天一次，最快 6 小时（推断）。
14. **连本机 socket**：协议里没有「连本机 Unix socket」的权限项。二选一：加一项，或者要求这类插件走本机 HTTP（[§9.2](../protocol/wrapping.md#92-包装本机-http-服务)）。
15. **私有 `HOME` 和钉死在家目录的 socket**：上游把 socket 写死在 `$HOME` 下时，插件连不到沙箱外的常驻进程（[§2.7](../protocol/manifest.md#27-权限-permissions)）。要么写明「这类上游只支持一次一进程」，要么和上一条一起解决。
16. **插件起的后台进程**：上游自己脱离进程组时，宿主怎么收掉它，[§7.1](../protocol/isolation.md#71-常驻进程的路径) 没写。

---

<a id="7-待核实清单"></a>
## 7. 待核实清单

**wx-cli**

1. 你说的 wx-cli 是不是 jackwener 的（[§1.1](#11-是哪个-wx-cli) 是推断）。
2. MSK 是不是 Mitchell Silberberg & Knupp：只有信头网址一条依据。
3. 视频号收藏是哪个类型号、XML 里有没有可用的链接或 ID：两家一个记 4、一个记 20，不运行查不了。
4. 收藏库的密钥，首次就能拿到，还是要等打开收藏页再补。
5. 宿主沙箱里：能不能连沙箱外的 socket；wx 另起的后台进程能不能收掉；读微信容器目录会不会另弹授权。
6. r266-tech 的收藏输出是否保留原始 `content`。
7. Knave963/wx-cli、123kyle/wx-cli 没逐个核对；GitHub 有没有收到针对那几个拷贝的通知，查不到。
8. WeFlow #411「取密钥时被记录」是开发者的判断；被警告的账号是否都走了取密钥，评论里看不全。
9. caigee-cmd/wechat-insight 现在里面是什么；bjdr69/docker-wxcli 是不是 wx-cli 的包装（已删，只能看通知的转述）。
10. 国内托管（Gitee 等）有没有对应动作，没查。

**订阅与兼容**

11. FreshRSS、Miniflux 读星标流的具体路径和鉴权：只看了文档目录和 README。
12. Readwise Reader API v3 用什么参数筛出你挑过的；它的 Feed 和 Library 是不是分开的（凭印象）。如果是，它就是 A 在单个产品里的先例。
13. Folo 的 `webhooks` 能不能在加星时触发；NetNewsWire、Inoreader 的星标外部能不能读；Feedly 的 OPML 导出页只核对了标题。
14. Get笔记有没有导出；Notion 导出包的结构。
15. Horizon 自建时换了 baseurl，Atom entry 的 `id` 会不会变（会变就被当成新条目）；HorizonHub 的上报有没有在别处实现。
16. Mastodon 各实例是否都开着账户 RSS；Bluesky 的 RSS 用 DID，填 handle 要先换。
17. 「一个 feed 打开时通常已有 10–50 条」是经验估计。
18. TrendRadar 的 MCP 有没有改动确认或撤销；宿主代发模型密钥在 Raycast、VS Code 等生态有没有先例。
