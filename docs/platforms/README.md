<a id="各平台怎么取"></a>
# 各平台怎么取

每个平台每一样东西要不要账号、走哪条路、做到哪，都在这一页。一个平台的细节（请求、身份、存下什么、错误怎么报）写在那个平台自己的设计里：[小红书](xiaohongshu.md)、[视频号](wechat-channels.md)、[B站](bilibili.md)、[X](x.md)。小红书、视频号、B站这三份照 2026-11-11 用 `gh api` 读的源码、README、release 和 issue 写，没有登录任何账号，没有对平台发请求，没有运行第三方代码，也没有读本机的微信数据目录；标「推断」的没有实测。证据和来源在[调研 06](../research/06-platform-access-and-account-risk.md)，这里不重抄。每个平台仓库里有哪几种取法、各用什么做成，在 [routes.md](routes.md)；手机上替你复制链接的那几种取法，在 [phone.md](phone.md)。

<a id="1-怎么读这一页"></a>
## 1. 怎么读这一页

**层级。** 表里的 `public-web`、`browser-session`、`session-api` 这些词照[插件协议 §2.4](../protocol/manifest.md#24-access-与-login) 的定义，这里不再解释；它们是给你挑的标签，不拦。

**要不要账号**，表里用六个说法：

| 说法 | 意思 |
|---|---|
| 不要 | 不登录，也不用任何身份 |
| 访客身份 | 不登录，但要一份站点给任何访客的 cookie，usuyo 开一个不登录的窗口取（[协议 §2.5.1](../protocol/manifest.md#251-访客身份-visitors)） |
| 账号只在你自己的浏览器里 | 你在日常浏览器里登录着，脚本只看页面自己拿回来的东西，把链接交给 usuyo（[协议 §9.4](../protocol/wrapping.md#94-浏览器扩展与油猴脚本)）；usuyo 碰不到你的 cookie |
| 要在 usuyo 里登录 | 你在 usuyo 的登录窗口里登录，插件带你的 cookie 调平台接口（[协议 §2.5](../protocol/manifest.md#25-登录站点-sites)） |
| 账号在你手机的 App 里 | 你手机上登录着的 App，经 iPhone 镜像或 adb 替你点「分享 → 复制链接」，只交链接（[phone.md](phone.md)）；usuyo 碰不到你的账号 |
| 没有路 | 调研里没找到可用的做法 |

**账号只用在非它不可的地方**（[BUILD §1](../BUILD.md#1-用户定下的做法)）。每个平台按「这一样要不要账号」拆开：不要账号能拿到的，一律不带账号去取，你登录过也不带；账号只用在没有别的办法的那几样上，多半是「列出你存过的」。这是分路的规矩，不是一个共用的下载器：每个平台还是一个插件，插件按能力声明各走各的路。

**清单现在怎么写出这个拆分。** 按能力分开声明（[协议 §2.3](../protocol/manifest.md#23-获取插件的字段)）：不要账号的写在 `fetch` 上，`login: none`；要账号的写在 `list` 上，`login: required`，`sites` 写是哪个账号；只要访客身份的写 `visitors`，它不算账号，`login: none` 也能写。B站的样子就是 `fetch` 不登录、`list` 要登录。

**只有某一样要账号时，写成另一种方法。** 视频号的作者、描述、封面不登录就取得到，只有视频要借元宝账号：清单里 `feed-info` 不带账号取前几样、在 `missing` 里写视频，`yuanbao` 列元宝站点、只取视频。宿主先调不带账号的那种，它说缺视频，再找声明了视频的方法只要视频（[协议 §2.3.1](../protocol/manifest.md#231-方法-methods)）；凭据只交给列了站点的方法，只要文字的调用拿不到元宝的登录。

按创作者主页列、按关键词列，协议也还没有对应的能力，记在[协议 §13](../protocol/open.md#13-待定) 第 26 项；表里这一行只写调研查到的路。

<a id="2-总表"></a>
## 2. 总表

「状态」三种：**做了**（插件写好了，测试全离线，没碰过真平台）、**设计了**（有获取设计，没写插件）、**只有调研**（只有[调研 06](../research/06-platform-access-and-account-risk.md) 和[旧笔记](../research/notes/README.md)里的判定，没再核过上游现在的样子）。四样东西：**图文**是单条的文字和图；**视频**是单条的视频；**存过的**是列出你收藏、赞过、稍后再看的；**创作者 / 关键词**是按某个人的主页或按搜索词列。

| 平台 | 哪一样 | 要不要账号 | 走哪条路（层级） | 状态 | 在哪 |
|---|---|---|---|---|---|
| 小红书 | 图文 | 访客身份 | 公开笔记页，读页面里的状态对象，不签名（`public-web`） | 做了 | `usuyo-fetch/xiaohongshu`；[xiaohongshu.md §2.2](xiaohongshu.md#22-取单条) |
| 小红书 | 视频 | 访客身份 | 同一个 `fetch`，下载页面给的地址；不登录只有低清（`public-web`） | 做了 | 同上 |
| 小红书 | 存过的 | 账号只在你自己的浏览器里；或账号在你手机的 App 里 | 油猴脚本只记收藏页、赞过页自己的请求，交 `/v1/batches`（`browser-session`）；或 iPhone 镜像替你逐条复制链接（`vision-ui`） | 脚本做了；手机设计了 | `usuyo-fetch/xiaohongshu/userscript/`；[xiaohongshu.md §2.1](xiaohongshu.md#21-列出你存过的)；[phone.md §7.2](phone.md#72-小红书收藏) |
| 小红书 | 创作者 / 关键词 | 不要（推断） | RSSHub 用不登录的无头浏览器截获主页请求（`public-web`）；搜索只找到要签名的（`private-api`） | 只有调研 | [多平台笔记 RSSHub 表](../research/notes/multi-platform-tier-table-and-base-deps.md#rsshub) |
| 视频号 | 图文 | 不要 | 分享链接对应的匿名接口，只有作者、描述、封面（`public-web`） | 做了 | `usuyo-fetch/wechat-channels`；[wechat-channels.md §3.2](wechat-channels.md#32-取单条) |
| 视频号 | 视频 | 要在 usuyo 里登录（腾讯元宝，不是微信） | 元宝换临时凭证，再匿名要视频地址（`session-api`） | 做了 | 同上 |
| 视频号 | 存过的 | 账号在你手机的 App 里 | iPhone 镜像替你在「赞和收藏」里逐条复制链接（`vision-ui`，微信记中）；改写微信客户端的工具（`client-hook`）每次微信更新都失效 | 设计了 | [phone.md §7.1](phone.md#71-视频号赞和收藏)；[wechat-channels.md §3.1](wechat-channels.md#31-列出你存过的) |
| 视频号 | 创作者 / 关键词 | 没有路 | 调研没查到 | 只有调研 | — |
| B站 | 图文 | 不要 | `view` 接口取标题、简介、封面、分 P（`public-web`） | 做了 | `usuyo-fetch/bilibili`；[bilibili.md §4.2](bilibili.md#42-取单条) |
| B站 | 视频 | 不要 | 不签名的 `playurl` 取低清音视频（`public-web`）；高清和字幕要登录，不取，文字交给转写 | 做了 | 同上 |
| B站 | 存过的 | 要在 usuyo 里登录 | 收藏夹、稍后再看的不签名接口（`session-api`）；公开收藏夹也许不登录就列得出（推断） | 做了 | 同上；[bilibili.md §4.1](bilibili.md#41-列出你存过的) |
| B站 | 创作者 / 关键词 | 不要，但要签名 | UP 主投稿列表要 WBI 签名（`private-api`） | 只有调研 | [多平台笔记 RSSHub 表](../research/notes/multi-platform-tier-table-and-base-deps.md#rsshub) |
| X | 图文 | 不要 | 访客令牌调帖子接口，或嵌入用的 syndication 接口（`public-web`）；要登录才看得到的帖子除外 | 做了 | `usuyo-fetch/x`；[x.md](x.md#3-取单条) |
| X | 视频 | 不要 | 同一次回答里的视频地址，不带 cookie 下载（`public-web`） | 做了 | 同上 |
| X | 存过的 | 账号只在你自己的浏览器里 | 油猴脚本只记书签页、喜欢页自己的请求，交 `/v1/batches`（`browser-session`）；或读官方归档（`official-export`） | 做了 | `usuyo-fetch/x/userscript/`；[x.md](x.md#2-列出你存过的) |
| X | 创作者 / 关键词 | 账号只在你自己的浏览器里 | 不登录看不到主页时间线和搜索；只能由脚本记你自己翻到的 | 设计了（等[协议 §13](../protocol/open.md#13-待定) 第 26 项） | [x.md](x.md#4-按创作者或关键词列) |
| 抖音 | 图文 | 不要 | 分享页服务端渲染的数据（`public-web`） | 只有调研 | [调研 06 §1.3](../research/06-platform-access-and-account-risk.md#13-混合路径视觉层拿链接公开页面取内容) |
| 抖音 | 视频 | 不要 | 同一个分享页给的播放地址（`public-web`）；yt-dlp 那条路要一份新的访客 cookie | 只有调研 | 同上；[yt-dlp 笔记](../research/notes/tier-ytdlp-and-sph-attribution.md) |
| 抖音 | 存过的 | 账号只在你自己的浏览器里；或不要（读你申请的导出） | 在真抖音页面里由页面自己的请求取收藏（`browser-session`）；页面外的请求 2026 下半年被拦；官方「个人信息下载」的导入（`official-export`），包里有什么没核实 | 只有调研 | [抖音笔记](../research/notes/douyin-direction-findings.md)；[routes.md §7](routes.md#7-抖音) |
| 抖音 | 创作者 / 关键词 | 不要（创作者）；关键词没有路 | 不登录的无头浏览器截获主页请求（`public-web`）；搜索出验证码 | 只有调研 | [多平台笔记 RSSHub 表](../research/notes/multi-platform-tier-table-and-base-deps.md#rsshub) |
| 快手 | 图文 | 不要 | 分享链接的页面数据（`public-web`） | 只有调研 | [调研 06 §1.3](../research/06-platform-access-and-account-risk.md#13-混合路径视觉层拿链接公开页面取内容) |
| 快手 | 视频 | 不要 | 同上；频繁请求要验证 | 只有调研 | 同上 |
| 快手 | 存过的 | 账号在你手机的 App 里 | 没找到浏览器里取收藏的工具；手机上替你逐条复制链接（`vision-ui`） | 只有调研 | [routes.md §8](routes.md#8-快手) |
| 快手 | 创作者 / 关键词 | 没有路 | 按账号批量在 KS-Downloader 里被注释掉 | 只有调研 | 同上 |
| 微博 | 图文 | 访客身份 | 访客握手后调不签名接口（`public-web`） | 只有调研 | [多平台笔记的口径](../research/notes/multi-platform-tier-table-and-base-deps.md#口径) |
| 微博 | 视频 | 访客身份 | 同上 | 只有调研 | 同上 |
| 微博 | 存过的 | 账号只在你自己的浏览器里，或要在 usuyo 里登录 | 油猴脚本翻自己的收藏（Weibo-archiver），或带 cookie 调收藏接口（`session-api`） | 只有调研 | [调研 06 §1.3](../research/06-platform-access-and-account-risk.md#13-混合路径视觉层拿链接公开页面取内容) |
| 微博 | 创作者 / 关键词 | 访客身份（创作者）；关键词没查 | 访客 cookie 调用户主页接口（`public-web`） | 只有调研 | [多平台笔记 RSSHub 表](../research/notes/multi-platform-tier-table-and-base-deps.md#rsshub) |
| 贴吧 | 四样都是 | 没有路（不要账号的） | 只找到计算客户端签名的（`private-api`） | 只有调研 | [层级笔记](../research/notes/tier-calls-weibo-zhihu-douban-x-ig-reddit-tg.md) |
| 知乎 | 图文 | 不要 | 公开回答、文章页（`public-web`） | 只有调研 | [调研 06 §1.3](../research/06-platform-access-and-account-risk.md#13-混合路径视觉层拿链接公开页面取内容) |
| 知乎 | 视频 | 不要 | 不签名的视频接口（`public-web`） | 只有调研 | [多平台笔记 yt-dlp 表](../research/notes/multi-platform-tier-table-and-base-deps.md#yt-dlp) |
| 知乎 | 存过的 | 不要（公开收藏夹）；要在 usuyo 里登录（私密的） | 收藏夹接口不签名，公开的不带 cookie 也能取 | 只有调研 | [层级笔记](../research/notes/tier-calls-weibo-zhihu-douban-x-ig-reddit-tg.md) |
| 知乎 | 创作者 / 关键词 | 不要，但要签名（创作者）；关键词走开放平台 | 用户动态要算 `x-zse-96`（`private-api`）；搜索有按额度计费的开放平台（`official-export`） | 只有调研 | 同上 |
| YouTube | 图文 | 不要，但层级高 | 不登录取字幕和元数据，youtube-transcript-api 记 `private-api` | 只有调研 | [调研 06 §1.3](../research/06-platform-access-and-account-risk.md#13-混合路径视觉层拿链接公开页面取内容) |
| YouTube | 视频 | 不要，但层级高 | yt-dlp 默认以 App 客户端身份请求、本地解 JS 挑战（`private-api`）；访客约每小时 300 个视频 | 只有调研 | [yt-dlp 笔记](../research/notes/tier-ytdlp-and-sph-attribution.md) |
| YouTube | 存过的 | 不要（导出时你自己登录 Google） | Google Takeout 导出的「稍后观看」（`official-export`） | 只有调研 | [调研 06 §1.3](../research/06-platform-access-and-account-risk.md#13-混合路径视觉层拿链接公开页面取内容) |
| YouTube | 创作者 / 关键词 | 不要（创作者）；关键词没查 | 频道的公开订阅源（`public-web`） | 只有调研 | [调研 06 §2](../research/06-platform-access-and-account-risk.md#2-覆盖矩阵每个平台有哪些层级可选) |
| 公众号 | 图文 | 不要 | 文章页（`public-web`）；批量取会出环境验证 | 只有调研 | [调研 06 §1.3](../research/06-platform-access-and-account-risk.md#13-混合路径视觉层拿链接公开页面取内容) |
| 公众号 | 视频 | 不要 | 文中的视频号卡片只有元数据和封面，视频照视频号那一行 | 只有调研 | 同上 |
| 公众号 | 存过的 | 账号在你手机的 App 里，或你电脑的微信里 | iPhone 镜像或电脑版微信替你在收藏里逐条复制文章链接（`vision-ui`，微信记中） | 设计了 | [phone.md §7.3](phone.md#73-微信收藏里的文章)；[协议 §9.5](../protocol/wrapping.md#95-视觉层流程) |
| 公众号 | 创作者 / 关键词 | 要在 usuyo 里登录（公众号后台） | 公众号后台的文章列表（`session-api`）；wechat-article-exporter 用的那条 2026-07 下旬关闭（[#200](https://github.com/wechat-article/wechat-article-exporter/issues/200)），we-mp-rss 2026-11-10 仍有推送，现在能不能用没核 | 只有调研 | [routes.md §13](routes.md#13-公众号和微信收藏) |

从表里看：单条的文字、图、视频，除了视频号的视频、B站的高清和字幕、X 上要登录才看得到的帖子，都不要账号（贴吧例外，不要账号的路一条都没找到）；要账号的几乎都是「存过的」。YouTube 不要账号但层级是 `private-api`，要不要账号和层级是两回事。

<a id="3-现在的插件哪里不合这条规矩"></a>
## 3. 现在的插件哪里不合这条规矩

照小红书、视频号、B站三个插件的清单和代码（`usuyo-fetch/*/usuyo-plugin.yaml`，11-12 起一种取法一个方法）：

- **小红书**：合。`fetch` 是 `login: none`，只用访客身份；清单由你浏览器里的脚本交，usuyo 不碰你的账号。
- **视频号**：合。`feed-info` 是 `login: none`，不带任何 cookie；`yuanbao` 只取视频，元宝的凭据只交给它，只发给 `yuanbao.tencent.com`。
- **B站**：合。`view` 是 `public-web`、`login: none`，不读凭据、不带 cookie，取标题、简介、封面和低清音视频；字幕要登录才有，不取，文字交给转写。别人公开的收藏夹由 `favorites-public` 不带 cookie 列；账号只用在 `favorites`（你自己的收藏夹和稍后再看）上。

<a id="4-怎么相对安全地试"></a>
## 4. 怎么相对安全地试

先说几件证据说明了的事，再按风险从小到大排步骤。四个插件都还没向真平台发过一次请求，所以每一步都是第一次。

- **不用账号不等于没风险。** 小红书上有人不填 cookie、批量取约 108 条分享链接后，复制链接的那个账号被下线（[XHS-Downloader #305](https://github.com/JoeanAmier/XHS-Downloader/issues/305)），分享链接里的参数能把匿名请求连回账号。小红书插件不发这些参数，但从浏览器里交来的链接比手机分享的短链干净。
- **新注册的号比用了几年的备用号更容易被拦。** X 上新注册的小号第一次搜索就被锁，注册 3 年的主号一次取 800 个关注也被封（[调研 06 §1.1](../research/06-platform-access-and-account-risk.md#11-七个层级) 下的规律）。要试登录这一步，用一个用了一阵的备用号，不要现注册。可是「列出存过的」只能用存东西的那个号，这一步备用号替不了。
- **出口 IP 跳来跳去，哪一层都加风险**（[调研 06 §16.2](../research/06-platform-access-and-account-risk.md#162-issue-里的账号问题) 的规律）。你的 Mac 开着代理：国内这几个平台在代理里设成直连；X 走不了直连，就固定走你浏览器平时上 X 的那一个出口，不要用自动选节点、负载均衡这类会换出口的模式。试的时候别中途换节点。
- **节奏比层级要紧。** 微信、微博的规则都按频率判，和怎么取无关。先几条，隔一天，再几十条。插件清单里的节奏建议（小红书每 15 秒一条、每天 100 条，视频号 30 秒、50 条，B站 5 秒、500 条）可以在设置里调得更慢。

步骤：

1. **不碰平台。** 开发这边跑 `just dev-plugins` 把四个插件装上。打开「获取插件」页，看每个插件那一行写的层级和要不要登录，和[总表](#2-总表)对得上就行。
2. **一条公开内容，不登录。** 每个平台粘一条：B站一个公开视频的链接；视频号一条手机「分享 → 复制链接」得到的链接；小红书先在「访客身份」里点「取访客身份」，窗口自己打开、取到就关，再粘一条从电脑浏览器地址栏复制的笔记链接（带 `xsec_token`）。你会看到：B站有标题和封面；视频号是「存下了一部分」，缺视频；小红书有正文和图。X 同样粘一条。
3. **几条，隔一天。** 每个平台再收三到五条，第二天在手机上看账号有没有提示、有没有被下线。小红书这一步就用浏览器里复制的链接，不用手机分享的短链。
4. **你自己的浏览器交清单，账号不进 usuyo。** 在导入页「交进来的清单」里发一个令牌，按小红书插件 README 装好油猴脚本，到收藏页**自己往下翻**一屏，交给 usuyo，在导入预览里只勾十条确认。会看到它们按 15 秒一条慢慢取下来。X 的脚本（`usuyo-fetch/x/userscript/`）同样做。
5. **在 usuyo 里登录。** B站：在「获取插件」里登录（扫码），先只列「稍后再看」，再列一个小收藏夹，导入十条。视频号：登录腾讯元宝，在一条已经「存下了一部分」的视频号条目上点「从原帖再取一次」，看视频有没有补上。
6. **放大。** 每天几十条，过几天没事再导入整个收藏夹；节奏照默认的走。

**界面上会看到的，和要告诉开发这边的。** 停下来的插件那一行会写原因：「要先取访客身份」或「这份访客身份不管用了，换一份再试」（再取一份就好）；「要重新登录」（重新登录那个站点）；「平台说请求太频繁」，到点自己再试；「平台挡住了这个插件」，停一小时，你可以点「现在再试」，但最好等；「平台改版了，插件读不到」，这时要改插件。出了「挡住」和「改版」，或账号收到提示、被下线，就停在这一步，告诉开发这边：哪个平台、哪一步、插件那一行的原话、出事前这一批取了几条、隔了多久、当时走的是直连还是哪个出口、账号收到的提示原文。不要发 cookie 或令牌的值，链接里的 `xsec_token` 也不用发。
