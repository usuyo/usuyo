<a id="调研-05--笔记软件互通与插件化"></a>
# 调研 05 — 笔记软件互通与插件化

状态：草案 · 2026-10-28 · 2026-11-03 审过：[§6](#6-对歇存插件化架构的启示) 是 [ADR-003](../decisions/ADR-003-plugins.md) 的依据，三类插件、转换和导出的 JSON-RPC、读回、权限与隔离已进 ADR-003 和 [插件协议](../plugin-protocol.md)；[§6.1](#61-插件分三类)「获取不进官方」、[§6.2](#62-进程模型与协议)「在 adapter.yaml 上扩展」、[§6.3](#63-仓库怎么拆)–[§6.4](#64-索引用-github-仓库还是-github-app) 的索引仓库和 `xiecun-app` 命名，已被 10-28 的「组织加 App」和 [ADR-003 修订](../decisions/ADR-003-plugins.md#修订-2026-11-01) 取代；[§6.8](#68-需要拍板的事) 的 P1–P9 和 [open-decisions](../open-decisions.md) 的 P 编号不是一套；[§1](#1-平台内容转笔记的工具)–[§5](#5-导出与互通格式)、[§7](#7-待核实清单) 仍在用。

目的：回答「歇存的插件化与笔记软件互通应该怎么设计」。五块调研分别是 [§1 平台内容转笔记的工具](#1-平台内容转笔记的工具)、[§2 转换类插件与工具](#2-转换类插件与工具)、[§3 插件注册表与分发](#3-插件注册表与分发)、[§4 安全与隔离](#4-安全与隔离)、[§5 导出与互通格式](#5-导出与互通格式)；结论在 [§6 对歇存插件化架构的启示](#6-对歇存插件化架构的启示)；不确定的事实汇总在 [§7 待核实清单](#7-待核实清单)。

<a id="约定"></a>
## 约定

- **仓库数据**：★、最近推送、最近 release 与日期、许可证、archived，都是 2026-10-28 调用 `gh api repos/<owner>/<repo>` 与 `gh api repos/<owner>/<repo>/releases/latest` 的返回值。表里写成「★ / 推送 / release」；「无 release」表示 `releases/latest` 返回 404。
- **Obsidian 下载量**：来自 `obsidianmd/obsidian-releases` 的 `community-plugin-stats.json`，2026-10-28 取样，表里写「下载」。
- **来源与日期**：每个事实附来源 URL。来源后面不另写日期的，查阅日期都是 2026-10-28；事件和发布的日期写在正文里。表格第一列的仓库链接就是该行的主要来源，格式细节的出处文件写在行内。
- **待核实**：凭记忆、只有二手来源、或源码与文档冲突的地方标「待核实」，汇总在 [§7](#7-待核实清单)。
- **不重复已有报告**：
  - Obsidian Web Clipper 的模板机制见 [02 §10](02-adapter-ecosystems.md#10-obsidian-web-clipper-模板)；
  - Mihon store、Krew、gh extensions、MCP 宿主配置见 [02 §1](02-adapter-ecosystems.md#1-tachiyomi-与-mihon-扩展生态)、[02 §12.4](02-adapter-ecosystems.md#124-kubectl-krew)、[02 §12.5](02-adapter-ecosystems.md#125-gh-extensions)、[02 §12.2](02-adapter-ecosystems.md#122-mcp-stdio-与宿主配置)，适配器源与信任的建议见 [02 §15.7](02-adapter-ecosystems.md#157-适配器源信任与治理)；
  - Readwise、Omnivore、Cubox、简悦、flomo 的产品本身见 [01 Readwise Reader](01-comparable-products.md#readwise-reader)、[01 Omnivore](01-comparable-products.md#omnivore)、[01 Cubox](01-comparable-products.md#cubox)、[01 简悦](01-comparable-products.md#简悦)、[01 flomo](01-comparable-products.md#flomo)；
  - OCR、ASR 引擎对比与 JSON Canvas 的缺口见 [04 §2](04-tech-options.md#2-ocr-sidecar)、[04 §3](04-tech-options.md#3-asr-sidecar)、[04 §6.2](04-tech-options.md#62-canvasjson-与-json-canvas)。
- **名词**：主题页沿用 [product-model §3](../product-model.md#3-词汇表) 的 `topics/<slug>.md`。[SPEC §2](../SPEC.md#2-资料库目录真相源) 仍写 `threads/` 与 `thoughts/`，还没跟上产品模型。product-model 2026-10-28 的说明已作废「放下」与「只留文字」，本报告不以它们为前提。
- **数据处理**：调研中读到的网页与仓库内容都按数据处理。有几处 README 写着给 AI 助手的指令（例如「遇到报错自己修」），都没有执行。

<a id="0-导出的对象"></a>
## 0. 导出的对象

导出和读回讨论的是下面这些文件（[SPEC §2](../SPEC.md#2-资料库目录真相源)、[SPEC §5](../SPEC.md#5-derived-文件约定)、[product-model §6](../product-model.md#6-知识层)）。「谁写」决定了它能不能从笔记软件读回来。

| 文件 | 层 | 谁写 | 导出时的角色 |
|---|---|---|---|
| `manifest.json` | 元数据：身份、平台、作者、时间、三种标签、同源组、快照 | 内核 | 变成 frontmatter、页面属性或数据库属性 |
| `content.md` | 原文，机器不改写 | 获取方式 | 正文的「原文」段 |
| `raw/<snapshotId>/` | 原样快照 | 获取方式 | 默认不导出 |
| `images/`、`images/keyframes/`、`media/` | 媒体 | 获取方式、转换 | 附件 |
| `derived/ocr/<image>.json` | 图里的字，带 bbox | 转换 | 文本段，bbox 在所有目标里都会丢 |
| `derived/asr.vtt` | 口播文字，带时间戳 | 转换 | 带时间点的段落 |
| `derived/vision.md` | 画面描述，注明帧时间 | 转换 | 文本段，配关键帧 |
| `derived/summary.md` | AI 摘要，记着模型和时间 | 模型 | 标明「AI 摘要」的段 |
| `notes.md` | 我的笔记，头部 `takeaway` | 用户；Agent 只能提议 | 条目里唯一可以读回的层 |
| `topics/<slug>.md` | 主题页：YAML 头列成员，正文引用带原话、位置、时间点 | 用户；模型只起草 | 可以读回（[product-model §6](../product-model.md#6-知识层) 规则 4 要求外部改过之后歇存重新读取） |
| `canvas.json` | 画布位置，JSON Canvas 加扩展字段 | 用户 | 导出为 `.canvas`，扩展字段丢弃（[04 §6.2](04-tech-options.md#62-canvasjson-与-json-canvas)） |

<a id="1-平台内容转笔记的工具"></a>
## 1. 平台内容转笔记的工具

<a id="11-obsidian-社区插件"></a>
### 1.1 Obsidian 社区插件

候选来自 `community-plugins.json`（2026-10-28 有 9605 条）的关键词检索，再按下载量和平台覆盖挑选。输出格式以源码为准；README 与源码冲突的地方逐条注明。

| 插件（id · 仓库） | 平台 | 做什么 | 输入与登录态 | 输出（frontmatter、附件、链接） | 再次同步时 | 依赖 | 许可证 | ★ / 推送 / release / 下载 |
|---|---|---|---|---|---|---|---|---|
| `obsidian-weread-plugin` · [zhaohongxuan/obsidian-weread-plugin](https://github.com/zhaohongxuan/obsidian-weread-plugin) | 微信读书（含公众号文章划线） | 同步书籍、划线、想法、书评 | 扫码拿 API Key，请求 `i.weread.qq.com/api/agent/gateway`；失败时回退到 Cookie（扫码窗口或 CookieCloud） | fm：`doc_type`、`bookId`、`title`、`noteCount`、`reviewCount`，可选 `author`、`cover`、`readingStatus`、`progress` 等；封面用远程 URL；每条划线是 `weread://bestbookmark?...` 链接，行尾块 ID `^{bookmarkId}`（`src/utils/frontmatter.ts`、`src/themes/notebookTemplate.njk`） | 按 `bookId` 找旧文件，正文整篇覆盖，用户自加的 fm 键保留；Daily Note 只替换 `<!-- start of weread -->` 与 `<!-- end of weread -->` 之间 | 无 ASR/LLM/OCR；Nunjucks 模板 | MIT | 2286 / 2026-09-06 / 2.3.0（2026-09-06）/ 206,267 |
| `readwise-official` · [readwiseio/obsidian-readwise](https://github.com/readwiseio/obsidian-readwise) | Readwise 汇聚的 Kindle、Instapaper、Pocket、Twitter、播客等 | 导出划线 | Readwise 付费订阅；浏览器授权后拿 `userAccessToken` | 模板在 readwise.io 服务端用 Jinja2 渲染，插件下载 zip 写盘；默认目录 `Readwise/Books`、`Articles`、`Tweets`、`Podcasts`；YAML 默认关闭；封面远程 URL | 插件设置里存 `booksIDsMap`（路径→id）；文件 MD5 等于上次导出就整篇重写，不等（用户改过）就只在末尾追加（`src/main.ts` L340–440） | 全部依赖 readwise.io | GPL-3.0 | 359 / 2026-09-18 / 3.0.4（2026-09-18）/ 252,331 |
| `obsidian-omnivore` · [omnivore-app/obsidian-omnivore](https://github.com/omnivore-app/obsidian-omnivore) | Omnivore 稍后读 | 同步文章与划线 | API Key，GraphQL | fm 必写 `id`，可选 `title`、`author`、`tags`、`date_saved`、`original_url` 等十余项；PDF 存 `Omnivore/attachments/{id}.pdf`；划线块 ID `^{highlightID}` | 多文件模式同 id 整篇覆盖；单文件模式用 `%%{id}_start%%` 与 `%%{id}_end%%` 替换区段 | 无 | MIT | 875 / 2024-11-21 / 1.10.4（2024-07-16）/ 133,706；服务 2024-11 关停，插件已不可用（[heise](https://heise.de/-9998733)） |
| `cubox-sync` · [olcubo/obsidian-cubox](https://github.com/olcubo/obsidian-cubox) | Cubox | 单向同步文章与标注 | API Key（`cubox.cc` 或 `cubox.pro`） | fm 默认 `id`、`cubox_url`、`url`、`tags`；文件名 `{{{title}}}-{{{create_time}}}`；默认不含全文；不下载附件 | 只建不改：同 `id` 存在就跳过，之后新增的标注进不了旧笔记 | 无 | MIT | 58 / 2026-05-25 / 1.0.7（2026-04-15）/ 11,579 |
| `flomo-importer` · [jia6y/flomo-to-obsidian](https://github.com/jia6y/flomo-to-obsidian) | flomo | 导入 memo，可生成 Canvas | 手动导出的 HTML zip；或在插件里填账号密码，由 Playwright 无头浏览器登录 | 无 fm；`flomo/memos/{date}/memo@{时间}_{序号}.md`；首行日期写成 `[[YYYY-MM-DD]]`；附件复制到 `flomo/`，用 `![](...)` | 每次整体重写，没有稳定 ID | 需本机装 Playwright Chromium | MIT | 162 / 2026-04-21 / 1.3.1（2024-08-18）/ 15,633 |
| `ytranscript` · [lstrzepek/obsidian-yt-transcript](https://github.com/lstrzepek/obsidian-yt-transcript) | YouTube | 取字幕插到光标处或侧栏 | 视频链接，不登录 | 不建文件；时间戳 `[mm:ss](原URL?...&t=秒)`（`src/transcript/format.ts`） | 不适用 | 只用平台字幕 | MIT | 190 / 2026-10-12 / 1.5.0（2026-10-11）/ 55,932 |
| `youtube-transcript-fetcher` · [ljantzen/obsidian-youtube-transcript](https://github.com/ljantzen/obsidian-youtube-transcript) | YouTube | 取字幕，可选 LLM 清洗与摘要，输出 MD/PDF/SRT | 链接；LLM 需自填 key | fm 可开关可改名：`title`、`url`、`videoId`、`channel`、`duration`、`published` 等；时间戳 `[5:30](...&t=330s)`，本地视频用 `file:///dir/ID.mp4?t=秒` | 可选按 fm `url` 查重，默认关 | 可选 OpenAI、Gemini、Claude、OpenAI 兼容（含 Ollama） | MIT | 24 / 2026-10-26 / 2.0.28（2026-10-08）/ 7,957 |
| `media-extended` · [aidenlx/media-extended](https://github.com/aidenlx/media-extended) | 本地音视频、YouTube、Vimeo、B站（web viewer） | 播放器、时间戳笔记、截图、字幕跟读 | 本地文件或 URL；B站需登录的字幕才要账号；需另装 main daemon；仅桌面 | 媒体笔记 fm：`mx-uid`、`media: "[[x.mp4]]"` 或 `video:`/`audio:` URL；时间戳 `[01:35](...#t=95)`、`[[lecture.mp4#t=95]]`、区段 `#t=1:10,1:52`；截图 `mx-img-...-pt102s.webp`（[timestamps 文档](https://mx.aidenlx.site/docs/v4/concepts/timestamps)、[media-notes 文档](https://mx.aidenlx.site/docs/v4/concepts/media-notes)） | 不适用 | 读取 `.srt`/`.vtt`，无内置 ASR | 主分支无 LICENSE（闭源）；`v3` 分支 MIT | 961 / 2026-10-08 / 4.2.9（2026-10-08）/ 405,358 |
| `xiaohongshu-importer` · [bonchiang96/xiaohongshu-importer](https://github.com/bonchiang96/xiaohongshu-importer) | 小红书 | 粘贴分享文本生成一篇笔记 | 不带 cookie，`requestUrl` 取网页后解析 `window.__INITIAL_STATE__`，只能导入公开笔记 | fm：`title`、`source`、`date`、`Imported At`、`category`；可选下载到 `<目录>/media/{标题}-{i}-{时间戳}.jpg`，用 `![Image](../media/...)`，视频用 `<video src>` | 同名已存在就报错 | 无 | MIT | 141 / 2026-01-14 / 1.1.3（2026-01-14）/ 16,928 |
| `xhs-importer` · [lxl448080113/ob-Plugin](https://github.com/lxl448080113/ob-Plugin) | 小红书 | 上一项的改编版，fm 与目录可配置 | 同上 | fm 预设 `aliases`、`created`、`tags`、`上级概念`；图片 `附件/XHS`；视频只留远程链接 | 同名加后缀另建 | 无 | MIT | 15 / 2026-07-23 / 1.0.3（2026-07-23）/ 8,178 |
| `douyin-capture` · [lyxdream/obsidian-douyin-capture](https://github.com/lyxdream/obsidian-douyin-capture) | 抖音 | 分享链接 → 视频、图集、文案、转写 | 不需要 cookie；必须在本机跑 Python 后端 [obsidian-content-capture-backend](https://github.com/lyxdream/obsidian-content-capture-backend)（`127.0.0.1:5050`） | fm：`type: douyin`、`content_type`、`douyin_id`、`author`、`source`、`captured_at`、`whisper_model`、`tags`；附件 `attachments/douyin/{douyin_id}/video.mp4`、`01.jpg`；用 `![[...]]`（`docs/obsidian-plugin-contract.md`） | 同名加 `-2` 另建，`douyin_id` 不用于去重 | 本地 faster-whisper + FFmpeg；不做 OCR | MIT（后端无 LICENSE） | 38 / 2026-06-05 / 1.0.5（2026-06-05）/ 3,111 |
| `douyin-favorites-sync` · [yumingtao/douyin-favorites-sync](https://github.com/yumingtao/douyin-favorites-sync) | 抖音收藏夹 | 定时批量同步收藏 | 本机后端 [douyin-favorites-sync-backend](https://github.com/yumingtao/douyin-favorites-sync-backend)（FastAPI + SQLite，`127.0.0.1:8765`）；登录态靠浏览器会话 | fm 同上一类；正文分 `## 配图`、`## 文案`、`## 转写`；附件 `attachments/douyin/{douyin_id}/` | 按 `douyin_id` 跳过已导入的，旧笔记不动 | 可选本地 faster-whisper | MIT | 0 / 2026-08-28 / 1.0.7（2026-08-28）/ 237 |
| `anycontent-vault-importer` · [vancoder4-cyber/anycontent-obsidian-importer](https://github.com/vancoder4-cyber/anycontent-obsidian-importer) | 抖音、TikTok、公众号、YouTube | 链接 → 转写、正文或逐图 OCR → Markdown | 本机后端 [anycontent-obsidian-backend](https://github.com/vancoder4-cyber/anycontent-obsidian-backend)（`127.0.0.1:8080`）；视频 ASR 与图片 OCR 要 SiliconFlow key | fm：`title`、`source`、`author: ["[[作者]]"]`、`platform`、`source_type`、`video_id`/`post_id`/`article_id`、`created`、`published`、`description`、`tags: [clippings, ...]`；路径 `raw/{date}-{author}-{platform}-{title}.md` | 同路径直接覆盖 | 云端 SenseVoice 与 Qwen-VL（SiliconFlow） | MIT（后端 Apache-2.0） | 1 / 2026-05-24 / 1.0.2（2026-05-24）/ 1,375 |
| `wechat-obsync` · [less1001/obsync](https://github.com/less1001/obsync) | 公众号文章（经微信小程序） | 手机上收集，云端解析，插件拉取 | 6 位绑定码；云端 `ob.agentok.top`（Cloudflare Workers + D1） | fm：`source_url`、`title`、`account`、`author`、`published_at`、`saved_at`、`sync_id`；可选把图片下到 `附件资源/{标题}/img_N.ext` | 维护 `sync_id`→路径映射，存在就整篇覆盖 | 云端 Readability + Turndown；云端代码是否开源待核实 | MIT | 35 / 2026-08-15 / 0.4.10（2026-07-14）/ 6,649 |
| `wechat-note-helper` · [notesynchelper/obsidian-notehelper-v2](https://github.com/notesynchelper/obsidian-notehelper-v2) | 转发给服务号的公众号、小红书、知乎、聊天记录等 | 服务端抓取清洗，插件拉取 | 服务号给 API Key；免费额度 + 付费 | 由 Omnivore 插件 fork，fm 默认写 `id` | 同 id 内容变了才写；同名不同 id 另建 | 服务端解析 | MIT | 4 / 2026-10-19 / 3.2.17（2026-10-19）/ 6,996 |
| `wechat-inbox-sync` · [mingjuner123-spec/wechat-inbox-sync](https://github.com/mingjuner123-spec/wechat-inbox-sync) | 小程序收件箱：公众号、小红书、小宇宙、B站、抖音、视频号、音频、文档 | 「先存后理」：云端收件箱 → 本地笔记 | 绑定码；同步 API 在腾讯 CloudBase | fm 默认 `title`、`author`、`url`、`synced_at`、`source`、`views`、`likes` 等；目录 `临时收集/YYYY-MM-DD/` | 只给标记为 partial 且记录 id 一致的笔记补写，状态变了就拒写 | ASR 可选云端或本地 whisper.cpp；OCR 可选本地 RapidOCR；用 `child_process.execFile` 调用 | NOASSERTION；README 称 MIT + Commons Clause | 33 / 2026-10-28 / 1.3.213（2026-10-28）/ 11,587 |
| `video-transcript-fetch` · [larryxu001/video-transcript-obsidian-fetch](https://github.com/larryxu001/video-transcript-obsidian-fetch) | B站、YouTube | 批量取字幕成笔记，无字幕时可选云 ASR | B站扫码登录，会话明文存 `data.json`；ASR 自填 Groq/OpenAI key | fm 与 youtube-transcript-fetcher 同形；B站嵌 `player.bilibili.com` iframe；时间戳 `[m:ss](https://www.bilibili.com/video/BV.../?t=秒)` | 可选按 `url` 查重 | 平台字幕优先 | MIT | 0 / 2026-10-03 / 3.1.3（2026-10-01）/ 82 |
| `srt-subtitle-toolkit` · [lamvanbien65-crypto/obsidian-srt-toolkit](https://github.com/lamvanbien65-crypto/obsidian-srt-toolkit) | B站 | 下载视频 → CC/AI 字幕或本地转写 → 字幕笔记 | 可填 cookie 或透传 `yt-dlp --cookies-from-browser` | 无 fm；`{stem}-标准字幕.md`，时间是纯文本 `[mm:ss → mm:ss]`；另出 `.txt`/`.srt`/`.json` | 不适用 | 本地 Python + yt-dlp + ffmpeg + whisper-cli | MIT | 1 / 2026-08-17 / 0.1.3（2026-08-17）/ 135 |
| `zerocloud-notes` · [lamvanbien65-crypto/obsidian-zerocloud-notes](https://github.com/lamvanbien65-crypto/obsidian-zerocloud-notes) | 小红书、B站、YouTube、网页 | 图文转笔记；视频本地下载加口播字幕 | 本地读浏览器 cookie 库（经 macOS 钥匙串解密）生成 cookies.txt | 无 fm；`小红书剪藏/笔记/`、`小红书剪藏/下载/`；末行 `原文：[url](url)` | 按标题写，同名覆盖 | 本地 yt-dlp、ffmpeg、whisper.cpp、silero-vad；仅 macOS | MIT | 0 / 2026-09-09 / 0.4.6（2026-09-09）/ 1,996 |
| `omni-collector` · [simonemuller6127-png/omni-collector](https://github.com/simonemuller6127-png/omni-collector) | B站收藏、YouTube 点赞、小红书收藏等 | 给收藏建索引并整理；README 说不是完整备份 | 本机 Node 引擎（Playwright）；cookie 以 AES-256-GCM 加密存放 | fm：`platform`、`url`、`priority`、`organize_status`、`tags`、`topics`；正文 `<!-- OMNI_SYSTEM_START -->`…`<!-- OMNI_SYSTEM_END -->` 是系统区，其后是用户区（`apps/obsidian-plugin/src/markdown/markdown-builder.ts`） | 只替换系统区和 fm，用户区保留 | 可选 DeepSeek / OpenAI 兼容；数据在 SQLite | MIT | 2 / 2026-10-16 / 0.10.0（2026-10-16）/ 1,341 |
| `bilinote-sync` · [rui-0221/bilinote-sync](https://github.com/rui-0221/bilinote-sync) | B站（读 BiliNote 的本地产物） | 把 BiliNote 生成的笔记和截图导入 vault | 读 BiliNote 的 `note_results/*.json` 与 `static/screenshots`；插件零网络请求 | fm：`source: "BiliNote"`、`bilinote_task`、`bilinote_source_hash`、`video_id`、`platform`、`imported_at`；图片 `BiliNote笔记/附件/{taskId}/{名}-{hash12}.ext` | 从不覆盖；源变了另建；凭 fm 找回被移动的笔记 | 转写与总结由上游 BiliNote 完成 | MIT | 1 / 2026-10-05 / 1.4.3（2026-10-05）/ 36 |
| `obsidian-tweet-to-markdown` · [kbravh/obsidian-tweet-to-markdown](https://github.com/kbravh/obsidian-tweet-to-markdown) | X | 推文、线程 → Markdown | 自备 X API bearer token（付费档）；作者的免费 API 2023-04-27 停用 | fm：`author`、`handle`、`source`、`date`、`likes`、`retweets`、`replies`；可选图片下到 `assets/` | 同名存在就中止 | 无 | MIT | 220 / 2023-05-08 / 2.12.1（2022-12-05）/ 13,973；停更三年以上 |
| `podnotes` · [chhoumann/PodNotes](https://github.com/chhoumann/PodNotes) | 播客（RSS、Apple Podcasts 搜索、本地音频） | 播放器、时间戳、单集笔记、转写 | RSS；转写自填 OpenAI 或 Deepgram key | 单集 fm：`type: podcastEpisode`、`podcast: "[[节目]]"`、`date`、`tags`；时间戳 `obsidian://podnotes?...&time=761`（`src/utility/encodePodnotesURI.ts`） | 待核实 | feed 自带转写或云 ASR | MIT | 266 / 2026-10-25 / 2.24.0（2026-10-25）/ 67,710 |
| `snipd-official` · [snipd-app/snipd-obsidian](https://github.com/snipd-app/snipd-obsidian) | Snipd 播客片段 | 同步片段、转写、AI 摘要 | Snipd 账号浏览器授权 | 服务端渲染；`Snipd/Data/{节目}/{单集}.md`；自带两个 Bases 视图；属性字段名由服务端生成（待核实） | 每个文件存 MD5；没改过整篇重写，改过只追加 | 转写与摘要在 Snipd 云端 | MIT | 54 / 2026-09-28 / 1.3.2（2026-09-28）/ 79,410 |
| 简悦 `simpread`（不在社区目录）· [Kenshin/simpread-obsidian-plugin](https://github.com/Kenshin/simpread-obsidian-plugin) | 网页剪藏、标注 | 浏览器扩展把全文或标注导入 Obsidian | 四种方式：URL Scheme（已废弃）、Local REST API 插件、官方插件读 `simpread_config.json`、同步助手导出 md + assets；自动导入需高级账户（[discussion #2902](https://github.com/Kenshin/simpread/discussions/2902)） | 模板变量 `{{url}}`、`{{title}}`、`{{author}}`、`{{desc}}`、`{{note}}`、`{{tags}}` 等 | 待核实 | 无 | 无 LICENSE，只发编译后的 `main.js` | 72 / 2026-07-30 / 4.0.0（2026-07-21）/ 无统计 |

**Web Clipper 模板的补充。** 机制见 [02 §10](02-adapter-ecosystems.md#10-obsidian-web-clipper-模板)。这里只补两件和导出有关的事：

- 默认模板（`src/managers/template-manager.ts` 的 `createDefaultTemplate`）：`path: 'Clippings'`、`noteNameFormat: '{{title}}'`；属性依次是 `title`、`source`（`{{url}}`）、`author`（拆分后写成 wikilink，类型 multitext）、`published`（date）、`created`（date）、`description`、`tags: clippings`（multitext）。来源：<https://github.com/obsidianmd/obsidian-clipper/blob/main/src/managers/template-manager.ts>。Clipper 1.7.1（2026-07-22），5412 ★。
- [02 §10](02-adapter-ecosystems.md#10-obsidian-web-clipper-模板) 待核实的「clipper-templates」示例仓库最可能是 [kepano/clipper-templates](https://github.com/kepano/clipper-templates)（名字吻合，账号与 [§3.1](#31-obsidian) 那篇 Obsidian 官方博文的作者 kepano 相同；帮助页是否确实链到它待核实）：MIT，1490 ★，推送 2026-03-13；`templates/` 下 12 个 JSON（arxiv、chatgpt、goodreads、google-maps、imdb、letterboxd、product、recipes、redfin、wikipedia、youtube 等），没有 B站、小红书、公众号。中文平台的成套模板集合没有找到（待核实）。

**Media Extended 闭源时间线**（`gh api` 提交与 release 记录）：

- 最后一个开源版本是 `v3` 分支的 3.2.6（2024-08-16），MIT。
- 主分支历史在 2025-04-23 重置，此后只放 README、CHANGELOG、manifest 与 web-clipper 模板，不含源码；4.0.0-beta.1 发布于 2025-04-24，4.0.0 正式版 release 时间 2025-09-16。
- [v4 发布说明](https://mx.aidenlx.site/blog/v4-release)说 B站等平台的直接嵌入暂时移除；现行 [web-viewer 文档](https://mx.aidenlx.site/docs/v4/concepts/web-viewer)把 B站列为支持时间戳与截图（仅桌面、需 main daemon）。

<a id="12-独立工具"></a>
### 1.2 独立工具

| 项目 | 平台 | 做什么 | 输入与登录态 | 输出格式 | 导出目标及方式 | 依赖 | 许可证 | ★ / 推送 / release |
|---|---|---|---|---|---|---|---|---|
| [JefferyHcool/BiliNote](https://github.com/JefferyHcool/BiliNote) | B站、YouTube、抖音、快手、本地视频（代码里另有小宇宙下载器） | 视频链接 → AI 结构化笔记，可插截图、原片跳转、RAG 问答 | 粘贴链接或上传；cookie 按平台存 `config/downloader.json`，浏览器扩展可同步浏览器 Cookie | Markdown 无 fm，首行 `> 来源链接：<url>`；`*Content-[mm:ss]` 替换成 `[原片 @ mm:ss](https://www.bilibili.com/video/<BV>&t=<秒>)`；截图 `![](/static/screenshots/<file>)` 指向自家后端（`backend/app/utils/note_helper.py`） | 复制、下载 Markdown，导出 PDF；README 里「导出到 Notion」的 TODO 未完成 | 本地 Fast-Whisper、MLX-Whisper；云端 Groq、必剪、快手；LLM 走 OpenAI 兼容 | MIT | 7652 / 2026-09-01 / v2.4.5（2026-08-25）；Docker、Tauri 桌面版、Web、扩展 |
| [JimmyLv/BibiGPT-v1](https://github.com/JimmyLv/BibiGPT-v1) | B站、YouTube | 抓字幕 → LLM 摘要 | B站字幕靠服务端环境变量 `BILIBILI_SESSION_TOKEN` | 页面内摘要 | flomo webhook（`hooks/notes/flomo.ts`） | 只用字幕；LLM 走 OpenAI 兼容 | GPL-3.0 | 6263 / 2026-05-04 / v2.38.0（2023-07-02），低频维护 |
| BibiGPT 商业版 [bibigpt.co](https://bibigpt.co)（无公开仓库） | B站、抖音、快手、小红书、YouTube、TikTok、喜马拉雅、小宇宙、播客、本地文件等 | 摘要、章节、字幕、对话 | 账号；网页、桌面端、扩展；有开放 API | Markdown、TXT、PDF、Word；桌面端「自动导出」可写属性、时间戳回链，语法待核实 | Notion（token）、flomo（webhook）、飞书（说法不一，待核实）、Readwise、Cubox、Obsidian/Logseq（网页端拉起本地应用，桌面端写本地文件夹） | 云端为主，有「本地隐私模式」 | 闭源，订阅制 | 文档 <https://docs.bibigpt.co/llms-full.txt> |
| [wechat-article/wechat-article-exporter](https://github.com/wechat-article/wechat-article-exporter)（原 `jooooock/...` 现 404） | 公众号 | 批量下载某个公众号的全部文章 | 用自己的公众号后台扫码登录 | html、json、excel、txt、md、docx | 本地文件 | 无 | MIT | 13108 / 2026-09-30 / 0.3（2025-03-19）；README 称 2026-07-30 停止维护，因微信关闭了它依赖的接口 |
| [jackwener/wechat-article-to-markdown](https://github.com/jackwener/wechat-article-to-markdown) | 公众号单篇 | 单篇 → Markdown | 文章 URL，用 Camoufox 抓取，无需登录 | `output/<标题>/<标题>.md` + `images/`；头部是引用块写的公众号、时间、原文链接，无 fm | 本地文件；附 `SKILL.md` | 无 | API 为 none（README 写 MIT） | 1076 / 2026-03-22 / 无 release（PyPI 发布） |
| [fengxxc/wechatmp2markdown](https://github.com/fengxxc/wechatmp2markdown) | 公众号单篇 | 单篇 → Markdown，CLI 或 web 服务 | URL，无需登录 | `--image=url\|save\|base64` | 本地文件 | 无（Go） | MIT | 266 / 2025-03-13 / v1.1.11（2025-03-13） |
| [cooderl/wewe-rss](https://github.com/cooderl/wewe-rss) | 公众号 | 借微信读书账号生成 RSS | 扫码登录微信读书 | Feed，可输出全文 | RSS 阅读器 | 无 | MIT | 9637 / archived / release-20241215130232 |
| [rachelos/we-mp-rss](https://github.com/rachelos/we-mp-rss) | 公众号 | 定时订阅，转 Markdown 或 PDF，RSS 与 Webhook | 扫码授权 | Markdown、PDF、RSS | Webhook、RSS | 无 | NOASSERTION（README 写 MIT） | 5060 / 2026-10-20 / v1.5.3（2026-08-13） |
| [JoeanAmier/XHS-Downloader](https://github.com/JoeanAmier/XHS-Downloader) | 小红书 | 只下载图文和视频文件，可取收藏链接；不生成笔记 | cookie 可选 | 媒体文件 + 作品数据 | 无；有 API 与 MCP 模式 | 无 | GPL-3.0 | 13136 / 2026-10-23 / 2.8（2026-09-12） |
| [zhulin025/xiaohongshu-exporter](https://github.com/zhulin025/xiaohongshu-exporter) | 小红书收藏夹 | Chrome 扩展，只采集元数据 | 页面内采集 | JSON、CSV、HTML、Markdown | 飞书多维表格（自建应用）、Notion 数据库（token） | 无 | API 为 none（README 写 MIT） | 13 / 2026-01-14 / 无 release |
| [fluoxtine2004/douyin-favorites-organizer](https://github.com/fluoxtine2004/douyin-favorites-organizer) | 抖音收藏夹（本人账号） | 抓收藏 → 下载 → 逐字稿 + 关键帧 → Obsidian 笔记与周报 | Playwright 扫码登录 | 中文 fm 键 `标题/作者/收藏夹/大类/原链接/抓取时间/时长/tags`；逐字稿放折叠 callout；末尾 `## ✍️ 我的批注`（`scripts/generate_outputs.py`） | 直接写 vault；可打包离线静态站 | 本地 faster-whisper；总结交给宿主 Agent | MIT | 5 / 2026-10-14 / v1.0.0（2026-10-14） |
| [Evil0ctal/Douyin_TikTok_Download_API](https://github.com/Evil0ctal/Douyin_TikTok_Download_API) | 抖音、TikTok、B站 | 自托管数据 API，可本地转文字 | 自托管 | JSON、媒体、文字稿 | 不导出笔记；提供 MCP | 本地 FunASR Paraformer | Apache-2.0 | 20961 / 2026-10-27 / v5.1.8（2026-10-27） |
| [suonian/vidknot](https://github.com/suonian/vidknot) | YouTube、B站、抖音、小红书、X 等 | 音频 → 两路 ASR 校正 → LLM 结构化笔记 | 抖音可配 cookie 文件 | Markdown；写 Obsidian 时加 fm `title/author/source/duration/tags` | Obsidian 写文件；飞书 docx（Markdown 转 blocks）；Notion API；语雀 `POST /api/v2/repos/.../docs`（`src/vidknot/adapters/*_writer.py`） | 云端 SenseVoice + 本地 faster-whisper；LLM OpenAI 兼容；有 CLI、FastAPI、MCP | MIT | 27 / 2026-10-28 / v0.6.10（2026-09-25） |
| [chubbyguan/chubbyskills](https://github.com/chubbyguan/chubbyskills) | 抖音、B站、小红书、公众号、X、YouTube 等 | 14 个 Agent Skill：存图、转文字稿、订阅、简报 | 字幕优先；可读浏览器 cookie | `creator-vault/00_Inbox/<标题>--<来源哈希>.md`，fm `title/platform/source/author/created/likes` | 本地 Markdown | 本地 SenseVoice-Small，可选云端；LLM 由宿主 Agent 提供 | MIT | 1285 / 2026-10-14 / v0.15.0（2026-10-08） |
| [lucky-Luor/bilibili-douyin-notes](https://github.com/lucky-Luor/bilibili-douyin-notes) | B站、抖音、YouTube、小红书视频 | Agent Skill：字幕 + 本地 ASR + 截图 → 课堂笔记 | 默认免登录 | Markdown + 按时间戳抽的帧 | 本地文件 | 本地 faster-whisper；写笔记交给宿主 Agent | MIT | 160 / 2026-10-27 / 无 release |
| [malinkang/weread2notion](https://github.com/malinkang/weread2notion) | 微信读书 | 同步书籍、划线、想法到 Notion | 微信读书 API Key + `NOTION_TOKEN`，GitHub secrets | Notion 页，父级 `data_source_id`；属性 `BookId`、`ISBN`、`作者`、`评分`、`状态`、`阅读进度` 等；划线写成 quote 与 callout 块 | GitHub Actions 每天定时运行；书有更新时删页重写，README 警告页内自加的笔记会被删（`src/weread2notion/cli.py`、`action.yml`） | 无 | API 为 none | 2935 / 2026-10-14 / 无 release；Pro 版 [weread2notion-pro](https://github.com/malinkang/weread2notion-pro) 自称已不能用 |

<a id="13-其他笔记软件的同类插件"></a>
### 1.3 其他笔记软件的同类插件

| 项目 | 宿主 | 平台 | 写入格式 | 写入方式 | 再次同步时 | 许可证 | ★ / 推送 / release |
|---|---|---|---|---|---|---|---|
| [readwiseio/logseq-readwise-official-plugin](https://github.com/readwiseio/logseq-readwise-official-plugin) | Logseq | Readwise 的全部来源 | 每个来源一页；首块是 `key:: value` 属性（服务端模板渲染）；高亮是子块；封面远程 URL（[Readwise 文档](https://docs.readwise.io/readwise/docs/exporting-highlights/logseq)） | 宿主 API `logseq.Editor.createPage` / `insertBatchBlock` + 云 API | 只追加 | GPL-3.0 | 66 / 2024-03-25 / v1.4.11（2024-03-25） |
| [stillfox-lee/cubox-2-logseq](https://github.com/stillfox-lee/cubox-2-logseq) | Logseq | Cubox | page properties `cubox-id`、`cubox-url`、`original-url`、`tags` 等；用 `logseq.DB.q` 按 `cubox-id` 查重 | 宿主 API + Cubox 云 API | 删除全部内容块后重插 | MIT | 2 / 2025-06-24 / v1.0.0（2025-06-24） |
| [briansunter/logseq-get-youtube-captions](https://github.com/briansunter/logseq-get-youtube-captions) | Logseq | YouTube | 字幕切块，每块以 `{{youtube-timestamp <秒>}}` 开头 | 宿主 API | 不适用 | MIT | 55 / 2026-04-23 / v1.3.3（2026-04-23） |
| [duiliuliu/logseq-flomo-plugin](https://github.com/duiliuliu/logseq-flomo-plugin) | Logseq | flomo | 每条 memo 一块，属性 `memo_link`、`created-at` | 宿主 API；flomo session 作为 URL 查询参数发给作者的第三方代理 | 不适用 | MIT | 6 / 2022-08-20 / 0.0.9（2022-08-20） |
| [ahonn/logseq-plugin-weread-importer](https://github.com/ahonn/logseq-plugin-weread-importer) | Logseq | 微信读书（手动粘贴导出文本） | `author:: [[...]]`、`date:: [[...]]`；章节 → 划线 → 想法三层块 | 宿主 API | 追加 | MIT | 20 / archived / v1.1.0（2023-02-02） |
| marketplace 的 `logseq-weread-sync` | Logseq | 微信读书 | — | — | — | — | manifest 指向的 `sethyuan/logseq-plugin-weread-sync` 返回 404 |
| [leofang230316-prog/siyuan-plugin-weread-sync](https://github.com/leofang230316-prog/siyuan-plugin-weread-sync) | 思源 | 微信读书 | 每本书一篇文档（`/api/filetree/createDocWithMd`），块属性 `weread-bmid`、`weread-hash` 等（无 `custom-` 前缀，内核是否保留待核实） | 内核 HTTP API；网络请求经 `/api/network/forwardProxy` | 三方比对：本地块 kramdown、上次快照 hash、远端；两端都改时保留本地并在旁边插入远端新版（`src/core/sync.ts`、`merge.ts`） | MIT | 0 / 2026-09-09 / v1.2.4（2026-09-09） |
| [onemorework/SiyuanWechatSync](https://github.com/onemorework/SiyuanWechatSync) | 思源 | 发给服务号或小程序的文本、图片、链接 | 追加时间标题块；链接转成子文档并插块引用；图片走 `/api/asset/upload` | 内核 HTTP API + 作者中转服务（README 承认服务号消息后台可查） | 追加 | LGPL-3.0 | 8 / 2026-01-26 / v1.1.0（2026-01-08） |
| [winter60/plugin-flomo-sync](https://github.com/winter60/plugin-flomo-sync) | 思源 | flomo | HTML 经 turndown 转 Markdown；图片 `/api/file/putFile` 存 `assets/flomo/` | 内核 HTTP API；在插件里存 flomo 邮箱密码 | 增量；可选给 memo 回写「已同步」标签 | MIT | 16 / 2023-12-14 / v0.1.0（2023-12-14） |
| [mm-o/siyuan-media-player](https://github.com/mm-o/siyuan-media-player) | 思源 | B站、本地、WebDAV、网盘 | 块属性 `custom-media="timestamp"`、`custom-timestamp="02:03"`、`custom-loop-start` 等（`docs/媒体播放器自定义属性说明.md`） | 推断为内核 API（仓库只有 `package.zip` 与文档） | 不适用 | MIT | 54 / 2026-10-25 / v2.2.0（2026-10-25） |
| Readwise → Notion（官方，无仓库） | Notion | Readwise 的全部来源 | 一张「Readwise」数据库，每个文档一页，新高亮追加到页底（[文档](https://docs.readwise.io/readwise/docs/exporting-highlights/notion)） | Notion 云 API | 只追加；改默认属性类型会让同步失效 | 闭源 | — |
| [ybh-ybh/know-where](https://github.com/ybh-ybh/know-where) | 飞书 | 公众号、小红书、抖音、B站等 | 多维表格记录：平台、分类、摘要、原链接 | 飞书云 API；企业自建应用长连接收事件 | 不适用 | Apache-2.0 | 99 / 2026-10-23 / 无 release |
| [teemoweng/lark-video2note](https://github.com/teemoweng/lark-video2note) | 飞书 | 抖音、B站、小红书、YouTube | 飞书 docx | 官方 `lark-cli`（云 API）；Claude Code skill | 不适用 | MIT | 21 / 2026-05-20 / 无 release |
| [lproc2006/feishu-clipper-pro](https://github.com/lproc2006/feishu-clipper-pro) | 飞书 | 网页 | 云文档 + 一条多维表格记录 | `lark-cli` | 两层查重；一端删另一端跟着删 | MIT | 4 / 2026-09-09 / v1.1.7（2026-09-09） |
| [tllovesxs/wandao](https://github.com/tllovesxs/wandao) | 飞书 Wiki、语雀、印象笔记等 | 公众号单篇、知乎、CSDN 等 | 保留目录、图片、附件 | 浏览器自动化，用用户自己的登录态（是否也调开放 API 待核实） | 不适用 | AGPL-3.0 | 1135 / 2026-10-23 / v1.5.3（2026-10-14） |
| [Kenshin/simpread](https://github.com/Kenshin/simpread)（简悦） | 语雀、Notion、本地 | 任意网页 | 按 Markdown 保存；同步助手把 md + assets 写本地目录 | 语雀、Notion 走云 API（OAuth）；飞书不在支持列表（待核实） | 待核实 | GPL-3.0 | 8744 / 2026-09-05 / 2.2.0.520（2025-07-13） |

<a id="14-共性与对歇存的含义"></a>
### 1.4 共性与对歇存的含义

1. **frontmatter 键名没有共识，Web Clipper 默认模板是最接近的基线。** 光「原文链接」就有 `source`、`url`、`source_url`、`original_url` 四种叫法；中文键（`标题`、`原链接`）与英文键并存。各家共有最多的是 `title`、`author`、`tags`、`platform`、`created`/`published`，与 Clipper 默认的 `title/source/author/published/created/description/tags` 基本重合。歇存导出的 frontmatter 建议直接沿用 Clipper 默认键名，再加 `xiecun_id`、`platform`、`kind` 等歇存自己的键，这样用户已有的 Clipper 笔记、Bases 视图和检索习惯不用改。
2. **去重 ID 写在 frontmatter 的才经得起插件数据丢失。** 写进 fm 的有 `bookId`（weread）、`id`（omnivore、cubox）、`douyin_id`、`bilinote_task` + `bilinote_source_hash`、`mx-uid`；存在插件 `data.json` 里的有 Readwise 的 `booksIDsMap` 和 Snipd 的 `fileHashMap`，删掉 `data.json` 就会重复导入。只有 bilinote-sync 明确处理了用户重命名、移动笔记：凭 fm 收据找回。这与歇存「文件为真相源」一致。
3. **再次同步对用户编辑有四种策略**：用哈希判断是否改过，改过就只追加（Readwise、Snipd）；用区段标记只替换机器区（omni-collector、Omnivore 单文件模式、weread 的 Daily Note）；只建不改（cubox、douyin-favorites-sync、bilinote-sync）；直接覆盖（Omnivore 多文件、obsync、anycontent、flomo、zerocloud-notes）。覆盖派会丢用户的字，[§5.9](#59-读回双向的先例与歇存的做法) 的设计以前两种为基础。
4. **附件目录按条目 ID 分子目录最稳。** 两款抖音插件的 `attachments/douyin/{douyin_id}/`、bilinote-sync 的 `附件/{taskId}/{名}-{hash12}` 不会撞名；只留远程 URL 的（weread 封面、Readwise、Cubox、Snipd）离线就打不开。歇存的 `images/`、`media/` 本来就按条目放，导出时保留这个结构即可。
5. **时间戳链接三种写法并存**：平台 URL 加 `?t=秒`（ytranscript、video-transcript-fetch、BiliNote）；媒体片段 `#t=`（Media Extended）；插件自定义协议（PodNotes 的 `obsidian://podnotes?...&time=`）。Obsidian 本身不支持音视频 `#t=`（见 [§5.1](#51-obsidian)），所以歇存导出要二选一：给平台原链接加 `?t=`，或者在用户装了 Media Extended 时写 `[[x.mp4#t=95]]`。
6. **平台抓取逻辑正在从插件里搬出去。** 24 个社区插件里，8 个把抓取写在插件 TS 里（两款小红书、三款 YouTube/B站字幕、flomo、Media Extended、PodNotes），7 个放进本地 Python/Node 后端或外部 CLI（两款抖音、anycontent、两款 lamvanbien65、omni-collector、bilinote-sync），这 7 个都是 2026 年建的仓库。这些本地后端做的正是歇存「获取方式」的事，说明「抓取在独立进程、笔记软件只收结果」这条路已经有人在走。
7. **云端依赖与凭据处理是普遍问题。** 9 个插件离开厂商云就拿不到内容，Omnivore 关停后插件随之失效。凭据处理有明显反例：B站会话明文存 `data.json`（video-transcript-fetch）、flomo session 放进 URL 发给第三方代理（logseq-flomo-plugin）、在插件里保存账号密码（plugin-flomo-sync、flomo-importer）。
8. **没有一个工具从笔记软件读回改动。** 保护本地编辑的做法最多到「只追加」「分区保留」「三方合并后两版并存」（思源 weread-sync）。写 Notion、飞书、语雀的工具全部走云 API 或浏览器自动化，没有工具用飞书的 Markdown 导入任务接口。
9. **抖音、小红书类工具都很小。** 除 chubbyskills（1285 ★）与 bilibili-douyin-notes（160 ★）外都不到 30 ★，多以 Agent Skill 或 MCP 形式发布；公众号导出最有名的 wechat-article-exporter 已因接口关闭停止维护，WeWe RSS 已归档。上游接口随时会断，与 [PRD §9](../PRD.md#9-风险) 的判断一致。

<a id="2-转换类插件与工具"></a>
## 2. 转换类插件与工具

本节只看「插件层」：每个插件怎么调用引擎（宿主进程内、外部 CLI、本地 HTTP、云 API），以及能抽出来复用的是什么。引擎本身的对比在 [04 §2](04-tech-options.md#2-ocr-sidecar)、[04 §3](04-tech-options.md#3-asr-sidecar)。「证据」路径都相对各仓库根目录。

<a id="21-转写插件"></a>
### 2.1 转写插件

| 插件 | 宿主 | 运行方式（证据） | 底层引擎 | 输出 | 能复用的部分 | 许可证 | ★ / 推送 / release / 下载 |
|---|---|---|---|---|---|---|---|
| [Whisper](https://github.com/nikdanilov/whisper-obsidian-plugin) | Obsidian | 云 API：multipart POST 到 `api.openai.com/v1/audio/transcriptions`，URL 可改，README 写明支持 Groq、Azure 等兼容端点（`src/AudioHandler.ts` L61–140）；key 存 `app.secretStorage` | OpenAI `whisper-1` 或兼容端点 | 新建笔记或插入光标处 | 协议：歇存 ASR 若提供 OpenAI 兼容端点，这个插件可以直连 | MIT | 380 / 2026-04-07 / 1.9.1（2026-04-07）/ 59,217 |
| [Transcription](https://github.com/djmango/obsidian-transcription) | Obsidian | 本地 HTTP：唯一后端是 whisper-asr-webservice，请求 `${baseUrl}/asr?output=json&word_timestamps=true`，默认 `http://localhost:9000`（`src/transcribe.ts` L130–144） | openai-whisper / faster-whisper / whisperX | 插入光标处，可带行首时间戳 | 协议：`POST /asr` | MIT | 224 / 2025-09-03 / 3.4.0（2024-04-12）/ 33,344 |
| [Scribe](https://github.com/mikealicea/obsidian-scribe)（`mikodin/obsidian-scribe` 已重定向） | Obsidian | 云 API：OpenAI SDK（可设 base URL）、AssemblyAI、Deepgram、ElevenLabs、Gemini、Mistral（`src/aiProviders/transcription/*.ts`） | 多家云 ASR 与 LLM | 按模板生成转写 + 摘要 + mermaid | 协议：OpenAI 兼容转写与 chat | MIT | 86 / 2026-07-15 / 2.4.3（2026-07-15）/ 20,605 |
| [Audio Notes](https://github.com/jjmaldonis/obsidian-audio-notes) | Obsidian | 云 API：Deepgram 与作者自建的 AWS API Gateway | Deepgram | 转写存成 vault 里的 JSON 或 `.srt`，笔记里只引用 | 做法：转写作为附件文件，笔记引用它 | MIT | 186 / 2023-05-02 / 0.7.2（2023-05-02）/ 29,471 |
| [NeuroVox](https://github.com/Synaptic-Labs-AI/NeuroVox) | Obsidian | 云 API；视频抽音轨在渲染进程跑 ffmpeg.wasm，运行时从 unpkg 拉 `@ffmpeg/core`（`src/utils/VideoProcessor.ts` L27–30）；本地 transformers.js 已停用，注释说明 Obsidian 审核不允许从 CDN 动态加载脚本 | OpenAI、Groq、Deepgram、AssemblyAI | 插入 callout | 协议：OpenAI 兼容转写 | MIT | 33 / 2026-10-01 / 1.1.7（2026-08-26）/ 6,567 |
| [AI Transcriber](https://github.com/mssoftjp/obsidian-ai-transcriber) | Obsidian | 云 API，只接 OpenAI，带 `timestamp_granularities[]`；媒体在渲染进程解码，自设 128MiB / 2 小时上限；可选 `fvad.wasm` VAD | OpenAI 转写 | 新建转写笔记 | 协议；去幻觉清洗规则可参考 | MIT | 4 / 2026-10-27 / 0.11.5（2026-09-10）/ 4,692 |
| [Speech Kit](https://github.com/brittain9/speech-kit-obsidian-plugin)（id `local-dictation`） | Obsidian | 外部进程：`spawn` Rust sidecar，stdin/stdout 上用长度帧（1 字节类型 + 长度，JSON 与 16 kHz PCM，单帧上限 16 MiB）（`src/sidecar/sidecar-process.ts`、`native/src/protocol.rs`） | `whisper-rs`（Metal/CUDA）、`ort`、`llama-cpp-2` | 写入当前笔记 | sidecar 本身（MIT）；帧格式是自定的 | MIT | 26 / 2026-10-27 / 2026.10.1（2026-10-03）/ 6,264 |
| [Offline Whisper](https://github.com/manthey/offline-whisper) | Obsidian | 外部 CLI：首次使用时下载 whisper.cpp 二进制（三个下载源里两个是非官方仓库），之后 `spawn(whisperPath, ['-m', model, '-f', wav, '-nt'])`（`src/desktop-transcriber.js` L37–50）；移动端改用 transformers.js | `whisper-cli` | 光标处纯文本 | whisper.cpp CLI | MIT | 5 / 2026-10-28 / 1.1.6（2026-10-28）/ 3,702 |
| [Voice Scribe](https://github.com/mrrepac/voice-scribe) | Obsidian | 进程内：Worker 里跑 transformers.js + onnxruntime-web（WebGPU 或 wasm），桌面默认 large-v3-turbo 约 720 MB（`src/engine.ts`） | Whisper ONNX、GigaAM | 插入笔记 | 否 | MIT | 2 / 2026-10-13 / 0.5.0（2026-10-13）/ 3,723 |
| [logseq-transcriber](https://github.com/marijnbent/logseq-transcriber) | Logseq | 云 API：Deepgram（`index.ts` L69–74） | Deepgram | 插入兄弟块 | 否 | MIT | 3 / 2025-05-20 / 0.1.5（2025-05-20） |
| [logseq-whisper-subtitles](https://github.com/usoonees/logseq-plugin-whisper-subtitles) + [server](https://github.com/usoonees/logseq-whisper-subtitles-server) | Logseq | 本地 HTTP `127.0.0.1:5014`，服务端 Flask + openai-whisper | openai-whisper | 带时间戳的字幕块 | 协议私有，服务端无许可证，不如用 speaches 或 whisper-asr-webservice | 插件 MIT，服务端无 | 23 / 2023-10-29 / v1.1.3；服务端 18 / 2024-08-20 / 无 release |
| [voice2text](https://github.com/chenxuan520/siyuan-plugin-voice2text) | 思源 | 云 API（小米 MiMo ASR）或浏览器 Web Speech；经内核出站代理 `/api/network/forwardProxy` 解决跨域 | MiMo、`webkitSpeechRecognition` | 光标处插入 | 做法：由宿主代为联网 | MIT | 0 / 2026-07-21 / v0.1.4（2026-07-21） |
| [speech2text](https://github.com/FlorianZierer/speech2text-transcription-plugin-siyuan) | 思源 | 云 API，端点可填本地；自动补 `/audio/transcriptions` | 任意 OpenAI 兼容转写 | 预览确认后插入 | 协议 | MIT | 0 / 2026-10-15 / v1.0.2（2026-10-15） |

任务候选里的思源 [TCOTC/Whisper-Plus](https://github.com/TCOTC/Whisper-Plus) 不是转写插件，是「Whisper」主题的配套插件（`src/` 下只有配色与样式模块），已换成 voice2text。

<a id="22-ocr-插件"></a>
### 2.2 OCR 插件

| 插件 | 宿主 | 运行方式（证据） | 底层引擎 | 输出 | 能复用的部分 | 许可证 | ★ / 推送 / release / 下载 |
|---|---|---|---|---|---|---|---|
| [Text Extractor](https://github.com/scambier/obsidian-text-extractor) | Obsidian | 进程内：tesseract.js 3 跑在 Web Worker 池（`lib/src/ocr/ocr-manager.ts`）；PDF 用编译成 wasm 的 pdf-extract；macOS 开 `useSystemOCR` 时 `spawn('osascript')` 调 Vision（`lib/src/ocr/ocr-mac.ts`） | Tesseract.js / Apple Vision | 不写笔记；按内容哈希缓存 `.obsidian/plugins/text-extractor/cache/<hash>.json`，随 vault 同步；对外 API `getTextExtractor()?.extractText(file)` | 做法：内容哈希缓存 + `extractText(file)` 契约 | GPL-3.0 | 649 / 2026-09-07 / 0.7.0（2025-12-26）/ 342,740 |
| [Omnisearch](https://github.com/scambier/obsidian-omnisearch) | Obsidian | 进程内 MiniSearch；图片与 PDF 文本经插件间 API 从 Text Extractor 取；可选本地 HTTP `/search`（`src/tools/api-server.ts`） | MiniSearch | 只建索引 | 做法：OCR 与检索拆成两个插件 | GPL-3.0 | 2183 / 2026-10-24 / 1.32.0（2026-10-24）/ 2,052,510 |
| [Taskbone](https://github.com/schlundd/obsidian-ocr-plugin) | Obsidian | 云 API，闭源：`src/main.ts` 只 re-export 私有包，产物里唯一业务域名 `app.taskbone.com` | 未公开 | 模板化插入 | 否 | GPL-3.0 | 103 / 2024-09-11 / 2.3.2（2024-09-11）/ 27,693 |
| [image-ocr](https://github.com/kaffarell/obsidian-tesseract-ocr) | Obsidian | 外部 CLI：`spawn(tesseract, [path, '-', '-l', lang])`，读 stdout（`src/main.ts` L153–179） | Tesseract CLI | 在图片嵌入后插 `<details>` | `tesseract img - -l chi_sim+eng` | MIT | 22 / 2023-12-05 / 0.3.8（2023-08-14）/ 8,296 |
| [TextLens](https://github.com/Nexround/obsidian-text-lens) | Obsidian | 混合：默认 PaddleOCR 经 `onnxruntime-node`（N-API）在宿主进程跑，首次运行从 npm 下载原生扩展；系统 OCR 走独立进程（macOS Swift helper 安装前校验 SHA-256，Windows 用 PowerShell），stdio 交换 JSONL（`src/native-process.ts`、`src/native-helper.ts`） | PaddleOCR ONNX / Vision / Windows.Media.Ocr | 图片下方插入 callout | Vision helper 与 JSONL 协议 | MIT | 1 / 2026-10-07 / 1.4.0（2026-10-07）/ 1,334 |
| [Power Extract](https://github.com/obsidian-power-plugins/power-extract) | Obsidian | 外部进程：常驻 `powershell.exe`（用 `%SystemRoot%\System32\...` 绝对路径），stdin 每行 `id\tpath`，stdout 回 JSON（`src/worker.ts`、`src/ocr.ts`） | Windows.Media.Ocr | 缓存随 vault；暴露与 Text Extractor 同形的 API | PowerShell 脚本与行协议可直接用于 Windows sidecar | MIT | 2 / 2026-08-28 / 1.1.8（2026-08-28）/ 2,422 |
| [Lens OCR](https://github.com/erincayaz/obsidian-lens-ocr) | Obsidian | 外部 CLI：macOS 先 `screencapture -i`，首次用用户机器上的 `swiftc` 编译内嵌 `ocr.swift`；Windows 用 PowerShell（`src/services/swift-runner.ts`） | Vision / Windows.Media.Ocr | 插入光标处 | 思路可用，应改为预编译签名的 helper | GPL-3.0 | 5 / 2026-06-26 / 0.1.1（2026-06-26）/ 1,071 |
| [AI Image OCR](https://github.com/rootiest/obsidian-ai-image-ocr) | Obsidian | 云 API 或本地 HTTP：OpenAI `/v1/chat/completions`、Gemini；README 写明支持 Ollama、LM Studio | 视觉大模型 | 插入笔记 | 协议：chat completions 传 base64 图片 | MIT | 51 / 2026-05-12 / 0.9.2（2026-05-12）/ 12,302 |
| [logseq-ocr](https://github.com/xxchan/logseq-ocr) | Logseq | 进程内 tesseract.js | Tesseract.js | 插入块 | 否 | MIT | 24 / 2023-05-19 / v0.0.1（2023-04-30） |
| [siyuan-plugin-paddleocr](https://github.com/Achuan-2/siyuan-plugin-paddleocr) | 思源 | 进程内：`@paddleocr/paddleocr-js` + onnxruntime-web，Worker 里优先 WebGPU，失败回落 WASM；模型经 `/api/network/forwardProxy` 下载进工作空间（`src/ocr.ts`、`src/modelStore.ts`） | PP-OCR ONNX | 写入思源资源的 OCR 文本（`/api/asset/setImageOCRText`），可搜索 | 否 | AGPL-3.0 | 0 / 2026-10-10 / v0.2.1（2026-10-10） |
| [siyuan-plugin-ocr-api](https://github.com/hogmoff/siyuan-plugin-ocr-api) | 思源 | 云 API：默认 `api.mistral.ai/v1/ocr`（`src/api/ocr-service.ts` L37） | Mistral OCR | 逐页 Markdown 经 `createDocWithMd` 建文档 | 协议 | MIT | 0 / 2025-12-19 / v0.1.2（2025-12-19） |

<a id="23-摘要与-llm-插件"></a>
### 2.3 摘要与 LLM 插件

| 插件 | 宿主 | 运行方式（证据） | 输出 | 能复用的部分 | 许可证 | ★ / 推送 / release / 下载 |
|---|---|---|---|---|---|---|
| [Copilot](https://github.com/logancyang/obsidian-copilot) | Obsidian | 混合：BYOK 云 provider，也能接 Ollama `localhost:11434/v1`、LM Studio `localhost:1234/v1`（`docs/llm-providers.md`）；Agent 模式经 ACP 启动 Claude Code / Codex / OpenCode，stdio ndjson（`src/agentMode/acp/AcpBackendProcess.ts`）；本地检索与文档解析交给独立应用 Miyo，`127.0.0.1:8742`，先读服务发现文件找端口（`docs/miyo-api.md`）；key 存 Obsidian Keychain | 聊天面板，可写回 | 协议：OpenAI chat、ACP；「独立本地服务 + 服务发现文件」的做法 | AGPL-3.0 | 7905 / 2026-10-28 / 4.0.16（2026-10-23）/ 2,987,551 |
| [Smart Connections](https://github.com/brianpetro/obsidian-smart-connections) | Obsidian | 进程内：默认本地嵌入 `TaylorAI/bge-micro-v2`，transformers.js 跑在 iframe 或 worker（依赖库 [brianpetro/jsbrains](https://github.com/brianpetro/jsbrains) 的 `smart-embed-model/adapters/`） | 索引存 `.smart-env/` | 否；自定义许可证第 2 条禁止作为与 Obsidian 类笔记软件互操作产品的组件 | 自定义（NOASSERTION） | 5524 / 2026-10-10 / 4.7.2（2026-08-06）/ 1,252,533 |
| [Text Generator](https://github.com/nhaouari/obsidian-textgenerator-plugin) | Obsidian | 云 API 或本地 HTTP：LangChain provider（`src/LLMProviders/langchain/*`）；key 可选 `safeStorage` 加密 | 按模板插入 | 协议：OpenAI chat / Ollama | MIT | 1994 / 2026-08-06 / 0.8.7（2026-04-27）/ 591,273 |
| [Local GPT](https://github.com/pfrankov/obsidian-local-gpt) | Obsidian | 本地 HTTP 为主：经 AI Providers 的 SDK 调 Ollama；RAG 嵌入用 `nomic-embed-text` / `bge-m3` | 替换选区或插入 | 协议 | MIT | 677 / 2026-10-26 / 4.3.13（2026-10-26）/ 105,507 |
| [AI Providers](https://github.com/pfrankov/obsidian-ai-providers) | Obsidian | 进程内共享 provider 层：注册全局对象 `app.aiProviders`，其他插件经 `@obsidian-ai-providers/sdk` 的 `initAI` 调 `execute()`、嵌入与检索；服务 API 带版本号（v3 / v4），能按模型探测能力（`checkModelCapabilities`）；嵌入缓存在 IndexedDB；provider 列表含 `apiKey` 经 `saveData()` 明文写入 `data.json`，SDK 把含 `apiKey` 的对象交给调用方（`packages/sdk/README.md`） | 不写笔记 | 模式值得借鉴，密钥处理是反例（见 [§2.5](#25-运行方式统计与可复用协议)） | MIT | 131 / 2026-10-11 / 1.13.0（2026-10-11）/ 65,877 |
| [YT Video Summarizer](https://github.com/mbramani/obsidian-yt-video-summarizer) | Obsidian | 云 API：字幕经 YouTube 非公开 innertube 接口获取（`src/services/youtube.ts`），再交 Gemini / OpenAI / Anthropic | 插入摘要 | 否 | MIT | 44 / 2026-04-28 / 1.2.1（2026-04-28）/ 9,599 |
| [logseq-summarize](https://github.com/avelino/logseq-summarize) | Logseq | 云 API：`api.tldr.chat/v0` | 子块 | 否 | MIT | 13 / 2025-02-15 / v0.1.8（2025-02-15） |

<a id="24-独立工具与服务"></a>
### 2.4 独立工具与服务

这些可以直接当歇存的 sidecar 后端，或当作协议参考。

| 工具 | 形态 | 接口（证据） | 引擎 | 输出 | 注意 | 许可证 | ★ / 推送 / release |
|---|---|---|---|---|---|---|---|
| [VideoCaptioner](https://github.com/WEIFENG2333/VideoCaptioner) | Python GUI + CLI | `videocaptioner transcribe video.mp4 --asr <engine>`；本地后端 `subprocess.Popen` 调 whisper.cpp、faster-whisper；`-q` 只输出结果路径；退出码 0–5 有文档（`docs/cli.md`） | whisper.cpp、faster-whisper、OpenAI 兼容、必剪、剪映 | srt（默认）、ass、txt、json | 必剪、剪映是非公开接口，剪映还依赖第三方签名服务（`core/asr/jianying.py`），歇存不应默认启用 | GPL-3.0 | 16395 / 2026-09-12 / v1.4.2（2026-05-24） |
| [Buzz](https://github.com/chidiwilliams/buzz) | Python/Qt 桌面 + CLI | `buzz add --model-type ... --srt --vtt --txt`（`docs/docs/cli.md`）；Windows 上先起 `whisper-server --inference-path /audio/transcriptions` 再用 OpenAI 客户端访问，就绪判断是固定 `sleep(10)`（`buzz/transcriber/local_whisper_cpp_server_transcriber.py`） | whisper、whisper.cpp、faster-whisper | TXT、SRT、VTT | 证明 whisper.cpp server 可以对外提供 OpenAI 形状接口 | MIT | 22243 / 2026-10-27 / v1.4.5（2026-08-23） |
| [whisper-asr-webservice](https://github.com/ahmetoner/whisper-asr-webservice) | FastAPI 服务（Docker） | `POST /asr`（`output=txt\|vtt\|srt\|tsv\|json`、`word_timestamps`、`vad_filter`、`diarize`）、`POST /detect-language`；默认端口 9000（`app/webservice.py`） | openai-whisper / faster-whisper / whisperX | 直接返回字幕文件 | 没有 OpenAI 兼容路由 | MIT | 3367 / 2026-10-01 / v1.10.0（2026-08-09） |
| [speaches](https://github.com/speaches-ai/speaches) | 服务（原 faster-whisper-server） | OpenAI 兼容 `POST /v1/audio/transcriptions`、`/v1/audio/translations`，支持 `timestamp_granularities[]` 与 SSE；模型按请求动态加载（`src/speaches/routers/stt.py`） | faster-whisper、parakeet 等 | OpenAI 响应格式 | 最新是预发布 | MIT | 3741 / 2026-10-26 / v0.9.0-rc.3（2025-12-27） |
| [WhisperX](https://github.com/m-bain/whisperX) | CLI / Python | `whisperx audio.wav --diarize --highlight_words True` | faster-whisper + wav2vec2 对齐 + pyannote | 字幕、JSON | 依赖 PyTorch，体积大；分离要 HF token | BSD-2-Clause | 24907 / 2026-10-22 / v3.8.6（2026-05-25） |
| [noScribe](https://github.com/kaixxx/noScribe) | Python/Tk + 无界面 CLI | `noScribe in.mp3 out.txt --model fast --speaker-detection 2`（`docker/README.md`） | faster-whisper + pyannote | html、txt、vtt（按扩展名） | Docker 只有 amd64 | GPL-3.0 | 2222 / 2026-10-27 / v0.8（2026-10-06） |
| [Umi-OCR](https://github.com/hiroi-sora/Umi-OCR) | Python/Qt 桌面 | HTTP `127.0.0.1:1224/api/ocr`（base64 图片，`data.format: dict\|text`）、`/api/doc`；另有 CLI（`docs/http/api_ocr.md`） | RapidOCR / PaddleOCR | 文本框 + 置信度 | 只支持 Windows 与 Linux，无 macOS；最近 release 2025-03 | MIT | 48245 / 2025-11-20 / v2.1.5（2025-03-25） |
| [SenseVoice-OneApi](https://github.com/LuckLittleBoy/SenseVoice-OneApi) | 服务（Docker） | OpenAI 兼容 `POST /v1/audio/transcriptions`（README 的 curl 示例） | funasr SenseVoice | OpenAI 风格 JSON；srt/vtt 待核实 | 仓库没有许可证 | 无 | 95 / 2026-08-12 / 无 release |
| [api4sensevoice](https://github.com/0x5446/api4sensevoice) | 服务 | `POST /transcribe` 与 WebSocket 流式（`server.py`）；默认监听 `0.0.0.0` | funasr SenseVoiceSmall | JSON | 协议私有；无 LICENSE 文件 | 无 | 542 / 2024-10-23 / 无 release |

<a id="25-运行方式统计与可复用协议"></a>
### 2.5 运行方式统计与可复用协议

**统计。** 每个插件按默认或主要推理路径计一次；Omnisearch（只消费 OCR 结果）、AI Providers（中间层）、Whisper-Plus（不是转写）与 [§2.4](#24-独立工具与服务) 的独立工具不计入，合计 29 个：

| 运行方式 | 数量 | 插件 |
|---|---|---|
| 云 API | 15 | Whisper、Scribe、Audio Notes、NeuroVox、AI Transcriber、Taskbone、AI Image OCR、Copilot、Text Generator、YT Video Summarizer、logseq-transcriber、logseq-summarize、voice2text、speech2text、siyuan-plugin-ocr-api |
| 宿主进程内（JS / wasm / WebGPU / N-API） | 6 | Voice Scribe、Text Extractor、TextLens、Smart Connections、logseq-ocr、siyuan-plugin-paddleocr |
| 外部 CLI 或 sidecar | 5 | Speech Kit、Offline Whisper、image-ocr、Power Extract、Lens OCR |
| 本地 HTTP | 3 | Transcription、Local GPT、logseq-whisper-subtitles |

插件本身全部依赖宿主 API，没有一个能直接拿给歇存用。能复用的是它们背后的协议与 CLI：

1. **转写的事实标准是 OpenAI `/v1/audio/transcriptions`（multipart）。** 用到或提供它的有 Whisper、Scribe、speech2text、AI Transcriber、NeuroVox（经 Groq），以及 speaches、SenseVoice-OneApi、Buzz 启动的 whisper.cpp server（路径不带 `/v1`）、VideoCaptioner 的 `whisper-api` 后端。歇存的转写插件对外提供这一个端点（支持 `response_format=json|text|srt|vtt|verbose_json` 与 `timestamp_granularities[]`），现有客户端都能直连；歇存调外部服务时也按这个形状写客户端。
2. **第二个值得兼容的是 whisper-asr-webservice 的 `POST /asr`。** 它能直接返回 `srt`/`vtt`，并带 `word_timestamps`、`vad_filter`、`diarize`，正好对应 [SPEC §5](../SPEC.md#5-derived-文件约定) 的 `asr.vtt`。
3. **LLM 一侧全部是 OpenAI chat 形状**，本地端点也是：Ollama `localhost:11434/v1`、LM Studio `localhost:1234/v1`。视觉模型做 OCR 也是在 chat completions 里传 base64 图片。歇存的摘要插件与视觉 OCR 插件只需兼容 OpenAI chat，Ollama 原生 `/api/*` 作为可选。
4. **OCR 没有共同协议。** 独立本地协议只看到 Umi-OCR 的 `/api/ocr`，它没有 macOS 版。Obsidian 里事实上的契约是 `extractText(file)`（Text Extractor 定义，Power Extract 照搬，Omnisearch 依赖），它定在宿主进程的 JS 对象上。歇存应把同类契约（`extract(path) → text + boxes`）定在进程边界，输出沿用 [SPEC §5](../SPEC.md#5-derived-文件约定) 的 `[{text, bbox, confidence}]`。
5. **反例集中在「宿主进程里做重计算」和「运行时拉代码」**：渲染进程推理（Voice Scribe 约 720 MB 模型、思源 PaddleOCR、Text Extractor、Smart Connections）；从 CDN 拉可执行代码（NeuroVox 的 ffmpeg-core，它自己的注释承认同类做法因违反 Obsidian 审核规则已下线）；往插件目录下载原生扩展（TextLens）；在用户机器上现场编译 helper（Lens OCR）；从非官方仓库下载二进制（Offline Whisper）；本地服务默认监听 `0.0.0.0`（api4sensevoice）。做得对的对照：TextLens 安装 helper 前校验 SHA-256，Speech Kit 把 sidecar 单独发版。
6. **密钥与 provider 配置。** 已查实把 key 移出明文文件的只有 Whisper（`app.secretStorage`）、AI Transcriber（Electron `safeStorage`）、Copilot（Keychain），Text Generator 可选加密；Scribe、NeuroVox、AI Image OCR、AI Providers 等在抓到的源码里没看到加密存储。AI Providers「一处配置、多插件共享、服务 API 带版本、按模型探测能力」的方向是对的，但它把含 `apiKey` 的完整对象交给每个调用方，等于任何插件都能读走所有密钥。歇存的做法应该是：provider 配置由内核持有，密钥放系统钥匙串、不写进会同步的资料库；插件只拿到 `provider` 名称与能力声明，请求由内核或代理进程代签代发。思源 voice2text 经内核出站代理 `/api/network/forwardProxy` 联网，是「宿主代为联网」的现成样子。
7. **可直接当 sidecar 调的现成工具**：CLI 有 VideoCaptioner、Buzz、noScribe、WhisperX、`tesseract`、`whisper-cli`；服务有 speaches、whisper-asr-webservice、Umi-OCR、SenseVoice-OneApi。许可证要分开看：VideoCaptioner、noScribe、Text Extractor、Lens OCR 是 GPL-3.0，Copilot、思源 PaddleOCR 是 AGPL-3.0，SenseVoice-OneApi 与 api4sensevoice 没有许可证文件，代码不能直接拿来用；GPL 工具作为独立进程被调用、并随歇存分发时的合规边界待核实（需要法律意见）。
8. **结果落盘的三种做法**：Audio Notes 把 JSON/SRT 作为附件，Text Extractor 与 Power Extract 按内容哈希缓存 JSON 并随库同步，思源由内核统一保存（`setImageOCRText`）。歇存的 `derived/` 已经是第一种，转换插件只写 `derived/` 下的约定文件即可。

<a id="3-插件注册表与分发"></a>
## 3. 插件注册表与分发

每个注册表回答同一组问题：插件怎么声明、怎么提交、谁审核、自动化是 GitHub Actions 还是 GitHub App、怎么更新、怎么下架、安全模型。机器人账号类型都用 `gh api users/<login>` 实测：GitHub App 的机器人 `type` 是 `Bot`、login 以 `[bot]` 结尾；由 Actions 或外部服务拿 PAT 驱动的机器用户 `type` 是 `User`。Mihon store、Krew、gh extensions 见 [02](02-adapter-ecosystems.md#14-对比表)，这里只在对比表里各列一行。

<a id="31-obsidian"></a>
### 3.1 Obsidian

仓库 [obsidianmd/obsidian-releases](https://github.com/obsidianmd/obsidian-releases)：22712 ★，推送 2026-10-28，无许可证。

- **声明**：插件仓库根目录的 `manifest.json`。通用必填 `author`、`minAppVersion`、`name`、`version`（`x.y.z`），可选 `authorUrl`、`fundingUrl`；插件另需 `description`、`id`、`isDesktopOnly`；`id` 只能是小写字母和连字符，不能以 `plugin` 结尾、不能含 `obsidian`（[Manifest.md](https://github.com/obsidianmd/obsidian-developer-docs/blob/main/en/Reference/Manifest.md)）。`versions.json` 把插件版本映射到 `minAppVersion`，供旧版 App 找兼容版本（[Versions.md](https://github.com/obsidianmd/obsidian-developer-docs/blob/main/en/Reference/Versions.md)）。release 的 tag 必须等于 `version`，附件是独立上传的 `main.js`、`manifest.json`、`styles.css`（可选）。索引 `community-plugins.json` 每条只有 `id`、`name`、`author`、`description`、`repo` 五个键。
- **提交方式在 2026-05 改了**：
  - 旧流程是向 `community-plugins.json` 提 PR、填 checklist，由 `validate-plugin-entry.yml`（`pull_request_target` 触发，`actions/github-script` 校验后用 `GITHUB_TOKEN` 评论、打 `Ready for review` / `Validation failed` 等标签）做初检。
  - 2026-05-12 kepano 的博文 [The future of plugins](https://obsidian.md/blog/future-of-plugins/) 宣布上线 Obsidian Community 目录与开发者面板；2026-05-15 的提交 [d4f06944](https://github.com/obsidianmd/obsidian-releases/commit/d4f06944)「remove PR templates & validation actions」删掉了 PR 模板和 6 个 workflow；仓库现在 PR 与 issue 都关闭。
  - 现行流程：在 community.obsidian.md 用 Obsidian 账号登录、连接 GitHub 核验仓库归属、填仓库 URL、同意开发者政策；目录读取默认分支 HEAD 的 `manifest.json`（[Submit your plugin](https://docs.obsidian.md/Plugins/Releasing/Submit+your+plugin)、[Set up and claim](https://github.com/obsidianmd/obsidian-developer-docs/blob/main/en/Community%20directory/Set%20up%20and%20claim.md)）。
  - GitHub 仓库只剩两个 workflow：`mirror-community-json.yml` 每小时从 `community.obsidian.md/assets/community-plugins.json` 拉取，校验形状、id 唯一、条目数不低于现有 95% 后，以 git 用户名「Obsidian Bot」提交；`plugin-stat.yml` 每天镜像下载统计。仓库已经是目录网站的只读镜像。
- **审核**：
  - 博文：每个版本都做自动的安全与代码质量扫描，不只首次提交；人工审核继续，但集中在热门、精选和被社区标记的插件；新提交通过自动审核才进搜索，新版本审核失败 24 小时内移出搜索；上线前几天处理了 2,300 多个排队的提交；暂不接受闭源插件。
  - 自动审核分 Manifest、Releases、Source code、Build verification 四组，扫描器找 `build` 等构建命令，核对构建产物与源码是否一致；本地可用 [obsidianmd/eslint-plugin](https://github.com/obsidianmd/eslint-plugin) 预检（[FAQ](https://github.com/obsidianmd/obsidian-developer-docs/blob/main/en/Community%20directory/Frequently%20asked%20questions.md)）。
  - 2026-08-27 起 Obsidian 员工给审核失败的老插件开 issue：2026-10-30 之后最新 release 仍不通过的将不再列出（例：[tldraw/obsidian-plugin#229](https://github.com/tldraw/obsidian-plugin/issues/229)）。
- **「This plugin has not been manually reviewed by Obsidian staff.」**：2026-10-28 的索引里 9605 条有 7019 条在 `description` 末尾带这句。官方没有解释（2026-06-08 有用户在[论坛](https://forum.obsidian.md/t/remove-this-plugin-has-not-been-manually-reviewed-from-plugin-browser-preview/115097)请求去掉，截至查阅无员工回复）。按提交历史统计：加入镜像前（652de396，2026-05-12T15:29Z）2750 条、0 条带后缀；第一次镜像（b80a2193，2026-05-12T15:30Z）3401 条、674 条带后缀；不带后缀的 2586 条 id 都在迁移前的人工审核清单里。推断：后缀标记的是迁移后只过了自动审核、没经人工审核就上架的插件（另有 4 个迁移前的老插件也带了后缀，规则并不干净；官方含义待核实）。半年里条目数从 2750 增到 9605。
- **机器人**：`ObsidianReviewBot` 是 `User`（2023-12-28 创建），是机器用户而不是 App；驱动方式待核实。2026-07-08 建的 GitHub App [`obsidian-community-directory`](https://github.com/apps/obsidian-community-directory) 只有 `contents:read`、`metadata:read`、订阅 `repository` 事件，用途是让作者把它装到私有源码仓库，目录据此核对公开产物与私有源码是否一致。
- **更新**：发版本号递增的新 release，目录定期检查，也可手动「Check for new releases」。
- **下架**：作者在目录里 Archive；违反政策时先给整改期，恶意或屡犯可立即移除（[Developer policies](https://github.com/obsidianmd/obsidian-developer-docs/blob/main/en/Community%20directory/Developer%20policies.md)）。仓库里的 `community-plugins-removed.json` 有 175 条（`id`、`name`、`reason`，最后更新 2026-04-20，迁移后是否维护待核实）；`community-plugin-deprecation.json` 按版本号弃用 7 个插件。
- **安全模型**：没有运行时沙箱和强制权限，详见 [§4.1](#41-宿主插件的安全模型)；每个版本扫描、构建一致性核对；2026-07-29 起示例 release workflow 加了 `actions/attest` 构建溯源证明；索引里没有校验和字段。

<a id="32-logseq-marketplace"></a>
### 3.2 Logseq marketplace

仓库 [logseq/marketplace](https://github.com/logseq/marketplace)：363 ★，推送 2026-10-28，MIT。

- **声明**：`packages/<name>/manifest.json`，字段 `title`、`description`、`author`、`repo`（`user/repo`），可选 `icon`、`theme`、`sponsors`、`web`、`effect`、`supportsDB`、`supportsDBOnly`。插件仓库要有 `package.json`、能构建 release zip 的 CI、README、LICENSE（[README](https://github.com/logseq/marketplace/blob/master/README.md)、`.github/pull_request_template.md`）。
- **提交与审核**：fork 后在 `packages/` 加目录提 PR（当天 651 个目录），人工合并；PR 上没有校验 CI。近期合并者 `xyhp915` 不是 logseq 组织的公开成员，身份待核实。
- **自动化**：唯一的 workflow `update-catalog-index.yml` 只能手动触发，生成 `catalog/generated/` 与 GitHub Pages 目录页。
- **更新**：插件仓库发新 release，客户端从插件仓库下载 zip（客户端实际行为待核实）。
- **下架**：`blocked.json` 支持通配与 `!` 排除，当前屏蔽 `sethyuan/*` 但放行其中一个仓库；恶意插件邮件举报。
- **安全**：插件默认跑在 iframe 里（详见 [§4.1](#41-宿主插件的安全模型)）；`effect: true` 表示与宿主同源，README 说这类插件审核更严；没有签名或校验和。

<a id="33-思源-bazaar"></a>
### 3.3 思源 bazaar

仓库 [siyuan-note/bazaar](https://github.com/siyuan-note/bazaar)：268 ★，推送 2026-10-28，许可证 NOASSERTION。

- **声明**：包根目录 `plugin.json`，必填 `name`、`author`、`url`、`version`（不带 `v` 的 semver，更新必须更高），可选 `displayName`、`description`、`readme`（对象）、`icon`、`funding`、`keywords`、`minAppVersion`，插件另有 `backends`、`frontends`、`kernels` 等（[rules/manifest.go](https://github.com/siyuan-note/bazaar/blob/main/rules/manifest.go)）。Latest Release 必须挂 `package.zip`，包根要有 `README.md`、清单与 `index.js`（[rules/files.go](https://github.com/siyuan-note/bazaar/blob/main/rules/files.go)）。
- **提交**：在 `plugins.txt` 等 5 个清单之一追加一行 `owner/repo`（`plugins.txt` 当天 608 行），一个 PR 只做一件事；PR 模板要求确认许可证允许再分发，闭源包要把私有源码仓库授权给三位维护者审核。
- **自动化全部是 Actions**，评论者是 `github-actions[bot]`：
  - [`pr-check.yml`](https://github.com/siyuan-note/bazaar/blob/main/.github/workflows/pr-check.yml) 用 `pull_request_target` 触发，权限 `contents: read`、`issues: write`、`pull-requests: write`；注释写明只把清单文件当数据读、不执行 PR 代码；另有每小时三次的 schedule 复检还没通过的 PR，通过后按 `vars.BAZAAR_REVIEWERS` 请求审查；拉 release 用 `secrets.PAT`。
  - `stage.yml` 每小时运行，Go 程序 `actions/stage` 校验 `package.zip` 后上传七牛对象存储，key 是 `package/<owner/repo>@<SHA-256 前 7 位>`，生成 `stage/*.json`；校验失败给对应包开 `stage-fail` issue，恢复后自动关闭。
- **审核**：PR Check 自动校验 + 维护者人工审阅（AGENTS.md）。
- **更新**：不需要 PR，作者发新 Latest Release，1–3 小时内自动入库，只过自动校验、没有人工复审。
- **下架**：维护者开「Delist owner/repo」issue 后直接提交删行；`deprecated.json` 表示「仍可安装、不再推荐」，可带本地化 `reason` 与 `alternatives`。
- **安全**：内容寻址缓存（SHA-256 前缀写进 URL）；运行时情况见 [§4.1](#41-宿主插件的安全模型)。

<a id="34-raycast"></a>
### 3.4 Raycast

仓库 [raycast/extensions](https://github.com/raycast/extensions)：7820 ★，推送 2026-10-28，MIT。

- **声明**：monorepo，`extensions/<name>/package.json` 里的 Raycast 字段：`name`、`title`、`description`、`icon`、`author`、`platforms`、`categories` 必填，可选 `commands`、`tools`、`preferences`、`contributors` 等（[manifest.md](https://github.com/raycast/extensions/blob/main/docs/information/manifest.md)）。
- **提交**：`npm run publish` 登录 GitHub 后自动向 monorepo 开 PR，也可手工 fork（`docs/basics/publish-an-extension.md`）。
- **审核**：Raycast 员工与社区代码审查；2026-10 的机器人评论说新扩展初审可能要 15 个工作日。
- **自动化**：`pr-bot.yml` 用 `pull_request_target` 加 `secrets.RAYCAST_BOT_API_ACCESS_TOKEN` 运行 bot 脚本，以 `raycastbot`（`User`）评论并 @ 扩展的 owner 与 contributors；`extensions_build_publish.yml` 在 PR 上跑 `ray build`，合并到 main 后用 `RAYCAST_CLI_ACCESS_TOKEN` 跑 `ray publish` 发布到 Store；`generate-code-owners.yml` 按各扩展的作者与贡献者生成 `CODEOWNERS`。
- **更新**：任何人可以 Fork Extension 后提 PR 并把自己加进 `contributors`，合并即发布（`docs/basics/contribute-to-an-extension.md`）。
- **下架**：员工提 PR 删目录（例 [#25851](https://github.com/raycast/extensions/pull/25851)）；Store 端机制待核实。
- **安全**（[security.md](https://github.com/raycast/extensions/blob/main/docs/information/security.md)）：扩展全部开源、合并前人工审查、Store 分发 Raycast CI 构建的产物；运行时见 [§4.1](#41-宿主插件的安全模型)；自动静态安全分析写的是「计划中」。

<a id="35-zed"></a>
### 3.5 Zed

仓库 [zed-industries/extensions](https://github.com/zed-industries/extensions)：1944 ★，推送 2026-10-27，无许可证。

- **声明**：中心仓库 `extensions.toml`（当天 1583 条）每条有 `submodule`、`version`（必须等于该提交里 `extension.toml` 的版本）、可选 `path`；扩展本体以 HTTPS git submodule 挂在 `extensions/<id>`；扩展清单 `extension.toml` 有 `id`、`name`、`version`、`schema_version`、`authors`、`description`、`repository`（[publishing guide](https://zed.dev/docs/extensions/publishing/publishing-guide)）。
- **提交**：一个 PR 只加或更新一个扩展，同时最多 3 个打开的 PR；[AI_POLICY.md](https://github.com/zed-industries/extensions/blob/main/AI_POLICY.md) 禁止用自主 agent 提交。
- **审核**：员工逐个审，首次反馈通常几周，也可能 1–2 个月（[FAQ](https://zed.dev/docs/extensions/publishing/faq)）。
- **自动化**：
  - `ci.yml` 在 PR 上构建测试，合并后用 `S3_*` secrets 发布扩展包。
  - `danger.yml` 用 `pull_request` 触发，GitHub 请求走 `https://danger-proxy.zed.dev/github` 代理、workflow 里的 `GITHUB_TOKEN` 是假值，评论者是 `zed-industries-bot`（`User`）：这是「外部服务替 fork PR 持有写权限」的做法。
  - `remove-extension.yml` 用 `actions/create-github-app-token` 现场换 App token，以 `zed-zippy[bot]`（`Bot`，2025-09-24 创建）开移除 PR；这个 App 在仓库里开了 271 个 PR，多数是版本更新，触发机制待核实。
- **更新**：PR 更新 submodule 与 `version`；文档推荐社区 Action `huacnlee/zed-extension-action` 自动开 PR。
- **下架**：作者请求，员工跑 `remove-extension.yml`；他人接手需原作者书面许可或 6 周联系不上的证据。
- **安全**：扩展代码编译成 Wasm，用户侧 `granted_extension_capabilities` 控制能力（详见 [§4.1](#41-宿主插件的安全模型)）；有许可证白名单。

<a id="36-homebrew-tap"></a>
### 3.6 Homebrew tap

[Homebrew/brew](https://github.com/Homebrew/brew)：50137 ★，推送 2026-10-28，BSD-2-Clause。

- **第三方 tap** 就是一个 `user/homebrew-<name>` 仓库，`brew tap user/repo` 即可用，文档里没有 Homebrew 审核第三方 tap 的流程；`brew tap-new` 生成带 Actions 的模板，bottle 默认传到 GitHub Releases（[How to Create and Maintain a Tap](https://docs.brew.sh/How-to-Create-and-Maintain-a-Tap)）。
- **Tap Trust**：Homebrew 6.0.0（2026-06-11）起，非官方 tap 要显式 `brew trust` 才会加载其 Ruby 代码（[Tap-Trust](https://docs.brew.sh/Tap-Trust)、[6.0.0 公告](https://brew.sh/2026/06/11/homebrew-6.0.0/)）。
- **BrewTestBot** 在文档里指 `brew test-bot` 这个 CI 编排命令，由 Homebrew 的 Actions 运行（[BrewTestBot](https://docs.brew.sh/BrewTestBot)）；同名 GitHub 账号是 `User`（2012 年创建）。homebrew-core 的 `autobump.yml` 每 3 小时跑 `brew bump --open-pr`，用 SSH 签名 key 与 PAT。
- **更新**：`brew bump-formula-pr` 为 formula 开 PR 并更新 URL 与 SHA-256。
- **安全**：formula 带 sha256；macOS 上构建与测试阶段有沙箱，6.0.0 起 Linux 用 Bubblewrap；第三方 tap 靠 Tap Trust。

<a id="37-open-vsx"></a>
### 3.7 Open VSX

[eclipse-openvsx/openvsx](https://github.com/eclipse-openvsx/openvsx)：2063 ★，EPL-2.0；[EclipseFdn/publish-extensions](https://github.com/EclipseFdn/publish-extensions)：347 ★，EPL-2.0。

- **声明**：VS Code 扩展的 `package.json`，`publisher` 即命名空间，打包成 `.vsix`。
- **发布**：注册 Eclipse 账号并关联 GitHub、签 Publisher Agreement、生成 token、`npx ovsx create-namespace`、`npx ovsx publish`（[Publishing-Extensions](https://github.com/EclipseFdn/open-vsx.org/wiki/Publishing-Extensions)）。首个创建者只是 contributor，扩展页显示「未验证发布者」；要成为 owner 需开 issue 认领，GitHub 账号须一年以上，若同名于 VS Code Marketplace 发布者还要证明控制权（[Managing-Namespaces](https://github.com/EclipseFdn/open-vsx.org/wiki/Managing-Namespaces)）。
- **代发仓库**：publish-extensions 的 `extensions.json`（404 条）由工作日 cron 代为发布；`extension-control/extensions.json` 含 `malicious`（952 条）与 `deprecated`（139 条）列表，供客户端与注册表读取。
- **安全事件与措施**：
  - 2025-10：泄露的发布 token 被与 GlassWorm 恶意扩展关联，受影响扩展下架、token 吊销，2025-10-21 宣布处置完毕；之后 token 加前缀 `ovsxat_`、缩短默认有效期、发布时自动扫描恶意模式与内嵌密钥（[Eclipse 博客，2025-10-27](https://blogs.eclipse.org/post/mika%C3%ABl-barbero/open-vsx-security-update-october-2025)）。
  - 2026-02：宣布发布前检查（冒名、误发密钥、已知恶意模式，可疑上传先隔离），先监控后强制（[The Hacker News，2026-02-04](https://thehackernews.com/2026/02/eclipse-foundation-mandates-pre-publish.html)，二手，Eclipse 原帖待核实）。
  - 2026-03：扫描器失败时被当作通过的缺陷「Open Sesame」，几天内修复（[The Hacker News](https://thehackernews.com/2026/03/open-vsx-bug-let-malicious-vs-code.html)，二手）。
- **自动化**：服务端扫描 + publish-extensions 里的 Actions，没看到 GitHub App。

<a id="38-其他先例"></a>
### 3.8 其他先例

- **HACS**（[hacs/default](https://github.com/hacs/default)：650 ★，MIT；[hacs/action](https://github.com/hacs/action)：38 ★，MIT）：每个集成一个公开仓库，`hacs.json` 至少有 `name`，要有完整 GitHub release、仓库描述、topics、OSI 许可证，提交前先让 `hacs/action` 无错误通过（[Include 文档](https://hacs.xyz/docs/publish/include)）；在 `hacs/default` 对应类型文件里按字母序加一行提 PR，审核通常要几个月。`checks.yml` 用 `pull_request` 触发、顶层 `permissions: {}`；评论者 `hacs-bot[bot]`（`Bot`，2019-10-21 创建），仓库 workflow 里没有生成它 token 的步骤，驱动方待核实。下架清单 `removed` 有 540 条（`repository`、`removal_type`、`reason`、`link`），由 workflow 推到 Cloudflare R2。
- **Flathub**（[flathub/flathub](https://github.com/flathub/flathub)：1546 ★）：基于 `new-pr` 分支提 PR，志愿者审查，评论 `bot, build` 触发测试构建；`flathubbot`（`User`）回帖构建结果，构建由外部编排服务 [flathub-infra/vorarbeiter](https://github.com/flathub-infra/vorarbeiter) 完成。通过后在 flathub 组织下为应用新建独立仓库，提交者获得写权限，之后的更新都在应用自己的仓库里进行（[submission 文档](https://docs.flathub.org/docs/for-app-authors/submission)）。
- **krew-release-bot**（补 [02 §12.4](02-adapter-ecosystems.md#124-kubectl-krew) 的待核实，[rajatjindal/krew-release-bot](https://github.com/rajatjindal/krew-release-bot)：55 ★，Apache-2.0）：是「插件仓库里的 Docker Action + 外部 webhook 服务 + 机器用户」，不是 App 驱动。插件发 release 后 Action 渲染 `.krew.yaml` 并 POST 到 `krew-release-bot.rajatjindal.com`（Netlify Functions）；服务端以 `krew-release-bot`（`User`）推到 fork 再向 krew-index 开 PR；维护者 `/lgtm /approve` 后由 `kubernetes-prow[bot]` 合并。krew-index 当天最近 40 个 PR 有 33 个是它开的；插件第一个版本仍须作者手工提交。服务端代码里没看到对请求来源的认证（待核实）。同名 GitHub App `krew-release-bot`（2019-11-30 创建，订阅 `release`）是更早的触发方式。
- **DefinitelyTyped mergebot**（[microsoft/DefinitelyTyped-tools](https://github.com/microsoft/DefinitelyTyped-tools) 的 `packages/mergebot`）：Azure Function 接收 webhook，另有每 6 小时补扫的 Action；2026-06-08 合并的 #1295 把身份从 `typescript-bot`（`User`，PAT）换成 `typescript-automation[bot]`（`Bot`，2026-06-02 创建），私钥放 Azure Key Vault 由它签 JWT。owner 回复「Ready to merge」后机器人合并。
- **conda-forge**（[conda-forge/staged-recipes](https://github.com/conda-forge/staged-recipes)、[conda-forge-webservices](https://github.com/conda-forge/conda-forge-webservices)）：linter 用 `pull_request` 触发、全部权限只读（注释说明可能经 `eval` 运行不可信代码），再由 `issue_comment` 类 workflow 负责打标签；webservices 是托管在 Heroku 的 web app，为 `conda-forge-webservices[bot]`（`Bot`）生成 App token，另用无特权机器用户做 fork。PR 合并后为每个包建 feedstock 仓库并给提交者写权限（[文档](https://conda-forge.org/docs/maintainer/adding_pkgs/)），建仓 job 在哪里执行待核实。
- **nixpkgs**（[NixOS/nixpkgs](https://github.com/NixOS/nixpkgs)）：所有 PR workflow 用 `pull_request_target`、默认 `permissions: {}`，README 强调绝不执行 PR 里的代码；在 Actions 里用 `actions/create-github-app-token` 换多个 App 的 token，评论者是 `nixpkgs-ci[bot]`（`Bot`，2024-12-02 创建）（[workflows README](https://github.com/NixOS/nixpkgs/blob/master/.github/workflows/README.md)）。

<a id="39-github-app-与-actions-的能力差别"></a>
### 3.9 GitHub App 与 Actions 的能力差别

**GitHub App 能做什么**（[Deciding when to build a GitHub App](https://docs.github.com/en/apps/creating-github-apps/about-creating-github-apps/deciding-when-to-build-a-github-app)）：

- 由用户或组织安装到选定的仓库，权限细粒度（例如只读 contents），自带集中式 webhook，速率限制随安装的仓库数增长；可以独立行事，也可以代表用户行事。
- installation access token 1 小时过期，换 token 时还能用 `repositories` 与 `permissions` 进一步收窄（[generating an installation access token](https://docs.github.com/en/apps/creating-github-apps/authenticating-with-a-github-app/generating-an-installation-access-token-for-a-github-app)）。
- App 改了权限后，每个安装方的账号所有者要批准，批准前安装仍用旧权限（[Approving updated permissions](https://docs.github.com/en/apps/using-github-apps/approving-updated-permissions-for-a-github-app)）。
- Checks API 的写操作只有 GitHub App 能做，OAuth App 与已认证用户只能读（[REST: check runs](https://docs.github.com/en/rest/checks/runs)）。这不排除 Actions：`GITHUB_TOKEN` 本身就是一个 GitHub App 的 installation token（[GITHUB_TOKEN](https://docs.github.com/en/actions/concepts/security/github_token)），nixpkgs PR 上的 check run 都来自 `github-actions` App。被排除的是 PAT 驱动的机器用户（raycastbot、BrewTestBot、krew-release-bot）。
- App 需要持久运行的计算资源；Actions 不持久运行，只响应所在仓库的事件（同上 Deciding 页）。

**Actions 能做什么、不能做什么**（[Events that trigger workflows](https://docs.github.com/en/actions/reference/workflows-and-actions/events-that-trigger-workflows)）：

- 来自 fork 的 `pull_request`：文档原话 "The `GITHUB_TOKEN` has read-only permissions in pull requests from forked repositories."；除 `GITHUB_TOKEN` 外 secrets 不传给 runner。
- `pull_request_target` 在基仓库上下文运行，可以给 fork PR 评论、打标签；文档同时警告在其中运行不可信代码会导致安全漏洞。GitHub Security Lab 推荐把不可信代码放在 `pull_request` 里跑、结果作为 artifact，再由有写权限的 `workflow_run` 处理（[preventing pwn requests](https://securitylab.github.com/resources/github-actions-preventing-pwn-requests/)）。[actions/checkout](https://github.com/actions/checkout) v7 在 `pull_request_target` 与 `workflow_run` 下默认拒绝签出 fork 代码，需显式 `allow-unsafe-pr-checkout: true`。
- `GITHUB_TOKEN` 只限所在仓库；由它触发的事件不会再创建新的 workflow run（`workflow_dispatch`、`repository_dispatch` 除外）。

所以，**纯 Actions 的校验器完全可以给外部提交评论、打标签**（思源 `pr-check.yml`、Raycast `pr-bot.yml`、Obsidian 旧的 `validate-plugin-entry.yml`、nixpkgs 都这么做），前提只有一条：`pull_request_target` 里只把 PR 内容当数据读，绝不执行。真正需要 App 或外部服务的只有四件事：

1. 不需要作者配合就收到插件仓库的事件（例如新 release）。索引仓库的 workflow 只能响应本仓库事件，所以思源、Obsidian 用定时轮询，krew 要求作者在插件仓库加一个 Action 去 POST 外部服务。
2. 跨仓库写入，并以 `[bot]` 身份出现。不托管服务的中间路线是在 Actions 里用 [actions/create-github-app-token](https://github.com/actions/create-github-app-token)（905 ★，MIT）现场换 1 小时的 App token（Zed、nixpkgs）。
3. 持久运行与跨事件的状态（排队、重试、跨 PR 调度）。Actions 只能用 schedule 近似：思源每 20 分钟复检，DefinitelyTyped 每 6 小时补扫。
4. 读取作者的私有仓库（Obsidian 目录 App 的用途）。

**机器人类型实测**：

| login | type | 用途 |
|---|---|---|
| `ObsidianReviewBot` | User | Obsidian 旧提交流程 |
| `raycastbot` | User | Raycast PR 评论（Actions + PAT） |
| `BrewTestBot` | User | Homebrew 提交与 bottle（Actions + PAT + SSH 签名） |
| `flathubbot` | User | Flathub 构建回帖（外部编排服务） |
| `krew-release-bot` | User | krew-index 版本更新 PR（webhook 服务） |
| `zed-industries-bot` | User | Zed PR 评论（Danger 代理） |
| `typescript-bot` | User | DefinitelyTyped 2026-06 之前 |
| `conda-forge-admin` | User | conda-forge 命令 |
| `github-actions[bot]` | Bot | Actions 内置（思源 bazaar 等） |
| `hacs-bot[bot]` | Bot | HACS 排队与状态评论 |
| `zed-zippy[bot]` | Bot | Zed 移除与更新 PR |
| `nixpkgs-ci[bot]` | Bot | nixpkgs 打标签、评论 |
| `typescript-automation[bot]` | Bot | DefinitelyTyped 现行 |
| `conda-forge-webservices[bot]` | Bot | conda-forge lint |

趋势：老机器人都是 PAT 驱动的 `User`；2024–2026 年新加的都是 App（`nixpkgs-ci[bot]` 2024-12、`zed-zippy[bot]` 2025-09、`typescript-automation[bot]` 2026-06、Obsidian 目录 App 2026-07）。构建 GitHub App 的框架 [probot/probot](https://github.com/probot/probot)（9628 ★，ISC，v14.3.2 于 2026-04-03）仍在维护，但 2026-04 之后只有依赖更新。

<a id="310-注册表对比表"></a>
### 3.10 注册表对比表

| 注册表 | 声明文件 | 提交方式 | 审核者 | 自动化（机器人 type） | 更新方式 | 下架方式 | 安全模型 |
|---|---|---|---|---|---|---|---|
| Obsidian（[§3.1](#31-obsidian)） | `manifest.json` + `versions.json`；索引 5 个字段 | 2026-05 起在目录网站表单提交；仓库 PR 已关 | 每版本自动扫描；人工只看热门、精选、被举报 | 目录网站（外部服务）；仓库 Actions 只做镜像；只读 App 读私有源码 | 发 release，目录轮询 | 作者 Archive；员工按政策移除；按版本弃用 JSON | 无沙箱无强制权限；逐版本扫描 + 构建一致性；推荐 attestation；无校验和 |
| Logseq（[§3.2](#32-logseq-marketplace)） | `packages/<n>/manifest.json` | fork + PR | 人工 | 只有手动触发的目录生成 | 插件仓库发 release | `blocked.json` | iframe；`effect` 同源审核更严；无校验和 |
| 思源（[§3.3](#33-思源-bazaar)） | 包内 `plugin.json` + `package.zip` | PR 在 TXT 里加 `owner/repo` | PR Check + 人工 | Actions：`pull_request_target` 只读数据 + 定时复检 + 每小时 stage（`github-actions[bot]` Bot） | 发 release 自动入库，无人工复审 | 删行；`deprecated.json` | 内容寻址缓存；运行时无隔离 |
| Raycast（[§3.4](#34-raycast)） | monorepo `package.json` | CLI 自动开 PR | 员工 + 社区 | Actions + PAT（`raycastbot` User）；合并后 CI 发布 | 任何人提 PR，合并即发布 | 员工删目录 | 开源 + 人工审查 + CI 构建产物；isolate 无文件网络沙箱 |
| Zed（[§3.5](#35-zed)） | `extensions.toml` + `extension.toml` | PR 加 submodule | 员工逐个审 | Actions；Danger 代理（User）；Actions 内换 App token（`zed-zippy[bot]` Bot） | PR 更新 submodule 与 version | 员工跑移除 workflow | Wasm + 用户侧能力授权；许可证白名单 |
| Homebrew tap（[§3.6](#36-homebrew-tap)） | formula | 第三方 tap 不提交 | 第三方不审 | Actions + PAT（`BrewTestBot` User） | `bump-formula-pr` / autobump | tap 作者处理 | sha256；构建沙箱；6.0.0 起 Tap Trust |
| Open VSX（[§3.7](#37-open-vsx)） | `package.json` + `.vsix` | `ovsx publish` | 自动检查，可疑隔离待审 | 服务端扫描；代发仓库 Actions | 再次 publish | `malicious` / `deprecated` 列表 | 命名空间认领；token 前缀与短有效期；发布前扫描 |
| HACS（[§3.8](#38-其他先例)） | `hacs.json` + release | PR 加一行 | 人工，排队数月 | Actions 校验；`hacs-bot[bot]` Bot | 插件仓库发 release | `removed` 列表 | 提交时校验 + 下架清单 |
| Flathub（[§3.8](#38-其他先例)） | Flatpak manifest | PR 到 `new-pr` | 志愿者 | 外部编排 + `flathubbot` User | 在应用独立仓库提 PR | 待核实 | 在 Flathub 基础设施构建 |
| krew（[§3.8](#38-其他先例)、[02 §12.4](02-adapter-ecosystems.md#124-kubectl-krew)） | 索引 `plugins/<name>.yaml` | 首版手工 PR | 维护者（Prow） | Action → webhook 服务 → `krew-release-bot` User | 每次 release 自动开 PR | 待核实 | sha256；人工审 PR |
| Mihon / Keiyoushi（[02 §1](02-adapter-ecosystems.md#1-tachiyomi-与-mihon-扩展生态)） | Gradle DSL → 签名 APK | 社区 store 自行收录 | store 维护者 | store 自己的 CI | 只从来源 store 更新 | store 删条目，客户端标 orphaned | APK 签名 + store 指纹 |
| gh extensions（[02 §12.5](02-adapter-ecosystems.md#125-gh-extensions)） | 仓库名 `gh-*` | 不提交，topic 发现 | 无 | 无 | `gh extension upgrade` | 无 | 无，建议自行审源码 |

<a id="4-安全与隔离"></a>
## 4. 安全与隔离

<a id="41-宿主插件的安全模型"></a>
### 4.1 宿主插件的安全模型

| 宿主 | 插件运行位置 | 权限声明 | 用户授权 | 审核与签名 | 已知事件 | 来源 |
|---|---|---|---|---|---|---|
| Obsidian | 同进程（Electron 渲染进程）；帮助页说插件继承 Obsidian 的访问级别，可读写文件、联网、安装程序 | 没有强制声明；博文说「未来几个月」上线网络、文件系统、剪贴板等能力的**披露**，安装前展示给用户，不做拦截；Scorecard 已静态列出用到的 API | Restricted mode 默认开启，关掉后社区插件整体放行，没有逐插件授权 | 每版本自动扫描（政策、代码质量、已知漏洞、恶意软件）；没有代码签名，Scorecard 会查 artifact attestation | Elastic REF6598（2026-04-14）：攻击者诱导受害者登录攻击者控制的同步 vault 并手动打开「社区插件同步」，被篡改的 Shell Commands 插件配置在打开 vault 时执行 PowerShell 或 osascript，Hider 插件隐藏界面；Windows 上投递 PHANTOMPULSE 远控，macOS 用 LaunchAgent 持久化；报告的结论是攻击者滥用正常功能，没有利用软件漏洞。论坛 [#7544](https://forum.obsidian.md/t/security-of-the-plugins/7544) 里 staff 2020-10-25 回复说 Electron 下没法限制插件的网络等能力 | [Plugin security](https://obsidian.md/help/plugin-security)；[future of plugins](https://obsidian.md/blog/future-of-plugins/)；[Elastic](https://www.elastic.co/security-labs/phantom-in-the-vault) |
| Logseq | iframe：Postmate 动态创建，只设 `allow`（Permissions Policy），没设 `sandbox` 属性；插件页面从独立 origin `lsp://` 加载，经 postMessage + MessageChannel 调宿主 API；宿主窗口 `nodeIntegration false`、`contextIsolation true`。插件不能直接用 Node，但能经宿主 API `logseq.Git.execCommand`、`App.invokeExternalCommand` 执行外部命令；`effect: true` 的插件与宿主同源 | 没有权限字段 | 没有逐插件授权（安装界面是否提示待核实） | marketplace 人工合并；无签名（待核实） | 没找到 | logseq/logseq `libs/src/LSPlugin.caller.ts`、`libs/src/postmate/index.ts`、`src/electron/electron/window.cljs`（v2.0.2） |
| 思源 | 前端插件在渲染进程里用 `window.eval` 执行并拿到 `window.require`，窗口 `nodeIntegration: true`、`contextIsolation: false`，等于完整 Node 能力；内核插件（PR [#17487](https://github.com/siyuan-note/siyuan/pull/17487)，2026-05-09 合入）由 goja 在 Go 内核进程里运行，`siyuan` 全局对象冻结，外部请求走拦截私网与回环地址的拨号器，30 秒超时 | `plugin.json` 的 `kernels` 只声明平台，没有权限字段 | 集市里启用即生效 | PR Check + 人工；无签名（待核实） | 没找到 | siyuan `app/electron/main.js`、`app/src/plugin/loader.ts`、`kernel/plugin/sandbox.go`（v3.8.8） |
| Raycast | 一个 Node 子进程，每个扩展一个 V8 isolate，堆大小受限；文件、网络不再加沙箱 | 没有逐扩展权限 | macOS 的系统授权给 Raycast 整体 | 开源 + 人工审查 + CI 构建 | 没找到 | [Raycast security](https://developers.raycast.com/information/security) |
| Zed | Wasm（wasmtime + WIT 接口） | 用户设置 `granted_extension_capabilities`，三种 kind：`process:exec`（`command`、`args`）、`download_file`（`host`、`path`）、`npm:install`（`package`）；`assets/settings/default.json` 里三项默认都是 `*` / `**`，即全放行 | 用户收紧设置；能力不够时扩展 API 返回错误，文档提醒设成 `[]` 会让很多扩展不可用。`process:exec` 能启动本机程序（例如语言服务器），执行就离开了 Wasm 沙箱 | 员工逐个审；签名待核实 | 没找到 | [Zed capabilities](https://zed.dev/docs/extensions/capabilities)；zed `assets/settings/default.json` |
| VS Code | extension host 独立进程，与 VS Code 本身同权限，运行时无沙箱 | 没有权限声明 | 1.97 起首次安装某第三方发布者的扩展要确认信任；Workspace Trust 只限制不受信工作区里的部分行为 | Marketplace 多引擎扫描、沙箱动态分析、扩展签名并在安装时校验 | GlassWorm（Koi Security 2025-10-20 披露）：恶意代码用不可见 Unicode 字符藏起来，经自动更新推给 Open VSX 与 Marketplace 用户，根因是泄露的发布 token（二手报道） | [extension runtime security](https://code.visualstudio.com/docs/configure/extensions/extension-runtime-security)；[BleepingComputer](https://www.bleepingcomputer.com/news/security/self-spreading-glassworm-malware-hits-openvsx-vs-code-registries/) |
| Figma | 主线程沙箱：2019 年起用编译成 Wasm 的 QuickJS，主线程没有 `fetch` 与 DOM；UI 在 iframe，两边消息通信 | manifest `networkAccess.allowedDomains`（`["none"]` 即禁网）、`reasoning`、`permissions` | 按 manifest 生效 | 社区发布审核（待核实） | 2019 年旧 Realms shim 有逃逸漏洞，官方称未发现被利用 | [how plugins run](https://developers.figma.com/docs/plugins/how-plugins-run/)、[manifest](https://developers.figma.com/docs/plugins/manifest/)、[2019 博文](https://www.figma.com/blog/an-update-on-plugin-security/) |
| Chrome MV3 | 扩展进程 | `permissions`、`host_permissions`，另有 optional 版本 | 安装时提示；optional 的运行时授予 | — | — | [declare permissions](https://developer.chrome.com/docs/extensions/develop/concepts/declare-permissions) |
| Deno | 运行时进程 | `--allow-read` / `--allow-write` / `--allow-net` / `--allow-run` 等，可限定路径或主机 | 命令行标志，默认无任何 I/O；文档要求把 `--allow-run`、`--allow-ffi` 视同全部放行 | — | — | [Deno security](https://docs.deno.com/runtime/fundamentals/security/) |
| Navidrome（Go 宿主） | Wasm：`extism/go-sdk` v1.7.1 + `wazero` v1.12.0 | manifest `permissions`：`http.requiredHosts`（要附 `reason`）、`storage`、`library.filesystem` 等；Extism 自带 HTTP 被关掉，只能走宿主的 `http_send`（默认 10 秒超时、每次重定向重查白名单、响应最多 10 MB、按解析出的 IP 拦私网） | 管理员启用；写权限要单独开 | 无集中审核（待核实） | 没找到 | navidrome [`plugins/README.md`](https://github.com/navidrome/navidrome/blob/master/plugins/README.md) |
| sqlc（Go 宿主） | wasm 插件（wazero v1.12.0）与 process 插件并存 | wasm 插件无网络、文件、环境变量，配置必须带 `url` + `sha256`；文档说 process 插件安全性很低；环境变量默认不继承 | 配置文件 | 无 | — | [sqlc plugins](https://docs.sqlc.dev/en/latest/guides/plugins.html) |

两点观察：

- 主流笔记宿主都不做运行时强制，靠的是审核、扫描和告知。Obsidian 目录 9605 条有 7019 条带「未经人工审核」后缀（[§3.1](#31-obsidian)），说明铺开之后审核覆盖不了大多数插件。
- 能做到强制的先例分两类：Wasm + 能力由宿主中转（Figma、Zed、Navidrome、sqlc），或者 OS 沙箱包住原生进程（见 [§4.2](#42-独立进程的隔离手段)）。Zed 默认全放行再加上 `process:exec`，说明一旦给了「执行本机程序」的能力，Wasm 沙箱就不再起作用。REF6598 说明，随库同步的插件配置文件本身就能成为执行入口。

<a id="42-独立进程的隔离手段"></a>
### 4.2 独立进程的隔离手段

| 手段 | 平台 | 能限制什么 | Go 里怎么用 | 先例 | 成熟度与风险 | 来源 |
|---|---|---|---|---|---|---|
| `sandbox-exec` + SBPL（Seatbelt） | macOS | 按路径读写、网络（全禁或只放 localhost 代理端口）、`process-exec`、IPC；不管内存和 CPU | `exec.CommandContext("/usr/bin/sandbox-exec", "-p", profile, ...)`，profile 由宿主按声明生成，用绝对路径 | OpenAI Codex（`codex-rs/sandboxing/src/seatbelt.rs`）、[anthropics/sandbox-runtime](https://github.com/anthropics/sandbox-runtime)、Chromium、[fence](https://github.com/fencesandbox/fence)、itch.io [smaug](https://github.com/itchio/smaug) | man page 标 DEPRECATED（本机 `man sandbox-exec`，Darwin 24.6.0），SBPL 无公开文档；放行 GPU 要放宽规则（具体规则待核实） | [openai/codex](https://github.com/openai/codex)（rust-v0.165.0，2026-10-27）；[Chromium mac sandbox](https://chromium.googlesource.com/chromium/src/+/HEAD/sandbox/mac/README.md) |
| bubblewrap | Linux | 文件视图（只读或读写 bind、tmpfs）、`--unshare-net`、PID/IPC 命名空间、`--die-with-parent`，可传 seccomp 过滤器；不管内存和 CPU | exec 外部 `bwrap`（系统包或随应用附带） | Flatpak、Codex（Linux 默认文件沙箱）、sandbox-runtime、fence、smaug | [containers/bubblewrap](https://github.com/containers/bubblewrap) v0.13.1（2026-10-14），9262 ★；README 说它不是现成的完整沙箱策略；需要非特权 user namespace，Ubuntu 23.10/24.04 的 AppArmor 限制要额外 profile（[Ubuntu 说明](https://discourse.ubuntu.com/t/understanding-apparmor-user-namespace-restriction/58007)） | codex `codex-rs/linux-sandbox/README.md` |
| Landlock | Linux 5.13+ | 按目录树的文件访问；ABI v4 起 TCP `bind`/`connect` 只能按端口 | [landlock-lsm/go-landlock](https://github.com/landlock-lsm/go-landlock)（纯 Go，有 `!cgo` 分支）：小 launcher 先 `RestrictPaths` 再 `syscall.Exec` 插件 | Codex 早期用它，现在列为 legacy：README 说它管不住 app-server 的 Unix socket | v0.11.0（2026-10-24），369 ★，MIT；各内核支持不一，用 `BestEffort()` 降级 | go-landlock README |
| seccomp-bpf | Linux | 系统调用（禁 socket 族、`io_uring` 等） | `bwrap --seccomp FD`，或 launcher 里安装（纯 Go 库现状待核实） | Codex、sandbox-runtime | 只认系统调用，不认路径；每个架构单独维护 | codex linux-sandbox README |
| cgroups v2 | Linux | 内存 `memory.max`、CPU、进程数 | `SysProcAttr.UseCgroupFD` + `CgroupFD`；桌面会话要有可写的委派子树（待核实） | — | 取决于发行版与 systemd | Go `src/syscall/exec_linux.go` |
| 进程生命周期 | 全平台 | 超时、取消、孤儿进程 | `exec.CommandContext` + `Cmd.Cancel` + `Cmd.WaitDelay`；Linux `Pdeathsig`、`Setpgid`；Windows Job Object | smaug、MXC | 成熟 | Go `src/os/exec/exec.go` |
| Job Objects | Windows | 内存、CPU 速率、进程数；`KILL_ON_JOB_CLOSE` 让父进程退出时杀整棵树；不管文件与网络 | `golang.org/x/sys/windows` 已有 `CreateJobObject`、`SetInformationJobObject`、`AssignProcessToJobObject` | smaug、MXC（挂起创建，先放进 job 再恢复） | 成熟 | golang/sys `windows/syscall_windows.go` |
| AppContainer | Windows 8+ | 文件（只能访问授权给容器 SID 的 ACL）、网络（整体开关） | x/sys 没有 `CreateAppContainerProfile` 封装，要自己调 userenv.dll；成熟 Go 库没找到 | Chromium 部分进程 | 给输入输出目录加 ACL 等于改用户文件权限；做不到按域名 | [Implementing an AppContainer](https://learn.microsoft.com/en-us/windows/win32/secauthz/implementing-an-appcontainer) |
| 专用低权用户 + WFP + ACL | Windows | 文件（ACL）、网络（按用户 SID 拦出站，只放代理端口） | 需要管理员一次性装好，Go 侧 exec 安装器 | sandbox-runtime（`srt-sandbox` 用户 + WFP）、Codex elevated 模式、smaug | 要提权；sandbox-runtime README 列了 WFP 管不住的情况 | sandbox-runtime README（v0.0.81，2026-10-20）；[Codex Windows sandbox](https://learn.chatgpt.com/docs/windows/windows-sandbox.md) |
| MXC（Microsoft eXecution Container） | Windows 11 / Linux / macOS | 文件读写拒绝列表、出站策略与代理、剪贴板等；后端分别是 `processcontainer`、bubblewrap、Seatbelt | 只有 Rust / .NET / Node SDK；Go 只能 exec 执行器，README 说执行器用于测试或 SDK 无法嵌入时 | Codex `codex-rs/mxc-sandbox` | [microsoft/mxc](https://github.com/microsoft/mxc) v1.0.1（2026-10-17），3032 ★，MIT；`processcontainer` 要求 Win11 24H2/25H2 的 2026-08 更新之后 | mxc README、`docs/backends/process-container/os-version-support.md` |
| 网络代理白名单 | 全平台 | 按域名放行 HTTP CONNECT 与 SOCKS5，连接前拦回环、私网、云元数据地址 | 宿主内写 CONNECT 代理（`net/http`）或用 fence 的 Go 包；OS 层必须同时封死直连，否则只是约定 | sandbox-runtime、Codex network proxy、fence；Navidrome 是 Wasm 宿主函数版 | 不认 `HTTPS_PROXY` 的程序会直接失败 | sandbox-runtime、fence README |
| 剥离环境变量 + 临时目录 | 全平台 | 少暴露凭据与路径（不是强制隔离） | `cmd.Env` 写成白名单，`os.MkdirTemp` 建工作目录并把 `HOME`、`TMPDIR` 指过去 | sqlc、Codex | 各平台都能做，算最低保证 | sqlc plugins 文档 |

Go 生态里现成的封装：[fencesandbox/fence](https://github.com/fencesandbox/fence)（Go CLI 与包，macOS Seatbelt + Linux bubblewrap + 域名代理；1005 ★，v0.1.67 于 2026-09-01，Apache-2.0）；[itchio/smaug](https://github.com/itchio/smaug)（itch.io 桌面应用给第三方游戏二进制套沙箱，Linux firejail/bubblewrap、macOS sandbox-exec、Windows 隔离用户 + Job Object；24 ★，MIT，无 release）；[can1357/isobox](https://github.com/can1357/isobox)（一套能力模型编译成 Seatbelt / gVisor / AppContainer；28 ★，v0.1.0，早期）。[anthropics/sandbox-runtime](https://github.com/anthropics/sandbox-runtime)（6818 ★，Apache-2.0）是 TypeScript 包，README 标 Beta Research Preview，Go 只能当外部 CLI 调。

<a id="43-wasm-运行时"></a>
### 4.3 Wasm 运行时

| 运行时 | 语言与 cgo | WASI 与 component model | SIMD / threads / GPU | 资源限制 | 插件语言 | 活跃度 |
|---|---|---|---|---|---|---|
| [wazero](https://github.com/wazero/wazero)（原 `tetratelabs/wazero`，import path 不变） | 纯 Go；README 写明零依赖、不依赖 cgo | WASI preview1；component model / WASI 0.2 明确不做（[#2200](https://github.com/tetratelabs/wazero/issues/2200) 2024-05-02 以 not planned 关闭，维护者说要等 W3C 标准化） | SIMD 默认开；threads 在 experimental；没有 wasi-nn 或 WebGPU 导入，所以用不了 GPU | `WithMemoryLimitPages`；`WithCloseOnContextDone(true)` 让 ctx 取消或超时即终止；默认无文件与网络 | Rust `wasm32-wasip1`、TinyGo、Go `GOOS=wasip1`、C/C++（wasi-sdk） | 6440 ★，v1.12.0（2026-05-29），推送 2026-10-19，Apache-2.0；编译器后端只支持 amd64/arm64；签名后的 macOS 应用用编译器模式要加 `allow-jit` 与 `allow-unsigned-executable-memory` entitlement（[#2393](https://github.com/tetratelabs/wazero/issues/2393)） |
| [Extism go-sdk](https://github.com/extism/go-sdk) | 纯 Go，基于 wazero（go.mod 固定 v1.9.0） | 可选 WASI p1；无 component model | 同 wazero | manifest `allowed_hosts`、`allowed_paths`、`memory.max_pages`、`memory.max_http_response_bytes`、`timeout_ms`；HTTP 由宿主实现 | PDK：Rust、JS、Python、Go（TinyGo）、Haskell、AssemblyScript、.NET、C、C++、Zig | go-sdk 185 ★，最后提交 2025-05-14，v1.7.1（2025-03-19），停了约 17 个月；主仓 [extism/extism](https://github.com/extism/extism)（Rust 运行时）5818 ★，v1.41.0（2026-10-09），BSD-3-Clause |
| [knqyf263/go-plugin](https://github.com/knqyf263/go-plugin) | 纯 Go，wazero + protobuf 代码生成 | WASI p1 | 同 wazero | 同 wazero | Go / TinyGo | 733 ★，v0.9.0（2025-03-12），MIT |
| Wasmtime / [wasmtime-go](https://github.com/bytecodealliance/wasmtime-go) | wasmtime-go README 说它经 cgo 调 C API，与歇存「不用 cgo」的 [ADR-001](../decisions/ADR-001-stack.md) 冲突 | 完整 component model 与 WASI 0.2；WASI 0.3.0 于 2026-06-11 发布，Wasmtime 46.0.0（2026-06-22）起默认支持（[wasi.dev](https://wasi.dev/releases/wasi-p3)） | 待核实 | fuel / epoch 中断（待核实） | Rust、C/C++、JS 等 | [wasmtime](https://github.com/bytecodealliance/wasmtime) 18792 ★，v50.0.0（2026-10-20） |
| [wasi-nn](https://github.com/WebAssembly/wasi-nn)（提案） | — | Phase 2 | — | — | — | 561 ★，最后推送 2024-10-28 |

**ffmpeg、ASR 放进 Wasm 的可行性证据**（文档原文）：

- **ffmpeg.wasm**（[ffmpegwasm/ffmpeg.wasm](https://github.com/ffmpegwasm/ffmpeg.wasm)，17914 ★，MIT）：Emscripten 构建、面向浏览器，0.12.0 起不支持 Node；多线程版需要 SharedArrayBuffer。[性能页](https://ffmpegwasm.netlify.app/docs/performance)测同一转码：原生 5.2 秒，单线程 core 128.8 秒（0.04 倍），多线程 core-mt 60.4 秒（0.08 倍）。[FAQ](https://ffmpegwasm.netlify.app/docs/faq) 说输入最大 2 GB 是 WebAssembly 的硬限制。它不是 wasip1 产物，wazero 不能直接加载；即使重编，wazero 的 threads 还是实验功能。
- **whisper.wasm**（[whisper.cpp `examples/whisper.wasm`](https://github.com/ggml-org/whisper.cpp/tree/master/examples/whisper.wasm)）：Emscripten 构建、要求 WASM SIMD；tiny/base 模型约 2–3 倍实时，最大只支持到 small，音频最长 120 秒。原生 whisper.cpp 支持 Metal、Core ML、CUDA、Vulkan 等后端。
- **onnxruntime-web**：`wasm` 后端支持全部算子，`webgpu`、`webgl`、`webnn` 只支持一部分且依赖浏览器 API，wazero 里没有这些（[onnxruntime web 教程](https://onnxruntime.ai/docs/tutorials/web/)）。
- **Shopify Functions** 是「Wasm + 硬执行限制」的生产先例：编译产物不超过 256 kB、线性内存 10,000 kB、200 行以内购物车最多 1100 万条指令（[Shopify Functions](https://shopify.dev/docs/api/functions/latest)）。

**Go 宿主跑 Wasm 插件的先例**（都对过 go.mod）：sqlc（wazero v1.12.0）、Navidrome（Extism go-sdk + wazero，插件包是 zip 里的 `manifest.json` + `plugin.wasm`）、Trivy modules（wazero，标 EXPERIMENTAL，[文档](https://trivy.dev/latest/docs/advanced/modules/)写明模块未经安全审计）、Redpanda Connect（wazero 的 WASM processor）。

<a id="44-原生进程还是-wasm"></a>
### 4.4 原生进程还是 Wasm

| 转换 | 判断 | 理由 | 建议的权限档 |
|---|---|---|---|
| ffmpeg 抽音轨与关键帧 | 原生进程 | ffmpeg.wasm 只有原生 0.04–0.08 倍速、输入上限 2 GB；长视频常超过 2 GB；硬件解码只有原生能用 | 读单个输入文件；写宿主分配的临时目录；无网络；不许再起子进程 |
| ASR 推理（Metal / CUDA / Core ML） | 原生进程 | wazero 用不了 GPU；whisper.wasm 最大到 small、最长 120 秒；模型几百 MB 到几 GB | 读音频与模型目录；写输出目录；无网络（模型由宿主下载）；要 GPU |
| OCR 推理（ONNX Runtime / 系统 Vision / Windows OCR） | 原生进程 | 系统 OCR 只能原生调用；ONNX Runtime 原生版才有 CoreML、DirectML、CUDA 加速 | 同 ASR，GPU 可选 |
| 画面描述（本地 VLM） | 原生进程 | 吃 GPU | 同 ASR |
| AI 摘要、标签建议、走 HTTP 的 VLM | Wasm 更合适，原生进程 + 代理也行 | 本质是 HTTP 调用；Wasm 的请求由宿主函数代发，三个平台都能一致强制域名白名单，并由宿主在代发时注入密钥 | 网络 allowlist（回环地址也要显式列出）；无文件；内存 64–256 MB |
| 文本规范化（HTML→Markdown、VTT/SRT 互转、清洗） | Wasm | 纯计算、输入小，默认零能力，内存与超时都管得住 | 无文件无网络；stdin/stdout 或宿主函数传数据 |
| 导出格式映射（条目 → Obsidian / Logseq / 思源 / Notion 的 Markdown 或 JSON） | Wasm | 纯数据变换；插件只返回字节，由宿主写到用户选的目录，宿主始终是唯一写文件的一方 | 同上 |
| 云端导出（Notion、飞书、语雀 API） | 原生进程或 Wasm，网络必须走宿主 | 需要网络与密钥；密钥不应交给插件 | 网络 allowlist 限定到目标域名；密钥由宿主注入 |

结论：ffmpeg、ASR、OCR、本地 VLM 只能是原生进程；纯变换与 HTTP 型的插件适合 Wasm。真正要定的是「一套机制分档」还是「两套机制」——建议内核只认一种协议（独立进程 + JSON），Wasm 插件由官方的 `wasm-runner` 进程承载（内嵌 wazero，纯 Go）。这样 wazero 不进内核，符合 [ADR-001](../decisions/ADR-001-stack.md)；`allow-jit` entitlement 只加在 runner 上；对内核来说 runner 只是又一个进程插件。

<a id="45-权限声明与各平台落实程度"></a>
### 4.5 权限声明与各平台落实程度

插件清单里加一段权限声明，原生插件与 Wasm 插件用同一个结构（字段名是提议）：

```yaml
permissions:
  fs:
    read:  ["${input}", "${models}/whisper"]   # 宿主展开成具体路径；插件拿不到资料库根目录
    write: ["${output}"]                       # 宿主分配的临时目录，结束后宿主校验再移入 derived/
  network:
    mode: none            # none | allowlist
    hosts: []             # 例如 ["api.openai.com:443", "127.0.0.1:11434"]
  gpu: false
  exec: false             # 是否允许再起子进程
  env: []                 # 允许透传的环境变量
  limits:
    memory_mb: 4096
    timeout_s: 1800
```

「强制」指 OS 或运行时会拦截；「告知」指只在安装或运行前展示，由用户判断：

| 声明项 | macOS | Linux | Windows | Wasm runner |
|---|---|---|---|---|
| `fs.read` / `fs.write` | 强制：Seatbelt profile | 强制：bwrap bind，降级 Landlock，都不行则告知 | 默认告知；有 MXC 时强制 | 强制：只挂载声明的目录 |
| `network: none` | 强制：Seatbelt 禁出站 | 强制：`--unshare-net` | 默认告知；MXC 或 WFP 时强制 | 强制：没有 socket |
| `network: allowlist` | 强制：只放 localhost 代理端口，代理按域名过滤 | 强制：独立 netns + 代理 | 告知（代理环境变量只约束守规矩的程序） | 强制：宿主函数检查 |
| `gpu` | 告知 | 部分：bind GPU 设备（待核实） | 告知 | 不支持 |
| `exec` | 强制：Seatbelt `process-exec` | 部分：bwrap + seccomp | 部分：Job 进程数上限 | 不适用 |
| `limits.memory_mb` | 告知 + 宿主轮询 RSS 超限即杀 | 强制：cgroup v2 或 prlimit | 强制：Job Object | 强制：`max_pages` |
| `limits.timeout_s` | 强制：ctx 到期杀进程组 | 强制 | 强制：Job kill-on-close | 强制：`WithCloseOnContextDone` |
| `env` | 强制：显式 `cmd.Env` | 强制 | 强制 | 强制：默认不传 |

界面上按平台显示「强制 / 部分 / 仅告知」三档，不要一律写「已沙箱」。风险与成本：sandbox-exec 已标 deprecated，profile 生成器要单独成模块并在每个 macOS 大版本回归；bubblewrap 在 Ubuntu 的 userns 限制下会直接失败，要有降级与清楚的报错；Windows 上 AppContainer 无 Go 封装、专用用户要管理员权限、MXC 很新且只支持 Win11 24H2 以后，现实的默认是只强制 Job Object 与环境变量，文件和网络先告知；Extism go-sdk 停滞，用它就要准备 fork。

<a id="5-导出与互通格式"></a>
## 5. 导出与互通格式

<a id="51-obsidian"></a>
### 5.1 Obsidian

截至查阅日，最新公开桌面版是 1.14.5（2026-10-22）（[changelog](https://obsidian.md/changelog)）。帮助页已从 `help.obsidian.md` 301 跳转到 `obsidian.md/help/`。

- **存储**：vault 是本地文件夹，笔记是 Markdown，设置在 `.obsidian/`（[data storage](https://obsidian.md/help/data-storage)）。
- **Properties**（[properties](https://obsidian.md/help/properties)）：7 种类型 Text、List、Number、Checkbox、Date（`2020-08-21`）、Date & time（`2020-08-21T10:30:00`）、Tags；同一属性名在整个库只能有一种类型；默认属性 `tags`、`aliases`、`cssclasses`，旧写法 `tag`/`alias`/`cssclass` 自 1.9.0 起不再支持（[1.9.0](https://obsidian.md/changelog/2025-05-21-desktop-v1.9.0)）；属性里写内部链接要加引号 `"[[Link]]"`；嵌套对象列在 Not supported 下，属性面板编辑不了。类型存在 `.obsidian/types.json`（官方无文档，只有社区实例，待核实）。
- **改属性会重写 YAML**：用属性面板或 `processFrontMatter` 改属性时会删掉 YAML 注释、改写引号与列表格式，2025-08-13 的回复说没有改的计划（[论坛](https://forum.obsidian.md/t/yaml-bug-editing-in-live-preview-or-reading-mode-reformats-and-deletes-comments/104006)）。所以机器标记不能放在 YAML 注释里，也不能指望 frontmatter 保持原写法。
- **附件**：「Default location for new attachments」有四个选项：vault 根目录、指定文件夹、当前文件所在文件夹、当前文件夹下的子文件夹（[attachments](https://obsidian.md/help/attachments)）。可识别的音视频格式含 mp3、m4a、wav、ogg、mp4、mov、mkv、webm（[file formats](https://obsidian.md/help/file-formats)）。
- **链接**（[links](https://obsidian.md/help/links)）：「Use [[Wikilinks]]」默认开；「New link format」有最短路径、相对路径、库内绝对路径三种；Markdown 链接要 URL 编码；块引用 `[[Note#^id]]`，块 ID 只能是拉丁字母、数字、短横线，普通段落在行尾加 ` ^id`，列表、表格等结构块的 `^id` 单独占一行。块引用是 Obsidian 私有语法。
- **嵌入与时间片段**（[embeds](https://obsidian.md/help/embeds)）：`![[file]]`、`![[Note#^id]]`、图片 `![[x.png|640x480]]`、PDF `#page=N`。官方帮助没有音视频 `#t=`；2026-01-05 论坛还有人请求 `![[video.mp4#t=97]]`（[帖子](https://forum.obsidian.md/t/start-local-video-files-at-a-specific-timestamp/109617)），可推断原生不支持，社区插件 Media Extended 提供。
- **注释** `%%…%%` 在阅读视图不显示，1.13 起只认恰好两个 `%`（[Obsidian Flavored Markdown](https://obsidian.md/help/obsidian-flavored-markdown)、[1.13.4](https://obsidian.md/changelog/2026-07-30-desktop-v1.13.4/)），可以用来标受管区块。
- **Bases**（`.base`）：1.9.0（2025-05-21）引入，1.9.10（2025-08-18）进公开版（[1.9.10](https://obsidian.md/changelog/2025-08-18-desktop-v1.9.10/)）；视图有 Table、Cards（1.9）、List、Map（1.10）、Kanban（1.14）（[views](https://obsidian.md/help/bases/views)）。文件是 YAML，顶层键 `filters`（字符串或 `and`/`or`/`not` 嵌套）、`formulas`、`properties`、`summaries`、`views[]`；属性分 `note.*`（即 frontmatter）、`file.*`、`formula.*`；可用 `file.inFolder("X")` 收窄范围；嵌入写 `![[File.base#View]]`（[syntax](https://obsidian.md/help/bases/syntax)）。适合给一批带 frontmatter 的条目笔记配一个列表视图。
- **写入途径**：直接写文件（不需要 Obsidian 运行）；Obsidian URI `obsidian://new?...`，1.13 起执行前弹确认框，不适合批量（[URI](https://obsidian.md/help/uri)）；官方 CLI 1.12.0（2026-02-10）加入、1.12.4（2026-02-27）进公开版，需在设置里开启且 App 在运行，有 `create`、`append`、`property:set`、`read`、`properties format=json`、`base:query format=json` 等命令（[CLI](https://obsidian.md/help/cli)、[1.12.0](https://obsidian.md/changelog/2026-02-10-desktop-v1.12.0)）。
- **外部修改**：官方说会自动刷新以跟上外部改动（[data storage](https://obsidian.md/help/data-storage)）；外部改动碰上编辑器里未保存的内容时自动合并，自 0.11.6 起（[0.11.6](https://obsidian.md/changelog/2021-03-09-desktop-v0.11.6/)），论坛有合并后内容重复或丢失的报告；1.13.1 有一个 `Vault.modify` 与 `Vault.process` 重叠时外部改动会被覆盖的 bug，开发者说下一版修，具体版本待核实（[帖子](https://forum.obsidian.md/t/overlapping-vault-modify-vault-process-calls-permanently-stick-file-saving/116346)）。
- **JSON Canvas**：Obsidian 原生打开 `.canvas`；歇存画布的扩展字段处理见 [04 §6.2](04-tech-options.md#62-canvasjson-与-json-canvas)。

<a id="52-logseq"></a>
### 5.2 Logseq

- **2026 年拆成两个产品**：2026-04-24 的公告（[logseq.io/p/e3YDyX5AYr](https://logseq.io/p/e3YDyX5AYr)）把基于文件的版本改名 Logseq OG，代码迁到 [logseq/og](https://github.com/logseq/og)（2025-12-25 建，默认分支 `version/file`，描述「Logseq og (file version)」，只有 1.0.0 一个 release），只做安全更新与 Electron 升级、不加新功能；DB 版留在 [logseq/logseq](https://github.com/logseq/logseq)，2.0.1（2026-07-13）与 2.0.2（2026-10-07）都标「Beta Testing」，截至查阅日没有 stable。公告说不强制迁移，DB 版的 Markdown 支持还在研究，提到的想法里包括双向同步。
- **File graph（OG）格式**：`pages/`、`journals/`、`assets/`、`logseq/config.edn`；标题里的 `/` 在文件名写成 `___`；每个块是一个 `- ` 列表项；属性 `key:: value`，页面属性写在第一个块，值不能跨行，属性名转小写；`tags`、`alias` 按逗号拆成页面引用；YAML `---` 写法被标为不再推荐；块 ID `id:: <uuid>`，块引用 `((uuid))`；资源写 `![name](../assets/x.png)`（[Properties](https://github.com/logseq/docs/blob/master/pages/Properties.md)、[Block Reference](https://github.com/logseq/docs/blob/master/pages/Block%20Reference.md)、[Assets alias](https://github.com/logseq/docs/blob/master/pages/Assets%20alias.md)）。
- **外部修改（读 OG 源码）**：主进程用 chokidar 监听 graph 目录；内容和 DB 不一致时先把旧内容备份到 `logseq/bak/` 再重新解析；文件里的块被别处引用却没写 `id::` 时，Logseq 会把 `id::` 写回文件，也就是会改动歇存生成的文件；Logseq 写盘前发现磁盘内容变了会弹 diff 让用户选（`src/electron/electron/fs_watcher.cljs`、`src/main/frontend/fs/node.cljs`）。
- **本地 HTTP API**：桌面版开发者模式里开启，`POST http://127.0.0.1:12315/api`，`Authorization: Bearer <token>`，App 须在运行（[Local Http server](https://github.com/logseq/docs/blob/master/pages/Local%20Http%20server.md)）。
- **DB 版**：graph 存 `~/logseq/graphs/<graph>/db.sqlite`（[db-version.md](https://github.com/logseq/docs/blob/master/db-version.md)）；界面导入「File to DB graph」每次新建一个 graph；CLI 有 `upsert page`、`upsert block --blocks-file <md>`、`upsert asset`，只能用已存在的属性，且文档说每次 Markdown 插入都会新建一棵树、不幂等（[logseq-cli.md](https://github.com/logseq/logseq/blob/master/docs/cli/logseq-cli.md)）；MCP Server `127.0.0.1:12315/mcp`。Markdown Mirror（ADR 0016）把 DB 输出成 `mirror/markdown/`，文档明说它不是双向同步（[ADR 0016](https://github.com/logseq/logseq/blob/master/docs/adr/0016-markdown-mirror.md)）。

<a id="53-思源笔记"></a>
### 5.3 思源笔记

最新正式版 v3.8.8（2026-10-20）。API 文档已移到 [`docs/API.zh-CN.md`](https://github.com/siyuan-note/siyuan/blob/master/docs/API.zh-CN.md)，根目录的 `API_zh_CN.md` 现在 404。

- **数据格式**：文档是 `data/<notebookID>/<docID>.sy`，内容是序列化成 JSON 的 Lute AST，根节点 `{ID, Spec, Type: "NodeDocument", Properties, Children}`；每个块的 `Properties` 是扁平的字符串映射（IAL），自定义属性用 `custom-` 前缀；资源在 `data/assets/<原名>-<NodeID>.<ext>`；`temp/` 下是可重建的索引（[SY-FORMAT.zh-CN.md](https://github.com/siyuan-note/siyuan/blob/master/docs/SY-FORMAT.zh-CN.md)、[WORKSPACE.zh-CN.md](https://github.com/siyuan-note/siyuan/blob/master/docs/WORKSPACE.zh-CN.md)）。官方允许外部直接写 `.sy`，但只建议离线批量迁移时用，之后要重建索引；没找到内核监听 `.sy` 变化的代码。
- **写入接口**（`http://127.0.0.1:6806`，POST + JSON，`Authorization: Token <token>`）：
  - `/api/filetree/createDocWithMd`：参数 `notebook`、`path`、`markdown`，返回文档 ID；读 `kernel/model/file.go` 看到它没开 YAML Front Matter 解析，frontmatter 不会变属性，要另调 `/api/attr/setBlockAttrs`（属性名必须 `custom-` 前缀，值只能是字符串）。
  - `/api/asset/upload`：multipart，返回 `succMap` 把原文件名映射到 `assets/foo-<id>.png`，正文里的图片链接要按它改写。
  - `/api/import/importStdMd`：文档称内部 API；读 `kernel/model/import.go`，YAML 的 `title`、`date`、`lastmod`、`tags` 分别映射为标题、ID 时间、`updated`、标签，其他键变 `custom-<key>`，相对路径的资源会被复制进工作区，相对 `.md` 链接转成块引用。
  - 另有块级 `insertBlock`、`appendBlock`、`updateBlock`（`dataType: markdown`），v3.7.0 起有内置 CLI，v3.8.0 起有 MCP（`app/changelogs`）。
- **读回**：`/api/export/exportMdContent` 返回 `{hPath, content}`，frontmatter 只含 `title`、`date`、`lastmod`、`tags`；`/api/attr/getBlockAttrs` 返回全部属性含 `custom-*` 与 `updated`；`/api/block/getBlockKramdown` 返回带块 ID 的 kramdown；`/api/query/sql` 可查块表。歇存可以在文档块上写 `custom-xiecun-id` 当身份标记，用 `updated` 判断改过没有，具体改了什么由歇存自己 diff。

<a id="54-notion"></a>
### 5.4 Notion

- **数据模型**：云端块树；API 版本 `2025-09-03` 起数据库拆成 database（容器）与 data source（表），多数操作改用 `data_source_id`（[升级指南](https://developers.notion.com/guides/get-started/upgrade-guide-2025-09-03)）；最新版本 `2026-03-11`（[升级指南](https://developers.notion.com/guides/get-started/upgrade-guide-2026-03-11)）。元数据只能放数据库属性；父级是普通页面时只能有 `title`（[create page](https://developers.notion.com/reference/post-page)）。
- **认证**：internal connection、OAuth，以及 2026-05-12 上线的 Personal access token，官方建议本地脚本与 CLI 用后者（[PAT](https://developers.notion.com/guides/get-started/personal-access-tokens)）。
- **写入**：`POST /v1/pages` 正文可传 `children`（最多 100 块）或 `markdown`（2026-02-26 上线，超过 5,000 块报错，可 `allow_async`）；`PATCH /v1/blocks/{id}/children` 每次最多 100 个子块、最多两层嵌套；`PATCH /v1/pages/{id}/markdown` 支持 `update_content`（`old_str` → `new_str` 搜索替换，`old_str` 必须唯一）与 `replace_content`（[working with markdown](https://developers.notion.com/guides/data-apis/working-with-markdown-content)、[update page markdown](https://developers.notion.com/reference/update-page-markdown)）。rich text 单段 2000 字符，请求体最多 1000 个块元素且不超过 500KB（[request limits](https://developers.notion.com/reference/request-limits)）。
- **速率**：2026-09-09 起按套餐分，每个连接 Business 与 Enterprise 每分钟 600 次，其他每分钟 180 次；另有未公开的工作区级限额。多成员 Free 工作区终身 1,000 块的上限对 internal connection 生效、对 PAT 不生效（[workspace block limits](https://developers.notion.com/reference/workspace-block-limits)；生效日期 changelog 与限额页写法不一，待核实）。
- **文件**：File Upload API 直传最大 20MB，更大要分片且只限付费工作区；单文件上限 Free 5 MiB、付费 5 GiB；上传后 1 小时内必须挂到块上（[files and media](https://developers.notion.com/guides/data-apis/working-with-files-and-media)）。上线日期待核实（JS SDK v3.1.0 于 2025-05-21 加入支持）。
- **网页导入**：支持 md、csv、html、zip 等；frontmatter 是否变属性、相对路径图片是否保留，帮助页没写（[import](https://www.notion.com/help/import-data-into-notion)，待核实）。
- **读回**：`GET /v1/pages/{id}/markdown` 返回整页 Markdown，约 20,000 块以上截断并在 `unknown_block_ids` 列出（[retrieve page markdown](https://developers.notion.com/reference/retrieve-page-markdown)）；页面属性要另调 `GET /v1/pages/{id}`；可按 `last_edited_time` 过滤 data source 轮询。Webhooks（2025-03-26 上线）只告诉有变化、不带内容，接收端必须是公网 HTTPS，文档说 localhost 不可达（[webhooks](https://developers.notion.com/reference/webhooks)），本地应用只能轮询或自建中转。

<a id="55-飞书"></a>
### 5.5 飞书

- **数据模型**：docx 是云端块树，每个块有 `block_id`、`block_type`、`children`；没有 frontmatter，也没有用户自定义属性；文档有 `revision_id`（[get document](https://open.feishu.cn/document/server-docs/docs/docs/docx-v1/document/get.md)）。
- **导入任务**：先 `POST /open-apis/drive/v1/medias/upload_all`（`parent_type: ccm_import_open`，文件不超过 20 MB），再 `POST /open-apis/drive/v1/import_tasks`（异步，每分钟 100 次）；md、mark、markdown、txt、html 可导入为 docx，上限 20 MB（[import guide](https://open.feishu.cn/document/server-docs/docs/drive-v1/import_task/import-user-guide.md)）。导入任务只收一个文件，Markdown 里相对路径的图片应该带不进去（推断，待实测）。
- **转块**：`POST /open-apis/docx/v1/documents/blocks/convert` 把 Markdown 或 HTML 转成块，图片块要另外上传素材再 `replace_image` 绑定（[convert](https://open.feishu.cn/document/ukTMukTMukTM/uUDN04SN0QjL1QDN/document-docx/docx-v1/document/convert.md)；上线日期待核实，SDK 最早 2025-06-26 出现）。创建子块每次 1–50 个，每个应用、每篇文档各每秒 3 次；嵌套块接口每次最多 1000 个。
- **认证**：都要自建应用；`tenant_access_token` 由 `app_id` + `app_secret` 换取，最长 2 小时，应用身份建的文档归应用所有；`user_access_token` 走 OAuth，权限要企业管理员审核（[tenant_access_token](https://open.feishu.cn/document/server-docs/authentication-management/access-token/tenant_access_token_internal.md)）。
- **读回**：`GET /open-apis/docs/v1/content?doc_type=docx&content_type=markdown` 直接取 Markdown，每秒 5 次、单篇不超过 10 MB（[docs content](https://open.feishu.cn/document/docs/docs-v1/get.md)）；块树与纯文本也能读；编辑事件 `drive.file.edit_v1` 只有文档所有者或管理者能订阅，经 HTTP 回调或 SDK 长连接推送（[file edited](https://open.feishu.cn/document/server-docs/docs/drive-v1/event/list/file-edited.md)；长连接是否不需要公网地址待核实）。

<a id="56-语雀"></a>
### 5.6 语雀

接口定义取自官方仓库 [yuque/yuque-open-cli](https://github.com/yuque/yuque-open-cli) 的 `spec/yuque-openapi.yaml`（info.version 2.0.1）。

- **认证与限额**：请求头 `X-Auth-Token`；个人 token 标为「超级会员专享权益」，团队 token 标为「旗舰版空间专享权益」（[开发者文档](https://www.yuque.com/yuque/developer/api)，起始日期待核实）；每小时 5000 次、每秒 100 次，同一账号名下的 token 共用额度（[openapi](https://www.yuque.com/yuque/developer/openapi)）。
- **写入**：`POST /api/v2/repos/{book_id}/docs`，`format` 默认 markdown，也可 html、lake；新文档不会自动进目录，要另调 `PUT /api/v2/repos/{book_id}/toc`；请求体没有 tags 与自定义属性。OAS 里没有附件上传接口，官方 MCP 也写明不支持上传（[capability scope](https://github.com/yuque/yuque-mcp-server/blob/main/docs/capability-scope.md)），所以 Markdown 里的图片只能用公网 URL。
- **读回**：`GET /api/v2/repos/{book_id}/docs/{id}` 返回 `body`、`body_html`、`body_lake`、`updated_at`；列表支持 `changed_at_gte` 增量；`doc_versions` 返回最近 100 个版本；Webhook 回调要公网可达。网页编辑后 `body` 是否仍是 Markdown 待核实。

<a id="57-目标对比表"></a>
### 5.7 目标对比表

| 目标 | 存储形态 | 元数据载体 | 附件 | 链接与块引用 | 写入方式 | 主要限制 | 能否读回改动 |
|---|---|---|---|---|---|---|---|
| Obsidian 1.14 | 本地文件 | YAML frontmatter，7 种类型，不支持嵌套编辑 | vault 内普通文件，`![[x.png]]` 或 `![](path)` | `[[N#标题]]`、`[[N#^id]]`；无原生音视频 `#t=` | 写文件；URI（有确认框）；CLI（App 须运行） | 改属性会重写 YAML；重命名会改写其他笔记里的链接 | 能，直接读文件 |
| Logseq OG | 本地 Markdown | 首块 `key:: value`，单行 | `assets/`，`../assets/x.png` | `[[Page]]`、`((uuid))`，被引用的块要有 `id::` | 写文件；本地 HTTP API | 只维护不加功能；会回写 `id::` | 能，直接读文件 |
| Logseq 2.0 DB（beta） | 本地 SQLite | 带类型的 DB 属性与 tag | graph 下 `assets/` | 节点 uuid | 手动导入（每次新建 graph）；CLI `upsert`（不幂等）；MCP | beta；只能用已有属性 | 能（CLI `query`、EDN 导出）；Markdown Mirror 只往外写 |
| 思源 v3.8.8 | 本地 `.sy`（JSON AST）+ 可重建索引 | 块属性 IAL，`custom-*`，值为字符串 | `data/assets/`，经 `/api/asset/upload` | `((id "锚文本"))` | 本地 API（6806）；CLI；MCP；手动导入 | `createDocWithMd` 不解析 frontmatter；`importStdMd` 是内部 API | 能：`exportMdContent`、`getBlockAttrs`、SQL |
| Notion | 云 | 数据库属性（data source） | File Upload：Free 每文件 5 MiB | 按 URL 提及，无文本块锚点 | 云 API：块或 `markdown` 建页、`PATCH .../markdown` 局部替换 | 每请求 100 块、两层嵌套；每分钟 180 次（高档 600） | 能：`GET .../markdown` + `last_edited_time` 轮询；webhook 要公网 |
| 飞书 | 云 docx 块树 | 无；只能写进正文或另放多维表格 | `upload_all` ≤20 MB，图片块要先传再绑 | 超链接、`mention_doc` | 云 API：导入任务或 convert + 建块 | 自建应用 + 管理员审批；建块每秒 3 次 | 能：`docs/v1/content` 取 Markdown、`revision_id` |
| 语雀 | 云 | 无 | 无上传接口，只能公网 URL | 文档 URL | 云 API `POST .../docs`（markdown） | token 要超级会员或旗舰版空间 | 能：详情接口、`changed_at_gte`、版本列表 |

<a id="58-条目与主题页的映射与损失"></a>
### 5.8 条目与主题页的映射与损失

下表是导出时每个歇存元素落到各目标的位置。「Markdown 族」指 Obsidian 与 Logseq OG（两者共用「文件夹 + Markdown」形态，区别在元数据与块结构）。

| 歇存元素 | Obsidian | Logseq OG | 思源 | Notion | 飞书 / 语雀 |
|---|---|---|---|---|---|
| 条目本身 | 一条一个笔记 `歇存/条目/<标题> (<id8>).md` | 一条一页，页名 `歇存/<标题>`（文件名 `歇存___<标题>.md`） | 一条一篇文档，路径 `/歇存/条目/<标题>` | data source「歇存」里一页 | 一条一篇文档；飞书另配一张多维表格存元数据 |
| 身份 `id`、`identities[]` | fm `xiecun_id`、`source`（首个 URL）；其余身份写 list 属性 | 首块 `xiecun-id::`、`source::` | 文档块 `custom-xiecun-id`、`custom-source` | 属性 `Xiecun ID`（文本）、`URL` | 只能写进正文首段或表格 |
| 标题、作者、时间 | fm `title`、`author: ["[[作者]]"]`、`published`、`created`（对齐 Web Clipper 默认键） | `title::`、`author:: [[作者]]`、`published::` | 文档标题 + `custom-author`、`custom-published` | Title、Author、Published（date） | 正文首段 |
| 平台、类型 | fm `platform`、`kind` | `platform::`、`kind::` | `custom-platform`、`custom-kind` | Select | 正文或表格 |
| 三种标签 | `tags` 只放用户标签；平台标签放 `platform_tags`；建议标签不导出 | `tags::` 只放用户标签 | 思源标签 + `custom-platform-tags` | Multi-select 只放用户标签 | 正文 |
| 原文 `content.md` | 正文「原文」段 | 一级块「原文」，段落拆成子块 | 「原文」标题下的块 | 正文块 | 正文 |
| 图里的字 `ocr/*.json` | 「图里的字」段，每张图 `![[图]]` 后接文字；bbox 丢失 | 子块；bbox 丢失 | 可写成图片块的 OCR 文本（插件 [siyuan-plugin-paddleocr](https://github.com/Achuan-2/siyuan-plugin-paddleocr) 用 `/api/asset/setImageOCRText`），bbox 丢失 | 正文；bbox 丢失 | 正文；bbox 丢失 |
| 口播文字 `asr.vtt` | 「口播文字」段，每段一行，行首时间点链接：有平台 URL 时写 `[01:42](<url>?t=102)`，本地视频且用户装了 Media Extended 时写 `[[video.mp4#t=102]]`；每行带块 ID `^t102` 供引用；另附 `.vtt` 原文件 | 每段一块，时间点写平台链接；块带 `id::` | 每段一块，块属性存秒数 | 正文段落，时间点写平台链接 | 正文；本地视频的时间点失效 |
| 关键帧与画面描述 | `![[keyframes/0102.jpg]]` + 描述 | 同左，用 `../assets/` | 资源 + 块 | 图片块（受 5 MiB 限制） | 飞书要逐张上传绑定；语雀只能外链，本地关键帧基本丢失 |
| AI 摘要 | callout `> [!summary] AI 摘要（模型、时间）` | 一级块，属性 `source:: ai` | 块 + `custom-layer=ai` | callout 块 | 正文段，文字标注 |
| 我的笔记 `notes.md` | fm `takeaway` + 正文「我的笔记」受管区段（见 [§5.9](#59-读回双向的先例与歇存的做法)） | 首块 `takeaway::` + 「我的笔记」块 | `custom-takeaway` + 子文档或块 | 属性 Takeaway + 正文末段 | 正文末段 |
| 媒体 `images/`、`media/` | 复制或硬链接到 `歇存/附件/<id>/`，保持条目内相对结构 | `assets/xiecun/<id>/` | `/api/asset/upload` | 图片走 File Upload；视频只放链接 | 飞书逐个上传；语雀只能外链 |
| `raw/`、`fingerprints`、`snapshots`、`jobs` | 不导出 | 不导出 | 不导出 | 不导出 | 不导出 |
| 同一内容 `sameAs` | fm list `same_as: ["[[另一条]]"]` | `same-as:: [[...]]` | 块引用 | Relation 属性 | 正文链接 |
| 主题页 `topics/<slug>.md` | `歇存/主题/<标题>.md`；fm `members: ["[[条目]]", ...]`；正文引用写 `[[条目#^t102]]` 并保留原话引用块 | 页面 + `members::`；引用写 `((uuid))` | 文档 + 块引用 `((id "原话"))` | 页面 + Relation 指向成员；引用是页面提及 + 原话 | 文档；引用只剩原话与链接 |
| 画布 `canvas.json` | `.canvas`，扩展字段丢弃（[04 §6.2](04-tech-options.md#62-canvasjson-与-json-canvas)） | 无对应 | 无对应（思源白板能力待核实） | 无对应 | 无对应 |
| 条目列表视图 | 一个 `歇存.base`：`file.inFolder("歇存/条目")`，Table 与 Cards 视图 | 查询块（待核实） | 属性视图数据库（待核实） | data source 视图即是 | 飞书多维表格 |

**各目标的主要损失**：

- 所有目标都丢 OCR 的 bbox、多次快照、raw、指纹，也都不区分「平台带的 / 建议的 / 我的」三种标签来源，只保留用户标签。
- Obsidian：音视频时间点没有原生跳转，依赖平台 URL 或 Media Extended；属性面板改动会重写 YAML。
- Logseq OG：Markdown 正文要改写成大纲块，嵌套与多行属性值表达不了；Logseq 会往文件回写 `id::`；产品已进入只维护状态，DB 版仍是 beta。
- 思源：要走 API 并在 App 运行时导出；frontmatter 不会自动变属性，资源路径要按 `succMap` 改写。
- Notion：Free 工作区单文件 5 MiB、多成员 Free 工作区有终身块数上限，大量关键帧与视频放不进去；本地时间点链接失效。
- 飞书：没有元数据位，要另建多维表格；需要自建应用与管理员审批，个人用户门槛高。
- 语雀：没有附件上传，本地图片与关键帧只能先放到公网图床（这会让内容离开本机，要在设置里写明）；token 需要付费档。

<a id="59-读回双向的先例与歇存的做法"></a>
### 5.9 读回：双向的先例与歇存的做法

| 项目 | 方向 | 怎么区分机器区与用户区 | 冲突处理 | 来源 |
|---|---|---|---|---|
| ZotFlow | Zotero ↔ Obsidian | fm `zotflow-locked: true`、`zotero-key`、`item-version`；正文用 HTML 注释分三种区：`ZF_NOTE_*`、`ZF_ANNO_*` 可编辑并回传，`ZF_PERSIST_*` 只留本地；区外在编辑器里只读；重新生成时先取出 PERSIST 区再放回，模板删掉的区移到文末孤儿段；标记写错、ID 重复、区嵌套时拒绝覆盖 | 按字段三方合并，两边都改同一字段才算冲突，让用户逐字段选 | [duanxianpi/zotflow](https://github.com/duanxianpi/zotflow)（214 ★，v1.7.0）`src/utils/persist-regions.ts`、`src/db/sync/decide.ts` |
| Readwise 官方插件 | Readwise → Obsidian | 不分区，整个文件 MD5 与服务端 `last_content_hash` 比 | 没改过整份覆盖，改过只追加；帮助页说暂无双向同步 | `src/main.ts`；[docs](https://docs.readwise.io/readwise/docs/exporting-highlights/obsidian) |
| Readwise Mirror | Readwise → Obsidian | 正文归插件；`protectedFields` 列出的 fm 键不覆盖 | 以远端为准 | [jsonMartin/readwise-mirror](https://github.com/jsonMartin/readwise-mirror) |
| Zotero Integration | Zotero → Obsidian | 模板 `{% persist "notes" %}` 写成 `%% begin notes %%`…`%% end notes %%`；文末 `%% Import Date: <ISO> %%` | 保留区是旧内容加本次输出，区外整份覆盖 | [community-archive/obsidian-zotero-integration](https://github.com/community-archive/obsidian-zotero-integration)（1786 ★，最近版本 2024-08-11）`docs/Templating.md` |
| Kindle 插件 | Kindle → Obsidian | 每条高亮行尾块 ID `^ref-<id>` 当锚点 | 只插入新高亮，已有文字不动 | [hadynz/obsidian-kindle-plugin](https://github.com/hadynz/obsidian-kindle-plugin) `src/sync/diffManager/index.ts` |
| Zotero Better Notes | Zotero 笔记 ↔ Markdown | 不分区；fm `$version`、`$libraryID`、`$itemKey`，Zotero 设置里记两边上次同步的 MD5 | 一边变了用这边覆盖另一边；两边都变弹对比窗口手动合并 | [windingwind/zotero-better-notes](https://github.com/windingwind/zotero-better-notes)（8391 ★）`src/modules/sync/hooks.ts` |
| Notional | Obsidian ↔ Notion | 不分区；fm 记 `notionPageId`、`notionLastEditedTime`、`obsidianLastSyncedAt` | 两边都改就停下让用户选一边 | [bryanbans/Notional](https://github.com/bryanbans/Notional)（10 ★，2026-06 建） |
| 思源 weread-sync | 微信读书 → 思源 | 块属性存远端 hash | 三方比对；两端都改时保留本地，旁边插远端新版 | [§1.3](#13-其他笔记软件的同类插件) |

**歇存的做法（提议）**，前提是 [product-model §6](../product-model.md#6-知识层) 的两条：主题页是普通 Markdown、外部改过之后歇存重新读取；机器写的层可以重算。

1. **只读回用户写的东西。** 可读回的是 `notes.md` 的 `takeaway` 与正文、主题页、用户标签；原文、图里的字、口播、画面、AI 摘要都是导出时重新生成的，用户在笔记软件里改了也不读回（界面上写明「这几段每次导出会重写」）。
2. **身份写在文件里，不靠映射表。** 每个导出文件带 `xiecun_id`（Obsidian fm / Logseq `xiecun-id::` / 思源 `custom-xiecun-id` / Notion 属性）；导出记录 `.xiecun/exports/<target>.json` 只是缓存，丢了可以按身份扫描重建（bilinote-sync 与 [SPEC §2](../SPEC.md#2-资料库目录真相源) 的「文件是真相」）。用户重命名、移动导出文件后照样能认回来。
3. **用户区用正文标记，不用 YAML 注释。** Obsidian 用 `%% xiecun:begin notes %%`…`%% xiecun:end notes %%`（Obsidian 会删 YAML 注释；`%%` 在阅读视图隐藏），通用 Markdown 与 Logseq 用 HTML 注释（omni-collector、ZotFlow 的做法）。标记缺失、重复或嵌套时不覆盖，只报告（ZotFlow）。
4. **变更检测用「上次导出的哈希」**：导出时记下每个文件及其用户区的哈希（Readwise、Better Notes 的做法）。再次导出时：文件没被改过就整份重写；用户区被改过就先把它读回成一次「来自 Obsidian 的改动」，写进改动记录（可整批撤销），再重写机器区；歇存这边与笔记软件那边都改过同一个用户区时，不自动合并，两版并列给用户选（Notional、Better Notes）。
5. **主题页有两种接法，二选一由用户定**：(a) 资料库里的 `topics/` 直接作为 Obsidian vault 的一个文件夹（或整个资料库当 vault），主题页只有一份，Obsidian 里改的就是真相文件，歇存监听文件变化重新读取；(b) 导出一份副本到已有 vault，按上面 1–4 条读回。(a) 最接近 product-model 的原意，但要处理 Obsidian 改属性时重写 YAML、重命名改写链接这两件事；(b) 不碰用户 vault 的结构，代价是有两份文件。
6. **云端目标的读回只做轮询。** Notion 与语雀的 webhook 要公网 HTTPS，本地应用收不到；用 `last_edited_time`（Notion）、`changed_at_gte`（语雀）、`revision_id`（飞书）轮询，读回 Markdown 后同样只取用户区。没有用户区概念的平台（飞书、语雀没有注释语法可用，HTML 注释是否保留待核实）建议只做单向导出。

<a id="6-对歇存插件化架构的启示"></a>
## 6. 对歇存插件化架构的启示

下面是形状建议，不替 SPEC 和 ADR 做决定；需要拍板的事列在 [§6.8](#68-需要拍板的事)。

<a id="61-插件分三类"></a>
### 6.1 插件分三类

| 类 | 做什么 | 输入 | 输出 | 典型例子 | 法律与信任 |
|---|---|---|---|---|---|
| 获取 fetch（即「获取方式」） | 从平台或外部文件取回内容 | URL、订阅源、导出包 | `raw/` 快照 + 文本 + 媒体（[SPEC §8](../SPEC.md#8-适配器协议-v0)） | 抖音、公众号获取方式；Omnivore zip、BiliNote `note_results`、flomo 导出包的导入 | 风险最高；官方不内置、不收录（[PRD §4](../PRD.md#4-原则) 原则 1，Tachiyomi 教训见 [02 §1](02-adapter-ecosystems.md#1-tachiyomi-与-mihon-扩展生态)） |
| 转换 transform | 把条目里已有的文件变成新的一层 | 条目文件的只读副本或路径 | `derived/` 下的约定文件、`images/keyframes/` | ffmpeg 抽音轨与关键帧、ASR、OCR、画面描述、AI 摘要与标签建议、HTML→Markdown、字幕互转 | 平台中立，可进官方索引 |
| 导出 export | 把条目和主题页写到别的软件，可选读回用户改动 | 条目与主题页的只读视图（JSON） | 交给宿主写入的文件，或经宿主代理的云 API 调用；读回时返回结构化改动 | Obsidian、通用 Markdown 文件夹、Logseq OG、思源、Notion、飞书、语雀 | 平台中立，可进官方索引；云端目标要标「离开本机」 |

不单列的两样：

- **导入**是获取的一种（来源是文件而不是 URL），沿用获取方式的协议，`origin.entry` 记 `import`。
- **模型 provider** 不是插件，是内核里的一份配置：名称、OpenAI 兼容端点、模型、密钥（存系统钥匙串）。转换插件只引用 provider 名称，请求由内核代签代发（[§2.5](#25-运行方式统计与可复用协议) 第 6 条）。这样同一个本地 Ollama 或云端 key 只配一次，插件拿不到密钥明文，避免 AI Providers 那种「谁都能读走全部密钥」的问题。

留在内核、不做成插件的是：编排（按转换插件声明的输入输出把 ffmpeg → ASR → 摘要串起来）、权限落实、provider 代理、导出记录与读回合并、写资料库。宿主始终是唯一写资料库的一方：插件把结果写到宿主分配的临时目录，宿主校验后移入 `derived/` 或导出目录（[§4.4](#44-原生进程还是-wasm)）。

<a id="62-进程模型与协议"></a>
### 6.2 进程模型与协议

| 类 | 进程模型 | 控制协议 | 数据通道 | 网络 | 默认权限 |
|---|---|---|---|---|---|
| 获取 | 独立进程（[SPEC §8](../SPEC.md#8-适配器协议-v0) 的 `cli` / `local-service` / `mcp` / `http-api` / `browser`） | 现有子命令 `canonicalize` / `fetch` / `list` / `health`，加 [02 §15.3](02-adapter-ecosystems.md#153-调用错误与产物) 建议的错误信封、退出码、超时 | `--out` 临时目录 | 需要；登录态按 [02 §15.4](02-adapter-ecosystems.md#154-登录态) 经环境变量传 cookie 文件 | 网络 allowlist 限到平台域名；只写临时目录 |
| 转换（重：ffmpeg、ASR、OCR、本地 VLM） | 原生独立进程，可常驻（模型加载一次处理多条） | stdio 上换行分隔的 JSON-RPC 2.0（与 MCP stdio 同一种分帧）：`describe`、`health`、`run`，`progress` 通知，取消，关 stdin 后退出 | 只传文件路径，大块数据不走管道（Speech Kit 的 PCM 帧是反例） | 默认无；模型由宿主下载 | 读输入与模型目录、写临时目录、GPU 按声明 |
| 转换（轻：规范化、字幕互转）与导出到文件 | 先用原生进程；有了纯变换插件后由官方 `wasm-runner`（内嵌 wazero）承载 | 同上 | stdin 进 JSON，stdout 出 JSON 或文件字节 | 无 | 零能力 |
| 转换（HTTP 型：AI 摘要、云 ASR、云 VLM）与导出到云 | 同上 | 同上 | 同上 | 只经宿主代理，按 allowlist；密钥由宿主注入 | 网络 allowlist |
| 外部服务后端 | 转换插件做成薄客户端，背后是用户自己跑的服务 | 对外兼容 OpenAI `/v1/audio/transcriptions`、whisper-asr-webservice `/asr`、OpenAI chat、Umi-OCR `/api/ocr` | HTTP | 只到 `127.0.0.1` 或用户配置的主机 | 网络 allowlist |

三类插件共用一份清单格式，在现有 `adapter.yaml` 上扩展而不是另起三套（字段名是提议）：

- 共有：`id`、`name`、`version`、`protocol`（整数，[02 §15.1](02-adapter-ecosystems.md#151-声明与匹配) 已建议）、`kind: fetch | transform | export`、`license`、`homepage`、`maintainers`、`install`（各平台资产 + sha256，[02 §15.2](02-adapter-ecosystems.md#152-安装定位与更新)）、`requires.runtime`、`permissions`（[§4.5](#45-权限声明与各平台落实程度)）。
- 转换另有：`consumes`（如 `media:video`、`media:audio`、`image`、`text/html`）与 `produces`（如 `derived/asr.vtt`、`derived/ocr/*.json`、`images/keyframes/`、`derived/summary.md`），内核据此编排；`provider: optional | required`。
- 导出另有：`target`（如 `obsidian`、`notion`）、`writes: files | api`、`readback: true | false`。

导出插件的 `readback` 是一个独立调用：宿主给出目标侧当前内容与上次导出记录，插件只返回用户区的结构化改动，由宿主按 [§5.9](#59-读回双向的先例与歇存的做法) 的规则写进改动记录。

<a id="63-仓库怎么拆"></a>
### 6.3 仓库怎么拆

| 仓库 | 内容 | 许可证 | 维护 |
|---|---|---|---|
| `xiecun-app/xiecun`（内核） | 内核、CLI、MCP、桌面壳；插件运行器、权限落实、provider 代理、导出记录与读回合并；不含任何平台抓取逻辑 | AGPL-3.0 | 项目维护者 |
| `xiecun-app/plugin-kit`（协议与 SDK） | 插件清单与消息的 JSON Schema、`derived/` 文件格式、契约测试套件与 `testdata/`、Go / TypeScript / Python SDK、插件模板、本地校验命令 | MIT（[PRD §4](../PRD.md#4-原则) 原则 7） | 项目维护者 |
| `xiecun-app/plugins`（官方索引） | 只收转换与导出插件：每个插件一个 `index/<id>.yaml`（仓库地址、类别），生成的 `index.json`（版本、各平台资产 sha256、权限摘要）、`removed.yaml`、`deprecated.yaml`；校验与定时 stage 的 workflow | MIT 或 CC0 | 项目维护者 + 审核志愿者 |
| `xiecun-app/<kind>-<name>`（官方插件） | 一个插件一个仓库，例如 `transform-ffmpeg`、`transform-asr-sherpa`、`transform-ocr-rapid`、`export-obsidian`、`export-notion`；release 附各平台二进制与 sha256 | MIT；包装 GPL 工具的按其许可证（合规边界待核实） | 项目维护者 |
| 社区获取方式 store | 按 [02 §15.7](02-adapter-ecosystems.md#157-适配器源信任与治理) 的 `store.json` → `index.json` 分层；官方客户端不内置任何获取方式 store | 各 store 自定 | 社区 |
| 社区插件仓库 | 转换与导出插件提 PR 进官方索引，或自建 store；获取方式只进社区 store | 作者自定 | 作者 |

两条拆分理由：

- **按法律风险分索引。** 获取方式是唯一涉及平台的一类，官方索引不收，客户端也不预置任何获取方式 store；转换与导出平台中立，可以有官方索引。这样即使某个获取方式出事，官方索引和插件生态不受连累（Tachiyomi 是应用、扩展、网站同在一个组织，一起下线；Keiyoushi 用独立组织接手，见 [02 §1](02-adapter-ecosystems.md#1-tachiyomi-与-mihon-扩展生态)）。
- **官方插件也走插件协议。** Obsidian 导出、ffmpeg、默认 ASR、默认 OCR 都做成独立仓库的官方插件，随安装包预装，而不是写进内核。内核保持小，协议由官方插件先用起来，社区插件拿到的是同一套能力。

命名约定：`xiecun-app` 组织里的官方插件不加前缀（如上表的 `transform-ffmpeg`）；社区插件仓库名用前缀 `xiecun-transform-`、`xiecun-export-`、`xiecun-fetch-`。两者都加 GitHub topic `xiecun-plugin` 便于发现（gh extensions 用 `gh-` 前缀加 topic，见 [02 §12.5](02-adapter-ecosystems.md#125-gh-extensions)）。

<a id="64-索引用-github-仓库还是-github-app"></a>
### 6.4 索引用 GitHub 仓库还是 GitHub App

**判断依据**是 [§3.9](#39-github-app-与-actions-的能力差别) 的那条事实：fork 来的 `pull_request` 里 `GITHUB_TOKEN` 只读，但 `pull_request_target` 可以给外部提交评论、打标签，只要不执行 PR 代码。所以「每插件一个仓库 + 向索引仓库提 PR + Actions 自动校验」不需要任何托管服务就能做全；GitHub App 只在四件事上不可替代：不需作者配合就收到插件仓库的 release 事件、跨仓库写入并以 `[bot]` 出现、持久运行与跨事件调度、读私有源码。

**建议：仓库为真相，App 只做加速，按需后加。**

1. **先做「GitHub 仓库索引」这种读法。** 规模参照：思源 bazaar 608 个插件、Logseq 651 个、Zed 1583 个，都是仓库索引。
   - 提交：PR 只加一个 `index/<id>.yaml`（一个 PR 一个插件，思源与 Zed 的规则）。
   - 校验分两段（Security Lab 的拆法）：`pull_request` 只读任务在临时环境里下载插件 release、校验清单 schema、sha256、权限声明、跑契约测试，结果上传为 artifact；`workflow_run` 任务读结果、以 `github-actions[bot]` 评论和打标签。`pull_request_target` 里只读 YAML，不签出、不执行任何 PR 带来的东西（[actions/checkout](https://github.com/actions/checkout) v7 默认也拒绝）。
   - 更新：定时 stage 任务轮询各插件的 Latest Release，校验后把版本与 sha256 写进生成的 `index.json`；校验失败在索引仓库开 issue 并 @ 作者（思源模式）。更新不需要作者再提 PR，但每个新版本都过同一套自动校验（Obsidian 改制的教训：只审首版不够）。
   - 下架：`removed.yaml`（原因、日期）、按版本禁用、`malicious` 清单，客户端读取后执行（Logseq `blocked.json`、Open VSX `extension-control`、Mihon 的 orphaned）。
2. **需要 `[bot]` 身份或跨仓库写入时，先用 Actions 内换 App token**：注册一个只装在索引仓库的 GitHub App，私钥放仓库 secrets，workflow 里用 `actions/create-github-app-token` 换 1 小时 token（Zed、nixpkgs 的做法），仍然不需要托管服务。
3. **托管 GitHub App 放到确实需要时再加**，触发条件是：插件多到轮询不经济、要推送式即时更新、要读私有源码核对构建、或要逐版本扫描服务（Obsidian 在约 9600 个插件时走到这一步，并把提交入口搬到了自己的网站）。成本是持久运行的 webhook 接收端、私钥托管（DefinitelyTyped 用 Azure Key Vault）、webhook 签名校验、排队与重试。krew-release-bot 的服务端没看到请求来源认证，说明这类端点的安全容易被忽视。即使加了 App，它也只给索引仓库开 PR 或提交，索引仍以仓库里的文件为准：每次变更是一条提交，可审计、可 fork，与「文件为真相」一致。
4. **另一种读法值得借用的是 GitHub App 的权限模型，而不是 App 本身**：插件在清单里声明权限与订阅的事件；用户按资料库安装并同意；插件新版本扩大权限时，像 GitHub App 改权限需要安装方重新批准一样，新版本先停用、等用户重新同意（[§3.9](#39-github-app-与-actions-的能力差别)）；插件拿到的凭据短期有效、由宿主注入。这一条直接对应 [§4.5](#45-权限声明与各平台落实程度) 的权限块和 GlassWorm「恶意代码经自动更新下发」的教训：客户端默认不静默更新扩大了权限的版本。

<a id="65-怎么复用现有插件与工具"></a>
### 6.5 怎么复用现有插件与工具

不在歇存里运行 Obsidian 插件：它们依赖 Obsidian API、跑在渲染进程、没有隔离（[§4.1](#41-宿主插件的安全模型)）。复用有五条路：

1. **导出格式对齐，让 Obsidian 插件直接在导出的笔记上工作。** frontmatter 沿用 Web Clipper 默认键（[§1.1](#11-obsidian-社区插件)）；视频条目写 Media Extended 认的 `media:` / `video:` 属性和 `#t=` 时间链接，用户装了 Media Extended 就能点时间点播放；条目列表配 `.base` 文件；画布导出 `.canvas`。
2. **协议兼容，双向都能用。** 歇存的转写插件做成 OpenAI 兼容转写客户端，后端可以是 speaches、whisper.cpp server、SenseVoice-OneApi 或云端；反过来，歇存自己的本地 ASR 也可以对外提供同一个端点，用户 Obsidian 里的 Whisper、Scribe 插件就能用上歇存已经下载好的本地模型。LLM 同理用 OpenAI chat；OCR 可选 Umi-OCR（仅 Windows、Linux）作后端。
3. **把现成 CLI 包成转换插件**：`whisper-cli`、`tesseract`、Buzz、WhisperX、noScribe、VideoCaptioner。二进制只从上游官方 release 下载并锁 sha256（Offline Whisper 从非官方仓库下载二进制是反例）；GPL 工具只以独立进程调用，随包分发的合规边界待核实；VideoCaptioner 的必剪、剪映后端是非公开接口，不作默认。
4. **把别的工具的产物当导入来源**：BiliNote 的 `note_results/*.json` 与截图（bilinote-sync 已证明格式可读）、Readwise / Omnivore / flomo / 微信读书导出、Web Clipper 生成的 `Clippings/` 笔记，都写成导入类获取方式，落成带 `origin.entry: import` 的条目。
5. **借代码起步**：Power Extract 的 PowerShell OCR 脚本（MIT）、TextLens 的 Vision helper（MIT）、Speech Kit 的 Rust sidecar（MIT）可以作为官方 Windows OCR、macOS OCR、本地 ASR 插件的起点。Text Extractor 是 GPL-3.0，不能搬进 MIT 插件。

平台抓取型插件（两款小红书、两款抖音、B站字幕类、anycontent）不作为歇存组件复用。它们的本地后端做的就是获取方式的事；更好的结果是请这些作者把后端包装成歇存获取方式，放进社区 store。

<a id="66-导出目标的优先顺序"></a>
### 6.6 导出目标的优先顺序

| 顺序 | 目标 | 理由 |
|---|---|---|
| 1 | Obsidian（同时产出通用 Markdown 文件夹） | 直接写文件、能直接读回、没有配额；歇存 `content.md` 带 frontmatter 本来就是为它设计（[01 Obsidian Web Clipper](01-comparable-products.md#obsidian-web-clipper)）；Bases 与 Canvas 能承载列表与画布；把平台内容转进 Obsidian 的插件最多（[§1.1](#11-obsidian-社区插件)，weread 20.6 万、Readwise 25.2 万下载） |
| 2 | 思源 | 本地 API、能读回、`custom-*` 属性能承载身份；中文用户多；代价是 App 须在运行、要 token、frontmatter 要单独转属性 |
| 3 | Notion | API 成熟，2026 年有了 Markdown 读写端点，能读回；代价是云端、Free 工作区单文件 5 MiB 与块数上限、本地媒体受限、webhook 收不到只能轮询 |
| 4 | Logseq OG | 和 Obsidian 同属 Markdown 族，实现成本低；但 OG 只维护不加功能，会回写 `id::`；DB 版等 stable 后再看（CLI 插入不幂等，Markdown Mirror 不是双向） |
| 5 | 飞书 | 有 Markdown 导入与读取接口；但要自建应用和管理员审批，没有元数据位，图片要逐个上传绑定；适合有企业租户的用户 |
| 6 | 语雀 | token 要超级会员或旗舰版空间；没有附件上传，本地图片只能先外链，这会让内容离开本机 |

<a id="67-读回与文件为真相源"></a>
### 6.7 读回与「文件为真相源」

- 读回只取用户写的层（`notes.md`、主题页、用户标签），机器层每次导出重写，规则见 [§5.9](#59-读回双向的先例与歇存的做法)。
- 从笔记软件读回的改动按「用户自己的改动」写进改动记录，可整批撤销；它不是 Agent 写入，不走改动提议。两边都改过同一个用户区时不自动合并。
- 导出插件不直接写目标目录：它把结果交给宿主，宿主写文件、记哈希。导出记录放 `.xiecun/exports/`，可以按文件里的 `xiecun_id` 重建。
- 插件的启用状态、配置与密钥放本机配置目录，不放进会被同步的资料库，避免 REF6598 那种「同步过来的插件配置就能执行」（[§4.1](#41-宿主插件的安全模型)）。

<a id="68-需要拍板的事"></a>
### 6.8 需要拍板的事

| 编号 | 问题 | 建议 | 影响 |
|---|---|---|---|
| P1 | 三类插件共用一份清单，还是各自一份 | 在 `adapter.yaml` 上扩展 `kind`，共用 schema | [SPEC §8](../SPEC.md#8-适配器协议-v0)、`pkg/contract` |
| P2 | 转换插件的控制协议 | stdio 换行分隔 JSON-RPC 2.0，只传文件路径 | 新 ADR；plugin-kit SDK |
| P3 | 要不要 `wasm-runner` | 要，但等出现纯变换插件再做；wazero 只进 runner，不进内核 | 新 ADR；macOS entitlement 只给 runner |
| P4 | OS 沙箱做到哪一档 | macOS Seatbelt、Linux bubblewrap（降级 Landlock）、Windows 先只强制 Job Object；界面分「强制 / 部分 / 仅告知」 | 新 ADR；设置页 |
| P5 | provider 由内核代理、密钥进钥匙串 | 是 | [product-model §8.1](../product-model.md#81-设置清单) 的 AI 分组 |
| P6 | 主题页用「资料库即 vault」还是「导出副本」 | 两种都支持，默认导出副本，不碰用户 vault 结构 | [product-model §6](../product-model.md#6-知识层) 规则 4 |
| P7 | 官方索引的范围 | 只收转换与导出；获取方式只进社区 store | [PRD §4](../PRD.md#4-原则) 原则 1 |
| P8 | 「导出」进不进 [PRD §4](../PRD.md#4-原则) 原则 4 的「一切可插拔」列表 | 进，和获取方式、识别器并列 | PRD；product-model 设置清单加「导出」分组 |
| P9 | GPL 工具能否随安装包分发 | 先只做「用户自己安装、歇存调用」，分发前取得法律意见 | 官方插件选型 |

<a id="7-待核实清单"></a>
## 7. 待核实清单

**平台内容转笔记（[§1](#1-平台内容转笔记的工具)）**

- Readwise、Snipd 导出的属性字段名（都在服务端渲染）；Readwise 帮助页说不覆盖用户改动，源码却在哈希一致时整份覆盖，服务端哈希怎么算。
- wechat-obsync 的云端解析代码是否开源；wechat-note-helper 默认目录 README 与代码不一致。
- Media Extended v4 的可选在线 AI 用哪家服务。
- BibiGPT 飞书导出的方式；网页端「保存到 Obsidian / Logseq」是否用 URL scheme；桌面端时间戳回链语法。
- BiliNote B站跳转链接在无 `?p=` 时写成 `/video/<BV>&t=<秒>`，能否跳到对应时间未实测。
- 思源 weread-sync 不带 `custom-` 前缀的块属性，内核是否保留。
- 中文平台（B站、小红书、公众号）的 Web Clipper 模板集合。
- 语雀、飞书方向专门剪藏 B站或公众号内容的工具，除 VidkNot、wandao、简悦外没找到。

**转换（[§2](#2-转换类插件与工具)）**

- Obsidian `app.secretStorage` 的起始版本与加密方式；Copilot 说的 Keychain 是否同一个 API。
- Offline Whisper、Speech Kit 下载二进制时是否校验哈希。
- speaches、SenseVoice-OneApi、WhisperX 支持的输出格式完整取值。
- GPL-3.0 / AGPL-3.0 工具作为独立进程被调用并随歇存分发的合规边界（需要法律意见）。
- Umi-OCR 是否有 macOS 计划。

**注册表（[§3](#3-插件注册表与分发)）**

- Obsidian 描述后缀「not manually reviewed」的官方含义与规则（现为按提交历史推断）；`ObsidianReviewBot` 的驱动方式；2026-10-30 之后老插件的实际下架；迁移后 `community-plugins-removed.json` 是否还维护。
- Logseq 客户端从哪里下载插件；合并者 `xyhp915` 的身份。
- Raycast Store 端下架机制；Zed `zed-zippy[bot]` 版本更新 PR 的触发方式；HACS `hacs-bot[bot]` 托管在哪；Flathub 下架流程；conda-forge 建 feedstock 的 job 位置。
- krew-release-bot webhook 是否校验请求来源；旧 App 路径是否还在用。
- Open VSX 2026-02 发布前检查的 Eclipse 原帖与强制执行日期；「Open Sesame」细节（目前都是二手报道）。

**安全与隔离（[§4](#4-安全与隔离)）**

- Logseq 安装插件时有无授权界面；`effect` 插件能否调 preload 暴露的 API。
- 思源前端与内核插件有无签名、有无逐插件授权；goja 内核插件有无 CPU 与内存上限。
- Zed 扩展商店有无签名。
- Seatbelt 放行 Metal / VideoToolbox 需要的最少规则；macOS 上 `RLIMIT_AS` 是否生效。
- Linux 桌面会话里 cgroup v2 委派是否普遍可用；GPU 设备最少要 bind 哪些；纯 Go seccomp 库现状；bubblewrap 的许可证（GitHub API 返回 NOASSERTION）。
- Windows AppContainer 对 GPU 与 Media Foundation 的影响；MXC 执行器能否用于生产。
- wazero 有无按指令计数的配额；Wasmtime 的资源限制与 wasi-nn 实现。
- GlassWorm 的时间线与数字（只有二手报道）。

**导出格式（[§5](#5-导出与互通格式)）**

- Obsidian `.obsidian/types.json` 与 `app.json` 的键名（官方无文档）；1.13.1 外部改动被覆盖的 bug 在哪个版本修复；CLI 的退出码与无图形界面时能否使用。
- Logseq 2.0 stable 的日期；OG 是否仍把 YAML frontmatter 解析成页面属性；OG 关着时文件被改，下次启动是否自动重新解析。
- 思源对同一 `path` 重复调用 `createDocWithMd` 的实际结果；内核运行时外部改了 `.sy` 会不会被覆盖；思源白板与属性视图能否承载画布和条目列表。
- Notion File Upload API 的上线日期；网页导入 Markdown 时 frontmatter 与相对路径图片怎么处理；Free 工作区块数上限的生效日期（2026-09-01 还是 2026-09-08）。
- 飞书导入 Markdown 时图片与 frontmatter 怎么处理（需实测）；`blocks/convert` 与 `docs/v1/content` 的上线日期；长连接收编辑事件是否不需要公网地址；HTML 注释在飞书、语雀里是否保留。
- 语雀「个人 token 限超级会员」的起始日期；网页编辑后 `body` 是否仍是 Markdown；外链图片会不会被转存。
- ZotFlow 在 Obsidian 外改了文件，可编辑区的改动能否传回 Zotero。

