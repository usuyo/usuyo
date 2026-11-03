<a id="04--技术选型调研搜索ocrasr关键帧去重画布与桌面壳"></a>
# 04 · 技术选型调研：搜索、OCR、ASR、关键帧、去重、画布与桌面壳

状态：草案 · 2026-10-21 · 2026-11-03 审过：画布结论已进 [ADR-002](../decisions/ADR-002-frontend-stack.md)；「向量检索只用 sqlite-vec」「cjk 开 `output_unigram`」还没进 [SPEC §11](../SPEC.md#11-索引)；MyGo 已到 v0.7.4（2026-11-01），[§6.4](#64-桌面壳mygo-与回退路径) 和 [ADR-001](../decisions/ADR-001-stack.md) 的版本说法过时，壳的退出条件还没写成 ADR；主界面改为点阵迷雾地图后，渲染选型在原型上重看；[§7](#7-对歇存技术决策的启示) 末尾的待核实在落地前做。

> 里程碑口径已废弃：项目改为按完整功能一次性开发。文中的 M0–M3 只指写作时 SPEC 的分段，请按功能面阅读。

关联：[PRD](../PRD.md) · [SPEC §7 去重](../SPEC.md#7-去重) · [SPEC §11 索引](../SPEC.md#11-索引) · [ADR-001](../decisions/ADR-001-stack.md) · [feature 001](../features/001-m0-core-loop.md)

<a id="约定"></a>
## 约定

- 推荐度：★★★ 首选 · ★★ 备选或特定场景可用 · ★ 不建议。
- 日期：带 URL 的事实用来源自身的日期（发布日或页面日期）；页面无日期的写「查阅 2026-10-21」。
- 「待核实」：来自记忆或二手来源，未在官方页面确认，落地前必须实测或查官方文档。
- 本文只给输入，不改 SPEC / ADR。与现有决策的一致或冲突集中写在 [§7](#7-对歇存技术决策的启示)。
- 标识符、项目名、模型名保留英文。

<a id="目录"></a>
## 目录

1. [搜索索引与向量检索](#1-搜索索引与向量检索)
2. [OCR sidecar](#2-ocr-sidecar)
3. [ASR sidecar](#3-asr-sidecar)
4. [关键帧与画面描述](#4-关键帧与画面描述)
5. [去重](#5-去重)
6. [画布与桌面壳](#6-画布与桌面壳)
7. [对歇存技术决策的启示](#7-对歇存技术决策的启示)

<a id="1-搜索索引与向量检索"></a>
## 1. 搜索索引与向量检索

需求回顾：标题、正文、OCR、ASR、画面描述、摘要、笔记、标签全量可搜；中文二字词、拼音、语义相近；索引可删可重建；内核纯 Go、无 cgo。

<a id="11-全文索引候选"></a>
### 1.1 全文索引候选

| 候选 | 语言与分发 | 中文效果 | 许可证 | 最近发布 | 推荐度 |
|---|---|---|---|---|---|
| bleve v2 + `cjk` 分析器 | 纯 Go 库，进程内嵌入，索引目录可删可重建 | `cjk` = unicode tokenizer → `cjk_width` → lowercase → `cjk_bigram`。对类型为 Ideographic 的 token 逐字拆开再两两成 bigram；Latin / 数字 token 原样通过，不跨中英边界成对。`output_unigram` 默认 false：`中文字` → `中文`、`文字`，末尾单字不单独输出；夹在非中文之间的单字会作为 unigram 输出。无词典、无拼音、无语义 | Apache-2.0 | [v2.6.1 · 2026-08-24](https://github.com/blevesearch/bleve/releases) | ★★★（M0 已定；需加 `output_unigram: true`，见推荐） |
| bleve v2 + gse 分词 | gse 为纯 Go 的 jieba 移植；官方桥 `vcaesar/gse-bleve` 已归档（最后推送 2023-08-30，BSD-2），需自写几十行 bleve Tokenizer 包装 | 词典分词，长词、专名、搜索引擎模式召回更准；词典内嵌会增大二进制（具体 MB 待核实） | Apache-2.0 | [gse v1.1.0 · 2026-09-12](https://github.com/go-ego/gse)（2843★、15 open issues） | ★★（二字词召回不足时作为第二字段叠加） |
| SQLite FTS5（`modernc.org/sqlite`）+ `unicode_bigram` tokenizer | 纯 Go（C 转译），FTS5 已编译进 modernc；运行时不能加载 .so/.dylib 扩展；自定义 tokenizer 走 `hakazv/sqlite-fts5-bigram` 的 `driver/modernc` 适配（通过转译 ABI 传 Go 函数指针） | bigram 按相邻 Unicode 码点切分（`全文検索` → `全文`、`文検`、`検索`），大小写不敏感、NFC 归一；中英混排的 Latin 段如何切分文档未说明（待核实）；BM25 由 FTS5 内置；与 `jobs.db` 同一驱动 | SQLite 公有领域；modernc BSD-3；bigram MIT | [modernc v1.60.1 · 2026-09-29](https://pkg.go.dev/modernc.org/sqlite)（SQLite 3.53.4）；[bigram v0.4.0 · 2026-09-04](https://pkg.go.dev/github.com/hakazv/sqlite-fts5-bigram)（pre-v1） | ★★（M1 分叉候选，见推荐） |
| SQLite FTS5（`ncruces/go-sqlite3`） | 无 cgo，wasm 构建再转 Go；内置 fts5、`vec1`（SQLite 官方向量扩展，不是 sqlite-vec）等 21 个扩展 | 同 FTS5；无 CJK tokenizer（待核实）；每连接一个 wasm 沙箱，内存比 modernc 高 | MIT | [v0.35.6 · 2026-09-23](https://pkg.go.dev/github.com/ncruces/go-sqlite3) | ★ |
| FTS5 + C 扩展 tokenizer（如 `simple`） | C 扩展；需 cgo 或原生加载，modernc / ncruces 都做不到 | 中文分词 + 拼音一体，效果好 | MIT | 待核实 | ★（与无 cgo 冲突） |
| Tantivy（`anyproto/tantivy-go`） | Rust 库 + Go 绑定，需 cgo 与预编译静态库；anyproto 生产使用一年以上 | 内置 jieba（词典约 5MB，可裁掉） | MIT | [查阅 2026-10-21](https://github.com/anyproto/tantivy-go)（无 release 页，42★） | ★（cgo；进内核违反 ADR；做 sidecar 要自写进程） |
| Meilisearch sidecar | Rust 单二进制 HTTP 服务；release 资产 macOS arm64 约 348MB、Windows 约 353MB（待核实是否含符号） | charabia 用 jieba 分词 + 拼音归一；社区讨论反馈中文分词质量不足 | MIT | [v1.55.0 · 2026-10-12](https://github.com/meilisearch/meilisearch/releases) | ★（体积与常驻进程成本高于收益） |

来源：bleve `cjk` 分析器与 bigram 过滤器源码 [analyzer_cjk.go](https://github.com/blevesearch/bleve/blob/master/analysis/lang/cjk/analyzer_cjk.go)、[cjk_bigram.go](https://github.com/blevesearch/bleve/blob/master/analysis/lang/cjk/cjk_bigram.go)（查阅 2026-10-21）；gse-bleve [仓库](https://github.com/vcaesar/gse-bleve)（archived）；`fts5modernc` [包文档](https://pkg.go.dev/github.com/hakazv/sqlite-fts5-bigram/fts5modernc)（v0.4.0 · 2026-09-04）；ncruces [ext/README](https://github.com/ncruces/go-sqlite3/blob/main/ext/README.md)；Meilisearch 分词 [charabia](https://github.com/meilisearch/charabia) 与 [中文支持讨论 #503](https://github.com/meilisearch/product/discussions/503)。

<a id="12-向量检索候选"></a>
### 1.2 向量检索候选

| 候选 | 语言与分发 | 中文效果 | 许可证 | 最近发布 | 推荐度 |
|---|---|---|---|---|---|
| bleve vector（kNN + hybrid） | 需先编译 `blevesearch/faiss` 的 C API 共享库（`libfaiss_c`），再用 `-tags=vectors` 构建；go-faiss 为 C 绑定（cgo）；文档只覆盖 Linux / macOS，未提 Windows；bleve 版本与 faiss 检查点需一一对应 | 与全文同一索引，支持 RRF / RSF 混合排序 | Apache-2.0 | 同 bleve [v2.6.1 · 2026-08-24](https://github.com/blevesearch/bleve/releases) | ★（与「无 cgo」直接冲突） |
| sqlite-vec via `modernc.org/sqlite/vec` | 纯 Go（C 转译）；`import _ "modernc.org/sqlite/vec"` 即注册 auto-extension；内置 sqlite-vec v0.1.9；`vec0` 虚表，float / int8 / binary 向量；默认暴力 KNN（IVF、DiskANN 源码存在但未文档化） | 不涉中文；与 `jobs.db` 同驱动，一个文件即可承载 | sqlite-vec Apache-2.0 / MIT 双许可；pre-v1，有破坏性变更预期 | [sqlite-vec v0.1.9 · 2026-03-31](https://github.com/asg017/sqlite-vec)；[modernc vec 包](https://pkg.go.dev/modernc.org/sqlite/vec)（v1.60.1 · 2026-09-29） | ★★★ |
| chromem-go | 纯 Go、零第三方依赖；gob 文件持久化；暴力余弦；自报 1k 文档约 0.3ms、100k 文档约 40ms（2020 年中端笔记本）；无 ANN，HNSW 在 roadmap | — | MPL-2.0；beta | [v0.7.0 · 2024-09-01](https://github.com/philippgille/chromem-go) | ★★（最简；MPL 文件级 copyleft 需在内核 AGPL 下确认兼容） |
| usearch（Go 绑定） | C++ 库 + Go 绑定（cgo，待核实）；HNSW、多种量化、mmap 加载 | — | Apache-2.0 | [v2.26.5 · 查阅 2026-10-21](https://github.com/unum-cloud/usearch) | ★ |
| ncruces `vec1` | SQLite 官方 vec1 ANN 扩展，随 ncruces 内置 | — | MIT | [v0.35.6 · 2026-09-23](https://pkg.go.dev/github.com/ncruces/go-sqlite3) | ★（仅当整体切到 ncruces） |

来源：bleve [docs/vectors.md](https://github.com/blevesearch/bleve/blob/master/docs/vectors.md)（查阅 2026-10-21）；[go-faiss README](https://github.com/blevesearch/go-faiss)；modernc 支持 sqlite-vec 的 [Gorse 博文](https://gorse.io/posts/sqlite-vec)。

<a id="13-本地-embedding-的-sidecar-方式"></a>
### 1.3 本地 embedding 的 sidecar 方式

| 候选 | 语言与分发 | 中文效果 | 许可证 | 最近发布 | 推荐度 |
|---|---|---|---|---|---|
| llama.cpp `llama-server --embedding` + GGUF | C++ 单二进制，各平台预编译；`/v1/embeddings`（OpenAI 兼容）与 `/embedding`（可选归一化） | 取决于模型 | MIT | 随 llama.cpp 滚动发布（查阅 2026-10-21） | ★★★（默认 sidecar） |
| Ollama `/api/embed` 或 `/v1/embeddings` | Go 单二进制 + 模型管理；`/api/embed` 支持批量，输入可为 text / image / audio 对象 | 取决于模型 | MIT | [API 文档](https://github.com/ollama/ollama/blob/main/docs/api.md)（查阅 2026-10-21） | ★★（用户已装 Ollama 时直接复用） |
| 模型：Qwen3-Embedding-0.6B | 0.6B，1024 维（MRL 可 32–1024），32K 上下文；社区 GGUF 多个，官方 GGUF 待核实；查询侧需加指令前缀，否则检索下降约 1%–5% | C-MTEB Mean Task 66.33（4B 72.27、8B 73.84，自报） | Apache-2.0 | [HF 模型卡](https://huggingface.co/Qwen/Qwen3-Embedding-0.6B)（2025-06 发布） | ★★★ |
| 模型：bge-m3 | 约 568M（待核实），1024 维，8192 tokens，100+ 语种 | 中文检索社区常用基线；无本文可引的具体分数（待核实） | MIT | [HF 模型卡](https://huggingface.co/BAAI/bge-m3)（论文 2024-02-05） | ★★ |

来源：llama.cpp server [README](https://github.com/ggml-org/llama.cpp/blob/master/tools/server/README.md)；Ollama [OpenAI 兼容说明](https://docs.ollama.com/api/openai-compatibility)；Qwen3-Embedding [仓库](https://github.com/QwenLM/Qwen3-Embedding)。

<a id="14-推荐与风险搜索"></a>
### 1.4 推荐与风险（搜索）

推荐：

1. M0 维持 bleve + `cjk` 分析器，但构造分析器时设 `output_unigram: true`，否则单字查询（如「茶」）与奇数长度词的末字不可搜；拼音字段按 feature 001 用 `go-pinyin` 另建字段。索引大小会因 bigram + unigram 约翻倍，M0 验收时记录 `.xiecun/index/` 体积。
2. M0 后用真实库做一次混排实测（公众号正文 + 抖音 OCR/ASR 文本 + 英文术语），对比 `cjk` 与 gse 两种字段的召回；gse 只在二字词召回明显不足时作为第二字段叠加，不替换。
3. 向量检索（M1）选 sqlite-vec via `modernc.org/sqlite/vec`：纯 Go、与 `jobs.db` 同驱动、一个 import 即可；embedding 由 llama.cpp 或 Ollama sidecar 产出，模型默认 Qwen3-Embedding-0.6B，向量写入 `derived/embeddings.bin` 作为真相源，sqlite 向量表可重建。
4. 保留一条 M1 决策分叉：若 bleve 在混排或索引体积上不达预期，可把全文迁到同一 SQLite 的 FTS5 + `unicode_bigram`，全文与向量落在一个文件里；这是分叉而不是现在的推荐，因为 M0 feature doc 已批准 bleve，且 `hakazv/sqlite-fts5-bigram` 仍 pre-v1。

风险：

- bleve 近两年的发布几乎全是向量与 geo（v2.5–v2.6），CJK 分析器无变化；中文相关问题要靠自己写分析器解决，不要期待上游。
- bleve vector 被「无 cgo」排除，[SPEC §11](../SPEC.md#11-索引) 里「bleve vector 或 sqlite-vec」的二选一实际只剩后者。
- sqlite-vec 与 `sqlite-fts5-bigram` 都是 pre-v1；modernc 导入 `vec` 后不能再 `RegisterPageCache`。
- Qwen3-Embedding 的指令前缀在 llama.cpp embedding 接口下如何传递，官方 issue 未答复（待核实）；实现时在 sidecar 协议里把 query / document 两种输入分开。

<a id="2-ocr-sidecar"></a>
## 2. OCR sidecar

需求回顾：中文为主、混排英文；输出 `[{text, bbox:[x,y,w,h], confidence}]`；独立进程；安装体积可控；能无 Python 分发。

<a id="21-候选对比ocr"></a>
### 2.1 候选对比（OCR）

| 候选 | 语言与分发 | 中文效果 | 许可证 | 最近发布 | 推荐度 |
|---|---|---|---|---|---|
| PaddleOCR PP-OCRv6（官方） | Python 包依赖 PaddlePaddle；C++ 本地部署与 ONNX 导出有文档；三档 tiny 1.5M / small 7.7M / medium 34.5M 参数 | medium 相对 PP-OCRv5_server 检测 +4.6%、识别 +5.1%（官方内部集）；单模型覆盖中、英、日及 46 种拉丁文字；自报 CPU（OpenVINO）5.2×、Apple M4 6.1× 提速。PP-OCRv5 内部集：印刷中文 0.9013、手写中文 0.5807、繁体 0.7472（0–1 分） | Apache-2.0 | [PaddleOCR v3.7.0 · 2026-06-11](https://github.com/PaddlePaddle/PaddleOCR/releases/tag/v3.7.0)（91k★） | ★★（模型首选；运行时不用官方 Python 包） |
| RapidOCR（ONNX Runtime） | Python 主包；C++ / C# / Java 在独立仓库；默认中英；模型列表已含 `multi_PP-OCRv6_{det,rec}_{tiny,small,medium}`，引擎 onnxruntime / openvino / mnn / paddle / torch | 同 PP-OCR 模型；输出 box、text、score | Apache-2.0（捆绑模型亦 Apache-2.0，见 MODEL_LICENSES.md） | [v3.10.0 · 2026-10-08](https://github.com/RapidAI/RapidOCR/releases) | ★★★（跨平台默认 sidecar 的模型与参考实现） |
| PaddleOCR-json / Umi-OCR 式打包 | Windows C++ exe，stdout 输出 JSON，供其他程序调用；Umi-OCR 以它和 RapidOCR-json 为引擎做离线 OCR | 同 PP-OCRv4/v5 | Apache-2.0 | [PaddleOCR-json 最后推送 2025-04-07](https://github.com/hiroi-sora/PaddleOCR-json)（1555★） | ★（仅 Windows、更新慢；可作「exe + JSON」打包范式参考） |
| macOS Vision（`VNRecognizeTextRequest`，自写 Swift CLI） | 系统框架，零安装；现成 CLI `ocrit` 只输出文本，bbox 需自写 | `.accurate` 支持 zh-Hans / zh-Hant / yue；中文只能与英文配对；中文的 bbox 为行或片段级而非字词级（开发者论坛报告）；竖排不稳 | 系统 API；自写 CLI 可随内核 AGPL | macOS 14.4 的语言列表见 ocrit README；zh 最早支持版本待核实 | ★★（macOS 零体积快路径，坐标粒度粗） |
| Windows.Media.Ocr（`OcrEngine`） | WinRT API，需安装对应 OCR 语言包（功能按需包名形如 `Language.OCR~~~zh-CN~…`，具体名称待核实）；PowerToys Text Extractor 用它 | 结果分 line / word，word 带位置与尺寸；中文准确率无公开基准（待核实） | 系统 API | Windows 10+（查阅 2026-10-21） | ★★（Windows 零体积快路径） |
| Tesseract 5 + `chi_sim` | C++ CLI，各平台包管理器可装 | 本次未找到中文基准；英文发票测试中 PaddleOCR 零错、Tesseract 5.5 三错；旋转与噪声下明显落后；中文弱为社区共识（待核实） | Apache-2.0 | [5.5.3 · 2026-07-24](https://github.com/tesseract-ocr/tesseract/releases) | ★ |
| VLM 做 OCR（Qwen3-VL 等） | 见 [§4](#4-关键帧与画面描述) | Qwen3-VL 自报 OCR 32 语种、抗模糊倾斜；慢、重、坐标需额外 grounding | Apache-2.0 | — | ★（复杂版面兜底，不做默认） |

来源：PP-OCRv5 [官方介绍页](http://www.paddleocr.ai/main/en/version3.x/algorithm/PP-OCRv5/PP-OCRv5.html)（查阅 2026-10-21）；RapidOCR [模型列表](https://rapidai.github.io/RapidOCRDocs/main/model_list/)；Apple [Recognizing text in images](https://developer.apple.com/documentation/vision/recognizing-text-in-images)；[中文竖排与字级框问题帖](https://developer.apple.com/forums/thread/749234)（2024）；[ocrit](https://github.com/insidegui/ocrit)（BSD-2）；Microsoft [OcrEngine](https://learn.microsoft.com/uwp/api/windows.media.ocr.ocrengine)、[PowerToys Text Extractor](https://learn.microsoft.com/windows/powertoys/text-extractor)；[Umi-OCR](https://gitee.com/hbwei/Umi-OCR)；Tesseract 对比 [DEV 测试](https://dev.to/felipe_anyformat/best-python-ocr-libraries-179)（2026-03）。

<a id="22-推荐与风险ocr"></a>
### 2.2 推荐与风险（OCR）

推荐：

1. 契约先定、引擎可换：OCR sidecar 协议为 `<bin> ocr <image> --json`，stdout 输出 [SPEC §5](../SPEC.md#5-derived-文件约定) 的 `[{text, bbox, confidence}]` 加 `{by, version}`；内核只认协议。
2. 默认跨平台引擎：PP-OCRv6 small 或 medium 的 ONNX 模型 + ONNX Runtime。实现形式二选一，M1 定：(a) 把 RapidOCR 的 C++ 实现编成单一可执行文件（Windows / macOS / Linux 各一份）；(b) 用 Go 或 Rust 写一个 onnxruntime 薄壳。两者都不带 Python。ONNX Runtime 动态库 + 模型的体积（估 50–100MB）待实测。
3. 系统快路径：macOS 用自写 Swift CLI 调 Vision（约 100 行），Windows 用 `OcrEngine`；零体积、速度快，但坐标粒度为行，内核把它们标为 `by: "vision"` / `by: "winocr"`，并允许用户在设置里切到 PP-OCR 重算（SPEC 的 derived 可重算正好覆盖）。
4. 不选 Tesseract；VLM 只做人工触发的兜底。

风险：

- 任务要求调研 PP-OCRv5，但 2026-06 起 PP-OCRv6 已是现行版本且 RapidOCR 已发 ONNX；M1 的「留接口」应按 v6 的三档模型设计体积预算。
- PP-OCR 的内部评测集不是公开基准，手写与繁体分数仍明显低于印刷体；小红书图文帖里的手写体与艺术字需要自建一小组样本做回归。
- Vision 的行级 bbox 对「OCR 带坐标」的用途（画布高亮、点击跳转）够用，对逐词高亮不够；文档里要写清这是能力差异而非 bug。
- Windows OCR 依赖语言包是否已装；sidecar `health` 需报告缺包并给出安装命令。

<a id="3-asr-sidecar"></a>
## 3. ASR sidecar

需求回顾：中文口播（抖音、B站、视频号）为主；输出 WebVTT 分段带时间戳（[SPEC §5](../SPEC.md#5-derived-文件约定) `asr.vtt`）；独立进程；体积与许可证可接受；只在空闲且接电时跑。

<a id="31-候选对比asr"></a>
### 3.1 候选对比（ASR）

| 候选 | 语言与分发 | 中文效果 | 许可证 | 最近发布 | 推荐度 |
|---|---|---|---|---|---|
| whisper.cpp | C/C++ 单二进制，各平台预编译；后端 Metal / Core ML / CUDA / Vulkan / OpenVINO 等；`whisper-cli` 与 `whisper-server`（HTTP）；VAD 需另下 Silero 模型；词级时间戳为实验性（`-ml 1`），DTW 对齐选项待核实 | 模型 tiny 75MiB … large 2.9GiB，large-v3-turbo 有下载目标；Whisper-large-v3 在 Qwen3-ASR 自报对照里 WenetSpeech 9.86 / 19.11、AISHELL-2 5.06 WER；中文幻觉与标点问题为社区常见反馈（待核实） | MIT | [v1.9.5 · 2026-10-06](https://github.com/ggml-org/whisper.cpp/releases) | ★★（跨平台最成熟；中文非最优） |
| faster-whisper | Python + CTranslate2；带词级时间戳 | 同 Whisper 模型 | MIT | [v1.2.1 · 2025-10-31](https://github.com/SYSTRAN/faster-whisper/releases) | ★（Python 分发，不符合无 Python 目标） |
| FunASR llama.cpp runtime（SenseVoiceSmall / Paraformer-zh / Fun-ASR-Nano） | C++ 静态单二进制、运行时无 Python；预编译：Windows CPU / CUDA / Vulkan zip、Linux Vulkan tar；macOS 预编译未列出（待核实，可自编译）；`--srt` 输出；FSMN-VAD 分段；量化 q8 / f16（Nano 另有 q4km） | 中文为主训练：SenseVoiceSmall 234M（zh / yue / en / ja / ko，带情感与事件标签）、Paraformer-zh 220M（官方表注明「ASR + timestamps」）、Fun-ASR-Nano 800M（zh / en / ja + 方言口音）；FunASR 官方博客自称 CPU 上比 whisper.cpp 更准（厂商口径） | 代码 MIT；权重为 FunASR Model Open Source License（维护者在 SenseVoice issue #334 说明允许商用、需署名与模型名） | FunASR 1.4.16；runtime v0.2.6（日期待核实；[FunASR 仓库](https://github.com/modelscope/FunASR) 查阅 2026-10-21） | ★★★（中文默认 sidecar） |
| sherpa-onnx | C++ + ONNX Runtime；预编译 CLI（如 `sherpa-onnx-non-streaming-asr-x64.exe`）与 Linux / macOS / Windows 共享库；Go 绑定需 cgo（按 OS 拆成三个模块，内含预编译库）；支持 Paraformer-zh、SenseVoice、Whisper、Zipformer-CTC-zh、TeleSpeech 等 | 结果含 tokens / timestamps，是否逐模型可用待核实（Go 结果结构体字段未能在 pkg.go.dev 读到） | Apache-2.0 | [v1.13.8 · 2026-09-10](https://github.com/k2-fsa/sherpa-onnx/releases) | ★★★（一套 CLI 跑多模型；与 FunASR runtime 二选一） |
| Qwen3-ASR-0.6B / 1.7B + Qwen3-ForcedAligner-0.6B | Python（transformers / vLLM）、GPU 导向；无官方 GGUF；时间戳靠独立对齐器（≤ 5 分钟一段）；流式不出时间戳 | 52 种语言与方言（含 22 种中文方言）；1.7B 自报 WenetSpeech 4.97 / 5.88、AISHELL-2 2.71、KeSpeech 5.10 WER，全部低于 Whisper-large-v3 | Apache-2.0 | [2026-01-29 发布](https://github.com/QwenLM/Qwen3-ASR) | ★★（质量上限；本机 sidecar 条件苛刻，适合有 GPU 或走云端的用户） |
| macOS SpeechAnalyzer / SpeechTranscriber（`yap` CLI） | macOS 26 系统框架；`yap` CLI 输出 txt / srt / vtt / json，`--word-timestamps`；CC0 | 中文 locale 是否在 `supportedLocales` 待核实；一份自报中文会议数据 CER 约 34%–40% | 系统 API；yap CC0-1.0 | macOS 26；[yap](https://github.com/finnvoor/yap)（查阅 2026-10-21） | ★（零体积但中文质量未知） |

未调研：Windows 本地语音识别系统 API 的中文口播质量，留作备注。

来源：whisper.cpp [README](https://github.com/ggml-org/whisper.cpp)；FunASR [llama.cpp runtime README](https://github.com/modelscope/FunASR/blob/main/runtime/llama.cpp/README.md)；FunASR [模型选型博文](https://www.funasr.com/en/blog/which-funasr-model.html)（2026）；SenseVoice [仓库](https://github.com/FunAudioLLM/SenseVoice)；sherpa-onnx [Go API 文档](https://k2-fsa.github.io/sherpa/onnx/go-api/index.html)；Qwen3-ASR [技术报告](https://arxiv.org/html/2601.21337v2)（arXiv 2601.21337）；SpeechAnalyzer [addpipe 介绍](https://blog.addpipe.com/apple-speechanalyzer-api/)（2025）。

<a id="32-推荐与风险asr"></a>
### 3.2 推荐与风险（ASR）

推荐：

1. 协议：`<bin> asr <audio> --vtt`，stdout 输出 WebVTT；内核另记 `{by, version, model}`。音频抽取由 ffmpeg 先转 16kHz 单声道 wav（见 [§4](#4-关键帧与画面描述)）。
2. 默认引擎：中文口播首选 SenseVoiceSmall 或 Paraformer-zh，运行时选 FunASR llama.cpp runtime 或 sherpa-onnx 其一。倾向 sherpa-onnx：三平台都有预编译、一套 CLI 可切 Paraformer / SenseVoice / Whisper，且 Apache-2.0；FunASR runtime 的优势是 Fun-ASR-Nano 与 GGUF 量化，但 macOS 预编译未确认。M1 用同一段 10 分钟抖音口播样本对两者做 CER 与耗时实测再定。
3. 回退：whisper.cpp（large-v3-turbo 或 small），作为多语种与「用户已有 whisper 模型」的兼容路径。
4. 高质量可选：Qwen3-ASR 作为「云端 / 有 GPU」适配器，走 OpenAI 兼容接口（`qwen-asr-serve`），不做本机默认。

风险：

- 时间戳粒度：FunASR runtime 的 `--srt` 为 VAD 分段级，不是词级；SPEC `asr.vtt`「分段带时间戳」够用，但若后续要「点击字幕跳到词」需换 Paraformer 的 timestamp 输出或 Qwen3-ForcedAligner。
- 权重许可证：SenseVoice / Paraformer 权重不是 MIT，而是 FunASR 模型许可（允许商用、需署名）；歇存不商业化，但 README 与设置页要列出模型名与许可。
- sherpa-onnx 的 Go 绑定是 cgo；只能以独立进程形式用，不能 import 进内核（与 ADR 一致）。
- 体积：SenseVoiceSmall q8 约 250MB（估，待核实）、Paraformer-zh 约 220M 参数、ONNX Runtime 数十 MB（估，待核实）；whisper large 2.9GiB。首次使用时下载，不打进安装包。

<a id="4-关键帧与画面描述"></a>
## 4. 关键帧与画面描述

需求回顾：视频抽关键帧存 `images/keyframes/`；画面描述可选，写 `derived/vision.md` 并注明帧时间；ffmpeg 为独立进程；本地多模态模型通过 sidecar 调用。

<a id="41-ffmpeg场景切换与关键帧"></a>
### 4.1 ffmpeg：场景切换与关键帧

| 候选 | 语言与分发 | 中文效果 | 许可证 | 最近发布 | 推荐度 |
|---|---|---|---|---|---|
| `select='gt(scene,T)'` + `showinfo` | ffmpeg 内置；`scene` 为 0–1 的帧间变化分；社区经验阈值 0.3–0.5，调高则帧更少；`-vf "select='eq(n,0)+gt(scene,0.4)',showinfo" -vsync vfr` 可保证首帧并打印时间戳 | 与语言无关 | LGPL-2.1+ / GPL（依构建） | [FFmpeg 9.0.2 · 2026-09-18；8.1.3 · 2026-09-21](https://ffmpeg.org/download.html) | ★★★ |
| `scdet` 滤镜 | 专用场景切换检测；`threshold`（百分比，默认 10，建议 8–14）、`sc_pass`（只输出切换帧）；每帧写 `lavfi.scd.mafd`、`lavfi.scd.score`，切换帧写 `lavfi.scd.time` | 与语言无关 | 同上 | [FFmpeg n4.3.1 已含 vf_scdet.c](https://fuchsia.googlesource.com/third_party/ffmpeg/+/refs/tags/n4.3.1/libavfilter/vf_scdet.c)（2020） | ★★★（元数据更规整，优先于 `select`） |
| `-skip_frame nokey` 只解 I 帧 | 不解码全片，速度快；位置由编码器决定，不等于内容切换 | — | 同上 | — | ★★（长视频先粗抽，再对候选段跑 scdet） |
| `thumbnail` 滤镜 | 在 N 帧窗口中选最有代表性的一帧 | — | 同上 | — | ★★（封面图） |
| ffmpeg `whisper` 音频滤镜 | 需构建时 `--enable-whisper` 链接 whisper.cpp；`queue`（默认 3s，10–20s 更准）、`vad_model`、输出 text / srt / json | 同 whisper | 同上 | 补丁 2025-07，静态构建是否默认启用待核实 | ★（不替代 [§3](#3-asr-sidecar) 的 ASR sidecar） |

Go 调用惯例（不 cgo）：

- `os/exec` + `exec.CommandContext`（带超时），`ffprobe -v quiet -print_format json -show_format -show_streams` 取时长与流信息，解析为结构体；`ffmpeg` 的 stderr 按行解析 `showinfo` / `scdet` 输出。这是 Go 社区主流做法，不需要包装库。
- 包装库 `u2takey/ffmpeg-go`（ffmpeg-python 移植，Apache-2.0，2357★）最后推送 2024-05-24，60 个 open issues；只是参数拼接器，不建议引入。
- 二进制来源：macOS evermeet.cx、Windows gyan.dev / BtbN、Linux BtbN 静态构建；歇存不打包 ffmpeg，由 `make init` 检查 PATH 并提示安装，`health` 报告版本。

来源：ffmpeg-user [scene 阈值讨论](https://lists.ffmpeg.org/pipermail/ffmpeg-user/2012-November/011101.html)（2012）；`scdet` [补丁说明](https://lists.ffmpeg.org/pipermail/ffmpeg-devel/2020-May/262639.html)（2020-05）与 [vf_scdet.c 源码](https://ffmpeg.org/doxygen/7.0/vf__scdet_8c_source.html)；whisper 滤镜 [补丁线程](https://ffmpeg.org/pipermail/ffmpeg-devel/2025-July/346409.html)（2025-07）；[ffmpeg-go](https://github.com/u2takey/ffmpeg-go)。

<a id="42-本地多模态模型做画面描述"></a>
### 4.2 本地多模态模型做画面描述

| 候选 | 语言与分发 | 中文效果 | 许可证 | 最近发布 | 推荐度 |
|---|---|---|---|---|---|
| Qwen3-VL 2B / 4B / 8B（Instruct） | 官方 GGUF（如 `Qwen3-VL-8B-Instruct-GGUF`，Q4_K_M 5.03GB + mmproj）；llama.cpp `llama-server --mmproj`；Ollama `qwen3-vl`（需 Ollama ≥ 0.12.7） | 中文原生；OCR 32 语种；画面描述为主要用例之一 | Apache-2.0 | 2B / 32B 2025-10-21；4B / 8B 2025-10-15（[仓库](https://github.com/QwenLM/Qwen3-VL)） | ★★★（默认；8B 需 ≥ 8GB 可用内存，4B / 2B 给低配机） |
| MiniCPM-V 4.7（1B、35B-A3B MoE）/ 4.5（8B） | GGUF 与 Ollama 官方可用；4.5 基于 Qwen3-8B + SigLIP2 | 中文团队（OpenBMB），中英双语；PhotoPrism 对比中 4.5 的标签覆盖率 84% 高于 qwen3-vl:8b 的 75%（第三方测试） | Apache-2.0（页面称代码与权重皆为 Apache-2.0；另有单独「Model license」文档，条款待核实） | [4.7 · 2026-10-12](https://github.com/OpenBMB/MiniCPM-V) | ★★（1B 版适合低配机的粗描述） |
| Florence-2-large | 0.77B，ONNX 可跑；caption / OCR / detection 多任务 | 训练与输出以英文为主（待核实），不适合中文画面描述 | MIT | [HF 模型卡](https://huggingface.co/microsoft/Florence-2-large)（论文 2023-11） | ★（仅作英文标签 / 区域检测） |
| 云端 VLM 适配器 | 走 OpenAI 兼容接口 | 取决于服务 | — | — | ★★（用户自选；UI 标明数据离开本机） |

sidecar 接口：

- Ollama：`POST /api/chat`，`messages[].images` 为 base64 数组；或 OpenAI 兼容 `POST /v1/chat/completions`，`image_url` 只接受 base64 data URI，不接受 http URL。`/api/tags` 列模型，`/api/pull` 拉模型（可断点续传）。
- llama.cpp：`llama-server -m <gguf> --mmproj <mmproj.gguf>`，`/v1/chat/completions` 同 OpenAI；多模态标为实验特性；先查 `/v1/models` 的 `multimodal` 能力位。
- 内核只持有「OpenAI 兼容聊天端点 + 模型名」这一种配置；Ollama 与 llama.cpp 都满足，云端也满足。

来源：Ollama [API](https://github.com/ollama/ollama/blob/main/docs/api.md) 与 [OpenAI 兼容](https://docs.ollama.com/api/openai-compatibility)；Ollama 库 [qwen3-vl](https://ollama.com/library/qwen3-vl)、[minicpm-v4.5](https://ollama.com/library/minicpm-v4.5)；Qwen [8B GGUF 模型卡](https://huggingface.co/Qwen/Qwen3-VL-8B-Instruct-GGUF)；PhotoPrism [模型对比](https://docs.photoprism.app/developer-guide/vision/model-comparison/)（查阅 2026-10-21）。

<a id="43-推荐与风险关键帧与描述"></a>
### 4.3 推荐与风险（关键帧与描述）

推荐：

1. 关键帧两段式：先 `-skip_frame nokey` 抽 I 帧做候选并限速，再对整片跑 `scdet threshold=10`（短视频可直接 `select gt(scene,0.4)`），合并去重（相邻帧 pHash 距离 ≤ 5 视为同帧，见 [§5](#5-去重)），上限每分钟 6 帧、每条 60 帧，写入 `images/keyframes/<mm-ss-ms>.jpg`。
2. 画面描述默认关闭；开启时用 Qwen3-VL-4B / 8B 经 Ollama 或 llama.cpp，提示词要求中文、一句话 + 可见文字，逐帧写入 `vision.md` 并带帧时间；每帧一次请求，顺序执行，只在空闲接电时跑。
3. ffmpeg 不打包；sidecar `health` 报告 ffmpeg / ffprobe 版本并要求 ≥ 6.x。

风险：

- `scdet` 与 `select scene` 对抖音式快剪、转场特效会过检，阈值必须用真实样本调；上限策略比阈值更重要。
- VLM 8B Q4 约 5GB 下载、推理每帧数秒（PhotoPrism 测 Qwen3-VL-4B 推理版 6.4s / 张）；一条 3 分钟视频 18 帧约 1–2 分钟，可接受但必须后台。
- MiniCPM-V 的单独「Model license」条款未读到；选它之前先核实。
- Florence-2 不适合中文；若要英文标签作为补充索引字段再考虑。

<a id="5-去重"></a>
## 5. 去重

需求回顾（[SPEC §7](../SPEC.md#7-去重)）：内容层指纹 = 文本 SimHash（海明距离 ≤ 3）、图片 pHash、视频「时长 + 关键帧 pHash」；命中写 `sameAs`，不自动合并。

<a id="51-候选对比去重"></a>
### 5.1 候选对比（去重）

| 候选 | 语言与分发 | 中文效果 | 许可证 | 最近发布 | 推荐度 |
|---|---|---|---|---|---|
| 自写 SimHash（64 位，Charikar） | 约 100 行 Go；特征直接复用 [§1](#1-搜索索引与向量检索) 的 `cjk` bigram token 流，哈希用 FNV-64 或 xxhash | 以 bigram 为特征对中文更贴切；按词分特征的现成库对无空格中文失效 | — | — | ★★★ |
| `mfonda/simhash` | Go，MIT；`WordFeatureSet` 按空白切词；Kernighan 位计数 | 中文无空格 → 整段成一个特征，不可用；需自写 FeatureSet | MIT | [最后推送 2015-10-07](https://github.com/mfonda/simhash)（232★） | ★（可读源码，不引依赖） |
| `go-dedup/simhash` + `sho` 索引 | mfonda 的分叉，附 SimHash Oracle 做 k 位近邻查找 | 同上 | MIT | [最后推送 2017-09-04](https://github.com/go-dedup/simhash)（43★） | ★ |
| `ophymx/semblance` | SimHash + MinHash + LSH，`HammingIndex` 支持 ≤ 3 位精确查找 | 特征提取仍需自定义 | 待核实 | [pkg.go.dev](https://pkg.go.dev/github.com/ophymx/semblance)（查阅 2026-10-21） | ★★（库规模 > 10 万条再考虑索引） |
| `corona10/goimagehash` | 纯 Go；aHash / dHash / pHash 及 Ext 变体；依赖仅标准库 + x/image（待核实） | 与语言无关 | BSD-2-Clause | [v1.1.0 · 2022-05-26](https://github.com/corona10/goimagehash)；最后推送 2024-01-21（843★，13 open issues） | ★★★（稳定但停更；API 小，可 fork） |
| `kotylevskiy/go-phash`、`enot-style/go-phash` | 2026-02 新发的纯 Go 64 位 pHash | 与语言无关 | MIT | [kotylevskiy 2026-02-07](https://pkg.go.dev/github.com/kotylevskiy/go-phash)、[enot-style 2026-02-20](https://pkg.go.dev/github.com/enot-style/go-phash) | ★（太新、无用户；作为 goimagehash 的替补观察） |
| `kavu/go-phash` | 包装 C 版 pHash，cgo；上游 pHash 库已停更 | — | — | — | ★（排除） |
| 视频：时长 + 采样帧 pHash（SPEC 现方案） | 用 [§4](#4-关键帧与画面描述) 的关键帧，每帧 pHash，集合比较 | — | — | — | ★★★ |
| 视频：vPDQ（Meta ThreatExchange） | 每秒采样帧算 PDQ（256 位），可按距离剪枝；匹配参数：帧距离阈 D（示例 31）、质量阈 F（50）、匹配比例 Pc（80%）/ Pq；帧视为无序集合 | — | 见仓库（依赖 FFmpeg LGPL-2.1） | [vpdq README](https://github.com/facebook/ThreatExchange/blob/main/vpdq/README.md)（查阅 2026-10-21） | ★★（借其「比例匹配」思路，不引其实现） |
| 视频：`videohash`（Python，拼贴帧成 64 位） | 每秒一帧缩放拼贴后再哈希 | — | MIT | [PyPI](https://pypi.org/project/videohash/) | ★（思路可参考，Python 不引） |

阈值经验值：

- SimHash 64 位：Manku、Jain、Sarma（WWW 2007）在 80 亿网页上验证 k = 3 合理，[SPEC §7](../SPEC.md#7-去重) 的 ≤ 3 与之一致。来源：[论文 PDF](https://www.www2007.cpsc.ucalgary.ca/papers/paper215.pdf)。
- pHash 64 位：教程类来源给 ≤ 10「通常同图」、≤ 5「保守」、无关图约 32；没有找到严格的公开基准，必须用自己的样本调。来源：[pHash in Python 指南](https://mojoauth.com/security-guides/phash-in-python)（查阅 2026-10-21）。
- vPDQ 256 位用 D = 31（约 12%）作为帧匹配阈，换算到 64 位约为 ≤ 8，与上面的 5–10 区间吻合。

<a id="52-推荐与风险去重"></a>
### 5.2 推荐与风险（去重）

推荐：

1. 文本 SimHash 自写，特征用 `cjk` bigram（与索引同一分析器，保证同一文本在索引与指纹两侧切法一致），k = 3；库规模万级以内直接线性比较，超过再加 4 表分段索引（Manku 方案）或 semblance。
2. 图片 pHash 用 `corona10/goimagehash` 的 `PerceptionHash`（64 位）；阈值初始 ≤ 8，候选命中进入 `sameAs` 由用户裁决；对 `ExtPerceptionHash`（256 位）在小红书多图帖上做一次对比实验再决定是否升级。
3. 视频：时长差 ≤ 2s 为预筛，再对关键帧集合做 vPDQ 式比例匹配（≥ 80% 的帧有距离 ≤ 8 的对应帧）；不引外部实现。
4. 指纹写入 `manifest.fingerprints`，格式 `simhash:<hex16>`、`phash:<hex16>`、`dur:<s>|phash:<hex16>,...`，与 [SPEC §3](../SPEC.md#3-manifestjson) 已有字段兼容。

风险：

- goimagehash 两年多未推送；若 Go 版本或 x/image 升级出问题，需 fork 到组织下维护（代码量小）。
- 平台转码、加水印、裁切后的图片 pHash 距离会上升到 10 以上，漏检比误检更可能；UI 要提供「查找相似」的手动入口。
- 所有阈值在 M1 用真实库数据校准，并把校准样本放进 `testdata/`。

<a id="6-画布与桌面壳"></a>
## 6. 画布与桌面壳

需求回顾（[PRD §5](../PRD.md#5-功能范围)、[SPEC §2](../SPEC.md#2-资料库目录真相源)）：节点（素材 / 线索 / 念头）、连线、可重叠分组（一条素材可属于多个分组，没有唯一中心）、语义缩放、位置稳定；`canvas.json` 只存位置与视图状态；MyGo 壳、系统 webview、TS + Vite；产品默认 i18n。

<a id="61-画布库对比"></a>
### 6.1 画布库对比

| 候选 | 语言与分发 | 中文效果 | 许可证 | 最近发布 | 推荐度 |
|---|---|---|---|---|---|
| tldraw SDK | TS / React；npm `tldraw` | 文本框中文输入正常（待核实 CJK 字体） | 「tldraw License」，非开源：生产环境必须有 license key，无 key 五秒后停渲染；hobby 许可限非商用、保留水印且需 tldraw 审核；商用年费面谈；软件内含遥测用于许可核查 | [v5.5.2 · 2026-10-02](https://github.com/tldraw/tldraw/releases)；license key 要求自 v4.0 起 | ★（AGPL 项目不宜依赖专有许可 + 遥测） |
| Excalidraw（`@excalidraw/excalidraw`） | TS / React；手绘风格；元素 `groupIds: GroupId[]`（嵌套组，深到浅排序）、`frameId: string \| null`（单一 frame） | 中文正常 | MIT | [0.18.1](https://www.npmjs.com/package/@excalidraw/excalidraw)（查阅 2026-10-21；仓库 134k★） | ★★（组是选择组、frame 单父，都不是「可重叠分组」；风格与产品不符） |
| React Flow（`@xyflow/react`） | TS / React；节点为 DOM 元素；`parentId` 单父，子节点随父移动，`extent: 'parent'` 可限制出框 | 中文正常 | MIT（Pro 订阅只提供示例与支持） | [12.12.0](https://www.npmjs.com/package/@xyflow/react)（查阅 2026-10-21；仓库 38.7k★） | ★★（节点编辑器成熟；单父模型与数千 DOM 节点性能是硬限制，待核实） |
| Konva（+ react-konva） | Canvas 2D 场景图；多层 canvas；官方示例 10,000 形状拖拽、20,000 节点 | 文本用系统字体渲染，中文正常 | MIT | [10.9.0 · 2026-10-15](https://github.com/konvajs/konva/releases) | ★★★（自建画布模型的渲染层） |
| PixiJS v8 | WebGL / WebGPU 自动选择；8.20 以后面向 WebGPU 路径优化 | 文本需位图或 SDF 字体方案，中文字形集大是成本 | MIT | [v8.22.0 · 2026-10-01](https://github.com/pixijs/pixijs/releases) | ★★（数千节点 + 平滑缩放最快；文本与可访问性成本高） |
| 原生 Canvas 2D 自绘 | 无依赖；自写命中测试、脏矩形、视口裁剪 | 同 Konva | — | — | ★★（Konva 做的就是这层；除非要极小依赖） |

对「可重叠分组」的判定：tldraw / Excalidraw / React Flow 的分组或父子关系都是树（单父或嵌套），一个节点同时属于两个相交的分组在三者的数据模型里都不成立；要实现 PRD 的「一条素材可属于多个分组」只能自建数据模型，把库当渲染层用。这把选择收敛到 Konva / PixiJS / 原生 Canvas。

来源：tldraw [license key 文档](https://tldraw.dev/sdk-features/license-key)、[LICENSE.md](https://github.com/tldraw/tldraw/blob/main/LICENSE.md)、[v4.0.0 发布说明](https://tldraw.dev/releases/v4.0.0)；Excalidraw [element types.ts](https://github.com/excalidraw/excalidraw/blob/master/packages/element/src/types.ts)；React Flow [Sub Flows](https://reactflow.dev/learn/layouting/sub-flows)；Konva [10,000 形状示例](https://konvajs.org/docs/sandbox/Drag_and_Drop_Stress_Test.html)、[性能提示](https://konvajs.org/docs/performance/All_Performance_Tips.html)；PixiJS [v8 发布](https://pixijs.com/blog/pixi-v8-launches)。

<a id="62-canvasjson-与-json-canvas"></a>
### 6.2 canvas.json 与 JSON Canvas

| 候选 | 语言与分发 | 中文效果 | 许可证 | 最近发布 | 推荐度 |
|---|---|---|---|---|---|
| JSON Canvas 1.0（Obsidian 发起） | `nodes[]`（text / file / link / group）+ `edges[]`；节点有 id、x、y、width、height、color；group 有 label、background；edge 有 fromSide / toSide / toEnd / label；数组顺序即 z 序 | 与语言无关 | MIT | [1.0 · 2024-03-11](https://jsoncanvas.org/spec/1.0/) | ★★★（作为基础格式） |
| 自定义 JSON | 完全自定 | — | — | — | ★（放弃 Obsidian 互通） |

JSON Canvas 1.0 对歇存的两处缺口：

1. 没有 group 成员语义。规范只说 group 是「视觉容器」、把子节点放在边界内；Obsidian 的实际行为是几何包含（节点完全落在组框内即随组移动），一个节点同时落在两个相交组的行为未在帮助文档说明（待核实）。歇存需要显式成员关系：在 group 节点上加 `members: [nodeId]`，并允许同一 nodeId 出现在多个 group 里；为了 Obsidian 打开时大致可读，写文件时仍把成员节点的几何位置放进各自组框内（相交区域放共享节点）。
2. 没有视图状态字段（zoom、平移、语义缩放级别）。加顶层 `view: {x, y, zoom}` 与节点级 `x-xiecun: {kind, attention, lod}`；规范未定义扩展字段的处理方式，Obsidian 读取时是否保留未知字段待核实，所以 `canvas.json` 的真相源地位要以歇存自己为准，导出 Obsidian 版本时可丢弃扩展。

<a id="63-参考产品的画布交互"></a>
### 6.3 参考产品的画布交互

| 产品 | 与歇存相关的交互 | 状态（查阅 2026-10-21） |
|---|---|---|
| Heptabase | 卡片可放在多个白板上（同一卡片多处引用，待核实）；白板可嵌套（wiki 目录提到 nested whiteboards）；有 sections（分区，待核实其是否允许相交）；桌面 Mac / Windows / Linux + 移动端 | 在售；[官网](https://heptabase.com/)、[wiki](https://wiki.heptabase.com/) |
| Muse → Allume | 板中嵌板、板间可互链；明确「不做无尽缩放与旋转」，用进入 / 退出板替代缩放 | 2026 年更名 Allume，v4.0 在售；[allume.com](https://allume.com/) |
| Kosmik | 无限画布 + 内置浏览器剪藏 | 2026-05-31 关停，站点仍可下载导出；[kosmik.app](https://www.kosmik.app/) |
| Scrintal | 无限画布、嵌套内容、双向链接；桌面 0.2.13 | 在售但版本号早期；[官网](https://www.scrintal.com/) |
| Obsidian Canvas | group 几何包含、随组移动；双击改名、组背景图、「Jump to group」命令 | 在售；[帮助页](https://obsidian.md/help/plugins/canvas) |

对歇存的启发：若 Heptabase 确为「一张卡多个白板」，那就是 PRD 要的「一条素材多个分组」，只是它用多白板而不是同画布相交组实现；Allume 用进入板代替缩放，说明「语义缩放」可以是离散级别（缩略 → 标题 → 全文）而不是连续缩放，这与 PRD「位置稳定、不整体重排」相容。Kosmik 关停提醒：画布工具生态淘汰快，`canvas.json` 的可读性比库选择更重要。

<a id="64-桌面壳mygo-与回退路径"></a>
### 6.4 桌面壳：MyGo 与回退路径

| 候选 | 语言与分发 | 中文效果 | 许可证 | 最近发布 | 推荐度 |
|---|---|---|---|---|---|
| MyGo | 纯 Go、无 cgo，可从任一机器交叉构建；WKWebView / WebKitGTK / WebView2；自报约 7MB 二进制、带 webview 约 62MB 内存；`mygo.Bind` 生成 TS 客户端，支持 channel 流与事件；打包 .app / dmg / Windows 安装器 / deb、签名、公证、签名自动更新 | 0.5.0 加入 localization；`App.Locale` 返回显示语言 | MIT | [v0.6.0 · 2026-10-21](https://github.com/egoist/mygo/releases)；仓库 2026-09-26 创建，0.4.8（10-12）到 0.6.0（10-21）9 天 8 个版本；0.5.0 与 0.6.0 含破坏性变更（Windows WebView2 快捷键、`App.Locale`、`Shell.TrashItem`）；2496★、82 个 open items 多为 PR；贡献者 egoist 471 次、`claude` 79 次 | ★★（保留，但设退出条件） |
| Wails | Go + 系统 webview；v2 稳定、v3 仍 beta | — | MIT | [v2.17.0 · 2026-10-18；v3.0.0-beta.34 · 2026-10-20](https://github.com/wailsapp/wails/releases) | ★★（回退 1：Go 栈与绑定模型相近） |
| Tauri 2 | Rust 壳 + 系统 webview；sidecar 机制成熟；3.0 进入 alpha | — | MIT / Apache-2.0 | [tauri-v2.12.4 · 2026-10-14；3.0.0-alpha.5 · 2026-10-14](https://github.com/tauri-apps/tauri/releases) | ★（需引入 Rust 工具链） |
| Electron | 自带 Chromium；包体与内存最大但渲染一致 | — | MIT | [v44.9.3 · 2026-10-21；45 beta](https://github.com/electron/electron/releases) | ★★（回退 2：ADR 已列） |

MyGo 的事实核对（ADR 写「仍在 0.5.x」）：

- 项目极年轻（创建 2026-09-26）且发布节奏极快；近期 PR 集中在 native UI、终端、shader、文档等组件，没有看到 webview 相关 issue，说明 webview 路径既不是问题多发区也不是作者当前的投入区。
- 单一作者 + AI 协作提交的模式，总线因子为 1。
- WebView2 需要完整 Runtime（0.4.6 说明）；Linux 0.6.0 起可打包不含 WebKitGTK 的 native 版本，与歇存无关。

来源：MyGo [官网](https://mygo.egoist.dev/)、[README](https://github.com/egoist/mygo)、GitHub API（仓库元数据、贡献者、open issues，查阅 2026-10-21）。

<a id="65-i18n-惯例"></a>
### 6.5 i18n 惯例

| 候选 | 语言与分发 | 中文效果 | 许可证 | 最近发布 | 推荐度 |
|---|---|---|---|---|---|
| `nicksnyder/go-i18n` v2 | Go；消息文件任意格式（JSON / TOML / YAML 自注册 unmarshal）；CLDR 复数规则 200+ 语言；`goi18n extract` / `merge` 工作流；`text/template` 变量 | 中文无复数变化，规则简单 | MIT | [v2.6.1 · 2026-01-01](https://github.com/nicksnyder/go-i18n/releases) | ★★★（内核 CLI / MCP / API 错误文案） |
| `golang.org/x/text/message` + `gotext` | Go 官方；`fmt` 的本地化替代，`%m` 翻译占位；`gotext extract` 抽取到 `textdata/`，输出格式仍待完善 | 同上 | BSD-3 | [v0.43.0 · 2026-10-09](https://pkg.go.dev/golang.org/x/text/message) | ★★（数字 / 日期格式化可单独用） |
| i18next | TS 运行时字典；生态最大；v24 起 react-i18next 支持 tree-shaking | 中文正常 | MIT | [v26.5.3 · 2026-10-19](https://github.com/i18next/i18next/releases) | ★★ |
| Paraglide JS（inlang） | 编译期生成类型安全的消息函数，tree-shaking，仅打包用到的消息；适配 Vite | 中文正常 | MIT | [2.29.1](https://www.npmjs.com/package/@inlang/paraglide-js)（查阅 2026-10-21） | ★★★（前端） |
| MyGo 内建 localization | 0.5.0 加入，面向 native UI 组件 | — | MIT | 0.5.0 · 2026-10-14 | 待评估（web 前端用不到） |

惯例建议：消息 key 用英文稠密 id（`search.noResults`），源语言 zh-CN、第二语言 en；内核与前端各自一套目录（`internal/i18n/locales/*.toml`、`apps/desktop/web/messages/*.json`），由 CLI 的 `--lang` 与前端的 `navigator.language` 分别选择；日期与数字格式交给 `x/text` 与 `Intl`。

<a id="66-推荐与风险画布与壳"></a>
### 6.6 推荐与风险（画布与壳）

推荐：

1. 画布：自建数据模型（节点、连线、分组成员关系、注意力状态、LOD 级别），渲染层用 Konva（react-konva 或原生 Konva 皆可）；节点超过约 2,000 时再评估 PixiJS。不选 tldraw（许可 + 遥测）、不选 React Flow / Excalidraw（单父模型）。
2. `canvas.json` 以 JSON Canvas 1.0 为基础格式，加 `members`、`view`、`x-xiecun` 三类扩展字段；提供导出纯 1.0 的降级路径。
3. 语义缩放做离散三级（缩略 / 标题 / 内容），级别切换不改位置；参考 Allume 的进入板模式作为分组的「聚焦」动作。
4. 桌面壳维持 MyGo，但在 M3 开始前设退出条件：(a) 连续 60 天无破坏性变更或版本进入 1.x；(b) Windows / Linux 各有一个第三方 webview 应用公开发布；(c) 作者以外至少一名持续贡献者。不满足则回退 Wails v2（Go 栈不变）或 Electron（ADR 已列）。前端只通过本地 API 与内核通信的约束必须保持，壳可换。
5. i18n：内核 go-i18n、前端 Paraglide JS；首版就以 zh-CN / en 双语发布。

风险：

- Konva 的文本渲染在高 DPI 与大量节点下需要缓存（`cache()`）与分层，内存开销要测；中文长文本在 canvas 里不可选中，需要在编辑态切到 DOM 覆盖层。
- JSON Canvas 的扩展字段无规范保障，Obsidian 打开后保存是否丢字段待核实；把 `canvas.json` 视作歇存私有、Obsidian 兼容为导出能力。
- MyGo 的 webview 路径缺少公开的第三方生产案例；绑定代码生成（TS 客户端）在频繁破坏性变更下维护成本高。
- Paraglide 2.x 的框架适配变动较多（SvelteKit / TanStack 为主），纯 Vite + 自选框架的集成要先做个最小样例。

<a id="7-对歇存技术决策的启示"></a>
## 7. 对歇存技术决策的启示

与 [ADR-001](../decisions/ADR-001-stack.md)、[SPEC §11](../SPEC.md#11-索引)、[SPEC §7](../SPEC.md#7-去重) 的逐条对照。

一致之处：

- 「重计算作为独立 sidecar、不 import 进内核」被所有候选验证为必要：sherpa-onnx Go 绑定、tantivy-go、bleve vector、usearch 全部依赖 cgo 或原生共享库，只有以进程为边界才能守住「纯 Go、无 cgo」。
- bleve 作为 M0 全文索引仍成立；`cjk` 分析器的 bigram 行为与 [SPEC §11](../SPEC.md#11-索引)「CJK 双字分析器」吻合，拼音字段另建的做法与 feature 001 一致。
- [SPEC §7](../SPEC.md#7-去重) 的 SimHash 海明距离 ≤ 3 有 Manku 2007 的实证支撑；pHash 阈值 SPEC 未写数字，建议初值 ≤ 8 并校准。
- 「本地还是云端由用户选、标明数据是否离开本机」在 ASR / VLM 两处都对应到「OpenAI 兼容端点 + 模型名」这一种配置形态，内核不需要为每个引擎写适配。

冲突或需修订之处：

1. ADR-001「MyGo 仍在 0.5.x」已过时：仓库 2026-09-26 创建，今天发布 v0.6.0 并含破坏性变更，9 天内 8 个版本，单一作者。ADR 的「若届时不稳，回退 Electron」需要补成可执行的退出条件（见 [§6.6](#66-推荐与风险画布与壳)），并把 Wails v2.17.0 加为第一回退（Go 栈不变）。
2. [SPEC §11](../SPEC.md#11-索引)「向量检索候选：bleve vector 或 sqlite-vec」实际只剩一个：bleve vector 需要 FAISS 共享库与 cgo，被 ADR 排除；sqlite-vec 经 `modernc.org/sqlite/vec` 纯 Go 可用，且与 `jobs.db` 同驱动。建议 SPEC §11 直接写 sqlite-vec，并补一句 embedding 由 sidecar 产出、向量以 `derived/embeddings.bin` 为真相源。
3. [SPEC §11](../SPEC.md#11-索引) 的 bleve `cjk` 分析器要显式 `output_unigram: true`，否则单字查询与奇数长度词末字不可搜；这不改变决策，但要写进 feature 001 的实现要点与 `internal/index` 测试用例。feature 001 的「不理想则换 gse」回退要注意 `gse-bleve` 已归档，需自写 Tokenizer 包装。
4. 一个未在 SPEC 中出现的分叉：modernc SQLite 同时具备 FTS5（内建）、`unicode_bigram` tokenizer（第三方 pre-v1）与 sqlite-vec（内建），全文 + 向量可以落在 `.xiecun/` 下一个 SQLite 文件里，比 bleve 目录 + SQLite 两套存储简单。不建议改 M0，但建议在 M1 向量落地时对两套方案做一次体积与查询延迟实测后再定，并记一条 ADR。
5. PRD「可重叠分组」排除了最常见的三个画布库（tldraw、Excalidraw、React Flow）的内建分组模型，M3 的工作量应按「自建画布模型 + Konva 渲染」估算，而不是「接一个画布库」；同时 tldraw 的专有许可与遥测使它对 AGPL 项目不可用。CLAUDE.md「画布库待定」可以在此基础上收敛。
6. OCR 与 ASR 的「留接口」应按 2026 年的现状设计：OCR 目标模型已从 PP-OCRv5 变为 PP-OCRv6（三档 1.5M / 7.7M / 34.5M 参数，RapidOCR 已有 ONNX）；ASR 中文默认应为 SenseVoice / Paraformer 经 sherpa-onnx 或 FunASR llama.cpp runtime，而不是 whisper.cpp；两者的输出契约（bbox 行级 vs 词级、时间戳段级 vs 词级）要在 [SPEC §5](../SPEC.md#5-derived-文件约定) 注明可选粒度。
7. 许可证清单需要进入文档：权重层面 FunASR 模型许可（非 MIT）、MiniCPM-V 单独模型许可待核实、chromem-go 为 MPL-2.0；内核 AGPL-3.0 下引入 MPL 文件级 copyleft 库通常可行但要确认。

待核实清单（落地前必须验证）：

- bleve `cjk` 分析器对中英混排的实际召回与索引体积（真实样本）。
- `hakazv/sqlite-fts5-bigram` 对 Latin 段的切分方式；modernc 导入 `vec` 后的内存与启动开销。
- FunASR llama.cpp runtime 的 macOS 预编译与许可证；sherpa-onnx Go / CLI 结果中 timestamps 对 Paraformer / SenseVoice 的可用性。
- Vision 对 zh-Hans 的最低 macOS 版本；Windows OCR 中文准确率。
- MiniCPM-V「Model license」条款；Florence-2 语言覆盖。
- Obsidian 对 `canvas.json` 未知字段的保留行为。
- Meilisearch release 二进制体积是否含符号（约 350MB 的数字异常）。
