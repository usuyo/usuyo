<a id="拼思路的引擎设计草案"></a>
# 拼思路的引擎：设计草案

2026-11-11 · 调研和设计，没有改仓库。依据：`docs/PRD.md` 的 1–5 节和 7.4–7.8 节，`docs/SPEC.md` 的 1.2–1.4 节，`docs/plugin-protocol.md` 的 4.2–4.3 节，`docs/decisions/ADR-004-rust-tauri.md`，调研 01、03、04、05，open-decisions 1.8 节最后三行。

推荐一条路：**先用本机的词和向量把条目拼成「线头」，只摘原话、不写总结；你留下的线头就是一个主题；模型分析只在你点名时跑，每次是一份带引用、可校验的记录，和旧的并排放着。** 不建知识图谱，不做摘要树，应用里不放聊天。

<a id="0-从别家借什么避什么"></a>
## 0. 从别家借什么、避什么

- **NotebookLM**：借「每句话点得回原文」。避它的毛病：引用是生成时顺手带的，有人抱怨它随便引一页；资料一多就漏细节。所以这里引用由机器另行核对，见 [§2](#2-每一步怎么指得回去怎么重做)。
- **mymind**：借「不用自己整理」的感觉。避它的信任模型：它是 AI 替你分类、替你写总结。这里只摘原话，分组标「机器挑的」，你留下才算数（调研 01 的 3.2 节）。
- **Readwise Reader、Recall**：避收件箱、分拣、每日回顾、测验这类处理队列（调研 01 的 3.2 节）。线头没有「看过」状态。
- **Heptabase、Tana**：前者全靠你手动摆白板，后者要你先设计结构才好用。所以这里第一眼的东西不要求你先做任何事。
- **Obsidian Smart Connections**：借「读一条时旁边列相近的」。避它的两件事：第一次建索引要几十分钟，2026 年又改成了源码可见的许可证。这里的模型和格式都用开源许可，索引后台慢慢建。
- **Khoj**：2026-04 关了云服务，只留自托管。说明靠别家云端的个人知识库会断。这里文件和索引都在本机，模型由你选。

<a id="1-这一刻怎么出现"></a>
## 1. 这一刻怎么出现

**流水线**（导入以后自己跑，不出进度条，也不出数字）：

1. **切段。** 每条已有的文字层（原文、口播、图里的字、字幕、平台简介）切成段落大小的「段」。每段记三样：层、快照、原样坐标（秒的区间、`imageKey` 加框、文字片段），和现在的引用链接是同一套坐标。段的 id 由这几样加文字的哈希算出来。
2. **去掉平台腔。** 「建议收藏」「干货」「一定要看到最后」这类标题套话和平台水印字进停用表，否则会把不相干的条目拼到一起。
3. **建索引。** 仓库里的 `crates/usuyo-index` 已经用 tantivy 0.27 切出 jieba 词、单字加二字、拼音三种词，照用，只是把单位从条目改成段。有本机向量以后再加一路向量检索，两路用倒数排名融合（RRF，k=60）合起来。
4. **拼线头。** 每条算一个条目向量（它各段向量的加权平均；只有标题和简介的条目就用这些）。条目之间建「互为近邻」图，再做社区划分，分出一组组候选。一条长视频可以落进几组：它的段各自投票。
5. **排序。** 不按组大小排，按**跨度**排：来自几个平台、时间拉开多久、有没有你自己写过的字。「在几个地方、隔了很久、收下了同一件事」排最前，正是这一刻要的；一周里刷到的一堆同款排后面。
6. **配字。** 每组的名字用 c-TF-IDF 选三四个关键词，不让模型起名。每组挑 4–6 段原话：离组中心近、又互相不重复（MMR）。每段标平台、日期、位置，例如「B站 · 2024-03 · 口播 12:40」。

**你看到的**：地图照旧是雾，只有你关注的主题那片是清楚的。线头不算关注，在雾里只是微微发亮。旁边一栏列几条线头，每条写关键词、「小红书 · B站 · 公众号，2023-03 到 2026-10」，下面是几段能直接读的原话。整栏标「机器挑的」，沿用现在给机器挑的条目用的那个标签，不另起新词。

**你能做的**：点一段，从那一秒、那一段打开；**关注**这条，它就变成一个主题，成员就是你留下的那些条目（走现有的主题和 `topics` 字段，不加新对象）；关注之前拿掉不相干的条目；什么都不做也行。线头不是队列，不记「看过没有」，也没有「不感兴趣」按钮。不理的线头下次照样可能出现，不必为它存状态。

**第一次打开时转写还没跑完**：重任务只在接电、空闲时做，所以先用已经有的标题、简介、正文、平台字幕拼。小红书的正文和 B 站的字幕往往已经够用。视频号没有字幕、简介又短，它的条目要等口播转出来才进得了线头，在那之前只靠标题挂在别的线头边上。口播和图里的字转出来以后，线头悄悄变准，不提示，不闪。**位置稳定**：新条目只并进最近的已有组，整体重算只在空闲时做，新组和旧组按成员重合度对上号，沿用旧 id 和旧位置。

<a id="2-每一步怎么指得回去怎么重做"></a>
## 2. 每一步怎么指得回去、怎么重做

一条规则管放在哪：**免费、确定、能重算的放索引目录；花钱或结果不确定的写成资料库里的文件，带 meta；凡是要碰你那一层的，一律走 `.usuyo/changes/` 的改动提议。**

- **段**：放索引目录，从各层随时能重切。
- **向量**：`derived/embeddings/<模型id>@<维数>.bin`，旁边照现有约定写 `.meta.json`：`{by, version, at, model, chunker, inputs: [{path, sha256}], anchors: [{layer, snapshot, fragment, sha256}]}`。每个向量自带它那段的原样坐标和切段规则的版本，索引目录删了也对得上。换一个模型就多一个文件，旧的不删。和现在的 `embeddings.bin` 一样不同步。
- **同一层的第二次转写**（换了 ASR 引擎）：`derived/asr.vtt` 仍是这一层现在用的那份。上一份连同 meta 移到 `derived/runs/asr/<runId>/`，不删。引用链接里的 `asr` 指现在那份。时间类引用按秒找回，换引擎也不断；文字类按 SPEC 1.4.3 节写的模糊匹配找回，找不到就显示存下的原话。
- **线头**：放索引目录。输入是向量文件的哈希、参数和随机种子，同样的输入算出同样的组。
- **分析**（模型做的，见 [§3](#3-多个角度和支线)）：`.usuyo/runs/<runId>/` 放三样。`recipe.md` 是当时那份配方的原样副本，配方以后改了，旧的照样能读、能重做。`run.json` 写 `{lens, lensVersion, model, provider, temperature, seed, at, by, inputs: [{item, layer, snapshot, fragment, sha256}], parent?}`。`output.md` 里每一句带 `usuyo://item/...` 引用。同一条线换个角度或换个模型重跑，就是一个新的 `runId`，用 `parent` 指回上一份，并排列出，旧的不覆盖。这些文件同步，因为重跑要花钱。
- **引用链接只用原样坐标**，不用段 id 或 run id。这样重新切段、换模型，你页面上的引用都不会断。
- **条目清掉、设成私密以后**：分析里抄的原话是机器的字，不是你的，所以不像你页面里的引用那样留着原话。条目清干净时，引它的那几句连原话一起删掉，那份分析标「有来源已删掉」；`inputs` 里那一行照现在改动记录的做法换成 `{"redacted": true}`。设成私密的：引它的分析照留，但和条目一样不给 Agent、不进导出，以后重跑也不再把它的段送出本机。

**引用怎么校验**（每份分析写完立刻做，结果写进 `run.json`）：

1. **找得到**：层、快照都在，秒数在时长内，文字片段对得上。
2. **原话对得上**：引号里的字，规范化之后（NFKC、去标点、数字归一）是那一层那一处的子串。对不上就标「引文对不上」，这句不当引用显示。
3. **撑不撑得住**：用本机重排模型给「这句话、引的那段」打分，分低的标「引文和这句关系弱」。你也可以选让自己的模型复核。

校验只标出来，不替你删。这一步专治「NotebookLM 随便引一页」那类问题：引用和生成分开，由不会编的机器去核。

<a id="3-多个角度和支线"></a>
## 3. 多个角度和支线

**角度**（lens）是一份配方：提示词、输出的 JSON 结构、需要什么样的模型。输入永远是一组段，不是整个资料库。先做五种：

| 角度 | 做什么 | 要模型吗 |
|---|---|---|
| 时间上怎么变 | 这条线按收下时间排开，你最早在哪、最近在哪 | 不要 |
| 素材板 | 这条线里的图和关键帧按相似摆在一起 | 不要语言模型，要图向量（第四阶段，你打开才有） |
| 主张和论据 | 照 Claimify 的做法，只在意思确定时抽主张，每条附原话 | 要 |
| 分歧 | 找出说法相反的几段，并排放 | 要 |
| 还没覆盖的 | 列出这条线里没回答的问题 | 要 |

**默认开着的**：切段、索引、本机向量、线头、关键词、原话摘录、读一条时旁边列「相近的」，还有「时间上怎么变」。**只在你点名时跑的**：所有要模型的角度、用模型给段补上下文（Anthropic 的 contextual retrieval）、用模型给线头起名。摘要照旧默认关。

**不让它变成杂讯**：分析从不上地图；一条线上每种角度只露最新一份，旧的折起来，不删；机器写的句子一直标「机器写的」，进你的页面才算你的。

**支线**：在一条线上挑一部分条目，或者拿一份分析的某一段，点「从这里分一条」，就新建一个主题，和原来的主题共用成员。成员本来就是多对多，所以不加新对象，主题上只加一个 `from` 字段指回出发的主题（SPEC 改一处）。

**机器停在哪**：机器能做到改动提议为止，比如起草一张卡片、主题页的一段，每句带引用，你逐段接受或不要。你接受过的字机器不再改，成稿是你写的。对资料库提问用你自己的 Agent：MCP 现有的 `search` 改成返回带坐标的段，另加一个 `submit_analysis`，让 Agent 交一份结构化的分析（主张加引用）。宿主照 [§2](#2-每一步怎么指得回去怎么重做) 的三步校验，存成 `by: agent:<名字>` 的一份记录。这样有分析，但应用里没有聊天。

<a id="4-本机和云端"></a>
## 4. 本机和云端

**在 Rust 进程里跑的**（ADR-004 说检索和小模型可以和内核同进程）：

- 全文：仓库里已有的 tantivy 0.27、jieba-rs 0.11（MIT）、pinyin 0.11。
- 向量模型：**Qwen3-Embedding-0.6B**，Apache-2.0，0.6B 参数，MRL 支持 32–1024 维，存 512 维 int8。C-MTEB 聚类 68.74、检索 71.03，查询要加指令前缀。Q8 量化约 0.6 GB（按参数推算，待核实）。走 `fastembed` 7.7.0（Apache-2.0）的 `qwen3` 特性：这一路用 candle 后端，能读 GGUF，不经 ONNX Runtime。低配机备选 bge-small-zh-v1.5（MIT，24M）。
- 重排：bge-reranker-v2-m3（Apache-2.0，0.6B，F32 约 2.4 GB，量化后小得多，待核实），fastembed 里现成就有，但走 `ort`，也就是 ONNX Runtime。它第二阶段才进来，三个系统上怎么链接、安装包大多少，待核实。Qwen3-Reranker-0.6B（Apache-2.0，Q4 约 378 MiB）中文分数相近，要走 llama.cpp 绑定，到时一起比。
- 向量检索：十万到几十万段，int8 平铺存、mmap 加载、暴力扫一遍就够快。超过再换 usearch（Apache-2.0）。调研 04 当时选 sqlite-vec，是因为 Go 要一个不用 cgo 的驱动；换成 Rust 以后，平铺文件更简单，少一层 SQL。
- 分组：互为近邻图加社区划分，自己写，几百行；`hdbscan` 0.12（MIT 或 Apache-2.0）留作对照。
- 图向量（第四阶段，你打开才用）：Qwen3-VL-Embedding-2B，Apache-2.0，2B 参数、2048 维，能吃视频帧，显存约 4 GB（第三方数字，待核实）。轻的一档用 SigLIP 2 base（Apache-2.0，0.4B），只做图找图，它搜中文文字行不行待核实。
- **内存代价**：调研 04 当时让向量模型跑在 llama.cpp 旁路进程里，调研 05 也把「宿主进程里做重计算」列为反例。这里改成进程内，是因为 ADR-004 允许，而且少一个要分发、要看着的进程。代价是守护进程在嵌入时多占约 1–1.5 GB 内存（推断，待实测）。所以模型只在空闲时加载，跑完就放掉。真要常驻，再拆成旁路进程，写同样的文件，接口不变。

**要你选的模型的**（经插件协议 4.3 节写的宿主代发，密钥不出钥匙串）：所有要模型的角度、补上下文、起名、引用复核。模型可以是你本机的 Ollama 或 LM Studio，也可以是云端。另一条路是你自己的 coding agent 经 MCP 做，算力和费用在它那边。

**什么会离开这台电脑**：只有你点名的那条线里的那些段，加上配方。整库不出去，私密条目不出去，向量不出去。每次分析的记录里写明去了哪个端点，界面标「离开本机」。

**排除的**：

- GraphRAG、LightRAG：建索引在基准测试里要约 8000 万 token，实体图不是思路，中文实体抽取噪声也大。
- RAPTOR 的摘要树：用摘要替代原文，正是你说的杂讯。只借它分层的想法：线头可以分大小两级，但不写摘要。
- ColBERT 一类的逐词向量：存储大几十倍，在这个规模上重排已经拿到大部分好处。
- BERTopic 加 UMAP 的 Python 旁路：多一个运行时，UMAP 每次结果不一样，位置稳不住。
- jina-embeddings-v5：权重是 CC BY-NC，和内核的 AGPL 放在一起分发、别人拿去改都受限。
- EmbeddingGemma：Gemma 条款，多语言分数也低于 Qwen3-0.6B。
- LanceDB：为几十万个向量拖进 Arrow 和 DataFusion，不值。
- 应用内聊天，以及 mymind、Recall 那样自动给每条写总结：和 PRD 7.7、7.8 节冲突。

<a id="5-第一版做什么以后加什么"></a>
## 5. 第一版做什么、以后加什么

每一阶段，内核只多一样小东西，其余都是能换的零件。

| 阶段 | 做什么 | 内核和零件之间的接口 |
|---|---|---|
| **一（现在）** | 切段和坐标；现有的 tantivy 索引改成按段建；条目的关键词向量（jieba 词加平台腔停用表）；用 tantivy 的「更多相似」建图、拼线头、按跨度排、摘原话；关注变成主题；读条目时列「相近的」 | 段清单格式（内核定）；分组器是一个进程内的 trait：输入条目向量和元数据，输出组和成员分数 |
| 二 | 本机向量；两路检索加 RRF；向量取代关键词向量去拼线头；搜索加重排 | 向量文件格式加 meta；向量模型挂在转换接口后面，默认在进程内，也可以换成外部插件，写同样的文件 |
| 三 | 角度配方、`.usuyo/runs/`、引用三步校验、MCP 的 `submit_analysis`、支线 | 配方文件；宿主代发的模型调用；插件协议 4.2 节的 `run` 加一种「一组段」的输入。不新增第四类插件；真要新增，再写 ADR |
| 四 | 图和关键帧向量、素材板线头 | 同二，层名是 `image` 和 `keyframes` |
| 五 | 个人 Agent、computer use | 都经 MCP 和 ACP，只用 search、段、run、改动提议这几样，内核不动 |

阶段一只用 tantivy，就能先验证 [§1](#1-这一刻怎么出现) 的排序和摆法对不对；阶段二换上向量，界面不用改。

<a id="6-拿不准的"></a>
## 6. 拿不准的

以下实验都用小红书和视频号的真实条目做，它们是最先做的两个平台，视频号又最依赖转写。

1. **只靠词能不能拼出「这就是我」。** 我猜不够，所以第二阶段要早做，但没有证据。最便宜的实验：拿你真实导入的 2,000 条，用词和向量各拼 8 条线头，打乱顺序给你看，你只标「是我 / 不是」。
2. **按条目分组还是按段分组。** 一条 40 分钟的视频可能讲三件事。实验：挑最长的 100 条，两种分法各出一版，看长视频是被拆进几条线更对，还是整条放在一处更对。
3. **跨度优先还是数量优先。** 我猜「隔了很久还在收下」更像你在想的事，但也可能最近刷的一堆才是。实验：同一批线头按两种排序各列一份，问你哪份的前三条更像你。
4. **笔记本上跑不跑得动、转写噪声多大。** 几万条大约是几十万段。0.6B 模型每秒能嵌入多少段，在你的 Mac 上没量过：每秒 100 段的话要将近一小时，只在空闲时跑可以接受；每秒 10 段就得换小模型。可小模型不一定够：两张模型卡上的 C-MTEB 聚类分，bge-small-zh 是 44.18，Qwen3-0.6B 是 68.74，差得多，但测法不完全相同。实验：取 1,000 条，两个模型都计时、都拼一遍线头，你看哪版像你；同时看去掉标题、只用口播以后，线头是不是还站得住。

<a id="来源"></a>
## 来源

厂商页面和 SEO 评测站的数字只当参考，已经在正文里标了待核实。

- Qwen3-Embedding-0.6B 模型卡：<https://huggingface.co/Qwen/Qwen3-Embedding-0.6B>
- bge-m3 模型卡：<https://huggingface.co/BAAI/bge-m3>
- bge-small-zh-v1.5 模型卡：<https://huggingface.co/BAAI/bge-small-zh-v1.5>
- fastembed 7.7.0：<https://docs.rs/crate/fastembed/latest>
- tantivy-jieba 0.21.0：<https://docs.rs/crate/tantivy-jieba/latest>
- hdbscan 0.12.0：<https://docs.rs/crate/hdbscan/latest>
- Qwen3-VL-Embedding-2B：<https://huggingface.co/Qwen/Qwen3-VL-Embedding-2B>，<https://docs.voxel51.com/plugins/plugins_ecosystem/qwen3vl_embeddings.html>
- bge-reranker-v2-m3：<https://huggingface.co/BAAI/bge-reranker-v2-m3>
- SigLIP 2 base：<https://huggingface.co/google/siglip2-base-patch16-224>
- Qwen3-Reranker-0.6B GGUF（体积）：<https://huggingface.co/ggml-org/Qwen3-Reranker-0.6B-Q8_0-GGUF>，<https://huggingface.co/paulalesius/Qwen3-Reranker-0.6B-GGUF>
- jina-embeddings-v5-text-small（许可证）：<https://huggingface.co/jinaai/jina-embeddings-v5-text-small>
- GraphRAG-Bench（建索引的 token 数）：<https://arxiv.org/pdf/2506.02404>；什么时候该用图：<https://arxiv.org/pdf/2506.05690>
- Jina-ColBERT-v2：<https://arxiv.org/abs/2408.16672v1>
- BERTopic 增量建模：<https://github.com/maartengr/bertopic>
- Contextual retrieval 的数字（二手转述）：<https://www.maginative.com/article/anthropics-contextual-retrieval-technique-enhances-rag-accuracy-by-67>
- Claimify：<https://www.microsoft.com/en-us/research/?p=1134179>
- 引用核对（NLI、后补引用的问题）：<https://tianpan.co/blog/2026-04-23-rag-citations-post-hoc-rationalization>，<https://arxiv.org/pdf/2607.04223>
- NotebookLM 的抱怨：<https://www.toolworthy.ai/blog/notebooklm-alternatives>，<https://support.google.com/notebooklm/answer/16269187>
- Smart Connections 改许可证：<https://www.obsidianstats.com/plugins/smart-connections>
- Khoj 停掉云服务：<https://dev.to/jovan_chan_9500711396d4e6/khoj-review-2026-your-self-hosted-ai-second-brain-28mi>
- mymind 自动标签：<https://mymind.helpscoutdocs.com/article/80-will-the-automatic-ai-tags-get-better-over-time>，<https://alternativeto.net/news/2025/10/mymind-introduces-automatic-note-summarization-and-tagging>
- Recall 2.0：<https://www.recall.it/second-brain>
- Tana、Heptabase（竞品写的对比，偏颇）：<https://blog.saner.ai/tana-reviews>，<https://dev.to/pickuma/heptabase-review-visual-note-taking-for-thinking-through-problems-5daa>
- 嵌入式向量库对比：<https://www.actian.com/blog/developer/comparing-embedded-vector-databases/>

Readwise Reader、Recall 的队列做法取自仓库里的调研 01（`01-comparable-products.md` 的 3.2 节）。这次没查到可用材料的：Reflect、AnythingLLM、RAGFlow、Mem，正文没拿它们当依据。
