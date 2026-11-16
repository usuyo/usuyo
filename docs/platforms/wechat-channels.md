<a id="视频号获取设计"></a>
# 视频号：获取设计

文中「协议」指[插件协议](../protocol/README.md)，「06」指[调研 06](../research/06-platform-access-and-account-risk.md)。各平台的总表、要不要账号的几个说法、怎么相对安全地试，在 [platforms/README](README.md)；这里不重复。

<a id="0-结论"></a>
## 0. 结论

| 平台 | 首个实现 | 兜底 | 做不到的 |
|---|---|---|---|
| 视频号 | 分享链接 → 匿名 `get_feed_info` 取作者、描述、封面；视频地址这一步可选，借腾讯元宝的登录 cookie 换临时 token | 录下播放的声音再转写 | 自动列出「赞和收藏」只有 Hook 客户端一条路，而且 11-05 起在微信 4.2 上反复失效；本机 3.8.8.19 能不能用没有文档（[§3.1](#31-列出你存过的)） |

视频号的风险在可用性：能列清单的工具每次微信更新都要跟着修。

<a id="3-视频号"></a>
## 3. 视频号

**设备权限。** 三个首个实现都不要 `devices`。只有视频号的录音兜底要 `audio-capture`，`interaction: foreground`。

**本机微信 3.8.8.19。** 从文档只能得出：读本地库的工具都只支持 4.x（调研 11（同组笔记 `11-subscriptions-compat-and-wx-cli.md`，不在仓库里）），3.8 用不上；3.8 是 AppKit 界面，无障碍树大概率还读得到（调研 10（同组笔记 `10-desktop-and-android-agents.md`，不在仓库里），推断）。wx_channels_download 有 macOS 包，但文档没写最低微信版本，近两月的修复都针对 4.1、4.2（[v261106](https://github.com/ltaoo/wx_channels_download/releases/tag/v261106)、[#597](https://github.com/ltaoo/wx_channels_download/pull/597)）。3.8.8.19 里有没有视频号的「赞和收藏」、分享菜单有没有「复制链接」，只有你能看。


<a id="31-列出你存过的"></a>
### 3.1 列出你存过的

- **官方导出**：没有。微信隐私保护指引 5.1.10 的导出不含收藏，视频号「互动历史」能查看关注、点赞、收藏，没有导出（official-export 笔记（同组笔记 `official-export-findings.md`，不在仓库里））。
- **Hook 客户端**（`client-hook`，很高）：[ltaoo/wx_channels_download](https://github.com/ltaoo/wx_channels_download)（MIT 加 Commons Clause，Go，macOS 和 Windows 都有包）装根证书、设系统代理、改写微信里的视频号页面。它的 MCP 有 `get_wxchannels_interacted_videos`（赞过或收藏）、`get_wxchannels_play_history`、`get_wxchannels_live_replays`、`get_wxchannels_video_comments`，要先在微信里打开视频号页面（[commands.md](https://github.com/ltaoo/wx_channels_download/blob/main/docs/feature/mcp/commands.md)）。[nobiyou/wx_channel](https://github.com/nobiyou/wx_channel)（MIT）有收藏、点赞批量下载，但只有 Windows 版（macOS 支持还是未合并的 [#119](https://github.com/nobiyou/wx_channel/pull/119)）。
- **这个月的状态**：11-05 起两家都在微信 4.2.0.x 上失去下载按钮或注入失败（ltaoo [#594](https://github.com/ltaoo/wx_channels_download/issues/594)、[#596](https://github.com/ltaoo/wx_channels_download/issues/596)，11-06 修复后又有 [#603](https://github.com/ltaoo/wx_channels_download/issues/603)、[#604](https://github.com/ltaoo/wx_channels_download/issues/604) 未关；nobiyou [#127](https://github.com/nobiyou/wx_channel/issues/127)、[#131](https://github.com/nobiyou/wx_channel/issues/131)、[#133](https://github.com/nobiyou/wx_channel/issues/133)）。另有两个安全问题：根证书私钥随仓库公开分发（[#586](https://github.com/ltaoo/wx_channels_download/issues/586)），macOS 异常退出后系统代理残留、整机断网（[#575](https://github.com/ltaoo/wx_channels_download/issues/575)）。
- **读界面**（`vision-ui`，微信记中）：[微信软件许可及服务协议](https://weixin.qq.com/agreement?lang=zh_CN) 8.2.1.6 明文禁止；收藏页的视频号条目能否复制出链接待核实（调研 10（同组笔记 `10-desktop-and-android-agents.md`，不在仓库里））。

结论：自动列出你的赞和收藏，现在做不到可靠。现在的做法是在手机上逐条「分享 → 复制链接」（`weixin.qq.com/sph/<短码>`），拷到电脑上（iPhone 和 Mac 之间可以用通用剪贴板），在导入页「粘贴链接」里一次贴进多条：一行一条、夹在分享文字里、几条连在一起的都认，重复的合并，认不出的列在预览里；确认后按清单的节奏（`interval: 30s`、`daily: 50`）一条条取（[SPEC 本机接口](../spec/local-api.md#本机接口第-2-层11-11-照代码) 的 `POST /imports/links`）。

<a id="32-取单条"></a>
### 3.2 取单条

- **不登录**：`https://weixin.qq.com/sph/<短码>` 对应 `POST channels.weixin.qq.com/finder-preview/api/feed/get_feed_info`，body 是 `{"baseReq":{"generalToken":""},"shortUri":"<短码>"}`，只带 Origin、Referer。2026-11-12 不带账号实测：回作者、头像、描述、封面、发布时间（`createtime`）和四个互动数；缺 `baseReq` 回 `{"error":{"name":"ServerError"}}`，缺 Origin、Referer 回 `permission verification failed`；回答里没有固定的 feed id（`sceneInfo.dynamicExportId` 每次不同），条目的身份用短码。2026-11-01 在浏览器里打开 4 条分享链接，页面的匿名请求都返回了作者、描述、封面和互动数，没有视频地址，图集为空（[06 §18](../research/06-platform-access-and-account-risk.md#18-待核实清单)）。media-parser 的文档 08-30 版说匿名有标题，09-29 版说只剩作者和封面（[690118db](https://github.com/ucmao/media-parser/commit/690118db)、[7d498406](https://github.com/ucmao/media-parser/commit/7d498406)；share-entry 笔记（同组笔记 `share-entry-favorites-sph-nonhook.md`，不在仓库里））。所以不登录能拿到的是元数据和封面，描述可能随时收紧。
- **借元宝**（`session-api`，风险在元宝账号；登录怎么认见 [§3.4](#34-插件做成的样子)）：带元宝 cookie `POST yuanbao.tencent.com/api/weixin/get_parse_result`，body `{"type":"video_channel_url","url":<分享链接>,"scene":1}`，返回 `playable_url`，取出其中的 `token` 和 `eid`；再匿名 `get_feed_info`（`baseReq.generalToken` 加 `exportId`），拿到 h264/h265 `videoUrl`、`coverUrl`、`description`、作者昵称。[ParseHub `wxchannels.py`](https://github.com/z-mio/ParseHub/blob/master/src/parsehub/provider_api/wxchannels.py)（MIT，10-10 加入；10-30 还在修视频号时长的 bug [#200](https://github.com/z-mio/ParseHub/issues/200)，说明这条路本月有人在用）只用 cookie、Origin、Referer 就在页外完成，可以回答协议 [§2.5](../protocol/manifest.md#25-登录站点-sites) 里「只用 cookie 在页外能不能换到」那一问：代码层面能，实测要你来。ltaoo 的 Cloudflare worker 另外写死了 `t-userid`、`x-device-id`、`x-hy92`、`x-hy93` 几个请求头（share-entry 笔记（同组笔记 `share-entry-favorites-sph-nonhook.md`，不在仓库里））；先照 ParseHub 不带，失败再看是否要补。元宝账号有被封的先例，是维护者公开的解析服务被大量调用（[ltaoo #495](https://github.com/ltaoo/wx_channels_download/issues/495)）。
- **身份**：匿名回答里没有固定的 feed id（上面 11-12 的实测），所以 `contentId` 是 `sph-<短码>`，登没登录元宝都一样。同一条视频分享两次得到的短码是否相同没核；不同的话，库里会是两条。
- **评论**：只有 Hook 工具能取，不取。

<a id="33-基础与打包"></a>
### 3.3 基础与打包

| 项目 | 许可证 / 语言 | 最近推送 | 用法 |
|---|---|---|---|
| [z-mio/ParseHub](https://github.com/z-mio/ParseHub) | MIT / Python | 2026-11-09 | 移植 `wxchannels.py`，约 120 行 |
| [joeseesun/qiaomu-clipper](https://github.com/joeseesun/qiaomu-clipper) | MIT / TypeScript | 2026-11-09 | 参照：元宝页内同源请求、登录态不出浏览器的写法（`src/utils/channels-study.ts`） |
| [ltaoo/wx_channels_download](https://github.com/ltaoo/wx_channels_download) | MIT + Commons Clause / Go | 2026-11-10 | 只当参考：不包装、不调用它和它的 MCP（[BUILD 011](../BUILD.md#011-更多平台)）；改客户端的那一类要写也是我们自己的实现，放 `usuyo-fetch-hook` |

清单：`platforms: [wechat-channels]`，`match: ["weixin.qq.com/sph/*", "channels.weixin.qq.com/finder-preview/pages/*"]`，`sites` 写 `yuanbao.tencent.com`、`credential: cookies`。取单条是两种方法：`feed-info`（`public-web`、不用账号，交文字、封面和元数据，视频总记在 `missing`，`missingReason`「分享页没有给出视频地址」）和 `yuanbao`（`session-api`、`account: in-usuyo`、`sites: [yuanbao.tencent.com]`，只交视频）。用哪种照宿主的获取策略（[协议 §2.3.2](../protocol/manifest.md#232-获取策略)）：默认先用 `feed-info`，缺的视频再找 `yuanbao`；选「仅不使用账号」时不调 `yuanbao`，条目是 `partial`，写「没存下来：视频（分享页没有给出视频地址）」和「按当前策略未获取：视频」。

<a id="34-插件做成的样子"></a>
### 3.4 插件做成的样子

插件在 `usuyo-fetch/wechat-channels`，照这个做法：分享链接加匿名元数据，元宝换视频地址做可选的一步。兜底：没有视频地址的条目交给转写；你在 Mac 上播放时录声音（`audio-capture`），见 [06 §17.3](../research/06-platform-access-and-account-risk.md#173-首批自己适配的插件)。改客户端的方法排在最后，现在没有。

- **`canonicalize`**：两种链接，离线取短码，不发请求；`contentId` 是 `sph-<短码>`（[§3.2](#32-取单条) 身份）。取不出短码的回 `match: false`。
- **`fetch` 匿名段**：`get_feed_info`（`shortUri`），存作者、描述和封面，另存接口的原样回答。
- **`fetch` 元宝段**：宿主交了元宝的凭据时换 token，下载 MP4 存成 `media/video.mp4`；`401` 或错误码 20000 回 `login-required`、`site: yuanbao.tencent.com`。给出的不是能直接播的 MP4 时不存那个文件，只记原因。
- **登录**：元宝的登录由宿主管（`credential: cookies`），登录窗口和从文件导入 cookies.txt 两种来源（[协议 §6](../protocol/credentials.md#6-凭据)）。登录后才有的 cookie 名没核实，两个参考项目也没写：ParseHub 把你填的整串 cookie 原样发出去（`src/parsehub/parsers/parser/wxchannels.py:17-23`），WXD 读 `.tencent.com` 下的全部 cookie（`pkg/scraper/wxchannels/client.go:33,108`），所以清单不写 `cookies`，宿主照「不写时」那条认登录（[协议 §2.5](../protocol/manifest.md#25-登录站点-sites)）：没登录时导出的文件也导得进，登录有没有用，到第一次调 `yuanbao` 才知道，元宝回 401 或错误码 20000 时插件回 `login-required`。
- 还没有的：plugin-kit 的平台列表（`wechat-channels` 的 `contentId` 格式）。

<a id="35-测试"></a>
### 3.5 测试

- **不用账号能测**：`canonicalize`；匿名 `get_feed_info` 用公开文章里嵌的分享链接（对真平台跑过的记录在 [routes.md §4](routes.md#4-对真平台跑过的记录)）；用录下的 JSON 测解析和错误映射；从 cookies.txt 经宿主交给 `yuanbao`、换 token、带 `generalToken` 取视频地址、下载，对本机的假站点测过（插件的 `tests/fetch.rs`）。
- **只有你能测**：元宝真换一次 token；元宝用微信登录时风控是否与微信号共用（[06 §18](../research/06-platform-access-and-account-risk.md#18-待核实清单)）；3.8.8.19 里有没有「赞和收藏」、能不能复制链接；wx_channels_download 在 3.8.8.19 上能不能注入。
