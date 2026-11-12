<a id="4-转换与导出插件协议"></a>
## 4. 转换与导出插件协议

<a id="41-分帧与生命周期"></a>
### 4.1 分帧与生命周期

- stdio 上换行分隔的 JSON-RPC 2.0：一行一条消息，消息里不能有换行；stdout 只放协议消息，日志写 stderr；分帧和 MCP stdio 相同（[03 无状态模型与 stdio](../research/03-agent-protocols.md#无状态模型与-stdio)）。
- 宿主用 `run.command` 启动进程，先调 `describe`，核对 `protocol` 和 `version`，对不上不加载；可以常驻，模型加载一次处理多条。`describe` 的回答在进程活着的时候不变，宿主只在起进程时问一次；它不列能产出哪几种，种类照清单在每次请求和每个结果上核。宿主同一时间向一个进程只发一个 `run`；`gpu: true` 的插件同一时间只开一个进程，其余的按「同时处理几个」的设置并行开进程。
- [§3.2](fetch.md#32-调用约定) 的环境变量表同样适用于转换、导出插件，只有 `USUYO_SETTINGS`、`USUYO_SECRETS`、`USUYO_INPUT`、`USUYO_CREDENTIALS`、`USUYO_COOKIES_TXT`、`USUYO_VISITOR`、`USUYO_BROWSER_PROFILE` 不给：这两类插件的非密钥设置只经 `run` 的 `options` 传，密钥只经宿主代发。常驻的转换进程拿到 `USUYO_PLUGIN_DIR`（安装目录）和 `USUYO_MODELS_DIR`（`${MODELS}`），没有 `USUYO_CALL_ID`：一个进程接好几次 `run`，哪一次看请求的 `id`。标准流按本节。进程能看到哪些路径见 [§7.1](isolation.md#71-常驻进程的路径)。
- 退出：宿主关闭 stdin，插件 5 秒内退出；否则 SIGTERM，再 5 秒后杀进程组。
- 转换进程闲了（120 秒没有 `run`）宿主可以关掉，放掉模型占的内存，下次要用再起。
- `progress` 只用来判断卡住（[§3.4](fetch.md#34-超时与取消) 的无进度时长），不往界面报：界面上没有百分比。卡住的时钟从这次 `run` 的第一个 `progress` 起算，从没发过的只受总时长约束。
- stdout 上每一行都要是回答或通知，最长 1 MiB。不是 JSON 对象、`jsonrpc` 不是 `"2.0"`、插件发来请求、回答里 `result` 和 `error` 不是恰好一个、回答的 `id` 不是正在等的那个，或者一行太长，宿主记 `invalid-output`，关掉整个进程组，下次要用再起。认不出的通知忽略。两次调用之间写到 stdout 的，宿主在下一次调用前换一个进程。
- 错误：协议错误用 JSON-RPC 标准码（`-32700`、`-32600`、`-32601`、`-32602`）；插件自己的失败用 `-32000`，`data` 是错误对象，`code` 取 [§3.3](fetch.md#33-信封与错误码) 里适用的值（多是 `unsupported-input`、`dependency-missing`、`network`、`cancelled`、`internal`）。

| 方向 | 方法 | 用途 |
|---|---|---|
| 宿主 → 插件 | `describe` | 返回 `{protocol, version, models?}`，`models` 是实际可用的模型或语言 |
| 宿主 → 插件 | `health` | 同 [§3.10](fetch.md#310-health) 的 `ready`、`checks`，例如模型在不在、GPU 能不能用、后端服务通不通 |
| 宿主 → 插件 | `run` | 处理一条（转换）或一批（导出） |
| 宿主 → 插件 | `readback` | 只有导出插件，`readback: true` 时 |
| 宿主 → 插件（通知） | `cancel` | `{id}`；插件仍要回这个请求，错误码 `cancelled`（LSP 的规则） |
| 插件 → 宿主（通知） | `progress` | `{id, done, total, unit, message}`，只用来判断卡住 |

<a id="42-转换的-run"></a>
### 4.2 转换的 run

```json
{"jsonrpc":"2.0","id":7,"method":"run","params":{"item":{"id":"bilibili-4plrcxk343onmyxibmrr672kga","kind":"video","title":"手冲咖啡：三段式注水","lang":"zh"},"want":["asr"],"inputs":[{"kind":"audio:pcm16k","mime":"audio/wav","path":"/private/var/folders/xx/usuyo-proc-03/in/7/0/audio.wav","snapshot":"019a6f2b-4a10-7c3e-8b21-0d9e8f7a6b5c"}],"out":"/private/var/folders/xx/usuyo-proc-03/out/7","options":{"vadThreshold":0.4},"provider":null}}
```

```json
{"jsonrpc":"2.0","id":7,"result":{"outputs":[{"kind":"asr","path":"asr.vtt","lang":"zh"}],"tool":{"name":"sherpa-onnx","version":"2.0.1"},"options":{"language":"auto","itn":true,"vad":"silero-vad-v5","vadThreshold":0.4,"maxCueSec":15},"model":{"name":"sense-voice-int8-2024-07-17"},"marks":[],"empty":false}}
```

```json
{"jsonrpc":"2.0","id":8,"result":{"outputs":[],"tool":{"name":"sherpa-onnx","version":"2.0.1"},"options":{"vadThreshold":0.5},"marks":[{"code":"no-speech","speechSec":0.4,"durationSec":5.0}],"empty":true,"reason":"no-speech"}}
```

- `want` 是这次要哪几种产出，取清单 `produces` 的子集、不为空、不重复；一个插件能做几种时（抽音轨的插件做 `play`、`audio:pcm16k`、`probe`、`keyframes`）靠它分。宿主只核这几条，一次可以要几种；预装的 `media`、`asr-sherpa`、`ocr-vision` 一次只收一种，`want` 多于一项回 `invalid-args`。
- `inputs` 是条目文件放进进程暂存目录后的绝对路径，只读（`${INPUT}`，怎么放进去见 [§7.1](isolation.md#71-常驻进程的路径)）：`in/<callId>/<下标>/<原文件名>`，下标是它在 `inputs` 里的位置，原文件名留着扩展名；和 `out` 指的 `out/<callId>/` 在同一个暂存目录下。大块数据不走管道。`consumes` 写了 `mime` 的，宿主只给符合格式的文件，必要时先让抽音轨一类插件转好。每个输入另有可选的 `snapshot`（所在快照）、`imageKey`（图文的图和封面）、`frameSec`（关键帧在第几秒）、`role`（`play` 时哪个是 `video`、哪个是 `audio`）。
- `options` 是这个插件的非密钥设置。
- 插件只写 `out`。结果里：
  - `outputs[]` 每项 `{kind, path, lang?}`：`kind` 取 `want` 里会出文件的种类，每种一项；`path` 相对 `out`；`lang` 要像语言标签（字母和数字，用 `-` 分段，每段最多 8 个，整个最多 35 个）。`keyframes` 的 `path` 是 `keyframes.json`，帧图片在它旁边的 `keyframes/` 里。文件格式见 [SPEC §1.2.2](../spec/library.md#122-raw-和-derived)。
  - `tool` 必填：插件里干活的程序和版本 `{name, version}`，两项都不能空，例如 `ffmpeg 9.0.2`、`sherpa-onnx 2.0.1`；宿主不猜。
  - `options`：这次实际用的参数，含插件补上的默认值；宿主原样记下，重做时原样交回。
  - `model`：用了模型时 `{name}`，`name` 要是清单 `models[].name` 之一；sha256 由宿主自己核。
  - `marks[]`：很明显的标记 `{code, speechSec?, durationSec?, frozenSec?}`，`code` 是 `no-audio-track`、`no-speech`、`static-picture`、`no-text`（feature 003 [§6](../features/003-transforms.md#6-很明显的标记)）；秒数有限、不小于 0。
  - `probe[]`：`want` 有 `probe` 时，每个输入一项 `{input, durationSec?, streams: [{kind, codec?}]}`，`input` 是 `inputs` 的下标，`kind` 是 `video`、`audio`、`subtitle`、`other`；每个流只报种类和编码。宿主用它配对画面和音轨、判出没有声音轨。
- `out` 照 [§3.6](fetch.md#36-fetch-与输出目录) 的输出目录规则检查；宿主核对后连同自己算的哈希写成一次转录 `derived/runs/<runId>/`，替插件写 `run.json`（[SPEC §1.2.2](../spec/library.md#122-raw-和-derived)），不再写 `<文件>.meta.json`；插件不用自己写。
- `empty: true` 表示这条没有可产出的东西，这时 `outputs` 为空、`reason` 必填：`no-speech`（没听到人说话）、`no-audio-track`（没有声音轨）、`no-text`（没认出字）；`empty: false` 时不带 `reason`。宿主不再往下游编排摘要，免得模型对着空文稿编造（AI-Video-Transcriber 的 `no_speech` 做法）。不合这几条的结果、`kind` 不在 `want` 里或重复的，宿主按 `invalid-output`。
- 缺模型回 `dependency-missing`，`needs: ["model:<名字>"]`（[§3.3](fetch.md#33-信封与错误码)）。
- 这些消息的类型在 `crates/usuyo-plugin` 的 `protocol::transform`，JSON Schema 是 `crates/usuyo-plugin/schema/transform-*.schema.json`；Rust 插件的循环在 plugin-kit 的 `transform` 模块（[§10](plugin-kit.md#10-契约测试与-plugin-kit)）；宿主一侧是 `crates/usuyo-plugin` 的 `host::transform`。

<a id="43-模型与密钥由宿主代发"></a>
### 4.3 模型与密钥由宿主代发

- 模型 provider 是宿主的一份配置（名称、OpenAI 兼容端点、模型、密钥），密钥在系统钥匙串里（[ADR-003 §1](../decisions/ADR-003-plugins.md#1-插件分三类)）。
- `provider` 不是 `none` 的插件，`run` 参数里会有：

```json
{"provider": {"baseUrl": "http://127.0.0.1:47822/v1", "token": "q8Zr2VbN4kTxW1mYc7HdLs0JpEa5Gu9F", "model": "summary"}}
```

- 插件按 OpenAI 兼容接口请求 `baseUrl`（`/chat/completions`、`/audio/transcriptions`），带 `Authorization: Bearer <token>`。这个 token 是至少 128 位的随机值，只对这一次调用、这个插件、宿主的本机端点有效，调用结束即作废；宿主比对时用常数时间比较，因为本机任何进程都能连这个端口；宿主换上真正的密钥转发给用户配的 provider，记下这次请求是否离开本机。宿主自动把这个端点加进插件的网络许可。
- **获取插件也可以用。** 能力声明了 `provider`（[§2.3](manifest.md#23-获取插件的字段)）、用户配了模型时，一次一进程的调用经 `USUYO_PROVIDER` 拿到同样的 `{baseUrl, token, model}`，常驻进程在请求的 `context.provider` 里拿到；token 的规则不变：一次调用、一个插件、宿主的本机端点。视觉层插件用看图模型找控件时走这里，界面照样标「离开本机」（[§9.5](wrapping.md#95-视觉层流程)）。
- 包装自带模型的第三方 Agent（Mika 的 MCP 或 HTTP 接口一类）不在这一条之内：它的模型调用在手机上、由它自己发，宿主标不了「离开本机」，节奏也管不到。这类插件 `usuyo-fetch` 收不收是待定项（[§13](open.md#13-待定) 第 20 项）。
- 带 `inject` 的密钥设置（导出到 Notion 这类云端目标）用同一个办法：宿主在 `run` 参数的 `endpoints` 里给一个本机地址，例如 `{"notion": "http://127.0.0.1:47823/"}`，转发时按 `inject` 填请求头。插件始终拿不到密钥明文。

自带模型、每次走第三方模型的手机 Agent 插件 `usuyo-fetch` 不收：宿主的节奏和「离开本机」的标注都管不到。你自己装的第三方插件照常能用；手机上导出的清单走导入插件读。

<a id="44-导出的-run-与-readback"></a>
### 4.4 导出的 run 与 readback

资料库的形状按 usuyo 自己的需要定（[SPEC 第 1 层](../spec/library.md#1-资料库-v1第-1-层)）；导出插件把宿主给的副本转成目标软件的样子。读回是 `run` 反过来做：把目标里的文件按 `usuyo_id`（主题、卡片、成稿按它们的 id）转回 `run` 收到的用户层，交回值，不是改动；宿主拿它和底本、库里现在的那份照 [SPEC §1.4](../spec/library.md#14-你写的那一层)「读进来」、[§1.8](../spec/library.md#18-同步) 三方合并，同步、在 usuyo 外改文件、导出读回用的是同一个合并。插件不读资料库文件，宿主不解析目标的语法。

```json
{"jsonrpc":"2.0","id":2,"method":"run","params":{"items":[{"id":"douyin-4plrcxk343onmyxibmrr672kga","manifest":"/…/in/2/douyin-4plrcxk343onmyxibmrr672kga/manifest.json","content":"/…/content.md","notes":"/…/notes.md","derived":{"derived/summary.md":"/…/derived/summary.md","derived/asr.vtt":"/…/derived/asr.vtt","derived/keyframes/":["/…/derived/keyframes/0012.jpg","/…/derived/keyframes/0047.jpg"]},"files":[{"path":"/…/raw/<snapshotId>/images/001.jpg","kind":"image","mime":"image/jpeg"},{"path":"/…/raw/<snapshotId>/media/video.mp4","kind":"video","mime":"video/mp4"}]}],"topics":[{"slug":"手冲咖啡","path":"/…/in/2/topics/手冲咖啡.md"}],"options":{"folder":"usuyo"},"previous":{"douyin-4plrcxk343onmyxibmrr672kga":{"target":"usuyo/手冲咖啡：三段式注水.md"}},"endpoints":{},"out":"/…/out/2"}}
```

```json
{"jsonrpc":"2.0","id":2,"result":{"files":[{"path":"items/douyin-4plrcxk343onmyxibmrr672kga.md","target":"usuyo/手冲咖啡：三段式注水.md","usuyoId":"douyin-4plrcxk343onmyxibmrr672kga"},{"from":"item:raw/<snapshotId>/images/001.jpg","target":"usuyo/附件/douyin-4plrcxk343onmyxibmrr672kga-001.jpg","usuyoId":"douyin-4plrcxk343onmyxibmrr672kga"}]}}
```

- `items[].derived` 的键是 `produces` 里的值（`derived/asr.vtt`、`derived/ocr/*.json`、`derived/keyframes/` 等），值是路径，目录或通配的值是路径数组。`items[].files` 列出条目的原图和媒体：`path`、`kind`、`mime`；是各快照里还在的那些，同一 `sha256` 只列一次（先取主快照那份）。
- `writes: files`：插件把文件写进 `out`，结果里写明每个文件要放到目标的哪个相对路径；`out` 照 [§3.6](fetch.md#36-fetch-与输出目录) 的输出目录规则检查。原图、视频这类不用改的文件，结果里写 `{from: "item:<相对条目目录的路径>", target, usuyoId}`（例如 `item:raw/<snapshotId>/images/001.jpg`），由宿主从资料库直接复制，不经过 `out`。宿主写进目标文件夹、记哈希，导出记录放按机器目录的 `libraries/<libraryId>/exports/<target>.json`（每个对象 `{path, hash, base}`，见下面的 `readback`），导出路径每台各管（[SPEC §1.1](../spec/library.md#11-目录树)）。`run` 时插件不碰目标文件夹，`readback` 时只读它（[ADR-003 §7](../decisions/ADR-003-plugins.md#7-导出与读回)、[§7.1](isolation.md#71-常驻进程的路径)）。
- `target` 必须是规范化后的相对路径，不含 `..`、不是绝对路径，解析后仍在目标文件夹内；宿主写入时不跟随目标侧已有的符号链接。目标路径上已经有文件、而文件里没有对得上的 `usuyo_id` 时，宿主不覆盖，记进这次导出的问题列表让用户处理。
- `previous` 是导出过的对象的目标路径 `{id: {target}}`，取读回找到的路径，没找到的取导出记录里的 `path`（推断）。插件写回这个路径，写主题链接也照它；只有第一次导出的对象才按标题算路径。
- `writes: api`：插件经 `endpoints` 调云端接口，结果里返回 `records: [{usuyoId, remoteId}]`。
- 每个导出文件带 `usuyo_id`。**Obsidian 导出照这样转**（宿主给的字段照 [§13 第 34 项](open.md#13-待定)，待定）：
  - 条目一篇笔记。属性是 Web Clipper 的 `title` `source` `author` `published` `created` `description` `tags`（`description` 取摘要层，没有不写；`tags` 取 `tags.user`），加 `usuyo_id` `platform` `kind` `usuyo_takeaway` `usuyo_topics`（指向主题笔记的 wikilink，写成相对 vault 的路径，例如 `[[usuyo/主题/手冲咖啡]]`，不和你自己的笔记撞名，推断）；YAML 标量一律双引号。`published`、`created` 写 `YYYY-MM-DD`：`published` 有 `publishedAtOriginal` 就按它的偏移取日期，没有按导出这台的本地日期；`created` 取 `addedAt`（主题、卡片、成稿取 `createdAt`）的本地日期。正文先是你的笔记（`notes.md` 的正文），接一行分隔线 `%% usuyo：这行以下每次导出重写 %%`，下面是划线（照 [SPEC §1.4.2](../spec/library.md#142-卡片成稿记一句和划线) 的写法，行末 ` ^<id>` 本来就是 Obsidian 的块 id）、摘要、原文（宿主给的 `text`）、口播等各层。`notes` 里有一行正好等于分隔线，就转义，readback 时还原。通用 Markdown 文件夹用同一条分隔线，写成 HTML 注释。
  - 主题、卡片、成稿各一篇：`usuyo_id`、`title`、`created`，卡片另加 `usuyo_topics`；正文就是宿主给的副本，没有标记。主题页不写成员列表：条目笔记的 `usuyo_topics` 会出现在 Obsidian 的反向链接里，那就是成员。
  - 文件名照 [SPEC §0.6](../spec/conventions.md#06-文本和文件)，另把 Obsidian 链接、标题、块用的 `# ^ [ ] |` 换掉；目标文件夹里撞名的按 id 顺序加「 2」，记下目标路径；`folder` 变成子文件夹。在 vault 里改名、挪文件夹不碍事，读回按 `usuyo_id` 认；再导出时写回读回找到的路径。
  - 脚注、` ^<id>`、`usuyo://` 链接照原样过去（私密条目的脚注宿主给副本时已换掉，`usuyo://item/forgotten` 照留），不另加 wikilink。
  - readback 照这样转回来：
    - 属性：`tags` → `tags.user`，`usuyo_takeaway` → `takeaway`，`usuyo_topics` → `topics`（先由 wikilink 找到那篇笔记，再换成它的 `usuyo_id`）。卡片只有 `usuyo_topics` 这一项。
    - 条目正文：front matter 和分隔线之间的部分 → `notes`。
    - 主题、卡片、成稿：front matter 之后的整篇 → `body`。
    - 划线、分隔线以下的各层、日期、标题、文件名和别的属性不读回。
  - 待核实：Obsidian 留不留带偏移的日期时间属性（要加一个带时刻的属性，照它认的写法）。
- `readback`：插件只读整个目标文件夹，在里面按 `usuyo_id` 认出导出过的文件，挪过、改过名的也认得；不看以 `.` 开头的文件夹（`.trash`、`.obsidian`），那里的文件当作不在（推断）；把每份转回 `run` 收到的用户层，交回值：

```json
{"jsonrpc":"2.0","id":5,"method":"readback","params":{"vault":"/…/Obsidian/笔记","options":{"folder":"usuyo"}}}
```

```json
{"jsonrpc":"2.0","id":5,"result":{"objects":[{"id":"douyin-4plrcxk343onmyxibmrr672kga","kind":"item","path":"usuyo/手冲咖啡：三段式注水.md","hash":"sha256:<读到的那份的哈希>","takeaway":"闷蒸 30 秒比水温要紧","notes":"第一段水量记得称。\n\n周末试 1:15 粉水比","tags":{"user":["咖啡","周末要试"]},"topics":[{"id":"0193f2a4-7c1e-7a3b-9d2e-5f0a1b2c3d4e"}]},{"id":"0193f2a4-7c1e-7a3b-9d2e-5f0a1b2c3d4e","kind":"topic","path":"usuyo/主题/手冲咖啡入门.md","hash":"sha256:<读到的那份的哈希>","body":"……"}],"problems":[{"path":"usuyo/读书笔记.md","reason":"unparseable"}]}}
```

- `readback` 的参数：`vault` 是目标文件夹的绝对路径（`${TARGET}`，只读，[§7.1](isolation.md#71-常驻进程的路径)），`options` 同 `run`。
- `objects[]` 里 `id`、`kind`、`path`、`hash` 是读回自己的键：`kind` 是对象种类（`item` `topic` `card` `draft`），不是条目或成稿的 `kind`；主题和成稿笔记的属性一样，插件分不出，`kind` 由宿主按 id 查，插件可以不给。`path` 相对目标文件夹，不是 `run` 里副本的 `path`。用户层的键照 `run`（[§13 第 34 项](open.md#13-待定)）：`takeaway`、`notes`、`tags`（只有 `user`）、`topics`（每项只有 `id`）；主题、卡片、成稿的正文叫 `body`，是 `run` 里 `path` 那份副本的内容（推断）。`hash` 是读到的那份文件的 sha256。
- `problems[]` 是 `{path, reason, id?}`：属性坏了也按行找 `usuyo_id:`，找得到就带上 `id`，带 `id` 的对象这次不重新生成、不写；找不到 `usuyo_id` 的文件不算导出过的，不报（推断）。`reason` 只有两种，这份文件整篇不读：
  - `unparseable`：front matter 坏了、分隔线没有或不止一条、两个文件同一个 `usuyo_id`。
  - `unknown-topic`：`usuyo_topics` 里有链接指不到带 `usuyo_id` 的笔记。指到的不是库里的主题（成稿、卡片、条目），由宿主照 `unknown-topic` 办，那份不读也不写（推断）。
- `readback(run(V))` 按值等于 V 的用户层：字符串逐字节相同，集合相等。readback 就按这条定义，`usuyo plugin check` 跑这条往返测试（[§10](plugin-kit.md#10-契约测试与-plugin-kit)）。这里说的用户层只有 `takeaway`、`notes`、`tags.user`、`topics`，主题、卡片、成稿再加 `body`；划线、标题不算。`base`、往返测试、「和 `base` 一样」都只比这几项。
- 导出记录每个对象存 `{path, hash, base}`。`base` 是上次和 vault 对上的那份用户层：写出去了就是交给 `run` 的那份，没写出去就是那次读回的值（下面第 3、4 步）；私密条目的脚注已换成占位（[SPEC §1.3](../spec/library.md#13-条目的四个维度)）；占位行在底本和 vault 里一模一样，三方合并自然留下库里的真值。不另调一次 `readback` 读写出去的文件来取底本：每次导出只读回一次，转回去再转回来不变由往返测试保证。
- 导出只在你点「现在导出」时跑，不做自动、定时导出（[回答表](../answers.md)）。`readback: true` 的导出插件，宿主每次导出做四步：
  1. 调 `readback`。
  2. 逐个对象处理：
     - 库里没有，或在回收站里：列成 `item-deleted`（[SPEC §1.7](../spec/library.md#17-删掉)）。
     - 和 `base` 一样：跳过。
     - 其余做 merge3(`base`, 库里现在的, vault)，照 [SPEC §1.8](../spec/library.md#18-同步)：集合两边各自加的都留，一边去掉的就去掉；标量只有同一项两边都改了才出卡片；正文按行合并，只有改到重叠的行才算合不上，相邻的不算。
     - 合不上的：库里那份留下，vault 那份进回收站（`reason: conflict`，`kind` 用已有的 `notes` `topic` `card` `draft`），出「两边都改过的」卡片，vault 里这篇照库里的重写。
     - 差异照 [SPEC §1.4](../spec/library.md#14-你写的那一层)「读进来」记成已有的 op（`set-tags` `set-topics` `set-takeaway` `edit-notes` `edit-topic-page` `edit-card` `edit-draft`，[SPEC §1.5.1](../spec/library.md#151-op-词表)），记 `actor: external`、`via: readback:<插件 id>`。一次导出算一批，能整批撤销。
     - 没有底本的（导出记录丢了，或另一台第一次导出到同一个 vault）：照 [SPEC §1.8](../spec/library.md#18-同步)「云盘的冲突副本」那条：集合取并集，标量和正文两边不一样就算合不上，库里那份留下，vault 那份进回收站、出卡片。两台轮流导出到同一个 vault 时，底本按机器存，一台写出去的在另一台看来就是在 Obsidian 里改的：之后在库里去掉的会被加回来，改到同一段会出卡片（都记进改动记录，能整批撤销）。一个 vault 建议只由一台导出（推断）。
  3. 按合并后的库调 `run`，写到读回找到的路径。写之前再比一次文件哈希：和读回时的 `hash` 不一样（比如你正在 Obsidian 里打字），这一份这次不写，下次导出再读；它的 `base` 换成这次读回的值（推断：换成交给 `run` 的那份，下次会把没写过去的库里改动当成 vault 那边删掉的）。这次没交给 `run` 的（比如导出后设成私密的）同样，`base` 换成这次读回的值（推断）。
  4. 写出去的那些，`base` 换成这次交给 `run` 的那份用户层；没写出去的（`problems`、`item-deleted`）`base` 不动。
- 出了问题的文件（`problems`、`item-deleted`）不读也不写，列进这次导出的问题列表。在 vault 里删掉的笔记不算删掉，下次导出重新生成。
