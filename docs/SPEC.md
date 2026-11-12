<a id="歇存-xiecun--技术规格-v01m0"></a>
<a id="usuyo--技术规格-v01m0"></a>
<a id="usuyo--技术规格-v1草案"></a>
# usuyo — 技术规格 v1（草案）

状态：v1 草案 · 2026-11-05。第 0、1 层写好了，等你过目；第 2–4 层还没写（[总纲 现在做到哪](BUILD.md#3-现在做到哪)）。2026-11-11 照代码补了一遍：实现替草案定下的地方标「11-11 代码」，实现还没做到的标「11-11 还没做」；第 2 层先有一节照代码写的[本机接口](#本机接口第-2-层11-11-照代码)。代码在 `crates/`（Rust，[ADR-004](decisions/ADR-004-rust-tauri.md)）。要你看一眼的在 [§0.9](#09-要你看一眼)、[§1.10](#110-要你看一眼)；v0 各节去了哪里见[附](#附v0-各节和-18-处的去向)。

<a id="0-约定第-0-层"></a>
## 0. 约定（第 0 层）

资料库文件、内核服务（HTTP、MCP、命令行）、插件协议都照这一层。

<a id="01-命名"></a>
### 0.1 命名

| 东西 | 写法 |
|---|---|
| 产品、命令、crate | `usuyo`；crate 叫 `usuyo-<部分>`，在 `crates/` 下（`usuyo-contract`、`usuyo-library`、`usuyo-index`、`usuyo-plugin`、`usuyo-plugin-kit`、`usuyo-core`、`usuyo-api`、`usuyo-cli`） |
| 环境变量 | `USUYO_<大写_下划线>` |
| 资料库里的隐藏目录 | `.usuyo/`，只放跟着资料库走的；`tmp/`、`lock` 例外，原子改名和锁要在同一个文件系统上，不同步 |
| 按机器的目录 | macOS `~/Library/Application Support/usuyo/`，Windows `%LOCALAPPDATA%\usuyo\`，Linux `$XDG_STATE_HOME/usuyo/`；每个库一个 `libraries/<libraryId>/` |
| URL scheme | 打开 `usuyo://<名词>/<id>[/<层>[@<snapshotId>]][#片段]`，名词 `item` `topic` `card` `draft` `changeset`；`usuyo://add?url=…` 只填好窗口，你点了才做 |
| JSON、YAML 字段、front matter 键 | camelCase（[协议 §2.1](plugin-protocol.md#21-文件名与位置)），MCP、前端同；不加前缀。导出文件里的属性键归导出插件（[协议 §4.4](plugin-protocol.md#44-导出的-run-与-readback)） |
| 枚举值、错误码、命令行选项 | 小写 kebab-case：`login-required`、`media:video`、`--dry-run` |
| 别的工具加的 JSON 字段 | `x-` 开头，我们不用 |

<a id="02-版本与兼容"></a>
### 0.2 版本与兼容

semver：patch 改错；minor 加可选字段、枚举值、错误码、接口；major 是删、改名、改类型或含义、可选变必填。线上只带主版本整数。

| 对象 | 线上带的 | 读到更新的主版本 |
|---|---|---|
| 插件协议 | `protocol: 1`；文档头写 `1.0.0` | 不加载，写原因 |
| 内核服务 | 路径 `/v1/`；MCP 工具、命令行 `--json` 跟它走。11-11 代码：只有投递接口在 `/v1/` 下（`/v1/ping` 回 `protocol: 1` 和版本），给窗口用的路径还不带 `/v1/`，版本由 `GET /health` 报（[本机接口](#本机接口第-2-层11-11-照代码)） | `/v1/ping` 报版本 |
| JSON Schema | `$id` 为 `urn:usuyo:schema:<名字>:<主版本>`（没有域名也能用） | 校验失败 |
| 资料库 | `.usuyo/library.json` 的 `formatVersion`、`extensions` | 只读，不给 Agent、不导出，提示升级 |

枚举开放，不认识的值按各处的退路办。旧程序不能忽略的新东西（改变能不能给 Agent、能不能删）写进 `extensions`，不认识就只读，照 git 的 `extensions.*`。

**迁移**只由宿主做，先问你，之前只读：

- 要改写的文件先复制到按机器的 `backups/`，留 30 天；不碰 `raw/` 和媒体，不改你写的正文；front matter 和 JSON 一样，只改迁移认识的键，不认识的原样留着（同下面的 JSON 演进）。
- 计划写在 `.usuyo/tmp/migrate-<从>-<到>.json`，逐步打勾，断了从没打勾的那步重跑。
- 开始时 `extensions` 写 `migrating`，没升级的旧程序（库在 iCloud 里的另一台）见了就只读；最后写 `formatVersion`，再去掉 `migrating`。
- 结果确定：新 id 由旧 id 算（推断：UUIDv8 或哈希），不读时钟，两台各迁各的一样。同步的记录带 `formatVersion`：远端新就不同步，拉到旧格式的先迁再合并。

**JSON 演进**：读到不认识的字段不报错，资料库文件原样写回，插件信封照[协议 §0.1](plugin-protocol.md#01-读者与状态) 忽略并记日志。收到的命令（HTTP 请求体、MCP 参数）有不认识的字段回 `invalid-args`，新功能在旧内核上看得见地失败。schema 不写 `additionalProperties: false`（加字段会变成 major）：运行时容忍，`usuyo plugin check` 和测试拒绝，怎么实现归第 3 层。

**弃用**：只在下个 major 删，之前至少两个 minor、6 个月；标 `deprecated: true`，HTTP 带 `Deprecation`（RFC 9745）、`Sunset`（RFC 8594）头。插件协议的旧主版本再加载 12 个月。

<a id="03-错误模型"></a>
### 0.3 错误模型

错误对象就是[协议 §3.3](plugin-protocol.md#33-信封与错误码) 的 `error`，宿主另填 `retryable`（不改什么、过会儿再发可能成功）和 `plugin`（哪个插件报的）。`needs` 加前缀 `version:`，例 `version:usuyo>=1.3`。

| 面 | 写法 |
|---|---|
| 插件 | 照协议的信封；JSON-RPC 插件用 `-32000`，`data` 是错误对象 |
| HTTP | RFC 9457：加 `type`（`urn:usuyo:problem:<code>`，客户端认 `code`）、`title`（固定英文）、`status`、`detail`（`hint`，没有就 `message`）、`instance`（`urn:uuid:<callId>`）；带 `Retry-After` |
| MCP | `isError: true`，文字是 `detail`，`structuredContent` 是错误对象（和输出 schema 怎么配，第 2 层核实） |
| 命令行 | 退出码 `0` 成功、`1` 失败、`2` 用法错（和插件一致）、`130` 被取消；`--json` 时 stdout 是 `{"ok": false, "error": {…}}` |

客户端只按 `code` 分支，不认识的按 `retryable` 和状态码大类办。插件报的错误同步返回一律 `502`、带 `plugin`，免得平台的 `login-required` 像令牌不对；下表的状态码只给宿主自己报的。

| 码 | 宿主报时的 HTTP | 可重试 |
|---|---|---|
| `invalid-args`、`cursor-invalid` | 400 | 否 |
| `unauthorized` | 401 | 否 |
| `forbidden`（令牌类型或 `Origin` 不许） | 403 | 否 |
| `not-found`（对 Agent 隐去的私密条目也回它，不带 `reason`） | 404 | 否 |
| `gone`（`reason`：`purged`、`expired`） | 410 | 否 |
| `conflict`、`cancelled`（多数只是任务状态）、`library-version`（`reason`：`too-new`、`needs-migration`、`unknown-extension`） | 409 | 否 |
| `too-large` | 413 | 否 |
| `unsupported-input` | 422 | 否 |
| `dependency-missing`（缺的是本机的东西） | 424 | 否 |
| `rate-limited` | 429 | 是 |
| `internal` | 500 | 否 |
| `crashed`、`invalid-output` | 502 | 否 |
| `login-required`、`blocked`、`upstream-changed`、`network` | 只由插件报，502 | 只有 `network` 是 |
| `library-busy`、`unavailable`（在启动、索引在重建）、`profile-busy` | 503 | 是 |
| `timeout` | 504 | 是 |

<a id="04-id"></a>
### 0.4 ID

**条目 id 由身份算**（[调研 14 §2](research/14-sync-like-hatoba.md#2-本产品要同步什么)），两台离线收下同一条得到同一个 id：

```
identityKey = <platform> ":" <contentId>        有 contentId
            = "web:" <规范化的 identity.url>     platform 是 web
itemId      = <platform> "-" base32(sha256(identityKey) 前 16 字节)
```

- base32 用 RFC 4648 字母表，小写，不补 `=`，26 位；128 位不用写碰撞分支。例：`douyin:7431234567890123456` → `douyin-4plrcxk343onmyxibmrr672kga`。
- `contentId` 照 plugin-kit 的平台列表（抖音十进制，B站 `BV…:p2`），JSON 里是字符串（19 位超过 2^53）。11-11 代码：平台列表还没有，宿主只查 `contentId` 非空、不超过 256 字节、没有控制字符；`platform` 要合 `[a-z0-9][a-z0-9-]*`。`identityKey` 按第一个 `:` 拆，`itemId` 按最后一个 `-` 拆（平台可以是 `wechat-mp`）。
- **web 身份**：插件规范化后，宿主再按 [RFC 3986 §6.2.2](https://www.rfc-editor.org/rfc/rfc3986#section-6.2.2) 做一遍：scheme、主机小写，去默认端口和 `#` 片段，空路径写成 `/`（`https://a.com` 和 `https://a.com/` 是同一条，`https://a.com?q=1` 写成 `https://a.com/?q=1`），路径和查询串不动（不改大小写、不解百分号编码、不去 `.`、`..` 段）。11-11 代码：只认 `http`、`https`；带用户名或密码的链接收下时就拒绝（`unsupported-input`），不进身份、标题和 `content.md`；收下的输入里取第一个 `http(s)://` 链接，到空白或非 ASCII 标点为止。没有插件命中的 http(s) 链接，和命中了插件、`canonicalize` 却因 `network`、`timeout` 失败的短链（下一条），宿主自己这样算，没装 web 插件也一样。
- **短链要联网展开**：`canonicalize` 因 `network` 或 `timeout` 失败时，宿主先拿短链本身照上一条算 `web:` 身份，马上建文件夹（`completeness: link`，之后的快照照记 `inputUrl`），只把重试放本机 `jobs.db`。联网后拿到平台身份，照下面「拿到更好的身份时」并过去，`web-` id 进 `aliases`；平台身份的 id 还没有条目时，这一步就是改名。并过去时撞上回收站或私密的，见下一条。两台离线收下同一条短链得到同一个 id；同一视频的两条不同短链先是两条，联网后并成一条。11-11 还没做：联网后拿到平台身份不并过去，fetch 交回的平台身份也不进 `identities`。
- **展开后撞上回收站或私密**（推断）：`jobs.db` 里的重试记下当初是哪个入口。主动收的照 [§1.7](#17-删掉)：在回收站里的拿回来，`restore` 和这次并过去（或改名）记在同一批，撤销那批时照 [§1.7](#17-删掉) 原样放回回收站。目标私密、当初是 Agent、`api` 令牌或终端外的命令行收的：`web-` 条目并进它，`origins` 不并过去（同直接收下时跳过），撤销那批不动这一条（`web-` id 成了别名，私密的对 Agent 照 [§1.3](#13-条目的四个维度) 回 `not-found`）。跟着收遇到 `canonicalize` 因 `network`、`timeout` 失败的不建短链条目，这一次不算跑完，联网后整次重跑。
- **没有身份的**（文件、聊天记录；fetch 输出省略 `identity` 或写 `{"platform": "file"}`，[协议 §3.7](plugin-protocol.md#37-fetch-输出)）：平台记 `file`，`file-` 加 26 位随机 base32（同样形状），放 `items/file/`，只靠内容层去重。`{platform: file}` 不算身份，不进 `identityKey` 和自动合并。不带 `contentId` 的身份只有 `web`，必带 `url`；`web`、`file` 以外的平台缺 `contentId` 照[协议 §3.1](plugin-protocol.md#31-运行器与节奏) 按 `invalid-output` 处理（feed 条目等[协议 §13 第 22 项](plugin-protocol.md#13-待定)）。
- **id 不改**，只能经合并换，被并掉的进 `aliases`。宿主自动合并两处（推断）：拿到更好的身份时（平台 > web > 随机），并进那个身份算出的 id；同步时两个 `itemId` 对上同一个 `identityKey`，并进由身份算出的那个。同一个 id 对上不同的 `identityKey` 不合并，报告。
- **算法冻住**：平台列表里 `contentId` 格式的改动（`BV…` 和 `BV…:p1` 算不算一个）算资料库的 major。web 身份尽力而为，对不上靠 `sameAs`；`inputUrl`、`resolvedUrl` 也进索引，查库里和回收站里的条目时再按它们查（推断）。
- **文件夹名就是 `itemId`**，引用只用 id，不用路径。
- **本机不带钥匙**：两台要在有钥匙之前就算出同一个名字。云端记录 id 是 `HMAC-SHA256(idKey, "item:" + itemId)`，服务器试不出你存着哪个公开视频（[调研 14 §4](research/14-sync-like-hatoba.md#4-加密)）。

**别的 id 用 UUIDv7**（RFC 9562）：按字符串排就是按时间排，Hatoba 也用；ULID 也是 26 位 base32，会和条目 id 混。用于资料库、快照、提议、改动记录、主题、卡片、成稿、导入批次、跟着收、任务、令牌、`origins[]` 每一项，还有 `deviceId`、`trashId`、改动记录的 `batch`、提议里 op 和 hunk 的 `id`、`callId`。块 id（划线、主题页段落）是 5 位随机小写 base32，文件内唯一，写在段末 ` ^<id>`（导出到 Obsidian 时正好是它的块 id 写法，不用转换）；脚注标签同样是 5 位小写 base32。`deviceId` 每台第一次打开这个库时生成。

**先后不靠时钟**：同一台按本台序号 `n`，跨设备按同步的 `seq`。要比时间的地方（主快照一样大时取新的）先比 `at`、再比 id 字符串，两台算出同一个结果。usuyo 写的时间照[调研 14 §5](research/14-sync-like-hatoba.md#5-改动和冲突) 钳制：本机时间和见过的最大远端时间 +1 ms 取大，有上限；和服务器差超过 5 分钟就提示、不推。

**引用**是普通链接，例 `[口播 01:42](usuyo://item/douyin-4plrcxk343onmyxibmrr672kga/asr@<snapshotId>#t=102.5)`。

- **`@<snapshotId>`**：秒数只对某一份媒体成立。时间类的层（`asr`、`video`、`audio`）必带，是那份媒体所在的快照；`text` 带引用时的主快照；`comments` 必带所在的快照；`vision` 带它的媒体所在的快照；`ocr/<imageKey>`、`image/<imageKey>`、`summary` 和 `notes` 不带。
- **片段**：时间 `#t=102.5`、`#t=102.5,110`（Media Fragments 1.0）；图片区域 `#xywh=percent:10,20,30,40`；画面上某一秒的一块（关键帧上的字）把时间和空间用 `&` 合写，例 `usuyo://item/<id>/video@<s>#t=12.04&xywh=percent:7.4,25.4,55,5.4`（系统的播放器不认空间部分，由 usuyo 自己解析）；原文 `#:~:text=[前缀-,]原话[,-后缀]`（URL Fragment Text Directives），太长写 `开头,结尾`；段落 `#^<块 id>`。
- **编码**：只编码原话部分里 ASCII 字母数字以外的字符，连同 `-`、`,`、`&`；语法字符和 `t=`、`xywh=` 后的数字照原样。例：「40 克」后面的「闷蒸 30 秒」写 `#:~:text=40%20%E5%85%8B-,%E9%97%B7%E8%92%B8%2030%20%E7%A7%92`。
- `asr@<s>#t=42` 和 `video@<s>#t=42` 指同一处：层名只决定打开哪一栏，去重、找回按条目、快照、时间比。

<a id="05-时间"></a>
### 0.5 时间

- RFC 3339 带偏移，不写纪元秒数。usuyo 写的用 UTC、固定 3 位毫秒，`2026-11-05T03:00:01.123Z`，能直接排序。
- 平台给的时间存 UTC，偏移不是 `Z` 时另存原样：`"actedAt": "2026-10-31T14:14:00.000Z", "actedAtOriginal": "2026-10-31T22:14:00+08:00"`；拿不到就不写。
- **front matter 里的时间**和 JSON 一样（第一条）：UTC、3 位毫秒，写成双引号字符串，例 `createdAt: "2026-11-05T03:00:01.123Z"`；读照 RFC 3339，别的写法算这个文件写坏了（[§1.4](#14-你写的那一层)）。front matter 里只有 `createdAt` 一个时间。
- **写进 Markdown 给人看的日期**（引用脚注的发布日期、`content.md` 头部）取 `publishedAtOriginal` 的日期部分，没有就取 UTC 日期，不按本机时区：两台写出一样的字节。
- 时长：JSON 里是秒、字段名以 `Sec` 结尾，YAML 写 `8s`、`10m`。按天的设置用整数天、名字以 `Days` 结尾（如 `trashDays`），「一直」写 `null`；错误对象的 `retryAfter` 是秒，照 HTTP `Retry-After` 不带 `Sec`，是例外。大小：JSON 里是字节，YAML 写 `512MiB`。媒体位置是秒数，最多 3 位小数；显示向下取整，`102.5` 写 `01:42`，过一小时写 `1:01:42`。

<a id="06-文本和文件"></a>
### 0.6 文本和文件

- UTF-8 不带 BOM，LF，以换行结尾；BOM、CRLF 照读，改写时保持原样。JSON 缩进两格、键顺序固定；JSONL 一行一个对象。
- **front matter** 只有 usuyo 写，用 YAML 的一个子集，同一个对象每台写出一样的字节：键照各处列的顺序，不认识的键排在认识的后面，按键名排；字符串一律双引号（时间、`no`、`yes` 也还是字符串）；布尔不加引号；列表一行一项、按字符串排；`false` 和空列表不写，没有这个键就是空的或 `false`；不用注释、锚点和行内写法。只有你写的 `.md` 带它：主题、卡片、成稿和 `notes.md`（[§1.4](#14-你写的那一层)）；`content.md`、`highlights.md` 不带。
- id、文件名、front matter 键、我们生成的字用 NFC；平台原文照收到的存（NFC 会换掉 CJK 兼容汉字）。
- **原子写**：同目录写 `.<名字>.usuyo-tmp-<随机>`，fsync，改名盖过去；Windows 上目标被占用就短暂重试，不行回 `library-busy`。写回前比一次哈希：和本机上次写的、读到的不一样，说明 usuyo 外有写方（库在云盘里时的另一台，或手改），先照 [§1.4](#14-你写的那一层)「usuyo 外改了资料库里的文件」读进来、合并，再写。
- **同一条只有一个写方**：本机按 `itemId` 加锁，任务按 `itemId` 去重。新条目在 `.usuyo/tmp/` 拼好再改名进去；目标已经在（同一链接贴了两次、跟着收和你同时收下、同步拉下同一条），就把新快照移进去，合并 manifest。
- **锁**：一台机器上一个库一个写方，`.usuyo/lock` 加系统建议锁。它管不住两台经云盘同时开，所以内置同步和云盘二选一。
- **大小写不敏感**（macOS、Windows）：我们起的名字全小写；标题来的名字按 NFC 加大小写折叠查重（排除文件自己），撞了加「 2」；只改大小写的改名分两步，经过临时名。
- **不能用的字符**：系统不许的 `< > : " / \ | ? *` 换成全角 `＜ ＞ ： ＂ ／ ＼ ｜ ？ ＊`；链接只用 id（[§0.4](#04-id)），不用文件名，所以 `# ^ [ ]` 照留；控制字符删掉；去掉开头的 `.` 和结尾的空格、`.`；`CON`、`NUL`、`COM1` 这类保留名前加 `_`，带扩展名也算。
- **长度**：每段不超过 200 字节，标题按字形截断，截时留出扩展名和本机后缀的位置（约 64 个汉字）。从库根起的相对路径不超过 200 个 UTF-16 单位，条目文件夹以内不超过 140；回收站里只放对象以内的相对路径，主题、卡片、成稿在回收站里存成 `body.md`，原路径记在 `entry.json` 的 `path`。库根在 55 个字符内就不过 Windows 的 260，超了提示一句。
- **平台来的字**（标题、作者、原话、正文）是不可信输入，按写进的位置转义：YAML 一律双引号标量，`\`、`"`、控制字符转义，换行写 `\n`；Markdown 链接文字和原话转义 `` [ ] ( ) < > * _ ` \ # ``，换行合成空格；`textFormat: plain` 的正文写进 `content.md` 时反斜杠转义 `` \ ` * _ [ ] < > # ! | ~ `` 和行首的 `-`、`+`、`=`、`数字.`，`&` 写 `&amp;`，渲染出来仍是纯文字。

<a id="07-哈希"></a>
### 0.7 哈希

SHA-256，小写十六进制，算存下来的字节。字段名叫 `sha256` 时写裸值（协议的 `install`、`models`，[§1.2.2](#122-raw-和-derived) 的 `inputs`、`files`）；通用的哈希字段写 `sha256:<hex>`（导出记录，[§1.5.2](#152-提议文件) 的 `base`，[§1.6](#16-改动记录) 的 `before`、`after`），以后换算法也装得下。离开本机的文件 id 是 `HMAC-SHA256(idKey, "blob:" + 内容)`，和记录 id 共用 id 钥匙、靠前缀分开（[调研 14 §4](research/14-sync-like-hatoba.md#4-加密) 只分记录、文件、id 三把）；切块见[调研 14 §3](research/14-sync-like-hatoba.md#3-存在哪和花多少)。你内容的普通哈希不出本机。

<a id="08-分页和游标"></a>
### 0.8 分页和游标

- 请求 `limit`（默认 50，最多 500）、`cursor`；响应 `items`、`nextCursor`（最后一页 `null`），和[协议 §3.8](plugin-protocol.md#38-list-与跟着收)、MCP 同名。
- 11-11 还没做：本机接口回的是普通数组，没有 `cursor`；搜索的 `limit` 默认 50，限在 1 到 200（[本机接口](#本机接口第-2-层11-11-照代码)）。
- 游标是不透明的 base64url，和查询绑定；查询变了或索引重建过回 `cursor-invalid`。按键分页，最后按 id 定序，新收下的不会让条目跳过或重复。有效期归第 2 层。
- 响应不带总数（按键分页算总数要另扫一遍）；另给哪些计数（搜到多少条、磁盘占用），第 2 层按视图定。永远不出现的数：几条没看过、几条没整理、几个提议没决定（[原则 4](PRD.md#4-原则)）。插件 `sources` 的 `count` 是平台页面上的数（[插件协议 L5](plugin-protocol.md#31-运行器与节奏)）。

<a id="09-要你看一眼"></a>
### 0.9 要你看一眼

| # | 什么 | 建议 |
|---|---|---|
| 1 | [§0.4](#04-id) 条目文件夹名像 `douyin-4plrcxk343onmyxibmrr672kga`，看不出标题，不按年月分 | **已定（11-04）**：定同步时照调研 14 定了目录名由身份算（[回答表第 14 条](answers.md#14-资料库能不能放进同步盘)，[回答表](answers.md)）：两台离线收下同一条要落到同一个文件夹 |
| 2 | [§0.1](#01-命名) 属性名怎么起 | **不再问（11-05 方向）**：front matter 键和 JSON 一样用 camelCase，不加前缀；Web Clipper 键只在导出文件里，见 [§0.1](#01-命名)；[回答表](answers.md) |
| 3 | [§0.5](#05-时间) 属性里的时间怎么写 | **不再问（11-05 方向）**：时间一律 RFC 3339 UTC 毫秒；只写日期是导出的事，见 [§0.5](#05-时间)；[回答表](answers.md) |
| 4 | [§0.6](#06-文本和文件) 文件名里不能用的字符和长度 | **不再问（11-05 方向）**：只把系统不许的 9 个字符换成全角，`# ^ [ ]` 照留，截到约 64 个汉字，见 [§0.6](#06-文本和文件)；[回答表](answers.md) |

<a id="1-资料库-v1第-1-层"></a>
## 1. 资料库 v1（第 1 层）

资料库就是这些文件，第 2、3 层按这里读写。

<a id="2-资料库目录真相源"></a>
<a id="11-目录树"></a>
### 1.1 目录树

```
<资料库>/
  items/<platform>/<itemId>/      一条一个文件夹
  topics/  cards/  drafts/        <标题>.md，可以分子文件夹
  canvas/                         等 M7 和画布原型
  .usuyo/
    library.json                  formatVersion、extensions、libraryId、sync
    settings.json                 跟着资料库走的设置
    devices/<deviceId>.json       名字、revokedAt、lastSeenAt
    journal/<deviceId>/<段>.jsonl 改动记录
    facts/<deviceId>/<段>.jsonl   发生过的事
    changes/<id>.json             改动提议
    subscriptions/<id>.json       跟着收：fetchDevice、take；其余归第 4 层
    imports/<importBatch>.json    导入记录，归第 4 层
    trash/<trashId>/              回收站
    tmp/  lock                    本机用，不同步
```

- 各项的格式：条目文件夹见 [§1.2](#12-条目文件夹)；主题、卡片、成稿见 [§1.4](#14-你写的那一层)；发生过的事见 [§1.4.4](#144-发生过的事)；改动提议见 [§1.5](#15-改动提议)；改动记录见 [§1.6](#16-改动记录)；回收站见 [§1.7](#17-删掉)。
- 按平台分一层，文件夹名是 `itemId`（[§0.4](#04-id)），两台离线算出同一路径；年月不进路径。回收站放 `.usuyo/trash/`：它是库的状态，各台各挪各的、只同步状态（[§1.7](#17-删掉)）；`items/`、`topics/`、`cards/`、`drafts/` 在 Finder 里只剩还在的。
- `library.json`：`{"formatVersion": 1, "extensions": {}, "libraryId": "<UUIDv7>", "createdAt": "…", "sync": null}`；开了内置同步，`sync` 是 `{"backend": "cloudflare", "remoteId": "…"}`，不含密钥。它在库里，库在 iCloud 里时另一台一打开就能提示二选一。
- `settings.json` 这里只定 `trashDays`（30，可选 90 或「一直」）；其余（转换哪几层、取不取评论、Agent 能看的范围、私密条目上不上云）归第 2 层。
- 跟着收的 `fetchDevice` 是哪台去取。`take` 只给订阅来的（RSS、简报、别人的公开账号）：`picked` 只收你挑的，`picked-or-matched` 也收命中你关注的，默认 `picked`（[PRD 7.3](PRD.md#73-导入和跟着收) 的建议）；你自己在平台上的收藏夹和列表不带 `take`，新条目都收下。
- 跟着收列过哪些身份（只存身份的哈希，不存标题）记在本机 `jobs.db`，和翻页游标、上次成功跑完的时间（`--since`）放在一起，不跟着资料库走（推断）；它只收自己没列过的，见 [§1.7](#17-删掉)。这一次跑完才把列出来的记成列过；还没跑完过一次的订阅，下一次仍照第一次跑（推断）。每台第一次跑只记下已有的，不收（[PRD 7.3](PRD.md#73-导入和跟着收) 对个人来源这样建议，这里所有订阅、每台都照它，推断）；换了 `fetchDevice`，换上的那台也照第一次跑，上一台最后一次跑完到这台第一次跑之间新出现的会漏掉（推断，这是代价）。

**留在每台机器上的**放 [§0.1](#01-命名) 的按机器目录；缓存放系统缓存目录（`~/Library/Caches/usuyo/`、`%LOCALAPPDATA%\usuyo\cache\`、`$XDG_CACHE_HOME/usuyo/`），删了不丢东西（11-11 代码：插件的缓存先放在按机器根目录的 `cache/plugins/` 下）。

| 在哪 | 放什么 |
|---|---|
| `libraries/<libraryId>/` | `device.json`（`deviceId`、库路径）、`index/`、`clock-mark`（回收站到期用的时钟记录，见 [§1.7](#17-删掉)）、`jobs.db`（任务状态、转换任务 `transform_jobs`（feature 003，还没做）、跟着收列过的身份、翻页游标和 `--since`、还没展开的短链（重试）、上次写的 `content.md` 的哈希）、`sync.db`（游标、底本（你写的那层和 manifest 上次见过的那份，没开同步也存，[§1.4](#14-你写的那一层) 外面改了靠它合并）、上次见过的路径）、`activity/`（Agent 搜过、读过什么，一行一次，不给 Agent、不同步；格式和留多久归第 2 层，推断）、`exports/<target>.json`（每个对象 `{path, hash, base}`，`base` 是上次和 vault 对上的那份用户层：写出去了就是交给导出插件的那份，没写出去就是那次读回的值，私密脚注已换成占位；[§1.4](#14-你写的那一层) 导出读回靠它合并）、`imports/<importBatch>/`（导入文件副本）、`backups/` |
| 根目录 | `settings.json`（外观、模型端点、插件启用和设置，`transforms.auto`（自动转换，默认 `true`）；11-11 代码只用 `library` 和 `plugins.<插件 id>.settings`）、`plugins/`、`profiles/`、`models/<owner>/<repo>/<名字>/`（转换插件的模型，宿主下载、校验后放这里，旁边写 `model.json`：`{name, sha256, size, license, files: [{path, size, sha256}]}`）、`cache/plugins/<owner>/<repo>/`、`cache/transform/<itemId>/<runId>/`（一条转换链的中间产物，例如给转写用的 16 kHz WAV，链跑完就删）、本机接口令牌 `api-token`、投递令牌 `deliver-tokens.json`、`daemon.json`、登录索引 `logins.json`、`run/`、`inbox/`、日志 `logs/daemon.log`（各项见下面「11-11 代码」） |
| 系统钥匙串 | 同步令牌和库钥匙、模型和插件密钥、平台登录态（[协议 §6](plugin-protocol.md#6-凭据)） |

11-11 代码：

- 按机器的根目录可以用 `USUYO_HOME` 换成别处（测试用）；Linux 没设 `$XDG_STATE_HOME` 时是 `~/.local/state/usuyo/`。根目录、`libraries/<libraryId>/` 和 `logs/` 只给本人（0700），`jobs.db`、`api-token`、`deliver-tokens.json`、`daemon.json`、`logs/daemon.log` 是 0600。
- `api-token` 存令牌本身，不存哈希：握手要拿它当密钥（[本机接口](#本机接口第-2-层11-11-照代码)）。每次启动守护进程换一个，正常退出时连同 `daemon.json`（`{port, pid}`）一起删掉。
- `device.json` 是 `{deviceId, libraryPath}`。`jobs.db`（建表分两版，`user_version` 是 1）里有六张表：取内容的任务（每条一行，按 `itemId` 去重，带优先级和所属导入）、插件的暂停（按插件和范围，范围是平台，web 插件按主机，`*` 是你点的「先别试」；记错误码、原因、到什么时候）、上次写的 `content.md` 的哈希、平台的节奏记账（每个平台上次调用结束的时间、今天批量调用了几次，重启后照旧）、导入、导入的清单（每个导入里每条链接一行，记身份、在库里的哪里、收下没有）。失败的任务清掉链接，只留错误码和原因。任务一次跑一个，同一平台同一时间只跑一个调用。
- `daemon.json` 是 `{port, pid}`。`api-token`、`daemon.json` 的来去见[本机接口](#本机接口第-2-层11-11-照代码)。根目录 `settings.json` 现在只有 `library`（当前资料库）和 `plugins.<插件 id>.settings`，别的键读写时原样留着。
- `deliver-tokens.json`：投递令牌的清单，一个数组，每项 `{id, name, kind, secret, createdAt}`，`kind` 取 `extension`、`mobile`、`api`。`secret` 是明文（读文件的权限只有本人），令牌整个只在发的那一次回给调用方；撤销马上生效。
- `logins.json`：登录站点、访客身份和密钥设置的索引，不含任何值。站点和访客身份各按主机记存下的时间和每个 cookie 的名字、域名、过期时间；密钥设置记每个插件填了哪几项。看登录状态、过期没有只读它，不碰钥匙串。
- `run/`：一次调用要交凭据给插件时，在这里建这次自己的文件夹 `usuyo-cred-<随机>`（0700），文件 0600：`credentials.json` 和 `cookies.txt`（账号站点）、`visitor.json`（访客身份）、`secrets.json`（密钥设置），路径经 `USUYO_CREDENTIALS`、`USUYO_COOKIES_TXT`、`USUYO_VISITOR`、`USUYO_SECRETS` 传给插件（[协议 §6](plugin-protocol.md#6-凭据)）；进程结束整个文件夹删掉，守护进程启动时再清一遍上次被杀留下的 `usuyo-cred-*`。没有可交的就不建。
- `inbox/`：收件文件夹，手机经云盘放进来。只认顶层的 `.txt` 普通文件；点开头的、别的扩展名、子文件夹都不碰；每 5 秒看一轮，一个文件 10 秒没变才处理；最多读 1 MiB、收 500 条链接，文件里每个 http(s) 链接收下一条（整段分享文字也行），入口记 `mobile`，放进去本身就是确认，不进导入预览。全收成了移到 `.usuyo-done/<日期>/`；没有链接、有链接没收成、或是符号链接的移到 `.usuyo-failed/`，旁边一个 `<名字>.error.json`（`{errors: […]}`，原因只写主机和错误码）。`GET /inbox` 回它在哪。
- `libraries/<libraryId>/clock-mark`：回收站到期检查记下的最大时间，见 [§1.7](#17-删掉)。
- 钥匙串（macOS、Windows）：服务名 `usuyo`，账户名是 `<12 位十六进制>/<键>`，前一段是按机器根目录（取规范路径）的 SHA-256 前 12 位，所以不同的 `USUYO_HOME` 各管各的，开发用的另一套不会盖掉正式那套。键有三类：`site:<主机>`（这个站点的登录，storageState 的 JSON）、`visitor:<主机>`（这个站点的访客身份，不是账号，和同一主机的 `site:` 分开）、`secret:<插件 id>:<设置键>`（一个密钥设置）。Linux 还没接 Secret Service，换成根目录里 0600 的 `secrets.json`，键一样，是替身：挡得住别的用户，挡不住以你的身份跑的程序。壳的登录窗口在 macOS 上用系统的网页数据仓库，Windows、Linux 上放在根目录的 `webviews/` 下，见 [apps/desktop/README.md](../apps/desktop/README.md#登录窗口)。
- 以写方式打开库（拿到 `.usuyo/lock`）时，清掉 `.usuyo/tmp/` 里崩溃留下的调用文件夹（`usuyo-call-*`）和以 UUIDv7 命名的暂存文件夹，别的不动。
- 扫描（启动、重建索引、地图、收下）遇到读不了的条目、回收站项、主题、卡片、成稿就跳过，写日志；条目另外列进结果的 `unreadable`（`{id, reason}`），一个坏文件不挡住别的。界面还不显示它们。

为什么搬出来：库进同步盘时 sqlite 和索引会被写坏；密钥不落普通文件；库拷到另一台自然有新 `deviceId`。索引不放缓存目录，macOS 磁盘紧时会清 Caches。同一台上同一个 `libraryId` 出现在第二个路径，问「这是拷贝吗」，是就换新 `libraryId`、`sync` 置 `null`，免得同步进同一个云端。

<a id="12-条目文件夹"></a>
### 1.2 条目文件夹

```
items/douyin/douyin-4plrcxk343onmyxibmrr672kga/
  manifest.json  content.md  notes.md  highlights.md
  raw/<snapshotId>/   usuyo.fetch.json  page.html  images/001.jpg  media/video.mp4  comments.json …
  derived/
    runs/<runId>/     run.json  play.mp4                          一次转换一个文件夹（§1.2.2）
    runs/<runId>/     run.json  asr.vtt
    runs/<runId>/     run.json  keyframes.json  keyframes/00012040.jpg …
    runs/<runId>/     run.json  ocr.json
    readable.md  vision.md  summary.md  embeddings.bin  thumb.jpg  fingerprints.json
```

<a id="3-manifestjson"></a>
<a id="121-manifestjson-v1"></a>
#### 1.2.1 manifest.json v1

不带 `schema`，版本看 `formatVersion`；数组总是出现。数组写回时排好序，两台写出一样的字节：`origins`、`snapshots` 按 `at` 再按 `id`，`identities` 按 `identityKey`，`tags.platform`、`tags.engine` 按 `tag`，`matched` 按 `topic` 再按 `by`，其余按字符串。最后一列给 [§1.8](#18-同步) 用。

| 字段 | 取值 | 谁写 | 同步时怎么合并 |
|---|---|---|---|
| `id`、`aliases` | `itemId`；并掉的 id | 宿主 | 不变；集合三方合并 |
| `identities` | `[{platform, contentId?, url?}]`，`id` 由建条目时的身份算（没有身份的随机），之后只经合并换（[§0.4](#04-id)），不看数组位置 | 宿主 | 集合三方合并，按 `identityKey` 认 |
| `kind` `title` `author` `publishedAt` `publishedAtOriginal` | 取自主快照；没有快照时 `title` 取 `list` 或投递给的值，再没有写原链接；`kind` 写 `unknown` | 宿主 | 跟主快照 |
| `lang` | 宿主从主快照的文字或字幕判断，BCP 47 | 宿主 | 跟主快照 |
| `addedAt` | 第一次收下的时间 | 宿主 | 取早的 |
| `origins`、`snapshots` | 见下 | 宿主 | 集合三方合并，按每项的 `id` 认 |
| `primarySnapshot` | 输出集合最大的，一样大先比 `at` 再比 `id`；没有快照为 `null`。11-11 代码：比的是各快照自己的 `outputs` 个数（不减 `textOnly.removed`），一样多取 `at` 晚的，再一样取 `id` 字符串大的 | 宿主 | 合并后重算 |
| `outputs` | 各快照 `outputs` 的并集，减去 `textOnly.removed` 涉及的 | 宿主 | 合并后重算 |
| `missing`、`missingReason` | 各快照 `missing` 的并集 − 条目 `outputs`，`state: text-only` 时只算文字类；快照的 `missing` 由取它的那台在取的时候算好，之后不改：这次的 `--want` − 实际拿到的；`--want` 一般是命中插件声明的 `outputs` ∩ 设置，重试只要缺的、「取评论」和只留文字以后才更窄（评论默认不算；投递来的、文件就是实际拿到的）。只用同步的字段算，两台算出一样的；设置、插件后来变了只影响下一次取。`missingReason` 是给你看的原因；11-11 代码取最新一份、还有缺的输出没补上的快照的 `missingReason` | 宿主 | 合并后重算 |
| `completeness` | `link` `partial` `full`：没有快照是 `link`，否则由 `missing` 算；只升不降（[PRD §8](PRD.md#8-条目的状态)）：重算出的比现值低时留现值，条目页照 `missing` 说缺什么 | 宿主 | 两边和重算的取高的 |
| `tags.platform`；`tags.engine` | 快照带的；`[{tag, by, at, confidence}]` | 宿主 | 并集；同一个 `tag` 取 `at` 新的，再比 `by` |
| `tags.user`、`tags.dismissed` | 你的标签；忽略过、不再建议的 | 你 | 集合三方合并 |
| `topics` | 放进的主题 id | 你 | 集合三方合并 |
| `sameAs`、`notSameAs` | 机器认出的同一内容；你说「不是」的 | 宿主；你 | 集合三方合并 |
| `private` | 默认 `false` | 你 | 两边改得不一样取 `true`，多私密一会儿不丢东西 |
| `state`、`stateAt` | `normal` `text-only` `trashed`；改它的时间 | 你；Agent、`api` 令牌收下和 Agent 撤它自己那批时（[§1.6](#16-改动记录)、[§1.7](#17-删掉)）；到期移除除外 | 两边都改了按 `trashed` > `text-only` > `normal`（推断），不比时钟 |
| `textOnly` | `{at, removed: [路径]}`；从回收站恢复时有它回 `text-only` | 宿主 | 跟 `state` |

**`origins[]`**，每收下一次一项（先由跟着收收下，后来你又粘贴一次，就是两项）：

| 字段 | 取值 |
|---|---|
| `id` | UUIDv7，同步时按它认 |
| `entry` | `paste` `drop` `extension` `mobile` `import` `subscription` `mcp` `cli` `api`；v0 的 `sync` 改名，「同步」现在指设备之间 |
| `takenBy` | 谁决定收的：`user`、`agent:<连接名>`（kebab-case，来源归第 2 层）、`subscription:<id>` |
| `machinePicked` | `null`、`{"by": "keywords"}`、`{"by": "similarity"}`、`{"by": "upstream", "name": "horizon"}`、`{"by": "agent", "name": "<连接名>"}`；只有你给的具体链接（Agent 收下时说明是你点名的链接，参数名归第 2 层）和你自己平台上的收藏、列表为 `null`；你给关键词让 Agent 去找的也算机器挑的（11-06 定，[§1.10 第 14 条](#110-要你看一眼)） |
| `matched` | 命中关注时 `[{topic, by: "keywords" \| "similarity"}]`，可以命中几个 |
| `subscription`、`importBatch` | 从哪个跟着收、哪一批来 |
| `action` `actedAt` `actedAtOriginal` | 平台上的动作和时间（[协议 §3.8](plugin-protocol.md#38-list-与跟着收)） |
| `at`、`device` | 时间、哪台；转换在 `at` 最早那一项（一样早比 `id`）的 `device` 做（[调研 14 §0](research/14-sync-like-hatoba.md#0-结论) 第 10 问），那台不在了怎么换归同步 feature |

你在跟着收的预览里自己点收下的，`takenBy: user`、`machinePicked: null`（推断：你一条条挑过）。跟着收按关键词或相似度命中收下的都标「机器挑的」。界面标「机器挑的」：每一项 `origins` 都带 `machinePicked`（你自己也收过一次就不标）；写过、放进主题、引用过都不去掉，它是来历（11-05 定，[§1.10 第 19 条](#110-要你看一眼)）；地图上怎么分开等 [M4](PRD.md#76-地图)。

**`snapshots[]`**（[协议 §3.5](plugin-protocol.md#35-canonicalize-与身份)–[§3.7](plugin-protocol.md#37-fetch-输出)）：`id`（UUIDv7）；`plugin`、`pluginVersion`（交来已取到的内容时：投递接口记 `deliver:<令牌名>`，收件文件夹记 `deliver:inbox`；链接和交给导入插件的文件，记去取它的那个插件）；`at`、`inputUrl`（你给的原链接）、`resolvedUrl`（`canonicalize` 展开的完整链接）；`access`（投递为 `deliver`）、`site`、`upstream`；`outputs`（实际拿到的，比完整度只比它）；`missing`、`missingReason`、`completeness`（宿主在取的时候算，`missing` 是这次的 `--want` − 实际拿到的，之后不改）；`textOrigin`（`platform` 或 `machine`，等[协议 §13](plugin-protocol.md#13-待定) 第 24 项）。

11-11 代码：快照的 `outputs` 由 `usuyo.fetch.json` 算：有 `text`、`html`、`images`、`media` 里的 `video`、`audio`、`subtitle`，标题、作者、发布时间有一样就算 `meta`，有 `comments`。快照的 `completeness`：没有输出是 `link`，没缺的是 `full`，否则 `partial`。只有 `missing`、`missingReason`，没有别的「缺」字段。11-11 还没做：`inputUrl` 记的是 `canonicalize` 给的 `resolvedUrl`（没有就是收下的链接），`resolvedUrl` 不写。

例：视频失败、文字成功，得快照 s1（`outputs: [text, images]`）；重试只要 `--want media:video`，得 s2（`outputs: [media:video]`，`missing: []`）。条目 `outputs` 是并集，`missing` 为空，不再显示「缺视频」。重试拿到的文件和已有快照逐个哈希都一样时，不新建快照。

<a id="5-derived-文件约定"></a>
<a id="122-raw-和-derived"></a>
#### 1.2.2 raw 和 derived

- **`raw/<snapshotId>/`** 是插件的输出目录整个移进来（[协议 §3.6](plugin-protocol.md#36-fetch-与输出目录)），之后不改。宿主只加 `usuyo.fetch.json`：fetch 输出去掉 `credentialsUpdate`，加 `files: [{path, size, sha256}]`。同步前和「检查与修复」时按它比对，对不上就报「原样被改过」、不上传。只有你选「只留文字」或「删掉」，里面的文件才移去回收站。
- **放进 `raw/` 之前**（11-11 代码）：插件的输出先在 `.usuyo/tmp/` 下暂存，不在那里的不收。暂存的文件夹里不能有符号链接（`invalid-args`）；`raw/<snapshotId>/<路径>` 不超过 140 个 UTF-16 单位（`too-large`）。宿主在拿资料库的锁之前按 1 MiB 一块算好各文件的哈希，拿到锁以后再比一次文件名和大小，变了就拒（「hash 之后改过」）。逐个哈希和已有快照一样时不新建快照。
- **`usuyo.fetch.json` 里的路径**（`images`、`media[].path`、`files[].path`）只能是快照文件夹里的相对路径：不空、没有 `.`、`..`、绝对路径和盘符。加快照时不合的回 `invalid-args`，读到不合的当这个文件写坏了。
- **`derived/runs/<runId>/`：一次转换一个文件夹**（feature 003 [§5](features/003-transforms.md#5-存在哪怎么重做)）。`runId` 是 UUIDv7。宿主先在 `.usuyo/tmp/` 拼好整个文件夹（插件的产出、宿主算的哈希、`run.json`），再改名进来；写进去以后不改，要改就是再转一次，新的一次和旧的并排。哪一次排在前面按 `runId`（就是时间）算，不另存「现在是哪份」；引用按秒、按 `imageKey` 找回，不认 `runId`，换了一次转录，写过的引用不断。改名进来之前先核：`runId` 是 UUIDv7、还没有这一次；文件夹里没有符号链接，文件和拿锁之前算好的哈希一样；声明的产物都在、大小和 sha256 对得上，`keyframes.json` 列的帧都在；没有声明以外的文件；`empty`、`outputs`、`reason` 对得上。不合的回 `invalid-args`，已有同一个 `runId` 回 `conflict`，条目不在回 `not-found`，暂存的文件夹删掉。核过以后各产物和 `run.json` 先 fsync，再整个改名；中途崩溃的只留在 `.usuyo/tmp/`，下次打开库时清掉，所以不会有没有 `run.json` 的转录。
- **做好没有**：一个任务（`job`）算做好，要有一次这个 `job` 的转录，它 `inputs` 的 sha256 盖住了现在要的那些输入；哪个插件、哪个版本做的都算，插件升级不自动重转。`empty: true` 的也算做好。失败、在等的只在本机 `jobs.db`。
- **按层找回**：转录的几层只从 `derived/runs/` 读，`derived/` 根下旧的 `asr.vtt`、`ocr/`、`keyframes/` 不再读。`asr`、`ocr`、`frames` 是这一层最新的一次（给了快照就只看输入在那个快照里的）；`asr/<runId>`、`ocr/<runId>`、`frames/<runId>` 是那一次；`ocr/<imageKey>` 是从新到旧第一份认了这张图的 `ocr` 转录；`frame/<runId>/<毫秒>` 是那次 `keyframes` 转录里秒数四舍五入到毫秒相等的那一帧的图片。最新的一次是空的（`empty: true`），这一层就是没有（`not-found`），不退回更早的一次。
- **中间产物不进资料库**：给转写用的 16 kHz WAV 一类放按机器缓存 `cache/transform/<itemId>/<runId>/`（[§1.1](#11-目录树)），链跑完就删；来历记在用它的那次转录的 `inputs[].via` 里。
- **同步**：`run.json` 和文字产物（`asr.vtt`、`ocr.json`、`keyframes.json`）跟着文字同步；二进制产物（`play.mp4`、`keyframes/*.jpg`）跟着媒体，现在不同步。另一台看得到关键帧上的字和秒数，看不到那张帧。
- **`run.json`**（`urn:usuyo:schema:run:1`）由宿主写，插件不写；照它写的插件、工具、模型、参数和输入能再跑出同样的东西，或换掉其中一样再跑一次。数组总是出现，没有值的写 `null`：

| 字段 | 取值 |
|---|---|
| `id` | UUIDv7，等于文件夹名 |
| `job` | `play` `asr` `keyframes` `ocr`（图文的图、封面）`frames`（关键帧上的字）；以后 `readable` `vision` `summary` `embeddings` 照这个形状加 |
| `by`、`version` | 插件 id 和版本；宿主自己判出来的写 `host` 和内核版本 |
| `tool` | 插件里干活的程序 `{name, version}`，插件在 `run` 的结果里报（[协议 §4.2](plugin-protocol.md#42-转换的-run)）；`by: host` 为 `null` |
| `model` | `{name, sha256}`，sha256 是宿主核过的；没用模型为 `null` |
| `options` | 这次实际用的参数，含插件补上的默认值；重做时原样交回 |
| `at`、`device` | 什么时候、哪台转的 |
| `trigger` | `auto`、`open`（你打开了这条）、`redo`（你点的） |
| `redoOf` | 点「再转一次」「换一个转」时当时眼前那一份的 `id`，不然 `null`；只是来历，不改谁在前 |
| `inputs[]` | `{path, sha256, snapshot, via}`：相对条目文件夹的路径（`raw/<snapshotId>/…`；`frames` 的是 `derived/runs/<runId>/keyframes/<文件>`）、所在快照；`via` 是中间转换的每一步 `{by, version, op, tool?, sha256?}`，不出文件的（`probe`）没有 `sha256` |
| `outputs[]` | `{path, kind, size, sha256, lang?}`：文件夹里的产物，路径相对这个文件夹；`asr` 带 `lang`；关键帧图片算在 `keyframes.json` 那一项里 |
| `empty`、`reason` | 没有可产出的东西时 `true`，`reason` 是 `no-speech`、`no-audio-track` 或 `no-text`，这时 `outputs` 为空；`false` 时 `reason` 为 `null` |
| `marks[]` | 很明显的标记 `{code, speechSec?, durationSec?, frozenSec?}`，`code` 是 `no-audio-track` `no-speech` `static-picture` `no-text`（feature 003 [§6](features/003-transforms.md#6-很明显的标记)）；只提示，不藏任何东西 |

- 词表（`job`、`trigger`、`kind`、`reason`、`code`）在类型里是字符串：别的设备上新版本写的转录，旧版本照样读得进来、照样写回。`inputs[].path`、`outputs[].path` 只能是普通相对路径，规则同 `usuyo.fetch.json`，读到不合的当这个文件写坏了。
- **`no-audio-track`** 不经转写插件：宿主让抽音轨的插件做一次 `probe`（[协议 §4.2](plugin-protocol.md#42-转换的-run)），看不到音频流就自己写一次 `job: asr` 的转录：`by: host`，`inputs` 是那个画面文件，`via` 记那次 `probe`，`empty: true`，`reason: no-audio-track`。
- `imageKey` 是图片字节 SHA-256 的前 16 位十六进制，两份快照里同一张图只认一次。`readable.md`（网页抽出的正文）由转换插件产出，现在没有插件产出它，做的时候照转录的规矩；`thumb.jpg`、`fingerprints.json` 由宿主产出；`embeddings.bin` 的格式归 plugin-kit，不同步。

**产物的格式**（schema 在 `crates/usuyo-contract/schema/`，[§1.9](#19-schema-和-usuyo-contract)）。时间是秒，最多 3 位小数（[§0.5](#05-时间)）；**图上的框**是 `[x, y, w, h]`，0–1，按图的宽高算，**原点在左上角**，引用链接里换成百分数（[§0.4](#04-id)）。

- **`asr.vtt`**：WebVTT，UTF-8；一条 cue 一段人声，长于 `maxCueSec`（默认 15 秒）的在标点处切开；cue 里是纯文字，不带模型的标签；开头一行 `NOTE` 写语言。一段人声都没有时不写文件，`empty: true`。平台给的字幕（`raw/<snapshotId>/` 里 `kind: subtitle` 的 `.vtt`）原样读、不转，语言取那一项的 `lang`，没有就从文件名 `subtitle.<lang>.vtt` 取。
- **`play.mp4`**：分开存的画面和音轨不重编码地合成一个能播的文件（`-c copy`，`+faststart`），原样不动。
- **`keyframes.json`**（`urn:usuyo:schema:keyframes:1`）和 `keyframes/<毫秒，8 位补零>.jpg`：`{snapshot, durationSec, frames: [{sec, path, sha256, width, height, scene}], frozenSec}`；帧图片只放在转录文件夹的 `keyframes/` 下，`path` 不在那里的整次转录不收；`scene` 是镜头变化的分，不是镜头变化处取的帧为 `null`；`path` 相对转录文件夹。
- **`ocr.json`**（`urn:usuyo:schema:ocr:1`），`ocr` 和 `frames` 都写它：`{images: [{input, imageKey | frameSec, width, height, lines: [{text, bbox, confidence}]}]}`。`input` 是 `run.json` 里 `inputs` 的下标；图文的图和封面带 `imageKey`，关键帧带 `frameSec`，两个只带一个。`confidence` 是引擎自己的分（0–1），不跨引擎比。一次 `ocr` 转录可以盖几张图；快照新来了图，只为没盖到的再排一次。

<a id="4-contentmd"></a>
<a id="123-contentmd-和各栏"></a>
#### 1.2.3 content.md 和各栏

**`content.md` 是宿主由主快照生成的副本**，给不开 usuyo 的人（Finder、文本编辑器）和 grep 看；脚本要字段读 `manifest.json`。不带 front matter，开头是标题和出处，后面是正文：

```markdown
# 手冲咖啡：三段式注水

咖啡日常 · 2026-10-01 · douyin · video
<https://www.douyin.com/video/7431234567890123456>

---

（主快照的 text，没有就 derived/readable.md）
```

- 第一行 `# ` 加标题。
- 第二行：作者 · 发布日期（照 [§0.5](#05-时间)）· 平台 id · `kind`，缺的不写（`kind` 是 `unknown` 也不写）。写平台 id 不写显示名，不同版本的应用写出一样的字节。
- 第三行是链接：主快照的 `resolvedUrl`，没有就 `inputUrl`，再没有就身份的 `url`；写成 `<…>`，空格、`<`、`>` 百分号编码。
- 标题、作者照 [§0.6](#06-文本和文件) 的 Markdown 链接文字转义。
- **正文**（11-11 代码）：插件给的 `textFormat: markdown` 只有两条都满足才照原样写：全文没有 `<`；每个链接目标（`](` 或 `]:` 后面，跳过空白）以 `http://`、`https://`、`mailto:`、`usuyo://` 或 `#` 开头，不分大小写。不满足的和 `plain` 一样照 [§0.6](#06-文本和文件) 转义成纯文字。原样还在 `raw/`。

主快照或头部用到的字段变了就重写，原样在 `raw/`，不违背 [PRD 原则 2](PRD.md#4-原则)（[K5 A](PRD.md#710-存储删除与联网)）；CLAUDE.md 的措辞见 [§1.10 第 10 条](#110-要你看一眼)。应用里的原文栏和 `text` 引用读主快照的 `text` 或 `derived/readable.md`（下表），不读 `content.md`。**能看不能改**（11-05 定，[回答表](answers.md)）：宿主把它写成只读文件（系统支持时），总照主快照重写；重写前发现外面改过（和本机记下的上次写的哈希不一样，也不等于这台照现在的 manifest 和主快照会生成的字节；等于的话是另一台经云盘写来的，照收、记下哈希），先把改过的那份放进回收站（`kind: content`），再换回原样（推断）。要写字写在 `notes.md`、主题页里，或者导出到你的 vault 再写（[协议 §4.4](plugin-protocol.md#44-导出的-run-与-readback)）。它不同步，各台照原样自己生成。这改了 v0「机器永不改写正文」，也偏离[调研 14 §5](research/14-sync-like-hatoba.md#5-改动和冲突) 把它归「原样」层，调研 14 跟着改（[总纲](BUILD.md#5-功能线)）。

| 栏 | 文件 | 来源 | 链接里的层 |
|---|---|---|---|
| 原文 | 主快照 `raw/<snapshotId>/usuyo.fetch.json` 的 `text`，或 `derived/readable.md`；`content.md` 是只读副本 | 平台；机器 | `text` |
| 图里的字 | `ocr` 转录的 `derived/runs/<runId>/ocr.json` | 机器 | `ocr/<imageKey>`、`image/<imageKey>` |
| 口播 | 平台字幕 `raw/…/media/*.vtt`，加各次 `asr` 转录的 `derived/runs/<runId>/asr.vtt`，几份并排 | 平台；机器 | `asr`、`video`、`audio` |
| 画面 | `play` 转录的 `play.mp4`，没有就是原样的画面；加最新一次 `keyframes` 转录的关键帧 | 原样；机器 | `video` |
| 画面里的字 | `frames` 转录的 `derived/runs/<runId>/ocr.json` | 机器 | `video`，带 `#t=` 和 `xywh`（[§0.4](#04-id)） |
| 画面描述、AI 摘要 | `derived/vision.md`、`derived/summary.md` | 机器 | `vision`、`summary` |
| 你的笔记 | `notes.md` | 你 | `notes` |
| 评论（可选） | `raw/<snapshotId>/comments.json` | 平台 | `comments` |

评论默认不取，`settings.json` 里按平台打开、设上限；单条在条目页点「取评论」，那是一次只 `--want comments` 的获取，存成新快照，评论留着当时的样子。

<a id="13-条目的四个维度"></a>
### 1.3 条目的四个维度

| 维度（[PRD §7.1](PRD.md#8-条目的状态)） | 存在哪 | 怎么变 |
|---|---|---|
| 存下了多少 | `outputs`、`completeness`、`missing` | 宿主算；`completeness` 只升不降 |
| 后台任务到哪一步 | 本机 `jobs.db`；做好的看 `derived/` | 不进资料库，两台不会来回改 |
| 存在 | `state`；移除后就没了 | 你触发；Agent 或 `api` 令牌收下时把回收站里的拿回来、Agent 经 MCP 撤它自己那批时放回回收站也改它（[§1.6](#16-改动记录)、[§1.7](#17-删掉)）；到期移除除外 |
| 私密 | `private` | 你。不给 Agent（回 `not-found`，不带 `reason`），不进导出，同步默认照样加密上云，可选不上（开关归第 2 层，[§1.1](#11-目录树)） |

不记看没看过，也没有「没整理」（[§1.4.4](#144-发生过的事)）。

**给 Agent 和导出的是副本**：宿主把主题页、卡片、成稿复制一份给 Agent 和导出插件，原文件不动。副本里引用私密条目的整条脚注换成 `[^k3f7q]: 「私密条目」`；主题的成员由索引给，私密的不列。你正文里自己写的字不动。导出前提示这篇引用了几条私密条目。

<a id="14-你写的那一层"></a>
### 1.4 你写的那一层

链路照 [T1 A](PRD.md#74-变成文字)：素材（各栏、划线、记一句）→ 卡片 → 主题页 → 成稿。都是普通 Markdown，id 在 front matter 的 `id`（`notes.md`、`highlights.md` 没有，文件夹名就是条目 id）。不写修改时间，两台都改过会在这一行撞上，修改时间从改动记录取。画布等 [M7](PRD.md#76-地图)（建议一张地图）和画布原型，先留 `canvas/`，按 JSON Canvas 1.0，同步以节点 id 为键合并。

**标题和文件名**：`title` 是真相，文件名 = `sanitize(title)`（[§0.6](#06-文本和文件)），子文件夹都算，同步的记录带子文件夹、不带文件名。

- 本机为避免撞名加的「 2」不同步，不算改标题；两个同名时 `id` 字符串序大的那份在本机加，每台算出同一结果；同名的不自动合并，界面提示一行。

**usuyo 外改了资料库里的文件**：不是支持的用法，改东西走应用、MCP，或导出再读回（[协议 §4.4](plugin-protocol.md#44-导出的-run-与-readback)）。宿主只保证不丢、不悄悄复活。

- **怎么发现**：每次写回前比哈希（[§0.6](#06-文本和文件)），加上启动时和定时按大小、修改时间扫一遍；文件监听只是让它快一点（第 2 层）。等文件不再变了再处理，等多久归第 2 层。「外面改过」指和本机上次写的、读到的不一样。底本是 `sync.db` 里上次见过的那份，没开内置同步也存；没有底本就以文件现在的样子为准（[原则 3](PRD.md#4-原则)），改动记录那条不带 `change`。
- 下面三类每一类都拿这个情形试过：库在 iCloud、Dropbox 里时，另一台 usuyo 改这些文件是正常的。

1. **读进来**（同步的文件都是）：主题、卡片、成稿的 `.md` 和 `notes.md` 的 front matter 和正文；`highlights.md`；`manifest.json`，先过 schema，再按 [§1.2.1](#121-manifestjson-v1) 最后一列逐字段合并。和底本照 [§1.8](#18-同步) 三方合并，记 `actor: external`、`via: file`，能撤销。导出读回走同一条路：另一边是导出插件转回来的用户层（[协议 §4.4](plugin-protocol.md#44-导出的-run-与-readback)），底本取导出记录里的 `base`（[§1.1](#11-目录树)），不取 `sync.db`，没有底本照 [§1.8](#18-同步)「云盘的冲突副本」那条：集合取并集，标量和正文两边不一样就算合不上，留库里这份，vault 那份进回收站、出卡片；记 `via: readback:<插件 id>`。只合并插件交回的这几项值，下面挪动、改名、删掉、解析不了这几条不用于读回：标题和文件名不读回，vault 里删掉的下次导出重新生成，解析不了的照[协议 §4.4](plugin-protocol.md#44-导出的-run-与-readback) 列进这次导出的问题列表。
   - 主题、卡片、成稿挪到自己文件夹里的另一个子文件夹：记 `move`。
   - 主题、卡片、成稿改了文件名：去掉本机后缀后不等于 `sanitize(title)`，且 `title` 和底本一样，就当改标题，`title` 取新文件名；`title` 也变了以 `title` 为准，文件名照 `title` 改。front matter 没跟着变的改名先等一会儿再判，云盘常先送到改名（多久归第 2 层）。文件名对得上 front matter 的 `title` 就照收，另一台改了标题、经云盘过来的就是这样。
   - 删掉了，或移出 `topics/`、`cards/`、`drafts/`：记 `move-to-trash`，底本里的文本进回收站，能恢复；回收站里已经有它（库在云盘里，另一台已经挪进去）就不再记。
   - `notes.md` 或 `highlights.md` 删掉了：上次那份进回收站（`kind: notes`），条目 `state` 不变；别的台照 `edit-notes`、`set-takeaway` 或 `remove-highlight` 同步，不按回收站。
   - 条目文件夹或 `manifest.json` 删掉了：当 `move-to-trash`（`reason: delete`），回收站里放能找回的（`sync.db` 底本里的 manifest、`notes.md`、`highlights.md`）；回收站里已经有这一条（库在云盘里，另一台已经挪进去，manifest 是 `trashed`），或它的 id 在别的条目的 `aliases` 里（并进了别的条目或改了名），就不再记。
   - 解析不了的（front matter 不合 [§0.6](#06-文本和文件) 的写法、`highlights.md` 有一行不合 [§1.4.2](#142-卡片成稿记一句和划线) 的写法、JSON 坏了、`manifest.json` 的 `id` 和文件夹名不一样）：不改写、不同步，沿用上次的值，界面一行「这个文件在 usuyo 外改坏了」；你下次在应用里改这个对象时，坏的那份先进回收站（`reason: conflict`）。
2. **换回原样**（usuyo 算出来的字节）：
   - `content.md` 照 [§1.2.3](#123-contentmd-和各栏)：只在要重写时比和换，扫到改过不马上换；改过的那份先进回收站（`kind: content`），删掉了就重新生成。
   - `raw/` 少了或改了文件照 [§1.2.2](#122-raw-和-derived) 报「原样被改过」（推断）。
3. **不收**：`topics/`、`cards/`、`drafts/` 里没有 front matter `id`、或 `id` usuyo 不认识的 `.md`；条目文件夹里多出来的文件。不收、不同步，「检查与修复」里列出来。名字是云盘冲突副本样式的（[§1.8](#18-同步)）：原文件是上面「读进来」那几种的，先照那条合并；`content.md`、`derived/` 的副本直接移走。两个文件同一个 `id`：云盘的冲突副本照 [§1.8](#18-同步)；别的情况上次见过的路径那份留 id，另一份算这一类。

<a id="141-主题和主题页"></a>
#### 1.4.1 主题和主题页

一个主题一个文件：front matter 是 `id`、`title`、`createdAt`、`followed`、`keywords`、`match`，正文是那一页。

```markdown
---
id: "0193f2a4-7c1e-7a3b-9d2e-5f0a1b2c3d4e"
title: "手冲咖啡"
createdAt: "2026-11-05T03:00:01.123Z"
followed: true
keywords:
  - "手冲"
  - "闷蒸"
match:
  - "keywords"
---
闷蒸决定后面三段注水均不均匀。[^k3f7q]

[^k3f7q]: 「第一段 40 克闷蒸 30 秒」[手冲咖啡：三段式注水 · 口播 00:42](usuyo://item/douyin-4plrcxk343onmyxibmrr672kga/asr@0193f2b0-1d2e-7f3a-8b4c-5d6e7f8a9b0c#t=42,48) · 咖啡日常 · 2026-10-01
```

- `followed` 是「关注」，取消叫「不再关注」（[S11](BUILD.md#7-只有用户能答的和能验的)）；只有关注着的主题参与命中。`match` 给跟着收判断命中（[M1 B](PRD.md#76-地图)）：`keywords` 按 `keywords` 里的词字面命中，`similarity` 按和成员的相似度，可以都写。两种都看标题层（标题、简介、标签）和内容层（正文、口播转的文字、字幕、图里的字），画面描述默认不算，主题里能打开（11-06 定，[§1.10 第 7 条](#110-要你看一眼)；开关的字段归 feature）。看内容层要先把新条目取下来转成文字；没命中的不进资料库，取下来的临时文件删掉（推断）。卡片放进主题写在卡片自己的 `topics`；条目放进主题写在 manifest 的 `topics`（[§1.2.1](#121-manifestjson-v1)）。`create-topic` 的 `match` 默认 `[keywords]`。
- 列表一行一项；空列表不写，没有这个键就是空的。
- 成员记在成员上，加减成员不碰主题文件，主题页只因你改它而变（[PRD 7.7](PRD.md#77-你写的那一层)）；命中关注只记在 `origins[].matched`，不自动加成员。主题彻底移除以后，`topics` 里指向它的 id 读的时候跳过，不去改写。

<a id="142-卡片成稿记一句和划线"></a>
#### 1.4.2 卡片、成稿、记一句和划线

- `cards/<标题>.md`：front matter 是 `id`、`title`、`createdAt`、`topics`（放进的主题 id），正文是你的一个观点。没写标题时，新建那次取第一行前 30 个字写进 `title`，之后不跟着变。
- `drafts/<标题>.md`：front matter 是 `id`、`title`、`createdAt`、`kind`（`article` `post` `script` `other`）、`from`（用到的主题、卡片的 id，是哪种按 id 查）。引用主题页某段写 `usuyo://topic/<id>#^<块 id>`，顺着链接一路到原文那一段、那一秒。
- 块 id 谁加：你在主题页上「引用这一段」时，宿主在段末补 `^块id`，记 `edit-topic-page`（`actor: user`），能撤销；Agent 起草的成稿要引用某段时，补块 id 放进同一个提议。
- 记一句在 `notes.md` 头部的 `takeaway`，一条一句（[M9 A](PRD.md#76-地图)），没有就不写这个键，也就不带 front matter；正文是你的笔记。划线在 `highlights.md`，一条贴在某一栏的某一处。

`items/<platform>/<itemId>/notes.md`：

```markdown
---
takeaway: "闷蒸 30 秒比水温要紧"
---
你的笔记，随便写。
```

`items/<platform>/<itemId>/highlights.md`：

```markdown
- 「第一段 40 克闷蒸 30 秒」[口播 00:42](usuyo://item/douyin-4plrcxk343onmyxibmrr672kga/asr@0193f2b0-1d2e-7f3a-8b4c-5d6e7f8a9b0c#t=42,48) ^h7k2m
```

`highlights.md` 一行一条划线，不用注释圈：`- 「<原话>」[<栏 位置>](usuyo://item/<本条 id>/<层>@<snapshotId>#<片段>) ^<块 id>`，`@<snapshotId>` 照 [§0.4](#04-id) 哪些层带，不带标签、标题、作者、日期；块 id 见 [§0.4](#04-id)；原话照 [§0.6](#06-文本和文件) 转义。各行按加的先后排，同步按行合并，两台同时各加一行都留，按 `deviceId` 排先后。`add-highlight` 在末尾加一行；`remove-highlight`（`id`）删掉以 ` ^<id>` 结尾的那一行。usuyo 外改了照 [§1.4](#14-你写的那一层) 读进来，有一行不合写法就算这个文件写坏了。引用只抄原话和位置，不指向划线的块 id。

<a id="143-引用"></a>
#### 1.4.3 引用

```
[^<5 位随机>]: 「原话」[标题 · 栏 位置](usuyo://item/<id>/<层>@<snapshotId>#<片段>) · 作者 · 发布日期
```

- 只有宿主插入的引用（`add-citation`、从界面拖进来的）写成脚注，正文保持干净；你手写的行内引用照样认，不改写。脚注标签随机，两台各加一处不撞。
- 原话、标题、日期写在链接外：任何查看器里、来源删掉以后这一页都完整，应用里链接写「来源已删掉」。脚注是 GFM、Pandoc 都认的写法，纯文本里来源排在页尾。
- 发布日期照 [§0.5](#05-时间)：取 `publishedAtOriginal` 的日期部分，没有就取 UTC 日期。
- 标题、作者、日期在插入引用时抄下，之后不跟着刷新：那是你这一页的字。
- MCP 读主题、卡片、成稿时，宿主把脚注解析成 `{label, item, layer, snapshot, fragment, quote, title, author, date, resolved}` 一并给；`add-citation` 收其中的 `item`、`layer`、`snapshot`、`fragment`、`quote`，加 `anchor`（[§1.5.1](#151-op-词表)），标签宿主随机起，标题、作者、日期宿主插入时抄；Agent 不读写脚注语法（工具形状归第 2 层）。
- 时间类的 `@<snapshotId>` 是那次转录 `run.json` 的 `inputs[].snapshot`，即那份媒体所在的快照（[§0.4](#04-id)）。链接里 `#:~:text=` 的原话超过 32 个字写 `开头,结尾`；「」里的原话照全文存，不截。
- **找回**：时间类按「那份媒体还在、这一秒在时长内」判有效，原话只作提示；那一秒的文字变了（新引擎把「40 克」写成「四十克」），把现在的文字并排显示，不标「找不到」。文字类按 `#:~:text=` 模糊匹配（去标点、数字归一，推断），还找不到才显示存下的原话并标出来。

<a id="144-发生过的事"></a>
#### 1.4.4 发生过的事

[M6 A](PRD.md#76-地图) 只记发生过的事。记过一句、被引用从文件里看得出，只有「打开过」另记，一行一次：`{"n":1,"t":"2026-11-05T03:00:01.123Z","ev":"opened","item":"douyin-4plrcxk343onmyxibmrr672kga"}`，放 `.usuyo/facts/<deviceId>/`，分段和改动记录同一套（[§1.6](#16-改动记录)）。

- 短时间里反复打开只记一次，间隔归第 2 层。不写进 manifest：每打开一次就改一次条目记录，同步要多传很多。
- 「打开过几次、最近哪天」由索引算；索引不从 `facts/` 推出「没打开过」的列表、数字或标记（[原则 4](PRD.md#4-原则)）。
- `facts/` 默认不给 Agent，和「我的笔记」同一个开关：你打开过什么是行为数据。

<a id="15-改动提议"></a>
### 1.5 改动提议

Agent（经 MCP）或宿主调的模型改已有内容，一个提议一个文件 `.usuyo/changes/<id>.json`；放 `jobs.db` 会跟着机器丢（[第 14 条的回答](answers.md#14-资料库能不能放进同步盘)）。模型起草由宿主照[协议 §4.3](plugin-protocol.md#43-模型与密钥由宿主代发) 自己调 provider，不经转换插件。

<a id="151-op-词表"></a>
#### 1.5.1 op 词表

提议和改动记录共用。删除类 Agent 和模型都不能提议（[原则 5](PRD.md#4-原则)）。

| op | 对象 | 参数 | 能提议 | 能撤销 |
|---|---|---|---|---|
| `add-item` | 条目 | `origin`、`created` | 不经提议，直接生效（[A3 A](PRD.md#78-agent)） | 能，整批 |
| `set-tags` | 条目 | `add`、`remove` | 能 | 能 |
| `set-topics` | 条目、卡片 | `add`、`remove`（主题 id） | 能 | 能 |
| `set-takeaway`；`edit-notes` | 条目 | `text`；`hunks` | 能，要看得到笔记 | 能 |
| `add-highlight` | 条目 | `id`、`layer`、`snapshot`、`fragment`、`quote` | 能，要看得到笔记 | 能 |
| `create-topic` | 主题 | `title`、`keywords`、`match` | 能 | 能 |
| `edit-topic` | 主题 | `title`、`keywords`、`match` | 能 | 能 |
| `create-card`、`create-draft` | 卡片、成稿 | `title`、`body`；成稿另有 `kind` | 能 | 能 |
| `edit-topic-page`、`edit-card`、`edit-draft` | 主题、卡片、成稿 | `hunks`；`edit-card`、`edit-draft` 另有可选 `title`（同 `edit-topic`），`edit-draft` 另有 `kind` 和 `from` 的 `add`、`remove` | 能 | 能 |
| `add-citation` | 主题、卡片、成稿 | `anchor`、`item`、`layer`、`snapshot`、`fragment`、`quote` | 能 | 能 |
| `remove-highlight`（`id`）、`dismiss-suggestion`（`tag`）、`set-followed`、`set-private`（`value`）、`set-setting`（`key`、`value`） | 条目、主题、资料库 | 见括号 | 否 | 能 |
| `move` | 主题、卡片、成稿 | `path` | 否 | 能 |
| `set-not-same` | 条目 | `add`、`remove` | 否 | 能 |
| `move-to-trash`（`reason`）、`keep-text` | 条目；`move-to-trash` 也管主题、卡片、成稿 | 见括号 | 否 | 还在回收站里才能 |
| `restore` | 回收站里的 | — | 否 | 能 |
| `merge-items` | 条目 | `into` | 否 | 能，照 `change` 反着挪 |
| `purge` | 回收站里的 | — | 否 | 不能 |

- **要看得到笔记**：这几个改的是 `notes.md`、`highlights.md`，设置里没把「我的笔记」给 Agent 时回 `not-found`。
- **`hunks`** 是 `[{id, before, after, anchor?}]`，按段（空行分开）：原文是 `before` 的那几段换成 `after`；`before` 为空时插在原文等于 `anchor` 的那段后面，`anchor` 为 `null` 插在末尾。
- **`merge-items`**（A 并进 B）：A 的 `raw/<snapshotId>/` 整个移进 B 的 `raw/`；`snapshots`、`identities`、`origins` 并过去，`topics` 取并集，B 的 `aliases` 加上 A 的 id 和 A 的 `aliases`。A 的笔记正文按段接在 B 后面，记一句不一样时问你留哪句（同步自动并时出卡片）；划线：A 的 `highlights.md` 各行接在 B 后面，块 id 在 B 的文件里撞了就换一个。A 的 `manifest.json`、`content.md`、`derived/` 和一份 `notes.md`、`highlights.md` 进回收站（`reason: merge`），`derived/` 能重算；`raw/` 不进回收站：[K5 A](PRD.md#710-存储删除与联网) 只许「只留文字」和「删掉」把 `raw/` 移出资料库。
- 主题「合并进另一个」（[PRD §7.3](PRD.md#82-其他对象)）的 op 等主题 feature 再加。

<a id="152-提议文件"></a>
#### 1.5.2 提议文件

顶层 `id`、`createdAt`、`proposer`、`model?`、`via`、`summary`（Agent 写的一句）、`state`、`decidedAt`、`ops`；每个 op 是 `{id, op, target: {kind, id}, base, args, state}`，[§1.5.1](#151-op-词表) 的参数都放在 `args` 里，`hunks` 也是（11-11 代码，`{"op": "<名字>", "args": {…}}`）；改了再接受的另有 `edited: true`。`proposer` 是 `agent:<name>`，或宿主调模型时的 `model:<provider 名>`（另记 `model`）。`base` 是提出时目标文件的 `sha256:<hex>`，新建对象为 `null`。

- **状态**：提议 `proposed` → `accepted` / `partly-accepted` / `rejected`；op 和 hunk `proposed` → `accepted` / `rejected` / `conflict`，逐个决定。改了再接受记 `edited: true`，实际写进去的字在改动记录里。
- **不作废**（11-06 定，[§1.10 第 6 条](#110-要你看一眼)）：没决定的一直留着，不计数、不提醒（[原则 4](PRD.md#4-原则)）；接受时内容已经变了的照下面逐项对，对不上的标 `conflict`。决定完 30 天后，每台按同一规则从 `changes/` 移走，先移的推墓碑。
- **撞上后来的改动**：目标哈希还等于 `base` 就照原样应用。不等时，集合类（标签、成员）照样应用；hunk 在现文件里找得到唯一一处 `before` 就应用，否则 `conflict`，两份并排给你看；记一句改过也是 `conflict`。
- **两台**：状态只往前走；同一项两台给了不同终态，按 `accepted` > `rejected`（你还能撤销）。
- 目标是私密条目回 `not-found`。op 数和大小的上限归第 2 层，超了回 `too-large`。

<a id="16-改动记录"></a>
### 1.6 改动记录

`.usuyo/journal/<deviceId>/<4 位本台序号>.jsonl`，一行一条，每行带本台单调递增的 `n`，先后按 `n`，日期只在 `t` 里（钟拨乱过，按日期起名的段会排错）。满 256 KiB 或过了一天就封口（11-11 代码：「一天」按 UTC 日期，现在的 UTC 日期和这段第一行 `t` 的日期不同就开新段；序号从 `0001` 起，每行写完 fsync）；封口的段不再改，同步只传一次，例外只有移除时的脱敏（到期或立即清除，[§1.7](#17-删掉)）。

| 字段 | 取值 |
|---|---|
| `id` `n` `t` `device` | UUIDv7；本台序号；UTC 毫秒；`deviceId` |
| `actor` | `user`、`external`（外面改的文件）、`agent:<name>`、`model:<provider 名>`、`plugin:<id>`、`subscription:<id>`、`sync`（本机合并出的结果）、`host`（到期移除、迁移） |
| `via` | `app` `cli` `mcp` `api` `extension` `mobile` `import` `subscription` `file` `readback:<插件 id>` |
| `op` | [§1.5.1](#151-op-词表) |
| `targets` | `[{kind, id, path, before, after}]`，`before`、`after` 是目标文件改前改后的 `sha256:<hex>`，新建的 `before` 为 `null` |
| `change` | 反着做所需的内容：`add`、`remove`，段落原文，移动前后的路径 |
| `changeSet`、`changeOp` | 来自哪个提议的哪一项；有它就是你接受过的，`actor` 是提出的那方 |
| `batch`、`undoes` | 一次动作一批（接受一次、读回一次、一次 MCP 调用）；撤销的是哪一条 |

11-11 代码：每行都写 `via` 和 `change`；不存 op 的 `args`，反着做要的东西都在 `change` 里。

- **整批撤销**：从新到旧反着做，写成新的一批，每条带 `undoes`，旧记录不删；撤销的撤销就是重做。现哈希等于 `after` 就原样反做，不等就照 [§1.5.2](#152-提议文件) 逐项来，做不了的列出「这 2 处之后改过，没撤」。撤销批量导入那批，新建的同样进回收站（`reason: undo`），你之后写过的照 Agent 那批撤之前问你一次（推断）。已有 `undoes` 指向的不再反做；同一批两台同时撤销，同步后只认 `id` 小的那批，另一批在索引里算 `superseded`（推断）。
- **撤不了的**：`purge`、回收站已到期的；已导出到别的软件的；平台上做过的动作；Agent 已读走的内容。
- **Agent 直接收下**（[A3 A](PRD.md#78-agent)）：`add-item`，`actor: agent:<name>`、`via: mcp`，一次调用一批。已在库里、能给它看的只多一项 `origins`（`created: false`），撤销只去掉这一项；私密的不收，回 `skipped`；在回收站里的拿回来（[§1.7](#17-删掉)；离线短链展开后的见 [§0.4](#04-id)）。撤销新建的进回收站（`reason: undo`）；之后你写过、划过、放进主题或引用过的，在应用里和终端里的命令行撤之前问你一次也撤不撤（11-05 定，[§1.10 第 9 条](#110-要你看一眼)）；问不了时跳过并列出：命令行不在终端里这半是推断，Agent 经 MCP 见下一条。
- **Agent 经 MCP 能撤销哪些批**（11-05 定，[§1.10 第 18 条](#110-要你看一眼)）：只能撤同一个 Agent 的批。它这一批收进来或从回收站拿回来的条目，你没写过东西的放进回收站：新建的用 `reason: undo`，从回收站拿回来的照 [§1.7](#17-删掉) 原样放回。写过的照上面跳过并列出；你之后设成私密的也跳过，列给 Agent 时不列它（[§1.3](#13-条目的四个维度)）。你接受过的提议那批（带 `changeSet`）里新建的主题、卡片、成稿，经 MCP 撤时跳过并列出，要在应用里撤（推断）。不进回收站的步骤照常撤。撤掉它自己收进来的不算删除（[原则 5](PRD.md#4-原则)）。
- **外面改的**：导出读回的（`via: readback:<插件 id>`）和 usuyo 外改了、照 [§1.4](#14-你写的那一层) 读进来合并的（`via: file`）不走提议（[ADR-003 §7](decisions/ADR-003-plugins.md#7-导出与读回)），一样能撤销。`actor` 记 `external`，不断言是你：直接改文件的 Agent（Claude Code 一类）分不出来。第 2 层的 MCP 工具说明写明：改已有内容走提议，不要直接改文件。
- **活动记录**（[PRD 7.8](PRD.md#78-agent)）：由 `activity/`（在按机器的目录，搜过、读过，[§1.1](#11-目录树)）、`journal/`（写过）、`changes/`（提议过）拼出。

<a id="17-删掉"></a>
### 1.7 删掉

照 [K1 A、K3 A、K5 A](PRD.md#710-存储删除与联网)；K1 A 里的删除记录 11-06 去掉了：删除就是删除，移除以后不留记录（[§1.10 第 12 条](#110-要你看一眼)）；K3 的「彻底忘掉」11-06 并进删掉，不再单独有（[§1.10 第 16 条](#110-要你看一眼)）。

- **回收站**：`.usuyo/trash/<trashId>/` 里一个 `entry.json` 加文件；`entry.json` 是 `{id, kind, target, path, files, reason, trashedAt}`，`kind` 取 `item` `topic` `card` `draft` `notes` `content`，`path` 是原相对路径，`files` 是对象以内的相对路径，`reason` 为 `delete` `keep-text` `merge` `undo` `conflict`。各台各挪各的，同步的是条目的 `state` 和主题、卡片、成稿记录上的 `trashedAt`（推断，归同步引擎）。进回收站时取消 `jobs.db` 里它的任务，之后回来的结果丢掉，不在原路径重建文件夹。
- **回收站里怎么放**（11-11 代码）：`trashId` 必须是 UUIDv7。`path` 和 `files` 里每一项都只能是相对路径，不能有 `.`、`..`、绝对路径和盘符，不合的这一项当写坏了。

| 进回收站的 | 文件夹里 | `files` |
|---|---|---|
| 整条条目 | 条目文件夹原样改名进来，加 `entry.json` | 里面全部文件 |
| 主题、卡片、成稿 | `body.md` | `["body.md"]` |
| `notes`、`content` | 原文件名的那个文件（`reason: conflict`） | 那一个 |
| 只留文字 | 照条目里的相对路径放，例 `raw/<snapshotId>/images/001.jpg`、`derived/runs/<runId>/play.mp4` | 这些路径 |
- **到期**：照回收站项 `entry.json` 的 `trashedAt`（条目、主题、卡片、成稿都一样；2026-11-11 定，原先写的是条目按 manifest 的 `stateAt`），加当前的 `trashDays` 现算（[调研 14 §5](research/14-sync-like-hatoba.md#5-改动和冲突)「到期照 A 写的算」），「一直」不到期；`entry.json` 的 `trashedAt` 照同步来的值写，不用本机挪进来的时间；`merge`、`conflict`、`kind: notes` 和 `kind: content` 只在本机，按本机的。移除本机文件，记 `purge`（`actor: host`）；`keep-text`、`merge`、`conflict`、`kind: notes` 和 `kind: content` 到期只做这一步。对象整个离开资料库的（条目 `reason: delete` 或 `undo`；主题、卡片、成稿，`conflict` 的除外）还要清干净、推墓碑，见下面的「清干净」。开了同步的库启动后先拉完一轮再判，按服务器时间（[调研 14 §5](research/14-sync-like-hatoba.md#5-改动和冲突)），离线时不移除，推墓碑走 CAS，底本不是最新就放弃这次、重新判断。没开同步的按本机时间；本机时间比记下的最大时间晚了超过 `trashDays`（会把回收站整个移除），先不移除，你点「照时间移除」才移除（推断；这个放行还没做，[总纲](BUILD.md#5-功能线)）。你在回收站里「立即清除」（[PRD §7.3](PRD.md#82-其他对象)）照到期办，不等 `trashDays`（11-06 定，[§1.10 第 16 条](#110-要你看一眼)）；确认框列出还在引用它的页（推断）。11-11 代码：守护进程启动时查一遍，之后每 6 小时一遍（每遍都是整个回收站），到期的照下面移除。到期时间照回收站项 `entry.json` 的 `trashedAt` 加 `trashDays` 现算；`trashDays` 是「一直」时不到期。一项读不了的跳过、只记回收站 id 和错误码（日志里没有标题），不拦别的。时钟守卫：记下的最大时间放在 `libraries/<libraryId>/clock-mark`（还没有就取改动记录里最晚的一行）；这一遍有到期的、而现在比它晚了超过 `trashDays` 整天时，一项也不移除，只记下有几项因此留着（日志里的 `held`），`clock-mark` 也不更新。没有这种情况的话，这一遍结束把 `clock-mark` 更新成它和现在中较晚的。代码里有「照时间移除」这条路（跳过守卫），但还没有接口和界面能触发（还没做，[总纲](BUILD.md#5-功能线)）；每 6 小时的那一遍永远只用 `Check`，守卫拦下以后会不会自己放行是个开放的问题，也记在 TODO。回收站里一项也可以「立即清除」：`POST /trash/{trashId}/purge`，`trashId` 是回收站项的 id，不是条目 id；成功回 204，一项照到期办，整个离开资料库的条目发 `item-removed`，只留文字移出的媒体清掉时发 `item-updated`。`GET /trash` 列出回收站（[本机接口](#本机接口第-2-层11-11-照代码)）。
- **再遇到同一身份**（11-06 改，[§1.10 第 12 条](#110-要你看一眼)：不记删过的，移除以后再收下就是普通收下）：

| 入口 | 在回收站里 | 私密 |
|---|---|---|
| 你或 Agent 主动收的：窗口（粘贴、拖入、快捷键、`usuyo://add`）、命令行、扩展、手机、Agent（`mcp`）、本机接口令牌（`api`）、批量导入 | 拿回来，记 `restore`，能撤销；不提醒 | 窗口、终端里的命令行、扩展、手机、批量导入照常，多一项 `origins`；Agent、`api`、终端外的命令行跳过（11-11 还没做：`POST /items` 一律记窗口里收的，命令行从不跳过；导入不跳私密，扩展、手机和 `api` 令牌的投递会跳过。见 [总纲](BUILD.md#5-功能线)） |
| 跟着收（这个订阅没列过的） | 跳过 | 照常 |

- **跟着收**只收这个订阅没列过的（[§1.1](#11-目录树)）：列过的一律跳过，它列过、你删掉的，到期移除以后也不会被它收回来；它没列过的（比如你自己粘贴过、删掉、已经到期移除的），以后它第一次列出来就照新的收（这是代价）。库里已有、它没列过的不新建，只多一项 `origins`（推断）。`newest-first` 的来源翻到一整页都是这个订阅列过的就停（[协议 §3.8](plugin-protocol.md#38-list-与跟着收)）。
- 跳过的回 `skipped`，不加 `origins`；主动收的只有 Agent、`api`、终端外的命令行遇到私密条目这一种。又私密又在回收站里的：Agent、`api`、终端外的命令行照私密那一列跳过；别的入口拿回来（记 `restore`），多一项 `origins`，`private` 不变（推断）。代价：`skipped` 现在只在私密时出现，Agent 和 `api` 令牌能据此知道这条在库里、是私密的（11-05 以前和删过的、在回收站的不区分）；11-06 定照这样（[§1.10 第 20 条](#110-要你看一眼)），内容、标题都不给。
- **撤销收下**：新建的进回收站（`reason: undo`）；从回收站拿回来的原样放回：同一个回收站项（`trashId`），`trashedAt`、`stateAt` 照拿回来之前的，到期照原来的算（应用里、命令行、MCP 都一样）。原来的到期时间已经过了的，`trashedAt`、`stateAt` 记放回的时间，再留 `trashDays`（推断：不然一撤销就被移除，Agent 经 MCP 撤就成了删除，[原则 5](PRD.md#4-原则)）。
- **撤销那批的例外**：原因是 `undo` 的条目，和撤销收下时原样放回回收站的条目，同步带来你写的改动（`actor` 是 `user` 或 `external` 的记一句、改笔记、划线、放进主题、引用）时恢复它，记 `restore`（`actor: sync`），出一行「撤销时跳过了这条，你在另一台写过」。
- **清干净**（11-06，原来的「彻底忘掉」并进这里，[§1.10 第 16 条](#110-要你看一眼)）：改动记录里提到它（含它 `aliases` 里的 id）的各行（包括封口的段和这次记的 `purge`），`targets[].id`、`targets[].path`、`change` 换成 `{"redacted": true}`，其余字段不动；所有提议里它的 op 删掉；本机 `sync.db` 底本、`backups/`、`exports/`、`imports/`、索引里它的部分去掉。条目另外：`facts/` 里它（含 `aliases`）的行删掉；并进它的（`reason: merge`）和它只留文字时移出来的（`reason: keep-text`）回收站项一起移除；别的条目 manifest 的 `sameAs`、`notSameAs`、`aliases` 去掉它；你的主题页、卡片、成稿、笔记里原话和标题留着，链接里的 id 是它或它的 `aliases` 的，换成 `usuyo://item/forgotten`（`actor: host`），应用里写「来源已删掉」（[§1.4.3](#143-引用)）；`jobs.db` 里跟着收列过的只有身份的哈希，不去掉（推断：去掉了，跟着收会把它再收回来）。云端推墓碑，多久后云端也没有了照[调研 14 §5](research/14-sync-like-hatoba.md#5-改动和冲突)（D1 时间回溯、R2 桶锁），界面写明。
- **移除时写什么**（11-11 代码）：只在本机的（`keep-text`、`merge`、`conflict`、`kind: notes`、`kind: content`）删掉文件夹，记一行普通的 `purge`，`change` 是 `{trashId}`。整个离开的记一行 `purge`，`targets[].id`、`targets[].path`、`change` 都是 `{"redacted": true}`；再照上面清干净：所有设备的段里，`targets[].id` 是它、路径里或 `change` 里出现它（含 `aliases`）的行都脱敏；提议里它的 op 删掉，删空了的提议文件删掉；页面里的链接换成 `usuyo://item/forgotten`，回收站里的副本也换，记一行 `purge`（`actor: host`，`targets` 是改过的文件）。索引、`sync.db`、`backups/`、`exports/` 由调用方清。还不推墓碑（同步还没做）。
- 别的台拉到墓碑后照上面移除自己的文件、清干净自己手上的。条目、主题、卡片、成稿的墓碑只在清干净时推，不另带标记，移除时不在线的那台回来拉到也照办（推断）；它离线时改过或拿回来的照下面「和同步」，那一份等它自己到期再清干净。改写是确定的：每台先改自己手上全部段的副本（包括别的台写的）；各段的主人再把改过的段推一次，记录换成新的 blob id，旧 blob 照[调研 14 §5](research/14-sync-like-hatoba.md#5-改动和冲突) 的清理规则清掉，以后加入的设备拉到的就是改过的。主人已被撤销（`devices/<deviceId>.json` 有 `revokedAt`）时由推墓碑的那台推，这是「每段只有一台写」唯一的例外。同一条一台重新收下、一台移除：拉到对方的版本后条目还在的，那台不做这些改写，界面一行「另一台又收下了，没清掉」（推断）。
- **导出到 Obsidian 的那份**（11-06 定，[§1.10 第 15 条](#110-要你看一眼)）：归那边，usuyo 删掉时不去动它。读回遇到库里没有或在回收站的 `usuyo_id`，由宿主判成 `item-deleted`（不由插件报），那份文件不读也不写，只在这次导出的问题列表里写一行，不重建文件夹；插件报的 `problems` 里的文件同样不读也不写（[协议 §4.4](plugin-protocol.md#44-导出的-run-与-readback)）。
- **只留文字**：先生成 `derived/thumb.jpg`，再把 `raw/` 里的图片、视频、音频和各次转录里的 `play.mp4`、`keyframes/` 移进回收站（`reason: keep-text`），`run.json` 和文字产物（`asr.vtt`、`ocr.json`、`keyframes.json`）留着，字都留着（11-11 代码：照各快照 `usuyo.fetch.json` 里的 `images` 和 `kind` 是 `video`、`audio` 的 `media` 移，加各次 `play` 转录合成的视频和各次转录 `keyframes/` 下的图片；只移解析掉符号链接以后还在条目文件夹里的，指到外面的留在原处、记日志；`keyframes` 本身是符号链接的不跟进去；字幕和 `other` 留着；算文字类的输出是 `text`、`html`、`subtitle`、`meta`、`comments`；`thumb.jpg` 还没生成）；`state: text-only`，以后只 `--want` 文字类。回收站里还在时能恢复，之后点「重新取视频」存新快照。
- **和同步**：一台删、另一台同时改了笔记，回收站赢（撤销那批例外），改动并进回收站那份，出「两边都改过的」卡片。已移除、离线的那台回来时改过它：推的时候撞上墓碑，就放进它自己的回收站再留 `trashDays`，不悄悄复活。离线那台主动收下、从回收站拿回来的不算这种：照 [§1.8](#18-同步) 以墓碑的 `revision` 为底写新版本（推断）。

<a id="18-同步"></a>
### 1.8 同步

按[调研 14 §5](research/14-sync-like-hatoba.md#5-改动和冲突) 的四层。本机是明文，只有离开本机的加密。S1、S2 是[调研 14 §2](research/14-sync-like-hatoba.md#2-本产品要同步什么) 的分期：S1 只同步文字，S2 媒体。

| 文件 | 层 · 哪一期 | 怎么合并 |
|---|---|---|
| `raw/` 的字；网页、图片、视频 | 原样 · S1；S2 放 R2 | 只增；同一路径内容不同只会是出错，两份都留；和 `usuyo.fetch.json` 对不上就不上传 |
| `manifest.json` | 混合 · S1 | 按 [§1.2.1](#121-manifestjson-v1) 最后一列 |
| `content.md` | 机器 · 不同步 | 各台照主快照自己生成，见 [§1.2.3](#123-contentmd-和各栏) |
| `derived/` 的字（含 `runs/<runId>/` 的 `run.json`、`asr.vtt`、`ocr.json`、`keyframes.json`）；`play.mp4`、`keyframes/`、`thumb.jpg` | 机器 · S1；S2 | 先推上服务器的留下，不出卡片；转录写了不改，同一个 `runId` 不会两边不一样 |
| `embeddings.bin` | 机器 · 不同步 | 各台自己算 |
| `notes.md`、`highlights.md`、`topics/`、`cards/`、`drafts/`、`canvas/` | 你写的 · S1 | 见下 |
| `library.json` | 元数据 · S1 | `formatVersion` 要一样，远端新就不同步 |
| `settings.json`、`subscriptions/`、`devices/` | 元数据 · S1 | 按字段，同一项两边改出卡片 |
| `changes/` | 元数据 · S1 | 并集；任何去掉都推墓碑；提议状态只往前走 |
| `journal/`、`facts/`、`imports/` | 元数据 · S1 | 每段只有一台写；到期或立即清除时的脱敏见 [§1.7](#17-删掉) |
| `trash/` | 只同步状态 | 各台各挪各的 |

- **集合三方合并**（manifest 的集合字段、front matter 的列表）：`sync.db` 存上次见过的底本（没开同步也存，[§1.4](#14-你写的那一层) 外面改了靠它合并）。两边各自加的都留，一边去掉的就去掉；没有底本取并集，宁可多留。
- **你写的那层**：front matter 按字段三方合并，列表（`keywords`、`match`、`topics`、`from`）按集合，标量只有同一字段两边都改了才出卡片；front matter 只有 usuyo 写、写法固定（[§0.6](#06-文本和文件)），两台写出一样的字节。`notes.md`、`highlights.md`、主题页、卡片和成稿的正文按行三方合并；两边在同一处都只是插入（各加一处引用、脚注追加在末尾、各加一条划线）时两段都留，按 `deviceId` 排先后。只有改到重叠的行才算合不上，相邻的不算；两边改成一样的（插了同一段、改了同一行、标量改成同一个值）算一次，不重复、不出卡片；三方合并接受任意底本（`sync.db` 的，或导出记录里的 `base`）；导出读回时 vault 那边没有 `deviceId`，同一处都插入时排在库里那份后面（推断）。合不上的：先推上服务器的当现在这份，CAS 失败的那台负责合并，不比时钟；输掉的那份进回收站（`reason: conflict`），「两边都改过的」卡片指向它；在回收站里点恢复它，就是换上它：记成一次改动（`actor: user`），现在那份不另进回收站，撤销这次就换回来（推断）。
- **墓碑**是记录的一个版本，带 `revision`；再收下或恢复时按 CAS 以墓碑的 `revision` 为底写新版本，推墓碑也走 CAS。Worker 支不支持，同步引擎实现时核实（推断：Hatoba 的 `UPDATE … WHERE revision = ?` 对墓碑行一样适用）。
- **自动合并**：拉下来的或本机新建的条目，id 在别的条目的 `aliases` 里、或身份对上，就照 `merge-items` 并过去（`actor: sync`），不出卡片；按身份查找时先查 `aliases`。
- **记录 id** 是 `HMAC-SHA256(idKey, "<种类>:" + id)`，种类 `item` `topic` `card` `draft` `canvas` `library` `changeset` `journal` `facts` `setting` `subscription` `device` `import`；种类字符串进了云端 id，改它算 major。
- **放法**：改动记录的段、发生过的事的段、提议文件按内容寻址进 D1 的 `blobs`（超过 16 KiB 切成几行，[调研 14 §3](research/14-sync-like-hatoba.md#3-存在哪和花多少)），记录里只放 blob id：信封上限 64 KB，一段 256 KiB 放不下。
- 别的设备的改动在那台已确认过，这台直接生效，记录带着它的 `device`。
- **库放在 iCloud、Dropbox 里时**（不开内置同步）：`devices/<deviceId>.json` 定时写 `lastSeenAt`（间隔归第 2 层），打开库时看到别的台最近写过，提示「另一台也开着」。另一台经云盘写来的改动它自己的改动记录里也有，这台怎么不重复记，归同步 feature。
- **云盘的冲突副本**：认出 iCloud 的「 2」和 Dropbox 的「conflicted copy」（名字样式归第 2 层的一张表）。题名来的文件再看 id：和原文件同一个 id 才是副本，id 不同是本机撞名加的「 2」。副本当另一边，照上面的规则和 `sync.db` 底本三方合并；没有底本，`manifest.json` 照 [§1.2.1](#121-manifestjson-v1) 最后一列；别的集合取并集，标量和正文两边不一样就算合不上、留这边。合不上的出「两边都改过的」卡片；副本和合并结果不一样就进回收站（`reason: conflict`），一样就直接从文件夹里移走。
- **和[调研 14](research/14-sync-like-hatoba.md#5-改动和冲突) 不一样的**：`derived/` 和你写的那层合不上时都是先推上服务器的留下，不比时钟；你写的那层输掉的那份进回收站，不放 `sync.db`；文件夹不按年月分，不用[调研 14 §2 第 1 条](research/14-sync-like-hatoba.md#2-本产品要同步什么)的改名（[§0.9 第 1 条](#09-要你看一眼)）。

<a id="19-schema-和-pkgcontract"></a>
<a id="19-schema-和-usuyo-contract"></a>
### 1.9 schema 和 usuyo-contract

磁盘上没有 v0 资料库，不写迁移程序。原来的 Go 包 `pkg/contract` 11-11 搬成了 `crates/usuyo-contract`（[ADR-004](decisions/ADR-004-rust-tauri.md)）：磁盘上各记录的 serde 类型，每个对象带 `extra`，不认识的字段原样写回；下面几条 schema 在 `crates/usuyo-contract/schema/`，样例在 `testdata/`。v0 到本层的对照（历史，留着查）：

- 目录：`items/YYYY/MM/<platform>-<id8>-<rand4>/` → `items/<platform>/<itemId>/`；`index/`、`jobs.db`、`config.json` 搬出 `.usuyo/`；`threads/`、`thoughts/`、`canvas.json` → `topics/`、`cards/`（按标题起名）、`canvas/`；`journal/YYYY-MM.jsonl`、`.trash/` → `.usuyo/journal/<deviceId>/`、`.usuyo/trash/`；条目根下的 `images/`、`media/` 进 `raw/<snapshotId>/`，`images/keyframes/` → `derived/keyframes/`。
- manifest：`schema` → `formatVersion`；`capturedAt`、`origin` → `addedAt`、`origins[]`（`sync` → `subscription`）；快照的 `adapter`、`adapterVersion` → `plugin`、`pluginVersion`，`rawDir` 去掉；`missing` 用 `outputs` 词表；`slimmed` → `text-only`；`jobs`、`fingerprints` 进 `jobs.db`、`derived/`；`attention.json` 去掉（[M6 A](PRD.md#76-地图)）；不写 `additionalProperties: false`。

schema 先定这几条：

- `$id`：`urn:usuyo:schema:manifest:1`、`…:changeset:1`、`…:journal-entry:1`、`…:topic:1`、`…:card:1`、`…:draft:1`、`…:notes:1`、`…:run:1`、`…:keyframes:1`、`…:ocr:1`。
- manifest：`author`、`publishedAt`、`publishedAtOriginal`、`lang`、`missingReason`、`stateAt`、`textOnly` 以外都必填（`topics` 也必填，可以是空数组）；`id` 匹配 `^[a-z0-9][a-z0-9-]*-[a-z2-7]{26}$`；`primarySnapshot` 可为 `null`；`origins` 至少一项，每项必填 `id` `entry` `takenBy` `at` `device`；快照必填 `id` `plugin` `at` `access` `outputs` `missing` `completeness`。`completeness`、`state` 是封闭枚举，`kind`、`outputs` 写 `examples` 不写 `enum`。
- changeset：必填 `id` `createdAt` `proposer` `state` `ops`，op 必填 `id` `op` `target` `base` `state`；`proposer` 匹配 `^(agent|model):`；`base` 为 `null` 或 `^sha256:[0-9a-f]{64}$`。
- journal-entry：必填 `id` `n`（从 1 起）`t` `device` `actor` `op` `targets` `batch`；`actor` 匹配 `^(user|external|sync|host|agent:[a-z0-9-]+|model:.+|plugin:.+|subscription:[0-9a-f-]{36})$`；脱敏后的 `targets[].id`、`targets[].path`、`change` 是 `{"redacted": true}`，所以允许对象。
- run：[§1.2.2](#122-raw-和-derived) 的字段全部必填，可为 `null` 的是 `tool`、`model`、`redoOf`、`reason`；`id`、`device`、`redoOf`、`inputs[].snapshot` 是 UUIDv7，`at` 是 UTC 毫秒；`sha256` 是 64 位小写十六进制；`inputs[].path`、`outputs[].path` 是普通相对路径；`empty: true` 时 `outputs` 为空、`reason` 有值，否则 `reason` 为 `null`。`trigger`、`reason`、`marks[].code` 写 `enum`，`job`、`outputs[].kind` 写 `examples`。keyframes、ocr：插件写、宿主读的产物，`bbox` 四个数都在 0–1，`imageKey` 和 `frameSec` 恰好一个。样例有完整的 `asr`、`no-speech`、宿主写的 `no-audio-track`、`keyframes`、输入是另一次转录的关键帧的 `frames`。
- topic、card、draft、notes：描述解析出来的 front matter 对象（[§1.4](#14-你写的那一层)）。必填：topic、card 是 `id` `title` `createdAt`，draft 另加 `kind`，notes 没有；`id` 匹配 `^[0-9a-f]{8}-[0-9a-f]{4}-7[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$`，时间是 RFC 3339；同样不写 `additionalProperties: false`。

<a id="110-要你看一眼"></a>
### 1.10 要你看一眼

| # | 什么 | 建议 |
|---|---|---|
| 1 | [§1.1](#11-目录树) 条目放在 `items/<平台>/<itemId>/` | **已定（11-06）**：按平台分一层；全放一层，上万个文件夹在 Finder 里难翻（[回答表](answers.md)） |
| 2 | [§1.1](#11-目录树) 回收站放哪 | **不再问（11-05 方向）**：照旧放 `.usuyo/trash/`，理由是它是库的状态，见 [§1.1](#11-目录树)；[回答表](answers.md) |
| 3 | [§1.4.1](#141-主题和主题页) 主题成员放哪 | **不再问（11-05 方向）**：成员记在成员上：条目 manifest 和卡片的 `topics`，见 [§1.4.1](#141-主题和主题页)；[回答表](answers.md) |
| 4 | [§1.4.3](#143-引用) 引用怎么写 | **不再问（11-05 方向）**：脚注照旧，理由是通用 Markdown、来源删了也完整，见 [§1.4.3](#143-引用)；[回答表](answers.md) |
| 5 | [§1.4.2](#142-卡片成稿记一句和划线) 记一句和划线放哪 | **不再问（11-05 方向）**：记一句是 `notes.md` 头部的 `takeaway`，划线放 `highlights.md`，见 [§1.4.2](#142-卡片成稿记一句和划线)；[回答表](answers.md) |
| 6 | [§1.5.2](#152-提议文件) 改动提议多久没决定就作废（[A2](PRD.md#78-agent)） | **已定（11-06）**：不作废，一直留着，不计数、不提醒（[回答表](answers.md)） |
| 7 | [§1.4.1](#141-主题和主题页) 跟着收的命中怎么判、看哪几层字 | **已定（11-06）**：默认按关键词，相似度你打开才用，命中收下的都标「机器挑的」（[第 14 条](#110-要你看一眼)）；看标题层和内容层，画面层默认不算（[回答表](answers.md)） |
| 8 | [§1.2.3](#123-contentmd-和各栏) `content.md` 是给「不开 usuyo 也能读」的正文副本，和导出到 vault 无关；它能不能改 | **已定（11-05）**：能看不能改，只读文件，总照原样重写；外面强行改过的那份先进回收站（[回答表](answers.md)） |
| 9 | [§1.6](#16-改动记录) 撤销 Agent 收下的一批时，你写过东西的条目怎么办 | **已定（11-05）**：撤之前问一次也撤不撤；问不了时跳过并列出（推断）（[回答表](answers.md)） |
| 10 | [§1.2.3](#123-contentmd-和各栏) [CLAUDE.md](../CLAUDE.md) 原则 2 照 [PRD 原则 2](PRD.md#4-原则) 改成「机器永不改写或删除原样」，`content.md` 算宿主生成的只读副本；原则 3 的「删掉 `.usuyo/index/`」改成「删掉索引目录」；文档地图里 SPEC 那行改成 v1 草案 | **已定（11-05）**：三处都照这样改了（[回答表](answers.md)） |
| 11 | [§1.7](#17-删掉) 删除记录多记一个删掉的时间 `deletedAt`（[K1 A](PRD.md#710-存储删除与联网) 说只记身份和标题） | **不再问（11-06 不留删除记录）**：删除记录去掉了，`deletedAt` 和「删掉过的」列表跟着没了；[回答表](answers.md) |
| 12 | [§1.7](#17-删掉) 主动收下一条以前删过的怎么办 | **已定（11-06 改）**：不记删过的，在回收站里的直接拿回来，不提醒（[回答表](answers.md)） |
| 13 | [§1.3](#13-条目的四个维度) 成稿引用了私密条目，导出时怎么办 | **先放着（11-06）**：导出到 Obsidian 和给 Agent 分开定，等有界面和具体例子再定；[§1.3](#13-条目的四个维度) 现在的写法（只换那条脚注）是暂定（[回答表](answers.md)） |
| 14 | [§1.2.1](#121-manifestjson-v1) Agent 自己挑着收下的算不算「机器挑的」 | **已定（11-06）**：机器找的、机器挑的都算，包括按关键词找的、跟着收按关键词命中的；只有你给的具体链接和你自己平台上的收藏、列表不算（[回答表](answers.md)） |
| 15 | [§1.7](#17-删掉) 删掉以后，导出到 Obsidian 的那份还在 vault 里 | **已定（11-06）**：不管，导出的归那边，usuyo 不去动它；读回碰到已经删掉的只在导出的问题列表里写一行（[回答表](answers.md)） |
| 16 | [§1.7](#17-删掉) 删掉的到期或立即清除以后，你主题页里引过它的原话和标题还留着 | **已定（11-06）**：删掉到期或立即清除时照这样：原话、标题留着，链接换成「已删掉」；不再单独有「彻底忘掉」（[回答表](answers.md)） |
| 17 | [§1.6](#16-改动记录) 撤销 Agent 收下的一批时不留删除记录，以后跟着收还可能再收进来 | **不再问（11-06 不留删除记录）**：撤销和删掉都不留记录；跟着收跳过这个订阅列过的和回收站里的，到期移除以后它没列过的照收（[§1.7](#17-删掉)）；[回答表](answers.md) |
| 18 | [§1.6](#16-改动记录) Agent 经 MCP 能撤销哪些批：撤销新建、撤销恢复会把东西移进回收站，[原则 5](PRD.md#4-原则) 说 MCP 不给删除 | **已定（11-05）**：能撤同一个 Agent 自己收进来、你没写过东西的，放进回收站；写过的跳过并列出；原则 5 写明这不算删除（[回答表](answers.md)） |
| 19 | [§1.2.1](#121-manifestjson-v1) 「机器挑的」是来历，还是「你还没碰过」 | **已定（11-05）**：是来历，你碰过也一直留着（[回答表](answers.md)） |
| 20 | [§1.7](#17-删掉) 删过的照收以后，`skipped` 只在私密时出现：Agent 和 `api` 令牌拿一条链接来试，就知道它在你库里、是私密的 | **已定（11-06）**：照这样，写明代价：只露「在库里、私密」这一点，内容、标题都不给（[回答表](answers.md)） |

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
- **投递令牌**：格式 `<id>.<秘密>`，`id` 是 16 位小写十六进制（不是秘密），`秘密` 是 64 位小写十六进制。主令牌调 `POST /deliver-tokens`（`{name, kind}`，`name` 1 到 80 个字，`kind` 取 `extension`、`mobile`、`api`）发一个，整个令牌只在这一次的回答里出现（在 `{id, name, kind, createdAt}` 之外多一项 `token`）；`GET /deliver-tokens` 列出投递端，没有令牌；`DELETE /deliver-tokens/{id}` 撤销，马上生效，回 204，没有这个 id 回 404。存在按机器目录的 `deliver-tokens.json`，见 [§1.1](#11-目录树)。投递令牌的 `kind` 决定条目 `origins[].entry`：`extension`、`mobile`、`api`。
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

- **错误**：照 [§0.3](#03-错误模型)：`application/problem+json`，字段 `type`（`urn:usuyo:problem:<code>`）、`title`、`status`、`detail`、`instance`（`urn:uuid:<随机>`）、`code`、`retryable`；插件报的另带 `plugin`，一律 502。可重试的带 `Retry-After: 5`。没有的路径 404 `not-found`，方法不对 405 `invalid-args`。
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
| `GET /inbox` | `{path}`，收件文件夹在哪（[§1.1](#11-目录树)） |
| `GET /v1/ping` | 投递：`{name: "usuyo", version, protocol: 1, client}`，`client` 是投递令牌的名字，主令牌是 `null` |
| `POST /v1/deliver` | 投递：见下 |
| `POST /v1/batches` | 投递：见下 |

守护进程另有四条，不在 `endpoints.ts` 的 `paths` 里：`POST /handshake`（上面）；`POST /reindex`，命令行用，回 `{items, unreadable}`；`GET /media/<itemId>/<层>`，地址由 `ItemView` 给；`GET /events` 在 `paths` 里，但不走 30 秒的时限。

**投递**（[协议 §5.1](plugin-protocol.md#51-本机-http-接口)）：
- `POST /v1/deliver`，`{url?, text?}`，给一个（两个都给取 `url`）。`text` 是手机分享出来的整段文字，从里面找第一个链接。只收链接，已经取到的内容（multipart）还不收。回 202 `{id, state: "accepted", duplicate}`，`duplicate` 为真是以前收下过。条目的入口取令牌的 `kind`，取内容照常排任务。遇到私密条目不收、不加来源，现在的回答和「收下过」一样（`duplicate: true`），没有单独的 `skipped`。
- `POST /v1/batches`，`{name?, action?, links: [{url, title?, actedAt?}]}`，进导入的预览，你确认了才按节奏去收。`links` 1 到 5000 条；`action` 取 `favorite`、`like`、`share`、`view`、`unknown`；每条 `url` 要是带主机的 http(s)、不带用户名密码，`actedAt` 要是 RFC 3339，`title` 超过 500 字截掉；`name` 不给写「交进来的一批」，最多 200 字。不合的回 400 `invalid-args`，说是第几条。回 202 `{id}`，是新建的导入的 id（`kind: delivered`，`client` 是令牌名）。每条的身份由宿主重新算，不信交来的；认不出的这一条去掉，记日志；清单里身份相同的合成一条。

**导入**：一次导入的状态不单独存，由几项现算：`listing`（还在列）、`ready`（列完了，等你看一眼）、`taking`（确认了，还有排着的任务）、`done`（确认了，没有排着的了），`held` 盖过其余，是你点了「先停下」。走向：
1. 开始（`POST /imports` 或 `POST /v1/batches`）：`source` 类要这个插件能列（不能回 422 `unsupported-input`），来源非空、最多 512 字节，平台没声明 `batch: false`（声明了回 422）。`source` 类按平台的节奏一页页调插件的 `list`，`delivered` 类每轮认 50 条的身份。每条按身份分成 `new`、`stored`（库里已经有）、`trashed`（在回收站里）；认不出来的是 `unresolved`。这一步不建任何条目。
2. 列完是 `ready`。`POST /imports/{id}/confirm`，请求体 `{new, trashed, except?}`（不认识的字段回 400）：`new` 是「新的」整组收不收；`trashed` 是「回收站里的」整组收不收，收下就是拿回来；`except` 是 `seq` 的数组，这些条目和自己那组选得反着来（整组收的，它不收；整组不收的，它收）。只交例外：预览按页看，没翻到的条目跟着整组走。`stored` 和 `unresolved` 的不收，也不能出现在 `except` 里，出现了回 400。还在列的、或已经确认过的再确认，回 400。
3. 确认后，挑中的每条排一个取内容的任务，优先级低于你手动收下的；按平台的节奏一条条来。轮到时先收下（入口 `import`，`takenBy: user`，同一批共用 `importBatch`，带 `action`、`actedAt`），再取内容。这一条已经有任务（你刚手动收过）的不动。轮到时发现已经进了回收站、而你没挑回收站那一组的，不收，`skipped` 加一；私密条目批量导入照收（[§1.7](#17-删掉) 的表）。`taken` 是收下的条数，`notStored` 是收下了但内容取不到的。
4. `POST /imports/{id}/hold` 随时可以，不再列、不再收，已经收下的不动；`resume` 从停的地方接着，停在要你处理的原因上（例如重新登录以后）也从这里再试。导入页只说事实（列了几页、各几条、收下几条、为什么在等和什么时候），不给还剩多少。

列的时候的错：联网、超时、插件崩了这一类，等 60 秒乘 2 的次方再试，试满 6 次就停下等你；被限流、被拦住记在插件的暂停上（见下），导入照它显示原因，暂停一去掉就接着列；别的错停在这次导入上，`waiting.code` 是错误码（例如 `login-required`，或页面把没有访客身份的请求送回首页时插件报的 `visitor-required`），没有 `until`，等你重新登录或取到访客身份以后点「接着导入」。`waiting.code` 还可能是 `pacing`（等间隔）、`daily-limit`（今天批量的次数用完了）、`held`（你让插件先别试）、`batch-off`（这个平台不做导入）。

**节奏和暂停**（[协议 §3.1](plugin-protocol.md#31-运行器与节奏)）：
- 节奏来自装着的插件清单里的 `pacing`，同一平台的几个插件取最严的：间隔取最长的，`daily` 取最小的，有一个写了 `batch: false` 就不做导入。没声明的平台不等。
- `interval`：上一次调用结束到下一次开始的最短间隔，`fetch`、`list` 的一页、`sources` 都算，你手动收下的也守，导入的和手动的在同一个平台上排队。`daily`：批量（导入）的调用一个本地日期最多几次，到本地零点重新数；你手动收下的单条不计入，也不被它挡住，只守间隔。同一平台同一时间只跑一个调用。记账在 `jobs.db`，重启后照旧。`canonicalize` 不计。
- 列一页要等的超过 60 秒就不等，记下到点的时间，到点再来。
- 插件报 `rate-limited`：暂停到 `retryAfter` 以后（没给 15 分钟，最多算 1 小时）。报 `blocked`：现在也只暂停 1 小时，到点自己解开；你点「现在再试」更早去掉它。暂停只管这个插件在这个平台上（web 插件按主机），别的插件照常；暂停期间它不被调用，排着的任务往后排，不白白耗掉重试次数。`hold`（「先别试」）没有 `until`，是 `byYou`，一直到「现在再试」。暂停和它的错误码、一句话原因在 `GET /plugins` 的 `pauses` 里。

**命令行**：`add`、`get`、`search`、`reindex` 先读 `daemon.json` 和 `api-token`，握手，再 `GET /health`；都成了就经守护进程，不然在本进程里直接开库。握手没过不发令牌。`init` 总在本进程。

**和别处还不一样的**：窗口用的路径不带 `/v1/`（只有投递接口带）；列表回普通数组，没有游标（[§0.8](#08-分页和游标)）；用主令牌经窗口用的路径做的写操作都记 `via: app`，`POST /items` 收下记 `entry: paste`、`takenBy: user`，私密条目照收，命令行经守护进程收下的也一样（[§1.7](#17-删掉) 的表要求 `api` 和终端外的命令行跳过私密）；经 `/v1/deliver` 的投递，扩展、手机、`api` 令牌遇到私密条目会跳过，终端里和终端外的命令行（本进程里直接收下时记 `cli`）都不跳。

<a id="2-留着的-v0第-2-层重写前保留"></a>
## 2. 留着的 v0（第 2 层重写前保留）

下面六节是 v0 原文，第 2 层（内核服务契约）重写时替换；和第 0、1 层冲突的以第 0、1 层为准，每节开头写了已知的几处。都是历史：现在的代码见 [ADR-004](decisions/ADR-004-rust-tauri.md) 和 `crates/`，本机接口见[上一节](#本机接口第-2-层11-11-照代码)。

<a id="1-仓库布局monorepo"></a>
<a id="21-仓库布局monorepo"></a>
### 2.1 仓库布局（monorepo）

v0，第 2 层重写前保留。`adapter` 照插件协议改叫插件，归第 3 层。历史，下面的 Go 布局 11-11 整个换掉了：现在是根目录的 cargo 工作区，`crates/`（各 crate 见 [§0.1](#01-命名)）、`apps/desktop/`（Tauri 2 壳 `src-tauri/` 加前端 `web/`）、`plugins/web/`（预装的通用网页获取）、`justfile`（[ADR-004](decisions/ADR-004-rust-tauri.md)）。

```
usuyo/
  cmd/usuyo/            # CLI 入口（也负责启动 daemon 和 mcp 子命令）
  internal/
    core/                # 内容包读写、生命周期状态机、去重
    index/               # bleve 索引（可重建）
    jobs/                # 任务队列（fetch / normalize / ocr / asr / embed）
    adapter/             # 适配器协议 + 运行器（子进程）
    mcp/                 # MCP 服务
    api/                 # 本地 HTTP/JSON-RPC（给桌面壳、扩展、手机桥）
    normalize/           # 通用正文抽取（go-readability 等）
  pkg/contract/          # 对外契约类型（manifest、适配器输入输出），MIT
  adapters/              # 仅放参考适配器与 SDK 示例，不放平台抓取逻辑；平台适配器在社区仓库
  apps/desktop/          # MyGo 桌面壳；apps/desktop/web 为 TS 前端（M3 才启用）
  docs/
  Makefile
```

<a id="6-标签规则"></a>
<a id="22-标签规则"></a>
### 2.2 标签规则

v0，第 2 层重写前保留。Agent 改标签走 `set-tags` 提议（[§1.5.1](#151-op-词表)）；忽略过的建议记在 `tags.dismissed`。

- `platform`：原样保留，不编辑。
- `engine`：模型/规则建议，带来源与置信度；只显示给用户确认用。
- `user`：用户确认或手打；Agent 通过 propose/apply 进入这里。
- 搜索三种都索引；画布默认只显示 `user`。

<a id="7-去重"></a>
<a id="23-去重"></a>
### 2.3 去重

v0，第 2 层重写前保留。11-11 代码只做了身份层（同一身份同一条）；内容层的 SimHash、pHash 还没做，`sameAs` 现在只在合并时维护。身份层改照 [§0.4](#04-id)：同一身份就是同一个 `itemId`，web 链接也有身份；你说「不是」的记在 `notSameAs`；并发照 [§0.6](#06-文本和文件)「同一条只有一个写方」。

1. 身份层：适配器实现 `canonicalize(url) -> identity`（展开短链、去追踪参数、提取平台 ID）。同身份不新建条目，追加快照；新快照更完整则升级 `completeness`。无平台 ID 的条目不参与身份层。
2. 内容层：文本 SimHash（海明距离 ≤ 3）、图片 pHash、视频时长 + 关键帧 pHash。命中写入 `sameAs`，不自动合并。
3. 用户裁决：合并 / 保持 / 忽略。

并发：条目创建对 identity 加唯一约束（index 层）+ 任务队列按 identity 去重。

<a id="9-clim0"></a>
<a id="24-clim0"></a>
### 2.4 CLI（M0）

v0，第 2 层重写前保留。退出码和 `--json` 的错误照 [§0.3](#03-错误模型)。11-11 代码（`crates/usuyo-cli`）有 `init`、`add`、`get`、`search`、`reindex`、`daemon`，怎么找守护进程见[本机接口](#本机接口第-2-层11-11-照代码)；`tag`、`jobs`、`mcp` 还没有。

```
usuyo init [dir]                 # 建库
usuyo add <url|file|->           # 收下；立即返回条目 id，后台获取
usuyo get <id> [--layer original|derived|notes|all]
usuyo search <query> [--src douyin] [--in asr] [--kind video] [--json]
usuyo tag <id> --add a,b --rm c
usuyo jobs [--retry <id>]
usuyo reindex
usuyo daemon                     # 常驻：任务队列 + 本地 API + 文件监听
usuyo mcp                        # stdio MCP 服务（转发到 daemon，daemon 不在则内嵌）
```

<a id="10-mcp-工具m0"></a>
<a id="25-mcp-工具m0"></a>
### 2.5 MCP 工具（M0）

v0，第 2 层重写前保留，11-11 还没实现。op 照 [§1.5.1](#151-op-词表)：`add_tag`、`rm_tag` 是 `set-tags`，`set_thread` 是 `set-topics`；另加收下工具（[A3 A](PRD.md#78-agent)，[§1.6](#16-改动记录)）；私密和笔记见 [§1.3](#13-条目的四个维度)。撤销照 [§1.6](#16-改动记录)。

- `search(query, filters) -> [{id, title, hitLayer, snippet, untrusted:true, source}]`
- `get_item(id, layers=[original|derived|notes])`
- `propose_changes(changes:[{itemId, op:"add_tag"|"rm_tag"|"set_thread", value}]) -> changesetId`
- `apply_changeset(changesetId)`（需用户确认策略：M0 为 CLI 交互确认）
- `undo_changeset(changesetId)`
- 不提供：delete、export、open_url。
- 标记 `private: true` 的条目和 `notes.md` 默认不返回。

按 MCP 2026-07-28 规范实现为无状态；changesetId 为显式句柄。

<a id="11-索引"></a>
<a id="26-索引"></a>
### 2.6 索引

v0，第 2 层重写前保留。索引目录在按机器的 `libraries/<libraryId>/index/`（[§1.1](#11-目录树)）。下面一段是历史：11-11 起索引是 tantivy（`crates/usuyo-index`），分词、拼音和字段以那个 crate 为准，删了能从资料库重建（`usuyo reindex`）。

bleve（纯 Go）。中文用 CJK 双字分析器（解决二字词），同时索引拼音字段（go-pinyin）。字段：title、body、ocr、asr、vision、summary、notes、tags.*、platform、kind、author、publishedAt。向量检索 M1 再加（候选：bleve vector 或 sqlite-vec）。

<a id="附和别的文档对不上的-18-处"></a>
<a id="8-适配器协议-v0"></a>
<a id="12-m0-范围"></a>
<a id="附v0-各节和-18-处的去向"></a>
## 附：v0 各节和 18 处的去向

v0 的「1. 仓库布局」「6. 标签规则」「7. 去重」「9. CLI」「10. MCP 工具」「11. 索引」留在 [§2](#2-留着的-v0第-2-层重写前保留)；「2. 资料库目录」「3. manifest.json」「4. content.md」「5. derived/」由第 1 层取代（[§1.1](#11-目录树)、[§1.2.1](#121-manifestjson-v1)、[§1.2.3](#123-contentmd-和各栏)、[§1.2.2](#122-raw-和-derived)）；「8. 适配器协议 v0」删掉，由[插件协议 v1](plugin-protocol.md) 取代，v0 没发布过，不做兼容，清单见[协议 §2](plugin-protocol.md#2-清单)；「12. M0 范围」删掉，不再分 M0，第一版范围见 [PRD §12](PRD.md#12-开发顺序)，feature 001 重写（[总纲](BUILD.md#5-功能线)）。

v0 附录记的 18 处对不上：

| # | 哪处 | 归哪 | 结论 |
|---|---|---|---|
| 1 | 本机接口 | 第 2 层 | 11-11 照代码先写了一节：[本机接口](#本机接口第-2-层11-11-照代码) |
| 2 | 知识层目录 | 这里了结 | `topics/`、`cards/`、`drafts/`；`set_thread` 换成 `set-topics` |
| 3 | 画布几个 | 这里一半 | 留 `canvas/`，个数等 [M7](PRD.md#76-地图) |
| 4 | 任务状态 | 这里了结 | 搬进本机 `jobs.db`，资料库只看 `derived/` |
| 5 | 注意力 | 这里了结 | `attention.json` 去掉，只记「打开过」 |
| 6 | 存在和私密 | 这里了结 | `state` 三个值，加 `private` |
| 7 | 通用网页获取 | 插件协议 | 预装照[协议 §8.4](plugin-protocol.md#84-三种来源)；没装时宿主自己算 `web:` 身份（[§0.4](#04-id)） |
| 8 | MCP 不可信标记 | 第 2 层 | — （11-11 MCP 还没做） |
| 9 | 配置目录 | 这里了结 | 按机器目录；库里只留库级的 `settings.json` |
| 10 | 本机接口的令牌 | 这里了结 | 11-11 代码：按机器目录的 `api-token` 存令牌本身，每次启动换，见 [§1.1](#11-目录树) 和[本机接口](#本机接口第-2-层11-11-照代码) |
| 11 | `.meta.json` | 这里了结 | [§1.2.2](#122-raw-和-derived)；[协议 §4.2](plugin-protocol.md#42-转换的-run) 已跟着改 |
| 12 | 快照字段 | 这里了结 | [§1.2.1](#121-manifestjson-v1)；`additionalProperties` 见 [§0.2](#02-版本与兼容) |
| 13 | `missing` | 这里了结 | 用 `outputs` 词表 |
| 14 | 导出身份键和导出记录 | 这里了结 | `usuyo_id` 一致；导出记录搬到按机器目录，[协议 §4.4](plugin-protocol.md#44-导出的-run-与-readback) 已跟着改 |
| 15 | 插件来源怎么显示 | 插件协议 | [协议 §8.4](plugin-protocol.md#84-三种来源) |
| 16 | GitHub App | 插件协议 | [O3](BUILD.md#7-只有用户能答的和能验的) 已定 A；部署写进 ADR-003 和协议，见 [总纲](BUILD.md#012-插件的安装和分发) |
| 17 | 改动提议多久作废 | 这里了结 | 不作废（[§1.10 第 6 条](#110-要你看一眼)） |
| 18 | 插件清单是 v0 还是 v1 | 第 3 层 | `crates/usuyo-contract`（原 `pkg/contract`）只留资料库类型；插件协议的类型 11-11 在 `crates/usuyo-plugin`，`crates/usuyo-plugin-kit` 转出给插件用，schema 文件还没有 |
