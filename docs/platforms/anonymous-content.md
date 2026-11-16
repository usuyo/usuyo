<a id="不带账号取内容参考项目怎么做我们差在哪"></a>
# 不带账号取内容：参考项目怎么做、我们差在哪

这一页回答一件事：各平台的单条内容（正文、图片、视频）怎样不带账号取到，参考项目在源码里是怎么做的，我们的插件和它们比差在哪；最后记 2026-11-15 小红书那次事故和按平台的改动清单。分路的规矩（账号只用来列出存过的，内容一律不带账号取）在 [BUILD §1](../BUILD.md#1-用户定下的做法) 和 [README §1](README.md#1-怎么读这一页)；每个平台的获取设计在 [小红书](rednote.md)、[视频号](wechat-channels.md)、[B站](bilibili.md)、[X](x.md)；各插件做到哪、对真平台跑过的记录在 [routes.md](routes.md)。这里写过的不在那几份重抄，那几份写过的这里只链接。

<a id="0-怎么读这一页"></a>
## 0. 怎么读这一页

- **出处三种标法。** 「源码」是在参考项目源码里读到的，写成 `项目/路径:行`；「issue」是别人在 issue 或 PR 里报告的，带链接；「推断」是这边的判断，没有实测。没有标的句子是我们自己插件的代码，路径相对 `usuyo-fetch/`。
- **读的是哪一版。** 2026-11-15 浅克隆、只读：MediaCrawler 4d629b5（11-14）、XHS-Downloader dfad51b（11-06）、ParseHub 7075f82（11-13）、xhs-vault b0b8e3f（08-18）、yt-dlp ee25591（11-14，只看 `yt_dlp/extractor`）、gallery-dl 26dfd4e（11-14）、TikTokDownloader f766a6e（11-03）、Douyin_TikTok_Download_API（下称 dtk）3c045e1（11-09）、parse-video 00e8e11（11-08）、yutto（11-15 克隆）、wx_channels_download（下称 WXD）1d3b467（11-10）、wx_channel 50daa8c（11-15）。行号对的是这几版。xhshow 只读了 GitHub 上的 HEAD。
- **这一轮没有对任何平台发请求**，只读代码、文档和 GitHub 上的 issue。结论里凡是「行不行」都还要真跑一次，见[第 11 节](#11-要真跑一次才知道的)。
- **issue 是抽样。** 两个查 issue 的会话碰到过 GitHub 搜索的限流，列出的 issue 够看出趋势，不是全部。
- 参考项目只当参考，不包装、不调用（[routes.md](routes.md#各平台的获取方式)）；下面「照哪个项目做」说的是照它的做法写我们自己的实现。

<a id="1-总表"></a>
## 1. 总表

「不带账号」指不用任何人的登录；「访客身份」指站点给任何没登录访客的那组 cookie（[协议 §2.5.1](../protocol/manifest.md#251-访客身份-visitors)），它不算账号。「签名」指请求里要带按平台的算法算出的参数，按 [BUILD §2](../BUILD.md#2-干活的规矩) 这类方法这边不对真平台跑。

| 平台 | 单条内容 | 图片 | 视频 | 收藏等清单 | 我们做到哪 |
|---|---|---|---|---|---|
| 小红书 | 能：笔记页 HTML，带 `xsec_token`、`xsec_source`，要访客身份；今天同一出口上空 cookie 只通了一次，原因是推断（[§2.2](#22-访客身份是什么)） | 能：CDN 直接下，带 Referer，不带 cookie | 能：同上；`originVideoKey` 原片 2026-01 起页面不给了 | 只能带账号；最低风险是你自己的浏览器里自己往下翻、脚本只截获页面自己的请求 | `note-page` 第一个请求就带访客身份和取它时的 UA，被拒再用手机版页面、不带 cookie 取一次；11-15 真跑两次都被送去 `/login`（[§11](#11-要真跑一次才知道的)）；改之前的差别见 [§2.5](#25-我们的-note-page-差在哪) |
| 微信视频号 | 部分：匿名 `get_feed_info` 只给作者、描述、封面、时间 | 封面能 | 不能：要元宝账号换 `generalToken`，或改客户端 | 只能带账号，网页版普通浏览器里拿不到 | `feed-info`、`yuanbao` 和参考项目一致（[§3](#3-微信视频号)） |
| B站 | 能：视频页 HTML 或接口；接口有的要 WBI 签名，不带 cookie 时有 412、-352 | 能 | 能，不登录约 480p，最多 720p | 公开收藏夹不带账号能列；自己的私密收藏夹、稍后再看要账号 | `view` 先取视频页、不带 cookie，读不到的那一半再调不签名的接口；11-15 真跑：视频信息来自页面，流来自 `playurl`（[§4](#4-b站)） |
| X | 能：访客令牌调 GraphQL，或 syndication 嵌入接口；敏感内容、受保护账号要登录 | 能，`name=orig` | 能，最高码率的 mp4 | 只能带账号；官方归档有喜欢 | `guest`、`embed` 和 gallery-dl、yt-dlp 一致（[§5](#5-x)） |
| 抖音 | 网页接口没有不带 cookie 的路：要访客 cookie（含只有真浏览器才拿得到的 `UIFID_TEMP`）加两种签名；分享页 2026-08 起不带作品数据 | 随单条 | 随单条，地址约 3 小时过期 | 只能带账号，接口要签名 | `share-page` 取不到；`h5-token` 默认关、没跑过（[§6](#6-抖音)） |
| 快手 | 能：手机页 `fw/photo/<id>` 的 `INIT_STATE`，最好走 App 分享短链 | 随单条 | 随单条 | 只能带账号 | `share-page` 对真平台跑通过（[§7.1](#71-快手)） |
| 微博 | 能：`ajax/statuses/show` 加访客 cookie `SUB`、`SUBP` | 能，带 Referer | 能 | 只能带账号 | `status` 用访客身份，11-16 对真平台跑通一条（[§7.2](#72-微博)） |
| 知乎 | 能：页面里的 `js-initialData`，要页面自己算出的 `__zse_ck` 和 `d_c0` | 能 | 视频要 `__zse_ck` | 公开收藏夹接口不带 cookie 回 401 | `page` 用访客身份，11-16 试了三次还取不到：访客窗口开首页拿不到 `__zse_ck`（[§7.3](#73-知乎)） |

<a id="2-小红书"></a>
## 2. 小红书

<a id="21-参考项目怎么取单条"></a>
### 2.1 参考项目怎么取单条

五个项目取一篇笔记都落到同一个地方：GET 笔记页 HTML，解析 `window.__INITIAL_STATE__.note.noteDetailMap`。页面请求不签名；签名（`x-s`、`x-t`、`x-s-common`）只用在 `edith.xiaohongshu.com` 的接口上。

| 项目 | 请求 | 头和 cookie | cookie 从哪来 | 节奏和重试 |
|---|---|---|---|---|
| XHS-Downloader | 你给的链接原样请求，正则要求链接带查询串（`source/application/app.py:117-128`），`xsec_token`、`xsec_source` 跟着链接走 | `accept`、`accept-language`，Referer 是首页（`source/module/static.py:27-30`、`source/module/tools.py:54-57`）；curl_cffi 模仿 Chrome 146（`static.py:25`） | 可选，默认空（`source/module/settings.py:19`）；写进 `.xhslink.com`、`.xhslink.cn`、`.xiaohongshu.com`、`.rednote.com` 四个域（`source/module/manager.py:24-28,346-358`），是 [#495](https://github.com/JoeanAmier/XHS-Downloader/issues/495) 的修复 | 每次请求后等对数正态分布的随机时长，均值 6 秒，截在 1.2–30 秒（`source/module/tools.py:76-88`）；重试 5 次、不退避；下载并发 4 |
| ParseHub | 你给的链接，解析时保留 `xsec_token`、`xsec_source` | 桌面 Chrome UA 配 Chrome 的 Accept，或 iPhone Safari UA 配 Safari 的 Accept；注释说手机 UA 配 Chrome 的 Accept 会被送去验证页（`src/parsehub/provider_api/xhs.py:20-30`） | 三层依次试：你的 cookie → 新开一个游客 session → 不带 cookie 的手机页（`src/parsehub/parsers/parser/xhs.py:36-58`）。游客 session 在本地生成 `a1`、`webId`，用 xhshow 签名 POST `/api/sns/web/v1/login/activate`，只取回答里的 `web_session`，每条链接新开一个（`provider_api/xhs.py:53-78`） | 最终地址含 `/login` 报「需要登录」，含 `/404/sec` 报「被风控拦截」（`provider_api/xhs.py:45-50`）；没有间隔 |
| xhs-vault | 清单里的链接，没有就拼 `explore/<id>?xsec_token=…&xsec_source=pc_user`（`scraper/fetch.py:163-164`） | Chrome 131 UA、Referer 首页 | 可选，读 `~/cookies.json`，没有就发空的（`fetch.py:55-67`）；README 说带 `xsec_token` 的 explore 链接未登录也回完整的状态对象（`README.md:40`，2026-08 的说法） | 每篇隔 1.5–3 秒随机（`fetch.py:191`）；落到 `/login` 整批停下（`fetch.py:168-171`） |
| MediaCrawler | 主路径是签名接口 POST `/api/sns/web/v1/feed`（`media_platform/xhs/client.py:331-366`），`xsec_source` 空时填 `pc_search`；回退到 GET `explore/<id>?xsec_token=&xsec_source=`（`client.py:716-728`） | 回退那一路调用时传 `enable_cookie=True`（`media_platform/xhs/core.py:317-318`），实际总带登录 cookie | 一律要登录：先开首页，`selfinfo` 不通就扫码登录（`core.py:104-120`、`client.py:249-281`）；没有匿名模式 | 并发 1（`config/base_config.py:105`），每篇后睡 2 秒（`base_config.py:145`）；401/403/429 算受限，461/471 算验证码，300012 算 IP 被封，300011 算账号受限（`client.py:153-184`） |
| yt-dlp | 普通 GET 你给的链接（`yt_dlp/extractor/xiaohongshu.py:45`） | 默认头；cookie 只来自 `--cookies` | 无 | 无 |

**解析。** 都要先把 JS 写法换成 JSON：XHS-Downloader 把 `new Map([])`、`new Set([])` 换成 `[]`、`:undefined` 换成 `:null`，取最后一个状态脚本（`source/expansion/converter.py:9-49`）；ParseHub 按词边界换 `undefined`（`provider_api/xhs.py:81-103`）；xhs-vault 只换值位置上的 `undefined`（`scraper/fetch.py:38-42`）。`new Set` 是 10-15 前后出现的（issue：[#497](https://github.com/JoeanAmier/XHS-Downloader/issues/497)），我们的 `state.rs` 已经处理（[rednote.md §2.2](rednote.md#22-取单条)）。

**在登录着的页面里用 `fetch` 取笔记页**会触发 300013，失败的原因是采集时和下载时的 `xsec_token` 不一致（issue：[#362](https://github.com/JoeanAmier/XHS-Downloader/issues/362)，2026-04）。

<a id="22-访客身份是什么"></a>
### 2.2 访客身份是什么

- **是一组没登录的 cookie。** 源码里出现的是 `a1`、`webId`、`xsecappid`（固定 `xhs-pc-web`）和一个游客 `web_session`（ParseHub `provider_api/xhs.py:56`）。`a1` 是毫秒时间戳的十六进制加 30 个随机字符加校验，`webId` 是 `md5(a1)`（xhshow `src/xhshow/utils/random_gen.py:105-129`，HEAD）；签名接口缺 `a1` 直接报错（xhshow `client.py:605-607`）。游客 `web_session` 由 `login/activate` 发，`Max-Age=31536000`，一年（issue：[yt-dlp #15467](https://github.com/yt-dlp/yt-dlp/issues/15467)，2026-01，issue 里贴的回答头）。
- **`gid`、`acw_tc`、`websectiger`、`sec_poison_id`、`webBuild`、`loadts`** 在五个项目的代码里都没出现。推断：`acw_tc` 像阿里云 WAF 的会话 cookie，`websectiger`、`sec_poison_id` 像前端安全脚本的标记，`gid` 像设备指纹，`webBuild`、`loadts` 是前端版本和加载时间。哪几个是页面放行必须的，没人写过。
- **三种来法。** ParseHub 自己算 `a1` 再签名换 `web_session`（签名，`private-api`）；MediaCrawler 开一个真浏览器打开首页，让站点脚本写 cookie 再读出来（`core.py:104`、`core.py:384-390`），它把登录前那个 `web_session` 记作「未登录」（`media_platform/xhs/login.py:191-193`）；XHS-Downloader 让你从浏览器里复制一份没登录的 cookie。我们的访客窗口（[协议 §2.5.1](../protocol/manifest.md#251-访客身份-visitors)）是第二种：不登录的看不见的窗口打开首页，存下页面自己生成的 cookie，不算任何东西。
- **不带任何 cookie 为什么只通了一次。** 报告：完整长链加空 cookie 被送回首页，加访客 cookie 成功（issue：[#495](https://github.com/JoeanAmier/XHS-Downloader/issues/495)，2026-10-14；[#499](https://github.com/JoeanAmier/XHS-Downloader/issues/499)）；不登录的桌面请求被送去 `/explore?undertake_note_error=…` 或 `/login`（issue：[ParseHub #163](https://github.com/z-mio/ParseHub/pull/163)）；没有 `web_session` 时页面有标题但没有视频流（issue：yt-dlp #15467）。今天的实测（[§9.1](#91-经过)）是第一次空 cookie 的 curl 回了完整页面，之后几小时同一出口每次 302 到 `/login`。推断：站点放行一个出口上第一次空 cookie 的访问，之后认出「这个出口上始终不带 `a1`、`web_session` 的客户端」，要它先有访客身份；curl 不执行脚本、不存 cookie，所以一直补不上。另一种解释同样说得通：那几个小时里同一出口上登录着的那一批正被转去验证页，被标记的是出口本身；那样的话带访客身份也未必有用。只有一次观察，两种解释分不开，要真跑（[第 11 节](#11-要真跑一次才知道的)）。`login/activate` 本身也按出口限制（issue：[rednote-skills #1](https://github.com/MrMao007/rednote-skills/issues/1)，300012「IP 存在风险」）。
- **笔记页和笔记接口要的不一样。** 笔记页：访客身份加 `xsec_token`、`xsec_source`，不签名。笔记接口（`/api/sns/web/v1/feed`）：访客或登录的 cookie，加 `x-s`、`x-t`、`x-s-common`、`x-b3-traceid`（MediaCrawler `client.py:90-125`），属 `private-api`。

<a id="23-xsec_token"></a>
### 2.3 xsec_token

- **和哪个 `xsec_source` 配。** 各项目照链接原样带；xhs-vault 把收藏页截获的 token 配 `pc_user`（`userscript/xhs-export.user.js:147`、`scraper/fetch.py:164`）；MediaCrawler 搜索来的默认 `pc_search`，创作者列表 `pc_feed`（`client.py:347-348,589,620`）。网页上收藏页里的笔记链接本身就是 `/user/profile/<uid>/<noteId>?xsec_token=…&xsec_source=pc_user`（今天的记录）。只带 token 不带 `xsec_source` 的 App 分享链会被送去 `/404/sec`（issue：[ParseHub #163](https://github.com/z-mio/ParseHub/pull/163)）。推断：照页面给的配，收藏页来的就是 `pc_user`。
- **多久失效。** 找到的 issue 里没有一个给出时长。维护者说「更换 IP 或账号后需要重新获取链接」（issue：[#309](https://github.com/JoeanAmier/XHS-Downloader/issues/309)，2025-10）；没有 token 拿不到笔记，只凭 note id 拿不到 token（issue：[#186](https://github.com/JoeanAmier/XHS-Downloader/issues/186)、[#397](https://github.com/JoeanAmier/XHS-Downloader/issues/397)）。调研里「几个小时后 404」的说法在 [06 §1.3](../research/06-platform-access-and-account-risk.md#13-混合路径视觉层拿链接公开页面取内容)，来源是别的项目的实测，不是平台说明。
- **匿名请求认不认收藏页拿到的 token。** 认：xhs-vault 的整条路就是这样（`README.md:40`），今天空 cookie 的第一次 curl 也用的是收藏页的 token 并拿到了完整页面。但 token 是给登录的那个账号发的，匿名批量用它可能算到那个账号头上：不填 cookie 批量取约 108 条后，复制链接的账号被下线（issue：[#305](https://github.com/JoeanAmier/XHS-Downloader/issues/305)；归因是推断，维护者只答了「作品链接携带跟踪参数」）。

<a id="24-图片和视频"></a>
### 2.4 图片和视频

- **图片字段。** `imageList[].urlDefault`（`sns-webpic-qc.xhscdn.com/<时间戳>/<签名>/<token>!nd_dft_…`，压过的 webp）、`url`、`infoList[]`、`urlPre`。
- **原始文件地址。** XHS-Downloader 从 `fileId`、`urlDefault`、`urlPre` 里取出 token（去掉主机、时间戳、32 位签名段和 `!` 后缀），拼 `https://sns-img-bd.xhscdn.com/<token>`，或按格式拼 `https://ci.xiaohongshu.com/<token>?imageView2/format/<fmt>`（`source/application/image.py:10-56`）。ParseHub 拼 `https://sns-img-hw.xhscdn.com/<path>?imageView2/format/jpg`（`provider_api/xhs.py:133-143`）；10-27 起部分图片地址换成 `sns-i27.rednotecdn.com/notes_pre_post/<id>?imageView2…&sign=…`，只取路径再拼才不 404，不加 `format/jpg` 时部分原图是 HEIC（issue：[ParseHub #198](https://github.com/z-mio/ParseHub/pull/198)）。
- **视频字段。** `video.consumer.originVideoKey` 拼 `https://sns-video-bd.xhscdn.com/<key>` 是原片（XHS-Downloader `source/application/video.py:8-28`）；没有就在 `video.media.stream` 下按高度、码率挑，取 `backupUrls[0]` 或 `masterUrl`（`video.py:30-56`）。MediaCrawler 注释说分档名从 `h264/h265/av1` 换成了 `EF4`–`EF7`，所以不写死档名，凡是列表都当候选（`media_platform/xhs/media.py:61-75`）。实况图在 `imageList[].stream`。
- **字幕。** 视频笔记的平台字幕在 `video.media.video.subtitles`（按 `source`、`zh-CN`、`en-US`），地址带 `sign` 和 `t` 过期参数，要在取页面的同一次里下载（xhs-vault `scraper/build_corpus.py:39-59`、`scraper/fetch.py:96-117`）；它实测 152 个视频里 124 个有字幕（`README.md`）。
- **时效和头。** 图片新地址带 `sign=`；字幕带 `t`；视频流地址有没有时效，代码里没写。下载：XHS-Downloader 不带 cookie、不带 Referer（`source/module/manager.py:115-121`），MediaCrawler 带 Referer 首页（`core.py:517-521`）。今天 83+50 条带 Referer、不带 cookie 零失败（[§9.1](#91-经过)）。
- **最高画质的条件。** `originVideoKey` 2026-01-05 起从 PC 页面消失，只剩 720p、1080p 转码（issue：[#325](https://github.com/JoeanAmier/XHS-Downloader/issues/325)；[yt-dlp #15572](https://github.com/yt-dlp/yt-dlp/issues/15572)）；维护者说不再提供原画（issue：[#429](https://github.com/JoeanAmier/XHS-Downloader/issues/429)），也有人配了 cookie 又能下原画（issue：[#475](https://github.com/JoeanAmier/XHS-Downloader/issues/475)）。README 说没有 cookie 只有低分辨率（`README.md:81`）。推断：页面给哪几路流随 cookie 变，挑流的代码不看 cookie。用户现在不追画质，记下即可。

<a id="25-我们的-note-page-差在哪"></a>
### 2.5 我们的 note-page 差在哪

`note-page` 在 11-15 改之前做的（`rednote/src/`；改了哪些见 [§10](#10-改动清单)，现在的做法在 [rednote §2.4](rednote.md#24-插件做成的样子)）：

- 请求 `https://www.xiaohongshu.com/<原路径>?xsec_token=…&xsec_source=…`，只留这两个参数（`link.rs:57`）；桌面 Chrome 142 UA 配 Chrome 的 Accept（`http.rs`）。
- **先不带 cookie 请求**（`fetch.rs:82`），被送去 `/`、`/explore`、`/login`、`/website-login/*` 或页面没有这篇时，宿主交了访客身份就带上再请求一次（`fetch.rs:82-103`、`fetch.rs:219-224`）。
- `/website-login/captcha`、HTTP 403/406/461、JSON 的 300012/300013/300015/461/-1、验证页字样回 `blocked`（`fetch.rs:142,216,253,283`）；`/404/sec_` 带 300031 回 `link-expired`。
- 图片取 `urlDefault` → `url` → `infoList` 里 `WB_DFT`（`state.rs:316`）；视频只在 `h264`、`h265`、`av1`、`h266` 四个名字下挑（`state.rs:346`）；图片视频带 Referer 首页、不带 cookie（`http.rs:118`），只连 `*.xhscdn.com`（`site.rs:37`）。
- 节奏 `interval: 15s`、`daily: 100`（`usuyo-plugin.yaml:106`）；当时宿主按固定间隔排，`blocked` 暂停一小时（现在的做法见 [§10](#10-改动清单)）。
- 两个脚本记下的链接是 `/explore/<id>?xsec_token=…`，没有 `xsec_source`（`userscript/src/window.js:56`、`userscript/src/manager.js:271`）。

和参考项目比：

| 哪里 | 我们 | 参考 | 判断 |
|---|---|---|---|
| 第一个请求带不带访客身份 | 不带，被送走才带 | ParseHub、XHS-Downloader 有 cookie 就先带 | 我们每条先发一次空 cookie 的请求；如果今天的观察是「空 cookie 在一个出口上只放行一次」（推断），之后每条都多一次被送去 `/login` 的请求；就算不是，第一个请求带上访客身份也和参考项目一致、少一次请求。要改 |
| `xsec_source` | 脚本不记 | xhs-vault 配 `pc_user`；页面自己的链接是 `pc_user` | 要改 |
| 间隔 | 固定 15 秒 | XHS-Downloader 均值 6 秒的随机；xhs-vault 1.5–3 秒随机 | 我们更慢，但每次一样长；加随机 |
| 碰到验证页 | 暂停一小时，之后自己接着取 | PR [#502](https://github.com/JoeanAmier/XHS-Downloader/pull/502)（未合并）碰到 429/461/471、captcha、300011/300012 等暂停全部任务，等你点开始；xhs-vault 落到 `/login` 整批停；有人说触发后半天才解开（issue：[MediaCrawler #833](https://github.com/NanmiCoder/MediaCrawler/issues/833)） | 小红书上一小时自己接着取太短；停到你说接着取 |
| 识别的风控码 | 没有 HTTP 471、JSON 300011 | MediaCrawler `client.py:153-184` | 补上 |
| 视频分档名 | 写死四个 | MediaCrawler 不写死（`media.py:61-75`） | 页面改成 `EF*` 时我们回「页面没有给出视频地址」；要改 |
| 图片主机 | 只认 `*.xhscdn.com` | 新地址在 `*.rednotecdn.com`（ParseHub #198） | 碰到新地址时一张都不下；要改 |
| 平台字幕 | 不取 | xhs-vault 取 `video.media.video.subtitles` | 能少一次转写；加进 `subtitle` 输出 |
| 原始文件地址 | 取页面给的 `urlDefault` | XHS-Downloader、ParseHub 拼原图地址 | 用户现在不追画质，不改，记下 |

<a id="3-微信视频号"></a>
## 3. 微信视频号

- **参考项目。** ParseHub 不是匿名的：先带腾讯元宝的登录 cookie POST `yuanbao.tencent.com/api/weixin/get_parse_result`，取回 `playable_url` 里的 `token`、`eid`，再不带 cookie POST `channels.weixin.qq.com/finder-preview/api/feed/get_feed_info`（body `{"baseReq":{"generalToken":token},"exportId":eid}`，Origin 和 Referer 是 `playable_url`），视频取 `feedInfo.h264VideoInfo.videoUrl` → `videoUrl` → `h265VideoInfo.videoUrl`，没有解密这一步（`src/parsehub/provider_api/wxchannels.py:47-110`）。WXD 的分享链接走同一条元宝的路（`pkg/scraper/wxchannels/yuanbao.go:104-247`），没有元宝 cookie 就去问被改过的微信页面（`pkg/scraper/wxchannels/client.go:93-98`），文档说元宝 cookie 约一个月、元宝的用法有被限制的风险（`docs/cli/deploy.md:132,159-162`）。wx_channel 在被改过的微信页面里带 `credentials:'include'` 调 `get_feed_info`（`internal/assets/inject/api_client.js:144-172`），要客户端上下文。
- **改客户端的那一路**拿到 `media.url + media.urlToken` 和 `decodeKey`，用 ISAAC64 生成的密钥流异或前 128 KB（WXD `pkg/scraper/wxchannels/decrypt.go:18-169`，wx_channel `internal/utils/crypto_helper.go:26-80`）；画质靠 `X-snsvideoflag`（WXD `pkg/scraper/wxchannels/inject/channels.utils.js:1121-1131`）。原画 2026-09 起拿不到、转发来的从不给原画（issue：[WXD #562](https://github.com/ltaoo/wx_channels_download/issues/562)、[#581](https://github.com/ltaoo/wx_channels_download/issues/581)、[wx_channel #98](https://github.com/nobiyou/wx_channel/issues/98)）；11-05 起在微信 4.2 上注入失败（issue：WXD [#596](https://github.com/ltaoo/wx_channels_download/issues/596)、[#603](https://github.com/ltaoo/wx_channels_download/issues/603)、[#607](https://github.com/ltaoo/wx_channels_download/issues/607)）。
- **没有完全匿名的单条路。** 能匿名拿到的只有 `get_feed_info`（`shortUri`）给的作者、描述、封面、时间（[wechat-channels.md §3.2](wechat-channels.md#32-取单条)）。视频地址要元宝换来的 `generalToken` 或客户端。
- **我们。** `feed-info` 发的就是分享页自己的那个匿名请求（`wechat-channels/src/feed.rs:98-118`），2026-11-12 跑通、11-15 宿主改了获取策略和节奏以后又跑通一次（[routes.md §4](routes.md#4-对真平台跑过的记录)）；`yuanbao` 照 ParseHub。和参考项目一致，没有要补的。下载头：wx_channel 带 Origin `channels.weixin.qq.com`、Referer `…/finder-preview/pages/feed`（`internal/api/share_resolve.go:263-266`），WXD 什么都不带；我们的 `download.rs` 照 ParseHub。节奏 `interval: 30s`、`daily: 50`（`wechat-channels/usuyo-plugin.yaml:110`）；`get_feed_info` 回 429 是 `rate-limited`，5xx 是网络错误，回答说这一条看不到是 `not-found`（`wechat-channels/src/feed.rs:60-80`）。
- **清单**只有改客户端的工具能自动列；现在的做法是手机上逐条复制分享链接，在导入页一次粘贴多条（[wechat-channels.md §3.1](wechat-channels.md#31-列出你存过的)）。

<a id="4-b站"></a>
## 4. B站

- **参考项目。** yt-dlp 先取视频页 HTML 读 `window.__INITIAL_STATE__`，页面带 `window._riskdata_.v_voucher` 就当限流，读不到再调签名的 `x/web-interface/wbi/view/detail`（`yt_dlp/extractor/bilibili.py:723-753`）；流先读页面的 `window.__playinfo__`，没有或没登录时调 `x/player/wbi/playurl`，`fnval=4048`，没登录加 `try_look=1`（`bilibili.py:213-230,829-834`）。yutto 调不签名的 `x/web-interface/view` 和 `x/player/playurl?qn=127&fnval=4048&fourk=1`，只带 UA 和 Referer，登录可选（`src/yutto/api/ugc_video.py:87,212`、`src/yutto/utils/fetcher.py:129-157`）。MediaCrawler 一律要登录（`media_platform/bilibili/core.py:103-122`）。
- **访客身份。** `buvid3`、`buvid4` 由 GET `x/frontend/finger/spi` 发（`data.b_3`、`data.b_4`）；ParseHub 每次换一对新的，旧的 `buvid3` 会招来 -352（`src/parsehub/provider_api/bilibili.py:61-62,149-155`）；dtk 同样取、自己写 `b_nut`，注释说只带这几个 cookie，签名的详情、流、评论都回 `code:0`（2026-10-18 实测，`src/dtk/identity/minting/native.py:1-19,112-145`）。`bili_ticket`、`_uuid` 哪个项目都没算。
- **WBI 签名**用 `nav` 回答里的 `img_key`、`sub_key` 打乱成 32 位盐，`w_rid = md5(排序后的查询串 + 盐)`（MediaCrawler `media_platform/bilibili/help.py:36-76`）。不登录也能算。
- **风控报告。** 不签名的 `x/web-interface/view` 登录与否、带不带 `buvid` 都回 412，签名的 `wbi/view` 不登录回 200（issue：[yutto #866](https://github.com/yutto-dev/yutto/issues/866)，2026-09-28，未关）；把 `wbi/playurl` 换成 `playurl` 修好了 412（issue：[yt-dlp #16962](https://github.com/yt-dlp/yt-dlp/issues/16962)，2026-06）；`-412 request was banned`，建议换出口或加 cookie（哪怕没登录的）（issue：[yt-dlp #17605](https://github.com/yt-dlp/yt-dlp/issues/17605)）；空间接口间歇 -352、请求快了升成 412，按出口和会话、不按用户（issue：[yutto #876](https://github.com/yutto-dev/yutto/issues/876)）；批量下载频道出口被封，维护者让加 sleep（issue：[yt-dlp #14857](https://github.com/yt-dlp/yt-dlp/issues/14857)）。
- **媒体。** DASH 的 `dash.video[]`、`dash.audio[]`，`baseUrl` 加 `backup_url`；下载带 Referer `https://www.bilibili.com/` 和 UA，MediaCrawler 特意不带 cookie，地址带 `deadline`、`upsig`（`core.py:702-714`）。时效没有项目写明，看地址里的 `deadline`（推断）。不登录约 480p（`core.py:118-122`），ParseHub 不登录用 `qn=64`（720p）或 16（`provider_api/bilibili.py:96-115`）；playurl 报得出高画质但 CDN 回 403，MediaCrawler 一档档往下试（`core.py:640-659`）。
- **节奏。** gallery-dl 3–6 秒（`gallery_dl/extractor/bilibili.py:17`）；MediaCrawler 并发 1、睡 2 秒；yutto 重试 4 次，指数退避封顶 4 秒（`fetcher.py:37-40,82-87`）。
- **我们。** `view` 照 yt-dlp 的先后，先不带 cookie 取视频页 `www.bilibili.com/video/<BV 号>/`（分 P 带 `?p=N`），读 `window.__INITIAL_STATE__` 的 `videoData` 和 `window.__playinfo__`（`bilibili/src/video.rs` 的 `video_page`）；页面带 `window._riskdata_` 的 `v_voucher` 或回 412 就回 `blocked`，不再调接口；页面里读不到的那一半才调不签名的 `x/web-interface/view`、`x/player/playurl`（`qn=127`、`fnval=16`，挑最低那一路），都不带 cookie、不签名。`meta.json` 的 `stage` 记两半各从哪来（`video-page` 或 `api`）。412、403、-412、-352、带 `v_voucher` 的回 `blocked`（`bilibili/src/api.rs:86,128,140,422`）。节奏 `interval: 5s`、`daily: 500`（`bilibili/usuyo-plugin.yaml:71`），比 gallery-dl 的 3–6 秒随机稍密、是固定的。只走接口的旧版 11-12 跑通过一条；先取页面的这一版 11-15 跑通一条（[§11](#11-要真跑一次才知道的)）。差的：没有 `buvid3`、`buvid4` 访客身份，等协议能声明可有可无的访客身份（[协议待定](../protocol/open.md#13-待定)第 43 项）。清单见 [bilibili.md §4.1](bilibili.md#41-列出你存过的)：公开收藏夹 `x/v3/fav/resource/list` 不带账号能列（yt-dlp `bilibili.py:1640-1651`），私密的回 -403。

<a id="5-x"></a>
## 5. X

- **参考项目。** 访客令牌：带网页的 bearer POST `api.x.com/1.1/guest/activate.json`（yt-dlp `yt_dlp/extractor/twitter.py:39,104-111`；gallery-dl `gallery_dl/extractor/twitter.py:1856-1869`，存 3600 秒，并写 `gt` cookie）。单条：GraphQL `TweetResultByRestId`，带 `x-guest-token`；gallery-dl 没有 cookie 时自己生成一个 `ct0` 配 `x-csrf-token`（`twitter.py:1333-1360`）。`x-client-transaction-id` gallery-dl 只在登录时带（`twitter.py:1896-1901`），yt-dlp 不带。yt-dlp 碰到 429 改走 `cdn.syndication.twimg.com/tweet-result?id=&token=`，token 由 id 算（`twitter.py:1156-1201`，公式修于 [PR #12537](https://github.com/yt-dlp/yt-dlp/pull/12537)）。
- **要登录的。** 敏感内容（`NsfwLoggedOut`）、受保护账号（`Protected`）（yt-dlp `twitter.py:1087-1092`、gallery-dl `twitter.py:1478-1483`）；维护者说只有年龄限制的要登录（issue：[yt-dlp #12291](https://github.com/yt-dlp/yt-dlp/issues/12291)）。长帖正文 `note_tweet.note_tweet_results.result.text` 访客也有（gallery-dl `twitter.py:504-507`）。11-06 还有人在修访客路径上的错（issue：[gallery-dl #9867](https://github.com/mikf/gallery-dl/issues/9867)），访客路径还活着。
- **节奏。** gallery-dl 读 `x-rate-limit-remaining`，低于 6 先等，429 等到 `x-rate-limit-reset`（没有就 60 秒）（`twitter.py:1911-1914,1958-1960`）；yt-dlp 自己不等。
- **媒体。** 图片改 `?format=jpg&name=orig`（gallery-dl `twitter.py:262-272`）；视频取 `video_info.variants` 里码率最高的 mp4（`twitter.py:236-245`）；`pbs.twimg.com`、`video.twimg.com` 普通 GET，不带 Referer、cookie，地址没有过期参数（推断不过期）。
- **我们。** `guest` 用同一个 bearer、同一个 `TweetResultByRestId`（`x/src/upstream.rs:14-23`），令牌存 3600 秒（`upstream.rs:71`、`x/src/fetch.rs:48`）；`embed` 走 syndication（`upstream.rs:75`）；读 `note_tweet`（`x/src/post.rs:159`）；图片改 `name=orig`（`fetch.rs:876`）；视频挑最高码率。接口不肯答（令牌被拒、出口被拒）回 `blocked`，宿主换 `embed`；限流回 `rate-limited`（`x/src/fetch.rs:170-180`）。节奏 `interval: 10s`、`daily: 300`（`x/usuyo-plugin.yaml:99`）。和参考项目一致。差的一件：没有读 `x-rate-limit-remaining`、`x-rate-limit-reset`（看宿主节奏够不够再定）。设计见 [x.md §3](x.md#3-取单条)。

<a id="6-抖音"></a>
## 6. 抖音

- **分享页不再带作品。** `iesdouyin.com/share/video/<id>` 还回 `_ROUTER_DATA`，但 `videoInfoRes`、`play_addr` 没了，两个出口都复现（issue：[parse-video #98](https://github.com/wujunwei928/parse-video/issues/98)，2026-08-08；之后 #99、#107 同样）。和我们 11-12 的实测一致（[routes.md §4](routes.md#4-对真平台跑过的记录)）。parse-video 仍先试分享页，再退到要运营者自己配 cookie 的网页详情接口（`parser/douyin.go:89-96`、`parser/douyin_detail.go:72-95`，[PR #100](https://github.com/wujunwei928/parse-video/pull/100)）。
- **网页详情接口**`www.douyin.com/aweme/v1/web/aweme/detail/` 除了 `a_bogus` 还要 `uifid` 和 `x-secsdk-web-signature`，不然回 403 `Uifid Not Found` / `Signature Not Found`（dtk `src/dtk/signing/protection.py:1-35`；TikTokDownloader `src/encrypt/douyin_params.py:31-65`，2026-09-08 量的）。dtk 加上这一种签名前 20 次成 8 次，加上后 24 次全成（`src/dtk/signing/native/signer.py:1-20`）。
- **访客身份。** `ttwid`：POST `ttwid.bytedance.com/ttwid/union/register/`，从 Set-Cookie 读（TikTokDownloader `src/encrypt/ttWid.py:17-53`、dtk `src/dtk/signing/native/tokens.py:109-125`）；`msToken`：POST `mssdk.bytedance.com/web/common`，dtk 说编一个或不带也回了完整详情（`src/dtk/signing/native/signer.py:156-171`）；`s_v_web_id`/`verifyFp` 本地生成，但 MediaCrawler 注释说自己生成的会被回 `Signature Not Found`，要用页面的（`media_platform/douyin/client.py:224-246`）；`UIFID_TEMP` 只发给真加载了页面的浏览器，「没有任何算法能产出它们」（dtk `documents/zh/07-playground-and-tools.md:430`），dtk 用真浏览器经同一出口去取（`src/dtk/identity/minting/client.py:1-12`）。MediaCrawler 抖音一律开真浏览器、`pong` 不通就扫码登录（`media_platform/douyin/core.py:104-112`），详情带 `uifid`、`verifyFp`、`fp`，签名经 execjs 算（`media_platform/douyin/client.py:224-246`、`help.py:61-97`）。不登录就行：dtk 详情默认 `login=NONE`（`src/dtk/platforms/base.py:344`），yt-dlp 的报错是「要新鲜的 cookie，不一定要登录」（`yt_dlp/extractor/tiktok.py:1495-1507`）。
- **唯一不带 cookie 的路**是 ParseHub 的安卓 App 接口：注册设备后调 `api.amemv.com` 等主机的 `/aweme/v1/aweme/detail/`，头由第三方签名库算（`src/parsehub/provider_api/douyin.py:894-975,1046-1074`）。签名，`private-api`。
- **媒体。** `video.play_addr.url_list`；parse-video 把 `playwm` 换成 `play` 得原始文件地址（`parser/douyin.go:114-122`）；TikTokDownloader 按分辨率、帧率、码率排 `bit_rate[]` 取最后一路，原画再试 `www.douyin.com/aweme/v1/play/?video_id=<uri>&ratio=default`，用 `Range: bytes=0-0` 比大小（`src/extract/extractor.py:515-620`）。图文 `images[].url_list`，实况图带 `video`。下载带 Referer `https://www.douyin.com/`；地址路径里的十六进制段是过期时间，约 3 小时（issue：[dtk #805](https://github.com/Evil0ctal/Douyin_TikTok_Download_API/issues/805)，2026-10-31）。
- **节奏。** TikTokDownloader 均值 6 秒的对数正态随机，每 10 个账号歇 5 分钟（`src/custom/function.py:12-74`）；dtk 按身份的令牌桶，详情容量 5、每秒补 0.3（`src/dtk/scheduler/policies.py:49-63`）。
- **我们。** `share-page` 读分享页 `_ROUTER_DATA`，取 `play_addr.url_list[0]` 原样请求（`douyin/src/page.rs:178`），现在取不到；`h5-token` 用分享页里的 `xsstoken`、`webId` 做 AES 得 `reflow_id`，调 `web/api/v2/aweme/iteminfo/`（`douyin/src/h5.rs:1-64`），令牌约 50 秒过期，没跑过。403、412、跳去含 `verify`、`captcha` 的地址、页面里有验证字样回 `blocked`（`douyin/src/api.rs:117-125`、`douyin/src/page.rs:102`）；节奏 `interval: 8s`、`daily: 300`（`douyin/usuyo-plugin.yaml:103`）。参考项目里没有一个把 iteminfo 当主路，它的寿命不知道。可行的方向只剩两条，都要先定层级：一是访客窗口里打开 `douyin.com/video/<id>`，让页面自己请求详情、脚本只读回答（不算签名，[§10](#10-改动清单) 里写成待定）；二是访客窗口取 `ttwid`、`s_v_web_id`、`UIFID_TEMP`，插件自己算两种签名（`private-api`，默认关）。

<a id="7-快手微博知乎"></a>
## 7. 快手、微博、知乎

<a id="71-快手"></a>
### 7.1 快手

parse-video 跟着分享短链跳到 `/fw/photo/<id>`，用 iPhone UA 取，读 `window.INIT_STATE` 里同时有 `result`、`photo` 的那一项，`result==1`；视频 `photo.mainMvUrls[0].url`，图集 `https://{atlas.cdn[0]}/{atlas.list[i]}`（`parser/kuaishou.go:18-99`）。维护者说只有 App 分享链接行，因为解析「需要 app 分享链接跳转时携带的 cookie」（issue：[parse-video #76](https://github.com/wujunwei928/parse-video/issues/76)，2025-10）。验证码：`result 2001 antispam need captcha`，归因频率或出口（issue：[parse-video #36](https://github.com/wujunwei928/parse-video/issues/36)）。MediaCrawler 快手走 GraphQL `visionVideoDetail`，一律要登录，列表接口还要页面里生成的 `__NS_hxfalcon` 签名（`media_platform/kuaishou/client.py:59,117-159,191-211`）。ParseHub 自带一份写死的登录 cookie（`src/parsehub/parsers/parser/kuaishou.py:14-29`），不是不带账号的路。我们的 `share-page` 照 parse-video；不带 cookie 的那一版 11-12 跑通四次。`captchaConfig`、`result` 2/2001/400002、验证字样回 `blocked`（`kuaishou/src/page.rs:186-194`），403、412、跳去验证页也是（`kuaishou/src/api.rs:114-120`）；节奏 `interval: 8s`、`daily: 300`。11-15 起跟短链时把各跳回答里 `Set-Cookie` 发的 cookie（`did` 一类）记在这一次调用的内存里，按 `Domain`、主机和路径对得上的带给后面的跳转和手机页，下载媒体不带，直接给作品链接时没有可带的（`kuaishou/src/api.rs` 的 `CookieJar`）；推断这就是 #76 说的那份，带上以后有没有用还没对真平台跑过。落地页的主机和短链主机不在同一个注册域下时，短链主机发的 cookie 照规则带不过去。

<a id="72-微博"></a>
### 7.2 微博

yt-dlp 调 `weibo.com/ajax/statuses/show?id=`，Referer `https://weibo.com/`，被送去 `passport.weibo.com` 时走 `visitor/genvisitor` 和 `incarnate` 拿访客 cookie（`yt_dlp/extractor/weibo.py:24-65,197`）；gallery-dl 用新的 `genvisitor2` 和 `crossdomain`，`SUB`、`SUBP` 存 365 天，长文用 `isGetLongText=true`（`gallery_dl/extractor/weibo.py:25-43,103-105,279-312`）。MediaCrawler 一律要登录（`media_platform/weibo/core.py:105-124`）。媒体：视频挑 `playback_list` 里 `quality_index` 最高的（gallery-dl `weibo.py:173-190`），图片 `pic_infos[id].largest`；下载带 Referer，不带 cookie。我们的 `status` 用访客窗口取的访客身份调 `ajax/statuses/show` 和 `longtext`（`weibo/src/api.rs:1,300-305`），没有就不发请求，和参考项目同一类做法；403、418、跳去验证页、`geetest` 字样回 `blocked`，被送回 `passport.weibo.com` 回访客身份不被收（`weibo/src/api.rs:110-121`）；节奏 `interval: 10s`、`daily: 200`。2026-11-16 跑通一条（[§11](#11-要真跑一次才知道的)）。

<a id="73-知乎"></a>
### 7.3 知乎

页面数据在 `<script id="js-initialData">` 的 `initialState.entities`（MediaCrawler `media_platform/zhihu/help.py:380-423`）。接口要 `d_c0` 和 `x-zse-96` 签名（MediaCrawler `libs/zhihu.js:155-166`、ParseHub `src/parsehub/provider_api/zhihu.py:183-208`）；页面要 `__zse_ck`：WXD 特意在干净的匿名会话里用一个小浏览器让页面自己算出它，存着用，页面回得不全就扔掉，旧的登录 cookie 反而让公开页被送去 `/signin`（`pkg/scraper/zhihu/pcweb.go:203-306`）。视频：yt-dlp 调 `api/v4/zvideos/<id>`（`yt_dlp/extractor/zhihu.py:26-47`），WXD 还要 `__zse_ck`（`pkg/scraper/zhihu/video.go:150-163`）。我们的 `page` 用访客窗口取的访客身份读 `js-initialData`，挑战页回 `blocked`（`zhihu/src/page.rs:13,352-355`），和 WXD 是同一个思路；带着访客身份还碰到挑战页、401，或被送去 `signin`、`/login`（没跟的 3xx，和跟着同主机的跳转落在登录页上），都回访客身份不被收（`visitor-required`，`zhihu/src/fetch.rs` 的 `rejected`、`zhihu/src/api.rs` 的 `SentToLogin`）。节奏 `interval: 30s`、`daily: 100`。2026-11-16 试了三次，还取不到（[§11](#11-要真跑一次才知道的)）：访客身份要的 `__zse_ck` 首页不生成；页面请求当时用的是写死的 Chrome UA，不是取身份时的 UA，已改成用同一个；带着内嵌数据的安全验证页当时被当成「没有这条回答」，已改成回 `blocked`：安全验证页（含 `/account/unhuman`）不换访客身份再试，清单的节奏写了 `blocked: stop`，整个平台停到你点「继续获取」。公开收藏夹接口不带 cookie 回 401（[routes.md §4](routes.md#4-对真平台跑过的记录)），哪个参考项目都没做收藏夹。

<a id="8-清单和内容的分工"></a>
## 8. 清单和内容的分工

- **列出收藏、赞过只能带账号**（视频号连带账号的网页都拿不到，B站公开收藏夹除外）。参考项目里能列的都是用你的 cookie 调签名接口：MediaCrawler（小红书、抖音、快手全要扫码登录）、TikTokDownloader 的 `favorite`、`listcollection`（要你的 cookie 加签名，issue：[#785](https://github.com/JoeanAmier/TikTokDownloader/issues/785) 8 月起 403、维护者说不再维护签名参数），dtk 的 `author_likes` 只有登录的身份拿得到（`src/dtk/platforms/douyin/endpoints.py:307-331`）。这一类都是 `session-api` 或 `private-api`，记过永久封禁（[rednote.md §2.1](rednote.md#21-列出你存过的)）。
- **最低风险的拿法**是 xhs-vault 那种：你自己的浏览器里登录着、你自己往下翻，脚本只截获页面自己发的列表请求，不另发请求（`userscript/xhs-export.user.js:26-50`）。我们的 `userscript` 就是这样；`in-usuyo` 让 usuyo 每 4.5–7.5 秒叫页面往下翻一屏，比你自己翻高一级（[rednote.md §2.1](rednote.md#21-列出你存过的)）。两者都只交链接，内容由不带账号的方法去取。B站（收藏夹、稍后再看）、X（书签、喜欢）、抖音（喜欢、收藏）、微博（收藏、赞）的脚本是同一种：只看页面自己请求回来的列表，不另发请求、不自动滚动、不读 cookie（各插件 `userscript/*.user.js` 开头的说明）。
- **清单里让后续匿名取成为可能的字段。** 小红书：笔记 id、`xsec_token`、`xsec_source`（[§2.3](#23-xsec_token)），三样缺一不可，脚本现在丢了第三样。抖音、快手：作品 id 就够（取不到是因为单条接口本身，不是缺字段）。B站：BV 号。X：帖子 id。视频号：分享短码，清单拿不到，只能手机分享。
- **两步之间隔多久。** 小红书的 token 可能几小时到一天就失效（没有可靠的时长，[§2.3](#23-xsec_token)），而 `daily: 100` 让 381 条要分四天取。两者对不上：要么每天你翻一次、当天只交当天取得完的那些，要么先量出 token 的寿命再定（[BUILD 011](../BUILD.md#011-更多平台) 末尾记着这一缺口）。

<a id="9-2026-11-15-小红书的事故"></a>
## 9. 2026-11-15 小红书的事故

<a id="91-经过"></a>
### 9.1 经过

事实记录（当天给用户取小红书收藏时）：

- **清单。** 在用户自己登录的浏览器里滚动收藏页拿到 381 条，没出问题。
- **内容。** 在同一个登录着的标签页里用 `fetch` 逐条请求笔记页 `/user/profile/<uid>/<noteId>?xsec_token=…&xsec_source=pc_user`。第一轮 3–5 秒一条，取到第 101 篇时被转到 `/website-login/captcha`；用户过了验证后第二轮 12–18 秒一条，取到第 50 篇时被转到 `/login`，账号被登出。
- **不带 cookie 的对照。** 期间用 curl 不带任何 cookie 请求 `/explore/<noteId>?xsec_token=…&xsec_source=pc_user`：第一次回了带 `noteDetailMap` 的完整页面，之后几小时内每次都被 302 到 `/login`（同一个出口）。
- **媒体。** 图片（`sns-webpic-qc.xhscdn.com/<时间戳>/<签名>/…`）和视频（`sns-video-*.xhscdn.com/stream/…mp4?sign=…`，另有 `sns-bak-v1.xhscdn.com` 备用地址）不带账号、带 Referer 就能下，83+50 条零失败。
- **用户的批评**：视频这些本来就能匿名取，参考的那些工具没有被认真学。

哪里错了，对照上面：

1. **内容是用账号取的。** 这直接违反分路的规矩（[BUILD §1](../BUILD.md#1-用户定下的做法)）。在登录的页面里 `fetch` 笔记页，每一次都记在账号上；参考项目里只有 MediaCrawler 这样做，它的用户报告过 461 后网页和手机同时被登出（issue：[MediaCrawler #747](https://github.com/NanmiCoder/MediaCrawler/issues/747)）、永久封禁（[#865](https://github.com/NanmiCoder/MediaCrawler/issues/865)）；在页面里 `fetch` 还会因 token 不一致招来 300013（issue：[XHS-Downloader #362](https://github.com/JoeanAmier/XHS-Downloader/issues/362)）。
2. **3–5 秒一条太密。** 比 XHS-Downloader 的默认（均值 6 秒）还密，又是固定节律；约 100 条处出验证页，和 #305 的约 108 条、OpenCLI 的「Requests too frequent」（issue：[OpenCLI #2562](https://github.com/jackwener/OpenCLI/issues/2562)）是同一个量级。
3. **过了验证马上接着取。** 推断：计数按账号累计，验证之后没有冷却就再取，处罚升到登出；第二轮放慢到 12–18 秒也只撑了 50 条。PR #502 和 xhs-vault 都是碰到就停、等人；MediaCrawler #833 里有人说半天才解开。
4. **匿名对照用的是不带任何 cookie 的 curl**，而且和登录的那一批在同一个出口、同一段时间里跑，被 302 到 `/login` 是因为空 cookie 还是因为出口已被标记分不开（推断，[§2.2](#22-访客身份是什么)）；正确的对照是带访客身份、在登录的那一批停下之后再试。
5. **媒体那一半是对的**：CDN 不要账号，带 Referer 就行，和 XHS-Downloader、MediaCrawler 的做法一致。

<a id="92-按上面的知识该怎么做"></a>
### 9.2 按上面的知识该怎么做

| # | 步骤 | 谁做 | 怎么做 |
|---|---|---|---|
| 1 | 列清单 | 用户 | 在自己的浏览器（或 usuyo 的站点窗口）里登录，打开收藏页，**自己往下翻**；脚本只截获页面自己请求回来的 `collect/page`，记下 id、`xsec_token`、`xsec_source`（`pc_user`），不另发任何请求 |
| 2 | 交清单 | 应用 | 脚本把链接交给 `/v1/batches`，落成一次导入；你在预览里勾选当天要取的（[§8](#8-清单和内容的分工) 的两步间隔） |
| 3 | 访客身份 | 应用 | 内核要一份访客身份，壳开看不见、不登录、自己一份网页数据的窗口打开首页，存下页面自己生成的 cookie（[协议 §2.5.1](../protocol/manifest.md#251-访客身份-visitors)）；不在登录的那份网页数据里取，也不用 curl 空 cookie 去试 |
| 4 | 取内容 | 应用 | `note-page` 在自己的进程里请求笔记页，**第一个请求就带访客身份**，只带 `xsec_token`、`xsec_source`；不碰登录的标签页、不读账号的 cookie |
| 5 | 节奏 | 应用 | 同一平台一次一条，间隔 15 秒上下随机（例如 12–25 秒），每天 100 条；先 3–5 条，隔一天看账号有没有提示，再放大（[README §4](README.md#4-怎么相对安全地试)） |
| 6 | 媒体 | 应用 | 在同一次调用里下图片、视频（和字幕），带 Referer 首页、不带 cookie；主地址失败试备用地址 |
| 7 | 碰到 `/login` | 应用 | 带着访客身份还被送去 `/login`：插件在同一次调用里再用手机版页面、不带 cookie 请求一次（ParseHub 的第三段）；也不行就回 `visitor-required`，这份访客身份不管用，换一份新的再试一次；还不行就停下整个平台 |
| 8 | 碰到验证页 | 应用停，用户决定 | `/website-login/captcha`、461、471、300011–300013：马上停下整个平台的队列，不自动在一小时后接着取；界面说清楚是匿名请求被拦，你的账号没参与 |
| 9 | 过验证、恢复 | 只有用户 | 验证只在你自己的浏览器里过；接着取等至少半天，你点了才接着；账号收到提示或被登出时，整批停到你说接着 |

<a id="10-改动清单"></a>
## 10. 改动清单

按平台，每个平台里按先后。只写要动什么、照哪个项目哪段；实现时照 [BUILD §2](../BUILD.md#2-干活的规矩) 的规矩做、测试全离线。

**小红书（`rednote`）**

1. 有访客身份时第一个请求就带上，不先发空 cookie 的请求；没有访客身份时才不带（`src/fetch.rs:82-103`）。照 ParseHub `parsers/parser/xhs.py:36-58` 的顺序和 XHS-Downloader 有 cookie 就带（`source/module/manager.py:346-358`）。
2. 两个脚本记下并交出 `xsec_source`：照截获的回答或页面链接给的值，没有就写 `pc_user`（`userscript/src/window.js:56`、`userscript/src/manager.js:271`，改完重拼两个脚本）。照 xhs-vault `userscript/xhs-export.user.js:147`。
3. 补风控码：HTTP 471、JSON 300011 回 `blocked`（`src/fetch.rs:142`、`src/fetch.rs:253`）。照 MediaCrawler `media_platform/xhs/client.py:153-184` 和 PR #502 的列表。
4. 视频分档不写死名字：`video.media.stream` 下凡是列表都当候选，按高度、码率挑，h264 优先的规则留着（`src/state.rs:346`）。照 MediaCrawler `media_platform/xhs/media.py:61-75`。
5. 媒体主机加 `*.rednotecdn.com`（`src/site.rs:37`、清单 `permissions.network.hosts`）。照 ParseHub PR #198 记下的新地址。推断：这个主机要不要 Referer 没人写过。
6. 取平台字幕：`video.media.video.subtitles`，按 `source`、`zh-CN`、`en-US` 挑一个，同一次调用里下载，存成字幕文件，`outputs` 加 `subtitle`。照 xhs-vault `scraper/build_corpus.py:39-59`、`scraper/fetch.py:96-117`。
7. 已做（11-15）：访客身份声明 `cookies: [a1, webId, web_session]`，三个都出现了才算取到（清单 `visitors`）。`a1`、`webId` 照 ParseHub `provider_api/xhs.py:56`；`web_session` 是 ParseHub 游客那一段带去笔记页的唯一一个 cookie（`provider_api/xhs.py:53-78`），MediaCrawler 在没登录的浏览器里也读到它（`media_platform/xhs/login.py:191-193`）。别的名字没有出处，不写。

**宿主和协议（为小红书提，不写平台名）**

8. 已做（11-15）：`pacing` 加了 `intervalMax`，宿主每次在 `interval` 到它之间随机取，见[协议 §3.1](../protocol/fetch.md#31-运行器与节奏)；小红书写 15–25 秒。
9. 已做（11-15）：`pacing` 的 `blocked: stop` 让整个平台停到你点「继续获取」，访客身份换过一次还被拒也这样停，见[协议 §3.1](../protocol/fetch.md#31a-平台停下)；小红书声明了它，别的平台照旧一小时。
10. 导入取什么、链接多久失效：已记在 [BUILD 011](../BUILD.md#011-更多平台)「清单和协议还缺的」，不在这里重写。

**B站（`bilibili`）**

11. 已做（11-15）：`view` 先取视频页 HTML，读 `window.__INITIAL_STATE__` 和 `window.__playinfo__`，带 `v_voucher` 的回 `blocked`；页面里没有的那一半再调接口。照 yt-dlp `yt_dlp/extractor/bilibili.py:723-753,829-834`。不签名。
12. 没做，等[协议待定](../protocol/open.md#13-待定)第 43 项：现在一种方法列了访客身份，没有那一份时宿主就不调它，`view` 会在没有桌面壳的环境里用不了。要做成的样子：加 `buvid3`、`buvid4` 的访客身份，清单 `visitors` 声明 `www.bilibili.com`，由访客窗口取，只发给 API 主机；接口被拒而这次没带时回 `visitor-required`，带了还被拒回 `blocked`。照 ParseHub `provider_api/bilibili.py:149-155`、dtk `src/dtk/identity/minting/native.py:112-145`（它们是调 `finger/spi`；我们走窗口，不调）。
13. 接口那一路改用 `wbi/view`、`wbi/playurl` 要 WBI 签名，算不算 `private-api` 要先定；定了再做，单写一种方法。照 MediaCrawler `media_platform/bilibili/help.py:36-76`、yutto #866。

**X（`x`）**

14. 读 `x-rate-limit-remaining`、`x-rate-limit-reset`：余量少时回 `rate-limited` 带 `retryAfter`。照 gallery-dl `gallery_dl/extractor/twitter.py:1911-1914,1958-1960`。其余和参考项目一致，不改。

**抖音（`douyin`）**

15. 先定层级：访客窗口里打开作品页、只读页面自己请求回来的详情（不算签名），算不算 `browser-session` 之外的新一种、要不要新协议字段，在 [协议待定](../protocol/open.md#13-待定) 记一项。参考的是 dtk 用真浏览器取 `UIFID_TEMP` 的做法（`src/dtk/identity/minting/client.py:1-12`）。
16. 下载时记下地址里的过期时间，过期了重取单条而不是重下（照 dtk #805 的说法）。

**快手（`kuaishou`）**

17. 已做（11-15）：跟分享短链时把站点在跳转里发的 cookie 带到后面的跳转和落地页，只发给主机和路径对得上的快手页面主机（`src/api.rs`）；为读全一次回答里的几条 `Set-Cookie`，plugin-kit 加了 `Response::headers`。照 parse-video #76 维护者的说法；推断，要真跑核实。（`result 2001`、`captchaConfig` 已经回 `blocked`，不用改。）

**知乎（`zhihu`）**

18. 已做（11-15）：被送去 `signin`、`/login`（含 401、跟着跳转落在登录页上）时回 `visitor-required`（带着访客身份时说这份不被收），不回 `login-required`。照 [协议 §2.5.1](../protocol/manifest.md#251-访客身份-visitors) 和 WXD `pkg/scraper/zhihu/pcweb.go:203-245`（旧的登录 cookie 让公开页被送去 `/signin`）。

**微信视频号、微博**：和参考项目同一类做法，不改；先各真跑一次（[§11](#11-要真跑一次才知道的)）。

<a id="11-要真跑一次才知道的"></a>
## 11. 要真跑一次才知道的

都是推断，按 [BUILD §2](../BUILD.md#2-干活的规矩) 只走不带账号、不带签名的路，一个平台一次一个会话、几次请求为限：

- 小红书：访客窗口在这台机器上取到的是哪几个 cookie；带着它，今天被 302 到 `/login` 的那个出口上笔记页回不回完整页面；同一份访客身份连着取几十条会不会被换掉；`xsec_token` 从收藏页拿到后多久失效；`rednotecdn.com` 的图要不要 Referer。
  - 2026-11-15 跑了两次（策略「仅不使用账号」）：一次用 11 小时前从收藏页拿的链接，一次用刚在站点窗口里读到的链接。两次都是访客窗口约 8 秒交回访客身份，带着它请求笔记页被 302 到 `/login`（失败日志 `visitor-required: sent to /login (visitor cookie used: true)`），各只发了这一次。新链接也一样，所以不是令牌过期。之后只做了离线排查，结论分三类：
    - **确定的差异。** ① UA：访客窗口是 WKWebView，用的是它自己的默认 UA（`…AppleWebKit/605.1.15 (KHTML, like Gecko)`，没有 `Version/… Safari/…` 那一截），插件请求笔记页用的是写死的桌面 Chrome 142 UA 配 Chrome 的 Accept，取身份和用身份的不是同一个「浏览器」。宿主当时也没有把取身份时的 UA 交给插件。② 收下的时机：清单没声明访客 cookie 的名字，壳等「名字连着 3 次没变」就收；8 秒里收下的是哪几个名字没有记录，没有任何东西保证 `web_session` 在里面。③ 插件的 TLS 和 HTTP 指纹（rustls、HTTP/1.1）和 WebKit 的不一样，这一条没改。
    - **推断。** ParseHub 游客那一段只带一个 `web_session` 就取得到笔记页（`provider_api/xhs.py:53-78`），所以它是要紧的那个；页面要等 `login/activate` 回来才有它，首屏出来、名字停了 3 秒时它可能还没生成。另外两种解释同样没排除：站点把访客 cookie 和取它的 UA 对上，或者这个出口被标记了（[§2.2](#22-访客身份是什么)）。
    - **已改（11-15）。** 访客身份要 `a1`、`webId`、`web_session` 都出现才收（[§10](#10-改动清单)）；窗口在 macOS 上用和系统 Safari 一样的完整 UA，取身份时记下 UA，插件用同一个 UA 和配它的 Accept（[协议 §6](../protocol/credentials.md#6-凭据)）；访客身份被拒时插件再用手机版页面、不带 cookie 取一次（[rednote §2.4](rednote.md#24-插件做成的样子)）；`GET /visitors` 和插件页「诊断信息」写存着的 cookie 名字和 UA，失败日志记这次带了哪些 cookie 名（[协议 §2.3.1](../protocol/manifest.md#231-方法-methods) 的「试跑和失败」）。
  - **下次真跑看这几样。** 先在插件页小红书下面「更换」访客身份：以前存下的那份没有 UA，名字也是没等 `web_session` 就收的。请求数照这个算：一条笔记一次调用，访客那一段被拒时同一次调用里接着发手机版那一个，宿主换一份访客身份再试一回，最多四个请求；手机版取到了，宿主当访客身份被收了，之后每条都是两个。插件页小红书的访客身份「诊断信息」：存着的名字里有没有 `web_session`，UA 是不是带 `Version/… Safari/…` 的那个；60 秒没取到时「未能获取」那一行缺了哪个名字。`fetch-failures.jsonl` 那一行的 `cookies` 和 `message`：开头的 `visitor-page:`、`mobile-page:` 各被送去了哪。手机版那一段取到了而访客那一段被拒，问题在访客身份或 UA；两段都被送去 `/login`，更像是出口被标记，先停下，隔一阵再试一条。
  - 2026-11-15 夜里改完以后又在独立实例里跑了一回（策略「仅不使用账号」，链接是当天从站点窗口读的收藏）：访客窗口交回 `a1`、`webId`、`web_session` 等 10 个 cookie，UA 是完整的 Safari 写法；网页版笔记页那一段仍被转去 `/login`，同一次调用里接着取手机版页面（不带 cookie），取到正文、图片和视频。之后按 15–25 秒一条导入 81 条，全部取到，没有出现验证页；另有 2 条用新链接「重新获取」后取到。所以这个出口上现在能用的是手机版页面那一段，网页版那一段被拒的原因仍不清楚。
- B站：不签名的 `x/web-interface/view` 现在是不是一律 412；访客窗口取不取得到 `buvid3`、`buvid4`。
  - 2026-11-15 夜里在独立实例里取了一条公开视频（策略「仅不使用账号」，经系统代理）：视频页不带 cookie 回了 `__INITIAL_STATE__`，视频信息从页面读到（`stage.view: video-page`），没有调 `x/web-interface/view`；页面里没有读到带 DASH 的 `__playinfo__`，流地址由不签名的 `x/player/playurl` 给（`stage.streams: api`），360p 的画面和音轨各一个文件、封面都存下。没有出现 412 和验证。
- 抖音：访客窗口里打开作品页时，页面自己的详情请求在不在、回答能不能读到。
- 快手：带上跳转 cookie 后网页版 `/f/` 链接能不能取到。
- 微博、知乎：带访客窗口的访客身份各取一条。
  - 微博，2026-11-16（独立实例，策略「仅不使用账号」）：内核要访客身份，看不见的访客窗口开首页交回 `SUB`、`SUBP`、`WBPSESS`、`XSRF-TOKEN`，`status` 带着它取到一条带视频的公开微博，文字、作者、视频都存下，从收下到存好约 17 秒，没有出现验证。
  - 知乎，2026-11-16 同一个实例，三次，之后停下：
    1. 访客窗口开首页，交回 13 个 cookie（有 `d_c0`、`captcha_session_v2`，没有 `__zse_ck`）；带着它请求回答页，回的是 `zh-zse-ck` 挑战页，插件回 `visitor-required`。
    2. 试着让访客身份必须有 `__zse_ck`、窗口改开 `/explore`：60 秒里还是那 13 个，没有 `__zse_ck`，没取到身份，没有请求回答页。
    3. 窗口改开要取的那条回答的页面：交回的 cookie 里有 `__zse_ck`；带着它请求回答页，回的是标题「安全验证 - 知乎」的页面。当时页面请求用的是插件写死的 Chrome UA，访客窗口是 Safari 的 UA；插件还把这一页当成了「没有这条回答」。
    - 之后改的：页面请求用取身份时的 UA；跟着跳转落到验证页、和带着内嵌数据的安全验证页都回 `blocked`、停下整个平台；清单声明访客身份要 `__zse_ck`。改完没有再对真平台跑。还缺的是协议：访客窗口要能打开正在取的那条链接（[协议待定](../protocol/open.md#13-待定)第 44 项）；UA 对上以后还回不回安全验证页，要隔一阵再试一条才知道。
- 账号这一边的事（隔一天看提示、导入一批几十条）只有用户能试，见 [README §4](README.md#4-怎么相对安全地试)。
