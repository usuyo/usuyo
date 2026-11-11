<a id="003--变成文字口播图里的字画面"></a>
# 003 — 变成文字：口播、图里的字、画面

状态：in-progress（2026-11-11。这份写给实现的模型和以后查，不等用户批，见 [CLAUDE.md](../../CLAUDE.md) 开发流程）
关联：[PRD 原则](../PRD.md#4-原则) · [SPEC §1.2.2](../SPEC.md#122-raw-和-derived)、[§1.2.3](../SPEC.md#123-contentmd-和各栏) · [插件协议 §2.9](../plugin-protocol.md#29-转换插件的字段)、[§4](../plugin-protocol.md#4-转换与导出插件协议) · [调研 04](../research/04-tech-options.md) · [调研 16](../research/16-piecing-engine.md) · [feature 001](001-m0-core-loop.md) · [feature 002](002-platforms.md)

<a id="目标"></a>
## 目标

收下的东西多半是你看过的视频和长文。这条线把它们变成能搜、AI 能读、以后能拼进你自己思路里的文字和素材：说了什么、画面上写了什么、图里写了什么，都转成带位置的纯文字；图和画面本身也留着当素材。每一步都留着，指得回原样，能换一个工具重做，转出来的东西从不顶替原样。

现在做，是因为 002 已经把小红书、视频号、B 站的原样取回来了，而小红书大多是图、视频号几乎只有声音，不转就搜不到、拼不进去。

<a id="用户故事"></a>
## 用户故事

- 作为收了一堆视频的人，我想两个月后搜「水温 92」，结果写「口播 01:42」，点开从那一秒播。
- 作为收了很多小红书图文的人，我想搜到图里写的字，点开就是那张图、那一块。
- 作为不确定机器认得对不对的人，我想看到每段字是哪个工具、哪个模型转的，觉得不对就换一个再转一遍，两份并排放着。
- 作为在 B 站收视频的人，我想条目页上的视频有声音（B 站给的是分开的音轨和画面）。
- 作为不想被催的人，我不想看到「还有 312 条没转」，也不想电脑在电池上发烫。

<a id="1-一条有几面"></a>
## 1. 一条有几面

一条视频有好几面，彼此可能对不上：

| 面 | 是什么 | 从哪来 |
|---|---|---|
| 标题面 | 标题、封面、简介（作者写的字）、封面上的字 | 平台；封面上的字由机器认 |
| 说的 | 说了什么：平台给的字幕、机器转的口播 | 平台；机器 |
| 画面 | 画面本身（能播的视频、关键帧）、画面上写的字 | 原样；机器 |

图文只有标题面和图：图就是内容，图里的字和正文一样重要。

用户的要求，这里照做：

- **每一面都认出来、都摆出来。** 不判断哪一面要紧，不挑，不藏。有时只有说的有内容、画面无关；有时反过来；标题和封面常和内容不一样。判断留给人。
- **只标特别明显的事**，比如没听到人说话、画面几乎不动（[§6](#6-很明显的标记)）。标了也不藏任何东西，不改排序，不跳过下一步。
- **不删掉另一面。** 有人就是靠画面联想到别的，所以哪一面都不因为另一面「更像内容」而被丢掉或少算。
- 关键词一层可以叠在内容层上，归 004（[§9](#9-给-004-的)）。

<a id="2-范围"></a>
## 2. 范围

<a id="做"></a>
### 做

1. **合起来能播**：分开存的音轨和画面（B 站插件存 `media/video.m4s`、`media/audio.m4s`）合成一个能播的文件，放在 `derived/`；原样不动。
2. **说的转成文字**：带时间点，先中文，中英混说常见；平台已经给了字幕的，照样摆出来，机器也照样转一份，两份并排。
3. **图里的字**：图文的每张图、视频的封面、视频的关键帧（按镜头变化取），每一行字记下在哪张图、哪一块、视频的第几秒。
4. **怎么存**：每次转换是一次「转录」（run），一个文件夹，记下工具、模型、版本、参数、来源；换个工具再转是新的一次，和旧的并存（[§5](#5-存在哪怎么重做)）。
5. **条目页摆法和搜索**：几面并排的数据形状（不管样子，样子归画布那边）；搜索覆盖口播、字幕、图里的字、画面里的字（[§7](#7-条目页和搜索)）。
6. **很明显的标记**：五种，只提示（[§6](#6-很明显的标记)）。
7. **工具怎么到这台电脑上**：ffmpeg 随插件带，模型第一次用到时问一句再下（[§4.3](#43-ffmpeg-和模型怎么到这台电脑上)）。

<a id="不做"></a>
### 不做

- 摘要、关键词、向量、拼线头、多角度分析：004。这里只交出 004 要的东西（[§9](#9-给-004-的)）。
- 画面描述（看图模型写 `vision`）：默认关的那一层，以后单独加一个转换插件，照同样的转录规矩。
- 网页正文抽取（`readable.md`）：现在没有转换插件产出它，路径先不动；做的时候照 [§5](#5-存在哪怎么重做)。
- 硬字幕的专门提取（逐帧找字幕带再认）：先靠关键帧上的字；不够再加插件。
- 说话人分开、逐词时间点：以后换模型时加，格式留了位置。
- Windows、Linux 上的图里认字默认插件（PP-OCRv6）：列在工作包里，排在 macOS 之后（[§3.2](#32-图里的字)）。
- 插件沙箱（[协议 §7](../plugin-protocol.md#7-权限与隔离)）：照旧还没有。
- 视频号「录下播放的声音再转」那条路：要 `audio-capture`，归获取插件；录下来的音频进 `raw/` 以后，这里照常转。

<a id="3-用哪些工具"></a>
## 3. 用哪些工具

这台：Apple M1（`arm64`），16 GB 内存，macOS 15.6.1（24G90）。Homebrew 的 ffmpeg 9.0.1 在 `/opt/homebrew/bin`，Swift 6.2.3。

每样活一个默认，一个能换的。版本是 2026-11-11 在官方 release 页查的（[来源](#来源)）。

| 活 | 默认 | 为什么 | 能换成 |
|---|---|---|---|
| 合起来、抽音轨、取关键帧、量画面动不动 | ffmpeg 9.0.2，自己编的精简 LGPL 构建，随 `builtin/media` 插件带 | 只要拆封装、解码、几个滤镜（`scdet`、`freezedetect`、`scale`）、出 JPEG 和 WAV，LGPL 够用（[G2](../open-decisions.md#14-法律意见) 的建议 B）；体积小，版本由我们锁 | 开发时用 Homebrew 的 ffmpeg；以后可以换 PySceneDetect 0.7.3 做关键帧 |
| 说的转成文字 | sherpa-onnx 2.0.1（官方 Rust crate，静态链接）跑 SenseVoice-Small int8（2025-09-09），silero VAD v5 切段 | 中文为主训练，中英日韩粤，自带标点和数字转写；int8 包 165.8 MB；三系统都有预编译；同一套运行时还能跑 FireRedASR2、Qwen3-ASR，换模型不换插件 | 同插件里选 FireRedASR2-AED int8（838.6 MB，自报普通话 CER 2.89%，覆盖中英混说）、Qwen3-ASR-0.6B int8（878.7 MB），只要中文时可选 Paraformer-zh int8（228.3 MB）；以后另一个插件包 whisper.cpp 1.9.5（多语种）；macOS 26 以后可以加系统的 SpeechAnalyzer |
| 图里的字（macOS） | 系统 Vision（`VNRecognizeTextRequest`，revision 3，`.accurate`，`zh-Hans` + `en-US`），一个 Swift 写的插件 | 不用下载；这台实测 1080×1440 的图一张 0.4–1.0 秒；简体、繁体、粤语、英文都在列表里 | PP-OCRv6 small（经 `oar-ocr` 0.10.2 和 ONNX Runtime），Windows、Linux 的默认，macOS 上「换一个认」 |

<a id="31-说的转成文字"></a>
### 3.1 说的转成文字

| 候选 | 中文 | 这台的速度 | 要下多少 | 许可 | 三系统 |
|---|---|---|---|---|---|
| Apple SpeechAnalyzer / SpeechTranscriber | 有中文（第三方文章；官方要在运行时查 `supportedLocales`）；调研 04 记过一份自报的中文会议 CER 约 34–40% | 测不了：要 macOS 26，这台是 15.6.1 | 系统按语言下资产 | 系统 API | 只有 Apple |
| Apple SFSpeechRecognizer（旧接口） | 这台查到 `zh-CN` 可用、支持本机识别 | 没测成：从命令行程序请求语音识别授权，8 秒内没有回结果（推断：系统要 App 带用途说明，插件这种子进程拿不到） | 无 | 系统 API | 只有 Apple |
| whisper.cpp 1.9.5 | 中文不是最好，社区常说有幻觉和标点问题（调研 04） | 未实测 | tiny 75 MiB 到 large 2.9 GiB（调研 04）；large-v3-turbo 的大小没查 | MIT | 都有 |
| sherpa-onnx + Paraformer-zh int8（2025-10-07） | 中文专用，官方说带时间点（调研 04）；不出英文 | 未实测 | 228,262,632 字节；small 版（2024-03-09）77,920,048 字节 | 权重是 FunASR 模型许可（调研 04） | 都有 |
| **sherpa-onnx 2.0.1 + SenseVoice-Small int8** | 中文、粤语占优（SenseVoice 仓库的对比，数字在图里没读到）；ITN 出标点和数字 | 实测（11-11，合成的 `say` 语音，机器上同时有别的编译）：11 分 23 秒的样例，普通优先级、crate 默认线程（实际 1 线程）实时率 0.13–0.15，2 线程 0.073–0.081；`taskpolicy -b` 下 0.36–1.2，是上限不是干净的数；加载模型约 1 秒，内存峰值 450–525 MB。CER 约 13%（2024-07-17 那份；合成语音偏向模型，真人口播会更差）。**2025-09-09 那份是 WSYue 粤语微调版**（解包后 README 写明来自 `ASLP-lab/WSYue-ASR`），普通话不出标点、不做 ITN，`speech.m4a` 转成「咖F」，CER 约 18%；原版是 `sherpa-onnx-sense-voice-zh-en-ja-ko-yue-int8-2024-07-17.tar.bz2`（163,002,883 字节，sha256 `7d1efa2138a65b0b488df37f8b89e3d91a60676e416f515b952358d83dfd347e`，解包后 `model.int8.onnx` 239 MB），它转出「咖啡」「注水」、标点和数字 | 165,783,878 字节（压缩包，sha256 和 §4.3 一致；解包后约 247 MB） | 运行时 Apache-2.0；SenseVoice 代码 MIT，权重是 FunASR 模型许可（可商用，要署名、写模型名） | 都有 |
| sherpa-onnx + FireRedASR2-AED int8 | 自报普通话 CER 2.89%，覆盖方言和中英混说（10-30 的调研笔记，这次没重查） | 未实测 | 838,589,068 字节 | Apache-2.0 | 都有 |
| sherpa-onnx + Qwen3-ASR-0.6B int8 | 调研 04 记的 1.7B 自报分数都好过 Whisper-large-v3；0.6B 的没查 | 未实测 | 878,702,423 字节 | Apache-2.0 | 都有 |

选 SenseVoice 的理由：小，中文第一，sherpa-onnx 的 Rust crate 是官方的、默认静态链接、给逐 token 的时间点（`offline-recognizer-sense-voice-impl.h` 里有 `timestamps`），Rust 示例里现成有 SenseVoice、FireRedASR、Qwen3-ASR、silero VAD 四样。Apple 的两条路在这台都用不上：新接口要 macOS 26，旧接口要授权，而授权记在 App 上，插件是子进程。

速度和中文效果都还没在这台量过，所以第一个工作包就是量（[WP0](#10-工作包)），量完再定「接电时自动转」是否够快。

<a id="32-图里的字"></a>
### 3.2 图里的字

| 候选 | 中文 | 这台的速度 | 要下多少 | 许可 | 三系统 |
|---|---|---|---|---|---|
| **Apple Vision `VNRecognizeTextRequest`** | 印刷体好；框是一行一个，不到字；竖排不稳（调研 04 引的开发者论坛帖） | 实测：4 行中英混排的 1080×1440 图，第一次 1.01 秒，之后 0.42–0.85 秒；4 行全认对 | 无 | 系统 API | 只有 Apple |
| PP-OCRv6 small（ONNX，经 `oar-ocr`） | 官方内部集识别 81.3%，高过 PP-OCRv5_server 的 78.1%；手写、艺术字没公开数 | 未实测；官方在 M4 + ONNX Runtime 上每张 1.29 秒 | 未查到（ModelScope 不给大小）；medium 3450 万参数 | Apache-2.0 | 都有 |
| PP-OCRv6 medium | 83.2% | M4 上每张 5.55 秒 | 同上 | Apache-2.0 | 都有 |
| Tesseract 5 | 中文弱（调研 04） | — | — | Apache-2.0 | 都有 |

Vision 有两点要知道：它的 `boundingBox` 是 0–1、原点在左下角，存的时候换成原点在左上角（[§5.3](#53-各样东西的格式)）；它的 `confidence` 很粗，这次只出现过 0.3、0.5、1.0 几个值，不能和别的引擎比。

PP-OCRv6 这条路依赖 `ort`，它现在只有 2.0.0-rc.15，没有正式版；和「只用最新的正式版」冲突，所以排在后面，等你定（[§13](#13-风险和待定)）。

<a id="33-关键帧"></a>
### 3.3 关键帧

ffmpeg 自带的 `scdet` 按镜头变化打分（调研 04 [§4.3](../research/04-tech-options.md#43-推荐与风险关键帧与描述) 推荐它优先于 `select`），`freezedetect` 量画面停住了多久。这台的 Homebrew 构建里都有；它缺 `drawtext`，所以测试图不靠 ffmpeg 画字（[§12](#12-测试方案)）。

<a id="4-怎么跑"></a>
## 4. 怎么跑

<a id="41-四个转换插件"></a>
### 4.1 四个转换插件

都是转换插件，独立进程，照[协议 §4](../plugin-protocol.md#4-转换与导出插件协议) 在 stdio 上说 JSON-RPC。不带平台知识，所以放主仓库 `plugins/`，用 `builtin/` 前缀，像 `plugins/web` 一样随安装包带（[协议 §8.4](../plugin-protocol.md#84-三种来源)）。许可同 `plugins/web`（MIT）。

| 插件 | 目录 | 语言 | 产出 | 吃 | 模型 |
|---|---|---|---|---|---|
| `builtin/media` | `plugins/media` | Rust | `play`（合起来能播的）、`audio:pcm16k`（给转写用的中间音频，不存进资料库）、`keyframes`；另报画面停住多久、有没有音轨 | `media:video`、`media:audio` | 无；资产里带 `ffmpeg`、`ffprobe` |
| `builtin/asr-sherpa` | `plugins/asr-sherpa` | Rust（`sherpa-onnx` 2.0.1） | `asr` | `audio:pcm16k` | SenseVoice（第一次用到时下）；silero VAD v5（2.3 MB，打进资产） |
| `builtin/ocr-vision` | `plugins/ocr-vision` | Swift 6.2（Swift Package） | `ocr` | `image`、`derived:keyframes` | 无 |
| `builtin/ocr-ppocr`（后做） | `plugins/ocr-ppocr` | Rust（`oar-ocr`） | `ocr` | 同上 | PP-OCRv6 small |

`ocr-vision` 用 Swift：Vision 在 Rust 里只能经 `objc2`，要写 `unsafe`，仓库不许（[CLAUDE.md](../../CLAUDE.md) 代码约定）。插件是独立进程，语言不影响宿主。它自己带一个一百来行的 JSON-RPC 循环。

一条视频的链（宿主编排，插件之间不知道彼此）：

```
raw 里的画面 + 音轨 ──media:play──▶ play.mp4（只在两者分开时）
raw 里的音（或画面里的音轨）──media:audio:pcm16k──▶ 16 kHz 单声道 WAV（按机器缓存）──asr──▶ asr.vtt
raw 里的画面 ──media:keyframes──▶ keyframes/*.jpg ──ocr──▶ 画面里的字
raw 里的图、封面 ──ocr──▶ 图里的字
平台给的字幕 ──（不转）──▶ 直接摆出来、进索引
```

**哪两个文件是一段**：宿主按内容配，不按平台。同一个快照的 `media[]` 里恰好一个 `video`、一个 `audio`，`ffprobe` 看画面文件没有音轨、音频文件没有画面，两者时长差不超过 1 秒或 2%（取大的），就配成一对，交给 `play`。配不上的不合，各自照常转。协议 [§13 第 38 项](../plugin-protocol.md#13-待定) 加一个 `group` 字段以后，有 `group` 先按它（[§8.2](#82-要写回-spec--协议的)）。画面文件本来就带音轨的（小红书、视频号）不需要 `play`。

<a id="42-宿主要加的"></a>
### 4.2 宿主要加的

**现在有的**（照代码）：

- `crates/usuyo-plugin` 能解析转换插件的清单（`consumes`、`produces`、`provider`、`models`，[§3.13](../plugin-protocol.md#313-宿主和-plugin-kit-现在做到的11-11)），`discover` 把它们放进 `Discovery.other`，没人调。
- 一次一进程的调用器（`host/process.rs`）：自己的进程组、三种计时、先 SIGTERM 再 SIGKILL；输出目录检查（`host/outdir.rs`）；运行时只按 `PATH` 加几个 Homebrew 目录找（`find_runtime`）。
- `jobs.db` 只有取内容的任务，按 `item_id` 一条，一次跑一个（`crates/usuyo-core/src/jobs.rs`）。
- `resolve_layer` 已经认 `asr`（`derived/asr.vtt`）、`ocr/<imageKey>`（`derived/ocr/<imageKey>.json`）、`video`、`audio`（`raw/` 里的媒体）（`crates/usuyo-library/src/item.rs`）。
- 索引只建了 `text`、`comments`、`takeaway`、`notes` 四层（`crates/usuyo-core/src/index_doc.rs`）；搜索把认不出的层都当 `text`。
- 前端的 `Facet` 已有 `asr`（带 `by: machine | platform`）、`ocr`、`video.frames`、`remark`，但按 `layer` 当 key，同一层出两份会撞（`FacetColumn.tsx`）。

**要加的**：

| 加在哪 | 什么 |
|---|---|
| `crates/usuyo-plugin`（协议类型） | `protocol/transform.rs`：`describe`、`health`、`run`、`cancel`、`progress` 的消息类型；清单词表改成 [§8.2](#82-要写回-spec--协议的) 的样子；`models[]` 加 `format`、`mirrors`、`license` |
| `crates/usuyo-plugin`（宿主） | `host/transform.rs`：常驻进程。启动、`describe` 核对、`health`；同一进程一次只发一个 `run`；`cancel`；关 stdin 等 5 秒，再 SIGTERM、SIGKILL 进程组；闲 120 秒自己关掉，放掉模型占的内存。暂存目录照[协议 §7.1](../plugin-protocol.md#71-常驻进程的路径)：输入先硬链接进 `in/<callId>/`，跨文件系统再复制；`out/<callId>/` 照输出目录规则查。`progress` 只用来判断卡住（120 秒没有新的算卡住），不往外报 |
| `crates/usuyo-plugin-kit` | `transform` 模块：Rust 插件用的 JSON-RPC 循环，作者只写 `run` 处理函数（[协议 §10](../plugin-protocol.md#10-契约测试与-plugin-kit) 说 SDK 管这个） |
| `crates/usuyo-core` | `models.rs`：模型下载和校验（[§4.3](#43-ffmpeg-和模型怎么到这台电脑上)）。`transform.rs`：排任务、配对、编排一条链、写转录、事件。`jobs.rs` 加 `transform_jobs` 表（[§4.4](#44-什么时候转)） |
| `crates/usuyo-library` | 转录的写入、列出、按层找（[§5](#5-存在哪怎么重做)）；「只留文字」也移走转录里的二进制产物 |
| `crates/usuyo-contract` | 转录记录 `run.json` 的类型、schema `urn:usuyo:schema:run:1`、`testdata/` 样例 |
| `crates/usuyo-index` | 新层 `asr`、`ocr`、`frames`；每段多存一个 `source`（转录 id 或平台字幕）；索引版本加一，下次启动整个重建 |
| `crates/usuyo-api`、`endpoints.ts`、`crates/usuyo-cli` | [§8.1](#81-接口) 的接口和命令 |

进程优先级：自动跑的转换在 macOS 上用 `/usr/sbin/taskpolicy -b` 起（后台 QoS，只上能效核、限磁盘），Linux 用 `nice -n 10` 加 `ionice -c 3`，Windows 用 `BELOW_NORMAL_PRIORITY_CLASS`（`CommandExt::creation_flags`）。你点名的（打开了这一条、点了「再转一次」）照普通优先级跑。

<a id="43-ffmpeg-和模型怎么到这台电脑上"></a>
### 4.3 ffmpeg 和模型怎么到这台电脑上

**ffmpeg** 随 `builtin/media` 的资产走，不另下载、不从 `PATH` 找（[协议 §2.8](../plugin-protocol.md#28-安装与启动) 说插件自带的第三方程序放进自己的资产、锁版本）。

- 插件仓库 CI 从 ffmpeg.org 的 9.0.2 源码包编（锁源码包的 sha256），`--disable-gpl --disable-nonfree --disable-everything`，只打开要的：mov/mp4/matroska/webm/flv/mpegts/aac/mp3/wav 的拆封装；mp4、wav、image2、`null` 封装；h264、hevc、vp9、av1（libdav1d，BSD）、aac、mp3、opus、flac 的解码器和对应的 parser；`aac_adtstoasc`、`extract_extradata` 这类拷流要的 bsf；mjpeg、pcm_s16le 编码；`scdet`、`freezedetect`、`scale`、`aresample`、`fps`、`format`、`null`、`anull` 滤镜；`file`、`pipe` 协议；静态链接，每个平台一份。编译脚本和 LGPL 声明放 `plugins/media/ffmpeg/`，和资产一起发。体积未实测。
- ADR-003 [§8](../decisions/ADR-003-plugins.md#8-gpl-工具) 写的是「第三方二进制只从上游官方 release 下载」。ffmpeg 官方不发二进制，所以从官方源码编是最接近的做法，要你确认（[§13](#13-风险和待定)）。现成的备选：martin-riedl.de 有签名的 macOS arm64 9.0.2 构建，带 sha256，但许可没写（多半带 x264，是 GPL）。
- 开发时 `just dev-plugins` 把 `plugins/media` 编好，资产里的 `ffmpeg` 换成指向 Homebrew 那份的链接；只在开发机上。

**模型**第一次用到时问一句再下（[T2](../open-decisions.md#8-转换与-ai) 的建议 B）：

- 不在安装时下，不自己悄悄下。第一条要转口播的条目在那一面写：「转口播要先下载 SenseVoice 模型，约 166 MB，只下一次。」旁边一个「下载」；转换插件页也有同一个按钮，写模型名、大小、许可。下好以后，等着的条目照常转。没点就一直等着，不提醒第二次。
- 大于 1 GiB 的（以后的 FireRedASR2 不到，Qwen3-ASR 不到，都写大小）同样只点了才下。换模型（插件设置里选另一个）也是先问。
- 放按机器目录的 `models/<owner>/<repo>/<模型名>/`（[SPEC §1.1](../SPEC.md#11-目录树) 的 `models/`）。宿主下，插件不联网。
- 下载：先试 `url`，不行按顺序试 `mirrors`（同一个 sha256）；支持 Range 续传；连接 15 秒，60 秒没收到字节算断；下完按整个文件的 sha256 校验，对不上整个丢掉、报原因。压缩包（sherpa 的模型是 `.tar.bz2`）解到同目录下的临时文件夹，再改名进去，同时写 `model.json`：`{name, sha256, size, license, files: [{path, size, sha256}]}`。插件 `describe` 时宿主核对 `files`。
- SenseVoice 的条目，写进插件清单：

```yaml
models:
  - name: sense-voice-int8-2025-09-09
    url: https://github.com/k2-fsa/sherpa-onnx/releases/download/asr-models/sherpa-onnx-sense-voice-zh-en-ja-ko-yue-int8-2025-09-09.tar.bz2
    sha256: 7305f7905bfcf77fa0b39388a313f3da35c68d971661a65475b56fb2162c8e63
    size: 165783878
    format: tar.bz2
    license: FunASR Model Open Source License（可商用，署名并写明模型名）
    setting: model
    value: sense-voice
```

sha256 取自 GitHub 给这个资产的 `digest`，实现时下载一次再核。

<a id="44-什么时候转"></a>
### 4.4 什么时候转

自动和点名混着来：

- **自动**：一个快照落进资料库（收下、再取一次、导入、交进来的）以后，宿主看它有什么，排上要做的：有视频就排 `play`（配得上时）、`asr`、`keyframes`，关键帧那次落地以后排 `frames`（认关键帧上的字）；有图（含封面）就排 `ocr`。平台给了字幕也照样排 `asr`，两份并排（为什么，见 [§13](#13-风险和待定)「平台给了字幕也照样转」）。设置里 `transforms.auto: false` 时不自动排，只在点名时转。
- **点名**：打开一条、还有面没转出来，它的任务挪到最前、照普通优先级跑，在电池上也跑；点「再转一次」「换一个转」同样。
- **在哪台转**：照 [SPEC §1.2.1](../SPEC.md#121-manifestjson-v1)，`origins` 里最早那一项的那台。别的设备不自动转，等同步来的文字。

**限着，免得一次导入 500 条把电脑烤热**：

- 转换一次只跑一个，整台机器算；和取内容是两条道，各自一次一个，可以同时各跑一个（取内容主要等网络）。
- 自动的只在接着电源时跑；拔了电，跑着的那个跑完，后面的等着，接上再接着。读电源状态用 `starship-battery` crate（三系统都有）；读不出来记一次日志，当接着电。
- 自动的用后台优先级（[§4.2](#42-宿主要加的)）。
- 先后：点名的（0）> 你手动收下的（1）> 导入、跟着收、交进来的（2）；同一档按排进来的先后。
- 失败照取内容的退避（`RetryPolicy`，30 秒乘 4 的次方，最多 6 次）。缺模型（`dependency-missing`，`needs: model:<名字>`）不算失败、不算次数，等模型。条目进回收站或只留文字时，删掉它的转换任务（只留文字只删要媒体的）。
- 重启后 `running` 的放回 `queued`，同取内容。
- 一条一个 `job` 只有一行：点「再转一次」时那一行还排着，就换上你给的插件、参数，挪到最前；正在跑，就回 `conflict`（界面在转的时候不给点）。
- 你打开了一条、它的任务正在后台优先级下跑：宿主对那个进程 `taskpolicy -B -p <pid>`，让它照普通优先级跑完。

`jobs.db` 加一张表：

```sql
CREATE TABLE transform_jobs (
    item_id    TEXT NOT NULL,
    job        TEXT NOT NULL,      -- play | asr | keyframes | ocr | frames
    plugin     TEXT,               -- 点名换插件时有；空是用默认
    options    TEXT,               -- 点名时的参数 JSON
    state      TEXT NOT NULL,      -- queued | running | waiting | failed
    wait       TEXT,               -- waiting 时：power | model:<名字>
    priority   INTEGER NOT NULL,
    due_at     INTEGER NOT NULL,
    attempts   INTEGER NOT NULL DEFAULT 0,
    error_code TEXT,
    error_hint TEXT,
    PRIMARY KEY (item_id, job)
);
PRAGMA user_version = 2;
```

**你看到的**：没有计数，没有列表，没有全局的转着的标志。只在条目页的那一面写一句实话：

| 状态 | 那一面写 |
|---|---|
| 排着、接着电 | 「口播还没转出来，会慢慢转。」 |
| 排着、在电池上 | 「口播还没转出来，接上电源时会转。」（你打开了这条，它就直接转，见上） |
| 在转 | 「在转口播。」不写百分比 |
| 等模型 | 「转口播要先下载 SenseVoice 模型，约 166 MB，只下一次。」加「下载」 |
| 没转出来 | 「口播没转出来：<原因>」加「再试」 |
| 转好了 | 那一面的内容，头上写来源（「机器转的 · SenseVoice」） |

画面、图里的字同理。没有「未转」「待处理」这类字眼，也不在地图、列表上标。

<a id="5-存在哪怎么重做"></a>
## 5. 存在哪、怎么重做

<a id="51-一次转换一个文件夹"></a>
### 5.1 一次转换一个文件夹

SPEC 现在是一层一个文件（`derived/asr.vtt` 加旁边的 `asr.vtt.meta.json`），换个工具再转只能盖掉。调研 16 [§2](../research/16-piecing-engine.md#2-每一步怎么指得回去怎么重做) 提议「现在那份放原处、旧的挪进 `derived/runs/asr/<runId>/`」，要挪文件，还要一个「现在是哪份」的概念。这里改成更少的一条规矩：**每次转换写一个新文件夹，写好以后不再改；哪份在前面，按时间算，不存。**

```
items/bilibili/bilibili-…/
  raw/<snapshotId>/   media/video.m4s  media/audio.m4s  media/subtitle.zh.vtt  usuyo.fetch.json …
  derived/
    runs/
      <runId>/        run.json  play.mp4                       ← media 合起来的
      <runId>/        run.json  asr.vtt                        ← SenseVoice 转的
      <runId>/        run.json  asr.vtt                        ← 后来换 FireRedASR2 再转的，并排
      <runId>/        run.json  keyframes.json  keyframes/00000500.jpg …
      <runId>/        run.json  ocr.json                       ← 关键帧上的字
      <runId>/        run.json  ocr.json                       ← 封面上的字
    thumb.jpg  fingerprints.json                               ← 宿主产出，照旧
```

- `runId` 是 UUIDv7，按字符串排就是按时间排（[SPEC §0.4](../SPEC.md#04-id)）。
- 宿主先在 `.usuyo/tmp/` 拼好整个文件夹（插件的产出、宿主算的哈希、`run.json`），再改名进 `derived/runs/`。写进去以后不改；要改就是再转一次。
- **一层现在给谁看**：同一层（同一个 `job`、同样的输入）有几次，最新的一次排第一，别的照样摆着、能切换（[§7.1](#71-条目页的数据形状)）。引用链接按秒、按 `imageKey` 找回，不认 `runId`，所以换了一次转录，你写过的引用不断。
- **做好没有**：一个任务算做好，要有一次这个 `job` 的转录，它的 `inputs` 的 sha256 盖住了现在要的那些输入。哪个插件、哪个版本做的都算。插件升级不自动重转（不让你的电脑莫名其妙地忙），想要就点「再转一次」。`empty: true` 的转录也算做好（没有人声就是结果）。
- **中间产物不进资料库**：给转写用的 16 kHz WAV 放按机器缓存 `cache/transform/<itemId>/<runId>/`，这条链跑完就删。它的来历记在用它的那次转录的 `inputs[].via` 里，链照样清楚。
- **同步**：`run.json` 和文字产物（`asr.vtt`、`ocr.json`、`keyframes.json`）跟着文字同步；二进制产物（`play.mp4`、`keyframes/*.jpg`）和媒体一样，以后媒体同步时再说，现在不同步。另一台看得到关键帧上的字和秒数，看不到那张帧。
- **只留文字**：除了现有的 `raw/` 里的媒体，再把各次转录里的 `play.mp4`、`keyframes/` 移进回收站，`run.json` 和文字产物留着；「把移走的图和视频拿回来」照样拿回。
- **删掉一次转录**：第一版不做。转录很小，多了也不碍事；真要做，进回收站，照 [SPEC §1.7](../SPEC.md#17-删掉)。

<a id="52-转录记录-runjson"></a>
### 5.2 转录记录 run.json

能重做靠这一个文件：照它写的插件、工具、模型、参数和输入，能再跑出同样的东西，或换掉其中一样再跑一次。

```json
{
  "id": "019a6f3e-8c1d-7b2a-9e44-5d0c1a2b3c4d",
  "job": "asr",
  "by": "builtin/asr-sherpa",
  "version": "0.1.0",
  "tool": {"name": "sherpa-onnx", "version": "2.0.1"},
  "model": {"name": "sense-voice-int8-2025-09-09", "sha256": "7305f7905bfcf77fa0b39388a313f3da35c68d971661a65475b56fb2162c8e63"},
  "options": {"language": "auto", "itn": true, "vad": "silero-vad-v5", "vadThreshold": 0.5, "maxCueSec": 15},
  "at": "2026-11-12T03:00:01.123Z",
  "device": "019a6f00-0000-7000-8000-000000000001",
  "trigger": "auto",
  "redoOf": null,
  "inputs": [
    {
      "path": "raw/019a6f2b-…/media/audio.m4s",
      "sha256": "…",
      "snapshot": "019a6f2b-…",
      "via": [{"by": "builtin/media", "version": "0.1.0", "op": "audio:pcm16k", "tool": {"name": "ffmpeg", "version": "9.0.2"}, "sha256": "…"}]
    }
  ],
  "outputs": [{"path": "asr.vtt", "kind": "asr", "size": 2048, "sha256": "…", "lang": "zh"}],
  "empty": false,
  "reason": null,
  "marks": []
}
```

| 字段 | 取值 |
|---|---|
| `id` | UUIDv7，等于文件夹名 |
| `job` | `play` `asr` `keyframes` `ocr`（图文的图、封面）`frames`（关键帧上的字）；以后 `readable` `vision` `summary` `embeddings` 照这个形状加 |
| `by`、`version` | 插件 id 和版本；宿主自己判出来的写 `host` 和内核版本（见下） |
| `tool` | 插件里干活的那个程序和版本，插件在 `run` 的结果里报；宿主不猜 |
| `model` | 用了模型时：名字和宿主核过的 sha256；没有为 `null` |
| `options` | 这次实际用的参数，含插件补上的默认值（插件在结果里回显），重做时原样交回 |
| `at`、`device` | 什么时候、哪台转的 |
| `trigger` | `auto`、`open`（你打开了这条）、`redo`（你点的） |
| `redoOf` | 点「再转一次」「换一个转」时，指当时眼前那一份；不然 `null`。只是来历，不改谁在前 |
| `inputs[]` | 相对条目文件夹的路径、sha256、所在快照；`frames` 的输入是 `derived/runs/<runId>/keyframes/<文件>`；`via` 是中间转换的每一步：谁、哪个版本、什么操作、什么工具、产物的 sha256 |
| `outputs[]` | 文件夹里的产物：路径、种类、大小、sha256；`asr` 另有 `lang` |
| `empty`、`reason` | 没有可产出的东西时 `true`，`reason` 用协议的词（`no-speech`、`no-audio-track`、`no-text`），这时 `outputs` 为空 |
| `marks[]` | 很明显的标记，见 [§6](#6-很明显的标记) |

`no-audio-track` 不经转写插件：宿主先让 `builtin/media` 做一次 `probe`（`run` 的 `want: ["probe"]`，只回流的信息，不出文件），看不到音频流，就自己写一次 `job: asr` 的转录：`by: host`，`inputs` 是那个画面文件，`via` 记那次 `probe`（`builtin/media`、`ffprobe` 的版本），`empty: true`，`reason: no-audio-track`。

宿主写 `run.json`，插件不写（同现在的 meta）。它取代 SPEC 里的 `<文件>.meta.json`（字段是旧 meta 的超集），要写回 SPEC（[§8.2](#82-要写回-spec--协议的)）。

<a id="53-各样东西的格式"></a>
### 5.3 各样东西的格式

坐标统一：**时间**是秒，最多 3 位小数（[SPEC §0.5](../SPEC.md#05-时间)）；**图上的框**是 `[x, y, w, h]`，0–1，按图的宽高算，原点在左上角。引用链接里换成百分数（`#xywh=percent:…`，[SPEC §0.4](../SPEC.md#04-id)）。

- **`asr.vtt`**：WebVTT，UTF-8。一条 cue 一个 VAD 段，长于 `maxCueSec`（15 秒）的按 token 时间点在标点处切开。cue 的字是纯文字，去掉 SenseVoice 的 `<|zh|><|NEUTRAL|><|Speech|><|withitn|>` 这类标签；开头一行 `NOTE` 写语言。一个人声段都没有时不写文件，`empty: true`。
- **平台字幕**：`raw/<snapshotId>/` 里 `media[]` 的 `kind: subtitle` 那些 `.vtt`，原样读，不转。语言取那一项的 `lang`，没有就从文件名 `subtitle.<lang>.vtt` 取。
- **`play.mp4`**：`ffmpeg -i <画面> -i <音轨> -map 0:v:0 -map 1:a:0 -c copy -movflags +faststart`。这台用合成的 h264 画面加 aac 音轨试过，出来的文件两条流都在。不加 `-shortest`：配对时已经限住了时长差，留着整段声音。不重编码；画面是 AV1 而这台解不了时（M1 没有 AV1 硬解，系统 webview 一般播不了），合起来也播不了，这是获取插件选编码的事，条目页照实说「这台播不了这种视频」。
- **`keyframes.json`** 和 `keyframes/<毫秒，8 位补零>.jpg`：

```json
{"snapshot": "019a6f2b-…", "durationSec": 63.2, "frames": [{"sec": 0.5, "path": "keyframes/00000500.jpg", "sha256": "…", "width": 960, "height": 540, "scene": null}, {"sec": 12.04, "path": "keyframes/00012040.jpg", "sha256": "…", "width": 960, "height": 540, "scene": 14.2}], "frozenSec": 2.1}
```

  取法，两遍：第一遍低清扫一遍（`fps=4`、`scale=-2:360`、`scdet=threshold=10`、`freezedetect=n=0.003:d=2`），拿到镜头变化的时间和停住的区间；`freezedetect` 停到片尾时不打 `freeze_end`（这台试过），按停到片尾算。选帧：总有 0.5 秒那一帧；镜头变化处各一帧，相邻至少隔 2 秒，每分钟最多 6 帧，一条最多 60 帧；选出来少于 `min(3, 时长/10 秒)` 帧时按时长均分补齐。第二遍逐个时间点 `-ss <t> -frames:v 1`，长边缩到不超过 960，`-q:v 4` 出 JPEG。这些参数都进 `options`。
- **`ocr.json`**：

```json
{"images": [{"input": 0, "imageKey": "3f2a9c0d1e4b5a67", "width": 1080, "height": 1440, "lines": [{"text": "水温 92°C，粉水比1:15", "bbox": [0.0736, 0.2542, 0.5504, 0.0542], "confidence": 0.5}]}, {"input": 1, "frameSec": 12.04, "width": 960, "height": 540, "lines": []}]}
```

  `input` 是 `run.json` 里 `inputs` 的下标。图文和封面的图带 `imageKey`（[SPEC §1.2.2](../SPEC.md#122-raw-和-derived)），关键帧带 `frameSec`。`confidence` 是引擎自己的分，不跨引擎比。一次 `ocr` 转录可以盖几张图；快照新来了图，只为没盖到的那些再排一次。

<a id="6-很明显的标记"></a>
## 6. 很明显的标记

只有五种，都是量出来的，不是猜的。每种写进那次转录的 `marks`（或由宿主从原样算），在对应那一面的头上显示一句（`Facet.remark`）。**标了什么都不藏**：那一面照样摆出来，哪怕是空的；不改排序，不挡下一步，不进地图，不进搜索的权重。

| 码 | 界面写 | 怎么判 | 记在 |
|---|---|---|---|
| `no-audio-track` | 没有声音轨 | `ffprobe` 看不到音频流 | `asr` 转录，`empty: true` |
| `no-speech` | 没听到人说话 | silero VAD（阈值 0.5）判出的人声加起来不到 1 秒，或不到时长的 2% | `asr` 转录，`empty: true`；`marks` 带 `speechSec`、`durationSec` |
| `static-picture` | 画面几乎不动 | `freezedetect` 停住的时间加起来到时长的 90% | `keyframes` 转录；带 `frozenSec`、`durationSec` |
| `no-text` | 没认出字 | 这次 `ocr` 或 `frames` 的所有图都没有一行 `confidence ≥ 0.3` | 那次转录，`empty: true` |
| `duplicate-images` | 有几张图是同一张 | 同一条的图里 `imageKey` 相同（字节一样） | 宿主从 `raw/` 算，不进转录 |

明确**不做**的标记（都要判断，留给人）：标题和内容对不上、封面不是视频里的画面、画面和口播无关、只有音乐、广告、画质差、字幕和口播说的不一样。

<a id="7-条目页和搜索"></a>
## 7. 条目页和搜索

<a id="71-条目页的数据形状"></a>
### 7.1 条目页的数据形状

样子归画布那边，这里只定数据。`ItemView.facets` 改成一面可以有好几份，每份带来源：

```ts
/** 一份是从哪来的。 */
export interface FacetSource {
  by: "platform" | "machine";
  /** 机器的：工具和模型的显示名，例如「SenseVoice」「Vision」。 */
  tool?: string;
  runId?: string;
  /** RFC 3339。 */
  at?: string;
  /** 平台字幕、转写的语言。 */
  lang?: string;
}

export type Facet =
  | { layer: "asr"; key: string; source: FacetSource; segments: Segment[]; remark?: string }
  | { layer: "ocr"; key: string; source: FacetSource; segments: Segment[]; remark?: string }
  | { layer: "frames"; key: string; source: FacetSource; segments: Segment[]; remark?: string }
  | { layer: "text"; key: string; segments: Segment[]; remark?: string }
  | { layer: "image"; key: string; images: ImageRef[]; remark?: string }
  | { layer: "video"; key: string; snapshotId: string; durationSec: number; url?: string; frames: Frame[]; cover?: ImageRef; quality?: string; remark?: string }
  | { layer: "comments"; key: string; comments: Comment[]; remark?: string };

export type Position =
  | { kind: "time"; sec: number; endSec?: number; snapshotId: string; xywh?: [number, number, number, number]; frameUrl?: string }
  | { kind: "image"; imageKey: string; xywh?: [number, number, number, number] };

/** 还没转出来的面，只在条目页那一面写一句。 */
export interface Pending {
  layer: "asr" | "ocr" | "frames" | "video";
  state: "later" | "on-power" | "now" | "needs-model" | "failed";
  model?: { plugin: string; name: string; size: number; license: string };
  hint?: string;
}
// Item 多一个 pending: Pending[]
```

- `key` 在一条里唯一：`asr:platform:<snapshotId>:<lang>`、`asr:<runId>`、`ocr:<runId>`、`frames:<runId>`（`frames` 的 `<runId>` 是那次 `frames` 转录）、`video:<snapshotId>`；`FacetColumn` 改用它当 React 的 key。旧的 `asr.by` 去掉，看 `source.by`（开发期，不留兼容）。
- 同一层有几份（平台字幕和机器转的，或两次机器转的），都给，按 `source.at` 新的在前；平台的排在机器的前面。前端怎么并排、怎么切换归画布那边，但不许只给一份、把别的藏到设置里。
- `frames` 是新的一面：关键帧上的字。连着几帧一模一样的一行合成一段，`endSec` 写到最后出现的那一帧，这是压缩，不是挑。
- `video.url` 有合起来的 `play.mp4` 时指它，没有指原样的画面；`video.frames` 来自最新一次 `keyframes`，每帧的地址是 `/media/<itemId>/frame/<runId>/<毫秒>`。
- `remark` 由宿主照 [§6](#6-很明显的标记) 的表写中文。
- 每一面头上能「再转一次」，有别的插件能做这一层时能「换一个转」（[§8.1](#81-接口)）。

<a id="72-搜索"></a>
### 7.2 搜索

搜索框已经写着「搜标题、口播、图里的字」。索引照 `index_doc` 一个函数建，`reindex` 以后不变：

| 索引的层 | 收什么 | 段的定位 | 搜到时写 |
|---|---|---|---|
| `asr` | 每次机器转录的每条 cue；每份平台字幕的每条 cue | `t=<起>,<止>`；`source` 是 `runId` 或 `sub:<snapshotId>:<文件名>` | 「口播 01:42」 |
| `ocr` | 图文的图、封面上的每一行 | `img=<imageKey>&xywh=<…>` | 「图 3 里的字」「封面上的字」 |
| `frames` | 关键帧上的每一行（合并后的段） | `t=<秒>&xywh=<…>` | 「画面 00:12 的字」 |

- 所有转录都进索引，不只最新的：搜到同一条的几段，按条目收成一条（现在就是 `segments_per_item: 1`），先给最新那次的段。
- `engine.search` 把 `asr`、`ocr`、`frames` 映射成各自的 `HitLayer`，从定位串回 `SearchHit.position`，点开从那一秒、那一张、那一块打开。
- 私密条目照现在的规则。拼音、二字词照现有的分词。

<a id="8-接口变更"></a>
## 8. 接口变更

<a id="81-接口"></a>
### 8.1 接口

先改 `apps/desktop/web/src/api/endpoints.ts`（[SPEC 本机接口](../SPEC.md#本机接口第-2-层11-11-照代码) 说它是唯一的契约文件），Rust 照它做，样例库跟着补。

| 方法与路径 | 做什么 |
|---|---|
| `GET /items/{id}` | 多了 `pending`；`facets` 照 [§7.1](#71-条目页的数据形状) |
| `POST /items/{id}/transform` | `{jobs: "pending"}`：把这条还没转的挪到最前（条目页打开时调，不记别的）；`{job, plugin?, options?}`：再转一次或换一个转，回 `202 {queued: true}`；这个 `job` 正在跑回 `conflict`；`plugin` 不能做这个 `job` 回 `invalid-args` |
| `GET /transforms` | 装了哪些转换插件、各做哪些 `job`、`health`、模型：`{name, size, license, state: absent \| downloading \| present \| failed}` |
| `POST /transforms/{owner}/{repo}/models/{name}/download` | 下这个模型；已经有回 `200` |
| `POST /transforms/{owner}/{repo}/models/{name}/remove` | 删掉这个模型（按机器目录里的，不是资料库）；之后要它的任务等模型 |
| `GET /media/{id}/frame/{runId}/{ms}` | 一张关键帧；`/media/{id}/video` 有 `play.mp4` 时给它 |
| 事件 | 一次转录落地发 `item-updated`；模型状态变了发 `transforms-updated` |

命令行：`usuyo transform <id> [--job play|asr|keyframes|ocr|frames] [--plugin <id>] [--option <键>=<值>]… [--wait]`（`--option` 进 `options`；`--wait` 等这次落地，打印 `runId`，等的是模型时直接退出并说缺哪个模型；没给 `--job` 就是排上这条所有没转的）；`usuyo models [list | download <plugin> <name> | remove <plugin> <name>]`。都经守护进程，和 `add` 一样先握手。

<a id="82-要写回-spec--协议的"></a>
### 8.2 要写回 SPEC / 协议的

这份不改那两个文件；实现时一起改，改完在这里打勾。13 条都在 11-11 随 WP1、WP2 写回了。顺手多写的几处：协议 [§3.3](../plugin-protocol.md#33-信封与错误码) 的 `needs` 加前缀 `model:`（[§4.4](#44-什么时候转) 要它）；[§4.2](../plugin-protocol.md#42-转换的-run) 的结果加 `probe[]`（`want: [probe]` 回的流信息放这里，原来没有字段）；SPEC [§1.8](../SPEC.md#18-同步) 的同步表、[§1.4.2](../SPEC.md#142-卡片成稿记一句和划线) 里指向旧 meta 的一句跟着改；`keyframes.json`、`ocr.json` 也各有 schema（`urn:usuyo:schema:keyframes:1`、`…:ocr:1`）。

**SPEC**

1. [x] [§1.2](../SPEC.md#12-条目文件夹) 的目录例子：`derived/` 下换成 `runs/<runId>/…`，`asr.vtt`、`ocr/`、`keyframes/`、`audio/`、`<文件>.meta.json` 从 `derived/` 根下去掉；`thumb.jpg`、`fingerprints.json` 不变。
2. [x] [§1.2.2](../SPEC.md#122-raw-和-derived)：写进 [§5](#5-存在哪怎么重做) 的规矩（一次一个文件夹、写了不改、谁在前按时间、做好没有怎么判、中间产物在按机器缓存、哪些同步）和 `run.json`（[§5.2](#52-转录记录-runjson)）；`ocr.json`、`keyframes.json`、`asr.vtt` 的格式（`frames` 也写 `ocr.json`）（[§5.3](#53-各样东西的格式)）；框是 0–1、原点左上。
3. [x] [§1.2.3](../SPEC.md#123-contentmd-和各栏) 的表：口播 = 平台字幕 + `asr` 转录；图里的字 = `ocr` 转录；加一行「画面里的字」（`frames`，链接里的层是 `video`，带 `#t=` 和 `xywh`）；画面 = `play` 或原样画面，加关键帧。
4. [x] [§0.4](../SPEC.md#04-id) 的引用：关键帧上的字写 `usuyo://item/<id>/video@<s>#t=12.04&xywh=percent:7.4,25.4,55,5.4`（Media Fragments 的时间和空间合用 `&`）。
5. [x] [§1.1](../SPEC.md#11-目录树)：按机器目录的 `models/<owner>/<repo>/<名字>/`（含 `model.json`）、缓存的 `cache/transform/`；`jobs.db` 多 `transform_jobs`；按机器的 `settings.json` 多 `transforms.auto`（默认 `true`）。
6. [x] [§1.7](../SPEC.md#17-删掉) 和只留文字：转录里的 `play.mp4`、`keyframes/` 跟着媒体走。
7. [x] [§1.9](../SPEC.md#19-schema-和-usuyo-contract)：加 schema `urn:usuyo:schema:run:1`。

**插件协议**

1. [x] [§2.9](../plugin-protocol.md#29-转换插件的字段) `produces` 不再写路径，写产物种类：`play`、`audio:pcm16k`（中间产物）、`probe`（只回信息、不出文件）、`asr`、`keyframes`、`ocr`，以后 `readable`、`vision`、`summary`、`embeddings`；路径由宿主定在 `derived/runs/<runId>/` 下。`consumes` 加 `audio:pcm16k`。
2. [x] [§2.9](../plugin-protocol.md#29-转换插件的字段) `models[]` 加 `format`（`file`、`tar.bz2`、`tar.gz`、`zip`，压缩包由宿主解到 `${MODELS}/<名字>/`）、`mirrors`（同一个 sha256 的别的地址）、`license`（下载前给你看）；`size` 改必填。
3. [x] [§4.2](../plugin-protocol.md#42-转换的-run) `run` 的参数加 `want`（这次要哪几种产出，一个插件能做几种时用）；`inputs[]` 加 `snapshot`、`imageKey`、`frameSec`、`role`（`video`、`audio`）；结果加 `tool {name, version}`、`options`（实际用的参数，含默认值）、`marks[]`、每个输出的 `kind`；宿主照它写 `run.json`，不再写 `<文件>.meta.json`。`empty` 的 `reason` 词表：`no-speech`、`no-audio-track`、`no-text`。
4. [x] [§4.1](../plugin-protocol.md#41-分帧与生命周期)：宿主可以用后台优先级起转换进程；闲了可以关掉；`progress` 只用来判断卡住，不往界面报。
5. [x] [§13 第 38 项](../plugin-protocol.md#13-待定) 的答案：fetch 输出的 `media[]` 加可选 `group`，同一个字符串表示同一段时间线拆开的几个文件；没有 `group` 时宿主照 [§4.1](#41-四个转换插件) 的内容规则配。B 站插件交 `video.m4s`、`audio.m4s` 时写同一个 `group`。
6. [x] [§7.1](../plugin-protocol.md#71-常驻进程的路径)：没有沙箱时硬链接进 `in/` 挡不住插件改原文件，原样靠 `usuyo.fetch.json` 的哈希查出来（同步前和「检查与修复」时）；有沙箱以后照原文。

<a id="9-给-004-的"></a>
## 9. 给 004 的

摘要、关键词层、向量、拼线头都在 004（调研 16 [§1](../research/16-piecing-engine.md#1-这一刻怎么出现)、[§5](../research/16-piecing-engine.md#5-第一版做什么以后加什么)）。004 能接上，靠 003 交出这些：

- **每一面都是带原样坐标的段**：层（`asr`、`ocr`、`frames`，加已有的 `text`、`comments`）、快照、坐标（时间区间；`imageKey` 加框；帧的秒数加框）、字、来源（`runId` 或平台字幕）。004 切段直接用 cue 和 OCR 行，不用自己再找时间。
- **来源都在 `run.json`**：004 的向量 meta 里的 `inputs` 指向这些文件和哈希，换了一次转录就是换了输入，照它的规则重算。
- **标题面、内容层、画面层分得开**：标题、简介、封面上的字是标题面；口播、字幕、正文、图文的图里的字是内容层；关键帧和关键帧上的字是画面层。跟着收的命中默认看标题层加内容层、画面层不算（[SPEC §1.10 第 7 条](../SPEC.md#110-要你看一眼)），004 照这个分。
- **标记只给信号，不给结论**：`no-speech` 的条目，004 只靠标题面挂在线头边上（调研 16 已经这样说视频号）；`static-picture` 不让 004 少看画面。
- **空也是结果**：`empty: true` 告诉 004 这一面确实没东西，免得对着空文稿编（[协议 §4.2](../plugin-protocol.md#42-转换的-run)）。
- 003 不做平台腔停用表、不起名、不写任何总结。

<a id="10-工作包"></a>
## 10. 工作包

按顺序做；同一行里的可以交给一个模型。难度给主持的会话选 Opus 还是 Sonnet。

| # | 做什么 | 碰哪里 | 难度 | 挡住 |
|---|---|---|---|---|
| WP0 | 在这台量：sherpa-onnx + SenseVoice int8 转 [§12](#12-测试方案) 的口播样例和一段用户自己的 10 分钟口播（只在本机，不提交），记实时率、内存峰值；Vision 认 20 张小红书式的图（用户自己的，不提交）记每张耗时。先确认 `sherpa-onnx` crate 在这台编得过：它的 build script 第一次要联网下原生库，而 `just dev-plugins` 用 `--offline`，要么先联网编一次留在缓存里，要么改用 `shared` 特性指向本地的库。量完写进本文 [§3.1](#31-说的转成文字)。实时率比 0.2 慢（10 分钟要两分钟以上），回头换模型或改成只点名时转。**11-11 做了一部分**：合成样例和 11 分钟合成口播量了速度、内存、CER（[§3.1](#31-说的转成文字)）；crate 默认静态链接，第一次编译从 GitHub release 下 `sherpa-onnx-v2.0.1-osx-arm64-static-lib.tar.bz2`（约 21 MB，不校验哈希），缓存在 `target/sherpa-onnx-prebuilt/`，同一个 target 目录再编 `--offline` 能过，换 target 目录或 `cargo clean` 后要联网或设 `SHERPA_ONNX_ARCHIVE_DIR`。还没量：用户自己的真人口播、Vision 认 20 张图、干净机器上的后台优先级 | `plugins/asr-sherpa/bench/`（脚本进 git，数据不进） | 低 | WP10 的默认 |
| WP1 | `run.json` 类型、schema、样例；把 [§8.2](#82-要写回-spec--协议的) 的 SPEC 各条写回 | `crates/usuyo-contract`、`docs/SPEC.md` | 低 | WP5 |
| WP2 | 转换的协议类型、清单词表、`models[]` 新字段；plugin-kit 的 JSON-RPC 循环；把 [§8.2](#82-要写回-spec--协议的) 的协议各条写回 | `crates/usuyo-plugin/src/protocol`、`crates/usuyo-plugin-kit`、`docs/plugin-protocol.md` | 中 | WP3、WP9、WP10 |
| WP3 | 常驻转换进程：启动、核对、`run`、取消、卡住、闲了关、暂存目录、优先级 | `crates/usuyo-plugin/src/host/transform.rs` | 高 | WP6 |
| WP4 | 模型下载：续传、镜像、校验、解包、`model.json`、状态 | `crates/usuyo-core/src/models.rs` | 中 | WP6 |
| WP5 | 转录写入（暂存后改名）、列出、按层找（`resolve_layer` 认转录、`video` 先给 `play.mp4`、`frame/<runId>/<ms>`）、只留文字移走二进制 | `crates/usuyo-library` | 中 | WP6、WP7 |
| WP6 | 排任务、配对、编排链、电源和先后、`transform_jobs`、事件 | `crates/usuyo-core/src/transform.rs`、`jobs.rs`、`engine.rs` | 高 | WP7 |
| WP7 | 条目页的 facets 和 `pending`、标记文字、索引三层和 `source`、搜索的层映射 | `crates/usuyo-core/src/content.rs`、`view.rs`、`index_doc.rs`、`crates/usuyo-index` | 中 | WP12 |
| WP8 | [§8.1](#81-接口) 的接口和命令行 | `apps/desktop/web/src/api/endpoints.ts`、`crates/usuyo-api`、`crates/usuyo-cli` | 中 | WP12 |
| WP9 | `builtin/media`：配对时的 `ffprobe`、`play`、`audio:pcm16k`、`keyframes`；精简 LGPL ffmpeg 的编译脚本和 CI；`just dev-plugins` 装上它（开发机链到 Homebrew 的 ffmpeg） | `plugins/media`、`justfile` | 中 | 验收 |
| WP10 | `builtin/asr-sherpa`：VAD 切段、SenseVoice、cue 切分、去标签、`no-speech` | `plugins/asr-sherpa` | 中 | 验收 |
| WP11 | `builtin/ocr-vision`：Swift Package、JSON-RPC 循环、Vision 调用、坐标换算、`no-text` | `plugins/ocr-vision` | 中 | 验收 |
| WP12 | 前端接上新数据：按 `key` 摆多份、来源标签、`pending` 那一句、「再转一次」「换一个转」、模型下载按钮；样子等画布那边 | `apps/desktop/web/src/item`、`src/plugins` | 中 | 验收 6 |
| WP13 | 测试用的获取插件和样例文件（[§12](#12-测试方案)）；`just dev-fixture-plugin` | `crates/usuyo-core/testdata/transform/`、`justfile` | 低 | 验收 |
| WP14（后做） | `builtin/ocr-ppocr`：Windows、Linux 的默认 | `plugins/ocr-ppocr` | 中 | 等 `ort` 那条（[§13](#13-风险和待定)） |

<a id="11-验收标准"></a>
## 11. 验收标准

都在这台跑，用临时的 `USUYO_HOME` 和端口，不碰 47615，不碰真资料库 `~/Documents/usuyo`，不请求任何平台。电脑接着电源（自动转只在接电时跑）。

```sh
export USUYO_HOME="$(mktemp -d)/home"
export USUYO_PORT=47699
usuyo init "$USUYO_HOME/library"
just dev-plugins                         # 装 builtin/media、asr-sherpa、ocr-vision（开发机版）
just dev-fixture-plugin                  # 装 WP13 的测试获取插件 fixture/local
usuyo daemon --port 47699 &
```

1. `usuyo add https://fixture.test/dash`（画面、音轨分开，带一份 `subtitle.zh.vtt`）。`usuyo transform <id> --job play --wait` 以后，条目里有一次 `job: play` 的转录，`play.mp4` 用 `ffprobe` 看有一条 `video`、一条 `audio`；`raw/` 下每个文件的 sha256 和 `usuyo.fetch.json` 的 `files` 一样（原样没动）。
2. 同一条：`usuyo models list` 显示 SenseVoice `absent`、约 166 MB；`asr` 任务在 `transform_jobs` 里是 `waiting`、`model:sense-voice-int8-2025-09-09`，资料库里没有 `asr` 转录，模型目录是空的（没自己下）。`usuyo models download builtin/asr-sherpa sense-voice-int8-2025-09-09` 以后，`model.json` 在，sha256 对得上；之后那次 `asr` 转录落地，`asr.vtt` 里有「咖啡」和「注水」。
3. 同一条 `GET /items/<id>` 的 `facets` 里有两份 `asr`：`source.by: platform` 和 `source.by: machine`，`key` 不同。
4. `usuyo transform <id> --job asr --plugin builtin/asr-sherpa --option vadThreshold=0.4 --wait`，出第二次 `asr` 转录，`redoOf` 指第一次；两次的文件夹都在，`GET /items/<id>` 里三份 `asr` 都在。
5. `usuyo search 咖啡` 命中这条，层是 `asr`，位置是时间；点开的地址带 `#t=`。删掉索引目录，`usuyo reindex` 以后结果一样。
6. `usuyo add https://fixture.test/note`（两张图，其中一张印着「水温 92°C，粉水比 1:15」，另一张和第一张字节一样）。`ocr` 转录的 `ocr.json` 里有这一行，`bbox` 四个数都在 0–1；`usuyo search 粉水比` 命中，层是 `ocr`，位置带 `imageKey`；`image` 那一面的 `remark` 是「有几张图是同一张」。（窗口里的样子不在这里验：窗口的 CSP 只放行 47615；由主持的会话在 Vite 开发页里看，代理指向这个临时守护进程，或用样例库。）
7. `usuyo add https://fixture.test/silent`（画面是印着字的图，没有音轨）：`asr` 转录 `empty: true`、`reason: no-audio-track`，那一面照样在、`remark` 是「没有声音轨」；`keyframes` 转录的 `marks` 有 `static-picture`；`frames` 那一面有「Pour over in 3 stages」；`usuyo search Pour` 命中，层是 `frames`。
8. `usuyo add https://fixture.test/quiet`（有音轨，只有静音）：`asr` 转录 `empty: true`、`reason: no-speech`，`remark` 是「没听到人说话」。
9. 用 debug 构建的 `usuyo`（测试开关 `USUYO_TEST_POWER=battery` 只在 debug 构建里认），在 `transform_jobs` 里放一条导入档（2）的任务，模拟在电池上：它一直 `waiting`、`power`；`POST /items/<id>/transform {"jobs": "pending"}` 以后它马上跑，用的是普通优先级。
10. 对 `dash` 那条做「只留文字」：`play.mp4`、`keyframes/` 进了回收站，`run.json`、`asr.vtt`、`ocr.json` 还在，搜索照样命中；「把移走的图和视频拿回来」以后又能播。
11. `ps -o pid,pri,nice,command` 看自动跑的转换进程：经 `taskpolicy -b` 起的进程在这台显示 `PRI 4`、`NI 5`（用 `taskpolicy -b sleep 20` 试过）；第 9 步点名以后跑的那个不是 `PRI 4`。
12. `GET /items/<id>`、条目页文字、日志里没有「未转」「待处理」「进度」「队列」这类字，也没有任何计数。
13. 插件仓库 CI 编出来的精简 ffmpeg（不是 Homebrew 那份）跑一遍 `plugins/media` 的样例测试，全过：`USUYO_TEST_FFMPEG=<CI 的 ffmpeg> USUYO_TEST_FFPROBE=<CI 的 ffprobe> cargo test --manifest-path plugins/media/Cargo.toml`。
14. `just check` 和前端的检查都过。

<a id="12-测试方案"></a>
## 12. 测试方案

样例都是本机合成的小文件，提交进 `crates/usuyo-core/testdata/transform/`，连同生成它们的脚本 `make.sh`（只在 macOS 上能重新生成，用 `say` 和 ffmpeg；CI 直接用提交的文件）。不放任何平台内容。

| 样例 | 怎么做 | 大小（估） |
|---|---|---|
| `speech.m4a` | `say -v "Reed (Chinese (China mainland))" "大家好，今天讲手冲咖啡的三段式注水。水温九十二度，粉水比一比十五。"`，再转 aac 64k；这台出来 7.4 秒 | 约 60 KB |
| `video-only.mp4` | `ffmpeg -f lavfi -i testsrc2=size=320x180:rate=25 -t 7.5 -an`，h264；和 `speech.m4a` 的时长差在配对条件内 | 约 200 KB |
| `subtitle.zh.vtt` | 手写两条 cue | 不到 1 KB |
| `card.png`、`card-copy.png` | Swift + AppKit 画四行中英混排的字（ffmpeg 没有 `drawtext`），1080×1440；第二张是第一张的拷贝 | 约 50 KB |
| `silent.mp4` | `ffmpeg -loop 1 -i card.png -t 5 -an`，h264 | 约 50 KB |
| `quiet.mp4` | `testsrc2` 加 `anullsrc` 5 秒 | 约 300 KB |

**测试用的获取插件 `fixture/local`**（WP13）：一个 shell 脚本插件，`platforms: [fixture]`，`match: ["fixture.test/*"]`，`fetch` 把 `${PLUGIN_DIR}/fixtures/<名字>/` 拷进 `--out`，打印写好的 fetch 输出。只给开发和测试用，不进安装包。

**单元和契约**

- contract：`run.json` 样例过 schema；未知字段原样写回。
- plugin：转换协议消息的往返；清单里旧的路径式 `produces` 被拒；`models[].format` 只认那四种。
- host：用假转换插件（脚本）测 `describe` 版本不对不加载、`run` 超时、卡住、取消、闲了关掉、进程退出后整组都没了、`out` 里有符号链接被拒、输出没报的文件被拒。
- models：本机起一个 HTTP 服务器测续传、sha256 不对整个丢掉、镜像依次试、解包后的 `files` 核对。
- library：转录写到一半崩溃（暂存里留着）下次打开被清掉；`resolve_layer("asr")` 给最新的那次；`video` 有 `play.mp4` 时给它；只留文字移走的正好是二进制。
- core：配对规则（时长差刚好在 1 秒、刚好超出；两个视频一个音频不配）；「做好没有」的判法（换了快照、换了插件版本）；先后和电源；缺模型不算次数；条目进回收站删掉任务。
- index：三层的定位串；`reindex` 前后结果相同；连着几帧一样的字合成一段。
- 插件自己：`media` 用上面的样例测 `play`、`keyframes`（`static-picture` 在 `silent.mp4` 上出、在 `video-only.mp4` 上不出）；`asr-sherpa` 测 `quiet.mp4` 出 `no-speech`、`speech.m4a` 出含「咖啡」的 cue、cue 不超过 15 秒；`ocr-vision` 测 `card.png` 四行都出来、框换算到左上原点（第一行的 `y` 小于第四行）。要模型的测试在没有模型时跳过并打印原因；CI 上下载一次、缓存起来跑。

**手动**：验收 6 的窗口部分；拔电源看自动的停下、接上接着跑；一次导入 50 条（用 `fixture/local` 复制出 50 个名字）时风扇和温度，`powermetrics` 看只在能效核上。

<a id="13-风险和待定"></a>
## 13. 风险和待定

**主持的会话定了的（2026-11-11）**

1. **ffmpeg 自己编。** ADR-003 说第三方二进制「只从上游官方 release 下载」，ffmpeg 不发二进制。定：从官方源码包（锁 sha256）在插件 CI 里编精简的 LGPL 静态构建，发版前做，做的时候给 ADR-003 补一条修订；在那之前开发机链到 Homebrew 的 ffmpeg，用精简构建再跑一遍的那条[验收](#11-验收标准)跟着发版走。不用 martin-riedl.de 的构建（多半是 GPL）。
2. **平台给了字幕也照样转。** 定：照转，两份并排（用户说每一面都认出来；B 站的字幕很多本来就是平台机器转的）；代价是多花一点电，WP0 量完太慢再回头看。
3. **`ort` 没有正式版。** PP-OCRv6 那条路（WP14）只能用 `ort` 2.0.0-rc.15。定：先只做 macOS 的 Vision，Windows、Linux 上线前再看 `ort` 出没出 2.0.0。
4. **转录不删。** 定：第一版不给「删掉这一次」；删掉条目时跟着条目走。
5. **SPEC 的 `derived/` 改成一次一个文件夹**（定：照这样，几次并排正是用户要的「换个角度再认一遍」），和调研 16 [§2](../research/16-piecing-engine.md#2-每一步怎么指得回去怎么重做) 的「现在那份放原处」不一样（[§5.1](#51-一次转换一个文件夹)）；004 的向量文件也照这个放。
6. **自动转只看接没接电，不看空闲。** PRD 写「接电、空闲时做」；空闲要读系统的输入空闲时间。定：第一版用后台优先级代替，量过温度再说。
7. **模型什么时候下。** 定：第一条用到它的条目上问一次，写大小和许可，一个按钮；不点就不下，也不再提。
8. **口播为主的视频也出画面那一面。** 定：照出，头上标「画面几乎不动」，不藏。

**没核实的**

- SenseVoice 在这台只用合成语音量过（[§3.1](#31-说的转成文字)）；真人口播的效果、干净机器上后台优先级的速度没量。FireRedASR2、Qwen3-ASR 都没跑（没下载模型）；表里只有厂商的数和调研 04 的数。
- PP-OCRv6 各档模型多大（ModelScope 不给）；精简 ffmpeg 静态构建多大。
- SFSpeechRecognizer 从 Tauri 的 App 里（而不是命令行子进程）能不能拿到授权：没试，也不打算用它当默认。
- 中英混说一句里切换时 SenseVoice 的效果；小红书的手写、艺术字在 Vision 上的效果。
- Media Fragments 里 `#t=` 和 `xywh=` 用 `&` 合写，系统 webview 播放器不认空间部分，只是我们自己解析：要在前端自己处理。

**风险**

- 抖音式快剪 `scdet` 会多检，靠每分钟 6 帧、一条 60 帧的上限压住（调研 04 说上限比阈值更要紧）。
- 硬链接进暂存目录、又没沙箱时，插件能改到原样；靠哈希查出来，不靠挡。
- 一条 1 小时的视频，第一遍低清扫要把整片解一遍，后台优先级下可能要几分钟；没有进度条，就是那一面晚一点出来。

<a id="来源"></a>
## 来源

2026-11-11 查阅；GitHub 的版本用 `gh api repos/<repo>/releases` 查。

- sherpa-onnx v2.0.1（2026-10-26）：<https://github.com/k2-fsa/sherpa-onnx/releases/tag/v2.0.1>；ASR 模型（大小、digest）：<https://github.com/k2-fsa/sherpa-onnx/releases/tag/asr-models>；SenseVoice 文档：<https://k2-fsa.github.io/sherpa/onnx/sense-voice/index.html>；Rust crate 2.0.1：<https://crates.io/crates/sherpa-onnx>；Rust 示例：<https://github.com/k2-fsa/sherpa-onnx/tree/master/rust-api-examples>
- SenseVoice（代码 MIT，权重另有许可，时间点，速度对比）：<https://github.com/QwenAudio/SenseVoice>；FunASR 模型许可：<https://github.com/modelscope/FunASR/blob/main/MODEL_LICENSE>；FunASR v1.4.25（2026-11-11）：<https://github.com/modelscope/FunASR/releases>
- FireRedASR2S（Apache-2.0）：<https://github.com/FireRedTeam/FireRedASR2S>
- whisper.cpp v1.9.5（2026-10-06）：<https://github.com/ggml-org/whisper.cpp/releases>
- Apple SpeechAnalyzer（macOS 26.0 起）：<https://developer.apple.com/documentation/speech/speechanalyzer>；SpeechTranscriber：<https://developer.apple.com/documentation/speech/speechtranscriber>；中文在首发语言里（第三方）：<https://blog.addpipe.com/apple-speechanalyzer-api/>
- Apple Vision 认字：<https://developer.apple.com/documentation/vision/recognizing-text-in-images>（这台的语言列表和耗时是本机实测）
- PP-OCRv6（各档精度、M4 上的耗时）：<https://www.paddleocr.ai/latest/en/version3.x/algorithm/PP-OCRv6/PP-OCRv6.html>；PaddleOCR v3.7.0：<https://github.com/PaddlePaddle/PaddleOCR/releases>；RapidOCR v3.10.0 的模型列表（默认 PP-OCRv6 small）：<https://rapidai.github.io/RapidOCRDocs/main/model_list/>
- oar-ocr 0.10.2（Apache-2.0，经 `ort`）：<https://github.com/greatv/oar-ocr>；`ort` 2.0.0-rc.15：<https://github.com/pykeio/ort/releases>；ONNX Runtime 1.31.1：<https://github.com/microsoft/onnxruntime/releases>
- FFmpeg 9.0.2（2026-09-18）和各平台构建：<https://ffmpeg.org/download.html>；macOS arm64 签名构建：<https://ffmpeg.martin-riedl.de/>
- PySceneDetect v0.7.3（2026-11-08）：<https://github.com/Breakthrough/PySceneDetect/releases>
- 早先的调研：[04 §2](../research/04-tech-options.md#2-ocr-sidecar)、[04 §3](../research/04-tech-options.md#3-asr-sidecar)、[05 §2](../research/05-note-apps-and-plugins.md#2-转换类插件与工具)、[15 §4](../research/15-fetch-design-xhs-sph-bili.md#4-b站)、[16 §2](../research/16-piecing-engine.md#2-每一步怎么指得回去怎么重做)
