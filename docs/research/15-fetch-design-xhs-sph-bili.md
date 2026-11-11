<a id="小红书视频号b站获取插件设计2026-11-11"></a>
# 小红书、视频号、B站：获取插件设计（2026-11-11）

写给发起 session 和接手写插件的 Agent。依据是仓库文档、同组笔记，以及 2026-11-11 用 `gh api` 读的源码、README、release 和 issue。没有登录任何账号，没有对平台发请求，没有运行第三方代码，也没有读本机的微信数据目录。标「推断」的没有实测。

文中「协议」指 [plugin-protocol.md](../plugin-protocol.md)，「06」指 调研 06（同组笔记 `06-platform-access-and-account-risk.md`，不在仓库里）。

<a id="0-结论"></a>
## 0. 结论

| 平台 | 首个实现 | 兜底 | 做不到的 |
|---|---|---|---|
| 小红书 | `fetch` 走公开笔记页（带 `xsec_token` 的长链，解析 `__INITIAL_STATE__`），不签名；清单由你在日常浏览器里用油猴脚本交给 [`/v1/batches`](../plugin-protocol.md#51-本机-http-接口)，插件第一版不做 `list` | 你自装的 XHS-Downloader，按 [协议 §9.2](../plugin-protocol.md#92-包装本机-http-服务) 包装 | 不带任何 cookie 取单条不稳定：2026-10 有报告空 cookie 的长链被重定向到首页，带一份未登录的访客 cookie 才成功（[§2.2](#22-取单条)） |
| 视频号 | 分享链接 → 匿名 `get_feed_info` 取作者、描述、封面；视频地址这一步可选，借腾讯元宝的登录 cookie 换临时 token | 录下播放的声音再转写；或你自装的 wx_channels_download（`client-hook`） | 自动列出「赞和收藏」只有 Hook 客户端一条路，而且 11-05 起在微信 4.2 上反复失效；本机 3.8.8.19 能不能用没有文档（[§3.1](#31-列出你存过的)） |
| B站 | `list` 带你的 cookie 调不签名的收藏夹、稍后再看接口（`session-api`）；`fetch` 用不签名的 `view`、`playurl` | 日常浏览器里的扩展导出收藏夹 JSON，交给导入插件 | 字幕要登录；不碰要 WBI 签名的接口时，没有字幕的视频只能交给转写插件 |

最大的风险在小红书：2026 年只读也有警告、封号的报告（[06 §16.2](06-platform-access-and-account-risk.md#162-issue-里的账号问题)），匿名批量取还牵连过复制链接的账号（[XHS-Downloader #305](https://github.com/JoeanAmier/XHS-Downloader/issues/305)）。视频号的风险在可用性：能列清单的工具每次微信更新都要跟着修。

<a id="1-共用宿主要提供的"></a>
## 1. 共用：宿主要提供的

1. **协议形状。** `match` 是清单里的 glob 字段，不是子命令（[协议 §2.3](../plugin-protocol.md#23-获取插件的字段)）。每个获取插件必须有 `canonicalize` 和 `health`；有 `list` 就要有 `sources`（[协议 §3.2](../plugin-protocol.md#32-调用约定)、[§3.8](../plugin-protocol.md#38-list-与跟着收)）。
2. **登录窗口。** 用 Tauri 2 的网页窗口：`Webview::cookies_for_url()` 能读到 HttpOnly cookie（[Tauri 源码](https://github.com/tauri-apps/tauri/blob/dev/crates/tauri/src/webview/mod.rs)，2.x 由 [#12665](https://github.com/tauri-apps/tauri/pull/12665) 加入；Windows 上同步调用会死锁，要放在 async 命令里）。读出来的 cookie 按 [协议 §6](../plugin-protocol.md#6-凭据) 的 `storageState` 交给插件，所以对应 `credential: cookies`。`browser-profile` 要的是 Chrome 资料目录，WKWebView 给不了；本文三个首个实现都只用 `cookies`。每个站点用单独的数据存储，免得混在一起：macOS 上 `data_directory` 不可用，要用 `data_store_identifier`，只支持 macOS 14 及以上；Windows、Linux 用 `data_directory`（[webview_window.rs](https://github.com/tauri-apps/tauri/blob/dev/crates/tauri/src/webview/webview_window.rs)）。登录窗口按 `sites.cookies` 判断登录完成后就关。
3. **访客 cookie。** 小红书要一份未登录的访客 cookie（[§2.2](#22-取单条)）。建议宿主用同一套窗口打开 `https://www.xiaohongshu.com/` 但不登录，页面自己生成访客会话后读出 cookie，存进专用数据目录。协议里还没有「不是账号的 cookie」这一类，现在的 `login: none` 不传任何 cookie。要在 [协议 §13](../plugin-protocol.md#13-待定) 加一项：例如 `sites` 里加 `credential: visitor`，界面写「不会用到你的账号」。在这之前，按现行协议可以先用一个 `secret: true` 的插件设置（[协议 §2.11](../plugin-protocol.md#211-插件设置-settings)），你从没登录的浏览器里复制访客 cookie 粘进去，经 `USUYO_SECRETS` 传入，和 `login` 无关。
4. **节奏默认值**（推断，按下文证据定，你可以改）：

| 平台 | `batch` | `interval` | `daily` |
|---|---|---|---|
| 小红书 | `true`，只限刚交来的清单 | 15 s | 100 |
| 视频号（元宝那一步） | `true` | 30 s | 50 |
| B站 | `true` | 5 s | 500 |

5. **手机分享入口。** 协议已有：监听文件夹（[协议 §5.2](../plugin-protocol.md#52-监听文件夹)）收 `.txt` 分享文字，局域网直传默认关。三个平台都先用它，不用设备权限。
6. **设备权限。** 三个首个实现都不要 `devices`。只有视频号的录音兜底要 `audio-capture`，`interaction: foreground`。
7. **本机微信 3.8.8.19。** 从文档只能得出：读本地库的工具都只支持 4.x（调研 11（同组笔记 `11-subscriptions-compat-and-wx-cli.md`，不在仓库里）），3.8 用不上；3.8 是 AppKit 界面，无障碍树大概率还读得到（调研 10（同组笔记 `10-desktop-and-android-agents.md`，不在仓库里），推断）。wx_channels_download 有 macOS 包，但文档没写最低微信版本，近两月的修复都针对 4.1、4.2（[v261106](https://github.com/ltaoo/wx_channels_download/releases/tag/v261106)、[#597](https://github.com/ltaoo/wx_channels_download/pull/597)）。3.8.8.19 里有没有视频号的「赞和收藏」、分享菜单有没有「复制链接」，只有你能看。

<a id="2-小红书"></a>
## 2. 小红书

<a id="21-列出你存过的"></a>
### 2.1 列出你存过的

- **官方导出**：没有。[隐私政策](https://agree.xiaohongshu.com/h5/terms/ZXXY20220509001/-1)只列 App 内查看「笔记/收藏/赞过」，其他按邮件申请（official-export 笔记（同组笔记 `official-export-findings.md`，不在仓库里））。可以发邮件试，结果不能当计划。
- **日常浏览器里的脚本**（`browser-session`，低到中）：[wkwunju/xhs-vault](https://github.com/wkwunju/xhs-vault)（MIT，2026-08-18）的 `userscript/xhs-export.user.js` 改写 `XMLHttpRequest`，只截获收藏页自己发出的 `collect`、`board` 请求，取出 note id、`xsec_token`、标题；另有一个按钮自动滚到底。自动滚动属于节奏异常，去掉，改成你自己往下翻。「赞过」页同理，接口名待核实。这条也不是零风险：小红书网页会对数据采集类扩展弹「有害插件」提示（[XHS-Downloader #283](https://github.com/JoeanAmier/XHS-Downloader/issues/283)），经扩展驱动用户自己已登录的 Chrome 读小红书也被判过违规（[OpenCLI #842](https://github.com/jackwener/OpenCLI/issues/842)）。只截获、不发额外请求的油猴脚本没有找到处罚报告。
- **自动化浏览器**（`browser-session`，高）：Playwright 打开要登录的页面返回 300012（[web-image-sourcing](https://github.com/danzey0425-lgtm/web-image-sourcing)）；[xiaohongshu-mcp](https://github.com/xpzouying/xiaohongshu-mcp) 的用户一两天内被警告、封号（[#720](https://github.com/xpzouying/xiaohongshu-mcp/issues/720)、[#726](https://github.com/xpzouying/xiaohongshu-mcp/issues/726)、[#728](https://github.com/xpzouying/xiaohongshu-mcp/issues/728)），10 月还有搜索被限（[#877](https://github.com/xpzouying/xiaohongshu-mcp/issues/877)）。它的「收藏」工具是收藏动作，不是列清单。
- **本地签名**（`private-api`，高）：[redbook](https://github.com/lucasygu/redbook)、[MediaCrawler](https://github.com/NanmiCoder/MediaCrawler)、[Spider_XHS](https://github.com/cv-cat/Spider_XHS)，签名库是 [xhshow](https://github.com/Cloxl/xhshow)，有永久封禁记录（[MediaCrawler #865](https://github.com/NanmiCoder/MediaCrawler/issues/865)）。
- **翻页**：收藏页是游标翻页，每页约 20–30 条（推断）。`xsec_token` 只在会话内有效，有项目实测几小时后链接 404（06 [§1.3](06-platform-access-and-account-risk.md#13-混合路径视觉层拿链接公开页面取内容)），所以清单交来后要在当天取完。

<a id="22-取单条"></a>
### 2.2 取单条

- **页面**：`https://www.xiaohongshu.com/explore/<id>?xsec_token=…`（`discovery/item/`、`user/profile/<作者>/<id>` 同样），HTML 里的 `window.__INITIAL_STATE__` 有标题、正文、图片列表、视频流、封面、作者、发布时间、标签、互动数。解析前要把 JS 的 `undefined`、`new Map([])`、`new Set([])` 换成 JSON；`new Set` 是 10-15 前后新出现的，XHS-Downloader 当时整批失败（[#497](https://github.com/JoeanAmier/XHS-Downloader/issues/497)）。
- **这个月的变化**：[#495](https://github.com/JoeanAmier/XHS-Downloader/issues/495)（10-14）实测「长链加空 cookie」被重定向到首页，「长链加未登录访客 cookie」成功；短链（`xhslink.cn`）跳转时 cookie 没带上会失败，维护者说只有部分链接这样。[#499](https://github.com/JoeanAmier/XHS-Downloader/issues/499)（10-22）也是设了 cookie 才取到。[ParseHub](https://github.com/z-mio/ParseHub) 10-14 给小红书加了游客 session 和移动端 UA 回退，10-27 适配了新的图片 CDN 地址；它取游客 session 要用 xhshow 签名，属 `private-api`，不照搬。
- **要登录的部分**：没有 cookie 时视频只有低清（XHS-Downloader README）；评论要调签名接口，第一版不做。
- **分享链接、不登录**：手机分享出的短链展开后带跟踪参数。#305 的用户没填 cookie、批量取约 108 条后，复制链接的账号被下线。哪些参数能去掉、去掉后还取不取得到，要你实测（[§2.5](#25-测试)）。

<a id="23-基础与打包"></a>
### 2.3 基础与打包

| 项目 | 许可证 / 语言 | 最近推送 | 用法 |
|---|---|---|---|
| [wkwunju/xhs-vault](https://github.com/wkwunju/xhs-vault) | MIT / JS、Python | 2026-08-18 | fork 油猴脚本：留 XHR 截获，删自动滚动和导出 `cookies.json`，改成 `POST /v1/batches`；`scraper/fetch.py` 是取单条的参照 |
| [chenjin-cmd/xhs-account-teardown](https://github.com/chenjin-cmd/xhs-account-teardown) | MIT / Python | 2026-09-29 | 单条脚本的参照 |
| [z-mio/ParseHub](https://github.com/z-mio/ParseHub) `provider_api/xhs.py` | MIT / Python | 2026-11-09 | 读它的 `new Set` 处理、CDN 地址换算、移动端回退；不用签名部分 |
| [JoeanAmier/XHS-Downloader](https://github.com/JoeanAmier/XHS-Downloader) | GPL-3.0 / Python | 2026-11-07 | 兜底：你自装，插件经 HTTP 调；[#502](https://github.com/JoeanAmier/XHS-Downloader/pull/502) 正在加扫码登录和「风控暂停」 |

清单：`platforms: [xiaohongshu]`，`match: ["www.xiaohongshu.com/explore/*", "www.xiaohongshu.com/discovery/item/*", "www.xiaohongshu.com/user/profile/*", "xhslink.com/*", "xhslink.cn/*"]`，`access: public-web`，`fetch` 的 `outputs: [text, images, media:video, meta]`。`canonicalize` 取 24 位十六进制 note id 作 `contentId`，`identity.url` 写 `https://www.xiaohongshu.com/explore/<id>`，`resolvedUrl` 保留 `xsec_token`（[协议 §3.5](../plugin-protocol.md#35-canonicalize-与身份)）。看到重定向到首页或登录页回 `login-required`（访客 cookie 过期），看到验证页或风控 JSON 回 `blocked`，`xsec_token` 失效回 `not-found` 加 `reason: link-expired`。

<a id="24-推荐与工作清单"></a>
### 2.4 推荐与工作清单

推荐：油猴脚本交清单，插件用访客 cookie 取公开页。兜底：XHS-Downloader 包装插件。三个月后最可能变的是 SSR 字段和访客 cookie 的要求，所以解析放在一个文件里，字段缺了回 `upstream-changed`。

1. plugin-kit 平台列表加 `xiaohongshu`：`contentId` 正则 `^[0-9a-f]{24}$`，加 `testdata/canonicalize.json`。
2. `canonicalize`：四种长链，加短链展开（只跟重定向，不带 cookie）。
3. `fetch`：GET 长链，先不带 cookie；被重定向到首页时，若设置里有访客 cookie 就带上重试一次，没有就回 `login-required`，`hint` 写「请在设置里填访客 cookie」。这一步不等第 7 步，现行协议就能跑。用普通桌面 UA；清洗后解析 `__INITIAL_STATE__`；图片、视频写进 `--out`；没有视频时 `missing: [media:video]`。
4. 错误码映射和 `health`（`--probe` 取一篇固定的公开笔记）。
5. 用保存下来的 HTML 做解析测试，含 `undefined`、`new Map`、`new Set`。
6. fork xhs-vault 的油猴脚本，按 [协议 §9.4](../plugin-protocol.md#94-浏览器扩展与油猴脚本) 交 `/v1/batches`。
7. 宿主：访客 cookie 窗口（[§1](#1-共用宿主要提供的) 第 3 条）和 `credential: visitor` 的协议条目；落地后插件从设置改成读宿主传的访客 cookie。

<a id="25-测试"></a>
### 2.5 测试

- **不用账号能测**：`canonicalize` 全部；解析器对保存的 HTML；在一个从没登录过的浏览器里打开公开笔记页，拿访客 cookie 跑 `fetch` 冒烟，每次几条、间隔十几秒。注意开发机和你的账号可能同一 IP（推断有关联）。
- **只有你能测**：收藏页、赞过页的截获和完成判定；手机分享链接去掉哪些参数仍可取；取一批（几十条）后账号有没有提示；邮件申请导出的结果。

<a id="3-视频号"></a>
## 3. 视频号

<a id="31-列出你存过的"></a>
### 3.1 列出你存过的

- **官方导出**：没有。微信隐私保护指引 5.1.10 的导出不含收藏，视频号「互动历史」能查看关注、点赞、收藏，没有导出（official-export 笔记（同组笔记 `official-export-findings.md`，不在仓库里））。
- **Hook 客户端**（`client-hook`，很高）：[ltaoo/wx_channels_download](https://github.com/ltaoo/wx_channels_download)（MIT 加 Commons Clause，Go，macOS 和 Windows 都有包）装根证书、设系统代理、改写微信里的视频号页面。它的 MCP 有 `get_wxchannels_interacted_videos`（赞过或收藏）、`get_wxchannels_play_history`、`get_wxchannels_live_replays`、`get_wxchannels_video_comments`，要先在微信里打开视频号页面（[commands.md](https://github.com/ltaoo/wx_channels_download/blob/main/docs/feature/mcp/commands.md)）。[nobiyou/wx_channel](https://github.com/nobiyou/wx_channel)（MIT）有收藏、点赞批量下载，但只有 Windows 版（macOS 支持还是未合并的 [#119](https://github.com/nobiyou/wx_channel/pull/119)）。
- **这个月的状态**：11-05 起两家都在微信 4.2.0.x 上失去下载按钮或注入失败（ltaoo [#594](https://github.com/ltaoo/wx_channels_download/issues/594)、[#596](https://github.com/ltaoo/wx_channels_download/issues/596)，11-06 修复后又有 [#603](https://github.com/ltaoo/wx_channels_download/issues/603)、[#604](https://github.com/ltaoo/wx_channels_download/issues/604) 未关；nobiyou [#127](https://github.com/nobiyou/wx_channel/issues/127)、[#131](https://github.com/nobiyou/wx_channel/issues/131)、[#133](https://github.com/nobiyou/wx_channel/issues/133)）。另有两个安全问题：根证书私钥随仓库公开分发（[#586](https://github.com/ltaoo/wx_channels_download/issues/586)），macOS 异常退出后系统代理残留、整机断网（[#575](https://github.com/ltaoo/wx_channels_download/issues/575)）。
- **读界面**（`vision-ui`，微信记中）：[微信软件许可及服务协议](https://weixin.qq.com/agreement?lang=zh_CN) 8.2.1.6 明文禁止；收藏页的视频号条目能否复制出链接待核实（调研 10（同组笔记 `10-desktop-and-android-agents.md`，不在仓库里））。

结论：自动列出你的赞和收藏，现在做不到可靠。最不坏的办法是手机上逐条「分享 → 复制链接」，经监听文件夹交给电脑。

<a id="32-取单条"></a>
### 3.2 取单条

- **不登录**：`https://weixin.qq.com/sph/<短码>` 对应 `POST channels.weixin.qq.com/finder-preview/api/feed/get_feed_info`，body 带 `shortUri`，只带 Origin、Referer。2026-11-01 在浏览器里打开 4 条分享链接，页面的匿名请求都返回了作者、描述、封面和互动数，没有视频地址，图集为空（[06 §18](06-platform-access-and-account-risk.md#18-待核实清单)）。media-parser 的文档 08-30 版说匿名有标题，09-29 版说只剩作者和封面（[690118db](https://github.com/ucmao/media-parser/commit/690118db)、[7d498406](https://github.com/ucmao/media-parser/commit/7d498406)；share-entry 笔记（同组笔记 `share-entry-favorites-sph-nonhook.md`，不在仓库里））。所以不登录能拿到的是元数据和封面，描述可能随时收紧。
- **借元宝**（`session-api`，风险在元宝账号）：带元宝 cookie `POST yuanbao.tencent.com/api/weixin/get_parse_result`，body `{"type":"video_channel_url","url":<分享链接>,"scene":1}`，返回 `playable_url`，取出其中的 `token` 和 `eid`；再匿名 `get_feed_info`（`baseReq.generalToken` 加 `exportId`），拿到 h264/h265 `videoUrl`、`coverUrl`、`description`、作者昵称。[ParseHub `wxchannels.py`](https://github.com/z-mio/ParseHub/blob/master/src/parsehub/provider_api/wxchannels.py)（MIT，10-10 加入；10-30 还在修视频号时长的 bug [#200](https://github.com/z-mio/ParseHub/issues/200)，说明这条路本月有人在用）只用 cookie、Origin、Referer 就在页外完成，可以回答协议 [§2.5](../plugin-protocol.md#25-登录站点-sites) 里「只用 cookie 在页外能不能换到」那一问：代码层面能，实测要你来。ltaoo 的 Cloudflare worker 另外写死了 `t-userid`、`x-device-id`、`x-hy92`、`x-hy93` 几个请求头（share-entry 笔记（同组笔记 `share-entry-favorites-sph-nonhook.md`，不在仓库里））；先照 ParseHub 不带，失败再看是否要补。元宝账号有被封的先例，是维护者公开的解析服务被大量调用（[ltaoo #495](https://github.com/ltaoo/wx_channels_download/issues/495)）。
- **身份**：短码每次分享可能不同（推断）。`contentId` 用 feed 返回的 `exportId`（或 objectId），短码当别名；匿名响应里有没有这个字段要先测。
- **评论**：只有 Hook 工具能取，第一版不做。

<a id="33-基础与打包"></a>
### 3.3 基础与打包

| 项目 | 许可证 / 语言 | 最近推送 | 用法 |
|---|---|---|---|
| [z-mio/ParseHub](https://github.com/z-mio/ParseHub) | MIT / Python | 2026-11-09 | 移植 `wxchannels.py`，约 120 行 |
| [joeseesun/qiaomu-clipper](https://github.com/joeseesun/qiaomu-clipper) | MIT / TypeScript | 2026-11-09 | 参照：元宝页内同源请求、登录态不出浏览器的写法（`src/utils/channels-study.ts`） |
| [ltaoo/wx_channels_download](https://github.com/ltaoo/wx_channels_download) | MIT + Commons Clause / Go | 2026-11-10 | 单独一个插件，`access: client-hook`，你自装，按 [协议 §9.3](../plugin-protocol.md#93-包装-mcp-服务) 连它的本机 MCP（`127.0.0.1:2022/mcp`），实现 `list`；只调读取工具 |

清单：`platforms: [wechat-channels]`，`match: ["weixin.qq.com/sph/*", "channels.weixin.qq.com/finder-preview/pages/*"]`。`fetch` 顶层 `login: optional`，`sites` 写 `yuanbao.tencent.com`、`credential: cookies`；没登录元宝时只给元数据，回 `completeness: partial`、`missing: [media:video]`、`missingReason`「分享页没有给出视频地址」。

<a id="34-推荐与工作清单"></a>
### 3.4 推荐与工作清单

推荐：分享链接加匿名元数据，元宝换视频地址做可选的一步。兜底：没有视频地址的条目交给转写；你在 Mac 上播放时录声音（`audio-capture`），见 [06 §17.3](06-platform-access-and-account-risk.md#173-首批自己适配的插件)。Hook 插件放最后，界面写清它要装根证书、改系统代理。

1. plugin-kit 平台列表加 `wechat-channels`，定 `contentId` 格式。
2. `canonicalize`：两种链接，取短码；离线时按协议 [§3.1](../plugin-protocol.md#31-运行器与节奏) 退到 `web:` 身份。
3. `fetch` 匿名段：`get_feed_info`（`shortUri`），写封面和描述。
4. `fetch` 元宝段：有凭据时换 token，下载 MP4；`401` 或错误码 20000 回 `login-required`、`site: yuanbao.tencent.com`。
5. 宿主：元宝登录窗口（Tauri，`credential: cookies`）。
6. 第二个插件：wx_channels_download 的 MCP 转接，`sources` 给「赞和收藏」「播放记录」。

<a id="35-测试"></a>
### 3.5 测试

- **不用账号能测**：`canonicalize`；匿名 `get_feed_info` 用公开文章里嵌的分享链接；用录下的 JSON 测解析和错误映射。
- **只有你能测**：手机视频号分享菜单里有没有「复制链接」、复制出来是不是 `weixin.qq.com/sph/` 链接（最不坏的那条路靠它）；元宝换 token；元宝用微信登录时风控是否与微信号共用（[06 §18](06-platform-access-and-account-risk.md#18-待核实清单)）；3.8.8.19 里有没有「赞和收藏」、能不能复制链接；wx_channels_download 在 3.8.8.19 上能不能注入。

<a id="4-b站"></a>
## 4. B站

<a id="41-列出你存过的"></a>
### 4.1 列出你存过的

- **官方导出**：[隐私政策](https://www.bilibili.com/blackboard/privacy-pc.html)五.2 有「个人信息下载」（【我的】-【设置】-【个人信息收集清单】），包里有什么没写（official-export 笔记（同组笔记 `official-export-findings.md`，不在仓库里）；06 的覆盖矩阵当时记「未找到」，以这条为准）。值得你申请一次，看有没有收藏夹。
- **带 cookie、不签名**（`session-api`，中）：收藏夹 `x/v3/fav/folder/created/list-all?up_mid=`、`x/v3/fav/resource/list?media_id=&pn=&ps=`，稍后再看 `x/v2/history/toview/web`（[yutto `api/space.py`](https://github.com/yutto-dev/yutto/blob/main/src/yutto/api/space.py)）。按页码翻页，`ps` 上限约 20（推断）。[PiliPlus](https://github.com/bggRGjQaUbCoE/PiliPlus) 给稍后再看加了 WBI，[HistoryHub](https://github.com/LifeArchiveProject/HistoryHub) 没加，不签名能否一直可用要实测（favorites-inbox 笔记（同组笔记 `favorites-inbox-similar-findings.md`，不在仓库里））。不要像 yutto 那样再调 `space/wbi/acc/info` 取用户名，那一步要签名（bilibili-video-verify 笔记（同组笔记 `bilibili-video-verify-corrections.md`，不在仓库里））。
- **点赞**：没有找到列出自己点赞视频的现成工具；空间页有「最近点赞的视频」，接口和条数上限未核实，第一版不做。
- **公开收藏夹**：收藏夹设为公开时，不带 cookie 可能也列得出（推断，要测）。
- **日常浏览器扩展**（`browser-session`）：[ayasa520/bilibili-favorites-exporter](https://github.com/ayasa520/bilibili-favorites-exporter)（MIT，2025-09 后没更新）导出 JSON。
- **平台会做什么**：带登录态批量下载的账号有长期风控，[bili-sync #756](https://github.com/amtoaer/bili-sync/issues/756)（10-20）主账号半年下载都提示风控；一例大会员被冻结（[yutto #452](https://github.com/yutto-dev/yutto/issues/452)，因果不明）；缺指纹参数的部分接口返回 412（[BiliNote #397](https://github.com/JefferyHcool/BiliNote/issues/397)）。2026 年的律师函针对签名、登录态、付费内容和弹幕反查（[06 §16.3](06-platform-access-and-account-risk.md#163-下架停更与法律风险)）。只列清单、不批量下高清视频，风险最小。

<a id="42-取单条"></a>
### 4.2 取单条

- **不登录**：`x/web-interface/view?bvid=` 给标题、简介、作者、发布时间、封面、分 P；`x/player/playurl`（不签名）给低清音视频，要 ffmpeg 合并（[yutto `api/ugc_video.py`](https://github.com/yutto-dev/yutto/blob/main/src/yutto/api/ugc_video.py)）。
- **要登录**：高清；字幕。yutto 不签名调 `x/player/wbi/v2` 取字幕，带 cookie 才有，标 `need_login_subtitle` 的不登录拿不到。拿不到字幕时文字交给转写插件。
- **评论、弹幕**：第一版不做。
- **分享链接、不登录**：`b23.tv` 短链展开后是普通视频页，能拿到上面「不登录」的全部。

<a id="43-基础与打包"></a>
### 4.3 基础与打包

| 项目 | 许可证 / 语言 | 最近推送 | 用法 |
|---|---|---|---|
| [yutto-dev/yutto](https://github.com/yutto-dev/yutto) | GPL-3.0 / Python | 2026-11-09 | 只读，看接口和参数，不抄代码；也可做你自装的包装插件 |
| [amtoaer/bili-sync](https://github.com/amtoaer/bili-sync) | MIT / Rust | 2026-09-10 | 可借收藏夹请求代码（`fav_time` 的处理在 `crates/bili_sync/src/utils/convert.rs`，`DedeUserID` 在 `bilibili/credential.rs`）；它会签名、按网页流程刷新 cookie，这两部分不要 |
| [ayasa520/bilibili-favorites-exporter](https://github.com/ayasa520/bilibili-favorites-exporter) | MIT / JS | 2025-09-09 | 兜底：扩展导出 JSON，写导入插件读 |

清单：`platforms: [bilibili]`，`match: ["www.bilibili.com/video/*", "m.bilibili.com/video/*", "b23.tv/*"]`；`fetch` 记 `session-api`、`login: optional`，`list` 记 `session-api`、`login: required`、`order: newest-first`；`sites` 写 `www.bilibili.com`、`credential: cookies`、`domains: [".bilibili.com"]`，登录判定 cookie `SESSDATA`。插件不调 cookie 刷新接口（[协议 §6](../plugin-protocol.md#6-凭据)）。`contentId` 用 BV 号，分 P 写 `BV…:p2`。

<a id="44-推荐与工作清单"></a>
### 4.4 推荐与工作清单

推荐：自己写一个小插件，`list` 调不签名的收藏夹、稍后再看接口，`fetch` 调 `view` 和 `playurl`，宿主默认 `--want text,meta` 加封面，视频按需要再取。兜底：扩展导出 JSON 加导入插件；再不行用官方的个人信息下载。

1. `canonicalize`：BV、av、`b23.tv`、分 P 参数。
2. `sources`：列你建的收藏夹和稍后再看；`mid` 取自 cookie `DedeUserID`，不调签名接口。
3. `list`：一页一调，`actedAt` 用收藏时间 `fav_time`。
4. `fetch`：`view` 加 `playurl`；有 cookie 时取字幕；返回码映射（推断，照抓到的响应核对）：`-412`、`-352` 回 `blocked`，`-101`（未登录）回 `login-required`。
5. 宿主：B站登录窗口（扫码），`credential: cookies`。
6. 测试：录下的 JSON 测解析；公开视频做冒烟。

<a id="45-测试"></a>
### 4.5 测试

- **不用账号能测**：`canonicalize`；公开视频的 `view`、低清 `playurl`、下载合并；公开收藏夹能否匿名列出。
- **只有你能测**：私密收藏夹和稍后再看、字幕、高清；连续列几百条后有没有 `-352`；个人信息下载包里有什么。
