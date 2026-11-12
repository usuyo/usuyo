<a id="本机接口第-2-层11-11-照代码"></a>
## 本机接口（第 2 层，11-11 照代码）

守护进程（`usuyo daemon`，`crates/usuyo-api`）给桌面窗口、命令行、开发服务器和投递端（浏览器扩展、油猴脚本、手机）用的本机 HTTP 接口，照 11-11 的代码写（HEAD 5e98e5a）。完整的第 2 层（MCP、OpenAPI、搜索语法）还没写。**路径和类型以 `apps/desktop/web/src/api/endpoints.ts` 为准**：改接口先改这个文件，Rust 那边照它做。

- **监听**：只绑 `127.0.0.1`，端口固定 47615（`--port`、`USUYO_PORT` 能改，但窗口的 CSP 只放行 47615）。一个监听口，窗口用的路由和投递接口（`/v1/…`）都在上面，不另开端口。端口被占时守护进程起不来，已有的 `api-token`、`daemon.json` 不动；壳不再起第二个，窗口里说端口被别的程序占着。
- **谁能调什么**：每个请求过一道门（`guard.rs`），分三种调用方。
  - 带主令牌的（`api-token` 里的那个：壳、命令行、开发代理、别的本机程序）：所有路由，包括 `/v1/…`；经 `/v1/deliver` 收下的记入口 `api`。
  - 带投递令牌的：只能进 `GET /v1/ping`、`POST /v1/deliver`、`POST /v1/batches`，名单以外一律 401 `unauthorized`，和没带令牌一样。
  - 不带令牌也放行的：CORS 预检（只限窗口用的路径）、`POST /handshake`、带对的 `k` 的 `/media/<itemId>/…`。
- **令牌**：除了上面三种，每个请求都带 `Authorization: Bearer <令牌>`，`/health`、`/events` 也要。主令牌是 32 字节随机数的小写十六进制，每次启动新生成，绑上端口以后写进按机器目录的 `api-token`（0600，先写临时文件再改名），正常退出时连同 `daemon.json` 删掉（只删还是自己写的那份）。按定长时间比对；错了回 401 `unauthorized`。
- **`Host`**：最先查，只认 `127.0.0.1:<端口>`、`localhost:<端口>`、`[::1]:<端口>`（不分大小写），别的回 403 `forbidden`。
- **`Origin`**：只当跨站过滤，谁都能伪造，不用来放行。没带的照常查令牌（命令行、开发代理、油猴脚本管理器常不带）；带了的按路径分：
  - 窗口用的路径：只认 Tauri 的 webview，`tauri://localhost`、`http://tauri.localhost`。
  - `/v1/…`：只认浏览器扩展的来源，以 `chrome-extension://`、`moz-extension://`、`safari-web-extension://` 开头的；窗口的来源、普通网页、`null` 都回 403。
  - `/handshake`：扩展的来源和窗口的来源都认（扩展发令牌以前要先确认对面是内核）。
  - 不合的回 403 `forbidden`，令牌对也一样。开发服务器的来源不在名单里：Vite 代理只接自己页面的同源请求，在服务端带上令牌，去掉 `Origin`、`Referer`。
  - CORS 头只给窗口的来源（允许 GET、POST、PUT、DELETE，请求头 `Content-Type`、`Authorization`、`Range`，暴露 `Content-Range`、`Accept-Ranges`、`Retry-After`）。`/v1/…` 不回 CORS 头，也不答 CORS 预检（预检不带令牌，落到 401）。
- **握手**：客户端先确认端口上是这次启动的守护进程，再发令牌。`POST /handshake`，请求 `{"nonce": "<64 位小写十六进制>", "client": "<投递令牌的 id>"}`，`client` 可省（请求体最多 256 字节，不合格回 400 `invalid-args`），回 `{"proof": "<64 位小写十六进制>"}`：以密钥的 UTF-8 字节为密钥，对 `usuyo-daemon-proof/v1\n127.0.0.1:<端口>\n<nonce>` 做 HMAC-SHA256。不给 `client` 时密钥是主令牌；给了，密钥是那个投递令牌的整个 `<id>.<秘密>`（投递端只发 id，令牌本身不上线路）。不认识的 id 也回一个证明，用随机的密钥，对面分不出那个 id 在不在、撤没撤。端口写进消息，换个端口转来的回答对不上。核对的是客户端，用自己读到的令牌：壳（找守护进程时和每次重新连时）、命令行、Vite 代理（令牌文件变了就重新握手，对不上回 502，不把令牌发过去）、投递端（油猴脚本核对不过就什么都不发）。
- **投递令牌**：格式 `<id>.<秘密>`，`id` 是 16 位小写十六进制（不是秘密），`秘密` 是 64 位小写十六进制。主令牌调 `POST /deliver-tokens`（`{name, kind}`，`name` 1 到 80 个字，`kind` 取 `extension`、`mobile`、`api`）发一个，整个令牌只在这一次的回答里出现（在 `{id, name, kind, createdAt}` 之外多一项 `token`）；`GET /deliver-tokens` 列出投递端，没有令牌；`DELETE /deliver-tokens/{id}` 撤销，马上生效，回 204，没有这个 id 回 404。存在按机器目录的 `deliver-tokens.json`，见 [§1.1](library.md#11-目录树)。投递令牌的 `kind` 决定条目 `origins[].entry`：`extension`、`mobile`、`api`。
- **媒体**：`<img>`、`<video>` 带不了头，`ItemView` 里的媒体地址是 `/media/<itemId>/<层>?k=<key>`。`k` 是 HMAC-SHA256 的十六进制，密钥每次启动另生成（和令牌无关），消息是 `usuyo-media/v1\n<itemId>`，只对这一条有效。支持 Range。`Content-Type` 只回允许的几种，别的写 `application/octet-stream`；另带 `Content-Security-Policy: default-src 'none'; sandbox`、`Cache-Control: private, max-age=3600`、`Cross-Origin-Resource-Policy: cross-origin`。所有回答都带 `X-Content-Type-Options: nosniff`。
- **事件**：`GET /events` 是 server-sent events，要带令牌，所以前端用 `fetch` 读，不用 `EventSource`。每条是 `event: <type>` 加 `data: <JSON>`，`data` 里有 `type` 和下面的字段。收到就重新拉对应的数据；落后丢掉的不补发。不受 30 秒的时限，守护进程要停时结束。

| `type` | 什么时候发 | 字段 |
|---|---|---|
| `item-added` | 收下了一条新的（粘贴、投递、收件文件夹、导入） | `id` |
| `item-updated` | 已经有的又被收下一次；取到了内容；记一句、主题、拿回来、只留文字、再取一次这类改了条目；回收站里只留文字移出的媒体被清除了 | `id` |
| `item-fetch-failed` | 这次没取到；`retry` 为真时过会儿自己再试 | `id`、`code`、`hint`（可省）、`retry` |
| `item-trashed` | 条目进了回收站 | `id` |
| `item-removed` | 回收站里的条目到期、或你「立即清除」，从资料库里移除了；再拉它是 404 | `id` |
| `topic-updated` | 建了主题，或改了关注 | `id` |
| `import-updated` | 一次导入变了：列了一页、认了一段身份、确认了、停下或接着、收下了一条、删掉了 | `id`（导入的） |
| `plugins-updated` | 插件的暂停变了（被限流或拦住、「现在再试」「先别试」）、站点的登录、访客身份、密钥设置变了 | 无 |

- **错误**：照 [§0.3](conventions.md#03-错误模型)：`application/problem+json`，字段 `type`（`urn:usuyo:problem:<code>`）、`title`、`status`、`detail`、`instance`（`urn:uuid:<随机>`）、`code`、`retryable`；插件报的另带 `plugin`，一律 502。可重试的带 `Retry-After: 5`。没有的路径 404 `not-found`，方法不对 405 `invalid-args`。
- **上限**：请求体 64 KiB，超了 413 `too-large`；握手 256 字节；登录和访客身份的 cookie 256 KiB；`/v1/batches` 4 MiB；每个请求 30 秒，超了 504 `timeout`，列来源（`GET /plugins/{owner}/{repo}/sources`）120 秒；搜索 `limit` 默认 50，限在 1 到 200。
- **日志**：守护进程和命令行的日志里，链接 `?`、`#` 以后的部分和 `Bearer ` 后面的字换成 `…`。

`endpoints.ts` 的 `paths` 和 `deliverPaths`（11-11）：

| 方法与路径 | 做什么 |
|---|---|
| `GET /health` | `{status, version, library}`，不拿资料库的锁 |
| `GET /map` | 地图 |
| `GET /topics`；`POST /topics` | 列出主题；按 `{name}` 建一个，同名的已经有了就回那个 |
| `POST /topics/{id}/follow` | `{followed}` |
| `POST /items` | 收下 `{input, kind}`，`kind` 现在只认 `link`；记 `via: app`、入口 `paste` |
| `GET /items/{id}` | 一条；在回收站里的也回，带 `trashedAt` |
| `POST /items/{id}/note` | `{text}` |
| `POST /items/{id}/topics` | `{topicId, member}` |
| `POST /items/{id}/trash`；`POST /items/{id}/restore` | 进回收站；整条拿回来 |
| `POST /items/{id}/text-only`；`POST /items/{id}/restore-media` | 只留文字；把移走的图和视频拿回来 |
| `POST /items/{id}/refetch` | 再取一次 |
| `GET /trash` | `{trashDays, entries, unreadable?}`：回收站里的整条条目（`reason` 是 `delete`、`undo`）和只留文字移出的媒体（`keep-text`），新的在前；每项 `{id, reason, itemId, title, platform, author?, trashedAt, expiresAt?}`，`id` 是 `trashId`；`trashDays` 是 `null` 时没有 `expiresAt`；读不了的项列进 `unreadable` |
| `POST /trash/{trashId}/purge` | 立即清除回收站里的一项，204 |
| `GET /search?q=&limit=` | 搜索 |
| `GET /events` | 事件流 |
| `GET /plugins` | 装着的获取插件（标签、设置、节奏、暂停）和没加载的（目录和原因） |
| `GET /plugins/{owner}/{repo}/sources` | 这个插件现在能列的来源；会调插件，计入节奏，要等的超过 60 秒就不等，回 422 `unsupported-input` 并说什么时候能再试 |
| `POST /plugins/{owner}/{repo}/try-now` | 「现在再试」：去掉这个插件所有的暂停（包括「先别试」），排着的任务和导入马上再看 |
| `POST /plugins/{owner}/{repo}/hold` | 「先别试」：到你点「现在再试」为止哪儿都不调它，别的插件照常 |
| `PUT /plugins/{owner}/{repo}/secrets/{key}`；`DELETE …` | 填一个密钥设置 `{value}`（空的回 400），存进钥匙串；`DELETE` 去掉。值从不回出来，回的是插件 |
| `GET /sites` | 装着的插件会用到的账号站点和登录状态 |
| `PUT /sites/{site}/login`；`DELETE …` | 壳的登录窗口交来 storageState，只留属于这个站点的，存进钥匙串，回的里面没有值；`DELETE` 忘掉登录，204 |
| `GET /visitors` | 装着的插件在清单里声明的访客身份，和有没有存着、最早什么时候过期；不出现在 `/sites` 里 |
| `PUT /visitors/{site}`；`DELETE …` | 同上，存成 `visitor:<站点>`，和同一主机的账号分开；`DELETE` 去掉，账号不动 |
| `GET /imports`；`POST /imports` | 全部导入（新的在前）；按 `{pluginId, source, title?}` 从一个插件的来源开始列 |
| `GET /imports/{id}`；`DELETE …` | 一次导入；不要了：清单和还没收的去掉，已经收下的留着，204 |
| `GET /imports/{id}/entries?standing=&offset=&limit=` | 预览里的条目；`standing` 取 `new`、`stored`、`trashed`、`unresolved`，不给就是全部；`limit` 默认 100，限在 1 到 500 |
| `POST /imports/{id}/confirm` | 确认，见下 |
| `POST /imports/{id}/hold`；`POST /imports/{id}/resume` | 先停下；接着导入 |
| `GET /deliver-tokens`；`POST /deliver-tokens`；`DELETE /deliver-tokens/{id}` | 见上「投递令牌」 |
| `GET /inbox` | `{path}`，收件文件夹在哪（[§1.1](library.md#11-目录树)） |
| `GET /v1/ping` | 投递：`{name: "usuyo", version, protocol: 1, client}`，`client` 是投递令牌的名字，主令牌是 `null` |
| `POST /v1/deliver` | 投递：见下 |
| `POST /v1/batches` | 投递：见下 |

守护进程另有四条，不在 `endpoints.ts` 的 `paths` 里：`POST /handshake`（上面）；`POST /reindex`，命令行用，回 `{items, unreadable}`；`GET /media/<itemId>/<层>`，地址由 `ItemView` 给；`GET /events` 在 `paths` 里，但不走 30 秒的时限。

**投递**（[协议 §5.1](../protocol/delivery.md#51-本机-http-接口)）：
- `POST /v1/deliver`，`{url?, text?}`，给一个（两个都给取 `url`）。`text` 是手机分享出来的整段文字，从里面找第一个链接。只收链接，已经取到的内容（multipart）还不收。回 202 `{id, state: "accepted", duplicate}`，`duplicate` 为真是以前收下过。条目的入口取令牌的 `kind`，取内容照常排任务。遇到私密条目不收、不加来源，现在的回答和「收下过」一样（`duplicate: true`），没有单独的 `skipped`。
- `POST /v1/batches`，`{name?, action?, links: [{url, title?, actedAt?}]}`，进导入的预览，你确认了才按节奏去收。`links` 1 到 5000 条；`action` 取 `favorite`、`like`、`share`、`view`、`unknown`；每条 `url` 要是带主机的 http(s)、不带用户名密码，`actedAt` 要是 RFC 3339，`title` 超过 500 字截掉；`name` 不给写「交进来的一批」，最多 200 字。不合的回 400 `invalid-args`，说是第几条。回 202 `{id}`，是新建的导入的 id（`kind: delivered`，`client` 是令牌名）。每条的身份由宿主重新算，不信交来的；认不出的这一条去掉，记日志；清单里身份相同的合成一条。

**导入**：一次导入的状态不单独存，由几项现算：`listing`（还在列）、`ready`（列完了，等你看一眼）、`taking`（确认了，还有排着的任务）、`done`（确认了，没有排着的了），`held` 盖过其余，是你点了「先停下」。走向：
1. 开始（`POST /imports` 或 `POST /v1/batches`）：`source` 类要这个插件能列（不能回 422 `unsupported-input`），来源非空、最多 512 字节，平台没声明 `batch: false`（声明了回 422）。`source` 类按平台的节奏一页页调插件的 `list`，`delivered` 类每轮认 50 条的身份。每条按身份分成 `new`、`stored`（库里已经有）、`trashed`（在回收站里）；认不出来的是 `unresolved`。这一步不建任何条目。
2. 列完是 `ready`。`POST /imports/{id}/confirm`，请求体 `{new, trashed, except?}`（不认识的字段回 400）：`new` 是「新的」整组收不收；`trashed` 是「回收站里的」整组收不收，收下就是拿回来；`except` 是 `seq` 的数组，这些条目和自己那组选得反着来（整组收的，它不收；整组不收的，它收）。只交例外：预览按页看，没翻到的条目跟着整组走。`stored` 和 `unresolved` 的不收，也不能出现在 `except` 里，出现了回 400。还在列的、或已经确认过的再确认，回 400。
3. 确认后，挑中的每条排一个取内容的任务，优先级低于你手动收下的；按平台的节奏一条条来。轮到时先收下（入口 `import`，`takenBy: user`，同一批共用 `importBatch`，带 `action`、`actedAt`），再取内容。这一条已经有任务（你刚手动收过）的不动。轮到时发现已经进了回收站、而你没挑回收站那一组的，不收，`skipped` 加一；私密条目批量导入照收（[§1.7](library.md#17-删掉) 的表）。`taken` 是收下的条数，`notStored` 是收下了但内容取不到的。
4. `POST /imports/{id}/hold` 随时可以，不再列、不再收，已经收下的不动；`resume` 从停的地方接着，停在要你处理的原因上（例如重新登录以后）也从这里再试。导入页只说事实（列了几页、各几条、收下几条、为什么在等和什么时候），不给还剩多少。

列的时候的错：联网、超时、插件崩了这一类，等 60 秒乘 2 的次方再试，试满 6 次就停下等你；被限流、被拦住记在插件的暂停上（见下），导入照它显示原因，暂停一去掉就接着列；别的错停在这次导入上，`waiting.code` 是错误码（例如 `login-required`，或页面把没有访客身份的请求送回首页时插件报的 `visitor-required`），没有 `until`，等你重新登录或取到访客身份以后点「接着导入」。`waiting.code` 还可能是 `pacing`（等间隔）、`daily-limit`（今天批量的次数用完了）、`held`（你让插件先别试）、`batch-off`（这个平台不做导入）。

**节奏和暂停**（[协议 §3.1](../protocol/fetch.md#31-运行器与节奏)）：
- 节奏来自装着的插件清单里的 `pacing`，同一平台的几个插件取最严的：间隔取最长的，`daily` 取最小的，有一个写了 `batch: false` 就不做导入。没声明的平台不等。
- `interval`：上一次调用结束到下一次开始的最短间隔，`fetch`、`list` 的一页、`sources` 都算，你手动收下的也守，导入的和手动的在同一个平台上排队。`daily`：批量（导入）的调用一个本地日期最多几次，到本地零点重新数；你手动收下的单条不计入，也不被它挡住，只守间隔。同一平台同一时间只跑一个调用。记账在 `jobs.db`，重启后照旧。`canonicalize` 不计。
- 列一页要等的超过 60 秒就不等，记下到点的时间，到点再来。
- 插件报 `rate-limited`：暂停到 `retryAfter` 以后（没给 15 分钟，最多算 1 小时）。报 `blocked`：现在也只暂停 1 小时，到点自己解开；你点「现在再试」更早去掉它。暂停只管这个插件在这个平台上（web 插件按主机），别的插件照常；暂停期间它不被调用，排着的任务往后排，不白白耗掉重试次数。`hold`（「先别试」）没有 `until`，是 `byYou`，一直到「现在再试」。暂停和它的错误码、一句话原因在 `GET /plugins` 的 `pauses` 里。

**命令行**：`add`、`get`、`search`、`reindex` 先读 `daemon.json` 和 `api-token`，握手，再 `GET /health`；都成了就经守护进程，不然在本进程里直接开库。握手没过不发令牌。`init` 总在本进程。

**和别处还不一样的**：窗口用的路径不带 `/v1/`（只有投递接口带）；列表回普通数组，没有游标（[§0.8](conventions.md#08-分页和游标)）；用主令牌经窗口用的路径做的写操作都记 `via: app`，`POST /items` 收下记 `entry: paste`、`takenBy: user`，私密条目照收，命令行经守护进程收下的也一样（[§1.7](library.md#17-删掉) 的表要求 `api` 和终端外的命令行跳过私密）；经 `/v1/deliver` 的投递，扩展、手机、`api` 令牌遇到私密条目会跳过，终端里和终端外的命令行（本进程里直接收下时记 `cli`）都不跳。
