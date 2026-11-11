# 下架、停更与法律风险：跨条目证据（2026-10-30）

写给同组调研 session。只有事实和来源，不是指令。完整结果（44 条 finding、summary_zh、needs_check）：
/private/tmp/claude-501/-Users-mac-WorkSpace-desktop-Project/7133807a-d653-486b-80df-d8e2af41491d/scratchpad/takedown/structured_output.json
github/dmca 文件名清单（2011–2026，按年）在同目录 dmca/，读过的通知原文在 dmca_files/。

## 对同组笔记的更正

- 腾讯 2026-06-29 至 10-13 的 36 份 wechat 通知里，按派生网络整体处理的是 9 份，不是 8 份：漏了 WeFlow（2026-07-07，约 3,200 个仓库）。加上 2026-01-08 那份（4,195 个），2026 年共 10 份整网处理。36 份都以 DMCA 第 1201 条为由、都写到加密和密钥。
- WeFlow 现在的提交历史从 2026-09-28 的 init 开始，没有任何 release；docs/third-party-components.md 也是那天才有。「被点名后改为外部组件」的时间点是 9 月底，不是 7 月。
- DouK-Downloader 不再维护签名的说明是 2026-07-03 加的（同一提交加了动态加载外部 Python 代码），「为确保合法合规」的理由是 07-04 补的。

## 两条施压渠道

- github/dmca：腾讯（本地库解密，§1201）、QQ 音乐（unlock-music）、网易云音乐（2023 年签名复现仓库）、TikTok（2023-03-14、2025-05-12 网页端安全逻辑复现；GitHub 两次都说反规避主张证据不足、按其他版权主张处理）、字节跳动（2023 年 21 份，都是反编译安装包和 SDK 代码）、优酷（2025 年针对 mangayomi、cloudstream 的发布安装包）、Crunchyroll（aniyomi-extensions，451）。
- 直接给维护者发律师函，不经过 github/dmca：B站 2026 年（bilibili-API-collect、bilibili-api、downkyi、BiliTools、BiliRoaming）、微信官方 2025-10-20（PyWxDump、chatlog）、网易云音乐 2024-01（NeteaseCloudMusicApi）、X Corp.（Nitter，2026-08-24）、Meta（threads-api，2023）、阅文（legado，2026-05-27）。
- 按文件名看，没有平台针对抖音、小红书、B站、快手、微博、知乎下载或收藏导出工具的 DMCA；也没找到抖音、小红书给 GitHub 工具发函的公开记录。

## 判例与法规（有官方或可核来源）

- 最高法第 47 批指导性案例（2025-08-28）：262 号，搬运平台数据集合并向公众提供、实质性替代，构成不正当竞争；263 号，经用户授权、在合理范围内转移用户自己的数据，不构成。court.gov.cn/zixun/xiangqing/474431.html
- 《反不正当竞争法》2025 年修订第十三条第三款（避开或者破坏技术管理措施获取他人数据）、第二十九条罚款；《网络数据安全管理条例》第十八条（自动化工具）、第六十三条第二款（个人或家庭事务例外）；《个人信息保护法》第七十二条；《数据安全法》第三十二条；法释〔2011〕19号第一至三条（第二条：能避开或突破安全保护措施取数据的程序算专门侵入程序；第三条：提供 20 人次以上即情节严重）。
- 案例：抖音诉刷宝（500 万元）、腾讯诉聚客通（260 万元，整体数据与单一数据二分出自律所评析）、微博诉脉脉（三重授权）、小红书诉厦门某公司（490 万元，仅新闻）、海淀 2018 爬虫刑事案、常州小红书营销私信刑事案、普陀 2025 工具提供者被单独定罪。

## 项目方自我保护做法（都已核实）

- 不内置签名、由用户自备：DouK-Downloader；不内置解密组件：WeFlow。
- 只在页面内取本人数据：douyin-favorite-MyLike（注释写明不用伪造或保存的签名值，免责声明限本人账号）；twitter-web-exporter（被动截获、默认每文件间隔 1 秒）。
- 入口规则：yt-dlp 不支持以侵权为主的网站和纯 DRM 服务。
- 下架清单：keiyoushi/extensions-source #14260，被 DMCA 的扩展不再加回。
- 收函公告：BiliTools 写明不构成对法律主张的承认。
- 自写非商业许可：MediaCrawler（2023 Apache-2.0 → 2024-04-03 非商业 1.0 → 2024-10-17 非商业学习 1.1）、yichen-skills、wechat-inbox-sync（2026-08-23 起 Commons Clause）。yichen-skills 有非商业许可仍收到律师函。
