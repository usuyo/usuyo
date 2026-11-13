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
| JSON、YAML 字段、front matter 键 | camelCase（[协议 §2.1](../protocol/manifest.md#21-文件名与位置)），MCP、前端同；不加前缀。导出文件里的属性键归导出插件（[协议 §4.4](../protocol/transform-export.md#44-导出的-run-与-readback)） |
| 枚举值、错误码、命令行选项 | 小写 kebab-case：`login-required`、`media:video`、`--dry-run` |
| 别的工具加的 JSON 字段 | `x-` 开头，我们不用 |

<a id="02-版本与兼容"></a>
### 0.2 版本与兼容

semver：patch 改错；minor 加可选字段、枚举值、错误码、接口；major 是删、改名、改类型或含义、可选变必填。线上只带主版本整数。

| 对象 | 线上带的 | 读到更新的主版本 |
|---|---|---|
| 插件协议 | `protocol: 1`；文档头写 `1.0.0` | 不加载，写原因 |
| 内核服务 | 路径 `/v1/`；MCP 工具、命令行 `--json` 跟它走。11-11 代码：只有投递接口在 `/v1/` 下（`/v1/ping` 回 `protocol: 1` 和版本），给窗口用的路径还不带 `/v1/`，版本由 `GET /health` 报（[本机接口](local-api.md#本机接口第-2-层11-11-照代码)） | `/v1/ping` 报版本 |
| JSON Schema | `$id` 为 `urn:usuyo:schema:<名字>:<主版本>`（没有域名也能用） | 校验失败 |
| 资料库 | `.usuyo/library.json` 的 `formatVersion`、`extensions` | 只读，不给 Agent、不导出，提示升级 |

枚举开放，不认识的值按各处的退路办。旧程序不能忽略的新东西（改变能不能给 Agent、能不能删）写进 `extensions`，不认识就只读，照 git 的 `extensions.*`。

**迁移**只由宿主做，先问你，之前只读：

- 要改写的文件先复制到按机器的 `backups/`，留 30 天；不碰 `raw/` 和媒体，不改你写的正文；front matter 和 JSON 一样，只改迁移认识的键，不认识的原样留着（同下面的 JSON 演进）。
- 计划写在 `.usuyo/tmp/migrate-<从>-<到>.json`，逐步打勾，断了从没打勾的那步重跑。
- 开始时 `extensions` 写 `migrating`，没升级的旧程序（库在 iCloud 里的另一台）见了就只读；最后写 `formatVersion`，再去掉 `migrating`。
- 结果确定：新 id 由旧 id 算（推断：UUIDv8 或哈希），不读时钟，两台各迁各的一样。同步的记录带 `formatVersion`：远端新就不同步，拉到旧格式的先迁再合并。

**JSON 演进**：读到不认识的字段不报错，资料库文件原样写回，插件信封照[协议 §0.1](../protocol/README.md#01-读者与状态) 忽略并记日志。收到的命令（HTTP 请求体、MCP 参数）有不认识的字段回 `invalid-args`，新功能在旧内核上看得见地失败。schema 不写 `additionalProperties: false`（加字段会变成 major）：运行时容忍，`usuyo plugin check` 和测试拒绝，怎么实现归第 3 层。

**弃用**：只在下个 major 删，之前至少两个 minor、6 个月；标 `deprecated: true`，HTTP 带 `Deprecation`（RFC 9745）、`Sunset`（RFC 8594）头。插件协议的旧主版本再加载 12 个月。

<a id="03-错误模型"></a>
### 0.3 错误模型

错误对象就是[协议 §3.3](../protocol/fetch.md#33-信封与错误码) 的 `error`，宿主另填 `retryable`（不改什么、过会儿再发可能成功）和 `plugin`（哪个插件报的）。`needs` 加前缀 `version:`，例 `version:usuyo>=1.3`。

| 面 | 写法 |
|---|---|
| 插件 | 照协议的信封；JSON-RPC 插件用 `-32000`，`data` 是错误对象 |
| HTTP | RFC 9457：加 `type`（`urn:usuyo:problem:<code>`，客户端认 `code`）、`title`（固定英文）、`status`、`detail`（插件报的错是它的 `hint`，没有就 `message`；宿主自己报的是英文的 `message`，给日志）、`hint`（给人看的一句中文，界面照它显示；插件给了 `hint` 也放这里，宿主自己报的错只在界面要显示的那几处有：列来源、导入、从文件导入、交内容）、`instance`（`urn:uuid:<callId>`）；带 `Retry-After` |
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

**条目 id 由身份算**（[调研 14 §2](../research/14-sync-like-hatoba.md#2-本产品要同步什么)），两台离线收下同一条得到同一个 id：

```
identityKey = <platform> ":" <contentId>        有 contentId
            = "web:" <规范化的 identity.url>     platform 是 web
itemId      = <platform> "-" base32(sha256(identityKey) 前 16 字节)
```

- base32 用 RFC 4648 字母表，小写，不补 `=`，26 位；128 位不用写碰撞分支。例：`douyin:7431234567890123456` → `douyin-4plrcxk343onmyxibmrr672kga`。
- `contentId` 照 plugin-kit 的平台列表（抖音十进制，B站 `BV…:p2`），JSON 里是字符串（19 位超过 2^53）。11-11 代码：平台列表还没有，宿主只查 `contentId` 非空、不超过 256 字节、没有控制字符；`platform` 要合 `[a-z0-9][a-z0-9-]*`。`identityKey` 按第一个 `:` 拆，`itemId` 按最后一个 `-` 拆（平台可以是 `wechat-mp`）。
- **web 身份**：插件规范化后，宿主再按 [RFC 3986 §6.2.2](https://www.rfc-editor.org/rfc/rfc3986#section-6.2.2) 做一遍：scheme、主机小写，去默认端口和 `#` 片段，空路径写成 `/`（`https://a.com` 和 `https://a.com/` 是同一条，`https://a.com?q=1` 写成 `https://a.com/?q=1`），路径和查询串不动（不改大小写、不解百分号编码、不去 `.`、`..` 段）。11-11 代码：只认 `http`、`https`；带用户名或密码的链接收下时就拒绝（`unsupported-input`），不进身份、标题和 `content.md`；收下的输入里取第一个 `http(s)://` 链接，到空白或非 ASCII 标点为止。没有插件命中的 http(s) 链接，和命中了插件、`canonicalize` 却因 `network`、`timeout` 失败的短链（下一条），宿主自己这样算，没装 web 插件也一样。
- **短链要联网展开**：`canonicalize` 因 `network` 或 `timeout` 失败时，宿主先拿短链本身照上一条算 `web:` 身份，马上建文件夹（`completeness: link`，之后的快照照记 `inputUrl`），只把重试放本机 `jobs.db`。联网后拿到平台身份，照下面「拿到更好的身份时」并过去，`web-` id 进 `aliases`；平台身份的 id 还没有条目时，这一步就是改名。并过去时撞上回收站或私密的，见下一条。两台离线收下同一条短链得到同一个 id；同一视频的两条不同短链先是两条，联网后并成一条。11-11 还没做：联网后拿到平台身份不并过去，fetch 交回的平台身份也不进 `identities`。
- **展开后撞上回收站或私密**（推断）：`jobs.db` 里的重试记下当初是哪个入口。主动收的照 [§1.7](library.md#17-删掉)：在回收站里的拿回来，`restore` 和这次并过去（或改名）记在同一批，撤销那批时照 [§1.7](library.md#17-删掉) 原样放回回收站。目标私密、当初是 Agent、`api` 令牌或终端外的命令行收的：`web-` 条目并进它，`origins` 不并过去（同直接收下时跳过），撤销那批不动这一条（`web-` id 成了别名，私密的对 Agent 照 [§1.3](library.md#13-条目的四个维度) 回 `not-found`）。跟着收遇到 `canonicalize` 因 `network`、`timeout` 失败的不建短链条目，这一次不算跑完，联网后整次重跑。
- **没有身份的**（文件、聊天记录；fetch 输出省略 `identity` 或写 `{"platform": "file"}`，[协议 §3.7](../protocol/fetch.md#37-fetch-输出)）：平台记 `file`，`file-` 加 26 位随机 base32（同样形状），放 `items/file/`，只靠内容层去重。`{platform: file}` 不算身份，不进 `identityKey` 和自动合并。不带 `contentId` 的身份只有 `web`，必带 `url`；`web`、`file` 以外的平台缺 `contentId` 照[协议 §3.1](../protocol/fetch.md#31-运行器与节奏) 按 `invalid-output` 处理（feed 条目等[协议 §13 第 22 项](../protocol/open.md#13-待定)）。
- **id 不改**，只能经合并换，被并掉的进 `aliases`。宿主自动合并两处（推断）：拿到更好的身份时（平台 > web > 随机），并进那个身份算出的 id；同步时两个 `itemId` 对上同一个 `identityKey`，并进由身份算出的那个。同一个 id 对上不同的 `identityKey` 不合并，报告。
- **算法冻住**：平台列表里 `contentId` 格式的改动（`BV…` 和 `BV…:p1` 算不算一个）算资料库的 major。web 身份尽力而为，对不上靠 `sameAs`；`inputUrl`、`resolvedUrl` 也进索引，查库里和回收站里的条目时再按它们查（推断）。
- **文件夹名就是 `itemId`**，引用只用 id，不用路径。
- **本机不带钥匙**：两台要在有钥匙之前就算出同一个名字。云端记录 id 是 `HMAC-SHA256(idKey, "item:" + itemId)`，服务器试不出你存着哪个公开视频（[调研 14 §4](../research/14-sync-like-hatoba.md#4-加密)）。

**别的 id 用 UUIDv7**（RFC 9562）：按字符串排就是按时间排，Hatoba 也用；ULID 也是 26 位 base32，会和条目 id 混。用于资料库、快照、提议、改动记录、主题、卡片、成稿、导入批次、跟着收、任务、令牌、`origins[]` 每一项，还有 `deviceId`、`trashId`、改动记录的 `batch`、提议里 op 和 hunk 的 `id`、`callId`。块 id（划线、主题页段落）是 5 位随机小写 base32，文件内唯一，写在段末 ` ^<id>`（导出到 Obsidian 时正好是它的块 id 写法，不用转换）；脚注标签同样是 5 位小写 base32。`deviceId` 每台第一次打开这个库时生成。

**先后不靠时钟**：同一台按本台序号 `n`，跨设备按同步的 `seq`。要比时间的地方（主快照一样大时取新的）先比 `at`、再比 id 字符串，两台算出同一个结果。usuyo 写的时间照[调研 14 §5](../research/14-sync-like-hatoba.md#5-改动和冲突) 钳制：本机时间和见过的最大远端时间 +1 ms 取大，有上限；和服务器差超过 5 分钟就提示、不推。

**引用**是普通链接，例 `[口播 01:42](usuyo://item/douyin-4plrcxk343onmyxibmrr672kga/asr@<snapshotId>#t=102.5)`。

- **`@<snapshotId>`**：秒数只对某一份媒体成立。时间类的层（`asr`、`video`、`audio`）必带，是那份媒体所在的快照；`text` 带引用时的主快照；`comments` 必带所在的快照；`vision` 带它的媒体所在的快照；`ocr/<imageKey>`、`image/<imageKey>`、`summary` 和 `notes` 不带。
- **片段**：时间 `#t=102.5`、`#t=102.5,110`（Media Fragments 1.0）；图片区域 `#xywh=percent:10,20,30,40`；画面上某一秒的一块（关键帧上的字）把时间和空间用 `&` 合写，例 `usuyo://item/<id>/video@<s>#t=12.04&xywh=percent:7.4,25.4,55,5.4`（系统的播放器不认空间部分，由 usuyo 自己解析）；原文 `#:~:text=[前缀-,]原话[,-后缀]`（URL Fragment Text Directives），太长写 `开头,结尾`；段落 `#^<块 id>`。
- **编码**：只编码原话部分里 ASCII 字母数字以外的字符，连同 `-`、`,`、`&`；语法字符和 `t=`、`xywh=` 后的数字照原样。例：「40 克」后面的「闷蒸 30 秒」写 `#:~:text=40%20%E5%85%8B-,%E9%97%B7%E8%92%B8%2030%20%E7%A7%92`。
- `asr@<s>#t=42` 和 `video@<s>#t=42` 指同一处：层名只决定打开哪一栏，去重、找回按条目、快照、时间比。

<a id="05-时间"></a>
### 0.5 时间

- RFC 3339 带偏移，不写纪元秒数。usuyo 写的用 UTC、固定 3 位毫秒，`2026-11-05T03:00:01.123Z`，能直接排序。
- 平台给的时间存 UTC，偏移不是 `Z` 时另存原样：`"actedAt": "2026-10-31T14:14:00.000Z", "actedAtOriginal": "2026-10-31T22:14:00+08:00"`；拿不到就不写。
- **front matter 里的时间**和 JSON 一样（第一条）：UTC、3 位毫秒，写成双引号字符串，例 `createdAt: "2026-11-05T03:00:01.123Z"`；读照 RFC 3339，别的写法算这个文件写坏了（[§1.4](library.md#14-你写的那一层)）。front matter 里只有 `createdAt` 一个时间。
- **写进 Markdown 给人看的日期**（引用脚注的发布日期、`content.md` 头部）取 `publishedAtOriginal` 的日期部分，没有就取 UTC 日期，不按本机时区：两台写出一样的字节。
- 时长：JSON 里是秒、字段名以 `Sec` 结尾，YAML 写 `8s`、`10m`。按天的设置用整数天、名字以 `Days` 结尾（如 `trashDays`），「一直」写 `null`；错误对象的 `retryAfter` 是秒，照 HTTP `Retry-After` 不带 `Sec`，是例外。大小：JSON 里是字节，YAML 写 `512MiB`。媒体位置是秒数，最多 3 位小数；显示向下取整，`102.5` 写 `01:42`，过一小时写 `1:01:42`。

<a id="06-文本和文件"></a>
### 0.6 文本和文件

- UTF-8 不带 BOM，LF，以换行结尾；BOM、CRLF 照读，改写时保持原样。JSON 缩进两格、键顺序固定；JSONL 一行一个对象。
- **front matter** 只有 usuyo 写，用 YAML 的一个子集，同一个对象每台写出一样的字节：键照各处列的顺序，不认识的键排在认识的后面，按键名排；字符串一律双引号（时间、`no`、`yes` 也还是字符串）；布尔不加引号；列表一行一项、按字符串排；`false` 和空列表不写，没有这个键就是空的或 `false`；不用注释、锚点和行内写法。只有你写的 `.md` 带它：主题、卡片、成稿和 `notes.md`（[§1.4](library.md#14-你写的那一层)）；`content.md`、`highlights.md` 不带。
- id、文件名、front matter 键、我们生成的字用 NFC；平台原文照收到的存（NFC 会换掉 CJK 兼容汉字）。
- **原子写**：同目录写 `.<名字>.usuyo-tmp-<随机>`，fsync，改名盖过去；Windows 上目标被占用就短暂重试，不行回 `library-busy`。写回前比一次哈希：和本机上次写的、读到的不一样，说明 usuyo 外有写方（库在云盘里时的另一台，或手改），先照 [§1.4](library.md#14-你写的那一层)「usuyo 外改了资料库里的文件」读进来、合并，再写。
- **同一条只有一个写方**：本机按 `itemId` 加锁，任务按 `itemId` 去重。新条目在 `.usuyo/tmp/` 拼好再改名进去；目标已经在（同一链接贴了两次、跟着收和你同时收下、同步拉下同一条），就把新快照移进去，合并 manifest。
- **锁**：一台机器上一个库一个写方，`.usuyo/lock` 加系统建议锁。它管不住两台经云盘同时开，所以内置同步和云盘二选一。
- **大小写不敏感**（macOS、Windows）：我们起的名字全小写；标题来的名字按 NFC 加大小写折叠查重（排除文件自己），撞了加「 2」；只改大小写的改名分两步，经过临时名。
- **不能用的字符**：系统不许的 `< > : " / \ | ? *` 换成全角 `＜ ＞ ： ＂ ／ ＼ ｜ ？ ＊`；链接只用 id（[§0.4](#04-id)），不用文件名，所以 `# ^ [ ]` 照留；控制字符删掉；去掉开头的 `.` 和结尾的空格、`.`；`CON`、`NUL`、`COM1` 这类保留名前加 `_`，带扩展名也算。
- **长度**：每段不超过 200 字节，标题按字形截断，截时留出扩展名和本机后缀的位置（约 64 个汉字）。从库根起的相对路径不超过 200 个 UTF-16 单位，条目文件夹以内不超过 140；回收站里只放对象以内的相对路径，主题、卡片、成稿在回收站里存成 `body.md`，原路径记在 `entry.json` 的 `path`。库根在 55 个字符内就不过 Windows 的 260，超了提示一句。
- **平台来的字**（标题、作者、原话、正文）是不可信输入，按写进的位置转义：YAML 一律双引号标量，`\`、`"`、控制字符转义，换行写 `\n`；Markdown 链接文字和原话转义 `` [ ] ( ) < > * _ ` \ # ``，换行合成空格；`textFormat: plain` 的正文写进 `content.md` 时反斜杠转义 `` \ ` * _ [ ] < > # ! | ~ `` 和行首的 `-`、`+`、`=`、`数字.`，`&` 写 `&amp;`，渲染出来仍是纯文字。

<a id="07-哈希"></a>
### 0.7 哈希

SHA-256，小写十六进制，算存下来的字节。字段名叫 `sha256` 时写裸值（协议的 `install`、`models`，[§1.2.2](library.md#122-raw-和-derived) 的 `inputs`、`files`）；通用的哈希字段写 `sha256:<hex>`（导出记录，[§1.5.2](library.md#152-提议文件) 的 `base`，[§1.6](library.md#16-改动记录) 的 `before`、`after`），以后换算法也装得下。离开本机的文件 id 是 `HMAC-SHA256(idKey, "blob:" + 内容)`，和记录 id 共用 id 钥匙、靠前缀分开（[调研 14 §4](../research/14-sync-like-hatoba.md#4-加密) 只分记录、文件、id 三把）；切块见[调研 14 §3](../research/14-sync-like-hatoba.md#3-存在哪和花多少)。你内容的普通哈希不出本机。

<a id="08-分页和游标"></a>
### 0.8 分页和游标

- 请求 `limit`（默认 50，最多 500）、`cursor`；响应 `items`、`nextCursor`（最后一页 `null`），和[协议 §3.8](../protocol/fetch.md#38-list-与跟着收)、MCP 同名。
- 11-11 还没做：本机接口回的是普通数组，没有 `cursor`；搜索的 `limit` 默认 50，限在 1 到 200（[本机接口](local-api.md#本机接口第-2-层11-11-照代码)）。
- 游标是不透明的 base64url，和查询绑定；查询变了或索引重建过回 `cursor-invalid`。按键分页，最后按 id 定序，新收下的不会让条目跳过或重复。有效期归第 2 层。
- 响应不带总数（按键分页算总数要另扫一遍）；另给哪些计数（搜到多少条、磁盘占用），第 2 层按视图定。永远不出现的数：几条没看过、几条没整理、几个提议没决定（[原则 4](../PRD.md#4-原则)）。插件 `sources` 的 `count` 是平台页面上的数（[插件协议 L5](../protocol/fetch.md#31-运行器与节奏)）。

<a id="09-要你看一眼"></a>
### 0.9 要你看一眼

| # | 什么 | 建议 |
|---|---|---|
| 1 | [§0.4](#04-id) 条目文件夹名像 `douyin-4plrcxk343onmyxibmrr672kga`，看不出标题，不按年月分 | **已定（11-04）**：定同步时照调研 14 定了目录名由身份算（[回答表第 14 条](../answers.md#14-资料库能不能放进同步盘)，[回答表](../answers.md)）：两台离线收下同一条要落到同一个文件夹 |
| 2 | [§0.1](#01-命名) 属性名怎么起 | **不再问（11-05 方向）**：front matter 键和 JSON 一样用 camelCase，不加前缀；Web Clipper 键只在导出文件里，见 [§0.1](#01-命名)；[回答表](../answers.md) |
| 3 | [§0.5](#05-时间) 属性里的时间怎么写 | **不再问（11-05 方向）**：时间一律 RFC 3339 UTC 毫秒；只写日期是导出的事，见 [§0.5](#05-时间)；[回答表](../answers.md) |
| 4 | [§0.6](#06-文本和文件) 文件名里不能用的字符和长度 | **不再问（11-05 方向）**：只把系统不许的 9 个字符换成全角，`# ^ [ ]` 照留，截到约 64 个汉字，见 [§0.6](#06-文本和文件)；[回答表](../answers.md) |

<a id="010-界面文案和多语言"></a>
### 0.10 界面文案和多语言

用户 2026-11-13 定：界面上写成熟软件里那种客观的话，不写「收下」「存着」「变成文字」这一类自己造的说法；参照 Apple 中文界面的写法。界面要能换语言。

**怎么写**

- 写事实和动作，不写感受，不拟人，不和用户聊天。按钮是动词或动宾短语：「添加」「导入」「删除」「重试」「下载」。标题和栏目名是名词：「设置」「导入」「废纸篓」。
- 用通行的词，同一样东西全应用只有一个叫法（下表）。平台自己的叫法照平台的：B站的「收藏夹」、X 的「书签」。
- 状态用短的陈述：「已添加」「正在下载」「无法连接」「需要登录」。出错先说发生了什么，再说可以怎么做；不道歉，不用感叹号。
- 称呼用「你」，能省就省。不写「我们」。
- 句子写完整才加句号；按钮、标签、表头、选项不加标点。数字和单位之间留一个空格（`163 MB`）；中文和西文、数字之间留一个空格。引用界面上的字用「」。
- 不催人：不写未读数、进度、还剩多少（项目原则第 4 条「不制造压力」）。风险只陈述记录到的事实和出处。

**用词**

| 说 | 不说 |
|---|---|
| 添加（动作）、已保存（状态） | 收下、存着 |
| 稍后阅读 | 回头看 |
| 转写（口播成文字）、文字识别（图里和画面上的字）；合起来的栏目叫「转写与识别」 | 变成文字 |
| 获取方式 | 取法、方法 |
| 导入已保存的内容 | 拿回存过的 |
| 获取访客身份 | 领访客身份 |
| 可用、不可用、尚未提供 | 现在能用、用不上、还没做 |
| 暂停、立即重试 | 先别试、现在再试 |
| 发送到 usuyo | 交给 usuyo |
| 废纸篓（macOS）、回收站（Windows） | — |

**多语言**

- 界面上的每一句话都从语言文件里取，源码里不写死；键按页面和用途起名，不拿中文原句当键。先有简体中文（`zh-Hans`）和英文（`en`），默认跟系统，设置里能换。
- 带数量和变量的句子用带占位的整句，不拼接；日期、数字、文件大小按语言格式化。
- 内核和插件给界面的话（`hint`）同样按语言给：宿主把语言经 `USUYO_LOCALE` 交给插件；错误码、层级这类给机器看的值不翻译，由界面按码取话。
- 英文同样照 Apple 的写法：句首大写，按钮用动词，不用 please。

