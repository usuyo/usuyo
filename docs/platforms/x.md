<a id="x推特获取设计"></a>
# X（推特）：获取设计

写给接手写 X 插件的模型和以后查。和[调研 15](../research/15-fetch-design-xhs-sph-bili.md) 里一个平台的那一节同样的写法。依据是 2026-11-11 用 `gh api` 读的上游源码、README 和 issue（版本写在[第 11 节](#11-依据)）；没有向 x.com、twitter.com、twimg.com 或 X 的任何主机发过请求，也没有读 X 自己的文档站（它也在 X 的主机上）。所以凡是线上行为，要么出自上游的代码和 issue，要么标「要你来核」。标「推断」的是没有证据的判断。

所有平台放在一起的总表、要不要账号的五个说法、怎么相对安全地试，在 [platforms/README](README.md)；这里不重复。

<a id="0-结论"></a>
## 0. 结论

- **单条不要账号。** 一条公开帖子的正文、图片、视频、引用的帖子，用访客令牌调帖子接口就拿得到；被限流时退到给嵌入用的 syndication 接口。yt-dlp 默认就这么走，gallery-dl 不带 cookie 时也这么走（[第 3 节](#3-取单条)）。层级 `public-web`，`login: none`。
- **列出你存过的，账号只在你自己的浏览器里。** 你在日常浏览器里登录着 X，油猴脚本只记书签页、喜欢页自己拿回来的回答，把**链接**交给 `POST /v1/batches`；usuyo 再按链接一条条不带账号去取。另一条不碰账号的路是读你从 X 申请下来的官方归档（[第 2 节](#2-列出你存过的)）。
- **分叉怎么定：(a) 脚本交链接，插件不带账号取。** 判据是「单条连视频在内，不登录能不能取到」：能，除了下面三种。所以 (b)「脚本交内容」不是默认，只用在这三种上：
  1. 要登录才看得到的帖子：接口回 `TweetUnavailable`，原因是 `NsfwLoggedOut`、`NsfwViewerHasNoStatedAge`（成人内容）或 `Protected`（受保护的账号）；
  2. 一条帖子**后面**的串（作者自己接着回的几条）；
  3. X 的长文章（Articles）正文，不登录拿不拿得到没核实。

  这三种的内容在你浏览器里页面自己拿回的回答里有（书签、喜欢的时间线带着帖子本身；打开帖子页时的 `TweetDetail` 带着整串）。要收它们，靠的是内核的通用投递口：`extension` 令牌经 `POST /v1/deliver` 交已经取到的内容，[协议 §5.1](../plugin-protocol.md#51-本机-http-接口) 设计了、还没做。它是给所有平台的扩展和脚本用的，不是 X 专用的路。在它做好以前，这三种条目停在「只有链接」。

  这个分叉靠一个前提：你那个出口上访客令牌或 syndication 至少有一个肯回答。两个都被拒时，单条也只能从登录着的页面拿，全部落到 (b)。所以[第 12 节](#12-测试)里你先测的就是这一条。
- **usuyo 里不登录 X。** 不做登录窗口、不做带 cookie 的 `list`，理由见[第 2 节](#2-列出你存过的)。代价：书签和喜欢没有 `list` 能力，就进不了跟着收，每次都是你自己翻一遍、脚本交一批；新存的不会自己进来。
- **清单**见[第 9 节](#9-清单)。

<a id="1-要账号的和不要账号的"></a>
## 1. 要账号的和不要账号的

| 哪一样 | 要不要账号 | 怎么取 |
|---|---|---|
| 单条：正文、图片、视频、引用的帖子 | 不要 | 插件的 `fetch`，访客令牌或 syndication |
| 单条：前面的串（这条回复的是同一作者的上一条） | 不要 | 同一次 `fetch` 里按 `in_reply_to_status_id_str` 往上再取 |
| 单条：后面的串、长文章正文、成人内容、受保护账号的帖子 | 要登录着的页面 | 等通用的内容投递（见[第 0 节](#0-结论)） |
| 列出书签、喜欢 | 账号只在你自己的浏览器里 | 油猴脚本交链接；或读官方归档 |
| 按创作者、按关键词列 | 账号只在你自己的浏览器里 | 见[第 4 节](#4-按创作者或关键词列) |

<a id="2-列出你存过的"></a>
## 2. 列出你存过的

- **日常浏览器里的脚本**（`browser-session`，推荐）：照 [prinsss/twitter-web-exporter](https://github.com/prinsss/twitter-web-exporter)（MIT，v1.4.3，2026-08-31）的做法。它在页面环境里改写 `XMLHttpRequest.prototype.open`，等请求完成后读回答，不发任何额外请求（`src/core/extensions/manager.ts`）。书签认 URL 里的 `/graphql/<id>/Bookmarks`，从 `data.bookmark_timeline_v2.timeline.instructions` 取帖子（`src/modules/bookmarks/api.ts`）；喜欢认 `/graphql/<id>/Likes`，从 `data.user.result.timeline.timeline.instructions` 取（`src/modules/likes/api.ts`）。它要注入到页面环境、不能是内容脚本环境，否则截不到（同一个文件里的检查，和它的 [#19](https://github.com/prinsss/twitter-web-exporter/issues/19)）。作者明说不带自动滚动，免得被当成机器（[#120](https://github.com/prinsss/twitter-web-exporter/issues/120)），一屏屏要你自己翻（[#51](https://github.com/prinsss/twitter-web-exporter/issues/51)）。调研 06 扫过它的 issue，没有使用者被锁的报告（[06 §16.2](../research/06-platform-access-and-account-risk.md#162-issue-里的账号问题)）。

  usuyo 的脚本放在 X 插件仓库的 `userscript/` 下，照小红书插件的脚本写（握手、令牌只存脚本管理器、只连 `127.0.0.1`，见 `usuyo-fetch/xiaohongshu/README.md`；小红书那边记下的条目 12 小时作废是因为 `xsec_token` 会过期，X 的链接不过期，不照搬）。它只记：帖子 id、作者的 `screen_name`、正文第一行当标题；书签记 `action: favorite`，喜欢记 `action: like`；交的链接是 `https://x.com/<screen_name>/status/<id>`。喜欢只记你自己的：请求里的 `userId` 要等于登录着的那个人（怎么在页面里拿到自己的 id 要实现时核，推断可以读 `twid` cookie 或页面状态；X 2024 年起不给别人看喜欢列表，推断页面上本来就只有你自己的）。

  回答里没有「什么时候存的」：两种时间线只给 `sortIndex`。fieldtheory-cli 也拿不到书签时间，把 `bookmarkedAt` 记成空（`src/graphql-bookmarks.ts`）。所以 `actedAt` 留空，不用抓取时间冒充（[协议 §3.8](../plugin-protocol.md#38-list-与跟着收)）。
- **官方归档**（`official-export`，不碰账号）：在 X 的设置里申请下载自己的数据，拿到一个 zip。[steipete/birdclaw](https://github.com/steipete/birdclaw)（MIT）读里面的 `data/like.js`、`data/like-partN.js`，也接受 `data/bookmark(s).js`（`src/lib/archive/slices.ts`）。它的代码接受书签文件，不等于今天的归档里一定有书签；调研笔记记过 timelinize 的 Twitter 源不导入喜欢和书签（[官方导出笔记](../research/notes/official-export-findings.md)）。归档里有没有书签、`like.js` 每条有哪些字段，要你申请一份来看。插件的 `list` 读这个 zip，只交链接，内容照样由 `fetch` 不带账号去取。
- **带 cookie 调书签接口**（`session-api`，不做）：[afar1/fieldtheory-cli](https://github.com/afar1/fieldtheory-cli)（MIT，2026-09-05）的 `src/graphql-bookmarks.ts` 带网页的公开 bearer、`ct0` 和 `auth_token` cookie 调 `Bookmarks`，不生成 `x-client-transaction-id`，默认每页隔 600 毫秒；它的 issue 里没搜到锁号或 403 的报告。不做的原因：要你在 usuyo 里登录 X，cookie 出了浏览器、由程序按程序的节奏翻页；而 gallery-dl 带登录时每次都生成 `x-client-transaction-id`（`gallery_dl/extractor/twitter.py` 的 `_call`，第 1893 行起），说明登录着的接口可能在查这个头（推断）。浏览器脚本那条路已经够用，这条只多一样：能做跟着收，新存的书签自己进来。为这一样把账号交给程序，不值（[第 0 节](#0-结论)写了代价）；你要是更看重跟着收，这是要你答的一问。
- **付费的官方接口**（`official-export`，要开发者账号）：[xdevplatform/xurl](https://github.com/xdevplatform/xurl)（MIT，2026-11-07 仍在推送）调 `GET /2/users/:id/bookmarks` 和 `GET /2/users/:id/liked_tweets`，每次最多 100 条（`api/shortcuts.go`）；要 OAuth 2.0 用户授权，应用要放进按量付费的套餐（README）。价格和条数上限写在 X 自己的文档站上，没核。可以作为你自己装 xurl、usuyo 包装它的插件（[协议 §9.1](../plugin-protocol.md#91-包装命令行)），不是首选。

<a id="3-取单条"></a>
## 3. 取单条

**先走帖子接口，访客令牌。**

1. `POST https://api.x.com/1.1/guest/activate.json`，带网页的公开 bearer，拿 `guest_token`。
2. `GET https://x.com/i/api/graphql/<queryId>/TweetResultByRestId`，参数 `variables`（`tweetId`）、`features`、`fieldToggles`，请求头 `Authorization: Bearer <公开 bearer>` 和 `x-guest-token`。

这是 yt-dlp 不带 cookie 时的默认路径（`yt_dlp/extractor/twitter.py`：`_fetch_guest_token` 第 104 行，`_call_api` 第 120 行起，`_extract_status` 第 1182 行起）。2026-06 的 [yt-dlp #17058](https://github.com/yt-dlp/yt-dlp/issues/17058) 里，日志是「Downloading guest token → Downloading GraphQL JSON」，失败只因为那条是成人内容；2026-10-27 的 [#17939](https://github.com/yt-dlp/yt-dlp/issues/17939) 里，一条公开帖子不带 cookie 走同样两步，视频下载下来了（报的是多出一个空条目的 bug）。gallery-dl 没有 `auth_token` 时也走访客令牌，而且只在登录时才生成 `x-client-transaction-id`（`_call` 第 1897–1901 行）；[旧笔记](../research/notes/tier-calls-weibo-zhihu-douban-x-ig-reddit-tg.md)把 gallery-dl 整个记成 `private-api`，对它不登录的这条路不成立。没有签名、没有设备指纹，所以记 `public-web`（同[多平台笔记 yt-dlp 表](../research/notes/multi-platform-tier-table-and-base-deps.md#yt-dlp)的口径）。

`queryId` 写死在代码里，三家各用一个（yt-dlp `2ICDjqPd81tulZcYrtpTuQ`，gallery-dl `qxWQxcMLiTPcavz9Qy5hwQ`，fieldtheory `fHLDP3qFEjnTqhWBVvsREg`），推断 X 还收旧的。fieldtheory 的注释说过期了就去网页的 `main.<hash>.js` 里找 `operationName:"TweetResultByRestId"`。插件把 `queryId` 和 `features` 放在一个文件里，回答说参数不对时报 `upstream-changed`。

访客令牌不是访客身份：它不是页面生成的 cookie，是插件自己一次 POST 换来的，所以清单不写 `visitors`（[协议 §2.5.1](../plugin-protocol.md#251-访客身份-visitors) 管的是宿主开窗口取 cookie）。gallery-dl 把它缓存一小时（`_authenticate_guest`）；插件照做，放在 `${CACHE}`，不算账号凭据（推断）。换令牌会被拒：[gallery-dl #9505](https://github.com/mikf/gallery-dl/issues/9505)（2026-05）用户那边 `guest/activate.json` 回 403，维护者那边同一条能取，看出口。

**被限流或令牌被拒，退到 syndication。** `GET https://cdn.syndication.twimg.com/tweet-result?id=<id>&token=<t>`，`token` 由 id 算出：`((id / 1e15) * π).toString(36)` 去掉 `0` 和 `.`。这是 X 给嵌入帖子用的接口；照 [vercel/react-tweet](https://github.com/vercel/react-tweet)（MIT）的 `packages/react-tweet/src/api/fetch-tweet.ts` 写，用普通 UA，回空对象当不存在，`TweetTombstone` 当已删。yt-dlp 在帖子接口回 429 时也退到它（第 1197–1201 行），不过它用 `Googlebot` 当 UA，我们不照这个。

**拿到什么：**

- **正文。** 帖子接口的 `legacy.full_text`；长帖（超过普通长度）的全文在 `note_tweet.note_tweet_results.result.text`，请求的 `features` 里要开 `longform_notetweets_consumption_enabled`（yt-dlp 的写法），twitter-web-exporter 也是先读它再退到 `full_text`（`src/utils/api.ts` 的 `extractTweetFullText`）。访客令牌下长帖全文给不给，没核。syndication 只给截断的 `text`，长帖只有 `note_tweet.id`（react-tweet `src/api/types/tweet.ts`）；走 syndication 时在 `missingReason` 写「长帖只拿到前一段」（一类输出只拿到一部分，协议还说不出，[协议 §13](../plugin-protocol.md#13-待定) 第 35 项）。正文里的 `t.co` 短链按 `entities.urls[].expanded_url` 展开，不去请求 `t.co`。
- **图片。** `extended_entities.media[]` 里 `type: photo` 的 `media_url_https`，按 gallery-dl 的写法加 `name=orig` 取原图（第 79 行、第 1298 行的例子）。syndication 在 `mediaDetails[]` 里，字段同名。
- **视频和动图。** `type: video`、`animated_gif` 的 `video_info.variants[]`，取 `content_type: video/mp4` 里 `bitrate` 最高的一个（gallery-dl 第 236–238 行）；`m3u8` 不取。不带 cookie 下载，地址在 `video.twimg.com`（推断，要你核）。
- **引用的帖子。** 帖子接口的 `quoted_status_result.result`（yt-dlp 第 1102 行），syndication 的 `quoted_tweet`，带它自己的 `mediaDetails`。只取一层，写进同一个快照。
- **转帖。** 链接指向一条转帖时，取被转的原帖，身份用原帖的 id（yt-dlp `_extract_status` 最后一行的做法）。
- **前面的串。** 这条是回复、`in_reply_to_user_id_str` 等于作者自己时，在同一次 `fetch` 里按 `in_reply_to_status_id_str` 往上取，最多 20 条，每条之间隔 1 秒（推断）；不是作者自己回的就停。syndication 只给上一条（`parent`）。
- **后面的串。** 要 `TweetDetail`。gallery-dl 不登录时默认不用它，只用 `TweetResultByRestId`（`TwitterTweetExtractor.tweets`，`tweet-endpoint` 为 `auto` 时只在登录时走 detail）；访客令牌调它行不行没核，不调。你打开帖子页时页面自己会取 `TweetDetail`，twitter-web-exporter 从里面读整串（`src/modules/tweet-detail/api.ts`），所以这一部分归到[第 0 节](#0-结论)的那三种里。
- **长文章。** gallery-dl 2026-04 加了 Articles 的处理（`fieldToggles` 的 `withArticleRichContentState`，提交 974cdfee9），不登录给不给没核。拿不到时 `missingReason` 写「文章正文要登录着的页面」。

**要登录才看得到的**，接口回 `TweetUnavailable`，`reason` 是 `NsfwLoggedOut`、`NsfwViewerHasNoStatedAge`、`Protected`（yt-dlp 第 1088–1092 行，gallery-dl 第 1478–1483 行）。插件不重试，回 `not-found`、`reason: private`，`message` 写原因，`hint` 写「这条要登录着才看得到；usuyo 不用账号取 X，链接先存着」。协议的 `reason` 里没有「只有登录才看得到」，`private` 最接近；要不要加一个值，留给写插件时一起提。

**不做：** Spaces、直播（回 `unsupported-input`）；评论；投票的结果；第三方转发服务（FxTwitter 一类，[FxEmbed/FxEmbed](https://github.com/FxEmbed/FxEmbed) 的 `packages/atmosphere/src/providers/twitter/` 里有自己的账号代理和 transaction id 生成，等于把你的链接交给别人的账号去取）。

<a id="4-按创作者或关键词列"></a>
## 4. 按创作者或关键词列

不登录看不到主页时间线和搜索：gallery-dl 不登录时时间线回 403、404 就报「要登录的 cookie」（`_call` 里的 `AuthRequired("authenticated cookies", "timeline")`）。所以这两样只能在你登录着的浏览器里由脚本记你自己翻到的；twitter-web-exporter 也截搜索结果页和列表时间线（`src/modules/search-timeline/`、`src/modules/list-timeline/`）。协议还没有这类能力（[协议 §13](../plugin-protocol.md#13-待定) 第 26 项），等它定了，脚本交来的同样只是链接。

<a id="5-身份和链接"></a>
## 5. 身份和链接

- **平台 id** `x`，写在[协议 §2.3](../plugin-protocol.md#23-获取插件的字段) 的平台 id 例子里；plugin-kit 的平台列表本身还没做（[协议 §3.1](../plugin-protocol.md#31-运行器与节奏)），宿主现在只查 `contentId` 非空、不超长。`contentId` 是帖子的十进制 id，`^[0-9]{1,20}$`。
- **认得的链接**（照 yt-dlp 的 `_BASE_REGEX` 和 `TwitterIE._VALID_URL`）：主机 `x.com`、`twitter.com`，可带 `www.`、`mobile.`、`m.`；路径 `<screen_name>/status/<id>`、`i/web/status/<id>`、`statuses/<id>`，后面可以跟 `/photo/<n>`、`/video/<n>`（不进身份）。手机「分享」复制出的链接带 `?s=…&t=…`：插件的请求只用 id 拼，这些参数从不发出去，usuyo 记下的原链接不变。
- **`t.co` 短链**：`canonicalize` 只请求 `t.co`、只看它跳去哪里，不带 cookie，一看到帖子链接就停（yt-dlp 有 `TwitterShortenerIE`）。`t.co` 回的是重定向还是带跳转的页面，没核。
- **第三方转发域名**（`fxtwitter.com`、`fixupx.com` 上同样路径的链接，FxEmbed 的 README 里写的两种）：离线认出 id，不请求这些主机。别的转发域名没核，不认。
- **规范链接** `identity.url` 写 `https://x.com/i/web/status/<id>`（gallery-dl 的文档用这个形状举例，yt-dlp 认它）；取到作者以后 `resolvedUrl` 写 `https://x.com/<screen_name>/status/<id>`。作者改名不影响身份。

<a id="6-存下什么"></a>
## 6. 存下什么

```
<out>/
  post.json          接口的原样回答（帖子接口的 result 或 syndication 的 JSON），以后解析改进了可以重算
  thread/<id>.json   往上取到的同一作者的前几条，原样
  meta.json          作者、发布时间、互动数、走的哪个接口、引用和串的 id、每个媒体的原地址、没拿到的和原因
  images/001.jpg     图片，扩展名按服务器给的类型；引用帖子的图写 images/q-001.jpg
  media/video-001.mp4  视频和动图；引用帖子的写 media/q-video-001.mp4
```

`text` 是 Markdown：前面的串按时间排在前，本条，再把引用的帖子写成引用块；`kind: post`。没要的不下载（宿主只要文字时，图和视频都不取）。拿不到的写进 `missing` 和 `missingReason`，不写空文件。

<a id="7-错误码"></a>
## 7. 错误码

对到[协议 §3.3](../plugin-protocol.md#33-信封与错误码) 的码：

| 看到什么 | 回什么 |
|---|---|
| 链接认不出帖子 id，或是 Spaces、直播 | `unsupported-input` |
| `TweetTombstone`、syndication 回 404 或空对象、帖子接口说找不到 | `not-found`；只有墓碑的文字明说删了才写 `reason: deleted`（账号被封的帖子也是墓碑，而全是 `deleted` 时条目会记成「平台上已看不到」） |
| `TweetUnavailable` 的 `NsfwLoggedOut`、`NsfwViewerHasNoStatedAge`、`Protected` | `not-found`，`reason: private`（见[第 3 节](#3-取单条)） |
| 帖子接口 429，退到 syndication 后也 429 | `rate-limited`，`retryAfter` 取 `x-rate-limit-reset`（gallery-dl `_handle_ratelimit` 读这个头） |
| `guest/activate.json` 回 403，syndication 也拒 | `blocked`（看出口，见 [gallery-dl #9505](https://github.com/mikf/gallery-dl/issues/9505)） |
| 帖子接口回参数、`queryId`、`features` 一类的错，或回答里没有 `tweetResult` | `upstream-changed` |
| 媒体地址不在 `pbs.twimg.com`、`video.twimg.com` 上，或下载失败 | 不报错，`missing` 加上那一类，`missingReason` 写原因 |
| 连不上、超时 | `network` |

不回 `login-required`：这个插件不用账号，回了会让界面去找一个不存在的登录。

<a id="8-节奏"></a>
## 8. 节奏

建议值 `batch: true`、`interval: 10s`、`daily: 300`（推断）。依据只有这些：不带账号，风险在出口 IP 上，不在账号上；yt-dlp、gallery-dl 都按 429 和 `x-rate-limit-remaining` 退让，说明访客令牌有限额，但没查到具体数；X 服务条款对 24 小时内超过 100 万条帖子定了违约金（[06 §16.1](../research/06-platform-access-and-account-risk.md#161-平台规则与可检测性)），离这个量差得远。往上取串时一次 `fetch` 里最多 20 个请求，算一次调用。

<a id="9-清单"></a>
## 9. 清单

照现行协议的词（[协议 §2.3](../plugin-protocol.md#23-获取插件的字段)）：

```yaml
protocol: 1
id: usuyo-fetch/x
kind: fetch
name: X
description: 按帖子链接取正文、图片、视频和引用的帖子，不用账号；书签和喜欢由你浏览器里的脚本交来，或读你下载的官方归档
version: 0.1.0
license: MIT
maintainers: [Fldicoahkiin]
upstream:
  - name: twitter-web-exporter（脚本截获书签、喜欢的做法）
    url: https://github.com/prinsss/twitter-web-exporter
    license: MIT
  - name: react-tweet（syndication 的请求）
    url: https://github.com/vercel/react-tweet
    license: MIT
platforms: [x]
match:
  - "x.com/*"
  - "*.x.com/*"
  - "twitter.com/*"
  - "*.twitter.com/*"
  - "t.co/*"
  - "fxtwitter.com/*"
  - "fixupx.com/*"
access: public-web
login: none
capabilities:
  fetch:
    access: public-web
    login: none
    timeout: 30m
    outputs: [text, images, media:video, meta]
  list:
    access: official-export
    login: none
    order: unordered
    files: ["twitter-*.zip"]
pacing:
  x: {batch: true, interval: 10s, daily: 300}
permissions:
  fs:
    read: ["${INPUT}"]
    write: ["${OUT}"]
  network:
    mode: allowlist
    hosts: ["api.x.com", "x.com", "cdn.syndication.twimg.com", "pbs.twimg.com", "video.twimg.com", "t.co"]
  env: []
  devices: []
  gpu: false
run:
  command: ["${PLUGIN_DIR}/usuyo-fetch-x"]
```

- 没有 `sites`：usuyo 里不登录 X。没有 `visitors`：访客令牌是插件自己换的，不是宿主开窗口取的 cookie（[第 3 节](#3-取单条)）。
- `match` 写宽，粗筛交给 `canonicalize`（[协议 §3.5](../plugin-protocol.md#35-canonicalize-与身份)）；`*.` 前缀不含裸域，所以两种都写。
- `list` 只读归档，交的条目只带 `url`，内容照样由 `fetch` 去取（[协议 §3.9](../plugin-protocol.md#39-从文件导入)）。归档的文件名照协议的例子写成 `twitter-*.zip`，X 现在给的文件名要你核。这个插件有网络权限，所以读归档的 `list` 也照节奏排，不能像 `network.mode: none` 的导入插件那样并行；归档一次几千条，可以接受。
- 浏览器脚本不在清单里：它不是宿主启动的进程，走[协议 §9.4](../plugin-protocol.md#94-浏览器扩展与油猴脚本)，交来的链接按 `fetch` 取。

<a id="10-风险和证据"></a>
## 10. 风险和证据

- **单条不带账号**：账号不会因此出事，没有找到 X 上像小红书 [XHS-Downloader #305](https://github.com/JoeanAmier/XHS-Downloader/issues/305) 那样匿名请求连回分享人账号的报告；插件也从不发分享链接里的参数。风险在出口：访客令牌被拒（[gallery-dl #9505](https://github.com/mikf/gallery-dl/issues/9505)）、被限流。
- **脚本记书签和喜欢**：在你自己的浏览器里，不发额外请求，你自己翻，和平时看书签一样。X 的自动化规则禁止「用脚本操作网站」（[06 §16.1](../research/06-platform-access-and-account-risk.md#161-平台规则与可检测性)），只截获、不操作的脚本没有找到处罚报告（[06 §16.2](../research/06-platform-access-and-account-risk.md#162-issue-里的账号问题)）。
- **带账号的程序**（usuyo 不做的那几条）：X 上注册 3 年的主号一次取 800 个关注后被封（[twikit #338](https://github.com/d60/twikit/issues/338)，`private-api`）；Agent-Reach 的维护者建议账号注册和使用保持同一个 IP 段（[Agent-Reach #63](https://github.com/Panniantong/Agent-Reach/issues/63)）。这两条说明的是：账号一旦交给程序，节奏和出口都要像人。
- **开发者这一侧**：X 对 Nitter 发过停止侵权函，仓库已归档（[06 §16.3](../research/06-platform-access-and-account-risk.md#163-下架停更与法律风险)）；X 服务条款写明只能经公开接口访问、未经同意不许爬取。插件放在用户自己的获取插件组织（[CLAUDE.md](../../CLAUDE.md) 原则 1），只读、不发帖、不点赞。
- **会变的**：`queryId` 和 `features` 的写法、syndication 的 `token` 算法、访客令牌的发放。都集中在一个文件里，变了回 `upstream-changed`。

<a id="11-依据"></a>
## 11. 依据

2026-11-11 用 `gh api` 读的，没有运行任何一份：

| 项目 | 读的版本 | 读了什么 |
|---|---|---|
| [yt-dlp/yt-dlp](https://github.com/yt-dlp/yt-dlp)（Unlicense） | master `412e7f0`（2026-11-04） | `yt_dlp/extractor/twitter.py`；issue [#17058](https://github.com/yt-dlp/yt-dlp/issues/17058)、[#17563](https://github.com/yt-dlp/yt-dlp/issues/17563)；2025-12 去掉了账号密码登录（[#15432](https://github.com/yt-dlp/yt-dlp/pull/15432)），只剩 cookie |
| [mikf/gallery-dl](https://github.com/mikf/gallery-dl)（GPL-2.0，只读不抄） | master `811d917`（2026-10-31；维护者在 issue 里链接 Codeberg 上的提交，GitHub 这份照常同步） | `gallery_dl/extractor/twitter.py`、`docs/configuration.rst` 的 `conversations`；issue [#9505](https://github.com/mikf/gallery-dl/issues/9505) |
| [vercel/react-tweet](https://github.com/vercel/react-tweet)（MIT） | main（2026-09-24） | `packages/react-tweet/src/api/fetch-tweet.ts`、`src/api/types/` |
| [prinsss/twitter-web-exporter](https://github.com/prinsss/twitter-web-exporter)（MIT） | v1.4.3（2026-08-31） | `src/core/extensions/manager.ts`、`src/modules/{bookmarks,likes,tweet-detail}/api.ts`、`src/utils/api.ts`；issue #19、#51、#120 |
| [afar1/fieldtheory-cli](https://github.com/afar1/fieldtheory-cli)（MIT） | 2026-09-05 | `src/graphql-bookmarks.ts`；按 locked、suspended、403、banned 搜 issue，没有相关的 |
| [steipete/birdclaw](https://github.com/steipete/birdclaw)（MIT） | main | README、`docs/archive.md`、`src/lib/archive/slices.ts` |
| [xdevplatform/xurl](https://github.com/xdevplatform/xurl)（MIT） | 2026-11-07 | README、`api/shortcuts.go` |
| [FxEmbed/FxEmbed](https://github.com/FxEmbed/FxEmbed)（MIT） | main（2026-11-11） | 只看了目录：`packages/atmosphere/src/providers/twitter/` 下有账号代理和 transaction id |

**没能核的，都要你来看：** 访客令牌下长帖全文、长文章正文给不给；`TweetDetail` 用访客令牌行不行；`video.twimg.com`、`pbs.twimg.com` 不带 cookie 下不下得到（成人内容的媒体尤其）；`t.co` 回的形状；官方归档里有没有书签、文件名和字段；付费接口的价格和上限；在页面里怎么拿到自己的用户 id；你那个出口上访客令牌发不发得出来。

<a id="12-测试"></a>
## 12. 测试

步骤和要注意的事照 [platforms/README 第 4 节](README.md#4-怎么相对安全地试)。X 自己的部分：

- **离线能测的**：`canonicalize` 的全部链接形状（含 `?s=&t=`、`/photo/1`、转发域名，`t.co` 对着本机假站点）；syndication 的 `token` 算法，拿 react-tweet 的公式对几组 id；用照上游类型手写的 JSON 测帖子接口和 syndication 两种回答的解析，含长帖、引用、动图、转帖、`TweetUnavailable` 各个 `reason`、`TweetTombstone`；错误码映射；脚本对着保存的 `Bookmarks`、`Likes` 回答样例记条目、只记自己的喜欢、握手和交 `/v1/batches`（照小红书脚本的测试）；归档 zip 的 `list` 用手写的 `like.js`。
- **只有你能测的**：先测分叉的前提：在你平时上 X 的那个出口上粘一条公开帖子，看正文、图、视频有没有；插件那一行要是写「平台挡住了这个插件」，就是访客令牌和 syndication 都被拒了，告诉开发这边，X 的做法要改。再测一条长帖、一条带视频的引用、一条串的第一条；一条成人内容的帖子，看是不是停在「只有链接」；书签页、喜欢页用脚本交十条；申请一份官方归档，看里面有没有书签。
