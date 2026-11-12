<a id="5-投递接口"></a>
## 5. 投递接口

很多现成工具不是宿主能启动的进程：浏览器扩展、油猴脚本、iOS 快捷指令、Android 分享、Agent Skill。它们走投递接口，把链接或已经取到的内容交给宿主。内容的形状就是 [§3.7](fetch.md#37-fetch-输出) 的 fetch 输出；来源由宿主按令牌标明，投递方自己不能改。不带令牌的入口（`usuyo://` 链接一类）都要用户在窗口里确认一次才进队列。

<a id="51-本机-http-接口"></a>
### 5.1 本机 HTTP 接口

对应 PRD 的浏览器扩展、本地 HTTP API，约定照 [03 §6.3](../research/03-agent-protocols.md#63-对歇存的意义)。做了的：`/v1/ping`、`/v1/deliver`（只收链接）、`/v1/batches` 和按投递端发的令牌，和窗口用的路由在同一个监听口上，规则见 [SPEC 本机接口](../spec/local-api.md#本机接口第-2-层11-11-照代码)；下面没做的部分标了「还没做」。

- 只绑 `127.0.0.1`，端口 `47615`，和给窗口用的路由是同一个监听口，不另开端口。
- 每个请求带 `Authorization: Bearer <令牌>`。令牌按投递端分别生成：名字（例如「xhs-vault 脚本」）、入口类型（`extension`、`mobile`、`api`），可单独撤销。在应用的导入页里发（`POST /deliver-tokens`，主令牌调），格式 `<id>.<秘密>`，整个令牌只在发的那一次回给调用方；存在按机器目录的 `deliver-tokens.json`，不放进资料库。投递令牌只能进 `/v1/ping`、`/v1/deliver`、`/v1/batches`。
- 除 `chrome-extension://`、`moz-extension://`、`safari-web-extension://` 以外，带任何 `Origin` 头的请求都拒绝，包括 `Origin: null`（沙箱 iframe、`file://` 页面会带它），也不回 CORS 头：普通网页发不进来，也读不到响应。没有 `Origin` 的请求放行。宿主另外核对 `Host` 头是 `127.0.0.1:<端口>`、`localhost:<端口>` 或 `[::1]:<端口>`。各油猴脚本管理器的 `GM_xmlhttpRequest` 实际带什么 `Origin` 待核实。投递令牌发出去以前先握手：`POST /handshake`，请求 `{nonce, client}`，`client` 是令牌 `.` 前面的 id，内核用整个令牌当密钥回 HMAC 证明，令牌本身不上线路（[SPEC 本机接口](../spec/local-api.md#本机接口第-2-层11-11-照代码)）。`/v1/…` 不回 CORS 头，也不答 CORS 预检（Safari 的 Userscripts 没有网站访问权限时会先发预检，还没做，[总纲](../BUILD.md#5-功能线)）。
- 条目在 `origins[]` 里那一项的 `entry` 取令牌的入口类型。`extension` 令牌交来已取到的内容时，快照的 `plugin` 记 `deliver:<令牌名>`；只交链接的，快照记去取它的那个插件。投递来的内容同样按不可信输入处理。
- **令牌能交什么。** `api` 和 `mobile` 类令牌只接受链接（`url`、`text` 和 `/v1/batches`），不接受已取到的内容；已取到的内容只接受 `extension` 类令牌和收件文件夹（[§5.2](#52-监听文件夹)）（还没做：`/v1/deliver` 现在只收链接，没有 multipart，任何令牌都不能交已取到的内容）。Agent 用 `api` 令牌交已取好的内容，等于没经过改动提议就改了已有条目，和「Agent 写入先提议后生效」的原则冲突。
- **身份由宿主重新算。** 宿主不采用投递方给的 `identity`，而是用 `identity.url`（没有就用 `url`）重新调 `canonicalize` 得出身份；拿不到身份的（没有网址的；http(s) 链接马上就有身份，最差是这条链接的 `web:` 身份，离线时的短链也一样，之后可能并进平台 id，见 [§3.1](fetch.md#31-运行器与节奏)），按无平台 ID 的条目处理（[SPEC §0.4](../spec/conventions.md#04-id)）。不然一个持有令牌的脚本报出某个已有条目的身份、再带上 `completeness: full`，就能覆盖那个条目。

| 方法与路径 | 用途 | 返回 |
|---|---|---|
| `GET /v1/ping` | 检查令牌和连通 | `{"name":"usuyo","version":"…","protocol":1,"client":"xhs-vault 脚本"}` |
| `POST /v1/deliver` | 收下一条：只有链接或分享文字，或已取到的内容（已取到的内容还没做；请求体只认 `url`、`text`，别的字段包括 `action` 都被忽略） | `202`，身份一算出就回条目 id（一般 3 秒内）；`canonicalize` 因网络失败或超时（例如离线时的短链）时，回这条链接的 `web-` id，联网后可能并进平台 id，旧 id 留作别名（[SPEC §0.4](../spec/conventions.md#04-id)） |
| `POST /v1/batches` | 交一批链接，进导入预览 | `202`，批次 id；新增、已有的数目在导入预览里随身份算出逐步出现 |

只有链接时，宿主照 [§3.1](fetch.md#31-运行器与节奏) 去取内容；`text` 是手机分享出来的整段文字，宿主从里面找出第一个链接：

```json
{"url": "https://v.douyin.com/iRNBho6u/", "action": "favorite"}
```

```json
{"text": "7.94 复制打开抖音，看看【咖啡日常的作品】手冲咖啡：三段式注水 https://v.douyin.com/iRNBho6u/ 12/30 abc:/"}
```

返回（不收的条目怎么回要改，见 [§13](open.md#13-待定) 第 33 项；现在私密条目被跳过时现在回的和「收下过」一样，`duplicate: true`，没有单独的 `skipped`）：

```json
{"id": "douyin-4plrcxk343onmyxibmrr672kga", "state": "accepted", "duplicate": false}
```

已经取到内容的（例如在页面里拿到了正文和图片），用 `multipart/form-data`（还没做，下面是设计）：`item` 部分是 fetch 输出 JSON；每个文件部分的**字段名**就是 JSON 里引用的相对路径，宿主按字段名找文件，`filename` 只作参考。原因：有的 multipart 库取文件名时只留最后一段（Go 的 `Part.FileName()` 会先过一遍 `filepath.Base`，用 Go 1.26 的 `go doc` 核对过），`images/001.jpg` 会变成 `001.jpg`；各家浏览器扩展对带斜杠的 `filename` 处理也不一致（推断）。字段名照 [§3.6](fetch.md#36-fetch-与输出目录) 的输出目录规则校验，一次投递默认最多 1 GiB。`item` 按 fetch 输出的 schema 校验，只有 `evidence.access` 可以省略，宿主在快照里记成 `deliver`。

```
curl -X POST http://127.0.0.1:47615/v1/deliver \
  -H "Authorization: Bearer $USUYO_TOKEN" \
  -F 'item=@item.json;type=application/json' \
  -F 'images/001.jpg=@images/001.jpg'
```

一批链接（例如油猴脚本导出的小红书收藏夹清单）进导入预览，用户确认后按节奏去取：

```json
{"name": "小红书收藏夹 2026-11-01", "action": "favorite", "links": [
  {"url": "https://www.xiaohongshu.com/explore/671234567890abcdef012345?xsec_token=…", "title": "…", "actedAt": "2026-10-30T21:00:00+08:00"}
]}
```

<a id="52-监听文件夹"></a>
### 5.2 监听文件夹

不开网络端口的办法，适合手机经用户自己的云盘（iCloud 云盘等）投递，对应 PRD 的「自有云盘监听目录」。用户在设置里选一个收件文件夹，宿主处理里面的（现在收件文件夹固定是按机器目录里的 `inbox/`，不能选，设置里还没有；只读顶层的 `.txt`，下表别的行都还没做）：

| 放进去的东西 | 当作 |
|---|---|
| `.txt`、`.url`、`.webloc` | 链接，`.txt` 里每个链接一条，也可以是整段分享文字 |
| `<名字>.usuyo/` 文件夹，里面一个 `item.json` 加它引用的文件 | 已取到的内容；`item.json` 最后写 |
| 文件名命中某个已装导入插件的 `files`（例如 WeChatBridge 的合并转发包） | 交给那个插件的 `list`（[§3.9](fetch.md#39-从文件导入)）；多个插件都命中时，用用户排在最前的那个 |
| 其他文件（截图、PDF、视频） | 文件条目，`kind: file` 或 `image` |

- 一个条目 10 秒没有变化、里面没有云盘的占位文件，才开始处理。
- 收件文件夹里的条目照 [§3.6](fetch.md#36-fetch-与输出目录) 的输出目录规则校验，例如放进来的符号链接一律不收；不合规的移到 `.usuyo-failed/`。
- 从收件文件夹来的导入，是直接收下，还是进导入预览等你确认：收件文件夹没人值守，放进去本身就是一次确认，默认直接收下，见下。
- `<名字>.usuyo/` 里的 `item.json` 和投递接口一样，身份由宿主重新算（[§5.1](#51-本机-http-接口)）；没有平台内容 ID 的（聊天记录、转发的图片）可以不写 `identity`（[§3.7](fetch.md#37-fetch-输出)）。
- 处理完移到 `.usuyo-done/<日期>/`，失败的移到 `.usuyo-failed/`，旁边放一个 `.error.json`。
- 条目在 `origins[]` 里那一项的 `entry` 记 `mobile`，收件文件夹由用户指定入口类型时照指定的记；放进来的是已取到的内容（`<名字>.usuyo/`）或其他文件时，快照的 `plugin` 记 `deliver:inbox`，链接和交给导入插件的文件，快照照常记取它的那个插件。

从收件文件夹来的导入直接收下，处理完提示一次这次收下了多少条；设置里能改成先进导入预览。只有链接、要联网去取的条目照批量导入的节奏排队。

<a id="53-各种入口怎么接"></a>
### 5.3 各种入口怎么接

| 入口 | 走哪条 | `origins[].entry` |
|---|---|---|
| usuyo 自己的浏览器扩展「保存当前页」 | `POST /v1/deliver`，带正文和文件 | `extension` |
| 别人的扩展和油猴脚本（xhs-vault、Weibo-archiver 一类） | `POST /v1/batches` 交清单；或导出 JSON 文件，再用导入插件读（[§3.9](fetch.md#39-从文件导入)） | `extension` 或 `import` |
| iOS 快捷指令 | 写进监听文件夹；或开了局域网直传时 `POST` 到电脑 | `mobile` |
| Android 分享（HTTP Shortcuts 一类） | 局域网直传 | `mobile` |
| Agent Skill（用户自己的 Agent 操作完把链接交过来） | `usuyo add <url>`，或用自己的 `api` 令牌 `POST /v1/deliver`；只交链接，内容由宿主照 [§3.1](fetch.md#31-运行器与节奏) 去取 | `cli` 或 `api` |
| Raycast、Alfred | 优先调用 `usuyo add <url>`；`usuyo://add?url=<链接>` 只打开收下窗口并填好链接，用户点了「收下」才进队列，因为任何网页都能触发这个链接 | `cli` 或 `api` |
| 别的工具的产物（BiliNote 的 `note_results`） | 导入插件，或监听文件夹 | `import` |
| 照搬 qiaomu-clipper 一类做成的浏览器扩展（例如视频号在元宝页内取） | 扩展只能经 `POST /v1/deliver` 往宿主推内容，不在运行器的顺序里；手机分享过来的链接，宿主没法交给扩展去取。要让扩展也当获取方式，就给 `extension` 令牌加一个领任务的接口：例如 `GET /v1/jobs?platform=wechat-channels&wait=30` 长轮询领链接，交回时 `POST /v1/deliver` 带上 `jobId`。不做：扩展不当获取方式，见下 | `extension` |

**局域网直传**（还没做）是唯一听非回环地址的入口，默认关。打开后另起一个端口，手机扫电脑上的二维码拿到地址和这台手机专用的令牌。局域网上的明文 HTTP 会让同一网络里的人看到令牌和内容；手机端快捷指令对自签证书的支持待核实，所以默认推荐监听文件夹（默认走哪条见 [§13](open.md#13-待定) 第 6 项）。

浏览器扩展不当获取方式、不领任务，只做「保存当前页」和投递；要在页面里取的平台做成带专用资料目录的插件，进运行器的顺序，手机分享来的链接也取得到。
