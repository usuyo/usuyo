<a id="本机接口第-2-层11-11-照代码"></a>
## 本机接口（第 2 层，11-11 照代码）

守护进程（`usuyo daemon`，`crates/usuyo-api`）给桌面窗口、命令行、开发服务器和投递端（浏览器扩展、油猴脚本、手机）用的本机 HTTP 接口，照 11-12 的代码写。完整的第 2 层（MCP、OpenAPI、搜索语法）还没写。**路径和类型以 `apps/desktop/web/src/api/endpoints.ts` 为准**：改接口先改这个文件，Rust 那边照它做。

- **监听**：只绑 `127.0.0.1`，端口固定 47615（`--port`、`USUYO_PORT` 能改，但窗口的 CSP 只放行 47615）。一个监听口，窗口用的路由和投递接口（`/v1/…`）都在上面，不另开端口。端口被占时守护进程起不来，已有的 `api-token`、`daemon.json` 不动；壳不再起第二个，窗口里说端口被别的程序占着。
- **谁能调什么**：每个请求过一道门（`guard.rs`），分三种调用方。
  - 带主令牌的（`api-token` 里的那个：壳、命令行、开发代理、别的本机程序）：所有路由，包括 `/v1/…`；经 `/v1/deliver` 收下的记入口 `api`。
  - 带投递令牌的：只能进 `GET /v1/ping`、`POST /v1/deliver`、`POST /v1/batches`、`POST /v1/file-imports`，名单以外一律 401 `unauthorized`，和没带令牌一样。
  - 不带令牌也放行的：CORS 预检（只限窗口用的路径）、`POST /handshake`、带对的 `k` 的 `/media/<itemId>/…`。
- **令牌**：除了上面三种，每个请求都带 `Authorization: Bearer <令牌>`，`/health`、`/events` 也要。主令牌是 32 字节随机数的小写十六进制，每次启动新生成，绑上端口以后写进按机器目录的 `api-token`（0600，先写临时文件再改名），正常退出时连同 `daemon.json` 删掉（只删还是自己写的那份）。按定长时间比对；错了回 401 `unauthorized`。
- **`Host`**：最先查，只认 `127.0.0.1:<端口>`、`localhost:<端口>`、`[::1]:<端口>`（不分大小写），别的回 403 `forbidden`。
- **`Origin`**：只当跨站过滤，谁都能伪造，不用来放行。没带的照常查令牌（命令行、开发代理、油猴脚本管理器常不带）；带了的按路径分：
  - 窗口用的路径：只认 Tauri 的 webview，`tauri://localhost`、`http://tauri.localhost`。
  - `/v1/…`：只认浏览器扩展的来源，以 `chrome-extension://`、`moz-extension://`、`safari-web-extension://` 开头的；窗口的来源、普通网页、`null` 都回 403。
  - `/handshake`：扩展的来源和窗口的来源都认（扩展发令牌以前要先确认对面是内核）。
  - 不合的回 403 `forbidden`，令牌对也一样。开发服务器的来源不在名单里：Vite 代理只接自己页面的同源请求，在服务端带上令牌，去掉 `Origin`、`Referer`。
  - CORS 头只给窗口的来源（允许 GET、POST、PUT、PATCH、DELETE，请求头 `Content-Type`、`Authorization`、`Range`，暴露 `Content-Range`、`Accept-Ranges`、`Retry-After`）。`/v1/…` 不回 CORS 头，也不答 CORS 预检（预检不带令牌，落到 401）。
- **握手**：客户端先确认端口上是这次启动的守护进程，再发令牌。`POST /handshake`，请求 `{"nonce": "<64 位小写十六进制>", "client": "<投递令牌的 id>"}`，`client` 可省（请求体最多 256 字节，不合格回 400 `invalid-args`），回 `{"proof": "<64 位小写十六进制>"}`：以密钥的 UTF-8 字节为密钥，对 `usuyo-daemon-proof/v1\n127.0.0.1:<端口>\n<nonce>` 做 HMAC-SHA256。不给 `client` 时密钥是主令牌；给了，密钥是那个投递令牌的整个 `<id>.<秘密>`（投递端只发 id，令牌本身不上线路）。不认识的 id 也回一个证明，用随机的密钥，对面分不出那个 id 在不在、撤没撤。端口写进消息，换个端口转来的回答对不上。核对的是客户端，用自己读到的令牌：壳（找守护进程时和每次重新连时）、命令行、Vite 代理（令牌文件变了就重新握手，对不上回 502，不把令牌发过去）、投递端（油猴脚本核对不过就什么都不发）。
- **投递令牌**：格式 `<id>.<秘密>`，`id` 是 16 位小写十六进制（不是秘密），`秘密` 是 64 位小写十六进制。主令牌调 `POST /deliver-tokens`（`{name, kind}`，`name` 1 到 80 个字，`kind` 取 `extension`、`mobile`、`api`）发一个，整个令牌只在这一次的回答里出现（在 `{id, name, kind, createdAt}` 之外多一项 `token`）；`GET /deliver-tokens` 列出投递端，没有令牌；`DELETE /deliver-tokens/{id}` 撤销，马上生效，回 204，没有这个 id 回 404。存在按机器目录的 `deliver-tokens.json`，见 [§1.1](library.md#11-目录树)。投递令牌的 `kind` 决定条目 `origins[].entry`：`extension`、`mobile`、`api`。
- **媒体**：`<img>`、`<video>` 带不了头，`ItemView` 里的媒体地址是 `/media/<itemId>/<层>?k=<key>`。层是 `image/<imageKey>`、`video`（有合起来的 `play.mp4` 时给它，不然给原样的画面）、`audio`、`frame/<runId>/<毫秒>`（那次 `keyframes` 转录的一帧）；有 `play.mp4` 时画面的地址多一个 `v=<那次 play 的 runId>`，合起来的那份落地以后换了地址，不拿缓存里的原样。`k` 是 HMAC-SHA256 的十六进制，密钥每次启动另生成（和令牌无关），消息是 `usuyo-media/v1\n<itemId>`，只对这一条有效。支持 Range。`Content-Type` 只回允许的几种，别的写 `application/octet-stream`；另带 `Content-Security-Policy: default-src 'none'; sandbox`、`Cache-Control: private, max-age=3600`、`Cross-Origin-Resource-Policy: cross-origin`。所有回答都带 `X-Content-Type-Options: nosniff`。
- **事件**：`GET /events` 是 server-sent events，要带令牌，所以前端用 `fetch` 读，不用 `EventSource`。每条是 `event: <type>` 加 `data: <JSON>`，`data` 里有 `type` 和下面的字段。收到就重新拉对应的数据；落后丢掉的不补发。不受 30 秒的时限，守护进程要停时结束。

| `type` | 什么时候发 | 字段 |
|---|---|---|
| `item-added` | 收下了一条新的（粘贴、投递、收件文件夹、导入） | `id` |
| `item-updated` | 已经有的又被收下一次；取到了内容；记一句、主题、拿回来、只留文字、再取一次这类改了条目；回收站里只留文字移出的媒体被清除了；一样转换开始转、转好了、等模型、没转出来、不用转了；等着的模型到了；`transforms.auto` 关上时删掉了自动排的 | `id` |
| `item-fetch-failed` | 这次没取到；`retry` 为真时过会儿自己再试 | `id`、`code`、`hint`（可省）、`retry` |
| `item-trashed` | 条目进了回收站 | `id` |
| `item-removed` | 回收站里的条目到期、或你「立即清除」，从资料库里移除了；再拉它是 404 | `id` |
| `topic-updated` | 建了主题，或改了关注 | `id` |
| `import-updated` | 一次导入变了：列了一页、认了一段身份、确认了、停下或接着、收下了一条、删掉了 | `id`（导入的） |
| `plugins-updated` | 插件的暂停变了（被限流或拦住、「现在再试」「先别试」）、开关了一种方法、站点的登录、访客身份、密钥设置、设置里的 `network` 变了 | 无 |
| `transforms-updated` | 模型开始下、下好了、没下成、删掉了；设置里的 `transforms.auto` 变了 | 无 |
| `phone-updated` | 手机的驱动开始装、装好、没装上、卸掉；剪贴板试出了结果。授权、镜像、adb 的变化不发，窗口回到前面时和页面看得见时隔一会儿重拉 | 无 |

- **错误**：照 [§0.3](conventions.md#03-错误模型)：`application/problem+json`，字段 `type`（`urn:usuyo:problem:<code>`）、`title`、`status`、`detail`、`instance`（`urn:uuid:<随机>`）、`code`、`retryable`、`hint?`；插件报的另带 `plugin`，一律 502。`hint` 是给人看的一句话，照设置的语言（下「设置」的 `language`）写：插件给了就是它的（插件照 `USUYO_LOCALE` 写）；内核自己报的，列来源、开始导入、翻预览、确认、从文件导入、交内容这几条的 400、422（还有找不到读文件的插件的 404、文件太大的 413）都带，这时 `detail` 是英文，留给日志。内核自己的句子两种语言都在各 crate 的 `words.rs` 里，写出 problem+json 时照这次请求进来时的语言挑；错误码不翻译。视图里内核写的话（`blockers[].hint`、`pauses[].hint`、`pending[].hint`、`remark`、`missing`、模型和驱动的 `hint`、`driver.step.text`）同样照这个语言；`jobs.db` 里宿主自己记下的原因两种语言都存，换了语言，读出来就是新的。可重试的带 `Retry-After: 5`。没有的路径 404 `not-found`，方法不对 405 `invalid-args`。
- **上限**：请求体 64 KiB，超了 413 `too-large`；握手 256 字节；登录和访客身份的 cookie 256 KiB；`/v1/batches` 4 MiB；交文件 `POST /file-imports` 16 GiB、`POST /v1/file-imports` 256 MiB（内核边收边数，超了 413 带 `hint`，写了一半的副本去掉）；每个请求 30 秒，超了 504 `timeout`，列来源（`GET /plugins/{owner}/{repo}/sources`）120 秒，交文件 30 分钟；搜索 `limit` 默认 50，限在 1 到 200。
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
| `POST /items/{id}/transform` | 见下「转换」；回 202 `{queued: true}` |
| `GET /trash` | `{trashDays, entries, unreadable?}`：回收站里的整条条目（`reason` 是 `delete`、`undo`）和只留文字移出的媒体（`keep-text`），新的在前；每项 `{id, reason, itemId, title, platform, author?, trashedAt, expiresAt?}`，`id` 是 `trashId`；`trashDays` 是 `null` 时没有 `expiresAt`；读不了的项列进 `unreadable` |
| `POST /trash/{trashId}/purge` | 立即清除回收站里的一项，204 |
| `POST /trash/release-held` | 「照时间移除」：时钟守卫拦下的回收站项照时间移除（[§1.7](library.md#17-删掉)），回 `{removed}`（移除了几项） |
| `GET /search?q=&limit=` | 搜索 |
| `GET /events` | 事件流 |
| `GET /plugins` | 装着的获取插件和没加载的（目录和原因），见下「获取插件」。会先问一遍各插件过期了的本机检查（`health`，不带凭据、不碰平台，几个插件同时问） |
| `GET /plugins/{owner}/{repo}/sources?method=&link=` | 这个插件一种方法现在能列的来源，每项 `{source, title, action?, count?, method}`；`method` 必给，`link` 只给 `list` 写了 `link` 的方法（一条链接，或带链接的分享文字，照下面「链接从哪来」取；http(s)、带主机、不带用户名密码、至多 4096 字节，不合的回 400 `invalid-args`）。这种方法不能列、没写 `link` 却给了链接、这台机器上用不上，回 422 `unsupported-input`。会调插件，计入节奏，要等的超过 60 秒就不等，回 422 并说什么时候能再试 |
| `PUT /plugins/{owner}/{repo}/methods/{method}`；`DELETE …` | `{enabled}`：开或关一种方法，写进按机器的 `settings.json`（[按机器的目录](machine.md#按机器的目录)），马上生效，回整个插件。`DELETE` 去掉拨过的开关，这种方法回到各平台各自那一层的默认（`perPlatform` 抬到 `private-api` 的平台关着，别的平台开着；一个 `PUT` 做不到），回整个插件。没做的方法回 422 `unsupported-input`，没有这个插件或方法回 404 |
| `GET /plugins/{owner}/{repo}/methods/{method}/script` | `{text}`：这种方法清单里 `script` 写的脚本文件的全文，给「复制脚本」。只认装着的清单写了的那个文件，解开链接以后要还在那个插件的文件夹里、是个普通文件，至多 4 MiB；路径不从请求里来。没有这个插件、方法，方法没写 `script`，文件不在、跑到文件夹外面，都回 404 |
| `POST /plugins/{owner}/{repo}/methods/{method}/script/reveal` | 在系统的文件管理器里显示同一个文件（macOS `open -R`，Windows 资源管理器选中它，Linux `xdg-open` 它所在的文件夹），204；找文件同上，回 404；文件管理器 10 秒没回话或出错回 500 |
| `POST /plugins/{owner}/{repo}/try-now` | 「现在再试」：去掉这个插件所有的暂停（包括「先别试」），排着的任务和导入马上再看 |
| `POST /plugins/{owner}/{repo}/hold` | 「先别试」：到你点「现在再试」为止哪儿都不调它，别的插件照常 |
| `PUT /plugins/{owner}/{repo}/secrets/{key}`；`DELETE …` | 填一个密钥设置 `{value}`（空的回 400），存进钥匙串；`DELETE` 去掉。值从不回出来，回的是插件 |
| `GET /sites` | 装着的插件会用到的账号站点和登录状态 |
| `PUT /sites/{site}/login`；`DELETE …` | 壳的登录窗口交来 storageState，只留属于这个站点的，存进钥匙串，回的里面没有值；`DELETE` 忘掉登录，204 |
| `GET /visitors` | 装着的插件在清单里声明的访客身份，和有没有存着、最早什么时候过期；不出现在 `/sites` 里 |
| `PUT /visitors/{site}`；`DELETE …` | 同上，存成 `visitor:<站点>`，和同一主机的账号分开；`DELETE` 去掉，账号不动 |
| `GET /imports`；`POST /imports` | 全部导入（新的在前）；按 `{pluginId, method, source, title?}` 从一个插件一种方法的来源开始列。导入的视图多一项 `method`；`kind` 是 `source`、`delivered`、`file`，`file` 类多一项 `file: {name, reader}`（交来的文件名、读它的插件的名字） |
| `GET /imports/{id}`；`DELETE …` | 一次导入；不要了：清单和还没收的去掉，已经收下的留着，204 |
| `GET /imports/{id}/entries?standing=&offset=&limit=` | 预览里的条目；`standing` 取 `new`、`stored`、`trashed`、`unresolved`，不给就是全部；`limit` 默认 100，限在 1 到 500；从文件导入时只有插件的 `ref`、没有链接的条目没有 `url` |
| `POST /imports/{id}/confirm` | 确认，见下 |
| `POST /imports/{id}/hold`；`POST /imports/{id}/resume` | 先停下；接着导入 |
| `GET /deliver-tokens`；`POST /deliver-tokens`；`DELETE /deliver-tokens/{id}` | 见上「投递令牌」 |
| `GET /inbox` | `{path}`，收件文件夹在哪（[§1.1](library.md#11-目录树)） |
| `GET /file-readers` | 能读文件的方法（做了的、`list` 写了 `files` 的），照插件和清单的顺序，每项 `{id, name, plugin, reads, patterns, lists, note?}`：`id` 是 `<插件 id>#<方法 id>`，`name` 是方法的名字，`plugin` 是插件的名字，`reads` 是清单的 `list.reads`，`patterns` 是 `list.files`，`lists` 是 `list.sources` 的名字，`note` 是方法的 `risk.text` |
| `POST /file-imports?reader=&name=` | 从文件导入（[协议 §3.9](../protocol/fetch.md#39-从文件导入)）：请求体是文件本身（任何 `Content-Type`），`reader` 是 `GET /file-readers` 的 `id`，`name` 是文件名，只当显示的字。那种方法要能读文件、开着、用得上（不然 422 `unsupported-input`，插件不在 404），空文件 400。副本存进按机器目录的 `imports/<导入 id>/input[.扩展名]`，马上列第一页：读不了回 400 `invalid-args`，`hint` 是插件的那句话，什么都不留；读得了回 200 `ImportView`（`kind: file`，`state` 是 `listing` 或 `ready`），之后照下面「导入」看预览、确认 |
| `GET /transforms` | `{plugins, auto}`：装着的转换插件，每个 `{id, name, version, jobs, bundled?, models, health, hint?}`；`jobs` 是它能做的 `play`、`asr`、`keyframes`、`ocr`、`frames`；`bundled` 是清单 `upstream` 的 `{name, license}`；`health` 是插件自己报的 `ok` 或 `unavailable`（`hint` 是没过的检查；它声明的模型还没下时只写「需要先下载模型」），几个插件同时问，正在转一条的插件不等它、算 `ok`；`models` 每个 `{name, size, license, state, hint?}`，`state` 是 `absent`、`downloading`、`present`、`failed`（`hint` 是一句没下成的原因）。`auto` 是 `transforms.auto` |
| `POST /transforms/{owner}/{repo}/models/{name}/download` | 下这个模型（[§1.1](library.md#11-目录树) 按机器目录的 `models/`）：已经在了回 200 `{state: "present"}`，开始下或已经在下回 202 `{state: "downloading"}`；插件或模型不在回 404。下完以后等它的转换回到排着，那些条目发 `item-updated` |
| `POST /transforms/{owner}/{repo}/models/{name}/remove` | 删掉这个模型，204；正在下回 409 `conflict`。插件的进程在后台关掉，之后要它的转换等模型 |
| `GET /settings`；`PATCH /settings` | 见下「设置」 |
| `GET /phone` 和 `/phone/…` 七条 | 见下「手机」 |
| `GET /v1/ping` | 投递：`{name: "usuyo", version, protocol: 1, client}`，`client` 是投递令牌的名字，主令牌是 `null` |
| `POST /v1/deliver` | 投递：见下 |
| `POST /v1/batches` | 投递：见下 |
| `POST /v1/file-imports?reader=&name=` | 投递：交一个装着已取到内容的文件，见下 |

**转换**（`POST /items/{id}/transform`）：请求体二选一，别的字段回 400 `invalid-args`。
- `{jobs: "pending"}`：条目页打开时调。这一条还没做好的转换都挪到最前（点名），在电池上、`transforms.auto` 关着时也转。
- `{job, plugin?, options?}`：再转一次，或换一个插件、换一组参数转；`job` 是 `play`、`asr`、`keyframes`、`ocr`、`frames`。做好了也再做一次，新的和旧的并排，`redoOf` 指现在最新的那一次。这一样正在转回 409 `conflict`；没有这种 `job`、`plugin` 做不了它、没装能做它的插件回 400 `invalid-args`；只留文字以后回 422 `unsupported-input`；条目不在回 404。

`GET /items/{id}` 的 `facets` 一层可以有几份，每份带 `key`（在一条里唯一）；`asr`、`ocr`、`frames` 三层每份带 `source`（`by: platform` 是平台字幕，`by: machine` 带 `tool`、`plugin`（做这一份的转换插件 id，即 `run.json` 的 `by`；宿主自己判的没有）、`runId`、`at`；有语言时 `lang`），段带 `position`。口播是每份平台字幕和每次 `asr` 转录，平台的在前，机器的新的在前；`ocr` 是每次图里的字的转录，`frames` 是每次关键帧上的字的转录，连着几帧一样的一行合成一段；`video` 的 `frames` 是最新一次 `keyframes` 的帧。`remark` 只写量出来的事：「没有音轨」「未检测到语音」「画面几乎不变」「未识别到文字」「有重复的图片」（照设置的语言）。`pending` 是还没转出来的那几面，一面一句：`layer` 是 `asr`、`ocr`、`frames`、`video`（`play`、`keyframes` 都算画面），`state` 是 `now`（在转）、`needs-model`（带 `model: {plugin, name, size, license}`）、`failed`（带 `hint`，例如「这台电脑上没有可转写的插件」）、`on-power`（在电池上，接上电源时转）、`later`（排着），同一面有几行时取前面那个。没有计数。`GET /search` 命中 `asr`、`ocr`、`frames` 时带 `position`：口播和画面上的字是 `{kind: "time", sec, endSec?, snapshotId, xywh?}`，图里的字是 `{kind: "image", imageKey, xywh}`。

**设置**：
- `GET /settings` 回 `{library, trashDays, trashHeld?, network: {mode, proxy?, fakeIp}, transforms: {auto}, window: {systemTitleBar}, language, logs}`。`window.systemTitleBar` 是按机器的 `settings.json` 的同名项，没写是 `false`。`language` 是同一个文件的 `language`：`system`（没写时，跟随系统：守护进程启动时看系统的首选语言，中文开头的用简体中文，别的用英文）、`zh-Hans` 或 `en`。`trashDays` 是资料库的（[§1.1](library.md#11-目录树)），`null` 是一直；`trashHeld: {count, since}` 只在时钟守卫现在拦着到期的项时有，`since` 是记下的最晚时间；`network.mode` 是 `system`、`direct`、`custom`，`custom` 的 `proxy` 不带密码；`fakeIp` 是这台电脑开着假 IP 模式的代理；`logs` 是按机器目录的 `logs/daemon.log`。
- `PATCH /settings`，`{trashDays?, network?: {mode, proxy?}, transforms?: {auto}, window?: {systemTitleBar}, language?}`，给了的几项才改，改好回整份，马上生效。`trashDays` 取 30、90 或 `null`，记一次资料库的 `set-setting`；`network` 写进按机器的 `settings.json` 的 `network.proxy`（`system`、`direct` 或代理地址，`custom` 才带 `proxy`，地址是 `http://`、`socks5://`、`socks5h://`，有主机、没有路径），之后的插件调用和模型下载照它走；`transforms.auto` 关上时自动排的转换（没在跑的）都删掉，打开时照启动那样补排；`window.systemTitleBar` 只写进文件，桌面壳下次开窗口时读（[按机器的目录](machine.md#按机器的目录)）；`language` 马上生效：之后的 `hint` 和视图里的话照新的语言，之后起的插件进程拿到新的 `USUYO_LOCALE`（已经在跑的转换插件进程留着起来时的那个，闲下来关掉、下次再起时才换），桌面壳的话（连不上服务、登录窗口）每次现读这个文件。不认识的字段、不合的值回 400 `invalid-args`，原因里不写代理地址。`settings.json` 里别的键原样留着；这个文件只有自己能读（0600，先写同样 0600 的临时文件再改名），读回来的代理地址不带密码，交回一个不带密码、别处一样的地址时留着存着的密码。

守护进程另有四条，不在 `endpoints.ts` 的 `paths` 里：`POST /handshake`（上面）；`POST /reindex`，命令行用，回 `{items, unreadable}`；`GET /media/<itemId>/<层>`，地址由 `ItemView` 给；`GET /events` 在 `paths` 里，但不走 30 秒的时限。

<a id="手机"></a>
**手机**（[手机那条路 §12](../platforms/phone.md#12-手机页做到的11-12-代码)）：只在 macOS 上的守护进程有，别处回 404 `not-found`（`hint`「此版本的 usuyo 服务不支持手机。」）。
- `GET /phone` 回 `PhoneView`，现读：macOS 版本（`sw_vers`）、有没有 `/System/Applications/iPhone Mirroring.app`、驱动、两项授权、镜像、上一次剪贴板试验、adb 和设备。`ios.driver.step?: {text, commands}` 是要你自己在终端里做的那一步（登记、重新登记或停掉「mirroir serve」）。`driver.state` 没装上是 `failed`，`hint` 是一句原因；换新没成时 `state` 还是 `present`（旧的那份在），也带 `hint`。
  - 看这一页不起任何程序：`android.adb` 是 `{path, version, running}`，`adb version` 不连 adb 的服务；`running` 是 adb 自己的服务（127.0.0.1:5037）这会儿在不在听，只连一下这个口子，不起它。在跑时经这个口子问一次 `host:devices-l` 列 `devices`，带 `android.step`（服务留在后台，`'<adb>' kill-server` 停掉它）；不在跑时 `devices` 是空的，不跑 `adb devices`。
  - 「mirroir serve」的令牌只发给认过的那一条连接：先看令牌文件在不在（不读），再连上服务地址，用 `lsof` 找这条连接的另一头在哪个进程手里，要正好一个、是这个守护进程的用户，而且是 launchd 里登记的 `dev.mirroir.serve` 的那个 `pid`，或程序就是按机器目录里装着的 `mirroir-mcp`，才读令牌，请求在这同一条连接上发；另开的连接从头再认。看不见的（别的用户的进程）算认不出。认不出来时什么都不发，授权是 `unknown`，镜像照 `pgrep` 看，当它没在跑（装着驱动时 `step` 是登记那一步）。
- `POST /phone/ios/driver/install`、`…/update`：已经是钉的那一版就直接回；不然马上回 `installing`，在后台下，装好或没装上发 `phone-updated`。这台机器没有对应架构的包时回 422 `unsupported-input`。`…/remove`：删掉按机器目录里的驱动，回 `PhoneView`；正在装回 409 `conflict`。不停「mirroir serve」、不删它的登录项，也不动系统设置里的授权；它还在跑时 `step` 是停掉它的那一条。装和换新也不登记「mirroir serve」，那一步在 `step` 里交给你。
- `POST /phone/open-settings`，`{pane}`（`screen-recording`、`accessibility`、`iphone-mirroring`，别的字段回 400），204；打不开回 500 `internal`。
- `POST /phone/ios/clipboard-test`：马上回 `waiting`（`waitSec` 是 10），有结果发 `phone-updated`；还在等时再点回同一次。读不出 `changeCount` 回 500。
- `POST /phone/android/devices`：你点「找一下连着的手机」时跑一次 `adb devices -l`（10 秒），adb 的服务没在跑时它会起来、留在后台；回 `PhoneView`，`android` 是这次读到的设备，`running` 为真，`step` 说怎么停掉它。不发 `phone-updated`。没找到 adb 回 424 `dependency-missing`（「这台电脑上没找到 adb。」）。守护进程别处都不起 adb。
- `POST /phone/ios/dry-run`：回 `PhoneDryRun`。「mirroir serve」不在（没有令牌文件、连不上）回 424 `dependency-missing`；它的工具报错（镜像没连上、没授权）也是 424；回话看不懂（令牌不对、不是 JSON）回 503 `unavailable`；超时 504。

**链接从哪来**：你给的链接都可以是 App 的整段分享文字。整段就是一条 http(s) 链接就用它；不然取文字里第一段 `http://`、`https://` 开头的（不分大小写），到第一个空白、控制字符、非 ASCII 字符、引号或尖括号为止，末尾的 `.,;:!?)]}` 去掉；之后照常核 http(s)、主机、用户名密码、长度。没有链接的回错，带一句 `hint`；有几条时只看第一条，第一条没收下时 `hint` 里说「只检查了第一个链接」。`POST /items` 的 `input`、`POST /v1/deliver` 的 `url` 和 `text`、`POST /v1/batches` 每条的 `url`、列来源的 `link`、收件文件夹的 `.txt` 都照它（`usuyo-core` 的 `link_in`），插件拿到的总是一条干净的链接。

**投递**（[协议 §5.1](../protocol/delivery.md#51-本机-http-接口)）：
- `POST /v1/deliver`，`{url?, text?}`，给一个（两个都给取 `url`）。`text` 是手机分享出来的整段文字，从里面找第一个链接。只收链接，已经取到的内容（multipart）还不收。回 202 `{id, state: "accepted", duplicate}`，`duplicate` 为真是以前收下过。条目的入口取令牌的 `kind`，取内容照常排任务。遇到私密条目不收、不加来源，现在的回答和「收下过」一样（`duplicate: true`），没有单独的 `skipped`。
- `POST /v1/file-imports?reader=&name=`：浏览器脚本交它在页面里读到的内容，装在一个文件里，请求体就是文件。只认 `extension` 类投递令牌和主令牌，`mobile`、`api` 类回 403 `forbidden`。照 `POST /file-imports` 落成一次 `file` 类导入（`client` 是令牌名），回 202 `{id}`，只有 id：投递令牌读不了导入，你在导入页上看、确认。读不了的文件回 400 `invalid-args` 带 `hint`。
- `POST /v1/batches`，`{name?, action?, links: [{url, title?, actedAt?}]}`，进导入的预览，你确认了才按节奏去收。`links` 1 到 5000 条；`action` 取 `favorite`、`like`、`share`、`view`、`unknown`；每条 `url` 要是带主机的 http(s)、不带用户名密码，`actedAt` 要是 RFC 3339，`title` 超过 500 字截掉；`name` 不给写「交进来的一批」，最多 200 字。不合的回 400 `invalid-args`，说是第几条。回 202 `{id}`，是新建的导入的 id（`kind: delivered`，`client` 是令牌名）。每条的身份由宿主重新算，不信交来的；认不出的这一条去掉，记日志；清单里身份相同的合成一条。

**获取插件**（`GET /plugins` 的 `plugins[]`，[协议 §2.3.1](../protocol/manifest.md#231-方法-methods)）：`{id, name, description?, version, platforms, access, login, canList, sources, sites, visitors, settings, pacing, upstream?, pauses, methods}`。`upstream` 是清单 `upstream` 的 `{name, license}`，没写时不给；`GET /plugins` 另带 `failureLog`，失败日志 `logs/fetch-failures.jsonl` 在这台电脑上的路径。`access`、`login` 是做了的方法里最高的；`canList` 是有做了的方法能列；`sources` 是清单里写死的来源，每项带 `method`；`sites` 是做了的方法用到的登录站点；`pauses` 是所有方法的暂停，每项 `{method, scope, byYou, until?, code?, hint?}`，你点的「先别试」`method`、`scope` 都是 `*`。`methods` 是清单里每一种方法，照清单的顺序，没做的也在：
- `{id, name, access, login, account: {kind, sites?, where?}, interaction, drives?, perPlatform?, gets: {fetch?, list?, link?, delivers?}, built, enabled, usable, blockers, lastCall?, pauses, riskNote?, script?}`。
- `account.kind` 是清单的 `account`（`none`、`visitor`、`own-browser`、`own-phone`、`own-app`、`in-usuyo`、`developer`）；`sites` 是 `in-usuyo` 的登录站点或 `visitor` 的访客身份站点；`where` 是账号所在的地方给人看的名字：`visitor` 是访客身份的名字，`in-usuyo`、`own-browser`、`own-phone`、`own-app`（做没做都算）是方法列的站点的名字，没列站点时是插件的名字；`none`、`developer` 没有。
- `drives` 是清单的 `drives`：`iphone`、`android`、`desktop-app`，手机页照它说要哪种手机。
- `perPlatform` 是层级和方法本身不同的平台（清单的 `perPlatform`），每项 `{platform, access, login, enabled}`；`enabled` 是这种方法在那个平台上开没开：没拨过开关时照那一层的默认（[协议 §2.3.1](../protocol/manifest.md#231-方法-methods)），拨过就和方法的 `enabled` 一样。方法的 `enabled` 照方法本身那一层。没有这样的平台时不给。
- `gets.fetch` 是输出的词表（`text`、`images`、`media:video`…），`gets.list` 是清单里写死的来源名，`gets.link` 是来源从链接开始时链接框上的那句，`gets.delivers` 是 `links`。没做的方法清单里写了能力的也给。
- `usable` 是这台机器上现在用得上；`blockers` 是用不上的原因，每项 `{code, hint}`，`code` 取 `not-built`、`off`、`login-missing`、`login-expired`、`visitor-missing`、`visitor-expired`、`check-failed`（插件的本机检查没过）、`health-failed`（插件没能自查）、`paused`（停着：被限流、被拦，`hint` 写到几点，例如「平台提示请求过于频繁。11月12日 15:30前不会使用。」；或你点的「暂停」，写到你点「立即重试」为止；通用网页一类按网站停的不算），`hint` 照设置的语言写，名字取自清单。
- `lastCall` 是上一次调用 `{at, ok, code?, hint?}`，没调过就没有；`pauses` 是这种方法的暂停加上「先别试」；`riskNote` 是清单的 `risk`；`script` 只在清单写了 `script` 的方法上有，`{path, present}`：文件在这台电脑上的路径，`present` 是它在插件文件夹里、是个普通文件。

**导入**：一次导入的状态不单独存，由几项现算：`listing`（还在列）、`ready`（列完了，等你看一眼）、`taking`（确认了，还有排着的任务）、`done`（确认了，没有排着的了），`held` 盖过其余，是你点了「先停下」。走向：
1. 开始（`POST /imports`、`POST /v1/batches`、`POST /file-imports` 或 `POST /v1/file-imports`）：`file` 类和 `source` 类一样一页页调插件的 `list`，只是读本机的副本，不守节奏，第一页在交的时候就列（见上）。`source` 类要这个插件的这种方法能列、开着、这台机器上用得上（不然回 422 `unsupported-input`，原因写明缺什么），来源非空、最多 512 字节，平台没声明 `batch: false`（声明了回 422）。`source` 类按平台的节奏一页页调插件的 `list`，`delivered` 类每轮认 50 条的身份。每条按身份分成 `new`、`stored`（库里已经有）、`trashed`（在回收站里）；认不出来的是 `unresolved`。这一步不建任何条目。
2. 列完是 `ready`。`POST /imports/{id}/confirm`，请求体 `{new, trashed, except?}`（不认识的字段回 400）：`new` 是「新的」整组收不收；`trashed` 是「回收站里的」整组收不收，收下就是拿回来；`except` 是 `seq` 的数组，这些条目和自己那组选得反着来（整组收的，它不收；整组不收的，它收）。只交例外：预览按页看，没翻到的条目跟着整组走。`stored` 和 `unresolved` 的不收，也不能出现在 `except` 里，出现了回 400。还在列的、或已经确认过的再确认，回 400。
3. 确认后，挑中的每条排一个取内容的任务，优先级低于你手动收下的；按平台的节奏一条条来。轮到时先收下（入口 `import`，`takenBy: user`，同一批共用 `importBatch`，带 `action`、`actedAt`），再取内容。这一条已经有任务（你刚手动收过）的不动。轮到时发现已经进了回收站、而你没挑回收站那一组的，不收，`skipped` 加一；私密条目批量导入照收（[§1.7](library.md#17-删掉) 的表）。`taken` 是收下的条数，`notStored` 是收下了但内容取不到的。
4. `POST /imports/{id}/hold` 随时可以，不再列、不再收，已经收下的不动；`resume` 从停的地方接着，停在要你处理的原因上（例如重新登录以后）也从这里再试。导入页只说事实（列了几页、各几条、收下几条、为什么在等和什么时候），不给还剩多少。

列的时候的错：联网、超时、插件崩了这一类，等 60 秒乘 2 的次方再试，试满 6 次就停下等你；被限流、被拦住记在这种方法的暂停上（见下），导入照它显示原因，暂停一去掉就接着列；别的错停在这次导入上，`waiting.code` 是错误码（例如 `login-required`，或页面把没有访客身份的请求送回首页时插件报的 `visitor-required`），没有 `until`，等你重新登录或取到访客身份以后点「接着导入」。`waiting.code` 还可能是 `pacing`（等间隔）、`daily-limit`（今天批量的次数用完了）、`held`（你让插件先别试）、`batch-off`（这个平台不做导入）。

**节奏和暂停**（[协议 §3.1](../protocol/fetch.md#31-运行器与节奏)）：
- 节奏来自装着的插件清单里的 `pacing`，同一平台的几个插件取最严的：间隔取最长的，`daily` 取最小的，有一个写了 `batch: false` 就不做导入。没声明的平台不等。
- `interval`：上一次调用结束到下一次开始的最短间隔，`fetch`、`list` 的一页、`sources` 都算，你手动收下的也守，导入的和手动的在同一个平台上排队。`daily`：批量（导入）的调用一个本地日期最多几次，到本地零点重新数；你手动收下的单条不计入，也不被它挡住，只守间隔。同一平台同一时间只跑一个调用。记账在 `jobs.db`，重启后照旧。`canonicalize` 不计。
- 列一页要等的超过 60 秒就不等，记下到点的时间，到点再来。
- 插件报 `rate-limited`：这种方法暂停到 `retryAfter` 以后（没给 15 分钟，最多算 1 小时）。报 `blocked`：现在也只暂停 1 小时，到点自己解开；你点「现在再试」更早去掉它。暂停只管这个插件的这种方法在这个平台上（web 插件按主机），别的方法、别的插件照常；暂停期间它不被调用，排着的任务往后排，不白白耗掉重试次数。`hold`（「先别试」）没有 `until`，是 `byYou`，一直到「现在再试」。暂停和它的错误码、一句话原因在 `GET /plugins` 的 `pauses` 和各方法的 `pauses` 里。每次调插件失败，另记一行失败日志（[协议 §2.3.1](../protocol/manifest.md#231-方法-methods)）。

**命令行**：`add`、`get`、`search`、`reindex` 先读 `daemon.json` 和 `api-token`，握手，再 `GET /health`；都成了就经守护进程，不然在本进程里直接开库。握手没过不发令牌。`init` 总在本进程。

**和别处还不一样的**：窗口用的路径不带 `/v1/`（只有投递接口带）；列表回普通数组，没有游标（[§0.8](conventions.md#08-分页和游标)）；用主令牌经窗口用的路径做的写操作都记 `via: app`，`POST /items` 收下记 `entry: paste`、`takenBy: user`，私密条目照收，命令行经守护进程收下的也一样（[§1.7](library.md#17-删掉) 的表要求 `api` 和终端外的命令行跳过私密）；经 `/v1/deliver` 的投递，扩展、手机、`api` 令牌遇到私密条目会跳过，终端里和终端外的命令行（本进程里直接收下时记 `cli`）都不跳。
