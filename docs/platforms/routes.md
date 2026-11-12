<a id="各平台的取法"></a>
# 各平台的取法

每个平台一个获取插件仓库 `usuyo-fetch/<平台>`，仓库里有几种取法，每种取法有自己的层级、账号和记录到的情况；包一个上游工具也是一种取法。yt-dlp 单独一个跨平台仓库，见[第 15 节](#15-yt-dlp-通用视频)。手机那几种取法的设计在 [phone.md](phone.md)。各平台每一样走哪条路的一页总表在 [README](README.md#2-总表)。

依据：2026-11-12 用 `gh api` 读的上游仓库（[第 16 节](#16-依据)），[调研 06](../research/06-platform-access-and-account-risk.md)，[旧笔记](../research/notes/README.md)。没有向平台的主机发过请求，没有运行第三方代码。「推断」是没有证据的判断；「命令待核」是上游文档里没看到确切的命令或接口。

<a id="1-怎么读这一页"></a>
## 1. 怎么读这一页

每个平台一张表，一行一种取法，先列已有的，再按用到的账号从少到多排。平台的顺序：小红书、视频号、B站、X，再其余。

| 栏 | 内容 |
|---|---|
| 取法 | 在仓库里的名字 |
| 拿到什么 | 单条图文、单条视频、存过的（收藏、赞过、稍后再看、书签）、按创作者或关键词 |
| 账号 | [总表的六个说法](README.md#1-怎么读这一页) |
| 层级 | [协议 §2.4](../protocol/manifest.md#24-access-与-login) 的标签 |
| 做成什么样 | 自己写的代码，或包的上游工具：上游、许可证、安装方式、查的版本、调的命令或接口 |
| 你要装或给的 | 程序、系统授权、要登录的站点 |
| 已有 | 做了、设计了、没有 |
| 记录到的情况 | 上游 issue 和调研里记下的事，带出处 |
| 不用账号能试 | 「能」：粘一条公开链接就能真试；「不能」：要账号或你导出的文件。开发机上只跑离线样例（[BUILD §2](../BUILD.md#2-干活的规矩)） |

规矩：

- 不要账号能拿到的，不带账号取（[BUILD §1](../BUILD.md#1-用户定下的做法)）。「存过的」那几种取法只交链接，内容由同一仓库里不要账号的取法去取。
- 包的工具里发帖、点赞、关注、私信的命令，取法不调：命令行只调表里写的子命令；MCP 只调读的工具，写的工具在上游的权限配置里关掉（[协议 §9.3](../protocol/wrapping.md#93-包装-mcp-服务)）；手机流程只点白名单里的字（[phone.md §5](phone.md#5-流程永远不做的事)）。
- GPL 系的工具由你安装，取法以独立进程或本机 HTTP 调用；MIT、Apache、BSD、Unlicense 的可以进资产，第三方程序只取上游官方 release，锁 sha256（[协议 §2.8](../protocol/manifest.md#28-安装与启动)、[§9.6](../protocol/wrapping.md#96-许可证)、[ADR-003 §8](../decisions/ADR-003-plugins.md#8-gpl-工具)）。非商业或源码可见许可的工具不调，那几行写「等法律意见」。没有 release 二进制的 MIT 项目，把用到的那一段移植成自己的代码，保留版权声明。
- `client-hook` 的取法放 `usuyo-fetch-hook/<平台>`（[CLAUDE.md](../../CLAUDE.md) 原则 1），列在每张表最后。
- 一个仓库里几种取法，清单现在一个能力只能声明一份（[协议 §2.3](../protocol/manifest.md#23-获取插件的字段)）；一个仓库能写几份 `fetch`、`list` 的写法，协议还没有。

<a id="2-先做的十个"></a>
## 2. 先做的十个

| # | 仓库 · 取法 | 层级 | 做成什么样 | 等什么 |
|---|---|---|---|---|
| 1 | plugin-kit 视觉层 SDK · iPhone 驱动 | — | 包 mirroir-mcp，加自己的剪贴板和节奏（[phone.md §4](phone.md#4-驱动和流程分开)） | — |
| 2 | `wechat-channels` · `iphone` | `vision-ui` | 自己写的流程 | 1 |
| 3 | `xiaohongshu` · `iphone` | `vision-ui` | 自己写的流程 | 1 |
| 4 | `ytdlp` | 按平台 | 包 yt-dlp | — |
| 5 | `douyin` · `share-page` | `public-web` | 移植 parse-video 的抖音解析 | — |
| 6 | `douyin` · `export` | `official-export` | 自己写的导入 | 你申请的一份「个人信息下载」 |
| 7 | `wechat-mp` · `article` | `public-web` | 移植 wechat-article-exporter 的单篇管线 | — |
| 8 | `wechat-mp` · `favorites-iphone` | `vision-ui` | 自己写的流程 | 1 |
| 9 | `kuaishou` · `share-page` | `public-web` | 移植 parse-video 的快手解析 | — |
| 10 | `weibo` · `status`、`userscript` | `public-web`、`browser-session` | 自己写的单条，改写的 Weibo-archiver 脚本 | — |

不用账号能试的：4、5、7、9，和 10 的单条。1 在 iPhone 的「设置」「备忘录」里试，不碰平台账号。

顺序（用户 11-12）：电脑上的取法先做先试，4、5、6、7、9、10 在前；用到 iPhone 驱动的 1、2、3、8 排在后面，手机上一条一条点比电脑上慢。

<a id="3-小红书"></a>
## 3. 小红书

`usuyo-fetch/xiaohongshu`

| 取法 | 拿到什么 | 账号 | 层级 | 做成什么样 | 你要装或给的 | 已有 | 记录到的情况 | 不用账号能试 |
|---|---|---|---|---|---|---|---|---|
| `note-page` | 单条图文、视频（不登录是低清） | 访客身份 | `public-web` | 自己写（[xiaohongshu.md §2.4](xiaohongshu.md#24-插件做成的样子)） | 在 usuyo 里取一次访客身份 | 做了 | 空 cookie 的长链被送回首页（[XHS-Downloader #495](https://github.com/JoeanAmier/XHS-Downloader/issues/495)）；不填 cookie 批量取约 108 条后，复制链接的账号被下线（[#305](https://github.com/JoeanAmier/XHS-Downloader/issues/305)） | 能 |
| `userscript` | 存过的：收藏、赞过 | 账号只在你自己的浏览器里 | `browser-session` | 自己写，改写自 xhs-vault（MIT） | 脚本管理器和脚本，导入页发的令牌 | 做了 | 网页对采集类扩展弹「有害插件」（[#283](https://github.com/JoeanAmier/XHS-Downloader/issues/283)）；经扩展驱动用户自己的 Chrome 读，被判违规（[OpenCLI #842](https://github.com/jackwener/OpenCLI/issues/842)）；只截获、不发请求的脚本没有处罚报告（[06 §16.2](../research/06-platform-access-and-account-risk.md#162-issue-里的账号问题)） | 不能 |
| `iphone` | 存过的：收藏（笔记）、赞过 | 账号在你手机的 App 里 | `vision-ui` | 自己写的流程，iPhone 驱动（[phone.md §7.2](phone.md#72-小红书收藏)） | 「mirroir serve」的屏幕录制和辅助功能；接力（通用剪贴板） | 没有 | 镜像没有公开的识别接口，没有处罚报告（[06 §1.2](../research/06-platform-access-and-account-risk.md#12-视觉层的风险从哪里来)）；分享链接带跟踪参数（#305）；`xsec_token` 几小时后 404（[06 §1.3](../research/06-platform-access-and-account-risk.md#13-混合路径视觉层拿链接公开页面取内容)） | 不能 |
| `android` | 存过的：收藏、赞过 | 账号在你安卓手机的 App 里 | `vision-ui` | 自己写的流程，adb 驱动（[phone.md §8](phone.md#8-安卓经-adb)） | 安卓手机，USB 调试，那台手机上登录小红书 | 设计了（[协议 §9.5](../protocol/wrapping.md#95-视觉层流程) 例子二） | USB 调试开关在 Android 16 及以前 App 读得到（[调研 10 §5.1](../research/10-desktop-and-android-agents.md#51-平台能不能察觉)） | 不能 |
| `xhs-downloader` | 单条图文、视频 | 不要（它配置里的 cookie 清空） | `public-web` | 包 [XHS-Downloader](https://github.com/JoeanAmier/XHS-Downloader)（GPL-3.0，2.8，2026-09-12；官方 release 的 macOS 包，或 `uv run main.py api`）：`POST http://127.0.0.1:5556/xhs/detail`，带它启动时给的令牌，`download: false` | 安装并开 API 模式 | 设计了（[协议 §9.2](../protocol/wrapping.md#92-包装本机-http-服务)） | 同 `note-page` | 能 |
| `rsshub` | 按创作者列 | 不要（推断） | `public-web` | 包你自己跑的 RSSHub（AGPL-3.0，2026-11-12 的构建），它用不登录的无头浏览器截获主页请求；路由命令待核 | docker 跑一份 RSSHub | 没有；等[协议 §13](../protocol/open.md#13-待定) 第 26 项 | 无头浏览器在小红书上记高（[06 §1.1](../research/06-platform-access-and-account-risk.md#11-七个层级)） | 能 |

没有列的：本地签名（redbook、Spider_XHS、MediaCrawler），有永久封禁（[MediaCrawler #865](https://github.com/NanmiCoder/MediaCrawler/issues/865)），MediaCrawler 的许可没有分发权；关键词搜索只找到要签名的。

<a id="4-视频号"></a>
## 4. 视频号

`usuyo-fetch/wechat-channels`

| 取法 | 拿到什么 | 账号 | 层级 | 做成什么样 | 你要装或给的 | 已有 | 记录到的情况 | 不用账号能试 |
|---|---|---|---|---|---|---|---|---|
| `feed-info` | 单条：作者、描述、封面 | 不要 | `public-web` | 自己写，移植 ParseHub 的 `wxchannels.py`（MIT）（[wechat-channels.md §3.4](wechat-channels.md#34-插件做成的样子)） | — | 做了；2026-11-12 对真链接跑通 | 匿名接口给的字段在收紧（[wechat-channels.md §3.2](wechat-channels.md#32-取单条)） | 能 |
| `yuanbao` | 单条视频 | 要在 usuyo 里登录腾讯元宝 | `session-api` | 同上，元宝换临时凭证 | 登录元宝 | 做了；账号只用在视频这一样，清单要等 `accountOutputs`（[README 第 1 节](README.md#1-怎么读这一页)） | 公开部署的解析服务被大量调用后，元宝账号被封（[ltaoo #495](https://github.com/ltaoo/wx_channels_download/issues/495)） | 不能 |
| `userscript` | 存过的：收藏、赞过、看过的 | 账号只在你自己的浏览器里 | `browser-session` | 照页面公开的脚本读（2026-11-12，没登录）：网页版 `channels.weixin.qq.com/web/pages/account/like`、`…/account/history` 的列表不经 fetch、XHR、WebSocket 出页面，都走 `WeixinJSBridge.invoke("finderH5ExtTransfer")`，这个桥只有微信桌面客户端注入；页面没有扫码登录，普通浏览器里落到「当前版本暂不支持」。条目里没有分享码，要再经桥调 `FinderGetFeedH5Url` 才换得到 `weixin.qq.com/sph/` 链接 | 脚本管理器和探针脚本 | 做了探针（`userscript/usuyo-sph-probe.user.js`，只报结构、不带内容，不联网）；正式脚本没有：照代码看普通浏览器里做不了，等你跑一次探针确认 | 同 `iphone` 一行的协议措辞。在客户端里读这个桥就是改客户端，归 `client-hook` | 不能 |
| `iphone` | 存过的：「赞和收藏」 | 账号在你手机的微信里 | `vision-ui` | 自己写的流程，iPhone 驱动（[phone.md §7.1](phone.md#71-视频号赞和收藏)） | 同小红书 `iphone` | 没有；分享菜单里有「复制链接」，复制的东西 Mac 拿得到（用户 11-12） | 微信协议 8.2.1.6 禁止用第三方软件读界面、自动操作；处罚记录在 PC 微信 UI 自动化和安卓系统级注入上，镜像没有报告（[06 §1.2](../research/06-platform-access-and-account-risk.md#12-视觉层的风险从哪里来)） | 不能 |
| `mac` | 存过的：「赞和收藏」 | 账号在你电脑的微信里 | `vision-ui` | 自己写的流程，资产里带 agent-desktop（Apache-2.0，v1.2.0，2026-11-05） | usuyo 的辅助功能 | 没有；等你看 Mac 微信里有没有「赞和收藏」（[调研 10 §0](../research/10-desktop-and-android-agents.md#0-结论) 问题 4） | 视频号窗口由 `WeChatAppEx` 承载，树里有什么没核；Mac 微信 4.1 在 macOS 26 上树只剩空按钮 | 不能 |
| `android` | 存过的：「赞和收藏」 | 账号在你安卓手机的微信里 | `vision-ui` | adb 驱动上的流程 | 登录微信的那台手机是安卓 | 没有 | 微信同一时间只登录一台手机（推断，[phone.md §8](phone.md#8-安卓经-adb)） | 不能 |
| `audio` | 单条的口播文字 | 账号在你手机的 App 里，或你电脑的微信里（播放在那里） | `vision-ui` | 自己写：录 iPhone 镜像或电脑微信播放时这一个 App 的声音，交预装的转写；macOS 26 上可用 mirroir-mcp 的 `listen` | usuyo 的「屏幕与系统录音」 | 没有 | 只有声音；`listen` 要 macOS 26，这台 Mac 是 15.6.1 | 不能 |
| `usuyo-fetch-hook/wechat-channels` · `wxcd` | 存过的：赞过、收藏、播放记录 | 账号在你电脑的微信里 | `client-hook` | 包 [wx_channels_download](https://github.com/ltaoo/wx_channels_download)（MIT 加 Commons Clause，v261106）：本机 MCP `127.0.0.1:2022/mcp`，只调 `get_wxchannels_interacted_videos` 一类读的工具 | 安装它的根证书，设系统代理 | 没有；等法律意见（[协议 §9.6](../protocol/wrapping.md#96-许可证)） | 11-05 起在微信 4.2 上失效又修（[#594](https://github.com/ltaoo/wx_channels_download/issues/594)、[#603](https://github.com/ltaoo/wx_channels_download/issues/603)）；根证书私钥随仓库公开（[#586](https://github.com/ltaoo/wx_channels_download/issues/586)）；异常退出后系统代理残留（[#575](https://github.com/ltaoo/wx_channels_download/issues/575)） | 不能 |

手机上手动「分享 → 复制链接」交收件文件夹，一直可用（[wechat-channels.md §3.1](wechat-channels.md#31-列出你存过的)）；`iphone` 替你点这几下。

<a id="5-b站"></a>
## 5. B站

`usuyo-fetch/bilibili`

| 取法 | 拿到什么 | 账号 | 层级 | 做成什么样 | 你要装或给的 | 已有 | 记录到的情况 | 不用账号能试 |
|---|---|---|---|---|---|---|---|---|
| `view` | 单条：标题、简介、封面、分 P、低清音视频 | 不要 | `public-web` | 自己写（[bilibili.md §4.4](bilibili.md#44-插件做成的样子)） | — | 做了 | 2026-06 起缺指纹参数的部分接口回 412（[BiliNote #397](https://github.com/JefferyHcool/BiliNote/issues/397)） | 能 |
| `favorites-public` | 存过的：公开的收藏夹（给收藏夹或个人空间的链接） | 不要 | `public-web` | 自己写：和 `favorites` 同两个接口，不带 cookie。2026-11-12 不带账号实测：`x/v3/fav/folder/created/list-all?up_mid=` 列出这个人公开的收藏夹，`x/v3/fav/resource/list?media_id=` 分页回每条的 `bvid` 和收藏时间；内容由 `view` 取 | — | 没有；等清单里的 `methods`（[BUILD 011](../BUILD.md#011-更多平台)） | 只有公开的收藏夹列得到；设成私密的和稍后再看要 `favorites` 或 `userscript` | 能 |
| `favorites` | 存过的：收藏夹、稍后再看 | 要在 usuyo 里登录 | `session-api` | 自己写，不签名的接口 | 扫码登录 B站 | 做了 | 一例大会员被冻结，因果不明（[yutto #452](https://github.com/yutto-dev/yutto/issues/452)） | 不能 |
| `userscript` | 存过的：收藏夹、稍后再看 | 账号只在你自己的浏览器里 | `browser-session` | 自己写，照 [bilibili-favorites-exporter](https://github.com/ayasa520/bilibili-favorites-exporter)（MIT，2025-09-09 后没推送）只读页面自己拿回的数据，交 `/v1/batches` | 脚本 | 做了（脚本 0.1.0，11-12）；没在真浏览器里跑过，稍后再看的回答形状没实测 | 同类脚本（X）调研里没有处罚记录 | 不能 |
| `yutto` | 单条：高清音视频、字幕 | 要在 usuyo 里登录 | `session-api` | 包 [yutto](https://github.com/yutto-dev/yutto)（GPL-3.0，v2.3.1，2026-08-28；`uv tool install yutto`）：`yutto <链接> --subtitle-only --no-danmaku --auth-file <0600 的临时文件>`，画面、声音分两次 `--video-only`、`--audio-only`；输出目录参数和下完是否调 ffmpeg，命令待核 | 安装 yutto | 没有 | B站 2026 年给接口文档、SDK、下载器的维护者发律师函，点名登录态、签名鉴权的适配（[06 §16.3](../research/06-platform-access-and-account-risk.md#163-下架停更与法律风险)） | 不能 |
| `usuyo-fetch-hook/bilibili` · `cache` | 你在 B站 客户端里缓存过的视频 | 账号在你电脑的 B站 客户端里 | `client-hook` | 读客户端缓存的 m4s，照 [m4s-converter](https://github.com/mzky/m4s-converter)；没有建 | — | 没有 | B站 2026-01 至 07 给接口文档、SDK、下载器维护者发律师函，bilibili-API-collect 删文档归档，downkyi、BiliTools、BiliRoaming 收函（[06 §16.3](../research/06-platform-access-and-account-risk.md#163-下架停更与法律风险)） | 不能 |

没有列的：UP 主投稿列表要 WBI 签名（`private-api`）；BBDown 已归档（最后推送 2026-05-14）。

<a id="6-x"></a>
## 6. X

`usuyo-fetch/x`

| 取法 | 拿到什么 | 账号 | 层级 | 做成什么样 | 你要装或给的 | 已有 | 记录到的情况 | 不用账号能试 |
|---|---|---|---|---|---|---|---|---|
| `tweet` | 单条：正文、图片、视频、引用 | 不要 | `public-web` | 自己写（[x.md §3](x.md#3-取单条)） | — | 做了 | 访客令牌在有的出口被拒（[gallery-dl #9505](https://github.com/mikf/gallery-dl/issues/9505)） | 能 |
| `userscript` | 存过的：书签、喜欢 | 账号只在你自己的浏览器里 | `browser-session` | 自己写，照 twitter-web-exporter（MIT，v1.4.3） | 脚本 | 做了 | 没有使用者被锁的报告（[06 §16.2](../research/06-platform-access-and-account-risk.md#162-issue-里的账号问题)） | 不能 |
| `archive` | 存过的：喜欢，可能有书签 | 不要（导出时你登录 X） | `official-export` | 自己写，照 birdclaw（MIT，v0.17.0）读 zip | 一份 X 归档 | 做了 | 归档里有没有书签没核（[x.md §2](x.md#2-列出你存过的)） | 不能（要你的文件） |
| `xurl` | 存过的：书签、喜欢 | 开发者账号加你的 OAuth 授权 | `official-export` | 包 [xurl](https://github.com/xdevplatform/xurl)（MIT）：`GET /2/users/:id/bookmarks`、`GET /2/users/:id/liked_tweets` | 开发者账号，按量付费的套餐 | 没有 | 价格和上限没核 | 不能 |

<a id="7-抖音"></a>
## 7. 抖音

`usuyo-fetch/douyin`

| 取法 | 拿到什么 | 账号 | 层级 | 做成什么样 | 你要装或给的 | 已有 | 记录到的情况 | 不用账号能试 |
|---|---|---|---|---|---|---|---|---|
| `share-page` | 单条图文、视频 | 不要 | `public-web` | 自己写：读 `iesdouyin.com/share/video/<id>` 服务端渲染的数据，移植 [parse-video](https://github.com/wujunwei928/parse-video)（MIT，Go，2026-11-08 有推送，release 只有 v0.0.2、没有二进制）的抖音解析，不移植它设 `PARSE_VIDEO_DOUYIN_COOKIE` 后的详情接口那一段 | — | 没有 | 分享页字段随前端改版；未登录 cookie 轮询带签名的单作品接口约 200 次后限流、IP 约两小时恢复（[Douyin_TikTok_Download_API #626](https://github.com/Evil0ctal/Douyin_TikTok_Download_API/issues/626)，不是分享页） | 能 |
| `export` | 存过的：喜欢、收藏（包里有哪些没核） | 不要（你在 App 里申请） | `official-export` | 自己写的导入，照 [DouyinDataReview](https://github.com/ZhangYukwiver/DouyinDataReview)（MIT，v1.47.1）读「个人信息下载」JSON、ZIP 的那一段 | 一份「个人信息下载」 | 没有 | 导出类别没核（[06 §2](../research/06-platform-access-and-account-risk.md#2-覆盖矩阵每个平台有哪些层级可选)） | 不能（要你的文件） |
| `userscript` | 存过的：喜欢、收藏 | 账号只在你自己的浏览器里 | `browser-session` | 自己写：抖音网页里由页面自己的请求带回数据，脚本记作品 id 交 `/v1/batches`；参照 [douyin-favorite-MyLike](https://github.com/Hannah2227/douyin-favorite-MyLike)（MIT，v2.0.6）`legacy/` 里的页内取法（`legacy/` 没读，推断），它的取消点赞、取消收藏不搬 | 脚本 | 做了脚本（0.1.0，11-12，`usuyo-fetch/douyin/userscript/`）；回答的形状全照公开资料写，没实测，没在真浏览器里跑过；插件本体还没有 | 2026-08 起页面外带 cookie 调收藏接口常回 403，页面自己发的请求能取（[06 §0](../research/06-platform-access-and-account-risk.md#0-要点)）；限制多是几小时，没有封号报告（[06 §16.2](../research/06-platform-access-and-account-risk.md#162-issue-里的账号问题)） | 不能 |
| `ddr-import` | 存过的：喜欢、收藏、观看历史 | 账号在 DouyinDataReview 里 | `official-export`（读文件） | 自己写的导入，读 DouyinDataReview 导出的 JSON；`description` 写「清单由 DouyinDataReview 用你的登录会话发签名请求取得」 | 安装 DouyinDataReview 并导出 | 没有 | 取清单那一步在 DouyinDataReview 里，是 `private-api` | 不能（要你的文件） |
| `iphone`、`android` | 存过的：喜欢、收藏 | 账号在你手机的 App 里 | `vision-ui` | 手机驱动上的流程 | 同小红书 `iphone`、`android` | 没有 | 调研里没有处罚记录 | 不能 |
| `usuyo-fetch-hook/douyin` · `proxy` | 单条视频 | 账号在你手机或电脑的抖音里 | `client-hook` | 本机代理截获客户端的请求，照 [res-downloader](https://github.com/putyy/res-downloader)；没有建 | 安装根证书，设系统代理 | 没有 | 调研里没有抖音账号因此被处罚的记录；字节跳动 2023 年 21 份 DMCA 针对反编译安装包和 SDK 代码（[06 §16.3](../research/06-platform-access-and-account-risk.md#163-下架停更与法律风险)） | 不能 |

没有列的：[TikTokDownloader](https://github.com/JoeanAmier/TikTokDownloader)（GPL-3.0，5.8）2026-07 起不维护加密参数，要使用者自备签名代码（`private-api`）。

<a id="8-快手"></a>
## 8. 快手

`usuyo-fetch/kuaishou`

| 取法 | 拿到什么 | 账号 | 层级 | 做成什么样 | 你要装或给的 | 已有 | 记录到的情况 | 不用账号能试 |
|---|---|---|---|---|---|---|---|---|
| `share-page` | 单条图文、视频 | 不要 | `public-web` | 自己写，移植 parse-video 的快手解析（MIT）；对照 [ParseHub](https://github.com/z-mio/ParseHub) `kuaishou.py`（MIT，v2.4.1，cookie 可选）和 Downvid 的快手解析器（MIT，v1.12.0） | — | 没有 | 频繁请求要验证（[06 §1.3](../research/06-platform-access-and-account-risk.md#13-混合路径视觉层拿链接公开页面取内容)） | 能 |
| `ks-downloader` | 单条图文、视频 | 不要（一份不登录的网页 cookie） | `public-web` | 包 [KS-Downloader](https://github.com/JoeanAmier/KS-Downloader)（GPL-3.0，1.6，官方 release 有 macOS 包）：`python main.py api`，文档在 `127.0.0.1:5557/docs`；接口路径命令待核 | 安装并开 API 模式 | 没有 | 同上 | 能 |
| `iphone`、`android` | 存过的：收藏、赞过 | 账号在你手机的 App 里 | `vision-ui` | 手机驱动上的流程 | 同小红书 `iphone`、`android` | 没有 | 浏览器里没有取收藏的工具；调研里没有处罚记录 | 不能 |

<a id="9-微博"></a>
## 9. 微博

`usuyo-fetch/weibo`

| 取法 | 拿到什么 | 账号 | 层级 | 做成什么样 | 你要装或给的 | 已有 | 记录到的情况 | 不用账号能试 |
|---|---|---|---|---|---|---|---|---|
| `status` | 单条图文、视频 | 访客身份 | `public-web` | 自己写：访客身份由 usuyo 开窗口取（[协议 §2.5.1](../protocol/manifest.md#251-访客身份-visitors)），调不签名的 `ajax/statuses/show`；对照 yt-dlp `weibo.py`、ParseHub `weibo.py` | 在 usuyo 里取一次访客身份 | 没有 | 访客路径只见到 IP 层的限制（[06 §1.3](../research/06-platform-access-and-account-risk.md#13-混合路径视觉层拿链接公开页面取内容)）；微博协议 4.5.1 按频率判 | 能 |
| `userscript` | 存过的：收藏 | 账号只在你自己的浏览器里 | `browser-session` | 改写 [Weibo-archiver](https://github.com/Chilfish/Weibo-archiver) 的油猴版（Apache-2.0，v0.6.2 release 的 `weibo-archiver.user.js`）：留翻收藏的那一段，去掉下载和导出，微博 id 交 `/v1/batches` | 脚本 | 没有 | 脚本在页面里翻页；调研里没有处罚记录 | 不能 |
| `favorites` | 存过的：收藏，能跟着收 | 要在 usuyo 里登录 | `session-api` | 自己写，带 cookie 调收藏接口 | 登录微博 | 没有 | 调研里没有处罚记录 | 不能 |

<a id="10-知乎"></a>
## 10. 知乎

`usuyo-fetch/zhihu`

| 取法 | 拿到什么 | 账号 | 层级 | 做成什么样 | 你要装或给的 | 已有 | 记录到的情况 | 不用账号能试 |
|---|---|---|---|---|---|---|---|---|
| `page` | 单条：回答、文章 | 不要 | `public-web` | 自己写，读公开页；obsidian-zhihu-crawler 没有许可证，只参考做法 | — | 没有 | 调研里没有处罚记录 | 能 |
| `collection` | 存过的：公开收藏夹 | 不要 | `public-web` | 自己写，收藏夹接口不签名（[层级笔记](../research/notes/tier-calls-weibo-zhihu-douban-x-ig-reddit-tg.md)） | 收藏夹地址（等[协议 §13](../protocol/open.md#13-待定) 第 21 项） | 没有 | — | 能 |
| `collection-private` | 存过的：私密收藏夹 | 要在 usuyo 里登录 | `session-api` | 同一个接口带 cookie | 登录知乎 | 没有 | 调研里没有处罚记录 | 不能 |

没有列的：用户动态要算 `x-zse-96`（`private-api`，ParseHub `zhihu.py` 算的就是它）；关键词搜索是按额度计费的知乎开放平台。知乎视频由 `ytdlp` 取。

<a id="11-贴吧"></a>
## 11. 贴吧

`usuyo-fetch/tieba`

| 取法 | 拿到什么 | 账号 | 层级 | 做成什么样 | 你要装或给的 | 已有 | 记录到的情况 | 不用账号能试 |
|---|---|---|---|---|---|---|---|---|
| `thread` | 单条：帖子正文和图 | 不要 | `private-api` | 移植 ParseHub `tieba.py`（MIT）：电脑客户端接口 `c/f/pb/page_pc`，参数加 md5 签名；[aiotieba](https://github.com/lumina37/aiotieba)（Unlicense，v4.8.4）同样算客户端签名 | — | 没有 | 没有不要签名的工具；没有百度来函的记录（[06 §16.3](../research/06-platform-access-and-account-risk.md#163-下架停更与法律风险)） | 能 |
| `iphone` | 存过的：收藏的帖子 | 账号在你手机的 App 里 | `vision-ui` | 手机驱动上的流程 | 同小红书 `iphone` | 没有 | 调研里没有处罚记录 | 不能 |

公开帖子页能不能直接读（`public-web`），调研没查。

<a id="12-youtube"></a>
## 12. YouTube

`usuyo-fetch/youtube`；单条视频由 `ytdlp` 取。

| 取法 | 拿到什么 | 账号 | 层级 | 做成什么样 | 你要装或给的 | 已有 | 记录到的情况 | 不用账号能试 |
|---|---|---|---|---|---|---|---|---|
| `transcript` | 单条字幕 | 不要 | `private-api` | 包 [youtube-transcript-api](https://github.com/jdepoix/youtube-transcript-api)（MIT，v1.2.5，2026-11-05）；命令待核 | 安装它 | 没有 | 云服务器 IP 常被拦（[06 §1.3](../research/06-platform-access-and-account-risk.md#13-混合路径视觉层拿链接公开页面取内容)） | 能 |
| `takeout` | 存过的：稍后观看、喜欢的视频 | 不要（导出时你登录 Google） | `official-export` | 自己写的导入，照 [google_takeout_parser](https://github.com/purarue/google_takeout_parser)（MIT，v0.1.13）和 WatchNow 读 Takeout 的 CSV | 一份 Takeout | 没有 | — | 不能（要你的文件） |
| `channel-feed` | 按创作者列 | 不要 | `public-web` | 自己写，读频道的公开订阅源 | 频道地址 | 没有；等[协议 §13](../protocol/open.md#13-待定) 第 21、26 项 | — | 能 |

<a id="13-公众号和微信收藏"></a>
## 13. 公众号和微信收藏

`usuyo-fetch/wechat-mp`

| 取法 | 拿到什么 | 账号 | 层级 | 做成什么样 | 你要装或给的 | 已有 | 记录到的情况 | 不用账号能试 |
|---|---|---|---|---|---|---|---|---|
| `article` | 单条文章：正文、图片、作者、发布时间 | 不要 | `public-web` | 自己写，移植 [wechat-article-exporter](https://github.com/wechat-article/wechat-article-exporter) 的单篇管线（MIT；2026-07-30 停止维护，网站 2026-10 下线）：取文章页，换回懒加载的图，在不联网的沙箱里跑文章页自带的数据脚本取标题和时间；对照 [wechatmp2markdown](https://github.com/fengxxc/wechatmp2markdown)（MIT，v1.1.11，2025-03）和 ParseHub `weixin.py` | — | 没有 | 匿名抓取出环境验证页（[wewe-rss #388](https://github.com/cooderl/wewe-rss/issues/388)） | 能 |
| `favorites-iphone` | 存过的：微信收藏里的文章链接 | 账号在你手机的微信里 | `vision-ui` | 自己写的流程，iPhone 驱动（[phone.md §7.3](phone.md#73-微信收藏里的文章)） | 同小红书 `iphone` | 没有 | 同视频号 `iphone` | 不能 |
| `favorites-mac` | 存过的：微信收藏里的文章链接 | 账号在你电脑的微信里 | `vision-ui` | 自己写的流程，资产里带 agent-desktop（[协议 §9.5](../protocol/wrapping.md#95-视觉层流程) 例子一）；做法照 [wechat-favorites-exporter](https://github.com/pengyulong/wechat-favorites-exporter)（MIT） | usuyo 的辅助功能 | 设计了 | Mac 微信 4.1 在 macOS 26 上不给树；这台的 3.8.8.19 是 AppKit 界面（推断树还在）；PC 微信读界面有强制下线（[pywechat #260](https://github.com/Hello-Mr-Crab/pywechat/issues/260)） | 不能 |
| `usuyo-fetch-hook/wechat-mp` · `local-db` | 存过的：微信收藏全部条目 | 账号在你电脑的微信里 | `client-hook` | 取密钥、解密本机微信数据库的工具（WeFlow 一类）；没有建 | — | 没有 | 腾讯 2026-01-08 一份 DMCA 按 4,195 个仓库的派生网络处理，2026-06-29 至 10-13 又发 36 份，wx-cli 返回 451；WeFlow 用户 2026-03 起成批报告收到微信警告（[06 §16.2](../research/06-platform-access-and-account-risk.md#162-issue-里的账号问题)、[§16.3](../research/06-platform-access-and-account-risk.md#163-下架停更与法律风险)） | 不能 |

按公众号列全部文章靠公众号后台的接口（`session-api`）：wechat-article-exporter 说它用的那条 2026-07 下旬关闭（[#200](https://github.com/wechat-article/wechat-article-exporter/issues/200)）；[we-mp-rss](https://github.com/rachelos/we-mp-rss)（许可证 NOASSERTION）2026-11-10 仍有推送，2026-10-30 有 issue 报它的容器里有挖矿程序和后门，现在能不能用没核，没有列。收藏里的笔记、图片、聊天记录没有公开链接，手机流程只认链接类条目。

<a id="14-另外几个平台"></a>
## 14. 另外几个平台

收调研里有官方导出或公开页的。即刻只有带登录态的扩展和签名 SDK，导出扩展没有许可证，没有列。

| 仓库 · 取法 | 拿到什么 | 账号 | 层级 | 做成什么样 | 你要装或给的 | 已有 | 记录到的情况 | 不用账号能试 |
|---|---|---|---|---|---|---|---|---|
| `douban` · `rss` | 你标记的书影音 | 不要 | `public-web` | 自己写，读你主页的公开 RSS，照 [douban-sync-skill](https://github.com/cosformula/douban-sync-skill)（MIT，v0.2.2） | 豆瓣 id（等[协议 §13](../protocol/open.md#13-待定) 第 21 项） | 没有 | 全量翻页出算力验证（[06 §1.3](../research/06-platform-access-and-account-risk.md#13-混合路径视觉层拿链接公开页面取内容)） | 能 |
| `reddit` · `saved` | 存过的：saved | 你自己的 Reddit 应用凭据 | `official-export` | 照 [rexport](https://github.com/karlicoss/rexport)（MIT）调官方接口；命令待核 | 一个个人应用 | 没有 | 官方接口有配额 | 不能 |
| `telegram` · `export` | 存过的：Saved Messages | 不要（你在 Telegram Desktop 里导出） | `official-export` | 自己写的导入，读 Telegram Desktop「导出聊天记录」的 JSON；字段没核 | 一份导出 | 没有 | — | 不能（要你的文件） |
| `instagram` · `export` | 存过的：收藏、赞过 | 不要（你申请导出） | `official-export` | 自己写的导入，照 [scrollback](https://github.com/meizuflux/scrollback)（MIT） | 一份数据导出 | 没有 | 不登录取单条常回 429、401（[instaloader #2726](https://github.com/instaloader/instaloader/issues/2726)、[#2501](https://github.com/instaloader/instaloader/issues/2501)） | 不能（要你的文件） |
| `podcast` · `feed` | 单集音频和节目说明 | 不要 | `public-web` | 自己写，读节目的公开 RSS；[podcast-dl](https://github.com/lightpohl/podcast-dl)（MIT，v12.1.5，官方 release 有 macOS 二进制）可包，命令待核 | 节目地址 | 没有；等[协议 §13](../protocol/open.md#13-待定) 第 21–23 项 | — | 能 |

<a id="15-yt-dlp-通用视频"></a>
## 15. yt-dlp 通用视频

`usuyo-fetch/ytdlp`，包 [yt-dlp](https://github.com/yt-dlp/yt-dlp)（Unlicense，2026.10.26；Homebrew，或官方 release 的 `yt-dlp_macos`，release 带 `SHA2-256SUMS` 和签名）。只有 `fetch`，`login: none`，不传 cookie，不用 `--cookies-from-browser`。各平台的层级用 `perPlatform` 写（[协议 §2.3](../protocol/manifest.md#23-获取插件的字段)）。默认顺序照[协议 §3.1](../protocol/fetch.md#31-运行器与节奏)：不要账号的在前，同类按层级从低到高，再按安装顺序；你拖动过的照你排的。

不带 cookie 的路径，依据[多平台笔记 yt-dlp 表](../research/notes/multi-platform-tier-table-and-base-deps.md#yt-dlp)（读的是 master `b77c930`，比 2026.10.26 多三个提交，没有改抽取器）：

| 平台 | 不带 cookie 怎么取 | 层级 | 列进 `platforms` |
|---|---|---|---|
| YouTube | 以 App 客户端身份请求，本地解 JS 挑战，要 deno | `private-api` | 列 |
| B站 | 本地算 WBI 签名 | `private-api` | 不列：签名接口是律师函点名的一类 |
| 小红书 | 匿名读公开页的 `__INITIAL_STATE__` | `public-web` | 列 |
| 微博 | 访客握手（提交固定的指纹）加不签名接口 | `public-web`；按指纹的口径是 `private-api` | 列 |
| 知乎 | 不签名的 `api/v4/zvideos`，只有知乎视频 | `public-web` | 列 |
| X | 访客令牌调帖子接口，再退到嵌入接口 | `public-web` | 列 |
| Reddit | 不签名的 `<帖子>/.json` | `public-web` | 列 |
| Instagram | 匿名调页面接口，装了 curl_cffi 才发（模仿浏览器 TLS 指纹） | 笔记记 `public-web`；按指纹的口径是 `private-api` | 列 |
| 抖音 | 要一份新的访客 cookie | — | 不列：`douyin` 的 `share-page` 不要 cookie |

快手、视频号、公众号、贴吧没有抽取器（`supportedsites.md`，2026-11-12）。

调的命令，一条内容一次，照[协议 §9.1](../protocol/wrapping.md#91-包装命令行)：

```
yt-dlp --ignore-config --no-playlist --no-progress --fixup never \
  --proxy "<宿主代理的地址>" \
  -f "bv*,ba" -o "$OUT/media/%(id)s.%(format_id)s.%(ext)s" \
  --write-info-json --write-subs --sub-langs "zh.*,en.*" -- "$URL"
```

- **合并。** `-f "bv*,ba"` 的逗号让画面和声音各下一份，输出名带 `format_id`；`--fixup never` 不调 ffmpeg。`media[]` 里这两份给同一个 `group`，宿主照 B站 的做法配对，交预装的 `builtin/media` 的 `play` 合成（[bilibili.md §4.4](bilibili.md#44-插件做成的样子)）。`requires.runtime` 是 `yt-dlp>=2026.10`，YouTube 加 `deno>=2.3`，不写 ffmpeg。只有一条合流的站点上 `-f "bv*,ba"` 的结果，和 HLS 站点要不要 `--downloader m3u8:native`，没实测。
- **代理。** 宿主的本机代理和传给插件的代理变量还没做（[协议 §3.13](../protocol/fetch.md#313-宿主和-plugin-kit-现在做到的11-11)）；现在插件的出站直接出去，和已有的四个插件一样，命令里先不带 `--proxy`。代理做好以后，插件的网络只经它（[协议 §7](../protocol/isolation.md#7-权限与隔离)），yt-dlp 只用 `--proxy` 给的地址，`--ignore-config` 不读你的 `yt-dlp.conf`；宿主代理再按你的规则出去：国内平台直连，YouTube、X、Reddit、Instagram 走固定的一个出口（[README 第 4 节](README.md#4-怎么相对安全地试)）。
- yt-dlp 没有写操作；`--` 在链接前。

<a id="16-依据"></a>
## 16. 依据

2026-11-12，`gh api repos/<owner>/<repo>` 和 `releases/latest`：

| 上游 | 许可证 | 最近推送 | 最新 release |
|---|---|---|---|
| [yt-dlp/yt-dlp](https://github.com/yt-dlp/yt-dlp) | Unlicense | 2026-11-04 | 2026.10.26 |
| [jfarcand/mirroir-mcp](https://github.com/jfarcand/mirroir-mcp) | Apache-2.0 | 2026-11-12 | v0.47.0（2026-11-12） |
| [vercel-labs/agent-desktop](https://github.com/vercel-labs/agent-desktop) | Apache-2.0 | 2026-11-11 | v1.2.0（2026-11-05） |
| [JoeanAmier/XHS-Downloader](https://github.com/JoeanAmier/XHS-Downloader) | GPL-3.0 | 2026-11-12 | 2.8（2026-09-12） |
| [JoeanAmier/KS-Downloader](https://github.com/JoeanAmier/KS-Downloader) | GPL-3.0 | 2026-10-26 | 1.6（2026-07-05） |
| [JoeanAmier/TikTokDownloader](https://github.com/JoeanAmier/TikTokDownloader) | GPL-3.0 | 2026-11-04 | 5.8（2026-09-13） |
| [wujunwei928/parse-video](https://github.com/wujunwei928/parse-video) | MIT | 2026-11-08 | v0.0.2（2026-04-18），没有二进制 |
| [z-mio/ParseHub](https://github.com/z-mio/ParseHub) | MIT | 2026-11-09 | v2.4.1（2026-11-09） |
| [yutto-dev/yutto](https://github.com/yutto-dev/yutto) | GPL-3.0 | 2026-11-09 | v2.3.1（2026-08-28） |
| [nilaoda/BBDown](https://github.com/nilaoda/BBDown) | MIT | 2026-05-14，已归档 | 1.6.3（2024-08-14） |
| [Hannah2227/douyin-favorite-MyLike](https://github.com/Hannah2227/douyin-favorite-MyLike) | MIT | 2026-11-10 | v2.0.6（2026-11-10） |
| [ZhangYukwiver/DouyinDataReview](https://github.com/ZhangYukwiver/DouyinDataReview) | MIT | 2026-11-12 | v1.47.1（2026-11-12） |
| [wechat-article/wechat-article-exporter](https://github.com/wechat-article/wechat-article-exporter) | MIT | 2026-11-09，停止维护 | 0.3（2025-03-19） |
| [Chilfish/Weibo-archiver](https://github.com/Chilfish/Weibo-archiver) | Apache-2.0 | 2026-10-03 | v0.6.2（2025-06-28） |
| [lumina37/aiotieba](https://github.com/lumina37/aiotieba) | Unlicense | 2026-11-08 | v4.8.4（2026-11-01） |
| [ltaoo/wx_channels_download](https://github.com/ltaoo/wx_channels_download) | MIT 加 Commons Clause | 2026-11-12 | v261106（2026-11-06） |
| [jdepoix/youtube-transcript-api](https://github.com/jdepoix/youtube-transcript-api) | MIT | 2026-11-09 | v1.2.5（2026-11-05） |
| [lightpohl/podcast-dl](https://github.com/lightpohl/podcast-dl) | MIT | 2026-11-08 | v12.1.5（2026-11-08） |
| [DIYgod/RSSHub](https://github.com/DIYgod/RSSHub) | AGPL-3.0 | 2026-11-12 | 2026-11-12 的构建 |
