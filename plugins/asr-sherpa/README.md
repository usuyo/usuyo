<a id="asr-sherpa"></a>
# asr-sherpa

口播转文字的原型：一段 16 kHz 单声道的 WAV 进去，带时间点的 WebVTT 出来。做法和格式照 [feature 003](../../docs/features/003-transforms.md)；这份现在还是一个命令行程序，接着要改成说转换协议的插件（那份的工作包 WP10），改的时候就地改，不另起一份。

<a id="怎么跑"></a>
## 怎么跑

```
cargo build --release
target/release/spike-asr in.wav out.vtt --model <SenseVoice 模型目录> --vad <silero_vad_v5.onnx> --threads 2
```

模型不进仓库：SenseVoice 用 sherpa-onnx 发的 `sherpa-onnx-sense-voice-zh-en-ja-ko-yue-int8-2024-07-17`，VAD 用 `silero_vad_v5.onnx`（2,313,101 字节；`silero_vad.onnx` 是另一个模型）。输入先用 ffmpeg 转成 16 kHz 单声道，别的采样率程序不收。

<a id="量出来的"></a>
## 量出来的

脚本在 `bench/`：`make-long.sh` 用 `say` 把 `long.txt` 合成一段约 11 分钟的样例，`run.sh` 计时，`cer.py` 对着 `ref.txt` 算字错率，结果在 `results.tsv`。数字和结论写在 [feature 003 的「说的转成文字」](../../docs/features/003-transforms.md#31-说的转成文字)，这里不重复。

<a id="改成插件时要知道的"></a>
## 改成插件时要知道的

- `sherpa-onnx` crate 默认静态链接。第一次编译会从它的 GitHub release 下一个预编译的静态库（约 21 MB），不核对哈希，放在 `target/sherpa-onnx-prebuilt/`；换一个 target 目录或 `cargo clean` 以后要再下一次。发版的构建要自己先下好、核过哈希，再用 `SHERPA_ONNX_ARCHIVE_DIR` 指过去。
- 线程数不给就是 1。给 2 比 1 快将近一倍，给 8 反而比 2 慢。
- 结果里没有语言：运行库自己去掉了 `<|zh|>` 这类标签，crate 的结果结构又没带出 C 库回的语言字段，要拿得写 `unsafe`。原型按文字的种类猜语言。
- VAD 切出来的段太长（到 30 秒）时转得明显差；照 15 秒在标点处切开以后正常。
- 用到的调用：VAD 是 `VoiceActivityDetector::create`，每次喂 512 个采样，`front`、`pop`、`flush`；识别是 `OfflineRecognizer::create` 配 `OfflineSenseVoiceModelConfig { language: "auto", use_itn: true }`，再 `create_stream`、`accept_waveform`、`decode`、`get_result`；时间点在结果的 `tokens` 和 `timestamps` 里，一个 token 一个。
