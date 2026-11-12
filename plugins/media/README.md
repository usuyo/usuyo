<a id="media"></a>
# media

预装的转换插件 `builtin/media`：包着 ffmpeg 和 ffprobe，做四件事：分开的画面和音轨合成能播的 `play.mp4`、抽出给转写用的 16 kHz 音频、取关键帧并量画面停住了多久、报文件里有哪些流。做法和格式照 [feature 003](../../docs/features/003-transforms.md)（[§4.1](../../docs/features/003-transforms.md#41-四个转换插件)、[§5.3](../../docs/features/003-transforms.md#53-各样东西的格式)、[§6](../../docs/features/003-transforms.md#6-很明显的标记)）。许可 MIT，同 `plugins/web`。

<a id="目录里有什么"></a>
## 目录里有什么

| 路径 | 是什么 |
|---|---|
| `usuyo-plugin.yaml` | 清单：`kind: transform`，吃 `media:video`、`media:audio`，出 `play`、`audio:pcm16k`、`probe`、`keyframes`，不要模型，不联网 |
| `src/main.rs` | 进程入口：在 `USUYO_PLUGIN_DIR` 下找 `ffmpeg`、`ffprobe`，问版本，跑 plugin-kit 的 JSON-RPC 循环 |
| `src/plugin.rs` | `run` 的分发、`health`；查 `want`、`inputs`、`options`，错误在这里换成协议的错误对象 |
| `src/products.rs`、`src/keyframes.rs` | 四种产物的做法 |
| `src/scan.rs`、`src/select.rs` | 第一遍扫描的 stderr 怎么读、选哪几帧（纯函数，没有 ffmpeg 也能测） |
| `src/exec.rs`、`src/tools.rs`、`src/probe.rs`、`src/budget.rs` | 起子进程和盯着它、命令行的拼法、读 ffprobe 的 JSON、超时上限 |
| `src/options.rs`、`src/jpeg.rs`、`src/error.rs` | `keyframes` 的参数、JPEG 头里的宽高、错误类型 |

<a id="ffmpeg-从哪来"></a>
## ffmpeg 从哪来

插件只认 `${PLUGIN_DIR}/ffmpeg` 和 `${PLUGIN_DIR}/ffprobe`，不从 `PATH` 找；启动时各问一次 `-version`，缺一个或版本行读不出就退出，原因写在 stderr。`run` 结果里的 `tool` 是这个版本行里的名字和版本号，不是插件自己写的。

- 发版：随插件的资产带精简的 LGPL 静态构建（[§4.3](../../docs/features/003-transforms.md#43-ffmpeg-和模型怎么到这台电脑上)），编译脚本还没写。
- 开发机：资产里这两个文件换成指向 Homebrew 的链接：

```
ln -s /opt/homebrew/bin/ffmpeg  "$PLUGIN_DIR/ffmpeg"
ln -s /opt/homebrew/bin/ffprobe "$PLUGIN_DIR/ffprobe"
```

用到的 ffmpeg 功能：mov/mp4 的拆封装和 mp4 封装（`-movflags +faststart`）、wav 和 `image2` 封装、`null` 封装；h264、aac 的解码；`-c copy`；`aresample`（`-ar`、`-ac`）、`pcm_s16le`、`mjpeg` 编码；`fps`、`scale`、`format`、`scdet`、`freezedetect` 滤镜；输入前的 `-ss` 会让 ffmpeg 自动加 `trim` 滤镜（`-loglevel debug` 里看到的；feature 003 的 [§4.3](../../docs/features/003-transforms.md#43-ffmpeg-和模型怎么到这台电脑上) 滤镜清单里没有它），转音频时自动加 `aresample`；`-progress pipe:1`；输入选项 `-protocol_whitelist`、`-format_whitelist`；ffprobe 的 `-show_entries` 和 `-of json`。

<a id="跑起来是什么样"></a>
## 跑起来是什么样

照[协议 §4.2](../../docs/protocol/transform-export.md#42-转换的-run) 在 stdio 上说 JSON-RPC；循环由 plugin-kit 管。一次 `run` 的 `want` 只有一项，输入和产物：

| `want` | 输入 | 产物 | 做法 |
|---|---|---|---|
| `probe` | 一个或几个 `media:video`、`media:audio` | 没有文件；结果里的 `probe[]`：每个输入的时长、每条流的 `kind` 和 `codec` | `ffprobe -show_entries … -of json`；工具名是 `ffprobe` |
| `play` | 一个 `media:video`、一个 `media:audio`，顺序不论 | `play.mp4` | `ffmpeg -i <画面> -i <音轨> -map 0:v:0 -map 1:a:0 -c copy -movflags +faststart`，不重编码、不加 `-shortest` |
| `audio:pcm16k` | 一个 `media:audio` 或 `media:video` | `audio.wav`：16 kHz 单声道 16 位 | 取第一条音轨，`-ac 1 -ar 16000 -c:a pcm_s16le`；没有音轨回 `empty: true`、`reason: no-audio-track`，`marks` 带 `no-audio-track` |
| `keyframes` | 一个 `media:video`，要带 `snapshot` | `keyframes.json` 和 `keyframes/<毫秒，8 位补零>.jpg` | 两遍，见下 |

- **拒绝**：`play` 的画面文件有音频流、或音轨文件有画面流（封面图不算）回 `unsupported-input`，什么也不写；ffprobe 认不出的文件（图片、坏文件）也是 `unsupported-input`；`want` 不是一项、输入种类不对、`probe`/`play`/`audio:pcm16k` 带了 `options`、`keyframes` 的参数越界或缺 `snapshot` 回 `invalid-args`；取消回 `cancelled`；ffmpeg 失败、超时回 `internal`（超时是宿主才能报的码）。
- **`keyframes` 第一遍**：`ffmpeg -vf fps=4,scale=-2:360,scdet=threshold=10,freezedetect=n=0.003:d=2 -f null -`，只读 stderr 里以 `[滤镜名 @ 地址]` 开头的 `lavfi.scd.score/time` 和 `lavfi.freezedetect.freeze_start/end` 行。`freezedetect` 停到片尾时不打 `freeze_end`，这种区间按停到片尾算。停住的时间加起来到时长的 `staticRatio`（0.9）就在 `marks` 里加 `static-picture`，带 `frozenSec`、`durationSec`；`keyframes.json` 里的 `frozenSec` 总是写。
- **选帧**：总有 `firstSec`（0.5 秒）一帧；镜头变化处按分数从高到低挑，和已选的每一帧至少隔 `minGapSec`（2 秒），每分钟最多 `maxPerMinute`（6）帧，总共最多 `maxFrames`（60）帧，0.5 秒那一帧也算数；选出来的帧少于 `min(minFrames, 时长 / minFrameEverySec)`（3，10 秒）时按时长均分补齐，离已选的帧不到 0.5 秒的点跳过；时间点最晚到片尾前 0.1 秒。
- **第二遍**：每个时间点 `-ss <t> -i … -frames:v 1`，长边缩到不超过 `maxSide`（960，不放大），`format=yuvj420p`，`-q:v jpegQuality`（4）。哪个时间点取不到帧，那一帧不写进 `keyframes.json`，stderr 留一行；一帧也取不到才报错。宽高从 JPEG 头里读，`sha256` 是文件的。
- **`keyframes` 参数**：都在 `options` 里，不在清单设置里（设置的键只能小写加 `-`，也没有小数）。名字和默认值：`scanFps` 4、`scanHeight` 360、`sceneThreshold` 10、`freezeNoise` 0.003、`freezeMinSec` 2、`firstSec` 0.5、`minGapSec` 2、`maxPerMinute` 6、`maxFrames` 60、`minFrames` 3、`minFrameEverySec` 10、`maxSide` 960、`jpegQuality` 4、`staticRatio` 0.9。结果的 `options` 回显实际用的全部参数，原样交回照样认；不认识的键、越界的值回 `invalid-args`。
- **进度**：ffmpeg 的 `-progress pipe:1` 每约半秒一条，转成 `progress`（`sec`；取帧时是 `frames`）；没有新进度时每 10 秒补一条心跳，一小时的文件扫描也不会触发宿主的 120 秒卡住判定。
- **超时**：按时长算：第一遍 120 秒加 2 倍时长，`play` 120 秒加 0.2 倍，`audio:pcm16k` 120 秒加 0.5 倍，取一帧 60 秒，ffprobe 60 秒，最多 6 小时；时长不明时一小时。超时和取消都杀掉子进程再等它收尾，不留孤儿。

<a id="怎么起-ffmpeg"></a>
## 怎么起 ffmpeg

- 参数是向量，不经 shell；子进程 stdin 是空的、环境变量清空，加 `-nostdin`。
- 输入输出路径一律变成绝对路径、加 `file:` 前缀：以 `-` 开头的文件名不会被读成选项，`concat:`、`http:` 开头的文件名不会被读成协议。
- 每个输入前加 `-protocol_whitelist file` 和 `-format_whitelist mov,mp4,m4a,3gp,3g2,mj2,matroska,webm,flv,mpegts,aac,mp3,wav`：内容是 `ffconcat` 清单或别的封装的文件不会被跟着读；这几种就是 [§4.3](../../docs/features/003-transforms.md#43-ffmpeg-和模型怎么到这台电脑上) 给精简构建开的拆封装器，别的封装（avi、ogg 等）会被拒绝。
- 输出用 `-n`（不覆盖），先确认产物不在，做完确认它在。
- stderr 是不可信的文字：一行最多 8 KiB，只留要的几种行（上限 20 万条），出错时带最后 12 行去掉控制字符放进错误信息；ffprobe 的 stdout 最多收 4 MiB。

<a id="测试"></a>
## 测试

```
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
```

- 不要 ffmpeg 的：清单过仓库的清单类型（`tests/manifest.rs`）；`scdet`、`freezedetect` 输出的解析（文字样例，含「停到片尾没有 `freeze_end`」和冒充输出的元数据行）、选帧（间隔、每分钟上限、总数上限、补帧、片尾）、ffprobe JSON、参数、JPEG 头、版本行、路径前缀（各模块里的单元测试）。
- 要 ffmpeg 的（`tests/plugin.rs`）：起插件的可执行文件，在 stdio 上和它说 JSON-RPC。`probe` 每个样例；`play` 合成后一条画面一条音频、编码没变、输入字节没变，拒绝已经有声音的画面；`audio:pcm16k` 从 `speech.m4a` 和 `quiet.mp4` 出 16 kHz 单声道，没音轨回 `no-audio-track`；`keyframes` 在 `silent.mp4` 上标 `static-picture`、在 `video-only.mp4` 上不标，红蓝镜头变化处取帧，首尾相接的长片段不超每分钟上限、间隔至少 2 秒，`keyframes.json` 过契约的 schema 和类型；取消时 ffmpeg 停下、没有子进程；`-loglevel.mp4`、`concat:…`、`http:x.mp4` 这样的文件名当文件；ffconcat 清单不被跟着读；坏请求回 `invalid-args`；插件目录里没有 ffmpeg 时起不来。
- ffmpeg 默认用 `/opt/homebrew/bin/ffmpeg`、`/opt/homebrew/bin/ffprobe`，不在就跳过并打印原因（`cargo test -- --nocapture` 看得到）。换成别的构建（例如 CI 编的精简版）：

```
USUYO_TEST_FFMPEG=<路径> USUYO_TEST_FFPROBE=<路径> cargo test
```

取消的测试用 Homebrew 的 ffmpeg 现合成约 50 分钟的片段（样例首尾相接 400 遍，拷流），断言扫描只报到一半以内；镜头变化、首尾相接的片段、分段 mp4（`.m4s`，和获取插件存的一样）也是这样合成的，合成用的 ffmpeg 一直是 `/opt/homebrew/bin/ffmpeg`，`USUYO_TEST_FFMPEG` 换的只是被测的那一份。

样例是 [`crates/usuyo-core/testdata/transform/`](../../crates/usuyo-core/testdata/transform/README.md) 里的 `speech.m4a`、`video-only.mp4`、`silent.mp4`、`quiet.mp4`、`card.png`；镜头变化和长片段由测试用 ffmpeg 现合成。
