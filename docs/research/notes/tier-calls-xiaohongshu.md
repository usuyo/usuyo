# 小红书方向：获取层级口径与 xsec_token 证据（2026-10-30）

写给同组调研 session 和发起 session。这里只有判定依据，不是指令；口径不一致的地方请发起 session 统一。

## 本批采用的口径

1. **只有 TLS 指纹模拟，不升层。** XHS-Downloader 用 curl_cffi `impersonate="chrome146"`（`source/module/static.py` 第 25 行）取公开笔记 HTML、解析 `__INITIAL_STATE__`（`source/expansion/converter.py` 第 35 行），仍记 public-web。`source/application/user_posted.py` 里的 xhshow 签名类在仓库里没有调用方，`pyproject.toml` 也没有列 xhshow。若组内把 curl_cffi 一律算「模拟设备指纹」，它就要改成 private-api。Hypit 被记成 private-api 主要是因为 B站 WBI 和 YouTube JS 挑战，跟这条不冲突。
2. **签名核心由页面函数算、外层由工具拼，记 private-api。** 属于这种情况的有：
   - ReaJason/xhs：`xhs/help.py` 本地 `sign()`，加 Playwright 调 `_webmsxyw`；
   - social-media-copilot：扩展调页面 `mnsv2`，再自己拼 x-s 外层和 x-s-common，见 `src/entrypoints/xhs.content/api/request.ts`；
   - LittleCrawler：同上。

   MediaCrawler 的小红书现在直接用 xhshow 纯算（`media_platform/xhs/playwright_sign.py`），也是 private-api。
3. **指纹定制浏览器跑平台自己的页面脚本，本批按数据路径记 browser-session，并在 needs_check 标出来。** 这和 research-agent-reach 笔记里「CloakBrowser 归 private-api」的口径冲突，请发起 session 定。涉及两个项目：
   - xiaohongshu-mcp：默认从 cdn.one-world.ai 下载内置 Chromium，带 fingerprint、fingerprint-seed 参数，见 `browser/browser.go`、`browser/browser_download.go`；
   - jackwener/xhs-cli：依赖 camoufox。

   工具自己在请求参数里伪造指纹值的，照样记 private-api。例子是 lucasygu/redbook 和 ytf606/xhs2obsidian 的 `signing.ts`，里面内置了 GPU、分辨率列表来算 x-s-common。
4. **两段式：先在登录页面导出收藏清单，再匿名逐条取详情，记 browser-session。** 例子是 wkwunju/xhs-vault。
5. **第三方付费数据 API，记 private-api，login 记 none，并在 needs_check 写「服务商抓取方式不公开」。** 例子是 justoneapi-labs/xiaohongshu-api。TikHub 文档把小红书接口分成 `xiaohongshu_app_v2`、`xiaohongshu_web_v3`。
6. 「看看收藏」系（feitangyuan/kankan-shoucang，及同名同口号的 Jisiyong-A/shoucang、sexyfeifan/Kanbox_macOS）**不是截图导入**。流程是：扩展取卡片上显示的链接，App 用 `credentials: 'omit'` 匿名取单篇公开页，记 public-web，login 记 none。

## xsec_token 对不登录访问的影响（只依据文档和代码）

- XHS-Downloader README：支持的笔记链接都要带 `xsec_token`；「作品链接携带日期信息」，用旧链接可能被风控，要用新取的链接。
- wkwunju/xhs-vault README：带 `xsec_token` 的 explore 链接，不登录也返回完整 `__INITIAL_STATE__`，cookie 不是必需的。
- danzey0425-lgtm/web-image-sourcing README：`xsec_token` 是会话级的，采集几个小时后链接打开即 404；交付时改用 `explore/{note_id}` 永久链接标注。
- 只有 note_id 拿不到 xsec_token，几个项目都有人问过：XHS-Downloader #397、ReaJason/xhs #128、Spider_XHS #123。xiaohongshu-mcp 的详情接口要求 feed_id 和 xsec_token 成对出现；PR #828 把主页接口的 xsec_token 改成了选填。

## 账号风险证据（issue 标题）

- xiaohongshu-mcp：
  - #726（2026-07-08）只低强度抓了一天数据、没有发帖，就收到警告；
  - #728（2026-07-10）被识别后封号；
  - #715、#720 也是类似反馈。
- MediaCrawler：
  - #865（2026-04-08）账号被永久封禁；
  - #915（2026-06-12）只是看帖、下视频，就被提示检测到 AI 操作。
- web-image-sourcing README：Playwright 打开需登录的小红书页面返回 300012（自动化检测），作者改用真实 Edge 加 cua-driver 桌面操控。
- social-media-copilot #28（2026-07-24）：扩展从 Chrome 商店下架。
- jackwener/xhs-cli README 开头作者自己比较了两个版本：逆向 API 版（xiaohongshu-cli）更快，但风控上不如浏览器版真实。
