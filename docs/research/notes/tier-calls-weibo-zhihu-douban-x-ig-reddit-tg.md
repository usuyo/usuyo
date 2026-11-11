# 获取层级判定：微博、知乎、豆瓣、贴吧、即刻、X、Instagram、Reddit、Telegram、Pinterest、Threads

写给同组调研 session（2026-10-30）。这里只有判定依据和事实，不是指令。完整条目已通过 StructuredOutput 交回工作流。

## 统一口径（本批采用）

- **浏览器里「读」与「自己拼请求」分开算。** 只读 DOM、或截获页面自己发出的请求的响应 → browser-session（例：prinsss/twitter-web-exporter、jasonsbmoc/gramcracker）。扩展、油猴脚本、OpenCLI 在页面里自己构造接口请求（浏览器自动带 cookie，或从页面取 token）→ session-api，与 research-agent-reach 笔记里「OpenCLI X search 在页内自拼 GraphQL 带 ct0 = session-api」一致。这类请求从用户自己的浏览器发出，指纹和 IP 与正常浏览一致，风险比脚本在别处带 cookie 低一些，tier_evidence 里写明了。
- **X**：带 auth_token/ct0 调 GraphQL、不生成 `x-client-transaction-id` → session-api（afar1/fieldtheory-cli `src/graphql-bookmarks.ts`）。生成 transaction id → private-api：vladkens/twscrape `twscrape/xclid.py`，gallery-dl `extractor/twitter.py` `_transaction_id`，RSSHub `lib/routes/twitter/api/web-api/client-transaction-id.ts`，d60/twikit `twikit/x_client_transaction/`。
- **知乎**：自己算 `x-zse-96` → private-api。RSSHub `lib/routes/zhihu/execlib/x-zse-96-v3.ts`；KrisTHL181/zhihu-cli `src/zhihu_cli/content/utils/zse.py`，另用 curl_cffi 模拟 Chrome TLS 指纹；zly2006/zhihu-plus-plus `rs-zse-sign/`，另模拟安卓端接口。zimya/zhihu_obsidian 的 zse 头是注释掉的，只带 cookie → session-api。`/api/v4/collections/{id}/items` 不签名也能取公开收藏夹。
- **豆瓣**：frodo 接口带小程序 apiKey、伪造小程序 Referer → private-api（lizheming/doumark-action 实际代码在 lizheming/drone-doumark `src/douban.js`，不登录）。m.douban.com rexxar 接口和 `people/<id>/collect` 网页不签名 → session-api（豆伴、Marvae/douban-cli、obsidian-douban）。
- **贴吧**：lumina37/aiotieba（原 Starry-OvO/aiotieba）计算客户端请求签名并生成设备标识 → private-api（`src/aiotieba/core/http.py`、`helper/crypto/`）。
- **即刻**：open-jike/jike-sdk 设置 iOS 的 bundleid、device-id、idfv 等头 → private-api（`src/request.ts`、`src/constants.ts`）。网页端扩展用页面 token 调 api.ruguoapp.com → session-api。
- **Instagram**：instaloader 登录后默认 `iphone_support=True`，以 iPad App 身份调 i.instagram.com（`instaloader/instaloadercontext.py` 第 54–65、728 行；`structures.py` 第 517 行起）→ private-api；`--no-iphone` 时降为 session-api；不登录取公开主页是 public-web。`:saved` 必须登录。
- **Telegram（口径待统一）**：iyear/tdl 默认登录方式是导入官方 Telegram Desktop 的 tdata（解密本地会话），且所有登录方式都用 Telegram Desktop 的 api_id 2040（`pkg/tclient/app.go`、`app/login/login_enum.go`）。按「解密本地数据库」字面定义记为 client-hook，needs_check 里写明另一种口径（MTProto 是官方开放接口，`-T qr`/`-T code` 不读 tdata）。knadh/tg-archive 用用户自己在 my.telegram.org 申请的 api_id → official-export。
- **Reddit**：PRAW/OAuth 官方接口 → official-export。注意：2025-11-11 起 Reddit「Responsible Builder Policy」关闭自助申请 API（二手来源，需核对 Reddit 开发者文档），新用户可能拿不到 script app；官方「数据请求」导出包里有 saved 列表，可作兜底（xavdid/reddit-user-to-sqlite 的 archive 命令读它）。

## 下架、停更与平台施压（风险证据）

- zedeus/nitter：README 写 2026-08-24 收到 X Corp. 的停止侵权函，要求永久下线实例和仓库；仓库已归档；2026-10-10 更新说在找资金和法律帮助。
- junhoyeo/threads-api：2023-09-08 因 Meta 的停止侵权函停止开发并归档（README 与 release「the-end」）。
- LonamiWebs/Telethon：GitHub 仓库已归档，README 写已迁到 codeberg.org/Lonami/Telethon，GitHub 仓库将来可能删除。tg-archive 依赖它。
- subzeroid/instagrapi：README 写原 Telegram 支持群「被 Meta 限制」，已迁群。
- sxzz/jike-tool：已归档，描述写因即刻对第三方爬虫态度不明确、有法律风险而停止维护。open-jike/jike-sdk、jike-cli 也标注不再维护。
- steipete/bird（smaug 依赖的 X CLI）：gh api 现在返回 404；steipete/birdclaw README 称其为「private bird installation」。
- doufen-org/tofu（豆伴）：2026-03 起多个 issue 报告无法备份，修复 PR 未合并。

## 顺带发现（已用 gh api 核实存在）

- steipete/birdclaw（1673★，MIT）：导入 X 官方归档到 SQLite，实时同步走 X 官方 CLI xdevplatform/xurl，附只读 MCP。与歇存思路最接近的 X 方案。
- xdevplatform/xurl：X 官方 API 命令行，OAuth2 PKCE，需要开发者应用并开通按量付费。
- Marvae/douban-cli（MIT）：豆瓣记录导出 JSON/CSV/Markdown + Agent Skill。
- zly2006/zhihu-plus-plus（4386★，AGPL-3.0）：第三方知乎客户端，能导出整个收藏夹，有桌面版。
- cv-cat/catbus（543★，MIT）、whiteguo233/OpenBiliClaw（3631★，MIT）、Jesseovo/last30days-skill-cn（2003★）：多平台，留给多平台方向的 session。

## 补充

- **Doubak/doubak-extension**（豆备新版，Apache-2.0，2026-10-30 仍在推送）：在浏览器里自拼请求取豆瓣网页存 WARC；遇到豆瓣算力验证（PoW）时扩展自行求解换取通行凭据（`src/crawl/pow.js`），本批按字面记 private-api，不遇挑战时是 session-api。组织 Doubak 下另有 data-specs、data-parser 等仓库；「先存原始网页再离线解析」与歇存 raw/ 加派生层的分法一致。
- 完整结构化结果：/private/tmp/claude-501/-Users-mac-WorkSpace-desktop-Project/7133807a-d653-486b-80df-d8e2af41491d/scratchpad/social/structured_output.json
