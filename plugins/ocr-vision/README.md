<a id="ocr-vision"></a>
# ocr-vision

预装的转换插件 `builtin/ocr-vision`：图文的图、封面、视频的关键帧进去，一份 `ocr.json` 出来，每一行字记下在哪张图、哪一块。用系统的 Vision（`VNRecognizeTextRequest` revision 3），只在 macOS 上有，不下载模型，不联网。做法和格式照 [feature 003](../../docs/features/003-transforms.md)，选型和量出来的数字在它的[「图里的字」](../../docs/features/003-transforms.md#32-图里的字)。许可 MIT，同 `plugins/web`。

用 Swift 写：Vision 在 Rust 里只能经 `objc2`，要写 `unsafe`，仓库不许（[§4.1](../../docs/features/003-transforms.md#41-四个转换插件)）。所以它不用 plugin-kit，自己带一个小的 JSON-RPC 循环，线上的表现照 plugin-kit 的那一份。

<a id="目录里有什么"></a>
## 目录里有什么

| 路径 | 是什么 |
|---|---|
| `usuyo-plugin.yaml` | 清单：`kind: transform`，吃 `image`、`derived:keyframes`，出 `ocr` |
| `Package.swift` | Swift Package，Swift 6 语言模式，只用 Foundation、Vision、CoreGraphics、ImageIO 和标准库 |
| `Sources/TransformLoop/` | JSON-RPC 循环和协议消息，不碰 Vision：分帧、`describe`、`health`、`run`、`cancel`、`progress`、错误码 |
| `Sources/VisionOCR/` | `run` 和 `health`：查参数和路径、读图、调 Vision、换算框、写 `ocr.json` |
| `Sources/usuyo-plugin-ocr-vision/main.swift` | 进程入口 |
| `Tests/OcrVisionTests/` | 测试，见[测试](#测试) |

<a id="跑起来是什么样"></a>
## 跑起来是什么样

插件是常驻进程，照[协议 §4.2](../../docs/protocol/transform-export.md#42-转换的-run) 在 stdio 上说 JSON-RPC。

- **循环**：一行一条消息，最长 1 MiB，更长的回 `-32600` 并丢掉这一行；不是 JSON、不是对象、`jsonrpc` 不对、没有 `method` 的各回对应的标准码，不退出。读 stdin 在一个专门的线程上，一帧处理完才读下一帧。同一时间只跑一个 `run`，跑着的时候 `describe`、`health`、`cancel` 照样收，第二个 `run` 回 `invalid-args`。收到 `cancel` 以后，`run` 怎么结束都回 `cancelled`。stdin 关了：取消跑着的，最多等 4 秒它回完，再退出；等不到就以失败退出。stdout 只放协议消息。
- **`describe`**：`models` 是 accurate 一档 Vision 认得的语言。`health` 查这台系统有没有 revision 3。
- **一次 `run`**：`want` 只能是 `[ocr]`。每个输入要么是 `image` 带 `imageKey`（16 位小写十六进制），要么是 `derived:keyframes` 带 `frameSec`，不然回 `invalid-args`。然后一张一张读图、认字；每张之前查取消，每张之后发一次 `progress`（`unit: image`）。读不了的图回 `unsupported-input`，消息只写下标（「input 1 is not a readable image」），不写路径。图按 EXIF 方向摆正再认，`width`、`height` 是摆正以后的。
- **路径**：只认[协议 §7.1](../../docs/protocol/isolation.md#71-常驻进程的路径) 的布局：`out` 是 `<暂存>/out/<callId>/`，输入都要在同一个暂存目录的 `in/<callId>/` 里。路径要是绝对路径、不带 `.` `..`，输入要是普通文件（不是符号链接），解开上级的符号链接以后还在 `in/<callId>/` 里；不合的回 `invalid-args`，只写下标。`ocr.json` 写进 `out`，已经有同名的（含符号链接）就不写。
- **结果**：所有输入都认完、没被取消才写 `ocr.json`，一份盖住这次所有的图，格式见 [§5.3](../../docs/features/003-transforms.md#53-各样东西的格式)：`input` 是输入的下标，带 `imageKey` 或 `frameSec`，`lines` 从上到下。Vision 的框是 0–1、原点在左下角，换成原点在左上角的 `[x, y, w, h]`，夹到 0–1，留 4 位小数；分数留 3 位。分数低的行照样写进去。所有图都没有一行分数到 0.3 时不写文件，回 `empty: true`、`reason: no-text`，`marks` 带 `no-text`（[§6](../../docs/features/003-transforms.md#6-很明显的标记)）。`tool` 是 `Vision`，版本写系统版本、build 和请求的 revision，例如 `macOS 15.6.1 (24G90), VNRecognizeTextRequest revision 3`。

参数（`run` 的 `options`），回显的那份原样交回来照样认；不认识的键回 `invalid-args`：

| 键 | 默认 | 取值 |
|---|---|---|
| `level` | `accurate` | `accurate` `fast`；fast 一档在这台只认欧洲语言，没有中文 |
| `languages` | `zh-Hans` `en-US` `zh-Hant` `yue-Hans` `yue-Hant` | 这一档 Vision 认得的语言，按先后；默认里这一档不认的去掉 |
| `languageCorrection` | `true` | 让 Vision 按语言纠字 |

清单里不把它们写成设置：设置没有列表类型，键也不能有大写。

<a id="开发时怎么编"></a>
## 开发时怎么编

不在仓库的 cargo 工作区里（工作区的成员是明着列的），要 macOS 15 以上和 Swift 6.2：

```
cd plugins/ocr-vision
swift build -c release
```

出来的是 `.build/release/usuyo-plugin-ocr-vision`，只链系统框架，不要别的资产。装的时候把它和 `usuyo-plugin.yaml` 放进同一个目录（清单的命令是 `${PLUGIN_DIR}/usuyo-plugin-ocr-vision`）。

这台 M1 上量的（release，`card.png`，1080×1440，四行）：进程起来以后第一次 `run` 1.9–5.2 秒（Vision 第一次加载），之后每次约 0.2 秒；同一台机器上同时跑着别的编译时偏慢的那一头。读图和认字在一个自己的线程上跑，不占 Swift 并发的线程池。

<a id="测试"></a>
## 测试

```
swift test
```

- 不调 Vision 的：框的换算和夹边、分数取整和 0.3 的边界、排序；`ocr.json` 过 `crates/usuyo-contract/schema/ocr.schema.json`（测试里照 schema 自己查，机器上有 `/opt/homebrew/bin/jsonschema` 时再用它校一次）、和 contract 的样例 `ocr.images.json` 逐字段一样；参数；循环在管道上照 plugin-kit 的 `transform/tests.rs` 一条条走：`describe`、`run`、跑着时的 `cancel`、坏的和太长的帧、EOF、不理取消的 `run` 不拖住退出。
- 调 Vision 的：[`crates/usuyo-core/testdata/transform/`](../../crates/usuyo-core/testdata/transform/README.md) 的 `card.png` 认出四行，第一行的 `y` 小于最后一行；测试里画的空白图出 `no-text`、不写文件；一次两个输入出两项、下标对；读不了的图、越出暂存目录的路径、种类不对的输入被拒。
- 起编好的可执行文件，像宿主那样清空环境、在管道上说话：每一行 stdout 都是 JSON-RPC，`describe` 的版本和清单一样，`card.png` 出四行，关 stdin 以后正常退出。
- Vision 认中文时把汉字和数字之间的空格去掉，`card.png` 那一行出来是「水温 92°C，粉水比1:15」，所以比字时不看空白。

没验过的：竖排的字、手写、艺术字，以及 macOS 15.6.1 以外的系统（别的版本 Vision 认得的语言和分数可能不一样）。
