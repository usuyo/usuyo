<a id="各平台的获取方式"></a>
# 各平台的获取方式

每个平台一个获取插件仓库 `usuyo-fetch/<平台>`，名字用平台官方的英文名（小红书是 `rednote`）。插件里的每种获取方式都是我们自己写的实现，别的项目只当参考，不包装、不调用别人的工具，也不读别人工具的输出文件；平台官方的导出照读（用户 11-13，[回答表](../answers.md)）。每种获取方式的做法、账号、风险层级和状态，写在各插件仓库 README 开头的那张「功能 × 实现方式 × 风险层级」表里，这里不再抄一遍；这一页只放跨平台的总览、对真平台跑过的记录和还没定的事。手机上的那几种获取方式的设计在 [phone.md](phone.md)；要不要账号、怎么相对安全地试，在 [README](README.md)。

<a id="1-怎么读这一页"></a>
## 1. 怎么读这一页

- **获取方式**写的是清单里方法的 `id`（[协议 §2.3.1](../protocol/manifest.md#231-方法-methods)），层级是 [协议 §2.4](../protocol/manifest.md#24-access-与-login) 的标签；各自的说明点进插件仓库的 README。
- **状态**两种：**已实现**（插件里写好了，测试全离线；对真平台跑没跑过见[第 4 节](#4-对真平台跑过的记录)）；**未实现**（清单里列了，或只占了一个仓库）。
- 不要账号能拿到的，不带账号取（[BUILD §1](../BUILD.md#1-用户定下的做法)）。列出收藏这一类的获取方式只交链接，内容由同一插件里不要账号的获取方式去取。
- 要改客户端的获取方式（`client-hook`）放另一个组织 `usuyo-fetch-hook/<平台>`（[CLAUDE.md](../../CLAUDE.md) 原则 1），现在一个都没有。
- 现在只管取到内容，不追画质：视频取页面或接口直接给的那一份；高清视频以后作为试验性功能，在各平台插件里另写一种获取方式（用户 11-13）。

<a id="2-先做的十个"></a>
## 2. 先做的十个

主要的平台是小红书、视频号、B站、X、抖音，其余接着做，不排先后。电脑上的获取方式先做先试，用到 iPhone 驱动的排在后面，手机上一条一条点比电脑上慢（用户 11-12）。

| # | 插件 · 获取方式 | 等什么 |
|---|---|---|
| 1 | `wechat-channels` · `userscript`（网页版的浏览器脚本） | 你在浏览器里跑一次探针脚本（[BUILD §7](../BUILD.md#7-只有用户能答的和能验的)）；照页面代码看，普通浏览器里拿不到列表 |
| 2 | `douyin` · `export`（个人信息下载） | 一份真的导出文件 |
| 3 | `douyin` 取单条：`share-page` 现在取不到（[第 4 节](#4-对真平台跑过的记录)），`h5-token` 默认关着 | 找一种不用账号、不带加密参数的做法；找不到就照现在的样子 |
| 4 | `tiktok` 取单条 | 仓库是占位，获取方式待定 |
| 5 | `weibo` · `favorites` | — |
| 6 | 视频文件：`youtube`、`zhihu`、`instagram` | 各自插件里写，不另建通用的视频插件 |
| 7 | iPhone 驱动（[phone.md §4](phone.md#4-驱动和流程分开)） | — |
| 8 | `wechat-channels` · `iphone` | 7 |
| 9 | `rednote` · `iphone` | 7 |
| 10 | `wechat` · `favorites-iphone` | 7 |

<a id="3-总览"></a>
## 3. 总览

一行一个插件，格里是那一类用哪几种获取方式、做到哪；没写的就是还没有。四类：**单条内容**是按链接取一条的文字、图和元数据；**视频文件**是那一条里的视频；**收藏等列表**是列出收藏、赞过、稍后再看、书签这类你存过的；**官方导出**是读平台官方给你的导出文件。

| 平台（仓库） | 单条内容 | 视频文件 | 收藏等列表 | 官方导出 |
|---|---|---|---|---|
| 小红书（[rednote](https://github.com/usuyo-fetch/rednote)） | `note-page` 已实现 | 随 `note-page` 一起取，不登录是低清 | `in-usuyo`、`userscript` 已实现；`iphone`、`android` 未实现 | — |
| 视频号（[wechat-channels](https://github.com/usuyo-fetch/wechat-channels)） | `feed-info` 已实现：作者、描述、封面 | `yuanbao` 已实现 | `userscript`、`iphone`、`mac`、`android` 未实现 | — |
| B站（[bilibili](https://github.com/usuyo-fetch/bilibili)） | `view` 已实现 | 随 `view` 一起取，低画质 | `favorites-public`、`favorites`、`userscript` 已实现 | — |
| X（[x](https://github.com/usuyo-fetch/x)） | `guest`、`embed` 已实现；`deliver-content` 未实现 | 随 `guest`、`embed` 一起取 | `userscript` 已实现；`session`、`iphone` 未实现 | `archive` 已实现 |
| 抖音（[douyin](https://github.com/usuyo-fetch/douyin)） | `share-page`、`h5-token` 已实现 | 随单条一起取 | `userscript` 已实现；`iphone`、`android` 未实现 | `export` 未实现 |
| TikTok（[tiktok](https://github.com/usuyo-fetch/tiktok)） | 未实现 | 未实现 | 未实现 | 未实现 |
| 微信（[wechat](https://github.com/usuyo-fetch/wechat)）：公众号文章、微信收藏 | `article` 已实现 | 文章里的视频、音频只记地址 | `favorites-iphone`、`favorites-mac` 未实现 | — |
| 快手（[kuaishou](https://github.com/usuyo-fetch/kuaishou)） | `share-page` 已实现 | 随 `share-page` 一起取 | `iphone`、`android` 未实现 | — |
| 微博（[weibo](https://github.com/usuyo-fetch/weibo)） | `status` 已实现 | 微博自带的视频随 `status` 一起取；别处的视频页未实现 | `userscript` 已实现；`favorites` 未实现 | — |
| 知乎（[zhihu](https://github.com/usuyo-fetch/zhihu)） | `page` 已实现 | 未实现 | `collection`、`collection-private` 未实现 | — |
| YouTube（[youtube](https://github.com/usuyo-fetch/youtube)） | `transcript`（字幕）未实现 | 未实现 | `channel-feed`（按创作者）未实现 | `takeout` 已实现 |
| Instagram（[instagram](https://github.com/usuyo-fetch/instagram)） | 未实现 | 未实现 | — | `export` 已实现 |
| Telegram（[telegram](https://github.com/usuyo-fetch/telegram)） | `export` 里的单条 | — | — | `export` 已实现 |
| 贴吧（[tieba](https://github.com/usuyo-fetch/tieba)） | 未实现 | — | 未实现 | 未实现 |
| Reddit（[reddit](https://github.com/usuyo-fetch/reddit)） | 未实现 | 未实现 | 未实现 | — |
| QQ（[qq](https://github.com/usuyo-fetch/qq)） | 未实现 | — | 未实现 | 未实现 |

TikTok、贴吧、Reddit、QQ 四个仓库现在只有 README 和许可证，获取方式都待定。

<a id="4-对真平台跑过的记录"></a>
## 4. 对真平台跑过的记录

都在 2026-11-12，不带账号，一个平台一次一个会话、几次请求为限（[BUILD §2](../BUILD.md#2-干活的规矩)）。带加密或签名参数的获取方式这边不跑。

| 插件 · 获取方式 | 结果 |
|---|---|
| `bilibili` · `view` | 完整存下一条（约 11 MB）；之后合成能播的文件、取关键帧、识别画面中文字也走通了 |
| `bilibili` · `favorites-public` | 列出一个公开收藏夹的第一页，20 条，有下一页；那一页按收藏时间从旧到新排，清单写 `order: unordered`。私密的收藏夹和稍后再看列不到 |
| `x` · `guest` | 经系统代理取回文字的和带图的帖子各一条 |
| `x` · `embed` | 没单独跑 |
| `wechat-channels` · `feed-info` | 作者、描述、封面、发布时间；视频记缺 |
| `kuaishou` · `share-page` | 短链、不带参数的链接、网页版分享链接 `/f/` 共四次，取到视频（一条约 57 MB）或不可用的回答，删掉的作品回 `not-found`；图文和验证页没碰到 |
| `wechat` · `article` | `/s/<短码>` 的一篇完整取到（正文转成 Markdown、52 张图、原样的页面）；长链接（`__biz`、`mid`、`idx`、`sn`）两次都落到「环境异常，完成验证后即可继续访问」 |
| `douyin` · `share-page` | 取不到：`iesdouyin.com/share/video/<id>` 的页面数据里没有作品内容，只有「请尝试在抖音内观看」；插件回 `upstream-changed`，留下页面原文 |
| `douyin` · `h5-token` | 没跑：这个环境的权限检查不放行带加密参数的请求 |
| `zhihu` · `page` | 不带 cookie 时文章页和回答页都回 403 的校验页（`zh-zse-ck`），所以改成用访客身份；带访客身份没跑 |
| `zhihu` · `collection` | 收藏夹页面只带收藏夹自己的信息；条目接口 `/api/v4/collections/<id>/items` 对不带 cookie 的请求回 401「身份未经过验证」 |
| `rednote` · `note-page`、`weibo` · `status` | 没跑：要先在应用里取一次访客身份 |

要账号、要你的文件、要手机的获取方式，只有你能试，见[一轮测试](test-round.md)。

<a id="5-客户端流量读取抓包"></a>
## 5. 客户端流量读取（抓包）

一种候选的实现方式，还没有哪个插件用。在你自己的设备上，经本机的代理读官方客户端已经收到的回答，不另发请求：客户端照常向平台请求，代理只把回答里的条目记下来交给 usuyo。用户 11-13 说过浏览器里读收藏夹效率低，倾向这一类做法（[回答表](../answers.md)）。

- **前提。** 你自己在设备上装代理的根证书，自己把系统或手机的代理设到这台电脑；这两步只能你自己做，usuyo 不替你装、不替你改设置。
- **层级还没定**（[协议 §2.4](../protocol/manifest.md#24-access-与-login)）；未实现。
- **要定的。** 证书由谁生成、私钥放在哪；代理怎么撤、异常退出后系统代理会不会留着；哪些平台的客户端校验证书、不认自己装的根证书。别的项目出过根证书私钥随仓库公开、异常退出后系统代理残留的问题（[wx_channels_download #586](https://github.com/ltaoo/wx_channels_download/issues/586)、[#575](https://github.com/ltaoo/wx_channels_download/issues/575)）。
- 浏览器脚本的那几种获取方式照留。

<a id="6-还没定的事"></a>
## 6. 还没定的事

- **按创作者主页、按关键词列**：协议还没有对应的能力（[协议 §13](../protocol/open.md#13-待定) 第 26 项）；YouTube 的 `channel-feed` 还等第 21 项。
- **抖音单条**：`share-page` 取不到以后，`h5-token` 带加密参数、默认关着，这边不对真平台跑；有没有不带加密参数的做法没找到。
- **视频号的列表**：网页版的列表只经微信桌面客户端注入的桥（`WeixinJSBridge`）出页面，普通浏览器里落到「当前版本暂不支持」（照页面代码看，2026-11-12），等探针确认；电脑微信里看不看得到「赞和收藏」没核。
- **知乎收藏夹**：条目接口不带 cookie 回 401，访客身份够不够没试。
- **X 的官方归档**里有没有书签没核。
- **贴吧**公开帖子页能不能直接读，没查。
- **高清视频**：做成试验性功能，在各平台插件里另写一种获取方式，还没开始。
- **客户端流量读取**的层级和证书、代理的做法（[第 5 节](#5-客户端流量读取抓包)）。
- **豆瓣**（你标记的书影音的公开 RSS）、**播客**（节目的公开 RSS）还没有仓库，等订阅那条线（[协议 §13](../protocol/open.md#13-待定) 第 21–23 项）。
