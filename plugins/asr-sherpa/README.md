<a id="asr-sherpa"></a>
# asr-sherpa

预装的转换插件 `builtin/asr-sherpa`：一段 16 kHz 单声道的 WAV 进去，带时间点的 `asr.vtt` 出来。silero VAD v5 切段，sherpa-onnx 2.0.1（静态链接）跑 SenseVoice-Small int8（2024-07-17 那份）。做法和格式照 [feature 003](../../docs/features/003-transforms.md)，选型和量出来的数字在它的[「说的转成文字」](../../docs/features/003-transforms.md#31-说的转成文字)。许可 MIT，同 `plugins/web`。

<a id="目录里有什么"></a>
## 目录里有什么

| 路径 | 是什么 |
|---|---|
| `usuyo-plugin.yaml` | 清单：`kind: transform`，吃 `audio:pcm16k`，出 `asr`，模型条目 |
| `src/main.rs` | 进程入口：核 VAD，跑 plugin-kit 的 JSON-RPC 循环 |
| `src/plugin.rs` | `run`、`health`：查输入、找模型、转写、写 `asr.vtt` 或回 `empty` |
| `src/engine.rs` | VAD 切段、建识别器、逐段转写；模型和 VAD 的文件名、大小、哈希 |
| `src/cues.rs` | cue 切分、去标签、猜语言、`no-speech` 的判法、写 WebVTT |
| `src/options.rs`、`src/wav.rs` | 参数（读、补默认、回显）和 WAV 头检查 |
| `examples/bench.rs`、`bench/` | 量引擎的速度和字错率，见[量速度](#量速度) |
| `sherpa-onnx-v2.0.1.sha256` | sherpa-onnx 预编译静态库各平台压缩包的 sha256（取自 GitHub release 的 digest） |

<a id="跑起来是什么样"></a>
## 跑起来是什么样

插件是常驻进程，照[协议 §4.2](../../docs/protocol/transform-export.md#42-转换的-run) 在 stdio 上说 JSON-RPC；循环由 plugin-kit 管（[只写 `run`](../../crates/usuyo-plugin-kit/README.md#转换插件只写-run)）。

- **启动**：读 `USUYO_PLUGIN_DIR`、`USUYO_MODELS_DIR`，缺一个就退出。核 `${PLUGIN_DIR}/silero_vad_v5.onnx` 的 sha256（`6b99cbfd39246b6706f98ec13c7c50c6b299181f2474fa05cbc8046acc274396`，2,313,101 字节），对不上就退出。`describe` 的 `models` 是启动时已经在的模型。
- **模型在哪**：插件要宿主把压缩包原样解进 `${MODELS}/sense-voice-int8-2024-07-17/`、不去掉顶层那一层目录（宿主的模型下载还没做，协议也还没写明这一点），所以文件在 `…/sense-voice-int8-2024-07-17/sherpa-onnx-sense-voice-zh-en-ja-ko-yue-int8-2024-07-17/model.int8.onnx` 和 `tokens.txt`。插件只查这两个文件在、字节数对；不在或不全回 `dependency-missing`，`needs: ["model:sense-voice-int8-2024-07-17"]`。插件不联网，不下载任何东西。
- **一次 `run`**：只认一个 `audio:pcm16k` 输入；WAV 头不是 PCM、单声道、16 kHz、16 位的回 `unsupported-input`，参数不对回 `invalid-args`。识别器建好以后留在进程里，下一次 `language`、`itn`、`threads` 一样就接着用，变了才重建。VAD 每次新建；它切出来的段最长 30 秒，原型量过这样长的段转得明显差，所以 cue 再按 `maxCueSec` 切。音频按秒读，每个 VAD 段落下来就转；每转完一段、每扫过 30 秒音频发一次 `progress`，同时查取消。
- **结果**：人声加起来不到 1 秒或不到时长的 2% 时不写文件，回 `empty: true`、`reason: no-speech`，`marks` 带 `speechSec`、`durationSec`（[§6](../../docs/features/003-transforms.md#6-很明显的标记)）。否则写 `asr.vtt`（[§5.3](../../docs/features/003-transforms.md#53-各样东西的格式)）：一个 VAD 段一条 cue，长于 `maxCueSec` 的按 token 时间点在标点处切开，去掉 `<|…|>` 标签，`&`、`<`、`>` 转义，开头 `NOTE language: <lang>`。`tool` 是 `sherpa-onnx` 和运行库报的版本，`options` 回显实际用的参数。
- **语言**：sherpa-onnx 的安全接口不带出 SenseVoice 认出的语言（运行库自己去掉了 `<|zh|>`，crate 的结果结构没有语言字段，要拿得写 `unsafe`），所以 `language: auto` 时按文字的种类猜：假名多是 `ja`，谚文多是 `ko`，有汉字是 `zh`，只有拉丁字母是 `en`，都没有是 `und`；分不出普通话和粤语。指定了语言就写指定的那个。

参数（`run` 的 `options`）：

| 键 | 默认 | 取值 | 清单里是设置吗 |
|---|---|---|---|
| `model` | `sense-voice` | 只有这一个 | 是 |
| `language` | `auto` | `auto` `zh` `en` `ja` `ko` `yue` | 是 |
| `itn` | `true` | 出标点和数字 | 是 |
| `threads` | `2` | 1–16；这台 2 个比 1 个快将近一倍，再多反而慢 | 是 |
| `vad` | `silero-vad-v5` | 只有这一个，回显用 | 否 |
| `vadThreshold` | `0.5` | 0–1 之间 | 否：设置的键只能是小写加 `-`，也没有小数类型 |
| `maxCueSec` | `15` | 1–30 秒 | 否：同上 |

回显的那份原样交回来照样认（宿主重做时就是这样）；不认识的键回 `invalid-args`。

<a id="开发时怎么编"></a>
## 开发时怎么编

这是一个独立的 cargo 包，有自己的 `[workspace]` 和锁文件，不在仓库的工作区里。

```
cd plugins/asr-sherpa
cargo build --release
```

- 第一次编译时 `sherpa-onnx-sys` 从 sherpa-onnx 的 GitHub release 下一个预编译的静态库（macOS arm64 约 21 MB），**不核对哈希**，解在 `target/sherpa-onnx-prebuilt/`。之后同一个 `target/` 不再下；`cargo clean` 或换一个 target 目录要再下一次。
- 以后离线编：把那个压缩包（`target/sherpa-onnx-prebuilt/sherpa-onnx-v2.0.1-<平台>-static-lib.tar.bz2`）留一份在别处，用 `SHERPA_ONNX_ARCHIVE_DIR` 指过去，加 `--offline`。
- 出来的可执行文件约 27 MB（macOS arm64，release），只链系统库。

<a id="发版时怎么锁住预编译库"></a>
## 发版时怎么锁住预编译库

发版的构建不让 `sherpa-onnx-sys` 自己下：先自己下好那个平台的压缩包，核过 sha256，再交给它。

```
mkdir -p /tmp/sherpa && cd /tmp/sherpa
curl -fLO https://github.com/k2-fsa/sherpa-onnx/releases/download/v2.0.1/sherpa-onnx-v2.0.1-osx-arm64-static-lib.tar.bz2
shasum -a 256 -c --ignore-missing <仓库>/plugins/asr-sherpa/sherpa-onnx-v2.0.1.sha256
cd <仓库>/plugins/asr-sherpa
SHERPA_ONNX_ARCHIVE_DIR=/tmp/sherpa cargo build --release --locked --offline
```

`sherpa-onnx` 在 `Cargo.toml` 里锁成 `=2.0.1`；升级时连同 `sherpa-onnx-v2.0.1.sha256` 一起换。发版的资产里除了可执行文件，还要放 `silero_vad_v5.onnx`（从 sherpa-onnx 的 `asr-models` release 取，按上面那个 sha256 核）和 silero-vad 的 MIT 许可声明；这个文件不进仓库。

<a id="测试"></a>
## 测试

```
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
```

- 不要模型的：清单过仓库的清单类型（`tests/manifest.rs`），cue 切分、去标签、语言、WAV 头、参数（各模块里的单元测试），VAD 对不上时进程不肯起。
- 要模型的（`tests/plugin.rs`）：起插件的可执行文件，在 stdio 上和它说 JSON-RPC，每一行 stdout 都得是 JSON-RPC。`speech` 出含「咖啡」「注水」的 cue、没有一条超过 15 秒；`quiet` 回 `empty` / `no-speech`；8 kHz 不收；空的和没下完的模型目录回 `dependency-missing`；`cancel` 在段之间停下、不写文件。样例是 [`crates/usuyo-core/testdata/transform/`](../../crates/usuyo-core/testdata/transform/README.md) 里的 `speech.m4a`、`quiet.mp4`，测试里用 `/opt/homebrew/bin/ffmpeg` 转成 WAV；没有 ffmpeg 就跳过。
- 要模型的测试找 `USUYO_TEST_SHERPA_DIR`，没设就跳过（原因打在 stderr 上，`cargo test -- --nocapture` 看得到）。这个目录照宿主的样子摆：

```
$USUYO_TEST_SHERPA_DIR/
  sense-voice-int8-2024-07-17/sherpa-onnx-sense-voice-zh-en-ja-ko-yue-int8-2024-07-17/   压缩包原样解开
  silero_vad_v5.onnx
```

```
USUYO_TEST_SHERPA_DIR=<目录> cargo test
```

<a id="量速度"></a>
## 量速度

`examples/bench.rs` 用和插件一样的 VAD、识别器、cue 切分，不走 JSON-RPC，往 stderr 打一行 JSON 的计时；`bench/` 里的脚本调它：

- `bench/make-long.sh`：用 `say` 把 `long.txt` 合成一段约 11 分钟的样例 `target/bench/long.wav`，参考文本写进 `ref.txt`。
- `bench/run.sh <wav> <模型目录> <silero_vad_v5.onnx> <标签> [次数] [线程数…]`：模型目录照[测试](#测试)那样摆；每次跑普通优先级和后台优先级各一遍，结果追加进 `bench/results.tsv`，转出来的字在 `target/bench/<标签>.vtt`。
- `python3 -I bench/cer.py bench/ref.txt target/bench/<标签>.vtt`：粗算字错率。

`results.tsv` 里现在那些行是原型时量的（`threads` 为 0 是 crate 的默认，即 1 个线程）。
