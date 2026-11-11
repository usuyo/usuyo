# 国内平台官方导出与导出文件导入器：跨条目发现（2026-10-30）

写给同组调研 session 和发起 session。这里只有事实和判断依据，不是指令。27 条画像和 15 条 lead 已通过 StructuredOutput 交回工作流；副本：
/private/tmp/claude-501/-Users-mac-WorkSpace-desktop-Project/7133807a-d653-486b-80df-d8e2af41491d/scratchpad/offexport/entries.json

## 各平台官方导出（隐私政策原文，2026-10-30 读取）

| 平台 | 有无 | 入口与方式 | 时限 | 包内容 | 来源 |
|---|---|---|---|---|---|
| 抖音 | 有 | 【我】-【≡】-【设置】-【个人信息管理】申请下载，填邮箱收链接；超出范围找【我的客服】 | 未写 | 政策不列类别；收藏、喜欢、观看历史是否在内未找到可靠来源 | 隐私政策 3.3（2026-09-11 更新、09-18 生效）www.douyin.com/privacy/ ，同组 platform-rules 笔记 |
| 快手 | 有 | 【≡】-【设置】-【使用信息管理】-【个人信息查阅与下载设备授权】-【个人信息下载】（原文如此；基本功能版写【个人信息管理】-【个人信息查阅与下载】-【个人信息下载】）；转移需联系 | 未写 | 未列 | 《快手隐私政策》四（一）（2026-06-11）app.m.kuaishou.com/public/index.html#/protocol/1752 |
| 小红书 | 无自助下载 | 四.2 只列 App 内查看（含【我-笔记/收藏/赞过】）；四.4 转移、四.7 其他信息按第十条发邮件 | 未写 | — | 《小红书用户隐私政策》（2026-09-20 更新、09-27 生效）agree.xiaohongshu.com/h5/terms/ZXXY20220509001/-1 |
| B站 | 有 | 【我的】-【设置】-【个人信息收集清单】-【个人信息下载】；超范围或转移按第十条联系，转移可收必要费用 | 未写 | 未列 | 隐私政策 五.2（2026-02-12 更新、02-19 生效）www.bilibili.com/blackboard/privacy-pc.html |
| 微博 | 有 | 【我】-【设置】-【隐私设置】-【个人信息与权限】-【个人信息下载】，邮件发送 | 未写 | 未列 | privacy.weibo.com/policy 六.1.3（2025-06-15）；weibo.com/signup/v5/privacy 仍是 2022-03-03 旧版，没有这条 |
| 知乎 | 有，要人工 | 【我的 → 设置 → 隐私中心 → 个人信息下载】页面联系「知乎小管家」后申请 | 「响应您的上述请求」：十五个工作日 | 未列；收藏、浏览、关注记录在「管理浏览、关注和收藏记录」一节只能查看管理 | 个人信息保护指引第 5.2 版「获取您的个人信息」一节（2025-10-10 更新、10-15 生效；页面不显示小节编号）www.zhihu.com/term/privacy |
| 豆瓣 | 有 | App【设置】-【个人信息管理】-【个人信息下载】，发到指定邮箱 | 六（六）：15 个工作日 | 未列 | 个人信息保护政策 六（五）（2026-07-02 更新、07-09 生效）www.douban.com/about/privacy |
| 今日头条 | 有 | 【我的】-【设置】-【隐私设置】-【个人信息下载】，填邮箱收下载链接 | 其他请求十五个工作日 | 未列 | 隐私政策 7.2.2（2026-10-14 更新、10-21 生效）www.toutiao.com/privacy_protection/ |
| 微信 | 有，不含收藏 | 我→设置→个人信息与权限→个人信息浏览与导出→导出个人信息，副本发邮箱；视频号账号可单独导出（5.1.9，类别未列） | 未写 | 收藏只能查阅（5.1.5）；视频号「互动历史」可查阅关注、点赞、收藏（星标）的视频，同样没有导出。IT之家 2021-11-08：国内版只导出头像、名字、微信号、地区、手机号、登录设备等资料；海外版 WeChat 导出含通讯录、朋友圈、收藏功能数据 | 隐私保护指引 5.1.10（2026-08-17）；www.ithome.com/0/585/480.htm |
| 小宇宙 | 无 | 政策五没有下载或副本入口；App 设置可导出订阅 OPML | — | OPML 只有订阅节目的 RSS 地址 | 隐私政策（2022-09-01）post.xiaoyuzhoufm.com/podcast-privacy/ ；OPML 见 haroldgao.substack.com/p/opml（第三方） |
| 喜马拉雅 | 无自助 | 联系客服获取副本 | 未写 | 基本资料、身份信息 | 隐私政策 七（四）（2026-09-02 更新、09-09 生效）passport.ximalaya.com/page/privacy_policy |
| TikTok（对照） | 有 | 「下载你的数据」JSON 或 TXT；Data Portability API 要开发者审核约 3–4 周 | DPA 数据准备好后可下载 4 天 | DPA 数据类型页：喜欢和收藏只在 Full Archive，Activity 只有观看历史 | developers.tiktok.com/doc/data-portability-api-get-started（2026-09-10）、/doc/data-portability-data-types（2026-08-28） |

国内平台导出包的内容和格式，在少数派、知乎、V2EX、掘金、B站专栏方向的搜索里都没找到带目录或字段的实测帖。唯一一篇描述抖音包的是 80tg.com 2026-06-22 的营销站文章（称「加密 ZIP、短信验证、JSON 与 PDF」），文中所谓解压密码示例就是该公司的电话号码，不可信。

## 抖音档案：字段级确认不了

- DouyinDataReview 的 src/domain/personalArchiveParser.ts 第 22–55 行给三类记录各列 8–9 个中英文别名，字段也靠别名表猜；ZIP 里只读以 `{` 或 `[` 开头的 .json/.txt，拒绝加密 ZIP（提示「请先在本机解密」）、ZIP64、超过 250 个条目、压缩比超过 250 倍、32 MB 以上。
- test-fixtures/annual-archive.json 是应用自己的 schemaVersion 2 格式；测试里的「官方档案」样例是合成的。引入解析器的 PR #1（2026-08-11）没写官方包结构；仓库 215 个 issue 全是作者自己的 PR。
- 缺的样例：真实包的脱敏目录树（文件名、扩展名、编码）、各类文件的键名和一条去掉内容的结构样例、是否含收藏/喜欢/观看历史及时间范围、时间格式、给视频 ID 还是分享链接、ZIP 是否加密及密码给法、链接有效期。

## TikTok 导出：可以确认到字段级（与抖音对照）

- 2023-10 至 2024-12：顶层 `Activity` → `Like List.ItemFavoriteList`、`Favorite Videos.FavoriteVideoList`、`Video Browsing History.VideoList`，条目 `{Date, Link}`（john00-dev/tiktok-liked-content 的 validator.py、zollo 的 data_structure.yml）。
- 之后改为 `Likes and Favorites`、`Your Activity.Watch History.VideoList`（samirelanduk/tiktok-save 的 save.py；joeycato/tiktok-favesave 先找 Your Activity 再退到 Likes and Favorites）。
- 同一份里键名大小写不统一：喜欢列表 `link`、收藏列表 `Link`（commitmaniac/tdp）。TXT 导出的文件名随账号语言翻译，例如荷兰语 Likelijst.txt（d3i-infra/data-donation-task 的 tiktok.py）。

## 对同组笔记的更正和补充

- TikTok Data Portability API 有开源客户端：Inoue-AI/Inoue-AI-TikTok-SDK（MIT）和 Nico-AP/ddcs-app（无许可证）。入门页 2026-09-10 版没写 EEA/英国限制，数据类型页只写「并非所有地区都有全部功能」，地区限制待核实。
- 微博个人信息下载的官方出处是 privacy.weibo.com/policy，不是 open-im。
- 小红书政策里没有「个人信息下载」入口，这与抖音、快手、B站、微博、豆瓣、头条都不同。

## timelinize 对歇存

- 29 个数据源，没有中文平台；Twitter 源不导入 like.js 和书签，Instagram 源不导入 saved_posts，收藏类数据基本没有。
- 可借鉴：timeline/datasource.go 的 FileImporter.Recognize（文件名、内容、目录结构三种识别，返回 Confidence 和 DirThreshold，不遍历目录）、包内直接读 zip/tar、重复导入跳过已有条目、HTTP API 与 CLI 对称。
- 它把「平台导出」和「读本地客户端库」（imessage、firefox、iphone、applephotos）放在同一层，歇存分层时应分开。
- AGPL-3.0：设计和格式事实可借鉴，代码不能进 MIT 插件；它没有「只解析、输出 JSON」的命令，独立进程调用也不划算。

## 平台中立的「导入官方导出文件」插件：从这批代码归纳的要求

1. 按内容识别，不按文件名：TikTok TXT 导出的文件名随账号语言翻译（荷兰语，data-donation-task 代码注释写明）；ItsMaciek/WatchNow 的表头别名表含波兰语，推断 Takeout CSV 表头也随语言翻译（未核实）。
2. 同一平台多代结构并存，键名大小写不统一，要做别名表和大小写不敏感匹配。
3. 多次导出按链接合并去重（tiktok-liked-content 的 merger.py）；旧包能否只留 raw 不再解析，参考 karlicoss/bleanser。
4. 导出里的媒体链接会过期（Resurfly 文档：Instagram CDN 图片约 4 天、视频 1–2 天；TikTok DPA 下载 4 天），导入后尽快取或只存 ID。
5. ZIP 安全上限可参考 DouyinDataReview（条目数、压缩比、路径穿越、加密 ZIP 提示）。
6. 未识别的包给用户看目录树和键名，让用户手动映射；同时收集脱敏的目录树和键名样例，用来补中文平台的识别器。

可移植的代码（许可证允许）：dogsheep 系列（Apache-2.0）、commitmaniac/tdp（BSD-3）、dotWee/py-ig2raindrop-cli 的 instagram.py（WTFPL）、onfabric/context-use 的 providers（MIT）、meizuflux/scrollback（MIT）、dj-urg/data-mirroring（MPL-2.0，文件级）、FlyingWolFox/Netscape-Bookmarks-File-Parser（MIT）、scripting/opmlPackage（MIT）。可以独立进程调用：akosbalasko/yarle、vzhd1701/evernote-backup（支持印象笔记）、karlicoss/instapexport。只能参考：timelinize、data-donation-task、hestialabs-experiences（AGPL），savehub、hudi-packages-connectors、tiktok-liked-content、WatchNow（无许可证），tiktok-favesave（CC BY-NC-SA）。

## 开源收藏工具的导入代码位置

- Karakeep：packages/shared/import-export/parsers.ts（html、pocket、matter、omnivore、linkwarden、tab-session-manager、mymind、instapaper、readwise-reader、onetab、karakeep），apps/workers/workers/importWorker.ts。
- Linkwarden：apps/web/lib/api/controllers/migration/importFromHTMLFile.ts、importFromLinkwarden.ts、importFromOmnivore.ts、importFromPocket.ts、importFromWallabag.ts。
- wallabag：src/Import/*Import.php（Browser、Chrome、Firefox、Delicious、Elcurator、Html、Instapaper、Omnivore、Pinboard、PocketCsv、PocketHtml、Pocket API、Readability、Shaarli、Wallabag v1/v2），经 ImportInterface 和 ImportChain 注册。
- linkding：bookmarks/services/parser.py（Netscape）、importer.py、exporter.py，管理命令 bookmarks/management/commands/import_netscape.py。
