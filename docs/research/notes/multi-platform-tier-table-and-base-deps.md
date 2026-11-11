# 多平台工具逐平台层级表，和歇存会直接依赖的基础库（2026-10-30）

写给发起 session 和同组调研 session。这里只有事实、判定依据和建议，不是指令。28 条基础依赖画像和 lead 已通过 StructuredOutput 交回工作流，JSON 副本：
/private/tmp/claude-501/-Users-mac-WorkSpace-desktop-Project/7133807a-d653-486b-80df-d8e2af41491d/scratchpad/deps/structured_output.json
矩阵计算脚本与结果：
/private/tmp/claude-501/-Users-mac-WorkSpace-desktop-Project/7133807a-d653-486b-80df-d8e2af41491d/scratchpad/deps/matrix.py
/private/tmp/claude-501/-Users-mac-WorkSpace-desktop-Project/7133807a-d653-486b-80df-d8e2af41491d/scratchpad/deps/matrix_corrected.json
/private/tmp/claude-501/-Users-mac-WorkSpace-desktop-Project/7133807a-d653-486b-80df-d8e2af41491d/scratchpad/deps/matrix_naive.json

<a id="口径"></a>
## 口径

本表沿用同组已有口径，下面几条是本批要特别说明的：

- **页内自拼请求**：工具在页面上下文里自己构造请求、页面不参与签名（OpenCLI 知乎收藏、YouTube 稍后观看的续页），按现行字面定义记 session-api。页面脚本会给请求附加签名的（OpenCLI 抖音 a_bogus），沿用 agent-access 笔记记 browser-session。
- **不登录的自动化浏览器**：记 public-web（Kazumi 先例）。
- **只有 TLS 指纹模拟**：不升层（小红书方向口径）；微信方向把 curl_cffi 记 private-api，仍待统一。
- **微博访客握手**：yt-dlp（`genvisitor` 提交固定的伪造 fp 字段）、gallery-dl（`genvisitor2` 带 `webdriver=false`）、lux（内置固定访客 SUB cookie）、RSSHub（无头浏览器真实加载）四种做法，本批**统一记 public-web**。若按「工具在请求参数里伪造指纹值即 private-api」的严格口径，yt-dlp 与 gallery-dl 的微博路径、以及跟随 yt-dlp 的 yichen-skills 和 chubbyskills 的微博路径都要改成 private-api；微博 public-web 这一格不会因此变空（另有 dataabc/weibo-crawler、lux、RSSHub 等）。
- **改设备指纹的浏览器**（CloakBrowser、Camoufox）：按数据路径定层，并标注「严格口径 private-api」；patchright 只隐藏自动化标记，按 browser-use 先例不升层。
- **第三方托管服务**（r.jina.ai、FxTwitter）：只用于通用网页或公开帖子，本批记 public-web 并标出来；同组「第三方付费数据 API 记 private-api」的口径是给社媒数据商定的，这里没套用。
- 每个（工具，平台）先给取数机制标签，再给层级；一个平台有多条路由时分行列出。

<a id="part1"></a>
## 第一部分：工具 × 平台 → 层级

<a id="yt-dlp"></a>
### yt-dlp（master b77c930，2026-10-29）

B站、YouTube、抖音沿用 tier-ytdlp 笔记，不重做。

| 平台 | 取数机制 | 层级 | 登录 | 依据 |
|---|---|---|---|---|
| bilibili | 本地算签名（wbi） | private-api | 不需要 | 已核（tier-ytdlp 笔记） |
| youtube | 模拟 App 客户端 + 本地解 JS 挑战 | private-api | 不需要 | 已核 |
| douyin | 带 cookie 直连不签名（`aweme/v1/web/aweme/detail`，需要新鲜的 `s_v_web_id`，访客 cookie 也行） | session-api | 可选 | `yt_dlp/extractor/tiktok.py` 1495–1506 |
| tiktok | 匿名读公开页（`__UNIVERSAL_DATA_FOR_REHYDRATION__`）+ TLS 指纹模拟；App 接口只在用户传 `app_info`/`device_id` 时启用 | public-web（App 路径 private-api，非默认） | 不需要 | tiktok.py 60–63、275–287、993–1006 |
| xiaohongshu | 匿名读公开页（`__INITIAL_STATE__`） | public-web | 不需要 | xiaohongshu.py 14、43–49 |
| weibo | 匿名访客握手（提交固定 fp）+ 匿名调不签名接口（`ajax/statuses/show`） | public-web（严格口径 private-api） | 不需要 | weibo.py 24–54、56–65、196 |
| zhihu | 匿名调不签名接口（`api/v4/zvideos`，只支持知乎视频） | public-web | 不需要 | zhihu.py 6、28–29 |
| instagram | 不登录：匿名调页面 GraphQL（`doc_id` + 首页取 LSD token），只在装了 curl_cffi 时发出（TLS 指纹模拟）；带 cookie：调 `www.instagram.com/api/v1` 网页接口（web App-ID） | public-web / session-api；`app_id=ios` 参数才走 `i.instagram.com`（private-api，非默认） | 可选 | instagram.py 46–49、77–91、421–431、437–445、471–488 |
| pinterest | 匿名调不签名接口（`resource/*Resource/get`） | public-web | 不需要 | pinterest.py 23–28 |
| reddit | 匿名调不签名接口（`<slug>/.json`，先访问 old.reddit 取 loid）；带 `reddit_session` cookie | public-web / session-api | 可选 | reddit.py 266–283、294–308 |
| x | 匿名：网页 bearer + 访客 token 调 GraphQL；带 cookie：auth_token + ct0；**不生成** x-client-transaction-id；429 时退回 syndication（按嵌入脚本公式本地算 token，UA 写 Googlebot） | public-web / session-api | 可选 | twitter.py 36–40、95–133、1156–1170、1180–1204 |
| podcast、generic-web | 匿名读公开页或订阅源 | public-web | 不需要 | 通用抽取器 |

<a id="rsshub"></a>
### RSSHub（c175b17，2026-10-30）

浏览器层是 patchright 1.64.2（`lib/utils/playwright.ts` 1–2、62；`package.json` 120），只隐藏自动化标记。

| 平台 | 路由 | 取数机制 | 层级 | 登录 | 依据 |
|---|---|---|---|---|---|
| weibo | `/weibo/user_bookmarks/:uid` 本人收藏 | 带 cookie 直连不签名（m.weibo.cn container） | session-api | 必需 | `lib/routes/weibo/user-bookmarks.ts` 16–54、74–126 |
| weibo | `/weibo/user/:uid` | 无头浏览器取访客 cookie，再匿名调 m.weibo.cn 接口；配 cookie 则带 cookie | public-web | 可选 | user.ts 76–110；utils.ts 121–150 |
| weibo | `/weibo/timeline/:uid` | 官方接口（OAuth，`api.weibo.com/2/statuses/home_timeline`） | official-export | 必需 | timeline.ts 27–33、112、185–200 |
| zhihu | `/zhihu/collection/:id`、`/zhihu/people/allCollections/:id` | 带 cookie 直连不签名（只带 `x-api-version`） | session-api（公开收藏夹不配 cookie 也能取，即 public-web） | 可选 | collection.ts 45–51、68–75；all-collections.ts 61–81；utils.ts 15–17 |
| zhihu | `/zhihu/people/activities`、`answers`、`posts` | 本地算 x-zse-96；没配 cookie 时在 JSDOM 里跑知乎挑战脚本取 `__zse_ck`（Workers 下改用浏览器） | private-api | 可选 | sign.ts 5–10；utils.ts 132–195、264–290 |
| douban | `/douban/people/:userid/status` | 匿名调不签名接口（rexxar），cookie 可选 | public-web / session-api | 可选 | people/status.ts 24、535–566 |
| douban | `/douban/people/:userid/wish` | 匿名读公开页 | public-web | 可选 | people/wish.ts 29–38 |
| douban | 推荐片单、即将上映（非用户内容） | 带 frodo apiKey 与 `_sig`，伪装安卓 UA | private-api | 不需要 | other/recommended.ts 37–38；tv/coming.ts 12–15、168–170 |
| xiaohongshu | `/xiaohongshu/user/:id/collect` | 不登录的无头浏览器打开主页，点「收藏」并截获页面自身的 `note/collect/page` 请求 | public-web | 不需要 | xiaohongshu/util.ts 54–125（截获 98–112） |
| xiaohongshu | `/xiaohongshu/user/:id/notes` 配 cookie | 带 cookie 取主页 HTML、解析 `__INITIAL_STATE__` | session-api | 可选 | user.ts 65–81；util.ts 282–321 |
| xiaohongshu | `/xiaohongshu/board/:id` | 无头浏览器匿名读 `__INITIAL_SSR_STATE__` | public-web | 不需要 | util.ts 133–160 |
| douyin | `/douyin/likes/:uid`、`/douyin/collection` | 自动化浏览器注入 DOUYIN_COOKIE、点标签页、截获页面自身请求 | browser-session | 本人列表必需 | saved-posts.ts 27–58；likes.ts 13–18；collection.ts 6–16 |
| douyin | `/douyin/user/:uid` | 无头浏览器匿名截获页面请求 | public-web | 不需要 | user.ts 47–56 |
| bilibili | `/bilibili/fav/:uid/:fid`、`/bilibili/user/fav/:uid` | 不签名接口（`x/v3/fav/resource/list`、`x/v2/fav/video`），cookie 可选 | public-web / session-api | 可选 | fav.ts 31–35；user-fav.ts 37–42 |
| bilibili | `/bilibili/watchlater/:uid` | 带 cookie 直连不签名（`x/v2/history/toview`） | session-api | 必需 | watchlater.ts 44–56 |
| bilibili | `/bilibili/user/video/:uid` | 本地算 wbi（`space/wbi/arc/search`） | private-api | 可选 | video.ts 65、148–156；utils.ts 74–80 |
| bilibili | `/bilibili/user/dynamic/:uid` | 请求带工具伪造的 `dm_img_*` 指纹参数；匿名 cookie 由无头浏览器取 | private-api | 可选 | dynamic.ts 276–283；utils.ts 186–196；cache.ts 44–70 |

本批没核：jike、tieba、instagram、telegram、pinterest 的路由；X 已由 social 笔记核过（生成 transaction id，private-api）。

<a id="lux"></a>
### lux（dd00f6d，2025-12-29）

| 平台 | 取数机制 | 层级 | 依据 |
|---|---|---|---|
| bilibili | 匿名调不签名 `x/player/playurl`；番剧 `pgc/player/web/playurl` 也不签名（第 79 行只是注释）；有 cookie 时先取 utoken | public-web / session-api | `extractors/bilibili/bilibili.go` 27–29、36–84 |
| douyin | 本地算签名（内嵌 sign.js 在 goja 里算 X-Bogus），本地生成 msToken、注册 ttwid | private-api | douyin.go 27、66–93、158–206 |
| youtube | 模拟 App 客户端（kkdai/youtube v2.10.5，默认 AndroidClient） | private-api | go.mod 16；kkdai/youtube client.go 33、192 |
| xiaohongshu | 匿名读公开页 | public-web | xiaohongshu.go 32 |
| weibo | 内置固定访客 SUB cookie，getversion 取 XSRF，匿名调 `tv/api/component` | public-web | weibo.go 45–62、143–159 |
| zhihu | 匿名调不签名接口（lens.zhihu.com） | public-web | zhihu.go 17 |
| instagram | 匿名读嵌入页 | public-web | instagram.go 69 |
| x | 匿名调 1.1 `videos/tweet/config`（固定 bearer） | public-web | twitter.go 55–60 |
| reddit、pinterest、tiktok、kuaishou | 匿名读公开页 | public-web | reddit.go 22–53；pinterest.go 34；tiktok.go 26；kuaishou.go 26–56 |
| toutiao-xigua | 读页面，cookie 可选 | public-web / session-api | ixigua.go 45–74 |

<a id="you-get"></a>
### you-get（049548f，2025-04-27）

| 平台 | 取数机制 | 层级 | 依据 |
|---|---|---|---|
| bilibili | 默认流程里，除不签名 playurl 外还调用内置 appkey/secret 做 md5 签名的接口 | private-api | `src/you_get/extractors/bilibili.py` 74–75、98–103、268–290 |
| youtube | dukpy 在本地执行播放器 JS 算签名和 n 参数 | private-api | youtube.py 78–110、266–272 |
| douyin | 匿名调旧 iteminfo 接口 | public-web | douyin.py 46–56 |
| weibo | 匿名调不签名接口（miaopai.py 处理微博视频） | public-web | miaopai.py 22–93 |
| zhihu | 匿名调不签名接口 | public-web | zhihu.py 27、59 |
| instagram | 带 cookie 直连 `i.instagram.com/api/v1/media/<id>/info`，x-ig-app-id 取自网页；没有 cookie 就失败 | session-api | instagram.py 20–31 |
| x | 匿名 syndication（token 写死为 `!`） | public-web | twitter.py 48 |
| tiktok | 匿名读公开页 | public-web | tiktok.py 22–30 |
| toutiao-xigua | 西瓜：内置固定 cookie 读页面；头条：本地 CRC32 签名 | public-web / private-api | ixigua.py 17–29；toutiao.py 23–35 |

<a id="cobalt"></a>
### cobalt（a636575，2026-04-06，`api/src/processing/services/`）

| 平台 | 取数机制 | 层级 | 依据 |
|---|---|---|---|
| youtube | 模拟 App 客户端（youtubei.js，默认 IOS；配会话令牌时 WEB_EMBEDDED） | private-api | youtube.js 60、194、226、264 |
| instagram | 默认先用安卓 Instagram App 的 UA 和请求头匿名调 `i.instagram.com` 移动端接口，失败再读嵌入页、GraphQL | private-api | instagram.js 14–25、109–136、422–445 |
| bilibili | 匿名读视频页 `__playinfo__`（UA 写 facebookexternalhit）；bilibili.tv 走不签名 playurl | public-web | bilibili.js 19–38、68–75 |
| tiktok、pinterest | 匿名读公开页 | public-web | tiktok.js 36–42；pinterest.js 19 |
| x | 网页 bearer + 访客 token 调 GraphQL；实例 cookie（ct0）可选；退回 syndication | public-web / session-api | twitter.js 6–15、74–107 |
| reddit | 匿名 `.json`；实例配 OAuth 凭据时走 oauth.reddit.com | public-web / official-export | reddit.js 16–48、76–78 |

<a id="gallery-dl"></a>
### gallery-dl（175d21f，2026-10-25）

| 平台 | 取数机制 | 层级 | 依据 |
|---|---|---|---|
| x | 本地生成 x-client-transaction-id | private-api | 已核（social 笔记） |
| instagram | 带 cookie 直连 `www.instagram.com/api/v1`（web App-ID）；不支持账号密码登录 | session-api | `gallery_dl/extractor/instagram.py` 36–51、181–196、1113–1125 |
| pinterest | 匿名调 resource 接口，cookie 可选 | public-web / session-api | pinterest.py 420–428、526 |
| weibo | 匿名访客握手（genvisitor2）+ 不签名接口 | public-web（严格口径同 yt-dlp） | weibo.py 60–75、280–310 |
| bilibili | 不签名 web-dynamic 接口；收藏的专栏要 cookie | public-web / session-api | bilibili.py 120–177 |
| reddit | 官方 OAuth 接口（内置 client-id，可换成用户自己的） | official-export | reddit.py 407–446 |

<a id="opencli"></a>
### OpenCLI v2.0.2（97dc0cd，2026-10-26）

B站、小红书、抖音、X 沿用 agent-access 笔记。

| 平台 | 命令 | 取数机制 | 层级 | 依据 |
|---|---|---|---|---|
| weibo | favorites | 读页面 DOM（打开收藏页滚动读取；uid 先读页面 Vue store，失败才页内请求 get_config） | browser-session | `clis/weibo/favorites.js` 103–126；weibo/utils.js 35–55 |
| zhihu | collections、collection、favorite | 页内自拼请求，不签名（`fetch` 带 `credentials: 'include'` 调 api/v4） | session-api（页内口径 browser-session） | `clis/zhihu/collections.js` 14–16、56–74；collection.js 23–26；favorite.js 40–70 |
| douban | marks | 读页面 DOM（逐页打开 movie.douban.com/people/<uid>/<状态>） | browser-session | `clis/douban/marks.js` 9、43–60 |
| weread | notebooks、highlights | 带 cookie 直连不签名：从浏览器取 cookie，在 Node 里请求 i.weread.qq.com 私有接口 | session-api | `clis/weread/utils.js` 5–6、124–156 |
| weread | weread-official | 官方接口（个人 API Key） | official-export | `clis/weread-official/` |
| youtube | watch-later | 先读页面 `ytInitialData`，续页在页内自拼 InnerTube browse 请求（不算 SAPISIDHASH） | session-api（只取首页时 browser-session） | `clis/youtube/watch-later.js` 15–53；utils.js 94–105 |
| podcast | xiaoyuzhou history | **模拟 App 客户端**：用 iOS 版小宇宙的 UA、BundleID、App-Version、`x-jike-device-id` 和本地凭据文件里的 access_token 调 api.xiaoyuzhoufm.com | private-api | `clis/xiaoyuzhou/auth.js` 6–11、94–115；history.js 154–187、240–242 |

本批没核：wechat-mp、jike、reddit、instagram、tiktok、toutiao-xigua、tieba、pinterest、generic-web 适配器。

<a id="wx-channels"></a>
### wx_channels_download（124f044，2026-09-15）

**不是同一套代理注入。**只有视频号（和公众号历史消息列表）依赖中间人代理与客户端页面；其他平台是各自独立的 HTTP 客户端，cookie 来自它内置的 CookieCloud 兼容上传接口（`internal/api/handler_cookie.go` 20–40：解密后写 `cookies.json`，`pkg/cookies/persistent.go` 按域名读取）。

| 平台 | 取数机制 | 层级 | 依据 |
|---|---|---|---|
| wechat-channels | 中间人代理注入 | client-hook | 已核 |
| wechat-mp | 单篇读公开页；历史消息列表经客户端里的页面运行时（按 adapter.go 注释推断，运行时代码未读） | public-web / client-hook | `internal/adapter/wxmp/adapter.go` 50–75 |
| douyin | 单条：读分享页 SSR + iteminfo（不签名，cookie 可选），失败退回移动端 router 数据；主页列表：本地算 a_bogus | public-web / private-api | `pkg/scraper/douyin/client.go` 71–117；pc.go 29–45；web.go 69–78 |
| xiaohongshu | 单篇：带 cookies.json 取笔记页；主页列表：本地生成与 xhshow 兼容的 XYW 签名 | session-api / private-api | `pkg/scraper/xiaohongshu/client.go` 110–207；home_xyw.go 3–4；home_api.go 20、142–146 |
| bilibili | 本地算 wbi | private-api | `pkg/scraper/bilibili/homepage.go` 129、218–246；bv.go 25 |
| youtube | 模拟 App 客户端（VISIONOS、TVHTML5、ANDROID_VR）+ goja 里解 n 挑战 | private-api | `pkg/scraper/youtube/`（challenge_goja.go 等；只核到目录，没有精确到行） |
| zhihu | Go 实现 x-zse-96；在 goja/minib 里跑知乎页面运行时；z_c0 来自 cookies.json | private-api | `pkg/scraper/zhihu/sign.go` 29–55；`internal/mcpserver/zhihu_tools.go` 40–77 |
| weibo | 带 cookie 直连不签名（`ajax/statuses/mymblog`） | session-api | `pkg/scraper/weibo/client.go` 51、86–90 |
| x | 主页列表：从 main.js 取 queryId 与 bearer，带 cookies.json 的 ct0 调 GraphQL，不生成 transaction id；单条：自制无渲染浏览器 minib 打开公开帖 | session-api / public-web | `pkg/scraper/x/homepage.go` 40–91；client.go 75–84 |
| instagram | 读公开页，cookie 可选 | public-web / session-api | `pkg/scraper/instagram/client.go` 34–35、66 |
| kuaishou | 带 cookie 调 `rest/v/profile/feed` | session-api | `pkg/scraper/kuaishou/client.go` 14、42 |

另外 go.mod 有 bogdanfinn/tls-client v1.7.8、bogdanfinn/utls v1.6.1（94–95 行，replace）、refraction-networking/utls v1.5.4（170 行，indirect）、JohannesKaufmann/html-to-markdown v1.4.1（70 行）。

<a id="catbus"></a>
### catbus（bcc72f8，2026-10-30）

小红书、抖音、TikTok、B站、快手、X 都是本地算签名（上游签名 JS 在沙箱运行或移植），HTTP 层用 wreq-js 模拟 Chrome 的 TLS 与 HTTP/2 指纹（`package.json` 66），记 private-api。依据：douyin `web/sign.ts` 47–49（a_bogus）、bilibili `web/api.ts` 56–186（wbi、wbiDm）、x `web/api.ts` 326、453–463（transaction id）、xhs `web/api.ts` 212–237（x-s-common）。**微博例外**：`weibo/web/sign.ts` 6–19 只生成上传参数，读取路径是带 cookie 直连（session-api）。

<a id="agent-reach"></a>
### Agent-Reach 1.6.0（14c2134，2026-10-25）

| 平台 | 默认后端 | 层级 | 依据 |
|---|---|---|---|
| bilibili | bili-cli（依赖 bilibili-api-python，本地算签名） | private-api | `agent_reach/channels/bilibili.py` 2–9、57–59 |
| youtube | yt-dlp | private-api | channels/youtube.py 30–52 |
| xiaohongshu | OpenCLI，在 CloakBrowser 专用浏览器里读页面、截获页面请求 | browser-session（严格口径 private-api） | channels/xiaohongshu.py 2–46；backends/cloak.py 1–25 |
| x | OpenCLI twitter（页内自拼 GraphQL），同一专用浏览器 | session-api（严格口径 private-api） | channels/twitter.py 2–30 |
| reddit、instagram | OpenCLI 在专用浏览器里读页面（reddit 的命令说明写「先搜公开帖、先读公开页」，默认是否先走公开路径未核） | browser-session（同上） | channels/reddit.py；instagram.py 2–13 |
| podcast | 匿名读小宇宙公开单集页 | public-web | channels/xiaoyuzhou.py |
| generic-web | 第三方托管服务（r.jina.ai，失败再 Exa MCP） | public-web | channels/web.py 29、141–190 |

<a id="yichen"></a>
### yichen-skills（ed6200f，2026-10-28，只看取数类）

| 平台 | 取数机制 | 层级 | 依据 |
|---|---|---|---|
| xiaohongshu、douyin 收藏 | 读页面 DOM（用户日常 Chrome） | browser-session | `yichen-bookmarks-export/SKILL.md` 34–35；scripts/chrome_collectors.mjs |
| xiaohongshu 单篇 | 匿名读公开页，授权后才带 XHS_COOKIE | public-web / session-api | `yichen-content-archive/scripts/xiaohongshu_fetch.py` 5–6、50–61、84–112 |
| douyin 单条 | 不登录的无头 Playwright 截获 aweme/detail | public-web | douyin_download.py 108–147 |
| x 书签 | fieldtheory-cli（带 cookie 调 GraphQL） | session-api | yichen-bookmarks-export/SKILL.md 36–41 |
| x 单条 | 第三方托管服务（FxTwitter → Jina） | public-web | x_known_url.py 17–18 |
| bilibili | bili-cli + yt-dlp | private-api | yichen-content-archive/SKILL.md 69 |
| youtube | yt-dlp（浏览器 cookie 默认关） | private-api | youtube_known_url.py 2–5、141、164–165 |
| weibo | yt-dlp `--no-cookies` 匿名访客 | public-web（随 yt-dlp 微博口径） | weibo_known_url.py 34–35、136、241 |
| wechat-mp | 历史文章：wechat-article-exporter（公众号后台登录）；阅读量等：wxdown-service（mitmproxy 代理取客户端凭据） | session-api / client-hook | `yichen-wechat-mp-batch-exporter/SKILL.md` 29–30；scripts/start_wxdown_service.py 2–23 |
| wechat-chat | 本地库解密 | client-hook | 已核 |
| podcast | 匿名读单集页；清单走 OpenCLI | public-web | yichen-content-archive/SKILL.md 70 |
| zhihu | zhihu-open-platform-cli（Keychain 认证，只搜公开内容，运行时来源仓库自己也没核） | 未定 | yichen-unified-search/SKILL.md 63 |
| generic-web | Jina Reader、Firecrawl | public-web | yichen-content-archive/SKILL.md 61 |

<a id="others"></a>
### chubbyskills、bilibili-douyin-notes、Downvid

- **chubbyskills（d066d76）**：微博、知乎、TikTok、B站、YouTube 的 Skill 都调 yt-dlp（`weibo-transcribe/transcribe.py` 27、58–66；`zhihu-transcribe` 28、52；`tiktok-transcribe` 27、52–60），层级跟随 yt-dlp 对应抽取器：B站、YouTube private-api，微博、知乎（只有知乎视频）、TikTok public-web。抖音、小红书、公众号、播客匿名读页面（public-web），小红书和 X 配 cookie 后为 session-api。
- **bilibili-douyin-notes**：沿用已有条目的分平台说明（抖音 private-api，B站 public-web/session-api，小红书 public-web，YouTube private-api）。
- **Downvid**：抖音 session-api、快手 public-web；其余站点交给 yt-dlp，且默认 `cookieMode=auto` 会读本机浏览器 cookie，所以 X、Instagram 实际是 session-api，B站、YouTube 是 private-api。

<a id="corrections"></a>
## 已收录条目的更正

1. **downvid/downvid**：access_tier 应由 session-api 改为 private-api。B站、YouTube 链接默认交给 yt-dlp（`app/internal/parse/parse.go`），条目自己的 needs_check 已写「若 yt-dlp 定为 private-api，本条需同步」。
2. **iawia002/lux**：tier_evidence 说「番剧路径另有签名参数」不对，`bilibili.go` 79 行只是注释，没有签名代码；B站 路径是 public-web/session-api。条目整体仍是 private-api（抖音 X-Bogus、YouTube Android 客户端）。platforms 可补 weibo、zhihu、instagram、x、reddit、pinterest、kuaishou、tiktok、toutiao-xigua。
3. **soimort/you-get**：tier_evidence 说 B站 路径属 public-web 不对，默认流程会调用 appkey/secret 做 md5 签名的 `bilibili_interface_api`（98–103、268–290），B站 是 private-api。platforms 可补 douyin、weibo、zhihu、instagram、x、tiktok、toutiao-xigua、kuaishou。
4. **diygod/rsshub**：platforms 漏了 bilibili、xiaohongshu、douyin（都有取用户内容或收藏的路由）；知乎收藏夹路由不签名，算签名的只有动态、回答、文章三类路由。
5. **ltaoo/wx_channels_download**：最新一份条目把 douyin、xiaohongshu、weibo、x、youtube、instagram、kuaishou 都记成 client-hook，实际只有视频号和公众号历史列表走代理注入，其余见上表。
6. **cv-cat/catbus**：tier_evidence 说「微博有 sign.ts」，那只是上传参数；微博读取是 session-api。TLS 层是 wreq-js。
7. **yt-dlp/yt-dlp**：platforms 可补 xiaohongshu、weibo、zhihu、pinterest、reddit；x、instagram、tiktok、podcast、generic-web 的实际层级比条目整体低（见上表）。
8. **mikf/gallery-dl**：platforms 可补 bilibili；weibo、pinterest 是 public-web，instagram 是 session-api，reddit 走官方 OAuth。
9. **jackwener/opencli**：小宇宙 history 是模拟 App 客户端（private-api），微信读书非官方路径是 session-api（原矩阵里 weread × private-api 是假格子）。
10. **panniantong/agent-reach**：通用网页默认交给 r.jina.ai（第三方托管）。

<a id="matrix"></a>
## 剔除假值后真正空着的格子

**范围**：只算 xiecun_fit 为 fetch 或 fetch+transform 的条目。拆开了 14 个多平台工具（上面各表，外加 bilibili-douyin-notes、Downvid）；其余条目仍按条目级层级 × platforms 计。同一仓库有多个版本时取文件修改时间最新的一份。

**去掉的假格子（10 个）**：抖音、小红书、微博、X、YouTube、Instagram、快手、通用网页 × client-hook（来自 wx_channels_download 和 yichen-skills 的条目级层级）；TikTok × session-api（Downvid）；微信读书 × private-api（OpenCLI）。

**拆开后新出现的真格子（9 个）**：豆瓣 public-web（RSSHub）；Instagram public-web（yt-dlp、lux、wx_channels_download）；Reddit public-web（yt-dlp、lux、cobalt）、session-api（yt-dlp）、browser-session（Agent-Reach）；Pinterest session-api（gallery-dl）；微博 browser-session（OpenCLI）、official-export（RSSHub 官方 OAuth）；YouTube browser-session（OpenCLI）。

**仍然空着的格子**（主要平台）：

| 平台 | 空着的层级 |
|---|---|
| 抖音 | official-export（见下方「被掩盖的格子」）、client-hook |
| TikTok | vision-ui、session-api、client-hook |
| 快手 | official-export、vision-ui、browser-session、client-hook |
| 头条/西瓜 | official-export、vision-ui、browser-session |
| 小红书 | official-export、client-hook |
| B站 | official-export |
| YouTube | public-web、vision-ui、client-hook |
| 公众号 | official-export、browser-session |
| 视频号 | 除 client-hook 外全部空着 |
| 微信读书 | public-web、vision-ui、browser-session、private-api、client-hook |
| 微博 | client-hook |
| 知乎 | official-export |
| 豆瓣 | official-export、client-hook |
| X | vision-ui、client-hook |
| Instagram | official-export、vision-ui、client-hook |
| Reddit | vision-ui、client-hook |
| 播客 | official-export、vision-ui、browser-session、session-api |
| 通用网页 | official-export、vision-ui、client-hook |

判读注意：

- vision-ui 这一列是否空着，取决于通用手机或桌面 Agent 条目有没有在 platforms 里列这个平台（多数只写 other）。这类工具本身不分平台，「空着」只说明没有专门针对这个平台的视觉方案。
- **被掩盖的格子**（条目级取最高一层，把低层路径藏起来了）：ZhangYukwiver/DouyinDataReview 能导入抖音官方「个人信息下载」档案（official-export），但条目整体记 private-api；instaloader 加 `--no-iphone` 是 session-api、不登录取公开主页是 public-web；zimya/zhihu_obsidian 只带 cookie（session-api），已作为单独条目计入。
- **只靠未核路由撑着的格子**：jike、pinterest、tieba、telegram × private-api（RSSHub、OpenCLI 未核），头条/西瓜、知乎 × client-hook（yichen-skills 未核）。
- **对版本选择敏感**：若 OpenCLI 条目级层级取另一个版本（session-api），TikTok 与贴吧 × session-api 会被 OpenCLI 未核适配器填上，Reddit × private-api 会变空。

<a id="part2"></a>
## 第二部分：基础依赖库

<a id="q-extract"></a>
### Go 内核做 web-generic 与 HTML→Markdown 的候选排序

1. **正文提取首选 markusmobius/go-trafilatura v2**（Apache-2.0，v2.2.6，2026-09-29）：trafilatura 2.2.0 主流程的 Go 原生移植，带日期、语言、JSON-LD 元数据，内置 go-readabilityV2 与 go-domdistiller 两套备选。README 公布了与 Rust 版的速度和质量对比（2659 个页面，FAST 模式约 11.3 ms/页）。代价：依赖 wasilibs/go-re2（RE2 编译成 Wasm、由 wazero 运行），体积大，放进 Wasm 沙箱等于 Wasm 套 Wasm（待核能否换回标准库 regexp）。
2. **轻量选择 codeberg.org/readeck/go-readability/v2**（MIT，v2.1.3，2026-09-30，pkg.go.dev）：go-shiori 版 README 指定的续作，兼容 Readability.js 0.6；Tangentry 已在用。依赖少，适合放进沙箱，但 Readability 系对中文站点的老问题会一并继承（go-shiori #101 未回复）。
3. **补充算法 markusmobius/go-domdistiller**（MIT，v1.0.0，2026-09-15）：go.mod 只有 go-shiori/dom、x/net、x/text、zerolog，纯 Go；能给出上一页/下一页链接，多页文章有用。
4. **不要用 go-shiori/go-readability**：已归档，只兼容 Readability.js 0.5。
5. **HTML→Markdown 用 JohannesKaufmann/html-to-markdown v2**（MIT，v2.5.2）：插件式，表格、删除线插件，`WithDomain` 把相对链接改成绝对链接；它不处理页面声明的字符集（#213），GBK 一类页面要先转码。Go 里没有同等成熟度的第二选择；turndown（JS）是 defuddle、OpenCLI、baoyu-skills 等用的参照实现。

中文站点（公众号、知乎）的已知问题：mozilla/readability #921 公众号正文几乎全丢；obsidian-clipper #849 公众号图片随链接参数丢失、#647 知乎排版丢失；defuddle #353 公众号代码块转义（已修）、PR #356 公众号公式（未合并）；jina-ai/reader #1187 公众号返回「环境异常」验证页；trafilatura #541 gb2312 识别错误（已修）。建议：通用提取前先做站点预处理（`data-src` 改 `src`、显示 `js_content`），站点规则复用 fivefilters/ftr-site-config 的格式（已收录）。

<a id="q-cookie"></a>
### session-api 插件拿 cookie：可行方式与各系统差异

| 系统 | Chrome / Edge | Firefox | Safari |
|---|---|---|---|
| macOS | 可读：密钥在钥匙串「Chrome Safe Storage」，读取时弹钥匙串授权（yt-dlp `cookies.py` 995–1000 用 `security find-generic-password`） | 可读，cookies.sqlite 不加密 | 读 `~/Library/Containers/com.apple.Safari/.../Cookies.binarycookies`（cookies.py 578–582）；是否要「完全磁盘访问权限」待核 |
| Windows | **基本不可行**：Chrome 127+ 的应用绑定加密（v20），kooky #92（2024-10-30）未关，yt-dlp #10927（2024-09-05）未关，维护者 2026-01-07 提醒变通办法会关掉一项安全功能并清空全部 cookie；Edge 由策略 ApplicationBoundEncryptionEnabled 默认开启同类加密；Chrome 运行时数据库被锁（yt-dlp #7271 未关）；rookie 声称以管理员权限可读，但已归档 | 可读 | — |
| Linux | 可读：密钥在 libsecret 或 KWallet（cookies.py 780–970） | 可读 | — |

另外两条限制：Chrome 136 起，`--remote-debugging-port/pipe` 对默认用户目录不再生效（Chrome 官方博客 2025-03-17，理由就是应用绑定加密后攻击者转用远程调试偷 cookie），所以不能靠 CDP 连上用户日常配置去读 cookie；Chrome 144 在 `chrome://inspect#remote-debugging` 加了开关，每次连接弹窗确认（2025-12 时为 Beta，现状待核）。go-rod 的 NewUserMode 在 Chrome 136+ 不可用（#1189 未关）。

建议的接口设计顺序：

1. **用户用扩展导出**（Get-cookies.txt-LOCALLY：只在本地导出 Netscape 或 JSON；只导出当前站点，隐私面最小）。yt-dlp 维护者在 #10927 里也把用户引到「导出 cookies」这条路。
2. **本机 CookieCloud 兼容接收端**：扩展把选定域名的 cookie 用 `md5(uuid-密码)` 前 16 位作密钥、CryptoJS AES 加密后推送（README 143–170）。wx_channels_download 已在 Go 里实现接收端（`internal/api/handler_cookie.go` 20–40）。歇存可只监听 localhost，不用第三方服务器；CookieCloud 是 GPL-3.0，按 README 公开的协议独立实现、不复制代码，是否需要法律意见请发起 session 定。
3. **直接读浏览器存储**（kooky，Go，MIT）：只在 macOS、Linux 和 Firefox 上提供；会接触所有站点的 cookie，要按域名过滤、只在内存里用、不落盘，并在界面上说明会弹钥匙串授权。Windows 上的 Chrome、Edge 不提供。

<a id="q-browser"></a>
### 浏览器底座的几个事实

- **Playwright 默认参数里的 `--enable-automation`**：v1.59.0 及以前在 `chromiumSwitches.ts` 里（`assistantMode ? '' : '--enable-automation'`）；提交 4b1b9d681f（2026-04-14，PR #40190）删掉，首个版本 v1.60.0（2026-05-11）；v1.64.1 的 chromiumSwitches.ts 59–104 行已没有。同一提交也删掉了只在 assistantMode 下生效的 `AutomationControlled` 关闭项。现在默认启动时 `navigator.webdriver` 的值未核。patchright README 写的「去掉 `--enable-automation`」现在上游也做了。
- **连接用户已登录的 Chrome**：Playwright `connectOverCDP`、chromedp 的 `remote` 包都需要对方开调试端口，而 Chrome 136+ 对默认配置不生效。可行的做法只有：单独的持久化配置（用户在里面再登录一次）、Chrome 144+ 的 inspect 开关加逐次确认、浏览器扩展桥接（OpenCLI、Playwright MCP 扩展），或改走 cookie 导出。
- **三个改版浏览器的口径**：patchright 只到「隐藏自动化标记」（避开 Runtime.enable、关 Console、改启动参数，不改设备特征值）；Camoufox 与 CloakBrowser 是「改设备指纹」（C++ 层回报伪造的设备特征）。CloakBrowser 的浏览器二进制是专有授权（BINARY-LICENSE 1.3，最新大版本需付费），包装代码才是 MIT。

<a id="q-tls"></a>
### TLS 指纹库（只记事实）

- **curl_cffi**（MIT，v0.17.0）：已收录条目里依赖它的有 yt-dlp（可选，pyproject 60–61 行，限制 <0.18）、XHS-Downloader、zhihu-cli、twitter-cli、twscrape、wechat-download-api、trove-ai、NewsCrawler、MoneyPrinter、FunClip；KS-Downloader 见抖音方向笔记。
- **utls**（BSD-3-Clause，v1.8.2）与 **tls-client**（BSD-3-Clause，v1.16.0）：已收录条目里只有 wx_channels_download（go.mod 94–95、170）。
- **wreq-js**（Rust wreq 的 Node 绑定）：catbus（package.json 66）。本批没用 gh api 核实仓库。

<a id="q-thanks"></a>
### 第二部分里建议进致谢名单的

歇存内核或官方插件会直接依赖、或直接采用其协议与约定的：go-trafilatura、go-domdistiller、readeck/go-readability（注明源自 go-shiori）、html-to-markdown、trafilatura（go-trafilatura 的上游）、mozilla/readability、defuddle、turndown（算法与站点规则参照）、Obsidian Web Clipper（模板与 frontmatter 约定）、Playwright、chromedp、kooky、CookieCloud（协议兼容）、Get-cookies.txt-LOCALLY（推荐给用户）。

不进致谢、只在风险说明里引用的：patchright、Camoufox、CloakBrowser、curl_cffi、utls、tls-client、rookie（已归档）、crawl4ai、Jina Reader、Firecrawl（AGPL）、markdownload、web-clipper（GPL-2.0+）、go-rod、go-shiori/go-readability（已归档）。

<a id="defuddle-finding"></a>
### 给 05 的一条发现

Obsidian Web Clipper 默认调用 `defuddle.parseAsync()`（`src/content.ts` 216–222）。在 YouTube 页面上，defuddle 会依次以 IOS、ANDROID、WEB 客户端身份请求 InnerTube player 取字幕（`src/extractors/youtube.ts` 29–47、796–867）；X 文章在页面无内容时会请求第三方 FxTwitter API（`x-oembed.ts` 162、226；README 300 行）。按组内 YouTube 口径，剪 YouTube 页面这条默认路径属 private-api，login 为 none；剪普通页面只是读当前页面 DOM（browser-session）。歇存若在内核里用 defuddle 一类库，应只用同步的 `parse()`，不触发联网。
