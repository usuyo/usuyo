<a id="调研-02--社区维护的来源适配器生态"></a>
# 调研 02 — 社区维护的来源适配器生态

状态：草案 · 2026-10-21

目的：看别人怎么让"来源"由社区维护、以独立进程接入宿主，提炼歇存 [SPEC §8 适配器协议 v0](../SPEC.md#8-适配器协议-v0) 可以借鉴的模式。事实均附来源 URL；凭记忆或只有二手来源的地方标「待核实」，汇总在 [§16](#16-待核实清单)。仓库活跃度数据来自 GitHub API（`gh api repos/<owner>/<repo>`，2026-10-21 取样），引用时给仓库主页 URL。

目录：[§0 缺口清单](#0-范围与缺口清单) · [§1 Tachiyomi/Mihon](#1-tachiyomi-与-mihon-扩展生态) · [§2 yt-dlp](#2-yt-dlp) · [§3 gallery-dl](#3-gallery-dl) · [§4 you-get](#4-you-get) · [§5 lux](#5-lux) · [§6 BBDown](#6-bbdown) · [§7 RSSHub](#7-rsshub-与-rsshub-radar) · [§8 Cobalt](#8-cobalt) · [§9 Karakeep](#9-karakeep-抓取器) · [§10 Obsidian Web Clipper](#10-obsidian-web-clipper-模板) · [§11 opencli](#11-opencli) · [§12 进程插件系统](#12-独立进程加-json-协议的插件系统) · [§13 其他模式](#13-其他相关模式) · [§14 对比表](#14-对比表) · [§15 启示](#15-对歇存适配器协议的启示) · [§16 待核实](#16-待核实清单)

<a id="0-范围与缺口清单"></a>
## 0. 范围与缺口清单

调研对象是任务指定的清单；每个对象按同一组问题写：如何声明一个来源、如何分发与安装、宿主与来源之间的调用协议、cookie 与登录态如何传递、测试与健康检查、法律与治理、许可证、活跃度。[§13](#13-其他相关模式) 补了四个任务未点名但形态高度相关的系统，写得短。

读 [SPEC §8](../SPEC.md#8-适配器协议-v0) 与 `pkg/contract/adapter.go`、`pkg/contract/schema/adapter.schema.json` 后，整理出下面这些缺口，后文每条启示都对应其中一项：

| 编号 | 缺口 | 现状 |
|---|---|---|
| G1 | `adapter.yaml` 没有 `version`，但 [SPEC §3](../SPEC.md#3-manifestjson) 的 `snapshots[].adapterVersion` 要填一个值 | 未定义 |
| G2 | 没有 `name` / `description` / `maintainers` / `homepage` / 语言 / 内容分级 / 优先级 | 未定义 |
| G3 | `match` 的语义：是否含 scheme、host 与 path 怎么比、多个适配器同时命中的次序、与 `builtin/web-generic` 的 `*` 如何共存 | 只说"URL glob patterns" |
| G4 | `method: cli` 时 `<bin>` 怎么找到；`local-service` / `mcp` / `http-api` / `browser` 四种 method 没有调用约定 | 未定义 |
| G5 | `upstream.install` 自由形状（Go 注释原话："Free-form until SPEC §8 fixes its shape"）；没有运行时依赖（node / python / ffmpeg / JS runtime）声明 | 未定义 |
| G6 | `requires.cookie` / `requires.proxy` 是裸字符串，没有取值集合；cookie 怎么传给进程没写 | 未定义 |
| G7 | `list <source> --cursor <c>` 的 `source` 形式、输出形状、游标结束条件；`watch` 完全未定义；`health` 的输出未定义；`canonicalize` 对"不是我的 URL"如何回答 | 未定义 |
| G8 | 没有错误信封、退出码、错误分类（not-found / login-required / rate-limited / upstream-changed）、stderr 的角色、超时、取消信号 | 未定义 |
| G9 | `--out <dir>` 是绝对路径还是相对 cwd、谁创建、适配器写的 `raw/` 与内核的 `raw/<snapshotId>/` 如何对应、失败时谁清理；`evidence.status` 只适用 HTTP | 部分定义 |
| G10 | 协议版本与兼容性协商；适配器源的信任与签名（PRD 原则 1 与风险一节要求适配器外置） | 未定义 |

<a id="1-tachiyomi-与-mihon-扩展生态"></a>
## 1. Tachiyomi 与 Mihon 扩展生态

PRD 把它当作模板，所以写得最细。三个主体：应用 Mihon（Tachiyomi 的社区 fork，2024-01-15 建仓，见下文"法律与治理"）、扩展源码仓库 keiyoushi/extensions-source、扩展发布仓库 keiyoushi/extensions。

**如何声明一个来源**

- 一个扩展是一个 Android APK，一个 APK 里可以含多个 source。源码仓库的 CONTRIBUTING 用 Gradle DSL 声明：`name`（站点名，非英文需罗马化）、`versionCode`（正整数，"incremented with any change to the code"）、`contentWarning`（必须显式写 `ContentWarning.SAFE` / `MIXED` / `NSFW`，取代旧的 `isNsfw`）、`libVersion`（现为 `"1.6"`，source 类必须实现 `KeiSource`）、`theme`（继承 `lib-multisrc/` 里的多站主题，如 `"madara"`）、`pkgName`、`source {}`（含 `lang`、`baseUrl`，KSP 代码生成）、`deeplink {}`（URL intent filter）。来源：<https://github.com/keiyoushi/extensions-source/blob/main/CONTRIBUTING.md>
- 版本名由 `libVersion` 与版本码拼接自动生成，例如 `1.6.1`；用主题时 `versionCode = theme.baseVersionCode + versionCode`。来源：<https://github.com/keiyoushi/extensions-source/blob/main/CONTRIBUTING.md>。
- source API（`KeiSource`）必须实现：`getPopularManga(page)`、`getLatestUpdates(page)`、`getSearchMangaList(page, query, filters)`、`fetchMangaUpdate(...)`、`getPageList(chapter)`；可选 `getMangaByUrl(url)`、`getMangasByUrl(url, page)`（深链接 → 条目，相当于歇存的 `canonicalize`）、`configureClient()`、`configureHeaders()`、`fetchFilterData()`。`lang`、`versionId` 在 `source {}` 块里声明而不是类上。来源：<https://github.com/keiyoushi/extensions-source/blob/main/CONTRIBUTING.md>。
- 应用侧读取 APK 时看的元数据键：`tachiyomi.extension.class`、`tachiyomi.extension.factory`、`tachiyomi.extension.nsfw`（旧）、`tachiyomix.name`、`tachiyomix.extensionLib`、`tachiyomix.contentWarning`（新）。支持的库版本写死为 `SUPPORTED_LIB_VERSIONS = listOf(1.4, 1.6)`，不在集合内直接拒载（`Reason.UnsupportedLibVersion`）。来源：<https://github.com/mihonapp/mihon/blob/main/app/src/main/java/eu/kanade/tachiyomi/extension/util/ExtensionLoader.kt>
- Mihon v0.21.0（2026-10-11）宣布下一个版本将移除 1.4 扩展兼容，"to reduce maintenance burden"。来源：<https://github.com/mihonapp/mihon/releases/tag/v0.21.0>

**如何分发与安装**

- 发布仓库 keiyoushi/extensions 的 `repo` 分支是纯产物分支：`index.json`、`index.min.json`、`index.pb`、`repo.json`、`release-assets.json`、`icon/`。用户加的 URL 是 `https://github.com/keiyoushi/extensions/raw/repo/index.pb`，或点 `mihon://extension-store?url=...` 深链接。来源：<https://github.com/keiyoushi/extensions> 、<https://keiyoushi.github.io/docs/guides/getting-started>
- 2024 年起 Mihon 要求 repo 旁放 `repo.json`（官方 X 账号公告，2024-01-31，待核实原文）。现在 Keiyoushi 的 `repo.json` 实际内容：`{"index_v2": ".../index.pb", "meta": {"name": "Keiyoushi", "website": "https://keiyoushi.github.io", "signingKeyFingerprint": "9add655a..."}}`。来源：<https://raw.githubusercontent.com/keiyoushi/extensions/repo/repo.json> 、<https://x.com/mihonapp/status/1753142567796084809>
- 应用把旧偏好里的 `.../index.min.json` 去掉后缀拼 `/repo.json` 做迁移（`TrustExtensionRepositoryMigration`）。来源：<https://github.com/mihonapp/mihon/blob/main/app/src/main/java/mihon/core/migration/migrations/TrustExtensionRepositoryMigration.kt>
- 新索引是 protobuf（v0.20.0 "Added support for tachiyomix 1.6 extensions and index format"，2026-06-27）。应用模型的字段（`@ProtoNumber`）：store 级 `name`、`badgeLabel`、`signingKey`、`contact{website, discord}`、`extensionList` 或 `extensionListUrl`（索引可以只给一个指向列表的 URL，列表与元数据分离）；extension 级 `name`、`packageName`、`resources{apkUrl, iconUrl}`、`extensionLib`、`versionCode`、`versionName`、`contentWarning`、`sources[]`；source 级 `id`、`name`、`language`、`homeUrl`、`mirrorUrls`、`message`（给用户看的提示文本）。来源：<https://github.com/mihonapp/mihon/blob/main/data/src/main/java/mihon/data/extension/model/NetworkExtensionStore.kt> 、<https://github.com/mihonapp/mihon/releases/tag/v0.20.0>
- 旧 JSON 索引每项字段：`name`、`pkg`、`apk`、`lang`、`code`、`version`、`nsfw`、`sources[{name, lang, id, baseUrl}]`。2026-10-21 实测 Keiyoushi 的 `index.min.json` 只剩 2 条哨兵条目："Outdated App"（`baseUrl` 指向文档站）与 "Update to Mihon 0.20.1+"——用假扩展告诉旧版应用"你该升级了"。来源：<https://raw.githubusercontent.com/keiyoushi/extensions/repo/index.min.json>
- `release-assets.json` 给每个包 `apk` 与 `jar` 的文件名和 `sha256`。来源：<https://raw.githubusercontent.com/keiyoushi/extensions/repo/release-assets.json>
- 安装门槛：Keiyoushi 文档写 "Requires Mihon 0.20.1 or newer"，并要求先卸载所有旧扩展，否则拿不到更新。来源：<https://keiyoushi.github.io/docs/guides/getting-started>
- 签名与信任：加载时 `check(!extensionSignatures.isNullOrEmpty()) { "The extension isn't signed" }`；未签名 → `Reason.Unsigned`；签名不在任何已添加 store 的 `signingKey` 内且用户没手动信任 → `Reason.Untrusted(signatures.last())`；扩展归属哪个 store 由 `stores.firstOrNull { it.signingKey in signatures }` 决定。来源：<https://github.com/mihonapp/mihon/blob/main/app/src/main/java/eu/kanade/tachiyomi/extension/util/ExtensionLoader.kt>
- v0.21.0："Extensions now only install and update from the repo they come from, to keep you safe from tampered extensions"；扩展详情显示来源 repo 与网站链接；加载失败的扩展"now tell you why, and can still be updated"；NSFW 开关换成三档内容偏好（Safe / Mixed / 18+）。来源：<https://github.com/mihonapp/mihon/releases/tag/v0.21.0>
- v0.20.0 把"extension repo"改叫"extension store"，"obsolete"改叫"orphaned"（store 里已没有的已装扩展）。来源：<https://github.com/mihonapp/mihon/releases/tag/v0.20.0>

**调用协议**

- 进程内：扩展是 APK 内的 Kotlin 类，由应用动态加载后直接调用接口方法，不是子进程。因此没有 stdout/JSON 一层；超时、重试、Cloudflare 页面处理都在应用的 OkHttp 层统一做（CONTRIBUTING："Do not manually check for Cloudflare challenges... The app handles this before calling the parser"），限速用 `rateLimit(permits, period, interval, shouldLimit)` 在 `configureClient()` 里声明。来源：<https://github.com/keiyoushi/extensions-source/blob/main/CONTRIBUTING.md>

**登录态**

- 扩展用 `ConfigurableSource` + `SharedPreferences` 存自己的设置；cookie 用 `addCookie` 加到应用的 cookie jar，"Do not manually set the Cookie header"；User-Agent 走应用默认。来源：<https://github.com/keiyoushi/extensions-source/blob/main/CONTRIBUTING.md>。
- 需要用户交互的站点由应用层 WebView 承担，源代码只拿结果。来源：<https://github.com/keiyoushi/extensions-source/blob/main/CONTRIBUTING.md>。

**测试与健康**

- 没有强制的自动化站点测试；文档只给"本地调试"（注释掉 `loadAllIndividualExtensions()`、用 `loadIndividualExtension("<lang>", "<name>")`）和网络抓包建议。来源：<https://github.com/keiyoushi/extensions-source/blob/main/CONTRIBUTING.md>。
- 健康靠 CI 构建徽章（build_push.yml）、issue 模板（source request / bug）、以及 Discord。来源：<https://github.com/keiyoushi/extensions-source>
- 应用侧有"orphaned"状态与"加载失败原因"展示。来源：<https://github.com/mihonapp/mihon/releases/tag/v0.21.0>

**法律与治理**

- 2024-01-09 Tachiyomi 官网公告 "Extensions list removed"："we have made the decision to purge our list of extensions"，转向 "bring-your-own-content model"，只保留连接 Komga 之类个人媒体库的少数扩展。来源：<https://github.com/tachiyomiorg/website/blob/main/website/src/news/2024-01-09-extensions-removal.md>
- 2024-01-13 "Goodbye"：主项目停止开发，原因是 "Kakao Entertainment Corp's threats to both myself and others"；"Our core GitHub repositories will be taken down"。该文现以 tachiyomiorg 组织 README 形式存档。来源：<https://github.com/tachiyomiorg>
- 仓库状态：tachiyomiorg/extensions 已 archived，最后推送 2024-01-08；tachiyomiorg/tachiyomi 已不存在（API 404）。keiyoushi/extensions 创建于 2024-01-08（同一天），Mihon 仓库创建于 2024-01-15。来源：<https://github.com/tachiyomiorg/extensions> 、<https://github.com/keiyoushi/extensions> 、<https://github.com/mihonapp/mihon>
- 二手报道称 Kakao 的反盗版部门先要求下架扩展、团队照办后仍被追责；待核实。来源：<https://alternativeto.net/news/2024/1/manga-reader-app-tachiyomi-ceases-development-amid-legal-threats-from-kakao-entertainment>
- Mihon 的立场写进 FAQ："Mihon does not have, associate, or provide any repositories or extensions"，并警告第三方扩展 "will have full access to the app and may contain malware"。来源：<https://mihon.app/docs/faq/browse/extensions>
- Keiyoushi 的 README 声明 "This project is not affiliated with Mihon/Tachiyomi"，源请求"volunteer-based"、不保证时限。来源：<https://github.com/keiyoushi/extensions-source>
- 下架处理是一条置顶 issue "Removed Extensions (Legal / Maintenance Reasons)"（2026-03-24 开，2026-09-16 更新）："Extensions that were DMCA'd will not be added back and there is no support from us"；列出 2026 年 FAKKU 发出 DMCA 后移除的 6 个源、2024 年 ReComics 移除的 1 个源，以及因维护负担移除的 2 个源。来源：<https://github.com/keiyoushi/extensions-source/issues/14260>
- GitHub 公开 DMCA 仓库 2024–2026 年目录里没有以 tachiyomi / keiyoushi / mihon 命名的通知文件（扫描 `github/dmca` 各月目录文件名）；说明压力主要走私下渠道而非 GitHub DMCA 流程。来源：<https://github.com/github/dmca>

**许可证与活跃度**

- Mihon：Apache-2.0；v0.21.2 发布于 2026-10-13；约 24.6k stars。来源：<https://github.com/mihonapp/mihon>
- keiyoushi/extensions-source：Apache-2.0（版权头 "Copyright 2015 Javier Tomás"）；最后推送 2026-10-20；约 4.8k stars。keiyoushi/extensions：无许可证文件（纯产物）；约 15.3k stars。来源：<https://github.com/keiyoushi/extensions-source> 、<https://github.com/keiyoushi/extensions>

<a id="2-yt-dlp"></a>
## 2. yt-dlp

**如何声明一个来源**

- 一个 extractor 是 `yt_dlp/extractor/<site>.py` 里继承 `InfoExtractor` 的类，类名以 `IE` 结尾；`_VALID_URL` 是带命名组 `id` 的正则；注册方式是往 `yt_dlp/extractor/_extractors.py` 加一行 import。来源：<https://github.com/yt-dlp/yt-dlp/blob/master/CONTRIBUTING.md>
- 基类上可声明的属性：`_VALID_URL`、`IE_DESC`、`_WORKING`（标记已知失效）、`_ENABLED`、`_NETRC_MACHINE`、`SEARCH_KEY`、`_EMBED_REGEX`、`_GEO_COUNTRIES`；方法 `suitable(url)`、`_match_valid_url`、`_match_id`、`working()`、`_perform_login(username, password)`、`_real_initialize`、`_real_extract`、`raise_login_required`、`raise_geo_restricted`、`raise_no_formats`、`_get_cookies(url)`、`get_testcases()`。来源：<https://github.com/yt-dlp/yt-dlp/blob/master/yt_dlp/extractor/common.py>
- 输出契约："Only `id` and either `url` or `formats` are mandatory"，其他字段必须非致命地提取；成人站点必须返回 `age_limit`。来源：<https://github.com/yt-dlp/yt-dlp/blob/master/CONTRIBUTING.md>
- 接收前的法律门槛：新站点 PR 前 "make sure this site is not dedicated to copyright infringement"，否则 "pull requests adding support for them will be rejected"；沿用 youtube-dl 的政策不支持主要用于侵权的服务，也 "cannot support any service that serves only DRM protected content"。来源：<https://github.com/yt-dlp/yt-dlp/blob/master/CONTRIBUTING.md>

**如何分发与安装**

- 单一程序，三个发布渠道：`stable`（"(mostly) monthly schedule"）、`nightly`（有改动的每天，"recommended channel for regular users"）、`master`；`--update-to [CHANNEL]@[TAG]` 可切换渠道或升降级，`--update-to owner/repo` 可指向任意仓库但 "there is no verification done for binaries from different repositories"。来源：<https://github.com/yt-dlp/yt-dlp/blob/master/README.md>
- 2026 年 stable 发布日：03-17、06-09、07-04、08-19；GitHub API 单页 100 条 release 覆盖 2021-09-02 至 2026-08-19。来源：<https://github.com/yt-dlp/yt-dlp/releases>
- 插件：从命名空间包 `yt_dlp_plugins.extractor` / `yt_dlp_plugins.postprocessor` 加载；搜索位置包括 `${XDG_CONFIG_HOME}/yt-dlp/plugins/<package name>/yt_dlp_plugins/`、`~/.yt-dlp/plugins/...`、`/etc/yt-dlp/plugins/...`、可执行文件旁的 `yt-dlp-plugins/`、`PYTHONPATH`（pip 安装）；支持 `.zip` / `.egg` / `.whl`；`--plugin-dirs DIR` 追加、`--no-plugin-dirs` 清空、环境变量 `YTDLP_NO_PLUGINS` 禁用。"Extractor plugins take priority over built-in extractors"；"there are no checks performed on plugin code. Use plugins at your own risk"。来源：<https://github.com/yt-dlp/yt-dlp/blob/master/README.md>
- 插件模板仓库 yt-dlp/yt-dlp-sample-plugins；元数据在 `setup.cfg`（name、version），建议每次改动都升版本；发现靠 GitHub topic `yt-dlp-plugins`。来源：<https://github.com/yt-dlp/yt-dlp/wiki/Plugin-Development> 、<https://github.com/yt-dlp/yt-dlp/wiki/Plugins>
- 运行时依赖：YouTube 现在需要外部 JavaScript runtime（EJS 方案，脚本来自 yt-dlp/ejs）。Deno 推荐且默认启用（最低 2.3.0），Node 22.0.0+ 用 `--js-runtimes node`，QuickJS 用 `--js-runtimes quickjs`，Bun 已弃用；可执行文件需在 `PATH` 或用 `--js-runtimes "deno:/path/to/deno"` 指定；解题脚本随二进制打包，或 `--remote-components ejs:npm|ejs:github` 远程拉取（默认不拉）。wiki 最后编辑 2026-07-12。来源：<https://github.com/yt-dlp/yt-dlp/wiki/EJS>

**调用协议**

- 命令行 + 可选 JSON：`-j, --dump-json`、`-J, --dump-single-json`（每个视频/播放列表一个 JSON）、`--print "%()j"`、`--load-info-json FILE` 回放；错误控制 `-i, --ignore-errors`、`--abort-on-error`、`--no-abort-on-error`、`--ignore-no-formats-error`；网络 `--socket-timeout SECONDS`、`-R, --retries`（默认 10）、`--extractor-retries`、`--sleep-requests`、`--impersonate CLIENT[:OS]`；`--extractor-args IE_KEY:ARGS` 给单个 extractor 传参。来源：<https://github.com/yt-dlp/yt-dlp/blob/master/README.md>
- 退出码在 README 里没有列表（待核实）。

**登录态**

- `--cookies FILE`（Netscape 格式，读写同一文件）；`--cookies-from-browser BROWSER[+KEYRING][:PROFILE][::CONTAINER]`，支持 brave / chrome / chromium / edge / firefox / opera / safari / vivaldi / whale，Linux 上 Chromium 系需指定 keyring（basictext / gnomekeyring / kwallet / kwallet5 / kwallet6）；`--netrc` / `--netrc-location` 按 extractor 的 `_NETRC_MACHINE` 读账号。来源：<https://github.com/yt-dlp/yt-dlp/blob/master/README.md>

**测试与健康**

- 每个 extractor 必须带 `_TESTS`：`url`、`md5`（前 10241 字节校验）、`info_dict`（`id` 与 `ext` 必填，其他值可写精确值、`md5:`、`re:`、`count:` 或 Python 类型）、`skip`（无法自动测时写原因）；`hatch test YourExtractor` 或 `python -m devscripts.run_tests` 运行，多个用例编号 `YourExtractor_1`、`_all`。来源：<https://github.com/yt-dlp/yt-dlp/blob/master/CONTRIBUTING.md>
- `_WORKING = False` 标记已知失效的 extractor，运行时提示用户。来源：<https://github.com/yt-dlp/yt-dlp/blob/master/yt_dlp/extractor/common.py>
- 对抗站点变化的编码规范：`.get()` 而非 `[]`、`traverse_obj`、`fatal=False`、多来源回退、宽松正则、`int_or_none` 等安全转换。来源：<https://github.com/yt-dlp/yt-dlp/blob/master/CONTRIBUTING.md>

**法律与治理**

- 前身 youtube-dl 于 2020 年被 GitHub 依 DMCA §1201 通知下架，2020-11-16 恢复："showed the project was not circumventing a technical protection measure (TPM)"；GitHub 同时改流程：每个 1201 通知先做技术与法律审查，"In the case where the claim is ambiguous, we will err on the side of the developer"，下架前先联系仓库主，并 "establish and donate $1M to a developer defense fund"。来源：<https://github.blog/news-insights/policy-news-and-insights/standing-up-for-developers-youtube-dl-is-back/>
- yt-dlp 自身的治理手段是入口审查（上面的"法律门槛"）与不接受特定站点。来源：<https://github.com/yt-dlp/yt-dlp/blob/master/CONTRIBUTING.md>
- 2024–2026 年 GitHub 公开 DMCA 目录中没有以 yt-dlp / youtube-dl 命名的通知文件。来源：<https://github.com/github/dmca>

**许可证与活跃度**

- Unlicense（yt-dlp 与 youtube-dl 相同）；yt-dlp 约 199.8k stars，最后推送 2026-10-10；youtube-dl 仍有推送（2026-02-19）。来源：<https://github.com/yt-dlp/yt-dlp> 、<https://github.com/ytdl-org/youtube-dl>

<a id="3-gallery-dl"></a>
## 3. gallery-dl

**如何声明一个来源**

- 一个 extractor 是继承 `Extractor` 的类，类属性：`category`、`subcategory`、`basecategory`、`pattern`（正则，首次用到时编译）、`directory_fmt`、`filename_fmt`、`archive_fmt`、`root`、`cookies_domain`、`cookies_index`、`referer`、`useragent`、`browser`、`request_interval` / `request_interval_min`、`tls12`、`ciphers`；子类有 `GalleryExtractor`、`ChapterExtractor`、`MangaExtractor`、`Dispatch`（用户页 → 派生多个子 URL）。来源：<https://github.com/mikf/gallery-dl/blob/master/gallery_dl/extractor/common.py>
- 匹配：`extractor.find(url)` 按 `modules` 列表顺序遍历已缓存的类，`cls.pattern.match(url)` 第一个命中即返回实例；外部 extractor 目录由配置 `extractor.module-sources`（目录列表，`.py` 文件被 import 并搜索带 `pattern` 属性的类，`null` 代表内置模块）加载。来源：<https://github.com/mikf/gallery-dl/blob/master/gallery_dl/extractor/__init__.py> 、<https://github.com/mikf/gallery-dl/blob/master/docs/configuration.rst>

**如何分发与安装**

- 单一程序（pip / 独立可执行文件）；`-U, --update`、`--update-to CHANNEL[@TAG]`（stable / dev）。约每周一个版本：v1.32.14 2026-09-27、v1.32.15 10-03、v1.32.16 10-09、v1.32.17 10-17。来源：<https://github.com/mikf/gallery-dl/blob/master/docs/options.md> 、<https://github.com/mikf/gallery-dl/releases>
- 支持站点文档由脚本从 extractor 类与测试结果生成（`scripts/supportedsites.py`，"Generate a Markdown document listing all supported sites"，读取 `test.results`）。来源：<https://github.com/mikf/gallery-dl/blob/master/scripts/supportedsites.py>

**调用协议**

- `-j, --dump-json`、`-J, --resolve-json`、`-s, --simulate`、`-E, --extractor-info`、`-K, --list-keywords`、`--list-extractors [CATEGORIES]`、`--list-modules`；`-R, --retries N`、`--http-timeout SECONDS`（默认 30.0）、`--sleep-request`、`--sleep-429`（收到 429 时等多久）、`-A, --abort N`（连续 N 个已存在文件即停）、`-T, --terminate N`；`--download-archive FILE` 记录已下载；`--write-metadata`、`--write-info-json`；`-o KEY=VALUE` 临时配置，`-c` / `--config-yaml` / `--config-toml`。来源：<https://github.com/mikf/gallery-dl/blob/master/docs/options.md>

**登录态**

- `-C, --cookies FILE`；`--cookies-from-browser BROWSER[/DOMAIN][+KEYRING][:PROFILE][::CONTAINER]`；配置项 `extractor.*.cookies` 可以是 cookies.txt 路径、`{name: value}` 对象、或最多 5 项的浏览器 profile 列表（浏览器名、profile、keyring、Firefox container、域名）；`extractor.*.cookies-select` 轮换多个 cookie 源；`extractor.*.cookies-update` 回写。基类有 `cookies_check(cookies_names, domain)`、`cookies_load`、`login()`。来源：<https://github.com/mikf/gallery-dl/blob/master/docs/configuration.rst> 、<https://github.com/mikf/gallery-dl/blob/master/gallery_dl/extractor/common.py>

**测试与健康**

- 测试样例集中在 `test/results/<site>.py` 的 `__tests__` 元组：每条含 `#url`、`#class`、可选 `#options`、`#results`（期望产出的 URL 列表）、`#sha1_url`、`#count`，以及期望元数据键值。来源：<https://github.com/mikf/gallery-dl/blob/master/test/results/pixiv.py>

**法律与治理**

- 未检索到公开的下架事件；GitHub 公开 DMCA 仓库 2024–2026 年目录中没有以 gallery-dl 命名的通知文件（只按文件名扫描）。来源：<https://github.com/github/dmca>

**许可证与活跃度**

- GPL-2.0；约 20.2k stars；最后推送 2026-10-17。来源：<https://github.com/mikf/gallery-dl>

<a id="4-you-get"></a>
## 4. you-get

- **声明**：`src/you_get/common.py` 里一张 `SITES` 字典（101 条，域名关键字 → 模块名，如 `'bilibili': 'bilibili'`、`'cctv': 'cntv'`），`url_to_module(url)` 据此路由；不在表内的站点交给 universal extractor（"For all other sites not on the list, the universal extractor will take care of finding and downloading interesting resources from the page"）。来源：<https://github.com/soimort/you-get/blob/develop/src/you_get/common.py> 、<https://github.com/soimort/you-get/blob/develop/README.md>
- **分发**：pip 或源码；没有插件机制。来源：<https://github.com/soimort/you-get/blob/develop/README.md>
- **调用**：`--json` 输出提取结果摘要；`-u, --url` 只列资源 URL；`-i, --info`；`-o` / `-O` 指定目录与文件名；`-x, --http-proxy`、`-y, --extractor-proxy`（只在提取阶段用的代理）、`--no-proxy`。来源：<https://github.com/soimort/you-get/blob/develop/README.md>
- **登录态**：`-c, --cookies` 读 Mozilla `cookies.sqlite` 或 Netscape `cookies.txt`。来源：<https://github.com/soimort/you-get/blob/develop/README.md>
- **测试健康**：仓库有 `tests/`，但是否对每个站点有样例未核实（待核实）。
- **治理**：网页检索未找到下架事件（待核实）。
- **许可证与活跃度**：MIT（`LICENSE.txt`："Copyright (c) 2012-2024 Mort Yao"）；最后一个 release v0.4.1743 发布于 2025-01-04，之后 2026-10-06 仍有推送；约 56.9k stars。来源：<https://github.com/soimort/you-get> 、<https://github.com/soimort/you-get/releases>

<a id="5-lux"></a>
## 5. lux

- **声明**：Go 接口 `Extractor { Extract(url string, option Options) ([]*Data, error) }`；每个站点包在 `init` 里 `extractors.Register(domain, New())` 注册到 `extractorMap`；路由按 `utils.Domain(u.Host)` 取二级域名，少数特判（`haokan.baidu.com` → `haokan`，`xhslink.com` → `xiaohongshu`，`av|BV|ep` 开头的 bilibili 短 ID 先补全 URL），找不到用 `extractorMap[""]` 的通用 extractor。来源：<https://github.com/iawia002/lux/blob/master/extractors/extractors.go>
- **输出结构**：`Data{URL, Site, Title, Type(video|image|audio), Streams{ID, Quality, Parts[{URL, Size, Ext}], Size, Ext, NeedMux}, Captions（弹幕、字幕）, Err}`；`Options{Playlist, Items, ItemStart, ItemEnd, ThreadNumber, Cookie, EpisodeTitleOnly, Youku*}`。来源：<https://github.com/iawia002/lux/blob/master/extractors/types.go>
- **分发**：单一二进制；没有插件机制。来源：<https://github.com/iawia002/lux/blob/master/README.md>
- **调用**：`-j` 打印提取数据 JSON；`-p` 播放列表；`-m` 多线程；`-retry`（README 正文说默认 100，`--help` 文本说默认 10，两处不一致）。来源：<https://github.com/iawia002/lux/blob/master/README.md>
- **登录态**：`-c` 接受 `name=value; ...` 字符串或 Netscape cookie 文件。来源：<https://github.com/iawia002/lux/blob/master/README.md>
- **测试健康**：各 extractor 包内有 Go 测试（待核实覆盖面）。
- **治理**：网页检索未找到下架事件（待核实）。
- **许可证与活跃度**：MIT；最后 release v0.24.1 发布于 2024-05-06；最后推送 2026-03-29；约 31.8k stars。活跃度低。来源：<https://github.com/iawia002/lux> 、<https://github.com/iawia002/lux/releases>

<a id="6-bbdown"></a>
## 6. BBDown

- **声明**：只做 bilibili，没有"来源"抽象；C# 单体。来源：<https://github.com/nilaoda/BBDown/blob/master/README.md>
- **分发**：GitHub Releases 二进制；最后 release 1.6.3 发布于 2024-08-14。来源：<https://github.com/nilaoda/BBDown/releases>
- **调用**：`-info, --only-show-info` 只解析不下载；解析模式 `-tv` / `-app` / `-intl`（TV / APP / 国际版接口）；`--work-dir`、`--ffmpeg-path`、`--aria2c-path`（混流和下载依赖外部程序 ffmpeg / mp4box / aria2c）；`--skip-subtitle` / `--skip-cover` / `--skip-ai`；`--save-archives-to-file` 记录已下载；`serve -l http://0.0.0.0:12450` 以 HTTP JSON API 模式运行（`json-api-doc.md`）。来源：<https://github.com/nilaoda/BBDown/blob/master/README.md>
- **登录态**：`login` / `logintv` 子命令用 APP 扫二维码登录，`-c, --cookie <cookie>` 手动传字符串，`-token` 传 TV/APP access_token；路线图里"自动刷新 cookie"未完成。来源：<https://github.com/nilaoda/BBDown/blob/master/README.md>
- **测试健康**：未见站点级自动化测试（待核实）。
- **治理**：README 顶部声明"本项目仅供个人学习、研究和非商业性用途……仅在有合法授权的情况下使用相关内容"。仓库于 2026-05-14 被所有者归档（GitHub API `archived: true`），作者未公开说明原因；二手文章把它与 2026 年多个 B 站下载工具收到律师函联系起来，但时间线对不上，待核实。来源：<https://github.com/nilaoda/BBDown> 、<https://noobclaw.com/cn/blog/bilibili-video-download-tool-2026/>
- **许可证与活跃度**：MIT；约 13.9k stars；已归档。来源：<https://github.com/nilaoda/BBDown>

<a id="7-rsshub-与-rsshub-radar"></a>
## 7. RSSHub 与 RSSHub-Radar

RSSHub 是"社区维护、每站一组路由、宿主以 HTTP 服务形式运行"的样本，也是 `list` / `watch` 能力最接近的参照。

**如何声明一个来源**

- 目录 `lib/routes/<namespace>/`，每个站点一个 `namespace.ts` 导出 `Namespace {name, url?, description?, lang?, categories?, zh?, ja?}`；每条路由一个文件导出 `Route`。`Route` 字段（`lib/types.ts`）：`path: string | string[]`、`name`、`url?`、`maintainers: string[]`（GitHub ID）、`handler(ctx)`、`example`、`parameters?: Record<string, string | {description, default?, options?}>`、`description?`、`categories?`、`features?: {requireConfig?: false | [{name, optional?, description}], requirePuppeteer?, antiCrawler?, supportRadar?, supportBT?, supportPodcast?, supportScihub?, nsfw?}`、`radar?: RadarItem[]`、`view?`、`zh?` / `ja?`（本地化覆盖）。来源：<https://github.com/DIYgod/RSSHub/blob/master/lib/types.ts>
- `RadarItem {title?, docs?, source: string[], target?: string | (params, url, document) => string}`：`source` 是不带协议、带 `:param` 的路径模式，匹配结果填进 `target` 路由路径。来源：<https://github.com/DIYgod/RSSHub/blob/master/lib/types.ts>；<https://docs.rsshub.app/joinus/new-rss/start-code>
- 返回数据 `Data {title, description?, link?, item?: DataItem[], allowEmpty?, image?, author?, language?, ttl?...}`；`DataItem {title, description?, pubDate?, link?, category?, author?, guid?, id?, content?{html,text}, image?, updated?, media?, attachments?...}`。来源：<https://github.com/DIYgod/RSSHub/blob/master/lib/types.ts>
- 实例：bilibili 收藏夹路由 `path: '/fav/:uid/:fid/:embed?'`，`parameters` 写明 uid / fid 从哪找，`maintainers: ['Qixingchen']`，handler 用 `got` 调 `api.bilibili.com/x/v3/fav/resource/list?media_id=...&ps=20`，`Cookie: config.bilibili.cookies[uid]`，每条 item 输出 `title / description / pubDate / link / author`。来源：<https://github.com/DIYgod/RSSHub/blob/master/lib/routes/bilibili/fav.ts>
- 2026-10-21 取样：`lib/routes/*/namespace.ts` 共 1971 个。来源：<https://github.com/DIYgod/RSSHub/tree/master/lib/routes>

**如何分发与安装**

- 路由随主仓库发布，没有独立安装源；用户自建实例（Docker / npm）或用公共实例。版本号按日期 + 构建号，几乎每天多次（v2026.10.20-8451.51f74b2 等）。来源：<https://github.com/DIYgod/RSSHub/releases>
- Radar 规则由服务端从各路由的 `radar` 字段聚合，`GET /api/radar/rules` 返回 "All Radar rules grouped by domain"；RSSHub-Radar 浏览器扩展（Chrome / Edge / Firefox / Safari）读这些规则在当前页面上发现可订阅的路由。来源：<https://github.com/DIYgod/RSSHub/blob/master/lib/api/radar/rules/all.ts> 、<https://github.com/DIYgod/RSSHub-Radar>

**调用协议**

- HTTP GET `/<namespace>/<path>`，返回 RSS/Atom/JSON Feed；全局配置 `REQUEST_RETRY`（默认 2）、`REQUEST_TIMEOUT`（默认 3000 ms）、`CACHE_EXPIRE`（路由缓存默认 300 s）、`CACHE_CONTENT_EXPIRE`（内容缓存默认 3600 s）、`PROXY_URI` / `PROXY_AUTH` / `PROXY_URL_REGEX`、`PUPPETEER_WS_ENDPOINT` / `PLAYWRIGHT_WS_ENDPOINT` / `CHROMIUM_EXECUTABLE_PATH`、`ACCESS_KEY`、`ALLOW_ORIGIN`。来源：<https://docs.rsshub.app/deploy/config>
- "watch"模型（据上一条的缓存配置归纳，非官方表述）：客户端定时拉取，服务端用缓存 TTL 限制对上游的请求频率。

**登录态**

- 登录态是实例级环境变量，由路由在 `features.requireConfig` 里声明：`BILIBILI_COOKIE_{uid}`、`BILIBILI_DM_IMG_LIST`、`WEIBO_COOKIES`、`WEIBO_APP_KEY` / `WEIBO_APP_SECRET`、`ZHIHU_COOKIES`、`TWITTER_AUTH_TOKEN`、`DOUYIN_COOKIE`（"The Cookie request header from your own signed-in Douyin web session"，供 `/douyin/likes/self` 与 `/douyin/collection`）。来源：<https://docs.rsshub.app/deploy/config>

**测试与健康**

- 仓库共 79 个 `*.test.ts`，全部在 `lib/utils/`、`lib/api/` 等公共层，`lib/routes/` 下为 0——路由本身没有单元测试，健康靠 `maintainers` 字段 + issue。来源：<https://github.com/DIYgod/RSSHub/tree/master/lib>
- 脚本规范：文件夹与文件 kebab-case；把别人的 GitHub ID 写进 `maintainers` 而未经同意，会被禁止提交 PR 一个月。来源：<https://docs.rsshub.app/joinus/advanced/script-standard>
- 是否有官方的路由状态页，未查到（待核实）。

**法律与治理**

- 未检索到针对 RSSHub 的公开下架事件（待核实）；GitHub 公开 DMCA 仓库 2024–2026 年目录中没有以 rsshub 命名的通知文件（只按文件名扫描）。来源：<https://github.com/github/dmca>

**许可证与活跃度**

- RSSHub 与 RSSHub-Radar 均 AGPL-3.0；RSSHub 约 46.6k stars，最后推送 2026-10-20；Radar 约 7.4k stars，最后推送 2026-09-01。来源：<https://github.com/DIYgod/RSSHub> 、<https://github.com/DIYgod/RSSHub-Radar>

<a id="8-cobalt"></a>
## 8. Cobalt

- **声明**：服务在 `api/src/processing/service-config.js` 的 `services` 对象里声明，每个服务给 `patterns`（url-pattern 语法，如 bilibili 的 `"video/:comId"`、`"_shortLink/:comShortLink"`、`"_tv/:lang/video/:tvId"`）、`subdomains`、`altDomains`、`tld`；`service-patterns.js` 的 `testers` 再对匹配出的参数做长度校验（如 `pattern.comId?.length <= 12`）。来源：<https://github.com/imputnet/cobalt/blob/main/api/src/processing/service-config.js> 、<https://github.com/imputnet/cobalt/blob/main/api/src/processing/service-patterns.js>
- **分发**：自建实例（Docker 镜像，watchtower 自动更新）；"there is currently no publicly available pre-hosted api"；托管实例 "use bot protection and are not intended to be used in other projects without explicit permission"。来源：<https://github.com/imputnet/cobalt/blob/main/docs/run-an-instance.md> 、<https://github.com/imputnet/cobalt/blob/main/docs/api.md>
- **调用协议**：`POST /`，必须带 `Accept: application/json` 与 `Content-Type: application/json`；请求体只有 `url` 必填，其余如 `videoQuality`、`audioFormat`、`downloadMode`、`filenameStyle`、`alwaysProxy`、`localProcessing`、`subtitleLang`；响应总有 `status`，取值 `tunnel` / `local-processing` / `redirect` / `picker` / `error`；错误体 `{"status":"error","error":{"code":"<machine-readable>","context":{"service","limit"}}}`；所有端点带 `RateLimit-*` 头（IETF 草案格式）。来源：<https://github.com/imputnet/cobalt/blob/main/docs/api.md>
- **鉴权**：`Authorization: Api-Key <uuid>` 或 `Authorization: Bearer <JWT>`（通过 `POST /session` 提交 turnstile 验证结果换取）；缺鉴权返回 `api.auth.<method>.missing` 错误码。来源：<https://github.com/imputnet/cobalt/blob/main/docs/api.md>。
- **登录态**：实例级 `cookies.json`，"if you want your instance to support services that require authentication to view public content"。来源：<https://github.com/imputnet/cobalt/blob/main/docs/run-an-instance.md>
- **测试健康**：未见服务级自动化测试（待核实）。
- **治理**：公共实例 api.cobalt.tools 自 2025 年 6 月起无法处理 YouTube（issue 讨论，二手摘要，待核实细节）；项目没有 GitHub Releases，仓库最后推送 2026-04-06。来源：<https://github.com/imputnet/cobalt/issues/1356> 、<https://github.com/imputnet/cobalt>
- **许可证与活跃度**：`api/` AGPL-3.0，`web/` CC-BY-NC-SA-4.0；约 45.3k stars。来源：<https://github.com/imputnet/cobalt/blob/main/api/LICENSE> 、<https://github.com/imputnet/cobalt/blob/main/web/LICENSE>

<a id="9-karakeep-抓取器"></a>
## 9. Karakeep 抓取器

Karakeep 不是适配器生态，但它的抓取器把"浏览器 / 无浏览器 / 外部程序 / 用户浏览器上传"四种获取方式与超时、限额都做成了配置，是歇存内核 `web-generic` 与 `method: browser` 的对照。

- **架构**：单个 worker（`apps/workers/workers/crawlerWorker.ts`）通过 CDP 连外部浏览器（`BROWSER_WEB_URL` / `BROWSER_WEBSOCKET_URL`，可接 browserless），没有浏览器时退回纯 HTTP（无 JS、无截图）；HTML 解析放在独立子进程并限制内存（`CRAWLER_PARSE_TIMEOUT_SEC` 60、`CRAWLER_PARSER_MEM_LIMIT_MB` 512）；整页归档调用 monolith（`CRAWLER_FULL_PAGE_ARCHIVE`、`CRAWLER_MONOLITH_TIMEOUT_SEC` 5、`CRAWLER_MONOLITH_ARGS` 以 `%%` 分隔）；视频调用 yt-dlp（`CRAWLER_VIDEO_DOWNLOAD`、`CRAWLER_VIDEO_DOWNLOAD_MAX_SIZE` 50、`CRAWLER_VIDEO_DOWNLOAD_TIMEOUT_SEC` 600、`CRAWLER_YTDLP_ARGS`）。来源：<https://docs.karakeep.app/configuration> 、<https://github.com/karakeep-app/karakeep/blob/main/apps/workers/workers/crawlerWorker.ts>
- **超时与限额**：`CRAWLER_JOB_TIMEOUT_SEC` 60、`CRAWLER_NAVIGATE_TIMEOUT_SEC` 30、`CRAWLER_SCREENSHOT_TIMEOUT_SEC` 5、`CRAWLER_CHALLENGE_WAIT_SEC` 15（等验证页自行消失）、`CRAWLER_NUM_WORKERS` 1、`CRAWLER_DOMAIN_RATE_LIMIT_WINDOW_MS` / `_MAX_REQUESTS` 按域限速、`CRAWLER_FULL_PAGE_ARCHIVE_MAX_SIZE_MB`。来源：<https://docs.karakeep.app/configuration>
- **登录态**：`BROWSER_COOKIE_PATH` 指向 JSON 文件，元素为 cookie 对象 `{name, value, domain?, path?, expires?, httpOnly?, secure?, sameSite?}`，启动时注入浏览器上下文。来源：<https://docs.karakeep.app/configuration>。
- **用户浏览器兜底**：SingleFile 扩展直接把当前页面 POST 到 `/api/v1/bookmarks/singlefile`（multipart 字段 `url`、`file`，API key 鉴权，`?ifexists=skip|overwrite|overwrite-recrawl|append|append-recrawl`），文档说明用途是 "websites that don't like to get crawled, has annoying cookie banner or require authentication"。来源：<https://docs.karakeep.app/guides/singlefile>
- **安全**：默认拦截解析到内网/回环地址的请求，`CRAWLER_ALLOWED_INTERNAL_HOSTNAMES` 放行；`CRAWLER_HTTP_PROXY` / `CRAWLER_NO_PROXY`。来源：<https://docs.karakeep.app/configuration>
- **声明/分发/测试/治理**：没有站点适配器概念，不适用。
- **许可证与活跃度**：AGPL-3.0；v0.34.0 发布于 2026-10-17；约 29.9k stars。来源：<https://github.com/karakeep-app/karakeep>

<a id="10-obsidian-web-clipper-模板"></a>
## 10. Obsidian Web Clipper 模板

模板是"声明式、按 URL 触发、用户可导入导出 JSON"的来源规则，对应歇存 `match` 与 `fetch` 输出映射那一层。

- **声明**：一个模板 = 行为（Create a new note / Add to an existing note 顶或底 / Add to daily note）+ 触发规则 + 笔记名格式 + 属性列表 + 正文格式。触发规则三种：URL 前缀（"matches when the page URL starts with the pattern"）、正则（`/.../`，如 `/^https:\/\/www\.imdb\.com\/title\/tt\d+\/reference\/?$/`）、schema.org（`schema:@Recipe`、`schema:@Recipe.name`、`schema:@Recipe.name=Cookie`）；多条规则各占一行；"The first matching template in the list is used, and the first template serves as the fallback"。来源：<https://obsidian.md/help/web-clipper/templates>
- **变量**：预设 `{{title}}`、`{{author}}`、`{{content}}`、`{{contentHtml}}`、`{{fullHtml}}`、`{{description}}`、`{{domain}}`、`{{favicon}}`、`{{highlights}}`、`{{image}}`、`{{language}}`、`{{published}}`、`{{site}}`、`{{url}}`、`{{words}}`、`{{selection}}`；元数据 `{{meta:name:...}}` / `{{meta:property:og:title}}`；选择器 `{{selector:css}}`、`{{selector:img.hero?src}}`、`{{selectorHtml:css}}`；结构化数据 `{{schema:@Type:key}}`、`{{schema:@Type:arr[0].prop}}`、`{{schema:@Type:arr[*].prop}}`；LLM 提示变量 `{{"a summary of the page"}}`（需 Interpreter）；过滤器用 `|` 链式，如 `|markdown`、`|wikilink|join`、`|split:"-"|slice:0,1`。来源：<https://obsidian.md/help/web-clipper/variables>
- **文件形状**：仓库测试夹具 `imdb.json` 的键：`noteNameFormat`、`noteContentFormat`、`properties: [{name, value, type}]`（`type` 为 `text` / `multitext` / `number`）。来源：<https://github.com/obsidianmd/obsidian-clipper/blob/main/src/utils/fixtures/templates/imdb.json>
- **分发**：设置页 Import / Export `.json`，或从剪贴板复制；帮助页指向示例模板仓库 "clipper-templates"（具体仓库地址待核实，`obsidianmd/clipper-templates` 不存在）；路线图里 "Template directory" 尚未完成，"Template validation (1.1.0)" 与 "Template logic (if/for) (1.1.0)" 已完成。来源：<https://obsidian.md/help/web-clipper/templates> 、<https://github.com/obsidianmd/obsidian-clipper/blob/main/README.md>
- **调用协议 / 登录态**：在用户浏览器内执行，天然带登录态；不适用进程协议。
- **测试健康**：仓库内对模板编译、触发器、过滤器有单元测试与 HTML 夹具（`src/utils/fixtures/templates/*.html|json`）。来源：<https://github.com/obsidianmd/obsidian-clipper/tree/main/src/utils>
- **治理**：不适用。
- **许可证与活跃度**：MIT；1.7.1 发布于 2026-07-22；约 5.4k stars。来源：<https://github.com/obsidianmd/obsidian-clipper>

<a id="11-opencli"></a>
## 11. opencli

**身份**

- PRD 所说的"收藏列表适配器参考 opencli"指 jackwener/opencli（npm `@jackwener/opencli`），沿用 [01 §OpenCLI](01-comparable-products.md#opencli) 的结论，本节不重复论证。来源：<https://github.com/jackwener/opencli>
- 同名但无关的项目：PyPI `opencli` 0.0.9（2019-04-27，OpenAPI 规范生成 CLI），npm `open-cli`（打开 URL / 文件的小工具）；linxule/opencli、kyangc/OpenCLI 等是 jackwener/opencli 的 fork。来源：<https://pypi.org/project/opencli/> 、<https://www.npmjs.com/package/open-cli> 、<https://github.com/kyangc/OpenCLI>

**如何声明一个来源**

- 一个"适配器"= 一个站点目录下的一个命令文件 `clis/<site>/<command>.js`（或 `.ts`），调用 `cli({...})` 注册：`site`、`name`、`access: 'read' | 'write'`、`description`、`domain`、`strategy: Strategy.COOKIE`、`args: [{name, type, required?, default?, help}]`、`columns`（表格列）、`func(page, kwargs)`。例：`clis/bilibili/favorite.js` 声明 `fid`（可选，缺省取第一个收藏夹）、`limit`（默认 20）、`page`，调 `/x/v3/fav/folder/created/list-all` 与 `/x/v3/fav/resource/list`（`signed: true`），输出 `rank / title / author / plays / url`。来源：<https://github.com/jackwener/opencli/blob/main/clis/bilibili/favorite.js>
- 打包的适配器由 `cli-manifest.json` 描述；用户目录 `~/.opencli/clis/<site>/<command>.js` 在运行时发现，同名覆盖官方适配器（`adapter eject` / `adapter reset`），`opencli doctor` 会警告这种遮蔽。来源：<https://github.com/jackwener/opencli/blob/main/docs/guide/extending-opencli.md>

**如何分发与安装**

- 主程序：npm `@jackwener/opencli`（Node >= 20.18.1）或桌面应用 OpenCLIApp（托管 `opencli` 命令、更新、"browser-login keepalive"）；浏览器端 Browser Bridge 扩展（Chrome Web Store）+ 按需自启的本地 daemon。来源：<https://github.com/jackwener/opencli/blob/main/README.md>
- 第三方命令以 plugin 形式：`opencli plugin install github:user/repo`、任意 git URL、`file://`；装到 `~/.opencli/plugins/<name>/`，目录下的 `.ts` / `.js` 都是命令；可选清单 `opencli-plugin.json {name, version, opencli: ">=1.0.0"（宿主版本范围）, description, plugins{<sub>: {path, description, version}}}`；`plugin update --all`。仓库名前缀 `opencli-plugin-` 自动去掉。来源：<https://github.com/jackwener/opencli/blob/main/docs/guide/plugins.md>
- 版本节奏：npm 首发 0.1.0 于 2026-03-14；GitHub 100+ 个 release，v2.0.0 2026-10-17、v2.0.2 2026-10-20。来源：<https://www.npmjs.com/package/@jackwener/opencli> 、<https://github.com/jackwener/opencli/releases>

**调用协议（"Agent command contract"）**

- 命令默认输出 JSON：成功 `{ "ok": true, "data": ... }`；失败 `{ "ok": false, "error": { "code", "message", "hint" } }` 写 stdout 且非零退出；日志走 stderr；流式/跟随模式用 JSON Lines。`opencli list` 返回分页的能力清单（`commands`、`total`、`nextOffset`）；`<command> --help -f json` 取参数定义。来源：<https://github.com/jackwener/opencli/blob/main/README.md>
- 浏览器会话有 `--session`、`--site-session ephemeral|persistent`、`OPENCLI_BROWSER_IDLE_TIMEOUT`、`OPENCLI_CLOSE_OWNED_TAB`；`OPENCLI_SITE_SLOTS`（如 `xiaohongshu:1:1500-3000`）把站点请求排到有限的热标签页并加抖动间隔，README 明说用于 "risk-controlled sites"（即风控严的站点要串行、放慢）。来源：<https://github.com/jackwener/opencli/blob/main/README.md>。
- 远程浏览器：`OPENCLI_CDP_ENDPOINT`。来源：<https://github.com/jackwener/opencli/blob/main/README.md>。

**登录态**

- 不传 cookie，直接在用户已登录的 Chrome 里执行（"your credentials never leave the browser"）。每个站点用 `registerSiteAuthCommands({site, domain, loginUrl, verify(page), quickCheck?, poll?, columns})` 生成 `whoami` / `login` 命令：`quickCheck` 看关键 cookie（bilibili 看 `SESSDATA` 与 `DedeUserID`），`verify` 调接口取身份；未登录抛 `AuthRequiredError`，提示 "Run `opencli <site> login` to open the login page, then retry"；`login` 默认最长等 300 s、每 2 s 轮询。来源：<https://github.com/jackwener/opencli/blob/main/clis/_shared/site-auth.js> 、<https://github.com/jackwener/opencli/blob/main/clis/bilibili/auth.js>

**测试与健康**

- `opencli adapter init|check|test`；`adapter test` 用夹具 `~/.opencli/sites/<site>/verify/<command>.json`（首次 `--write-fixture --args ...`）；`browser verify` 对行形状有硬约束："at most 12 top-level keys"、嵌套不超过一层、`id` / `user_id` 这类字段放顶层。来源：<https://github.com/jackwener/opencli/blob/main/docs/guide/extending-opencli.md>
- 仓库内大量 `*.test.js` 与适配器同目录（如 `clis/bilibili/comment.test.js`、`clis/twitter/archive.test.js`）。来源：<https://github.com/jackwener/opencli/tree/main/clis>
- `opencli doctor` 诊断浏览器连通性；还提供给 agent 的 `opencli-autofix` skill 用于修复失效适配器。来源：<https://github.com/jackwener/opencli/blob/main/README.md>

**法律与治理**

- 未检索到下架事件；项目把站点逻辑与宿主放在同一仓库，没有"下架后怎么办"的既定流程（待核实）。

**许可证与活跃度**

- Apache-2.0；仓库创建于 2026-03-14；约 30.5k stars；最后推送 2026-10-20。来源：<https://github.com/jackwener/opencli>

<a id="12-独立进程加-json-协议的插件系统"></a>
## 12. 独立进程加 JSON 协议的插件系统

<a id="121-lsp"></a>
### 12.1 LSP

- **帧与编码**：头部 `Content-Length`（必填）、`Content-Type`（默认 `application/vscode-jsonrpc; charset=utf-8`），`\r\n` 分隔，内容为 JSON-RPC 2.0。来源：<https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/>
- **生命周期**：`initialize` 必须是第一个请求，参数含 `processId`（父进程 ID，"If the parent process is not alive then the server should exit"）、`clientInfo`、`capabilities`、`initializationOptions`；初始化前的请求回 `-32002`（ServerNotInitialized），通知丢弃（除 `exit`）；服务端在回应前不得主动发请求。来源：<https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/>。
- **关闭**：`shutdown` 请求（只要求收尾、不退出，"Clients must not send any notifications other than `exit` or requests to a server to which they have sent a shutdown request"），然后 `exit` 通知；"The server should exit with `success` code 0 if the shutdown request has been received before; otherwise with `error` code 1"。来源：<https://github.com/microsoft/language-server-protocol/blob/gh-pages/_includes/messages/3.17/shutdown.md> 、<https://github.com/microsoft/language-server-protocol/blob/gh-pages/_includes/messages/3.17/exit.md>
- **取消与进度**：`$/cancelRequest` 通知；"A request that got canceled still needs to return from the server and send a response back"，建议错误码 `RequestCancelled`；`$/progress` 对一个 token 追加部分结果，最终响应为空。错误码还有 `RequestFailed -32803`、`ServerCancelled -32802`、`ContentModified -32801`。来源：<https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/>
- **许可证**：规范仓库 CC-BY-4.0。来源：<https://github.com/microsoft/language-server-protocol>

<a id="122-mcp-stdio-与宿主配置"></a>
### 12.2 MCP stdio 与宿主配置

- **版本说明**：下面 stdio / 生命周期 / 超时的引文出自 2025-06-18 修订版；2026-07-28 版的 stdio 页已复核（见本节第三条），生命周期与超时两页未逐条复核（待核实）。来源：<https://modelcontextprotocol.io/specification/versioning>
- **stdio 传输**："The client launches the MCP server as a subprocess"；消息是以换行分隔的单个 JSON-RPC 消息，"MUST NOT contain embedded newlines"；"The server MAY write UTF-8 strings to its standard error (stderr) for logging purposes"；"The server MUST NOT write anything to its stdout that is not a valid MCP message"；"The client MUST NOT write anything to the server's stdin that is not a valid MCP message"。来源：<https://modelcontextprotocol.io/specification/2025-06-18/basic/transports>
- **生命周期**：`initialize {protocolVersion, capabilities, clientInfo{name, title, version}}` → `{protocolVersion, capabilities, serverInfo, instructions}` → `notifications/initialized`；版本协商：服务端支持就回同一版本，否则回自己支持的最新版本，客户端不支持则断开。关闭（stdio）：先关 stdin，等退出，"sending `SIGTERM` if the server does not exit within a reasonable time"，再 `SIGKILL`。来源：<https://modelcontextprotocol.io/specification/2025-06-18/basic/lifecycle>
- **超时**："Implementations SHOULD establish timeouts for all sent requests"，超时发 cancellation 通知并停止等待；"MAY choose to reset the timeout clock when receiving a progress notification"，但 "SHOULD always enforce a maximum timeout"。来源：<https://modelcontextprotocol.io/specification/2025-06-18/basic/lifecycle>。
- **2026-07-28 stdio 的变化**：消息规则不变（换行分隔、stdout 只放协议消息），新增几条：客户端 "SHOULD NOT assume `stderr` output indicates error conditions"；服务端 "SHOULD exit promptly when their standard input is closed"；关闭流程为关 stdin → 等待 → 强制终止（POSIX 从 SIGTERM 升级到 SIGKILL，Windows 用 TerminateProcess 或 Job Objects）；服务端意外退出时客户端 SHOULD 重启它，因为协议无状态，在途请求重试即可；取消用 `notifications/cancelled`，服务端收到后 "MUST NOT send any further messages for it"；不再有 `initialize` 握手，兼容旧版的客户端先发 `server/discover` 探测。来源：<https://modelcontextprotocol.io/specification/2026-07-28/basic/transports/stdio>
- **版本号**：`YYYY-MM-DD`，只在不兼容变更时递增；当前版本 2026-07-28，该版把协商改成每个请求在 `_meta` 里带 `io.modelcontextprotocol/protocolVersion`，并新增必备 RPC `server/discover`；与 2025-11-25 及更早的握手式版本保持向后兼容。来源：<https://modelcontextprotocol.io/specification/versioning>
- **Claude Code 的宿主配置**：`.mcp.json` → `{"mcpServers": {"<name>": {"command", "args", "env", "type"}}}`，无 `type` 视为 stdio；`${VAR}` 与 `${VAR:-default}` 在 `command` / `args` / `env` / `url` / `headers` 展开；`MCP_TIMEOUT` 控制启动超时；每个 server 可设 `"timeout"`（毫秒）覆盖 `MCP_TOOL_TIMEOUT`；输出超过 10,000 token 警告、默认截断 25,000（`MAX_MCP_OUTPUT_TOKENS`）；项目级 `.mcp.json` 首次使用需用户批准；插件可在自己的 `.mcp.json` 用 `${CLAUDE_PLUGIN_ROOT}` 指向随插件分发的二进制。来源：<https://code.claude.com/docs/en/mcp>
- **Codex 的宿主配置**：`~/.codex/config.toml` 的 `[mcp_servers.<name>]`：stdio 用 `command`（必填）、`args`、`env`、`env_vars`、`cwd`；HTTP 用 `url`、`bearer_token_env_var`、`auth`（默认 `oauth`）、`http_headers`；通用 `startup_timeout_sec`（默认 10）、`tool_timeout_sec`（默认 60）、`enabled`、`enabled_tools` / `disabled_tools`；CLI `codex mcp add <name> --env K=V -- <command>`、`codex mcp login <name>`。来源：<https://learn.chatgpt.com/docs/extend/mcp?surface=cli>
- **许可证**：规范仓库许可证 API 返回 NOASSERTION（待核实具体许可）。来源：<https://github.com/modelcontextprotocol/modelcontextprotocol>

<a id="123-docker-cli-plugins"></a>
### 12.3 Docker CLI plugins

- 可执行文件名 `docker-<name>`；搜索目录：`~/.docker/cli-plugins`（配置目录下的 `cli-plugins`）、`/usr/local/lib/docker/cli-plugins`、`/usr/local/libexec/docker/cli-plugins`、`/usr/lib/docker/cli-plugins`、`/usr/libexec/docker/cli-plugins`。来源：<https://github.com/docker/cli/blob/master/cli-plugins/manager/manager_unix.go>
- 宿主用子命令 `docker-cli-plugin-metadata` 探测插件，插件打印 JSON `{SchemaVersion, Vendor, Version, ShortDescription, URL, Hidden, Features}`；另有 `docker-cli-plugin-hooks` 子命令与环境变量 `DOCKER_CLI_PLUGIN_ORIGINAL_CLI_COMMAND`。来源：<https://github.com/docker/cli/blob/master/cli-plugins/metadata/metadata.go>
- 许可证 Apache-2.0。来源：<https://github.com/docker/cli>

<a id="124-kubectl-krew"></a>
### 12.4 kubectl Krew

- 清单 YAML：`apiVersion: krew.googlecontainertools.github.com/v1alpha2`、`kind: Plugin`、`metadata.name`（必须等于文件名）、`spec.version`（带 `v` 的 semver）、`spec.homepage`、`spec.shortDescription`、`spec.description`、`spec.caveats`（装后显示的额外依赖说明）、`spec.platforms[]`：`selector`（`matchLabels` 的 `os` / `arch` 或 `matchExpressions`）、`uri`（`.zip` / `.tar.gz`）、`sha256`、`files[{from, to}]`、`bin`。来源：<https://krew.sigs.k8s.io/docs/developer-guide/plugin-manifest/>
- 安装后 krew 创建符号链接 `kubectl-<name>`（名字里的 `-` 变 `_`，如 `view-logs` → `kubectl-view_logs`），kubectl 按 `kubectl-<name>` 在 PATH 上发现插件。来源：<https://krew.sigs.k8s.io/docs/developer-guide/plugin-manifest/>。
- 索引是独立仓库 kubernetes-sigs/krew-index（Apache-2.0，2026-10-21 仍有推送，约 700 stars），以 PR 收录清单；自动化发布机器人的细节待核实（官方页 404）。来源：<https://github.com/kubernetes-sigs/krew-index> 、<https://github.com/kubernetes-sigs/krew>

<a id="125-gh-extensions"></a>
### 12.5 gh extensions

- 仓库名必须以 `gh-` 开头，其余即扩展名；根目录放同名可执行脚本（推荐 bash），或在 Release 附二进制，命名 "OS-ARCHITECTURE[EXTENSION]" 后缀，如 `gh-whoami-linux-386`、`gh-whoami-windows-amd64.exe`；加 topic `gh-extension` 便于发现。来源：<https://docs.github.com/en/github-cli/github-cli/creating-github-cli-extensions>
- `gh extension install owner/repo`、`gh extension upgrade <name>` / `--all`、`gh extension list`（标出有更新的）、`gh extension remove`；"locally installed and are scoped to the user"；GitHub 声明第三方扩展 "not certified by GitHub"，建议 "audit the source code of the extension before installing or updating"。来源：<https://docs.github.com/en/github-cli/github-cli/using-github-cli-extensions>
- 许可证 MIT（cli/cli）。来源：<https://github.com/cli/cli>

<a id="126-hashicorp-go-plugin"></a>
### 12.6 HashiCorp go-plugin

- 宿主启动子进程，子进程在 stdout 写一行握手：`CORE-PROTOCOL-VERSION|APP-PROTOCOL-VERSION|NETWORK-TYPE|NETWORK-ADDR|PROTOCOL`，如 `1|3|unix|/path/to/socket|grpc`；core 版本必须是 `1`（"Any other value will cause your plugin to not load"）；网络 `unix` 或 `tcp`（`PLUGIN_MIN_PORT` / `PLUGIN_MAX_PORT` 限制端口）；协议 `netrpc` 或 `grpc`；之后走 RPC 而不是 stdio。来源：<https://github.com/hashicorp/go-plugin/blob/main/docs/internals.md>
- 特性清单里与歇存相关的：Protocol Versioning（"can be incremented to invalidate any previous plugins"）、Stdout/Stderr Syncing（插件的标准输出镜像回宿主）、Cryptographically Secure Plugins（校验和 + TLS）、Host upgrade while a plugin is running（reattach）。来源：<https://github.com/hashicorp/go-plugin/blob/main/README.md>
- 许可证 MPL-2.0；最后推送 2026-10-12。来源：<https://github.com/hashicorp/go-plugin>

<a id="127-nushell-plugins"></a>
### 12.7 Nushell plugins

- 可执行文件名以 `nu_plugin_` 开头；宿主以 `--stdio` 启动，stdin/stdout 通信，stderr 不重定向；可选 `--local-socket` 走本地 socket。来源：<https://www.nushell.sh/contributor-book/plugin_protocol_reference.html>
- 启动后插件先写编码声明（一字节长度 + 名字，`\x04json` 或 `\x07msgpack`），双方互发 `Hello {protocol: "nu-plugin", version, features}`；版本按 semver 兼容判定（0.116.2 满足 ^0.116.0，0.117.0 不满足），不兼容则拒载。来源：<https://www.nushell.sh/contributor-book/plugin_protocol_reference.html>。
- 调用：宿主发 `Call [id, "Signature"]` 取命令签名；`Call [id, Run{name, call{head, positional, named}, input}]` → `CallResponse [id, PipelineData | Error]`，错误是 `LabeledError`（只有 `msg` 必填）；宿主随时可发 `Signal Interrupt`（Ctrl+C）；`Goodbye` 表示不再有调用，插件收尾后退出；`Option GcDisabled` 控制是否允许宿主回收空闲插件进程。来源：<https://www.nushell.sh/contributor-book/plugin_protocol_reference.html>。
- 许可证 MIT。来源：<https://github.com/nushell/nushell>

<a id="13-其他相关模式"></a>
## 13. 其他相关模式

任务未点名、但形态与歇存更近的四个系统，各写三五行。

- **Zotero translators**：每个站点一个 JS 文件，文件头是一段 JSON：`translatorID`（"a stable GUID"，驱动自动更新）、`label`、`creator`、`target`（匹配 URL 的正则）、`minVersion`、`maxVersion`、`priority`（数字小者优先，站点专用用 100，多站通用须更大）、`inRepository`、`translatorType`（位掩码 1 import / 2 export / 4 web / 8 search）、`browserSupport`、`lastUpdated`；函数 `detectWeb(doc, url)` 回答"能不能"、`doWeb(doc, url)` 执行；文件尾嵌 `/** BEGIN TEST CASES **/ var testCases = [{type: "web", url, items}]`；客户端自动从 zotero/translators 仓库同步到数据目录的 `translators/`。仓库最后推送 2026-10-19，约 1.7k stars，许可证字段为空（待核实）。来源：<https://www.zotero.org/support/dev/translators> 、<https://github.com/zotero/translators/blob/master/Wikipedia.js> 、<https://github.com/zotero/translators>
- **fivefilters ftr-site-config**：每个域名一个 `example.com.txt`（`.example.com.txt` 匹配所有子域但不匹配裸域），沿用 Instapaper 公开的站点规则格式；仓库跑自动化测试并公开结果页（"look at the test results and see which files you'd like to contribute fixes for"），鼓励用 GitHub 网页编辑直接提 PR。最后推送 2026-10-18。来源：<https://github.com/fivefilters/ftr-site-config/blob/master/README.md> 、<https://github.com/fivefilters/ftr-site-config>
- **Prowlarr/Indexers（Cardigann YAML）**：`definitions/v<N>/<indexer>.yml`，每个 schema 版本一个目录（v1–v12，v11 为当前），每个目录自带 `schema.json`；根节点必填 `caps, description, encoding, id, language, links, name, search, type`，可选 `replaces`、`legacylinks`、`requestDelay`、`settings`、`login{method, path, inputs, error, test{path, selector}...}`、`search{paths, inputs, rows, fields, error...}`、`download`；`scripts/validate.py` 做 schema 校验；"Problematic indexers are automatically excluded using a configurable blocklist"。最后推送 2026-10-21，约 200 stars，许可证待核实。来源：<https://github.com/Prowlarr/Indexers/blob/master/README.md> 、<https://github.com/Prowlarr/Indexers/blob/master/definitions/v11/schema.json>
- **Stremio addon manifest**：addon 是一个 HTTP 服务，`/manifest.json` 声明 `id`（点分，如 `com.stremio.filmon`）、`name`、`description`、`version`（semver）、`resources`（`catalog` / `meta` / `stream` / `subtitles`...，可按资源细分 `types` 与 `idPrefixes`）、`types`、`idPrefixes`（只对以这些前缀开头的 ID 调用）、`catalogs`、`adult`、`p2p`、`configurable` / `configurationRequired`（指向 `/configure` 页面）；请求格式 `/{resource}/{type}/{id}`。SDK MIT，最后推送 2026-10-02。来源：<https://github.com/Stremio/stremio-addon-sdk/blob/master/docs/api/responses/manifest.md> 、<https://github.com/Stremio/stremio-addon-sdk>

<a id="14-对比表"></a>
## 14. 对比表

| 项目 | 声明方式 | 分发与安装 | 调用协议 | 登录态 | 测试健康 | 治理 | 许可证 |
|---|---|---|---|---|---|---|---|
| Mihon + Keiyoushi（[§1](#1-tachiyomi-与-mihon-扩展生态)） | Gradle DSL：`name` / `versionCode` / `contentWarning` / `libVersion` / `source{lang, baseUrl}` / `deeplink`；源类实现 `KeiSource` | 社区 store：`repo.json`（`index_v2` + `signingKeyFingerprint`）→ protobuf 索引 → 签名 APK；`release-assets.json` 带 sha256；应用零内置 store | 进程内 Kotlin 接口；限速、验证页、cookie jar 由应用层统一 | 扩展设置 + 应用 cookie jar；WebView 交互 | CI 构建；无强制站点测试；应用显示 "orphaned" 与加载失败原因 | 2024 Tachiyomi 清空扩展列表后停更；Keiyoushi 独立组织接手；DMCA 源永不回归（置顶 issue）；v0.21 扩展只能从来源 store 更新 | 应用与源码 Apache-2.0 |
| yt-dlp（[§2](#2-yt-dlp)） | Python 类：`_VALID_URL` 正则（命名组 `id`）、`_TESTS`、`_WORKING`、`_NETRC_MACHINE` | 单程序三渠道（stable / nightly / master）+ `--update-to`；插件：`yt_dlp_plugins.*` 命名空间包，多目录 + pip + zip，无校验 | CLI；`-j` / `-J` JSON；`--retries` 10、`--socket-timeout`、`--sleep-requests`；`--extractor-args` | `--cookies` Netscape、`--cookies-from-browser`（含 keyring / profile / container）、`.netrc` | 每个 extractor 必带 `_TESTS`；`hatch test <IE>` | 2020 youtube-dl 下架后恢复，GitHub 改 1201 流程 + $1M 基金；入口拒收侵权站点 | Unlicense |
| gallery-dl（[§3](#3-gallery-dl)） | Python 类：`category` / `subcategory` / `pattern` 正则 / `cookies_domain` / `request_interval` | 单程序；`--update-to`；外部 extractor 目录 `extractor.module-sources` | CLI；`-j` / `-J` / `-s`；`--http-timeout` 30、`--sleep-429`、`-A` / `-T` 早停 | `--cookies`、`--cookies-from-browser`、配置里 cookie 文件 / 键值 / 浏览器 profile 列表、`cookies-select` 轮换 | `test/results/<site>.py`：`#url` / `#class` / `#results` / `#count` | 未见下架事件 | GPL-2.0 |
| you-get（[§4](#4-you-get)） | `SITES` 域名→模块字典（101 条）+ universal 兜底 | pip；无插件 | CLI；`--json`、`-u`；`-x` / `-y` 代理 | `-c` cookies.sqlite 或 cookies.txt | 待核实 | 无 | MIT |
| lux（[§5](#5-lux)） | Go 接口 `Extract(url, Options) []*Data`；按域名 `Register` | 单二进制；无插件 | CLI；`-j` JSON；`-retry` | `-c` 字符串或 Netscape 文件 | Go 测试（待核实） | 无 | MIT |
| BBDown（[§6](#6-bbdown)） | 单站点，无抽象 | Releases 二进制 | CLI；`-info`；`serve` HTTP JSON API | `login` 扫码 / `-c` / `-token` | 待核实 | 2026-05-14 归档，原因未说明 | MIT |
| RSSHub（[§7](#7-rsshub-与-rsshub-radar)） | TS 对象 `Route {path, name, maintainers, example, parameters, features{requireConfig...}, radar[{source, target}], handler}` + `Namespace` | 随主仓库每日发版；自建实例；Radar 规则由 `/api/radar/rules` 聚合给扩展 | HTTP GET → Feed；`REQUEST_TIMEOUT` 3000 ms、`REQUEST_RETRY` 2、缓存 TTL | 实例环境变量（`BILIBILI_COOKIE_{uid}`、`DOUYIN_COOKIE`...），路由在 `requireConfig` 声明 | 路由无单元测试；`maintainers` + issue | 未见下架事件 | AGPL-3.0 |
| Cobalt（[§8](#8-cobalt)） | JS 对象：`patterns`（url-pattern）+ `subdomains` / `altDomains` / `tld` + `testers` 长度校验 | 自建 Docker 实例；无公共 API | HTTP `POST /` JSON；`status: tunnel/redirect/picker/local-processing/error`；错误码 + `context{service, limit}`；`RateLimit-*` | 实例 `cookies.json`；`Api-Key` / turnstile → JWT | 待核实 | 公共实例 YouTube 自 2025-06 不可用（待核实）；要求自建 | api AGPL-3.0 / web CC-BY-NC-SA-4.0 |
| Karakeep 抓取器（[§9](#9-karakeep-抓取器)） | 不适用（无站点适配器） | 随应用 | 内部 worker；每阶段超时（job 60 s、navigate 30 s、monolith 5 s、yt-dlp 600 s）；解析子进程限内存 | `BROWSER_COOKIE_PATH` JSON cookie 对象；SingleFile 扩展上传 | 不适用 | 不适用 | AGPL-3.0 |
| Obsidian Web Clipper（[§10](#10-obsidian-web-clipper-模板)） | JSON 模板：触发（前缀 / 正则 / schema）+ `noteNameFormat` / `noteContentFormat` / `properties` + 变量/过滤器 | 导入导出 `.json`；目录功能在路线图 | 浏览器内执行 | 浏览器自带 | 单元测试 + HTML 夹具 | 不适用 | MIT |
| opencli（[§11](#11-opencli)） | JS `cli({site, name, access, domain, strategy, args, columns, func})`；每命令一文件 | npm / 桌面应用 + Chrome 扩展 + daemon；plugin 从 git 安装，`opencli-plugin.json {name, version, opencli range}` | CLI；stdout `{ok, data}` / `{ok:false, error{code, message, hint}}` + 非零退出；stderr 日志；JSON Lines 流 | 复用已登录 Chrome；`whoami` / `login`；`AuthRequiredError` 带 hint | `adapter test` 夹具；行形状约束；`doctor` | 无既定流程（待核实） | Apache-2.0 |
| LSP（[§12.1](#121-lsp)） | 能力在 `initialize` 协商 | 编辑器配置启动命令 | stdio JSON-RPC + `Content-Length`；`$/cancelRequest` 仍须响应；`shutdown` → `exit`，退出码 0/1 | 不适用 | 不适用 | 不适用 | 规范 CC-BY-4.0 |
| MCP stdio（[§12.2](#122-mcp-stdio-与宿主配置)） | `initialize` 回 `serverInfo` + `capabilities`；2026-07-28 起 `server/discover` | 宿主配置 `command/args/env`（Claude Code `.mcp.json`，Codex `config.toml`）；`${VAR}` 展开 | 换行分隔 JSON-RPC；stdout 只放协议；stderr 日志；SHOULD 超时 + cancel；SIGTERM → SIGKILL | 由宿主 env 注入 | 不适用 | 项目级配置需用户批准 | 待核实 |
| Docker CLI plugins（[§12.3](#123-docker-cli-plugins)） | `docker-cli-plugin-metadata` 子命令输出 JSON `{SchemaVersion, Vendor, Version, ShortDescription, URL}` | 固定目录里的 `docker-<name>` | 子进程 CLI | 不适用 | 不适用 | 不适用 | Apache-2.0 |
| Krew（[§12.4](#124-kubectl-krew)） | YAML 清单：`spec.version`、`platforms[{selector, uri, sha256, bin, files}]`、`caveats` | 独立索引仓库 PR 收录；sha256 校验；符号链接 `kubectl-<name>` | 子进程 CLI | 不适用 | 索引仓库 PR 检查 | 不适用 | Apache-2.0 |
| gh extensions（[§12.5](#125-gh-extensions)） | 仓库名 `gh-<name>` + 同名可执行或 OS-ARCH 命名的 release 二进制 | `gh extension install owner/repo`；topic `gh-extension` | 子进程 CLI | 继承 gh 的登录 | 不适用 | "not certified by GitHub"，建议审源码 | MIT |
| go-plugin（[§12.6](#126-hashicorp-go-plugin)） | 握手行 `1\|APP\|unix\|addr\|grpc` | 宿主指定二进制 + 可选校验和 | stdout 握手 → RPC；协议版本不符拒载 | 不适用 | 不适用 | 不适用 | MPL-2.0 |
| Nushell plugins（[§12.7](#127-nushell-plugins)） | `Hello {protocol, version, features}` + `Signature` | `nu_plugin_*` 可执行 | stdio JSON/MsgPack；`Call` / `CallResponse` / `Signal Interrupt` / `Goodbye` | 不适用 | 不适用 | 不适用 | MIT |

<a id="15-对歇存适配器协议的启示"></a>
## 15. 对歇存适配器协议的启示

按功能面分组。每条标出对应的 [§0](#0-范围与缺口清单) 缺口编号和 [SPEC §8](../SPEC.md#8-适配器协议-v0) 字段；只给形状建议，不替 SPEC 做决定。

<a id="151-声明与匹配"></a>
### 15.1 声明与匹配

1. **`version` 与 `protocol` 分开写（G1 · 新增 `version`、`protocol`）。** Mihon 把扩展版本（`versionCode` + `versionName`）与接口版本（`extensionLib` / `libVersion`）分开，应用用 `SUPPORTED_LIB_VERSIONS` 拒载不兼容者，并把原因告诉用户（[§1](#1-tachiyomi-与-mihon-扩展生态)）；Nushell 按 semver 判兼容，go-plugin 用整数 core 版本，MCP 用日期版本（[§12](#12-独立进程加-json-协议的插件系统)）；opencli 的插件清单用 `opencli: ">=1.0.0"` 声明宿主版本范围（[§11](#11-opencli)）。建议 `adapter.yaml` 增加 `version`（semver，直接填进 `manifest.snapshots[].adapterVersion`）和 `protocol`（整数，当前为 0）。运行器只加载 `protocol` 在支持集合内的适配器，拒载原因显示在适配器列表里。

2. **身份与分级元数据（G2 · 新增 `name`、`description`、`homepage`、`maintainers`、`lang`、`contentWarning`）。** RSSHub 的 `name` / `maintainers` / `example` / `parameters`，Mihon 的 `contentWarning`（SAFE / MIXED / NSFW）与每个 source 的 `language`，Stremio 的 `adult`，Krew 的 `homepage` / `shortDescription` / `caveats`，是各生态都有的最小集（[§7](#7-rsshub-与-rsshub-radar)、[§1](#1-tachiyomi-与-mihon-扩展生态)、[§13](#13-其他相关模式)、[§12.4](#124-kubectl-krew)）。RSSHub 禁止未经同意把别人的 GitHub ID 写进 `maintainers`，这条可以照抄进适配器贡献规范。

3. **`match` 写清语义，兜底适配器显式取最低优先级（G3 · `match`，新增 `priority`）。** yt-dlp 用正则加 `suitable()`；gallery-dl 用正则，按模块顺序取第一个命中；lux 用域名表，`""` 键兜底；Cobalt 先按 `patterns` + `subdomains` / `altDomains` 匹配，再用 `testers` 校验 ID 长度；Zotero 用 `priority`（站点专用 100，通用的必须更大）；Obsidian 用"列表中第一个命中，第一个模板兜底"（[§2](#2-yt-dlp)、[§3](#3-gallery-dl)、[§5](#5-lux)、[§8](#8-cobalt)、[§13](#13-其他相关模式)、[§10](#10-obsidian-web-clipper-模板)）。建议：`match` 项定义为不含 scheme 的 `host/path` glob，`*.` 前缀匹配子域，并写明是否包含裸域（ftr-site-config 的 `.example.com.txt` 就不匹配 `example.com`）；按 `priority` 升序尝试，同值按用户排序；`builtin/web-generic` 的 `["*"]` 固定取最大 `priority`；`match` 只做粗筛，最终身份由 `canonicalize` 判定（见 [§15.5](#155-批量导入与同步)）。

<a id="152-安装定位与更新"></a>
### 15.2 安装、定位与更新

4. **`bin` 定位：写显式 `command`，不扫 `PATH`（G4 · `method`）。** MCP 宿主一律在配置里写 `command` / `args` / `env` 并支持 `${VAR}` 展开，插件用 `${CLAUDE_PLUGIN_ROOT}` 指向随包二进制（[§12.2](#122-mcp-stdio-与宿主配置)）；Docker / kubectl / gh 靠命名约定加固定目录（[§12.3](#123-docker-cli-plugins)、[§12.4](#124-kubectl-krew)、[§12.5](#125-gh-extensions)）；yt-dlp 插件搜索位置多达十几处（[§2](#2-yt-dlp)）。建议 `method: cli` 时写 `command: ["node", "${ADAPTER_DIR}/dist/cli.js"]`，运行器只在 `<config>/adapters/<id>/` 下解析 `${ADAPTER_DIR}`。`local-service` / `http-api` 写 `endpoint`，并约定 `GET /adapter.json` 返回同一份声明（Stremio `/manifest.json`、Docker `docker-cli-plugin-metadata` 的做法）；`mcp` 直接复用 MCP 自己的发现机制（2026-07-28 版的 `server/discover`，旧版的 `initialize`），不另定握手；`browser`（OpenCLI 这一类）声明它依赖的上游 CLI 与浏览器扩展，用 `requires` 表达（见 [§15.4](#154-登录态)）。

5. **`install` 改为带类型的联合（G5 · `upstream.install`，新增 `requires.runtime`）。** 参照 Krew 的 `platforms[{selector{os, arch}, uri, sha256, bin, files}]`、gh 的 release 资产命名、Keiyoushi 的 `release-assets.json` sha256、opencli 的 `plugin install github:user/repo`、yt-dlp 的 pip / zip（[§12.4](#124-kubectl-krew)、[§12.5](#125-gh-extensions)、[§1](#1-tachiyomi-与-mihon-扩展生态)、[§11](#11-opencli)、[§2](#2-yt-dlp)）。建议 `install` 取下列之一：`{type: github-release, repo, assets: {"darwin-arm64": {name, sha256}, ...}}`、`{type: npm, package, version}`、`{type: pip, package, version}`、`{type: manual, instructions}`。另加 `requires.runtime`（如 `["node>=20.18", "deno>=2.3"]`）：yt-dlp 的 YouTube 支持已依赖外部 JS runtime，OpenCLI 依赖 Node 与 Chrome 扩展，这类依赖应在安装前就让用户看到。

<a id="153-调用错误与产物"></a>
### 15.3 调用、错误与产物

6. **错误信封、退出码、stderr、超时与取消（G8 · 调用约定）。** 可借鉴的现成形状：opencli 的 `{ok:false, error:{code, message, hint}}` 加非零退出（[§11](#11-opencli)）；Cobalt 的 `error.code` 加 `context{service, limit}`（[§8](#8-cobalt)）；yt-dlp 的 `raise_login_required` / `raise_geo_restricted` / `raise_no_formats` 分类（[§2](#2-yt-dlp)）；MCP 的"stdout 只放协议，stderr 只放日志"与"关 stdin → 等待 → SIGTERM → SIGKILL"；LSP 的"取消后仍须响应"与 0/1 退出码（[§12](#12-独立进程加-json-协议的插件系统)）。建议：失败时 stdout 仍输出 JSON：`{"ok": false, "error": {"code": "unsupported-url | not-found | login-required | rate-limited | upstream-changed | network | internal", "message", "hint", "retryAfterSec"?}}`；成功输出外层包 `ok: true`，便于统一解析；退出码 0 成功、1 一般失败、2 用法错误、3 需要登录、4 被限流，与 `code` 一一对应；stderr 只放日志，且不作为失败信号（MCP 2026-07-28 明确客户端不应把 stderr 输出当作错误），成败只看退出码与 stdout 信封，内核保留 stderr 最后 N 行进 `evidence`；每个命令的默认超时写进 SPEC（例如 `canonicalize` 5 s、`fetch` 120 s、`list` 60 s、`health` 10 s，数值待定），超时由内核发 SIGTERM，宽限后 SIGKILL，并按 `network` 失败换下一个适配器。

7. **`--out` 的基准与归属（G9 · `--out`、`evidence`）。** Krew / gh 把安装目录完全交给宿主管理；Karakeep 每个产物先落 asset store 再登记，归档超限就丢弃（[§9](#9-karakeep-抓取器)）。建议 SPEC 写明：内核创建一个空的、绝对路径的临时目录传给 `--out`；适配器只在其中写文件，输出里的路径相对该目录（现 schema 已如此）；成功后内核整体移动为 `raw/<snapshotId>/`，失败则整目录丢弃；适配器不得依赖 cwd。`evidence` 增加 `method`，`status` 只在 HTTP 类方式下出现。

<a id="154-登录态"></a>
### 15.4 登录态

8. **`requires.cookie` 给枚举，并规定传递通道（G6 · `requires`）。** 各生态的登录态形态：Netscape cookie 文件（yt-dlp / gallery-dl / lux / you-get）、从浏览器 profile 直读（yt-dlp / gallery-dl 的 `--cookies-from-browser`）、JSON cookie 对象（Karakeep）、实例环境变量（RSSHub `DOUYIN_COOKIE`）、实例文件（Cobalt `cookies.json`）、根本不传而在用户已登录的浏览器里执行（opencli、Obsidian Web Clipper）。[01 §OpenCLI](01-comparable-products.md#opencli) 已建议 `requires.cookie` 区分"none / 浏览器会话 / 用户粘贴"三档，本次调研支持这个划分。建议取值 `none | browser-session | cookie-file`，另加 `requires.login: required | optional`；`cookie-file` 时内核把 cookie 写成 Netscape 格式的临时文件，用环境变量 `XIECUN_COOKIE_FILE` 传入，不走 argv（避免进入 shell 历史和进程列表）；`requires.proxy: none | optional | required`。登录是否有效由 `health` 的 `auth` 回报（见 [§15.6](#156-测试与健康)），失效时 `fetch` / `list` 返回 `login-required` 与 hint（opencli 的 `AuthRequiredError` 提示用户运行 `<site> login`，是可照搬的交互）。

<a id="155-批量导入与同步"></a>
### 15.5 批量导入与同步

9. **`canonicalize` 未命中、`list` 的输出与游标、`watch`（G7 · `canonicalize`、`list`、`capabilities: watch`）。** 建议：`canonicalize` 对不属于自己的 URL 输出 `{"match": false}` 并以 0 退出（这不是错误），命中时输出 `{"match": true, "identity": {...}}`。`list` 输出 `{items: [{identity, url, title?, author?, publishedAt?, action?, actedAt?}], nextCursor: string | null, exhausted: bool, evidence}`；游标是适配器私有的不透明字符串（RSSHub 用 `ps=20` 分页，opencli 用 `page` + `limit` 与 `nextOffset`，Stremio 用 `skip`，宿主都不解析）；`<source>` 用 `<platform>:<kind>:<id>`（如 `bilibili:fav:50948568`）。`watch` 不另定协议，由内核按 `list` 低频轮询（RSSHub 的模型：客户端定时拉，服务端靠缓存 TTL 限制上游请求）。风控退避：opencli 的 `OPENCLI_SITE_SLOTS`（如 `xiaohongshu:1:1500-3000`）把风控严的站点限制为单标签页串行加抖动间隔，与 PRD 的"增量、低频、单会话，遇风控自动退避"一致，可以由内核统一施加，并用 `rate-limited` + `retryAfterSec` 驱动退避。收藏列表的上游按 [01 §OpenCLI](01-comparable-products.md#opencli) 的结论用 OpenCLI：它的 `bilibili favorite` 输出 `rank / title / author / plays / url`，没有平台内容 ID 和收藏时间（[§11](#11-opencli)），包装成歇存适配器时要用 `canonicalize` 从 `url` 补出 `identity`，`actedAt` 缺失时留空而不是用抓取时间冒充。RSSHub 是另一条 `list` 上游（`method: local-service`）：它有 `/bilibili/fav/:uid/:fid` 与 `/douyin/collection`（需 `DOUYIN_COOKIE`）（[§7](#7-rsshub-与-rsshub-radar)），但 bilibili 收藏路由只请求最新 20 条（`ps=20`，没有翻页参数），条目带 `pubDate`（取自 `fav_time`），适合 `watch` 的增量轮询，不适合一次性回填全部历史。

<a id="156-测试与健康"></a>
### 15.6 测试与健康

10. **`health` 的输出（G7 · `health`）。** 参照 Docker 的 metadata JSON、Nushell 的 `Hello` + `Signature`、MCP 的 `serverInfo` + `capabilities`、opencli 的 `whoami {logged_in, site, id}` 与 `doctor`（[§12](#12-独立进程加-json-协议的插件系统)、[§11](#11-opencli)）。建议 `health` 输出 `{protocol, version, capabilities, auth: {state: ok | login-required | unknown, account?}, upstream?: {name, version}, checks: [{name, ok, message}]}`，退出码 0 表示可用、非 0 表示不可用。`upstream.version` 让用户看到被包装工具（如 OpenCLI、yt-dlp）的版本，这两个项目都按周甚至按天发版。

11. **契约样例与健康汇总（[SPEC §8](../SPEC.md#8-适配器协议-v0) 末句"契约校验器用 `testdata/` 固定样例跑"）。** yt-dlp 的 `_TESTS` 与 Zotero 的嵌入测试把样例放在适配器旁边；gallery-dl 的 `#url` / `#class` 样例可以离线验证"哪个类接住了哪个 URL"；ftr-site-config 把全库测试结果公开成一页；opencli 的 `adapter test` 用本地夹具，并对行形状设硬约束；RSSHub 路由零单元测试，只靠 `maintainers`，是反例（[§2](#2-yt-dlp)、[§13](#13-其他相关模式)、[§3](#3-gallery-dl)、[§11](#11-opencli)、[§7](#7-rsshub-与-rsshub-radar)）。建议适配器仓库约定 `testdata/canonicalize.json`（`[{url, identity | {match: false}}]`，离线可跑，内核的契约校验器直接消费）和 `testdata/fetch/*.json`（标 `live: true`，CI 可选跑）。"健康上报"汇总成一页"平台 × 适配器的最近一次 `health` / `fetch` 结果"，形式参考 ftr 的测试结果页；上报内容只含适配器 id、版本、结果码，不含 URL 与内容。

<a id="157-适配器源信任与治理"></a>
### 15.7 适配器源、信任与治理

12. **分层的适配器源：零内置、store 元数据、只从来源更新（G10 · 新增 store 层）。** Mihon 的做法最完整（[§1](#1-tachiyomi-与-mihon-扩展生态)）：应用不内置任何 store（FAQ 原话 "does not have, associate, or provide any repositories or extensions"）；每个 store 一个 `repo.json`（名称、网站、签名指纹）指向索引；索引条目带 `versionCode` / `contentWarning` / 每个 source 的 `message`，产物另有 sha256；安装时校验签名并归属到 store；v0.21 起"扩展只从它来自的 store 更新"，store 里已没有的叫 "orphaned"。建议歇存的适配器源照此分层：`store.json {name, homepage, signingKey, indexUrl}` → `index.json {protocol, adapters: [AdapterDecl + {assets, sha256, message?}], removed: [{id, reason, at}]}`；内核记录每个已装适配器来自哪个 store，只从该 store 更新；store 删除条目时内核标记 `orphaned` 并显示 `removed` 里的原因，不自动卸载，已经存下的快照不受影响。签名方案留 ADR，`sha256` 可以在协议 v0 就要求。

13. **治理：仓库分离、公开移除清单、把"不做"落到字段上（`limits`、`requires`）。** Tachiyomi 的应用、扩展、网站同在一个组织，法律压力一到全部下线；Keiyoushi 用独立组织 + 纯产物分支 + 置顶"已移除清单" issue（写明来源方与年份、"will not be added back"）接手；yt-dlp 用入口审查拒收以侵权内容为主的站点；Mihon 的 `message` 字段让 store 能向用户解释某个源为什么没了（[§1](#1-tachiyomi-与-mihon-扩展生态)、[§2](#2-yt-dlp)）。歇存已把平台适配器放在社区仓库（[PRD §9](../PRD.md#9-风险)），可再补两点：`limits` 从自由文本改成结构化的 `policy: {personalUseOnly: true, rateLimit?: "N/period"}`，内核在批量导入时按它限速；[PRD §6](../PRD.md#6-不做) 的"不内置需要本地代理 / 解密的工具"落到 `requires` 上——需要外部 runtime 或代理的适配器必须在 `requires.runtime` / `requires.proxy` 里声明，内核在安装前列出来，由用户决定装不装。

<a id="16-待核实清单"></a>
## 16. 待核实清单

- Mihon 2024-01-31 要求 `repo.json` 的原始公告文本（只看到 X 帖子标题）。
- Tachiyomi 事件中 Kakao 反盗版部门先要求下架扩展的说法（仅二手报道）。
- Mihon 的 `index.pb` 对应的 `.proto` 源文件位置（字段已从 Kotlin 模型反推）。
- yt-dlp 的退出码表；EJS 从哪个 stable 版本起成为 YouTube 必需。
- you-get、lux、BBDown、Cobalt 是否有站点级自动化测试及其覆盖面。
- you-get、lux 是否发生过下架事件（只做了网页检索）。
- BBDown 2026-05-14 归档的原因。
- Cobalt 公共实例 YouTube 不可用的起止时间与官方说明（仅 issue 摘要）。
- RSSHub 是否有官方路由状态页；RSSHub 历史上是否收到并执行过下架要求。
- Obsidian 帮助页提到的示例模板仓库 "clipper-templates" 的实际地址。最可能是 kepano/clipper-templates，帮助页是否链到它仍待核实，见 [05 §1.1](05-note-apps-and-plugins.md#11-obsidian-社区插件)。
- Krew 索引的 PR 自动检查与发布机器人（官方页面 404）。发布机器人的形态已在 [05 §3.8](05-note-apps-and-plugins.md#38-其他先例) 查明；PR 自动检查仍待核实。
- MCP 规范仓库、Zotero translators、ftr-site-config、Prowlarr/Indexers 的许可证（GitHub API 返回空或 NOASSERTION）。
- MCP 2026-07-28 版的生命周期与超时条文（[§12.2](#122-mcp-stdio-与宿主配置) 引用的是 2025-06-18 版；stdio 页已复核）。
- opencli 的下架/移除流程是否存在。
