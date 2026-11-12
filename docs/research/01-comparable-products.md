<a id="01--同类与相邻产品调研"></a>
# 01 · 同类与相邻产品调研

状态：草案 · 2026-10-21 · 2026-11-03 审过：[§1](#1-产品分述)、[§2](#2-对比表)、[§3](#3-三个重点问题)、[§5](#5-歇存刻意不做的) 的事实仍可参考；[§4](#4-对歇存的启示) 的启示 1、8、17 已进 [open-decisions P3](../PRD.md#72-获取插件)、[ADR-002](../decisions/ADR-002-frontend-stack.md)、[E1](../PRD.md#79-导出与互通)；启示 11–14 的参考适配器、M0–M3 排期和「放下」已作废；文中指向 [SPEC §8](../protocol/README.md) 的是已废的 v0，以 [插件协议](../protocol/README.md) 为准。

> 里程碑口径已废弃：项目改为按完整功能一次性开发。文中的 M0–M3 只指写作时 SPEC 的分段，请按功能面阅读。

对象：收藏 / 稍后读 / 网页归档 / 个人知识库 / 中文平台个人备份工具。24 个产品各占一节，另两节汇总中文平台备份工具与两个相邻开源项目。每节回答同一组问题：是什么、数据模型与存储（本地文件还是数据库、是否可导出、用户是否拥有文件）、如何处理平台内容（能不能存抖音/小红书/公众号，靠什么）、搜索能力、AI 与 Agent 能力、许可证、活跃度、定价、对歇存的启示。重点看三件事，见 [§3](#3-三个重点问题)：谁把「文件归用户所有、索引可重建」做成架构原则；谁在做「减少注意力」而不是「收藏更多」；谁有 Agent 可读接口。

方法：官方网站、仓库 README、发布页、定价页、帮助中心，辅以 GitHub API（`archived` / `pushed_at` / `license` / 最新 release）取活跃度与许可证。所有数据截至 2026-10-21。凭记忆或仅有二手来源的地方标「待核实」，汇总见 [§6](#6-待核实清单)。本文不写任何平台抓取方法，第三方工具的能力按其 README 的自述转述，统一表述为「个人备份 / 离线阅读」。

目录

- [§1 产品分述](#1-产品分述)：[Karakeep](#karakeep) · [Readwise Reader](#readwise-reader) · [Raindrop.io](#raindropio) · [Pinboard](#pinboard) · [Wallabag](#wallabag) · [Linkding](#linkding) · [Omnivore](#omnivore) · [Instapaper](#instapaper) · [Pocket](#pocket) · [ArchiveBox](#archivebox) · [SingleFile](#singlefile) · [Obsidian Web Clipper](#obsidian-web-clipper) · [Zotero](#zotero) · [Eagle](#eagle) · [Anytype](#anytype) · [mymind](#mymind) · [Are.na](#arena) · [Fabric](#fabric) · [Recall](#recall) · [Cubox](#cubox) · [简悦](#简悦) · [方片收集](#方片收集) · [flomo](#flomo) · [OpenCLI](#opencli) · [中文平台个人备份工具](#中文平台个人备份工具) · [Slax Reader 与 Linkwarden](#slax-reader-与-linkwarden)
- [§2 对比表](#2-对比表)
- [§3 三个重点问题](#3-三个重点问题)
- [§4 对歇存的启示](#4-对歇存的启示)
- [§5 歇存刻意不做的](#5-歇存刻意不做的)
- [§6 待核实清单](#6-待核实清单)

<a id="1-产品分述"></a>
## 1. 产品分述

<a id="karakeep"></a>
### Karakeep

- 是什么：自托管的「Bookmark Everything」应用（链接、笔记、图片），LLM 自动打标与摘要。原名 Hoarder，2025 年因商标争议改名 Karakeep（阿拉伯语「杂物」；具体月份待核实）。来源：<https://docs.karakeep.app/>、改名公告镜像 <https://piefed.jeena.net/post/148828>。
- 数据模型与存储：SQLite 放在 `DATA_DIR`（"This is where the db lives"）；抓取产物（截图、整页归档）放 `ASSETS_DIR`（默认 `DATA_DIR` 子目录）或 S3；全文搜索依赖外部 Meilisearch，`MEILI_ADDR` 未设则 "Search will be disabled"；整页归档 `CRAWLER_FULL_PAGE_ARCHIVE` 默认 false，默认只存可读文本；大于 5 KiB 的 HTML 从数据库挪到 asset store。来源：<https://docs.karakeep.app/configuration/environment-variables>。重建索引：CLI 有 `karakeep admin bookmarks reindex <bookmarkId>`（单条触发），另有 `recrawl`、`regenerate-embedding`、`retag`、`resummarize`。来源：<https://github.com/karakeep-app/karakeep/blob/main/apps/cli/src/commands/admin.ts>。导入 Netscape HTML、Pocket CSV、Omnivore JSON（`metadata_*.json`）；导出 Netscape HTML（0.33.1 发布说明提到 "Netscape HTML exports"）。来源：<https://docs.karakeep.app/using-karakeep/import>、<https://github.com/karakeep-app/karakeep/releases/tag/v0.33.1>。用户拥有服务器上的数据库与资产目录，但不是「每条一个可读文件夹」的结构；所有权靠自托管，不靠文件格式。
- 平台内容：通用无头浏览器抓取，无抖音/小红书/公众号专门支持；0.34 把抓取镜像换成 Chrome stable，因为受保护站点更常拦截 headless shell。来源：<https://github.com/karakeep-app/karakeep/releases/tag/v0.34.0>。
- 搜索：Meilisearch 全文；0.33 起语义搜索（embeddings 存在 Meilisearch 作为向量库）。图片走视觉模型打标（`INFERENCE_IMAGE_MODEL`）；是否做 OCR 文字抽取：待核实。来源：<https://github.com/karakeep-app/karakeep/releases/tag/v0.33.1>。
- AI 与 Agent：OpenAI 或本地 Ollama 打标/摘要；REST API；CLI（`@karakeep/cli`）；MCP server `@karakeep/mcp`（npx 或 Docker，`KARAKEEP_API_KEY`），文档列出 21 个工具，含 `delete-bookmark`、`delete-list`、`delete-tag`；0.33.1 发布说明称 MCP 从 8 个工具增至 29 个（文档与发布说明口径不一致，待核实）；0.34 升级到 MCP v2 SDK，"The CLI now bundles the official Karakeep skill"，Skill 可用 `npx skills add karakeep-app/karakeep` 安装。来源：<https://docs.karakeep.app/integrations/mcp>、<https://docs.karakeep.app/integrations/agentic-skills>、<https://docs.karakeep.app/integrations/command-line>。
- 许可证：AGPL-3.0。活跃度：v0.34.0 发布于 2026-10-17，仓库最近推送 2026-10-19，约 29.9k stars。来源：<https://github.com/karakeep-app/karakeep/releases>。
- 定价：自托管免费；官方托管版 Karakeep Cloud（cloud.karakeep.app）由团队运营，官方文档页未列价格；第三方称 Pro $4/月，待核实。来源：<https://docs.karakeep.app/installation/cloud-hosting>、<https://aisotools.com/pricing/karakeep>。
- 对歇存的启示：(1) 它是开源自托管里 Agent 接口最完整的，MCP 直接暴露删除且无确认环节，和歇存 [SPEC §10](../BUILD.md#006-agent)「不提供 delete」形成直接对照；(2) 「CLI 捆绑官方 Skill + skills.sh 分发」是比 MCP 更轻的接入方式，值得歇存在 M0 就写一份 `SKILL.md`；(3) 「搜索依赖外部服务、未配置则禁用」「默认只存可读文本、整页归档可选」是歇存反向选择的两点（bleve 内嵌、raw 永远保留）。

<a id="readwise-reader"></a>
### Readwise Reader

- 是什么：Readwise 公司的稍后读应用，与其 highlights 服务一体，订阅制。来源：<https://readwise.io/read>。
- 数据模型与存储：云端。导出：整库 CSV（"Export Library as CSV"）、订阅源 OPML、"Export Full Files and Articles" 生成 ZIP 并邮件发链接，含 "the full content of any articles you've saved from the web"；单篇高亮导出 Markdown；官方 Obsidian 插件把高亮和 Reader 文档写进 vault，append-only。来源：<https://docs.readwise.io/reader/docs/faqs/exporting>、<https://docs.readwise.io/readwise/docs/exporting-highlights/obsidian>。用户不拥有运行时文件，但可整库导出含正文，是云端产品里导出最完整的。
- 平台内容：网页文章、PDF、EPUB、YouTube 字幕高亮、Twitter/X 串、RSS、newsletter。公众号 / 抖音 / 小红书：无专门支持，公众号网页能否正常解析待核实。来源：<https://readwise.io/read>。
- 搜索："Blazingly fast full text search"，可离线。来源同上。
- AI 与 Agent：Ghostreader（GPT 阅读助手）；Reader API v3（`save` / `list` / `update` / `bulk_update` / `delete` / `tags`，`withHtmlContent=true` 可取回 HTML 全文，按 token 限速 20–50 次/分）；官方 MCP `https://mcp2.readwise.io/mcp`，22 个工具：highlights 6 个（含 `readwise_delete_highlight`）、Reader 文档 8 个（搜索、列出、创建、取详情含 Markdown 全文、移动位置、批量改元数据、触发导出）、标签 3 个、Reader 高亮 5 个；没有删除文档的工具（文档只能移到 archive）；支持 Claude Code、Codex、Cursor 等；2026-05-19 宣布脱离 beta。来源：<https://readwise.io/reader_api>、<https://readwise.io/mcp>、<https://x.com/readwise/status/2056788542614118592>（日期由帖子 ID 推算）、旧仓库 <https://github.com/readwiseio/readwise-mcp>（已标弃用）。
- 许可证：闭源。活跃度：2026 年持续更新（MCP 脱 beta 2026-05）。
- 定价：Readwise 完整版 $9.99/月（年付 $119.88）或 $12.99/月月付，Reader 含在内；Lite $5.59/月不含 Reader；30 天试用。来源：<https://readwise.io/pricing>。
- 注意力设计：收件箱 / Later / Archive 三态、"fun, game-like triage"、Daily Review 每日推送高亮——是「制造处理队列」的典型，与歇存 [PRD §4.5](../PRD.md#4-原则) 相反。
- 对歇存的启示：(1) 含正文的整库 ZIP 导出是云端产品的基线，歇存「文件即导出」天然满足，但迁出到其他工具仍需 Netscape HTML / CSV 这类通用格式（见 [§4](#4-对歇存的启示)）；(2) MCP 示例提示词写「做改动前先问我」，但这是提示词建议而非服务端保证——歇存把确认做进协议（propose → apply）是实质差异；(3) 不给 Agent 删除文档、只给「移到 archive」，和歇存回收站思路一致。

<a id="raindropio"></a>
### Raindrop.io

- 是什么：书签管理器，免费版无限书签 / 收藏夹 / 设备，Pro 加全文搜索、永久副本、备份。来源：<https://raindrop.io/pro>。
- 数据模型与存储：云端。导出 HTML（Netscape）/ CSV / TXT，可按收藏夹、搜索结果或手选导出；Pro 自动每日备份到 Dropbox / Google Drive（`Raindrop.io-Export.html` + `.csv`），保留 30 份；备份不含上传的文件，永久副本能否导出待核实。来源：<https://help.raindrop.io/export>、<https://help.raindrop.io/backups>。
- 平台内容：通用网页；Pro 全文搜索覆盖 YouTube 视频 "down to every spoken word"。来源：<https://raindrop.io/pro>。
- 搜索：Pro 全文搜索；MCP 文档提到 semantic search；图片 OCR 待核实。
- AI 与 Agent：Pro 功能列表含 "AI Assistant"、"AI Suggestions"、"MCP"、"Web archive"、"Reminders"。官方 MCP `https://api.raindrop.io/rest/v2/ai/mcp`（Streamable HTTP + OAuth 2.1，或 API token），工具覆盖书签 find / fetch full content / create / update / delete（"move to Trash, or delete permanently if already there"）、收藏夹、标签、高亮；免费版不能用文本与语义搜索。REST API OAuth，120 次/分。来源：<https://raindrop.io/pro/buy>、<https://help.raindrop.io/integrations/mcp>、<https://developer.raindrop.io/>。
- 许可证：闭源。活跃度：博客 2026 年仍在更新（Pocket 替代专题）。来源：<https://blog.raindrop.io/pocket-alternatives/>。
- 定价：官方购买页未标价（只标年付约 -20%）；第三方 2026-09 核对为 Pro $28/年，待核实。来源：<https://contextbolt.com/blog/raindrop-pricing/>。
- 对歇存的启示：删除做成两阶段（先入 Trash，再次删除才永久）说明商业产品也在给 Agent 删除加缓冲；歇存更进一步，MCP 不开删除，用户自己删也先进 `.trash/` 30 天（[SPEC §2](../spec/library.md#2-资料库目录真相源)）。

<a id="pinboard"></a>
### Pinboard

- 是什么：2009 年起的文本型个人书签站，自称 "Social Bookmarking for Introverts"、"a personal archive for things you find online and don't want to forget"，约 2.5 万活跃用户，单人维护。来源：<https://pinboard.in/>、<https://pinboard.in/tour/>。
- 数据模型与存储：云端。API v1（兼容 Delicious）返回 XML 或 JSON，`posts/all` 可拉全部书签（每 5 分钟一次）；归档副本（Pro 功能）不通过 API 提供。来源：<https://pinboard.in/api/>。
- 平台内容：归档账户会抓取每条书签的页面副本，"If the page you bookmarked goes offline, you'll still be able to see the archived copy indefinitely"；对中文平台无特殊处理。
- 搜索：归档账户可搜页面全文（待核实范围）。
- AI 与 Agent：仅 API；无 AI。
- 许可证：闭源。活跃度：官方博客最近一篇 2025-01-19 "A Cavalcade of Updates"，之前一篇是 2020 年。来源：<https://blog.pinboard.in/>。
- 定价：$22/年；归档账户 $39/年。来源：<https://pinboard.in/>、<https://pinboard.in/tour/>。
- 注意力设计："There are no ads and no trackers of any kind"、"no third party content"、高信息密度、快。是「少做」的早期范本，但仍有 popular / recent / network 等发现页。
- 对歇存的启示：(1) 归档副本锁在服务内且 API 拿不到，是「用户不拥有文件」的反例；(2) 它的自我描述方式（用否定句列出不做的事）值得歇存 [PRD §1](../PRD.md#1-定位) / [§6](../PRD.md#6-不做) 的文案借鉴。

<a id="wallabag"></a>
### Wallabag

- 是什么：自托管稍后读，"saves a web page by keeping content only"，2013 年起开源；官方托管服务 wallabag.it。来源：<https://doc.wallabag.org/>、<https://wallabag.it/en>。
- 数据模型与存储：PHP 应用，数据库可选 MySQL/MariaDB、PostgreSQL、SQLite（`pdo_mysql` / `pdo_pgsql` / `pdo_sqlite`）。导出 EPUB / PDF / TXT / CSV / JSON / XML，可按 Unread / Starred / Archive / 标签整批导出；导入 Pocket、Instapaper、Readability、Pinboard、elCurator、wallabag v1/v2，后续加入 Omnivore 导入（待核实版本）。来源：<https://doc.wallabag.org/en/admin/installation/requirements>、<https://helpdesk.wallabag.it/help/en-gb/3-your-data/29-export-to-your-e-reader>。wallabag.it 承诺 "export your articles, tags and annotations whenever you want"，"funded by subscriptions, not by ads, tracking or data resale"。
- 平台内容：通用正文抽取，无中文平台专门支持。
- 搜索：有文章搜索；是否全文索引待核实。
- AI 与 Agent：REST API（OAuth）；无官方 AI 或 MCP。
- 许可证：MIT。活跃度：最新 release 2.6.14 为 2025-10-07，仓库 2026-10-20 仍有推送，约 13k stars。来源：<https://github.com/wallabag/wallabag/releases>。
- 定价：自托管免费；wallabag.it 11 € / 年起，14 天试用。来源：<https://wallabag.it/en>。
- 对歇存的启示：导出格式面最宽的开源稍后读；它的 JSON 导出是歇存「其他工具导出」导入项之一（[PRD §5 导入方式](../PRD.md#导入方式)）。

<a id="linkding"></a>
### Linkding

- 是什么：自托管书签管理器，"designed to be minimal, fast, and easy to set up using Docker"。来源：<https://github.com/sissbruecker/linkding>、<https://linkding.link/>。
- 数据模型与存储：单容器 SQLite。HTML 快照两条路：`latest-plus` 镜像内置 `singlefile-cli` + headless Chromium 自动生成；或浏览器端 SingleFile 扩展上传到 `/api/bookmarks/singlefile/`，不存在的书签会自动创建；快照作为书签的 asset 保存；也可存到 Internet Archive。导入导出 Netscape HTML。来源：<https://linkding.link/archiving/>。
- 平台内容：通用；无中文平台处理。
- 搜索：标题 / 描述 / 笔记 / 标签；快照内容是否可搜待核实。
- AI 与 Agent：REST API；无官方 MCP。
- 许可证：MIT。活跃度：v1.47.0 发布于 2026-09-13，仓库 2026-10-01 推送，约 11.3k stars。来源：<https://github.com/sissbruecker/linkding/releases>。
- 定价：免费。
- 对歇存的启示：「浏览器 SingleFile 扩展 → 本地 API 收快照」是「所见即所存」的现成路径，歇存浏览器扩展（[PRD §5 收集入口](../PRD.md#收集入口)）可以直接复用 SingleFile 产出 HTML 作为 `raw/`。

<a id="omnivore"></a>
### Omnivore

- 是什么：开源稍后读。云服务 2024 年 11 月关停（团队加入 ElevenLabs），仓库声明 "now a completely Self-Hosted application"，"The Cloud application was deprecated in November of 2024"，社区在 Discord 继续。来源：<https://github.com/omnivore-app/omnivore>。
- 归档状态：任务前提说「已归档」，但 GitHub API 显示 `archived: false`；仓库 2026-10-21 仍在推送，最近 30 次提交全部来自维护者 Podginator（2026-10-18 至 10-20，内容为自托管文档、IMAP 邮件解析 Docker 镜像、RSS 命名），Android 版 0.230.0 于 2026-10-14 发布。结论：云关停、仓库未归档、由社区维护者低频维护。第三方提到的 omnivore.work 分叉站点未能核实。来源：<https://api.github.com/repos/omnivore-app/omnivore>、<https://github.com/omnivore-app/omnivore/commits/main>、<https://github.com/omnivore-app/omnivore/releases>。
- 数据模型与存储：PostgreSQL + Node/TypeScript API + 内容抓取微服务（puppeteer-parse）+ Next.js 前端；第三方称自托管还需 Elasticsearch（待核实）。导出：ZIP 内 `contents/*.html` 全文、高亮 Markdown、`metadata_*.json` 每 20 条一批。来源：<https://github.com/omnivore-app/omnivore>、<https://docs.omnivore.app/using/exporting.html>（本次抓取间歇失败，结构描述另据 <https://docs.karakeep.app/using-karakeep/import>）。
- 平台内容：通用正文抽取。
- 搜索：全文搜索（云版）；自托管实现待核实。
- AI 与 Agent：云版 API 已随服务关闭，依赖它的 Obsidian / Logseq 插件随之失效。来源：<https://www.yaps.ai/blog/omnivore-alternative>。
- 许可证：AGPL-3.0。
- 定价：无。
- 对歇存的启示：(1) 它的导出 ZIP 结构成为事实迁移格式，Karakeep、Readwise、Wallabag 都为它写了导入器——歇存导入方式应直接支持 Omnivore ZIP（[PRD §5 导入方式](../PRD.md#导入方式)）；(2) 「云 API 一关，整个插件生态同时死亡」是歇存「本地 HTTP API + MCP + 文件」三层并存的理由（[PRD §4.6](../PRD.md#4-原则)）。

<a id="instapaper"></a>
### Instapaper

- 是什么：2008 年起的稍后读。2026-07 推出改版网站、iOS 10 版和 Android 文本转语音重写。所有者在应用商店显示为 Instant Paper, Inc.；2018 年从 Pinterest 独立一事待核实。来源：<https://alternativeto.net/news/2026/7/instapaper-revamps-website-launches-major-ios-redesign-and-ai-text-to-speech-on-android/>、<https://www.apkmirror.com/apk/instant-paper-inc/instapaper/>。
- 数据模型与存储：云端。设置页可导出 CSV / HTML，只有链接、标题、文件夹，不含正文；Premium 才有「永久归档」。来源：<https://doc.wallabag.org/en/user/import/instapaper>、<https://contextbolt.com/blog/instapaper-alternatives/>。
- 平台内容：通用。
- 搜索：全文搜索为 Premium 功能（含已归档文章）。来源：<https://www.instapaper.com/premium>。
- AI 与 Agent：API 两档均可用（"API integration for third-party apps"）；Full API 需申请、xAuth 认证，`bookmarks/get_text` 可取正文——官方文档页本次未能抓到正文，待核实。来源：<https://www.instapaper.com/api>。
- 许可证：闭源。活跃度：2026-07 大版本。
- 定价：Premium $5.99/月或 $59.99/年。来源：<https://www.instapaper.com/premium>。
- 对歇存的启示：「导出只给链接、正文留在服务里」是稍后读的常态；歇存的 `content.md` 与 `raw/` 一起落盘，是刻意相反的选择（[PRD §4.2](../PRD.md#4-原则)、[§4.3](../PRD.md#4-原则)）。

<a id="pocket"></a>
### Pocket

- 是什么：Mozilla 旗下稍后读，2025-05 宣布关停。Mozilla 原话："Pocket shuts down July 8, 2025"，用户可导出至 2025-10-08，之后数据 "permanently deleted"；理由是 "the way people save and consume content on the web has evolved"，资源转向 Firefox。来源：<https://blog.mozilla.org/en/mozilla/building-whats-next/>。
- 导出窗口：2025-10-08 当天 Mozilla 支持页宣布延至 2025-11-06（Vice 2025-10-09 报道）；多篇 2026 年文章称又延至 2025-11-12 并在当日删除数据，该日期仅见二手来源，待核实。导出格式为邮件发送的 ZIP 内 CSV（Karakeep 文档称 "Pocket's new CSV format"）。来源：<https://www.vice.com/en/article/mozilla-pocket-gets-one-month-reprieve/>、<https://docs.karakeep.app/using-karakeep/import>。
- 搜索、AI 与 Agent、许可证、定价：闭源；服务已停，这些字段不再适用。
- 迁移去向：Raindrop、Instapaper、Readwise Reader、Wallabag、Karakeep、Cubox 都提供 Pocket 导入；替代清单文章普遍把 Raindrop 当「同类替换」、Instapaper 当「经典稍后读」、Wallabag / Karakeep 当「开源自托管」、Readwise 当「重注释」。来源：<https://blog.raindrop.io/pocket-alternatives/>、<https://stashr.me/blog/pocket-shutdown-alternatives>、<https://help.cubox.pro/start/4ad6/>。
- 对歇存的启示：(1) 服务关停是常态而非意外，歇存 [PRD §1](../PRD.md#1-定位)「内容随时会被删除、下架」可以直接引用 Pocket 和 Omnivore 两例；(2) Pocket CSV 必须在导入格式清单里（[PRD §5 导入方式](../PRD.md#导入方式)）；(3) 「保险赔付率」这个成功标准（[PRD §7](../PRD.md#7-成功标准)）正是 Pocket 用户在 2025-11 之后最需要的指标。

<a id="archivebox"></a>
### ArchiveBox

- 是什么：自托管网页归档，"Takes URLs/browser history/bookmarks/Pocket/Pinboard/etc., saves HTML, JS, PDFs, media, and more"，"stores data in ordinary files & folders"。来源：<https://github.com/ArchiveBox/ArchiveBox>。
- 数据模型与存储：`data/index.sqlite3` 是索引；快照按 `data/archive/users/{username}/snapshots/{YYYYMMDD}/{domain}/{Snapshot.id}/` 存放，每个目录带 `index.html` / `index.json` 和各抽取器的纯文件产物（SingleFile HTML、wget + WARC、PDF、截图、DOM、Readability/Mercury 正文、yt-dlp 媒体）。把快照目录复制到另一个库后执行 rescan，"The explicit rescan discovers directories absent from the database" 并恢复其身份与结果；"Conflicting identities are reported rather than overwritten"；"Copying captures does not require importing the old database"。普通 `init` / `update` 不扫全树。README 未直接写「索引可从目录完整重建」，但 rescan 机制事实上支持。抽取器拆到独立仓库 abx-plugins。来源同上。
- 平台内容：通用；yt-dlp 可存视频，不针对中文平台。
- 搜索：后端可选 ripgrep / Sonic / SQLite FTS5。
- AI 与 Agent：REST API（ALPHA）、Python API（BETA）；无官方 MCP。
- 许可证：MIT。活跃度：v0.9.73 发布于 2026-10-06，仓库 2026-10-20 推送，约 28.8k stars。来源：<https://github.com/ArchiveBox/ArchiveBox/releases>。
- 定价：免费。
- 对歇存的启示：这是现有产品里最接近 [SPEC §2](../spec/library.md#2-资料库目录真相源)「目录即真相、索引可重建」的实现，三条做法可以直接吸收：(1) 每个快照目录自带 `index.json`，目录本身可脱离数据库被识别——对应歇存 `manifest.json`；(2) rescan 把「数据库里没有的目录」当作新发现而不是错误，冲突报告不覆盖——`xiecun reindex` 应照此定义（[SPEC §11](../BUILD.md#3-现在做到哪)、验收 5）；(3) 多抽取器并存写同一目录——对应歇存 `raw/` + `derived/` 分层。它也说明了代价：无头浏览器、WARC、yt-dlp 一起上，运行时很重，这是歇存把重计算放 sidecar（[ADR-001](../decisions/ADR-004-rust-tauri.md)）的理由。

<a id="singlefile"></a>
### SingleFile

- 是什么：把完整网页存成单个 HTML 文件的浏览器扩展（Chrome、Firefox 含移动版、Edge、Safari 等），"a faithful copy"；另有命令行版 single-file-cli。来源：<https://github.com/gildas-lormeau/SingleFile>、<https://github.com/gildas-lormeau/single-file-cli>。
- 数据模型与存储：产物就是一个自包含 HTML，默认落到浏览器下载目录，也可上传 Google Drive / GitHub；auto-save 可在每次页面加载后自动保存；可在保存前高亮、加注、删除内容。
- 平台内容：所见即所存，登录后的页面也能存（扩展运行在用户自己的浏览器会话里）；这是对抖音/小红书/公众号网页版最不依赖接口的保存方式。
- 搜索：无。
- AI 与 Agent：无；但被 linkding、ArchiveBox、Karakeep（0.33.2 起 CLI 可上传 SingleFile 归档）用作快照格式。来源：<https://linkding.link/archiving/>、<https://github.com/karakeep-app/karakeep/releases/tag/v0.33.2>。
- 许可证：AGPL-3.0（作者另提供商业授权）。活跃度：SingleFile v1.31.5 与 single-file-cli v2.19.0 均发布于 2026-10-19，约 22.6k stars。
- 定价：免费。
- 对歇存的启示：`raw/<snapshotId>/page.html` 直接采用 SingleFile 格式，可与 linkding / ArchiveBox / Karakeep 互通；歇存浏览器扩展（[PRD §5 收集入口](../PRD.md#收集入口)）可以就是「SingleFile → 本地 API」，不必自写页面序列化。

<a id="obsidian-web-clipper"></a>
### Obsidian Web Clipper

- 是什么：Obsidian 官方浏览器扩展，"Highlight and capture the web"，"saves content locally to your Obsidian vault"，"Your data is not collected, and we do not gather any usage metrics"。来源：<https://obsidian.md/help/web-clipper>、<https://github.com/obsidianmd/obsidian-clipper>。
- 数据模型与存储：写入 vault 的本地 Markdown 文件（官方帮助页未明写格式，仓库为 Markdown 剪藏）；模板 / 变量 / 过滤器 / 条件逻辑按站点定制属性。背后理念是 Steph Ango 的 "File over app"："if you want to create digital artifacts that last, they must be files you can control"，"Apps are ephemeral, but your files have a chance to last"，开发者应 "give people ownership over their data"。来源：<https://stephango.com/file-over-app>。
- 平台内容：当前页面抽取，公众号网页可剪（待核实效果）；抖音/小红书的视频内容不在能力范围。
- 搜索：依赖 Obsidian 本体搜索。
- AI 与 Agent：Interpreter 用自然语言提示处理页面，支持 Anthropic / OpenAI / Gemini / Azure / DeepSeek / Ollama 本地等，"Interpreter requests are sent directly to the provider you choose"，"Obsidian does not gather or store any data about your requests"，用户自备 API key。Obsidian 本体无官方 MCP，社区 MCP 多经 Local REST API 插件（待核实现状）。来源：<https://obsidian.md/help/web-clipper/interpreter>。
- 许可证：Clipper MIT；Obsidian 本体闭源，个人与商用均免费（"Free without limits"），Sync $4/月、Publish $8/月、Catalyst $25 一次性、商业许可 $50/人/年。来源：<https://obsidian.md/pricing>。活跃度：Clipper 1.7.1 发布于 2026-07-22，仓库 2026-09-22 推送。
- 对歇存的启示：(1) 「文件归用户」在 Obsidian 生态是第一原则而非功能点，歇存 `content.md` 带 frontmatter 的设计（[SPEC §4](../spec/library.md#4-contentmd)）正是为了让资料库在 Obsidian 里直接可读；(2) Interpreter 的「BYO key、请求直达 provider、不经我方服务器」与 [PRD §4.4](../PRD.md#4-原则)「标明数据是否离开本机」是同一件事，歇存可以借用它的措辞。

<a id="zotero"></a>
### Zotero

- 是什么：非营利组织 Digital Scholar 的文献管理软件，收集、整理、注释、引用。来源：<https://www.zotero.org/support/licensing>。
- 数据模型与存储：本地数据目录：`zotero.sqlite`（"the majority of your data: item metadata, notes, tags"）+ `storage/` 下 8 字符子目录放附件（PDF、网页快照）+ `fulltext.sqlite`（全文索引，"is not backed up, since it can be regenerated"）+ 每日自动 `zotero.sqlite.bak`。备份方式是关掉 Zotero 后复制整个目录；官方明确说不要用 RDF/BibTeX/RIS 导出当备份。来源：<https://www.zotero.org/support/zotero_data>。附件是用户文件，元数据在 SQLite——「文件 + 数据库」混合，索引可重建但主数据不是文件。
- 平台内容：网页快照由浏览器 Connector 抓当前页；不针对中文平台。
- 搜索：全文索引（PDF、HTML 快照）。
- AI 与 Agent：Zotero 7 起提供本地 HTTP API（`http://localhost:23119/api/users/0/items`），需开启「允许本机其他应用与 Zotero 通信」；Zotero 10.0 加入 "Local API write support"；只在桌面 GUI 运行时可用，无 headless 模式（论坛有功能请求）。社区 MCP `54yyyu/zotero-mcp` 约 5.5k stars，v0.15.1 发布于 2026-10-16。来源：<https://groups.google.com/g/zotero-dev/c/ElvHhIFAXrY/m/fA7SKKwsAgAJ>、<https://www.zotero.org/support/changelog>、<https://forums.zotero.org/discussion/132184/>、<https://github.com/54yyyu/zotero-mcp>。
- 许可证：AGPL-3.0。活跃度：Zotero 10.0.6 发布于 2026-10-07（Zotero 8 于 2026-01-22 发布，之后 9、10 在同年推出）。来源：<https://www.zotero.org/support/changelog>、<https://forums.zotero.org/discussion/comment/508488>。
- 定价：软件免费；云存储 300 MB 免费、2 GB $20/年、6 GB $60/年、无限 $120/年。来源：<https://www.zotero.org/storage>。
- 对歇存的启示：(1) 「全文索引不备份、可重建；主数据与附件必须整目录备份」是对 [SPEC §2](../spec/library.md#2-资料库目录真相源) `.xiecun/index/` 可删与 `items/` 不可删的直接佐证；(2) 本地 API 绑定 GUI 进程是长期痛点，歇存 `xiecun daemon` 独立于桌面壳、`xiecun mcp` 在 daemon 不在时内嵌运行（[SPEC §9](../BUILD.md#1-用户定下的做法)）避开了这个坑。

<a id="eagle"></a>
### Eagle

- 是什么：面向设计师的本地素材管理器（图片、视频、字体、PDF、网页书签等任意格式），Windows / macOS，一次性买断。来源：<https://en.eagle.cool/>。
- 数据模型与存储：资料库是本地文件夹：`images/<id>.info/` 每条一个子目录，内含原文件与 `metadata.json`（id、name、tags、url、annotation、palettes、时间戳等），第三方可直接读取并发布到网页。官方页面未描述目录结构，此处据第三方实践文章。来源：<https://ivomynttinen.com/blog/publishing-eagle-app-library-on-the-web-part-1/>、<https://medium.com/@sifemalakai/enhancing-your-digital-asset-management-with-eagle-cool-app-batch-import-web-bookmarks-7e1b44fc44c1>。
- 平台内容：浏览器扩展批量存图、拖拽、截图；网页书签；不处理视频平台正文。
- 搜索：关键词、颜色、筛选、以图搜图；4.0 加 AI Search（视觉、语义、跨源）与 AI Action（自动重命名、打标、分类、笔记）。文字 OCR 待核实。来源：<https://en.eagle.cool/>。
- AI 与 Agent：本地 HTTP API `http://localhost:41595/`（Eagle 必须在运行）；2026 年推出 Eagle Skill / MCP 插件（Plugin Center 搜 "MCP" 安装），AI Agent 可扫描标签、建文件夹、合并/删除/翻译标签、改标题描述、从网页收图，执行前 "shows you a confirmation list first"，"Review it, confirm, done"；Eagle 5.0 预告以 AI 整理为主题。来源：<https://api.eagle.cool/>、<https://eagle.cool/blog/post/eagle-plugin-mcp-skill>、<https://en.eagle.cool/blog/post/eagle5-teaser>。
- 许可证：闭源。活跃度：官网显示 v4.0.0 Build 23（2026-04-01）。
- 定价：$34.95 一次性（2024-11-01 起，此前 $29.95），加设备 $17.47，30 天试用，"No subscription needed"。来源：<https://eagle.cool/blog/post/price-adjustment-notice-2024>。
- 对歇存的启示：(1) 商业软件已经验证「每条一个文件夹 + metadata.json」在十万级素材下可用，和 [SPEC §2](../spec/library.md#2-资料库目录真相源) `items/<id>/manifest.json` 同构，歇存可参考其字段命名；(2) MCP 执行前「确认清单」是业界最接近歇存 propose → confirm → apply 的实现，但它没有撤销与日志，歇存多出的 `undo_changeset` 与 journal 是实质差异（[SPEC §10](../BUILD.md#006-agent)）；(3) API 依赖 GUI 运行，同 Zotero。

<a id="anytype"></a>
### Anytype

- 是什么：本地优先、端到端加密的个人/协作知识库，瑞士协会 Any 开发。来源：<https://github.com/anyproto/anytype-ts>。
- 数据模型与存储：本地加密对象库（CRDT，any-sync 同步），文件不是真相源；导出 Markdown 或 Any-Block（Protobuf / JSON），文档注明 "relations are not exported at this time"（可能已过时，社区 beta 有「关系导出为 YAML frontmatter」线程）。来源：<https://doc.anytype.io/anytype-docs/basics/import-export>、<https://github.com/anyproto/any-block>。
- 平台内容：网页剪藏扩展；无中文平台处理。
- 搜索：全局与空间内搜索；语义搜索待核实。
- AI 与 Agent：本地 REST API（默认 `http://127.0.0.1:31009`，API key 在应用内生成，带 `Anytype-Version` 头）；官方 MCP `anyproto/anytype-mcp`（MIT，由 OpenAPI 规范自动转成 MCP 工具，覆盖搜索、空间、对象、列表、属性、类型），v2.0.1 发布于 2026-09-19，约 534 stars；开发者门户 v2 预览称 "objects as editable JSON documents, built for agents and humans"。来源：<https://github.com/anyproto/anytype-mcp>、<https://developers.anytype.io/>。
- 许可证：Any Source Available License 1.0——"only (a) for Non-Commercial Use, or (b) for Commercial Use in Allowed Networks"，源可用但不是 OSI 开源。来源：<https://github.com/anyproto/anytype-ts/blob/main/LICENSE.md>。活跃度：anytype-ts v0.57.4 发布于 2026-10-03。
- 定价：官方定价页为 JS 渲染，本次未抓到；第三方数据互相矛盾（免费档 100 MB 至 1 GB、付费 $5–$19/月不一），待核实。来源：<https://toolradar.com/tools/anytype>、<https://checkthat.ai/brands/anytype/pricing>。
- 对歇存的启示：(1) 「本地 HTTP API → 用 OpenAPI 自动生成 MCP」让 API 与 MCP 永远一致，歇存 [SPEC §9](../BUILD.md#1-用户定下的做法) 的本地 API 与 [§10](../BUILD.md#006-agent) MCP 可以考虑同源生成；(2) 源可用许可是歇存不走的路（[PRD §4.7](../PRD.md#4-原则) AGPL-3.0 + MIT）；(3) 「本地优先但文件只是导出物」与歇存「文件即真相」是两种不同的本地优先，歇存应在 PRD 里把这个区别说清。

<a id="mymind"></a>
### mymind

- 是什么：私密的「第二大脑」收集工具，口号 "Just save it"，"Artificial Intelligence takes care of the rest"。来源：<https://mymind.com/>。
- 数据模型与存储：云端。帮助中心称可把保存的全部内容导出到电脑（仅限最新 Chrome / Edge），不支持导入。来源：<https://mymind.helpscoutdocs.com/article/18-can-i-export>。
- 平台内容：网页、图片、笔记、商品；无中文平台处理。
- 搜索：关键词、颜色、品牌、日期；OCR "lets you search for text within graphics, images & screenshots you save, and it can even read your handwriting"。来源：<https://mymind.helpscoutdocs.com/article/54-how-ocr-text-recognition-works>。
- AI 与 Agent：自动打标、摘要、Smart Spaces 自动归类；未见公开 API 或 MCP（待核实）。
- 许可证：闭源。活跃度：持续运营（定价页在线）。
- 定价：Student of Life $7.99/月或 $79/年；Mastermind $12.99/月或 $129/年；The Bookmarker $4.99/月；"We don't run ads"，"We will never sell your data"。来源：<https://mymind.com/pricing>。
- 注意力设计：这是对象里把「少」说得最直接的："Folders are dead"、"No need to file it, label or even tag it"，宣言明确不做 social features、vanity metrics、social pressure、collaboration、ads，"Nobody can see what you save to your mind"。来源：<https://mymind.com/>。
- 对歇存的启示：(1) 「不要求命名、分类、评价」的收下体验（[PRD §3](../PRD.md) 收下）在它这里已被验证为卖点；(2) 但它用「AI 替你归类」换取「不整理」，歇存选择「engine 标签只建议、不替用户决定」（[PRD §6](../PRD.md#6-不做)、[SPEC §6](../spec/library.md#121-manifestjson-v1)），这是刻意分歧；(3) 云端、无 API、导出受浏览器限制——所有权弱，是歇存不采纳的部分。

<a id="arena"></a>
### Are.na

- 是什么：会员制的收集与联想平台，"a mindful space where you can work through any project over time"；模型是 block（内容）与 channel（集合）多对多。来源：<https://www.are.na/about>。
- 数据模型与存储：云端。所有档位可 "Export channels as PDF, ZIP, and HTML"；API "completely open"，"Use our API to display content elsewhere"。
- 平台内容：链接、图片、文本、文件；无中文平台处理。
- 搜索：站内搜索；全文与 OCR 待核实。
- AI 与 Agent：仅 API；官方立场不做算法推送。
- 许可证：闭源。活跃度：持续运营。
- 定价：Guest 免费（页面写 "Up to 200 total blocks"，同页又写 unlimited blocks，上限待核实）、Premium $7/月或 $70/年、Supporter $120/年。"We ask our members to pay a small subscription"，"We rely on no other external funding"，目标之一是 "promote ad-free businesses and ethical design principles"。来源：<https://www.are.na/about>。
- 对歇存的启示：(1) block 可同时属于多个 channel、没有唯一中心——与 [PRD §5 画布](../PRD.md#组织与找回)「一条素材可属于多个分组，没有唯一中心」同构，Are.na 是这个交互已经跑了十余年的样本；(2) 会员制、无广告、无外部资金是它维持「慢」的方式，歇存不商业化，但「谁付钱决定产品为谁服务」这个推理可以写进 [PRD §1](../PRD.md#1-定位)。

<a id="fabric"></a>
### Fabric

- 是什么：AI 工作区（笔记、画布、书签、文件、AI 会议记录、语音笔记、音视频转写、"intelligent memory"），iOS / Android / 桌面 / CLI / 网页剪藏。来源：<https://fabric.so/pricing-and-plans-for-individuals>。
- 数据模型与存储：云端；导出能力待核实（页面未提）；"Fabric API" 列在附加项。
- 平台内容：通用网页与文件。
- 搜索：AI 搜索与助手（Pro 加助手联网搜索）。
- AI 与 Agent：官方远程 MCP `https://mcp.api.fabric.so/mcp`（OAuth 自动），文档列出的工具以创建为主（bookmarks、folders、memories、notes、tasks），支持 Claude Code / Claude Desktop / Cursor / Codex / VS Code / ChatGPT。来源：<https://fabric.so/guide/ai-tools/MCP-setup>。
- 许可证：闭源。活跃度：持续运营。
- 定价：Plus $8/月（年付 $96）50 GB；Pro $18/月（年付 $216）250 GB；Max $54/月（年付 $648）1 TB；第三方站点列出的价格与官网不一致，以官网为准。来源：<https://fabric.so/pricing-and-plans-for-individuals>。
- 对歇存的启示：「MCP 只给写入口、不给读」是一种收口方式，但方向与歇存相反——歇存给 Agent 的是读与提议，不是直接写（[SPEC §10](../BUILD.md#006-agent)）。

<a id="recall"></a>
### Recall

- 是什么：AI 摘要 + 知识图谱 + 复习的收集工具，支持 YouTube、播客、PDF、文章；域名已从 getrecall.ai 301 到 recall.it。来源：<https://www.recall.it/pricing>。
- 数据模型与存储：云端；导出格式待核实。
- 平台内容：YouTube / 播客有转写摘要；中文平台无。
- 搜索：知识图谱与 AI 对话。
- AI 与 Agent：免费档即含 "API and MCP access"；Plus 加无限摘要、自动知识图谱、"Quiz and spaced repetition to reinforce long-term memory"、"Personal AI chat with your knowledge, the internet, or both"。来源同上。
- 许可证：闭源。活跃度：持续运营。
- 定价：Free $0；Plus $10/月（年付）；Max $38/月（年付）。
- 注意力设计：间隔重复与测验是「再来一遍」机制，是歇存 [PRD §4.5](../PRD.md#4-原则)「没有打卡、连续奖励」的反例。
- 对歇存的启示：把 API 与 MCP 放在免费档，说明「Agent 可读」已是基础能力而非增值项；歇存 M0 就做 MCP（[PRD §8](../PRD.md#8-里程碑)）与此一致。

<a id="cubox"></a>
### Cubox

- 是什么：中国团队（OLCUBO）的「AI 稍后阅读助手」，iOS / iPadOS / macOS / Android / Web / 扩展 / 墨水屏版。来源：<https://cubox.pro/>。
- 数据模型与存储：云端。导出：已解析文章与标注可导出 TXT / Markdown / HTML / PDF，未解析网页只能导出 Webarchive HTML，书签类从「偏好设置 > 导入 & 导出」导出 Netscape HTML；"会员到期后可以随时导出你的数据"；可同步到 Obsidian / Notion / Readwise。导入 Netscape HTML（Safari、Chrome、Edge、Firefox、Pinboard、Pocket、Instapaper）。来源：<https://help.cubox.cc/share/export/>、<https://help.cubox.pro/start/4ad6/>、<https://cubox.pro/pricing/>。
- 平台内容：公众号文章靠微信服务号——把文章转发给「Cubox 收藏助手」，Cubox 解析正文并存快照；另有邮件投递、API、浏览器扩展。小红书 / 抖音：官网未声明支持，推测按通用链接处理，待核实。来源：<https://cubox.pro/>、<https://help.cubox.pro/save/ecf6/>。
- 搜索："全文与语义搜索"；Pro+AI 含 RAG 搜索。来源：<https://cubox.pro/pricing/>。
- AI 与 Agent：AI 摘要、AI 问答、AI 语音、AI 定时报告（每日 8:00 推送到 Cubox 邮箱）。官方 Cubox CLI（`OLCUBO/cubox-cli`，MIT，v1.0.10 发布于 2026-08-27，约 65 stars）配 Skill，面向 Claude Code / Codex / Cursor：批量保存、按文件夹/标签/星标/已读/标注/时间筛选与 RAG 搜索、读取解析后的 Markdown 正文与 AI 解读、读标注、改星标/已读/归档/移动/打标/改标题描述；API key 在应用内生成，文档特别写 "请不要把 Key 粘贴到 AI Agent 对话里"。文档未提 MCP。来源：<https://help.cubox.pro/ai/agents/>、<https://github.com/OLCUBO/cubox-cli>。
- 许可证：闭源（CLI MIT）。活跃度：2026 年持续更新。
- 定价：免费版 200 篇容量、每篇 3 个标注；Pro ¥98/年（无限收藏与标注、快照存档解析、嵌套收藏夹，AI 仅有限试用）；Pro+AI ¥198/年（每月 2000 AI 积分，14 天试用）。来源：<https://cubox.pro/pricing/>。
- 对歇存的启示：(1) 公众号最可靠的入口是「用户在微信里主动转发」，而不是后台接口——歇存没有云账号接不了服务号，等价物是「手机 → 桌面投递」里的文件传输助手兜底（[PRD §5 收集入口](../PRD.md#收集入口)）；(2) 它选择 CLI + Skill 而非 MCP 对接 Agent，且把 key 安全写进文档，歇存 M0 的 stdio MCP 不需要 key，但本地 HTTP API 的 token 文档可照此写（[SPEC §9](../BUILD.md#1-用户定下的做法)）；(3) AI 定时报告、每日推送是「制造注意力」的典型，歇存不做（[PRD §6](../PRD.md#6-不做)）。

<a id="简悦"></a>
### 简悦

- 是什么：简悦 SimpRead，阅读模式 + 剪藏 + 标注 + 稍后读的浏览器扩展，自述 "做你的知识管理粘合剂"。来源：<https://github.com/Kenshin/simpread>、<https://simpread.pro/>。
- 数据模型与存储：扩展本地存储；配套桌面应用「同步助手」（随 2.1.0 发布）可 "导出到文件 本地的任意位置"，支持导出 Markdown / PDF / PNG / Epub / HTML 与离线 HTML，并对接 Notion、语雀、坚果云、WebDAV、Obsidian / Roam / Logseq 等。用户可把稍后读与标注落成本地 Markdown。来源：<https://github.com/Kenshin/simpread>、<https://simpread.pro/>。
- 平台内容：靠站点适配规则做正文抽取；公众号适配情况待核实；不处理视频平台。
- 搜索：扩展内检索稍后读列表；全文与 OCR 无。
- AI 与 Agent：无官方 API / MCP（待核实）。
- 许可证：1.x（至 1.1.4.6205）GPL-3.0 开源，"2.x 为闭源版本，且仅闭源了 标注 与 稍后读 相关功能"。来源：<https://github.com/Kenshin/simpread>。活跃度：最新 release 2.2.0.520 于 2025-07-13，仓库 2026-09-05 推送，约 8.7k stars。
- 定价：1.0 功能永久免费；高级账户一次性买断，官网价格页当前显示占位（"限时折扣"），承诺 "每年不超过 3次 的打折促销"；第三方 2026 年文章称原价约 ¥68、授权 3 台设备、促销低至 ¥8，待核实。来源：<https://simpread.pro/price.html>、<https://bbs.fcgvisa.com/t/simpread-50/57659>。
- 对歇存的启示：中文稍后读里少见的「本地文件夹是目的地」实践；它把 Markdown 导出做成同步而不是一次性动作，和歇存 `content.md` 常驻本地殊途同归。闭源部分恰是标注与稍后读，说明单人项目靠这两项变现——歇存不商业化，不受此约束。

<a id="方片收集"></a>
### 方片收集

- 是什么：长沙快记网络（Quickwis）出品的跨平台快速收集工具，宣传「收集一条灵感仅需 0.13 秒」，支持文字、链接、图片、视频、语音，云同步，可备份到 Evernote。来源：<https://www.appinn.com/fang-pian-shou-ji/>、<https://apps.apple.com/us/app/%E6%96%B9%E7%89%87%E6%94%B6%E9%9B%86/id1111109828>。
- 现状：App Store 最新版本 1.7，日期 2017-02-20；官网 funp.in 的 TLS 证书已指向无关域名（`ecpm.ftzwl.cn`），页面不可达。未找到正式停服公告，判断为事实停更，待核实。
- 数据模型、搜索、AI、许可证：云端、闭源；其余不再适用。
- 定价：免费。
- 对歇存的启示：中文「快速收集」类工具生命周期短，用户数据随服务消失——再次支持 [PRD §4.3](../PRD.md#4-原则)「本地文件是真相源」。

<a id="flomo"></a>
### flomo

- 是什么：浮墨笔记，"全平台的卡片笔记"，口号 "持续不断记录，意义自然浮现"；立场是记录想法而非收藏文章。来源：<https://help.flomoapp.com/>。
- 数据模型与存储：云端；文字存储不计入配额（免费 500M / PRO 20G / MAX 500G 为附件配额）；导出仅 HTML，可全量或按标签 / 搜索词导出。来源：<https://help.flomoapp.com/basic/storage.html>。
- 平台内容：微信输入——关注服务号后可发文字与图片；文档明说 "暂不支持直接转发内容到 flomo"，微信没有把文章转发给服务号的接口，所以公众号文章进不来。来源：<https://help.flomoapp.com/basic/wechat-input.html>。
- 搜索：全文搜索；PRO 有「发现相关笔记」。
- AI 与 Agent：开发者 API（PRO，入站 webhook，文档只描述写入）；URL scheme `flomo://create`；MCP 文档页：把 flomo 连接到豆包 / WorkBuddy / ChatGPT / Claude，"在对话中搜索旧笔记、调用记忆档案或保存讨论结论"。来源：<https://help.flomoapp.com/advance/api.html>、<https://help.flomoapp.com/advance/mcp.html>。
- 许可证：闭源。活跃度：持续运营。
- 定价：PRO 价格 "以 App 内展示为准"（第三方称 ¥99/年，待核实）；PRO 含 API、每日回顾、引用批注、30 天历史版本等。来源：<https://help.flomoapp.com/membership/pro.html>。
- 注意力设计：官方帮助有「为何收藏无用」一节，并借丰田「搬运浪费」比喻说收藏只是把信息从一处挪到另一处、不增值；少数派文章转述其观点："不要像微信和 QQ 收藏那样，让 read it later 变成了 read it never"。但 flomo 自己有「每日回顾」推送。来源：<https://help.flomoapp.com/>、<https://sspai.com/post/66416>。
- 对歇存的启示：flomo 与歇存目标同向（让真正要看的东西变少），手段相反——flomo 不收原文，歇存收原文但不催看。歇存 [PRD §1](../PRD.md#1-定位) 可以直接回应「收藏无用论」：无用的是躺在平台里的收藏，不是本地可搜的文本。

<a id="opencli"></a>
### OpenCLI

- 是什么：[PRD §8 M2](../PRD.md#8-里程碑) 提到的参考项目。"Make Any Website into CLI & Use your logged-in browser by AI agent"，通过 Browser Bridge 扩展 + 本地 daemon 复用用户已登录的 Chrome 会话，把 100+ 站点变成统一命令。来源：<https://github.com/jackwener/opencli>。
- 数据模型与存储：无存储，只输出结构化结果（`--format json|table|yaml|md|csv`）。
- 平台内容：bilibili 有 `favorite`（读自己的收藏）、`download`；xiaohongshu 有 `search` / `note` / `comments` / `feed` / `user` / `download` 等，没有收藏夹命令；douyin 列为支持站点但 README 未列命令；公众号文章可下载为 Markdown。需要用户在浏览器里已登录。来源同上。
- AI 与 Agent：面向 Agent 设计——`opencli list` 返回分页能力清单，四个 Skill（`opencli-browser`、`opencli-usage` 等）用 `npx skills add jackwener/opencli` 安装；README 未提 MCP。
- 许可证：Apache-2.0。活跃度：v2.0.2 发布于 2026-10-20，约 30.5k stars。
- 定价：免费。
- 对歇存的启示：(1) 它是 [SPEC §8](../protocol/README.md) 适配器 `method: browser` 与 `capabilities: [list]` 的现成上游，M2「收藏列表适配器」可以是一个很薄的 OpenCLI 包装；(2) 「复用登录态、不导出 Cookie」是中文社区对边界的通行表达，歇存 `adapter.yaml` 的 `requires.cookie` 字段应能区分「none / 浏览器会话 / 用户粘贴」三档；(3) 平台命令覆盖参差（B 站有收藏、小红书没有），印证 [PRD §9](../PRD.md#9-风险)「平台接口随时失效」，适配器必须按能力声明而非按平台承诺。

<a id="中文平台个人备份工具"></a>
### 中文平台个人备份工具

这一组不是收藏产品，而是歇存社区适配器可能依赖或借鉴的上游。内核不内置其中任何一个（[PRD §4.1](../PRD.md#4-原则)、[§6](../PRD.md#6-不做)）。

- XHS-Downloader（JoeanAmier）：小红书作品信息采集与下载，用户脚本可从推荐、个人主页、收藏、点赞、专辑、搜索结果提取链接；Cookie 为非强制项（未设则视频只有低分辨率）；输出 txt / md 与 SQLite（`ExploreData.db`）；内置请求延时，"启用该功能可能会被小红书检测为自动化操作"；提供 API 模式（端口 5556）与 MCP 模式（`/mcp/`）。GPL-3.0，2.8 发布于 2026-09-12，约 13k stars。来源：<https://github.com/JoeanAmier/XHS-Downloader>。
- DouK-Downloader（原 TikTokDownloader，JoeanAmier）：抖音 / TikTok 账号作品、喜欢、收藏、收藏夹采集与下载；需要 Cookie；CSV / XLSX / SQLite；Web API 模式（端口 5555）；README 注明抖音加密参数算法不再维护，失效需用户自行提供生成代码。GPL-3.0，5.8 发布于 2026-09-13，约 17k stars。来源：<https://github.com/JoeanAmier/TikTokDownloader>。
- 抖喜欢（Chrome 扩展，开发者 Twitter Kept）："把抖音点赞、收藏和作者主页作品备份到本机文件夹"，mp4 / JPEG，可断点续传与离线浏览本地库；"没有开发者服务器，没有账号，没有云同步"。v12.34，更新于 2026-10-19，84 用户。来源：<https://chromewebstore.google.com/detail/fohpobbbmekjomojghfiblikkfeoogbm>。同类另有「抖珍藏」（v2.8.37，2026-04-15，第三方商店镜像）。来源：<https://chromeboard.com/extension/抖珍藏-下载备份你爱过的所有抖音-ldllbcnpihgljdgpjgdopgeejenhnabf>。
- wechat-article-exporter：公众号文章批量下载，导出 html / json / excel / txt / md / docx，HTML 版打包图片与样式。README 声明 "本项目已停止维护（2026-07-30）"：它依赖公众号后台「写文章时搜索其他公众号文章」的接口，该接口已被官方关闭，代码层面无法恢复；在线站域名 2026-10-30 到期。MIT，约 13k stars。来源：<https://github.com/wechat-article/wechat-article-exporter>。
- 个人收藏夹导出 Skill（逸尘，X 帖 2026-07-29）：把 X / 抖音 / 小红书收藏导出整合为一个 Skill，做法是复用已登录的 Chrome 会话、滚动收藏页、从 DOM 提取链接去重、不导出 Cookie。来源：<https://x.com/gengdaJ/status/2082277952700788914>。
- MediaCrawler 等多平台采集工具偏向评论与数据分析，不是个人收藏备份，这里不展开。来源：<https://cloud.tencent.com/developer/article/2550627>。

对歇存的启示：(1) 这一组全部依赖登录态或 Cookie、全部带请求延时与风控提示、公众号一条已因上游接口关闭而终止——[PRD §9](../PRD.md#9-风险) 的风险不是假设，适配器协议里的 `requires.cookie`、`limits`、`health` 字段（[SPEC §8](../protocol/README.md)）和同步「遇风控自动退避并告知」（[PRD §5](../PRD.md#5-功能范围)）都对得上实际；(2) XHS-Downloader 已提供 MCP 模式，说明社区适配器以 `method: mcp` 对接是可行的；(3) 「抖喜欢」式的浏览器扩展（本地文件夹、无服务器）是最贴近歇存原则的形态，M1「抖音最难」的参考适配器可以先做扩展路线而非接口路线（[PRD §8](../PRD.md#8-里程碑)）。

<a id="slax-reader-与-linkwarden"></a>
### Slax Reader 与 Linkwarden

搜索中遇到的两个相邻开源项目，简要记录。

- Slax Reader："Open-source, AI-powered read-it-later — save web pages, highlight, and discuss"，永久快照、AI 大纲与摘要、高亮与分享讨论；API 跑在 Cloudflare Workers 上，monorepo 含 web、扩展、API、CLI（`@slax-lab/reader-cli`）。Apache-2.0，cli-v1.0.1 发布于 2026-10-14，约 83 stars。来源：<https://github.com/slax-lab/slax-reader>。
- Linkwarden：自托管协作书签，"collect, read, annotate, and fully preserve what matters"。AGPL-3.0，v2.16.3 发布于 2026-09-09，约 20.2k stars。来源：<https://github.com/linkwarden/linkwarden>。

<a id="2-对比表"></a>
## 2. 对比表

「存储与所有权」一列：**文件** = 用户目录里的可读文件是真相源；**库** = 自托管数据库 + 资产目录；**云** = 厂商服务器，导出是补救。活跃度为最新 release 或可见更新日期。

| 产品 | 存储与所有权 | 平台内容 | 搜索 | AI·Agent | 许可证 | 活跃度 |
|---|---|---|---|---|---|---|
| [Karakeep](#karakeep) | 库：SQLite + 资产目录（或 S3）；Meilisearch 外部索引；导出 Netscape HTML | 通用无头浏览器；无中文平台 | Meilisearch 全文 + 语义 | LLM 打标摘要；REST；CLI；MCP 21–29 工具含删除；官方 Skill | AGPL-3.0 | v0.34.0 · 2026-10-17 |
| [Readwise Reader](#readwise-reader) | 云；整库 ZIP 含正文、CSV、OPML、Obsidian 插件 | 网页/PDF/EPUB/YouTube/X/RSS | 全文（可离线） | Ghostreader；API v3；官方 MCP 22 工具，无删文档 | 闭源 | MCP 脱 beta 2026-05 |
| [Raindrop.io](#raindropio) | 云；HTML/CSV/TXT；Pro 每日备份不含上传文件 | 通用；YouTube 口播可搜 | Pro 全文 + 语义 | Pro：AI 助手、官方 MCP（删除两阶段） | 闭源 | 博客 2026 更新 |
| [Pinboard](#pinboard) | 云；API 可拉全部书签，归档副本不经 API | 通用 | 归档账户全文 | 仅 API | 闭源 | 博客 2025-01-19 |
| [Wallabag](#wallabag) | 库：MySQL/PostgreSQL/SQLite；导出 EPUB/PDF/TXT/CSV/JSON/XML | 通用正文抽取 | 文章搜索（全文待核实） | REST API；无 AI | MIT | 2.6.14 · 2025-10-07（仓库 2026-10 仍推送） |
| [Linkding](#linkding) | 库：SQLite；SingleFile 快照为 asset；Netscape HTML | 通用 | 标题/描述/笔记/标签 | REST API | MIT | v1.47.0 · 2026-09-13 |
| [Omnivore](#omnivore) | 库：PostgreSQL；导出 ZIP（contents/*.html + metadata_*.json） | 通用 | 全文（云版） | 云 API 已关 | AGPL-3.0 | 云关停 2024-11；仓库未归档，Android 0.230.0 · 2026-10-14 |
| [Instapaper](#instapaper) | 云；CSV/HTML 仅链接 | 通用 | Premium 全文 | API（Full API 需申请） | 闭源 | 大版本 2026-07 |
| [Pocket](#pocket) | 云，已删除；导出曾为 CSV ZIP | 通用 | — | — | 闭源 | 2025-07-08 关停 |
| [ArchiveBox](#archivebox) | **文件**：目录即数据，index.sqlite3 为索引，rescan 可发现未入库目录 | 通用；yt-dlp 媒体 | ripgrep / Sonic / FTS5 | REST API alpha；无 MCP | MIT | v0.9.73 · 2026-10-06 |
| [SingleFile](#singlefile) | **文件**：单 HTML | 所见即所存，含登录后页面 | — | — | AGPL-3.0 | v1.31.5 · 2026-10-19 |
| [Obsidian Web Clipper](#obsidian-web-clipper) | **文件**：vault 内 Markdown | 当前页抽取 | 依赖 Obsidian | Interpreter（BYO key，直达 provider） | MIT（Obsidian 闭源免费） | 1.7.1 · 2026-07-22 |
| [Zotero](#zotero) | 文件 + 库：storage/ 附件 + zotero.sqlite；fulltext.sqlite 可重建 | 浏览器快照 | 全文（PDF/HTML） | 本地 API 23119（10.0 可写）；社区 MCP 5.5k★ | AGPL-3.0 | 10.0.6 · 2026-10-07 |
| [Eagle](#eagle) | **文件**：images/<id>.info/metadata.json | 扩展存图、书签 | 关键词/颜色/以图搜图/AI 语义 | 本地 API 41595；官方 MCP/Skill 带确认清单 | 闭源 | 4.0.0 · 2026-04 |
| [Anytype](#anytype) | 本地加密对象库；导出 Markdown / Any-Block | 网页剪藏 | 全局搜索 | 本地 API 31009；官方 MCP（OpenAPI 生成） | Any Source Available 1.0 | v0.57.4 · 2026-10-03 |
| [mymind](#mymind) | 云；可导出到电脑（限 Chrome/Edge） | 通用 | 关键词/颜色/OCR | 自动打标摘要；无公开 API | 闭源 | 运营中 |
| [Are.na](#arena) | 云；channel 导出 PDF/ZIP/HTML；开放 API | 通用 | 站内搜索 | API | 闭源 | 运营中 |
| [Fabric](#fabric) | 云；导出待核实 | 通用、音视频转写 | AI 搜索 | Fabric API；官方 MCP 偏写入 | 闭源 | 运营中 |
| [Recall](#recall) | 云；导出待核实 | YouTube/播客/PDF | 知识图谱 + 对话 | 免费档含 API 与 MCP | 闭源 | 运营中（recall.it） |
| [Cubox](#cubox) | 云；TXT/MD/HTML/PDF、Netscape HTML；到期可导出 | 公众号经服务号转发解析 + 快照；小红书/抖音待核实 | 全文 + 语义（RAG） | AI 摘要问答；官方 CLI + Skill；文档未提 MCP | 闭源（CLI MIT） | CLI v1.0.10 · 2026-08-27 |
| [简悦](#简悦) | 扩展本地 + 同步助手导出到任意本地目录（MD/PDF/HTML…） | 站点适配正文抽取 | 列表检索 | 无 | 1.x GPL-3.0；2.x 部分闭源 | 2.2.0.520 · 2025-07-13 |
| [方片收集](#方片收集) | 云（服务疑似停摆） | 文字/图/视频 | — | — | 闭源 | iOS 1.7 · 2017-02-20 |
| [flomo](#flomo) | 云；导出 HTML | 微信输入限文字图片；公众号文章进不来 | 全文 | 入站 API（PRO）；MCP 搜索与保存 | 闭源 | 运营中 |
| [OpenCLI](#opencli) | 无存储，结构化输出 | B 站收藏列表、小红书单条、公众号转 MD | — | Agent 优先：list + Skills | Apache-2.0 | v2.0.2 · 2026-10-20 |
| [XHS-Downloader](#中文平台个人备份工具) | 本地文件 + SQLite | 小红书收藏/点赞链接提取与作品下载 | — | API 模式；MCP 模式 | GPL-3.0 | 2.8 · 2026-09-12 |
| [DouK-Downloader](#中文平台个人备份工具) | 本地文件 + CSV/XLSX/SQLite | 抖音作品/喜欢/收藏/收藏夹 | — | Web API | GPL-3.0 | 5.8 · 2026-09-13 |
| [抖喜欢](#中文平台个人备份工具) | **文件**：本机文件夹，无服务器 | 抖音点赞/收藏/作者作品 | 本地库浏览 | — | 闭源扩展 | 12.34 · 2026-10-19 |
| [wechat-article-exporter](#中文平台个人备份工具) | 本地导出 html/json/md/docx | 公众号批量 | — | — | MIT | 停止维护 2026-07-30 |

<a id="3-三个重点问题"></a>
## 3. 三个重点问题

<a id="31-谁把文件归用户所有索引可重建做成架构原则"></a>
### 3.1 谁把「文件归用户所有、索引可重建」做成架构原则

分四档：

1. 目录即数据，索引可丢：**ArchiveBox**（`index.sqlite3` 只是索引，快照目录自带 `index.json`，复制目录后 rescan 恢复）、**Eagle**（`<id>.info/metadata.json`，第三方可直接读目录）、**Obsidian 生态**（"File over app"，Clipper 直接写 Markdown）、**SingleFile**（产物就是文件）。中文工具里只有**简悦同步助手**和**抖喜欢**接近这一档。
2. 文件 + 数据库混合，索引可重建但主数据不是文件：**Zotero**（`storage/` 是文件，`zotero.sqlite` 是主数据，`fulltext.sqlite` 明确「不备份、可重建」）。
3. 自托管数据库：**Karakeep**（SQLite + 资产目录，搜索依赖外部 Meilisearch，未配置则禁用）、**Linkding**、**Wallabag**、**Omnivore**。用户拥有服务器，但离开应用无法直接读内容。
4. 云端 + 导出补救：Readwise（导出最完整，含正文 ZIP）、Raindrop（备份不含上传文件）、Pinboard（归档副本拿不出来）、Instapaper（只给链接）、Cubox、flomo（仅 HTML）、mymind、Are.na、Fabric、Recall、Anytype（本地优先但文件是导出物）。

没有一个对象同时满足歇存 [SPEC §2](../spec/library.md#2-资料库目录真相源) 的三条：每条一个可读目录、raw 与 derived 分层、索引目录可整体删除后全量重建。最近的是 ArchiveBox（有目录、有多抽取器产物、有 rescan，但 rescan 不是「全量重建」的默认路径，且无 OCR/ASR 层）。

<a id="32-谁在做减少注意力而不是收藏更多"></a>
### 3.2 谁在做「减少注意力」而不是「收藏更多」

明确以「少」为卖点的：

- **mymind**："Folders are dead"；不做 social、vanity metrics、social pressure、collaboration、ads；"Nobody can see what you save"。表达方式是一串否定句。
- **Pinboard**："Social Bookmarking for Introverts"；"no ads and no trackers of any kind"；文本、高密度、快。
- **Are.na**："a mindful space"；会员付费、不靠外部资金、推广无广告业务。
- **flomo**：直接写「为何收藏无用」，用「搬运浪费」比喻反对收藏本身。
- **Obsidian**：无指标、无账号、本地文件。

反方向、在制造处理队列的：**Readwise Reader**（inbox / triage / Daily Review）、**Recall**（quiz、间隔重复）、**Raindrop**（Reminders）、**Cubox**（AI 定时报告每日推送）、**flomo**（每日回顾，与其反收藏立场并存）、Pocket（推荐流）。

两点观察：(1) 「少」在这些产品里都停在收下与整理阶段，对象中未见把「放下 / 删除」做成一等功能的，歇存 [PRD §3](../PRD.md) 的「放下」与 [§5 生命周期](../PRD.md#生命周期)的瘦身 → 回收站 → 清除是空白地带；(2) 「不整理」在 mymind 是用 AI 替用户归类换来的，在歇存是「engine 标签只建议」换来的——同一个用户体验，两种相反的信任模型。

<a id="33-谁有-agent-可读接口"></a>
### 3.3 谁有 Agent 可读接口

| 产品 | 接口形态 | 读 | 写 | 删 | 确认 / 撤销 | 不可信标记 |
|---|---|---|---|---|---|---|
| Karakeep | MCP（npx/Docker）+ REST + CLI + Skill | 搜索、取正文、取资产 | 书签/列表/标签 CRUD | `delete-bookmark` 等 | 无 | 无 |
| Readwise | 远程 MCP + REST v3 | 搜索、取 Markdown 全文 | 建文档、改元数据、移位置 | 删高亮；文档只能归档 | 提示词建议「先问我」 | 无 |
| Raindrop | 远程 MCP（OAuth 2.1）+ REST | find、fetch full content | CRUD | 删除两阶段（Trash → 永久） | 无 | 无 |
| Anytype | 官方 MCP（OpenAPI 生成）+ 本地 REST | 搜索、对象 | CRUD | 有 | 无 | 无 |
| Zotero | 本地 REST（GUI 运行时）+ 社区 MCP | 条目、全文 | 10.0 起可写 | 待核实 | 无 | 无 |
| Eagle | 本地 REST + 官方 MCP/Skill 插件 | 标签、文件 | 建文件夹、改标签元数据、收图 | 删标签 | 执行前确认清单；无撤销 | 无 |
| Cubox | 官方 CLI + Skill | 搜索（含 RAG）、读 Markdown 正文与标注 | 保存、改状态/文件夹/标签 | 未列 | 无 | 无 |
| Fabric | 远程 MCP | 未列 | 建书签/笔记/任务等 | 未列 | 无 | 无 |
| Recall | API + MCP（免费档） | 待核实 | 待核实 | 待核实 | 无 | 无 |
| flomo | 入站 API + MCP | 搜索旧笔记 | 保存 | 无 | 无 | 无 |
| Obsidian | Clipper Interpreter（BYO key）；社区 MCP | — | 写 Markdown | — | — | 无 |
| OpenCLI | CLI + Skills | 平台数据 | 平台操作 | — | — | 无 |
| XHS-Downloader | API + MCP 模式 | 作品信息 | 下载 | — | — | 无 |
| Pinboard / Wallabag / Linkding / ArchiveBox | REST | 是 | 是 | 是 | 无 | 无 |

三个共同缺口，正是歇存 [SPEC §10](../BUILD.md#006-agent) 的差异点（以本次查阅的官方文档为限，未实测 MCP 响应）：(1) 没有文档提到给返回内容打「不可信 / 来源」信封；(2) 未见「提议 → 确认 → 应用 → 整批撤销」的变更集，最接近的 Eagle 只有执行前确认；(3) 删除普遍直接暴露给 Agent，只有 Readwise MCP（文档不可删，REST API 仍可删）和 Raindrop（两阶段）做了收口。另一个趋势：Karakeep、Cubox、OpenCLI 都在 MCP 之外（或之前）用 **CLI + SKILL.md** 对接 Coding Agent，分发走 skills.sh。

<a id="4-对歇存的启示"></a>
## 4. 对歇存的启示

每条标明影响的 PRD / SPEC 节。

1. **导入格式清单可以现在定下**：Pocket CSV（ZIP）、Omnivore ZIP（`metadata_*.json` + `contents/*.html` + 高亮 Markdown）、Netscape HTML（浏览器、Raindrop、Linkding、Karakeep、Cubox、Pinboard）、Readwise CSV 与「Full Files and Articles」ZIP、Instapaper CSV、Wallabag JSON、wechat-article-exporter 的 html/json/md。其中 Omnivore ZIP 和 Readwise ZIP 带正文，导入时完整度可直接到 `partial` 或 `full`；其余只有链接，停在 `link`。→ [PRD §5 导入方式](../PRD.md#导入方式)、[SPEC §3](../spec/library.md#3-manifestjson) `completeness`。
2. **raw 快照采用 SingleFile HTML**：linkding、ArchiveBox、Karakeep 都收这个格式，歇存浏览器扩展可以就是 SingleFile → 本地 API，不自写序列化。→ [SPEC §2](../spec/library.md#2-资料库目录真相源) `raw/`、[§8](../protocol/README.md) `outputs: raw-html`、[PRD §5 收集入口](../PRD.md#收集入口)。
3. **`xiecun reindex` 按 ArchiveBox rescan 的语义定义**：遍历 `items/`，把索引里没有的目录当新发现，身份冲突报告不覆盖，索引目录可整体删除。→ [SPEC §11](../BUILD.md#3-现在做到哪)、[§12](../BUILD.md#6-依赖顺序) 验收 5。
4. **manifest.json 对照 Eagle metadata.json 补齐只读字段**：Eagle 的 `btime / mtime / palettes / annotation` 说明「每条一个 JSON」在十万级规模可用；歇存已有的字段够用，不需要加，但可以把「目录名创建后不变」写成与 Eagle 相同的保证。→ [SPEC §2](../spec/library.md#2-资料库目录真相源)、[§3](../spec/library.md#3-manifestjson)。
5. **MCP 不提供 delete 的决定保持**：Karakeep 直接给 `delete-bookmark`，Raindrop 做两阶段，Readwise MCP 让文档只能归档（其 REST API 仍有删除）——业界在收口但没收完。歇存更进一步：Agent 没有删除，用户删除也先进 `.trash/` 30 天。→ [SPEC §10](../BUILD.md#006-agent)、[§2](../spec/library.md#2-资料库目录真相源) `.trash/`、[PRD §5 Agent](../PRD.md#agent)。
6. **propose / apply / undo 是独有的，值得当卖点写**：Eagle 的确认清单是最接近的，但没有 journal 和整批撤销。→ [PRD §4 原则 6](../PRD.md#4-原则)、[SPEC §10](../BUILD.md#006-agent)。
7. **M0 就写一份 SKILL.md，和 MCP 并行**：Karakeep CLI 捆绑官方 Skill、Cubox 用 CLI + Skill 不做 MCP、OpenCLI 用 `npx skills add` 分发。对 Claude Code / Codex 用户，Skill 比 MCP 配置更轻。→ [PRD §4 原则 6](../PRD.md#4-原则)、[PRD §8 M0](../PRD.md#8-里程碑)、[SPEC §9](../BUILD.md#1-用户定下的做法)。
8. **本地 API 与 MCP 同源生成**：Anytype 用 OpenAPI 规范自动生成 MCP 工具，保证两者一致。歇存 [SPEC §9](../BUILD.md#1-用户定下的做法) 本地 HTTP API 与 [§10](../BUILD.md#006-agent) MCP 若共用一份契约定义，可避免双写。→ [SPEC §9](../BUILD.md#1-用户定下的做法)、[§10](../BUILD.md#006-agent)；可写成 ADR。
9. **daemon 独立于 GUI 的决定保持**：Zotero、Eagle、Anytype 的本地 API 都要求桌面应用在前台运行，是用户抱怨最多的点之一。→ [SPEC §9](../BUILD.md#1-用户定下的做法) `xiecun daemon` / `xiecun mcp` 内嵌回退、[ADR-001](../decisions/ADR-004-rust-tauri.md)。
10. **搜索不依赖外部服务**：Karakeep「`MEILI_ADDR` 未设则搜索禁用」是反例；bleve 内嵌正确。→ [ADR-001](../decisions/ADR-004-rust-tauri.md)、[SPEC §11](../BUILD.md#3-现在做到哪)。
11. **公众号的「最简」参考适配器定位为「从已打开页面抽取」**：wechat-article-exporter 因上游后台接口被关而终止，Cubox 的入口是用户在微信里主动转发，OpenCLI 的公众号能力是「下载当前文章为 Markdown」。接口路线不可靠，所见即所存最稳。→ [PRD §8 M1](../PRD.md#8-里程碑)、[SPEC §8](../protocol/README.md)。
12. **抖音参考适配器先走浏览器扩展路线**：现有工具（XHS-Downloader、DouK-Downloader）全部依赖 Cookie、带请求延时与风控提示，且抖音参数算法「不再维护」；「抖喜欢」式扩展（本机文件夹、无服务器）更贴近歇存原则。→ [PRD §8 M1](../PRD.md#8-里程碑)、[PRD §9](../PRD.md#9-风险)、[SPEC §8](../protocol/README.md) `requires.cookie` 应区分「none / 浏览器会话 / 用户粘贴」。
13. **OpenCLI 作为 M2 收藏列表适配器的上游**：B 站 `favorite` 已有；小红书、抖音的收藏列表命令缺失，适配器须按能力声明而非按平台承诺。→ [PRD §8 M2](../PRD.md#8-里程碑)、[SPEC §8](../protocol/README.md) `capabilities`。
14. **「放下」是空白地带**：对象中未见把删除 / 瘦身做成一等功能的。[PRD §3](../PRD.md) 的「放下」与 [§5 生命周期](../PRD.md#生命周期)可以在 M3 之前就在 CLI 里做出来（`xiecun slim` / `xiecun trash`），作为差异点。→ [PRD §5 生命周期](../PRD.md#生命周期)。
15. **文案可借 mymind / Pinboard / flomo 的否定句式**：[PRD §6](../PRD.md#6-不做)「不做」已是这种写法；对外页面可以把 Pocket（2025-07）、Omnivore（2024-11）、方片收集（2017 后停更）作为「平台和服务都会消失」的三个例子。→ [PRD §1](../PRD.md#1-定位)、[PRD §6](../PRD.md#6-不做)。
16. **对「收藏无用论」的回应**：flomo 说收藏是搬运、不增值。歇存的回答是：无用的是躺在平台里、搜不到、随时下架的收藏；变成本地可搜文本并能交给 Agent 之后，它才有用，而且不用你再看一遍。→ [PRD §1](../PRD.md#1-定位)。
17. **迁出通道**：歇存「文件即导出」，但别的工具不认 `manifest.json`。建议 M2 之后给 CLI 加 Netscape HTML / CSV 导出（这不违反 [SPEC §10](../BUILD.md#006-agent)「MCP 不提供 export」，那条约束的是 Agent 外发）。→ 建议新增到 [PRD §5](../PRD.md#5-功能范围)，待用户决定。
18. **成功标准里的「保险赔付率」有现实对照**：Pocket 用户在 2025-11 之后、Omnivore 用户在 2024-11 之后需要的正是这个数字。→ [PRD §7](../PRD.md#7-成功标准)。

<a id="5-歇存刻意不做的"></a>
## 5. 歇存刻意不做的

对照 [PRD §6](../PRD.md#6-不做) 与 [PRD §4](../PRD.md#4-原则)，列出对象里普遍有、歇存明确不做的东西：

| 不做 | 对象里的例子 | 依据 |
|---|---|---|
| 云账号、同步、协作 | Readwise、Raindrop、Cubox、mymind、flomo、Are.na、Fabric、Recall、Karakeep Cloud | [PRD §6](../PRD.md#6-不做) |
| 推荐与发现流 | Pinboard popular/network、Are.na explore、Pocket 推荐、Readwise feeds | [PRD §6](../PRD.md#6-不做) |
| 待办、提醒、复习、推送 | Raindrop Reminders、Recall 间隔重复与 quiz、Readwise Daily Review、Cubox AI 定时报告、flomo 每日回顾 | [PRD §4.5](../PRD.md#4-原则)、[§6](../PRD.md#6-不做) |
| 内置平台抓取代码 | XHS-Downloader、DouK-Downloader、wechat-article-exporter、OpenCLI 的平台命令 | [PRD §4.1](../PRD.md#4-原则)、[§6](../PRD.md#6-不做) |
| 自动分类替用户决定 | mymind Smart Spaces、Karakeep 自动打标直接生效、Eagle AI Action 自动归档 | [PRD §6](../PRD.md#6-不做)；[SPEC §6](../spec/library.md#121-manifestjson-v1) engine 标签只建议 |
| 搜索依赖外部服务 | Karakeep Meilisearch、Omnivore 自托管栈 | [ADR-001](../decisions/ADR-004-rust-tauri.md) |
| Agent 直接删除与外发 | Karakeep MCP `delete-*`、Raindrop MCP delete、Anytype MCP CRUD、Readwise 删高亮 | [SPEC §10](../BUILD.md#006-agent) |
| 应用内自建聊天助手 | Cubox AI 问答、Readwise Ghostreader、Recall chat、Fabric assistant | [PRD §4.6](../PRD.md#4-原则) |
| 源可用但非开源的许可 | Anytype ASAL 1.0、简悦 2.x 部分闭源 | [PRD §4.7](../PRD.md#4-原则) |
| 「未整理」状态与未读数 | Readwise inbox、Cubox 免费版容量提示 | [PRD §4.5](../PRD.md#4-原则) |

<a id="6-待核实清单"></a>
## 6. 待核实清单

- Karakeep：MCP 工具数（文档 21 vs 发布说明 29）；是否对图片做 OCR 文字抽取；Karakeep Cloud 定价（第三方 $4/月）。
- Readwise Reader：公众号网页解析效果。
- Raindrop：Pro 年价（第三方 $28）；永久副本能否导出；图片 OCR。
- Pinboard：归档账户全文搜索范围。
- Wallabag：全文索引实现；Omnivore 导入加入的版本。
- Linkding：快照内容是否可搜。
- Omnivore：自托管是否需 Elasticsearch；omnivore.work 分叉站点是否存在。
- Instapaper：Full API 细节（官方页未抓到正文）；2018 年独立一事。
- Pocket：最终删除日 2025-11-12（仅二手来源；官方确认到 2025-11-06）。
- Eagle：文字 OCR；MCP 插件发布日期。
- Anytype：官方定价；relations 导出现状；语义搜索。
- mymind：是否有任何 API。
- Are.na：全文与 OCR。
- Fabric、Recall：导出格式。
- Cubox：小红书 / 抖音链接的处理方式。
- 简悦：公众号适配；高级账户现价（第三方 ¥68）；是否有 API。
- 方片收集：是否有正式停服公告。
- flomo：PRO 年价（第三方 ¥99）。
- Zotero：社区 MCP 是否暴露删除。
- Obsidian：社区 MCP 现状。
