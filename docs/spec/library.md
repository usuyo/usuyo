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
- 按平台分一层，文件夹名是 `itemId`（[§0.4](conventions.md#04-id)），两台离线算出同一路径；年月不进路径。回收站放 `.usuyo/trash/`：它是库的状态，各台各挪各的、只同步状态（[§1.7](#17-删掉)）；`items/`、`topics/`、`cards/`、`drafts/` 在 Finder 里只剩还在的。
- `library.json`：`{"formatVersion": 1, "extensions": {}, "libraryId": "<UUIDv7>", "createdAt": "…", "sync": null}`；开了内置同步，`sync` 是 `{"backend": "cloudflare", "remoteId": "…"}`，不含密钥。它在库里，库在 iCloud 里时另一台一打开就能提示二选一。
- `settings.json` 这里只定 `trashDays`（30，可选 90 或「一直」）；其余（转换哪几层、取不取评论、Agent 能看的范围、私密条目上不上云）归第 2 层。
- 跟着收的 `fetchDevice` 是哪台去取。`take` 只给订阅来的（RSS、简报、别人的公开账号）：`picked` 只收你挑的，`picked-or-matched` 也收命中你关注的，默认 `picked`（[PRD 7.3](../PRD.md#73-导入和跟着收) 的建议）；你自己在平台上的收藏夹和列表不带 `take`，新条目都收下。
- 跟着收列过哪些身份（只存身份的哈希，不存标题）记在本机 `jobs.db`，和翻页游标、上次成功跑完的时间（`--since`）放在一起，不跟着资料库走（推断）；它只收自己没列过的，见 [§1.7](#17-删掉)。这一次跑完才把列出来的记成列过；还没跑完过一次的订阅，下一次仍照第一次跑（推断）。每台第一次跑只记下已有的，不收（[PRD 7.3](../PRD.md#73-导入和跟着收) 对个人来源这样建议，这里所有订阅、每台都照它，推断）；换了 `fetchDevice`，换上的那台也照第一次跑，上一台最后一次跑完到这台第一次跑之间新出现的会漏掉（推断，这是代价）。

每台机器自己的目录、缓存和钥匙串见[按机器的目录](machine.md#按机器的目录)。

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
| `identities` | `[{platform, contentId?, url?}]`，`id` 由建条目时的身份算（没有身份的随机），之后只经合并换（[§0.4](conventions.md#04-id)），不看数组位置 | 宿主 | 集合三方合并，按 `identityKey` 认 |
| `kind` `title` `author` `publishedAt` `publishedAtOriginal` | 取自主快照；没有快照时 `title` 取 `list` 或投递给的值，再没有写原链接；`kind` 写 `unknown` | 宿主 | 跟主快照 |
| `lang` | 宿主从主快照的文字或字幕判断，BCP 47 | 宿主 | 跟主快照 |
| `addedAt` | 第一次收下的时间 | 宿主 | 取早的 |
| `origins`、`snapshots` | 见下 | 宿主 | 集合三方合并，按每项的 `id` 认 |
| `primarySnapshot` | 输出集合最大的，一样大先比 `at` 再比 `id`；没有快照为 `null`。11-11 代码：比的是各快照自己的 `outputs` 个数（不减 `textOnly.removed`），一样多取 `at` 晚的，再一样取 `id` 字符串大的 | 宿主 | 合并后重算 |
| `outputs` | 各快照 `outputs` 的并集，减去 `textOnly.removed` 涉及的 | 宿主 | 合并后重算 |
| `missing`、`missingReason` | 各快照 `missing` 的并集 − 条目 `outputs`，`state: text-only` 时只算文字类；快照的 `missing` 由取它的那台在取的时候算好，之后不改：这次的 `--want` − 实际拿到的；`--want` 一般是命中插件声明的 `outputs` ∩ 设置，重试只要缺的、「取评论」和只留文字以后才更窄（评论默认不算；投递来的、文件就是实际拿到的）。只用同步的字段算，两台算出一样的；设置、插件后来变了只影响下一次取。`missingReason` 是给你看的原因；11-11 代码取最新一份、还有缺的输出没补上的快照的 `missingReason` | 宿主 | 合并后重算 |
| `completeness` | `link` `partial` `full`：没有快照是 `link`，否则由 `missing` 算；只升不降（[PRD §8](../PRD.md#8-条目的状态)）：重算出的比现值低时留现值，条目页照 `missing` 说缺什么 | 宿主 | 两边和重算的取高的 |
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
| `entry` | `paste` `drop` `extension` `mobile` `import` `subscription` `mcp` `cli` `api`；「同步」指设备之间 |
| `takenBy` | 谁决定收的：`user`、`agent:<连接名>`（kebab-case，来源归第 2 层）、`subscription:<id>` |
| `machinePicked` | `null`、`{"by": "keywords"}`、`{"by": "similarity"}`、`{"by": "upstream", "name": "horizon"}`、`{"by": "agent", "name": "<连接名>"}`；只有你给的具体链接（Agent 收下时说明是你点名的链接，参数名归第 2 层）和你自己平台上的收藏、列表为 `null`；你给关键词让 Agent 去找的也算机器挑的（11-06 定，[§1.10 第 14 条](#110-要你看一眼)） |
| `matched` | 命中关注时 `[{topic, by: "keywords" \| "similarity"}]`，可以命中几个 |
| `subscription`、`importBatch` | 从哪个跟着收、哪一批来 |
| `action` `actedAt` `actedAtOriginal` | 平台上的动作和时间（[协议 §3.8](../protocol/fetch.md#38-list-与跟着收)） |
| `at`、`device` | 时间、哪台；转换在 `at` 最早那一项（一样早比 `id`）的 `device` 做（[调研 14 §0](../research/14-sync-like-hatoba.md#0-结论) 第 10 问），那台不在了怎么换归同步 feature |

你在跟着收的预览里自己点收下的，`takenBy: user`、`machinePicked: null`（推断：你一条条挑过）。跟着收按关键词或相似度命中收下的都标「机器挑的」。界面标「机器挑的」：每一项 `origins` 都带 `machinePicked`（你自己也收过一次就不标）；写过、放进主题、引用过都不去掉，它是来历（11-05 定，[§1.10 第 19 条](#110-要你看一眼)）；地图上怎么分开等 [M4](../PRD.md#76-地图)。

**`snapshots[]`**（[协议 §3.5](../protocol/fetch.md#35-canonicalize-与身份)–[§3.7](../protocol/fetch.md#37-fetch-输出)）：`id`（UUIDv7）；`plugin`、`pluginVersion`（交来已取到的内容时：投递接口记 `deliver:<令牌名>`，收件文件夹记 `deliver:inbox`；链接和交给导入插件的文件，记去取它的那个插件）；`at`、`inputUrl`（你给的原链接）、`resolvedUrl`（`canonicalize` 展开的完整链接）；`access`（投递为 `deliver`）、`site`、`upstream`；`outputs`（实际拿到的，比完整度只比它）；`missing`、`missingReason`、`completeness`（宿主在取的时候算，`missing` 是这次的 `--want` − 实际拿到的，之后不改）；`textOrigin`（`platform` 或 `machine`，等[协议 §13](../protocol/open.md#13-待定) 第 24 项）。

11-11 代码：快照的 `outputs` 由 `usuyo.fetch.json` 算：有 `text`、`html`、`images`、`media` 里的 `video`、`audio`、`subtitle`，标题、作者、发布时间有一样就算 `meta`，有 `comments`。快照的 `completeness`：没有输出是 `link`，没缺的是 `full`，否则 `partial`。只有 `missing`、`missingReason`，没有别的「缺」字段。11-11 还没做：`inputUrl` 记的是 `canonicalize` 给的 `resolvedUrl`（没有就是收下的链接），`resolvedUrl` 不写。

例：视频失败、文字成功，得快照 s1（`outputs: [text, images]`）；重试只要 `--want media:video`，得 s2（`outputs: [media:video]`，`missing: []`）。条目 `outputs` 是并集，`missing` 为空，不再显示「缺视频」。重试拿到的文件和已有快照逐个哈希都一样时，不新建快照。

<a id="5-derived-文件约定"></a>
<a id="122-raw-和-derived"></a>
#### 1.2.2 raw 和 derived

- **`raw/<snapshotId>/`** 是插件的输出目录整个移进来（[协议 §3.6](../protocol/fetch.md#36-fetch-与输出目录)），之后不改。宿主只加 `usuyo.fetch.json`：fetch 输出去掉 `credentialsUpdate`，加 `files: [{path, size, sha256}]`。同步前和「检查与修复」时按它比对，对不上就报「原样被改过」、不上传。只有你选「只留文字」或「删掉」，里面的文件才移去回收站。
- **放进 `raw/` 之前**（11-11 代码）：插件的输出先在 `.usuyo/tmp/` 下暂存，不在那里的不收。暂存的文件夹里不能有符号链接（`invalid-args`）；`raw/<snapshotId>/<路径>` 不超过 140 个 UTF-16 单位（`too-large`）。宿主在拿资料库的锁之前按 1 MiB 一块算好各文件的哈希，拿到锁以后再比一次文件名和大小，变了就拒（「hash 之后改过」）。逐个哈希和已有快照一样时不新建快照。
- **`usuyo.fetch.json` 里的路径**（`images`、`media[].path`、`files[].path`）只能是快照文件夹里的相对路径：不空、没有 `.`、`..`、绝对路径和盘符。加快照时不合的回 `invalid-args`，读到不合的当这个文件写坏了。
- **`derived/runs/<runId>/`：一次转换一个文件夹**（feature 003 [§5](../features/003-transforms.md#5-存在哪怎么重做)）。`runId` 是 UUIDv7。宿主先在 `.usuyo/tmp/` 拼好整个文件夹（插件的产出、宿主算的哈希、`run.json`），再改名进来；写进去以后不改，要改就是再转一次，新的一次和旧的并排。哪一次排在前面按 `runId`（就是时间）算，不另存「现在是哪份」；引用按秒、按 `imageKey` 找回，不认 `runId`，换了一次转录，写过的引用不断。改名进来之前先核：`runId` 是 UUIDv7、还没有这一次；文件夹里没有符号链接，文件和拿锁之前算好的哈希一样；声明的产物都在、大小和 sha256 对得上，`keyframes.json` 列的帧都在；没有声明以外的文件；`empty`、`outputs`、`reason` 对得上。不合的回 `invalid-args`，已有同一个 `runId` 回 `conflict`，条目不在回 `not-found`，暂存的文件夹删掉。核过以后各产物和 `run.json` 先 fsync，再整个改名；中途崩溃的只留在 `.usuyo/tmp/`，下次打开库时清掉，所以不会有没有 `run.json` 的转录。
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
| `tool` | 插件里干活的程序 `{name, version}`，插件在 `run` 的结果里报（[协议 §4.2](../protocol/transform-export.md#42-转换的-run)）；`by: host` 为 `null` |
| `model` | `{name, sha256}`，sha256 是宿主核过的；没用模型为 `null` |
| `options` | 这次实际用的参数，含插件补上的默认值；重做时原样交回 |
| `at`、`device` | 什么时候、哪台转的 |
| `trigger` | `auto`、`open`（你打开了这条）、`redo`（你点的） |
| `redoOf` | 点「再转一次」「换一个转」时当时眼前那一份的 `id`，不然 `null`；只是来历，不改谁在前 |
| `inputs[]` | `{path, sha256, snapshot, via}`：相对条目文件夹的路径（`raw/<snapshotId>/…`；`frames` 的是 `derived/runs/<runId>/keyframes/<文件>`）、所在快照；`via` 是中间转换的每一步 `{by, version, op, tool?, sha256?}`，不出文件的（`probe`）没有 `sha256` |
| `outputs[]` | `{path, kind, size, sha256, lang?}`：文件夹里的产物，路径相对这个文件夹；`asr` 带 `lang`；关键帧图片算在 `keyframes.json` 那一项里 |
| `empty`、`reason` | 没有可产出的东西时 `true`，`reason` 是 `no-speech`、`no-audio-track` 或 `no-text`，这时 `outputs` 为空；`false` 时 `reason` 为 `null` |
| `marks[]` | 很明显的标记 `{code, speechSec?, durationSec?, frozenSec?}`，`code` 是 `no-audio-track` `no-speech` `static-picture` `no-text`（feature 003 [§6](../features/003-transforms.md#6-很明显的标记)）；只提示，不藏任何东西 |

- 词表（`job`、`trigger`、`kind`、`reason`、`code`）在类型里是字符串：别的设备上新版本写的转录，旧版本照样读得进来、照样写回。`inputs[].path`、`outputs[].path` 只能是普通相对路径，规则同 `usuyo.fetch.json`，读到不合的当这个文件写坏了。
- **`no-audio-track`** 不经转写插件：宿主让抽音轨的插件做一次 `probe`（[协议 §4.2](../protocol/transform-export.md#42-转换的-run)），看不到音频流就自己写一次 `job: asr` 的转录：`by: host`，`inputs` 是那个画面文件，`via` 记那次 `probe`，`empty: true`，`reason: no-audio-track`。
- `imageKey` 是图片字节 SHA-256 的前 16 位十六进制，两份快照里同一张图只认一次。`readable.md`（网页抽出的正文）由转换插件产出，现在没有插件产出它，做的时候照转录的规矩；`thumb.jpg`、`fingerprints.json` 由宿主产出；`embeddings.bin` 的格式归 plugin-kit，不同步。

**产物的格式**（schema 在 `crates/usuyo-contract/schema/`，[§1.9](#19-schema-和-usuyo-contract)）。时间是秒，最多 3 位小数（[§0.5](conventions.md#05-时间)）；**图上的框**是 `[x, y, w, h]`，0–1，按图的宽高算，**原点在左上角**，引用链接里换成百分数（[§0.4](conventions.md#04-id)）。

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
- 第二行：作者 · 发布日期（照 [§0.5](conventions.md#05-时间)）· 平台 id · `kind`，缺的不写（`kind` 是 `unknown` 也不写）。写平台 id 不写显示名，不同版本的应用写出一样的字节。
- 第三行是链接：主快照的 `resolvedUrl`，没有就 `inputUrl`，再没有就身份的 `url`；写成 `<…>`，空格、`<`、`>` 百分号编码。
- 标题、作者照 [§0.6](conventions.md#06-文本和文件) 的 Markdown 链接文字转义。
- **正文**（11-11 代码）：插件给的 `textFormat: markdown` 只有两条都满足才照原样写：全文没有 `<`；每个链接目标（`](` 或 `]:` 后面，跳过空白）以 `http://`、`https://`、`mailto:`、`usuyo://` 或 `#` 开头，不分大小写。不满足的和 `plain` 一样照 [§0.6](conventions.md#06-文本和文件) 转义成纯文字。原样还在 `raw/`。

主快照或头部用到的字段变了就重写，原样在 `raw/`，不违背 [PRD 原则 2](../PRD.md#4-原则)（[K5 A](../PRD.md#710-存储删除与联网)）；CLAUDE.md 的措辞见 [§1.10 第 10 条](#110-要你看一眼)。应用里的原文栏和 `text` 引用读主快照的 `text` 或 `derived/readable.md`（下表），不读 `content.md`。**能看不能改**（11-05 定，[回答表](../answers.md)）：宿主把它写成只读文件（系统支持时），总照主快照重写；重写前发现外面改过（和本机记下的上次写的哈希不一样，也不等于这台照现在的 manifest 和主快照会生成的字节；等于的话是另一台经云盘写来的，照收、记下哈希），先把改过的那份放进回收站（`kind: content`），再换回原样（推断）。要写字写在 `notes.md`、主题页里，或者导出到你的 vault 再写（[协议 §4.4](../protocol/transform-export.md#44-导出的-run-与-readback)）。它不同步，各台照原样自己生成。这偏离[调研 14 §5](../research/14-sync-like-hatoba.md#5-改动和冲突) 把它归「原样」层。

| 栏 | 文件 | 来源 | 链接里的层 |
|---|---|---|---|
| 原文 | 主快照 `raw/<snapshotId>/usuyo.fetch.json` 的 `text`，或 `derived/readable.md`；`content.md` 是只读副本 | 平台；机器 | `text` |
| 图里的字 | `ocr` 转录的 `derived/runs/<runId>/ocr.json` | 机器 | `ocr/<imageKey>`、`image/<imageKey>` |
| 口播 | 平台字幕 `raw/…/media/*.vtt`，加各次 `asr` 转录的 `derived/runs/<runId>/asr.vtt`，几份并排 | 平台；机器 | `asr`、`video`、`audio` |
| 画面 | `play` 转录的 `play.mp4`，没有就是原样的画面；加最新一次 `keyframes` 转录的关键帧 | 原样；机器 | `video` |
| 画面里的字 | `frames` 转录的 `derived/runs/<runId>/ocr.json` | 机器 | `video`，带 `#t=` 和 `xywh`（[§0.4](conventions.md#04-id)） |
| 画面描述、AI 摘要 | `derived/vision.md`、`derived/summary.md` | 机器 | `vision`、`summary` |
| 你的笔记 | `notes.md` | 你 | `notes` |
| 评论（可选） | `raw/<snapshotId>/comments.json` | 平台 | `comments` |

评论默认不取，`settings.json` 里按平台打开、设上限；单条在条目页点「取评论」，那是一次只 `--want comments` 的获取，存成新快照，评论留着当时的样子。

<a id="13-条目的四个维度"></a>
### 1.3 条目的四个维度

| 维度（[PRD §7.1](../PRD.md#8-条目的状态)） | 存在哪 | 怎么变 |
|---|---|---|
| 存下了多少 | `outputs`、`completeness`、`missing` | 宿主算；`completeness` 只升不降 |
| 后台任务到哪一步 | 本机 `jobs.db`；做好的看 `derived/` | 不进资料库，两台不会来回改 |
| 存在 | `state`；移除后就没了 | 你触发；Agent 或 `api` 令牌收下时把回收站里的拿回来、Agent 经 MCP 撤它自己那批时放回回收站也改它（[§1.6](#16-改动记录)、[§1.7](#17-删掉)）；到期移除除外 |
| 私密 | `private` | 你。不给 Agent（回 `not-found`，不带 `reason`），不进导出，同步默认照样加密上云，可选不上（开关归第 2 层，[§1.1](#11-目录树)） |

不记看没看过，也没有「没整理」（[§1.4.4](#144-发生过的事)）。

**给 Agent 和导出的是副本**：宿主把主题页、卡片、成稿复制一份给 Agent 和导出插件，原文件不动。副本里引用私密条目的整条脚注换成 `[^k3f7q]: 「私密条目」`；主题的成员由索引给，私密的不列。你正文里自己写的字不动。导出前提示这篇引用了几条私密条目。

<a id="14-你写的那一层"></a>
### 1.4 你写的那一层

链路照 [T1 A](../PRD.md#74-变成文字)：素材（各栏、划线、记一句）→ 卡片 → 主题页 → 成稿。都是普通 Markdown，id 在 front matter 的 `id`（`notes.md`、`highlights.md` 没有，文件夹名就是条目 id）。不写修改时间，两台都改过会在这一行撞上，修改时间从改动记录取。画布等 [M7](../PRD.md#76-地图)（建议一张地图）和画布原型，先留 `canvas/`，按 JSON Canvas 1.0，同步以节点 id 为键合并。

**标题和文件名**：`title` 是真相，文件名 = `sanitize(title)`（[§0.6](conventions.md#06-文本和文件)），子文件夹都算，同步的记录带子文件夹、不带文件名。

- 本机为避免撞名加的「 2」不同步，不算改标题；两个同名时 `id` 字符串序大的那份在本机加，每台算出同一结果；同名的不自动合并，界面提示一行。

**usuyo 外改了资料库里的文件**：不是支持的用法，改东西走应用、MCP，或导出再读回（[协议 §4.4](../protocol/transform-export.md#44-导出的-run-与-readback)）。宿主只保证不丢、不悄悄复活。

- **怎么发现**：每次写回前比哈希（[§0.6](conventions.md#06-文本和文件)），加上启动时和定时按大小、修改时间扫一遍；文件监听只是让它快一点（第 2 层）。等文件不再变了再处理，等多久归第 2 层。「外面改过」指和本机上次写的、读到的不一样。底本是 `sync.db` 里上次见过的那份，没开内置同步也存；没有底本就以文件现在的样子为准（[原则 3](../PRD.md#4-原则)），改动记录那条不带 `change`。
- 下面三类每一类都拿这个情形试过：库在 iCloud、Dropbox 里时，另一台 usuyo 改这些文件是正常的。

1. **读进来**（同步的文件都是）：主题、卡片、成稿的 `.md` 和 `notes.md` 的 front matter 和正文；`highlights.md`；`manifest.json`，先过 schema，再按 [§1.2.1](#121-manifestjson-v1) 最后一列逐字段合并。和底本照 [§1.8](#18-同步) 三方合并，记 `actor: external`、`via: file`，能撤销。导出读回走同一条路：另一边是导出插件转回来的用户层（[协议 §4.4](../protocol/transform-export.md#44-导出的-run-与-readback)），底本取导出记录里的 `base`（[§1.1](#11-目录树)），不取 `sync.db`，没有底本照 [§1.8](#18-同步)「云盘的冲突副本」那条：集合取并集，标量和正文两边不一样就算合不上，留库里这份，vault 那份进回收站、出卡片；记 `via: readback:<插件 id>`。只合并插件交回的这几项值，下面挪动、改名、删掉、解析不了这几条不用于读回：标题和文件名不读回，vault 里删掉的下次导出重新生成，解析不了的照[协议 §4.4](../protocol/transform-export.md#44-导出的-run-与-readback) 列进这次导出的问题列表。
   - 主题、卡片、成稿挪到自己文件夹里的另一个子文件夹：记 `move`。
   - 主题、卡片、成稿改了文件名：去掉本机后缀后不等于 `sanitize(title)`，且 `title` 和底本一样，就当改标题，`title` 取新文件名；`title` 也变了以 `title` 为准，文件名照 `title` 改。front matter 没跟着变的改名先等一会儿再判，云盘常先送到改名（多久归第 2 层）。文件名对得上 front matter 的 `title` 就照收，另一台改了标题、经云盘过来的就是这样。
   - 删掉了，或移出 `topics/`、`cards/`、`drafts/`：记 `move-to-trash`，底本里的文本进回收站，能恢复；回收站里已经有它（库在云盘里，另一台已经挪进去）就不再记。
   - `notes.md` 或 `highlights.md` 删掉了：上次那份进回收站（`kind: notes`），条目 `state` 不变；别的台照 `edit-notes`、`set-takeaway` 或 `remove-highlight` 同步，不按回收站。
   - 条目文件夹或 `manifest.json` 删掉了：当 `move-to-trash`（`reason: delete`），回收站里放能找回的（`sync.db` 底本里的 manifest、`notes.md`、`highlights.md`）；回收站里已经有这一条（库在云盘里，另一台已经挪进去，manifest 是 `trashed`），或它的 id 在别的条目的 `aliases` 里（并进了别的条目或改了名），就不再记。
   - 解析不了的（front matter 不合 [§0.6](conventions.md#06-文本和文件) 的写法、`highlights.md` 有一行不合 [§1.4.2](#142-卡片成稿记一句和划线) 的写法、JSON 坏了、`manifest.json` 的 `id` 和文件夹名不一样）：不改写、不同步，沿用上次的值，界面一行「这个文件在 usuyo 外改坏了」；你下次在应用里改这个对象时，坏的那份先进回收站（`reason: conflict`）。
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

- `followed` 是「关注」，取消叫「不再关注」（[S11](../BUILD.md#7-只有用户能答的和能验的)）；只有关注着的主题参与命中。`match` 给跟着收判断命中（[M1 B](../PRD.md#76-地图)）：`keywords` 按 `keywords` 里的词字面命中，`similarity` 按和成员的相似度，可以都写。两种都看标题层（标题、简介、标签）和内容层（正文、口播转的文字、字幕、图里的字），画面描述默认不算，主题里能打开（11-06 定，[§1.10 第 7 条](#110-要你看一眼)；开关的字段归 feature）。看内容层要先把新条目取下来转成文字；没命中的不进资料库，取下来的临时文件删掉（推断）。卡片放进主题写在卡片自己的 `topics`；条目放进主题写在 manifest 的 `topics`（[§1.2.1](#121-manifestjson-v1)）。`create-topic` 的 `match` 默认 `[keywords]`。
- 列表一行一项；空列表不写，没有这个键就是空的。
- 成员记在成员上，加减成员不碰主题文件，主题页只因你改它而变（[PRD 7.7](../PRD.md#77-你写的那一层)）；命中关注只记在 `origins[].matched`，不自动加成员。主题彻底移除以后，`topics` 里指向它的 id 读的时候跳过，不去改写。

<a id="142-卡片成稿记一句和划线"></a>
#### 1.4.2 卡片、成稿、记一句和划线

- `cards/<标题>.md`：front matter 是 `id`、`title`、`createdAt`、`topics`（放进的主题 id），正文是你的一个观点。没写标题时，新建那次取第一行前 30 个字写进 `title`，之后不跟着变。
- `drafts/<标题>.md`：front matter 是 `id`、`title`、`createdAt`、`kind`（`article` `post` `script` `other`）、`from`（用到的主题、卡片的 id，是哪种按 id 查）。引用主题页某段写 `usuyo://topic/<id>#^<块 id>`，顺着链接一路到原文那一段、那一秒。
- 块 id 谁加：你在主题页上「引用这一段」时，宿主在段末补 `^块id`，记 `edit-topic-page`（`actor: user`），能撤销；Agent 起草的成稿要引用某段时，补块 id 放进同一个提议。
- 记一句在 `notes.md` 头部的 `takeaway`，一条一句（[M9 A](../PRD.md#76-地图)），没有就不写这个键，也就不带 front matter；正文是你的笔记。划线在 `highlights.md`，一条贴在某一栏的某一处。

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

`highlights.md` 一行一条划线，不用注释圈：`- 「<原话>」[<栏 位置>](usuyo://item/<本条 id>/<层>@<snapshotId>#<片段>) ^<块 id>`，`@<snapshotId>` 照 [§0.4](conventions.md#04-id) 哪些层带，不带标签、标题、作者、日期；块 id 见 [§0.4](conventions.md#04-id)；原话照 [§0.6](conventions.md#06-文本和文件) 转义。各行按加的先后排，同步按行合并，两台同时各加一行都留，按 `deviceId` 排先后。`add-highlight` 在末尾加一行；`remove-highlight`（`id`）删掉以 ` ^<id>` 结尾的那一行。usuyo 外改了照 [§1.4](#14-你写的那一层) 读进来，有一行不合写法就算这个文件写坏了。引用只抄原话和位置，不指向划线的块 id。

<a id="143-引用"></a>
#### 1.4.3 引用

```
[^<5 位随机>]: 「原话」[标题 · 栏 位置](usuyo://item/<id>/<层>@<snapshotId>#<片段>) · 作者 · 发布日期
```

- 只有宿主插入的引用（`add-citation`、从界面拖进来的）写成脚注，正文保持干净；你手写的行内引用照样认，不改写。脚注标签随机，两台各加一处不撞。
- 原话、标题、日期写在链接外：任何查看器里、来源删掉以后这一页都完整，应用里链接写「来源已删掉」。脚注是 GFM、Pandoc 都认的写法，纯文本里来源排在页尾。
- 发布日期照 [§0.5](conventions.md#05-时间)：取 `publishedAtOriginal` 的日期部分，没有就取 UTC 日期。
- 标题、作者、日期在插入引用时抄下，之后不跟着刷新：那是你这一页的字。
- MCP 读主题、卡片、成稿时，宿主把脚注解析成 `{label, item, layer, snapshot, fragment, quote, title, author, date, resolved}` 一并给；`add-citation` 收其中的 `item`、`layer`、`snapshot`、`fragment`、`quote`，加 `anchor`（[§1.5.1](#151-op-词表)），标签宿主随机起，标题、作者、日期宿主插入时抄；Agent 不读写脚注语法（工具形状归第 2 层）。
- 时间类的 `@<snapshotId>` 是那次转录 `run.json` 的 `inputs[].snapshot`，即那份媒体所在的快照（[§0.4](conventions.md#04-id)）。链接里 `#:~:text=` 的原话超过 32 个字写 `开头,结尾`；「」里的原话照全文存，不截。
- **找回**：时间类按「那份媒体还在、这一秒在时长内」判有效，原话只作提示；那一秒的文字变了（新引擎把「40 克」写成「四十克」），把现在的文字并排显示，不标「找不到」。文字类按 `#:~:text=` 模糊匹配（去标点、数字归一，推断），还找不到才显示存下的原话并标出来。

<a id="144-发生过的事"></a>
#### 1.4.4 发生过的事

[M6 A](../PRD.md#76-地图) 只记发生过的事。记过一句、被引用从文件里看得出，只有「打开过」另记，一行一次：`{"n":1,"t":"2026-11-05T03:00:01.123Z","ev":"opened","item":"douyin-4plrcxk343onmyxibmrr672kga"}`，放 `.usuyo/facts/<deviceId>/`，分段和改动记录同一套（[§1.6](#16-改动记录)）。

- 短时间里反复打开只记一次，间隔归第 2 层。不写进 manifest：每打开一次就改一次条目记录，同步要多传很多。
- 「打开过几次、最近哪天」由索引算；索引不从 `facts/` 推出「没打开过」的列表、数字或标记（[原则 4](../PRD.md#4-原则)）。
- `facts/` 默认不给 Agent，和「我的笔记」同一个开关：你打开过什么是行为数据。

<a id="15-改动提议"></a>
### 1.5 改动提议

Agent（经 MCP）或宿主调的模型改已有内容，一个提议一个文件 `.usuyo/changes/<id>.json`；放 `jobs.db` 会跟着机器丢（[第 14 条的回答](../answers.md#14-资料库能不能放进同步盘)）。模型起草由宿主照[协议 §4.3](../protocol/transform-export.md#43-模型与密钥由宿主代发) 自己调 provider，不经转换插件。

<a id="151-op-词表"></a>
#### 1.5.1 op 词表

提议和改动记录共用。删除类 Agent 和模型都不能提议（[原则 5](../PRD.md#4-原则)）。

| op | 对象 | 参数 | 能提议 | 能撤销 |
|---|---|---|---|---|
| `add-item` | 条目 | `origin`、`created` | 不经提议，直接生效（[A3 A](../PRD.md#78-agent)） | 能，整批 |
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
- **`merge-items`**（A 并进 B）：A 的 `raw/<snapshotId>/` 整个移进 B 的 `raw/`；`snapshots`、`identities`、`origins` 并过去，`topics` 取并集，B 的 `aliases` 加上 A 的 id 和 A 的 `aliases`。A 的笔记正文按段接在 B 后面，记一句不一样时问你留哪句（同步自动并时出卡片）；划线：A 的 `highlights.md` 各行接在 B 后面，块 id 在 B 的文件里撞了就换一个。A 的 `manifest.json`、`content.md`、`derived/` 和一份 `notes.md`、`highlights.md` 进回收站（`reason: merge`），`derived/` 能重算；`raw/` 不进回收站：[K5 A](../PRD.md#710-存储删除与联网) 只许「只留文字」和「删掉」把 `raw/` 移出资料库。
- 主题「合并进另一个」（[PRD §7.3](../PRD.md#82-其他对象)）的 op 等主题 feature 再加。

<a id="152-提议文件"></a>
#### 1.5.2 提议文件

顶层 `id`、`createdAt`、`proposer`、`model?`、`via`、`summary`（Agent 写的一句）、`state`、`decidedAt`、`ops`；每个 op 是 `{id, op, target: {kind, id}, base, args, state}`，[§1.5.1](#151-op-词表) 的参数都放在 `args` 里，`hunks` 也是（11-11 代码，`{"op": "<名字>", "args": {…}}`）；改了再接受的另有 `edited: true`。`proposer` 是 `agent:<name>`，或宿主调模型时的 `model:<provider 名>`（另记 `model`）。`base` 是提出时目标文件的 `sha256:<hex>`，新建对象为 `null`。

- **状态**：提议 `proposed` → `accepted` / `partly-accepted` / `rejected`；op 和 hunk `proposed` → `accepted` / `rejected` / `conflict`，逐个决定。改了再接受记 `edited: true`，实际写进去的字在改动记录里。
- **不作废**（11-06 定，[§1.10 第 6 条](#110-要你看一眼)）：没决定的一直留着，不计数、不提醒（[原则 4](../PRD.md#4-原则)）；接受时内容已经变了的照下面逐项对，对不上的标 `conflict`。决定完 30 天后，每台按同一规则从 `changes/` 移走，先移的推墓碑。
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
- **Agent 直接收下**（[A3 A](../PRD.md#78-agent)）：`add-item`，`actor: agent:<name>`、`via: mcp`，一次调用一批。已在库里、能给它看的只多一项 `origins`（`created: false`），撤销只去掉这一项；私密的不收，回 `skipped`；在回收站里的拿回来（[§1.7](#17-删掉)；离线短链展开后的见 [§0.4](conventions.md#04-id)）。撤销新建的进回收站（`reason: undo`）；之后你写过、划过、放进主题或引用过的，在应用里和终端里的命令行撤之前问你一次也撤不撤（11-05 定，[§1.10 第 9 条](#110-要你看一眼)）；问不了时跳过并列出：命令行不在终端里这半是推断，Agent 经 MCP 见下一条。
- **Agent 经 MCP 能撤销哪些批**（11-05 定，[§1.10 第 18 条](#110-要你看一眼)）：只能撤同一个 Agent 的批。它这一批收进来或从回收站拿回来的条目，你没写过东西的放进回收站：新建的用 `reason: undo`，从回收站拿回来的照 [§1.7](#17-删掉) 原样放回。写过的照上面跳过并列出；你之后设成私密的也跳过，列给 Agent 时不列它（[§1.3](#13-条目的四个维度)）。你接受过的提议那批（带 `changeSet`）里新建的主题、卡片、成稿，经 MCP 撤时跳过并列出，要在应用里撤（推断）。不进回收站的步骤照常撤。撤掉它自己收进来的不算删除（[原则 5](../PRD.md#4-原则)）。
- **外面改的**：导出读回的（`via: readback:<插件 id>`）和 usuyo 外改了、照 [§1.4](#14-你写的那一层) 读进来合并的（`via: file`）不走提议（[ADR-003 §7](../decisions/ADR-003-plugins.md#7-导出与读回)），一样能撤销。`actor` 记 `external`，不断言是你：直接改文件的 Agent（Claude Code 一类）分不出来。第 2 层的 MCP 工具说明写明：改已有内容走提议，不要直接改文件。
- **活动记录**（[PRD 7.8](../PRD.md#78-agent)）：由 `activity/`（在按机器的目录，搜过、读过，[§1.1](#11-目录树)）、`journal/`（写过）、`changes/`（提议过）拼出。

<a id="17-删掉"></a>
### 1.7 删掉

照 [K1 A、K3 A、K5 A](../PRD.md#710-存储删除与联网)；K1 A 里的删除记录 11-06 去掉了：删除就是删除，移除以后不留记录（[§1.10 第 12 条](#110-要你看一眼)）；K3 的「彻底忘掉」11-06 并进删掉，不再单独有（[§1.10 第 16 条](#110-要你看一眼)）。

- **回收站**：`.usuyo/trash/<trashId>/` 里一个 `entry.json` 加文件；`entry.json` 是 `{id, kind, target, path, files, reason, trashedAt}`，`kind` 取 `item` `topic` `card` `draft` `notes` `content`，`path` 是原相对路径，`files` 是对象以内的相对路径，`reason` 为 `delete` `keep-text` `merge` `undo` `conflict`。各台各挪各的，同步的是条目的 `state` 和主题、卡片、成稿记录上的 `trashedAt`（推断，归同步引擎）。进回收站时取消 `jobs.db` 里它的任务，之后回来的结果丢掉，不在原路径重建文件夹。
- **回收站里怎么放**（11-11 代码）：`trashId` 必须是 UUIDv7。`path` 和 `files` 里每一项都只能是相对路径，不能有 `.`、`..`、绝对路径和盘符，不合的这一项当写坏了。

| 进回收站的 | 文件夹里 | `files` |
|---|---|---|
| 整条条目 | 条目文件夹原样改名进来，加 `entry.json` | 里面全部文件 |
| 主题、卡片、成稿 | `body.md` | `["body.md"]` |
| `notes`、`content` | 原文件名的那个文件（`reason: conflict`） | 那一个 |
| 只留文字 | 照条目里的相对路径放，例 `raw/<snapshotId>/images/001.jpg`、`derived/runs/<runId>/play.mp4` | 这些路径 |
- **到期**：照回收站项 `entry.json` 的 `trashedAt`（条目、主题、卡片、成稿都一样；2026-11-11 定，原先写的是条目按 manifest 的 `stateAt`），加当前的 `trashDays` 现算（[调研 14 §5](../research/14-sync-like-hatoba.md#5-改动和冲突)「到期照 A 写的算」），「一直」不到期；`entry.json` 的 `trashedAt` 照同步来的值写，不用本机挪进来的时间；`merge`、`conflict`、`kind: notes` 和 `kind: content` 只在本机，按本机的。移除本机文件，记 `purge`（`actor: host`）；`keep-text`、`merge`、`conflict`、`kind: notes` 和 `kind: content` 到期只做这一步。对象整个离开资料库的（条目 `reason: delete` 或 `undo`；主题、卡片、成稿，`conflict` 的除外）还要清干净、推墓碑，见下面的「清干净」。开了同步的库启动后先拉完一轮再判，按服务器时间（[调研 14 §5](../research/14-sync-like-hatoba.md#5-改动和冲突)），离线时不移除，推墓碑走 CAS，底本不是最新就放弃这次、重新判断。没开同步的按本机时间；本机时间比记下的最大时间晚了超过 `trashDays`（会把回收站整个移除），先不移除，你点「照时间移除」才移除（推断；这个放行还没做，[总纲](../BUILD.md#5-功能线)）。你在回收站里「立即清除」（[PRD §7.3](../PRD.md#82-其他对象)）照到期办，不等 `trashDays`（11-06 定，[§1.10 第 16 条](#110-要你看一眼)）；确认框列出还在引用它的页（推断）。11-11 代码：守护进程启动时查一遍，之后每 6 小时一遍（每遍都是整个回收站），到期的照下面移除。到期时间照回收站项 `entry.json` 的 `trashedAt` 加 `trashDays` 现算；`trashDays` 是「一直」时不到期。一项读不了的跳过、只记回收站 id 和错误码（日志里没有标题），不拦别的。时钟守卫：记下的最大时间放在 `libraries/<libraryId>/clock-mark`（还没有就取改动记录里最晚的一行）；这一遍有到期的、而现在比它晚了超过 `trashDays` 整天时，一项也不移除，只记下有几项因此留着（日志里的 `held`），`clock-mark` 也不更新。没有这种情况的话，这一遍结束把 `clock-mark` 更新成它和现在中较晚的。代码里有「照时间移除」这条路（跳过守卫），但还没有接口和界面能触发（还没做，[总纲](../BUILD.md#5-功能线)）；每 6 小时的那一遍永远只用 `Check`，守卫拦下以后会不会自己放行是个开放的问题，也记在 TODO。回收站里一项也可以「立即清除」：`POST /trash/{trashId}/purge`，`trashId` 是回收站项的 id，不是条目 id；成功回 204，一项照到期办，整个离开资料库的条目发 `item-removed`，只留文字移出的媒体清掉时发 `item-updated`。`GET /trash` 列出回收站（[本机接口](local-api.md#本机接口第-2-层11-11-照代码)）。
- **再遇到同一身份**（11-06 改，[§1.10 第 12 条](#110-要你看一眼)：不记删过的，移除以后再收下就是普通收下）：

| 入口 | 在回收站里 | 私密 |
|---|---|---|
| 你或 Agent 主动收的：窗口（粘贴、拖入、快捷键、`usuyo://add`）、命令行、扩展、手机、Agent（`mcp`）、本机接口令牌（`api`）、批量导入 | 拿回来，记 `restore`，能撤销；不提醒 | 窗口、终端里的命令行、扩展、手机、批量导入照常，多一项 `origins`；Agent、`api`、终端外的命令行跳过（11-11 还没做：`POST /items` 一律记窗口里收的，命令行从不跳过；导入不跳私密，扩展、手机和 `api` 令牌的投递会跳过。见 [总纲](../BUILD.md#5-功能线)） |
| 跟着收（这个订阅没列过的） | 跳过 | 照常 |

- **跟着收**只收这个订阅没列过的（[§1.1](#11-目录树)）：列过的一律跳过，它列过、你删掉的，到期移除以后也不会被它收回来；它没列过的（比如你自己粘贴过、删掉、已经到期移除的），以后它第一次列出来就照新的收（这是代价）。库里已有、它没列过的不新建，只多一项 `origins`（推断）。`newest-first` 的来源翻到一整页都是这个订阅列过的就停（[协议 §3.8](../protocol/fetch.md#38-list-与跟着收)）。
- 跳过的回 `skipped`，不加 `origins`；主动收的只有 Agent、`api`、终端外的命令行遇到私密条目这一种。又私密又在回收站里的：Agent、`api`、终端外的命令行照私密那一列跳过；别的入口拿回来（记 `restore`），多一项 `origins`，`private` 不变（推断）。代价：`skipped` 现在只在私密时出现，Agent 和 `api` 令牌能据此知道这条在库里、是私密的（11-05 以前和删过的、在回收站的不区分）；11-06 定照这样（[§1.10 第 20 条](#110-要你看一眼)），内容、标题都不给。
- **撤销收下**：新建的进回收站（`reason: undo`）；从回收站拿回来的原样放回：同一个回收站项（`trashId`），`trashedAt`、`stateAt` 照拿回来之前的，到期照原来的算（应用里、命令行、MCP 都一样）。原来的到期时间已经过了的，`trashedAt`、`stateAt` 记放回的时间，再留 `trashDays`（推断：不然一撤销就被移除，Agent 经 MCP 撤就成了删除，[原则 5](../PRD.md#4-原则)）。
- **撤销那批的例外**：原因是 `undo` 的条目，和撤销收下时原样放回回收站的条目，同步带来你写的改动（`actor` 是 `user` 或 `external` 的记一句、改笔记、划线、放进主题、引用）时恢复它，记 `restore`（`actor: sync`），出一行「撤销时跳过了这条，你在另一台写过」。
- **清干净**（11-06，原来的「彻底忘掉」并进这里，[§1.10 第 16 条](#110-要你看一眼)）：改动记录里提到它（含它 `aliases` 里的 id）的各行（包括封口的段和这次记的 `purge`），`targets[].id`、`targets[].path`、`change` 换成 `{"redacted": true}`，其余字段不动；所有提议里它的 op 删掉；本机 `sync.db` 底本、`backups/`、`exports/`、`imports/`、索引里它的部分去掉。条目另外：`facts/` 里它（含 `aliases`）的行删掉；并进它的（`reason: merge`）和它只留文字时移出来的（`reason: keep-text`）回收站项一起移除；别的条目 manifest 的 `sameAs`、`notSameAs`、`aliases` 去掉它；你的主题页、卡片、成稿、笔记里原话和标题留着，链接里的 id 是它或它的 `aliases` 的，换成 `usuyo://item/forgotten`（`actor: host`），应用里写「来源已删掉」（[§1.4.3](#143-引用)）；`jobs.db` 里跟着收列过的只有身份的哈希，不去掉（推断：去掉了，跟着收会把它再收回来）。云端推墓碑，多久后云端也没有了照[调研 14 §5](../research/14-sync-like-hatoba.md#5-改动和冲突)（D1 时间回溯、R2 桶锁），界面写明。
- **移除时写什么**（11-11 代码）：只在本机的（`keep-text`、`merge`、`conflict`、`kind: notes`、`kind: content`）删掉文件夹，记一行普通的 `purge`，`change` 是 `{trashId}`。整个离开的记一行 `purge`，`targets[].id`、`targets[].path`、`change` 都是 `{"redacted": true}`；再照上面清干净：所有设备的段里，`targets[].id` 是它、路径里或 `change` 里出现它（含 `aliases`）的行都脱敏；提议里它的 op 删掉，删空了的提议文件删掉；页面里的链接换成 `usuyo://item/forgotten`，回收站里的副本也换，记一行 `purge`（`actor: host`，`targets` 是改过的文件）。索引、`sync.db`、`backups/`、`exports/` 由调用方清。还不推墓碑（同步还没做）。
- 别的台拉到墓碑后照上面移除自己的文件、清干净自己手上的。条目、主题、卡片、成稿的墓碑只在清干净时推，不另带标记，移除时不在线的那台回来拉到也照办（推断）；它离线时改过或拿回来的照下面「和同步」，那一份等它自己到期再清干净。改写是确定的：每台先改自己手上全部段的副本（包括别的台写的）；各段的主人再把改过的段推一次，记录换成新的 blob id，旧 blob 照[调研 14 §5](../research/14-sync-like-hatoba.md#5-改动和冲突) 的清理规则清掉，以后加入的设备拉到的就是改过的。主人已被撤销（`devices/<deviceId>.json` 有 `revokedAt`）时由推墓碑的那台推，这是「每段只有一台写」唯一的例外。同一条一台重新收下、一台移除：拉到对方的版本后条目还在的，那台不做这些改写，界面一行「另一台又收下了，没清掉」（推断）。
- **导出到 Obsidian 的那份**（11-06 定，[§1.10 第 15 条](#110-要你看一眼)）：归那边，usuyo 删掉时不去动它。读回遇到库里没有或在回收站的 `usuyo_id`，由宿主判成 `item-deleted`（不由插件报），那份文件不读也不写，只在这次导出的问题列表里写一行，不重建文件夹；插件报的 `problems` 里的文件同样不读也不写（[协议 §4.4](../protocol/transform-export.md#44-导出的-run-与-readback)）。
- **只留文字**：先生成 `derived/thumb.jpg`，再把 `raw/` 里的图片、视频、音频和各次转录里的 `play.mp4`、`keyframes/` 移进回收站（`reason: keep-text`），`run.json` 和文字产物（`asr.vtt`、`ocr.json`、`keyframes.json`）留着，字都留着（11-11 代码：照各快照 `usuyo.fetch.json` 里的 `images` 和 `kind` 是 `video`、`audio` 的 `media` 移，加各次 `play` 转录合成的视频和各次转录 `keyframes/` 下的图片；只移解析掉符号链接以后还在条目文件夹里的，指到外面的留在原处、记日志；`keyframes` 本身是符号链接的不跟进去；字幕和 `other` 留着；算文字类的输出是 `text`、`html`、`subtitle`、`meta`、`comments`；`thumb.jpg` 还没生成）；`state: text-only`，以后只 `--want` 文字类。回收站里还在时能恢复，之后点「重新取视频」存新快照。
- **和同步**：一台删、另一台同时改了笔记，回收站赢（撤销那批例外），改动并进回收站那份，出「两边都改过的」卡片。已移除、离线的那台回来时改过它：推的时候撞上墓碑，就放进它自己的回收站再留 `trashDays`，不悄悄复活。离线那台主动收下、从回收站拿回来的不算这种：照 [§1.8](#18-同步) 以墓碑的 `revision` 为底写新版本（推断）。

<a id="18-同步"></a>
### 1.8 同步

按[调研 14 §5](../research/14-sync-like-hatoba.md#5-改动和冲突) 的四层。本机是明文，只有离开本机的加密。S1、S2 是[调研 14 §2](../research/14-sync-like-hatoba.md#2-本产品要同步什么) 的分期：S1 只同步文字，S2 媒体。

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
- **你写的那层**：front matter 按字段三方合并，列表（`keywords`、`match`、`topics`、`from`）按集合，标量只有同一字段两边都改了才出卡片；front matter 只有 usuyo 写、写法固定（[§0.6](conventions.md#06-文本和文件)），两台写出一样的字节。`notes.md`、`highlights.md`、主题页、卡片和成稿的正文按行三方合并；两边在同一处都只是插入（各加一处引用、脚注追加在末尾、各加一条划线）时两段都留，按 `deviceId` 排先后。只有改到重叠的行才算合不上，相邻的不算；两边改成一样的（插了同一段、改了同一行、标量改成同一个值）算一次，不重复、不出卡片；三方合并接受任意底本（`sync.db` 的，或导出记录里的 `base`）；导出读回时 vault 那边没有 `deviceId`，同一处都插入时排在库里那份后面（推断）。合不上的：先推上服务器的当现在这份，CAS 失败的那台负责合并，不比时钟；输掉的那份进回收站（`reason: conflict`），「两边都改过的」卡片指向它；在回收站里点恢复它，就是换上它：记成一次改动（`actor: user`），现在那份不另进回收站，撤销这次就换回来（推断）。
- **墓碑**是记录的一个版本，带 `revision`；再收下或恢复时按 CAS 以墓碑的 `revision` 为底写新版本，推墓碑也走 CAS。Worker 支不支持，同步引擎实现时核实（推断：Hatoba 的 `UPDATE … WHERE revision = ?` 对墓碑行一样适用）。
- **自动合并**：拉下来的或本机新建的条目，id 在别的条目的 `aliases` 里、或身份对上，就照 `merge-items` 并过去（`actor: sync`），不出卡片；按身份查找时先查 `aliases`。
- **记录 id** 是 `HMAC-SHA256(idKey, "<种类>:" + id)`，种类 `item` `topic` `card` `draft` `canvas` `library` `changeset` `journal` `facts` `setting` `subscription` `device` `import`；种类字符串进了云端 id，改它算 major。
- **放法**：改动记录的段、发生过的事的段、提议文件按内容寻址进 D1 的 `blobs`（超过 16 KiB 切成几行，[调研 14 §3](../research/14-sync-like-hatoba.md#3-存在哪和花多少)），记录里只放 blob id：信封上限 64 KB，一段 256 KiB 放不下。
- 别的设备的改动在那台已确认过，这台直接生效，记录带着它的 `device`。
- **库放在 iCloud、Dropbox 里时**（不开内置同步）：`devices/<deviceId>.json` 定时写 `lastSeenAt`（间隔归第 2 层），打开库时看到别的台最近写过，提示「另一台也开着」。另一台经云盘写来的改动它自己的改动记录里也有，这台怎么不重复记，归同步 feature。
- **云盘的冲突副本**：认出 iCloud 的「 2」和 Dropbox 的「conflicted copy」（名字样式归第 2 层的一张表）。题名来的文件再看 id：和原文件同一个 id 才是副本，id 不同是本机撞名加的「 2」。副本当另一边，照上面的规则和 `sync.db` 底本三方合并；没有底本，`manifest.json` 照 [§1.2.1](#121-manifestjson-v1) 最后一列；别的集合取并集，标量和正文两边不一样就算合不上、留这边。合不上的出「两边都改过的」卡片；副本和合并结果不一样就进回收站（`reason: conflict`），一样就直接从文件夹里移走。
- **和[调研 14](../research/14-sync-like-hatoba.md#5-改动和冲突) 不一样的**：`derived/` 和你写的那层合不上时都是先推上服务器的留下，不比时钟；你写的那层输掉的那份进回收站，不放 `sync.db`；文件夹不按年月分，不用[调研 14 §2 第 1 条](../research/14-sync-like-hatoba.md#2-本产品要同步什么)的改名（[§0.9 第 1 条](conventions.md#09-要你看一眼)）。

<a id="19-schema-和-pkgcontract"></a>
<a id="19-schema-和-usuyo-contract"></a>
### 1.9 schema 和 usuyo-contract

`crates/usuyo-contract` 是磁盘上各记录的 serde 类型，每个对象带 `extra`，不认识的字段原样写回；下面几条 schema 在 `crates/usuyo-contract/schema/`，样例在 `testdata/`。不写迁移程序：磁盘上没有更早格式的资料库。

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
| 1 | [§1.1](#11-目录树) 条目放在 `items/<平台>/<itemId>/` | **已定（11-06）**：按平台分一层；全放一层，上万个文件夹在 Finder 里难翻（[回答表](../answers.md)） |
| 2 | [§1.1](#11-目录树) 回收站放哪 | **不再问（11-05 方向）**：照旧放 `.usuyo/trash/`，理由是它是库的状态，见 [§1.1](#11-目录树)；[回答表](../answers.md) |
| 3 | [§1.4.1](#141-主题和主题页) 主题成员放哪 | **不再问（11-05 方向）**：成员记在成员上：条目 manifest 和卡片的 `topics`，见 [§1.4.1](#141-主题和主题页)；[回答表](../answers.md) |
| 4 | [§1.4.3](#143-引用) 引用怎么写 | **不再问（11-05 方向）**：脚注照旧，理由是通用 Markdown、来源删了也完整，见 [§1.4.3](#143-引用)；[回答表](../answers.md) |
| 5 | [§1.4.2](#142-卡片成稿记一句和划线) 记一句和划线放哪 | **不再问（11-05 方向）**：记一句是 `notes.md` 头部的 `takeaway`，划线放 `highlights.md`，见 [§1.4.2](#142-卡片成稿记一句和划线)；[回答表](../answers.md) |
| 6 | [§1.5.2](#152-提议文件) 改动提议多久没决定就作废（[A2](../PRD.md#78-agent)） | **已定（11-06）**：不作废，一直留着，不计数、不提醒（[回答表](../answers.md)） |
| 7 | [§1.4.1](#141-主题和主题页) 跟着收的命中怎么判、看哪几层字 | **已定（11-06）**：默认按关键词，相似度你打开才用，命中收下的都标「机器挑的」（[第 14 条](#110-要你看一眼)）；看标题层和内容层，画面层默认不算（[回答表](../answers.md)） |
| 8 | [§1.2.3](#123-contentmd-和各栏) `content.md` 是给「不开 usuyo 也能读」的正文副本，和导出到 vault 无关；它能不能改 | **已定（11-05）**：能看不能改，只读文件，总照原样重写；外面强行改过的那份先进回收站（[回答表](../answers.md)） |
| 9 | [§1.6](#16-改动记录) 撤销 Agent 收下的一批时，你写过东西的条目怎么办 | **已定（11-05）**：撤之前问一次也撤不撤；问不了时跳过并列出（推断）（[回答表](../answers.md)） |
| 10 | [§1.2.3](#123-contentmd-和各栏) [CLAUDE.md](../../CLAUDE.md) 原则 2 照 [PRD 原则 2](../PRD.md#4-原则) 改成「机器永不改写或删除原样」，`content.md` 算宿主生成的只读副本；原则 3 的「删掉 `.usuyo/index/`」改成「删掉索引目录」；文档地图里 SPEC 那行改成 v1 草案 | **已定（11-05）**：三处都照这样改了（[回答表](../answers.md)） |
| 11 | [§1.7](#17-删掉) 删除记录多记一个删掉的时间 `deletedAt`（[K1 A](../PRD.md#710-存储删除与联网) 说只记身份和标题） | **不再问（11-06 不留删除记录）**：删除记录去掉了，`deletedAt` 和「删掉过的」列表跟着没了；[回答表](../answers.md) |
| 12 | [§1.7](#17-删掉) 主动收下一条以前删过的怎么办 | **已定（11-06 改）**：不记删过的，在回收站里的直接拿回来，不提醒（[回答表](../answers.md)） |
| 13 | [§1.3](#13-条目的四个维度) 成稿引用了私密条目，导出时怎么办 | **先放着（11-06）**：导出到 Obsidian 和给 Agent 分开定，等有界面和具体例子再定；[§1.3](#13-条目的四个维度) 现在的写法（只换那条脚注）是暂定（[回答表](../answers.md)） |
| 14 | [§1.2.1](#121-manifestjson-v1) Agent 自己挑着收下的算不算「机器挑的」 | **已定（11-06）**：机器找的、机器挑的都算，包括按关键词找的、跟着收按关键词命中的；只有你给的具体链接和你自己平台上的收藏、列表不算（[回答表](../answers.md)） |
| 15 | [§1.7](#17-删掉) 删掉以后，导出到 Obsidian 的那份还在 vault 里 | **已定（11-06）**：不管，导出的归那边，usuyo 不去动它；读回碰到已经删掉的只在导出的问题列表里写一行（[回答表](../answers.md)） |
| 16 | [§1.7](#17-删掉) 删掉的到期或立即清除以后，你主题页里引过它的原话和标题还留着 | **已定（11-06）**：删掉到期或立即清除时照这样：原话、标题留着，链接换成「已删掉」；不再单独有「彻底忘掉」（[回答表](../answers.md)） |
| 17 | [§1.6](#16-改动记录) 撤销 Agent 收下的一批时不留删除记录，以后跟着收还可能再收进来 | **不再问（11-06 不留删除记录）**：撤销和删掉都不留记录；跟着收跳过这个订阅列过的和回收站里的，到期移除以后它没列过的照收（[§1.7](#17-删掉)）；[回答表](../answers.md) |
| 18 | [§1.6](#16-改动记录) Agent 经 MCP 能撤销哪些批：撤销新建、撤销恢复会把东西移进回收站，[原则 5](../PRD.md#4-原则) 说 MCP 不给删除 | **已定（11-05）**：能撤同一个 Agent 自己收进来、你没写过东西的，放进回收站；写过的跳过并列出；原则 5 写明这不算删除（[回答表](../answers.md)） |
| 19 | [§1.2.1](#121-manifestjson-v1) 「机器挑的」是来历，还是「你还没碰过」 | **已定（11-05）**：是来历，你碰过也一直留着（[回答表](../answers.md)） |
| 20 | [§1.7](#17-删掉) 删过的照收以后，`skipped` 只在私密时出现：Agent 和 `api` 令牌拿一条链接来试，就知道它在你库里、是私密的 | **已定（11-06）**：照这样，写明代价：只露「在库里、私密」这一点，内容、标题都不给（[回答表](../answers.md)） |
