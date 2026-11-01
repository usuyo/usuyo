<a id="致谢候选"></a>
# 致谢候选

状态：候选 · 2026-10-30

歇存从下面这些项目学到了做法，或者计划包装、适配它们。这是以后放进 README 或关于页的候选名单，正式发布前要逐个确认许可证和署名要求。每条都注明它和歇存的关系。

<a id="怎么用这份名单"></a>
## 怎么用这份名单

每条一行，依次是项目、一句话介绍、许可证、关系、详细出处。关系分三种：

- **计划适配**：计划包装它的命令行、HTTP 接口或 MCP，或者 fork 后改成插件。获取方式只能放在第三方仓库（[ADR-003 §1](decisions/ADR-003-plugins.md#1-插件分三类)）；GPL、AGPL 工具在取得法律意见前只由用户自己安装、歇存以独立进程调用（[ADR-003 §8](decisions/ADR-003-plugins.md#8-gpl-工具)）。
- **参考设计**：借鉴它的做法、格式或协议，不用它的代码。
- **相邻项目**：同一生态里的创作、发布、录屏、自动化等工具，歇存不接入。

名单不是依赖清单。歇存代码直接依赖的库，致谢以后从 `go.mod` 和 `package.json` 生成。计划适配里有几条预计会直接编进歇存（readeck/go-readability、go-trafilatura、go-domdistiller、html-to-markdown、chromedp、kooky），编进去以后改由依赖清单致谢；[05 §4](research/05-note-apps-and-plugins.md#4-安全与隔离) 里的隔离工具也属于依赖，这里不收。

收录范围：

- [06 调研](research/06-platform-access-and-account-risk.md#调研-06--平台内容获取的开源生态与账号风险)里标「（致谢）」的 260 条，按类别放进下面各节，出处链到 06 的同名小节；用户点名的项目链到 [06 §3](research/06-platform-access-and-account-risk.md#3-用户点名的项目)。
- 用户点名、06 没标致谢的 7 条。用户说过，点名的项目都是以后的致谢候选，所以照收，按 06 里的类别和关系放进各节，限制写在各自那一行：
  - NanmiCoder/MediaCrawler：自定义的非商业学习许可，条款里没有分发授权，只借做法，不 fork、不分发、不包装。
  - ahang008/ah-skills：Skill 索引仓库，本身不从平台取内容，许可为非商业源码可见；同作者的 ah-douyin-clean-downloader 在 06 标了致谢，已收。
  - open-wxa/wxa-runner：在浏览器里运行自己的小程序、做本地调试和测试的工具，不取平台内容；内置的基础库是腾讯代码，不能分发。
  - pabatiba89-sys/kali-publish：social-auto-upload 派生的商业产品发布模块，视频号流程写死了推广话题和商品逻辑，只借登录经验；上游 social-auto-upload 已收。
  - ycccccccy/echotrace：用户写的是 `echo`，对应关系是推断；属 client-hook，已停更，作者的另一个仓库在腾讯 2026 年的 DMCA 通知里。
  - yikart/AiToEarn：用户 2026-11-01 给的视频号链接介绍的项目；发布和互动工具，不读收藏夹，互动功能是对平台账号的批量写操作，只记录。
  - google-gemini/computer-use-preview：用户 2026-11-01 给的视频号链接演示 Gemini 的 Computer Use，对应到这个参考实现是推断；截图要发到 Gemini 接口。
- [02](research/02-adapter-ecosystems.md#调研-02--社区维护的来源适配器生态) 和 [05](research/05-note-apps-and-plugins.md#调研-05--笔记软件互通与插件化) 里被当作做法来源、准备兼容或借代码的 61 条，出处链到对应小节。只在表格里出现、没被引为做法的，以及只被当作反例的，不收。这 61 个仓库 2026-10-30 用 `gh api` 确认存在，许可证以返回值为准。
- 归档、停更、删库的项目照收，状态写在一句话里。

合计 328 条：计划适配 127 条，参考设计 185 条，相邻项目 16 条。

用户给的视频号链接，按视频演示的内容推断对应 mingjuner123-spec/wechat-inbox-sync（待核实）。2026-11-01 又给了 4 条：MediaCrawler 已收；AiToEarn、computer-use-preview 新收；开源文件搜索工具那条认不出是哪个项目，没有收录（见 [06 §3](research/06-platform-access-and-account-risk.md#3-用户点名的项目)）。

用户写的 `echo`，按名字和星数推断对应 ycccccccy/echotrace（待核实），其他同名候选见 [06 §3](research/06-platform-access-and-account-risk.md#3-用户点名的项目)。

正式发布前逐条确认：

1. 许可证还是这里记的版本。NOASSERTION、无许可证文件、源码可见和非商业许可的条目，先读原文，确认能否公开列名、要不要附声明。
2. 署名要求：Apache-2.0 的 NOTICE、MIT 的版权行，以及项目自己的额外要求，例如 DownVid 的商标说明不许暗示官方背书，SATA 许可要求先 star 并致谢。
3. 状态：仓库还在，没有被归档、清空，也没有因律师函或 DMCA 下架。client-hook、private-api 两层的项目，以及同类项目收过律师函的，公开列名前再查一次。
4. 关系：计划适配的项目做成插件以后，在致谢里写清插件仓库和所包装的上游版本。
5. 06 还是草案，这里的 06 链接 2026-10-30 已按 06 的锚点核对过，06 定稿后再核对一次。

<a id="多平台下载与抓取"></a>
## 多平台下载与抓取

**计划适配（8）**

- [chubbyguan/chubbyskills](https://github.com/chubbyguan/chubbyskills)：14 个 Agent Skill 加统一命令行，把抖音、B站、小红书、公众号、X、播客等链接存成带来源字段的 Markdown，附本地检索、订阅日报和知识库 MCP。MIT。计划适配。见 [06 §3](research/06-platform-access-and-account-risk.md#3-用户点名的项目)
- [cv-cat/catbus](https://github.com/cv-cat/catbus)：Node 命令行，用同一套命令和统一 JSON 调用小红书、抖音、TikTok、B站、快手、微博等平台的网页端能力，面向人和 Agent。MIT。计划适配。见 [06 §4](research/06-platform-access-and-account-risk.md#4-多平台下载与抓取)
- [iszhouhua/social-media-copilot](https://github.com/iszhouhua/social-media-copilot)：小红书、抖音、快手的数据采集浏览器扩展，可下载作品原始文件，server 分支提供 HTTP 调用；2026-07 有用户反馈扩展被 Chrome 商店下架。GPL-3.0。计划适配。见 [06 §4](research/06-platform-access-and-account-risk.md#4-多平台下载与抓取)
- [lucky-Luor/bilibili-douyin-notes](https://github.com/lucky-Luor/bilibili-douyin-notes)：按平台取 B站、抖音、YouTube、小红书视频的字幕或媒体，无字幕时本地转写、按时间戳抽帧，再由宿主 Agent 写成 Markdown 笔记的 Agent Skill。MIT（抖音签名组件为 GPLv3，不在仓库内，运行时下载）。计划适配。见 [06 §3](research/06-platform-access-and-account-risk.md#3-用户点名的项目)
- [mikf/gallery-dl](https://github.com/mikf/gallery-dl)：从图片站和社交平台批量下载图片、视频与元数据的命令行，支持 X 书签、Instagram 收藏、Pinterest 画板、Reddit、微博等。GPL-2.0。计划适配。见 [06 §4](research/06-platform-access-and-account-risk.md#4-多平台下载与抓取)
- [NanmiCoder/NewsCrawler](https://github.com/NanmiCoder/NewsCrawler)：多平台新闻与文章抓取集合，主产品 news-extractor Skill 把公众号、今日头条等文章链接转成结构化 JSON 或 Markdown。GPL-3.0。计划适配。见 [06 §4](research/06-platform-access-and-account-risk.md#4-多平台下载与抓取)
- [tmwgsicp/ForgeRSS](https://github.com/tmwgsicp/ForgeRSS)：把网站和社交平台主页转成 RSS 2.0 订阅源的生成器，社交平台在本机带登录浏览器运行。AGPL-3.0。计划适配。见 [06 §4](research/06-platform-access-and-account-risk.md#4-多平台下载与抓取)
- [ucmao/media-parser](https://github.com/ucmao/media-parser)：自建的多平台分享链接解析服务，输入分享链接返回作者、标题、文案、封面和视频、图集、音频地址。MIT。计划适配。见 [06 §4](research/06-platform-access-and-account-risk.md#4-多平台下载与抓取)

**参考设计（14）**

- [arnoldhao/xiadown](https://github.com/arnoldhao/xiadown)：本地优先的媒体资源库桌面应用，集成 yt-dlp 下载、网页媒体嗅探、YouTube 浏览和 RSS 阅读。Apache-2.0。参考设计。见 [06 §4](research/06-platform-access-and-account-risk.md#4-多平台下载与抓取)
- [cshuangyy/videdown](https://github.com/cshuangyy/videdown)：Electron 桌面下载器，以 yt-dlp 为引擎下载抖音、小红书、B站、YouTube 等站点的原始视频文件，DownVid 由它改写而来。MIT。参考设计。见 [06 §4](research/06-platform-access-and-account-risk.md#4-多平台下载与抓取)
- [d3i-infra/data-donation-task](https://github.com/d3i-infra/data-donation-task)：学术数据捐赠前端，参与者在浏览器里本地解析自己的 TikTok、YouTube、Instagram、X 等平台导出包，确认后才提交给研究者。AGPL-3.0。参考设计。见 [06 §4](research/06-platform-access-and-account-risk.md#4-多平台下载与抓取)
- [digipower-academy/digipower-experiences](https://github.com/digipower-academy/digipower-experiences)：在浏览器里载入用户自己的平台导出包，按平台做可视化和查询的数据体验站点与组件库，覆盖约二十个平台。NOASSERTION（LICENSE 写明 AGPL-3.0-or-later）。参考设计。见 [06 §4](research/06-platform-access-and-account-risk.md#4-多平台下载与抓取)
- [dj-urg/data-mirroring](https://github.com/dj-urg/data-mirroring)：布鲁塞尔自由大学的研究工具，把用户上传的 TikTok、Instagram、YouTube、Netflix 导出 JSON 转成易读的表格和图表。NOASSERTION（LICENSE 写明 MPL-2.0）。参考设计。见 [06 §4](research/06-platform-access-and-account-risk.md#4-多平台下载与抓取)
- [Downvid/downvid](https://github.com/Downvid/downvid)：跨平台桌面视频下载器，给 yt-dlp 和 FFmpeg 加图形界面，抖音、快手另有直连解析。MIT（另有商标说明，不得用名称暗示官方背书）。参考设计。见 [06 §3](research/06-platform-access-and-account-risk.md#3-用户点名的项目)
- [dvlin-dev/social-media-favorites-archiver](https://github.com/dvlin-dev/social-media-favorites-archiver)：经专用 Chrome 会话枚举本人 B站、小红书、抖音收藏，在本地做 ASR、OCR 后写进 Markdown 或 Obsidian 的命令行加 Skill。MIT。参考设计。见 [06 §4](research/06-platform-access-and-account-risk.md#4-多平台下载与抓取)
- [JimLiu/baoyu-skills](https://github.com/JimLiu/baoyu-skills)：宝玉的 Agent Skill 合集，以配图、排版、多平台发文等创作为主，另有网页转 Markdown、X 帖子转 Markdown、YouTube 字幕等取数 Skill。MIT。参考设计。见 [06 §4](research/06-platform-access-and-account-risk.md#4-多平台下载与抓取)
- [js713-lab/savehub](https://github.com/js713-lab/savehub)：本地优先的收藏汇总面板加 MCP，导入 Instagram、TikTok 等官方导出和浏览器书签，中文平台用分享文本或链接清单，SQLite FTS5 支持中文检索。无许可证文件。参考设计。见 [06 §4](research/06-platform-access-and-account-risk.md#4-多平台下载与抓取)
- [karlicoss/HPI](https://github.com/karlicoss/HPI)：个人数据访问层，用统一的 Python 接口读取用户从 Reddit、X、Instagram、Telegram 等平台导出的数据。MIT。参考设计。见 [06 §4](research/06-platform-access-and-account-risk.md#4-多平台下载与抓取)
- [mcncarl/yichen-skills](https://github.com/mcncarl/yichen-skills)：逸尘面向内容创作者的 Agent Skill 合集，含收藏夹链接导出、链接归档、跨平台搜索路由、ASR 和微信本地数据解析；2026-10-23 收到律师函后删除了剪映相关 Skill。NOASSERTION（自定义个人学习与非商业使用许可，禁止作为公开合集再发布）。参考设计。见 [06 §3](research/06-platform-access-and-account-risk.md#3-用户点名的项目)
- [onfabric/context-use](https://github.com/onfabric/context-use)：把对话记录和 Instagram、Google、Telegram 等平台导出包转成 AI 记忆的命令行，附 Agent Skill。MIT。参考设计。见 [06 §4](research/06-platform-access-and-account-risk.md#4-多平台下载与抓取)
- [timelinize/timelinize](https://github.com/timelinize/timelinize)：本地优先的个人时间线应用，把各平台官方导出包、手机备份和本地文件导入同一个 SQLite 库，按时间、地点、人物浏览。AGPL-3.0。参考设计。见 [06 §4](research/06-platform-access-and-account-risk.md#4-多平台下载与抓取)
- [whiteguo233/OpenBiliClaw](https://github.com/whiteguo233/OpenBiliClaw)：本地运行的跨平台内容推荐 Agent，浏览器扩展同步用户在各平台的观看历史、收藏和关注，生成兴趣画像后找内容并说明推荐理由。MIT。参考设计。见 [06 §4](research/06-platform-access-and-account-risk.md#4-多平台下载与抓取)

**相邻项目（1）**

- [NanmiCoder/MediaCrawler](https://github.com/NanmiCoder/MediaCrawler)：多平台自媒体内容采集工具，按关键词、帖子 ID 或创作者主页抓小红书、抖音、快手、B站、微博、贴吧、知乎的帖子和评论；用户点名；非商业学习许可，只借做法，不分发。NOASSERTION（自定义「非商业学习使用许可证 1.1」，没有分发授权）。相邻项目。见 [06 §3](research/06-platform-access-and-account-risk.md#3-用户点名的项目)

<a id="抖音快手与-tiktok"></a>
## 抖音、快手与 TikTok

**计划适配（11）**

- [ahang008/ah-douyin-clean-downloader](https://github.com/ahang008/ah-douyin-clean-downloader)：面向 Codex 的抖音 Agent Skill，按分享链接下载原始视频，本机转写成 Markdown 与 SRT，也能把博主的公开作品整号归档。NOASSERTION（AH Source Available Non-Commercial License 1.0，源码可见，仅限非商业）。计划适配。见 [06 §3](research/06-platform-access-and-account-risk.md#3-用户点名的项目)
- [davidteather/TikTok-Api](https://github.com/davidteather/TikTok-Api)：非官方的 TikTok Python 封装，用 Playwright 会话取热门、用户、话题、音乐、评论和单条视频数据，不支持需要登录的接口。MIT。计划适配。见 [06 §5](research/06-platform-access-and-account-risk.md#5-抖音快手与-tiktok)
- [Evil0ctal/Douyin_TikTok_Download_API](https://github.com/Evil0ctal/Douyin_TikTok_Download_API)：自托管的抖音、TikTok、B站数据服务，通过 REST、MCP 和命令行取作品、作者、评论、收藏夹，下载后可在本机转写。Apache-2.0。计划适配。见 [06 §5](research/06-platform-access-and-account-risk.md#5-抖音快手与-tiktok)
- [Hannah2227/douyin-favorite-MyLike](https://github.com/Hannah2227/douyin-favorite-MyLike)：整理、搜索和回看本人抖音喜欢与收藏的桌面应用，请求由网页自己发出，支持增量采集、断点续采和收藏夹同步。MIT。计划适配。见 [06 §5](research/06-platform-access-and-account-risk.md#5-抖音快手与-tiktok)
- [Inoue-AI/Inoue-AI-TikTok-SDK](https://github.com/Inoue-AI/Inoue-AI-TikTok-SDK)：TikTok 官方开放接口的异步 Python SDK，覆盖内容发布、Display API 和 Data Portability API。MIT。计划适配。见 [06 §5](research/06-platform-access-and-account-risk.md#5-抖音快手与-tiktok)
- [JoeanAmier/KS-Downloader](https://github.com/JoeanAmier/KS-Downloader)：快手作品下载工具，按分享链接取单条视频或图集和作品信息，有终端界面和本地 Web API。GPL-3.0。计划适配。见 [06 §5](research/06-platform-access-and-account-risk.md#5-抖音快手与-tiktok)
- [JoeanAmier/TikTokDownloader](https://github.com/JoeanAmier/TikTokDownloader)：抖音与 TikTok 作品下载和数据采集工具，可按账号批量取发布、喜欢、收藏和收藏夹作品，提供本地 Web API。GPL-3.0。计划适配。见 [06 §5](research/06-platform-access-and-account-risk.md#5-抖音快手与-tiktok)
- [Johnserf-Seed/f2](https://github.com/Johnserf-Seed/f2)：多平台下载与接口封装的 Python 库和命令行，支持抖音、TikTok、微博、X，抖音侧能取主页、点赞、收藏、收藏夹、合集和直播。Apache-2.0。计划适配。见 [06 §5](research/06-platform-access-and-account-risk.md#5-抖音快手与-tiktok)
- [mlbb229229-create/douyin-favorites-mcp](https://github.com/mlbb229229-create/douyin-favorites-mcp)：用 Playwright 持久化浏览器登录本人抖音，只读读取默认收藏、收藏夹列表及其中作品的 MCP 服务。MIT。计划适配。见 [06 §5](research/06-platform-access-and-account-risk.md#5-抖音快手与-tiktok)
- [wujunwei928/parse-video](https://github.com/wujunwei928/parse-video)：Go 写的短视频分享链接解析服务和命令行，覆盖抖音、快手、西瓜视频等二十多个站点，返回标题、作者、视频和图集地址。MIT。计划适配。见 [06 §5](research/06-platform-access-and-account-risk.md#5-抖音快手与-tiktok)
- [ZhangYukwiver/DouyinDataReview](https://github.com/ZhangYukwiver/DouyinDataReview)：读取本人抖音观看历史、喜欢、收藏并导出 JSON 的本地桌面应用，也能导入抖音官方个人信息下载档案、生成年度回顾。MIT。计划适配。见 [06 §5](research/06-platform-access-and-account-risk.md#5-抖音快手与-tiktok)

**参考设计（17）**

- [commitmaniac/tdp](https://github.com/commitmaniac/tdp)：C 写的 TikTok 导出解析命令行，读 `user_data.json`，按参数打印收藏视频、喜欢视频或观看历史的链接。BSD-3-Clause。参考设计。见 [06 §5](research/06-platform-access-and-account-risk.md#5-抖音快手与-tiktok)
- [fluoxtine2004/douyin-favorites-organizer](https://github.com/fluoxtine2004/douyin-favorites-organizer)：把本人抖音收藏夹按原分类抓下来，生成逐字稿、关键帧和 Obsidian 笔记，再出可批注网页和周、季、年总结。MIT。参考设计。见 [06 §5](research/06-platform-access-and-account-risk.md#5-抖音快手与-tiktok)
- [Hype-yolo/backtap-douyin](https://github.com/Hype-yolo/backtap-douyin)：iOS 快捷指令加 Node.js 解析服务，复制抖音链接后轻点手机背面，用匿名浏览器打开公开页面取视频地址并存进相册。MIT。参考设计。见 [06 §5](research/06-platform-access-and-account-risk.md#5-抖音快手与-tiktok)
- [JBGruber/traktok](https://github.com/JBGruber/traktok)：R 语言包，既封装 TikTok 官方研究 API，也提供模拟网页请求的非官方接口，可取视频、用户、评论等数据。GPL-3.0。参考设计。见 [06 §5](research/06-platform-access-and-account-risk.md#5-抖音快手与-tiktok)
- [jiji262/douyin-downloader](https://github.com/jiji262/douyin-downloader)：抖音批量下载命令行，带 SQLite 去重、增量和浏览器兜底；README 写明单条、合集、点赞、收藏下载目前被抖音的请求校验拦下。MIT（同仓库的桌面版 Douzy 闭源）。参考设计。见 [06 §5](research/06-platform-access-and-account-risk.md#5-抖音快手与-tiktok)
- [joeycato/tiktok-favesave](https://github.com/joeycato/tiktok-favesave)：桌面应用，读 TikTok 官方导出里的收藏和喜欢清单，按时间过滤后用 yt-dlp 批量下载，跳过已下载和已失效的条目。NOASSERTION（CC BY-NC-SA 4.0，README 另写未经许可不得再分发、修改或商用）。参考设计。见 [06 §5](research/06-platform-access-and-account-risk.md#5-抖音快手与-tiktok)
- [john00-dev/tiktok-liked-content](https://github.com/john00-dev/tiktok-liked-content)：单页 TikTok 活动查看器，读官方导出里的喜欢、收藏视频、收藏音乐和特效，用嵌入播放器浏览，附多次导出合并脚本。无许可证文件。参考设计。见 [06 §5](research/06-platform-access-and-account-risk.md#5-抖音快手与-tiktok)
- [lid664951-crypto/douyin-creator-toolkit](https://github.com/lid664951-crypto/douyin-creator-toolkit)：Windows 桌面工具，把抖音链接解析、批量下载、本地转写、AI 拆解、私有知识库和创作 Agent 串成一条本地流程。MIT（内置的 dy-mcp 来自无许可证仓库）。参考设计。见 [06 §5](research/06-platform-access-and-account-risk.md#5-抖音快手与-tiktok)
- [llm-xiaoye/douyin-to-obsidian](https://github.com/llm-xiaoye/douyin-to-obsidian)：给 cc-connect 加 Claude Code 用的 `/dy` 技能，把抖音视频、图文和收藏夹整理成 Obsidian 知识笔记，转写和 OCR 都在本机。MIT。参考设计。见 [06 §5](research/06-platform-access-and-account-risk.md#5-抖音快手与-tiktok)
- [lyxdream/obsidian-content-capture-backend](https://github.com/lyxdream/obsidian-content-capture-backend)：Obsidian 抖音入库插件的本地 Python 后端，把分享链接解析成视频或图集，下载原始文件并用本地 Whisper 提取文案。无许可证文件（配套插件 obsidian-douyin-capture 为 MIT）。参考设计。见 [06 §5](research/06-platform-access-and-account-risk.md#5-抖音快手与-tiktok)
- [lyxdream/obsidian-douyin-capture](https://github.com/lyxdream/obsidian-douyin-capture)：把抖音分享链接转成视频、图集、文案和转写的 Obsidian 插件，frontmatter 记作品 ID，附件按作品 ID 分目录，抓取交给本机后端。MIT。参考设计。见 [05 §1.1](research/05-note-apps-and-plugins.md#11-obsidian-社区插件)
- [samirelanduk/tiktok-save](https://github.com/samirelanduk/tiktok-save)：以 TikTok 官方数据导出里的喜欢、收藏和观看记录为清单，逐条下载视频并为每条保存元数据 JSON。无许可证文件。参考设计。见 [06 §5](research/06-platform-access-and-account-risk.md#5-抖音快手与-tiktok)
- [tars1230/douyin-favorites-to-knowledge](https://github.com/tars1230/douyin-favorites-to-knowledge)：把本人抖音收藏同步成本地 Markdown 或 Obsidian 笔记的 Agent Skill 加命令行，带转写、图文 OCR 和每日日报。MIT（运行依赖的 douyin_knowledge_core 轮子为专有许可）。参考设计。见 [06 §5](research/06-platform-access-and-account-risk.md#5-抖音快手与-tiktok)
- [yumingtao/douyin-favorites-sync](https://github.com/yumingtao/douyin-favorites-sync)：定时批量同步抖音收藏的 Obsidian 插件，按作品 ID 跳过已导入的笔记，附件按作品 ID 分目录，后端是本机 FastAPI 服务。MIT。参考设计。见 [05 §1.1](research/05-note-apps-and-plugins.md#11-obsidian-社区插件)
- [yumingtao/douyin-favorites-sync-backend](https://github.com/yumingtao/douyin-favorites-sync-backend)：Obsidian 抖音收藏同步插件的本地 FastAPI 后端，拉收藏、取文案、下载视频并可做 Whisper 转写，带 Web 管理页。MIT（签名模块是加密字节码，无法审计）。参考设计。见 [06 §5](research/06-platform-access-and-account-risk.md#5-抖音快手与-tiktok)
- [yzfly/douyin-mcp-server](https://github.com/yzfly/douyin-mcp-server)：从抖音分享链接取视频地址、再用云端语音识别提取文案的 MCP 服务，附 WebUI 和命令行，仓库已归档。Apache-2.0。参考设计。见 [06 §5](research/06-platform-access-and-account-risk.md#5-抖音快手与-tiktok)
- [zollo/tiktok-downloader](https://github.com/zollo/tiktok-downloader)：读取 TikTok 官方数据导出里的 `user_data.json`，按导出链接批量下载本人发布过的视频。GPL-3.0。参考设计。见 [06 §5](research/06-platform-access-and-account-risk.md#5-抖音快手与-tiktok)

<a id="小红书"></a>
## 小红书

**计划适配（5）**

- [jackwener/xhs-cli](https://github.com/jackwener/xhs-cli)：小红书命令行的浏览器版，用 Camoufox 打开网页读取页面状态，可查看收藏列表，命令都能输出 JSON。Apache-2.0。计划适配。见 [06 §6](research/06-platform-access-and-account-risk.md#6-小红书)
- [JoeanAmier/XHS-Downloader](https://github.com/JoeanAmier/XHS-Downloader)：小红书笔记链接解析与作品下载工具，有图形界面、命令行、本地 HTTP API 和 MCP，另附在网页上提取收藏、点赞、专辑链接的油猴脚本。GPL-3.0。计划适配。见 [06 §6](research/06-platform-access-and-account-risk.md#6-小红书)
- [lucasygu/redbook](https://github.com/lucasygu/redbook)：面向 AI Agent 的小红书 Node.js 命令行，读取本机 Chrome 的 Cookie，可搜索、读笔记、查看收藏和收藏专辑。MIT。计划适配。见 [06 §6](research/06-platform-access-and-account-risk.md#6-小红书)
- [wkwunju/xhs-vault](https://github.com/wkwunju/xhs-vault)：把小红书收藏夹导出成本地可搜索的知识库，收藏页导出清单，逐条抓正文和平台字幕，macOS Vision 做图片 OCR。MIT。计划适配。见 [06 §6](research/06-platform-access-and-account-risk.md#6-小红书)
- [xpzouying/xiaohongshu-mcp](https://github.com/xpzouying/xiaohongshu-mcp)：Go 写的小红书 MCP 和 HTTP 服务，用内置浏览器登录网页，提供搜索、笔记详情和评论、用户主页（含自己的收藏和点赞）等工具。Apache-2.0。计划适配。见 [06 §6](research/06-platform-access-and-account-risk.md#6-小红书)

**参考设计（12）**

- [bonchiang96/xiaohongshu-importer](https://github.com/bonchiang96/xiaohongshu-importer)：粘贴小红书分享文本生成一篇带图片或视频笔记的 Obsidian 插件，一次手动导入一条。MIT。参考设计。见 [06 §6](research/06-platform-access-and-account-risk.md#6-小红书)
- [ceciliayu56-cell/xhs-to-obsidian-scriptable](https://github.com/ceciliayu56-cell/xhs-to-obsidian-scriptable)：iPhone、iPad 上的快捷指令加 Scriptable 脚本，从剪贴板取小红书分享链接，匿名读笔记后交给大模型识别总结，写入 Obsidian。MIT。参考设计。见 [06 §6](research/06-platform-access-and-account-risk.md#6-小红书)
- [CH-YYK/obsidian-rednote](https://github.com/CH-YYK/obsidian-rednote)：粘贴小红书分享文案生成笔记的 Obsidian 插件，注册了 `obsidian://rednote-import` 协议，可由 iOS 快捷指令或 Android Tasker 从分享菜单触发。MIT。参考设计。见 [06 §6](research/06-platform-access-and-account-risk.md#6-小红书)
- [chenjin-cmd/xhs-account-teardown](https://github.com/chenjin-cmd/xhs-account-teardown)：不登录抓取小红书账号主页与第一页笔记封面，或凭笔记链接抓取单条正文与图片，再由 Agent 识图写账号拆解报告的 Agent Skill。MIT。参考设计。见 [06 §3](research/06-platform-access-and-account-risk.md#3-用户点名的项目)
- [chenxiachan/xhs-claude-skills](https://github.com/chenxiachan/xhs-claude-skills)：Claude Code 插件，用 `/xhs` 加链接把一篇小红书笔记整理成 Obsidian 笔记，图片做 OCR，视频优先取平台字幕。MIT。参考设计。见 [06 §6](research/06-platform-access-and-account-risk.md#6-小红书)
- [feitangyuan/kankan-shoucang](https://github.com/feitangyuan/kankan-shoucang)：整理小红书收藏的 macOS 本地应用，把网页上的笔记卡片拖进来逐条匿名解析，用系统 OCR 和本地转写整理后自动分类，提供 MCP。AGPL-3.0。参考设计。见 [06 §6](research/06-platform-access-and-account-risk.md#6-小红书)
- [LinkisLethe/REDnote-clipper](https://github.com/LinkisLethe/REDnote-clipper)：把小红书图文笔记转成可编辑 Markdown 的 Chrome 扩展，在浏览器里用 PP-OCRv6 做本地 OCR，可导出到 Obsidian。Apache-2.0。参考设计。见 [06 §6](research/06-platform-access-and-account-risk.md#6-小红书)
- [loresun/xhs-hub](https://github.com/loresun/xhs-hub)：油猴脚本在已登录页面截获列表和详情请求推给本地服务存进 SQLite，视频自动下载、转写和润色。MIT。参考设计。见 [06 §6](research/06-platform-access-and-account-risk.md#6-小红书)
- [PCPrincipal67/xhs-favorites-exporter](https://github.com/PCPrincipal67/xhs-favorites-exporter)：在已登录的小红书收藏页里，把收藏的笔记条目连同 `xsec_token` 导出成 JSON 的 Chrome 扩展。无许可证文件（README 写 MIT，仓库无 LICENSE 文件）。参考设计。见 [06 §6](research/06-platform-access-and-account-risk.md#6-小红书)
- [rowland-dot/favsense](https://github.com/rowland-dot/favsense)：本地运行的小红书收藏整理工具，在专用扫描浏览器里读收藏清单，匿名取详情、转写视频、识别画面文字，生成可搜索的静态知识库。MIT。参考设计。见 [06 §6](research/06-platform-access-and-account-risk.md#6-小红书)
- [themrv1ck/xiaohongshu-web-collection-organizing](https://github.com/themrv1ck/xiaohongshu-web-collection-organizing)：在用户已登录的浏览器里读取小红书收藏页、点赞页当前可见的卡片，做 OCR 与分类建议的 Skill，写入专辑前逐项由用户确认。MIT。参考设计。见 [06 §6](research/06-platform-access-and-account-risk.md#6-小红书)
- [ValerieTse/RedCaChe](https://github.com/ValerieTse/RedCaChe)：本地优先的小红书收藏整理工具，导入本地数据库后按关键词分类，逐条决定保留、移除或归档，可导出 Markdown。MIT。参考设计。见 [06 §6](research/06-platform-access-and-account-risk.md#6-小红书)

<a id="b站youtube-与播客"></a>
## B站、YouTube 与播客

**计划适配（13）**

- [ayasa520/bilibili-favorites-exporter](https://github.com/ayasa520/bilibili-favorites-exporter)：在 B站 收藏夹页把自建收藏夹和订阅的合集导出为 JSON，再在扩展页面里本地浏览、搜索的浏览器扩展。MIT。计划适配。见 [06 §7](research/06-platform-access-and-account-risk.md#7-b站youtube-与播客)
- [iawia002/lux](https://github.com/iawia002/lux)：Go 写的视频下载库和命令行，按域名路由到各站点抽取器，覆盖 B站、YouTube 和多个国内视频站，release 停在 2024-05。MIT。计划适配。见 [06 §7](research/06-platform-access-and-account-risk.md#7-b站youtube-与播客)
- [imputnet/cobalt](https://github.com/imputnet/cobalt)：自托管的媒体保存服务，提交链接就返回可下载的音视频或图片，网页前端和 API 分开部署。AGPL-3.0（`api/` 为 AGPL-3.0，`web/` 为 CC-BY-NC-SA-4.0）。计划适配。见 [06 §7](research/06-platform-access-and-account-risk.md#7-b站youtube-与播客)
- [jdepoix/youtube-transcript-api](https://github.com/jdepoix/youtube-transcript-api)：按视频 ID 取 YouTube 字幕（含自动字幕和翻译）的 Python 库和命令行，不需要 API key，也不需要浏览器。MIT。计划适配。见 [06 §7](research/06-platform-access-and-account-risk.md#7-b站youtube-与播客)
- [lightpohl/podcast-dl](https://github.com/lightpohl/podcast-dl)：读 RSS 下载播客单集音频的 Node 命令行，可另存节目和单集元数据 JSON，以及 feed 里声明的文字稿。MIT。计划适配。见 [06 §7](research/06-platform-access-and-account-risk.md#7-b站youtube-与播客)
- [LuanRT/YouTube.js](https://github.com/LuanRT/YouTube.js)：YouTube 内部 InnerTube 接口的 JavaScript 客户端，可取视频信息、字幕、评论、播放列表、媒体库和观看历史。MIT。计划适配。见 [06 §7](research/06-platform-access-and-account-risk.md#7-b站youtube-与播客)
- [sigma67/ytmusicapi](https://github.com/sigma67/ytmusicapi)：YouTube Music 的非官方 Python 接口，可读点赞歌曲、收听历史、播放列表，也能取播客单集。MIT。计划适配。见 [06 §7](research/06-platform-access-and-account-risk.md#7-b站youtube-与播客)
- [soimort/you-get](https://github.com/soimort/you-get)：Python 写的网页媒体下载器，靠一张站点表路由，表外站点交给通用抽取器，可只输出 JSON 信息。NOASSERTION（`LICENSE.txt` 正文为 MIT）。计划适配。见 [06 §7](research/06-platform-access-and-account-risk.md#7-b站youtube-与播客)
- [ultrazg/xyz](https://github.com/ultrazg/xyz)：Go 写的小宇宙 App 接口封装，起一个本地 HTTP 服务，提供搜索、订阅、收藏、收听历史等接口和文档页。MIT。计划适配。见 [06 §7](research/06-platform-access-and-account-risk.md#7-b站youtube-与播客)
- [xiaoleiy/podpull](https://github.com/xiaoleiy/podpull)：Python 播客下载命令行，接受 Apple Podcasts 链接、RSS 或小宇宙单集链接，支持断点续传，可选本地转写和摘要。MIT。计划适配。见 [06 §7](research/06-platform-access-and-account-risk.md#7-b站youtube-与播客)
- [XZXZZX-Ai/bilibili-mcp](https://github.com/XZXZZX-Ai/bilibili-mcp)：本地运行的 B站 MCP 服务，可读字幕并定位原话和时间点、读评论、遍历自己账号的全部收藏夹，无字幕时可本地转写。Apache-2.0。计划适配。见 [06 §7](research/06-platform-access-and-account-risk.md#7-b站youtube-与播客)
- [yt-dlp/yt-dlp](https://github.com/yt-dlp/yt-dlp)：命令行音视频下载器，内置上千个站点抽取器，可只输出 JSON 元数据和字幕，抽取器能以插件扩展。Unlicense。计划适配。见 [06 §7](research/06-platform-access-and-account-risk.md#7-b站youtube-与播客)
- [yutto-dev/yutto](https://github.com/yutto-dev/yutto)：Python 写的 B站 命令行下载器，支持投稿、收藏夹和稍后再看批量下载，可另存字幕和弹幕，也有 server 模式。GPL-3.0。计划适配。见 [06 §7](research/06-platform-access-and-account-risk.md#7-b站youtube-与播客)

**参考设计（14）**

- [amtoaer/bili-sync](https://github.com/amtoaer/bili-sync)：Rust 写的 B站 同步服务，定时把收藏夹、合集、稍后再看、UP 主投稿下载到本地，生成 Emby、Jellyfin 可读的元数据。MIT。参考设计。见 [06 §7](research/06-platform-access-and-account-risk.md#7-b站youtube-与播客)
- [bggRGjQaUbCoE/PiliPlus](https://github.com/bggRGjQaUbCoE/PiliPlus)：Flutter 写的 B站 第三方客户端，覆盖手机和桌面，支持收藏夹、稍后再看、观看历史和离线缓存。GPL-3.0。参考设计。见 [06 §7](research/06-platform-access-and-account-risk.md#7-b站youtube-与播客)
- [chhoumann/PodNotes](https://github.com/chhoumann/PodNotes)：在 Obsidian 里播放播客、记时间戳笔记、生成单集笔记的插件，可调用云端接口转写。MIT。参考设计。见 [06 §7](research/06-platform-access-and-account-risk.md#7-b站youtube-与播客)
- [dogsheep/google-takeout-to-sqlite](https://github.com/dogsheep/google-takeout-to-sqlite)：Dogsheep 系列，把 Google Takeout 里的「我的活动」和定位记录写进 SQLite，配合 Datasette 浏览。Apache-2.0。参考设计。见 [06 §7](research/06-platform-access-and-account-risk.md#7-b站youtube-与播客)
- [IndieKKY/bilibili-subtitle](https://github.com/IndieKKY/bilibili-subtitle)：在 B站 视频页显示字幕列表、可点击跳转和下载字幕，并调用大模型翻译总结的浏览器扩展。MIT。参考设计。见 [06 §7](research/06-platform-access-and-account-risk.md#7-b站youtube-与播客)
- [ItsMaciek/WatchNow](https://github.com/ItsMaciek/WatchNow)：读 Google Takeout 里 YouTube「稍后观看」播放列表的 CSV，显示缩略图并按添加时间或标题排序的单页工具。无许可证文件。参考设计。见 [06 §7](research/06-platform-access-and-account-risk.md#7-b站youtube-与播客)
- [JefferyHcool/BiliNote](https://github.com/JefferyHcool/BiliNote)：AI 视频笔记工具，把 B站、YouTube、抖音、快手链接或本地视频转写后生成带时间戳和截图的 Markdown 笔记。MIT（抖音签名文件 `abogus.py` 带 GPL-3.0 文件头）。参考设计。见 [06 §7](research/06-platform-access-and-account-risk.md#7-b站youtube-与播客)
- [jkawamoto/mcp-youtube-transcript](https://github.com/jkawamoto/mcp-youtube-transcript)：取 YouTube 字幕的 MCP 服务，长字幕分页返回并附视频元数据。MIT。参考设计。见 [06 §7](research/06-platform-access-and-account-risk.md#7-b站youtube-与播客)
- [lstrzepek/obsidian-yt-transcript](https://github.com/lstrzepek/obsidian-yt-transcript)：取 YouTube 字幕插到笔记里的 Obsidian 插件，时间戳写成带秒数参数的原视频链接。MIT。参考设计。见 [05 §1.1](research/05-note-apps-and-plugins.md#11-obsidian-社区插件)
- [mzky/m4s-converter](https://github.com/mzky/m4s-converter)：把 B站 客户端缓存的 m4s 音视频合并成 mp4 并生成弹幕字幕的小工具，用来保存已下架视频的本地缓存。MIT（附带的 MP4Box 二进制另有许可证）。参考设计。见 [06 §7](research/06-platform-access-and-account-risk.md#7-b站youtube-与播客)
- [purarue/google_takeout_parser](https://github.com/purarue/google_takeout_parser)：解析 Google Takeout 导出包的 Python 库和命令行，把 YouTube 观看历史、评论、点赞等转成结构化事件。MIT。参考设计。见 [06 §7](research/06-platform-access-and-account-risk.md#7-b站youtube-与播客)
- [Rui-0221/bilinote-sync](https://github.com/Rui-0221/bilinote-sync)：把 BiliNote 生成的笔记和截图导入 Obsidian 的插件，从不覆盖，靠 frontmatter 里的收据认回被移动或改名的笔记。MIT。参考设计。见 [05 §1.1](research/05-note-apps-and-plugins.md#11-obsidian-社区插件)
- [scripting/opmlPackage](https://github.com/scripting/opmlPackage)：OPML 格式作者 Dave Winer 维护的读写库，用于大纲工具、订阅阅读器和播客订阅列表的导入导出。MIT。参考设计。见 [06 §7](research/06-platform-access-and-account-risk.md#7-b站youtube-与播客)
- [snipd-app/snipd-obsidian](https://github.com/snipd-app/snipd-obsidian)：同步 Snipd 播客片段、转写和摘要的 Obsidian 插件，按文件 MD5 判断用户是否改过，没改过就重写，改过只追加。MIT。参考设计。见 [05 §1.1](research/05-note-apps-and-plugins.md#11-obsidian-社区插件)

<a id="微信生态"></a>
## 微信生态

**计划适配（10）**

- [BlueMatthew/WechatExporter](https://github.com/BlueMatthew/WechatExporter)：从 iTunes 或 Finder 的 iPhone 备份里读取微信聊天记录，导出为 HTML、PDF 等格式，有图形界面和命令行。GPL-2.0。计划适配。见 [06 §8](research/06-platform-access-and-account-risk.md#8-微信生态)
- [Bwkyd/wexin-read-mcp](https://github.com/Bwkyd/wexin-read-mcp)：让 AI 读取公众号文章的 MCP 服务，抓取交给作者的单文件程序 url-md，输出带 frontmatter 的 Markdown。MIT（实际抓取的 url-md 为 Apache-2.0）。计划适配。见 [06 §8](research/06-platform-access-and-account-risk.md#8-微信生态)
- [fengxxc/wechatmp2markdown](https://github.com/fengxxc/wechatmp2markdown)：把单篇公众号文章转成 Markdown 的 Go 命令行和 Web 服务，图片可保留链接、下载到本地或内嵌 base64。MIT。计划适配。见 [06 §8](research/06-platform-access-and-account-risk.md#8-微信生态)
- [Hello-Mr-Crab/pywechat](https://github.com/Hello-Mr-Crab/pywechat)：基于 pywinauto 的 Windows 微信 RPA，新版能把收藏里的笔记导出为 Markdown、批量复制收藏文章链接，附 MCP 和 Skill。NOASSERTION（LICENSE 为 LGPL-2.1）。计划适配。见 [06 §8](research/06-platform-access-and-account-risk.md#8-微信生态)
- [jackwener/wechat-article-to-markdown](https://github.com/jackwener/wechat-article-to-markdown)：抓取单篇公众号文章转成带本地图片 Markdown 的命令行，附 Agent Skill。无许可证文件（`pyproject.toml` 与 README 写 MIT，仓库无 LICENSE 文件）。计划适配。见 [06 §8](research/06-platform-access-and-account-risk.md#8-微信生态)
- [ltaoo/wx_channels_download](https://github.com/ltaoo/wx_channels_download)：视频号下载器扩展成的内容归档工具，以本机代理方式在视频号页面加下载按钮，提供 HTTP API、MCP、命令行和本地管理页，属 client-hook 层。NOASSERTION（MIT + Commons Clause）。计划适配。见 [06 §8](research/06-platform-access-and-account-risk.md#8-微信生态)
- [pengyulong/wechat-favorites-exporter](https://github.com/pengyulong/wechat-favorites-exporter)：用 macOS 辅助功能驱动已登录的 Mac 微信收藏页，把收藏里的公众号文章链接导出为 CSV。MIT。计划适配。见 [06 §8](research/06-platform-access-and-account-risk.md#8-微信生态)
- [rachelos/we-mp-rss](https://github.com/rachelos/we-mp-rss)：自托管的公众号订阅服务，定时抓取文章生成 RSS，导出 Markdown、PDF、Word、JSON，提供 API 与 Webhook。NOASSERTION（LICENSE 正文为 MIT）。计划适配。见 [06 §8](research/06-platform-access-and-account-risk.md#8-微信生态)
- [Tencent/WeChatReading](https://github.com/Tencent/WeChatReading)：微信读书官方发布的 Agent Skill，用个人 API Key 调用官方接口读书架、阅读进度、笔记划线、想法和阅读统计。NOASSERTION（Apache-2.0 全文前加了腾讯开源声明）。计划适配。见 [06 §8](research/06-platform-access-and-account-risk.md#8-微信生态)
- [wechat-article/wechat-article-exporter](https://github.com/wechat-article/wechat-article-exporter)：可自部署的公众号文章批量导出网站，用自己的公众号后台登录后导出 HTML、Markdown、DOCX 等格式；依赖的后台接口关闭后，2026-07-30 起停止维护。MIT（配套的 wxdown-service 无许可证）。计划适配。见 [06 §3](research/06-platform-access-and-account-risk.md#3-用户点名的项目)

**参考设计（13）**

- [BBbinlei/wechat-sentence-excerpt](https://github.com/BBbinlei/wechat-sentence-excerpt)：安卓无障碍服务，在手机微信里读公众号文章时按句点选并保存摘录，文字靠本机离线 OCR 获取。无许可证文件。参考设计。见 [06 §8](research/06-platform-access-and-account-risk.md#8-微信生态)
- [cooderl/wewe-rss](https://github.com/cooderl/wewe-rss)：借微信读书账号生成公众号 RSS 的自托管服务，可输出全文，仓库已归档。MIT。参考设计。见 [06 §8](research/06-platform-access-and-account-risk.md#8-微信生态)
- [dlv2008/weixin-favor-kb](https://github.com/dlv2008/weixin-favor-kb)：把视频号收藏视频整理成 Obsidian 知识库的本地流水线，含 Whisper 转写、关键帧 OCR 和大模型分类。无许可证文件。参考设计。见 [06 §8](research/06-platform-access-and-account-risk.md#8-微信生态)
- [dustin573/wechat-mcp](https://github.com/dustin573/wechat-mcp)：通过 macOS 辅助功能 API 读取并操作 Mac 微信 4.x 的 MCP 服务，可列会话、读消息。MIT。参考设计。见 [06 §8](research/06-platform-access-and-account-risk.md#8-微信生态)
- [freestylefly/WeChatBridge](https://github.com/freestylefly/WeChatBridge)：接住微信「合并转发到其他应用」生成的聊天记录压缩包，转交给本机 AI 应用、Obsidian 或指定文件夹。MIT（基于 MIT 许可的 qzz0518/Dukou 二次开发）。参考设计。见 [06 §8](research/06-platform-access-and-account-risk.md#8-微信生态)
- [halohazhang/wechat-favorites-to-obsidian](https://github.com/halohazhang/wechat-favorites-to-obsidian)：把 Mac 微信收藏夹里的公众号文章批量导出为 Obsidian 笔记的 Claude Skill，先存本地 Markdown 再去重入库，每步人工确认。MIT。参考设计。见 [06 §8](research/06-platform-access-and-account-risk.md#8-微信生态)
- [Higurashi-kagome/wereader](https://github.com/Higurashi-kagome/wereader)：微信读书网页版的浏览器扩展，把划线、想法、书评导出为 Markdown，并增强阅读页。MIT。参考设计。见 [06 §8](research/06-platform-access-and-account-risk.md#8-微信生态)
- [joeseesun/qiaomu-wx-video](https://github.com/joeseesun/qiaomu-wx-video)：下载视频号分享链接和直播回放的 Agent Skill，按在线解析、本机捕获、浏览器捕获、录屏逐级降级。MIT。参考设计。见 [06 §8](research/06-platform-access-and-account-risk.md#8-微信生态)
- [leofang230316-prog/siyuan-plugin-weread-sync](https://github.com/leofang230316-prog/siyuan-plugin-weread-sync)：把微信读书同步进思源的插件，按本地、上次快照、远端三方比对，两端都改时保留本地并在旁边插入远端新版。MIT。参考设计。见 [05 §1.3](research/05-note-apps-and-plugins.md#13-其他笔记软件的同类插件)
- [mingjuner123-spec/wechat-inbox-sync](https://github.com/mingjuner123-spec/wechat-inbox-sync)：手机上把公众号、视频号、抖音、小红书、B站、小宇宙等链接存进作者的云端收件箱，电脑端 Obsidian 插件拉取、解析、转写后写成 Markdown。NOASSERTION（1.3.117 起为 MIT + Commons Clause，1.3.116 及更早为 MIT）。参考设计。见 [06 §3](research/06-platform-access-and-account-risk.md#3-用户点名的项目)
- [MuZiJin701/wx-video-account-notes](https://github.com/MuZiJin701/wx-video-account-notes)：把视频号分享链接解析成视频、图文和音乐文件，在本机做 OCR、ASR 并整理成 Markdown 笔记的 Agent Skill。MIT（服务端文件来自上游，受 MIT + Commons Clause 约束）。参考设计。见 [06 §8](research/06-platform-access-and-account-risk.md#8-微信生态)
- [shibanyu333/wechat-chat-export](https://github.com/shibanyu333/wechat-chat-export)：Mac 微信聊天记录导出工具，读屏、自动滚动截图、本机 OCR，导出 Word 或 Markdown，附带图片和聊天文件。MIT。参考设计。见 [06 §8](research/06-platform-access-and-account-risk.md#8-微信生态)
- [zhaohongxuan/obsidian-weread-plugin](https://github.com/zhaohongxuan/obsidian-weread-plugin)：把微信读书的书籍、划线、想法、书评同步进 Obsidian 的社区插件。MIT。参考设计。见 [06 §8](research/06-platform-access-and-account-risk.md#8-微信生态)

**相邻项目（2）**

- [open-wxa/wxa-runner](https://github.com/open-wxa/wxa-runner)：在普通浏览器里用微信官方基础库运行自己的小程序编译产物，用于本地调试和 Playwright E2E 测试，不取平台内容；用户点名；内置的基础库是腾讯代码、没有许可证，不能随歇存分发。MIT。相邻项目。见 [06 §3](research/06-platform-access-and-account-risk.md#3-用户点名的项目)
- [ycccccccy/echotrace](https://github.com/ycccccccy/echotrace)：微信 4.x PC 端聊天记录的本地导出与分析工具，解密本机数据库后导出 JSON、HTML、Excel 并生成年度报告，2026-01-18 停止维护，默认分支只剩文档；用户点名的 echo，对应关系是推断；client-hook，作者的另一个仓库 wx_key 在腾讯 2026 年的 DMCA 通知里。MIT。相邻项目。见 [06 §3](research/06-platform-access-and-account-risk.md#3-用户点名的项目)

<a id="微博知乎豆瓣与海外社交平台"></a>
## 微博、知乎、豆瓣与海外社交平台

**计划适配（16）**

- [afar1/fieldtheory-cli](https://github.com/afar1/fieldtheory-cli)：把自己的 X 书签（含书签文件夹和链接正文）同步到本地，提供全文搜索、Markdown 导出和大模型分类，可给 Agent 当知识库的命令行。MIT。计划适配。见 [06 §9](research/06-platform-access-and-account-risk.md#9-微博知乎豆瓣与海外社交平台)
- [Chilfish/Weibo-archiver](https://github.com/Chilfish/Weibo-archiver)：油猴脚本加命令行，把自己的微博、关注列表和微博收藏导出成 JSON，用配套网页离线浏览。Apache-2.0。计划适配。见 [06 §9](research/06-platform-access-and-account-risk.md#9-微博知乎豆瓣与海外社交平台)
- [cosformula/douban-sync-skill](https://github.com/cosformula/douban-sync-skill)：导出并增量同步本人豆瓣书、影视、音乐、游戏标记到本地 CSV 的 Agent Skill，日常增量走公开 RSS。MIT。计划适配。见 [06 §9](research/06-platform-access-and-account-risk.md#9-微博知乎豆瓣与海外社交平台)
- [dataabc/weibo-crawler](https://github.com/dataabc/weibo-crawler)：按用户 ID 批量抓取微博用户资料和微博内容，写成 CSV、JSON 或数据库，可下载图片和视频，不填 cookie 也能取多数公开微博。无许可证文件。计划适配。见 [06 §9](research/06-platform-access-and-account-risk.md#9-微博知乎豆瓣与海外社交平台)
- [dataabc/weiboSpider](https://github.com/dataabc/weiboSpider)：抓指定微博用户的资料和全部微博，写入文件或数据库，必须提供登录 cookie。无许可证文件。计划适配。见 [06 §9](research/06-platform-access-and-account-risk.md#9-微博知乎豆瓣与海外社交平台)
- [handsomestWei/zhihu-fetch-skill](https://github.com/handsomestWei/zhihu-fetch-skill)：抓取知乎收藏夹、个人点赞与收藏动态、专栏和回答的正文与图片，支持断点续传并可写入 Obsidian 的 Agent Skill。MIT。计划适配。见 [06 §9](research/06-platform-access-and-account-risk.md#9-微博知乎豆瓣与海外社交平台)
- [instaloader/instaloader](https://github.com/instaloader/instaloader)：下载 Instagram 帖子、快拍、Reels 及其文案和元数据的命令行与库，登录后可下载自己的收藏。MIT。计划适配。见 [06 §9](research/06-platform-access-and-account-risk.md#9-微博知乎豆瓣与海外社交平台)
- [iyear/tdl](https://github.com/iyear/tdl)：Go 写的 Telegram 工具箱，可下载会话中的文件、转发、上传，并把消息导出为 JSON。AGPL-3.0。计划适配。见 [06 §9](research/06-platform-access-and-account-risk.md#9-微博知乎豆瓣与海外社交平台)
- [karlicoss/rexport](https://github.com/karlicoss/rexport)：把自己的 Reddit 收藏、点赞、发帖和评论导出为 JSON，并提供读取导出数据的接口。MIT。计划适配。见 [06 §9](research/06-platform-access-and-account-risk.md#9-微博知乎豆瓣与海外社交平台)
- [knadh/tg-archive](https://github.com/knadh/tg-archive)：定期把 Telegram 群组消息同步到本地 SQLite，再生成可发布的静态归档网站。MIT。计划适配。见 [06 §9](research/06-platform-access-and-account-risk.md#9-微博知乎豆瓣与海外社交平台)
- [Marvae/douban-cli](https://github.com/Marvae/douban-cli)：豆瓣命令行加 Agent Skill，把自己的看过、想看、读过等记录（含仅自己可见的）导出为 JSON、CSV 或 Markdown。MIT。计划适配。见 [06 §9](research/06-platform-access-and-account-risk.md#9-微博知乎豆瓣与海外社交平台)
- [public-clis/twitter-cli](https://github.com/public-clis/twitter-cli)：X 的终端命令行，读时间线、书签、用户帖子和文章，输出 YAML 或 JSON 给 Agent 用，附 Skill。Apache-2.0。计划适配。见 [06 §9](research/06-platform-access-and-account-risk.md#9-微博知乎豆瓣与海外社交平台)
- [sean1832/pinterest-dl](https://github.com/sean1832/pinterest-dl)：从 Pinterest 画板、单个 Pin、分区或搜索结果下载图片和视频的命令行与库，可把描述写进图片元数据。Apache-2.0。计划适配。见 [06 §9](research/06-platform-access-and-account-risk.md#9-微博知乎豆瓣与海外社交平台)
- [Serene-Arc/bulk-downloader-for-reddit](https://github.com/Serene-Arc/bulk-downloader-for-reddit)：Reddit 批量下载与归档工具，可按用户、子版、收藏、点赞下载帖子媒体，或把帖子和评论存成 JSON、YAML。GPL-3.0。计划适配。见 [06 §9](research/06-platform-access-and-account-risk.md#9-微博知乎豆瓣与海外社交平台)
- [steipete/birdclaw](https://github.com/steipete/birdclaw)：把 X 官方归档导入本地 SQLite，提供全文搜索、Web 界面、命令行和只读 MCP，需要时经 X 官方命令行增量同步。MIT。计划适配。见 [06 §9](research/06-platform-access-and-account-risk.md#9-微博知乎豆瓣与海外社交平台)
- [xdevplatform/xurl](https://github.com/xdevplatform/xurl)：X 官方出的 X API 命令行，支持多种 OAuth，带书签、点赞列表等快捷命令和 `SKILL.md`。MIT。计划适配。见 [06 §9](research/06-platform-access-and-account-risk.md#9-微博知乎豆瓣与海外社交平台)

**参考设计（19）**

- [alexknowshtml/smaug](https://github.com/alexknowshtml/smaug)：把 X 书签按日期归档成 Markdown，展开短链、抓取链接文章，再交给 Claude Code 分类分析；依赖的 bird 已无法安装，项目基本停摆。MIT。参考设计。见 [06 §9](research/06-platform-access-and-account-risk.md#9-微博知乎豆瓣与海外社交平台)
- [dogsheep/twitter-to-sqlite](https://github.com/dogsheep/twitter-to-sqlite)：Dogsheep 系列，把 X 官方归档里的每个数据文件导入 SQLite 的命令行。Apache-2.0。参考设计。见 [06 §9](research/06-platform-access-and-account-risk.md#9-微博知乎豆瓣与海外社交平台)
- [dotWee/py-ig2raindrop-cli](https://github.com/dotWee/py-ig2raindrop-cli)：把 Instagram 收藏同步到 Raindrop.io 的命令行，也能导入官方导出的 `saved_posts.json` 并映射收藏集。WTFPL。参考设计。见 [06 §9](research/06-platform-access-and-account-risk.md#9-微博知乎豆瓣与海外社交平台)
- [Doubak/doubak-extension](https://github.com/Doubak/doubak-extension)：在本机限速抓取自己的豆瓣网页存成 WARC 归档，再离线转成结构化数据、NeoDB 导入包或 Markdown 站点的浏览器扩展。Apache-2.0。参考设计。见 [06 §9](research/06-platform-access-and-account-risk.md#9-微博知乎豆瓣与海外社交平台)
- [doufen-org/tofu](https://github.com/doufen-org/tofu)：把豆瓣账号的书影音标记、评论、日记、豆列、广播备份到浏览器本地的 Chrome 扩展，2026-03 起有报告无法备份，修复未合并，处于停更状态。MIT。参考设计。见 [06 §9](research/06-platform-access-and-account-risk.md#9-微博知乎豆瓣与海外社交平台)
- [Geralt-TYH/obsidian-zhihu-crawler](https://github.com/Geralt-TYH/obsidian-zhihu-crawler)：把知乎收藏夹里的回答和文章逐条转成 Markdown、图片存到本地的命令行和图形界面工具。无许可证文件。参考设计。见 [06 §9](research/06-platform-access-and-account-risk.md#9-微博知乎豆瓣与海外社交平台)
- [klarkxy/zhihu-search](https://github.com/klarkxy/zhihu-search)：把知乎开放平台官方接口封装成 Skill、命令行、MCP 和 OpenAPI 服务，含本人近期收藏、收藏夹列表与收藏夹内容。NOASSERTION（SATA 2.0，即 MIT 条款外加先 star 并致谢作者的要求）。参考设计。见 [06 §9](research/06-platform-access-and-account-risk.md#9-微博知乎豆瓣与海外社交平台)
- [meizuflux/scrollback](https://github.com/meizuflux/scrollback)：纯浏览器端的 Instagram 导出分析器，载入官方 ZIP 浏览帖子、快拍、私信、喜欢和关注关系，可导出 SQLite。MIT。参考设计。见 [06 §9](research/06-platform-access-and-account-risk.md#9-微博知乎豆瓣与海外社交平台)
- [moonlin1213/cove-douban-mcp](https://github.com/moonlin1213/cove-douban-mcp)：默认只读的豆瓣 MCP 服务，经 OpenCLI 浏览器桥读取用户 Chrome 里能看到的看过、想看、在看、影评和豆列。Apache-2.0。参考设计。见 [06 §9](research/06-platform-access-and-account-risk.md#9-微博知乎豆瓣与海外社交平台)
- [prinsss/twitter-web-exporter](https://github.com/prinsss/twitter-web-exporter)：在 X 网页版里把推文、书签、点赞、列表、关注等导出为 JSON、CSV 或 HTML 的油猴脚本，只截获页面自己的请求。MIT。参考设计。见 [06 §9](research/06-platform-access-and-account-risk.md#9-微博知乎豆瓣与海外社交平台)
- [soberhacker/obsidian-telegram-sync](https://github.com/soberhacker/obsidian-telegram-sync)：把发给自建 Telegram 机器人的消息和文件写进 Obsidian 的插件，可用模板生成笔记、转写语音，2024-10 之后没有推送。AGPL-3.0。参考设计。见 [06 §9](research/06-platform-access-and-account-risk.md#9-微博知乎豆瓣与海外社交平台)
- [soyaine/jocker-chrome-extension](https://github.com/soyaine/jocker-chrome-extension)：把自己的即刻动态按圈子、年份、标签归类浏览，可查看收藏并按主题导出 CSV 的 Chrome 扩展。MIT。参考设计。见 [06 §9](research/06-platform-access-and-account-risk.md#9-微博知乎豆瓣与海外社交平台)
- [sytelus/xarchive](https://github.com/sytelus/xarchive)：导出自己全部 X 书签并保留书签文件夹、在本地累积存档的 Chrome 扩展，附离线查看器。MIT。参考设计。见 [06 §9](research/06-platform-access-and-account-risk.md#9-微博知乎豆瓣与海外社交平台)
- [tairqaldy/resurfly](https://github.com/tairqaldy/resurfly)：把 Instagram 收藏变成可搜索笔记的自托管应用，转写 Reels、读关键帧和轮播图，生成摘要标签，可导出 Obsidian 库。MIT。参考设计。见 [06 §9](research/06-platform-access-and-account-risk.md#9-微博知乎豆瓣与海外社交平台)
- [Tendo33/zhihu-md](https://github.com/Tendo33/zhihu-md)：把当前打开的知乎专栏文章、回答、问题页、首页推荐和热榜导出为带 YAML 头 Markdown 的 Chrome 扩展。MIT。参考设计。见 [06 §9](research/06-platform-access-and-account-risk.md#9-微博知乎豆瓣与海外社交平台)
- [timhutton/twitter-archive-parser](https://github.com/timhutton/twitter-archive-parser)：把 X 官方归档里的推文和私信转成 Markdown 与 HTML、还原短链的 Python 脚本，2022-11 之后没有推送。GPL-3.0。参考设计。见 [06 §9](research/06-platform-access-and-account-risk.md#9-微博知乎豆瓣与海外社交平台)
- [Wanxp/obsidian-douban](https://github.com/Wanxp/obsidian-douban)：按条目导入豆瓣电影、书、音乐、游戏、日记，并同步自己标记过的书影音的 Obsidian 插件。GPL-3.0。参考设计。见 [06 §9](research/06-platform-access-and-account-risk.md#9-微博知乎豆瓣与海外社交平台)
- [xavdid/reddit-user-to-sqlite](https://github.com/xavdid/reddit-user-to-sqlite)：把本人 Reddit 帖子和评论存进 SQLite，也能读 Reddit 官方数据请求导出的 CSV 并补全内容。MIT。参考设计。见 [06 §9](research/06-platform-access-and-account-risk.md#9-微博知乎豆瓣与海外社交平台)
- [zimya/zhihu_obsidian](https://github.com/zimya/zhihu_obsidian)：Obsidian 插件，主要把笔记发布成知乎文章或回答，也能把知乎推荐、关注动态和热榜内容拉进笔记库。0BSD。参考设计。见 [06 §9](research/06-platform-access-and-account-risk.md#9-微博知乎豆瓣与海外社交平台)

<a id="视觉与-ui-自动化"></a>
## 视觉与 UI 自动化

**计划适配（11）**

- [appium/WebDriverAgent](https://github.com/appium/WebDriverAgent)：以 XCTest 方式跑在 iPhone 上的 WebDriver 服务端，提供截图、控件树、点按和输入接口。NOASSERTION（LICENSE 为 BSD 3-Clause 文本）。计划适配。见 [06 §10](research/06-platform-access-and-account-risk.md#10-视觉与-ui-自动化)
- [danielpaulus/go-ios](https://github.com/danielpaulus/go-ios)：纯 Go 实现的 iOS 设备工具集和库，在 macOS、Linux、Windows 上都能截图、拉起 WebDriverAgent、做无障碍检查。MIT。计划适配。见 [06 §10](research/06-platform-access-and-account-risk.md#10-视觉与-ui-自动化)
- [Genymobile/scrcpy](https://github.com/Genymobile/scrcpy)：经 ADB 把 Android 手机的画面和声音投到电脑上，可用键鼠操作、可录屏，手机端不装 App。Apache-2.0。计划适配。见 [06 §10](research/06-platform-access-and-account-risk.md#10-视觉与-ui-自动化)
- [jfarcand/mirroir-mcp](https://github.com/jfarcand/mirroir-mcp)：通过 macOS「iPhone 镜像」窗口控制真实 iPhone 的 MCP 服务，截图后用 Apple Vision 做 OCR，用系统输入事件点按滑动。Apache-2.0。计划适配。见 [06 §10](research/06-platform-access-and-account-risk.md#10-视觉与-ui-自动化)
- [makeusabrew/audiotee](https://github.com/makeusabrew/audiotee)：用 Core Audio tap 录系统声音、按块把 PCM 写到标准输出的 macOS 命令行，可只录或排除指定进程。无许可证文件（README 写 MIT，仓库无 LICENSE 文件）。计划适配。见 [06 §10](research/06-platform-access-and-account-risk.md#10-视觉与-ui-自动化)
- [mobile-dev-inc/Maestro](https://github.com/mobile-dev-inc/Maestro)：用 YAML 写移动端和网页端端到端流程的测试框架，经系统无障碍层驱动 App，命令行内置 MCP 服务。Apache-2.0（Maestro Studio 与云服务不开源）。计划适配。见 [06 §10](research/06-platform-access-and-account-risk.md#10-视觉与-ui-自动化)
- [mobile-next/mobile-mcp](https://github.com/mobile-next/mobile-mcp)：面向 iOS、Android 真机和模拟器的 MCP 服务，优先读原生无障碍树，必要时退回截图和坐标。Apache-2.0。计划适配。见 [06 §10](research/06-platform-access-and-account-risk.md#10-视觉与-ui-自动化)
- [openatx/uiautomator2](https://github.com/openatx/uiautomator2)：经 ADB 在 Android 设备上运行 UiAutomator 服务，读取控件树、截图、点按和输入的 Python 库。MIT。计划适配。见 [06 §10](research/06-platform-access-and-account-risk.md#10-视觉与-ui-自动化)
- [openclaw/Peekaboo](https://github.com/openclaw/Peekaboo)：macOS 命令行和菜单栏应用，可截取任意窗口、检查无障碍树、执行原生 UI 自动化，也提供 MCP 服务。MIT。计划适配。见 [06 §10](research/06-platform-access-and-account-risk.md#10-视觉与-ui-自动化)
- [web-infra-dev/midscene](https://github.com/web-infra-dev/midscene)：视觉驱动的 UI 自动化 SDK，截图加视觉模型定位和操作，支持网页、Android、iOS、鸿蒙和桌面。MIT。计划适配。见 [06 §10](research/06-platform-access-and-account-risk.md#10-视觉与-ui-自动化)
- [zai-org/Open-AutoGLM](https://github.com/zai-org/Open-AutoGLM)：智谱开源的手机 Agent 框架和 9B 视觉模型，截图交给模型理解界面，再经 ADB、HDC 或 WDA 点按输入。Apache-2.0。计划适配。见 [06 §10](research/06-platform-access-and-account-risk.md#10-视觉与-ui-自动化)

**参考设计（4）**

- [danzey0425-lgtm/web-image-sourcing](https://github.com/danzey0425-lgtm/web-image-sourcing)：用真实 Edge 浏览器加桌面操控，从小红书、抖音、B站收集图片素材，经视觉质检后生成图鉴和 PDF 的 Agent Skill。MIT。参考设计。见 [06 §10](research/06-platform-access-and-account-risk.md#10-视觉与-ui-自动化)
- [google-gemini/computer-use-preview](https://github.com/google-gemini/computer-use-preview)：Google 给 Gemini Computer Use 模型的参考实现，模型看截图给出点击和输入，由本机 Playwright 浏览器或 Browserbase 云浏览器执行；用户给的视频号链接演示的就是这个模型，对应到仓库是推断。Apache-2.0。参考设计。见 [06 §3](research/06-platform-access-and-account-risk.md#3-用户点名的项目)
- [insidegui/AudioCap](https://github.com/insidegui/AudioCap)：macOS 14.4 起 Core Audio 进程 tap 的示例代码，按进程或全系统录制其他 App 发出的声音。BSD-2-Clause。参考设计。见 [06 §10](research/06-platform-access-and-account-risk.md#10-视觉与-ui-自动化)
- [screenpipe/screenpipe](https://github.com/screenpipe/screenpipe)：持续记录屏幕和音频的本地电脑记忆，读系统无障碍树或做 OCR，用本地 Whisper 转写，可搜索并经 MCP 交给 AI。NOASSERTION（Screenpipe Commercial License，源码可见，商业使用需购买许可）。参考设计。见 [06 §10](research/06-platform-access-and-account-risk.md#10-视觉与-ui-自动化)

<a id="agent-接入层"></a>
## Agent 接入层

**计划适配（4）**

- [browserutils/kooky](https://github.com/browserutils/kooky)：Go 写的浏览器 cookie 读取库，覆盖 Chrome、Firefox、Safari 等多种浏览器。MIT。计划适配。见 [06 §11](research/06-platform-access-and-account-risk.md#11-agent-接入层)
- [chromedp/chromedp](https://github.com/chromedp/chromedp)：Go 写的 Chrome DevTools 协议驱动，可启动自己的 Chrome，也可连接已运行的浏览器。MIT。计划适配。见 [06 §11](research/06-platform-access-and-account-risk.md#11-agent-接入层)
- [jackwener/OpenCLI](https://github.com/jackwener/OpenCLI)：借浏览器扩展和本地守护进程驱动用户已登录的 Chrome，把一百多个网站变成默认输出 JSON 的命令行，附 Agent Skill。Apache-2.0。计划适配。见 [06 §11](research/06-platform-access-and-account-risk.md#11-agent-接入层)
- [Panniantong/Agent-Reach](https://github.com/Panniantong/Agent-Reach)：面向 AI Agent 的统一命令行，把 B站、YouTube、小红书、X、Reddit、小宇宙等平台的搜索与按链接读取收成统一的渠道加动作，每个动作按序尝试多个上游工具。MIT（传递依赖许可各异，专用浏览器 CloakBrowser 的二进制为专有许可）。计划适配。见 [06 §3](research/06-platform-access-and-account-risk.md#3-用户点名的项目)

**参考设计（5）**

- [easychen/CookieCloud](https://github.com/easychen/CookieCloud)：浏览器扩展加自托管服务端，把浏览器 cookie 和 localStorage 端到端加密后同步到自己的服务器。GPL-3.0。参考设计。见 [06 §11](research/06-platform-access-and-account-risk.md#11-agent-接入层)
- [kairi003/Get-cookies.txt-LOCALLY](https://github.com/kairi003/Get-cookies.txt-LOCALLY)：只在本地导出 cookies.txt 的浏览器扩展，支持 Netscape 和 JSON 格式，不向外发送数据。MIT。参考设计。见 [06 §11](research/06-platform-access-and-account-risk.md#11-agent-接入层)
- [microsoft/playwright](https://github.com/microsoft/playwright)：微软的浏览器自动化与测试框架，驱动 Chromium、Firefox、WebKit。Apache-2.0。参考设计。见 [06 §11](research/06-platform-access-and-account-risk.md#11-agent-接入层)
- [sigcli/sigcli](https://github.com/sigcli/sigcli)：给 Agent 用的登录与凭据代理，用户在真实浏览器里登录一次，之后由它加密保存并注入凭据，Agent 看不到原文。MIT。参考设计。见 [06 §11](research/06-platform-access-and-account-risk.md#11-agent-接入层)
- [ZhongQuinnKing/caishi](https://github.com/ZhongQuinnKing/caishi)：在 OpenCLI 之上补抖音、小红书读取命令的中文平台读取包，附平台体检和独立 MCP，主张只读、单条、低频。NOASSERTION（代码 MIT，文档与平台清单为 CC BY-NC-ND 4.0，另声明商用须授权）。参考设计。见 [06 §11](research/06-platform-access-and-account-risk.md#11-agent-接入层)

<a id="转换转写字幕与-ocr"></a>
## 转换：转写、字幕与 OCR

**计划适配（36）**

- [ahmetoner/whisper-asr-webservice](https://github.com/ahmetoner/whisper-asr-webservice)：Whisper 的 FastAPI 服务，`POST /asr` 直接返回 srt、vtt 等格式，支持词级时间戳、VAD 和说话人分离。MIT。计划适配。见 [05 §2.4](research/05-note-apps-and-plugins.md#24-独立工具与服务)
- [argmaxinc/argmax-oss-swift](https://github.com/argmaxinc/argmax-oss-swift)：Argmax 的 Apple 平台端侧语音工具包，WhisperKit 用 Core ML 在 Mac 和 iPhone 上跑 Whisper，附命令行和 OpenAI 兼容本地服务。MIT。计划适配。见 [06 §12](research/06-platform-access-and-account-risk.md#12-转换转写字幕与-ocr)
- [brittain9/speech-kit-obsidian-plugin](https://github.com/brittain9/speech-kit-obsidian-plugin)：本地听写的 Obsidian 插件，推理跑在单独发版的 Rust sidecar 里，经 stdin、stdout 通信。MIT。计划适配。见 [05 §2.1](research/05-note-apps-and-plugins.md#21-转写插件)
- [bytefer/macos-vision-ocr](https://github.com/bytefer/macos-vision-ocr)：调用 Apple Vision 框架的命令行 OCR，支持批量图片，输出带文字位置和置信度的 JSON。MIT。计划适配。见 [06 §12](research/06-platform-access-and-account-risk.md#12-转换转写字幕与-ocr)
- [chidiwilliams/buzz](https://github.com/chidiwilliams/buzz)：离线转写和翻译的桌面应用，后端可选 Whisper、whisper.cpp、faster-whisper，能批量导出 TXT、SRT、VTT，带命令行。MIT。计划适配。见 [06 §12](research/06-platform-access-and-account-risk.md#12-转换转写字幕与-ocr)
- [datalab-to/indra](https://github.com/datalab-to/indra)：Datalab 开源的 CPU OCR 工具和小模型，把 PDF 与图片转成 Markdown、纯文本或带框 JSON，支持 80 多种语言。Apache-2.0（模型权重为修改版 OpenRAIL-M）。计划适配。见 [06 §3](research/06-platform-access-and-account-risk.md#3-用户点名的项目)
- [datalab-to/marker](https://github.com/datalab-to/marker)：把 PDF、图片、Office 文档和 EPUB 转成 Markdown、JSON、HTML 的工具，内部用 Surya 做 OCR 和版面分析。Apache-2.0（权重为修改版 OpenRAIL-M）。计划适配。见 [06 §12](research/06-platform-access-and-account-risk.md#12-转换转写字幕与-ocr)
- [datalab-to/surya](https://github.com/datalab-to/surya)：Datalab 的 OCR 模型和工具包，做文字行检测、识别、版面分析、阅读顺序和表格识别。Apache-2.0（权重为修改版 OpenRAIL-M）。计划适配。见 [06 §12](research/06-platform-access-and-account-risk.md#12-转换转写字幕与-ocr)
- [docling-project/docling](https://github.com/docling-project/docling)：IBM 发起、现属 LF AI & Data 的文档转换工具，解析 PDF、Office、HTML、音频和视频，输出 Markdown、JSON、WebVTT。MIT。计划适配。见 [06 §12](research/06-platform-access-and-account-risk.md#12-转换转写字幕与-ocr)
- [finnvoor/yap](https://github.com/finnvoor/yap)：调用 macOS 26 系统语音框架 SpeechAnalyzer 的转写命令行，输出 txt、srt、vtt、json。CC0-1.0。计划适配。见 [06 §12](research/06-platform-access-and-account-risk.md#12-转换转写字幕与-ocr)
- [FireRedTeam/FireRedASR2S](https://github.com/FireRedTeam/FireRedASR2S)：小红书 FireRed 团队的第二代中文语音识别系统，覆盖普通话、二十多种方言和中英混说，输出句级和字级时间戳。Apache-2.0。计划适配。见 [06 §12](research/06-platform-access-and-account-risk.md#12-转换转写字幕与-ocr)
- [ggml-org/whisper.cpp](https://github.com/ggml-org/whisper.cpp)：C/C++ 实现的 Whisper 推理，单个可执行文件在 CPU、Metal、CUDA、Vulkan 上转写音频，自带命令行和 HTTP 服务。MIT。计划适配。见 [06 §12](research/06-platform-access-and-account-risk.md#12-转换转写字幕与-ocr)
- [hiroi-sora/Umi-OCR](https://github.com/hiroi-sora/Umi-OCR)：免费离线的 OCR 桌面软件，内置 PaddleOCR、RapidOCR 引擎，提供本地 HTTP 接口和命令行，只有 Windows 和 Linux 版。MIT。计划适配。见 [06 §12](research/06-platform-access-and-account-risk.md#12-转换转写字幕与-ocr)
- [jianchang512/pyvideotrans](https://github.com/jianchang512/pyvideotrans)：视频翻译配音工具，识别语音生成字幕、翻译、配音再合成新视频，内置几十种本地和云端识别通道。GPL-3.0。计划适配。见 [06 §12](research/06-platform-access-and-account-risk.md#12-转换转写字幕与-ocr)
- [k2-fsa/sherpa-onnx](https://github.com/k2-fsa/sherpa-onnx)：新一代 Kaldi 的 ONNX 推理框架，一套 C++ 运行时跑 Paraformer、SenseVoice、Whisper 等离线和流式识别，另有 VAD 和说话人分离。Apache-2.0。计划适配。见 [06 §12](research/06-platform-access-and-account-risk.md#12-转换转写字幕与-ocr)
- [kaixxx/noScribe](https://github.com/kaixxx/noScribe)：面向访谈整理的本地转写桌面应用，用 faster-whisper 转写、pyannote 区分说话人，自带校对编辑器，也能无界面批处理。GPL-3.0。计划适配。见 [06 §12](research/06-platform-access-and-account-risk.md#12-转换转写字幕与-ocr)
- [LuckLittleBoy/SenseVoice-OneApi](https://github.com/LuckLittleBoy/SenseVoice-OneApi)：把 SenseVoice 包成 OpenAI 兼容转写接口的 Docker 服务，只作用户自己运行的后端。无许可证文件。计划适配。见 [05 §2.4](research/05-note-apps-and-plugins.md#24-独立工具与服务)
- [m-bain/whisperX](https://github.com/m-bain/whisperX)：在 faster-whisper 转写结果上做词级强制对齐和说话人分离，输出带词级时间戳的字幕和 JSON。BSD-2-Clause。计划适配。见 [06 §12](research/06-platform-access-and-account-risk.md#12-转换转写字幕与-ocr)
- [microsoft/markitdown](https://github.com/microsoft/markitdown)：微软的 Python 工具，把 PDF、Office、HTML、图片、音频等文件转成给大模型用的 Markdown，附 MCP 服务和插件机制。MIT。计划适配。见 [06 §12](research/06-platform-access-and-account-risk.md#12-转换转写字幕与-ocr)
- [modelscope/FunASR](https://github.com/modelscope/FunASR)：阿里通义实验室的语音识别工具包，含 Paraformer、SenseVoice、Fun-ASR-Nano 及 VAD、标点模型，另有免 Python 的单文件运行时。MIT（模型权重按各自模型卡）。计划适配。见 [06 §12](research/06-platform-access-and-account-risk.md#12-转换转写字幕与-ocr)
- [Nexround/obsidian-text-lens](https://github.com/Nexround/obsidian-text-lens)：Obsidian OCR 插件，系统 OCR 走独立 helper 进程、经 stdio 交换 JSONL，macOS helper 安装前校验 SHA-256。MIT。计划适配。见 [05 §2.2](research/05-note-apps-and-plugins.md#22-ocr-插件)
- [obsidian-power-plugins/power-extract](https://github.com/obsidian-power-plugins/power-extract)：Windows 上的 Obsidian OCR 插件，常驻 PowerShell 进程调用系统 OCR，按行收发请求，接口与 Text Extractor 同形。MIT。计划适配。见 [05 §2.2](research/05-note-apps-and-plugins.md#22-ocr-插件)
- [opendatalab/MinerU](https://github.com/opendatalab/MinerU)：OpenDataLab 的文档解析工具，把 PDF、图片、Office、EPUB 等转成 Markdown 和 JSON，分四档精度。NOASSERTION（MinerU Open Source License，Apache-2.0 加附加条款）。计划适配。见 [06 §12](research/06-platform-access-and-account-risk.md#12-转换转写字幕与-ocr)
- [QwenAudio/SenseVoice](https://github.com/QwenAudio/SenseVoice)：多语种语音理解小模型 SenseVoiceSmall 的仓库，识别中、粤、英、日、韩并给出情感和声音事件标签，提供 ONNX 导出。MIT（SenseVoiceSmall 权重用 FunASR 模型许可，可商用需署名）。计划适配。见 [06 §12](research/06-platform-access-and-account-risk.md#12-转换转写字幕与-ocr)
- [QwenLM/Qwen3-ASR](https://github.com/QwenLM/Qwen3-ASR)：通义千问的语音识别模型和配套的强制对齐模型，支持 52 种语言和方言，附 Python 工具包和 OpenAI 兼容服务。Apache-2.0。计划适配。见 [06 §12](research/06-platform-access-and-account-risk.md#12-转换转写字幕与-ocr)
- [SoMarkAI/OXR](https://github.com/SoMarkAI/OXR)：SoMark 开源的 1.3B 参数文档解析模型和配套服务，把 PDF 与文档图片转成 Markdown 和带版面坐标的 JSON。NOASSERTION（OXR License，以 Apache-2.0 为基础加附加条款）。计划适配。见 [06 §3](research/06-platform-access-and-account-risk.md#3-用户点名的项目)
- [speaches-ai/speaches](https://github.com/speaches-ai/speaches)：提供 OpenAI 兼容转写与翻译接口的服务，原名 faster-whisper-server，模型按请求动态加载。MIT。计划适配。见 [05 §2.4](research/05-note-apps-and-plugins.md#24-独立工具与服务)
- [steipete/summarize](https://github.com/steipete/summarize)：命令行加浏览器侧边栏，把网页、文件、YouTube、播客和音视频提取成干净文本再生成摘要，也可只提取并输出 JSON。MIT。计划适配。见 [06 §12](research/06-platform-access-and-account-risk.md#12-转换转写字幕与-ocr)
- [SWHL/RapidVideOCR](https://github.com/SWHL/RapidVideOCR)：读取 VideoSubFinder 导出的字幕关键帧，用 RapidOCR 识别后生成 srt、ass、txt 的硬字幕提取工具。Apache-2.0。计划适配。见 [06 §12](research/06-platform-access-and-account-risk.md#12-转换转写字幕与-ocr)
- [SYSTRAN/faster-whisper](https://github.com/SYSTRAN/faster-whisper)：基于 CTranslate2 重新实现的 Whisper 推理库，速度更快、显存更省，支持词级时间戳和 VAD 过滤。MIT。计划适配。见 [06 §12](research/06-platform-access-and-account-risk.md#12-转换转写字幕与-ocr)
- [tesseract-ocr/tesseract](https://github.com/tesseract-ocr/tesseract)：开源 OCR 引擎和命令行，装上 `chi_sim` 等语言包即可识别中文。Apache-2.0。计划适配。见 [05 §6.5](research/05-note-apps-and-plugins.md#65-怎么复用现有插件与工具)
- [thewh1teagle/vibe](https://github.com/thewh1teagle/vibe)：Tauri 写的跨平台离线转写桌面应用，后端 vibe-server 可单独运行，提供 OpenAI 兼容转写接口。MIT。计划适配。见 [06 §12](research/06-platform-access-and-account-risk.md#12-转换转写字幕与-ocr)
- [timminator/VideOCR](https://github.com/timminator/VideOCR)：带图形界面的硬字幕提取工具，可在时间轴上框选字幕区域，默认用本地 PaddleOCR 识别。MIT。计划适配。见 [06 §12](research/06-platform-access-and-account-risk.md#12-转换转写字幕与-ocr)
- [WEIFENG2333/VideoCaptioner](https://github.com/WEIFENG2333/VideoCaptioner)：字幕全流程工具，转写、用大模型断句校正、翻译、配音、压字幕，有桌面版、命令行和 Agent Skill。GPL-3.0。计划适配。见 [06 §12](research/06-platform-access-and-account-risk.md#12-转换转写字幕与-ocr)
- [wendy7756/AI-Video-Transcriber](https://github.com/wendy7756/AI-Video-Transcriber)：自部署的视频与播客转写工具，链接交给 yt-dlp 取字幕或音频，没有字幕时用本地 faster-whisper 转写，可选大模型校对、翻译和摘要。Apache-2.0。计划适配。见 [06 §3](research/06-platform-access-and-account-risk.md#3-用户点名的项目)
- [YaoFANGUK/video-subtitle-extractor](https://github.com/YaoFANGUK/video-subtitle-extractor)：在本机提取视频里压制的硬字幕并生成 SRT，先找出字幕帧，再用 PaddleOCR 识别。Apache-2.0（依赖的 VideoSubFinder 为 GPLv2，需用户自装）。计划适配。见 [06 §12](research/06-platform-access-and-account-risk.md#12-转换转写字幕与-ocr)

**参考设计（17）**

- [chenxuan520/siyuan-plugin-voice2text](https://github.com/chenxuan520/siyuan-plugin-voice2text)：思源语音转文字插件，经思源内核的出站代理联网，是宿主代插件联网的现成例子。MIT。参考设计。见 [05 §2.1](research/05-note-apps-and-plugins.md#21-转写插件)
- [FlorianZierer/speech2text-transcription-plugin-siyuan](https://github.com/FlorianZierer/speech2text-transcription-plugin-siyuan)：思源转写插件，接任意 OpenAI 兼容转写端点，端点可填本地服务，预览确认后插入。MIT。参考设计。见 [05 §2.1](research/05-note-apps-and-plugins.md#21-转写插件)
- [Huanshere/VideoLingo](https://github.com/Huanshere/VideoLingo)：一键给视频做字幕切分、翻译、对齐和配音的工具，默认本地识别用 Qwen3-ASR 加强制对齐模型。Apache-2.0。参考设计。见 [06 §12](research/06-platform-access-and-account-risk.md#12-转换转写字幕与-ocr)
- [JimLiu/baocut](https://github.com/JimLiu/baocut)：以可编辑时间线为中心的 AI 视频智能体，做转写、字幕、翻译、配音和剪口播，桌面、CLI、MCP 和网页共用一个本地运行时。NOASSERTION（BaoCut Community License 1.0，源码可见，商业分发需另签许可）。参考设计。见 [06 §12](research/06-platform-access-and-account-risk.md#12-转换转写字幕与-ocr)
- [jjmaldonis/obsidian-audio-notes](https://github.com/jjmaldonis/obsidian-audio-notes)：把转写结果存成库里的 JSON 或 SRT 附件、笔记只引用它的 Obsidian 插件，2023-05 之后没有推送。MIT。参考设计。见 [05 §2.1](research/05-note-apps-and-plugins.md#21-转写插件)
- [lanbinleo/bili2text](https://github.com/lanbinleo/bili2text)：把 B站 视频转成文字稿的命令行工具，抽音频后用 Whisper、SenseVoice 本地转写或云端转写。MIT。参考设计。见 [06 §12](research/06-platform-access-and-account-risk.md#12-转换转写字幕与-ocr)
- [logancyang/obsidian-copilot](https://github.com/logancyang/obsidian-copilot)：Obsidian 的 AI 助手插件，密钥存系统钥匙串，本地检索交给独立服务并用服务发现文件找端口，Agent 模式走 ACP。AGPL-3.0。参考设计。见 [05 §2.3](research/05-note-apps-and-plugins.md#23-摘要与-llm-插件)
- [mikealicea/obsidian-scribe](https://github.com/mikealicea/obsidian-scribe)：Obsidian 录音转写加摘要插件，支持 OpenAI 兼容端点和多家云端 ASR 与大模型，按模板生成笔记。MIT。参考设计。见 [05 §2.1](research/05-note-apps-and-plugins.md#21-转写插件)
- [modelscope/FunClip](https://github.com/modelscope/FunClip)：用 Paraformer 识别语音后按文字或说话人选段剪视频的本地工具，同时导出全片和片段的 SRT。MIT（Paraformer 等权重按 FunASR 模型许可）。参考设计。见 [06 §12](research/06-platform-access-and-account-risk.md#12-转换转写字幕与-ocr)
- [mssoftjp/obsidian-ai-transcriber](https://github.com/mssoftjp/obsidian-ai-transcriber)：接 OpenAI 转写的 Obsidian 插件，密钥用系统加密存储，带去幻觉清洗规则。MIT。参考设计。见 [05 §2.1](research/05-note-apps-and-plugins.md#21-转写插件)
- [nikdanilov/whisper-obsidian-plugin](https://github.com/nikdanilov/whisper-obsidian-plugin)：Obsidian 转写插件，调用 OpenAI 兼容的转写端点，密钥存在 Obsidian 的安全存储里。MIT。参考设计。见 [05 §2.1](research/05-note-apps-and-plugins.md#21-转写插件)
- [PaddlePaddle/PaddleOCR](https://github.com/PaddlePaddle/PaddleOCR)：百度飞桨的 OCR 与文档解析工具包，含 PP-OCRv6 文字检测识别、PP-StructureV3 版面解析和 PaddleOCR-VL 文档模型。Apache-2.0。参考设计。见 [06 §12](research/06-platform-access-and-account-risk.md#12-转换转写字幕与-ocr)
- [pfrankov/obsidian-ai-providers](https://github.com/pfrankov/obsidian-ai-providers)：Obsidian 的共享模型 provider 层，一处配置供多个插件调用，服务 API 带版本号，但会把含密钥的配置交给每个调用方。MIT。参考设计。见 [05 §2.3](research/05-note-apps-and-plugins.md#23-摘要与-llm-插件)
- [RapidAI/RapidOCR](https://github.com/RapidAI/RapidOCR)：把 PaddleOCR 模型转成 ONNX 等格式、用 ONNX Runtime、OpenVINO 等引擎运行的多语言 OCR 工具包。Apache-2.0。参考设计。见 [06 §12](research/06-platform-access-and-account-risk.md#12-转换转写字幕与-ocr)
- [scambier/obsidian-omnisearch](https://github.com/scambier/obsidian-omnisearch)：Obsidian 搜索插件，图片和 PDF 的文字从 Text Extractor 取，OCR 与检索分成两个插件。GPL-3.0。参考设计。见 [05 §2.2](research/05-note-apps-and-plugins.md#22-ocr-插件)
- [scambier/obsidian-text-extractor](https://github.com/scambier/obsidian-text-extractor)：Obsidian 的图片与 PDF 文字提取插件，按内容哈希缓存结果，对其他插件提供 `extractText(file)` 接口。GPL-3.0。参考设计。见 [05 §2.2](research/05-note-apps-and-plugins.md#22-ocr-插件)
- [zlxlabs/VideoTranscriptAPI](https://github.com/zlxlabs/VideoTranscriptAPI)：自部署的音视频转写服务，提交链接后下载、转写、用大模型校对和总结，结果可推送到企业微信或飞书。NOASSERTION（PolyForm Noncommercial 1.0.0，禁止商业使用）。参考设计。见 [06 §12](research/06-platform-access-and-account-risk.md#12-转换转写字幕与-ocr)

<a id="笔记知识库与收藏归档"></a>
## 笔记、知识库与收藏归档

**计划适配（10）**

- [akosbalasko/yarle](https://github.com/akosbalasko/yarle)：Evernote ENEX 转 Markdown 的工具，保留创建和修改时间、标签、来源链接、笔记间链接和附件。MIT。计划适配。见 [06 §13](research/06-platform-access-and-account-risk.md#13-笔记知识库与收藏归档)
- [gildas-lormeau/SingleFile](https://github.com/gildas-lormeau/SingleFile)：把当前网页连同图片、样式完整存成一个 HTML 文件的浏览器扩展，另有命令行版 single-file-cli。AGPL-3.0。计划适配。见 [06 §13](research/06-platform-access-and-account-risk.md#13-笔记知识库与收藏归档)
- [hc-tec/my-collection-skills](https://github.com/hc-tec/my-collection-skills)：拉取 B站、知乎、小红书收藏结构和内容的一组 Agent Skill 与脚本，视频可下载音频用 Whisper 转写。MIT。计划适配。见 [06 §13](research/06-platform-access-and-account-risk.md#13-笔记知识库与收藏归档)
- [JohannesKaufmann/html-to-markdown](https://github.com/JohannesKaufmann/html-to-markdown)：Go 写的 HTML 转 Markdown 库和命令行，规则以插件扩展，带表格、删除线等插件。MIT。计划适配。见 [06 §13](research/06-platform-access-and-account-risk.md#13-笔记知识库与收藏归档)
- [karlicoss/instapexport](https://github.com/karlicoss/instapexport)：用 Instapaper 官方 API 把本人的文章收藏和高亮导出为 JSON，附读取历次导出的数据访问层。MIT。计划适配。见 [06 §13](research/06-platform-access-and-account-risk.md#13-笔记知识库与收藏归档)
- [markusmobius/go-domdistiller](https://github.com/markusmobius/go-domdistiller)：Chrome 阅读模式所用 DOM Distiller 的 Go 原生移植，提取正文、图片、元数据和翻页链接。MIT（另附上游 DOM Distiller 与 boilerpipe 的许可文件）。计划适配。见 [06 §13](research/06-platform-access-and-account-risk.md#13-笔记知识库与收藏归档)
- [markusmobius/go-trafilatura](https://github.com/markusmobius/go-trafilatura)：trafilatura 的 Go 原生移植，提取正文和标题、作者、日期、语言等元数据，内置两套备选算法。Apache-2.0。计划适配。见 [06 §13](research/06-platform-access-and-account-risk.md#13-笔记知识库与收藏归档)
- [readeck/go-readability（Codeberg）](https://codeberg.org/readeck/go-readability)：Readeck 团队接手维护的 go-readability 分支，v2 兼容 Readability.js 0.6，托管在 Codeberg。MIT。计划适配。见 [06 §13](research/06-platform-access-and-account-risk.md#13-笔记知识库与收藏归档)
- [vzhd1701/evernote-backup](https://github.com/vzhd1701/evernote-backup)：把 Evernote 或印象笔记账号里的全部笔记增量同步到本地 SQLite，再离线导出为 ENEX 的命令行。MIT。计划适配。见 [06 §13](research/06-platform-access-and-account-risk.md#13-笔记知识库与收藏归档)
- [Y2Z/monolith](https://github.com/Y2Z/monolith)：把整个网页连同 CSS、图片、脚本内嵌进单个 HTML 文件的命令行和 Rust 库。CC0-1.0。计划适配。见 [06 §13](research/06-platform-access-and-account-risk.md#13-笔记知识库与收藏归档)

**参考设计（34）**

- [adbar/trafilatura](https://github.com/adbar/trafilatura)：Python 的网页正文与元数据提取工具，也能按站点地图和订阅源发现、下载网页。Apache-2.0（v1.7.0 及更早版本为 GPL-3.0）。参考设计。见 [06 §13](research/06-platform-access-and-account-risk.md#13-笔记知识库与收藏归档)
- [aidenlx/media-extended](https://github.com/aidenlx/media-extended)：Obsidian 的媒体播放与时间戳笔记插件，识别 `media`、`video` 属性和 `#t=` 时间片段链接。无许可证文件（主分支闭源，v3 分支为 MIT）。参考设计。见 [05 §1.1](research/05-note-apps-and-plugins.md#11-obsidian-社区插件)
- [ArchiveBox/ArchiveBox](https://github.com/ArchiveBox/ArchiveBox)：可自托管的网页归档工具，输入网址、浏览器历史或书签导出，用多种抽取器把每个页面存成普通文件夹。MIT。参考设计。见 [06 §13](research/06-platform-access-and-account-risk.md#13-笔记知识库与收藏归档)
- [basicmachines-co/basic-memory](https://github.com/basicmachines-co/basic-memory)：以本地 Markdown 文件为知识库、通过 MCP 让 Agent 读写的笔记工具，SQLite 只做索引。AGPL-3.0。参考设计。见 [06 §13](research/06-platform-access-and-account-risk.md#13-笔记知识库与收藏归档)
- [bryanbans/Notional](https://github.com/bryanbans/Notional)：Obsidian 与 Notion 双向同步插件，两边都改过就停下让用户选一边。GPL-3.0。参考设计。见 [05 §5.9](research/05-note-apps-and-plugins.md#59-读回双向的先例与歇存的做法)
- [community-archive/obsidian-zotero-integration](https://github.com/community-archive/obsidian-zotero-integration)：把 Zotero 导入 Obsidian 的插件，模板里的 persist 区段在重新导入时保留用户写的内容。GPL-3.0。参考设计。见 [05 §5.9](research/05-note-apps-and-plugins.md#59-读回双向的先例与歇存的做法)
- [dogsheep/dogsheep-beta](https://github.com/dogsheep/dogsheep-beta)：Dogsheep 系列的搜索插件，按配置把多个 SQLite 库的内容汇总成一张全文索引，在 Datasette 里分面搜索。Apache-2.0。参考设计。见 [06 §13](research/06-platform-access-and-account-risk.md#13-笔记知识库与收藏归档)
- [dogsheep/evernote-to-sqlite](https://github.com/dogsheep/evernote-to-sqlite)：Dogsheep 系列，把 Evernote 的 ENEX 导出转成 SQLite，并为正文和图片识别文字建全文索引。Apache-2.0。参考设计。见 [06 §13](research/06-platform-access-and-account-risk.md#13-笔记知识库与收藏归档)
- [duanxianpi/zotflow](https://github.com/duanxianpi/zotflow)：Zotero 与 Obsidian 双向同步插件，用 HTML 注释分出可编辑区和本地保留区，按字段三方合并，标记出错时拒绝覆盖。AGPL-3.0。参考设计。见 [05 §5.9](research/05-note-apps-and-plugins.md#59-读回双向的先例与歇存的做法)
- [duzui300/Obsiclip](https://github.com/duzui300/Obsiclip)：Android 分享桥，把任意 App 分享来的文字清理格式、套 Markdown 模板后追加到指定的 Obsidian 笔记。AGPL-3.0。参考设计。见 [06 §13](research/06-platform-access-and-account-risk.md#13-笔记知识库与收藏归档)
- [FlyingWolFox/Netscape-Bookmarks-File-Parser](https://github.com/FlyingWolFox/Netscape-Bookmarks-File-Parser)：Python 的 Netscape 书签 HTML 解析与生成库，把浏览器等导出的书签文件解析成文件夹和书签对象树。MIT。参考设计。见 [06 §13](research/06-platform-access-and-account-risk.md#13-笔记知识库与收藏归档)
- [hadynz/obsidian-kindle-plugin](https://github.com/hadynz/obsidian-kindle-plugin)：把 Kindle 高亮同步进 Obsidian 的插件，每条高亮带块 ID 作锚点，只插入新高亮、不动已有文字。MIT。参考设计。见 [05 §5.9](research/05-note-apps-and-plugins.md#59-读回双向的先例与歇存的做法)
- [ienone/VaultStream](https://github.com/ienone/VaultStream)：自托管的内容收藏存档与分发系统，采集多平台链接和平台收藏夹，存进 SQLite 做全文检索，再按规则推送到 Telegram、QQ。MIT。参考设计。见 [06 §13](research/06-platform-access-and-account-risk.md#13-笔记知识库与收藏归档)
- [joeseesun/qiaomu-clipper](https://github.com/joeseesun/qiaomu-clipper)：Chrome 扩展加可选本机助手，把网页和音视频剪藏进 Obsidian，配播放器、字幕和 AI 对话。MIT。参考设计。见 [06 §13](research/06-platform-access-and-account-risk.md#13-笔记知识库与收藏归档)
- [jsonMartin/readwise-mirror](https://github.com/jsonMartin/readwise-mirror)：把 Readwise 镜像进 Obsidian 的插件，列进 `protectedFields` 的 frontmatter 键不会被覆盖。MIT。参考设计。见 [05 §5.9](research/05-note-apps-and-plugins.md#59-读回双向的先例与歇存的做法)
- [karakeep-app/karakeep](https://github.com/karakeep-app/karakeep)：可自托管的「什么都收」书签应用，抓取网页正文和截图，用大模型自动打标签和摘要，带全文搜索。AGPL-3.0。参考设计。见 [06 §13](research/06-platform-access-and-account-risk.md#13-笔记知识库与收藏归档)
- [karlicoss/bleanser](https://github.com/karlicoss/bleanser)：对同一来源的多份导出做相邻比较，找出内容已被后一份完全包含的旧备份并清理。MIT。参考设计。见 [06 §13](research/06-platform-access-and-account-risk.md#13-笔记知识库与收藏归档)
- [karlicoss/promnesia](https://github.com/karlicoss/promnesia)：浏览器扩展加本地索引器，汇总浏览历史、平台导出和本地笔记里的链接，浏览时提示在哪里见过当前链接。MIT。参考设计。见 [06 §13](research/06-platform-access-and-account-risk.md#13-笔记知识库与收藏归档)
- [kepano/clipper-templates](https://github.com/kepano/clipper-templates)：Obsidian Web Clipper 的示例模板集合，12 个 JSON 模板，没有中文平台的模板。MIT。参考设计。见 [05 §1.1](research/05-note-apps-and-plugins.md#11-obsidian-社区插件)
- [kepano/defuddle](https://github.com/kepano/defuddle)：Obsidian Web Clipper 使用的正文提取库，输出 HTML 或 Markdown，附带 YouTube、B站、X、Reddit 等站点提取器。MIT。参考设计。见 [06 §13](research/06-platform-access-and-account-risk.md#13-笔记知识库与收藏归档)
- [LifeArchiveProject/HistoryHub](https://github.com/LifeArchiveProject/HistoryHub)：个人历史记录中心，同步 B站 观看历史并用扩展导入浏览器记录，存进本地 SQLite 供搜索和按时间查看。MIT。参考设计。见 [06 §13](research/06-platform-access-and-account-risk.md#13-笔记知识库与收藏归档)
- [logseq/logseq](https://github.com/logseq/logseq)：开源大纲笔记软件，OG 版是本地 Markdown 文件，DB 版改用 SQLite，是歇存的导出目标之一。AGPL-3.0。参考设计。见 [05 §5.2](research/05-note-apps-and-plugins.md#52-logseq)
- [mixmark-io/turndown](https://github.com/mixmark-io/turndown)：JS 写的 HTML 转 Markdown 库，规则可扩展，配插件支持表格等。MIT。参考设计。见 [06 §13](research/06-platform-access-and-account-risk.md#13-笔记知识库与收藏归档)
- [MODSetter/SurfSense](https://github.com/MODSetter/SurfSense)：可离线运行的 NotebookLM 替代品，在本机对用户的文件做带引用的问答、生成和编辑。Apache-2.0（`backend/plugins/proprietary/` 为 BSL 1.1）。参考设计。见 [06 §13](research/06-platform-access-and-account-risk.md#13-笔记知识库与收藏归档)
- [mozilla/readability](https://github.com/mozilla/readability)：Firefox 阅读视图使用的正文提取 JS 库，是多数阅读器和剪藏工具的算法来源。Apache-2.0。参考设计。见 [06 §13](research/06-platform-access-and-account-risk.md#13-笔记知识库与收藏归档)
- [obsidianmd/obsidian-clipper](https://github.com/obsidianmd/obsidian-clipper)：Obsidian 官方浏览器剪藏扩展，在用户浏览器里把当前页面按模板存成 Markdown 笔记。MIT。参考设计。见 [06 §13](research/06-platform-access-and-account-risk.md#13-笔记知识库与收藏归档)
- [OLCUBO/obsidian-cubox](https://github.com/OLCUBO/obsidian-cubox)：Cubox 的 Obsidian 单向同步插件，frontmatter 写 id，同 id 已存在就跳过。MIT。参考设计。见 [05 §1.1](research/05-note-apps-and-plugins.md#11-obsidian-社区插件)
- [omnivore-app/obsidian-omnivore](https://github.com/omnivore-app/obsidian-omnivore)：Omnivore 的 Obsidian 同步插件，frontmatter 写稳定 id，单文件模式用区段标记只替换机器区；服务 2024-11 关停后已不可用。MIT。参考设计。见 [05 §1.1](research/05-note-apps-and-plugins.md#11-obsidian-社区插件)
- [Raven7979/cyber-bookhouse](https://github.com/Raven7979/cyber-bookhouse)：一套 Agent Skill，用户把链接发给 Agent 后，在本机整理成带来源、逐字稿和截图的 Markdown 写进 Obsidian。MIT。参考设计。见 [06 §13](research/06-platform-access-and-account-risk.md#13-笔记知识库与收藏归档)
- [readwiseio/obsidian-readwise](https://github.com/readwiseio/obsidian-readwise)：Readwise 官方的 Obsidian 导出插件，按文件哈希判断用户是否改过，改过就只在末尾追加。GPL-3.0。参考设计。见 [05 §1.1](research/05-note-apps-and-plugins.md#11-obsidian-社区插件)
- [siyuan-note/siyuan](https://github.com/siyuan-note/siyuan)：开源的本地优先笔记软件，有本地 HTTP API 和 `custom-*` 块属性，是歇存的导出目标之一。AGPL-3.0。参考设计。见 [05 §5.3](research/05-note-apps-and-plugins.md#53-思源笔记)
- [volcengine/OpenViking](https://github.com/volcengine/OpenViking)：火山引擎开源的 Agent 上下文数据库，把资料、记忆和技能组织成虚拟文件系统，自动生成分层摘要供 Agent 检索。AGPL-3.0（部分目录为 Apache-2.0，README 另有改写与翻译限制）。参考设计。见 [06 §3](research/06-platform-access-and-account-risk.md#3-用户点名的项目)
- [wallabag/wallabag](https://github.com/wallabag/wallabag)：可自托管的稍后读应用，保存网页正文，支持标签、标注和 EPUB、PDF 等格式导出。MIT。参考设计。见 [06 §13](research/06-platform-access-and-account-risk.md#13-笔记知识库与收藏归档)
- [windingwind/zotero-better-notes](https://github.com/windingwind/zotero-better-notes)：Zotero 笔记与 Markdown 双向同步插件，记下两边上次同步的 MD5，两边都改时弹出对比窗口手动合并。AGPL-3.0。参考设计。见 [05 §5.9](research/05-note-apps-and-plugins.md#59-读回双向的先例与歇存的做法)

<a id="规则源与插件分发的先例"></a>
## 规则源与插件分发的先例

**计划适配（2）**

- [DIYgod/RSSHub](https://github.com/DIYgod/RSSHub)：把各平台内容生成 RSS 订阅源的服务，路由覆盖微博、知乎、豆瓣、即刻、B站、小红书、公众号、YouTube 等。AGPL-3.0。计划适配。见 [06 §14](research/06-platform-access-and-account-risk.md#14-规则源生态)
- [fivefilters/ftr-site-config](https://github.com/fivefilters/ftr-site-config)：FiveFilters 维护的站点正文提取规则库，每个域名一个文本文件，用 XPath 指定标题、正文和要删除的元素。NOASSERTION（`LICENSE.txt` 写明公有领域，即 CC0）。计划适配。见 [06 §14](research/06-platform-access-and-account-risk.md#14-规则源生态)

**参考设计（30）**

- [cli/cli](https://github.com/cli/cli)：GitHub 命令行的 gh extensions 机制，扩展就是以 `gh-` 开头的仓库，靠 topic 发现，`gh extension install owner/repo` 直接安装。MIT。参考设计。见 [02 §12.5](research/02-adapter-ecosystems.md#125-gh-extensions)
- [conda-forge/staged-recipes](https://github.com/conda-forge/staged-recipes)：conda-forge 的新包提交仓库，lint 只用只读权限，合并后为每个包建独立的 feedstock 仓库。BSD-3-Clause。参考设计。见 [05 §3.8](research/05-note-apps-and-plugins.md#38-其他先例)
- [creamycake-anime/animeko-subs](https://github.com/creamycake-anime/animeko-subs)：第三方维护的 Animeko 数据源订阅仓库，一个源一个 JSON，脚本合成订阅文件，附源质量评测规范。无许可证文件。参考设计。见 [06 §14](research/06-platform-access-and-account-risk.md#14-规则源生态)
- [DIYgod/RSSHub-Radar](https://github.com/DIYgod/RSSHub-Radar)：RSSHub 的浏览器扩展，读取服务端按域名聚合的 radar 规则，在当前页面上发现可订阅的路由。AGPL-3.0。参考设计。见 [02 §7](research/02-adapter-ecosystems.md#7-rsshub-与-rsshub-radar)
- [docker/cli](https://github.com/docker/cli)：Docker 命令行的 CLI 插件机制，插件是固定目录里以 `docker-` 开头的可执行文件，宿主用 metadata 子命令读它的 JSON 声明。Apache-2.0。参考设计。见 [02 §12.3](research/02-adapter-ecosystems.md#123-docker-cli-plugins)
- [eclipse-openvsx/openvsx](https://github.com/eclipse-openvsx/openvsx)：开放的 VS Code 扩展注册表，命名空间需要认领，发布时自动扫描恶意模式与内嵌密钥。EPL-2.0。参考设计。见 [05 §3.7](research/05-note-apps-and-plugins.md#37-open-vsx)
- [EclipseFdn/publish-extensions](https://github.com/EclipseFdn/publish-extensions)：Open VSX 的代发仓库，另维护供客户端读取的恶意扩展与弃用扩展清单。EPL-2.0。参考设计。见 [05 §3.7](research/05-note-apps-and-plugins.md#37-open-vsx)
- [flathub/flathub](https://github.com/flathub/flathub)：Flathub 的应用提交仓库，审核通过后为每个应用新建独立仓库，之后的更新都在应用自己的仓库里进行。LGPL-2.1。参考设计。见 [05 §3.8](research/05-note-apps-and-plugins.md#38-其他先例)
- [hacs/default](https://github.com/hacs/default)：Home Assistant 社区商店的默认索引，按类型文件加一行提 PR，维护公开的下架清单并写明原因。MIT。参考设计。见 [05 §3.8](research/05-note-apps-and-plugins.md#38-其他先例)
- [hashicorp/go-plugin](https://github.com/hashicorp/go-plugin)：HashiCorp 的 Go 插件框架，子进程在 stdout 写一行握手后改走 RPC，带协议版本、校验和与重新接管。MPL-2.0。参考设计。见 [02 §12.6](research/02-adapter-ecosystems.md#126-hashicorp-go-plugin)
- [Homebrew/brew](https://github.com/Homebrew/brew)：Homebrew 的第三方 tap 就是一个 GitHub 仓库，6.0.0 起非官方 tap 要用户显式信任后才加载。BSD-2-Clause。参考设计。见 [05 §3.6](research/05-note-apps-and-plugins.md#36-homebrew-tap)
- [keiyoushi/extensions-source](https://github.com/keiyoushi/extensions-source)：Mihon 扩展的社区源码仓库，发布仓库 keiyoushi/extensions 的 repo 分支只放索引、store 元数据和 sha256 清单，置顶 issue 公开因法律或维护原因移除的源。Apache-2.0。参考设计。见 [02 §1](research/02-adapter-ecosystems.md#1-tachiyomi-与-mihon-扩展生态)
- [kubernetes-sigs/krew](https://github.com/kubernetes-sigs/krew)：kubectl 插件管理器，YAML 清单按平台列出下载地址与 sha256，索引仓库 krew-index 以 PR 收录插件。Apache-2.0。参考设计。见 [02 §12.4](research/02-adapter-ecosystems.md#124-kubectl-krew)
- [logseq/marketplace](https://github.com/logseq/marketplace)：Logseq 插件市场，fork 后加目录提 PR，人工合并，用 `blocked.json` 屏蔽插件。MIT。参考设计。见 [05 §3.2](research/05-note-apps-and-plugins.md#32-logseq-marketplace)
- [microsoft/DefinitelyTyped-tools](https://github.com/microsoft/DefinitelyTyped-tools)：DefinitelyTyped 的工具集，其中的 mergebot 接收 webhook，owner 回复后以 GitHub App 身份合并。MIT。参考设计。见 [05 §3.8](research/05-note-apps-and-plugins.md#38-其他先例)
- [mihonapp/mihon](https://github.com/mihonapp/mihon)：Tachiyomi 的社区 fork，Android 漫画阅读器，本身不带任何扩展，扩展来自用户添加的 store，加载前校验签名，只从来源 store 更新。Apache-2.0。参考设计。见 [02 §1](research/02-adapter-ecosystems.md#1-tachiyomi-与-mihon-扩展生态)
- [NixOS/nixpkgs](https://github.com/NixOS/nixpkgs)：Nix 的包集合，PR workflow 默认不给权限、从不执行 PR 里的代码，在 Actions 里现场换 GitHub App token。MIT。参考设计。见 [05 §3.8](research/05-note-apps-and-plugins.md#38-其他先例)
- [nushell/nushell](https://github.com/nushell/nushell)：Nushell 的插件协议，插件以 `--stdio` 启动，用 JSON 或 MessagePack 收发调用，按 semver 判断兼容。MIT。参考设计。见 [02 §12.7](research/02-adapter-ecosystems.md#127-nushell-plugins)
- [obsidianmd/obsidian-releases](https://github.com/obsidianmd/obsidian-releases)：Obsidian 社区插件目录在 GitHub 上的镜像，2026-05 起提交改到目录网站，每个版本都做自动扫描。无许可证文件。参考设计。见 [05 §3.1](research/05-note-apps-and-plugins.md#31-obsidian)
- [open-ani/animeko](https://github.com/open-ani/animeko)：Kotlin 跨平台追番应用，收藏和进度同步 Bangumi，视频来自可订阅的数据源。AGPL-3.0。参考设计。见 [06 §14](research/06-platform-access-and-account-risk.md#14-规则源生态)
- [Predidit/Kazumi](https://github.com/Predidit/Kazumi)：Flutter 跨平台番剧采集与播放应用，视频来源由用户导入的 JSON 规则决定，规则在独立仓库 KazumiRules 托管和更新。GPL-3.0（应用图标另有版权）。参考设计。见 [06 §3](research/06-platform-access-and-account-risk.md#3-用户点名的项目)
- [Predidit/KazumiRules](https://github.com/Predidit/KazumiRules)：Kazumi 官方的规则托管仓库，一条规则一个 JSON，CI 自动生成目录，失效规则标 deprecated 后移出目录。MIT。参考设计。见 [06 §14](research/06-platform-access-and-account-risk.md#14-规则源生态)
- [Prowlarr/Indexers](https://github.com/Prowlarr/Indexers)：Prowlarr 的索引器定义仓库，每个站点一个 YAML，按 schema 版本分目录，用脚本校验并自动排除有问题的定义。无许可证文件。参考设计。见 [02 §13](research/02-adapter-ecosystems.md#13-其他相关模式)
- [rajatjindal/krew-release-bot](https://github.com/rajatjindal/krew-release-bot)：插件发 release 后自动向 krew-index 开版本更新 PR 的机器人，由插件仓库里的 Action 加外部 webhook 服务驱动。Apache-2.0。参考设计。见 [05 §3.8](research/05-note-apps-and-plugins.md#38-其他先例)
- [raycast/extensions](https://github.com/raycast/extensions)：Raycast 扩展的 monorepo，命令行自动开 PR，人工审查合并后由 CI 构建并发布。MIT。参考设计。见 [05 §3.4](research/05-note-apps-and-plugins.md#34-raycast)
- [siyuan-note/bazaar](https://github.com/siyuan-note/bazaar)：思源笔记集市，在清单文件里加一行 `owner/repo` 提交，Actions 只把 PR 当数据校验，定时按内容哈希入库。NOASSERTION（LICENSE 为木兰宽松许可证第 2 版）。参考设计。见 [05 §3.3](research/05-note-apps-and-plugins.md#33-思源-bazaar)
- [Stremio/stremio-addon-sdk](https://github.com/Stremio/stremio-addon-sdk)：Stremio 插件 SDK，插件是一个 HTTP 服务，用 `/manifest.json` 声明资源类型、ID 前缀和配置页。MIT。参考设计。见 [02 §13](research/02-adapter-ecosystems.md#13-其他相关模式)
- [tachiyomiorg/extensions](https://github.com/tachiyomiorg/extensions)：Tachiyomi 原来的扩展仓库，2024-01 迫于法律压力清空扩展列表、项目停止开发，仓库已归档。Apache-2.0。参考设计。见 [02 §1](research/02-adapter-ecosystems.md#1-tachiyomi-与-mihon-扩展生态)
- [zed-industries/extensions](https://github.com/zed-industries/extensions)：Zed 编辑器的扩展索引，扩展以 git submodule 挂入，员工逐个审核，扩展编译成 Wasm 并由用户授权能力。无许可证文件。参考设计。见 [05 §3.5](research/05-note-apps-and-plugins.md#35-zed)
- [zotero/translators](https://github.com/zotero/translators)：Zotero 的站点翻译器仓库，每个网站一个 JS 文件，从页面或站点接口提取文献元数据。无许可证文件（单个文件头为 AGPL-3.0）。参考设计。见 [06 §14](research/06-platform-access-and-account-risk.md#14-规则源生态)

<a id="相邻创作发布与分析"></a>
## 相邻：创作、发布与分析

**计划适配（1）**

- [Breakthrough/PySceneDetect](https://github.com/Breakthrough/PySceneDetect)：基于 OpenCV 的镜头切换检测命令行和 Python 库，输出每个场景的起止时间，可按场景截图或切分视频。BSD-3-Clause。计划适配。见 [06 §15](research/06-platform-access-and-account-risk.md#15-相邻创作发布与分析)

**参考设计（6）**

- [chengyi-ai/native-subtitle-quote-image](https://github.com/chengyi-ai/native-subtitle-quote-image)：从视频按时间点取帧，裁出画面里已烧录的字幕条或把台词画到真实帧上，拼成社交长图并逐张质检的 Agent Skill。MIT。参考设计。见 [06 §3](research/06-platform-access-and-account-risk.md#3-用户点名的项目)
- [dreammis/social-auto-upload](https://github.com/dreammis/social-auto-upload)：命令行加 Agent Skill 的多平台发布工具，用浏览器自动化把视频、图文和文章发到抖音、B站、小红书、视频号等 17 个平台入口。MIT。参考设计。见 [06 §3](research/06-platform-access-and-account-risk.md#3-用户点名的项目)
- [hanliang97/MatrixMedia](https://github.com/hanliang97/MatrixMedia)：Electron 桌面应用，把视频批量发布到抖音、快手、小红书、视频号、B站等平台的多个账号，附命令行、本机 HTTP 和 MCP。GPL-2.0。参考设计。见 [06 §3](research/06-platform-access-and-account-risk.md#3-用户点名的项目)
- [hypit-ai/hypit](https://github.com/hypit-ai/hypit)：给编码 Agent 用的视频制作语言与命令行，读入参考视频后转写、截帧分析，生成可编辑、可重跑的工作流。NOASSERTION（改过的 Apache-2.0，未经授权不得用于托管服务或商业获利）。参考设计。见 [06 §3](research/06-platform-access-and-account-risk.md#3-用户点名的项目)
- [pabatiba89-sys/kali-publish](https://github.com/pabatiba89-sys/kali-publish)：social-auto-upload 的派生，喀理 AI 的开源发布后端，用本机 Chrome 保存的登录态把视频发到小红书、视频号、抖音等 10 个平台；用户点名；只借登录与会话导出的经验，视频号流程写死了推广话题和商品逻辑，不包装。MIT。参考设计。见 [06 §3](research/06-platform-access-and-account-risk.md#3-用户点名的项目)
- [ZJU-REAL/Easel](https://github.com/ZJU-REAL/Easel)：浙江大学 REAL Lab 发起的社媒运营 Agent 工作台，用 Agent Skill 做热点发现、选题和图文音视频创作，扫码登录七个国内平台发布并读回账号数据。Apache-2.0（内置的 gzh-design 为 AGPL-3.0，抖音发布脚本的来源许可待核实）。参考设计。见 [06 §3](research/06-platform-access-and-account-risk.md#3-用户点名的项目)

**相邻项目（13）**

- [ahang008/ah-skills](https://github.com/ahang008/ah-skills)：阿杭的 Codex Skill 目录仓库，主要是指向独立仓库的索引，另有短视频制作入口、口播润色与剪辑包，本身不从平台取数据；用户点名；源码可见的非商业许可，只借来源说明的写法，不 fork、不分发。NOASSERTION（AH Source Available Non-Commercial License 1.0）。相邻项目。见 [06 §3](research/06-platform-access-and-account-risk.md#3-用户点名的项目)
- [caol64/wenyan-mcp](https://github.com/caol64/wenyan-mcp)：文颜的 MCP 版本，把 Markdown 转成公众号样式，连同图片、封面经官方接口上传到草稿箱。Apache-2.0。相邻项目。见 [06 §15](research/06-platform-access-and-account-risk.md#15-相邻创作发布与分析)
- [doocs/cose](https://github.com/doocs/cose)：doocs/md 配套的浏览器扩展，把编辑好的文章一次同步到公众号、知乎、头条、小红书长文、X Articles 等平台。Apache-2.0。相邻项目。见 [06 §15](research/06-platform-access-and-account-risk.md#15-相邻创作发布与分析)
- [doocs/md](https://github.com/doocs/md)：把 Markdown 即时渲染成公众号图文样式的编辑器，有网页版、浏览器扩展、VS Code 扩展和命令行。WTFPL。相邻项目。见 [06 §15](research/06-platform-access-and-account-risk.md#15-相邻创作发布与分析)
- [gitroomhq/postiz-app](https://github.com/gitroomhq/postiz-app)：可自托管的社交媒体排期发布平台，经各平台官方 OAuth 接口发布，并用官方接口拉取帖子数据做分析。AGPL-3.0。相邻项目。见 [06 §15](research/06-platform-access-and-account-risk.md#15-相邻创作发布与分析)
- [harry0703/MoneyPrinterTurbo](https://github.com/harry0703/MoneyPrinterTurbo)：根据主题或关键词自动写脚本、配音、匹配素材、加字幕和配乐并合成短视频的工具。MIT。相邻项目。见 [06 §3](research/06-platform-access-and-account-risk.md#3-用户点名的项目)
- [leaperone/MultiPost-Extension](https://github.com/leaperone/MultiPost-Extension)：在已登录的平台页面里自动填写内容，把文字、图片、视频、文章一键发到几十个平台的浏览器扩展，对外提供 API。Apache-2.0（商店版含未公开的 Agent 模块）。相邻项目。见 [06 §15](research/06-platform-access-and-account-risk.md#15-相邻创作发布与分析)
- [linyqh/NarratoAI](https://github.com/linyqh/NarratoAI)：用大模型给本地影视素材写解说文案，再自动剪辑、配音、加字幕合成解说视频的工具。MIT。相邻项目。见 [06 §15](research/06-platform-access-and-account-risk.md#15-相邻创作发布与分析)
- [Waboodoo/HTTP-Shortcuts](https://github.com/Waboodoo/HTTP-Shortcuts)：Android 上创建可从桌面、快捷设置或分享菜单触发的 HTTP 请求的 App，能把分享来的网址发到局域网里的本机服务。MIT。相邻项目。见 [06 §15](research/06-platform-access-and-account-risk.md#15-相邻创作发布与分析)
- [wechatsync/Wechatsync](https://github.com/wechatsync/Wechatsync)：浏览器扩展加命令行和 MCP，用浏览器里已有的登录态把 Markdown 或当前网页文章同步成知乎、头条、公众号等平台的草稿。GPL-3.0（部分适配器不开源）。相邻项目。见 [06 §15](research/06-platform-access-and-account-risk.md#15-相邻创作发布与分析)
- [yikart/AiToEarn](https://github.com/yikart/AiToEarn)：一人公司的 AI 内容营销平台，按平台授权把内容发到 14 个以上平台，浏览器插件在已登录页面上自动点赞、收藏、关注和回复，另有 AI 生成视频图文和商单结算；用户给的视频号链接介绍的项目。MIT。相邻项目。见 [06 §3](research/06-platform-access-and-account-risk.md#3-用户点名的项目)
- [YILS-LIN/short-video-factory](https://github.com/YILS-LIN/short-video-factory)：Electron 桌面应用，用大模型写口播文案、Edge TTS 配音，再把用户自备的素材混剪成带字幕的短视频。AGPL-3.0。相邻项目。见 [06 §3](research/06-platform-access-and-account-risk.md#3-用户点名的项目)
- [zhouxiaoka/autoclip](https://github.com/zhouxiaoka/autoclip)：开源 AI 剪辑桌面工具，从长视频自动挑高光切成短视频，并生成字幕、封面和发布文案。MIT。相邻项目。见 [06 §15](research/06-platform-access-and-account-risk.md#15-相邻创作发布与分析)
