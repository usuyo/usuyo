<a id="调研-06--平台内容获取的开源生态与账号风险"></a>
# 调研 06 — 平台内容获取的开源生态与账号风险

状态：草案 · 2026-10-30

目的：回答用户 2026-10-30 提的四件事。第一，用户点名的项目和它们的同类各是什么、怎么拿内容、怎么接进歇存：点名的项目见 [§3](#3-用户点名的项目)，同类按类别列在 [§4](#4-多平台下载与抓取) 到 [§15](#15-相邻创作发布与分析)。第二，获取方式按账号风险怎么分层：分层见 [§1](#1-账号风险分层)，每个平台有哪些层级可选见 [§2](#2-覆盖矩阵每个平台有哪些层级可选)，证据见 [§16](#16-账号风险的证据)。第三，哪些值得自己适配或 fork、许可证允不允许，见 [§17](#17-对歇存的启示)。第四，以后可能放进致谢名单的项目，在各表里标「致谢」，见 [约定](#约定)。不确定的事实汇总在 [§18](#18-待核实清单)。

<a id="约定"></a>
## 约定

- **数据来源与日期**：每个仓库的 ★、最近推送、最近 release、许可证、是否归档，都是 2026-10-30 调用 `gh api repos/<owner>/<repo>` 与 `gh api repos/<owner>/<repo>/releases/latest` 的返回。★ 用 `stargazers_count`；推送用 `pushed_at` 的日期；release 写「tag（日期）」，没有 release 写 none；许可证用 `spdx_id`，没有许可证写 none，GitHub 识别不出的写 NOASSERTION，实际条款写在「歇存怎么用」一列。只有 Readeck 维护的 go-readability 不在 GitHub（在 Codeberg），★ 一栏写「—」。判断只依据代码、文档和 issue，没有运行任何抓取工具，也没有对平台发请求。
- **范围**：共收录 470 个项目，层级分布 official-export 40、public-web 61、vision-ui 48、browser-session 48、session-api 31、private-api 111、client-hook 30、not-applicable 101。主要用途是刷量、刷赞刷粉、批量私信、养号、撞库或收集他人隐私数据的项目不收录，也不算相邻项目。复现签名和 Hook 客户端的项目只记它是什么、属于哪一层、维护和下架情况，不写技术细节。
- **获取层级**：按「实际怎么拿到数据」判断，不按项目自己的宣传。风险对比见 [§1.1](#11-七个层级)。
  - `official-export`：平台官方的数据导出、用户授权的官方开放 API，或读取用户已经导出的文件；
  - `public-web`：不登录，读公开页面或分享链接，不自己复现平台的请求签名；
  - `vision-ui`：只读屏幕上渲染出来的内容（截图、无障碍树、OCR、视觉模型），用系统级输入驱动真实客户端，包括 computer use、Android 自动化（ADB、uiautomator2、无障碍服务）、iOS（WebDriverAgent、iPhone 镜像）和录屏；
  - `browser-session`：驱动用户自己的或自动化的浏览器，带登录态读 DOM 或页面自己发出的请求，签名由页面自己的脚本完成；
  - `session-api`：在代码里直接带用户的 cookie 或 token 调平台网页或 App 接口，参数和官方客户端一致，不自己复现签名；
  - `private-api`：在官方客户端之外复现或模拟请求签名、加密参数、设备指纹，或模拟 App 调私有接口；
  - `client-hook`：注入或 Hook 官方客户端进程、读内存、解密本地数据库，或装根证书做中间人拦截、改写客户端流量和页面；
  - `not-applicable`：不从平台取内容，例如转换、知识库、发布。

  一个项目有多条路径时，取默认路径用到的最高一层。所以多平台工具的层级往往偏高：同一个工具在某个平台上可能走更低的一层，[§2](#2-覆盖矩阵每个平台有哪些层级可选) 对这种情况单独标记。
- **登录**：是否需要用户账号单独记，和层级分开。不登录的方式没有账号风险，只有 IP 层面的风控。表里写「不用 / 可选 / 必须」，不涉及平台的写「—」。
- **表格各列**（[§4](#4-多平台下载与抓取) 到 [§15](#15-相邻创作发布与分析)）：
  - 项目：仓库链接。后面标「（致谢）」的是以后可能放进致谢名单的项目，共 260 个，标注沿用各条调研的判断；
  - ★ / 推送 / release / 许可证：见上，归档的仓库另标「已归档」；
  - 平台：支持四个及以上平台的前面写「多平台」；「微信聊天」也包括客户端里的「收藏」；
  - 做什么：一句话。02、05 写过的项目在后面链过去，用户点名的项目链到 [§3](#3-用户点名的项目)；
  - 层级、登录：见上；
  - 接口：项目对外提供的形态；
  - 歇存怎么用：先写用途（获取方式、获取加转换、转换插件、参考、相邻），再写适配方式（包装它的命令行、调它的 HTTP 接口、经 MCP 调用、fork 后改造、借做法自己写、只作参考、不接入），最后一句许可证提示。获取方式按 [ADR-003 §5](../decisions/ADR-003-plugins.md#5-仓库拆分) 只能放第三方仓库；GPL 系按 [ADR-003 §8](../decisions/ADR-003-plugins.md#8-gpl-工具) 只做用户自装、独立进程调用。
  - 排序：先按层级，从 `official-export` 到 `client-hook`，最后是 `not-applicable`；同一层按 ★ 从高到低。
- **措辞**：一律写个人备份、离线阅读、下载原始文件、平台风控这类中性说法。
- **和 02、05 的分工**：[02](02-adapter-ecosystems.md) 写来源适配器生态怎么声明、分发、调用、治理，对象见 [02 §0](02-adapter-ecosystems.md#0-范围与缺口清单)；[05](05-note-apps-and-plugins.md) 写平台内容转笔记的工具和转换插件，见 [05 §1](05-note-apps-and-plugins.md#1-平台内容转笔记的工具)、[05 §2](05-note-apps-and-plugins.md#2-转换类插件与工具)。本报告只补「怎么拿到数据、账号要承担什么」这一维，两份报告写过的细节不重复。
- **数据处理**：读到的网页、仓库和 issue 内容都按数据处理。有的 README 写着给 AI 编程助手的指令（例如让它用 curl 下载安装文档），都没有执行。
- **待核实**：只有二手来源、源码与文档冲突、或属于推断的地方标「待核实」，汇总在 [§18](#18-待核实清单)。

<a id="0-要点"></a>
## 0. 要点

1. **视觉层的账号风险低，但不是零。** iPhone 镜像目前没有公开 API 让 App 识别，也没找到平台据此处罚的记录；但微信协议明文禁止用第三方软件读界面、自动操作，2026-01 点名辅助服务读取和模拟按键，豆包手机助手操作微信时用户被强制下线；Android 上已开启的无障碍服务和 USB 调试都能被 App 查到。见 [§1.2](#12-视觉层的风险从哪里来)。
2. **小红书是 2026 年风控最严的平台。** 浏览器驱动和本地签名两条路都有低频只读被警告、禁言 7 天到永久封禁的报告；不登录批量取公开页，也有复制链接的那个账号被下线。见 [§6](#6-小红书)、[§16.2](#162-issue-里的账号问题)。
3. **抖音网页端只放行页面内请求。** 2026 下半年起，代码里带 cookie 直接调收藏接口的路径失效，由页面自己发请求的工具仍能取列表；抖音的限制多是几小时的临时限制，没有找到封号报告。见 [§5](#5-抖音快手与-tiktok)。
4. **不登录就没有账号风险，但拿不到「我的收藏」。** 收藏列表这一步要登录，或者走官方导出。「视觉层或浏览器拿链接、公开页面取内容」的混合路径在抖音、公众号可行；小红书受 xsec_token 和链接跟踪参数限制；视频号要借腾讯元宝的登录态。见 [§1.3](#13-混合路径视觉层拿链接公开页面取内容)。
5. **官方导出这一层被低估。** 抖音「个人信息下载」、Google Takeout、X 归档、Instagram 与 TikTok 的数据导出、微信读书个人 API Key、知乎开放平台、微博开放接口都有现成的解析器或封装，账号风险接近零，代价是要手动操作、范围要逐个核实。见 [§2](#2-覆盖矩阵每个平台有哪些层级可选)。
6. **下架和法律风险集中在两类做法：解密本地数据、复现平台签名。** 腾讯 2026 年发了 37 份针对聊天库解密的 DMCA 通知，B站 2026 年给接口文档、SDK、下载器和 Xposed 模块的维护者发律师函；只读公开页、页内请求、UI 自动化、官方导出这几类没有找到来函记录。见 [§16.3](#163-下架停更与法律风险)。
7. **视频号是覆盖最差的平台。** 不装证书、不注入的路只有借腾讯元宝的登录态换凭证，账号风险从微信号转到元宝账号；其余都是本机代理改写页面，属于 `client-hook`。见 [§8](#8-微信生态)。
8. **热门中文项目多是非商业或源码可见许可。** MediaCrawler、逸尘的 Skill 合集、阿杭的两个仓库、wechat-inbox-sync 新版、Hypit、videodl 都只能参考，不能分发，也不能改成歇存的插件。见 [§17.4](#174-fork-与许可证)。
9. **用户点名的 26 个名字里，能直接拿来做获取插件的不多。** wechat-article-exporter 的单篇管线、Downvid 的抖音快手解析器、chubbyskills、bilibili-douyin-notes、xhs-account-teardown 可以 fork 或移植；转换侧的 AI-Video-Transcriber、indra 可以包装；其余是发布、创作工具或非商业项目。后补的 `easel` 是创作发布工作台，只借设计；`echo` 推断是已停更的 EchoTrace，属 `client-hook`，不接入。见 [§3](#3-用户点名的项目)。
10. **清单加两个声明字段，风险由宿主推出。** 提议插件声明 `access`（七层取值）和 `login`，宿主据此推出 `risk`；设置里按风险从低到高排序、默认先试低风险，高风险层单独确认，失败后自动换方式时不往更高风险的层换。见 [§17.1](#171-清单里声明获取层级)、[§17.2](#172-设置里怎么让用户选)。
11. **首批自己适配的方向。** 先做官方导出的导入和公开页单条获取，再做页内取列表和视觉层拿链接。按用户 2026-10-30 定的方向，自己维护的获取方式放进一个单独的 GitHub 组织，和官方组织分开，客户端不预置；组织名等产品改名定下后再起，届时修订 ADR-003。见 [§17.3](#173-首批自己适配的插件)、[§17.4](#174-fork-与许可证)。
12. **致谢候选有 260 个。** 在各表的项目名后标「（致谢）」。已下架或清空的、只记录层级的客户端注入类和运营类工具都没有标；有几个非商业许可的项目标了，意思是可以致谢，不能复用代码。见 [约定](#约定)。

<a id="1-账号风险分层"></a>
## 1. 账号风险分层

用户的判断是：纯视觉（computer use、手机自动化、iPhone 镜像）几乎没有账号风险，用登录态直接调接口是中风险，复现签名、Hook 客户端风险更高。证据基本支持这个顺序，有两处要修正：视觉层在微信上不是低风险；小红书上即使不登录也可能牵连账号。下面的风险档按 [§16](#16-账号风险的证据) 的证据定，分「无 / 低 / 中 / 高 / 很高」五档。

<a id="11-七个层级"></a>
### 1.1 七个层级

| 层级 | 怎么拿数据 | 用不用账号 | 平台怎么识别 | 账号风险 | 典型项目 |
|---|---|---|---|---|---|
| `official-export` | 平台的数据导出、用户授权的官方开放接口，或读用户已经导出的文件 | 导出时登录本人账号，或用开发者凭据；读文件时不碰账号 | 平台自己的功能，不存在识别问题；开放接口有配额和审核 | **无**。抖音隐私政策 3.3 提供「个人信息下载」，《个人信息保护法》第四十五条给了查阅、复制的权利；代价是要手动操作，导出包含哪些类别要逐个核实（[§16.1](#161-平台规则与可检测性)） | [Tencent/WeChatReading](https://github.com/Tencent/WeChatReading)、[xdevplatform/xurl](https://github.com/xdevplatform/xurl)、[purarue/google_takeout_parser](https://github.com/purarue/google_takeout_parser) |
| `public-web` | 不登录，读公开页面或分享链接，不复现签名 | 不用 | 只在 IP 和访客会话层：B站 对服务器 IP 返回 412，YouTube 访客约每小时 300 个视频，Instagram 不登录返回 401、429，豆瓣频繁取页出算力验证，公众号文章页对匿名抓取弹环境验证 | **无**，只有 IP 层的限制。小红书例外，记**低**：分享链接带跟踪参数，不登录批量取约 108 条后，复制链接的那个账号被下线（[JoeanAmier/XHS-Downloader #305](https://github.com/JoeanAmier/XHS-Downloader/issues/305)） | [JoeanAmier/XHS-Downloader](https://github.com/JoeanAmier/XHS-Downloader)、[wujunwei928/parse-video](https://github.com/wujunwei928/parse-video)、[lightpohl/podcast-dl](https://github.com/lightpohl/podcast-dl) |
| `vision-ui` | 读屏幕上渲染出来的内容（截图、无障碍树、OCR），用系统级输入驱动真实客户端 | 用：驱动的是用户已登录的客户端 | Android 上 App 能列出已开启的无障碍服务、能对非辅助工具隐藏敏感视图，有 Google Play 服务时还能拿到控制类应用的判定；Windows 能区分注入的鼠标事件；PC 微信 4.x 对新账号收紧 UI 树；iPhone 镜像没有公开的识别 API；操作节奏 | **低**；微信记**中**：协议 8.2.1.6 禁止用第三方软件读界面、自动操作，2026-01 点名辅助服务读取和模拟按键，UI 自动化读消息会被强制下线（[Hello-Mr-Crab/pywechat #260](https://github.com/Hello-Mr-Crab/pywechat/issues/260)） | [jfarcand/mirroir-mcp](https://github.com/jfarcand/mirroir-mcp)、[Genymobile/scrcpy](https://github.com/Genymobile/scrcpy)、[Hello-Mr-Crab/pywechat](https://github.com/Hello-Mr-Crab/pywechat) |
| `browser-session` | 用户自己的或自动化的浏览器带登录态读 DOM，或截获页面自己发出的请求 | 用 | `navigator.webdriver` 等自动化标记、浏览器指纹、已安装的扩展、操作节奏；抖音只放行页面内发起的请求 | **低到中**。用户日常浏览器里的扩展和油猴脚本没有找到使用者被处罚的报告（[prinsss/twitter-web-exporter](https://github.com/prinsss/twitter-web-exporter)）；自动化浏览器在小红书上记**高**，一两天内就有警告和封号（[xpzouying/xiaohongshu-mcp #728](https://github.com/xpzouying/xiaohongshu-mcp/issues/728)） | [Hannah2227/douyin-favorite-MyLike](https://github.com/Hannah2227/douyin-favorite-MyLike)、[prinsss/twitter-web-exporter](https://github.com/prinsss/twitter-web-exporter)、[xpzouying/xiaohongshu-mcp](https://github.com/xpzouying/xiaohongshu-mcp) |
| `session-api` | 代码里直接带用户 cookie 或 token 调平台接口，不复现签名 | 用 | 请求缺少页面脚本附带的参数；单会话限速；cookie 和设备、IP 不一致 | **中**。公众号后台出现频率控制、会话 2–4 天过期，微信读书出现「小黑屋」，B站 有用户的大会员被冻结（[yutto-dev/yutto #452](https://github.com/yutto-dev/yutto/issues/452)，因果不明）；yt-dlp 文档承认带账号有临时或永久封号的风险 | [wechat-article/wechat-article-exporter](https://github.com/wechat-article/wechat-article-exporter)、[yutto-dev/yutto](https://github.com/yutto-dev/yutto)、[afar1/fieldtheory-cli](https://github.com/afar1/fieldtheory-cli) |
| `private-api` | 在官方客户端之外复现签名、加密参数、设备指纹，或模拟 App | 多数要，也有不登录的 | 签名和指纹的特征、App 身份、请求节奏；平台一更新就失效 | **高**：小红书有永久封禁（[NanmiCoder/MediaCrawler #865](https://github.com/NanmiCoder/MediaCrawler/issues/865)）、45 秒间隔两三天被封（[cv-cat/Spider_XHS #150](https://github.com/cv-cat/Spider_XHS/issues/150)），X 老主号低量使用被封（[d60/twikit #338](https://github.com/d60/twikit/issues/338)），Instagram 备份自己 150 条帖子后永久封号（[instaloader/instaloader #2680](https://github.com/instaloader/instaloader/issues/2680)）。抖音、B站 多是几小时的临时限制，记**中** | [NanmiCoder/MediaCrawler](https://github.com/NanmiCoder/MediaCrawler)、[JoeanAmier/TikTokDownloader](https://github.com/JoeanAmier/TikTokDownloader)、[yt-dlp/yt-dlp](https://github.com/yt-dlp/yt-dlp) |
| `client-hook` | 注入或 Hook 官方客户端、解密本地数据库、装根证书做中间人 | 用 | 设备完整性检查（Play Integrity 发现 Hook 或 root 时判设备不完整）、客户端自身检查；平台法务 | **很高**：注入 PC 微信的用户被封 7 天、再犯被警告永久封号，还按实名牵连大号（[lich0821/WeChatFerry #126](https://github.com/lich0821/WeChatFerry/issues/126)）；微信 2019 年处理 Xposed 外挂直至限制登录。只读客户端缓存、不联网的做法检测面小，但同样违反协议 | [ltaoo/wx_channels_download](https://github.com/ltaoo/wx_channels_download)、[lich0821/WeChatFerry](https://github.com/lich0821/WeChatFerry)、[mzky/m4s-converter](https://github.com/mzky/m4s-converter) |

几条跨层级的规律（依据见 [§16.2](#162-issue-里的账号问题)）：

- **节奏比层级更直接。** 微信个人账号使用规范 3.4、微博协议 4.5.1 都把「明显高于正常用户的频率」本身列为违规，任何一层都要限速、按人的节奏。
- **新号更容易被拦，老主号也会被封。** X 上新注册的小号第一次搜索就被锁，注册 3 年的主号低量使用也被封。
- **写操作比读风险高，但小红书 2026 年只读也会触发。** 歇存只做读取，包装别人的工具时要关掉发帖、点赞、私信这类写命令。
- **云主机、虚拟机、VPN 和 IP 跳变在各层都加重风险。** 歇存跑在用户自己的电脑和家庭网络上，这一点对它有利。

<a id="12-视觉层的风险从哪里来"></a>
### 1.2 视觉层的风险从哪里来

**先说结论。** 「纯视觉没有账号风险」在下列情况基本成立：用 iPhone 镜像或电脑上的计算机操作工具，按人的节奏一条条读，平台不是微信，也不在云手机、云电脑上登录。下列情况不成立：微信（协议明文禁止，有处罚记录）；Android 上长时间开着无障碍服务或 USB 调试做自动操作（App 能查到）；高频连续滚动批量读取（节奏异常本身就违规）；在托管的云手机、远程电脑上登录（等于换了设备和 IP）。

风险来自五处：

1. **平台规则把「读界面」也算进去。** 《腾讯微信软件许可及服务协议》8.2.1.6 禁止借助第三方软件登录、使用或进行自动化操作，也禁止用它访问、读取软件的界面和数据（[来源](https://weixin.qq.com/agreement?lang=zh_CN)）。微信 2025-04 点名两类违规服务，一类是注入，一类是授权后监听或录屏、转储聊天记录（[IT之家](https://www.ithome.com/0/850/207.htm)）；2026-01 又点名录屏、截图、辅助服务读取、模拟按键（[快科技转述](https://tech.ifeng.com/c/8qKOznNW4za)）。其他平台的协议禁止的是「用自动化程序接入、收集」（抖音 5.1、小红书 4.1、B站 5.6、知乎二.9），字面上视觉自动化也在其中，但没有找到据此处罚的案例。
2. **设备环境能被 App 查到。**
   - Android：任何 App 都能列出已开启的无障碍服务，不需要权限（[AccessibilityManager](https://developer.android.com/reference/android/view/accessibility/AccessibilityManager)）；API 34 起 App 能让非辅助工具读不到、点不了敏感视图（[View](https://developer.android.com/reference/android/view/View#setAccessibilityDataSensitive(int))）；有 Google Play 服务时，Play Integrity 会报告正在运行的录屏类和控制类应用（[文档](https://developer.android.com/google/play/integrity/verdicts)），国内多数机型是否适用待核实；USB 调试开关现在对第三方 App 恒返回 0，旧系统和国内 ROM 是否跟进待核实；uiautomator2 的辅助应用包名固定（[openatx/uiautomator2 #808](https://github.com/openatx/uiautomator2/issues/808)）。单个触摸事件本身没有公开的「注入」标记，App 要靠这些环境信号和节奏判断。
   - Windows：低级鼠标钩子能看到 `LLMHF_INJECTED` 标志（[文档](https://learn.microsoft.com/en-us/windows/win32/api/winuser/ns-winuser-msllhookstruct)）；PC 微信 4.x 对从没开过无障碍模式的新账号不再暴露 UI Automation 树（[pywechat 的说明](https://github.com/Hello-Mr-Crab/pywechat/blob/main/Weixin4.0.md)）。
   - macOS：事件里有来源进程 PID 字段，没有找到客户端读取它的资料。
   - iOS：App 能感知录屏和投屏（`UIScreen.isCaptured`、`sceneCaptureState`）；iPhone 镜像目前没有公开 API 能识别，Apple DTS 2024-07 在开发者论坛这样答复（[thread 759314](https://developer.apple.com/forums/thread/759314)）。镜像时 iPhone 保持锁定并显示正被哪台 Mac 使用，欧盟不可用（[Apple 120421](https://support.apple.com/en-us/120421)），中国大陆能否使用待核实。
3. **有公开的处罚记录，集中在微信。** 豆包手机助手 2025-12 用系统级注入权限操作微信，用户被退出、无法登录（[澎湃新闻](https://www.thepaper.cn/newsDetail_forward_32094236)）；据 KrASIA 2026-06 的报道，微信接受厂商系统助手经加密协议调用微信内部 agent，不接受读屏加模拟点击的 GUI agent（[KrASIA](https://amp.kr-asia.com/wechat-opens-the-door-to-phone-based-ai-agents-on-its-own-terms)）；用 pywinauto 读 PC 微信的项目反映，读群消息 5–10 次就会被强制下线，夜间、云服务器、虚拟机、远程桌面下更容易触发，维护者说没见过因此封号（[Hello-Mr-Crab/pywechat #267](https://github.com/Hello-Mr-Crab/pywechat/issues/267)、[Hello-Mr-Crab/pywechat #181](https://github.com/Hello-Mr-Crab/pywechat/issues/181)）。手机和 iPhone 镜像类通用 Agent（Open-AutoGLM、mobilerun、mobile-use、mirroir-mcp）的 issue 里没有处罚报告，这只说明没有报告。
4. **节奏异常本身违规。** 微信个人账号使用规范 3.4、微博协议 4.5.1 都按频率判断，和获取方式无关；视觉层同样要单条触发、按人的节奏、遇到验证就停。
5. **不是账号风险、但要计入的代价。**
   - 隐私：有的工具把截图交给云端模型（[Elucidator-hky/phone-agent](https://github.com/Elucidator-hky/phone-agent) 的视觉兜底传到阿里云百炼），录屏记忆类默认记录全部屏幕（[screenpipe/screenpipe](https://github.com/screenpipe/screenpipe)）。
   - 换设备：托管的云手机、远程电脑（[bytedance/UI-TARS-desktop](https://github.com/bytedance/UI-TARS-desktop) 的远程 operator、[droidrun/mobilerun](https://github.com/droidrun/mobilerun) 的云手机）等于在陌生设备上登录。
   - 拿不全：只能拿到渲染出来的内容，拿不到原始文件；支付等安全页面截图为黑（[zai-org/Open-AutoGLM](https://github.com/zai-org/Open-AutoGLM)），受版权保护的内容在 iPhone 镜像里截图为黑（[nickatnight96/iphone-mirror-mcp](https://github.com/nickatnight96/iphone-mirror-mcp)）；客户端改版就要重写流程。

<a id="13-混合路径视觉层拿链接公开页面取内容"></a>
### 1.3 混合路径：视觉层拿链接，公开页面取内容

思路是把「列出我收藏了什么」和「取回每一条的内容」拆开：前一步必须在登录的一侧做（官方导出、用户自己的浏览器、视觉层驱动客户端，或者用户手动分享），后一步尽量不带账号。这样账号只在列清单时出现，而且列清单的节奏可以和人工浏览一样。

| 平台 | 拿链接（登录侧） | 取内容（不登录） | 可行性 | 限制与依据 |
|---|---|---|---|---|
| 抖音 | 页内取收藏列表（[Hannah2227/douyin-favorite-MyLike](https://github.com/Hannah2227/douyin-favorite-MyLike)），或手动分享 | 读分享页的服务端渲染数据（[wujunwei928/parse-video](https://github.com/wujunwei928/parse-video)、[lyxdream/obsidian-content-capture-backend](https://github.com/lyxdream/obsidian-content-capture-backend)） | 可行 | 分享页字段随前端改版变化；不登录时单作品接口约 200 次后限流、IP 约两小时解封（[Evil0ctal/Douyin_TikTok_Download_API #626](https://github.com/Evil0ctal/Douyin_TikTok_Download_API/issues/626)）；yt-dlp 的抖音路径要新鲜的访客 cookie，不传基本不可用 |
| 快手 | 手动分享 | 按分享链接取单条（[JoeanAmier/KS-Downloader](https://github.com/JoeanAmier/KS-Downloader)、[Downvid/downvid](https://github.com/Downvid/downvid) 的快手解析器） | 可行 | 频繁请求会要求验证（parse-video 的 issue 报过）；按账号批量在 KS-Downloader 里被注释掉 |
| 小红书 | 用户自己浏览器里的油猴脚本导出清单（[wkwunju/xhs-vault](https://github.com/wkwunju/xhs-vault)），或只读当前可见的收藏卡片（[themrv1ck/xiaohongshu-web-collection-organizing](https://github.com/themrv1ck/xiaohongshu-web-collection-organizing)） | 凭带 `xsec_token` 的链接取单篇（[chenjin-cmd/xhs-account-teardown](https://github.com/chenjin-cmd/xhs-account-teardown)、[JoeanAmier/XHS-Downloader](https://github.com/JoeanAmier/XHS-Downloader)） | 有条件可行 | token 和笔记绑定，错配时返回空数据；token 只在会话内有效，几小时后链接打开是 404（[danzey0425-lgtm/web-image-sourcing](https://github.com/danzey0425-lgtm/web-image-sourcing)）；分享链接带跟踪参数，不登录批量取后复制链接的账号被下线（[JoeanAmier/XHS-Downloader #305](https://github.com/JoeanAmier/XHS-Downloader/issues/305)）。只适合单条、低频，拿到链接尽快取 |
| B站 | 用户浏览器里的扩展导出收藏夹（[ayasa520/bilibili-favorites-exporter](https://github.com/ayasa520/bilibili-favorites-exporter)） | 公开视频的元数据、字幕 | 有条件可行 | 服务器 IP 上匿名请求常被 412（[Panniantong/Agent-Reach #180](https://github.com/Panniantong/Agent-Reach/issues/180)），家庭网络问题少；2026-06 起缺指纹参数的部分接口返回 412（[JefferyHcool/BiliNote #397](https://github.com/JefferyHcool/BiliNote/issues/397)）；B站 的律师函针对复现签名的工具，取内容要避开需要签名的接口 |
| 公众号 | PC 微信收藏里逐条复制文章链接（[Hello-Mr-Crab/pywechat](https://github.com/Hello-Mr-Crab/pywechat)、[pengyulong/wechat-favorites-exporter](https://github.com/pengyulong/wechat-favorites-exporter)） | 读文章页（[wechat-article/wechat-article-exporter](https://github.com/wechat-article/wechat-article-exporter) 的单篇管线、[fengxxc/wechatmp2markdown](https://github.com/fengxxc/wechatmp2markdown)） | 可行，是目前最完整的组合 | 匿名抓取偶尔出现环境验证页（[cooderl/wewe-rss #388](https://github.com/cooderl/wewe-rss/issues/388)），自动化浏览器抓取也被拦过（[Bwkyd/wexin-read-mcp #3](https://github.com/Bwkyd/wexin-read-mcp/issues/3)）；微信对 UI 自动化有强制下线记录，复制链接要按人的节奏 |
| 视频号 | 手动分享，或客户端里看过时录屏 | 不登录只能拿基础信息（[ucmao/media-parser](https://github.com/ucmao/media-parser) 的说明），文中嵌的视频号卡片只有元数据和封面（[jj-cheng25/weixin-articles-mcp](https://github.com/jj-cheng25/weixin-articles-mcp)），视频地址要借腾讯元宝的登录态换凭证（[joeseesun/qiaomu-clipper](https://github.com/joeseesun/qiaomu-clipper)、[MuZiJin701/wx-video-account-notes](https://github.com/MuZiJin701/wx-video-account-notes)） | 不完整 | 账号风险转到元宝账号，已有部署者的元宝账号被封（[ltaoo/wx_channels_download #495](https://github.com/ltaoo/wx_channels_download/issues/495)）；另一条路是录下播放的声音再转写（[insidegui/AudioCap](https://github.com/insidegui/AudioCap)、[Genymobile/scrcpy](https://github.com/Genymobile/scrcpy)） |
| 微博 | 用户浏览器里翻自己的收藏（[Chilfish/Weibo-archiver](https://github.com/Chilfish/Weibo-archiver) 的油猴版） | 访客方式取公开微博（[dataabc/weibo-crawler](https://github.com/dataabc/weibo-crawler)、[qinyuanpei/mcp-server-weibo](https://github.com/qinyuanpei/mcp-server-weibo)） | 可行 | 访客路径只有 IP 层限制 |
| 知乎 | 收藏夹页面或开放平台接口（[klarkxy/zhihu-search](https://github.com/klarkxy/zhihu-search)） | 公开收藏夹和回答不登录可取（[Geralt-TYH/obsidian-zhihu-crawler](https://github.com/Geralt-TYH/obsidian-zhihu-crawler)） | 可行 | 私密收藏夹要带 cookie |
| 豆瓣 | 本人主页的标记页 | 公开 RSS 做日常增量（[cosformula/douban-sync-skill](https://github.com/cosformula/douban-sync-skill)） | 可行 | 全量翻页会遇到算力验证（[Doubak/doubak-extension](https://github.com/Doubak/doubak-extension) 记录） |
| YouTube | Google Takeout 里的「稍后观看」CSV（[ItsMaciek/WatchNow](https://github.com/ItsMaciek/WatchNow)） | 不登录取字幕和元数据（[jdepoix/youtube-transcript-api](https://github.com/jdepoix/youtube-transcript-api)） | 可行，列表一步是官方导出 | 访客约每小时 300 个视频；云服务器 IP 常被拦 |
| TikTok | 官方数据导出里的喜欢、收藏（[samirelanduk/tiktok-save](https://github.com/samirelanduk/tiktok-save)、[commitmaniac/tdp](https://github.com/commitmaniac/tdp)） | 逐条取公开视频 | 可行，列表一步是官方导出 | 导出里的下载链接会过期；新版导出的字段名有变化 |
| Instagram | 官方数据导出（[meizuflux/scrollback](https://github.com/meizuflux/scrollback)） | 不登录取公开帖子 | 不稳定 | 不登录常返回 401「请稍后再试」和 429（[instaloader/instaloader #2501](https://github.com/instaloader/instaloader/issues/2501)） |

**手动分享入口也是一条低风险的路。** 用户在 App 里点「分享」，把链接交给本机服务，再由公开页取内容，整个过程没有自动化：iOS 有快捷指令加 Scriptable（[ceciliayu56-cell/xhs-to-obsidian-scriptable](https://github.com/ceciliayu56-cell/xhs-to-obsidian-scriptable)）、轻点背面触发（[Hype-yolo/backtap-douyin](https://github.com/Hype-yolo/backtap-douyin)）、经 `obsidian://` 唤起插件（[CH-YYK/obsidian-rednote](https://github.com/CH-YYK/obsidian-rednote)）；Android 有把分享文本塞进 HTTP 请求、直发局域网本机服务的 [Waboodoo/HTTP-Shortcuts](https://github.com/Waboodoo/HTTP-Shortcuts)，以及系统分享到 App 的 [JunkFood02/Seal](https://github.com/JunkFood02/Seal)、[sanzoghenzo/markdownr](https://github.com/sanzoghenzo/markdownr)；聊天机器人入口有 [wabarc/wayback](https://github.com/wabarc/wayback)。要注意有的入口经过作者的服务器（快捷指令的公共试用服务、[eduaiot/eduaiot-wechat-sync](https://github.com/eduaiot/eduaiot-wechat-sync) 的小程序收件箱），链接和内容会离开本机。

**限制。** 一是列清单这一步本身仍在登录侧，视觉层逐条复制链接的现成流程只在 PC 微信收藏上找到，抖音、小红书、B站、快手、微博、知乎的 App 收藏页都没有（待核实）。二是不登录取内容仍有 IP 层的风控，家庭网络比服务器好。三是链接里的跟踪参数、会话 token 可能把匿名请求关联回账号，小红书已有实例。

<a id="2-覆盖矩阵每个平台有哪些层级可选"></a>
## 2. 覆盖矩阵：每个平台有哪些层级可选

每格列 1–3 个代表项目，优先列专门针对这个平台的项目。读表前注意三点：

- **条目级层级取最高一层。** 多平台工具（yt-dlp、RSSHub、OpenCLI 等）在多数平台上走的路比条目级层级低，这里不把它们按条目级层级填进每个平台；只有条目自己的说明写明了某个平台走更低一层时，才放进那一格，并标「†」。
- **视觉层一列只列点名了这个平台的项目。** 手机和桌面自动化工具不分平台，任何 App 都能驱动，统一放在最后一行；所以这一列的「未找到」只说明没有专门针对这个平台的视觉方案。
- **「不适用」表示这一层在这个平台上没有对应的东西**，例如微信聊天没有公开网页。

| 平台 | official-export | public-web | vision-ui | browser-session | session-api | private-api | client-hook |
|---|---|---|---|---|---|---|---|
| 抖音 | 平台的「个人信息下载」；[ZhangYukwiver/DouyinDataReview](https://github.com/ZhangYukwiver/DouyinDataReview)† 能导入 | [wujunwei928/parse-video](https://github.com/wujunwei928/parse-video)、[lyxdream/obsidian-content-capture-backend](https://github.com/lyxdream/obsidian-content-capture-backend)、[Hype-yolo/backtap-douyin](https://github.com/Hype-yolo/backtap-douyin) | [zai-org/Open-AutoGLM](https://github.com/zai-org/Open-AutoGLM)、[danzey0425-lgtm/web-image-sourcing](https://github.com/danzey0425-lgtm/web-image-sourcing) | [Hannah2227/douyin-favorite-MyLike](https://github.com/Hannah2227/douyin-favorite-MyLike)、[mlbb229229-create/douyin-favorites-mcp](https://github.com/mlbb229229-create/douyin-favorites-mcp)、[tars1230/douyin-favorites-to-knowledge](https://github.com/tars1230/douyin-favorites-to-knowledge) | [yumingtao/douyin-favorites-sync-backend](https://github.com/yumingtao/douyin-favorites-sync-backend)、[Downvid/downvid](https://github.com/Downvid/downvid)† | [JoeanAmier/TikTokDownloader](https://github.com/JoeanAmier/TikTokDownloader)、[Evil0ctal/Douyin_TikTok_Download_API](https://github.com/Evil0ctal/Douyin_TikTok_Download_API)、[NanmiCoder/MediaCrawler](https://github.com/NanmiCoder/MediaCrawler) | [cnbattle/douyin](https://github.com/cnbattle/douyin)、[huami1314/DYYY](https://github.com/huami1314/DYYY)、[putyy/res-downloader](https://github.com/putyy/res-downloader) |
| 快手 | 未找到 | [JoeanAmier/KS-Downloader](https://github.com/JoeanAmier/KS-Downloader)、[wujunwei928/parse-video](https://github.com/wujunwei928/parse-video)、[Downvid/downvid](https://github.com/Downvid/downvid)† | 未找到 | [tmwgsicp/ForgeRSS](https://github.com/tmwgsicp/ForgeRSS) | [hanliang97/MatrixMedia](https://github.com/hanliang97/MatrixMedia)（只读自己账号的作品数据） | [NanmiCoder/MediaCrawler](https://github.com/NanmiCoder/MediaCrawler)、[cv-cat/catbus](https://github.com/cv-cat/catbus)、[iszhouhua/social-media-copilot](https://github.com/iszhouhua/social-media-copilot) | [putyy/res-downloader](https://github.com/putyy/res-downloader) |
| TikTok | 平台的数据导出；[commitmaniac/tdp](https://github.com/commitmaniac/tdp)、[zollo/tiktok-downloader](https://github.com/zollo/tiktok-downloader)、[Inoue-AI/Inoue-AI-TikTok-SDK](https://github.com/Inoue-AI/Inoue-AI-TikTok-SDK) | [davidteather/TikTok-Api](https://github.com/davidteather/TikTok-Api) | 未找到 | [samirelanduk/tiktok-save](https://github.com/samirelanduk/tiktok-save) | [JBGruber/traktok](https://github.com/JBGruber/traktok) | [Johnserf-Seed/f2](https://github.com/Johnserf-Seed/f2)、[JoeanAmier/TikTokDownloader](https://github.com/JoeanAmier/TikTokDownloader)、[Evil0ctal/Douyin_TikTok_Download_API](https://github.com/Evil0ctal/Douyin_TikTok_Download_API) | [BandarHL/BHTikTok](https://github.com/BandarHL/BHTikTok) |
| 头条/西瓜 | 未找到 | [wujunwei928/parse-video](https://github.com/wujunwei928/parse-video)、[NanmiCoder/NewsCrawler](https://github.com/NanmiCoder/NewsCrawler)†（头条文章不登录） | 未找到 | [mingjuner123-spec/wechat-inbox-sync](https://github.com/mingjuner123-spec/wechat-inbox-sync) | 未找到 | [iawia002/lux](https://github.com/iawia002/lux)、[jackwener/OpenCLI](https://github.com/jackwener/OpenCLI) | 未找到 |
| 小红书 | 未找到 | [JoeanAmier/XHS-Downloader](https://github.com/JoeanAmier/XHS-Downloader)、[chenjin-cmd/xhs-account-teardown](https://github.com/chenjin-cmd/xhs-account-teardown)、[bonchiang96/xiaohongshu-importer](https://github.com/bonchiang96/xiaohongshu-importer) | [Elucidator-hky/phone-agent](https://github.com/Elucidator-hky/phone-agent)、[danzey0425-lgtm/web-image-sourcing](https://github.com/danzey0425-lgtm/web-image-sourcing)、[zai-org/Open-AutoGLM](https://github.com/zai-org/Open-AutoGLM) | [xpzouying/xiaohongshu-mcp](https://github.com/xpzouying/xiaohongshu-mcp)、[wkwunju/xhs-vault](https://github.com/wkwunju/xhs-vault)、[themrv1ck/xiaohongshu-web-collection-organizing](https://github.com/themrv1ck/xiaohongshu-web-collection-organizing) | [chenxiachan/xhs-claude-skills](https://github.com/chenxiachan/xhs-claude-skills) | [cv-cat/Spider_XHS](https://github.com/cv-cat/Spider_XHS)、[lucasygu/redbook](https://github.com/lucasygu/redbook)、[NanmiCoder/MediaCrawler](https://github.com/NanmiCoder/MediaCrawler) | [SimpsonGet/XhsWatermarkRemove](https://github.com/SimpsonGet/XhsWatermarkRemove)、[cakeni/super-drag](https://github.com/cakeni/super-drag) |
| B站 | 未找到 | [34892002/bilibili-mcp-js](https://github.com/34892002/bilibili-mcp-js) | [OpenBMB/AgentCPM-GUI](https://github.com/OpenBMB/AgentCPM-GUI)、[zai-org/Open-AutoGLM](https://github.com/zai-org/Open-AutoGLM) | [ayasa520/bilibili-favorites-exporter](https://github.com/ayasa520/bilibili-favorites-exporter)、[IndieKKY/bilibili-subtitle](https://github.com/IndieKKY/bilibili-subtitle)、[the1812/Bilibili-Evolved](https://github.com/the1812/Bilibili-Evolved) | [yutto-dev/yutto](https://github.com/yutto-dev/yutto)、[LifeArchiveProject/HistoryHub](https://github.com/LifeArchiveProject/HistoryHub) | [amtoaer/bili-sync](https://github.com/amtoaer/bili-sync)、[XZXZZX-Ai/bilibili-mcp](https://github.com/XZXZZX-Ai/bilibili-mcp)、[yt-dlp/yt-dlp](https://github.com/yt-dlp/yt-dlp) | [mzky/m4s-converter](https://github.com/mzky/m4s-converter)、[yujincheng08/BiliRoaming](https://github.com/yujincheng08/BiliRoaming)（已归档） |
| YouTube | [purarue/google_takeout_parser](https://github.com/purarue/google_takeout_parser)、[dogsheep/google-takeout-to-sqlite](https://github.com/dogsheep/google-takeout-to-sqlite) | [ItsMaciek/WatchNow](https://github.com/ItsMaciek/WatchNow)、[RSS-Bridge/rss-bridge](https://github.com/RSS-Bridge/rss-bridge) | 未找到 | [joeseesun/qiaomu-clipper](https://github.com/joeseesun/qiaomu-clipper)、[zotero/translators](https://github.com/zotero/translators) | 未找到 | [yt-dlp/yt-dlp](https://github.com/yt-dlp/yt-dlp)、[LuanRT/YouTube.js](https://github.com/LuanRT/YouTube.js)、[jdepoix/youtube-transcript-api](https://github.com/jdepoix/youtube-transcript-api) | 未找到 |
| 公众号 | 未找到 | [wechat-article/wechat-article-exporter](https://github.com/wechat-article/wechat-article-exporter)†（单篇管线）、[fengxxc/wechatmp2markdown](https://github.com/fengxxc/wechatmp2markdown)、[Bwkyd/wexin-read-mcp](https://github.com/Bwkyd/wexin-read-mcp) | [Hello-Mr-Crab/pywechat](https://github.com/Hello-Mr-Crab/pywechat)、[pengyulong/wechat-favorites-exporter](https://github.com/pengyulong/wechat-favorites-exporter)、[halohazhang/wechat-favorites-to-obsidian](https://github.com/halohazhang/wechat-favorites-to-obsidian) | 未找到 | [rachelos/we-mp-rss](https://github.com/rachelos/we-mp-rss)、[cooderl/wewe-rss](https://github.com/cooderl/wewe-rss)（已归档） | [jackwener/wechat-article-to-markdown](https://github.com/jackwener/wechat-article-to-markdown)、[tmwgsicp/wechat-download-api](https://github.com/tmwgsicp/wechat-download-api) | [zhuyansen/wx-favorites-report](https://github.com/zhuyansen/wx-favorites-report)、[ltaoo/wx_channels_download](https://github.com/ltaoo/wx_channels_download) |
| 视频号 | 未找到 | [jj-cheng25/weixin-articles-mcp](https://github.com/jj-cheng25/weixin-articles-mcp)（只有文中卡片的元数据） | 未找到 | [joeseesun/qiaomu-clipper](https://github.com/joeseesun/qiaomu-clipper)（元宝页内） | [MuZiJin701/wx-video-account-notes](https://github.com/MuZiJin701/wx-video-account-notes)、[ucmao/media-parser](https://github.com/ucmao/media-parser)† | [z-mio/ParseHub](https://github.com/z-mio/ParseHub) | [ltaoo/wx_channels_download](https://github.com/ltaoo/wx_channels_download)、[nobiyou/wx_channel](https://github.com/nobiyou/wx_channel)、[putyy/res-downloader](https://github.com/putyy/res-downloader) |
| 微信读书 | [Tencent/WeChatReading](https://github.com/Tencent/WeChatReading)、[zhaohongxuan/obsidian-weread-plugin](https://github.com/zhaohongxuan/obsidian-weread-plugin)、[malinkang/weread2notion](https://github.com/malinkang/weread2notion) | 未找到 | 未找到 | [Higurashi-kagome/wereader](https://github.com/Higurashi-kagome/wereader) | [freestylefly/mcp-server-weread](https://github.com/freestylefly/mcp-server-weread) | [wallechfox/wechat-mp-archive](https://github.com/wallechfox/wechat-mp-archive) | 未找到 |
| 微信聊天与收藏 | [BlueMatthew/WechatExporter](https://github.com/BlueMatthew/WechatExporter)（iPhone 备份） | 不适用 | [freestylefly/WeChatBridge](https://github.com/freestylefly/WeChatBridge)、[Hello-Mr-Crab/pywechat](https://github.com/Hello-Mr-Crab/pywechat)、[dustin573/wechat-mcp](https://github.com/dustin573/wechat-mcp) | 不适用 | 未找到 | 未找到 | [hicccc77/WeFlow](https://github.com/hicccc77/WeFlow)、[lich0821/WeChatFerry](https://github.com/lich0821/WeChatFerry)（已归档）、[LC044/WeChatMsg](https://github.com/LC044/WeChatMsg)（代码已移除） |
| 微博 | [wangcch/weibo-mcp](https://github.com/wangcch/weibo-mcp)（开放接口，不含收藏） | [dataabc/weibo-crawler](https://github.com/dataabc/weibo-crawler)、[qinyuanpei/mcp-server-weibo](https://github.com/qinyuanpei/mcp-server-weibo) | [zai-org/Open-AutoGLM](https://github.com/zai-org/Open-AutoGLM) | 未找到 | [Chilfish/Weibo-archiver](https://github.com/Chilfish/Weibo-archiver)、[dataabc/weiboSpider](https://github.com/dataabc/weiboSpider) | [NanmiCoder/MediaCrawler](https://github.com/NanmiCoder/MediaCrawler)、[Johnserf-Seed/f2](https://github.com/Johnserf-Seed/f2) | 未找到 |
| 知乎 | [klarkxy/zhihu-search](https://github.com/klarkxy/zhihu-search)（知乎开放平台） | [Geralt-TYH/obsidian-zhihu-crawler](https://github.com/Geralt-TYH/obsidian-zhihu-crawler) | [zai-org/Open-AutoGLM](https://github.com/zai-org/Open-AutoGLM) | [hc-tec/my-collection-skills](https://github.com/hc-tec/my-collection-skills) | [handsomestWei/zhihu-fetch-skill](https://github.com/handsomestWei/zhihu-fetch-skill)、[Tendo33/zhihu-md](https://github.com/Tendo33/zhihu-md)、[zimya/zhihu_obsidian](https://github.com/zimya/zhihu_obsidian) | [KrisTHL181/zhihu-cli](https://github.com/KrisTHL181/zhihu-cli)、[zly2006/zhihu-plus-plus](https://github.com/zly2006/zhihu-plus-plus)、[NanmiCoder/MediaCrawler](https://github.com/NanmiCoder/MediaCrawler) | 未找到 |
| 豆瓣 | 未找到 | [cosformula/douban-sync-skill](https://github.com/cosformula/douban-sync-skill) | [zai-org/Open-AutoGLM](https://github.com/zai-org/Open-AutoGLM) | [moonlin1213/cove-douban-mcp](https://github.com/moonlin1213/cove-douban-mcp)、[zotero/translators](https://github.com/zotero/translators) | [Marvae/douban-cli](https://github.com/Marvae/douban-cli)、[doufen-org/tofu](https://github.com/doufen-org/tofu)、[Wanxp/obsidian-douban](https://github.com/Wanxp/obsidian-douban) | [Doubak/doubak-extension](https://github.com/Doubak/doubak-extension)、[lizheming/doumark-action](https://github.com/lizheming/doumark-action) | 未找到 |
| 贴吧 | 未找到 | 未找到 | 未找到 | 未找到 | 未找到 | [lumina37/aiotieba](https://github.com/lumina37/aiotieba)、[NanmiCoder/MediaCrawler](https://github.com/NanmiCoder/MediaCrawler) | 未找到 |
| 即刻 | 未找到 | 未找到 | 未找到 | 未找到 | [soyaine/jocker-chrome-extension](https://github.com/soyaine/jocker-chrome-extension)、[wujieli0207/jike-export-extension](https://github.com/wujieli0207/jike-export-extension) | [open-jike/jike-sdk](https://github.com/open-jike/jike-sdk) | 未找到 |
| X | [xdevplatform/xurl](https://github.com/xdevplatform/xurl)、[steipete/birdclaw](https://github.com/steipete/birdclaw)、[timhutton/twitter-archive-parser](https://github.com/timhutton/twitter-archive-parser) | [wujunwei928/parse-video](https://github.com/wujunwei928/parse-video) | 未找到 | [prinsss/twitter-web-exporter](https://github.com/prinsss/twitter-web-exporter) | [afar1/fieldtheory-cli](https://github.com/afar1/fieldtheory-cli)、[sytelus/xarchive](https://github.com/sytelus/xarchive)、[alexknowshtml/smaug](https://github.com/alexknowshtml/smaug) | [public-clis/twitter-cli](https://github.com/public-clis/twitter-cli)、[vladkens/twscrape](https://github.com/vladkens/twscrape)、[mikf/gallery-dl](https://github.com/mikf/gallery-dl) | 未找到 |
| Instagram | [meizuflux/scrollback](https://github.com/meizuflux/scrollback)、[timelinize/timelinize](https://github.com/timelinize/timelinize)、[karlicoss/HPI](https://github.com/karlicoss/HPI) | 未找到 | 未找到 | [jasonsbmoc/gramcracker](https://github.com/jasonsbmoc/gramcracker) | [tairqaldy/resurfly](https://github.com/tairqaldy/resurfly)、[instaloader/instaloader](https://github.com/instaloader/instaloader)†（加 `--no-iphone`） | [instaloader/instaloader](https://github.com/instaloader/instaloader)、[subzeroid/instagrapi](https://github.com/subzeroid/instagrapi) | 未找到 |
| Reddit | [karlicoss/rexport](https://github.com/karlicoss/rexport)、[Serene-Arc/bulk-downloader-for-reddit](https://github.com/Serene-Arc/bulk-downloader-for-reddit) | [xavdid/reddit-user-to-sqlite](https://github.com/xavdid/reddit-user-to-sqlite)、[RSS-Bridge/rss-bridge](https://github.com/RSS-Bridge/rss-bridge) | 未找到 | 未找到 | 未找到 | 未找到 | 未找到 |
| Telegram | [knadh/tg-archive](https://github.com/knadh/tg-archive)、[soberhacker/obsidian-telegram-sync](https://github.com/soberhacker/obsidian-telegram-sync) | [RSS-Bridge/rss-bridge](https://github.com/RSS-Bridge/rss-bridge) | 未找到 | 未找到 | 未找到 | 未找到 | [iyear/tdl](https://github.com/iyear/tdl) |
| Pinterest | [pinterest/api-quickstart](https://github.com/pinterest/api-quickstart) | [sean1832/pinterest-dl](https://github.com/sean1832/pinterest-dl) | 未找到 | 未找到 | 未找到 | 未找到 | 未找到 |
| 播客 | [scripting/opmlPackage](https://github.com/scripting/opmlPackage)（OPML 订阅列表） | [lightpohl/podcast-dl](https://github.com/lightpohl/podcast-dl)、[xiaoleiy/podpull](https://github.com/xiaoleiy/podpull)、[chhoumann/PodNotes](https://github.com/chhoumann/PodNotes) | [insidegui/AudioCap](https://github.com/insidegui/AudioCap)、[makeusabrew/audiotee](https://github.com/makeusabrew/audiotee)（录本机播放的声音） | [joeseesun/qiaomu-clipper](https://github.com/joeseesun/qiaomu-clipper) | 未找到 | [ultrazg/xyz](https://github.com/ultrazg/xyz)、[MosesHe/xiaoyuzhoufm-mcp](https://github.com/MosesHe/xiaoyuzhoufm-mcp) | [maximbilan/podcrack](https://github.com/maximbilan/podcrack)（Apple Podcasts 缓存） |
| 通用网页 | [FlyingWolFox/Netscape-Bookmarks-File-Parser](https://github.com/FlyingWolFox/Netscape-Bookmarks-File-Parser)（书签导出）、[karlicoss/promnesia](https://github.com/karlicoss/promnesia)、[karlicoss/instapexport](https://github.com/karlicoss/instapexport) | [Y2Z/monolith](https://github.com/Y2Z/monolith)、[adbar/trafilatura](https://github.com/adbar/trafilatura)、[karakeep-app/karakeep](https://github.com/karakeep-app/karakeep) | [screenpipe/screenpipe](https://github.com/screenpipe/screenpipe)、[openrecall/openrecall](https://github.com/openrecall/openrecall) | [gildas-lormeau/SingleFile](https://github.com/gildas-lormeau/SingleFile)、[webrecorder/archiveweb.page](https://github.com/webrecorder/archiveweb.page)、[browser-use/browser-use](https://github.com/browser-use/browser-use) | 未找到 | [ArchiveBox/ArchiveBox](https://github.com/ArchiveBox/ArchiveBox)、[obsidianmd/obsidian-clipper](https://github.com/obsidianmd/obsidian-clipper) | 未找到 |
| 任意 App（通用） | — | — | [jfarcand/mirroir-mcp](https://github.com/jfarcand/mirroir-mcp)（iPhone 镜像）、[Genymobile/scrcpy](https://github.com/Genymobile/scrcpy)、[openclaw/Peekaboo](https://github.com/openclaw/Peekaboo) | — | — | — | — |

† 的依据：DouyinDataReview 能导入抖音官方「个人信息下载」档案；Downvid 的快手解析器发不带登录的请求、抖音解析器带 cookie 请求网页详情接口；NewsCrawler 取头条文章不登录；wechat-article-exporter 的单篇管线不需要登录；media-parser 的视频号路径借元宝登录态；instaloader 加 `--no-iphone` 后不再模拟 App。这些都是对单个平台路径的判断，层级本身以各节表格为准。

小程序没有单列一行：只找到两个 `client-hook` 项目（[biggerstar/wedecode](https://github.com/biggerstar/wedecode)、[putyy/res-downloader](https://github.com/putyy/res-downloader)），拿到的是代码而不是内容；小程序里的内容只能走视觉层截图、录屏，再交给 OCR。

从矩阵看出的空缺：

- **官方导出**：抖音、TikTok、YouTube、X、Instagram、Reddit、微信读书、知乎、微博都有；小红书、B站、快手、头条、公众号、豆瓣没有找到平台导出或开放接口的工具，抖音的导出包含哪些类别也待核实。
- **视觉层**：只有微信（PC 客户端）有现成的收藏读取流程；其余平台要用通用工具自己写流程（[§17.3](#173-首批自己适配的插件)）。
- **视频号**：没有官方导出，没有不登录取视频的路，`browser-session` 和 `session-api` 两格都靠腾讯元宝的登录态。
- **快手、头条/西瓜**：没有用户浏览器里取收藏列表的工具。
- **Reddit、Telegram、Pinterest**：官方接口和公开页面已经够用，高风险层没有必要。

<a id="3-用户点名的项目"></a>
## 3. 用户点名的项目

用户先点了 24 个名字，对应 27 个仓库：逸尘名下另有 3 个仓库一并列出；之后又补了 `easel` 和 `echo` 两个，合计 26 个名字、29 个仓库，`echo` 的对应关系是推断。「用户写的」一列照抄原文。这些项目在各类别表里也各有一行。

| 项目 | 用户写的 | 层级 | 登录 | 许可证 | 歇存怎么用 |
|---|---|---|---|---|---|
| [pabatiba89-sys/kali-publish](https://github.com/pabatiba89-sys/kali-publish) | `kali-publish` | `not-applicable`（发布） | 必须 | MIT | 不接入；借登录和会话导出的经验 |
| [dreammis/social-auto-upload](https://github.com/dreammis/social-auto-upload) | `social-auto-upload` | `not-applicable`（发布，交互等同 `browser-session`） | 必须 | MIT | 不接入；借会话检查、Skill 分发、人工验证交接 |
| [NanmiCoder/MediaCrawler](https://github.com/NanmiCoder/MediaCrawler) | `NanmiCoder/MediaCrawler` | `private-api` | 必须 | 自定义非商业学习许可 | 只作 `private-api` 层的风险例子；不 fork、不包装 |
| [YILS-LIN/short-video-factory](https://github.com/YILS-LIN/short-video-factory) | `short-video-factory` | `not-applicable` | — | AGPL-3.0 | 不接入；借签名清单校验 |
| [wendy7756/AI-Video-Transcriber](https://github.com/wendy7756/AI-Video-Transcriber) | `ai-videio-transcriber` | `private-api`（链接模式跟随 yt-dlp） | 不用 | Apache-2.0 | 官方转换插件包装它的本地文件模式 |
| [Panniantong/Agent-Reach](https://github.com/Panniantong/Agent-Reach) | `agent reach` | `private-api` | 可选 | MIT，依赖另计 | 第三方获取插件按链接补全单条；借输出信封和错误码 |
| [SoMarkAI/OXR](https://github.com/SoMarkAI/OXR) | `Somarkai/oxr` | `not-applicable` | — | OXR License | 转换插件做薄客户端，托管 API 默认关闭 |
| [mingjuner123-spec/wechat-inbox-sync](https://github.com/mingjuner123-spec/wechat-inbox-sync) | `https://weixin.qq.com/sph/Al3IF9rsyg` | `browser-session` | 可选 | MIT 加 Commons Clause | 只借「手机先存、电脑后理」的流程和失败分类 |
| [wechat-article/wechat-article-exporter](https://github.com/wechat-article/wechat-article-exporter) | `wechat article exporter` | `session-api`（单篇管线不登录） | 必须 | MIT | fork 单篇管线做公众号获取插件 |
| [chenjin-cmd/xhs-account-teardown](https://github.com/chenjin-cmd/xhs-account-teardown) | `chenjin-cmd/xhs-account-teardown` | `public-web` | 不用 | MIT | 移植单条笔记脚本，配合登录侧的列表 |
| [lucky-Luor/bilibili-douyin-notes](https://github.com/lucky-Luor/bilibili-douyin-notes) | `bilibili-douyin-notes` | `private-api`（抖音路径） | 可选 | MIT，抖音组件 GPLv3 | fork B站、小红书、YouTube 三个脚本 |
| [open-wxa/wxa-runner](https://github.com/open-wxa/wxa-runner) | `open-wxa/wxa-runner` | `not-applicable` | 可选 | MIT，基础库不能分发 | 不接入；小程序内容走视觉层 |
| [volcengine/OpenViking](https://github.com/volcengine/OpenViking) | `openviking` | `public-web`（URL 导入） | 可选 | AGPL-3.0 | 设计参考；以后可做导出目标 |
| [harry0703/MoneyPrinterTurbo](https://github.com/harry0703/MoneyPrinterTurbo) | `harry0703/MoneyPrinterTurbo` | `not-applicable` | 可选 | MIT | 不接入；借退出码和确认参数约定 |
| [datalab-to/indra](https://github.com/datalab-to/indra) | `indra ocr` | `not-applicable` | — | 代码 Apache-2.0，权重另有条款 | 官方转换插件的第二 OCR 引擎 |
| [Downvid/downvid](https://github.com/Downvid/downvid) | `yxxbc/downvid` | `private-api` | 可选 | MIT | 移植抖音、快手解析器到第三方获取插件 |
| [ahang008/ah-skills](https://github.com/ahang008/ah-skills) | `ah-skill` | `not-applicable` | 可选 | 源码可见的非商业许可 | 不接入；借来源说明的写法 |
| [ahang008/ah-douyin-clean-downloader](https://github.com/ahang008/ah-douyin-clean-downloader) | `ahang008/ah-douyin-clean-downloader` | `private-api` | 可选 | 源码可见的非商业许可 | 借「记录过程不留凭据」的证据字段；调用前先取得法律意见 |
| [Predidit/Kazumi](https://github.com/Predidit/Kazumi) | `Predidit/Kazumi` | `public-web` | 不用 | GPL-3.0，规则仓库 MIT | 借规则格式、deprecated 和更新流程 |
| [chubbyguan/chubbyskills](https://github.com/chubbyguan/chubbyskills) | `ChubbySkills`、`chubbyguan/chubbyskills` | `private-api` | 可选 | MIT | fork 到第三方仓库加 JSON 输出层；转写流程移植到转换插件 |
| [chengyi-ai/native-subtitle-quote-image](https://github.com/chengyi-ai/native-subtitle-quote-image) | `chengyi-ai/native-subtitle-quote-image` | `not-applicable` | — | MIT | 移植取帧和质检到关键帧插件 |
| [hypit-ai/hypit](https://github.com/hypit-ai/hypit) | `Hypit` | `private-api`（下载跟随 yt-dlp） | 不用 | 改过的 Apache-2.0 | 只参考下载器隔离和能力版本 |
| [mcncarl/yichen-skills](https://github.com/mcncarl/yichen-skills) | `mcncarl/yichen什么什么` | `client-hook`（微信本地数据） | 可选 | 个人非商业许可 | 按收藏页采集规则自己写列表插件 |
| ↳ [mcncarl/agent-memory-vault](https://github.com/mcncarl/agent-memory-vault) | （同一作者） | `not-applicable` | — | MIT | 借写入意图和回执流程 |
| ↳ [mcncarl/ailu](https://github.com/mcncarl/ailu) | （同一作者） | `not-applicable` | 必须 | AGPL-3.0 | 借飞书导出的最小权限做法 |
| ↳ [mcncarl/wechat-relay](https://github.com/mcncarl/wechat-relay) | （同一作者） | `not-applicable` | 必须 | AGPL-3.0 | 借官方接口中转的安全边界 |
| [hanliang97/MatrixMedia](https://github.com/hanliang97/MatrixMedia) | `hanliang97/MatrixMedia` | `session-api` | 必须 | GPL-2.0-only | 借登录态页面里取列表的适配器结构 |
| [ZJU-REAL/Easel](https://github.com/ZJU-REAL/Easel) | `easel` | `private-api`（B站 登录模拟 TV 端） | 可选 | Apache-2.0，内置 gzh-design 为 AGPL-3.0 | 不接入；借登录状态文件、读回分档和资料目录锁 |
| [ycccccccy/echotrace](https://github.com/ycccccccy/echotrace) | `echo` | `client-hook`（微信本地数据） | 必须 | MIT，默认分支只剩文档 | 不接入；对应关系是推断，只记录层级和下架情况 |

**kali-publish（pabatiba89-sys/kali-publish）。** GitHub 按名字搜不到，按描述搜「kali 发布」找到这个仓库；npm、PyPI 上没有同名包。它是 social-auto-upload 的派生，喀理 AI 商业产品的开源发布模块，用本机 Chrome 保存的登录态把视频发到小红书、视频号、抖音等 10 个平台，不取内容。对歇存有用的只有登录这一块的踩坑记录：Google 系平台不接受在 Playwright 控制的 Chrome 里登录，所以首次登录用普通 Chrome 加独立资料目录，再经本机 CDP 导出 `storage-state.json`；macOS 上自动化启动的 Chrome 读不到系统钥匙串加密的 Cookie；判断登录是否完成只查必需 Cookie 的名字，不读也不解密值。这几条可以写进获取插件的登录指南。不包装也不 fork：视频号流程会在每条作品后追加推广话题、按账号名挂课程商品，本机 5409 端口的服务没有认证，按链接下载时忽略证书错误。MIT；单人维护，没有 issue，能否持续不确定。

**social-auto-upload（dreammis/social-auto-upload）。** 命令行加 Agent Skill 的多平台发布工具，用浏览器自动化载入保存的登录态，把视频、图文发到 17 个平台入口，不取内容。可借三处：每个平台 `login` 生成本地登录态文件，`check` 只回答有效或无效，`verify` 按上次的结果文件核对原作品，不重复操作——获取插件的 `health` 和登录检查可以照这个形状做；Skill 按平台各一份说明，参考资料分契约、运行要求、排障三份，并写明拿到成功证据才报告完成；短信验证码交给人工输入。反例是 B站 一路运行时拉 biliup 的最新 release，不锁版本也不校验哈希，正是 [ADR-003 §6](../decisions/ADR-003-plugins.md#6-分发与校验) 要求锁定 sha256 的情形。风险：即使走浏览器层也有限流和封禁记录，[dreammis/social-auto-upload #289](https://github.com/dreammis/social-auto-upload/issues/289) 多人被判定用 AI 或自动化工具发布，有人被封 15 天。MIT。

**MediaCrawler（NanmiCoder/MediaCrawler）。** 多平台自媒体内容采集，按关键词、帖子 ID 或创作者主页抓小红书、抖音、快手、B站、微博、贴吧、知乎的帖子和评论；默认在本地生成签名后直接请求，要登录，属 `private-api`。它面向关键词搜索和评论分析，没有收藏或点赞列表，和歇存回收个人收藏的目标不重合。可参考的是输出形状：每个平台一个存储实现，默认 JSONL，小红书一条记录含 `note_id`、`title`、`desc`、`video_url`、`tag_list`、`image_list`、`note_url` 等；导出时把作者和评论者 ID 做哈希、昵称打码。许可证是自定义的「非商业学习使用许可证 1.1」，只授予使用、复制、修改、合并，没有分发权，限学习研究用途：不 fork、不分发、不包装，也不引导用户自装后让插件调用。风险记录集中在小红书和抖音：小红书 2026 年有永久封禁（[NanmiCoder/MediaCrawler #865](https://github.com/NanmiCoder/MediaCrawler/issues/865)），只看帖子和下载视频也被提示检测到 AI 操作（[NanmiCoder/MediaCrawler #915](https://github.com/NanmiCoder/MediaCrawler/issues/915)）；抖音接口返回 account blocked 从 2024 年持续到 2026 年。README 的赞助方多为代理 IP 服务商，作者主推付费 Pro 版。

**short-video-factory（YILS-LIN/short-video-factory）。** Electron 应用，模型写口播文案、Edge TTS 配音，把用户自备的素材混剪成带字幕的短视频，不登录任何平台。和歇存方向相反，只借两处：它从作者服务拉公告和更新信息时，用 Ed25519 对规范化后的 JSON 验签，核对应用 ID、发布通道和版本，验签失败就不用——歇存的屏蔽清单和插件列表缓存可以照这样做；成片旁边导出同名 SRT。AGPL-3.0，和 MIT 的官方插件不兼容。生产版本默认把匿名使用事件发到作者的统计服务，没找到界面开关；Edge TTS 依赖微软未公开的接口，鉴权变化时出过 403。

**ai-videio-transcriber（wendy7756/AI-Video-Transcriber）。** 用户写的拼法有误，对应的是 AI-Video-Transcriber。自部署的音视频转写工具：链接交给 yt-dlp 取字幕或音频，没有字幕时用 faster-whisper 在本地转写，可选模型校对、翻译和摘要；不登录，链接模式继承 yt-dlp 的层级。可借两处：先取平台字幕、没有再转写的分流；转写后先判断有没有语音，没有就跳过模型、返回 `no_speech=true`，避免模型对着空文稿编造。无头入口在 `--json` 时 stdout 只输出结果、进度走 stderr，退出码区分输入不合法和下载失败，形态接近 [02 §15.3](02-adapter-ecosystems.md#153-调用错误与产物) 建议的命令行。接入：官方转换插件只包装本地文件模式（`--no-llm --json`），只喂歇存已经存下的音视频；链接模式归第三方获取插件，直接包装 yt-dlp 更好；不包装它的网页服务（默认监听 0.0.0.0、CORS 不限来源）。LICENSE 是 Apache-2.0，Codex 插件清单误写成 MIT。问题集中在 IP 层风控和 yt-dlp 跟不上改版，Docker 部署时 yt-dlp 版本固定在构建时，更容易失效。

**agent reach（Panniantong/Agent-Reach）。** 面向 AI Agent 的统一命令行：把 B站、YouTube、小红书、X、Reddit、小宇宙等的搜索与按链接读取收成 `agent-reach <渠道> <动作>`，每个动作按顺序尝试多个上游工具，输出统一 JSON。条目记 `private-api`：B站 后端依赖复现签名的库，专用浏览器 CloakBrowser 改了指纹；小红书、Reddit 等在专用浏览器里读页面。可借四处：统一的输出信封（条目只有标题、链接、作者、发布时间、正文加原始数据）；错误码里把 `PLATFORM_BLOCKED`、`RATE_LIMITED`、`NEED_LOGIN` 定为终止码，不再换后端重试，遇到平台拦截就停止自动化、等用户在官方页面处理；每个动作先用公开网页工具，失败才动用登录态；上游工具按已测版本、最低版本、排除版本固定，发版前做真实验收。接入：做成第三方获取插件，包装 `agent-reach read <url>`，只用来按链接补全单条，收藏列表另找来源；歇存不接管它的专用浏览器。本体 MIT，但 B站 链路带 GPL 库、CloakBrowser 二进制是专有许可，只能用户自装、以独立进程调用。风险：[Panniantong/Agent-Reach #498](https://github.com/Panniantong/Agent-Reach/issues/498) 用户约两天、不到 15 次查询就收到小红书封号警告；把主账号 Cookie 复制进第二个浏览器，可能被判为同一账号出现在另一套指纹上；PyPI 上的同名包已被他人注册（#547），第三方扫描器对安装脚本告警且维护者未回应（#802）；安装方式是让 Agent 自动装 Node、Chrome 或 CloakBrowser。

**Somarkai/oxr（SoMarkAI/OXR）。** SoMark 开源的 1.3B 文档解析模型和配套服务，把 PDF 与文档图片转成 Markdown 和带版面坐标的 JSON，能识别正文、标题、表格、公式、代码块等 13 类元素，不访问平台。版面分析是独立可替换的服务，识别和阅读顺序由主模型负责，这种拆分可作为歇存 OCR 契约的参考。接入：做转换插件的薄客户端（[ADR-003 §2](../decisions/ADR-003-plugins.md#2-进程与协议) 允许后端是用户自己跑的服务），把块级结果写成 `derived/ocr/<image>.json`，坐标换算成 `[x,y,w,h]`。适合 PDF 和表格、公式多的截图，不适合当默认 OCR：本地部署要 Linux 加 NVIDIA GPU，起步 24GB 显存；免费托管 API 会把文件传到 SoMark，插件要标「离开本机」并默认关闭。许可证是 OXR License：以 Apache-2.0 为基础、附加条款优先，营收或融资超过 200 万美元的商业使用要另购许可，任何规模都不得提供与 SoMark 竞争的托管文档解析或 OCR 接口；不把源码复制进 MIT 的官方插件。2026-10-05 才建仓，接口和条款可能变，插件要锁定版本。

**视频号链接（mingjuner123-spec/wechat-inbox-sync）。** 用户给的是一条视频号分享链接。对应关系是推断出来的：视频是「张张AI视界」2026-10-24 发的「蝴蝶号一键转存obsidian」；这个插件的 manifest 作者是 Zhang Zhang，最早的 LICENSE 署名「张张」，同一 GitHub 账号的另一个仓库写明作者的公众号叫「张张AI视界」；网页搜「蝴蝶号」没有别的结果。所以「蝴蝶号」应该就是视频号（图标像蝴蝶），这个插件就是视频里演示的工具。它是 Obsidian 插件：手机经作者的小程序（另有 iOS 快捷指令）把公众号、视频号、抖音、小红书、B站、小宇宙等链接和文件存进作者云端的收件箱，电脑端插件拉取、解析、转写后写成 Markdown。视频号链接默认交给作者的云端解析，抖音、小红书用插件自己的登录会话，条目记 `browser-session`。可借的是「手机先存、电脑后理」的顺序，以及运营节点的失败分类（需要登录、风控、内容不可用、不支持、网络、上传、转写），可以对照歇存插件协议的错误枚举；但歇存的收件箱应由用户自己掌控，不经开发者的云端。只借设计，不 fork、不打包：1.3.117（2026-08-23）起是 MIT 加 Commons Clause，手机端小程序和云端后端不开源；如确需复用代码，只能基于 1.3.116 及更早的 MIT 版本。作者几乎每天发版，用户反映视频号和抖音链接多数同步失败（[mingjuner123-spec/wechat-inbox-sync #135](https://github.com/mingjuner123-spec/wechat-inbox-sync/issues/135)）。

**wechat article exporter（wechat-article/wechat-article-exporter）。** 可自部署的公众号文章批量导出网站：用自己的公众号后台扫码登录，列出目标公众号的文章，再导出 HTML、JSON、Markdown 等格式，属 `session-api`。依赖的后台接口 2026-07-30 被关，作者停止维护（[wechat-article/wechat-article-exporter #200](https://github.com/wechat-article/wechat-article-exporter/issues/200)）。还能用的是单篇管线：按公开文章 URL 取页面，整理正文和懒加载图片，区分正常、已删除、异常，在不联网的 QuickJS 沙箱里执行文章页自带的数据脚本，取回标题、作者、公众号、发布时间。这一段不需要登录，正对应 [SPEC §8](../SPEC.md#8-适配器协议-v0) 示例里的公众号适配器。接入：fork 到第三方仓库，只留单篇管线，删掉后台扫码、批量列表、会员限流和凭证通道，入口改成 `canonicalize`、`fetch` 命令行；用户以前用它导出的文件可以作为导入格式。MIT；配套的 wxdown-service 没有许可证，不复用。凭证通道要装根证书对微信客户端做中间人，属 `client-hook`，不提供。

**chenjin-cmd/xhs-account-teardown。** Agent Skill：不登录抓小红书账号主页与第一页笔记封面，或凭带 `xsec_token` 的笔记链接抓单条笔记的正文和图片，再由 Agent 识图写账号拆解报告；属 `public-web`，不用账号。对歇存有用的是它实测记下的匿名边界：token 与笔记绑定，错配时返回空数据而不报错；主页给的 32 位 id 不能拼详情路由；token 要从登录态浏览器整链复制。接入：把两个只依赖 python3 标准库和 curl 的脚本移植成第三方获取插件的单条获取，`canonicalize` 从链接取笔记 ID 和 token，`fetch` 产出正文和图片；收藏列表交给 `browser-session` 或视觉层，组合起来账号只出现在列清单这一步（[§1.3](#13-混合路径视觉层拿链接公开页面取内容)）。MIT。只有两次提交，没有 issue；解析依赖页面内嵌状态，改版就失效，插件要带 `health` 检查，失败时提示用户而不是自动重试。README 也提醒小红书风控按 IP 生效，失败后反复重试只会延长封锁。

**bilibili-douyin-notes（lucky-Luor/bilibili-douyin-notes）。** Agent Skill：按平台抓 B站、抖音、YouTube、小红书视频的元数据和字幕或媒体，没有字幕时本地转写，按时间戳抽帧，再由宿主 Agent 写成课堂笔记并做质量检查。抖音路径要复现签名，条目记 `private-api`；B站 可选登录。四个平台的脚本共用一份 `metadata.json` 约定，stdout 最后一行是 JSON 摘要，包装成本低。B站 脚本记下一个数据质量问题：旧字幕接口会随机返回别的视频的字幕或空地址，所以改用更稳定的接口并多轮重试；歇存保存平台字幕时也要校验字幕和视频是否对应。小红书脚本用手机 UA 读分享页、要求链接带 `xsec_token`，是视频笔记的 `public-web` 路径。接入：在第三方仓库 fork B站、小红书、YouTube 三个脚本，改成歇存插件协议的输出，B站 的 SESSDATA 由内核以临时文件传入，不写进明文配置；抖音签名组件是 GPLv3，仓库不包含、运行时下载并校验 SHA-256，fork 时去掉，或做成用户自装的可选项。主体 MIT。2026-10-01 才建，签名组件固定在 2025-05 的上游版本，平台升级后会返回 403。

**open-wxa/wxa-runner。** 在普通浏览器里用微信官方基础库运行开发者自己的小程序编译产物，用于本地调试和 Playwright E2E 测试；输入是自己的项目目录，不能下载或解包别人的小程序。它和获取小程序内容基本无关：要用它运行别人的小程序，得先从微信客户端缓存解密代码包，那属于 `client-hook`，还涉及小程序代码版权，歇存不支持。小程序里的内容应走视觉层，截图、录屏后交给 OCR；小程序里的 web-view 文章和分享出的网页链接按 `public-web` 处理。请求拦截、页面直达、把运行时打包成静态站点的做法，对歇存以后测试自己的小程序收藏入口有参考价值。代码 MIT，但内置的基础库是腾讯的代码、没有许可证，不能随歇存分发。

**openviking（volcengine/OpenViking）。** 火山引擎开源的 Agent 上下文数据库，把资料、记忆和技能组织成 `viking://` 虚拟文件系统；语义处理过的目录带一句话摘要和要点摘要两个文件，Agent 先看摘要再决定是否读原文，检索可以限定在某个子目录。URL 导入是不登录的静态请求，遵守 robots.txt，公众号文章会去掉隐藏样式、换回懒加载图片。这和歇存「资料库目录是真相源、给 Agent 先摘要后全文」的方向一致，可借摘要文件命名、目录级检索范围和 MCP 工具的划分（find、search、read、ls、tree、grep）。以后若用户在用 OpenViking，可以做一个导出插件，经它的 HTTP 接口或 Apache-2.0 的 `ov` 命令行把条目推过去，不引用 AGPL 的 SDK；它需要嵌入模型和视觉模型，可配本地 Ollama，否则摘要要发给云端。主项目 AGPL-3.0，README 另外限制整体改写和翻译，不移植代码。更新快，导出插件要锁定版本。

**harry0703/MoneyPrinterTurbo。** 根据主题自动写脚本、配音、匹配素材、加字幕和配乐，合成短视频，可选经 Upload-Post 发到 TikTok、Instagram、YouTube；不碰用户的平台内容。可借的是它给 Agent 的调用约定：退出码 0 时打印结果块并写一份结果文件，终端截断输出时读文件；退出码 10 只列出缺少的凭据，让 Agent 一次问齐；退出码 1 打印错误和日志路径，只自动修复并重试一次；按次计费的素材服务必须显式带确认参数才执行，说明文档写明 Agent 不得自行加上。歇存让用户按风险层级选获取方式时可以照搬：高风险层要求一个只有用户确认后才带上的参数，退出码区分「需要用户输入」和「失败」。MIT。官方 PyPI 包名是 `mpt`，名为 MoneyPrinterTurbo 的包是第三方上传的。

**indra ocr（datalab-to/indra）。** Datalab 开源的 CPU OCR 工具和小模型，只依赖 ONNX Runtime、OpenCV 和 pypdfium2，不用 PyTorch 和 GPU，默认模型约 76MB，官方在 Apple silicon 上测过，4 核约 210 毫秒一页。逐页 JSONL 里的行级结果换算后可以直接写成 [SPEC §5](../SPEC.md#5-derived-文件约定) 的 OCR 格式；数字版 PDF 先读文字层，只对缺字区域做 OCR，适合歇存保存的 PDF 附件。接入：官方转换插件包装 `indra-ocr <文件> --format json`，常驻模式用它的 Python 接口只加载一次模型；作为 PDF、多语种、旋转页面的第二引擎。中文不是强项，04 选 PP-OCRv6 做中文默认引擎的结论不变（[04 §2](04-tech-options.md#2-ocr-sidecar)）；中文界面截图仍用 PP-OCR。代码 Apache-2.0；权重是修改版 OpenRAIL-M，上一年收入或累计融资超过 2000 万美元只能个人或研究用途，还有范围较宽的竞争条款，要等法律意见再决定能否进官方插件，权重由用户首次运行时下载并展示许可。2026-10-13 建仓，社区修复 PR 多被关闭未合并。

**yxxbc/downvid（Downvid/downvid）。** 用户写的地址对应现在的 Downvid/downvid。跨平台桌面视频下载器，给 yt-dlp 和 FFmpeg 加图形界面，抖音、快手另有直连解析，B站、YouTube 交给 yt-dlp，条目记 `private-api`。可借四处：抖音解析器跟随短链取作品 ID，带 cookie 请求网页详情接口，按码率整理出下载地址和标题、作者、封面、时长；快手解析器对公开作品发不带登录的请求；cookie 策略是手动 cookies.txt 优先，自动模式依次试本机浏览器、成功后缓存 30 分钟、失败的浏览器冷却 10 分钟；错误分类把「短链已失效」「可能需要登录」「不支持的网站」分开，正对应 [SPEC §8](../SPEC.md#8-适配器协议-v0) 运行器换下一个获取方式时需要的错误信封。接入：它只有桌面界面，Go 包在 `internal/` 下不能作为库引用，可在第三方仓库复制或重写这两个解析器（保留 MIT 声明），其余站点直接包装 yt-dlp。MIT；名称和 Logo 按 TRADEMARKS.md 不得用来暗示官方背书，致谢时连同上游 Videdown 一起列。2026-08 分叉，两个多月发了 21 个版本；默认自动模式读用户日常浏览器的登录态交给 yt-dlp，用户不一定意识到，歇存若借鉴应改为专用浏览器资料目录，或让用户显式导入 cookie。

**ah-skill（ahang008/ah-skills）。** 阿杭的 Codex Skill 目录仓库：主要是指向各独立仓库的索引，另有两个短视频系列制作入口、一个经 Gemini 网页润色口播稿的桥接包和一个本地口播剪辑包，本身不从内容平台取数据。对歇存有用的只有发布纪律：PROVENANCE.md 写清每次公开的范围、不包含的内容和依赖来源，THIRD_PARTY_NOTICES.md 单列外部依赖与许可边界，歇存的插件仓库模板可以要求同样的来源说明。许可证是「AH Source Available Non-Commercial License 1.0」，商业用途（包括放进插件、Skill、市场列表）要书面授权，不 fork、不分发。两个系列入口依赖的剪辑 Skill 已因律师函删除，入口实际已无可用依赖。

**ahang008/ah-douyin-clean-downloader。** 面向 Codex 的抖音 Skill：单条按分享链接下载平台播放源原视频，可在本机转写成 Markdown 和 SRT；整号归档在专用 Edge 资料目录里登录，由页面自己发分页请求再截获。单条路径用 App 侧接口，条目记 `private-api`。可借三处：单条命令行只依赖标准库和 curl，下载到临时文件再原子落盘，回报路径、大小、时长；整号归档的证据里把「是否导出 cookie、是否保存请求头、是否保存签名 URL」固定记为否，是「记录获取过程但不留凭据」的写法，歇存 `browser-session` 层列收藏时可以照搬；机器结果与人工确认分开标记，指标缺失写「未获取」而不是 0。许可证是源码可见的非商业许可，商业使用要书面授权：不能并入 AGPL 主仓库或官方组织；第三方插件调用用户自装的副本是否算许可证所说的「包含」，要先取得法律意见。用户已经生成的资料库可以作为导入来源。

**Predidit/Kazumi。** Flutter 番剧采集与播放应用，视频来源由用户导入的 JSON 规则决定（XPath 或 JSON 接口映射），规则在独立的 KazumiRules 仓库托管；不登录。值得借的是规则生态：规则里的接口级别高于客户端时拒装并提示升级，规则自身另有版本号；目录由 CI 脚本生成，客户端可以换镜像；启动时检查更新、由用户确认、永不降级；失效规则标 `deprecated`，保留文件、移出目录（86 个规则文件里只有 18 个在目录里）；规则测试页给出原始响应、命中片段和跳过节点的诊断。歇存的插件清单可以照搬「协议版本与插件版本分开、拒装时说明原因、deprecated 留档、更新由用户确认、测试输出诊断」，国内从 GitHub 装插件也要考虑镜像。验证页的处理是用户在 WebView 里自己完成，cookie 只在本次运行有效，维护者认为自动识别验证码不迫切。应用 GPL-3.0，不拷代码；规则仓库 MIT。

**ChubbySkills（chubbyguan/chubbyskills）。** 用户写的两个名字是同一个项目。14 个 Agent Skill 加统一命令行，把抖音、B站、小红书、公众号、X、播客等链接或本地文档存成带来源字段的 Markdown，再做本地检索、带出处的资料包和订阅日报，附知识库 MCP。多数平台匿名读页面，B站、YouTube 跟随 yt-dlp，条目记 `private-api`。可直接借四处：平台声明文件 `platforms/*.yaml` 写明对应的 Skill 与脚本、状态、依赖、失败模式和兜底命令，几乎就是歇存清单的现成样例；失败分型（缺依赖、cookie 问题、风控限流、来源失效、网络）可一一映射到错误码；输出 frontmatter 在标题、平台、来源之外加 `schema_version`、`run_id`、`source_hash`、`captured_at` 等并有校验脚本；凭据体检只看配置和上次结果，不发探活请求，避免为检查而触发风控。接入：fork 到第三方仓库，给各平台脚本加一层输出歇存 JSON 的薄壳；「字幕优先、没有再用 SenseVoice 本地转写」的流程适合移植成官方转换插件。MIT。文档写明不做反爬对抗，失败就走手动兜底；单人维护，一个月发了 6 个版本。

**chengyi-ai/native-subtitle-quote-image。** Agent Skill 加 Python 脚本：从本地视频（或按文档让 Agent 用 yt-dlp 取得的 YouTube 视频）按时间点精确取帧，裁出画面里已烧录的字幕条，或把审核过的台词绘到真实帧上，拼成社交长图并逐张质检；仓库自己不抓取。可移植到歇存关键帧转换插件的有三段：按给定时间点、或在整段里均匀抽最多 24 帧生成带时间标注的总览图；取帧前检查分辨率并解码前几秒，坏流提前失败；检测整段几乎相同的帧（静态封面视频）和相邻重复的字幕条。它坚持「画面里的字只取像素，后期绘制的字要标明是脚本字幕」，和歇存对派生内容标明来源的要求一致。MIT；issue 由仓库里的 Agent 工作流自动分诊和实现，子命令参数随版本变化，移植时固定到某个 tag。

**Hypit（hypit-ai/hypit）。** 给编码 Agent 用的视频制作语言与命令行：读参考视频，转写、截帧分析，生成可编辑、可重跑的工作流，再用生成模型或代码渲染新视频；下载不登录，层级跟随 yt-dlp。和歇存方向相反，可借三处：yt-dlp 不用机器上已有的版本，随包锁定依赖、显式准备，运行时不读用户配置、不加载插件、不自动更新，链接前加 `--` 结束参数解析（[hypit-ai/hypit #452](https://github.com/hypit-ai/hypit/issues/452)）；能力的逻辑版本和发布版本分开记，对应歇存清单里 protocol 与 version 分开；转写和按时间段出帧网格对应歇存的转写、关键帧插件。许可证是改过的 Apache-2.0：未经授权不得做多租户托管，不得为商业获利再分发，fork 须沿用同一许可，与 AGPL 内核不兼容，只参考。托管服务近期有额度误报和退款无回复的 issue。

**mcncarl/yichen什么什么（mcncarl/yichen-skills）。** 用户写的应该就是逸尘的 Skill 合集：收藏夹链接导出、已知链接归档、跨平台搜索路由、ASR、X 文章草稿上传、X 帖子切片，以及微信和企业微信本地数据解析；因为最后一项，条目记 `client-hook`。对歇存最有用的是收藏页采集规则：先确认页面状态再采集，小红书要求地址含 `tab=fav&subTab=note`、一级「收藏」和二级「笔记」都处于选中态，抖音打开收藏页并确认「收藏」「视频」选中，滚动的是页面内的列表容器而不是整个窗口；ID 只按严格路径接受，避免把作者主页链接算进去；先回到顶部，再连续 4 次到底且条数与滚动高度都不增长才算完成；页面标签上的数字和实际累计条数分开报告。这些可以直接写进歇存小红书、抖音收藏列表插件的规格，走 `browser-session` 或视觉层都适用。另一处是分层授权：读取、下载、登录态、写入飞书各自单独授权，收藏导出只产出链接文件。许可证是个人非商业学习许可，禁止作为 Skill 合集、模板包、市场条目再发布，只能按这些规则自己实现。2026-10-23 删除了剪映相关 Skill，维护者在 [mcncarl/yichen-skills #25](https://github.com/mcncarl/yichen-skills/issues/25) 说收到律师函；微信本地解析随客户端升级频繁失效（#16、#17），不宜作为依赖。

**mcncarl/agent-memory-vault（同一作者）。** 给 Claude Code、Codex 和 Ailu 共用的本地长期记忆库模板：Markdown 是唯一事实源，SQLite 做字段与全文索引，可选本地语义检索，所有派生库都能从 Markdown 重建，和歇存「文件为真相源、索引可丢可重建」一致。写入上，Agent 先声明写入意图，相当于对目标路径的写租约，用单调递增的令牌防止换主后旧进程误写；完成后留不可改的回执；任务结束跑检查、重建索引、审计，只提交本任务的改动。歇存在 [03](03-agent-protocols.md) 讨论的 Agent 写入变更集可以借这种「意图、回执、审计」的流程。MIT，不访问平台。

**mcncarl/ailu（同一作者）。** Obsidian 桌面插件：侧栏调用本机 Claude Code 或 Codex，把当前笔记生成公众号排版预览、经自建中转建公众号草稿、同步为飞书文档、创建 X Article 草稿。和歇存的关系在导出侧：飞书路线复用用户独立安装的官方 lark-cli，只申请建文档、读取、覆盖更新、传图和目录只读权限，首次新建放在个人文档库根目录，已关联的文档原位更新、链接不变，可作歇存飞书导出插件的参考；公众号路线创建后回读核验，回读失败时保留草稿 ID 并提示人工核对。X 草稿路线复用用户主账号的登录态驱动浏览器。AGPL-3.0，只参考流程。

**mcncarl/wechat-relay（同一作者）。** 自托管的公众号草稿接口中转服务，只转发上传素材、上传正文图片、新建草稿、读取草稿四个官方接口，让公众号的 IP 白名单只需登记服务器的固定出口 IP。可借它的安全写法：只开放四个路径且全部要认证，对路径、方法、参数、请求体与响应大小、速率、连接数都设上限；新建草稿必须带幂等键，数据库只存键的摘要、路由和请求体哈希，不存原文；只在 token 失效的三种错误码时换新 token 重试一次；日志按白名单字段输出，不含密钥和正文；进程只绑定回环地址。歇存以后做公众号草稿导出，或给需要固定出口 IP 的官方接口做中转时，可以照这套边界设计。AGPL-3.0，只作为用户自部署的独立服务经 HTTP 调用。

**hanliang97/MatrixMedia。** Electron 桌面应用，把本地或网络视频批量发到抖音、快手、小红书、视频号、B站、头条等的多个账号，附命令行、本机 HTTP 接口和 MCP；还能采集自己账号的作品与粉丝数据，部分请求带 Cookie 直接调接口，条目记 `session-api`。对歇存有用的是数据统计模块：为每个账号开一个隐藏窗口复用会话，先看是否被重定向到登录页，再在页面上下文执行按平台写的适配器脚本，签名一律交给页面自己完成，统一输出概览、粉丝趋势和作品列表——把「作品列表」换成「收藏列表」，就是 `browser-session` 层获取插件该有的形状。命令行的退出码区分成功、异常、参数错误、业务失败，账号和历史命令支持 `--json`。GPL-2.0-only，和 AGPL 内核、MIT 插件都不兼容，只借结构。README 承认小红书对内置窗口的自动化检测越来越严，所以加了真实 Chrome 发布模式，并对小红书强制显示窗口、两次点击间隔随机 5–10 秒；启动时默认向公开 Gist 写匿名打开事件，可以关闭。

**easel（ZJU-REAL/Easel）。** 用户后来补的名字。浙江大学 REAL Lab 发起的社媒运营 Agent 工作台，README 同列北京大学与 OpenDCAI Lab；默认用 OpenClaw 做执行底座，也可换 OpenCode、Codex 或通用 ACP；`skills/builtin` 下 115 个 Agent Skill 覆盖热点发现、选题、图文音视频创作、七个国内平台的扫码登录与发布，以及发布后的读回和账号数据。它不读收藏夹，取数按功能分层：热点雷达不登录，读第三方聚合服务和知乎、头条的公开热榜，属 `public-web`；账号数据和评论在 Playwright 保存的登录态里读创作中心页面和页面自己的请求，属 `browser-session`；抖音读回在页面里带 Cookie 请求作品列表，B站 用 cookies.json 直接调稿件和数据接口，属 `session-api`。条目记 `private-api`，因为默认路径里 B站 唯一的登录入口模拟 TV 端扫码；内置的小红书分析技能是 lucasygu/redbook（见 [§6](#6-小红书)）的源码副本，但 redbook 要用户另装。只按取内容那一步算，最高是 `session-api`，口径列在 [§18](#18-待核实清单)。取数部分歇存用不上，可借四处设计：登录状态文件由执行进程原子写 JSON、宿主轮询，分出等待扫码、需要短信验证、校验中等状态，获取插件扫码登录时可以照此把进度交给界面；读回结果分 verified、unverified、login_required、readback_error 四档，没有平台侧证据不算成功，可以映射进歇存的错误信封，把登录失效和通道故障分开；同一资料目录加跨进程锁，国内平台一律直连（[ZJU-REAL/Easel #52](https://github.com/ZJU-REAL/Easel/issues/52)、[ZJU-REAL/Easel #42](https://github.com/ZJU-REAL/Easel/issues/42)）；未合并的 [ZJU-REAL/Easel #171](https://github.com/ZJU-REAL/Easel/issues/171) 定了采集器协议（`version`、`fetched_at`、`items`，经 stdout、HTTP JSON 或 RSS 接入，子进程限时长、内存和进程数），和 [ADR-003 §2](../decisions/ADR-003-plugins.md#2-进程与协议) 的独立进程加 JSON 是同一路子。主体 Apache-2.0，可以并入 AGPL 内核；但内置的 gzh-design 是 AGPL-3.0，抖音发布脚本自述移植了 AGPL-3.0 项目的流程与选择器，相容性没有核实。只借设计，要用代码时保留 Apache-2.0 声明，避开这两处。风险集中在小红书：[ZJU-REAL/Easel #98](https://github.com/ZJU-REAL/Easel/issues/98) 用户登录一直被提示 IP 有风险，维护者修了误判，同时说明平台真返回 300012 时代码不能保证解除；#52 记录无头模式扫码时被要求验证码；#42 指出开着 VPN 时国内平台经境外出口登录、发布会触发 300012。README 提醒小红书自动发布有验证、限流和账号风控风险，建议用户确认后手动发布。issue 里没有封号报告。2026-08-28 建仓，维护活跃。

**echo（ycccccccy/echotrace）。** 用户后来补的名字，只写了 `echo`。对应到 EchoTrace 是推断，把握中等。依据有三：按名字核对到的 echo 项目里，它是唯一和国内平台数据有关、星数过百的（3856 星、689 个 fork），其余候选在 0–26 星之间；它正落在本报告微信本地数据、`client-hook` 和下架这条线上；按名字找收藏、稍后读或平台内容转笔记的 Echo，没有找到星数可观的项目。名气最大的 labstack/echo 是 Go 的 Web 框架，和内容获取无关，没有收录。EchoTrace 是微信 4.x PC 端聊天记录的本地导出与分析工具：取得本机数据库密钥后解密本地库，离线浏览，导出 JSON、HTML、Excel，并生成年度报告，属 `client-hook`。作者 2026-01-18 发布 5.2.0 宣布停止维护，默认分支换成只有文档的新提交，源码只留在 fork 和旧安装包里，哪一份和最后发布的一致无法核对。EchoTrace 本身不在腾讯 2026 年的 DMCA 通知里，但作者的另一个仓库 wx_key（取密钥工具）在 [2026-06-29 的通知](https://github.com/github/dmca/blob/master/2026/06/2026-06-29-wechat.md)里，README 推荐的继任者 [hicccc77/WeFlow](https://github.com/hicccc77/WeFlow) 在 [2026-07-07 的通知](https://github.com/github/dmca/blob/master/2026/07/2026-07-07-wechat.md)里（[§16.3](#163-下架停更与法律风险)）。歇存不接入：它处理的是聊天记录，不是收藏；有人把文章链接发到「文件传输助手」当稍后读，用户导出这个会话的文件可以作为导入来源，但产生文件的工具不由歇存推荐或预置。MIT。导出内容含聊天对象和群成员的消息，生成、分享年度报告涉及他人隐私。

如果用户说的是别的 echo，下面这些名字相近的候选在各类别表里各有一行；在 Obsidian、录音转写或 Agent 记忆的语境里，更可能是前三个：

- [anbang278/echo-notes](https://github.com/anbang278/echo-notes)：Obsidian 录音转写插件，附一个只监听本机的 MLX 转写服务，见 [§12](#12-转换转写字幕与-ocr)。
- [questionjie-max/echo-memory](https://github.com/questionjie-max/echo-memory)：Mac 上的本地个人记忆应用，经只读 MCP 交给 Agent，见 [§13](#13-笔记知识库与收藏归档)。
- [HuanHuanHuanFFF/Echo](https://github.com/HuanHuanHuanFFF/Echo)：本地 Markdown 笔记检索 MCP，见 [§13](#13-笔记知识库与收藏归档)。
- [Qc-riviere/EchoMind-releases](https://github.com/Qc-riviere/EchoMind-releases)：闭源的灵感速记应用，给绑定的微信 bot 发消息即可记录，见 [§13](#13-笔记知识库与收藏归档)。
- [Bin-Zhang-hhht/EchoForge](https://github.com/Bin-Zhang-hhht/EchoForge)：读公开 RSS 的技术播客雷达，见 [§7](#7-b站youtube-与播客)。
- [eudaemoniaYH/echo-podcast-memory](https://github.com/eudaemoniaYH/echo-podcast-memory)：小宇宙、机核的收听历史分析，带登录令牌调接口，见 [§7](#7-b站youtube-与播客)。
- [Rookage/echo-editorial](https://github.com/Rookage/echo-editorial)：短视频创作工作台，把抖音分享链接转成逐字稿再改写，见 [§15](#15-相邻创作发布与分析)。
- [Kerinlin/EchoDistill](https://github.com/Kerinlin/EchoDistill)：在浏览器里读评论区、交给模型归纳的油猴脚本，见 [§15](#15-相邻创作发布与分析)。

另有一条和 easel 有关：[ZJU-REAL/Awesome-GUI-Agents](https://github.com/ZJU-REAL/Awesome-GUI-Agents) 是同一组织维护的 GUI Agent 论文清单，没有代码，见 [§10](#10-视觉与-ui-自动化)。

<a id="4-多平台下载与抓取"></a>
## 4. 多平台下载与抓取

| 项目 | ★ / 推送 / release / 许可证 | 平台 | 做什么 | 层级 | 登录 | 接口 | 歇存怎么用 |
|---|---|---|---|---|---|---|---|
| [timelinize/timelinize](https://github.com/timelinize/timelinize)（致谢） | 3692 / 2026-05-22 / v0.0.28（2025-10-17） / AGPL-3.0 | 多平台：X、Instagram、Telegram、其他 | 本地优先的个人时间线：把各平台官方导出包和手机备份导入同一个 SQLite 库 | official-export | 不用 | 网页应用、命令行、HTTP 接口 | 只作参考。AGPL-3.0：借导出格式知识，不拷代码。 |
| [karlicoss/HPI](https://github.com/karlicoss/HPI)（致谢） | 1645 / 2026-10-30 / v0.8.20260912（2026-09-12） / MIT | 多平台：Reddit、X、Instagram、Telegram | 个人数据访问层：用统一的 Python 接口读用户从各平台导出的数据 | official-export | 不用 | 库、命令行 | 参考：借做法自己写。MIT，可 fork。 |
| [onfabric/context-use](https://github.com/onfabric/context-use)（致谢） | 36 / 2026-04-20 / v0.19.0（2026-04-20） / MIT | 多平台：Instagram、YouTube、Telegram、其他 | 把对话记录和平台导出包转成 AI 记忆的工具 | official-export | 不用 | 命令行、库、HTTP 接口、Agent Skill | 获取加转换：借做法自己写，获取部分放第三方仓库。MIT，可 fork；默认把导出内容发给云端模型，只取解析部分。 |
| [d3i-infra/data-donation-task](https://github.com/d3i-infra/data-donation-task)（致谢） | 11 / 2026-10-28 / v3.0.0（2026-07-16） / AGPL-3.0 | 多平台：TikTok、YouTube、Instagram、X、其他 | 学术数据捐赠前端：浏览器里用 Pyodide 本地解析平台导出包，内置十个平台的解析器 | official-export | 不用 | 网页应用、Docker | 获取方式：借做法自己写，放第三方仓库。AGPL-3.0：参照格式事实自己实现。 |
| [humandataincome/hudi-packages-connectors](https://github.com/humandataincome/hudi-packages-connectors) | 11 / 2024-07-11 / v3.0.1（2024-07-11） / none | 多平台：Instagram、TikTok、Reddit、X、其他 | GDPR 导出包解析库，校验、解析、汇总各平台导出 zip | official-export | 不用 | 库 | 只作参考。没有 LICENSE 文件（npm 声明 ISC），先按无许可处理。 |
| [digipower-academy/digipower-experiences](https://github.com/digipower-academy/digipower-experiences)（致谢） | 7 / 2026-10-30 / none / NOASSERTION | 多平台：TikTok、Instagram、YouTube、X、其他 | 在浏览器里载入自己的平台导出包做可视化和查询，覆盖约二十个平台 | official-export | 不用 | 网页应用、库 | 只作参考。LICENSE.md 写 AGPL-3.0-or-later：只参考平台清单和格式。 |
| [dj-urg/data-mirroring](https://github.com/dj-urg/data-mirroring)（致谢） | 1 / 2026-04-21 / none / NOASSERTION | 多平台：TikTok、Instagram、YouTube、其他 | 研究工具：上传 TikTok、Instagram、YouTube 导出 JSON，转成表格和图表 | official-export | 不用 | 网页应用、Docker | 获取方式：借做法自己写，放第三方仓库。LICENSE 写 MPL-2.0：改动的文件要公开。 |
| [js713-lab/savehub](https://github.com/js713-lab/savehub)（致谢） | 0 / 2026-10-18 / none / none | 多平台：Instagram、TikTok、X、抖音、小红书、公众号、通用网页、其他 | 本地优先的收藏汇总面板加 MCP：导入各平台官方导出，国内平台用粘贴分享文本 | official-export | 可选 | 网页应用、HTTP 接口、MCP、命令行、Docker | 参考：借做法自己写。没有许可证，只参考；可选采集器会自动重新登录。 |
| [tmwgsicp/ForgeRSS](https://github.com/tmwgsicp/ForgeRSS)（致谢） | 147 / 2026-10-26 / v1.5.1（2026-05-18） / AGPL-3.0 | 多平台：抖音、快手、小红书、B站、知乎、微博、播客、YouTube、TikTok、通用网页 | 把网站和社交平台主页转成 RSS 的生成器，社交平台要在本机带登录浏览器运行 | browser-session | 必须 | 命令行、Docker、其他 | 获取方式：包装它的命令行，放第三方仓库。AGPL-3.0：用户自装，歇存读它生成的 RSS。 |
| [NanmiCoder/NewsCrawler](https://github.com/NanmiCoder/NewsCrawler)（致谢） | 636 / 2026-08-08 / none / GPL-3.0 | 多平台：公众号、头条/西瓜、X、其他 | 新闻与文章抓取集合，主产品 news-extractor Skill 把公众号、头条等文章链接转成 JSON 或 Markdown | session-api | 可选 | Agent Skill、命令行、HTTP 接口、网页应用、MCP、Docker | 获取方式：包装它的命令行，放第三方仓库。GPL-3.0：用户自装，独立进程调用；写死的头条 Cookie 来源不明，接入前换掉。 |
| [NanmiCoder/MediaCrawler](https://github.com/NanmiCoder/MediaCrawler) | 68982 / 2026-10-27 / none / NOASSERTION | 多平台：小红书、抖音、快手、B站、微博、贴吧、知乎 | 按关键词、帖子 ID 或创作者主页采集七个平台的帖子与评论，存 JSONL、CSV、Excel 或数据库（用户点名，见 [§3](#3-用户点名的项目)） | private-api | 必须 | 命令行、HTTP 接口、网页应用 | 相邻，只作参考。自定义非商业学习许可，没有分发权：不 fork、不分发、不包装。 |
| [JunkFood02/Seal](https://github.com/JunkFood02/Seal) | 30185 / 2026-09-25 / v1.13.1（2024-10-16） / GPL-3.0 | 多平台：YouTube、B站、X、Instagram、通用网页 | 基于 yt-dlp 的 Android 下载器，可从系统分享菜单接收链接 | private-api | 可选 | 手机应用 | 参考：借做法自己写。GPL-3.0：用户自装，独立进程调用。 |
| [mikf/gallery-dl](https://github.com/mikf/gallery-dl)（致谢） | 20292 / 2026-10-25 / v1.32.18（2026-10-25） / GPL-2.0 | 多平台：X、Instagram、Pinterest、Reddit、微博、B站、TikTok | 从图片站和社交平台批量下载图片、视频和元数据，支持 X 书签、Instagram 收藏、Pinterest 画板（[02 §3](02-adapter-ecosystems.md#3-gallery-dl)） | private-api | 可选 | 命令行、库 | 获取方式：包装它的命令行，放第三方仓库。GPL-2.0：用户自装，独立进程调用。 |
| [whiteguo233/OpenBiliClaw](https://github.com/whiteguo233/OpenBiliClaw)（致谢） | 3632 / 2026-10-30 / openbiliclaw-v0.3.233（2026-10-30） / MIT | 多平台：B站、小红书、抖音、YouTube、TikTok、X、知乎、Reddit、微博、Instagram、通用网页 | 本地跨平台推荐 Agent：扩展同步各平台历史、收藏和关注，生成兴趣画像再去找内容 | private-api | 必须 | 浏览器扩展、HTTP 接口、网页应用、桌面应用、命令行、Agent Skill | 参考：借做法自己写。MIT，可 fork；写接口对局域网无鉴权开放的风险要注意。 |
| [CharlesPikachu/videodl](https://github.com/CharlesPikachu/videodl) | 3171 / 2026-10-30 / clitools（2025-12-14） / NOASSERTION | 多平台：抖音、快手、小红书、B站、TikTok、YouTube、微博、知乎、头条/西瓜、Reddit | 纯 Python 视频下载库，覆盖上百个站点，另带第三方解析站客户端 | private-api | 可选 | 命令行、库、MCP | 不接入。PolyForm Noncommercial；涉及 DRM 与付费内容，不接入。 |
| [cshuangyy/videdown](https://github.com/cshuangyy/videdown)（致谢） | 2067 / 2026-08-22 / v1.2.7（2026-08-22） / MIT | 多平台：抖音、小红书、B站、YouTube、Instagram、快手、TikTok、X | 以 yt-dlp 为引擎的 Electron 桌面下载器，DownVid 的上游 | private-api | 可选 | 桌面应用 | 只作参考。MIT；致谢 DownVid 时一起列。 |
| [iszhouhua/social-media-copilot](https://github.com/iszhouhua/social-media-copilot)（致谢） | 1391 / 2026-08-31 / none / GPL-3.0 | 小红书、抖音、快手 | 小红书、抖音、快手的数据采集浏览器扩展，server 分支可被 HTTP 调用 | private-api | 可选 | 浏览器扩展、HTTP 接口 | 获取方式：调它的 HTTP 接口，放第三方仓库。GPL-3.0：用户自装，独立进程调用；商店版闭源；扩展曾被 Chrome 商店下架后换 ID 上架。 |
| [chubbyguan/chubbyskills](https://github.com/chubbyguan/chubbyskills)（致谢） | 1289 / 2026-10-14 / v0.15.0（2026-10-08） / MIT | 多平台：抖音、TikTok、B站、YouTube、小红书、公众号、微博、知乎、X、播客 | 14 个 Agent Skill 加统一命令行，把各平台链接存成带来源字段的 Markdown，附检索、资料包和知识库 MCP（用户点名，见 [§3](#3-用户点名的项目)；[05 §1.2](05-note-apps-and-plugins.md#12-独立工具)） | private-api | 可选 | Agent Skill、命令行、MCP | 获取加转换：fork 后改造，获取部分放第三方仓库。MIT，保留版权声明；平台声明文件和失败分型可直接借给清单与错误码。 |
| [Downvid/downvid](https://github.com/Downvid/downvid)（致谢） | 1167 / 2026-10-28 / v1.9.0（2026-10-28） / MIT | 多平台：抖音、快手、小红书、头条/西瓜、B站、YouTube、TikTok、X、Instagram | 跨平台桌面视频下载器：给 yt-dlp、FFmpeg 加图形界面，抖音、快手另有直连解析（用户点名，见 [§3](#3-用户点名的项目)） | private-api | 可选 | 桌面应用 | 获取方式：借做法自己写，放第三方仓库。MIT，可 fork；名称和 Logo 受 TRADEMARKS.md 约束；Go 代码在 internal/ 下，借用要复制源码。 |
| [TikHub/TikHub-API-Python-SDK](https://github.com/TikHub/TikHub-API-Python-SDK) | 929 / 2026-10-03 / v2.2.0（2026-10-03） / Apache-2.0 | 多平台：抖音、TikTok、小红书、快手、微博、Instagram、YouTube、X | 商业社媒数据接口 TikHub 的 SDK 和命令行，一个 API Key 调各平台上千个端点 | private-api | 不用 | 库、命令行、MCP | 获取方式：调它的 HTTP 接口，放第三方仓库。Apache-2.0，可 fork；数据服务是商业服务，只作用户自愿开启的付费兜底。 |
| [arnoldhao/xiadown](https://github.com/arnoldhao/xiadown)（致谢） | 750 / 2026-07-27 / v1.0.4（2026-07-27） / Apache-2.0 | 多平台：YouTube、B站、抖音、快手、小红书、TikTok、通用网页 | 本地优先的媒体资源库桌面应用，集成 yt-dlp、网页媒体嗅探和 RSS 阅读 | private-api | 可选 | 桌面应用 | 参考：借做法自己写。Apache-2.0，可 fork。 |
| [cv-cat/catbus](https://github.com/cv-cat/catbus)（致谢） | 543 / 2026-10-29 / v0.2.1（2026-10-22） / MIT | 多平台：小红书、抖音、TikTok、B站、快手、微博、X、其他 | Node 命令行，用统一命令和 JSON 调小红书、抖音、TikTok、B站、快手、微博等的网页端能力 | private-api | 必须 | 命令行、Agent Skill | 获取方式：包装它的命令行，放第三方仓库。MIT，可 fork；只开放只读命令；它会自动处理验证码，风险高。 |
| [ucmao/media-parser](https://github.com/ucmao/media-parser)（致谢） | 440 / 2026-10-30 / none / MIT | 多平台：抖音、小红书、视频号、公众号、快手、B站、微博、知乎、头条/西瓜 | 自建的多平台分享链接解析服务，视频号用元宝登录态换临时凭证 | private-api | 可选 | HTTP 接口、网页应用、Docker | 获取方式：调它的 HTTP 接口，放第三方仓库。MIT，可 fork；自用时开 API_ONLY，不用动态代理。 |
| [mu-zi-lee/Tangentry](https://github.com/mu-zi-lee/Tangentry) | 210 / 2026-10-30 / v0.0.3.4（2026-10-30） / MIT | 多平台：B站、YouTube、小红书、抖音、播客、通用网页、其他 | 本地优先的多平台内容聚合与笔记桌面应用：订阅、字幕跟读、转写，导出 Markdown | private-api | 可选 | 桌面应用、网页应用、Docker | 相邻：借做法自己写。MIT，可 fork；适配器没有按风险分层，B站 路径带签名。 |
| [z-mio/ParseHub](https://github.com/z-mio/ParseHub) | 190 / 2026-10-30 / v2.3.16（2026-10-30） / MIT | 多平台：抖音、TikTok、小红书、B站、YouTube、微博、知乎、豆瓣、贴吧、快手、公众号、视频号、头条/西瓜、X、Instagram | 异步的社交媒体分享链接解析库和命令行，从分享文案提取链接、去跟踪参数 | private-api | 可选 | 库、命令行 | 获取方式：包装它的命令行，放第三方仓库。MIT，可 fork。 |
| [lucky-Luor/bilibili-douyin-notes](https://github.com/lucky-Luor/bilibili-douyin-notes)（致谢） | 161 / 2026-10-29 / none / MIT | 多平台：B站、抖音、YouTube、小红书 | Agent Skill：抓 B站、抖音、YouTube、小红书视频的字幕或媒体，转写、抽帧后写成课堂笔记（用户点名，见 [§3](#3-用户点名的项目)；[05 §1.2](05-note-apps-and-plugins.md#12-独立工具)） | private-api | 可选 | Agent Skill、命令行 | 获取加转换：fork 后改造，获取部分放第三方仓库。MIT，保留版权声明；抖音签名组件是 GPLv3，fork 时去掉，或改成用户自装的可选项。 |
| [daxiaamu/DBdown](https://github.com/daxiaamu/DBdown) | 37 / 2026-10-28 / v0.6.22（2026-10-28） / GPL-3.0 | 多平台：B站、抖音、YouTube、微博、视频号 | Android 下载器，接收系统分享的链接，下载 B站、抖音、YouTube、微博、视频号内容 | private-api | 可选 | 手机应用 | 只作参考。GPL-3.0。 |
| [dvlin-dev/social-media-favorites-archiver](https://github.com/dvlin-dev/social-media-favorites-archiver)（致谢） | 0 / 2026-08-09 / v1.0.4（2026-08-09） / MIT | B站、小红书、抖音 | 经专用 Chrome 会话枚举本人 B站、小红书、抖音收藏，本地 ASR、OCR 后写成 Markdown | private-api | 必须 | 命令行、Agent Skill | 获取加转换：借做法自己写，获取部分放第三方仓库。MIT，可 fork；枚举不完整时暂停，不写半成品。 |
| [JimLiu/baoyu-skills](https://github.com/JimLiu/baoyu-skills)（致谢） | 27085 / 2026-09-10 / v2.5.2（2026-06-18） / MIT | 多平台：X、YouTube、公众号、微信聊天、微博、通用网页 | 宝玉的 Skill 合集，以创作为主，也有网页转 Markdown、X 帖子、YouTube 字幕、群聊摘要 | client-hook | 可选 | Agent Skill、命令行 | 参考：借做法自己写。MIT，可 fork；群聊摘要依赖的 wx-cli 已被 DMCA 屏蔽。 |
| [putyy/res-downloader](https://github.com/putyy/res-downloader) | 20981 / 2026-10-27 / 3.1.3（2025-12-30） / Apache-2.0 | 多平台：视频号、小程序、抖音、快手、小红书、其他 | 开本机代理抓取视频号、小程序、抖音等页面加载的音视频与图片的资源嗅探器 | client-hook | 可选 | 桌面应用 | 只作参考。Apache-2.0；要装系统根证书，只记录层级。 |
| [mcncarl/yichen-skills](https://github.com/mcncarl/yichen-skills)（致谢） | 5124 / 2026-10-28 / x-article-draft-uploader-v1.0.1（2026-08-16） / NOASSERTION | 多平台：小红书、抖音、X、B站、YouTube、微博、知乎、公众号、微信聊天、播客、头条/西瓜、通用网页 | 逸尘的创作者 Skill 合集：收藏夹链接导出、链接归档、跨平台搜索、ASR、X 草稿，以及微信本地数据解析（用户点名，见 [§3](#3-用户点名的项目)） | client-hook | 可选 | Agent Skill、命令行 | 获取方式：借做法自己写，放第三方仓库。个人非商业许可，禁止再发布：只借收藏页采集规则自己实现。 |
| [cakeni/super-drag](https://github.com/cakeni/super-drag) | 2 / 2026-08-24 / none / MIT | 抖音、小红书、其他 | Android Root 加 Xposed 模块：长按拖拽，保存抖音、小红书 App 里显示的图片、画面或文字 | client-hook | 可选 | 手机应用 | 相邻，不接入。MIT。 |

- **条目级层级偏高，接入时要按平台拆开声明。** 同一个工具在不同平台走不同的路：Downvid 的抖音直连带 cookie、快手匿名、其余交给 yt-dlp；media-parser 的视频号路径借元宝登录态、抖音路径复现签名。按 [§17.1](#171-清单里声明获取层级) 的提议，清单要能按能力或按平台分别声明层级。
- **能直接拿来用的不多。** MediaCrawler、videodl 是非商业许可，LittleCrawler 的小红书模块和 MediaCrawler 一致且去掉了原声明；MIT 的 chubbyskills、bilibili-douyin-notes、Downvid、media-parser 可以 fork 或移植。
- **官方导出这一层有一组个人数据项目。** timelinize、HPI、data-donation-task、digipower-experiences、data-mirroring、context-use 解析的都是用户自己下载的导出包，没有账号风险；多数是 AGPL 或只能参考，借的是各平台导出格式的知识。
- **分享链接解析服务适合做公开页单条获取的后端。** parse-video、media-parser、ParseHub 把单条解析做成命令行或 HTTP 服务；它们把签名或第三方凭证藏在服务里，接入时要看清哪条路要账号、哪条路复现签名。

<a id="5-抖音快手与-tiktok"></a>
## 5. 抖音、快手与 TikTok

| 项目 | ★ / 推送 / release / 许可证 | 平台 | 做什么 | 层级 | 登录 | 接口 | 歇存怎么用 |
|---|---|---|---|---|---|---|---|
| [zollo/tiktok-downloader](https://github.com/zollo/tiktok-downloader)（致谢） | 5 / 2024-03-23 / none / GPL-3.0 | TikTok | 读 TikTok 官方数据导出的 user_data.json，批量下载本人发布过的视频 | official-export | 不用 | 命令行 | 获取方式：借做法自己写，放第三方仓库。GPL-3.0；逻辑很短，按导出格式自己写。 |
| [Inoue-AI/Inoue-AI-TikTok-SDK](https://github.com/Inoue-AI/Inoue-AI-TikTok-SDK)（致谢） | 1 / 2026-06-07 / none / MIT | TikTok | TikTok 官方开放接口的异步 Python SDK，含 Data Portability API | official-export | 必须 | 库 | 获取方式：调它的 HTTP 接口，放第三方仓库。MIT，可 fork；开发者审核门槛高，只用数据转移部分。 |
| [commitmaniac/tdp](https://github.com/commitmaniac/tdp)（致谢） | 0 / 2026-06-07 / 1.1.0（2025-12-07） / BSD-3-Clause | TikTok | C 写的 TikTok 导出解析命令行，打印收藏、喜欢或观看历史的链接 | official-export | 不用 | 命令行 | 获取方式：借做法自己写，放第三方仓库。BSD-3-Clause，可 fork；只认旧版导出的顶层键。 |
| [john00-dev/tiktok-liked-content](https://github.com/john00-dev/tiktok-liked-content)（致谢） | 0 / 2025-12-20 / none / none | TikTok | 读 TikTok 官方导出里的喜欢和收藏，按列表浏览并嵌入播放 | official-export | 不用 | 网页应用、命令行 | 获取方式：借做法自己写，放第三方仓库。没有许可证，按字段自己实现。 |
| [davidteather/TikTok-Api](https://github.com/davidteather/TikTok-Api)（致谢） | 6723 / 2026-08-24 / v7.3.3（2026-04-01） / MIT | TikTok | 非官方 TikTok Python 封装，用 Playwright 会话取公开数据，不支持需要登录的接口 | public-web | 不用 | 库 | 获取方式：包装它的命令行，放第三方仓库。MIT，可 fork。 |
| [yzfly/douyin-mcp-server](https://github.com/yzfly/douyin-mcp-server)（致谢） | 1281 / 2026-07-02 / none / Apache-2.0 · 已归档 | 抖音 | MCP 服务：从抖音分享链接拿视频地址，再用云端语音识别提取文案 | public-web | 不用 | MCP、网页应用、命令行、Agent Skill | 获取加转换：借做法自己写，获取部分放第三方仓库。Apache-2.0，可 fork；已归档，不作依赖。 |
| [wujunwei928/parse-video](https://github.com/wujunwei928/parse-video)（致谢） | 1080 / 2026-08-21 / v0.0.2（2026-04-18） / MIT | 多平台：抖音、快手、头条/西瓜、小红书、微博、B站、X、其他 | Go 写的短视频分享链接解析服务和命令行，覆盖二十多个站点，返回标题、作者和媒体地址 | public-web | 不用 | 命令行、HTTP 接口、Docker、库、Agent Skill | 获取方式：包装它的命令行，放第三方仓库。MIT，可 fork。 |
| [JoeanAmier/KS-Downloader](https://github.com/JoeanAmier/KS-Downloader)（致谢） | 884 / 2026-10-26 / 1.6（2026-07-05） / GPL-3.0 | 快手 | 快手作品下载：按分享链接取单条视频或图集，有终端界面和本地 Web API | public-web | 不用 | 命令行、HTTP 接口、Docker | 获取方式：调它的 HTTP 接口，放第三方仓库。GPL-3.0：用户自装，独立进程调用。 |
| [lid664951-crypto/douyin-creator-toolkit](https://github.com/lid664951-crypto/douyin-creator-toolkit)（致谢） | 747 / 2026-04-06 / v1.1.0（2026-01-14） / MIT | 抖音 | Windows 桌面工具，把抖音链接解析、下载、本地转写、AI 拆解和知识库串成一条流程 | public-web | 不用 | 桌面应用 | 只作参考。MIT；内置的 dy-mcp 没有许可证。 |
| [lyxdream/obsidian-content-capture-backend](https://github.com/lyxdream/obsidian-content-capture-backend)（致谢） | 33 / 2026-06-05 / none / none | 抖音 | Obsidian 抖音入库插件的本地后端：解析分享链接、下载原始文件、本地 Whisper 提取文案（[05 §1.1](05-note-apps-and-plugins.md#11-obsidian-社区插件)） | public-web | 不用 | HTTP 接口、命令行、Agent Skill | 获取加转换：借做法自己写，获取部分放第三方仓库。没有许可证，只参考做法；配套插件是 MIT。 |
| [Hype-yolo/backtap-douyin](https://github.com/Hype-yolo/backtap-douyin)（致谢） | 1 / 2026-09-18 / none / MIT | 抖音 | iOS 快捷指令加 Node 服务：轻点手机背面，把抖音链接发给解析服务，下载到相册 | public-web | 不用 | HTTP 接口、其他 | 获取方式：借做法自己写，放第三方仓库。MIT，可 fork。 |
| [Hannah2227/douyin-favorite-MyLike](https://github.com/Hannah2227/douyin-favorite-MyLike)（致谢） | 619 / 2026-10-30 / v2.0.3（2026-10-30） / MIT | 抖音 | 整理、搜索、回看本人抖音喜欢与收藏的桌面应用，请求由真实页面发出，支持增量和断点续采 | browser-session | 必须 | 桌面应用 | 获取方式：fork 后改造，放第三方仓库。MIT，保留版权声明；接入时去掉取消点赞、移动收藏等写操作。 |
| [samirelanduk/tiktok-save](https://github.com/samirelanduk/tiktok-save)（致谢） | 178 / 2026-04-13 / none / none | TikTok | 用 TikTok 官方导出里的喜欢、收藏、观看记录作清单，逐条下载视频并存元数据 | browser-session | 不用 | 命令行 | 获取方式：借做法自己写，放第三方仓库。没有许可证，只参考做法。 |
| [Kuhakucai/douyin-mcp](https://github.com/Kuhakucai/douyin-mcp) | 84 / 2026-07-29 / none / AGPL-3.0 | 抖音 | 给个人创作者的本地抖音 MCP：复用专用 Chrome 登录态，同步创作者中心的作品与指标 | browser-session | 必须 | MCP、命令行 | 参考：借做法自己写。AGPL-3.0：只借设计。 |
| [tars1230/douyin-favorites-to-knowledge](https://github.com/tars1230/douyin-favorites-to-knowledge)（致谢） | 78 / 2026-10-29 / v2.3.5（2026-10-18） / MIT | 抖音 | 把本人抖音收藏同步成 Obsidian 笔记的 Skill 加命令行，带转写、OCR 和日报 | browser-session | 必须 | 命令行、Agent Skill | 获取加转换：借做法自己写，获取部分放第三方仓库。仓库 MIT，但依赖的轮子是专有许可：不能整仓 fork。 |
| [fluoxtine2004/douyin-favorites-organizer](https://github.com/fluoxtine2004/douyin-favorites-organizer)（致谢） | 5 / 2026-10-14 / v1.0.0（2026-10-14） / MIT | 抖音 | 按原分类抓本人抖音收藏夹，生成逐字稿、关键帧和 Obsidian 笔记（[05 §1.2](05-note-apps-and-plugins.md#12-独立工具)） | browser-session | 必须 | 命令行、网页应用 | 获取加转换：借做法自己写，获取部分放第三方仓库。MIT，可 fork。 |
| [mlbb229229-create/douyin-favorites-mcp](https://github.com/mlbb229229-create/douyin-favorites-mcp)（致谢） | 2 / 2026-08-19 / none / MIT | 抖音 | 用 Playwright 持久化浏览器读本人抖音收藏和收藏夹的只读 MCP | browser-session | 必须 | MCP | 获取方式：fork 后改造，放第三方仓库。MIT，保留版权声明。 |
| [JBGruber/traktok](https://github.com/JBGruber/traktok)（致谢） | 117 / 2026-09-22 / v0.1.1（2025-12-09） / GPL-3.0 | TikTok | R 包：封装 TikTok 研究 API，也提供模拟网页请求的非官方接口 | session-api | 可选 | 库 | 只作参考。GPL-3.0。 |
| [yumingtao/douyin-favorites-sync-backend](https://github.com/yumingtao/douyin-favorites-sync-backend)（致谢） | 1 / 2026-10-08 / 1.0.0（2026-08-28） / MIT | 抖音 | Obsidian 抖音收藏同步插件的本地后端，拉收藏、取文案、下载视频并可转写（[05 §1.1](05-note-apps-and-plugins.md#11-obsidian-社区插件)） | session-api | 必须 | HTTP 接口、网页应用 | 只作参考。标 MIT，但签名模块是加密字节码，无法审计。 |
| [Evil0ctal/Douyin_TikTok_Download_API](https://github.com/Evil0ctal/Douyin_TikTok_Download_API)（致谢） | 20998 / 2026-10-27 / v5.1.8（2026-10-27） / Apache-2.0 | 抖音、TikTok、B站 | 自托管的抖音、TikTok、B站数据服务，经 REST、MCP、命令行取作品、作者、评论、收藏夹（[05 §1.2](05-note-apps-and-plugins.md#12-独立工具)） | private-api | 可选 | HTTP 接口、MCP、命令行、网页应用、Docker | 获取加转换：调它的 HTTP 接口，获取部分放第三方仓库。Apache-2.0，可 fork；它的上游风控标记和节流做法可借给退避策略。 |
| [JoeanAmier/TikTokDownloader](https://github.com/JoeanAmier/TikTokDownloader)（致谢） | 17228 / 2026-10-29 / 5.8（2026-09-13） / GPL-3.0 | 抖音、TikTok | 抖音、TikTok 作品下载与数据采集，可按账号批量取发布、喜欢、收藏和收藏夹 | private-api | 可选 | 命令行、HTTP 接口、Docker | 获取方式：调它的 HTTP 接口，放第三方仓库。GPL-3.0：用户自装，独立进程调用；签名已改由用户自备；不沿用多 IP 轮换。 |
| [jiji262/douyin-downloader](https://github.com/jiji262/douyin-downloader)（致谢） | 12500 / 2026-10-14 / desktop-v0.12.5（2026-10-08） / MIT | 抖音 | 抖音批量下载命令行，带去重、增量和浏览器兜底；同仓库另发闭源桌面版 | private-api | 必须 | 命令行、HTTP 接口、桌面应用 | 参考：借做法自己写。命令行 MIT，桌面版许可不明；它记录了 2026 下半年网页端只放行页内请求。 |
| [Johnserf-Seed/f2](https://github.com/Johnserf-Seed/f2)（致谢） | 2717 / 2026-10-19 / v0.0.1.7（2024-12-31） / Apache-2.0 | 多平台：抖音、TikTok、微博、X | 多平台下载与接口封装（抖音、TikTok、微博、X），抖音能取主页、点赞、收藏和合集 | private-api | 可选 | 命令行、库 | 获取方式：包装它的命令行，放第三方仓库。Apache-2.0，可 fork；正式 release 停在 2024-12，PyPI 版本可能落后。 |
| [ahang008/ah-douyin-clean-downloader](https://github.com/ahang008/ah-douyin-clean-downloader)（致谢） | 253 / 2026-10-03 / v0.1.0（2026-09-21） / NOASSERTION | 抖音 | 面向 Codex 的抖音 Skill：按分享链接下载原视频，本机转写成 Markdown、SRT，也能整号归档（用户点名，见 [§3](#3-用户点名的项目)） | private-api | 可选 | Agent Skill、命令行 | 获取加转换：包装它的命令行，获取部分放第三方仓库。源码可见的非商业许可：只能调用用户自装的副本，做之前先取得法律意见。 |
| [llm-xiaoye/douyin-to-obsidian](https://github.com/llm-xiaoye/douyin-to-obsidian)（致谢） | 206 / 2026-10-21 / v0.1.1（2026-10-21） / MIT | 抖音 | cc-connect 加 Claude Code 的 /dy 技能，把抖音视频、图文和收藏夹整理成 Obsidian 笔记 | private-api | 必须 | Agent Skill、命令行 | 获取加转换：借做法自己写，获取部分放第三方仓库。MIT，可 fork；收藏夹列表经 F2 签名请求；只完整支持 Apple Silicon。 |
| [joeycato/tiktok-favesave](https://github.com/joeycato/tiktok-favesave)（致谢） | 47 / 2025-09-28 / v1.2.0（2025-09-28） / NOASSERTION | TikTok | 读 TikTok 导出里的收藏和喜欢清单，用 yt-dlp 批量下载 | private-api | 不用 | 桌面应用 | 只作参考。CC BY-NC-SA 4.0 且禁止再分发，只参考。 |
| [ZhangYukwiver/DouyinDataReview](https://github.com/ZhangYukwiver/DouyinDataReview)（致谢） | 33 / 2026-10-30 / v1.32.1（2026-10-30） / MIT | 抖音 | 读本人抖音观看历史、喜欢、收藏和聊天记录导出 JSON，也能导入官方「个人信息下载」档案 | private-api | 必须 | 桌面应用、MCP、网页应用、浏览器扩展 | 获取方式：经 MCP 调用，放第三方仓库。MIT，可 fork；只用读取与导出，优先走导入官方档案的路径。 |
| [cnbattle/douyin](https://github.com/cnbattle/douyin) | 1229 / 2026-05-27 / v0.2.0（2021-06-20） / MIT | 抖音 | ADB 驱动抖音 App 刷推荐与搜索，同时用本地中间人代理记录 App 收到的视频列表 | client-hook | 可选 | 命令行 | 只作参考。MIT；只记录层级，不采用。 |
| [huami1314/DYYY](https://github.com/huami1314/DYYY) | 721 / 2026-05-31 / v2.2-8(01032308)（2026-01-03） / MIT | 抖音 | 抖音 iOS 越狱或注入插件，在官方 App 里调整界面、按配置的解析接口保存媒体 | client-hook | 可选 | 手机应用 | 相邻，不接入。MIT。 |
| [BandarHL/BHTikTok](https://github.com/BandarHL/BHTikTok) | 361 / 2024-05-01 / 0.0.1（2023-07-12） / none | TikTok | TikTok iOS 插件，在官方 App 里加下载、隐藏广告等选项；维护停滞 | client-hook | 可选 | 手机应用 | 相邻，不接入。没有许可证。 |
| [xiaodesetingyongzhanghao/FreedomPlus](https://github.com/xiaodesetingyongzhanghao/FreedomPlus) | 37 / 2023-03-13 / none / none | 抖音 | 抖音 Android Xposed 模块的早期副本；原仓库已返回 404 | client-hook | 可选 | 手机应用 | 相邻，不接入。没有许可证。 |

- **抖音网页端 2026 下半年只放行页面内发起的请求**（[jiji262/douyin-downloader #258](https://github.com/jiji262/douyin-downloader/issues/258)）。代码里带 cookie 直接调收藏接口的工具（TikTokDownloader、f2）收藏类接口时常 403；由页面自己发请求的 MyLike、douyin-favorites-mcp 仍能取列表，是抖音收藏最稳的一层。
- **抖音单条内容不登录可取。** 分享页的服务端渲染数据（parse-video、obsidian-content-capture-backend、backtap-douyin）不带账号，限制在 IP 层：不登录时单作品接口约 200 次后限流、IP 约两小时解封（[Evil0ctal/Douyin_TikTok_Download_API #626](https://github.com/Evil0ctal/Douyin_TikTok_Download_API/issues/626)）。抖音的账号侧限制多是几小时的临时限制（[Johnserf-Seed/f2 #406](https://github.com/Johnserf-Seed/f2/issues/406)、[JoeanAmier/TikTokDownloader #478](https://github.com/JoeanAmier/TikTokDownloader/issues/478)），没有找到封号报告。
- **官方导出是零风险入口。** TikTok 一侧已有多个导出解析器（tdp、tiktok-save、tiktok-liked-content、zollo/tiktok-downloader），开放接口有 Data Portability SDK；抖音一侧只有 DouyinDataReview 能导入「个人信息下载」档案，导出包含哪些类别待核实。
- **Hook 类都停更或下架。** DYYY、FreedomPlus、BHTikTok、cnbattle/douyin 只记录层级；erma0/douyin 2026-06 称因合规审查清空代码。

<a id="6-小红书"></a>
## 6. 小红书

| 项目 | ★ / 推送 / release / 许可证 | 平台 | 做什么 | 层级 | 登录 | 接口 | 歇存怎么用 |
|---|---|---|---|---|---|---|---|
| [JoeanAmier/XHS-Downloader](https://github.com/JoeanAmier/XHS-Downloader)（致谢） | 13158 / 2026-10-30 / 2.8（2026-09-12） / GPL-3.0 | 小红书 | 小红书笔记解析与作品下载，有图形界面、命令行、本地 API、MCP 和提取收藏链接的油猴脚本（[05 §1.2](05-note-apps-and-plugins.md#12-独立工具)） | public-web | 可选 | 桌面应用、命令行、HTTP 接口、MCP、Docker、其他 | 获取方式：调它的 HTTP 接口，放第三方仓库。GPL-3.0：用户自装，独立进程调用；要用新取的链接；脚本的自动滚动默认关。 |
| [NEORUAA/XHS_Downloader_Android](https://github.com/NEORUAA/XHS_Downloader_Android) | 615 / 2026-09-26 / 1.4.1（2026-09-26） / AGPL-3.0 | 小红书 | Android 应用：从剪贴板或系统分享接收小红书链接，下载原图、视频和实况照片 | public-web | 可选 | 手机应用 | 只作参考。AGPL-3.0；作手机分享入口的参考。 |
| [bonchiang96/xiaohongshu-importer](https://github.com/bonchiang96/xiaohongshu-importer)（致谢） | 141 / 2026-01-14 / 1.1.3（2026-01-14） / MIT | 小红书 | Obsidian 插件：粘贴小红书分享文本，生成带图片或视频的笔记（[05 §1.1](05-note-apps-and-plugins.md#11-obsidian-社区插件)） | public-web | 不用 | Obsidian 插件 | 获取方式：借做法自己写，放第三方仓库。MIT，可 fork。 |
| [chenjin-cmd/xhs-account-teardown](https://github.com/chenjin-cmd/xhs-account-teardown)（致谢） | 111 / 2026-09-29 / none / MIT | 小红书 | Agent Skill：不登录抓小红书主页与第一页笔记，或凭带 xsec_token 的链接抓单条笔记（用户点名，见 [§3](#3-用户点名的项目)） | public-web | 不用 | Agent Skill、命令行 | 获取方式：借做法自己写，放第三方仓库。MIT，可 fork；两个脚本可移植成单条获取插件。 |
| [nfe-w/xhs_pic_download](https://github.com/nfe-w/xhs_pic_download) | 85 / 2025-04-07 / none / none | 小红书 | iOS 快捷指令配合自建 Node 服务，一键把小红书笔记的原图或视频存进相册 | public-web | 可选 | HTTP 接口、Docker、其他 | 只作参考。没有 LICENSE 文件，不复用代码。 |
| [feitangyuan/kankan-shoucang](https://github.com/feitangyuan/kankan-shoucang)（致谢） | 53 / 2026-08-20 / v0.2.5（2026-08-20） / AGPL-3.0 | 小红书 | macOS 应用：把小红书笔记卡片拖进来，匿名解析单篇，OCR、转写后自动分类 | public-web | 不用 | 桌面应用、浏览器扩展、MCP、HTTP 接口 | 获取加转换：借做法自己写，获取部分放第三方仓库。AGPL-3.0：用户自装，独立进程调用；借「手动拖拽取链接、匿名取单篇」的做法。 |
| [CH-YYK/obsidian-rednote](https://github.com/CH-YYK/obsidian-rednote)（致谢） | 2 / 2026-09-14 / 1.2.1（2026-09-14） / MIT | 小红书 | Obsidian 插件：粘贴小红书链接匿名生成笔记，可由快捷指令经 obsidian:// 触发 | public-web | 不用 | Obsidian 插件 | 获取方式：借做法自己写，放第三方仓库。MIT，可 fork。 |
| [ceciliayu56-cell/xhs-to-obsidian-scriptable](https://github.com/ceciliayu56-cell/xhs-to-obsidian-scriptable)（致谢） | 1 / 2026-07-16 / v1.0.0（2026-07-10） / MIT | 小红书 | iPhone 快捷指令加 Scriptable：从剪贴板取小红书链接，匿名读笔记，经 obsidian:// 写入 | public-web | 不用 | 其他 | 获取加转换：借做法自己写，获取部分放第三方仓库。MIT，可 fork；媒体交给阿里云识别，不是本地处理。 |
| [xpzouying/xiaohongshu-mcp](https://github.com/xpzouying/xiaohongshu-mcp)（致谢） | 16671 / 2026-10-26 / v2.5.5（2026-09-22） / Apache-2.0 | 小红书 | Go 写的小红书 MCP 与 HTTP 服务，内置浏览器登录，提供搜索、详情、收藏等工具 | browser-session | 必须 | MCP、HTTP 接口、命令行、Docker | 获取方式：经 MCP 调用，放第三方仓库。Apache-2.0，可 fork；2026 年封号报告集中，只开放读取工具。 |
| [autoclaw-cc/xiaohongshu-skills](https://github.com/autoclaw-cc/xiaohongshu-skills) | 2005 / 2026-05-23 / v0.1.0-b043748（2026-05-23） / MIT | 小红书 | 小红书 Skill：本地 CLI 经 WebSocket 连浏览器扩展，在已登录的真实浏览器里读写 | browser-session | 必须 | Agent Skill、命令行、浏览器扩展 | 参考：借做法自己写。MIT，可 fork；借「CLI 连扩展」的结构；issue 里有封号报告。 |
| [iFurySt/RedNote-MCP](https://github.com/iFurySt/RedNote-MCP) | 1127 / 2025-05-11 / v0.1.3（2025-04-10） / MIT | 小红书 | 小红书 MCP：用 Playwright 打开登录后的网页，搜索笔记、按 URL 读内容；已停更 | browser-session | 必须 | MCP、命令行 | 只作参考。MIT。 |
| [jackwener/xhs-cli](https://github.com/jackwener/xhs-cli)（致谢） | 674 / 2026-10-09 / none / Apache-2.0 | 小红书 | 小红书命令行的浏览器版：Camoufox 打开网页读 `__INITIAL_STATE__`，能取收藏列表 | browser-session | 必须 | 命令行、Agent Skill | 获取方式：包装它的命令行，放第三方仓库。Apache-2.0，可 fork；包装 `xhs favorites --json`。 |
| [PCPrincipal67/xhs-favorites-exporter](https://github.com/PCPrincipal67/xhs-favorites-exporter)（致谢） | 28 / 2026-04-13 / none / none | 小红书 | Chrome 扩展：在已登录的小红书收藏页，把收藏条目（含 xsec_token）导出成 JSON | browser-session | 必须 | 浏览器扩展 | 获取方式：借做法自己写，放第三方仓库。没有 LICENSE 文件，只借 SSR、XHR、DOM 三路合并的做法。 |
| [ValerieTse/RedCaChe](https://github.com/ValerieTse/RedCaChe)（致谢） | 26 / 2026-07-10 / none / MIT | 小红书 | 本地优先的小红书收藏整理：Playwright 登录导入收藏，逐条决定保留、移除或归档 | browser-session | 必须 | 网页应用、浏览器扩展 | 获取方式：借做法自己写，放第三方仓库。MIT，可 fork。 |
| [LinkisLethe/REDnote-clipper](https://github.com/LinkisLethe/REDnote-clipper)（致谢） | 14 / 2026-10-04 / v1.0.1（2026-08-23） / Apache-2.0 | 小红书 | Chrome 扩展：把当前或按关键词选中的小红书图文转成 Markdown，在浏览器里本地 OCR | browser-session | 可选 | 浏览器扩展 | 获取加转换：借做法自己写，获取部分放第三方仓库。Apache-2.0，可 fork。 |
| [themrv1ck/xiaohongshu-web-collection-organizing](https://github.com/themrv1ck/xiaohongshu-web-collection-organizing)（致谢） | 14 / 2026-09-13 / none / MIT | 小红书 | 在用户已登录的浏览器里读小红书收藏页当前可见的卡片，确认后归入专辑的 Skill | browser-session | 必须 | Agent Skill、MCP、命令行 | 获取方式：借做法自己写，放第三方仓库。MIT，可 fork；只用读取部分；它不自动滚动。 |
| [zhulin025/xiaohongshu-exporter](https://github.com/zhulin025/xiaohongshu-exporter) | 13 / 2026-01-14 / none / none | 小红书 | Chrome 扩展：自动滚动采集小红书收藏夹，导出到文件、飞书多维表格或 Notion（[05 §1.2](05-note-apps-and-plugins.md#12-独立工具)） | browser-session | 必须 | 浏览器扩展 | 只作参考。没有 LICENSE 文件。 |
| [wkwunju/xhs-vault](https://github.com/wkwunju/xhs-vault)（致谢） | 7 / 2026-08-18 / none / MIT | 小红书 | 小红书收藏夹导出成本地知识库：油猴脚本导出清单，逐条抓正文和字幕，Vision OCR | browser-session | 必须 | 命令行、其他 | 获取加转换：fork 后改造，获取部分放第三方仓库。MIT，保留版权声明；逐条抓取不带账号，正是混合路径。 |
| [rowland-dot/favsense](https://github.com/rowland-dot/favsense)（致谢） | 5 / 2026-10-16 / none / MIT | 小红书 | 本地小红书收藏整理：专用扫描浏览器读收藏清单，匿名取详情，转写、OCR 后生成知识库网页 | browser-session | 必须 | Agent Skill、网页应用、命令行 | 获取方式：借做法自己写，放第三方仓库。MIT，可 fork；遇验证码即停，并把停止状态持久保存。 |
| [nonomil/XiaoHongshu_Collection](https://github.com/nonomil/XiaoHongshu_Collection) | 3 / 2026-05-11 / none / MIT | 多平台：小红书、知乎、公众号、通用网页 | Windows 上经 Chrome 调试端口导出小红书收藏夹为 Markdown，带 OCR 和摘要 | browser-session | 必须 | 命令行、网页应用、HTTP 接口 | 参考：借做法自己写。MIT，可 fork。 |
| [loresun/xhs-hub](https://github.com/loresun/xhs-hub)（致谢） | 0 / 2026-07-27 / none / MIT | 小红书 | 油猴脚本在已登录页面截获列表和详情请求，推给本地服务存 SQLite，视频自动转写 | browser-session | 必须 | 网页应用、HTTP 接口、其他 | 获取加转换：借做法自己写，获取部分放第三方仓库。MIT，可 fork。 |
| [chenxiachan/xhs-claude-skills](https://github.com/chenxiachan/xhs-claude-skills)（致谢） | 430 / 2026-08-14 / none / MIT | 小红书 | Claude Code 插件：/xhs 加链接，把一篇小红书笔记提取成 Obsidian 笔记，图片 OCR、视频转写 | session-api | 必须 | Agent Skill | 获取加转换：借做法自己写，获取部分放第三方仓库。MIT，可 fork；这一步可以改成不带 Cookie。 |
| [cv-cat/Spider_XHS](https://github.com/cv-cat/Spider_XHS) | 8316 / 2026-10-22 / v5.0.0（2026-09-07） / none | 小红书 | 小红书数据采集与运营库，封装网页端、创作者平台等接口，含私信、直播模块 | private-api | 必须 | 库、Agent Skill | 只作参考。没有 LICENSE 文件，README 禁止商业化：不复用。 |
| [jackwener/xiaohongshu-cli](https://github.com/jackwener/xiaohongshu-cli) | 2713 / 2026-03-21 / none / none | 小红书 | 小红书命令行的逆向 API 版：读本机 Cookie，搜索、读笔记、查收藏 | private-api | 必须 | 命令行、Agent Skill | 只作参考。没有 LICENSE 文件，不能复用。 |
| [ReaJason/xhs](https://github.com/ReaJason/xhs) | 2228 / 2025-07-01 / none / MIT | 小红书 | 小红书网页端接口的 Python 封装（PyPI 包名 xhs）；已停更 | private-api | 必须 | 库、HTTP 接口 | 只作参考。MIT。 |
| [Cloxl/xhshow](https://github.com/Cloxl/xhshow) | 1106 / 2026-06-11 / v0.2.0（2026-06-11） / MIT | 小红书 | 小红书网页端请求签名生成库 | private-api | 必须 | 库 | 只作参考。MIT；歇存官方不依赖。 |
| [lucasygu/redbook](https://github.com/lucasygu/redbook)（致谢） | 534 / 2026-09-12 / v0.8.2（2026-09-12） / MIT | 小红书 | 面向 Agent 的小红书命令行：读本机 Chrome Cookie，搜索、读笔记、看收藏和专辑 | private-api | 必须 | 命令行、Agent Skill | 获取方式：包装它的命令行，放第三方仓库。MIT，可 fork；只开放只读命令。 |
| [jobsonlook/xhs-mcp](https://github.com/jobsonlook/xhs-mcp) | 520 / 2026-10-29 / none / none | 小红书 | 小红书 MCP（Python）：带用户 Cookie 请求网页端接口，提供推荐、搜索、读笔记和评论 | private-api | 必须 | MCP、Docker | 只作参考。没有许可证，不能复用。 |
| [justoneapi-labs/xiaohongshu-api](https://github.com/justoneapi-labs/xiaohongshu-api) | 56 / 2026-09-23 / none / MIT | 小红书 | 第三方数据服务 Just One API 的小红书调用示例，用服务商 Token 搜索和取详情 | private-api | 不用 | 命令行、库 | 获取方式：调它的 HTTP 接口，放第三方仓库。MIT，可 fork；API 按次付费，查询内容经过服务商。 |
| [ytf606/xhs2obsidian](https://github.com/ytf606/xhs2obsidian) | 6 / 2026-05-28 / v1.2.0（2026-05-28） / MIT | 小红书 | Obsidian 插件：把小红书收藏、点赞、自己的帖子、订阅博主同步成 Markdown | private-api | 必须 | Obsidian 插件 | 只作参考。MIT。 |
| [SimpsonGet/XhsWatermarkRemove](https://github.com/SimpsonGet/XhsWatermarkRemove) | 14 / 2025-07-08 / release-v1.0.0（2025-07-08） / none | 小红书 | 在小红书 Android App 里保存原图的 frida 脚本，附 Xposed 模块版 | client-hook | 可选 | 手机应用、其他 | 相邻，不接入。没有许可证。 |

- **2026 年封号报告最集中的平台。** 浏览器驱动（xiaohongshu-mcp、OpenCLI）和本地签名（MediaCrawler、Spider_XHS、xiaohongshu-cli）两条路都有低频只读被警告或封禁的报告；恢复时间从 2024 年的几小时变成 2026 年的禁言 7 天到永久封禁，申诉要实名（[§16.2](#162-issue-里的账号问题)）。
- **单条笔记不登录可取，但链接有讲究。** 要带和笔记绑定的 `xsec_token`，token 会过期；分享链接带跟踪参数，不登录批量取会牵连复制链接的账号。只适合单条、低频、拿到链接尽快取。
- **收藏列表只能在登录侧拿，按风险从低到高有：** 只读当前可见的卡片、不自动滚动（themrv1ck/xiaohongshu-web-collection-organizing）；用户自己浏览器里的扩展或油猴脚本（xhs-favorites-exporter、xhs-vault、xhs-hub）；自动化浏览器（RedCaChe、favsense、xiaohongshu-mcp）；本地签名（redbook、xhs2obsidian）。
- **许可证普遍不理想。** 收藏导出写得最好的 xhs-favorites-exporter、xiaohongshu-exporter 没有 LICENSE 文件；可以 fork 的是 xhs-vault、RedCaChe、xhs-account-teardown（MIT）和 REDnote-clipper（Apache-2.0）。

<a id="7-b站youtube-与播客"></a>
## 7. B站、YouTube 与播客

| 项目 | ★ / 推送 / release / 许可证 | 平台 | 做什么 | 层级 | 登录 | 接口 | 歇存怎么用 |
|---|---|---|---|---|---|---|---|
| [purarue/google_takeout_parser](https://github.com/purarue/google_takeout_parser)（致谢） | 142 / 2026-07-03 / v0.1.13（2025-06-20） / MIT | YouTube | 解析 Google Takeout 导出包，把 YouTube 观看历史、评论、点赞转成结构化事件 | official-export | 不用 | 库、命令行 | 获取方式：借做法自己写，放第三方仓库。MIT，可 fork；稍后观看在 CSV 里，要自己补。 |
| [dogsheep/google-takeout-to-sqlite](https://github.com/dogsheep/google-takeout-to-sqlite)（致谢） | 122 / 2023-09-06 / none / Apache-2.0 | YouTube、其他 | 把 Google Takeout 里的「我的活动」和定位记录写进 SQLite | official-export | 不用 | 命令行 | 获取方式：借做法自己写，放第三方仓库。Apache-2.0，可 fork。 |
| [scripting/opmlPackage](https://github.com/scripting/opmlPackage)（致谢） | 42 / 2025-12-08 / none / MIT | 播客 | OPML 格式作者维护的读写库，用于订阅列表的导入导出 | official-export | 不用 | 库 | 获取方式：借做法自己写，放第三方仓库。MIT，可 fork。 |
| [lightpohl/podcast-dl](https://github.com/lightpohl/podcast-dl)（致谢） | 590 / 2026-08-17 / v12.1.4（2026-08-17） / MIT | 播客 | Node 写的播客下载归档命令行，读 RSS 下载音频，可存元数据和文字稿 | public-web | 不用 | 命令行 | 获取方式：包装它的命令行，放第三方仓库。MIT，可 fork。 |
| [chhoumann/PodNotes](https://github.com/chhoumann/PodNotes)（致谢） | 266 / 2026-10-29 / 2.24.0（2026-10-25） / MIT | 播客 | Obsidian 插件：在库里播放播客、记时间戳笔记，可调云端转写（[05 §1.1](05-note-apps-and-plugins.md#11-obsidian-社区插件)） | public-web | 不用 | Obsidian 插件 | 获取加转换：借做法自己写，获取部分放第三方仓库。MIT，可 fork。 |
| [34892002/bilibili-mcp-js](https://github.com/34892002/bilibili-mcp-js) | 196 / 2026-10-23 / v0.1.4（2026-10-23） / MIT | B站 | TypeScript 写的 B站 搜索 MCP：搜索、热门、视频详情、UP 主信息 | public-web | 不用 | MCP | 只作参考。MIT。 |
| [xiaoleiy/podpull](https://github.com/xiaoleiy/podpull)（致谢） | 147 / 2026-09-05 / none / MIT | 播客 | 播客下载命令行，接受 Apple Podcasts、RSS、小宇宙单集链接，可选本地转写 | public-web | 不用 | 命令行、网页应用 | 获取方式：包装它的命令行，放第三方仓库。MIT，可 fork。 |
| [weisi-gu/xiaoyuzhou-podcast-notes](https://github.com/weisi-gu/xiaoyuzhou-podcast-notes) | 57 / 2026-07-12 / none / MIT | 播客 | Agent Skill：小宇宙单集转结构化笔记，音频送阿里云转写 | public-web | 不用 | Agent Skill、命令行 | 获取加转换：借做法自己写，获取部分放第三方仓库。MIT，可 fork。 |
| [Bin-Zhang-hhht/EchoForge](https://github.com/Bin-Zhang-hhht/EchoForge) | 0 / 2026-10-30 / none / MIT | 播客 | 个人技术播客雷达：每天只读白名单播客 RSS 的元信息，本地先取官方逐字稿、没有才转写，写成带原文定位的中文精编 | public-web | 不用 | 命令行、Docker、Agent Skill、网页应用 | 参考：借做法自己写。MIT，可 fork；借「先找逐字稿、没有再转写」和产物检查的规则。 |
| [ItsMaciek/WatchNow](https://github.com/ItsMaciek/WatchNow)（致谢） | 0 / 2025-12-11 / none / none | YouTube | 读 Google Takeout 里 YouTube「稍后观看」的 CSV，显示缩略图并排序 | public-web | 不用 | 网页应用 | 获取方式：借做法自己写，放第三方仓库。没有许可证，只参考。 |
| [the1812/Bilibili-Evolved](https://github.com/the1812/Bilibili-Evolved) | 30940 / 2026-10-17 / v2.11.5（2026-10-17） / NOASSERTION | B站 | B站 网页增强用户脚本，以组件提供下载、快速收藏、稍后再看等功能 | browser-session | 可选 | 浏览器扩展 | 只作参考。MIT 加转发附加条件，只作参考。 |
| [xifangczy/cat-catch](https://github.com/xifangczy/cat-catch) | 22622 / 2026-10-28 / 2.7.4（2026-10-26） / GPL-3.0 | 通用网页 | 浏览器资源嗅探扩展，列出页面加载的音视频资源供下载 | browser-session | 可选 | 浏览器扩展 | 只作参考。GPL-3.0；商店里有加了广告代码的仿冒版。 |
| [IndieKKY/bilibili-subtitle](https://github.com/IndieKKY/bilibili-subtitle)（致谢） | 1187 / 2025-11-04 / none / MIT | B站 | 浏览器扩展：在 B站 视频页列出字幕，可下载，并调模型翻译和总结 | browser-session | 可选 | 浏览器扩展 | 获取加转换：借做法自己写，获取部分放第三方仓库。MIT，可 fork。 |
| [ayasa520/bilibili-favorites-exporter](https://github.com/ayasa520/bilibili-favorites-exporter)（致谢） | 48 / 2025-09-09 / none / MIT | B站 | 浏览器扩展：在 B站 收藏夹页把自建收藏夹和订阅合集导出为 JSON | browser-session | 可选 | 浏览器扩展 | 获取方式：fork 后改造，放第三方仓库。MIT，保留版权声明；失效视频的信息从第三方站补取。 |
| [yutto-dev/yutto](https://github.com/yutto-dev/yutto)（致谢） | 2118 / 2026-10-27 / v2.3.1（2026-08-28） / GPL-3.0 | B站 | Python 的 B站 命令行下载器，支持收藏夹、稍后再看批量，可另存字幕和弹幕 | session-api | 可选 | 命令行、HTTP 接口 | 获取方式：包装它的命令行，放第三方仓库。GPL-3.0：用户自装，独立进程调用；只开放收藏夹、稍后再看和普通投稿。 |
| [eudaemoniaYH/echo-podcast-memory](https://github.com/eudaemoniaYH/echo-podcast-memory) | 1 / 2026-08-14 / none / NOASSERTION | 播客 | macOS 上的播客收听分析原型：浏览器扩展读出小宇宙、机核的登录令牌，本机服务带令牌取收听历史，另读 Apple 播客的本机记录 | session-api | 必须 | 网页应用、浏览器扩展 | 不接入。自定义「保留所有权利」许可：只许原样运行做个人评估，不能修改或分发。 |
| [yt-dlp/yt-dlp](https://github.com/yt-dlp/yt-dlp)（致谢） | 203882 / 2026-10-29 / 2026.10.26（2026-10-26） / Unlicense | 多平台：YouTube、B站、抖音、TikTok、小红书、微博、知乎、X、Instagram、Reddit、Pinterest、播客、通用网页 | 命令行音视频下载器，内置上千个站点抽取器，可只输出 JSON 元数据和字幕（[02 §2](02-adapter-ecosystems.md#2-yt-dlp)） | private-api | 可选 | 命令行、库 | 获取方式：包装它的命令行，放第三方仓库。Unlicense，可 fork；不传 cookie 就没有账号风险；YouTube 要外部 JS 运行时。 |
| [soimort/you-get](https://github.com/soimort/you-get)（致谢） | 56863 / 2026-10-06 / v0.4.1743（2025-01-04） / NOASSERTION | 多平台：B站、YouTube、其他、通用网页 | Python 网页媒体下载器，按站点表路由，表外交给通用抽取器，可只输出 JSON 信息（[02 §4](02-adapter-ecosystems.md#4-you-get)） | private-api | 可选 | 命令行、库 | 获取方式：包装它的命令行，放第三方仓库。LICENSE.txt 正文是 MIT，可 fork。 |
| [imputnet/cobalt](https://github.com/imputnet/cobalt)（致谢） | 45648 / 2026-04-06 / none / AGPL-3.0 | 多平台：YouTube、B站、TikTok、X、Instagram、Reddit、Pinterest、其他 | 自托管的媒体保存服务，提交链接返回可下载的音视频或图片（[02 §8](02-adapter-ecosystems.md#8-cobalt)） | private-api | 可选 | HTTP 接口、网页应用、Docker | 获取方式：调它的 HTTP 接口，放第三方仓库。api 是 AGPL-3.0、web 是 CC-BY-NC-SA-4.0：用户自建实例，经 HTTP 调用。 |
| [TeamNewPipe/NewPipe](https://github.com/TeamNewPipe/NewPipe) | 40361 / 2026-10-28 / v0.29.1（2026-08-15） / GPL-3.0 | YouTube、其他 | Android 上不登录使用的 YouTube 等平台前端，订阅和历史只存本机 | private-api | 不用 | 手机应用、库 | 相邻，只作参考。GPL-3.0。 |
| [iawia002/lux](https://github.com/iawia002/lux)（致谢） | 31816 / 2026-03-29 / v0.24.1（2024-05-06） / MIT | 多平台：B站、YouTube、抖音、快手、头条/西瓜、小红书、微博、知乎、TikTok、X、Instagram、Reddit、Pinterest、其他 | Go 写的视频下载库和命令行，按域名路由到各站点抽取器（[02 §5](02-adapter-ecosystems.md#5-lux)） | private-api | 可选 | 命令行、库 | 获取方式：包装它的命令行，放第三方仓库。MIT，可 fork；release 停在 2024-05，站点改版后修得慢。 |
| [leiurayer/downkyi](https://github.com/leiurayer/downkyi) | 24362 / 2026-07-06 / v1.6.1（2023-12-10） / GPL-3.0 | B站 | Windows 版 B站 下载器；2026-07 收到律师函后宣布永久停止维护 | private-api | 可选 | 桌面应用 | 不接入。GPL-3.0；已收律师函，不建议 fork。 |
| [bggRGjQaUbCoE/PiliPlus](https://github.com/bggRGjQaUbCoE/PiliPlus)（致谢） | 20017 / 2026-10-30 / 2.1.6.2（2026-10-10） / GPL-3.0 | B站 | Flutter 写的 B站 第三方客户端，覆盖五个平台，支持收藏夹和稍后再看 | private-api | 可选 | 手机应用、桌面应用 | 只作参考。GPL-3.0。 |
| [nilaoda/BBDown](https://github.com/nilaoda/BBDown) | 13846 / 2026-05-14 / 1.6.3（2024-08-14） / MIT · 已归档 | B站 | C# 写的 B站 命令行下载器；2026-05 起源码删除并归档（[02 §6](02-adapter-ecosystems.md#6-bbdown)） | private-api | 可选 | 命令行、HTTP 接口 | 不接入。MIT；上游已停、同类收到律师函，不建议任何形式的适配。 |
| [tubearchivist/tubearchivist](https://github.com/tubearchivist/tubearchivist) | 8555 / 2026-10-25 / v0.6.0（2026-10-25） / GPL-3.0 | YouTube | 自托管 YouTube 归档：按频道、播放列表订阅，yt-dlp 下载，写进 Elasticsearch 供搜索 | private-api | 可选 | 网页应用、HTTP 接口、Docker | 相邻，只作参考。GPL-3.0。 |
| [jdepoix/youtube-transcript-api](https://github.com/jdepoix/youtube-transcript-api)（致谢） | 8532 / 2026-10-30 / v1.2.4（2026-01-29） / MIT | YouTube | 按视频 ID 取 YouTube 字幕的库和命令行，不要 API key，也不要浏览器 | private-api | 不用 | 库、命令行 | 获取方式：包装它的命令行，放第三方仓库。MIT，可 fork；云服务器 IP 常被拦，家用网络问题少。 |
| [JefferyHcool/BiliNote](https://github.com/JefferyHcool/BiliNote)（致谢） | 7664 / 2026-09-01 / v2.4.5（2026-08-25） / MIT | 多平台：B站、YouTube、抖音、快手、播客 | AI 视频笔记：输入链接或本地视频，转写后生成带时间戳和截图的 Markdown 笔记（[05 §1.2](05-note-apps-and-plugins.md#12-独立工具)） | private-api | 可选 | 网页应用、桌面应用、HTTP 接口、Docker、浏览器扩展 | 获取加转换：借做法自己写，获取部分放第三方仓库。MIT，但抖音签名文件头是 GPL-3.0；转写与笔记部分可借给转换插件。 |
| [LuanRT/YouTube.js](https://github.com/LuanRT/YouTube.js)（致谢） | 5402 / 2026-10-28 / v18.3.0（2026-10-23） / MIT | YouTube | YouTube 内部 InnerTube 接口的 JS 客户端，可取字幕、播放列表、媒体库和观看历史 | private-api | 可选 | 库 | 获取方式：fork 后改造，放第三方仓库。MIT，保留版权声明；可直接作第三方插件的依赖。 |
| [nICEnnnnnnnLee/BilibiliDown](https://github.com/nICEnnnnnnnLee/BilibiliDown) | 5342 / 2026-07-10 / V6.41（2026-04-24） / none | B站 | Java 版 B站 下载器，支持收藏夹、稍后再看；2026-07-10 仓库被清空 | private-api | 可选 | 桌面应用 | 不接入。没有许可证，不包装、不分发。 |
| [btjawa/BiliTools](https://github.com/btjawa/BiliTools) | 5033 / 2026-07-07 / none / none · 已归档 | B站 | 跨平台 B站 工具箱；2026-07 收到律师函后停止维护并归档 | private-api | 可选 | 桌面应用 | 不接入。内容已删，没有许可证。 |
| [Nemo2011/bilibili-api](https://github.com/Nemo2011/bilibili-api) | 4166 / 2026-07-06 / none / none · 已归档 | B站 | Python 的 B站 接口封装（PyPI bilibili-api-python）；2026-07 收到律师函后关停 | private-api | 可选 | 库 | 不接入。原为 GPL-3.0-or-later；上游已关停，不依赖。 |
| [sigma67/ytmusicapi](https://github.com/sigma67/ytmusicapi)（致谢） | 3089 / 2026-10-12 / 1.12.3（2026-09-16） / MIT | YouTube、播客 | YouTube Music 非官方 Python 接口：播放列表、点赞歌曲、收听历史、播客单集 | private-api | 可选 | 库 | 获取方式：fork 后改造，放第三方仓库。MIT，保留版权声明；只开放读接口。 |
| [amtoaer/bili-sync](https://github.com/amtoaer/bili-sync)（致谢） | 2648 / 2026-09-10 / v2.11.1（2026-05-07） / MIT | B站 | Rust 写的 B站 同步服务，定时把收藏夹、合集、稍后再看下载到本地 | private-api | 必须 | 网页应用、HTTP 接口、Docker | 获取方式：借做法自己写，放第三方仓库。MIT，可 fork；带登录态定时批量，风险落在账号上。 |
| [public-clis/bilibili-cli](https://github.com/public-clis/bilibili-cli) | 1134 / 2026-03-14 / none / Apache-2.0 | B站 | B站 终端命令行：视频、搜索、动态、收藏夹、稍后再看、观看历史，输出 YAML 或 JSON | private-api | 可选 | 命令行、Agent Skill | 获取方式：包装它的命令行，放第三方仓库。本体 Apache-2.0，运行时依赖已关停的 GPL 库：只能用户自装。 |
| [844704781/ximalaya_downloader](https://github.com/844704781/ximalaya_downloader) | 866 / 2026-09-20 / v1.0.5（2026-09-20） / AGPL-3.0 | 其他 | 喜马拉雅专辑下载命令行，下载本账号已购或会员可听的音频 | private-api | 必须 | 命令行 | 不接入。AGPL-3.0；涉及付费内容，法律风险高。 |
| [ZubeidHendricks/youtube-mcp-server](https://github.com/ZubeidHendricks/youtube-mcp-server) | 592 / 2026-08-08 / v1.0.2（2026-07-16） / MIT | YouTube | 用 YouTube Data API 查视频、频道、播放列表的 MCP 服务 | private-api | 不用 | MCP | 只作参考。MIT；字幕工具取不到字幕（#19）。 |
| [jkawamoto/mcp-youtube-transcript](https://github.com/jkawamoto/mcp-youtube-transcript)（致谢） | 494 / 2026-10-28 / v0.8.3（2026-10-28） / MIT | YouTube | 取 YouTube 字幕的 MCP 服务，长字幕分页并附元数据 | private-api | 不用 | MCP | 只作参考。MIT；直接用它依赖的库实现即可。 |
| [ultrazg/xyz](https://github.com/ultrazg/xyz)（致谢） | 278 / 2026-08-13 / v1.10.3（2026-07-25） / MIT | 播客 | Go 写的小宇宙 App 接口封装，本地 HTTP 服务提供订阅、收藏、收听历史等 | private-api | 必须 | HTTP 接口、库、Docker | 获取方式：调它的 HTTP 接口，放第三方仓库。MIT，可 fork；README 首行警告第三方登录可能被风控或封禁。 |
| [KingJing1/podcast-transcript-txt-skill](https://github.com/KingJing1/podcast-transcript-txt-skill) | 47 / 2026-09-19 / v0.1.0（2026-07-09） / MIT | 播客、YouTube、X | Agent Skill：把 YouTube、播客、小宇宙、X 链接转成纯文本逐字稿，先找现成文字稿 | private-api | 不用 | Agent Skill、命令行 | 获取加转换：借做法自己写，获取部分放第三方仓库。MIT，可 fork。 |
| [XZXZZX-Ai/bilibili-mcp](https://github.com/XZXZZX-Ai/bilibili-mcp)（致谢） | 45 / 2026-10-30 / v1.14.6（2026-10-30） / Apache-2.0 | B站 | 本地 B站 MCP：读字幕并定位原话、读评论、遍历自己的全部收藏夹，可本地转写 | private-api | 必须 | MCP、命令行 | 获取加转换：经 MCP 调用，获取部分放第三方仓库。Apache-2.0，可 fork。 |
| [MosesHe/xiaoyuzhoufm-mcp](https://github.com/MosesHe/xiaoyuzhoufm-mcp) | 29 / 2025-05-20 / none / MIT | 播客 | Go 写的小宇宙 MCP，手机号登录后查播客、单集和用户资料 | private-api | 必须 | MCP | 只作参考。MIT；所有用户共用写死的设备 ID，易被识别。 |
| [JackMeds/BiliDigest](https://github.com/JackMeds/BiliDigest) | 0 / 2026-10-07 / v0.2.3（2026-02-07） / GPL-3.0 | B站 | 读本人 B站 稍后再看与收藏夹，导出字幕、AI 小助手总结和本地转写的 Agent 工具 | private-api | 必须 | 命令行、HTTP 接口、Agent Skill | 获取加转换：包装它的命令行，获取部分放第三方仓库。GPL-3.0：用户自装，独立进程调用。 |
| [yujincheng08/BiliRoaming](https://github.com/yujincheng08/BiliRoaming) | 11532 / 2026-07-06 / v1.7.0（2023-10-12） / none · 已归档 | B站 | B站 Android 客户端的 Xposed 模块；2026-07 收到律师函后归档 | client-hook | 可选 | 手机应用 | 相邻，不接入。内容已删；只记录层级。 |
| [mzky/m4s-converter](https://github.com/mzky/m4s-converter)（致谢） | 711 / 2026-03-04 / v1.5.7（2026-03-03） / MIT | B站 | 把 B站 客户端缓存的 m4s 合并成 mp4，可同时生成弹幕字幕 | client-hook | 不用 | 命令行 | 获取方式：借做法自己写，放第三方仓库。MIT，可 fork；附带的 MP4Box 另有许可；付费内容的缓存法律风险高。 |
| [maximbilan/podcrack](https://github.com/maximbilan/podcrack) | 4 / 2026-02-07 / none / MIT | 播客 | macOS 命令行，读 Apple Podcasts 本地缓存的 TTML 文字稿并导出 | client-hook | 不用 | 命令行 | 获取方式：借做法自己写，放第三方仓库。MIT，可 fork；属于规避客户端的使用限制。 |
| [SocialSisterYi/bilibili-API-collect](https://github.com/SocialSisterYi/bilibili-API-collect) | 20152 / 2026-01-30 / none / none · 已归档 | B站 | 社区整理的 B站 接口文档集；2026-01 收到律师函后删除文档并归档 | not-applicable | — | 其他 | 不接入。内容已删；歇存文档不复述 B站 私有接口。 |

- **B站 2026 年对逆向非公开接口的项目发律师函**（[§16.3](#163-下架停更与法律风险)）。BBDown、downkyi、bilibili-api、BiliTools、BilibiliDown、BiliRoaming 都已停更、清空或归档；依赖 bilibili-api 的下游（bilibili-cli、PersonalizationMCP、adoresever/bilibili-mcp）随之失修。仍在维护的 bili-sync、yutto、PiliPlus 没有看到来函公告。
- **B站 路线优先选不签名的。** 用户浏览器里的扩展（bilibili-favorites-exporter、bilibili-subtitle）和带 cookie 的收藏夹、稍后再看接口（yutto 只开放这两类）；服务器 IP 上匿名请求常被 412 拦截，家庭网络问题少。
- **YouTube 的列表在 Google Takeout 里。** 稍后观看、观看历史可从 Takeout 解析（google_takeout_parser、WatchNow），是零风险的列表来源；单个视频的字幕和元数据用 yt-dlp 或 youtube-transcript-api，不传 cookie 就没有账号风险，只受访客限速和云 IP 拦截影响。
- **播客几乎都是公开页面。** RSS 和单集页不需要账号（podcast-dl、podpull、PodNotes）；小宇宙的 App 接口封装（xyz、xiaoyuzhoufm-mcp）属 `private-api`，xyz 的 README 首行就警告第三方登录可能被风控。

<a id="8-微信生态"></a>
## 8. 微信生态

| 项目 | ★ / 推送 / release / 许可证 | 平台 | 做什么 | 层级 | 登录 | 接口 | 歇存怎么用 |
|---|---|---|---|---|---|---|---|
| [BlueMatthew/WechatExporter](https://github.com/BlueMatthew/WechatExporter)（致谢） | 8572 / 2025-02-26 / v1.9.5.13（2022-07-15） / GPL-2.0 | 微信聊天 | 从 iPhone 备份读微信聊天记录，导出 HTML、PDF，有图形界面和命令行 | official-export | 不用 | 桌面应用、命令行 | 获取方式：包装它的命令行，放第三方仓库。GPL-2.0：用户自装，独立进程调用；最后发版在 2022-07，新版数据库格式可能已变。 |
| [malinkang/weread2notion](https://github.com/malinkang/weread2notion) | 2936 / 2026-10-14 / none / none | 微信读书 | 用 GitHub Actions 每天把微信读书书架、划线、想法同步到 Notion（[05 §1.2](05-note-apps-and-plugins.md#12-独立工具)） | official-export | 必须 | 命令行、其他 | 只作参考。没有许可证。 |
| [zhaohongxuan/obsidian-weread-plugin](https://github.com/zhaohongxuan/obsidian-weread-plugin)（致谢） | 2289 / 2026-09-06 / 2.3.0（2026-09-06） / MIT | 微信读书 | 把微信读书书籍、划线、想法、书评同步进 Obsidian 的社区插件（[05 §1.1](05-note-apps-and-plugins.md#11-obsidian-社区插件)） | official-export | 必须 | Obsidian 插件 | 获取方式：借做法自己写，放第三方仓库。MIT，可 fork。 |
| [Tencent/WeChatReading](https://github.com/Tencent/WeChatReading)（致谢） | 268 / 2026-10-10 / releasev1.0.4（2026-07-01） / NOASSERTION | 微信读书 | 微信读书官方 Agent Skill：个人 API Key 调 Gateway，读书架、划线、想法、阅读统计 | official-export | 必须 | Agent Skill、HTTP 接口 | 获取方式：调它的 HTTP 接口，放第三方仓库。实际按 Apache-2.0 使用；书架的文章收藏只返回入口。 |
| [Bwkyd/wexin-read-mcp](https://github.com/Bwkyd/wexin-read-mcp)（致谢） | 462 / 2026-04-19 / none / MIT | 公众号 | 让 AI 读公众号文章的 MCP，抓取交给作者的 url-md 单文件程序 | public-web | 不用 | MCP、命令行 | 获取方式：包装它的命令行，放第三方仓库。MIT，可 fork；url-md 是 Apache-2.0。 |
| [fengxxc/wechatmp2markdown](https://github.com/fengxxc/wechatmp2markdown)（致谢） | 266 / 2025-03-13 / v1.1.11（2025-03-13） / MIT | 公众号 | 把单篇公众号文章转成 Markdown 的 Go 命令行和 Web 服务（[05 §1.2](05-note-apps-and-plugins.md#12-独立工具)） | public-web | 不用 | 命令行、HTTP 接口 | 获取方式：fork 后改造，放第三方仓库。MIT，保留版权声明；2025-03 后没有发版，要自己维护。 |
| [bzd6661/wechat-article-for-ai](https://github.com/bzd6661/wechat-article-for-ai) | 104 / 2026-03-04 / none / none | 公众号 | 把公众号文章转 Markdown 的工具，有命令行、MCP 和 Skill，图片下载到本地 | public-web | 不用 | 命令行、MCP、Agent Skill | 只作参考。没有 LICENSE 文件，暂不复用。 |
| [Zlatanwic/Wechat-Read-MCP-in-Rust](https://github.com/Zlatanwic/Wechat-Read-MCP-in-Rust) | 19 / 2026-02-20 / linux（2026-02-19） / none | 公众号 | Rust 写的公众号文章读取 MCP，无头 Chrome 渲染后提取正文 | public-web | 不用 | MCP | 只作参考。没有许可证，只参考。 |
| [jj-cheng25/weixin-articles-mcp](https://github.com/jj-cheng25/weixin-articles-mcp) | 13 / 2026-05-01 / none / MIT | 公众号、视频号 | 读公众号文章的 MCP，正文、图片和视频关键帧作为图片块返回 | public-web | 不用 | MCP | 获取加转换：借做法自己写，获取部分放第三方仓库。MIT，可 fork。 |
| [cluic/wxauto](https://github.com/cluic/wxauto) | 7427 / 2026-04-13 / none / Apache-2.0 | 微信聊天 | 基于 Windows UIAutomation 的 PC 微信自动化库，开源版针对微信 3.9 | vision-ui | 必须 | 库 | 只作参考。Apache-2.0；README 禁止用于生产。 |
| [feeddd/feeds](https://github.com/feeddd/feeds) | 2087 / 2023-07-05 / none / none | 公众号 | 已关闭的公众号 RSS 项目，订阅源由志愿者手机上的无障碍脚本采集 | vision-ui | 必须 | 手机应用、其他 | 只作参考。没有许可证。 |
| [Hello-Mr-Crab/pywechat](https://github.com/Hello-Mr-Crab/pywechat)（致谢） | 2066 / 2026-09-26 / none / NOASSERTION | 公众号、微信聊天 | 基于 pywinauto 的 Windows 微信 RPA，能导出收藏笔记、批量复制收藏里的文章链接 | vision-ui | 必须 | 库、MCP、Agent Skill | 获取方式：经 MCP 调用，放第三方仓库。LICENSE 是 LGPL-2.1：用户自装、独立进程调用，只用收藏与链接读取。 |
| [freestylefly/WeChatBridge](https://github.com/freestylefly/WeChatBridge)（致谢） | 1907 / 2026-10-27 / none / MIT | 微信聊天 | 接住微信「合并转发 → 转发到其他应用」的聊天记录包，转给本机 AI 应用或文件夹 | vision-ui | 必须 | 桌面应用、其他 | 获取方式：借做法自己写，放第三方仓库。MIT，可 fork；基于 qzz0518/Dukou（MIT），致谢时一起列。 |
| [BiboyQG/WeChat-MCP](https://github.com/BiboyQG/WeChat-MCP) | 287 / 2026-03-09 / v0.2.0（2026-03-09） / MIT | 微信聊天 | macOS 微信 MCP，用辅助功能和截图读聊天、搜索会话 | vision-ui | 必须 | MCP | 参考：借做法自己写。MIT，可 fork。 |
| [halohazhang/wechat-favorites-to-obsidian](https://github.com/halohazhang/wechat-favorites-to-obsidian)（致谢） | 4 / 2026-07-20 / none / MIT | 微信聊天、公众号 | Claude Skill：把 Mac 微信收藏里的公众号文章批量导出为 Obsidian 笔记，每步人工确认 | vision-ui | 必须 | Agent Skill | 获取方式：借做法自己写，放第三方仓库。MIT，可 fork。 |
| [dustin573/wechat-mcp](https://github.com/dustin573/wechat-mcp)（致谢） | 2 / 2026-08-20 / none / MIT | 微信聊天 | 经 macOS 辅助功能读取并操作 Mac 微信 4.x 的 MCP 服务 | vision-ui | 必须 | MCP | 参考：借做法自己写。MIT，可 fork；回复、加好友、发朋友圈等写工具接入时关闭。 |
| [pengyulong/wechat-favorites-exporter](https://github.com/pengyulong/wechat-favorites-exporter)（致谢） | 1 / 2026-05-28 / none / MIT | 微信聊天、公众号 | 用 macOS 辅助功能驱动 Mac 微信收藏页，把收藏的公众号文章链接导出为 CSV | vision-ui | 必须 | 命令行 | 获取方式：fork 后改造，放第三方仓库。MIT，保留版权声明。 |
| [shibanyu333/wechat-chat-export](https://github.com/shibanyu333/wechat-chat-export)（致谢） | 1 / 2026-10-28 / v1.1.0（2026-09-21） / MIT | 微信聊天 | Mac 微信聊天记录导出：读屏、自动滚动截图、本机 OCR，导出 Word 或 Markdown | vision-ui | 必须 | 桌面应用 | 参考：借做法自己写。MIT，可 fork。 |
| [BBbinlei/wechat-sentence-excerpt](https://github.com/BBbinlei/wechat-sentence-excerpt)（致谢） | 0 / 2026-08-07 / none / none | 公众号 | 安卓无障碍服务：在手机微信读公众号时按句保存摘录，文字靠本机离线 OCR | vision-ui | 必须 | 手机应用 | 参考：借做法自己写。没有许可证，只参考做法。 |
| [Higurashi-kagome/wereader](https://github.com/Higurashi-kagome/wereader)（致谢） | 776 / 2025-08-31 / 5.3.0（2025-08-17） / MIT | 微信读书 | 微信读书网页版扩展，把划线、想法、书评导出为 Markdown | browser-session | 必须 | 浏览器扩展 | 参考：借做法自己写。MIT，可 fork。 |
| [mingjuner123-spec/wechat-inbox-sync](https://github.com/mingjuner123-spec/wechat-inbox-sync)（致谢） | 34 / 2026-10-29 / 1.3.214（2026-10-29） / NOASSERTION | 多平台：视频号、公众号、抖音、小红书、B站、播客、知乎、头条/西瓜、其他 | 「张张AI视界」的 Obsidian 插件：手机经小程序把链接存进作者云端收件箱，电脑端拉取、解析、转写成 Markdown（用户点名，见 [§3](#3-用户点名的项目)；[05 §1.1](05-note-apps-and-plugins.md#11-obsidian-社区插件)） | browser-session | 可选 | Obsidian 插件、其他 | 只作参考。1.3.117 起是 MIT 加 Commons Clause，只借设计。 |
| [wechat-article/wechat-article-exporter](https://github.com/wechat-article/wechat-article-exporter)（致谢） | 13116 / 2026-09-30 / 0.3（2025-03-19） / MIT | 公众号 | 自部署的公众号文章批量导出网站，靠公众号后台接口列文章；接口被关，2026-07-30 起停更（用户点名，见 [§3](#3-用户点名的项目)；[05 §1.2](05-note-apps-and-plugins.md#12-独立工具)） | session-api | 必须 | 网页应用、HTTP 接口、Docker | 获取方式：fork 后改造，放第三方仓库。MIT，保留版权声明；只取不登录的单篇管线；配套 wxdown-service 没有许可证，不复用。 |
| [qiye45/wechatDownload](https://github.com/qiye45/wechatDownload) | 9925 / 2026-10-07 / 4.7（2026-08-16） / none | 公众号 | 闭源的公众号文章批量下载桌面工具，附本地 MCP 和 Skill | session-api | 必须 | 桌面应用、MCP、Agent Skill | 不接入。闭源、没有许可证，有明文统计上报，不推荐。 |
| [cooderl/wewe-rss](https://github.com/cooderl/wewe-rss)（致谢） | 9634 / 2026-03-20 / release-20241215130232（2024-12-15） / MIT · 已归档 | 公众号、微信读书 | 借微信读书账号生成公众号 RSS 的自托管服务；已归档（[05 §1.2](05-note-apps-and-plugins.md#12-独立工具)） | session-api | 必须 | 网页应用、HTTP 接口、Docker | 只作参考。MIT；请求经第三方转发服务。 |
| [rachelos/we-mp-rss](https://github.com/rachelos/we-mp-rss)（致谢） | 5075 / 2026-10-20 / v1.5.3（2026-08-13） / NOASSERTION | 公众号、微信读书 | 自托管公众号订阅服务：定时抓文章、生成 RSS、导出多种格式，提供 API（[05 §1.2](05-note-apps-and-plugins.md#12-独立工具)） | session-api | 必须 | 网页应用、HTTP 接口、Docker | 获取方式：调它的 HTTP 接口，放第三方仓库。LICENSE 正文是 MIT；有 eval 漏洞报告，只在本机运行。 |
| [freestylefly/mcp-server-weread](https://github.com/freestylefly/mcp-server-weread) | 584 / 2025-05-18 / none / MIT | 微信读书 | 官方 Gateway 出现前的微信读书 MCP，用网页 Cookie 读书架和笔记 | session-api | 必须 | MCP | 只作参考。MIT。 |
| [MuZiJin701/wx-video-account-notes](https://github.com/MuZiJin701/wx-video-account-notes)（致谢） | 5 / 2026-10-28 / none / MIT | 视频号 | 视频号分享链接解析成媒体，本机 OCR、ASR 整理成笔记的 Skill；解析借元宝登录态 | session-api | 可选 | Agent Skill、HTTP 接口 | 获取加转换：借做法自己写，获取部分放第三方仓库。客户端 MIT，解析服务端受 Commons Clause 约束；默认服务走公网 HTTP。 |
| [tmwgsicp/wechat-download-api](https://github.com/tmwgsicp/wechat-download-api) | 1206 / 2026-07-27 / v1.7.0（2026-07-20） / AGPL-3.0 | 公众号 | 自托管公众号文章获取与 RSS 服务，HTTP API 与 MCP，支持整号导出 | private-api | 必须 | HTTP 接口、MCP、Docker | 获取方式：调它的 HTTP 接口，放第三方仓库。AGPL-3.0：用户自装，独立进程调用；2026-07-30 后有失效报告。 |
| [jackwener/wechat-article-to-markdown](https://github.com/jackwener/wechat-article-to-markdown)（致谢） | 1077 / 2026-03-22 / none / none | 公众号 | 用反检测浏览器抓单篇公众号文章转 Markdown，附 Agent Skill（[05 §1.2](05-note-apps-and-plugins.md#12-独立工具)） | private-api | 不用 | 命令行、Agent Skill、库 | 获取方式：包装它的命令行，放第三方仓库。仓库没有 LICENSE 文件（pyproject 写 MIT）：只以独立进程调用 PyPI 包。 |
| [wallechfox/wechat-mp-archive](https://github.com/wallechfox/wechat-mp-archive) | 9 / 2026-10-28 / v1.4（2026-10-28） / none | 公众号、微信读书 | 本机网页界面的公众号归档：备份自己的公众号，或借微信读书整号拉取 | private-api | 可选 | 网页应用 | 只作参考。没有许可证，只参考。 |
| [LC044/WeChatMsg](https://github.com/LC044/WeChatMsg) | 42302 / 2025-12-30 / none / none | 微信聊天 | 曾经最流行的微信聊天记录导出工具「留痕」，代码已从仓库移除 | client-hook | 必须 | 桌面应用 | 不接入。没有代码，也没有许可证。 |
| [hicccc77/WeFlow](https://github.com/hicccc77/WeFlow) | 15643 / 2026-10-28 / none / NOASSERTION | 微信聊天 | 本地微信聊天记录查看、分析与导出应用，解密组件已移除，改由用户自备 | client-hook | 必须 | 桌面应用、HTTP 接口 | 只作参考。CC BY-NC-SA 4.0，不复用代码；已被 DMCA 点名。 |
| [ltaoo/wx_channels_download](https://github.com/ltaoo/wx_channels_download)（致谢） | 10254 / 2026-10-29 / v260907（2026-09-07） / NOASSERTION | 多平台：视频号、公众号、B站、抖音、知乎、快手、小红书、微博、X、YouTube、Instagram | 视频号下载器扩成的通用归档工具：本机代理改写视频号页面，另有多平台适配器 | client-hook | 必须 | 命令行、HTTP 接口、MCP、网页应用 | 获取方式：调它的 HTTP 接口，放第三方仓库。MIT 加 Commons Clause：用户自装，第三方插件调它的本地 API。 |
| [xaoyaoo/PyWxDump](https://github.com/xaoyaoo/PyWxDump) | 9639 / 2025-10-20 / none / none | 微信聊天 | 读 PC 微信本地数据库的工具；2025-10-20 收函后删除全部代码 | client-hook | 必须 | 命令行、库 | 不接入。代码已删。 |
| [sjzar/chatlog](https://github.com/sjzar/chatlog) | 9180 / 2025-10-20 / none / none | 微信聊天 | 本地微信聊天记录查询工具，曾提供 HTTP API 和 MCP；2025-10-20 收函后移除代码 | client-hook | 必须 | 命令行、HTTP 接口、MCP | 不接入。代码已删。 |
| [lich0821/WeChatFerry](https://github.com/lich0821/WeChatFerry) | 6806 / 2026-07-10 / v39.5.2（2026-03-28） / MIT · 已归档 | 微信聊天 | 注入 PC 微信进程的机器人框架；已归档 | client-hook | 必须 | 库、HTTP 接口 | 相邻，不接入。MIT。 |
| [git-jiadong/wechatDataBackup](https://github.com/git-jiadong/wechatDataBackup) | 6421 / 2026-01-10 / none / Apache-2.0 | 微信聊天 | PC 微信聊天记录导出与浏览工具；2026-01 被 DMCA 点名后代码移除 | client-hook | 必须 | 桌面应用 | 不接入。Apache-2.0。 |
| [ycccccccy/echotrace](https://github.com/ycccccccy/echotrace) | 3856 / 2026-03-20 / 5.2.0（2026-01-18） / MIT | 微信聊天 | 微信 4.x PC 端聊天记录的本地导出与分析工具：解密本机数据库，导出 JSON、HTML、Excel，生成年度报告；2026-01-18 停更，默认分支只剩文档（用户点名，见 [§3](#3-用户点名的项目)） | client-hook | 必须 | 桌面应用、命令行 | 相邻，不接入。MIT；源码只在 fork 和旧安装包里，同作者的 wx_key 已被 DMCA 点名。 |
| [biggerstar/wedecode](https://github.com/biggerstar/wedecode) | 3349 / 2026-08-27 / none / GPL-3.0 | 小程序 | 微信小程序 wxapkg 包的反编译还原工具 | client-hook | 不用 | 命令行 | 相邻，不接入。GPL-3.0。 |
| [ttttupup/wxhelper](https://github.com/ttttupup/wxhelper) | 3192 / 2026-06-12 / 3.9.8.25-v2（2023-12-26） / MIT | 微信聊天 | 向 PC 微信注入 DLL 开放本机 HTTP 接口的逆向练习项目，只支持旧版本 | client-hook | 必须 | HTTP 接口、库 | 相邻，不接入。MIT。 |
| [nobiyou/wx_channel](https://github.com/nobiyou/wx_channel) | 2858 / 2026-10-28 / v5.7.13（2026-10-15） / MIT | 视频号 | Windows 视频号下载助手：本机代理注入按钮，支持收藏和点赞列表下载 | client-hook | 必须 | 桌面应用、HTTP 接口、网页应用 | 获取方式：调它的 HTTP 接口，放第三方仓库。MIT；和 ltaoo 早期代码的关系未核实，暂不 fork。 |
| [huohuoer/wechat-cli](https://github.com/huohuoer/wechat-cli) | 2684 / 2026-04-06 / none / Apache-2.0 | 微信聊天 | 面向大模型的本地微信数据命令行，可查聊天记录、联系人和收藏 | client-hook | 必须 | 命令行 | 不接入。Apache-2.0；依赖已被 DMCA 屏蔽。 |
| [ppwwyyxx/wechat-dump](https://github.com/ppwwyyxx/wechat-dump) | 2017 / 2026-02-23 / 0.2-wxgf（2025-01-29） / GPL-3.0 | 微信聊天 | 从已 root 的安卓手机读微信消息，导出为 HTML 并做统计 | client-hook | 必须 | 命令行 | 不接入。GPL-3.0。 |
| [jackwener/wx-cli-again](https://github.com/jackwener/wx-cli-again) | 886 / 2026-09-14 / none / Apache-2.0 | 微信聊天 | 读本机微信加密数据库的命令行，wx-cli 被 DMCA 屏蔽后重开的仓库 | client-hook | 必须 | 命令行、Agent Skill | 不接入。Apache-2.0；同类已被整网屏蔽，不推荐。 |
| [zhuyansen/wx-favorites-report](https://github.com/zhuyansen/wx-favorites-report) | 642 / 2026-10-19 / none / MIT | 微信聊天、公众号 | 把 Mac 微信收藏做成 HTML 报告的 Skill：提取密钥、解密本地库、解析收藏 | client-hook | 必须 | Agent Skill、命令行 | 只作参考。MIT；只参考收藏类型与字段，不用密钥提取部分。 |
| [joeseesun/qiaomu-wx-video](https://github.com/joeseesun/qiaomu-wx-video)（致谢） | 139 / 2026-08-05 / none / MIT | 视频号 | 下载视频号分享链接和直播回放的 Skill，按在线解析、本机捕获、浏览器捕获、录屏逐级降级 | client-hook | 可选 | Agent Skill | 获取方式：借做法自己写，放第三方仓库。MIT，可 fork。 |
| [open-wxa/wxa-runner](https://github.com/open-wxa/wxa-runner) | 631 / 2026-10-30 / v0.12.0（2026-10-30） / MIT | 小程序 | 在普通浏览器里用官方基础库运行自己的小程序编译产物，用于调试和 E2E 测试（用户点名，见 [§3](#3-用户点名的项目)） | not-applicable | 可选 | 命令行、库、网页应用 | 相邻，不接入。代码 MIT；内置的基础库是腾讯代码，不能分发。 |
| [dlv2008/weixin-favor-kb](https://github.com/dlv2008/weixin-favor-kb)（致谢） | 27 / 2026-04-12 / none / none | 视频号 | 把视频号收藏视频整理成 Obsidian 知识库的流水线：转写、关键帧 OCR、模型分类 | not-applicable | — | 命令行 | 转换插件：借做法自己写。没有许可证，只参考流程。 |

- **公众号：单篇不登录可取，批量列表的路断了。** wechat-article-exporter 的单篇管线、wechatmp2markdown、wexin-read-mcp 不用账号；批量列文章依赖的公众号后台接口 2026-07-30 关闭，we-mp-rss、wewe-rss 走的公众号后台或微信读书通道有频率控制和「小黑屋」，没有微信号被处罚的报告。
- **微信收藏：视觉层拿链接，公开页取正文。** PC 微信收藏可由 UI 自动化逐条复制文章链接（pywechat、wechat-favorites-exporter、wechat-favorites-to-obsidian），再交给公开页取正文，这是目前最完整的混合路径；合并转发导出（WeChatBridge）走的是微信官方功能。代价是微信对 UI 自动化有强制下线记录（[§1.2](#12-视觉层的风险从哪里来)）。
- **本地数据库解密是下架重灾区。** 腾讯 2026 年发了 37 份 DMCA 通知，其中 10 份按派生网络处理，一月那份覆盖 4,195 个仓库；PyWxDump、chatlog、WeChatMsg 的代码已删，wx-cli 被屏蔽后又有人重开。歇存不接入这一类。
- **视频号和微信读书。** 视频号不装证书、不注入的路只有借腾讯元宝登录态换凭证（qiaomu-clipper、media-parser、wx-video-account-notes），账号风险转到元宝账号；其余（wx_channels_download、wx_channel、res-downloader）是本机代理改写页面。微信读书有官方的个人 API Key（Tencent/WeChatReading），是这一节唯一的官方接口，但书架里的文章收藏只返回入口。

<a id="9-微博知乎豆瓣与海外社交平台"></a>
## 9. 微博、知乎、豆瓣与海外社交平台

| 项目 | ★ / 推送 / release / 许可证 | 平台 | 做什么 | 层级 | 登录 | 接口 | 歇存怎么用 |
|---|---|---|---|---|---|---|---|
| [Serene-Arc/bulk-downloader-for-reddit](https://github.com/Serene-Arc/bulk-downloader-for-reddit)（致谢） | 2618 / 2026-04-12 / v2.6.2（2023-01-31） / GPL-3.0 | Reddit | Reddit 批量下载与归档，可按收藏、点赞下载媒体，或把帖子和评论存成 JSON | official-export | 必须 | 命令行 | 获取方式：包装它的命令行，放第三方仓库。GPL-3.0：用户自装，独立进程调用。 |
| [timhutton/twitter-archive-parser](https://github.com/timhutton/twitter-archive-parser)（致谢） | 2436 / 2022-11-29 / none / GPL-3.0 | X | 把 X 官方归档里的推文和私信转成 Markdown 与 HTML，还原短链 | official-export | 不用 | 命令行 | 获取方式：借做法自己写，放第三方仓库。GPL-3.0：不拷代码，按公开格式自己实现。 |
| [steipete/birdclaw](https://github.com/steipete/birdclaw)（致谢） | 1673 / 2026-10-24 / v0.16.3（2026-10-24） / MIT | X | 把 X 官方归档导入本地 SQLite，提供搜索、Web 界面和只读 MCP，可经官方命令行增量同步 | official-export | 可选 | 命令行、网页应用、MCP | 获取方式：包装它的命令行，放第三方仓库。MIT，可 fork。 |
| [xdevplatform/xurl](https://github.com/xdevplatform/xurl)（致谢） | 1364 / 2026-09-29 / v1.3.4（2026-09-29） / MIT | X | X 官方的 API 命令行，带书签、点赞快捷命令，可桥接官方托管的 MCP | official-export | 必须 | 命令行、MCP、Agent Skill | 获取方式：包装它的命令行，放第三方仓库。MIT，可 fork；要开发者账号并按量付费。 |
| [knadh/tg-archive](https://github.com/knadh/tg-archive)（致谢） | 1173 / 2026-03-01 / v1.3.0（2025-05-01） / MIT | Telegram | 定期把 Telegram 群组消息同步到本地 SQLite，再生成静态归档网站 | official-export | 必须 | 命令行 | 获取方式：fork 后改造，放第三方仓库。MIT，保留版权声明；可改成取收藏（Saved Messages）。 |
| [soberhacker/obsidian-telegram-sync](https://github.com/soberhacker/obsidian-telegram-sync)（致谢） | 686 / 2024-10-30 / 4.0.0（2024-10-30） / AGPL-3.0 | Telegram | Obsidian 插件：把发给自建 Telegram 机器人的消息和文件写进笔记库 | official-export | 可选 | Obsidian 插件 | 只作参考。AGPL-3.0；借「转发给机器人即入库」的做法。 |
| [dogsheep/twitter-to-sqlite](https://github.com/dogsheep/twitter-to-sqlite)（致谢） | 417 / 2022-12-28 / 0.22（2021-09-21） / Apache-2.0 | X | 把 X 官方归档导入 SQLite，另有调官方 API 的命令 | official-export | 可选 | 命令行 | 获取方式：借做法自己写，放第三方仓库。Apache-2.0，可 fork；API 命令在 X 改为付费后基本不可用。 |
| [pinterest/api-quickstart](https://github.com/pinterest/api-quickstart) | 212 / 2026-04-13 / v2.0（2022-11-01） / Apache-2.0 | Pinterest | Pinterest 官方 API v5 示例，含 OAuth 和读取自己画板、Pin 的脚本 | official-export | 必须 | 命令行、库 | 参考：借做法自己写。Apache-2.0，可 fork。 |
| [karlicoss/rexport](https://github.com/karlicoss/rexport)（致谢） | 188 / 2026-10-24 / none / MIT | Reddit | 把自己的 Reddit 收藏、点赞、发帖和评论导出为 JSON | official-export | 必须 | 命令行、库 | 获取方式：包装它的命令行，放第三方仓库。MIT，可 fork；列表接口每类最多约 1000 条。 |
| [klarkxy/zhihu-search](https://github.com/klarkxy/zhihu-search)（致谢） | 24 / 2026-10-13 / v3.0.0（2026-10-13） / NOASSERTION | 知乎 | 把知乎开放平台官方接口封装成 Skill、命令行和 MCP，含本人收藏与收藏夹 | official-export | 必须 | Agent Skill、命令行、MCP、HTTP 接口 | 获取方式：借做法自己写，放第三方仓库。SATA 2.0（MIT 加致谢要求）；可按知乎官方文档自己实现。 |
| [wangcch/weibo-mcp](https://github.com/wangcch/weibo-mcp) | 1 / 2026-04-13 / none / MIT | 微博 | 接微博开放接口的 MCP：搜索、分类热搜、当前用户自己的微博 | official-export | 必须 | MCP | 参考：借做法自己写。MIT，可 fork；接口范围不含收藏。 |
| [meizuflux/scrollback](https://github.com/meizuflux/scrollback)（致谢） | 0 / 2026-09-21 / none / MIT | Instagram | 纯浏览器端的 Instagram 导出分析器，可导出为 SQLite | official-export | 不用 | 网页应用 | 获取方式：借做法自己写，放第三方仓库。MIT，可 fork。 |
| [dataabc/weibo-crawler](https://github.com/dataabc/weibo-crawler)（致谢） | 4682 / 2026-10-26 / none / none | 微博 | 按用户 ID 抓微博资料和内容，写 CSV、JSON 或数据库；不填 cookie 也能取多数公开微博 | public-web | 可选 | 命令行、HTTP 接口、Docker | 获取方式：包装它的命令行，放第三方仓库。没有许可证：只能用户自装后调用，插件仓库不放它的代码。 |
| [xavdid/reddit-user-to-sqlite](https://github.com/xavdid/reddit-user-to-sqlite)（致谢） | 230 / 2023-07-24 / v0.4.2（2023-07-23） / MIT | Reddit | 把本人 Reddit 帖子和评论存进 SQLite，也能读官方数据请求导出的 CSV | public-web | 不用 | 命令行 | 获取方式：借做法自己写，放第三方仓库。MIT，可 fork。 |
| [sean1832/pinterest-dl](https://github.com/sean1832/pinterest-dl)（致谢） | 211 / 2026-07-14 / 1.3.0（2026-06-15） / Apache-2.0 | Pinterest | 从 Pinterest 画板、Pin、搜索结果下载图片和视频的命令行与库 | public-web | 可选 | 命令行、库 | 获取方式：包装它的命令行，放第三方仓库。Apache-2.0，可 fork。 |
| [Geralt-TYH/obsidian-zhihu-crawler](https://github.com/Geralt-TYH/obsidian-zhihu-crawler)（致谢） | 98 / 2025-01-08 / v1.1（2022-05-12） / none | 知乎 | 把知乎收藏夹里的回答和文章逐条转成 Markdown，图片存到本地 | public-web | 可选 | 命令行、桌面应用 | 获取方式：借做法自己写，放第三方仓库。没有许可证，只能照做法自己重写。 |
| [qinyuanpei/mcp-server-weibo](https://github.com/qinyuanpei/mcp-server-weibo) | 64 / 2026-09-16 / none / MIT | 微博 | 微博 MCP 与命令行：用户资料、微博、热搜、评论、话题和关注关系 | public-web | 可选 | MCP、命令行、Agent Skill | 获取方式：借做法自己写，放第三方仓库。MIT，可 fork；不参考按 UID 生成「人格报告」的 Skill。 |
| [cosformula/douban-sync-skill](https://github.com/cosformula/douban-sync-skill)（致谢） | 33 / 2026-02-15 / v0.2.2（2026-02-15） / MIT | 豆瓣 | 导出并增量同步本人豆瓣标记到 CSV 的 Skill，日常增量走公开 RSS | public-web | 可选 | Agent Skill、命令行 | 获取方式：fork 后改造，放第三方仓库。MIT，保留版权声明。 |
| [prinsss/twitter-web-exporter](https://github.com/prinsss/twitter-web-exporter)（致谢） | 2786 / 2026-08-31 / v1.4.3（2026-08-31） / MIT | X | 油猴脚本：在 X 网页版导出推文、书签、点赞、列表等为 JSON、CSV、HTML，并批量下载媒体 | browser-session | 必须 | 浏览器扩展 | 获取方式：借做法自己写，放第三方仓库。MIT，可 fork；只截获页面自己的请求。 |
| [moonlin1213/cove-douban-mcp](https://github.com/moonlin1213/cove-douban-mcp)（致谢） | 15 / 2026-08-29 / v0.2.0（2026-08-29） / Apache-2.0 | 豆瓣 | 默认只读的豆瓣 MCP，经 OpenCLI 读用户 Chrome 里本来能看到的豆瓣页面 | browser-session | 必须 | MCP、命令行 | 获取方式：借做法自己写，放第三方仓库。Apache-2.0，可 fork。 |
| [jasonsbmoc/gramcracker](https://github.com/jasonsbmoc/gramcracker) | 0 / 2026-06-12 / v0.3.6（2026-06-12） / MIT | Instagram | Chrome 扩展：把 Instagram 收藏里的图片按规则命名，打包成 zip | browser-session | 必须 | 浏览器扩展 | 获取方式：借做法自己写，放第三方仓库。MIT，可 fork。 |
| [dataabc/weiboSpider](https://github.com/dataabc/weiboSpider)（致谢） | 9744 / 2026-02-04 / none / none | 微博 | 抓指定微博用户的资料和全部微博，写入文件或数据库，必须提供登录 cookie | session-api | 必须 | 命令行 | 获取方式：包装它的命令行，放第三方仓库。没有许可证：只能用户自装后调用。 |
| [afar1/fieldtheory-cli](https://github.com/afar1/fieldtheory-cli)（致谢） | 2053 / 2026-09-05 / none / MIT | X | 把自己的 X 书签同步到本地：全文搜索、导出 Markdown、模型分类，给 Agent 当知识库 | session-api | 必须 | 命令行、Agent Skill | 获取加转换：fork 后改造，获取部分放第三方仓库。MIT，保留版权声明。 |
| [alexknowshtml/smaug](https://github.com/alexknowshtml/smaug)（致谢） | 950 / 2026-02-03 / v0.3.1（2026-01-06） / MIT | X | 把 X 书签按日期归档成 Markdown，展开链接，交给 Claude Code 或 OpenCode 分类 | session-api | 必须 | 命令行 | 获取加转换：借做法自己写，获取部分放第三方仓库。MIT，可 fork；依赖的 bird 已无法获取，基本停摆。 |
| [doufen-org/tofu](https://github.com/doufen-org/tofu)（致谢） | 627 / 2025-03-25 / none / MIT | 豆瓣 | Chrome 扩展：把豆瓣书影音标记、评论、日记、豆列等备份到本地，可导出 Excel | session-api | 必须 | 浏览器扩展 | 获取方式：借做法自己写，放第三方仓库。MIT，可 fork；2026-03 起无法备份，已停更。 |
| [Chilfish/Weibo-archiver](https://github.com/Chilfish/Weibo-archiver)（致谢） | 490 / 2026-10-03 / v0.6.2（2025-06-28） / Apache-2.0 | 微博 | 油猴脚本加命令行，把自己的微博、关注和微博收藏导出为 JSON，配套网页离线浏览 | session-api | 必须 | 浏览器扩展、命令行、网页应用 | 获取方式：fork 后改造，放第三方仓库。Apache-2.0，保留版权声明；配套网页默认经作者的图片代理加载，要改走本地。 |
| [zimya/zhihu_obsidian](https://github.com/zimya/zhihu_obsidian)（致谢） | 287 / 2026-10-17 / 0.4.3（2026-10-15） / 0BSD | 知乎 | Obsidian 插件，主要把笔记发到知乎，也能把推荐、关注动态和热榜拉进笔记库 | session-api | 必须 | Obsidian 插件 | 参考：借做法自己写。0BSD，可 fork。 |
| [Wanxp/obsidian-douban](https://github.com/Wanxp/obsidian-douban)（致谢） | 274 / 2026-08-17 / 2.5.0（2026-08-12） / GPL-3.0 | 豆瓣 | Obsidian 插件：按条目导入豆瓣内容，同步自己标记过的书影音 | session-api | 可选 | Obsidian 插件 | 只作参考。GPL-3.0；借「验证页交给用户在窗口里完成」的做法。 |
| [Marvae/douban-cli](https://github.com/Marvae/douban-cli)（致谢） | 201 / 2026-10-30 / v0.7.0（2026-10-30） / MIT | 豆瓣 | 豆瓣命令行加 Skill：查条目，把自己的看过、想看等记录导出为 JSON、CSV、Markdown | session-api | 可选 | 命令行、Agent Skill | 获取方式：fork 后改造，放第三方仓库。MIT，保留版权声明；只开放导出和查询。 |
| [sytelus/xarchive](https://github.com/sytelus/xarchive)（致谢） | 119 / 2026-09-05 / none / MIT | X | Chrome 扩展：导出自己全部 X 书签并保留书签文件夹，附离线查看器 | session-api | 必须 | 浏览器扩展 | 获取方式：借做法自己写，放第三方仓库。MIT，可 fork。 |
| [soyaine/jocker-chrome-extension](https://github.com/soyaine/jocker-chrome-extension)（致谢） | 89 / 2025-08-08 / 3.1.4（2025-08-08） / MIT | 即刻 | Chrome 扩展：按圈子、年份、标签浏览自己的即刻动态，可看收藏并导出 CSV | session-api | 必须 | 浏览器扩展 | 参考：借做法自己写。MIT，可 fork。 |
| [handsomestWei/zhihu-fetch-skill](https://github.com/handsomestWei/zhihu-fetch-skill)（致谢） | 36 / 2026-09-20 / none / MIT | 知乎 | 抓知乎收藏夹、点赞与收藏动态、专栏和回答的 Skill，支持断点续传 | session-api | 可选 | Agent Skill、命令行 | 获取方式：fork 后改造，放第三方仓库。MIT，保留版权声明；Cookie 明文存在本地。 |
| [jackjin1997/douban-mcp](https://github.com/jackjin1997/douban-mcp) | 14 / 2026-07-25 / v1.0.0（2026-05-10） / MIT | 豆瓣 | 豆瓣 MCP 加命令行：查电影、图书，读自己的想看、在看、看过和豆列 | session-api | 可选 | MCP、命令行 | 获取方式：经 MCP 调用，放第三方仓库。MIT，可 fork。 |
| [Tendo33/zhihu-md](https://github.com/Tendo33/zhihu-md)（致谢） | 14 / 2026-10-30 / v1.1.9（2026-10-30） / MIT | 知乎 | Chrome 扩展：把当前打开的知乎文章、回答、问题页等导出为带 YAML 头的 Markdown | session-api | 可选 | 浏览器扩展 | 获取方式：借做法自己写，放第三方仓库。MIT，可 fork；知乎 HTML 转 Markdown 的代码可复用。 |
| [wujieli0207/jike-export-extension](https://github.com/wujieli0207/jike-export-extension) | 9 / 2026-03-01 / none / none | 即刻 | 浏览器扩展：把自己或即友的即刻动态和收藏导出为 Markdown、Excel 等 | session-api | 必须 | 浏览器扩展 | 只作参考。没有许可证，是收费扩展的源码，只参考。 |
| [tairqaldy/resurfly](https://github.com/tairqaldy/resurfly)（致谢） | 0 / 2026-09-06 / companion-v0.1.1（2026-08-16） / MIT | Instagram | 把 Instagram 收藏变成可搜索笔记：转写 Reels、读关键帧，做摘要和每日回顾 | session-api | 必须 | 网页应用、浏览器扩展、Docker | 获取加转换：借做法自己写，获取部分放第三方仓库。MIT，可 fork；不用 DM 自动化；分析默认交给 OpenAI。 |
| [instaloader/instaloader](https://github.com/instaloader/instaloader)（致谢） | 13728 / 2026-10-26 / v4.16（2026-10-26） / MIT | Instagram | 下载 Instagram 帖子、快拍、文案和元数据，登录后能下载自己的收藏 | private-api | 必须 | 命令行、库 | 获取方式：包装它的命令行，放第三方仓库。MIT，可 fork；包装时默认加 --no-iphone。 |
| [subzeroid/instagrapi](https://github.com/subzeroid/instagrapi) | 7041 / 2026-10-28 / 3.2.2（2026-10-27） / NOASSERTION | Instagram | Instagram 私有接口的 Python 库，覆盖私信、上传等自动化操作 | private-api | 必须 | 库 | 不接入。LICENSE 正文是 MIT。 |
| [zly2006/zhihu-plus-plus](https://github.com/zly2006/zhihu-plus-plus) | 4387 / 2026-10-28 / 0.31.1（2026-10-23） / AGPL-3.0 | 知乎 | 第三方知乎客户端，去广告、模型过滤内容，可导出回答和整个收藏夹 | private-api | 可选 | 手机应用、桌面应用 | 只作参考。AGPL-3.0。 |
| [public-clis/twitter-cli](https://github.com/public-clis/twitter-cli)（致谢） | 3009 / 2026-05-07 / none / Apache-2.0 | X | X 终端命令行：读时间线、书签、帖子和文章，也能发帖、点赞，输出 YAML 或 JSON | private-api | 必须 | 命令行、Agent Skill | 获取方式：包装它的命令行，放第三方仓库。Apache-2.0，可 fork；有封号报告，只开放读取。 |
| [vladkens/twscrape](https://github.com/vladkens/twscrape) | 2911 / 2026-10-30 / v0.20.1（2026-08-25） / MIT | X | X 的抓取库和命令行，用多账号池轮换应对限流 | private-api | 必须 | 命令行、库 | 不接入。MIT。 |
| [junhoyeo/threads-api](https://github.com/junhoyeo/threads-api) | 1618 / 2023-09-08 / the-end（2023-09-08） / MIT · 已归档 | 其他 | Threads 的非官方 Node.js 客户端；收到 Meta 来函后停止并归档 | private-api | 可选 | 库 | 不接入。MIT。 |
| [lumina37/aiotieba](https://github.com/lumina37/aiotieba) | 716 / 2026-10-30 / v4.8.2（2026-10-25） / Unlicense | 贴吧 | 百度贴吧的 Python 异步接口库，覆盖读帖、用户信息和吧务管理 | private-api | 可选 | 库 | 只作参考。Unlicense。 |
| [lizheming/doumark-action](https://github.com/lizheming/doumark-action) | 105 / 2023-12-31 / 0.2.0（2023-07-22） / MIT | 豆瓣 | GitHub Action：定期把豆瓣用户的公开标记同步成 CSV、JSON，或写入 Notion、NeoDB | private-api | 可选 | Docker、其他 | 获取方式：借做法自己写，放第三方仓库。MIT，可 fork；依赖小程序 apiKey，2023-12 后停更。 |
| [open-jike/jike-sdk](https://github.com/open-jike/jike-sdk) | 65 / 2026-05-05 / v0.28.1（2025-08-23） / AGPL-3.0 | 即刻 | 即刻的第三方 SDK，提供登录、动态、评论、通知等接口；不再积极维护 | private-api | 必须 | 库 | 只作参考。AGPL-3.0。 |
| [Doubak/doubak-extension](https://github.com/Doubak/doubak-extension)（致谢） | 35 / 2026-10-30 / v1.6.0（2026-10-18） / Apache-2.0 | 豆瓣 | 浏览器扩展：限速抓自己的豆瓣网页存成 WARC，再离线转结构化数据或 Markdown 站点 | private-api | 可选 | 浏览器扩展 | 获取方式：借做法自己写，放第三方仓库。Apache-2.0，可 fork；只开放备份本人账号；自动求解算力验证是自动化特征。 |
| [KrisTHL181/zhihu-cli](https://github.com/KrisTHL181/zhihu-cli) | 29 / 2026-10-30 / v0.4.0（2026-10-30） / MIT | 知乎 | 知乎命令行客户端：浏览、搜索、存 Markdown，也能发布和互动，附 MCP | private-api | 必须 | 命令行、MCP、库 | 获取方式：包装它的命令行，放第三方仓库。MIT，可 fork；只开放读取类命令。 |
| [dotWee/py-ig2raindrop-cli](https://github.com/dotWee/py-ig2raindrop-cli)（致谢） | 1 / 2026-10-23 / v1.0.1（2026-04-21） / WTFPL | Instagram | 把 Instagram 收藏同步到 Raindrop.io，也能导入官方 saved_posts.json | private-api | 必须 | 命令行、Docker | 获取方式：借做法自己写，放第三方仓库。WTFPL，可 fork；只借导出解析那一路。 |
| [iyear/tdl](https://github.com/iyear/tdl)（致谢） | 8228 / 2026-10-26 / v0.20.4（2026-08-23） / AGPL-3.0 | Telegram | Go 写的 Telegram 工具箱：下载会话里的文件，把消息导出为 JSON | client-hook | 必须 | 命令行 | 获取方式：包装它的命令行，放第三方仓库。AGPL-3.0：用户自装，独立进程调用；用 -T qr 新建会话，不借官方桌面端身份。 |

- **这些平台大多有零或低风险的入口。** X 官方归档和 xurl、Reddit 官方接口（rexport、bulk-downloader-for-reddit）、Telegram 官方接口（tg-archive）、Pinterest 官方 API、微博开放接口（不含收藏）、知乎开放平台（含本人收藏，按额度计费）、豆瓣公开 RSS。
- **用户自己浏览器里截获页面请求的脚本适合取书签和收藏。** twitter-web-exporter、Weibo-archiver 的油猴版没有使用者被锁的报告；连续翻页的扩展（xarchive、tofu、jike-export-extension）要注意频率。
- **复现签名或模拟 App 的都有封号或来函记录。** twscrape、twitter-cli、instagrapi 只记录不适配；instaloader 默认模拟 App，包装时加 `--no-iphone` 降到 `session-api`；X 对 Nitter、Meta 对 threads-api 都发过停止侵权函。
- **即刻、贴吧只有 SDK 级的私有接口和少量导出扩展。** 即刻另一个工具的作者因平台态度不明自行停更；贴吧的 aiotieba 同时带吧务写操作。

<a id="10-视觉与-ui-自动化"></a>
## 10. 视觉与 UI 自动化

| 项目 | ★ / 推送 / release / 许可证 | 平台 | 做什么 | 层级 | 登录 | 接口 | 歇存怎么用 |
|---|---|---|---|---|---|---|---|
| [Genymobile/scrcpy](https://github.com/Genymobile/scrcpy)（致谢） | 153578 / 2026-10-26 / v5.0.1（2026-10-08） / Apache-2.0 | 其他 | 经 ADB 把 Android 的画面和声音投到电脑，可键鼠操作、可录屏，手机端不装 App | vision-ui | 可选 | 命令行、桌面应用 | 获取方式：包装它的命令行，放第三方仓库。Apache-2.0，可 fork。 |
| [zai-org/Open-AutoGLM](https://github.com/zai-org/Open-AutoGLM)（致谢） | 26509 / 2026-03-06 / none / Apache-2.0 | 多平台：微信聊天、抖音、小红书、B站、知乎、豆瓣、微博、其他 | 智谱的手机 Agent 框架和 9B 视觉模型：截图理解界面，经 ADB、HDC 或 WDA 点按 | vision-ui | 可选 | 命令行、库、模型 | 获取方式：fork 后改造，放第三方仓库。Apache-2.0，保留版权声明；ADB Keyboard 是 GPL-2.0，要用户自装。 |
| [microsoft/OmniParser](https://github.com/microsoft/OmniParser) | 25594 / 2026-07-20 / v.2.0.1（2025-09-12） / CC-BY-4.0 | 其他 | 微软的纯视觉屏幕解析：把截图拆成可交互元素，供只看截图的 Agent 定位 | vision-ui | — | 模型、库 | 获取方式：调它的 HTTP 接口，放第三方仓库。仓库 CC-BY-4.0；v3 检测器权重 MIT，旧检测器是 AGPL-3.0。 |
| [screenpipe/screenpipe](https://github.com/screenpipe/screenpipe)（致谢） | 22174 / 2026-10-30 / app-v2.7.130（2026-10-28） / NOASSERTION | 其他 | 持续记录屏幕和音频的本地「电脑记忆」：读无障碍树或 OCR，本地转写，可搜索 | vision-ui | 不用 | 桌面应用、命令行、HTTP 接口、MCP、Agent Skill | 获取加转换：借做法自己写，获取部分放第三方仓库。Screenpipe 商业许可（源码可见）：只借做法或读它的接口。 |
| [appium/appium](https://github.com/appium/appium) | 22166 / 2026-10-29 / @appium/opencv@5.0.2（2026-10-27） / Apache-2.0 | 其他 | 基于 W3C WebDriver 协议的跨平台 App 自动化框架 | vision-ui | 可选 | 命令行、HTTP 接口、库 | 获取方式：调它的 HTTP 接口，放第三方仓库。Apache-2.0，可 fork。 |
| [ExistentialAudio/BlackHole](https://github.com/ExistentialAudio/BlackHole) | 19987 / 2026-10-28 / v0.7.1（2026-07-03） / NOASSERTION | — | macOS 虚拟音频回环驱动，把一个 App 的声音送进另一个 App | vision-ui | 不用 | 其他 | 相邻，只作参考。源码 GPL-3.0 加名称限制：只让用户自装。 |
| [mobile-dev-inc/Maestro](https://github.com/mobile-dev-inc/Maestro)（致谢） | 16271 / 2026-10-28 / cli-2.12.0（2026-10-26） / Apache-2.0 | 其他 | 用 YAML 写移动端流程的测试框架，经系统无障碍层驱动 App，内置 MCP | vision-ui | 可选 | 命令行、MCP | 获取方式：包装它的命令行，放第三方仓库。Apache-2.0，可 fork；iOS 只支持模拟器。 |
| [CursorTouch/Windows-MCP](https://github.com/CursorTouch/Windows-MCP) | 10770 / 2026-10-29 / v0.8.8（2026-10-10） / MIT | 其他 | Windows 上的 computer use MCP，靠 UI Automation 读控件并操作键鼠 | vision-ui | 可选 | MCP | 获取方式：经 MCP 调用，放第三方仓库。MIT，可 fork；默认开遥测，包装时设 ANONYMIZED_TELEMETRY=false。 |
| [mobile-next/mobile-mcp](https://github.com/mobile-next/mobile-mcp)（致谢） | 10144 / 2026-10-30 / 1.0.14（2026-10-26） / Apache-2.0 | 其他 | 面向 iOS、Android 真机与模拟器的 MCP，优先读无障碍树，必要时退回截图 | vision-ui | 可选 | MCP | 获取方式：经 MCP 调用，放第三方仓库。Apache-2.0，可 fork；默认发匿名统计，可关闭；底层 mobilecli 的许可另看。 |
| [droidrun/mobilerun](https://github.com/droidrun/mobilerun) | 9879 / 2026-10-28 / v0.7.4（2026-10-27） / MIT | 其他 | 用自然语言驱动 Android、iOS 的 Agent 框架，Android 端装 Portal 应用提供控件树 | vision-ui | 可选 | 命令行、库 | 获取方式：包装它的命令行，放第三方仓库。MIT，可 fork；默认开匿名遥测，包装时关闭。 |
| [X-PLUG/MobileAgent](https://github.com/X-PLUG/MobileAgent) | 9344 / 2026-10-28 / none / MIT | 其他 | 阿里通义的 GUI Agent 系列和 GUI-Owl 模型，以截图为输入操作手机和电脑 | vision-ui | 可选 | 库、模型 | 只作参考。MIT。 |
| [lihaoyun6/QuickRecorder](https://github.com/lihaoyun6/QuickRecorder) | 8763 / 2025-06-11 / 1.6.9（2025-06-11） / AGPL-3.0 | — | 基于 ScreenCaptureKit 的 macOS 录屏工具，可录经数据线连接的 iPhone | vision-ui | 不用 | 桌面应用 | 相邻，只作参考。AGPL-3.0。 |
| [openatx/uiautomator2](https://github.com/openatx/uiautomator2)（致谢） | 8458 / 2026-10-23 / 3.7.1（2026-10-23） / MIT | 其他 | Python 库：经 ADB 在 Android 上跑 UiAutomator，读控件树、截图、点击和输入 | vision-ui | 可选 | 库 | 获取方式：包装它的命令行，放第三方仓库。MIT，可 fork。 |
| [TencentQQGYLab/AppAgent](https://github.com/TencentQQGYLab/AppAgent) | 6913 / 2025-03-19 / none / MIT | 其他 | 腾讯 2023 年的多模态手机 Agent 研究：先探索或看演示生成操作文档，再按文档执行 | vision-ui | 可选 | 命令行 | 参考：借做法自己写。MIT，可 fork。 |
| [SuperMonster003/AutoJs6](https://github.com/SuperMonster003/AutoJs6) | 6550 / 2026-03-16 / v6.7.0（2026-03-14） / MPL-2.0 | 其他 | Android 上的 JavaScript 自动化工具，靠无障碍服务读控件和点按，带 OCR 插件 | vision-ui | 可选 | 手机应用 | 获取方式：借做法自己写，放第三方仓库。MPL-2.0：改动的文件要公开。 |
| [openclaw/Peekaboo](https://github.com/openclaw/Peekaboo)（致谢） | 5415 / 2026-10-27 / v4.9.7（2026-10-23） / MIT | 其他 | macOS 命令行与菜单栏应用：截取窗口、读无障碍树、执行原生 UI 自动化，附 MCP | vision-ui | 可选 | 命令行、MCP、桌面应用 | 获取方式：包装它的命令行，放第三方仓库。MIT，可 fork。 |
| [meta/idb](https://github.com/meta/idb) | 5398 / 2026-10-30 / v1.7.7（2026-10-22） / MIT | 其他 | Meta 的 iOS 自动化命令行，查询类功能只支持模拟器 | vision-ui | 可选 | 命令行、库 | 相邻，只作参考。MIT。 |
| [callstack/agent-device](https://github.com/callstack/agent-device) | 5199 / 2026-10-30 / v0.21.35（2026-10-29） / MIT | 其他 | 命令行、MCP 和 Node API，基于无障碍快照操作 iOS、Android、鸿蒙设备 | vision-ui | 可选 | 命令行、MCP、库 | 获取方式：包装它的命令行，放第三方仓库。MIT，可 fork。 |
| [MaaXYZ/MaaFramework](https://github.com/MaaXYZ/MaaFramework) | 5108 / 2026-10-29 / v5.14.3（2026-10-10） / LGPL-3.0 | 其他 | 基于图像识别的黑盒自动化框架：截图、模板匹配、OCR 后点按，流程用 JSON 描述 | vision-ui | 可选 | 库 | 参考：借做法自己写。LGPL-3.0：用户自装，独立进程调用。 |
| [yuka-friends/Windrecorder](https://github.com/yuka-friends/Windrecorder) | 3966 / 2026-09-25 / none / GPL-2.0 | 其他 | Windows 屏幕记忆工具：定时截图或录屏，画面变化时 OCR 并建索引 | vision-ui | 不用 | 网页应用、Agent Skill | 获取加转换：借做法自己写，获取部分放第三方仓库。GPL-2.0：用户自装，独立进程调用；它的 WeChatOCR 扩展不应采用。 |
| [minitap-ai/mobile-use](https://github.com/minitap-ai/mobile-use) | 3340 / 2026-10-18 / v3.3.0（2026-01-12） / Apache-2.0 | 其他 | 用自然语言操作 Android、iOS 的开源 Agent，可从任意 App 抽取信息输出 JSON | vision-ui | 可选 | 命令行、库 | 获取方式：包装它的命令行，放第三方仓库。Apache-2.0，可 fork。 |
| [openrecall/openrecall](https://github.com/openrecall/openrecall) | 2967 / 2025-09-24 / none / AGPL-3.0 | 其他 | 开源 Recall 替代品：定时截屏，本地 OCR 和语义索引 | vision-ui | 不用 | 网页应用 | 相邻：借做法自己写。AGPL-3.0：只借思路；维护停滞。 |
| [doronz88/pymobiledevice3](https://github.com/doronz88/pymobiledevice3) | 2944 / 2026-10-30 / v11.49.0（2026-10-30） / GPL-3.0 | 其他 | 纯 Python 的 iOS 设备通信库和命令行，含 iOS 17+ 的开发者工具 | vision-ui | 可选 | 命令行、库 | 相邻：包装它的命令行。GPL-3.0：用户自装，独立进程调用。 |
| [danielpaulus/go-ios](https://github.com/danielpaulus/go-ios)（致谢） | 2305 / 2026-10-30 / v1.3.2（2026-08-11） / MIT | 其他 | 纯 Go 的 iOS 设备工具集，在三大桌面系统上截图、拉起 WDA、做无障碍检查 | vision-ui | 可选 | 命令行、库、HTTP 接口 | 获取方式：包装它的命令行，放第三方仓库。MIT，可 fork；和歇存的 Go 技术栈一致。 |
| [stepfun-ai/gelab-zero](https://github.com/stepfun-ai/gelab-zero) | 2291 / 2026-05-11 / none / MIT | 其他 | 阶跃星辰的手机 GUI Agent：4B 模型加本地运行设施，可完全本地部署 | vision-ui | 可选 | 模型、MCP、命令行 | 获取方式：经 MCP 调用，放第三方仓库。MIT，可 fork。 |
| [appium/WebDriverAgent](https://github.com/appium/WebDriverAgent)（致谢） | 1938 / 2026-10-29 / v16.15.3（2026-10-16） / NOASSERTION | 其他 | iOS 上的 WebDriver 服务端，以 XCTest 方式运行，提供截图、控件树、点按和输入 | vision-ui | 可选 | HTTP 接口 | 获取方式：调它的 HTTP 接口，放第三方仓库。LICENSE 是 BSD 3-Clause 文本，可再分发。 |
| [IPADS-SAI/MobiAgent](https://github.com/IPADS-SAI/MobiAgent) | 1932 / 2026-07-17 / v1.0.1（2025-12-22） / Apache-2.0 | 其他 | 上海交大 IPADS 的手机 GUI Agent 全套，能把成功的操作序列录下来复用 | vision-ui | 可选 | 命令行、模型、库 | 参考：借做法自己写。Apache-2.0，可 fork。 |
| [OpenBMB/AgentCPM-GUI](https://github.com/OpenBMB/AgentCPM-GUI) | 1428 / 2026-01-11 / none / Apache-2.0 | 多平台：B站、小红书 | 端侧 GUI Agent 模型，针对中文 App 微调，附中文 App 评测集 CAGUI | vision-ui | — | 模型 | 获取方式：调它的 HTTP 接口，放第三方仓库。Apache-2.0，可 fork。 |
| [CursorTouch/Android-MCP](https://github.com/CursorTouch/Android-MCP) | 908 / 2026-10-27 / v0.2.0（2026-05-14） / MIT | 其他 | 只依赖 adb 的 Android MCP：列可交互元素、截图、点按，手机上不装应用 | vision-ui | 可选 | MCP | 获取方式：经 MCP 调用，放第三方仓库。MIT，可 fork。 |
| [insidegui/AudioCap](https://github.com/insidegui/AudioCap)（致谢） | 512 / 2025-08-07 / none / BSD-2-Clause | — | macOS Core Audio 进程 tap 示例：按进程录其他 App 发出的声音 | vision-ui | 不用 | 桌面应用、其他 | 获取方式：借做法自己写，放第三方仓库。BSD-2-Clause，可 fork。 |
| [jfarcand/mirroir-mcp](https://github.com/jfarcand/mirroir-mcp)（致谢） | 271 / 2026-10-30 / runner-v0.4.0（2026-10-27） / Apache-2.0 | 其他 | 经 macOS「iPhone 镜像」窗口控制真实 iPhone 的 MCP：截图、Vision OCR、系统输入 | vision-ui | 可选 | MCP、命令行、Agent Skill | 获取方式：经 MCP 调用，放第三方仓库。Apache-2.0，可 fork；欧盟不可用。 |
| [makeusabrew/audiotee](https://github.com/makeusabrew/audiotee)（致谢） | 92 / 2026-03-31 / none / none | — | macOS 命令行：用进程 tap 录系统声音，按块输出 PCM，可只录指定进程 | vision-ui | 不用 | 命令行 | 获取方式：包装它的命令行，放第三方仓库。README 写 MIT 但没有 LICENSE 文件，先向作者确认。 |
| [rPlayAI/rPlayHub](https://github.com/rPlayAI/rPlayHub) | 66 / 2026-10-01 / v0.1.1（2026-09-28） / GPL-3.0 | 其他 | macOS、Linux 上的 iPhone 镜像与控制工具，自己实现苹果设备的通信协议 | vision-ui | 可选 | 桌面应用 | 相邻，只作参考。GPL-3.0。 |
| [nickatnight96/iphone-mirror-mcp](https://github.com/nickatnight96/iphone-mirror-mcp) | 8 / 2026-10-30 / v1.0.1（2026-08-14） / MIT | 其他 | 驱动 macOS「iPhone 镜像」的 MCP（Swift，63 个工具），截图、OCR、录屏和点按 | vision-ui | 可选 | MCP | 只作参考。MIT。 |
| [Elucidator-hky/phone-agent](https://github.com/Elucidator-hky/phone-agent) | 1 / 2026-10-26 / none / none | 小红书、微信聊天、其他 | 让 Claude Code 经 Mac 操作 iPhone（WDA）和 Android（adb）上的 App，按 App 记录操作路径 | vision-ui | 必须 | 命令行、Agent Skill | 只作参考。没有许可证；视觉兜底会把截图传到云端。 |
| [danzey0425-lgtm/web-image-sourcing](https://github.com/danzey0425-lgtm/web-image-sourcing)（致谢） | 0 / 2026-08-05 / none / MIT | 多平台：小红书、抖音、B站、通用网页 | Agent Skill：真实 Edge 加桌面操控，从小红书、抖音、B站收集图片素材并做视觉质检 | vision-ui | 必须 | Agent Skill、命令行 | 参考：借做法自己写。MIT，可 fork；README 记录了小红书 300012 与 xsec_token 过期的实测。 |
| [bytedance/UI-TARS-desktop](https://github.com/bytedance/UI-TARS-desktop) | 39447 / 2026-10-09 / v0.3.1（2026-10-08） / Apache-2.0 | 通用网页、其他 | 字节跳动的多模态 Agent 套件，用视觉模型截图操作电脑和浏览器 | browser-session | 可选 | 桌面应用、命令行、网页应用、MCP | 只作参考。Apache-2.0；免费远程电脑等于在陌生设备上登录。 |
| [web-infra-dev/midscene](https://github.com/web-infra-dev/midscene)（致谢） | 15443 / 2026-10-30 / v1.17.3（2026-10-29） / MIT | 通用网页、其他 | 视觉驱动的 UI 自动化 SDK，支持网页、Android、iOS、鸿蒙和桌面 | browser-session | 可选 | 库、命令行、浏览器扩展、Agent Skill | 获取方式：包装它的命令行，放第三方仓库。MIT，可 fork。 |
| [ZJU-REAL/Awesome-GUI-Agents](https://github.com/ZJU-REAL/Awesome-GUI-Agents) | 467 / 2026-07-28 / none / none | — | ZJU-REAL 维护的 GUI Agent 论文与资源清单，配套一篇综述，按感知、探索、规划、交互分类，没有代码 | not-applicable | — | 其他 | 只作参考。没有许可证，只链接引用，不复制清单内容。 |

- **设备驱动层许可证友好，适合做通用视觉层插件的底座。** scrcpy、uiautomator2、Appium、WebDriverAgent、go-ios、mirroir-mcp、Peekaboo、Windows-MCP 都是 MIT、Apache 或 BSD；go-ios 是纯 Go，和歇存的技术栈一致。
- **没有找到抖音、小红书、B站 App 收藏页的现成流程**（待核实）。通用 Agent（Open-AutoGLM、mobilerun、mobile-use、MobiAgent）要为每个 App 自己写流程；MobiAgent 能把成功的操作序列录下来复用，Maestro 能把流程写成可回放的 YAML，适合把写好的流程固定下来。
- **检测面和代价。** Android 的无障碍服务和 USB 调试可被 App 查到，iPhone 镜像没有公开的识别 API（[§1.2](#12-视觉层的风险从哪里来)）；托管云手机、远程电脑等于在陌生设备登录；mobilerun、mobile-mcp、Windows-MCP 默认开遥测，包装时关掉；多数 Agent 把截图交给云端模型。
- **录屏、录音也是视觉层。** 录屏记忆类（screenpipe、Windrecorder、openrecall）记录全部屏幕，隐私面大；按进程录声音（AudioCap、audiotee）可以给视频号这类拿不到原始文件的平台做转写，声音不经平台。

<a id="11-agent-接入层"></a>
## 11. Agent 接入层

| 项目 | ★ / 推送 / release / 许可证 | 平台 | 做什么 | 层级 | 登录 | 接口 | 歇存怎么用 |
|---|---|---|---|---|---|---|---|
| [firecrawl/firecrawl](https://github.com/firecrawl/firecrawl) | 197001 / 2026-10-30 / v2.11.0（2026-06-19） / AGPL-3.0 | 通用网页 | 可自托管的网页抓取与转换服务，把网站转成 Markdown 或结构化数据 | public-web | 不用 | HTTP 接口、Docker、库 | 相邻，不接入。AGPL-3.0。 |
| [unclecode/crawl4ai](https://github.com/unclecode/crawl4ai) | 86594 / 2026-10-30 / v0.9.5（2026-10-14） / Apache-2.0 | 通用网页 | 面向大模型的 Python 爬取与 Markdown 转换框架，基于 Playwright | public-web | 可选 | 库、命令行、HTTP 接口、Docker | 相邻，只作参考。Apache-2.0，但 README 另有署名要求，不随包分发。 |
| [jina-ai/reader](https://github.com/jina-ai/reader) | 12213 / 2026-05-22 / none / Apache-2.0 | 通用网页 | r.jina.ai 的开源分支，把任意网址转成适合模型读取的 Markdown | public-web | 不用 | HTTP 接口、Docker | 相邻，不接入。Apache-2.0；托管版要把网址交给第三方。 |
| [browser-use/browser-use](https://github.com/browser-use/browser-use) | 119568 / 2026-10-30 / 0.13.14（2026-10-26） / MIT | 通用网页 | Python 浏览器 Agent 库，经 CDP 驱动本地 Chromium，把页面结构和截图交给模型 | browser-session | 可选 | 库、命令行、MCP | 获取方式：包装它的命令行，放第三方仓库。MIT，可 fork；不用云端 stealth 与验证码处理。 |
| [ChromeDevTools/chrome-devtools-mcp](https://github.com/ChromeDevTools/chrome-devtools-mcp) | 54487 / 2026-10-30 / chrome-devtools-mcp-v1.12.0（2026-10-22） / Apache-2.0 | 通用网页 | Chrome DevTools 团队的 MCP 与命令行，让编码 Agent 控制并检查真实的 Chrome | browser-session | 可选 | MCP、命令行、Agent Skill | 相邻，只作参考。Apache-2.0。 |
| [microsoft/playwright-mcp](https://github.com/microsoft/playwright-mcp) | 38853 / 2026-10-30 / v0.0.83（2026-09-28） / Apache-2.0 | 通用网页 | Playwright 团队的浏览器自动化 MCP，用无障碍树快照描述页面 | browser-session | 可选 | MCP | 相邻，只作参考。Apache-2.0。 |
| [Skyvern-AI/skyvern](https://github.com/Skyvern-AI/skyvern) | 23311 / 2026-10-30 / v1.0.56（2026-10-14） / AGPL-3.0 | 通用网页 | 用大模型加视觉自动化浏览器流程，支持按 schema 抽取和存储凭据登录 | browser-session | 可选 | 库、HTTP 接口、网页应用、MCP | 相邻，只作参考。AGPL-3.0。 |
| [hangwin/mcp-chrome](https://github.com/hangwin/mcp-chrome) | 12518 / 2026-01-06 / v1.0.0（2025-12-29） / MIT | 通用网页 | 基于 Chrome 扩展的 MCP，把日常 Chrome 的标签、书签、历史、脚本注入等交给 AI | browser-session | 可选 | MCP、浏览器扩展 | 相邻，不接入。MIT；安全 issue 未处理，不推荐。 |
| [BrowserMCP/mcp](https://github.com/BrowserMCP/mcp) | 7255 / 2026-10-30 / none / Apache-2.0 | 通用网页 | 让 AI 应用经 Chrome 扩展使用用户自己的 Chrome，包括已登录的站点 | browser-session | 可选 | MCP、浏览器扩展 | 相邻，只作参考。Apache-2.0。 |
| [Panniantong/Agent-Reach](https://github.com/Panniantong/Agent-Reach)（致谢） | 106656 / 2026-10-25 / v1.6.0（2026-10-23） / MIT | 多平台：B站、YouTube、小红书、X、Reddit、Instagram、播客、通用网页、其他 | 面向 Agent 的统一命令行，把各平台的搜索与按链接读取收成 `agent-reach <渠道> <动作>`，输出统一 JSON（用户点名，见 [§3](#3-用户点名的项目)） | private-api | 可选 | 命令行、Agent Skill | 获取方式：包装它的命令行，放第三方仓库。MIT，可 fork；B站后端依赖 GPL 库，CloakBrowser 二进制是专有许可，两者都只能由用户自装。 |
| [jackwener/OpenCLI](https://github.com/jackwener/OpenCLI)（致谢） | 30778 / 2026-10-26 / v2.0.2（2026-10-20） / Apache-2.0 | 多平台：B站、小红书、抖音、微博、知乎、公众号、豆瓣、即刻、播客、微信读书、X、YouTube、Reddit、Instagram、TikTok、头条/西瓜、贴吧、Pinterest、通用网页 | 经浏览器扩展和本地守护进程驱动用户已登录的 Chrome，把 100 多个站点变成命令行（[02 §11](02-adapter-ecosystems.md#11-opencli)） | private-api | 可选 | 命令行、Agent Skill、浏览器扩展、桌面应用 | 获取方式：包装它的命令行，放第三方仓库。Apache-2.0，可 fork；只调收藏类命令；小红书有安全拦截报告。 |
| [nashsu/AutoCLI](https://github.com/nashsu/AutoCLI) | 3014 / 2026-04-20 / v0.3.8（2026-04-20） / Apache-2.0 | 多平台：B站、小红书、知乎、微博、豆瓣、微信读书、即刻、公众号、X、YouTube、Reddit、Instagram、TikTok、播客、通用网页 | OpenCLI 的 Rust 重写版，借用户 Chrome 登录态把 55 个站点变成命令行 | private-api | 可选 | 命令行、浏览器扩展 | 获取方式：包装它的命令行，放第三方仓库。Apache-2.0，可 fork；只开放读取命令；2026-04 后停更。 |
| [sigcli/sigcli](https://github.com/sigcli/sigcli)（致谢） | 292 / 2026-09-28 / @sigcli-cli-v1.7.0（2026-08-18） / MIT | 多平台：通用网页、抖音、知乎、X、Reddit、YouTube、小红书、B站、TikTok | 给 Agent 用的登录与凭据代理：真实浏览器登录一次，加密保存，注入请求而不暴露原文 | private-api | 必须 | 命令行、Agent Skill、其他 | 参考：借做法自己写。MIT，可 fork；只借凭据流，不取签名脚本；不要求用户信任本机 CA。 |
| [adoresever/bilibili-mcp](https://github.com/adoresever/bilibili-mcp) | 159 / 2026-03-11 / none / MIT | B站 | Python 的 B站 MCP，读字幕、弹幕、评论、收藏夹，也能发私信、发动态、上传视频 | private-api | 可选 | MCP | 不接入。MIT，但依赖已关停的 GPL 库；仓库里提交了凭据备份文件。 |
| [YangLiangwei/PersonalizationMCP](https://github.com/YangLiangwei/PersonalizationMCP) | 63 / 2026-03-24 / none / MIT | B站、YouTube、Reddit、其他 | 把 Steam、YouTube、B站、Spotify、Reddit 的个人数据汇总给 AI 的 MCP | private-api | 必须 | MCP | 只作参考。MIT；B站部分依赖已关停的 GPL 库。 |
| [ZhongQuinnKing/caishi](https://github.com/ZhongQuinnKing/caishi)（致谢） | 2 / 2026-10-24 / v1.3.0（2026-10-19） / NOASSERTION | 多平台：抖音、小红书、B站、微博、知乎、公众号、头条/西瓜、通用网页 | 在 OpenCLI 之上补抖音、小红书命令的中文平台读取包，主张只读、单条、低频 | private-api | 可选 | 命令行、MCP、Agent Skill | 获取方式：借做法自己写，放第三方仓库。代码 MIT、文档 CC BY-NC-ND，另有全项目授权口径与之冲突：澄清前只参考。 |
| [microsoft/playwright](https://github.com/microsoft/playwright)（致谢） | 98883 / 2026-10-30 / v1.64.1（2026-10-14） / Apache-2.0 | — | 微软的浏览器自动化与测试框架，驱动 Chromium、Firefox、WebKit | not-applicable | — | 库、命令行 | 只作参考。Apache-2.0；Chrome 136 起连不上用户的默认配置目录。 |
| [CloakHQ/CloakBrowser](https://github.com/CloakHQ/CloakBrowser) | 32634 / 2026-10-29 / chromium-v155.0.8059.39.2-pro（2026-10-27） / MIT | — | 打了源码级指纹补丁的 Chromium，包装成 Playwright、Puppeteer 的替换包 | not-applicable | — | 库、命令行 | 不接入。包装代码 MIT，浏览器二进制是专有许可，不能分发。 |
| [chromedp/chromedp](https://github.com/chromedp/chromedp)（致谢） | 13345 / 2026-10-11 / v0.21.0（2026-10-11） / MIT | — | Go 写的 Chrome DevTools 协议驱动，可启动自己的 Chrome，也可连接已运行的浏览器 | not-applicable | — | 库 | 获取方式：fork 后改造，放第三方仓库。MIT，保留版权声明；作为 Go 依赖编进第三方插件。 |
| [daijro/camoufox](https://github.com/daijro/camoufox) | 12920 / 2026-10-29 / v156.0.1-beta.36（2026-10-06） / MPL-2.0 | — | 基于 Firefox 的自动化浏览器，每次启动从设备分布里抽一套设备特征 | not-applicable | — | 库、其他 | 只作参考。MPL-2.0；用它的工具按严格口径记 private-api。 |
| [go-rod/rod](https://github.com/go-rod/rod) | 7142 / 2026-08-11 / v0.116.2（2024-07-12） / MIT | — | Go 写的 Chrome DevTools 协议驱动，提供启动用户自己 Chrome 的模式 | not-applicable | — | 库 | 只作参考。MIT；那个模式在 Chrome 136 以上不可用。 |
| [lexiforest/curl_cffi](https://github.com/lexiforest/curl_cffi) | 6798 / 2026-10-27 / v0.17.0（2026-10-26） / MIT | — | curl-impersonate 的 Python 绑定，HTTP 请求可模拟浏览器的 TLS 指纹 | not-applicable | — | 库 | 只作参考。MIT。 |
| [openclaw/mcporter](https://github.com/openclaw/mcporter) | 5095 / 2026-10-27 / v0.16.1（2026-10-27） / MIT | — | MCP 的命令行和 TypeScript 运行时，能把 MCP 服务生成成命令行或带类型的客户端 | not-applicable | — | 命令行、库 | 只作参考。MIT。 |
| [Kaliiiiiiiiii-Vinyzu/patchright](https://github.com/Kaliiiiiiiiii-Vinyzu/patchright) | 5027 / 2026-10-25 / v1.64.1（2026-10-15） / Apache-2.0 | — | Playwright 的改版驱动，隐藏自动化标记，接口和 Playwright 相同 | not-applicable | — | 库 | 只作参考。Apache-2.0；歇存官方仓库不引入。 |
| [easychen/CookieCloud](https://github.com/easychen/CookieCloud)（致谢） | 3203 / 2026-10-23 / release-v1.0.3（2026-05-03） / GPL-3.0 | — | 浏览器扩展加自托管服务端，把 cookie 和 localStorage 端到端加密同步到自己的服务器 | not-applicable | — | 浏览器扩展、HTTP 接口、Docker | 获取方式：借做法自己写，放第三方仓库。GPL-3.0：用户自装，独立进程调用；兼容接收端只监听本机。 |
| [refraction-networking/utls](https://github.com/refraction-networking/utls) | 2619 / 2026-10-06 / v1.8.2（2026-01-13） / BSD-3-Clause | — | Go 标准 TLS 库的分支，可自定义 ClientHello 模仿浏览器握手 | not-applicable | — | 库 | 只作参考。BSD-3-Clause。 |
| [bogdanfinn/tls-client](https://github.com/bogdanfinn/tls-client) | 1920 / 2026-10-28 / v1.16.0（2026-09-02） / BSD-3-Clause | — | Go 的 HTTP 客户端，接口近似 net/http，可选浏览器 TLS 指纹配置 | not-applicable | — | 库 | 只作参考。BSD-3-Clause。 |
| [kairi003/Get-cookies.txt-LOCALLY](https://github.com/kairi003/Get-cookies.txt-LOCALLY)（致谢） | 1279 / 2025-10-07 / v0.7.2（2025-10-05） / MIT | — | 只在本地导出 cookies.txt 的浏览器扩展，不向外发送数据 | not-applicable | — | 浏览器扩展 | 只作参考。MIT；歇存只读它导出的文件。 |
| [thewh1teagle/rookie](https://github.com/thewh1teagle/rookie) | 363 / 2026-01-22 / v0.5.6（2024-11-01） / MIT · 已归档 | — | Rust 写的浏览器 cookie 读取库，带 Python 和 JS 绑定；已归档 | not-applicable | — | 库 | 不接入。MIT。 |
| [browserutils/kooky](https://github.com/browserutils/kooky)（致谢） | 295 / 2026-06-15 / v0.2.10（2026-06-14） / MIT | — | Go 写的浏览器 cookie 读取库，覆盖 Chrome、Firefox、Safari 等 | not-applicable | — | 库、命令行 | 获取方式：fork 后改造，放第三方仓库。MIT，保留版权声明；按域名过滤，只在内存里用。 |

- **这一层不针对单个平台**，提供驱动浏览器、管理凭据、读 cookie 的能力，风险由用在哪个平台、用哪个浏览器决定。
- **「复用日常浏览器」还是「另起专用浏览器」没有定论。** MediaCrawler 维护者认为连用户真实浏览器更难被识别；Agent-Reach 1.6 改用专用浏览器，只复制该站 cookie，但这样可能被判为同一账号出现在另一套指纹上。Chrome 136 起，自动化工具连不上用户的默认配置目录（Playwright、chromedp、go-rod）。
- **改指纹的浏览器和 TLS 模拟库按严格口径属 `private-api`。** CloakBrowser、Camoufox、patchright、curl_cffi、utls、tls-client 歇存官方不引入；CloakBrowser 的浏览器二进制是专有许可，不能分发。
- **cookie 读取是 `session-api` 插件的公共积木。** kooky、CookieCloud、Get-cookies.txt-LOCALLY 都可用，原则是只按域名读取、只在内存里用、导出文件由用户自己选；把带登录态的浏览器整个交给 Agent 的 MCP（mcp-chrome、chrome-devtools-mcp 的自动连接）扩大了提示注入的波及面。

<a id="12-转换转写字幕与-ocr"></a>
## 12. 转换：转写、字幕与 OCR

| 项目 | ★ / 推送 / release / 许可证 | 平台 | 做什么 | 层级 | 登录 | 接口 | 歇存怎么用 |
|---|---|---|---|---|---|---|---|
| [Zackriya-Solutions/meetily](https://github.com/Zackriya-Solutions/meetily) | 32322 / 2026-10-30 / v0.4.1（2026-09-12） / MIT | — | 本地优先的会议记录应用：同时录麦克风和系统声音，本地实时转写和摘要 | vision-ui | 不用 | 桌面应用 | 获取加转换：借做法自己写，获取部分放第三方仓库。MIT，可 fork。 |
| [steipete/summarize](https://github.com/steipete/summarize)（致谢） | 6769 / 2026-10-28 / v0.25.3（2026-10-21） / MIT | 多平台：YouTube、播客、X、通用网页 | 命令行加浏览器侧栏，把网页、YouTube、播客提取成文本再摘要，可只输出 JSON | browser-session | 可选 | 命令行、浏览器扩展、库、Agent Skill | 转换插件：包装它的命令行。MIT，可 fork；可选回退会把内容发给第三方。 |
| [Huanshere/VideoLingo](https://github.com/Huanshere/VideoLingo)（致谢） | 19007 / 2026-09-30 / v3.1.1（2026-09-27） / Apache-2.0 | YouTube | 一键做字幕切分、翻译、对齐和配音，默认本地 Qwen3-ASR 加对齐模型 | private-api | 可选 | 网页应用、HTTP 接口、Docker | 转换插件：借做法自己写。Apache-2.0，可 fork。 |
| [WEIFENG2333/VideoCaptioner](https://github.com/WEIFENG2333/VideoCaptioner)（致谢） | 16410 / 2026-09-12 / v1.4.2（2026-05-24） / GPL-3.0 | YouTube、B站 | 字幕全流程工具：转写、模型断句校正、翻译、配音、把字幕压进视频（[05 §2.4](05-note-apps-and-plugins.md#24-独立工具与服务)） | private-api | 可选 | 桌面应用、命令行、Agent Skill | 转换插件：包装它的命令行。GPL-3.0：用户自装，独立进程调用；调用时显式指定本地引擎（默认引擎把音频交给必剪）。 |
| [wendy7756/AI-Video-Transcriber](https://github.com/wendy7756/AI-Video-Transcriber)（致谢） | 3371 / 2026-10-11 / none / Apache-2.0 | 多平台：YouTube、B站、抖音、TikTok、X、Instagram、播客 | 自部署的音视频转写工具：先取平台字幕，没有再用 faster-whisper 转写，可选模型校对与摘要（用户点名，见 [§3](#3-用户点名的项目)） | private-api | 不用 | 网页应用、HTTP 接口、命令行、MCP、Agent Skill、Docker | 获取加转换：包装它的命令行，获取部分放第三方仓库。Apache-2.0，可 fork；只包装本地文件模式；插件清单误写 MIT，以 LICENSE 为准。 |
| [lanbinleo/bili2text](https://github.com/lanbinleo/bili2text)（致谢） | 1991 / 2026-06-16 / sv-1.0.0（2023-11-25） / MIT | B站 | 把 B站 视频转成文字稿的命令行，本地或云端转写 | private-api | 可选 | 命令行、网页应用、桌面应用、HTTP 接口 | 只作参考。MIT。 |
| [zlxlabs/VideoTranscriptAPI](https://github.com/zlxlabs/VideoTranscriptAPI)（致谢） | 281 / 2026-10-30 / v1.2.0（2026-10-15） / NOASSERTION | 多平台：YouTube、B站、抖音、小红书、视频号、X、播客 | 自部署的音视频转写服务：下载、转写、模型校对总结，结果可推送到企业微信或飞书 | private-api | 可选 | HTTP 接口、网页应用、Docker | 获取加转换：借做法自己写，获取部分放第三方仓库。PolyForm Noncommercial：只借设计。 |
| [JimmyLv/bibigpt-skill](https://github.com/JimmyLv/bibigpt-skill) | 133 / 2026-10-26 / none / MIT | 多平台：YouTube、B站、播客、抖音、TikTok、小红书、X | BibiGPT 的 Skill 与插件，经命令行或托管 MCP 总结各平台音视频（[05 §1.2](05-note-apps-and-plugins.md#12-独立工具)） | private-api | 不用 | Agent Skill、命令行、MCP | 只作参考。MIT；服务在云端，只能作用户自选的云端转换。 |
| [xiaokeaijqx/VideoNote](https://github.com/xiaokeaijqx/VideoNote) | 52 / 2026-06-17 / v2.3.8（2026-06-03） / Apache-2.0 | 多平台：B站、YouTube、抖音、快手、小红书、公众号、播客 | 基于 BiliNote 的视频与文章笔记工具，带跨笔记问答和热点订阅 | private-api | 可选 | 网页应用、桌面应用、浏览器扩展、HTTP 接口、Docker | 只作参考。标 Apache-2.0，但含 GPL-3.0 头的签名文件。 |
| [autogame-17/scribe-studio](https://github.com/autogame-17/scribe-studio) | 58 / 2026-07-25 / v0.4.12（2026-06-30） / NOASSERTION | 多平台：视频号、YouTube、B站、X、抖音、TikTok | 桌面应用：下载视频号或 yt-dlp 支持站点的视频，本地转写后模型校对并积累词表 | client-hook | 可选 | 桌面应用 | 转换插件：借做法自己写。GPL-3.0-or-later 叠加 Commons Clause 子树：只能用户自装。 |
| [microsoft/markitdown](https://github.com/microsoft/markitdown)（致谢） | 193003 / 2026-10-28 / v0.1.9（2026-10-21） / MIT | — | 微软的文件转 Markdown 工具，支持 PDF、Office、HTML、图片、音频，附 MCP | not-applicable | 不用 | 命令行、库、MCP、Docker | 转换插件：包装它的命令行。MIT，可 fork；音频转写和 OCR 插件会把数据发到云端。 |
| [PaddlePaddle/PaddleOCR](https://github.com/PaddlePaddle/PaddleOCR)（致谢） | 91992 / 2026-09-16 / v3.7.0（2026-06-11） / Apache-2.0 | — | 飞桨的 OCR 与文档解析工具包，含 PP-OCRv6、PP-StructureV3 和 PaddleOCR-VL（[05 §2.2](05-note-apps-and-plugins.md#22-ocr-插件)） | not-applicable | — | 库、命令行、HTTP 接口、模型 | 转换插件：借做法自己写。Apache-2.0，可 fork；按 04 的结论只取模型，经 RapidOCR 运行。 |
| [opendatalab/MinerU](https://github.com/opendatalab/MinerU)（致谢） | 82811 / 2026-10-30 / mineru-4.1.3-released（2026-10-30） / NOASSERTION | — | 文档解析工具，把 PDF、图片、Office 转成 Markdown 和 JSON，分四档 | not-applicable | — | 命令行、库、HTTP 接口、网页应用、Docker | 转换插件：包装它的命令行。Apache-2.0 加附加条款；较早的一版权重是 AGPL-3.0。 |
| [docling-project/docling](https://github.com/docling-project/docling)（致谢） | 69869 / 2026-10-30 / v2.145.0（2026-10-29） / MIT | — | 文档转换工具，解析 PDF、Office、HTML、音视频，输出 Markdown、JSON、WebVTT | not-applicable | — | 命令行、库、MCP、HTTP 接口 | 转换插件：包装它的命令行。MIT，可 fork。 |
| [ggml-org/whisper.cpp](https://github.com/ggml-org/whisper.cpp)（致谢） | 54797 / 2026-10-30 / v1.9.5（2026-10-06） / MIT | — | C/C++ 的 Whisper 推理，单个可执行文件在 CPU、Metal、CUDA 上转写，自带 HTTP 服务（[05 §2.1](05-note-apps-and-plugins.md#21-转写插件)） | not-applicable | — | 命令行、HTTP 接口、库 | 转换插件：包装它的命令行。MIT，可 fork。 |
| [hiroi-sora/Umi-OCR](https://github.com/hiroi-sora/Umi-OCR)（致谢） | 48296 / 2025-11-20 / v2.1.5（2025-03-25） / MIT | — | 离线 OCR 桌面软件，截图、批量图片和 PDF 识别，提供本地 HTTP 接口（[05 §2.4](05-note-apps-and-plugins.md#24-独立工具与服务)） | not-applicable | — | 桌面应用、HTTP 接口、命令行 | 转换插件：调它的 HTTP 接口。MIT，可 fork；没有 macOS 版。 |
| [datalab-to/marker](https://github.com/datalab-to/marker)（致谢） | 40707 / 2026-10-29 / v2.1.0（2026-10-29） / Apache-2.0 | — | 把 PDF、图片、Office、EPUB 转成 Markdown、JSON、HTML，内部用 Surya | not-applicable | — | 命令行、库 | 转换插件：包装它的命令行。代码 Apache-2.0；权重是修改版 OpenRAIL-M，有商用门槛。 |
| [SYSTRAN/faster-whisper](https://github.com/SYSTRAN/faster-whisper)（致谢） | 26324 / 2026-10-25 / v1.2.1（2025-10-31） / MIT | — | 基于 CTranslate2 的 Whisper 推理库，更快更省显存，支持词级时间戳和 VAD | not-applicable | — | 库 | 转换插件：调它的 HTTP 接口。MIT，可 fork。 |
| [m-bain/whisperX](https://github.com/m-bain/whisperX)（致谢） | 24946 / 2026-10-22 / v3.8.6（2026-05-25） / BSD-2-Clause | — | 在 faster-whisper 结果上做词级强制对齐和说话人分离（[05 §2.4](05-note-apps-and-plugins.md#24-独立工具与服务)） | not-applicable | — | 命令行、库 | 转换插件：包装它的命令行。BSD-2-Clause，可 fork；分离模型要先接受条款。 |
| [deepseek-ai/DeepSeek-OCR](https://github.com/deepseek-ai/DeepSeek-OCR) | 24101 / 2026-01-27 / none / MIT | — | DeepSeek 的「上下文光学压缩」OCR 模型 | not-applicable | — | 模型、库 | 只作参考。MIT。 |
| [chidiwilliams/buzz](https://github.com/chidiwilliams/buzz)（致谢） | 22288 / 2026-10-30 / v1.4.5（2026-08-23） / MIT | — | 离线转写和翻译的桌面应用，后端可选多种 Whisper 实现，带命令行（[05 §2.4](05-note-apps-and-plugins.md#24-独立工具与服务)） | not-applicable | 不用 | 桌面应用、命令行 | 转换插件：包装它的命令行。MIT，可 fork。 |
| [datalab-to/surya](https://github.com/datalab-to/surya)（致谢） | 21558 / 2026-10-29 / v0.23.0（2026-10-29） / Apache-2.0 | — | Datalab 的 OCR 模型和工具包，做行检测、识别、版面、阅读顺序和表格 | not-applicable | — | 命令行、库、模型 | 转换插件：包装它的命令行。代码 Apache-2.0；权重同 marker。 |
| [modelscope/FunASR](https://github.com/modelscope/FunASR)（致谢） | 20888 / 2026-10-28 / v1.4.17（2026-10-28） / MIT | — | 通义的语音识别工具包，含 Paraformer、SenseVoice 等，另有免 Python 的单文件运行时（[05 §2.4](05-note-apps-and-plugins.md#24-独立工具与服务)） | not-applicable | — | 库、命令行、HTTP 接口、Docker、模型 | 转换插件：包装它的命令行。工具包 MIT，权重按各模型卡。 |
| [allenai/olmocr](https://github.com/allenai/olmocr) | 19814 / 2026-03-25 / v0.4.27（2026-03-12） / Apache-2.0 | — | Allen AI 的 PDF 线性化工具，用 7B 视觉语言模型转 Markdown | not-applicable | — | 命令行、库、模型 | 只作参考。Apache-2.0；要 12GB 以上 NVIDIA 显存。 |
| [jianchang512/pyvideotrans](https://github.com/jianchang512/pyvideotrans)（致谢） | 19592 / 2026-10-30 / v4.17（2026-10-23） / GPL-3.0 | — | 视频翻译配音工具：识别、翻译、配音再合成，有桌面、命令行和网页界面 | not-applicable | — | 桌面应用、命令行、网页应用 | 转换插件：包装它的命令行。GPL-3.0：用户自装，独立进程调用。 |
| [k2-fsa/sherpa-onnx](https://github.com/k2-fsa/sherpa-onnx)（致谢） | 15627 / 2026-10-29 / v2.0.1（2026-10-26） / Apache-2.0 | — | ONNX 推理框架，一套运行时跑多种离线与流式识别模型，带 VAD 和说话人分离 | not-applicable | — | 命令行、库、HTTP 接口 | 转换插件：包装它的命令行。Apache-2.0，可 fork；刚升 2.0，接入前核对参数。 |
| [YaoFANGUK/video-subtitle-extractor](https://github.com/YaoFANGUK/video-subtitle-extractor)（致谢） | 9678 / 2026-04-09 / 2.2.0（2026-04-04） / Apache-2.0 | — | 在本机提取视频硬字幕生成 SRT：VideoSubFinder 找字幕帧，PaddleOCR 识别 | not-applicable | — | 桌面应用、命令行 | 转换插件：包装它的命令行。Apache-2.0，可 fork；VideoSubFinder 是 GPLv2，由用户自装。 |
| [QwenAudio/SenseVoice](https://github.com/QwenAudio/SenseVoice)（致谢） | 9637 / 2026-09-30 / runtime-llamacpp-v0.2.1（2026-08-27） / MIT | — | 多语种语音理解小模型 SenseVoiceSmall，附 ONNX 导出和单文件运行时（[05 §1.2](05-note-apps-and-plugins.md#12-独立工具)） | not-applicable | — | 模型、库、命令行 | 转换插件：包装它的命令行。代码 MIT；权重按 FunASR 模型许可，可商用、要署名。 |
| [studio-dots-ai/dots.ocr](https://github.com/studio-dots-ai/dots.ocr) | 9180 / 2026-03-24 / none / MIT | — | 小红书 hi lab 开源的 1.7B 多语言文档版面解析模型 | not-applicable | — | 模型、库 | 只作参考。MIT。 |
| [RapidAI/RapidOCR](https://github.com/RapidAI/RapidOCR)（致谢） | 8391 / 2026-10-30 / v3.10.0（2026-10-08） / Apache-2.0 | — | 把 PaddleOCR 模型转成 ONNX 等格式运行的多语言 OCR 工具包（[05 §2.4](05-note-apps-and-plugins.md#24-独立工具与服务)） | not-applicable | — | 库 | 转换插件：借做法自己写。Apache-2.0，可 fork。 |
| [thewh1teagle/vibe](https://github.com/thewh1teagle/vibe)（致谢） | 7962 / 2026-10-30 / v3.3.0（2026-10-29） / MIT | — | Tauri 写的离线转写应用，后端 vibe-server 是 OpenAI 兼容的单文件服务 | not-applicable | 不用 | 桌面应用、命令行、HTTP 接口、Agent Skill | 转换插件：调它的 HTTP 接口。MIT，可 fork；运行时拉 yt-dlp 不校验哈希。 |
| [modelscope/FunClip](https://github.com/modelscope/FunClip)（致谢） | 6474 / 2026-10-08 / v2.2.1（2026-09-01） / MIT | — | 用 Paraformer 识别后按文字或说话人选段剪视频，导出全片和片段 SRT | not-applicable | — | 网页应用、命令行 | 转换插件：借做法自己写。MIT，可 fork。 |
| [argmaxinc/argmax-oss-swift](https://github.com/argmaxinc/argmax-oss-swift)（致谢） | 6435 / 2026-10-27 / v1.1.1（2026-10-08） / MIT | — | Apple 平台端侧语音工具包，WhisperKit 用 Core ML 转写，附 OpenAI 兼容本地服务 | not-applicable | — | 库、命令行、HTTP 接口 | 转换插件：调它的 HTTP 接口。MIT，可 fork。 |
| [QwenLM/Qwen3-ASR](https://github.com/QwenLM/Qwen3-ASR)（致谢） | 3797 / 2026-06-26 / none / Apache-2.0 | — | 通义千问的语音识别模型和配套对齐模型，支持 52 种语言和方言 | not-applicable | — | 库、HTTP 接口、模型 | 转换插件：调它的 HTTP 接口。Apache-2.0，可 fork。 |
| [SoMarkAI/OXR](https://github.com/SoMarkAI/OXR)（致谢） | 2640 / 2026-10-24 / none / NOASSERTION | — | SoMark 开源的 1.3B 文档解析模型与服务，把 PDF、文档图片转成 Markdown 和带坐标的 JSON（用户点名，见 [§3](#3-用户点名的项目)） | not-applicable | — | HTTP 接口、命令行、Docker、模型、网页应用 | 转换插件：调它的 HTTP 接口。OXR License（Apache-2.0 加附加条款）：只做薄客户端，不复制源码；本地部署要 NVIDIA GPU。 |
| [kaixxx/noScribe](https://github.com/kaixxx/noScribe)（致谢） | 2224 / 2026-10-29 / v0.8（2026-10-06） / GPL-3.0 | — | 面向访谈的本地转写应用：faster-whisper 加说话人分离，自带校对编辑器（[05 §2.4](05-note-apps-and-plugins.md#24-独立工具与服务)） | not-applicable | — | 桌面应用、命令行、Docker | 转换插件：包装它的命令行。GPL-3.0：用户自装，独立进程调用。 |
| [finnvoor/yap](https://github.com/finnvoor/yap)（致谢） | 1635 / 2026-07-20 / 1.2.1（2026-07-20） / CC0-1.0 | — | 调用 macOS 26 系统语音框架的转写命令行，输出 txt、srt、vtt、json | not-applicable | — | 命令行 | 转换插件：包装它的命令行。CC0-1.0，可 fork。 |
| [JimLiu/baocut](https://github.com/JimLiu/baocut)（致谢） | 1628 / 2026-10-30 / baocut-v3.14.0-build.85（2026-10-29） / NOASSERTION | — | 以可编辑时间线为中心的 AI 视频智能体：转写、字幕、翻译、配音、剪口播 | not-applicable | 可选 | 桌面应用、命令行、MCP、Agent Skill、网页应用 | 只作参考。BaoCut Community License（源码可见）：只经 CLI 或 MCP 对接。 |
| [datalab-to/indra](https://github.com/datalab-to/indra)（致谢） | 1457 / 2026-10-22 / v1.1.2（2026-10-20） / Apache-2.0 | — | Datalab 开源的 CPU OCR 工具和小模型，把 PDF、图片转成 Markdown、纯文本或带框 JSON（用户点名，见 [§3](#3-用户点名的项目)） | not-applicable | — | 命令行、库、网页应用、模型 | 转换插件：包装它的命令行。代码 Apache-2.0；权重是修改版 OpenRAIL-M，有收入与竞争条款，首次运行由用户下载。 |
| [timminator/VideOCR](https://github.com/timminator/VideOCR)（致谢） | 852 / 2026-10-17 / v1.6.0（2026-09-09） / MIT | — | 带图形界面的硬字幕提取，默认本地 PaddleOCR，另有交给 Google Lens 的混合模式 | not-applicable | — | 桌面应用、命令行、Docker、库 | 转换插件：包装它的命令行。MIT，可 fork；显式指定 paddleocr（Python 接口默认 google_lens）。 |
| [FireRedTeam/FireRedASR2S](https://github.com/FireRedTeam/FireRedASR2S)（致谢） | 728 / 2026-06-02 / none / Apache-2.0 | — | 小红书 FireRed 团队的中文语音识别系统，覆盖方言和中英混说，带字级时间戳 | not-applicable | — | 命令行、库、HTTP 接口、模型 | 转换插件：包装它的命令行。Apache-2.0，可 fork。 |
| [SWHL/RapidVideOCR](https://github.com/SWHL/RapidVideOCR)（致谢） | 506 / 2026-10-25 / v3.1.1（2025-06-04） / Apache-2.0 | — | 读 VideoSubFinder 导出的字幕关键帧，用 RapidOCR 识别后生成字幕 | not-applicable | — | 命令行、库 | 转换插件：包装它的命令行。Apache-2.0，可 fork。 |
| [bytefer/macos-vision-ocr](https://github.com/bytefer/macos-vision-ocr)（致谢） | 322 / 2025-02-14 / v0.0.2（2025-02-14） / MIT | — | 调用 Apple Vision 的 OCR 命令行，输出带位置和置信度的 JSON | not-applicable | — | 命令行 | 转换插件：fork 后改造。MIT，保留版权声明。 |
| [anbang278/echo-notes](https://github.com/anbang278/echo-notes) | 26 / 2026-10-22 / 0.4.44（2026-10-22） / MIT | — | Obsidian 插件：把库里的录音或麦克风录音转成带来源链接的逐字稿，附一个只监听本机的 MLX 转写服务，接口与 OpenAI 兼容 | not-applicable | — | Obsidian 插件、HTTP 接口 | 转换插件：调它的 HTTP 接口。MIT，可 fork；本机服务只支持 Apple Silicon，所用模型的许可另看。 |

- **本地引擎够用。** 转写有 whisper.cpp、faster-whisper、SenseVoice、sherpa-onnx、FunASR、Qwen3-ASR，OCR 有 RapidOCR、PaddleOCR 模型、Apple Vision，内容不必发出本机；引擎对比见 [04 §2](04-tech-options.md#2-ocr-sidecar)、[04 §3](04-tech-options.md#3-asr-sidecar)，转换插件的做法见 [05 §2](05-note-apps-and-plugins.md#2-转换类插件与工具)。
- **权重许可要单独看。** SenseVoice、Paraformer 按 FunASR 模型许可（可商用、要署名）；indra、marker、surya 的权重有收入门槛；OXR、MinerU 有附加条款。
- **几个工具默认把数据发出本机。** VideoCaptioner 默认引擎把音频交给 B站 必剪，markitdown 的音频转写交给 Google，VideOCR 的 Python 接口默认用 Google Lens，BibiGPT 在云端处理；包装时显式指定本地引擎。
- **带链接下载的转写工具，获取部分另算。** AI-Video-Transcriber、VideoTranscriptAPI、bili2text、VideoNote 的下载跟随 yt-dlp 或第三方服务，歇存只包装它们的本地文件模式，下载交给获取插件。

<a id="13-笔记知识库与收藏归档"></a>
## 13. 笔记、知识库与收藏归档

| 项目 | ★ / 推送 / release / 许可证 | 平台 | 做什么 | 层级 | 登录 | 接口 | 歇存怎么用 |
|---|---|---|---|---|---|---|---|
| [karlicoss/promnesia](https://github.com/karlicoss/promnesia)（致谢） | 1910 / 2026-10-30 / extension-v.1.4.1（2026-04-07） / MIT | 多平台：Reddit、X、Telegram、通用网页 | 浏览器扩展加本地索引器，汇总历史和各平台导出里出现过的链接，提示「在哪里见过」 | official-export | 不用 | 浏览器扩展、命令行、库、HTTP 接口 | 参考：借做法自己写。MIT，可 fork。 |
| [akosbalasko/yarle](https://github.com/akosbalasko/yarle)（致谢） | 1816 / 2026-03-31 / v6.16.1（2026-03-05） / MIT | 其他 | Evernote ENEX 转 Markdown，保留时间、标签、来源、附件，可直接放进笔记库 | official-export | 不用 | 桌面应用、命令行 | 获取方式：包装它的命令行，放第三方仓库。MIT，可 fork。 |
| [dogsheep/pocket-to-sqlite](https://github.com/dogsheep/pocket-to-sqlite) | 105 / 2023-07-10 / 0.2.3（2023-07-09） / Apache-2.0 | 通用网页 | 用 Pocket 官方 API 把保存的条目拉进 SQLite；Pocket 已关停 | official-export | 必须 | 命令行 | 只作参考。Apache-2.0。 |
| [FlyingWolFox/Netscape-Bookmarks-File-Parser](https://github.com/FlyingWolFox/Netscape-Bookmarks-File-Parser)（致谢） | 52 / 2024-02-14 / v1.1（2020-05-20） / MIT | 通用网页 | Netscape 书签 HTML 的解析与生成库 | official-export | 不用 | 库 | 获取方式：借做法自己写，放第三方仓库。MIT，可 fork。 |
| [dogsheep/evernote-to-sqlite](https://github.com/dogsheep/evernote-to-sqlite)（致谢） | 44 / 2021-11-02 / 0.3.2（2021-08-26） / Apache-2.0 | 其他 | 把 Evernote ENEX 导出转成 SQLite，为正文和图中文字建全文索引 | official-export | 不用 | 命令行 | 获取方式：借做法自己写，放第三方仓库。Apache-2.0，可 fork。 |
| [karlicoss/instapexport](https://github.com/karlicoss/instapexport)（致谢） | 32 / 2026-10-19 / none / MIT | 通用网页 | 用 Instapaper 官方 API 把本人的文章收藏和高亮导出为 JSON | official-export | 必须 | 命令行、库 | 获取方式：包装它的命令行，放第三方仓库。MIT，可 fork。 |
| [questionjie-max/echo-memory](https://github.com/questionjie-max/echo-memory) | 20 / 2026-10-25 / v1.0.0（2026-10-25） / Apache-2.0 | 其他 | Apple Silicon Mac 上的本地个人记忆应用：收录录音、文档、ChatGPT 与 Claude 的导出记录和本机 Agent 会话，结论可点回原话，经只读 MCP 交给 Agent | official-export | 不用 | 桌面应用、MCP | 参考：借做法自己写。Apache-2.0，可 fork；借引用逐字核对、只读 MCP 默认关闭的做法。 |
| [volcengine/OpenViking](https://github.com/volcengine/OpenViking)（致谢） | 41194 / 2026-10-30 / v0.5.2.1（2026-10-27） / AGPL-3.0 | 通用网页、公众号 | 火山引擎的 Agent 上下文数据库，把资料、记忆和技能组织成 viking:// 文件系统并生成分层摘要（用户点名，见 [§3](#3-用户点名的项目)） | public-web | 可选 | HTTP 接口、命令行、MCP、库、网页应用、桌面应用、Docker、Agent Skill | 只作参考。主项目 AGPL-3.0，ov 命令行 Apache-2.0；可做导出目标，不复制代码。 |
| [khoj-ai/khoj](https://github.com/khoj-ai/khoj) | 37904 / 2026-08-02 / 2.0.0-beta.28（2026-03-26） / AGPL-3.0 | 通用网页、其他 | 可自托管的个人 AI 助手，索引本地文档，聊天时可联网读网页 | public-web | 可选 | 网页应用、桌面应用、Obsidian 插件、HTTP 接口、Docker | 只作参考。AGPL-3.0；维护放缓。 |
| [topoteretes/cognee](https://github.com/topoteretes/cognee) | 32926 / 2026-10-30 / v1.6.6（2026-10-29） / Apache-2.0 | 通用网页、其他 | Agent 记忆平台：把文档、网页和对话加工成知识图谱加向量索引 | public-web | 可选 | 库、HTTP 接口、MCP、网页应用、Docker | 只作参考。Apache-2.0。 |
| [karakeep-app/karakeep](https://github.com/karakeep-app/karakeep)（致谢） | 30159 / 2026-10-29 / v0.34.1（2026-10-22） / AGPL-3.0 | 通用网页 | 自托管的「什么都收」书签应用，抓正文和截图，模型打标签和摘要（[02 §9](02-adapter-ecosystems.md#9-karakeep-抓取器)） | public-web | 可选 | 网页应用、手机应用、浏览器扩展、HTTP 接口、命令行、MCP、Agent Skill、Docker | 参考：借做法自己写。AGPL-3.0：只借配置与流程。 |
| [linkwarden/linkwarden](https://github.com/linkwarden/linkwarden) | 20272 / 2026-10-26 / v2.16.3（2026-09-09） / AGPL-3.0 | 通用网页 | 自托管的协作书签管理器，收藏时保存截图、PDF 和单文件 HTML | public-web | 可选 | 网页应用、手机应用、浏览器扩展、HTTP 接口、Docker | 只作参考。AGPL-3.0。 |
| [Y2Z/monolith](https://github.com/Y2Z/monolith)（致谢） | 15564 / 2026-10-29 / v2.11.7（2026-10-29） / CC0-1.0 | 通用网页 | 把网页连同资源内嵌成单个 HTML 的命令行和 Rust 库（[02 §9](02-adapter-ecosystems.md#9-karakeep-抓取器)） | public-web | 可选 | 命令行、库、桌面应用 | 获取方式：包装它的命令行，放第三方仓库。CC0-1.0，可 fork；不执行 JS，单页应用常只拿到空壳。 |
| [wallabag/wallabag](https://github.com/wallabag/wallabag)（致谢） | 13034 / 2026-10-30 / 2.6.15（2026-10-28） / MIT | 通用网页 | 自托管稍后读，保存网页正文，支持标签、标注和多种格式导出 | public-web | 可选 | 网页应用、HTTP 接口、浏览器扩展、手机应用、Docker | 参考：借做法自己写。MIT，可 fork。 |
| [go-shiori/shiori](https://github.com/go-shiori/shiori) | 11696 / 2026-10-26 / v1.8.0（2025-09-26） / MIT | 通用网页 | Go 写的轻量书签管理器，单个可执行文件，可离线保存网页 | public-web | 可选 | 命令行、网页应用、浏览器扩展、HTTP 接口、Docker | 参考：借做法自己写。MIT，可 fork。 |
| [sissbruecker/linkding](https://github.com/sissbruecker/linkding) | 11365 / 2026-10-26 / v1.47.0（2026-09-13） / MIT | 通用网页 | 极简的自托管书签管理器，可在服务端或浏览器生成网页快照 | public-web | 可选 | 网页应用、浏览器扩展、HTTP 接口、Docker | 参考：借做法自己写。MIT，可 fork。 |
| [adbar/trafilatura](https://github.com/adbar/trafilatura)（致谢） | 7043 / 2026-10-30 / v2.3.1（2026-10-06） / Apache-2.0 | 通用网页 | Python 的网页正文与元数据提取工具，也能按站点地图、订阅源发现和下载网页 | public-web | 不用 | 库、命令行 | 转换插件：借做法自己写。Apache-2.0，可 fork；v1.7.0 及更早是 GPL-3.0，不取旧代码。 |
| [wabarc/wayback](https://github.com/wabarc/wayback) | 2249 / 2026-10-23 / v0.21.1（2026-07-05） / GPL-3.0 | 通用网页 | Go 写的网页存档工具，可作 Telegram 等聊天机器人常驻，收到网址就存档 | public-web | 不用 | 命令行、Docker、HTTP 接口、其他 | 参考：借做法自己写。GPL-3.0：只借思路；默认会提交公共归档。 |
| [sanzoghenzo/markdownr](https://github.com/sanzoghenzo/markdownr) | 183 / 2026-04-07 / v1.6.0（2023-11-05） / MIT | 通用网页 | Android App：分享网址进来，在手机本机转成 Markdown，再分享给笔记 App | public-web | 不用 | 手机应用 | 参考：借做法自己写。MIT，可 fork。 |
| [notesynchelper/obsidian-notehelper-v2](https://github.com/notesynchelper/obsidian-notehelper-v2) | 4 / 2026-10-19 / 3.2.17（2026-10-19） / MIT | 多平台：公众号、小红书、知乎、微信聊天、通用网页 | Obsidian 插件：把转发给服务号「笔记同步助手」的文章、链接、聊天记录拉进笔记库（[05 §1.1](05-note-apps-and-plugins.md#11-obsidian-社区插件)） | public-web | 必须 | Obsidian 插件、其他 | 相邻：借做法自己写。MIT，可 fork；服务端闭源、收费。 |
| [eduaiot/eduaiot-wechat-sync](https://github.com/eduaiot/eduaiot-wechat-sync) | 2 / 2026-10-28 / 1.0.8（2026-10-28） / MIT | 多平台：视频号、B站、抖音、公众号、小红书、快手 | Obsidian 插件：同步用户在小程序或快捷指令里存的链接和文件，附服务端生成的文字稿 | public-web | 必须 | Obsidian 插件、其他 | 相邻：借做法自己写。MIT，可 fork；服务端闭源，内容存在作者服务器。 |
| [yangyongzhen/juCang](https://github.com/yangyongzhen/juCang) | 2 / 2026-08-12 / none / none | 多平台：抖音、B站、YouTube、小红书、公众号、知乎、头条/西瓜、通用网页 | Tauri 2 的本地收藏聚合器：粘贴分享文本入库，视频可在应用内播放并缓存 | public-web | 不用 | 桌面应用、网页应用 | 参考：借做法自己写。没有 LICENSE 文件，只借思路。 |
| [gildas-lormeau/SingleFile](https://github.com/gildas-lormeau/SingleFile)（致谢） | 22781 / 2026-10-29 / v1.33.0（2026-10-28） / AGPL-3.0 | 通用网页 | 浏览器扩展：把当前网页完整存成一个 HTML 文件，另有命令行版（[02 §9](02-adapter-ecosystems.md#9-karakeep-抓取器)） | browser-session | 可选 | 浏览器扩展、命令行 | 获取方式：调它的 HTTP 接口，放第三方仓库。AGPL-3.0：用户自装扩展，歇存只提供接收上传的本地接口。 |
| [webclipper/web-clipper](https://github.com/webclipper/web-clipper) | 6866 / 2025-10-21 / v1.42.0（2025-10-21） / NOASSERTION | 通用网页 | 把网页剪藏到 Notion、OneNote、语雀等笔记服务的扩展 | browser-session | 可选 | 浏览器扩展 | 只作参考。LICENSE 写 GPL-2.0-or-later，只参照。 |
| [deathau/markdownload](https://github.com/deathau/markdownload) | 4051 / 2025-06-11 / 3.4.0（2024-08-23） / Apache-2.0 | 通用网页 | 把当前网页剪成 Markdown 文件下载的扩展；2024-08 后没有发布 | browser-session | 可选 | 浏览器扩展 | 只作参考。Apache-2.0。 |
| [webrecorder/archiveweb.page](https://github.com/webrecorder/archiveweb.page) | 1607 / 2026-09-04 / v0.17.1（2026-09-04） / AGPL-3.0 | 通用网页 | 在用户自己的 Chrome 里边浏览边录制网页，存成 WACZ/WARC 可离线回放 | browser-session | 可选 | 浏览器扩展、桌面应用 | 获取方式：借做法自己写，放第三方仓库。AGPL-3.0：只读它导出的 WACZ。 |
| [joeseesun/qiaomu-clipper](https://github.com/joeseesun/qiaomu-clipper)（致谢） | 399 / 2026-10-30 / 1.19.0（2026-10-20） / MIT | 多平台：视频号、YouTube、B站、播客、通用网页 | Chrome 扩展：剪藏网页和音视频到 Obsidian；视频号经用户已登录的元宝标签页取凭证 | browser-session | 可选 | 浏览器扩展、其他 | 获取加转换：借做法自己写，获取部分放第三方仓库。MIT，可 fork；元宝页内请求是视频号路线里风险最低的写法。 |
| [hc-tec/my-collection-skills](https://github.com/hc-tec/my-collection-skills)（致谢） | 56 / 2026-02-09 / none / MIT | B站、知乎、小红书 | 一组 Skill 和脚本，拉 B站、知乎、小红书的收藏结构和内容，视频可转写 | browser-session | 必须 | Agent Skill、命令行、Docker | 获取方式：包装它的命令行，放第三方仓库。MIT，可 fork；Cookie 被搬进 Docker 里的浏览器，比用户自己的浏览器更易被识别。 |
| [edequalsawesome/jiggyclipper](https://github.com/edequalsawesome/jiggyclipper) | 2 / 2026-02-15 / none / none | 通用网页 | iOS 分享扩展：在 Safari 点分享，按 Web Clipper 模板把当前网页剪进 Obsidian | browser-session | 可选 | 手机应用 | 只作参考。没有许可证，只参考。 |
| [vzhd1701/evernote-backup](https://github.com/vzhd1701/evernote-backup)（致谢） | 1778 / 2026-09-23 / 1.14.0（2026-08-05） / MIT | 其他 | 把 Evernote、印象笔记的全部笔记增量同步到 SQLite，再导出 ENEX | session-api | 必须 | 命令行、Docker | 获取方式：包装它的命令行，放第三方仓库。MIT，可 fork；不代收密码；import 方式属 client-hook，默认不用。 |
| [LifeArchiveProject/HistoryHub](https://github.com/LifeArchiveProject/HistoryHub)（致谢） | 1228 / 2026-10-25 / v1.7.2（2026-06-21） / MIT | B站、通用网页 | 个人历史记录中心：扫码同步 B站 观看历史，扩展导入网页浏览记录，存本地 SQLite | session-api | 必须 | 网页应用、浏览器扩展、桌面应用、Docker | 获取方式：借做法自己写，放第三方仓库。MIT，可 fork。 |
| [lfnovo/open-notebook](https://github.com/lfnovo/open-notebook) | 40981 / 2026-10-25 / v1.15.0（2026-10-04） / MIT | 通用网页、YouTube | 开源 NotebookLM 替代品，收 PDF、网页、音视频和 YouTube 链接，做问答和摘要 | private-api | 可选 | 网页应用、HTTP 接口、Docker | 只作参考。MIT；默认先把网址发给 Jina。 |
| [ArchiveBox/ArchiveBox](https://github.com/ArchiveBox/ArchiveBox)（致谢） | 28904 / 2026-10-29 / v0.9.77（2026-10-27） / MIT | 多平台：通用网页、YouTube | 自托管网页归档，用多种抽取器把每个页面存成普通文件夹 | private-api | 可选 | 命令行、网页应用、HTTP 接口、Docker、库 | 参考：借做法自己写。MIT，可 fork；默认把网址提交给 archive.org，要手动关。 |
| [omnivore-app/omnivore](https://github.com/omnivore-app/omnivore) | 16398 / 2026-10-27 / android-0.230.0（2026-10-14） / AGPL-3.0 | 通用网页、YouTube | 开源稍后读，云服务 2024-11 关停后改为纯自托管，由社区维护 | private-api | 不用 | 网页应用、浏览器扩展、手机应用、HTTP 接口、Docker | 只作参考。AGPL-3.0；依赖它的笔记插件一起失效。 |
| [obsidianmd/obsidian-clipper](https://github.com/obsidianmd/obsidian-clipper)（致谢） | 5431 / 2026-09-22 / 1.7.1（2026-07-22） / MIT | 通用网页、YouTube | Obsidian 官方剪藏扩展，按模板把当前页面存成笔记（[02 §10](02-adapter-ecosystems.md#10-obsidian-web-clipper-模板)；[05 §1.1](05-note-apps-and-plugins.md#11-obsidian-社区插件)） | private-api | 可选 | 浏览器扩展、命令行 | 参考：借做法自己写。MIT，可 fork；公众号图片、知乎排版有未修的问题。 |
| [weaiw/trove-ai](https://github.com/weaiw/trove-ai) | 188 / 2026-08-23 / v1.4.2（2026-08-23） / NOASSERTION | 多平台：公众号、视频号、头条/西瓜、抖音、小红书、B站、通用网页 | 面向中文互联网的自托管稍后读加知识库 Agent，保存各平台链接做检索和问答 | private-api | 不用 | 网页应用、HTTP 接口、MCP、Obsidian 插件、Docker | 只作参考。LICENSE 是 AGPL-3.0 全文；依赖的 B站 库已关停。 |
| [KristenPan/pickoo-your-favorites](https://github.com/KristenPan/pickoo-your-favorites) | 20 / 2026-09-17 / none / MIT | 多平台：小红书、公众号、抖音、B站、播客、通用网页 | 本地优先的 AI 收藏夹工作台，抓取链接内容生成带证据包的 Markdown | private-api | 可选 | 网页应用、Agent Skill、命令行 | 参考：借做法自己写。MIT，可 fork。 |
| [Raven7979/cyber-bookhouse](https://github.com/Raven7979/cyber-bookhouse)（致谢） | 15 / 2026-08-20 / v0.2.9（2026-08-20） / MIT | 多平台：抖音、B站、YouTube、小红书、公众号、视频号、X、播客、快手、微博、知乎、通用网页 | 一套 Skill：把各端发来的链接在本机整理成带来源、逐字稿和截图的 Obsidian 笔记 | private-api | 可选 | Agent Skill、命令行 | 获取加转换：借做法自己写，获取部分放第三方仓库。MIT，可 fork；入口经第三方云。 |
| [ienone/VaultStream](https://github.com/ienone/VaultStream)（致谢） | 6 / 2026-10-25 / v0.1.0（2026-03-10） / MIT | 多平台：B站、X、小红书、知乎、微博、Telegram、通用网页 | 自托管的收藏存档与分发系统：采集多平台链接和收藏夹，按规则推送到 IM | private-api | 可选 | HTTP 接口、网页应用、手机应用、Docker | 参考：借做法自己写。MIT，可 fork。 |
| [Sheldon460/link-to-inbox](https://github.com/Sheldon460/link-to-inbox) | 3 / 2026-10-03 / v1.7.0（2026-08-24） / MIT | 多平台：公众号、小红书、抖音、快手、B站、视频号、TikTok、YouTube、通用网页 | WorkBuddy Skill：把各平台链接归档进 Obsidian 收件箱，视频链接交给第三方解析 | private-api | 可选 | Agent Skill | 只作参考。MIT。 |
| [simonemuller6127-png/omni-collector](https://github.com/simonemuller6127-png/omni-collector) | 2 / 2026-10-16 / 0.10.0（2026-10-16） / MIT | 多平台：B站、YouTube、小红书、其他 | Obsidian 插件加本地引擎，把 B站、YouTube、小红书等的收藏和点赞同步成索引笔记（[05 §1.1](05-note-apps-and-plugins.md#11-obsidian-社区插件)） | private-api | 必须 | Obsidian 插件、其他 | 只作参考。MIT；登录态加签名加定时批量，是风险最高的组合。 |
| [thedotmack/claude-mem](https://github.com/thedotmack/claude-mem) | 105626 / 2026-10-30 / v15.6.0（2026-10-29） / Apache-2.0 | — | 编码 Agent 的会话记忆插件，把工具调用压缩成观察记录，下次按需注入 | not-applicable | — | 命令行、HTTP 接口、MCP、Agent Skill、网页应用 | 只作参考。Apache-2.0。 |
| [mem0ai/mem0](https://github.com/mem0ai/mem0) | 68104 / 2026-10-29 / ts-v3.3.1（2026-09-25） / Apache-2.0 | — | Agent 长期记忆层：从对话抽取事实、去重更新，按用户和会话检索 | not-applicable | — | 库、HTTP 接口、命令行、Agent Skill、Docker | 只作参考。Apache-2.0。 |
| [getzep/graphiti](https://github.com/getzep/graphiti) | 32367 / 2026-10-28 / v0.31.2（2026-10-20） / Apache-2.0 | — | 时序知识图谱框架，记录每条事实的有效时间段 | not-applicable | — | 库、MCP、HTTP 接口、Docker | 只作参考。Apache-2.0。 |
| [MODSetter/SurfSense](https://github.com/MODSetter/SurfSense)（致谢） | 16691 / 2026-10-30 / v2.2.5（2026-10-27） / Apache-2.0 | 多平台：通用网页、YouTube、TikTok、Reddit、Instagram | 可离线的 NotebookLM 替代品，对本机文件做带引用的问答，另有网页读取插件 | not-applicable | 不用 | 桌面应用、网页应用、Docker、MCP | 只作参考。根目录 Apache-2.0，proprietary 插件目录是 BSL 1.1。 |
| [mozilla/readability](https://github.com/mozilla/readability)（致谢） | 11518 / 2026-08-04 / none / Apache-2.0 | 通用网页 | Firefox 阅读视图使用的正文提取库，多数阅读器和剪藏工具的算法来源 | not-applicable | — | 库 | 转换插件：借做法自己写。Apache-2.0，可 fork；公众号正文提取有缺陷（#921）。 |
| [mixmark-io/turndown](https://github.com/mixmark-io/turndown)（致谢） | 11499 / 2026-10-18 / v7.2.4（2026-04-03） / MIT | 通用网页 | JS 写的 HTML 转 Markdown 库，规则可扩展 | not-applicable | — | 库 | 转换插件：借做法自己写。MIT，可 fork。 |
| [kepano/defuddle](https://github.com/kepano/defuddle)（致谢） | 9828 / 2026-10-25 / 0.19.6（2026-10-24） / MIT | 多平台：通用网页、YouTube、B站、X、Reddit | Obsidian Web Clipper 用的正文提取库，附 YouTube、B站、X、Reddit 专用提取器 | not-applicable | — | 库、命令行 | 转换插件：借做法自己写。MIT，可 fork；X 链接会发给 FxTwitter。 |
| [basicmachines-co/basic-memory](https://github.com/basicmachines-co/basic-memory)（致谢） | 4415 / 2026-10-30 / v1.0.0（2026-10-29） / AGPL-3.0 | — | 以本地 Markdown 为知识库、经 MCP 让 Agent 读写的笔记工具，SQLite 只做索引 | not-applicable | — | MCP、命令行、库、Agent Skill | 参考：借做法自己写。AGPL-3.0：借格式和做法，不拷代码。 |
| [JohannesKaufmann/html-to-markdown](https://github.com/JohannesKaufmann/html-to-markdown)（致谢） | 3838 / 2026-08-03 / v2.5.2（2026-06-07） / MIT | 通用网页 | Go 写的 HTML 转 Markdown 库和命令行，规则以插件扩展 | not-applicable | — | 库、命令行 | 转换插件：fork 后改造。MIT，保留版权声明；非 UTF-8 页面要先转码。 |
| [go-shiori/go-readability](https://github.com/go-shiori/go-readability) | 939 / 2025-12-05 / none / MIT · 已归档 | 通用网页 | Readability.js 的 Go 逐行移植；已归档，README 指定改用 Readeck 的 v2 | not-applicable | — | 库、命令行 | 只作参考。MIT。 |
| [mcncarl/agent-memory-vault](https://github.com/mcncarl/agent-memory-vault) | 341 / 2026-10-04 / none / MIT | — | Claude Code、Codex、Ailu 共用的本地记忆库模板：Markdown 为事实源，SQLite 索引，可选本地语义检索（用户点名，见 [§3](#3-用户点名的项目)） | not-applicable | — | 命令行 | 参考：借做法自己写。MIT，可 fork；写入意图、回执、审计的流程可借给 Agent 变更集。 |
| [dogsheep/dogsheep-beta](https://github.com/dogsheep/dogsheep-beta)（致谢） | 211 / 2026-04-08 / 0.11（2026-04-08） / Apache-2.0 | — | 把多个 SQLite 库里的内容汇总成一张全文索引的 Datasette 插件 | not-applicable | 不用 | 命令行、网页应用 | 只作参考。Apache-2.0。 |
| [markusmobius/go-trafilatura](https://github.com/markusmobius/go-trafilatura)（致谢） | 151 / 2026-09-30 / v2.2.6（2026-09-29） / Apache-2.0 | 通用网页 | trafilatura 的 Go 原生移植，提取正文和作者、日期、语言等元数据 | not-applicable | — | 库、命令行 | 转换插件：fork 后改造。Apache-2.0，保留版权声明；依赖 go-re2，体积和启动开销大。 |
| [markusmobius/go-domdistiller](https://github.com/markusmobius/go-domdistiller)（致谢） | 74 / 2026-09-29 / v1.0.0（2026-09-15） / MIT | 通用网页 | Chrome 阅读模式所用 DOM Distiller 的 Go 原生移植 | not-applicable | — | 库 | 转换插件：fork 后改造。MIT，保留版权声明；分发时保留上游的 BSD 与 Apache 声明。 |
| [karlicoss/bleanser](https://github.com/karlicoss/bleanser)（致谢） | 16 / 2026-10-25 / v0.2.20251009（2025-10-08） / MIT | — | 清理冗余备份：找出内容已被后一份完全包含的旧导出 | not-applicable | 不用 | 命令行、库 | 参考：借做法自己写。MIT，可 fork。 |
| [Qc-riviere/EchoMind-releases](https://github.com/Qc-riviere/EchoMind-releases) | 4 / 2026-06-29 / v0.3.8（2026-06-29） / NOASSERTION | 微信聊天 | 闭源的灵感速记桌面应用，本地语义检索；给绑定的微信 bot 发消息即可记录，README 称经腾讯官方 ClawBot 通道；仓库只放文档和安装包 | not-applicable | 可选 | 桌面应用、其他 | 相邻：借做法自己写。专有许可，不开源，不能 fork、包装或分发；只参考「微信发给 bot 就收下」的入口思路。 |
| [HuanHuanHuanFFF/Echo](https://github.com/HuanHuanHuanFFF/Echo) | 1 / 2026-10-06 / v0.2.0（2026-10-06） / MIT | — | 个人 Markdown 笔记的本地检索 MCP：按标题结构切块，BM25 加向量混合检索，返回带路径和行号的原文片段 | not-applicable | — | MCP、命令行 | 参考：借做法自己写。MIT，可 fork；借检索结果带行号和截断标记的格式。 |
| [readeck/go-readability](https://codeberg.org/readeck/go-readability)（致谢） | —（Codeberg） / 2026-09-30 / v2.1.3（2026-09-30） / MIT | 通用网页 | Readeck 团队接手维护的 go-readability 分支，v2 兼容 Readability.js 0.6 | not-applicable | — | 库 | 转换插件：fork 后改造。MIT，保留版权声明；托管在 Codeberg，经 Go 模块代理拉取。 |
| [duzui300/Obsiclip](https://github.com/duzui300/Obsiclip)（致谢） | 0 / 2026-10-08 / v1.1.0（2026-10-08） / AGPL-3.0 | 其他 | Android 分享桥：分享来的文字套模板经 obsidian:// 写入笔记；实验功能用无障碍读 Kindle 划线 | not-applicable | — | 手机应用 | 参考：借做法自己写。AGPL-3.0：只借思路。 |

- **正文提取有纯 Go 的选择。** Readeck 维护的 go-readability、go-trafilatura、go-domdistiller、html-to-markdown 许可证宽松，可以进歇存的构建；中文页面有已知问题（readability 对公众号正文丢失，html-to-markdown 不处理 GBK 等非 UTF-8 编码）。
- **归档工具的默认值要看清。** ArchiveBox 默认把网址提交给 archive.org，wayback 默认提交公共归档，open-notebook 先把网址发给 Jina；本地优先的用法要手动关掉。
- **收藏同步类风险最高，手机分享入口风险最低。** omni-collector、VaultStream、trove-ai 把多平台登录、签名、定时批量放在一起；Obsiclip、markdownr、jiggyclipper 这类手机分享入口没有自动化，但 eduaiot-wechat-sync、obsidian-notehelper-v2 把内容存在作者的服务器上。
- **云服务关停会连带下游。** Omnivore 云服务 2024-11 关停后，依赖它的笔记插件一起失效；Pocket 关停后 pocket-to-sqlite 无法运行。这是歇存坚持文件为真相源的直接例证。

<a id="14-规则源生态"></a>
## 14. 规则源生态

| 项目 | ★ / 推送 / release / 许可证 | 平台 | 做什么 | 层级 | 登录 | 接口 | 歇存怎么用 |
|---|---|---|---|---|---|---|---|
| [gedoor/legado](https://github.com/gedoor/legado) | 47184 / 2026-05-27 / none / none | 通用网页、其他 | 安卓阅读应用，书源由用户导入；2026-05-27 起仓库只剩一份侵权公告 | public-web | 可选 | 手机应用 | 只作参考。已清空，原为 GPL-3.0，不拷代码。 |
| [Predidit/Kazumi](https://github.com/Predidit/Kazumi)（致谢） | 40194 / 2026-10-29 / 2.4.2（2026-10-23） / GPL-3.0 | 通用网页、其他 | Flutter 番剧采集与播放应用，来源由用户导入的 JSON 规则决定，规则在独立仓库托管（用户点名，见 [§3](#3-用户点名的项目)） | public-web | 不用 | 桌面应用、手机应用 | 参考：借做法自己写。应用 GPL-3.0，不拷代码；规则仓库 MIT，借规则格式和 deprecated 机制。 |
| [open-ani/animeko](https://github.com/open-ani/animeko)（致谢） | 21340 / 2026-10-29 / v6.4.0（2026-10-24） / AGPL-3.0 | 通用网页、其他 | Kotlin 跨平台追番应用，视频来自可订阅的数据源，带弹幕和离线缓存 | public-web | 可选 | 手机应用、桌面应用 | 参考：借做法自己写。AGPL-3.0：只借订阅协议的设计。 |
| [RSS-Bridge/rss-bridge](https://github.com/RSS-Bridge/rss-bridge) | 9322 / 2026-10-28 / 2025-08-05（2025-08-05） / Unlicense | 多平台：通用网页、YouTube、Reddit、Telegram、TikTok | PHP 的 RSS 生成器，为没有订阅源的网站提供 bridge，另有用选择器写规则的通用 bridge | public-web | 可选 | 网页应用、HTTP 接口、Docker | 参考：借做法自己写。Unlicense，可 fork。 |
| [Predidit/KazumiRules](https://github.com/Predidit/KazumiRules)（致谢） | 2484 / 2026-10-28 / none / MIT | 通用网页、其他 | Kazumi 官方规则仓库：一条规则一个 JSON，CI 生成目录，失效规则标 deprecated | public-web | 不用 | 其他 | 参考：借做法自己写。MIT，可 fork。 |
| [fivefilters/ftr-site-config](https://github.com/fivefilters/ftr-site-config)（致谢） | 483 / 2026-10-25 / none / NOASSERTION | 多平台：通用网页、公众号 | 站点正文提取规则库，一个域名一个文件，用 XPath 指定标题、正文和要删的元素（[02 §13](02-adapter-ecosystems.md#13-其他相关模式)） | public-web | 可选 | 其他 | 转换插件：fork 后改造。公有领域（CC0），可直接内置。 |
| [lnreader/lnreader-plugins](https://github.com/lnreader/lnreader-plugins) | 364 / 2026-10-29 / v2.0.0（2024-03-21） / MIT | 通用网页、其他 | LNReader 的社区插件仓库，一站一个 TypeScript 插件，提 PR 前跑在线检查 | public-web | 不用 | 其他 | 参考：借做法自己写。MIT，可 fork。 |
| [creamycake-anime/animeko-subs](https://github.com/creamycake-anime/animeko-subs)（致谢） | 75 / 2026-10-25 / none / none | 通用网页、其他 | 第三方维护的 Animeko 数据源订阅仓库，一源一 JSON，附源质量评测规范 | public-web | 不用 | 其他 | 参考：借做法自己写。没有许可证，只借思路。 |
| [gkd-kit/gkd](https://github.com/gkd-kit/gkd) | 43713 / 2026-10-30 / v1.12.1（2026-05-20） / GPL-3.0 | 其他 | 基于无障碍服务的 Android 应用，按订阅的规则在指定界面自动点按 | vision-ui | 可选 | 手机应用 | 只作参考。GPL-3.0；只参考规则格式和订阅分发。 |
| [zotero/translators](https://github.com/zotero/translators)（致谢） | 1726 / 2026-10-28 / none / none | 多平台：通用网页、YouTube、豆瓣 | Zotero 的站点翻译器仓库，一站一个 JS 文件，从页面或站点接口提取元数据（[02 §13](02-adapter-ecosystems.md#13-其他相关模式)） | browser-session | 可选 | 其他 | 参考：借做法自己写。单个文件头是 AGPL-3.0：只借声明和下架机制。 |
| [DIYgod/RSSHub](https://github.com/DIYgod/RSSHub)（致谢） | 46731 / 2026-10-30 / v2026.10.30-8497.05b561f（2026-10-30） / AGPL-3.0 | 多平台：微博、知乎、豆瓣、即刻、贴吧、X、Instagram、Telegram、Pinterest、B站、小红书、抖音、快手、TikTok、头条/西瓜、公众号、YouTube、播客、其他 | 把各平台内容生成 RSS 订阅源的服务，路由覆盖国内外主要平台（[02 §7](02-adapter-ecosystems.md#7-rsshub-与-rsshub-radar)） | private-api | 可选 | HTTP 接口、Docker | 获取方式：调它的 HTTP 接口，放第三方仓库。AGPL-3.0：用户自建实例，经 HTTP 调用。 |
| [venera-app/venera](https://github.com/venera-app/venera) | 11311 / 2026-04-05 / v1.6.3（2026-03-08） / GPL-3.0 · 已归档 | 通用网页、其他 | Flutter 漫画阅读器，支持本地漫画和 JS 编写的网络源；已归档 | private-api | 可选 | 手机应用、桌面应用 | 只作参考。GPL-3.0。 |
| [venera-app/venera-configs](https://github.com/venera-app/venera-configs) | 936 / 2026-10-19 / none / none | 通用网页、其他 | Venera 的漫画源仓库，一源一 JS 文件，index.json 列版本，经 jsDelivr 分发 | private-api | 可选 | 其他 | 只作参考。没有许可证，只看做法。 |
| [vercel-labs/skills](https://github.com/vercel-labs/skills) | 37240 / 2026-10-29 / v1.7.4（2026-10-19） / MIT | — | 开放 Agent Skill 生态的安装命令行 npx skills 和 skills.sh 目录入口 | not-applicable | — | 命令行、网页应用 | 只作参考。MIT。 |
| [openclaw/clawhub](https://github.com/openclaw/clawhub) | 9542 / 2026-10-30 / v0.24.1（2026-10-28） / MIT | — | OpenClaw 的公开 Skill 与插件注册表，带安全审计、举报与下架流程 | not-applicable | — | 网页应用、命令行、HTTP 接口 | 只作参考。MIT。 |
| [iflytek/skillhub](https://github.com/iflytek/skillhub) | 5341 / 2026-10-30 / v0.2.25（2026-10-30） / Apache-2.0 | — | 讯飞的企业自托管 Skill 注册表，带命名空间、两级审核和安全扫描 | not-applicable | — | 网页应用、命令行、HTTP 接口、Docker | 只作参考。Apache-2.0。 |

- **规则仓库的形状相同。** KazumiRules、animeko-subs、venera-configs、lnreader-plugins、zotero/translators、ftr-site-config 都是一条规则一个文件、CI 生成目录、失效的标 deprecated 或移出目录，对应歇存的插件列表和屏蔽清单。
- **宿主不带来源也会被追责。** legado 应用本身不带书源，2026-05 仍清空了仓库；Crunchyroll 的 DMCA 直接点名第三方扩展仓库 aniyomi-extensions。
- **Skill 注册表的审核挡不住风险。** ClawHub 抽查 35 个中文平台 Skill，25 个被标可疑，全部没有签名，也追溯不到源码提交；skills.sh 的安全审计超时就照常安装；SkillHub 的扫描器可以由运营方关闭。
- **RSSHub、RSS-Bridge 把平台变成订阅源。** RSSHub 自建实例填入个人 cookie 后，订阅轮询会持续带账号请求，属于「跟着收」的高频场景，要单独提示。

<a id="15-相邻创作发布与分析"></a>
## 15. 相邻：创作、发布与分析

| 项目 | ★ / 推送 / release / 许可证 | 平台 | 做什么 | 层级 | 登录 | 接口 | 歇存怎么用 |
|---|---|---|---|---|---|---|---|
| [gitroomhq/postiz-app](https://github.com/gitroomhq/postiz-app)（致谢） | 40194 / 2026-10-30 / v2.25.3（2026-10-24） / AGPL-3.0 | 多平台：X、YouTube、TikTok、Instagram、Reddit、Pinterest、Telegram、其他 | 自托管的社媒排期发布平台，经官方 OAuth 接口发布并拉取帖子数据 | official-export | 必须 | 网页应用、HTTP 接口、MCP、命令行、Docker | 相邻，只作参考。AGPL-3.0。 |
| [inovector/mixpost](https://github.com/inovector/mixpost) | 3891 / 2026-03-16 / 2.6.0（2026-03-16） / MIT | X、其他 | 自托管社媒管理工具的 Lite 版，经官方接口排期发布 | official-export | 必须 | 网页应用、库 | 相邻，只作参考。MIT。 |
| [BetaStreetOmnis/xhs_ai_publisher](https://github.com/BetaStreetOmnis/xhs_ai_publisher) | 2094 / 2026-10-30 / v1.2.0（2026-06-28） / Apache-2.0 | 多平台：小红书、微博、头条/西瓜、B站、公众号、通用网页、其他 | 小红书笔记的本地编辑与发布工具，可取各平台热榜作素材 | public-web | 必须 | 桌面应用、网页应用、HTTP 接口、Docker | 相邻，不接入。Apache-2.0。 |
| [Rookage/echo-editorial](https://github.com/Rookage/echo-editorial) | 7 / 2026-10-30 / none / MIT | 抖音 | 短视频创作者的本地 Web 工作台：抖音分享链接或本地视频转逐字稿，改写成小红书或抖音风格的稿件，再配音合成竖屏视频 | public-web | 不用 | 网页应用、HTTP 接口、Agent Skill | 只作参考。MIT。 |
| [wechatsync/Wechatsync](https://github.com/wechatsync/Wechatsync)（致谢） | 6537 / 2026-10-23 / 1.0.10（2021-04-28） / GPL-3.0 | 多平台：公众号、知乎、微博、小红书、抖音、B站、豆瓣、头条/西瓜、X、通用网页、其他 | 用浏览器登录态把文章同步成多个平台草稿的扩展，附命令行和 MCP | browser-session | 必须 | 浏览器扩展、命令行、MCP、Agent Skill | 相邻，只作参考。GPL-3.0；部分适配器不开源。 |
| [Xiangyu-CAS/xiaohongshu-ops-skill](https://github.com/Xiangyu-CAS/xiaohongshu-ops-skill) | 2465 / 2026-03-19 / stable（2026-03-12） / none | 小红书 | 配合 OpenClaw 浏览器的小红书运营 Skill：分析账号、给选题，也能发布、回复 | browser-session | 必须 | Agent Skill | 相邻，不接入。没有许可证。 |
| [hanliang97/MatrixMedia](https://github.com/hanliang97/MatrixMedia)（致谢） | 1165 / 2026-10-29 / v0.13.3（2026-10-28） / GPL-2.0 | 多平台：抖音、快手、小红书、视频号、B站、头条/西瓜、其他 | Electron 应用：视频批量发到 8 个平台的多个账号，附命令行、本机 HTTP、MCP，能采集自己账号的数据（用户点名，见 [§3](#3-用户点名的项目)） | session-api | 必须 | 桌面应用、命令行、HTTP 接口、MCP、Agent Skill | 参考：借做法自己写。GPL-2.0-only，和 AGPL 内核、MIT 插件都不兼容：只借登录态页面里取列表的适配器结构。 |
| [Kerinlin/EchoDistill](https://github.com/Kerinlin/EchoDistill) | 23 / 2026-07-08 / none / MIT | 多平台：YouTube、Reddit、B站、知乎、小红书、X、贴吧、抖音、其他 | 油猴脚本：在已打开的帖子或视频页展开并读取评论区，交给用户配置的模型生成评论归纳和反方审视 | session-api | 可选 | 浏览器扩展 | 只作参考。MIT。 |
| [hypit-ai/hypit](https://github.com/hypit-ai/hypit)（致谢） | 26782 / 2026-10-30 / v0.3.1（2026-10-09） / NOASSERTION | 多平台：TikTok、YouTube、Instagram、B站 | 给编码 Agent 的视频制作语言与命令行：读参考视频、转写截帧，生成可重跑的工作流再渲染新视频（用户点名，见 [§3](#3-用户点名的项目)） | private-api | 不用 | 命令行、Agent Skill、网页应用 | 只作参考。改过的 Apache-2.0，禁止商业再分发：只借下载器隔离的做法。 |
| [elebumm/RedditVideoMakerBot](https://github.com/elebumm/RedditVideoMakerBot) | 12562 / 2026-10-30 / 3.4.0（2025-10-28） / GPL-3.0 | Reddit、YouTube、TikTok | 一条命令把 Reddit 帖子和评论做成短视频 | private-api | 必须 | 命令行、网页应用、Docker | 相邻，不接入。GPL-3.0。 |
| [zhouxiaoka/autoclip](https://github.com/zhouxiaoka/autoclip)（致谢） | 9871 / 2026-10-30 / v1.5.7（2026-10-24） / MIT | 多平台：B站、YouTube、TikTok、Instagram | AI 剪辑桌面工具，把长视频切成短视频并生成字幕、封面和发布文案 | private-api | 可选 | 桌面应用、命令行、MCP | 相邻，不接入。MIT。 |
| [RayVentura/ShortGPT](https://github.com/RayVentura/ShortGPT) | 8092 / 2025-02-10 / v0.3.0（2025-02-10） / MIT | YouTube | 实验性的短视频自动化框架；2025-02 后停更 | private-api | 不用 | 库、网页应用、Docker | 相邻，不接入。MIT。 |
| [Anil-matcha/AI-Youtube-Shorts-Generator](https://github.com/Anil-matcha/AI-Youtube-Shorts-Generator) | 5666 / 2026-10-24 / none / MIT | YouTube | 把 YouTube 长视频剪成竖版短视频，默认调用 MuAPI 云接口 | private-api | 不用 | 命令行、Agent Skill | 相邻，不接入。MIT。 |
| [biliup/biliup](https://github.com/biliup/biliup) | 5566 / 2026-10-27 / v1.3.4（2026-10-22） / MIT | 多平台：B站、抖音、快手、YouTube、其他 | 直播录制与 B站 投稿工具 | private-api | 可选 | 网页应用、命令行、桌面应用、Agent Skill、Docker | 相邻，不接入。代码 MIT，但 README 写禁止商业用途，按非商业处理。 |
| [ZJU-REAL/Easel](https://github.com/ZJU-REAL/Easel)（致谢） | 4217 / 2026-10-25 / v0.3.0（2026-10-12） / Apache-2.0 | 多平台：小红书、抖音、快手、知乎、B站、视频号、公众号、微博、头条/西瓜、通用网页、其他 | 浙江大学 REAL Lab 的社媒运营 Agent 工作台：热点发现、选题和图文音视频创作，扫码登录七个国内平台发布并读回账号数据（用户点名，见 [§3](#3-用户点名的项目)） | private-api | 可选 | 网页应用、命令行、Agent Skill | 参考：借做法自己写。Apache-2.0，可 fork；内置的 gzh-design 是 AGPL-3.0，抖音发布脚本来源许可存疑；只借登录状态文件、读回分档和资料目录锁。 |
| [pbeenigg/LittleCrawler](https://github.com/pbeenigg/LittleCrawler) | 762 / 2026-08-09 / none / MIT | 小红书、知乎、其他 | 异步爬虫框架，抓小红书、知乎、闲鱼的搜索、详情和评论 | private-api | 必须 | 命令行、HTTP 接口、网页应用 | 不接入。标 MIT，但小红书模块和 MediaCrawler 一致且去掉了原声明，不能 fork。 |
| [atian-create/lingzao-skill](https://github.com/atian-create/lingzao-skill) | 303 / 2026-07-02 / none / MIT-0 | 小红书、抖音、公众号 | 小红书运营 Skill 包，搜索公开笔记要接作者的付费服务 | private-api | 不用 | Agent Skill、命令行 | 相邻，不接入。MIT-0。 |
| [harry0703/MoneyPrinterTurbo](https://github.com/harry0703/MoneyPrinterTurbo)（致谢） | 136472 / 2026-10-30 / v1.3.9（2026-10-21） / MIT | TikTok、Instagram、YouTube | 按主题自动写脚本、配音、配素材和字幕，合成短视频，可经 Upload-Post 发布（用户点名，见 [§3](#3-用户点名的项目)） | not-applicable | 可选 | 命令行、HTTP 接口、网页应用、Agent Skill、Docker | 相邻，只作参考。MIT；借它给 Agent 的退出码和确认参数约定。 |
| [dreammis/social-auto-upload](https://github.com/dreammis/social-auto-upload)（致谢） | 15948 / 2026-10-22 / v1.0.0-skills.2（2026-10-22） / MIT | 多平台：抖音、B站、小红书、快手、视频号、微博、知乎、TikTok、YouTube、Instagram、其他 | 命令行加 Agent Skill 的多平台发布工具，用浏览器自动化把视频、图文发到 17 个平台入口（用户点名，见 [§3](#3-用户点名的项目)） | not-applicable | 必须 | 命令行、Agent Skill、库、网页应用、HTTP 接口、Docker | 参考：借做法自己写。MIT，可 fork；借会话检查、Skill 分发和人工验证交接的做法。 |
| [FujiwaraChoki/MoneyPrinter](https://github.com/FujiwaraChoki/MoneyPrinter) | 14122 / 2026-03-26 / none / MIT | YouTube | 按主题自动生成 YouTube Shorts，可选经官方接口上传 | not-applicable | 可选 | 网页应用、HTTP 接口、Docker | 相邻，不接入。MIT。 |
| [doocs/md](https://github.com/doocs/md)（致谢） | 13488 / 2026-10-30 / @doocs/md-cli@2.1.4（2026-10-30） / WTFPL | 公众号 | 把 Markdown 渲染成公众号样式的编辑器，有网页版、浏览器扩展和命令行 | not-applicable | 可选 | 网页应用、浏览器扩展、命令行、其他 | 相邻，只作参考。WTFPL。 |
| [linyqh/NarratoAI](https://github.com/linyqh/NarratoAI)（致谢） | 11641 / 2026-10-24 / v0.8.7（2026-07-17） / MIT | — | 给影视素材写解说文案，再自动剪辑、配音、加字幕的工具 | not-applicable | — | 网页应用、Docker | 相邻，不接入。MIT。 |
| [ddean2009/MoneyPrinterPlus](https://github.com/ddean2009/MoneyPrinterPlus) | 7273 / 2025-03-07 / v4.7（2024-09-23） / GPL-3.0 | 多平台：抖音、快手、小红书、视频号、B站 | 批量生成混剪短视频，再用 Selenium 发到国内平台；2025-03 后停更 | not-applicable | 必须 | 网页应用 | 相邻，不接入。GPL-3.0，和另一份禁止商用的文件冲突。 |
| [YILS-LIN/short-video-factory](https://github.com/YILS-LIN/short-video-factory)（致谢） | 5989 / 2026-10-28 / v1.3.0（2026-10-16） / AGPL-3.0 | — | Electron 应用：模型写口播文案、Edge TTS 配音，把自备素材混剪成带字幕的短视频（用户点名，见 [§3](#3-用户点名的项目)） | not-applicable | — | 桌面应用 | 相邻，只作参考。AGPL-3.0；只借签名清单校验和 SRT 导出。 |
| [HisMax/RedInk](https://github.com/HisMax/RedInk) | 5668 / 2026-06-30 / none / NOASSERTION | — | 一句话生成整套小红书风格图文的 Web 应用 | not-applicable | — | 网页应用、Docker | 相邻，不接入。CC BY-NC-SA 4.0 加商业授权条款，非商业。 |
| [Breakthrough/PySceneDetect](https://github.com/Breakthrough/PySceneDetect)（致谢） | 5306 / 2026-10-27 / v0.7.2（2026-10-12） / BSD-3-Clause | — | 基于 OpenCV 的镜头切换检测，输出场景起止时间，可按场景截图或切分 | not-applicable | — | 命令行、库、Docker | 转换插件：包装它的命令行。BSD-3-Clause，可 fork。 |
| [GuanYixuan/pyJianYingDraft](https://github.com/GuanYixuan/pyJianYingDraft) | 4742 / 2026-10-14 / 0.3.1（2026-10-14） / Apache-2.0 | — | 生成剪映草稿文件的 Python 库 | not-applicable | — | 库 | 相邻，不接入。Apache-2.0。 |
| [mdnice/markdown-nice](https://github.com/mdnice/markdown-nice) | 4675 / 2023-10-06 / none / GPL-3.0 | 公众号、知乎、其他 | 支持自定义主题的 Markdown 排版编辑器 | not-applicable | — | 网页应用 | 相邻，不接入。GPL-3.0。 |
| [leaperone/MultiPost-Extension](https://github.com/leaperone/MultiPost-Extension)（致谢） | 3837 / 2026-10-24 / extension-v1.5.0（2026-09-28） / Apache-2.0 | 多平台：抖音、TikTok、快手、头条/西瓜、小红书、B站、YouTube、公众号、视频号、微博、知乎、豆瓣、即刻、X、Instagram、Reddit、Pinterest、播客、其他 | 浏览器扩展：在已登录的页面里自动填写，把内容一键发到几十个平台，对外提供 API | not-applicable | 必须 | 浏览器扩展、HTTP 接口 | 相邻，只作参考。Apache-2.0；只借「网页调本机扩展」的接法。 |
| [geekjourneyx/md2wechat-skill](https://github.com/geekjourneyx/md2wechat-skill) | 3775 / 2026-10-14 / v3.8.0（2026-09-24） / NOASSERTION | 多平台：公众号、知乎、头条/西瓜、其他 | 面向 Agent 的公众号创作与发布命令行，经官方接口推送草稿 | not-applicable | 必须 | 命令行、Agent Skill | 相邻，不接入。基于 BSL 1.1 的源码可见许可，2030 年转 Apache-2.0。 |
| [chengyi-ai/native-subtitle-quote-image](https://github.com/chengyi-ai/native-subtitle-quote-image)（致谢） | 2751 / 2026-10-21 / v2.10.3（2026-10-13） / MIT | YouTube | Agent Skill：按时间点从视频精确取帧、裁出烧录字幕，拼成社交长图并逐张质检（用户点名，见 [§3](#3-用户点名的项目)） | not-applicable | — | Agent Skill、命令行 | 参考：借做法自己写。MIT，可 fork；取帧、坏流和重复帧检测可移植到关键帧插件。 |
| [sun-guannan/VectCutAPI](https://github.com/sun-guannan/VectCutAPI) | 2349 / 2026-10-24 / v1.5.0（2025-09-23） / Apache-2.0 | — | 让 Agent 经 HTTP 或 MCP 创建剪映、CapCut 草稿的接口代码 | not-applicable | — | HTTP 接口、MCP | 相邻，不接入。Apache-2.0。 |
| [Waboodoo/HTTP-Shortcuts](https://github.com/Waboodoo/HTTP-Shortcuts)（致谢） | 1855 / 2026-10-30 / v4.9.0（2026-10-17） / MIT | — | Android 上把分享来的文字或网址填进 HTTP 请求的 App，可直发局域网里的本机服务 | not-applicable | — | 手机应用 | 相邻，只作参考。MIT。 |
| [gyoridavid/short-video-maker](https://github.com/gyoridavid/short-video-maker) | 1448 / 2025-06-21 / none / MIT | — | 以 MCP 和 REST 提供的竖版短视频生成服务 | not-applicable | — | MCP、HTTP 接口、网页应用、Docker | 相邻，不接入。MIT；依赖的 Remotion 对公司有许可要求。 |
| [caol64/wenyan-mcp](https://github.com/caol64/wenyan-mcp)（致谢） | 1351 / 2026-09-20 / v2.0.4（2026-09-20） / Apache-2.0 | 公众号 | 文颜的 MCP 版，把 Markdown 排成公众号样式并上传到草稿箱 | not-applicable | 必须 | MCP、Docker | 相邻，不接入。Apache-2.0。 |
| [porjo/youtubeuploader](https://github.com/porjo/youtubeuploader) | 890 / 2026-08-07 / v1.25.5（2025-08-14） / Apache-2.0 | YouTube | Go 写的 YouTube 上传命令行，经官方 Data API | not-applicable | 必须 | 命令行 | 相邻，不接入。Apache-2.0。 |
| [wkaisertexas/tiktok-uploader](https://github.com/wkaisertexas/tiktok-uploader) | 781 / 2026-02-12 / build-uv-v1.7（2026-02-12） / MIT | TikTok | 基于 Playwright 的 TikTok 上传库和命令行 | not-applicable | 必须 | 命令行、库 | 相邻，不接入。MIT。 |
| [doocs/cose](https://github.com/doocs/cose)（致谢） | 772 / 2026-10-17 / v1.3.6（2026-06-01） / Apache-2.0 | 多平台：公众号、知乎、头条/西瓜、抖音、小红书、微博、B站、豆瓣、X、其他 | doocs/md 的配套扩展，把编辑好的文章一次同步到多个平台 | not-applicable | 必须 | 浏览器扩展 | 相邻，不接入。Apache-2.0。 |
| [funfan0517/MediaPublishPlatform](https://github.com/funfan0517/MediaPublishPlatform) | 166 / 2026-01-21 / none / MIT | 多平台：小红书、视频号、抖音、快手、TikTok、Instagram、B站、其他 | 基于 social-auto-upload 的自媒体发布平台，Playwright 定时批量发布 | not-applicable | 必须 | 网页应用、HTTP 接口 | 相邻，不接入。MIT。 |
| [BadKid90s/Spreado](https://github.com/BadKid90s/Spreado) | 143 / 2026-10-30 / v1.2.0（2026-05-02） / Apache-2.0 | 多平台：抖音、小红书、快手、视频号、B站 | 一键发布到抖音、小红书、快手、视频号的命令行与库，平台以插件目录自动发现 | not-applicable | 必须 | 命令行、库、Agent Skill | 相邻，不接入。Apache-2.0。 |
| [mcncarl/ailu](https://github.com/mcncarl/ailu) | 119 / 2026-10-21 / 0.2.0（2026-08-16） / AGPL-3.0 | 公众号、X、其他 | Obsidian 插件：侧栏调用本机 Claude Code、Codex，把笔记做成公众号、飞书文档、X Article 草稿（用户点名，见 [§3](#3-用户点名的项目)） | not-applicable | 必须 | Obsidian 插件 | 相邻，只作参考。AGPL-3.0，只参考飞书导出的最小权限做法。 |
| [ahang008/ah-skills](https://github.com/ahang008/ah-skills) | 58 / 2026-10-08 / ah-douyin-clean-downloader-v0.1.0（2026-09-21） / NOASSERTION | — | 阿杭的 Codex Skill 目录：独立仓库索引，加短视频制作入口、口播润色与剪辑包，不从平台取数据（用户点名，见 [§3](#3-用户点名的项目)） | not-applicable | 可选 | Agent Skill、命令行 | 相邻，不接入。源码可见的非商业许可：不 fork、不分发。 |
| [mcncarl/wechat-relay](https://github.com/mcncarl/wechat-relay) | 18 / 2026-10-25 / 0.1.0（2026-08-16） / AGPL-3.0 | 公众号 | 自托管的公众号草稿接口中转，只转发四个官方接口，便于登记固定出口 IP（用户点名，见 [§3](#3-用户点名的项目)） | not-applicable | 必须 | HTTP 接口 | 相邻，只作参考。AGPL-3.0：用户自部署后经 HTTP 调用，不复用代码。 |
| [pabatiba89-sys/kali-publish](https://github.com/pabatiba89-sys/kali-publish) | 1 / 2026-10-24 / v1.2.1（2026-10-24） / MIT | 多平台：小红书、视频号、抖音、快手、TikTok、YouTube、X、Instagram、其他 | social-auto-upload 的派生，喀理 AI 的开源发布后端，用本机 Chrome 的登录态发 10 个平台（用户点名，见 [§3](#3-用户点名的项目)） | not-applicable | 必须 | HTTP 接口 | 参考：借做法自己写。MIT，可 fork；只借登录与会话导出的经验；视频号流程写死推广内容，不包装。 |

- **发布都要用户账号，浏览器层也有限流和封禁记录。** social-auto-upload 的小红书发布被判定为 AI 或自动化（[dreammis/social-auto-upload #289](https://github.com/dreammis/social-auto-upload/issues/289)），doocs/cose 的维护者解释扩展在用户自己的浏览器里操作、最终由人工点发布。歇存不做发布，只借会话管理、人工验证交接和 Skill 分发的做法。
- **走官方接口的风险在密钥和配额。** postiz、mixpost、youtubeuploader、wenyan-mcp、md2wechat 经官方 OAuth 或公众号接口，没有封号类风险，要保管好 AppSecret、IP 白名单，未审核的 YouTube 项目上传的视频会被设为私享。
- **生成类与歇存无关。** MoneyPrinterTurbo、short-video-factory、NarratoAI、RedInk 等收录是因为用户点名或作为相邻参考；short-video-factory、MatrixMedia 默认上报使用数据。
- **许可证混杂。** biliup、MoneyPrinterPlus 的 README 或附加文件与 LICENSE 冲突，按非商业处理；RedInk、md2wechat-skill 是非商业或源码可见许可。

<a id="16-账号风险的证据"></a>
## 16. 账号风险的证据

三类证据：平台自己的规则和技术上能看到什么（[§16.1](#161-平台规则与可检测性)）；用户在 issue 里报告的账号后果（[§16.2](#162-issue-里的账号问题)）；项目被下架、停更和相关的法律规则（[§16.3](#163-下架停更与法律风险)）。

<a id="161-平台规则与可检测性"></a>
### 16.1 平台规则与可检测性

条款名后写页面更新日期，没有标日期的按访问日期 2026-10-30。微信个人账号使用规范、微博、知乎、豆瓣、YouTube、Instagram 的条款号来自抓取工具的摘要，没有逐条和原文比对；抖音协议与隐私政策 3.3、快手、小红书、B站、微信软件许可协议 8.2.1.x、公众平台运营规范已在浏览器里对照原文。

| 平台 | 条款 | 管什么 | 处罚 |
|---|---|---|---|
| 抖音 | [《抖音用户服务协议》](https://www.douyin.com/agreements/?id=6773906068725565448)（2026-02-13） | 2.4 未经授权不得以爬虫、镜像等方式使用；5.1 不得用插件、外挂、第三方工具，不得用自动化程序接入抖音收集信息；5.3 列出盗链、爬虫抓取、模拟下载 | 7.1 警示、删除内容、限制部分或全部功能，直至永久关闭账号 |
| 抖音 | [隐私政策](https://www.douyin.com/privacy/)（2026-09-11）3.3 | 可申请「个人信息下载」，链接发到邮箱；政策没有列出包含哪些类别 | — |
| 快手 | [《快手软件许可及服务协议》](https://app.m.kuaishou.com/public/index.html#/protocol/14)（2026-02-04） | 七.1.1 插件、外挂、第三方工具；七.1.4 获取客户端内存数据和交互数据；十.1 机器人、蜘蛛等程序下载内容 | 十五.1 警示、限制功能、关闭账号、禁止重新注册 |
| 快手 | [《社区管理规范》](https://www.kuaishou.com/norm)第六章 | 数据作弊、虚假注册、操纵账号；没有单独写读取内容的自动化 | — |
| 小红书 | [《小红书用户服务协议》](https://agree.xiaohongshu.com/h5/terms/ZXXY20220331001/-1)（2026-03-23） | 4.1 不得以盗链、非法抓取、模拟下载获取内容，不得用非授权的第三方软件登录、使用 | 2.5 限期改正、短期或永久封禁、注销 |
| 小红书 | 「薯管家」AI 托管治理公告（2026-03-10，[澎湃新闻转述](https://www.thepaper.cn/newsDetail_forward_32747698)） | 用技术手段自动生成、发布笔记和模拟真人互动 | 警告、限制分发、封禁；公告没提只读取的自动化 |
| B站 | [《哔哩哔哩弹幕网用户使用协议》](https://www.bilibili.com/protocal/licence.html)（2025-04-30） | 5.6(3) 不得用机器人、爬虫等自动程序获取内容；1.3 非官方渠道取得的客户端、账号可被处理 | 2.7 警示直至永久封号 |
| 微信 | [《腾讯微信软件许可及服务协议》](https://weixin.qq.com/agreement?lang=zh_CN) | 8.2.1.4 插件、外挂；8.2.1.6 借第三方软件登录、自动化操作，或读取软件的界面和数据；8.2.1.7 规避安全保护措施 | 8.5.1 限制功能、封禁直至注销或回收账号 |
| 微信 | [《微信个人账号使用规范》](https://weixin.qq.com/cgi-bin/readtemplate?lang=zh_CN&t=page/agreement/personal_account)（2025-03-28） | 1.2.4–1.2.8 插件、外挂（含抢红包、多开、一键转发）；3.4 异常大量收发消息、频繁加好友 | 冻结部分功能直至永久封号 |
| 公众号 | [《微信公众平台运营规范》](https://mp.weixin.qq.com/mp/opshowpage?action=newoplaw) | 3.1 外挂工具；3.27 非真人自动化创作；5.1.6 抓取内容和用户数据 | 限流、删除、封禁；约束的是运营方，对读者读文章没有专门条款 |
| 微博 | [《微博服务使用协议》](https://weibo.com/signup/v5/protocol) | 1.3.2 不得以程序或非正常浏览获取内容；1.4 遵守 Robots 协议；4.5.1 不得以明显高于正常用户的频率操作，平台可检测验证 | 8.5 删除、禁言、限制功能、注销 |
| 知乎 | [《知乎协议》](https://www.zhihu.com/term/zhihu-terms)（2025-03-20） | 二.9.a 第三方插件、自动化程序接入收集；二.9.b 爬取、模拟下载；三.10 抓取内容用于大模型训练 | 限制、暂停、终止，永久封禁的账号可能无法注销 |
| 豆瓣 | [《豆瓣使用协议》](https://accounts.douban.com/passport/agreement)（2025-05-27） | 10.4 须通过豆瓣界面使用；9.6 不得干扰服务；2.3 可设使用次数上限 | 3.1、6.1 暂停或关闭账号 |
| X | [服务条款](https://x.com/en/tos)（2026-10-09） | 只能经公开接口访问，未经同意禁止任何爬取；24 小时内超过 100 万条帖子按每 100 万条 1.5 万美元计违约金 | 暂停或终止账号 |
| X | [自动化规则](https://help.x.com/en/rules-and-policies/x-automation)（页面日期待核实） | 不用非 API 的自动化，例子是用脚本操作网站 | 过滤出搜索结果或永久封号 |
| YouTube | [服务条款](https://www.youtube.com/t/terms)（2023-12-15） | 未经允许不得下载；不得用机器人、抓取器等自动化方式访问；不得规避安全和防复制功能 | 暂停或终止 Google 账号，可申诉 |
| Instagram | [使用条款](https://help.instagram.com/581066165581870)（2025-01-01） | 4.2 不得未经许可以自动化方式访问、收集信息 | 第 6 节移除内容、停用或终止账号 |
| 法律 | [《个人信息保护法》](http://www.npc.gov.cn/npc/c2/c30834/202108/t20210820_313088.html)第四十五、五十条 | 个人有权查阅、复制自己的个人信息，符合条件可请求转移；处理者要建立便捷的受理机制 | — |

**和获取方式有关的处罚记录。**

- 微信 2019-06 打击基于 Xposed、substrate 的外挂，确认使用的账号限制功能直至限制登录（[IT之家](https://www.ithome.com/0/428/859.htm)）；2019 年上半年对上百万外挂账号做了短期或永久限制，列出的外挂包括一键转发、Android 模拟器、自动抢红包、多开（[安全内参转载](https://www.secrss.com/articles/11978)）。
- 微信 2025-04 点名两类违规服务：代码注入类，和授权后监听、录屏、转储聊天记录的转储类（[IT之家](https://www.ithome.com/0/850/207.htm)）；2026-01 点名录音、录屏、截图、辅助服务读取、解密本地数据库，以及模拟按键、自动化脚本（[快科技转述](https://tech.ifeng.com/c/8qKOznNW4za)，官方原文未取到）。
- 豆包手机助手 2025-12 用系统级注入权限操作微信，用户被退出、无法登录，豆包随后不再支持操作微信（[澎湃新闻](https://www.thepaper.cn/newsDetail_forward_32094236)）。

| 手段或环境 | 平台能看到什么 | 来源 |
|---|---|---|
| Android 无障碍服务 | 任何 App 都能列出已开启和已安装的无障碍服务，不需要权限；API 33 起还能监听变化 | [AccessibilityManager](https://developer.android.com/reference/android/view/accessibility/AccessibilityManager) |
| Android 敏感视图 | API 34 起 App 能让非辅助工具读不到、点不了指定视图 | [View](https://developer.android.com/reference/android/view/View#setAccessibilityDataSensitive%28int%29) |
| Play Integrity | 有 Google Play 服务时，报告正在录屏或控制设备的应用；发现 API Hook 或 root 时判设备不完整 | [文档](https://developer.android.com/google/play/integrity/verdicts)（2026-10-23） |
| USB 调试 | 相关设置项现在对第三方 App 恒返回 0；从哪个版本开始、国内 ROM 是否跟进待核实 | [Settings.Global](https://developer.android.com/reference/android/provider/Settings.Global) |
| 注入的触摸 | 单个触摸事件没有公开的注入标记，App 要结合环境信号和节奏判断 | [MotionEvent](https://developer.android.com/reference/android/view/MotionEvent) |
| uiautomator2 | 辅助应用包名固定，任何 App 都能查到（用户说法，未见处罚） | [openatx/uiautomator2 #808](https://github.com/openatx/uiautomator2/issues/808) |
| Windows | 低级鼠标钩子能看到注入标志；没找到国内客户端据此处罚的资料 | [MSLLHOOKSTRUCT](https://learn.microsoft.com/en-us/windows/win32/api/winuser/ns-winuser-msllhookstruct) |
| PC 微信 4.x | 对从没开过无障碍模式的新账号不暴露 UI Automation 树，企业微信更严 | [pywechat 的说明](https://github.com/Hello-Mr-Crab/pywechat/blob/main/Weixin4.0.md) |
| macOS | 事件里有来源进程 PID 字段，没找到客户端读取它的资料 | [CGEventField](https://developer.apple.com/documentation/coregraphics/cgeventfield/eventsourceunixprocessid) |
| iOS 录屏 | App 能感知录屏、镜像和投屏 | [UIScreen.isCaptured](https://developer.apple.com/documentation/uikit/uiscreen/iscaptured) |
| iPhone 镜像 | 没有公开 API 识别；镜像时 iPhone 锁定并显示正被哪台 Mac 使用；安全文本框的遮罩在镜像下不生效；欧盟不可用 | [Apple 120421](https://support.apple.com/en-us/120421)、[DTS 答复](https://developer.apple.com/forums/thread/759314) |
| 浏览器 | 带 `--enable-automation`、无头模式或开 marionette 时 `navigator.webdriver` 为 true；日常浏览器里的扩展不会让它置位 | [W3C WebDriver](https://w3c.github.io/webdriver/#interface)、[MDN](https://developer.mozilla.org/en-US/docs/Web/API/Navigator/webdriver) |
| 小红书 | Playwright 打开需登录的页面返回 300012；自动化浏览器一两天内被警告；网页对数据采集类扩展弹「有害插件」提示 | [danzey0425-lgtm/web-image-sourcing](https://github.com/danzey0425-lgtm/web-image-sourcing)、[xpzouying/xiaohongshu-mcp #720](https://github.com/xpzouying/xiaohongshu-mcp/issues/720)、[JoeanAmier/XHS-Downloader #283](https://github.com/JoeanAmier/XHS-Downloader/issues/283) |
| 抖音 | 只放行页面内请求；单会话约 35 个作者后 cookie 异常、约一小时恢复；自动化搜索出九宫格验证 | [jiji262/douyin-downloader #258](https://github.com/jiji262/douyin-downloader/issues/258)、[JoeanAmier/TikTokDownloader #478](https://github.com/JoeanAmier/TikTokDownloader/issues/478)、[NanmiCoder/MediaCrawler #996](https://github.com/NanmiCoder/MediaCrawler/issues/996) |
| 豆瓣 | 频繁访问详情页跳到算力验证页，通过后发短时通行 cookie；不区分是否登录 | [doubak-extension 的 pow.js](https://github.com/Doubak/doubak-extension/blob/main/src/crawl/pow.js) |
| YouTube | 访客会话约每小时 300 个视频，账号约每小时 2000 个 | [yt-dlp wiki](https://github.com/yt-dlp/yt-dlp/wiki/Extractors) |
| Instagram | 超过频率返回 429 并临时封禁，云和 VPN IP 更严；登录异常时发 checkpoint 挑战 | [Instaloader 文档](https://instaloader.github.io/troubleshooting.html) |

采集工具普遍在浏览器里隐藏自动化特征（[NanmiCoder/MediaCrawler](https://github.com/NanmiCoder/MediaCrawler) 注入 stealth 脚本，[xpzouying/xiaohongshu-mcp](https://github.com/xpzouying/xiaohongshu-mcp) 换成带指纹参数的浏览器），说明平台会检查这些特征，但平台的具体检测项没有公开资料。

<a id="162-issue-里的账号问题"></a>
### 16.2 issue 里的账号问题

扫了 34 个仓库共 12,585 条 issue 标题（BBDown、bilibili-api、wxauto 的 issue 已关闭，读不到），任务关键词命中 197 条；挑读 183 条正文，确认 80 条是账号层的后果（警告、强制下线、限制、禁言、封号），其中 31 条是禁言或封号。样本是按关键词挑的，各层的数字不能当发生率；标题里的 account blocked、461、「账号异常」常常是签名或代码失效（[NanmiCoder/MediaCrawler #895](https://github.com/NanmiCoder/MediaCrawler/issues/895)），只按读过正文的计数。真正的警告和封号多用「下线」「禁言」「checkpoint」「locked」等词，所以标题过滤之外另做了人工筛选。

| 层级 | 仓库（issue 数 / 命中 / 读过 / 账号层后果） | 看到的后果 | 代表 issue |
|---|---|---|---|
| `public-web` | XHS-Downloader（357 / 9 / 13 / 4，全是强制下线）；Douyin_TikTok_Download_API（647 / 6 / 7 / 0）；yt-dlp（只扫了标题含 B站 等平台名的 287 条，命中 3） | 基本是 IP 层的限制和验证码；小红书例外，匿名批量后复制链接的账号被下线 | [JoeanAmier/XHS-Downloader #305](https://github.com/JoeanAmier/XHS-Downloader/issues/305)、[Evil0ctal/Douyin_TikTok_Download_API #626](https://github.com/Evil0ctal/Douyin_TikTok_Download_API/issues/626)、[yt-dlp/yt-dlp #14857](https://github.com/yt-dlp/yt-dlp/issues/14857)、[instaloader/instaloader #2501](https://github.com/instaloader/instaloader/issues/2501) |
| `vision-ui` | pywechat（257 / 5 / 10 / 6，全是强制下线）；mobilerun（197）、Open-AutoGLM（254）、mobile-use（39）、mirroir-mcp（22）没有账号后果的报告 | PC 微信强制下线、要扫码重登，夜间、云服务器、虚拟机、远程桌面下更容易触发；维护者说没见过因此封号 | [Hello-Mr-Crab/pywechat #260](https://github.com/Hello-Mr-Crab/pywechat/issues/260)、[Hello-Mr-Crab/pywechat #267](https://github.com/Hello-Mr-Crab/pywechat/issues/267)、[Hello-Mr-Crab/pywechat #181](https://github.com/Hello-Mr-Crab/pywechat/issues/181) |
| `browser-session` | xiaohongshu-mcp（434 / 15 / 15 / 13，其中 9 条禁言或封号）；OpenCLI（最近 2500 条里的 600 条 issue / 4）；twitter-web-exporter（136 / 2，没有使用者被锁） | 小红书一次登录、两次采集、一天低频只读都有人被警告或封；换成内置指纹浏览器后仍有报告；复用用户自己 Chrome 的 OpenCLI 也有只读被判违规 | [xpzouying/xiaohongshu-mcp #715](https://github.com/xpzouying/xiaohongshu-mcp/issues/715)、[xpzouying/xiaohongshu-mcp #726](https://github.com/xpzouying/xiaohongshu-mcp/issues/726)、[xpzouying/xiaohongshu-mcp #877](https://github.com/xpzouying/xiaohongshu-mcp/issues/877)、[jackwener/OpenCLI #842](https://github.com/jackwener/OpenCLI/issues/842)、[jackwener/OpenCLI #2208](https://github.com/jackwener/OpenCLI/issues/2208) |
| `session-api` | we-mp-rss（358 / 3，没有封号）；wewe-rss（428 / 7 / 8 / 4，都在微信读书一侧）；wechat-article-exporter（153 / 1）；yutto | 公众号后台的频率控制和会话 2–4 天过期；微信读书「小黑屋」、半天到一天失效；没有微信号被处罚的报告；B站 有一例大会员被冻结，因果不明 | [rachelos/we-mp-rss #469](https://github.com/rachelos/we-mp-rss/issues/469)、[cooderl/wewe-rss #86](https://github.com/cooderl/wewe-rss/issues/86)、[wechat-article/wechat-article-exporter #200](https://github.com/wechat-article/wechat-article-exporter/issues/200)、[yutto-dev/yutto #452](https://github.com/yutto-dev/yutto/issues/452) |
| `private-api` | MediaCrawler（634 / 19 / 22 / 8，其中 3 条封号）；Spider_XHS（136 / 6 / 8 / 5，其中 3 条封号）；ReaJason/xhs（153 / 6，没有封号）；TikTokDownloader（675 / 9 / 9 / 1）；bili-sync（468 / 8 / 6 / 2）；twikit（314 / 15 / 8 / 7，其中 4 条封号）；twscrape（266 / 10 / 7 / 6）；instaloader（2399 / 22 / 7 / 4）；instagrapi（1166 / 12）；Agent-Reach（269 / 5） | 小红书最重；抖音以 1–12 小时的按功能限制为主；B站 常规风控会自行解除，个别账号持续半年；X 老主号低量使用也被封，申诉很少恢复；Instagram 备份自己的帖子后永久封号 | [NanmiCoder/MediaCrawler #865](https://github.com/NanmiCoder/MediaCrawler/issues/865)、[cv-cat/Spider_XHS #150](https://github.com/cv-cat/Spider_XHS/issues/150)、[Johnserf-Seed/f2 #406](https://github.com/Johnserf-Seed/f2/issues/406)、[amtoaer/bili-sync #756](https://github.com/amtoaer/bili-sync/issues/756)、[d60/twikit #338](https://github.com/d60/twikit/issues/338)、[instaloader/instaloader #2680](https://github.com/instaloader/instaloader/issues/2680) |
| `client-hook` | WeChatFerry（306 / 6 / 7 / 5，其中 4 条封号）；wx_channels_download（471 / 1，没有微信账号被处罚）；res-downloader（322 / 1，与账号无关） | 注入进程：警告、强制下线、封 7 天、再犯被警告永久封号，按实名牵连大号，只读监听也会被处理；本机代理类的风险主要在证书安全：有工具附带全体用户共用的根证书和私钥 | [lich0821/WeChatFerry #126](https://github.com/lich0821/WeChatFerry/issues/126)、[lich0821/WeChatFerry #422](https://github.com/lich0821/WeChatFerry/issues/422)、[ltaoo/wx_channels_download #586](https://github.com/ltaoo/wx_channels_download/issues/586) |
| `official-export` | 没有收集到 issue 证据 | — | — |

**规律。**

- **小红书的风控在变严。** 2024-03 维护者说封禁只封网页端、几小时解开（[NanmiCoder/MediaCrawler #156](https://github.com/NanmiCoder/MediaCrawler/issues/156)）；2025 年多是约 24 小时的取数限制（[JoeanAmier/XHS-Downloader #224](https://github.com/JoeanAmier/XHS-Downloader/issues/224)），并开始出现网页和手机同时被登出（[NanmiCoder/MediaCrawler #747](https://github.com/NanmiCoder/MediaCrawler/issues/747)）；2026-04 起是警告、禁言 7 天或封一周、永久封禁，申诉要实名（[xpzouying/xiaohongshu-mcp #674](https://github.com/xpzouying/xiaohongshu-mcp/issues/674)、[xpzouying/xiaohongshu-mcp #316](https://github.com/xpzouying/xiaohongshu-mcp/issues/316)）；有开发者说自己插件的风控周期从约 14 天缩到 1 天（[xpzouying/xiaohongshu-mcp #668](https://github.com/xpzouying/xiaohongshu-mcp/issues/668)）。
- **抖音多是临时限制。** 搜索不可用约 12 小时、cookie 异常约 1 小时、作品页失效几小时、IP 约两小时解封，外加滑块和九宫格验证；维护者 2024 年说一般不封号（[Evil0ctal/Douyin_TikTok_Download_API #417](https://github.com/Evil0ctal/Douyin_TikTok_Download_API/issues/417)）。读到的 issue 里没有抖音账号被封禁的明确报告。
- **云主机、虚拟机和 IP 跳变在各层都加重风险。** 小红书在阿里云服务器上定时发帖被处理（[xpzouying/xiaohongshu-mcp #680](https://github.com/xpzouying/xiaohongshu-mcp/issues/680)）；B站 对 AWS IP 返回 412（[Panniantong/Agent-Reach #180](https://github.com/Panniantong/Agent-Reach/issues/180)）；PC 微信在云主机、虚拟机、远程桌面下更容易被踢（[Hello-Mr-Crab/pywechat #158](https://github.com/Hello-Mr-Crab/pywechat/issues/158)）；X 的维护者建议注册和使用保持同一 IP 段（[Panniantong/Agent-Reach #63](https://github.com/Panniantong/Agent-Reach/issues/63)）。
- **写操作比读重，但只读不是安全线。** OpenCLI 维护者建议小红书只读不写（[jackwener/OpenCLI #677](https://github.com/jackwener/OpenCLI/issues/677)）；X 上发推后立刻被封、读时间线正常（[d60/twikit #318](https://github.com/d60/twikit/issues/318)）；Instagram 一小时评论 7 次被封（[subzeroid/instagrapi #2097](https://github.com/subzeroid/instagrapi/issues/2097)）。可小红书 2026 年只读也会被警告或封号。
- **发布类的处罚不能直接套到读取上。** social-auto-upload 被判定自动化发布（[dreammis/social-auto-upload #289](https://github.com/dreammis/social-auto-upload/issues/289)）、无头登录被识别（[dreammis/social-auto-upload #224](https://github.com/dreammis/social-auto-upload/issues/224)），是发布内容引起的处理。

<a id="163-下架停更与法律风险"></a>
### 16.3 下架、停更与法律风险

下架风险按证据强弱排，从高到低：解密本地数据或 DRM；复现平台签名、逆向客户端与网页的安全逻辑；涉及付费、会员或版权内容；包含平台自己的代码；规模与姿态（几千到几万星、作为官方功能内置、持续更新、随版本分发）；依赖作者托管的服务（上游一变或作者停手就整体失效）。只读公开分享页、页内请求、被动截获页面请求、UI 自动化、iOS 备份解析、官方导出这几类没有找到来函或下架记录；抖音、小红书没有向 GitHub 上的工具发函的公开记录，但这只说明没有公开。

| 日期 | 对象 | 发起方与方式 | 结果 |
|---|---|---|---|
| 2020-10-23 | [ytdl-org/youtube-dl](https://github.com/ytdl-org/youtube-dl) | RIAA 以规避技术措施为由发 DMCA | 2020-11-16 恢复；GitHub 改为先做技术和法律审查、给仓库主修改机会（[GitHub 博客](https://github.blog/news-insights/policy-news-and-insights/standing-up-for-developers-youtube-dl-is-back/)） |
| 2022 | unlock-music（解密下载的音乐文件） | QQ 音乐 DMCA，按超过 100 个仓库的网络处理 | 主仓库 2022-11-08 起返回 451 |
| 2023-03-14 | TikTok 网页端逆向仓库（[xtekky/TikTok-Web-Reverse](https://github.com/xtekky/TikTok-Web-Reverse) 等） | TikTok DMCA（[通知](https://github.com/github/dmca/blob/master/2023/03/2023-03-14-tiktok.md)），2025-05-12 又有一份 | 反规避主张证据不足，其他版权主张成立 |
| 2023-06 至 07 | 反编译的早期抖音、TikTok 安装包源码 | 字节跳动 20 份 DMCA | 针对平台自己的代码 |
| 2023-09-08 | [junhoyeo/threads-api](https://github.com/junhoyeo/threads-api) | Meta 停止侵权函 | 停止开发并归档 |
| 2024-01-13 | Tachiyomi | Kakao Entertainment 对开发者的威胁（[组织页公告](https://github.com/tachiyomiorg)） | 停止开发和官方扩展；后继的 Mihon 不内置扩展仓库（[02 §1](02-adapter-ecosystems.md#1-tachiyomi-与-mihon-扩展生态)） |
| 2024-01-22 | [Binaryify/NeteaseCloudMusicApi](https://github.com/Binaryify/NeteaseCloudMusicApi) | 网易云音乐的法律通知（[蓝点网](https://www.landian.news/archives/101953.html)） | 重置历史后归档 |
| 2025-01、2025-04 | mangayomi、cloudstream 的发布安装包 | 优酷 DMCA | 两个仓库仍在推送 |
| 2025-10-20 | [xaoyaoo/PyWxDump](https://github.com/xaoyaoo/PyWxDump)、[sjzar/chatlog](https://github.com/sjzar/chatlog) | 微信官方的律师函和函件 | 自删代码和提交历史；之后副本、续作继续被 DMCA |
| 2026-01-08 | 29 个聊天库解密仓库（含 WeChatMsg 的副本、wechatDataBackup、wechat-dump） | 腾讯 DMCA，按 4,195 个仓库的网络处理（[通知](https://github.com/github/dmca/blob/master/2026/01/2026-01-08-tencent.md)） | 屏蔽；理由含本地库加密属于技术措施、协议禁止逆向和自动化。wechat-dump 修改 README 后保留 |
| 2026-01-28 | [SocialSisterYi/bilibili-API-collect](https://github.com/SocialSisterYi/bilibili-API-collect) | B站 委托律所发律师函 | 删除文档并归档，只有接口文档也在维权范围内 |
| 2026-02-04 | aniyomi-extensions（第三方扩展仓库） | Crunchyroll DMCA | 2026-02-05 起返回 451 |
| 2026-05-14 | [nilaoda/BBDown](https://github.com/nilaoda/BBDown) | 不明 | 删除源码并归档，没写原因 |
| 2026-05-27 | [gedoor/legado](https://github.com/gedoor/legado) | 阅文相关的侵权公告 | 清空成一份公告；应用本身不带书源 |
| 2026-06 | [erma0/douyin](https://github.com/erma0/douyin) | 称「合规审查」，发起方没写 | 清空代码 |
| 2026-06-29 至 10-13 | 36 个解密、取密钥的仓库（含 WeFlow、jackwener/wx-cli、chatlog 的续作） | 腾讯 DMCA，其中 9 份按派生网络处理（[示例](https://github.com/github/dmca/blob/master/2026/07/2026-07-13-wechat.md)） | wx-cli 2026-07-15 起返回 451；WeFlow 去掉内置解密组件、改由用户自备后仍在 |
| 2026-07-06 | [Nemo2011/bilibili-api](https://github.com/Nemo2011/bilibili-api)、[leiurayer/downkyi](https://github.com/leiurayer/downkyi)、[btjawa/BiliTools](https://github.com/btjawa/BiliTools)、[yujincheng08/BiliRoaming](https://github.com/yujincheng08/BiliRoaming) | B站 委托律所直接发函，不经 github/dmca | 关停、归档或删源码；PyPI 上的 bilibili-api-python 17.4.2 仍可安装，不再更新 |
| 2026-07-10 | [nICEnnnnnnnLee/BilibiliDown](https://github.com/nICEnnnnnnnLee/BilibiliDown) | 不明（在 07-06 律师函之后四天） | 清空成空 README，release 仍可下载 |
| 2026-07-24 | [iszhouhua/social-media-copilot](https://github.com/iszhouhua/social-media-copilot) | Chrome 应用商店下架，原因未公开 | 换新的商店地址重新上架 |
| 2026-07-30 | [wechat-article/wechat-article-exporter](https://github.com/wechat-article/wechat-article-exporter) | 微信关闭上游接口 | 停止维护；技术原因，不是函件 |
| 2026-08-24 | [zedeus/nitter](https://github.com/zedeus/nitter) | X Corp. 停止侵权函 | 归档 |
| 2026-10-23 | [mcncarl/yichen-skills](https://github.com/mcncarl/yichen-skills) 里的剪映 Skill | 律师函，发起方没写 | 删除；上游仓库已 404 |

**B站 律师函点名了什么。** 函件主张 B站 对接口、技术架构、安全认证和访问控制享有权利，点名的行为包括：对非公开接口、认证体系、访问控制、付费内容保护和会员体系做逆向适配，针对登录态、反爬风控和签名鉴权做适配，抓取番剧和付费课程，采集弹幕并反查发送者身份。函件认为把这些能力作为官方功能内置、持续更新、随每次版本分发，已超出开源项目被动托管的范围；依据写的是《民法典》《反不正当竞争法》《网络安全法》；要求 2 日内回复、删除代码、停止接受相关贡献、公开声明功能未获授权（[bilibili-api 公开的函件](https://github.com/Nemo2011/bilibili-api)、[BiliRoaming](https://github.com/yujincheng08/BiliRoaming)）。

**法律规则与判例**（只摘来源，不是法律意见）：

| 规则或判例 | 要点 | 来源 |
|---|---|---|
| 最高法指导性案例 262 号（2025-08-28） | 未经许可获取平台数据集合并向公众提供、实质性替代平台的，可按《反不正当竞争法》第二条认定不正当竞争 | [最高人民法院](https://www.court.gov.cn/zixun/xiangqing/474431.html) |
| 指导性案例 263 号 | 经用户授权、在合理范围内把用户在关联平台上的数据转过来，没有扰乱竞争秩序的，不构成不正当竞争；这是和用户带走自己数据最接近的司法规则 | [最高人民法院](https://www.court.gov.cn/shenpan/xiangqing/474461.html) |
| 抖音诉刷宝 | 搬运 5 万余条视频、用户信息和评论，判赔 500 万元 | [新京报](https://m.bjnews.com.cn/detail/1694666687168863.html) |
| 腾讯诉聚客通（2020） | 群控外挂嵌入个人微信、抓取账号信息和好友关系链，判赔 260 万元；据律所评析，法院区分了整体数据资源和单个用户的数据 | [杭州网](https://hznews.hangzhou.com.cn/shehui/content/2020-06/02/content_7747527.htm)、[中伦评析](https://www.zhonglun.com/research/articles/9092.html) |
| 「数据精灵」外挂（2019） | 批量加粉、一键点赞转发等外挂，一审判赔 500 万元 | [安全内参转载](https://www.secrss.com/articles/11452) |
| 微博诉脉脉（2016） | 经开放接口取用户信息，要用户授权、平台授权、再次取得用户授权 | [中伦](https://www.zhonglun.com/research/articles/9519.html) |
| 小红书诉厦门某公司（2025） | 抓取用户和内容数据加工后出售，赔偿 490 万元 | [南都](https://m.mp.oeeee.com/a/BAAFRD0000202504271078619.html) |
| 常州小红书爬虫刑事案 | 突破前端验证抓取后批量发营销私信，单位罚金、三人缓刑；属本报告排除的用途 | [安全内参转载](https://www.secrss.com/articles/72097) |
| 海淀爬虫刑事案（2018） | 避开对方的防抓取措施进入其系统，以非法获取计算机信息系统数据罪定罪 | [安全内参转载](https://www.secrss.com/articles/7518) |
| 上海普陀案（2025） | 有偿提供专门针对验证机制的爬虫程序和接口的人，被单独以提供侵入计算机信息系统程序罪判刑 | [安全内参转载](https://www.secrss.com/articles/83484) |
| 法释〔2011〕19号 | 非法获取数据罪的入罪门槛；没有为个人用途设例外，看行为方式和情节 | [最高人民法院](https://www.court.gov.cn/fabu/xiangqing/3085.html) |
| 《反不正当竞争法》第十三条第三款（2025-10-15 施行） | 经营者不得避开或者破坏技术管理措施获取他人合法持有的数据；对个人非经营性的备份是否适用，条文没有直接回答 | [新华网](https://www.news.cn/legal/20250627/4b6ec78bc9be4ea9a2968a9d34abd724/c.html) |
| 《网络数据安全管理条例》第十八、六十三条 | 用自动化工具收集要评估影响、不得干扰服务；自然人因个人或家庭事务处理个人信息的不适用 | [网信办](https://www.cac.gov.cn/2024-09/30/c_1729384452307680.htm) |
| 《个人信息保护法》第七十二、十三、二十七条 | 个人或家庭事务不适用；合理范围内处理已公开的个人信息可不经同意 | [网信办](https://www.cac.gov.cn/2021-08/20/c_1631050028355286.htm) |
| 《数据安全法》第三十二条 | 收集数据要用合法、正当的方式，没有个人用途例外 | [国家统计局转载](https://www.stats.gov.cn/gk/tjfg/xgfxfg/202503/t20250310_1958928.html) |
| 国内爬虫涉案汇总 | 所列都是商业化、批量或涉及他人信息的案件，没有一例是个人备份自己的收藏 | [hiddendevj/Crawler_Illegal_Cases_In_China](https://github.com/hiddendevj/Crawler_Illegal_Cases_In_China) |

**项目方的做法和教训。**

- **把签名交给用户自备。** DouK-Downloader（[JoeanAmier/TikTokDownloader](https://github.com/JoeanAmier/TikTokDownloader)）2026-07 起不再维护加密参数算法，需要时由用户自备生成代码，并写明为确保合法合规。
- **只在页面里取本人数据。** [Hannah2227/douyin-favorite-MyLike](https://github.com/Hannah2227/douyin-favorite-MyLike) 只在应用打开的真实抖音页面里发请求，免责声明要求只整理本人账号的数据；[prinsss/twitter-web-exporter](https://github.com/prinsss/twitter-web-exporter) 只截获页面自己加载的数据。这两类没有找到来函记录。
- **拒收侵权站点和只有 DRM 内容的服务。** yt-dlp 的[贡献指南](https://github.com/yt-dlp/yt-dlp/blob/master/CONTRIBUTING.md)沿用 youtube-dl 的政策；2020 年恢复后，按文件名检索没有再出现针对它的通知。
- **宿主不内置来源，加一份下架清单。** keiyoushi 用置顶 issue 记录因 DMCA 移除的扩展，写明不再加回（[keiyoushi/extensions-source #14260](https://github.com/keiyoushi/extensions-source/issues/14260)）；Mihon 本身不内置扩展仓库；[recloudstream/cloudstream](https://github.com/recloudstream/cloudstream) 要求不做托管版权内容的扩展。legado 的结果说明这不是保证。
- **非商业许可挡不住追责。** yichen-skills 有自写的非商业许可，仍收到律师函；这类许可限制的是再分发和商业使用。
- **收函后的公告写法。** BiliTools 写明停止维护是综合法律风险、平台规则和维护成本后的决定，公告不构成对任何法律主张的承认（[btjawa/BiliTools](https://github.com/btjawa/BiliTools)）。
- **fork 保存不可靠。** 派生超过 100 个且通知方主张派生同样侵权时，GitHub 按整个派生网络处理，被屏蔽的仓库返回 451（[示例](https://github.com/github/dmca/blob/master/2026/07/2026-07-07-wechat.md)）。B站 的函件直接发给维护者，派生仓库是否被追没有公开记录。
- **源码删除后发布物还在。** bilibili-api-python、BBDown、downkyi、BilibiliDown 的发布包仍可下载，依赖它们的下游拿不到后续修复，法律状态也不清楚。

<a id="17-对歇存的启示"></a>
## 17. 对歇存的启示

以下都是提议。清单、契约和设置的改动留到实现插件运行器时再做，这次不改 [SPEC §8](../SPEC.md#8-适配器协议-v0)、ADR-003 和 product-model。

<a id="171-清单里声明获取层级"></a>
### 17.1 清单里声明获取层级

在 [ADR-003 §3](../decisions/ADR-003-plugins.md#3-清单) 定的统一清单上，给 `kind: fetch` 的插件加两个必填字段，再由宿主推出第三个：

- `access`：七层取值之一（[约定](#约定)），写默认路径用到的最高一层；
- `login`：`none`、`optional`、`required`；
- `risk`：不写进清单，由宿主按 `access`、`login` 和平台修正表推出，插件作者不能自己声明低风险。

一个插件有多条路径时（例如列表走登录侧、单条走公开页），在每个能力上分别声明；顶层的值取各能力里最高的一档，用于列表展示和安装确认。下面的例子是一个按 [§1.3](#13-混合路径视觉层拿链接公开页面取内容) 混合路径写的抖音插件：

```yaml
# 提议，字段名在实现插件运行器时定稿
id: someone/xiecun-fetch-douyin
name: 抖音收藏
version: 0.1.0
protocol: 1
kind: fetch
license: MIT
homepage: https://github.com/someone/xiecun-fetch-douyin
platforms: [douyin]
match: ["v.douyin.com/*", "www.douyin.com/video/*", "www.douyin.com/user/self*"]
access: browser-session          # 默认路径用到的最高一层
login: required                  # none | optional | required
capabilities:
  list:                          # 列本人收藏：在用户已登录的页面里读
    access: browser-session
    login: required
  fetch:                         # 按链接取单条：读公开分享页，不带 cookie
    access: public-web
    login: none
permissions:
  network: {mode: allowlist, hosts: ["*.douyin.com", "*.douyinvod.com"]}
  fs.write: ["${OUT}"]
  exec: none
  env: []
  limits: {timeout: 120s, memory: 512MB}
```

**risk 怎么推出。** 先按层级和是否带账号定基础档，再按平台修正（依据见 [§1.1](#11-七个层级)、[§16.2](#162-issue-里的账号问题)）：

| access | 不带账号 | 带账号 |
|---|---|---|
| `official-export` | 无 | 无 |
| `public-web` | 无（只有 IP 层） | 不适用：带了账号就按 `session-api` 算 |
| `vision-ui` | 无（例如只录本机播放的声音） | 低 |
| `browser-session` | 不适用：不登录的自动化浏览器按 `public-web` 算 | 低（用户日常浏览器）到中（自动化浏览器） |
| `session-api` | 不适用 | 中 |
| `private-api` | 无账号风险，但项目有下架和法律风险 | 高 |
| `client-hook` | 法律风险高（例如只读客户端缓存） | 很高 |

平台修正：小红书的 `browser-session`（自动化浏览器）记高，`public-web` 批量取记低；微信的 `vision-ui` 记中；抖音、B站 的 `private-api` 记中。修正表会随证据变化，提议和屏蔽清单一样放在主仓库发布，客户端缓存后使用，不必为改一档风险发新版本（待决）。

**和 `permissions` 怎么配合**（[ADR-003 §4](../decisions/ADR-003-plugins.md#4-权限与隔离)）：

- `login: none` 的能力拿不到任何凭据：宿主不传 cookie、不注入令牌，`permissions.env` 不能声明凭据类变量。
- 要登录的能力，凭据只由宿主在调用时传入（临时文件或标准输入），插件不自己读浏览器的 cookie 库；用户选「用浏览器里登录着的」时由宿主读取，只传对应域名。
- 只读导出文件的 `official-export` 插件，`network` 应为 `none`。
- `vision-ui` 插件要用到 ADB、iOS 设备、屏幕录制、辅助功能，现有 `permissions` 没有对应项，提议加一个设备类权限。
- `client-hook` 插件：宿主不代为安装根证书、不注入进程，安装时单独确认，不自动更新。
- 新版本调高了 `access` 或 `login`，按 ADR-003「扩大权限」处理：这个版本先停用，等用户重新同意。
- GitHub App 的校验可以加几条静态检查：`login: none` 却声明了凭据变量；`access` 比上一版高；获取插件里出现发帖、点赞、私信这类写命令（只做提示）。

<a id="172-设置里怎么让用户选"></a>
### 17.2 设置里怎么让用户选

- **每个平台一张列表。** 列出这个平台已安装的获取方式，每行显示层级、「不用账号」或「用账号」、风险档、来源（获取方式都是第三方：从单独的获取方式组织装的标「第三方来源 · 组织名」，按 owner/repo 装的标「第三方 · 未经歇存审核」，写法随 ADR-003 修订定）、GitHub App 校验结果、最近一次失败原因。不登录的方式标「不用账号」，并说明只有 IP 层的限制。
- **按风险从低到高排序，默认先试低风险的。** 同一档按安装顺序，用户可以拖动调整。
- **高风险层单独确认。** 启用 `private-api`、`client-hook` 时弹出说明：这一层做什么、平台规则怎么写、已有哪些处罚报告（链到证据）；`client-hook` 另外说明要用户自己装证书或注入，歇存不代办。微信上的视觉层也单独确认。
- **失败后自动换方式时不往上换。** [SPEC §8](../SPEC.md#8-适配器协议-v0) 的运行器是「失败换下一个、全部失败停在 link」；提议默认只在同档或更低档里换。遇到需要登录、限流、平台拦截时，不换到更高风险的层，也不换后端重试，停在 `link` 并提示用户（做法同 Agent-Reach 的终止码）；被平台拦截后，这个平台的自动化整体冷却一段时间。
- **节奏全局管。** 每个平台一个节奏上限（单条触发、间隔、每日上限），所有获取方式共用；「跟着收」的订阅用到要登录的方式时，默认频率更低并提示。
- **账号来源让用户选。** 「复用日常浏览器」和「另起专用浏览器资料目录」两种做法，证据看法不一（[§11](#11-agent-接入层)），提议两种都给，并写明各自已知的问题。

和 [product-model §8.1](../product-model.md#81-设置清单) 现有设置项的关系：

| 现有设置 | 提议 |
|---|---|
| 每个平台用哪些获取方式、按什么顺序试（默认按安装顺序） | 默认改为按风险从低到高；新增「允许自动换到更高风险的方式」，默认关 |
| 登录方式：不需要 / 用浏览器里登录着的 / cookie 文件 | 加「专用浏览器资料目录」；每个获取方式旁显示它会不会用到账号 |
| 获取方式来源列表 | ADR-003 已提出改为按仓库安装的插件列表，标「第三方 · 未经歇存审核」；再加层级、风险两列 |
| 自动更新获取方式 | 新版本调高层级或登录要求时，按扩大权限处理 |
| 跟着收：订阅列表和频率 | 用到要登录的方式时单独提示，默认频率更低 |
| （新增）平台节奏上限 | 每个平台一组，所有获取方式共用 |

<a id="173-首批自己适配的插件"></a>
### 17.3 首批自己适配的插件

挑选原则：先补零风险、低风险的层；许可证允许（MIT、Apache、BSD 一类可以 fork，GPL 系只包装用户自装的副本）；维护活跃；已有命令行、HTTP 或 MCP，好包装。下面的获取插件都不进官方组织，按用户的方向放进单独的获取方式组织（见 [§17.4](#174-fork-与许可证)），仓库命名暂用 `xiecun-fetch-<平台>`，产品改名后跟着换；协议按 [SPEC §8](../SPEC.md#8-适配器协议-v0) 是命令行子命令，上游只有 HTTP 或 MCP 的，插件里写一层薄的命令行转接。

| 平台 | 层级 | 推荐 | 理由 | 包装方式 |
|---|---|---|---|---|
| 通用（导入） | `official-export` | 自写导出包导入插件，参照 [purarue/google_takeout_parser](https://github.com/purarue/google_takeout_parser)、[meizuflux/scrollback](https://github.com/meizuflux/scrollback)、[commitmaniac/tdp](https://github.com/commitmaniac/tdp)、[karlicoss/rexport](https://github.com/karlicoss/rexport)、[akosbalasko/yarle](https://github.com/akosbalasko/yarle)、[FlyingWolFox/Netscape-Bookmarks-File-Parser](https://github.com/FlyingWolFox/Netscape-Bookmarks-File-Parser) | 零账号风险；都是 MIT、Apache 或 BSD；补齐多数平台空着的官方导出层 | Go 命令行，读用户选的导出包 |
| 抖音 | `official-export` | 导入「个人信息下载」档案，参照 [ZhangYukwiver/DouyinDataReview](https://github.com/ZhangYukwiver/DouyinDataReview) 的导入路径 | 零风险；MIT；导出包含的类别待核实 | 命令行 |
| 抖音 | `public-web` | 包装 [wujunwei928/parse-video](https://github.com/wujunwei928/parse-video) | MIT、Go，已有命令行和 HTTP；不登录取单条 | 命令行 |
| 抖音 | `browser-session` | fork [Hannah2227/douyin-favorite-MyLike](https://github.com/Hannah2227/douyin-favorite-MyLike) 的页内请求部分，或 [mlbb229229-create/douyin-favorites-mcp](https://github.com/mlbb229229-create/douyin-favorites-mcp) | MIT；页内请求是抖音收藏现在最稳的一层；去掉写操作 | 命令行 |
| 快手 | `public-web` | 移植 [Downvid/downvid](https://github.com/Downvid/downvid) 的快手解析器；或包装用户自装的 [JoeanAmier/KS-Downloader](https://github.com/JoeanAmier/KS-Downloader) 本地 API | 快手目前只有这一层能用；Downvid 是 MIT，KS-Downloader 是 GPL | 命令行 / HTTP |
| 小红书 | `public-web` | 移植 [chenjin-cmd/xhs-account-teardown](https://github.com/chenjin-cmd/xhs-account-teardown) 的单条脚本；或包装用户自装的 [JoeanAmier/XHS-Downloader](https://github.com/JoeanAmier/XHS-Downloader) 本地 API | 不登录；单条、低频 | 命令行 / HTTP |
| 小红书 | `browser-session` | fork [wkwunju/xhs-vault](https://github.com/wkwunju/xhs-vault) 的油猴导出，按 yichen-skills 的收藏页采集规则（[§3](#3-用户点名的项目)）补完成判定；最低一档参照 [themrv1ck/xiaohongshu-web-collection-organizing](https://github.com/themrv1ck/xiaohongshu-web-collection-organizing) 只读可见卡片 | MIT；列表在用户自己的浏览器里，单条不带账号 | 用户脚本加本机命令行 |
| B站 | `browser-session` | fork [ayasa520/bilibili-favorites-exporter](https://github.com/ayasa520/bilibili-favorites-exporter) | MIT，体量小，不签名 | 扩展导出 JSON，命令行导入 |
| B站 | `session-api` | 包装用户自装的 [yutto-dev/yutto](https://github.com/yutto-dev/yutto)，只开放收藏夹、稍后再看、普通投稿 | 仍在维护；GPL 只做独立进程 | 命令行 |
| YouTube | `official-export` | Takeout 导入，补上稍后观看的 CSV（参照 [ItsMaciek/WatchNow](https://github.com/ItsMaciek/WatchNow)） | 零风险 | 命令行 |
| YouTube | `private-api`，不登录 | 包装 [yt-dlp/yt-dlp](https://github.com/yt-dlp/yt-dlp) 和 [jdepoix/youtube-transcript-api](https://github.com/jdepoix/youtube-transcript-api)，不传 cookie | Unlicense、MIT；没有账号风险 | 命令行 |
| 公众号 | `public-web` | fork [wechat-article/wechat-article-exporter](https://github.com/wechat-article/wechat-article-exporter) 的单篇管线，或 [fengxxc/wechatmp2markdown](https://github.com/fengxxc/wechatmp2markdown)（Go） | MIT；不登录；正对应 [SPEC §8](../SPEC.md#8-适配器协议-v0) 的示例 | 命令行 |
| 公众号（微信收藏里的文章） | `vision-ui` | macOS fork [pengyulong/wechat-favorites-exporter](https://github.com/pengyulong/wechat-favorites-exporter)；Windows 包装用户自装的 [Hello-Mr-Crab/pywechat](https://github.com/Hello-Mr-Crab/pywechat) | 目前唯一现成的收藏读取流程；MIT / LGPL；要单独确认微信风险 | 命令行 / MCP |
| 微信聊天与收藏 | `official-export` | 移植 [freestylefly/WeChatBridge](https://github.com/freestylefly/WeChatBridge) 的合并转发接收；iPhone 备份走用户自装的 [BlueMatthew/WechatExporter](https://github.com/BlueMatthew/WechatExporter) | 走微信自己的功能；MIT / GPL | 监听文件夹 / 命令行 |
| 视频号 | `browser-session` | 移植 [joeseesun/qiaomu-clipper](https://github.com/joeseesun/qiaomu-clipper) 的元宝页内取凭证 | MIT；不装证书、不注入，登录态不出浏览器；风险在元宝账号，界面要写明 | 浏览器扩展 |
| 微信读书 | `official-export` | 包装 [Tencent/WeChatReading](https://github.com/Tencent/WeChatReading) 的个人 API Key 接口 | 官方接口 | HTTP |
| 微博 | `session-api` | fork [Chilfish/Weibo-archiver](https://github.com/Chilfish/Weibo-archiver) | Apache-2.0；导出本人收藏 | 用户脚本加命令行 |
| 知乎 | `official-export` | 按知乎开放平台官方文档自己写（参照 [klarkxy/zhihu-search](https://github.com/klarkxy/zhihu-search)） | 官方接口；它的许可附加了致谢要求，自己写更干净 | HTTP |
| 知乎 | `session-api` | fork [handsomestWei/zhihu-fetch-skill](https://github.com/handsomestWei/zhihu-fetch-skill) | MIT；收藏夹 | 命令行 |
| 豆瓣 | `public-web` | fork [cosformula/douban-sync-skill](https://github.com/cosformula/douban-sync-skill) | MIT；公开 RSS 增量 | 命令行 |
| X | `official-export`、`browser-session` | 包装 [steipete/birdclaw](https://github.com/steipete/birdclaw) 导入官方归档；书签参照 [prinsss/twitter-web-exporter](https://github.com/prinsss/twitter-web-exporter) | MIT；零或低风险 | 命令行 / 用户脚本 |
| Instagram | `official-export` | 参照 [meizuflux/scrollback](https://github.com/meizuflux/scrollback) 读官方导出 | MIT | 命令行 |
| Reddit | `official-export` | 包装 [karlicoss/rexport](https://github.com/karlicoss/rexport) | MIT；官方接口 | 命令行 |
| Telegram | `official-export` | fork [knadh/tg-archive](https://github.com/knadh/tg-archive)，改成取收藏（Saved Messages） | MIT；官方接口 | 命令行 |
| 播客 | `public-web` | 包装 [lightpohl/podcast-dl](https://github.com/lightpohl/podcast-dl)；订阅列表按 OPML 导入 | MIT | 命令行 |
| 通用网页 | `public-web` | 用 [Readeck 的 go-readability](https://codeberg.org/readeck/go-readability)、[markusmobius/go-trafilatura](https://github.com/markusmobius/go-trafilatura)、[JohannesKaufmann/html-to-markdown](https://github.com/JohannesKaufmann/html-to-markdown) 写网页通用获取 | MIT、Apache；纯 Go | 命令行 |

转换和导出可以进官方组织，方向沿用 [05 §6](05-note-apps-and-plugins.md#6-对歇存插件化架构的启示)：转写用 [ggml-org/whisper.cpp](https://github.com/ggml-org/whisper.cpp) 或 [k2-fsa/sherpa-onnx](https://github.com/k2-fsa/sherpa-onnx) 加 SenseVoice；OCR 用 [RapidAI/RapidOCR](https://github.com/RapidAI/RapidOCR)，macOS 上 fork [bytefer/macos-vision-ocr](https://github.com/bytefer/macos-vision-ocr) 调系统 OCR；PDF 和多语种的第二引擎 [datalab-to/indra](https://github.com/datalab-to/indra) 等权重条款的法律意见；关键帧可以移植 [chengyi-ai/native-subtitle-quote-image](https://github.com/chengyi-ai/native-subtitle-quote-image) 的取帧和质检；带下载的转写工具只包装本地文件模式（[wendy7756/AI-Video-Transcriber](https://github.com/wendy7756/AI-Video-Transcriber)）。

**通用视觉层获取插件：设想。** 做一个 `access: vision-ui`、`login: required` 的获取插件，分三段：

1. **设备通道**：iPhone 用 [jfarcand/mirroir-mcp](https://github.com/jfarcand/mirroir-mcp)（经 macOS 的 iPhone 镜像）或 Go 原生的 [danielpaulus/go-ios](https://github.com/danielpaulus/go-ios)；Android 用 [Genymobile/scrcpy](https://github.com/Genymobile/scrcpy) 加只依赖 adb 的 [CursorTouch/Android-MCP](https://github.com/CursorTouch/Android-MCP)，或 [openatx/uiautomator2](https://github.com/openatx/uiautomator2)；电脑客户端用 [openclaw/Peekaboo](https://github.com/openclaw/Peekaboo)（macOS）、[CursorTouch/Windows-MCP](https://github.com/CursorTouch/Windows-MCP)（Windows）。
2. **流程**：打开收藏页，逐条点开，「分享 → 复制链接」，读剪贴板，把链接交给公开页取内容的插件（[§1.3](#13-混合路径视觉层拿链接公开页面取内容)）；拿不到链接时截图加本机 OCR；视频号这类只能看的内容录下声音（[insidegui/AudioCap](https://github.com/insidegui/AudioCap)、[makeusabrew/audiotee](https://github.com/makeusabrew/audiotee)）再转写。流程写成 [mobile-dev-inc/Maestro](https://github.com/mobile-dev-inc/Maestro) 的 YAML，或用 [IPADS-SAI/MobiAgent](https://github.com/IPADS-SAI/MobiAgent) 的录制回放固定下来，不每次让模型现场决定。
3. **节奏**：单条触发，按人的速度，遇到验证、异常提示立刻停；不在云手机、远程电脑上跑。

**通用视觉层获取插件：限制。**

- 每个 App 要单独写流程，客户端改版就要重写；本次没有找到抖音、小红书、B站、快手、微博、知乎 App 收藏页的现成流程。
- 剪贴板这一环各平台不同：Android 经 scrcpy 能同步到电脑；iPhone 镜像只能经通用剪贴板，mirroir-mcp 目前没有读剪贴板的工具（都待核实）。
- 微信明文禁止，要单独确认；Android 的无障碍服务和 USB 调试可被 App 查到；iPhone 镜像要 macOS 15 以上、欧盟不可用，中国大陆能否使用待核实。
- 慢，每条要几秒到十几秒；只拿到渲染出来的内容，原始文件要靠链接那一步；受保护的画面截图为黑。
- 默认用本机 OCR 和本机模型，不把截图发给云端；用户选云端视觉模型时标「离开本机」。
- 另一种形态是不做插件、只提供 Skill，让用户自己的 Agent（例如 Claude Code 的 computer use）按流程操作，再把链接交给歇存的本地接口。这把节奏和判断交给 Agent，更难控制，不建议作为默认。

<a id="174-fork-与许可证"></a>
### 17.4 fork 与许可证

| 许可证 | fork | 随歇存分发 | 独立进程调用 | 例子 |
|---|---|---|---|---|
| MIT、Apache-2.0、BSD、ISC、Unlicense、CC0、0BSD、WTFPL、MIT-0 | 可以；保留版权声明，Apache 另保留 NOTICE、标明修改 | 可以 | 可以 | Downvid、chubbyskills、wechat-article-exporter 的单篇管线、parse-video、xhs-vault |
| GPL-2.0、GPL-3.0、AGPL-3.0、LGPL | fork 后仍是 GPL 系，要公开源码 | 取得法律意见前不随包分发（[ADR-003 §8](../decisions/ADR-003-plugins.md#8-gpl-工具)） | 用户自装，以独立进程调用 | XHS-Downloader、TikTokDownloader、yutto、gallery-dl、RSSHub；MatrixMedia 是 GPL-2.0-only，和 AGPL 内核也不兼容 |
| MPL-2.0 | 可以；改过的文件保持 MPL 并公开 | 可以，改动的文件要公开 | 可以 | AutoJs6、Camoufox、data-mirroring |
| 非商业或源码可见：自写非商业许可、Commons Clause、PolyForm Noncommercial、CC BY-NC-SA、基于 BSL 的许可、改过的 Apache-2.0、附加条款 | fork 后不能改许可证，不能进官方组织 | 不能 | 用户自装后由插件调用，是否算许可证所说的「包含」要法律意见 | MediaCrawler、yichen-skills、阿杭的两个仓库、wechat-inbox-sync 新版、wx_channels_download、Hypit、videodl、VideoTranscriptAPI、screenpipe、baocut、OXR、MinerU；indra、marker 的权重 |
| 没有许可证 | 不能 fork，只能参考做法 | 不能 | 用户自装后调用，只限自用 | weibo-crawler、weiboSpider、xhs-favorites-exporter、Spider_XHS、xiaohongshu-cli |
| 声明和文件不一致 | 以 LICENSE 文件为准，再看文件头 | 按更严的一方处理 | — | BiliNote 与 VideoNote 里 GPL 头的签名文件、AI-Video-Transcriber 的插件清单、doocs/cose 的 README 徽章、trafilatura 1.7.0 及更早是 GPL、MediaCrawler 2024-04-03 以前是 Apache-2.0、biliup 与 MoneyPrinterPlus 的禁止商用声明 |

按 ADR-003：

- **获取方式不进 `xiecun-app` 组织**（[ADR-003 §5](../decisions/ADR-003-plugins.md#5-仓库拆分)）。自己维护的获取方式放进单独的获取方式组织（见下面「已定方向」一条）；其他人在个人账号下 fork 或自写的（例如 `xiecun-fetch-douyin`）照旧是第三方仓库，客户端标「第三方 · 未经歇存审核」（[ADR-003 §6](../decisions/ADR-003-plugins.md#6-分发与校验)）。GitHub App 两种都能装，做清单、sha256 和契约测试的校验。
- **fork 时要做的事。** 保留原许可证和版权声明，README 写明上游和改动；去掉写操作、推广内容、遥测和来源不明的凭据（例如 NewsCrawler 写死的头条 Cookie）；外部二进制锁定版本和 sha256；按 [§17.1](#171-清单里声明获取层级) 声明层级。
- **官方组织可以收的**：转换和导出插件，以及不触碰平台的库，例如纯 Go 的正文提取、HTML 转 Markdown。
- **已定方向：自己维护的获取方式放进单独的 GitHub 组织。** 用户 2026-10-30 选了建组织，不放个人账号。照 Mihon 和 Keiyoushi 的分法，这个组织和 `xiecun-app` 分开：只放获取方式插件，包括 fork 后改过的版本；客户端不预置，用户在设置里手动添加组织名后，才列出里面带插件标签的仓库，标「第三方来源 · 组织名」；GitHub App 也装在这个组织上。这样既能自己维护，又保住 ADR-003 把获取方式挡在官方组织外的那层隔离：ADR-003 的理由是 Tachiyomi 的应用、扩展、网站同在一个组织，一起被下线（[02 §1](02-adapter-ecosystems.md#1-tachiyomi-与-mihon-扩展生态)）；[§16.3](#163-下架停更与法律风险) 的证据也显示，B站 的律师函是直接发给维护者的。两个组织的所有者是同一个人，隔离只到组织和仓库这一层。组织名、带不带产品名，等产品改名定下后再起（[调研 07](07-positioning-and-naming.md)），届时修订 ADR-003。
- **待决：只读导出文件的导入插件放哪个组织。** `official-export` 的导入插件不访问平台，但 ADR-003 目前把导入算作获取的一种，所以默认放获取方式组织；要进官方组织、随安装包预装，得在修订 ADR-003 时单独定。google_takeout_parser 的条目已把这一点列为待定。
- **待决：法律意见。** GPL、AGPL 工具作为独立进程被调用的边界（ADR-003 已列）；非商业许可的工具让用户自装、由第三方插件调用是否合规；indra、OXR、marker 权重条款下能否做官方插件。

<a id="18-待核实清单"></a>
## 18. 待核实清单

**[§1](#1-账号风险分层) 与 [§16.1](#161-平台规则与可检测性)：平台规则与可检测性**

- 《微信视频号运营规范》和微信读书用户协议原文没有取到，视频号、微信读书的自动化条款只能先按《腾讯微信软件许可及服务协议》理解。
- 微信安全中心 2026-01 的治理公告、小红书「薯管家」2026-03-10 的 AI 托管公告只看到媒体转述，没有取到官方原文。
- X 自动化规则页被验证页拦截，条款来自搜索摘要，页面日期未核实；Instagram 条款页注明 2027-01-20 生效的新版本没有读。
- 微信个人账号使用规范、微博、知乎、豆瓣、YouTube、Instagram 的条款号来自抓取工具的摘要，没有逐条和原文比对。
- Android 的 USB 调试设置项对第三方 App 恒返回 0 从哪个版本开始（二手来源称 Android 17 QPR1、Pixel），国内厂商 ROM 是否跟进，App 有没有其他办法判断 USB 调试状态。
- 列出已开启无障碍服务的接口在 Android 11 起的包可见性限制下是否过滤结果；文档没标权限要求，没有实机验证。
- 国内 App 在没有 Google Play 服务的机型上用什么风控 SDK，能否识别无障碍服务、ADB、注入点击，没有公开资料。
- iPhone 镜像在中国大陆能否使用；iOS 26、27 有没有新增识别镜像的 API；录屏感知接口在 iPhone 镜像时的取值；镜像的输入在 iPhone 上以触摸还是指针事件到达。
- macOS、Windows 客户端（如 PC 微信）是否读取合成输入的来源标记，没有公开资料。
- Playwright 默认启动参数是否包含让 `navigator.webdriver` 为 true 的开关，没有查文档。
- 抖音「个人信息下载」包含哪些类别（收藏、喜欢、观看历史是否在内），隐私政策没有列出。
- 微信公告里自动抢红包类外挂的实现方式（无障碍服务还是 Hook）没有说明。
- 抖音、快手、B站、知乎、豆瓣、微博对「读取用户自己收藏的自动化」有没有具体处罚案例；本次只找到小红书、微信的公开案例。

**[§1.3](#13-混合路径视觉层拿链接公开页面取内容)、[§10](#10-视觉与-ui-自动化) 与 [§17.3](#173-首批自己适配的插件)：混合路径与视觉层**

- 视频号分享链接不登录能拿到什么：作者和封面能拿到，标题和描述是否返回、个别作品能否不登录拿到视频地址，各仓库的说法不一致。
- 抖音、小红书、B站、快手、微博、知乎 App 收藏页逐条读取或复制链接的现成流程，本次没有找到。
- scrcpy 同步 Android 剪贴板的行为、mirroir-mcp 有没有读剪贴板的办法、iPhone 镜像能否带出声音（视频号转写要用）。
- 小红书 `xsec_token` 的有效期；「几小时后链接 404」只有一个项目的实测。
- 用微信登录的腾讯元宝账号与微信号是否共用风控。

**[§2](#2-覆盖矩阵每个平台有哪些层级可选)：覆盖矩阵**

- 带「†」的格子是对单个平台路径的判断，依据是条目自己的说明，没有逐个读代码复核。
- 多平台工具（yt-dlp、RSSHub、OpenCLI、gallery-dl、lux）在各平台的实际层级没有在本报告逐一列出；即刻、贴吧、Telegram、Pinterest 的 RSSHub 和 OpenCLI 路由没有核实。

**[§3](#3-用户点名的项目)：用户点名的项目**

- 「蝴蝶号」对应 wechat-inbox-sync 是推断：依据是作者署名和账号关联，没有作者本人确认；它的 iOS 快捷指令本体不在仓库里，是否直接调用提交接口核实不了。
- kali-publish 只有 1 个 ★、没有 issue，能否持续维护不确定。
- OXR 的许可条款和接口都很新，可能变化；indra 权重的竞争条款能否用于官方插件需要法律意见。
- 阿杭的源码可见许可下，第三方插件调用用户自装的副本是否算「包含」，需要法律意见。
- Hypit 改过的 Apache-2.0 与歇存 AGPL 内核的关系，需要法律意见。
- `echo` 对应 ycccccccy/echotrace 是推断，把握中等：依据是名字、星数和它与本报告微信本地数据这条线的关系，没有用户确认。如果用户是在 Obsidian、录音转写或 Agent 记忆的语境里提到 echo，更可能指 echo-notes、echo-memory 或 HuanHuanHuanFFF/Echo。
- Easel 记 `private-api` 是因为 B站 登录模拟 TV 端；只按取内容那一步算，默认路径最高是 `session-api`，要和其他条目统一口径。可选的 CloakBrowser 内核会改浏览器指纹（[ZJU-REAL/Easel #52](https://github.com/ZJU-REAL/Easel/issues/52)），是否也按 `private-api` 算，同样要统一。
- Easel 的抖音发布脚本自述移植了 AGPL-3.0 项目的流程与选择器，却随 Apache-2.0 发布，相容性没有核实；它自己的致谢文件也把部分来源的许可写成待核实。#171 的采集器协议还没合并，合并前可能再变。
- echo-podcast-memory 记 `session-api`，但它请求小宇宙接口时带 App 的应用标识；如果小宇宙播客后台网页自己不带，按「模拟 App 调私有接口」应改记 `private-api`。
- EchoMind 闭源，README 说经腾讯官方 ClawBot 通道收消息，没法从代码核实；这个通道对非 OpenClaw 应用的使用条款、消息是否经腾讯以外的服务器，也没有核实。

**[§4](#4-多平台下载与抓取) 到 [§15](#15-相邻创作发布与分析)：各类别条目**

- BiliNote、VideoNote 仓库标 MIT 或 Apache-2.0，但抖音签名文件头写 GPL-3.0，又写现属 Apache-2.0 的项目，前后矛盾。
- caishi 的代码层 MIT 与全项目授权口径冲突，要向作者确认。
- nobiyou/wx_channel 与 ltaoo/wx_channels_download 早期代码的关系未核实。
- jackwener/wechat-article-to-markdown、makeusabrew/audiotee、humandataincome/hudi-packages-connectors 只在 README、pyproject 或 npm 里声明了许可证，仓库里没有 LICENSE 文件。
- biliup、MoneyPrinterPlus 的禁止商用声明与 LICENSE 的关系；short-video-maker 依赖的 Remotion 公司许可。
- OmniParser 的 v3 检测器权重来自 Hugging Face 上未合并的 PR；mobile-mcp 底层 mobilecli 的许可证。
- sherpa-onnx 2.0 的命令行参数变化；FunASR 单文件运行时在 macOS 上的签名；yap 的中文转写质量。
- Agent-Reach #802 第三方扫描器的告警没有维护者回应；它的 B站 后端依赖已关停的库。
- MediaCrawler 2024-04-03 以前以 Apache-2.0 发布的版本能否作为参考基线，需要法律意见。

**[§16.2](#162-issue-里的账号问题)：issue 统计**

- 关键词表只覆盖一部分报告；标题里的 account blocked、461、账号异常常是签名或代码失效，只按读过正文的计数；样本按关键词挑选，不是随机抽样。
- xiaohongshu-mcp 2026-07-27 改用内置指纹浏览器后，按严格口径可能要改记 `private-api`；之后的报告都没写版本。
- Agent-Reach #498 没写小红书走的哪个后端；BettaFish 讨论 #670 的采集路径是否为 MediaCrawler；MediaCrawler #915 触发提示的原因。
- BBDown、bilibili-api、wxauto 的 issue 已关闭，读不到；OpenCLI 只抓了最近 2500 条，yt-dlp 只取了标题含平台名的 287 条。
- 手机自动化和 iPhone 镜像类通用 Agent 没有处罚报告，可能是很少有人拿来长期操作小红书或微信；官方导出层没有收集 issue 证据。
- instaloader #2680 是单条事后报告；yutto #452 的大会员冻结因果不明；XHS-Downloader #315、#321 没写是否填了 cookie。
- wx_channels_download 的本地代理路径没有处罚报告，不等于安全；wewe-rss 的请求经第三方转发，小黑屋和失效可能来自共用的上游。
- 小红书是否按设备或实名牵连其他账号没有回答；微信有按实名牵连的报告。

**[§16.3](#163-下架停更与法律风险)：下架与法律**

- WeChatMsg 2025-04 移除代码的原因；BBDown、BilibiliDown 清空的原因，是否与 B站 函件有关。
- WeFlow 在被点名到历史重置之间是否被屏蔽过；wechat-dump README 的改动内容是拿现状和通知引文对比得出的。
- 指导性案例 262 号的当事人、263 号的授权方式；聚客通案的判决原文；微博诉脉脉的终审日期；小红书 490 万元案的案号。
- legado 公告所说的法律责任是民事还是刑事；优酷 DMCA 之后发布附件是否被屏蔽；NeteaseCloudMusicApi 归档后 npm 新版本的发布者与原作者的关系。
- github/dmca 只按文件名检索，代理律所用其他名字提交的通知可能漏掉；平台直接发给维护者的函件不经过这个仓库。抖音、小红书有没有不公开的函件无法检索；erma0/douyin、yichen-skills 剪映 Skill 的发函方没有写明。
- 2025 年《反不正当竞争法》第十三条第三款对个人维护者、非经营性开源项目是否适用；开源公开分发的工具是否按法释〔2011〕19号第三条计算提供人次；海淀 2018 年案是否就是常说的抓取今日头条视频案。
- 海外判例（例如 X Corp. 诉 Bright Data）没有展开，X、Instagram 适配的法律面要单独看。

**[§17](#17-对歇存的启示)：启示**

- 平台风险修正表放在主仓库随屏蔽清单发布，还是写进客户端。
- 清单按能力分别声明层级，是否和 [SPEC §8](../SPEC.md#8-适配器协议-v0) 现有的 `capabilities` 列表写法兼容。
- 兼容 CookieCloud 公开协议、不复制代码地写一个本机接收端，是否需要法律意见。
