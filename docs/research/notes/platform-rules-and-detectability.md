# 平台规则与可检测性：给各获取层级标账号风险的证据（2026-10-30）

写给同组调研 session 和发起 session。这里只有事实和判断依据，不是指令。63 条 findings 已通过 StructuredOutput 交回工作流；完整 JSON：
/private/tmp/claude-501/-Users-mac-WorkSpace-desktop-Project/7133807a-d653-486b-80df-d8e2af41491d/scratchpad/rules/out/findings.json
（摘要和待核实清单在同目录 summary.json）

## 七层账号风险（一句话版）

| 层级 | 账号风险 | 主要依据 |
|---|---|---|
| official-export | 接近零 | 抖音隐私政策 3.3「个人信息下载」；Google Takeout；Tencent/WeChatReading 个人 API Key；《个人信息保护法》第四十五条 |
| public-web | 无账号风险，只有 IP 层限速 | yt-dlp wiki：YouTube 访客约每小时 300 个视频；Instaloader 文档：匿名 429，云和 VPN IP 更严；豆瓣频繁取页出算力验证 |
| vision-ui | iPhone 镜像、桌面 computer use 按人的节奏：接近零但不是零；Android 自动化：在微信上是中风险 | 见下节 |
| browser-session | 低到中，取决于是不是用户日常那个浏览器 | 抖音现在只放行页内请求（douyin-downloader #258）；小红书对自动化浏览器一两天内警告或封号（xiaohongshu-mcp #720、#726、#728；Playwright 返回 300012）；X 自动化规则：脚本操作网站可能永久封号 |
| session-api | 中 | 抖音收藏类接口被网页校验挡住，批量时 cookie 临时失效（TikTokDownloader #478）；yt-dlp 文档承认用账号有封号风险 |
| private-api | 中到高 | 小红书：MediaCrawler #865 永久封禁、#915 只看帖也提示 AI 操作、BettaFish #670 自动化脚本预警；开发者侧：B站 2026 律师函、X 对 Nitter 的停止侵权函 |
| client-hook | 高 | 微信 2019 对 Xposed、substrate 外挂限制功能直至限制登录，上半年处理上百万账号；Play Integrity 发现 Hook 或 root 判设备不完整；腾讯 DMCA；聚客通案判赔 260 万元 |

## 视觉层：哪些接近零，哪些不是

- **接近零：**
  - iPhone 镜像：Apple DTS 2024-07-12 答复，没有公开 API 能让 iPhone 上的 App 识别自己正被镜像；开发者实测安全文本框遮罩在镜像下不生效。
  - 桌面 computer use，按人的节奏操作用户日常用的客户端。
  - 以上两种都没找到平台据此处罚的公开记录。
- **不是零：**
  - 微信协议 8.2.1.6 把「用第三方软件自动操作、读取界面」写成违规。
  - 微信安全中心（2026-01-30，媒体转述）点名录屏、截图、辅助服务读取、模拟按键、自动化脚本。
  - 豆包手机助手（系统级 INJECT_EVENTS）2025-12 操作微信，用户被强制下线。
  - KrASIA 2026-06-09：微信只接受系统助手走 A2A 协议，不接受读屏加模拟点击的 GUI agent。
  - PC 微信 4.x 不再给新账号暴露 UI Automation 树（pywechat 的 Weixin4.0.md）。
- **Android 能被发现的信号：**
  - getEnabledAccessibilityServiceList（无权限要求）；
  - setAccessibilityDataSensitive（API 34，可对非辅助工具隐藏视图）；
  - Play Integrity appAccessRiskVerdict（报告 CAPTURING、CONTROLLING 类应用，需要 Google Play 服务）。
  - ADB 和开发者选项：官方文档现在写「对第三方 App 恒返回 0」，二手来源称从 Android 17 QPR1 起，旧系统和国内 ROM 待核实。
- **录屏 OCR：** iOS 的 isCaptured 和 sceneCaptureState 能让 App 感知录屏、投屏；iPhone 镜像会不会触发这两个状态，没有官方说明。
- **所有层共有：** 节奏异常本身会触发处置（微信个人账号使用规范 3.4、微博协议 4.5.1）。产品的限速和退避对视觉层同样需要。

## 平台条款速查（都已从官方页面读到）

| 平台 | 版本 | 条款号 |
|---|---|---|
| 抖音 | 2026-02-13 | 2.4、5.1、5.3；处罚 7.1 |
| 快手 | 2026-02-04 | 七.1.1、七.1.4（客户端内存和交互数据、群控）、十.1；处罚 十五.1。中文版在 app.m.kuaishou.com/public/index.html#/protocol/14 |
| 小红书 | 2026-03-23，境内注册用户 | 4.1 平台使用规范；处罚 2.5 |
| B站 | 2025-04-30 | 5.6(3)、1.3；处罚 2.7 |
| 微信 | 未标日期 | 8.2.1.4、8.2.1.6、8.2.1.7；处罚 8.5.1 |
| 微信个人账号使用规范 | 2025-03-28 | 1.2.8、3.4 |
| 公众平台运营规范 | 未标日期 | 3.1、3.27（非真人自动化创作，只管发布侧）、5.1.6 |
| 微博 | 未标日期 | 1.3.2、1.4、4.5.1（频率）；处罚 8.5 |
| 知乎 | 2025-03-20 | 二.9.a、二.9.b、三.10 |
| 豆瓣 | 2025-05-27 | 10.4、9.6、2.3、3.1 |
| X | 2026-10-09 | 禁止抓取；违约金按 24 小时内每 100 万条帖子 1.5 万美元 |
| YouTube | 2023-12-15 | 禁止自动化访问和下载 |
| Instagram | 2025-01-01 | 4.2 |

## 顺带确认

- 腾讯的 37 份 DMCA 通知（2026-01-08 到 2026-10-13）点名 66 个仓库，全部是数据库解密、密钥提取、聊天记录导出类。
- 没有 UI 自动化类，也没有视频号代理下载类。wx_channels_download 在 2026-10-29 仍在推送。
- Apple 支持文档 120421 确认，iPhone 镜像时声音从 Mac 播放（耳机接在 iPhone 上时除外）。vision-ui 笔记里「镜像能否带出声音」这一条，可以据此关掉。
