<a id="调研-08--只用英文的名字"></a>
# 调研 08 — 只用英文的名字

状态：草案 · 2026-11-01 · 2026-11-03 审过：推荐已被 [调研 09](09-fog-and-name.md) 取代，Guddle 等四个名字和 [§3](#3-命名标准第二版) 的第二版标准按第三版都排不进前 8（[09 §8](09-fog-and-name.md#8-上一轮的四个名字)）；[§1](#1-思鹭为什么不行)、[§2](#2-有梗的独立软件名字) 的约束和 [§8](#8-中文里怎么叫它)「只用英文时中文怎么叫」仍在用，其中靠吉祥物长外号的写法按 11-03「角色可换或不用」读；[§9](#9-获取插件组织的名字)、[§10](#10-待核实清单) 只对本篇的候选有效。

目的：上一轮（[调研 07](07-positioning-and-naming.md)）定下的「思鹭 / Seelu」在 2026-11-01 被用户否了：缺少梗、不够独立、像在蹭、容易被误解，也不喜欢这类中文名。这一轮只定一个英文名，不另起正式的中文名。结论放在 [§0 推荐](#0-推荐)；思鹭的问题和由此得到的约束在 [§1](#1-思鹭为什么不行)；有梗的独立软件名字有什么规律、中文用户怎么读英文名在 [§2](#2-有梗的独立软件名字)；据此改写的命名标准在 [§3](#3-命名标准第二版)；入围的 8 个名字在 [§4](#4-入围名单)，细查过的 18 个的可用性在 [§5](#5-可用性检查)，入围名的词义、中文读法、同名产品与商标信号在 [§6](#6-词义中文读法与同名产品)；全部 168 个候选在 [§7](#7-全部候选)；只用英文时中文里怎么叫它在 [§8](#8-中文里怎么叫它)；单独的获取插件组织怎么起名在 [§9](#9-获取插件组织的名字)；定名前还要查的在 [§10](#10-待核实清单)。名字定下之前，文中仍用「歇存」指这个产品。

<a id="约定"></a>
## 约定

- **查询日期**：所有查询都在 2026-11-01 做。本篇汇总的是流程里各 Agent 当天的查询记录，写报告时没有重跑；只有 [§0](#0-推荐) 推荐名和三个备选的 GitHub、npm 在 2026-11-01 复查过一次，见 [§5](#5-可用性检查)。
- **怎么查的**：
  - GitHub：`gh api users/<名>`（不分大小写，用户和组织共用一套命名空间），404 记为空着，200 记下账号类型和公开仓库数；同名仓库用 `gh search repos` / `gh api search/repositories`。每个名字查裸名、`<名>-app`、`<名>hq` 三种写法。
  - 包管理器：查注册表接口。npm 用 `registry.npmjs.org/<名>`，PyPI 用 `pypi.org/pypi/<名>/json`，crates 用 `crates.io/api/v1/crates/<名>`，Homebrew 用 `formulae.brew.sh` 的 formula 和 cask 接口。
  - 发行版同名包：Repology（含 AUR、nixpkgs 等），加 Debian / Ubuntu 的包文件列表。
  - App Store：iTunes Search API（`itunes.apple.com/search`，`country` 取 `cn` 和 `us`），看应用名里有没有这个名字。
  - 命令撞名：本机 `command -v` / `which`、`man`，加各包注册表里声明的可执行文件和 MCP 官方注册表。
  - 词义、同名产品、商标信号：网页搜索，加 Wiktionary、苏格兰语词典 DSL、Green's Dictionary of Slang、Urban Dictionary 等词典页，以及 Justia、CIPO 一类公开商标页面。
- **「空着」只表示查询时没被占用**，不保证能注册：GitHub 的 404 分不出没人用和被保留（`users/settings` 这类保留名也返回 404）；App Store 的名字要建应用记录时才知道能不能用。
- **商标没有做正式检索。** 文中的「商标信号」只是公开页面上看到的申请、注册和使用情况，不构成法律判断。
- **域名不查可注册性。** 用户用自己的一级域名，不买域名；文中提到的 .com 只作「有没有别人在用这个名字」的同名信号。
- **网页搜索有缺口。** 流程中途网页搜索额度用完，Wiktionary 也被限流过，所以部分词义、其他语言含义和商标搜索没做成，文中写「待核实」，缺的都列进 [§10](#10-待核实清单)。中文用户怎么念、会叫它什么，都是推断，没有找真人试读。
- **评审方式。** 168 个候选分 7 个方向提出（[§7](#7-全部候选)）。三位初筛评委（梗与独立感、英文与命令行、中文用户怎么读）各打 1–5 分，满分合计 15，也可以判「致命」；其中 18 个做了细查，每个给出绿灯、黄灯或红灯（上一轮没有绿灯，这一轮有 3 个）；三位终评评委（梗、角色与独立感；可用性与长期；中文用户与传播）各排前 8，按 Borda 计票：第 1 名 8 分，依次减 1，第 8 名 1 分，三人相加，满分 24。文中的「票数」都指这个分数。
- **推断与出处。** 事实尽量带出处；查不到的写「待核实」；没有出处、靠常识或类比得出的写「推断」。
- **只读。** 所有查询只读：没有注册、购买，没有提交任何表单，也没有 star 或 fork。

<a id="0-推荐"></a>
## 0. 推荐

**2026-11-02 补：** 用户没有对这一轮表态，讲了雾的概念（收了一大堆之后的脑雾）和新的起名要求。第三轮见 [调研 09](09-fog-and-name.md)；按第三版标准，本篇的 Guddle、Grepling、Fogdog、Butterbump 都排不进前 8（[调研 09 §8](09-fog-and-name.md#8-上一轮的四个名字)）。

**只用英文够不够：够用，而且比中英两套名字更合这次的口味；但要主动给中文外号留路。** 思鹭的毛病正是英文名寄生在中文名上，两套身份互相拖累；只留一个英文名，国风、「NPC 感」、中文谐音雷可以一起避开。代价是官方放弃对中文称呼的控制、中文平台上少一个现成的中文搜索词。补位做法是 README 中文版写一句汉字读音提示，并把吉祥物做成能用两三个汉字叫出来的具体角色，让社区自己长外号，详见 [§8](#8-中文里怎么叫它)。

**推荐：Guddle，命令 `guddle`。** 终评票数第一（22 票），梗、角色与独立感评委和中文用户与传播评委都排第一，可用性与长期评委排第三。

- 梗：苏格兰词，意思是伸手到溪边石头底下徒手摸鱼（Wiktionary）。英文名自己成立，不靠任何中文含义；README 一句「guddle = 徒手摸鱼」揭晓，中文里字面就是「摸鱼」，「上班 guddle 一下」「今天摸到一条好的」这类说法会自己长出来，和 Yazi 的「意思是鸭子」同一路。
- 角色：继续配 ASCII 鹭，鹭把喙探进石缝叼出一条字符鱼；静止时鹭和石头都清楚，一伸喙水面就散成字符。和 Clawd 螃蟹、Codex Pets、OpenClaw 龙虾都不撞。一个隐喻能顺下一套说法：`guddle add <链接>` 是「摸一条进来」，搜一句话跳到那一秒也是「摸」，收下的东西可以叫 catch。
- 中文用户怎么叫：官方只写一种读音「嘎豆」（英文 /ˈɡʌdl/）；会有人念成「咕嘟」「古德」，算预期的误读，不另列为正式读法。外号大概率是「摸鱼」（推断），这是名字的直译，不是另起的中文名。
- 可用性：npm、PyPI、crates、Homebrew、Repology、两区 App Store 全空，`guddle-app`、`guddlehq` 也空，没查到商标记录，本机和各发行版都没有同名命令。GitHub 三种写法和 npm 在 2026-11-01 复查过，结果不变（见 [§5](#5-可用性检查)）。GitHub 裸名 `guddle` 被一个仍有零星活动的个人账号占着，所以组织名要用 `guddlehq`（或 `guddle-app`），Go 模块路径是 `github.com/guddlehq/guddle`；命令、数据目录 `.guddle/`、环境变量前缀 `GUDDLE_`、URL scheme `guddle://`、插件 topic `guddle-plugin` 都不受影响。

**有没有硬伤：查过，没有。** 下面三点都是传播成本，不到换名的程度：

1. GitHub 裸名拿不到。任务里写的是「裸名最好」，不是硬要求；`guddlehq` 空着。
2. 「摸鱼」有两面：既是梗，也可能被理解成划水工具，和「把注意力放在自己看重的事上」有点反着。官方说法落在「只伸手摸自己要的那条」，不拿「上班划水」做宣传，梗交给社区（三位终评评委里两位明确这样建议）。
3. 搜索引擎会把 guddle 纠成 Google、huddle、cuddle，口头分享要报拼写；双写 d 容易漏打。早期靠 GitHub 和教程标题里同时出现「guddle 摸鱼」。

这些和思鹭那种「第一眼就读成死路 / SiLU / loo」的误解不是一类。

**备选**（按票数）：

1. **Grepling**（19 票，绿灯）：grep 加表示「小东西」的 -ling，鹭一动不动、一啄即中本身就是一次 grep，正好对上产品最能当面演示的那招「搜一句话跳到那一秒」。命名空间是 18 个里最干净的：GitHub 裸名、`-app`、`hq`、所有包、App Store、MCP 注册表全空，Go 路径可以直接写 `github.com/grepling/grepling`。短板是梗只在懂 grep 的人那里成立，小红书、视频号的普通用户读着就是「格瑞普林」，外号只能退到「小鹭」；和 Greptile 是同一种「grep + 后缀」造法；8 个字母；商标完全没查。
2. **Fogdog**（15 票，黄灯）：词典真词，「雾里亮着的一小片」（Wiktionary、dictionary.com），和点阵迷雾地图字面上就对得上；`fogdog fetch` 本来就是让狗叼回来，中文外号「雾狗」好念，能接上狗头文化。要把鹭换成狗；GitHub 裸名在一个 2014 年注册、2017 年后没再推送的个人号手里；加拿大有两件 FOGDOG 商标（体育用品零售、茶饮，不在软件类），fogdog.com 由品牌保护公司托管。
3. **Butterbump**（12 票，绿灯；和 Potoo 同票）：英格兰方言里大麻鳽（鹭科）的老叫法，叫声像雾笛，鹭线不用断；GitHub 裸名和所有包全空。中文外号几乎一定是「黄油包」，和 Bun 一样是个吃的，「养黄油包」顺口。问题是 10 个字母当命令偏长，中文可能简称成「黄油」，撞上 ACG 圈「黄油 = 成人游戏」（推断，证据只有一条评论）。同为 12 票的 Potoo 梗更强（林鸱瞪眼脸是成型的表情包），但 npm、PyPI、crates 被占、AUR 有同名二进制，还得放弃鹭，所以列在 Butterbump 后面。

<a id="1-思鹭为什么不行"></a>
## 1. 思鹭为什么不行

用户 2026-11-01 的原话：

> 「seelu我感觉缺少meme属性，不够独立，或者还是有点蹭和被误解，我记得有个叫思源的md笔记软件，然后感觉这种中文的太npc了不够有独立开发者的那种感觉，虽然比较简中互联网搞国风的那种，但是我并不喜欢这种风格，包括你文档调研的那些所有的名字，我在想其实只需要英文就够？然后中文感觉都挺怪的，然后能够中文和英文都做好的，反正我感觉挺难的」

上一轮的推荐和理由见 [调研 07 §0](07-positioning-and-naming.md#0-推荐)。逐条诊断：

**① 缺少 meme 属性。** Seelu 是一个拼出来的音，背后没有能画出来的东西，也没有能开玩笑的故事；它的意思得先讲「思 + 鹭」才成立，讲完也只是「想 + 一只鸟」，不好笑、不好转发。对照组：Ghostty 是幽灵 + tty，Yazi 就是「鸭子」，Bun 是朋友家兔子的名字，OpenClaw 被叫「龙虾」，部署它叫「养龙虾」。这里说的 meme 属性指三件事：名字指向一个具体可画的角色或物件；一句话来历本身有趣；能长出动词或黑话。

- 约束：新名字要能一句话讲完来历并让人想转述，指向具体角色或动作，能派生出「养 X」「X 一下」这类说法。

**② 不够独立。** see + lu 是英文词接拼音，读起来像给中文名「思鹭」套了一层英文壳：英文名的意义寄生在中文名上，离开中文就是一串音。鹭这只鸟又要靠中文意象撑场，英文读者感受不到。

- 约束：英文名自己成立，不依赖任何中文含义；不做「中文名 + 英文对音」的组合。

**③ 有点蹭。** 思源笔记 SiYuan 的名字取自「饮水思源」（官方文档），而且同样是本地优先的个人知识管理软件。思鹭和它撞了三层：同类目（本地优先、个人内容管理）、同格式（「思 + 单字」）、同首音（Si- / See-）。另外「思路」是常用词，思鹭读起来像在玩常用词的谐音。参考：Ghostty 1.0 改图标时，作者在 commit 里说新幽灵让它不再太像吃豆人（Pac-Man），连吉祥物的相似都会被当成借用。

- 约束：避开同类目已有名字的格式和首音节（Si- / See- / Note / Keep / Mind / Brain 等），避开常用词的谐音；吉祥物也要和 Clawd、Codex Pets、OpenClaw 龙虾拉开距离。

**④ 被误解。** 中文侧：Seelu / 思鹭读出来是 sīlù，和「思路」同音，和「死路」只差一个声调，中文用户会自动映射到这两个词，后者是负面的。英文侧：SiLU 是机器学习里的激活函数（Sigmoid Linear Unit），做 AI 的人看到 Seelu 会先想到它；loo 在英式英语里是厕所，see-loo 这个读法容易被拿来开坏玩笑（推断）。

- 约束：候选名同时查中文近音字（尤其死、屎、尸、输、丧这类）、英文俚语、AI/ML 术语缩写；第一眼的读法只有一种。

**⑤ 中文名「NPC 感」、像简中国风。** 双字雅词加自然意象（思、归、栖、隐、云、墨、观、鹭、鹤）是简中互联网国风产品、小程序、茶饮品牌的通用模板，像游戏里随机生成的 NPC 名字：好听，但没有作者。独立开发者的感觉来自私人、具体、有点不正经的来历（朋友的兔子、一只鸭子、tty 双关），而不是意境。用户也说上一轮调研的所有名字都有这个问题（上一轮全部候选见 [调研 07 §7](07-positioning-and-naming.md#7-全部候选)）。

- 约束：这一轮不起正式中文名；英文名不堆东方意象，来历要私人、具体、可以不严肃；中文外号留给社区去长。

<a id="2-有梗的独立软件名字"></a>
## 2. 有梗的独立软件名字

下表是这一轮拿来对照的软件名。名字后标「未核实」的，来历或吉祥物至少有一项没查到官方说明。

| 名字 | 来历 | 吉祥物 | 中文里怎么叫 | 为什么有梗 |
|---|---|---|---|---|
| Ghostty | Mitchell Hashimoto 写的终端。他在采访里说名字没什么深意：可爱、有好吉祥物、带 tty 致敬终端历史、暗合 ghost in the machine | 幽灵；1.0 时由 Michael Flarup 重画图标，作者说新幽灵不再太像吃豆人 | 多直接念 Ghostty，未见固定中文外号（待核实） | 一个词里同时有可画的角色和技术双关 tty，懂行的人会心一笑，不懂的也觉得可爱；作者用幽灵 emoji 签名，角色贯穿品牌 |
| Yazi | Rust 写的终端文件管理器，作者 sxyazi；README 原句 Yazi (means "duck") | 鸭子（Gitee 镜像中文名写作鸭子；图标样式待核实） | 名字本身是拼音，直接读出「鸭子」（推断） | 英文读者看到一个好念的词，中文读者一眼看出是「鸭子」，土、具体、有点好笑；拼音里没有 q、x、zh |
| Ollama（未核实） | 本地跑大模型的工具；名字和 Meta 的 Llama 模型相关（推断，官方说明未查到） | 羊驼形象（官方设定待核实） | 中文教程里直接写「Ollama, 羊驼」（CSDN） | 名字里藏着一只动物，中文用户顺着动物起外号，外号随教程传播 |
| OpenClaw（原 Warelay → Clawdbot → Moltbot） | 开源 AI 智能体；2026 年 1 月 Anthropic 以商标为由请求改名，Clawdbot 改为 Moltbot（molt = 蜕壳），因念着不顺口，1 月 30 日又改为 OpenClaw | 龙虾 Molty（早期叫 Clawd）；官方 lore：每次改名都是换新壳，「新壳，同一只龙虾」 | 「龙虾」，部署和使用叫「养龙虾」，因为图标是红色机械龙虾（IT之家等） | 吉祥物比名字稳，蜕壳把改名变成梗，长出一整套黑话和动词「养」；反面教训是 Clawdbot 太像 Claude，被要求改名 |
| Clawd（Claude Code 的吉祥物，未核实） | Claude Code 终端里的 8-bit 像素小螃蟹；Claude + claw 的说法是日文指南作者的推测，未见官方说明 | 像素螃蟹，主色约 #DA7758；v2.0.67 前后曾变成蓝色（GitHub issue） | Claude Code 常简称 cc（CSDN：「下文简称 cc」）；Clawd 本身的中文叫法待核实 | 像素角色住在终端里，开发者为它做 Slack 表情、改色 skill，一个小角色就能长出二创；和本产品的 ASCII 伙伴同类，要拉开距离 |
| Codex Pets | OpenAI 2026-05-01 给 Codex 桌面版加的动画伙伴，`/pet` 召唤，显示任务状态 | 内置 8 只；可用 `/hatch` 或 hatch-pet skill 孵化自定义宠物，本地存为 pet.json + spritesheet.webp | 待核实 | 「孵化」让用户自己造角色并分享，GitHub 上已有 codex-pets topic；伙伴的梗来自用户能参与 |
| Bun | Jarred Sumner 的 JS 运行时；朋友建议叫 bun，因为她有只兔子叫 bun，他起初拒绝，后来觉得说得通 | logo 是一个圆面包（推断）；名字同时指兔子和小圆面包 | 多直接念 bun（推断，有无「包子」之类外号待核实） | 三个字母、一个音节，来历是朋友的兔子：私人、不正经，正是独立开发者感 |
| Deno | Ryan Dahl 做的 Node 替代品，deno 是 node 的字母重排；「DEstroy NOde」的说法只见一篇博客，待核实 | 恐龙（通常认知，本轮未核实） | 多直接念 Deno（待核实） | 字母重排本身就是梗：作者用新项目「重排」自己的旧作；短、开音节结尾 |
| Zed（未核实） | Atom 原班人马用 Rust 重写的编辑器；名字来由未找到官方说明 | 无明显吉祥物 | 多直接念 Zed（待核实） | 单音节，短到像一个符号；梗不强，靠极短和速度定位 |
| Jellyfin（未核实） | 2018 年 Emby 停止开源后分出来的媒体服务器，名字由 Andrew Rabert 提出；「串流是水做的事」是社区说法，M→N 接在 Emby 后也只是猜测 | 水母形象（logo 说明未查到） | 是否叫「水母」待核实 | 用水里的生物接住「流」这个技术词，角色和功能之间有一层轻双关 |
| Immich（未核实） | 自托管照片和视频备份，Alex Tran 维护；名字来由未查到，「image」说法未证实 | 无明显吉祥物 | 待核实 | 造词，独特、好搜，但没有故事，梗弱；造词需要配来历才有梗 |
| Karakeep（原 Hoarder，未核实） | 书签 / 稍后读的自托管工具，2025 年 4 月从 Hoarder 改名，新名取自阿拉伯口语 karakeeb（杂七杂八的零碎），这两点见 v0.23.1 release note（2025-04-05）；改名原因是商标纠纷、作者找了几个月，出自 release note 链接的 Reddit 公告，2026-11-01 复查时 Reddit 返回 403，待核实 | 无明显吉祥物 | 待核实 | 常用英文词改名（据说因撞商标，待核实）；借少见外语词，既保留「一抽屉杂物」的意思，又独特可注册 |
| Raycast（未核实） | macOS 启动器；名字来由未查到，raycasting 是图形学术语（推断为借用） | 无明显吉祥物 | 待核实 | 技术行话做产品名，开发者一看就有感觉，外行没梗 |
| Obsidian | 本地 Markdown 笔记软件（本体闭源）；名字来由本轮未核实 | 紫色晶石图标 | 常叫「黑曜石」（CSDN、下载站）；「ob」简称待核实 | 英文词有现成中文译名，中文用户直接意译；好画好记，但意译会让名字在中文里变普通 |
| Excalidraw（未核实） | 手绘风白板；Excalibur + draw 是推断，未找到官方说明 | 无明显吉祥物 | 待核实 | 把传奇之剑塞进画图工具名，有点中二又好记（推断） |
| tldraw（未核实） | Steve Ruiz 的白板；Syntax 播客里说「tiny little drawing app」是后来补的说法，tl;dr 双关由播客标题暗示 | 无明显吉祥物 | 待核实 | 网络缩写 tl;dr 拼进产品名；连作者都承认来历是补的，名字先有梗，解释可以后补 |
| LocalSend | 跨平台局域网传文件；作者想用 Snapdrop 时它正好挂了，于是自己做一个不依赖第三方服务的 | 无明显吉祥物 | 多直接念 LocalSend（推断） | 反例：描述型名字一眼懂、好搜，但没梗，也不长外号 |
| Homebrew | Max Howell 2009 年写的 macOS 包管理器，取「自家酿酒」 | 啤酒杯图标；术语整套用酿酒词：formula、keg、cellar、rack、tap、bottle、cask | 多直接说 brew（推断） | 一个隐喻长出整套术语和命令（`brew install`），插件、数据目录一起命名时可学 |
| fish | friendly interactive shell 的缩写，启动问候语是 Welcome to fish, the friendly interactive shell | 鱼 | 多直接念 fish，口语叫「鱼」待核实 | 缩写刚好是一种动物，一个词同时是技术说明和角色；四个字母当命令自然 |
| Zellij（未核实） | 终端工作区；zellij 是摩洛哥马赛克瓷砖，词源是阿拉伯语（Wikipedia），取名原因项目没解释，「窗格像瓷砖」是推断 | 无明显吉祥物 | 待核实 | 外语实物词，好看独特；但 ij 结尾难拼难念，中文用户听了拼不出（推断） |
| goose（未核实） | Block 2025 年 1 月开源的 AI 智能体，发布时叫 codename goose，后转到 Linux 基金会下的 Agentic AI Foundation；取名原因未查到 | 鹅（推断，logo 未核实） | 待核实 | 常用动物词加 codename 前缀，像随手起的内部代号，反而不装 |
| Tampermonkey | 浏览器用户脚本管理器，直译「篡改猴」 | 猴子图标 | 「油猴」本是 Greasemonkey 的译名，Tampermonkey 流行后外号转到它身上；Violentmonkey 叫「暴力猴」 | 英文名长、难念，中文按动物加意译起了外号，外号还能在同类产品间迁移 |
| Shadowrocket | iOS 代理工具 | 火箭图标 | 「小火箭」（Wikipedia），V2EX 上还有人谐音叫「小火煎」 | 英文名长、图标具体，中文外号从图标长出来，比英文名传得更广 |
| 思源笔记 SiYuan（反例） | 名字取自「饮水思源」，寓意不忘来处（官方文档） | 无明显吉祥物 | 直接叫思源 | 思鹭撞上的对象：同为本地优先的个人知识管理，「思 + 单字」格式、Si- 首音都重合；成语式雅名正是用户这次不喜欢的风格 |

**规律：**

- 名字指向一个具体、能画出来的东西（幽灵、鸭子、兔子、龙虾、鱼、水母），吉祥物直接从名字里长出来，不用另起设定。
- 带一层技术双关：Ghostty 的 tty、Deno 是 node 的字母重排、fish 是缩写也是动物、tldraw 拼进 tl;dr。懂行的人会心一笑，外行照样能念。
- 来历是一句私人、不严肃的小故事：Bun 来自朋友的兔子；Hashimoto 说 Ghostty「没什么深意，就是可爱、有好吉祥物、带 tty」。不追求意境。
- 一个隐喻能长出整套术语：Homebrew 的 formula / keg / cellar / tap / bottle，OpenClaw 的蜕壳和龙虾黑话。插件、数据目录、命令跟着一起有梗。
- 能变成动词或动宾短语：`brew install`、`/hatch` 孵化宠物、养龙虾。用户在做的事有个好玩的说法，就会被反复说。
- 吉祥物比名字稳：OpenClaw 改名三次，龙虾一直在，「新壳，同一只龙虾」把改名本身变成梗。
- 借少见的外语或拼音实词（Yazi 中文、Karakeep 阿拉伯语、Zellij 阿拉伯语）：独特、可注册、少撞商标，但需要 README 揭晓含义，揭晓本身可以当彩蛋。
- 短：大多 1–2 个音节，元音结尾或单辅音结尾（Bun、Zed、Deno、Yazi、fish、goose），当全小写命令自然。
- 反例：描述型名字（LocalSend）一眼懂、好搜，但没梗、不长外号；纯造词（Immich）独特但没故事。

**中文用户读英文名的规则：**

- 双向都要测：中文用户会用英文规则和拼音规则各读一遍。Seelu 两边都读不稳，还会被拼音规则读成 sīlù；反过来 Qwen 说明拼音的 q、x、zh、c 对英文读者几乎不可读（英文社区要讨论才知道念 chee-wen）。
- 中文用户会自动把音节映射成汉字：每个候选名先写出最可能的两三种汉字近音，查负面字（死、屎、尸、输、丧、病）、粗口谐音和网络黑话。
- 同时查英文俚语和技术术语，尤其 AI/ML 缩写（SiLU 就是激活函数）和英式俚语（loo）。
- 难念的音：th、v / w 对立、r / l 对立、词尾辅音群（-sts、-lth、-rld）、不发音字母（gh、kn）。中文用户容易吞音或念成别的（推断）。
- 字母组合要只有一种读法：ee、oo、a 结尾稳定；c / k、g / j、ou / ough、结尾 e 发不发音容易念错；ij 结尾（Zellij）听到后拼不出（推断）。
- 两个音节、元音结尾最好念，也最好用汉字标音（Deno、Yazi、Ollama 式）。
- 英文名如果是现成常用词，会被直接意译成普通中文词（Obsidian → 黑曜石），独特性在中文里丢失；少见词或造词能保住名字本身。
- 中文聊天里要切输入法打英文，名字越短越好打；避开连字符和容易漏打的双写字母（推断）。
- 外号怎么长出来，见 [§8](#8-中文里怎么叫它)。

<a id="3-命名标准第二版"></a>
## 3. 命名标准（第二版）

上一轮的标准见 [调研 07 §3](07-positioning-and-naming.md#3-命名标准)。第二版按 [§1](#1-思鹭为什么不行) 的五条约束和 [§2](#2-有梗的独立软件名字) 的规律改写：

1. **有梗。** 一句话来历能让人笑或想转述；名字指向一个具体可画的角色或动作，能派生出动词和黑话（「养 X」「X 一下」），一个隐喻能顺带给插件、数据目录、命令起名。
2. **独立。** 英文名自己成立，不依赖任何中文含义；不是现有产品名的变体，不用同类目的前缀和格式（Si- / See- / Note / Keep / Mind / Brain、「思 + 单字」），不像 AI 创业公司，不是神话名。
3. **不被误解。** 中文近音字（尤其负面字）、英文俚语、AI/ML 术语缩写都查过；第一眼的读法只有一种。
4. **中英都好念。** 1–2 个音节为主，避开 th、v、r / l 对立、辅音群和不发音字母；中文用户听一遍能拼出来。不起正式中文名，README 中文版写一句汉字读音提示，外号留给社区。
5. **当一切标识都不难看。** 全小写 ASCII、无连字符、无数字，6 个字母上下为宜；同时当命令（不撞常见 CLI）、GitHub 裸组织名、插件 topic `<名字>-plugin`、Go 模块 `github.com/<组织>/<名字>`、数据目录 `.<名字>/`、环境变量前缀 `<NAME>_`、URL scheme `<名字>://`。
6. **有角色。** 能配一个点阵 / ASCII 伙伴，静止时清楚、动得快时散成字符；鹭可留可换，但要和 Clawd、Codex Pets、OpenClaw 龙虾拉开距离。
7. **定位不跑偏。** 不说「知识库」「第二大脑」，和「自己看重什么、雾、只增不改」不矛盾即可，不要求名字直接表达定位。

**常见的坑：**

- AI 创业公司腔：Mind、Brain、Cortex、Neuro、Synapse、Recall、Memo、Echo、Second Brain、以 ai 结尾；一看就像融资路演。
- 后缀模板：-ify、-ly、-io、-hub、-base、-verse、-flow，以及去元音拼法（Flickr 式）。
- 希腊罗马、北欧神话名：Mnemosyne、Athena、Hermes、Odin……大量被占，也显得自负。
- 常用英文词：Keep、Notes、Fog、Mist、Haze、Atlas、Lens、Haven。GitHub 裸名早被占、搜不到、易撞商标，Hoarder 改名 Karakeep 据说就是因为商标纠纷（原因待核实，见 [§2](#2-有梗的独立软件名字)）。
- 同类目前缀和格式：Note-、Keep、Mind-、Brain-、Si- / See-，以及「思 + 单字」。思鹭撞思源就在这里。
- 给中文名配英文壳、英文 + 拼音混搭：英文名寄生在中文含义上，两边都不独立（Seelu 本身）。
- 国风雅词模板：思、归、栖、隐、云、墨、观、见、鹭、鹤 + 一个字，像 NPC 名。
- 拼写难猜：听到拼不出、看到念不出，或故意错拼，分享时要反复解释。
- 撞别人的角色或品牌：Clawdbot 太像 Claude 被要求改名；Ghostty 改图标避开吃豆人。ASCII 伙伴要和 Clawd、Codex Pets、OpenClaw 龙虾、ASCII 水母类作品拉开。
- 需要先讲中文含义才成立的名字：对英文读者是一串音，对中文读者是一句雅话，梗只在解释之后出现。
- 负面标签词：hoarder 对应囤积障碍这类心理学标签（推断；Karakeep 改名的原因据 Reddit 公告是商标而非此，待核实）。
- 破坏标识的拼法：大写、连字符、数字、过长，`<名字>-plugin`、环境变量前缀、URL scheme 都会变难看；撞已有常见 CLI 命令也要查（这一轮撞上的有 dotty、scry、nah、steep、crane、noodle、lasso 等，见 [§7](#7-全部候选)）。

<a id="4-入围名单"></a>
## 4. 入围名单

入围的是终评票数前 8 名，正好都没被判红灯。另外 10 个细查过的名字（4 个黄灯、5 个红灯、1 个和「只定英文名」冲突的拼音名）列在本节末尾的总表里。按票数排列，可用性明细见 [§5](#5-可用性检查)，词义和同名产品见 [§6](#6-词义中文读法与同名产品)。上一轮的入围名单见 [调研 07 §4](07-positioning-and-naming.md#4-入围名单)。

**Guddle**：命令 `guddle`；一个小动作方向，22 票，绿灯，推荐。

- 含义：苏格兰词，动词，伸手到溪岸边、石头底下摸，徒手把鱼抓上来（Wiktionary）；in a guddle 还指「一团乱」（DSL 已证实）。
- 角色：继续配 ASCII 鹭。鹭把喙探进石缝叼出一条字符鱼；静止时鹭和石头都清楚，一伸喙水面就散成字符。设计时要避开石头底下冒出螃蟹的画面，只留鹭和鱼。
- 梗：README 一句「guddle = 徒手摸鱼」，中文梗自己长出来：「上班 guddle 一下」「今天摸到一条好的」；`guddle add <链接>` 读作「摸一条进来」，搜一句话跳到那一秒也是「摸」；英文侧 just guddle it。获取插件可以叫 stones（鱼躲在石头底下），收下的东西叫 catch。
- 中文读法：官方写「嘎豆」（/ˈɡʌdl/）；预期误读是「咕嘟」「古德」。查过死、屎、尸、输、丧、病，没有近音。外号大概率是「摸鱼」（推断），这是名字的直译。
- 风险：GitHub 裸名被仍有零星活动的个人账号占着，组织名要用 `guddlehq`；搜索引擎会纠成 Google、huddle、cuddle，口头要报拼写；双写 d 易漏打；「摸鱼」可能被读成划水工具；Urban Dictionary 有几条轻度性暗示（赞踩都是 0）；AlternativeTo 上有一个同名的 PRINCE2 项目管理工具，还在不在运营待核实。
- 评委怎么看：初筛 12 分（梗与独立感 4、英文与命令行 4、中文 4）。终评：梗、角色与独立感评委排第一，说它是唯一一个名字、角色、动词、中文梗四样都从同一个词里长出来的，来历私人、具体、不正经；中文用户与传播评委排第一，说外号来自名字本身，不用官方钦定；可用性与长期评委排第三，扣在裸名被占和搜索纠错。两位评委都建议官方只写一种读音，不拿「划水」做宣传。

**Grepling**：命令 `grepling`；终端里好看的造词方向，19 票，绿灯，备选第一。

- 含义：grep（按行搜文本的 Unix 命令）加 -ling（duckling、gosling 那种表示「小东西」的后缀），一只会 grep 的小家伙。
- 角色：继续配鹭，而且更顺：鹭捕鱼本身就是一次 grep，一动不动站在水里，看准一条一啄即中。点阵雏鹭，啄下去那一瞬散成字符再聚回来。
- 梗：「grepling 一下」= 搜那句话；表情包是雏鹭叼着一行字；-ling 顺带让人想到 gremlin。插件可叫 bait，每条收下的是一条「鱼」（推断，可选黑话）。
- 中文读法：「格瑞普林」，读法稳；近音没有负面字，-ling 和「灵」同音，偏正面。外号可能是「小鹭」「grep 鸟」（推断）。
- 风险：8 个字母，超出 6 个上下的目标；grep 对不写代码的人是生词，梗只在开发者圈成立；和 Greptile（融资 2500 万美元的 AI 代码审查公司）同一种造法，可能被当成它的周边（推断）；容易拼成 grapling、greppling；商标完全没查。
- 评委怎么看：初筛 11 分（5、3、3）。可用性与长期评委排第一：命名空间最干净，造词五年后搜出来基本只有它自己。梗、角色与独立感评委排第二，扣在和 Greptile 同法、外号只能落到「小鹭」。中文用户与传播评委排第五：产品默认中英双语，面向的不只开发者。

**Fogdog**：命令 `fogdog`；雾与浅水方向，15 票，黄灯，备选第二。

- 含义：词典真词，雾中看见的一道光、雾层里的一块亮斑，仿 sundog（幻日）造词（Wiktionary、dictionary.com 记 1865–70 年首见）；航海里还指海雾中的一块晴空缺口。
- 角色：换成狗：一只由雾点组成的狗，坐着不动时轮廓清楚，一跑起来散成字符；鹭可以当配角。要和 Datadog 的狗拉开（推断）。
- 梗：「养雾狗」「遛雾狗」；`fogdog fetch` 本来就是让狗叼回来；插件可叫 bones / treats；中文狗头表情文化能直接接上。
- 中文读法：FOG-dog 两个闭音节，听一遍能拼出来；外号推断是「雾狗」「狗子」，近音「五狗」「福狗」偏吉利。「狗子」太泛，单说分不出是哪只（推断）。
- 风险：得把鹭换成狗，狗是最常见的吉祥物；带 fog 原词，用户上一轮看过 fogcat、fogwick，可能觉得是同一种拼法；GitHub 裸名在一个 2014 年注册、2017 年后没再推送的个人号手里；加拿大有两件在效的 FOGDOG 商标（体育用品零售、茶饮），fogdog.com 由 MarkMonitor 托管、三重锁定。
- 评委怎么看：初筛 10 分（3、4、3）。中文用户与传播评委排第三，可用性与长期评委排第四，梗、角色与独立感评委排第五：意思最贴，但更像一句诗意描述，缺私人小故事的不正经感。

**Butterbump**：命令 `butterbump`；名字就是角色方向，12 票，绿灯，备选第三。

- 含义：英格兰方言里对大麻鳽（鹭科）的老叫法；butter 来自 bittern 的旧写法，bump 指雄鸟夜里像雾笛、像公牛一样的低沉「轰」声（Spectator「Dot Wordsworth」专栏、Merriam-Webster 标为 dialectal, England）。「诺福克」「1671 年」两说只有二手出处，对外宜写「英格兰方言」。
- 角色：换成大麻鳽，仍在鹭科：受惊时嘴朝天、身体拉直装成一根芦苇（推断），叫声像雾笛，正好接上迷雾；ASCII 画一根竖条加一个尖嘴。
- 梗：中文外号几乎一定是「黄油包」，和 Bun 一样是个吃的，「养黄油包」顺口；收进来一条它就「轰」一声；对程序员 bump 是升版本，在论坛是顶帖。
- 中文读法：没人会按「巴特邦普」叫，会直接叫「黄油包」；没有负面汉字谐音。
- 风险：10 个字母，命令、`.butterbump/`、`BUTTERBUMP_` 都偏长，多半要靠别名；中文可能简称「黄油」，撞 ACG 圈「黄油 = 成人游戏」（推断，证据只有一条 Steam 评论）；英文 butt 开头会被拿来开玩笑；同一只鸟的俗名 bogbumper 里 bog 是英式俚语的厕所，别选那个。
- 评委怎么看：初筛 10 分（5、2、3）。可用性与长期评委排第二：和 Grepling 一样干净，扣在长度。梗、角色与独立感评委排第四：来历最有故事，但梗不好控制。中文用户与传播评委没排进前 8，理由就是「黄油」。

**Potoo**：命令 `potoo`；名字就是角色方向，12 票，黄灯。

- 含义：林鸱。遇到危险会「定住」：身体伸长、嘴朝天，像一截断掉的树枝（Wikipedia）。
- 角色：换成林鸱，放弃鹭：一动不动时是一根枝条，动起来才露出大眼睛；ASCII 是一根竖线加「◉ ◉」。
- 梗：林鸱瞪眼照 2013 年上 Reddit、2014 年起被做成梗（Tetzoo 2019《Potoos As An Internet Phenomenon》）；「装树枝」= 不想被打扰；「养土豆」。
- 中文读法：「波兔 / 破土」，外号可能是「土豆」（像 potato），会想到土豆网；也可能听成「破图」，对存图存视频的工具不吉利；正式译名「林鸱」的鸱字多数人不会念。
- 风险：梗是这只鸟自带的，不是名字和产品之间的双关，借的是别人的热度；PyPI、crates 被小项目占，AUR / PyPI 有个叫 potoo 的终端幻灯片工具；韩国 CodiMe 有一个在运营的 AI SaaS 叫 Potoo；西班牙语 poto 在拉美部分地区是「屁股」。
- 评委怎么看：初筛 10 分（3、4、3）。梗、角色与独立感评委排第三，可用性与长期评委排第五，中文用户与传播评委排第七。

**Holdout**：命令 `holdout`；流外的立场方向，10 票，黄灯。

- 含义：不肯随大流的人或物。Wikipedia「Holdout (real estate)」里有「Nail house」一节，就是中文的「钉子户」；机器学习里 holdout data set 是不拿来训练的留出集。两层合起来：推荐流里的钉子户，存的东西是一份不喂给算法的留出集。
- 角色：一只单脚站在原地不挪窝的鹭，周围的点阵像工地一样流过去，被推得快了散成字符，停下又聚回原位。
- 梗：「当钉子户」= 不跟推荐流走；「我的收藏是 holdout set，不给模型训练」；表情包是四周推土机、中间一只鹭，配字「不搬」。
- 中文读法：「霍道特」，一种读法；外号「钉子户」「hold 住」都有梗；钉子户牵连强拆，偶尔敏感（推断）。
- 风险：holdout.com 正在做一个自称开源、带插件生态、数据归用户、反对少数公司垄断的个人 AI 平台，还画了「你自己的资料库」概念，立场几乎同路，谁后火都会被当成借用；常用词，2026 年 AI 评测圈也大量在用；GitHub 裸名被 2016 年起休眠的组织占着，PyPI、crates 被 2026 年的小项目占了；在赌博里还指藏牌出千的工具。
- 评委怎么看：初筛 12 分（5、3、4）。中文用户与传播评委排第二，梗、角色与独立感评委排第六，可用性与长期评委没排进前 8，理由都是 holdout.com 的被误认风险。

**Bernat**：命令 `bernat`；名字就是角色方向，6 票，黄灯。

- 含义：加泰罗尼亚语把苍鹭叫 bernat pescaire，直译「钓鱼的伯纳特」（Gran Enciclopèdia Catalana），这种鹭本来就有个人名。
- 角色：继续配 ASCII 苍鹭，人设是一个只钓自己想要的那条鱼的老钓鱼佬。
- 梗：「钓鱼佬永不空军」→「Bernat 永不空军」；苍鹭站在信息流里，鱼从身边游过它也不动。
- 中文读法：「伯纳特 / 贝尔纳特」，读法稳，没有负面谐音；外号可能是「钓鱼佬」「老伯」（推断）。
- 风险：常见人名和姓，毛线品牌 BERNAT 从 1964 年起有在用的美国商标（第 16 类），球员 Juan Bernat 也占搜索；GitHub 裸名在一个从 2009 年活跃到现在的真人手里；加泰罗尼亚语里 bernat 也是椿象、寄居蟹俗名的开头，「苍鹭本来就叫这个」要打折扣；梗要先讲加泰语才成立。
- 评委怎么看：初筛 9 分（4、2、3）。三位终评分别排第八、第七、第六。

**Stilly**：命令 `stilly`；名字就是角色方向，6 票，黄灯。

- 含义：诗里用的「寂静的」（Wiktionary 引 Coleridge 等，Thomas Moore 的「Oft in the Stilly Night」），也像 still + -y 的宠物名：一只总是站着不动的鹭。
- 角色：配 ASCII 鹭，人设就是「静」：它不动时你什么都看得清，它一动就散。
- 梗：中文外号顺理成章是「静静」，接上「我想静静」；表情包是单脚站着的鹭配字「我想静静」。
- 中文读法：「斯蒂利」，两个音节，「斯」不至于被读成「死」；听着像女名或美妆品牌（推断）。
- 风险：和 silly 只差一个 t，Urban Dictionary 也收了 stupid + silly 的意思；App Store 中美两区已有 2026 年上架、正好叫 Stilly 的两个 App，都打本地和隐私牌；stilly.com 是 STILLY® 预调酒品牌；「静静」的梗已经十年了；缺技术双关和动词。
- 评委怎么看：初筛 9 分（3、3、3）。三位终评分别排第七、第六、第八。

**总表**（细查过的 18 个，按票数排列）：

| 名字 | 方向 | 票数 | 结论 | 一句话 |
|---|---|---|---|---|
| Guddle | 一个小动作 | 22 | 绿灯，推荐 | 名字、角色、动词、中文梗「摸鱼」都从一个苏格兰词里长出来；裸名被占，用 `guddlehq` |
| Grepling | 终端里好看的造词 | 19 | 绿灯，备选第一 | 命名空间全空，鹭捕鱼就是一次 grep；梗只在开发者圈成立，像 Greptile 的造法 |
| Fogdog | 雾与浅水 | 15 | 黄灯，备选第二 | 「雾里亮着的一小片」字面对上迷雾地图；要换成狗，有非软件类商标 |
| Butterbump | 名字就是角色 | 12 | 绿灯，备选第三 | 鹭科大麻鳽的方言老名，全平台空着；10 个字母，「黄油」误读 |
| Potoo | 名字就是角色 | 12 | 黄灯 | 林鸱瞪眼脸是成型表情包；借的是鸟自带的热度，要放弃鹭，PyPI / crates 被占 |
| Holdout | 流外的立场 | 10 | 黄灯 | 钉子户 + 留出集双关；holdout.com 立场几乎同路，被误认风险实在 |
| Bernat | 名字就是角色 | 6 | 黄灯 | 苍鹭本来就叫「钓鱼的伯纳特」；常见人名、毛线品牌，搜不到 |
| Stilly | 名字就是角色 | 6 | 黄灯 | 「我想静静」；像 silly，App Store 已有两个同名本地隐私 App |
| Dotty | 名字就是角色 | 5 | 红灯 | Debian / Ubuntu 的 Graphviz 带 `/usr/bin/dotty`，Scala 3 旧代号，Teamwork 新出同名 AI 助手 |
| Blorbo | 念起来好玩 | 1 | 黄灯 | 「你偏爱的那个」很贴；npm 刚被活跃库占，中文用户不认识这个 Tumblr 梗 |
| Gugugu | 藏着中文的英文名 | 0 | 黄灯，冲突 | 「鸽了」梗最足，但它是拼音，和这一轮只定英文名冲突；英文里有冷僻的历史蔑称 |
| Ashore | 流外的立场 | 0 | 红灯 | 五天前已有中文、开源、本地优先的备考项目叫 Ashore（上岸），中文会读成考公上岸 |
| Scry | 一个小动作 | 0 | 红灯 | 万智牌「占卜」最够 Magic，但 nixpkgs / AUR 有 scry 二进制，USPTO 有第 42 类 SCRY 注册 |
| Grebe | 名字就是角色 | 0 | 黄灯 | 和 grep 一字之差；裸名被活跃个人占，npm / crates 同名包带 grebe 命令，搜索被鸟类内容淹没 |
| Spoony | 名字就是角色 | 0 | 红灯 | spoony 也是 spoonie（慢性病、残障者）的拼法，像借用群体称呼 |
| Nah | 流外的立场 | 0 | 红灯 | 活跃的 coding-agent 护栏（★494）命令就叫 `nah`，和本产品用户是同一群人 |
| Caligo | 雾与浅水 | 0 | 黄灯 | 猫头鹰环蝶、「咖喱狗」外号；裸名和 `-app` 被占，英文还是眼病名 |
| Cuttle | 名字就是角色 | 0 | 黄灯 | 墨鱼喷墨 = 雾；npm / PyPI / crates 全被占，已有两个 `cuttle` 命令，其一是 Agent 工具 |

<a id="5-可用性检查"></a>
## 5. 可用性检查

2026-11-01 查询，方法见 [约定](#约定)。行是细查过的 18 个名字；Homebrew 一列同时查 formula 和 cask；App Store 两列按应用名里有没有这个名字记。

| 名字 | GitHub 裸名 | GitHub -app | GitHub hq | npm | PyPI | crates | Homebrew | Repology | App Store 中国区 | App Store 美国区 |
|---|---|---|---|---|---|---|---|---|---|---|
| Guddle | 被占 | 空着 | 空着 | 空着 | 空着 | 空着 | 空着 | 空着 | 空着 | 空着 |
| Grepling | 空着 | 空着 | 空着 | 空着 | 空着 | 空着 | 空着 | 空着 | 空着 | 空着 |
| Fogdog | 被占 | 空着 | 空着 | 空着 | 空着 | 空着 | 空着 | 空着 | 空着 | 空着 |
| Butterbump | 空着 | 空着 | 空着 | 空着 | 空着 | 空着 | 空着 | 空着 | 空着 | 空着 |
| Potoo | 被占 | 空着 | 空着 | 被占 | 被占 | 被占 | 空着 | 被占 | 空着 | 空着 |
| Holdout | 被占 | 空着 | 空着 | 空着 | 被占 | 被占 | 空着 | 空着 | 未知 | 被占 |
| Bernat | 被占 | 空着 | 空着 | 空着 | 空着 | 空着 | 空着 | 空着 | 空着 | 空着 |
| Stilly | 被占 | 空着 | 空着 | 空着 | 空着 | 空着 | 空着 | 空着 | 被占 | 被占 |
| Dotty | 被占 | 被占 | 空着 | 被占 | 被占 | 被占 | 空着 | 被占 | 被占 | 被占 |
| Blorbo | 被占 | 空着 | 空着 | 被占 | 空着 | 空着 | 空着 | 空着 | 未知 | 被占 |
| Gugugu | 被占 | 空着 | 空着 | 空着 | 空着 | 空着 | 空着 | 空着 | 被占 | 空着 |
| Ashore | 被占 | 空着 | 被占 | 被占 | 空着 | 空着 | 空着 | 空着 | 被占 | 被占 |
| Scry | 被占 | 空着 | 空着 | 被占 | 被占 | 被占 | 空着 | 被占 | 未知 | 被占 |
| Grebe | 被占 | 空着 | 空着 | 被占 | 空着 | 被占 | 空着 | 空着 | 空着 | 空着 |
| Spoony | 被占 | 被占 | 空着 | 空着 | 空着 | 空着 | 空着 | 空着 | 未知 | 被占 |
| Nah | 被占 | 空着 | 空着 | 被占 | 被占 | 被占 | 空着 | 被占 | 空着 | 被占 |
| Caligo | 被占 | 被占 | 空着 | 被占 | 空着 | 空着 | 空着 | 空着 | 被占 | 被占 |
| Cuttle | 被占 | 空着 | 空着 | 被占 | 被占 | 被占 | 空着 | 空着 | 未知 | 被占 |

App Store 的记法：细查记录里有几条只列了被占的应用、没写是哪一区（Holdout、Blorbo、Scry、Spoony、Cuttle、Nah 的 NAH SH）。iTunes Search API 不带 `country` 时默认查美国区，这几条都记在美国区（推断上游按默认区查询），中国区写「未知」，分区待核实（见 [§10](#10-待核实清单)）。Potoo 的 npm 一格：初筛快查记为已占，细查返回未知；2026-11-01 复查 `registry.npmjs.org/potoo` 返回 200，是 2017 年发布、2020 年后没更新的包，记为被占。Gugugu 中国区的「被占」是近名「谷谷」，不是完全同名。

**复查（2026-11-01）**：对 [§0](#0-推荐) 的推荐名 Guddle 和备选 Grepling、Fogdog、Butterbump 重查了 GitHub 裸名、`-app`、`hq`（`gh api users/<名>`）和 npm（`registry.npmjs.org/<名>`），结果和上表一致：`guddle`、`fogdog` 是个人账号，其余 GitHub 写法和四个 npm 名都返回 404。

**被占的是谁：**

- **Guddle**：GitHub `guddle` 是个人账号，6 个 0 星仓库（多为 jeecg 等项目的复制），仍有零星活动。
- **Fogdog**：GitHub `fogdog` 是 2014 年注册的个人账号，1 个仓库（hello-world，最后推送 2017-05）、0 关注。
- **Potoo**：GitHub `potoo` 是 2014 年注册、0 仓库的个人账号；PyPI / AUR 是终端幻灯片工具 potoo；crates 是小项目。
- **Holdout**：GitHub `holdout` 是 0 仓库的组织（2016 年起休眠）；PyPI holdout 是 MAGI Design Team 的留存决策包；crates holdout 是给 agent 用的验证工具，会装出 `holdout` 二进制；App Store 有 Holdout - Savings、Holdout: Shoot To Survive 等。
- **Bernat**：GitHub `bernat` 是从 2009 年活跃到现在的个人用户（16 个仓库）。
- **Stilly**：GitHub `stilly` 是 Steve Tilly 的 0 仓库账号；App Store 有 PRIVISIO 的相机 App「Stilly」和 Kam Hung Cheung 的「Stilly – BP & Health Log」，都是 2026 年上架。
- **Dotty**：GitHub `dotty` 是 2013 年的个人账号，`dotty-app` 2026-09 才被注册；npm、PyPI、crates 都有同名包；Repology 里有 AUR 点文件管理器 dotty / dotty-bin，Debian / Ubuntu 的 graphviz 带 `/usr/bin/dotty`；App Store 有 Dotty: Personal CRM & Notes（中国区名 Dotty: Your People）、Dotty: Habit & Task Planner 等一串。
- **Blorbo**：GitHub `blorbo` 是休眠账号（2 个仓库）；npm blorbo 是 adambiggs 一个月前发的 Canvas 背景库；App Store 有游戏 Blorbo's Run。
- **Gugugu**：GitHub `gugugu` 是 2014 年的 0 仓库账号；中国区 App Store 有「谷谷 - 谷子收藏与整理工具」。
- **Ashore**：GitHub `ashore` 是个人用户（显示名 lshore，5 个仓库），`ashorehq` 三天前刚被注册；npm 是 npm 官方的安全占位包；App Store 有 Ashore: Quit Smoking & Sober（中国区名「向岸」）、Ashore: Speaker Water Cleaner 等。
- **Scry**：GitHub `scry` 是 0 仓库的组织；npm scry 是 2022 年后没更新的小库，PyPI、crates 也有同名包；Repology 里 nixpkgs / AUR 有 Crystal 语言服务器的 scry 二进制；App Store 有 Scry MTG、Scry: Remote Desktop 等。
- **Grebe**：GitHub `grebe` 是 Paul Rigge 的活跃账号（47 个仓库）；npm grebe 是测试运行器，crates grebe 是 2026-10 发布的 DuckDB SQL formatter，两者都声明了 `grebe` 可执行文件。
- **Spoony**：GitHub `spoony` 是 2011 年的个人账号（3 个仓库），`spoony-app` 也被占；App Store 有 Spoony Gloss 和一串 spoonie 类 App。
- **Nah**：GitHub `nah` 是 2 个仓库的组织；npm nah 是 Node.js 版本管理器；PyPI / AUR 的 nah 是 manuelschipper/nah（coding agent 安全护栏，★494）；crates 也有同名包；App Store 有交通 App NAH.SH。
- **Caligo**：GitHub `caligo`（1 个仓库）和 `caligo-app` 都被不活跃账号占着；npm 是一个停更的 Preact 相关包；App Store 两区都有 Caligo 卡路里 App。
- **Cuttle**：GitHub `cuttle` 是 2017 年的 0 仓库账号；npm、PyPI（CuttleLabs 的 `cuttle` 命令）、crates（LLM 推理引擎）都被占；App Store 有 Cuttle: Card Battle 等。

<a id="6-词义中文读法与同名产品"></a>
## 6. 词义、中文读法与同名产品

每个入围名一条，按 [§4](#4-入围名单) 的顺序。上一轮同一节的写法见 [调研 07 §6](07-positioning-and-naming.md#6-词义谐音与同名产品)。

**Guddle**

- 词义：英语、苏格兰语动词，伸手在溪岸边、石头底下摸鱼，徒手抓上来（Wiktionary）。苏格兰语还有鸭子戏水、玩泥水、干脏活的义项，名词是「一团乱」；苏格兰国家词典 DSL（dsl.ac.uk/entry/snd/guddle）证实了「一团乱、混乱」，The Scotsman 的「Scottish word of the day」用过 in a guddle；DSL 另收捅、戳、粗活、钝刀、撬棍。Urban Dictionary 有 10 条，含借摸鱼动作的性暗示和「脏乱的地方」，赞踩都是 0。Wiktionary 只有英语和苏格兰语两节，日、西、德、法、韩语里没见到同形词（只查了 Wiktionary，待核实）。
- 中文读法与谐音：官方写「嘎豆」；预期误读「咕嘟」（冒泡声，和水、鱼搭）、「古德」（good 的中式谐音）。没有死、屎、尸、输、丧、病的近音。直译「摸鱼」撞上简中「上班摸鱼、划水」热梗，有 meme 属性也有两面性；也可能联想到「浑水摸鱼」（推断）。
- 同名产品：GitHub 名字含 guddle 的仓库 6 个，全在个人账号 guddle 下、0 星。AlternativeTo 有一个叫 Guddle 的条目，搜索摘要说是给小团队用的 PRINCE2 项目管理工具，用 AI 起草行政文书；条目页返回 403、相关域名连不上，是否还在运营待核实，和本产品不同类。搜 app、software、AI、open source 都没有别的同名产品，搜索引擎把它当成 Huddle、Cuddlr、Google Goggles、Google AI 的错拼；App Store 美区返回的也全是 Huddle 和 Hudl。guddle.com 2001 年注册，NS 在 Afternic，推断是挂牌出售的停放域名。
- 商标信号：英文和中文都没搜到 GUDDLE 的商标记录；只有拼写相近的 GUNDLE（USPTO，已失效）和 GUDLEY（印度，第 25 类服装）。Justia 检索页返回 403；CNIPA、EUIPO、UKIPO 没直接查，待核实。
- 命令撞名：不撞。本机 PATH 和 man 页都没有，Homebrew、Repology、npm、PyPI、crates 都没有同名包或二进制，没找到同名 MCP 服务；`GUDDLE_`、`.guddle/`、`guddle://` 都没看到撞名。

**Grepling**

- 词义：英语没有词典义；Wiktionary 收了 grep（用 grep 这类程序搜文件），-ling 表示「小东西」（常识）。推断读音 /ˈɡrɛplɪŋ/，外形和读音接近 gremlin（小捣蛋鬼）、grayling（茴鱼）、grappling（扭打）。R 语言有 `grepl()`，R 用户可能读成 grepl-ing（推断）。德语 Greifling（婴儿抓握玩具）拼法相近（推断，待核实）。其他语言待核实。
- 中文读法与谐音：「格瑞普林 / 格雷普灵」；-ling 和「灵」同音，可能被叫成「grep 灵」「小精灵」，偏正面（推断）；粤语近似「格烈普玲」（推断）。没有负面近音。不熟悉 grep 的用户读不出梗。
- 同名产品：GitHub 名字含 grepling 的仓库 0 个；网页搜索只出无关结果：Pathfinder 怪物 groplit、Perl 的 App::Greple、奥地利葡萄园 Ried Greppling 的绿维特利纳。最近的相近名是 Greptile：旧金山的 AI 代码审查公司，YC 出身，SiliconANGLE 2025-09-23 报道融资 2500 万美元。grepling.com 2010 年注册，2026-10-24 刚变更（推断是续费），HTTPS 返回 403，持有人与用途待核实。
- 商标信号：本轮没检索（搜索额度用完），待核实；没看到任何用 Grepling 的产品。要顺带看 Greptile 的商标范围是否覆盖 GREP- 开头的词（待核实）。
- 命令撞名：不撞。各包管理器和 MCP 注册表都没有；终端里输入 grep 按 Tab 会带出 grepling（推断），算小麻烦，也算个梗。

**Fogdog**

- 词义：英语真词，雾层里有时出现的一块亮斑（dictionary.com，1865–70 年首见，仿 sundog；Wordnik 引 American Heritage：a ray of light seen amidst fog）；航海用法另指海雾里的一块晴空缺口。Urban Dictionary 没有 fog dog 词条。其他语言没查到含义（待核实），外国用户一般读成「雾 + 狗」。
- 中文读法与谐音：「雾狗」「狗子」，也可能念成「佛格多格」（推断）；「X 狗」（单身狗、加班狗）是自嘲但常见的说法，不算负面；粤语 mou6 gau2 没发现不雅谐音（推断）。
- 同名产品：GitHub 同名仓库只有 6 个 0 星个人空仓库。网页搜索没找到同名软件，最接近的是 fogsightai/Fogsight（AI 动画引擎，约 936 星，只是前缀相近）。历史同名品牌两个：Fogdog Sports（1999–2000 年体育用品电商，后归 GSI Commerce，Adweek 记载其广告用过穿狗服装的吉祥物）；Fogdog Cold Brew（加州冷萃茶 / 咖啡，2017 年亮相，是否仍在经营待核实）。fogdog.com 1998 年注册，MarkMonitor 托管、三重锁定，推断属于 Fogdog Sports 的承继方。
- 商标信号：加拿大 CIPO 两件在效：TMA559967，权利人 Radial, Inc.，第 3–28 类体育用品和第 35 类零售，2032-04-05 到期；TMA1046759，权利人 Zuan Technologies, LLC，第 30 类茶、咖啡饮料，2029-08-02 到期，并声明美国注册号 5,101,851。美国 1999 年的 FOGDOG 申请 75704090 已放弃（Justia）。没看到第 9、42 类的 FOGDOG；美国 5,101,851 的现状和中国商标待核实。
- 命令撞名：不撞。不是常见 Unix 命令，Homebrew、Repology、npm 都没有；Ruby 的 fog 云库带一个 `fog` 命令，只是前缀相同（推断）。

**Butterbump**

- 词义：英语方言词，大麻鳽（Botaurus stellaris），Merriam-Webster 标 dialectal, England；Spectator 专栏解释 butter 来自 bittern 的旧写法、bump 指鳽鸣，丁尼生《Northern Farmer: Old Style》里写过 butter-bump；1671 年首见只有 wehd.com 转载 Skinner 词典的二手出处；wehd 还收了约克郡的 Butter Bump / Bottle Bump。英国家具品牌 Loaf 拿「butterbump sofas」做过广告词，意思是起鸡皮疙瘩加心里小鹿乱撞（Spectator 2016）。英语 bump 常指孕肚，有孕肚霜叫 Bump Butter（推断会被误读）。德语 Butter 也是黄油；日语会音译成バターバンプ（推断）。
- 中文读法与谐音：「黄油包 / 黄油泵」（推断），粤语 butter 叫「牛油」没有问题；中文 ACG 圈「黄油」常指成人向游戏（只查到一条 Steam 创意工坊评论，流行程度待核实）；论坛里 bump 是顶帖。
- 同名产品：GitHub 只有一个 0 星空仓库 opontuslandn/Butterbump；网页搜索没有同名软件、App 或 AI 产品。butterbump.com 是文案 Adam Marek 的个人工作室页 Butterbump Creative（2026-04 注册，可能是重新注册，推断）。
- 商标信号：没找到 BUTTERBUMP 的注册记录；最接近的是 BUTTERBOMB（USPTO 90490299，黄油制品）。Loaf 那句广告词是否注册待核实；官方库没查。
- 命令撞名：不撞；缺点是 10 个字母天天要敲，可以靠 shell 别名缓解，但那样就不再是「名字即命令」。

**Potoo**

- 词义：英语只有「林鸱」一义（Wiktionary、Dictionary.com），俚语义没查到（Urban Dictionary 待核实）。Tetzoo、Mongabay、ABC Birds 都写过它「一脸永远受惊」的表情在各平台走红，TikTok 上叫 ghost bird。西班牙语 poto 在拉美部分地区是口语「屁股」（Wiktionary）；意大利语 poto 是「饮料」。这只鸟在日、德、法、葡语里另有名字，potoo 这个拼写在这些语言里没有意思（推断）。
- 中文读法与谐音：正式译名「林鸱」（lín chī）；直接念「波兔 / 破兔」，可能谐音成「破图」，或因像 potato 被叫「土豆」（推断）；粤语「波」有俚语义，「波兔」可能被调侃（推断，弱）。中文网上没有成型的林鸱梗。
- 同名产品：没找到同类目同名。韩国 CodiMe 的 Potoo 是时尚电商 AI 模特图 SaaS（CB Insights 记 2022 年成立于首尔，2025-12 有进日本的新闻稿，原文没核对）。GitHub 上的 potoo 都是小项目：BlackPhlox/potoo（Bevy 编辑器，★9）、gnacho/potoo（Go 单二进制统计后端，★5）、casavi/potoo（PDF 区域 OCR 命令行，★4）等。potoo.com 推断是停放域名。
- 商标信号：没出现以 POTOO 本身注册的商标，只有拼写相近的无关商标；CodiMe 在韩、日是否注册待核实；官方库没查。
- 命令撞名：不是常见 Unix 命令；会装进 PATH 的同名小工具有 AUR / PyPI 的 potoo（终端幻灯片）、casavi/potoo、gnacho/potoo，都不知名；Homebrew 没有，MCP 搜索 0 条。

**Holdout**

- 词义：英语，拒不同意、想等更好条件的人，或死守已失势事业的人；另一义是牌桌上出千藏牌的工具（Wiktionary、Wikipedia「Holdout (gambling)」）。Wikipedia 消歧义页还有体育罢训、房产钉子户、二战后还在打仗的日本兵、债券 holdout problem、holdout data set。hold out on sb 是瞒着某人、藏着不给（Green's），这层可能被读成藏私。西班牙语有外来词词条，其他语言待核实；日语机器学习里推断叫ホールドアウト法。
- 中文读法与谐音：「霍道特」，ld 连读可能念成「后道」（推断）；外号「钉子户」「hold 住」（推断）；没有负面近音。
- 同名产品：holdout.com 的 Holdout（「Just you, and your AI」，自称 free、skinnable、open-source AI platform，有插件生态，已上线访客聊天，开发中的个人版用 `name.holdout.app` 子域名，概念页有 Atlas、Fieldnotes、Margin 等）；还没正式上线，也没找到仓库链接。GitHub 名字含 holdout 的仓库 250 个，星数都低；holdout-labs 组织（2026-08 创建，金融 AI 研究的证据留档工具）。推断 2026 年 AI 评测圈很爱用这个词。
- 商标信号：没找到软件类 Holdout 商标，搜索结果只有 patent holdout 的法学论文；holdout.com 没写公司名、没见 ™ / ®；中文没检索。
- 命令撞名：没有知名的 `holdout` 命令；`cargo install holdout` 会装出同名二进制（下载 57 次）；PyPI holdout 带不带命令行入口待核实。

**Bernat**

- 词义：加泰罗尼亚语、奥克语男名，等于 Bernard，来自日耳曼语「熊 + 坚强」（Wiktionary、behindthename），加泰语读 [bərˈnat]。Gran Enciclopèdia Catalana 收了 bernat pescaire = 苍鹭，也收了 bernat pudent = 椿象、bernat ermità = 寄居蟹，所以加泰语用户听到 Bernat 不一定想到鹭。英语没有词典义，Urban Dictionary 只有低质量条目。匈牙利语有男名 Bernát；日、西、德、法、韩语没有普通词义（推断，待核实）。
- 中文读法与谐音：大陆体育媒体译「贝尔纳特」，台湾译「伯納特」，港澳「貝拿特 / 巴尼特」（中文维基 Juan Bernat 条）；可能联想圣伯纳犬（推断）；没发现不好的谐音。英语用户容易拼成 Bernard（推断）。
- 同名产品：没有叫 Bernat 的软件产品，bernat + mcp 搜索为 0；名气大的同名都不做软件：Bernat 毛线品牌（Yarnspirations / Spinrite）、球员 Juan Bernat；bernat.com 据品牌史网站说跳到 Yarnspirations 的 Bernat 页（本次 curl 返回 403）。
- 商标信号：第三方追踪站显示美国 BERNAT 注册号 0765926，1964 年注册，在用，第 16 类，权利人 Spinrite Inc.（以 TSDR 为准）；没找到第 9、42 类；中国待核实。
- 命令撞名：不撞。

**Stilly**

- 词义：英语形容词「寂静的、平静的」，副词「静静地」（Wiktionary，引 Coleridge 1795 年《The Eolian Harp》、Stephen King《The Green Mile》）；Moore 的《Oft in the Stilly Night》本次没直接核对。Urban Dictionary 有 stupid + silly、跑了气的汽水、对俄克拉何马州 Stillwater 的贬义叫法。华盛顿州 Stillaguamish 河当地有 Stilly 的叫法（Everett Herald 网址佐证）。德语 still 是「安静」，德语用户会觉得自然（推断）；其他语言待核实。
- 中文读法与谐音：「斯蒂利 / 丝缇丽」，像女名或美妆牌子（推断）；懂英文的人可能从 still 联想出「静静」，接上「我想静静」（推断，梗本身本次没检索核实）。没发现不好的谐音。
- 同名产品：App Store 两个 2026 年上架的 Stilly（PRIVISIO 的相机 App、Kam Hung Cheung 的健康记录），都打本地和隐私牌；stilly.com 是 STILLY® 伏特加预调酒；GitHub 含 stilly 的仓库 45 个，都很小；名字相近的 An-idd/stillyou（★0）做的是管理 Agent 产出、用普通文件记账、跨宿主走 MCP，和本产品挨得近。「Stilly app / software / AI」没搜，其他同名待核实。
- 商标信号：stilly.com 标 STILLY®，推断是第 33 类酒类注册；两个 App 页面没见 ® / ™；正式检索没做。
- 命令撞名：不撞。

<a id="7-全部候选"></a>
## 7. 全部候选

168 个候选按 7 个方向分组，和上一轮（[调研 07 §7](07-positioning-and-naming.md#7-全部候选)）没有重复。「初筛分」是三位初筛评委打分的合计，满分 15，括号里是判「致命」的评委人数。去留原因里写「入围」的 8 个见 [§4](#4-入围名单)；写「细查判红 / 判黄」的附终评票数，详情在 [§4](#4-入围名单) 的总表和 [§5](#5-可用性检查)；其余停在初筛，原因取自评委的主要扣分点。

**名字就是角色**（25 个）：先定角色，再让名字就是这个角色的名字。来源有三个：鸟自己的老名字或俗名（Butterbump、Hernshaw、Bernat、Dabchick、Shypoke、Quawk），来历现成，一句话讲完，不需要先讲中文；宠物式的写法（Dotty、Stilly、Spoony、Hunch、Bitty），中文外号容易自己长出来；换一个角色（Potoo、Cuttle、Tyto、Morepork）。踩到的坑：quawk、kwak、quok 在中文里都读成「夸克」；任何苍鹭吉祥物都会让人想到宫崎骏《你想活出怎样的人生》里那只会说话的苍鹭；猫头鹰会被认成催打卡的多邻国；crane 会被意译成「仙鹤」；带 -tty 的名字像在学 Ghostty。

| 名字 | 命令 | 含义 | 中文读法 | 初筛分 | 去留原因 |
|---|---|---|---|---|---|
| Dotty | `dotty` | dot + tty；英式口语「有点怪」 | 多蒂；外号「点点」 | 11 | 细查判红，5 票：Debian / Ubuntu 有 `dotty` 命令，Scala 3 旧代号 |
| Butterbump | `butterbump` | 英格兰方言里的大麻鳽，叫声像雾笛 | 外号「黄油包」 | 10 | 入围，12 票，备选第三 |
| Grebe | `grebe` | 䴙䴘，和 grep 一字之差 | 格瑞贝（结尾 e 不稳）；䴙䴘生僻 | 10 | 细查判黄，0 票：裸名被活跃个人占，npm / crates 同名包带命令 |
| Hernshaw | `hernshaw` | 方言「苍鹭」；《哈姆雷特》hawk from a handsaw | 汉肖（吞 r）；外号「手锯」 | 7 | 停在初筛：-rnsh- 难念难拼，莎翁出处掉书袋，苍鹭像宫崎骏 |
| Spoony | `spoony` | spoonbill + -y；旧口语「犯傻、痴情」 | 斯普尼；外号「勺子」 | 10 | 细查判红，0 票：spoony 也是 spoonie（慢性病、残障者）的拼法 |
| Potoo | `potoo` | 林鸱，遇险定住像断树枝 | 波兔；外号「土豆」，近「破图」 | 10 | 入围，12 票 |
| Bernat | `bernat` | 加泰语苍鹭「钓鱼的伯纳特」 | 伯纳特；外号「钓鱼佬」 | 9 | 入围，6 票 |
| Stilly | `stilly` | 诗里的「寂静的」 | 斯蒂利；外号「静静」 | 9 | 入围，6 票 |
| Dabchick | `dabchick` | 小䴙䴘的英国俗名 | 达布奇克，像「搭不起」 | 6 | 停在初筛：-bch- 难念，chick 有旧俚语包袱，dab 舞梗过时 |
| Hunch | `hunch` | 鹭缩脖弓背；直觉、预感 | 汉趣；外号「憨憨」 | 7 | 停在初筛：常用词搜不到，hunchback 联想，有同名推荐算法公司 |
| Curlew | `curlew` | 杓鹬，藏着 curl | 柯留；-lew 像 loo | 6 | 停在初筛：`cur` 补全和 curl 抢，r / l 同词，有同名桌面多媒体工具 |
| Cuttle | `cuttle` | cuttlefish 简写，墨鱼喷墨像雾 | 卡特尔；外号「墨鱼」 | 9 | 细查判黄，0 票：npm / PyPI / crates 全被占，已有两个 `cuttle` 命令 |
| Stilty | `stilty` | 踩高跷的长腿鹭 | 斯蒂提（吞 l）；外号「跷跷」 | 6 | 停在初筛：stilted 负面，和 Stilly 易混，梗弱 |
| Shypoke | `shypoke` | 绿鹭的美国叫法，shitepoke 的变体 | 晒扑克 | 2（致命 1） | 停在初筛：词源一查就是「屎」 |
| Squacco | `squacco` | 池鹭英文名，像一声鸟叫 | 斯夸扣；外号「瓜瓜」 | 6 | 停在初筛：三音节加 sq- 辅音群，组织名 2026-08 刚被注册 |
| Bitty | `bitty` | 小苇鳽昵称，itty-bitty，藏 bit | 比提；外号「比特」 | 7 | 停在初筛：itty-bitty（★3463）占联想，常见女性昵称 |
| Arapapa | `arapapa` | 船嘴鹭的葡语名 | 外号「阿拉爸爸」 | 6 | 停在初筛：近「阿里巴巴」，「叫爸爸」玩笑易粗俗 |
| Cranky | `cranky` | crane + 脾气臭 | 克兰奇；外号「臭脸鹭」 | 8 | 停在初筛：crane 会被意译成鹤，滑回国风 |
| Scatty | `scatty` | scatter + tty；英式「迷糊」 | 斯卡蒂（撞《明日方舟》干员） | 4（致命 3） | 停在初筛：scat 是粪便和色情黑话 |
| Quawk | `quawk` | 夜鹭的美国民间叫法 | 「夸克」 | 3（致命 3） | 停在初筛：读成「夸克」，撞阿里同样存内容的产品 |
| Egbert | `egbert` | egret + 老派人名 | 艾格伯特；外号「蛋伯」 | 6 | 停在初筛：真实人名，egret 双关只有英文读者懂 |
| Agami | `agami` | 栗腹鹭英文名 | 阿嘎米；外号「阿咪」 | 7 | 停在初筛：有同名 AI 数据信任层项目，来历待核实 |
| Tyto | `tyto` | 仓鸮属名 | 泰托；外号「猴面鹰」 | 3（致命 1） | 停在初筛：猫头鹰像天天催打卡的多邻国，和「不催」相反 |
| Ardeola | `ardeola` | 拉丁语「小鹭」，池鹭属 | 阿迪奥拉 | 6 | 停在初筛：像分类学名，4 个音节，梗弱 |
| Morepork | `morepork` | 新西兰鹰鸮，叫声像 more-pork | 外号「更多猪肉」 | 6 | 停在初筛：`more` 补全撞，pork 宗教敏感，猫头鹰像多邻国 |

**终端里好看的造词**（24 个）：每个名字藏一个终端或文本处理的行话（grep、tty、tar、esc、@、DEL、~、char、dither、glyph、awk、curl、WAL），再拼进一个能画出来的角色词。懂行的人会心一笑，不懂的照样能念。初筛前去掉了 Anansi（神话名）、Haar（机器学习术语，后来在雾方向重提）、Tildo（和 dildo 只差一个字母）、Ttybara（tty 放词首会读成 titty）。4–6 个字母的真词在 GitHub 上几乎都被占了。

| 名字 | 命令 | 含义 | 中文读法 | 初筛分 | 去留原因 |
|---|---|---|---|---|---|
| Grepling | `grepling` | grep + -ling，一只会 grep 的小家伙 | 格瑞普林 | 11 | 入围，19 票，备选第一 |
| Capytty | `capytty` | capybara + tty | 外号「水豚」「卡皮」 | 9 | 停在初筛：追热点，-tty 套 Ghostty，Ruby 的 Capybara 同名 |
| Tarling | `tarling` | tar（可追加归档）+ starling | 塔林（墓塔联想） | 9 | 停在初筛：裸名在活跃个人手里，是英文姓氏 |
| Escargot | `escargot` | esc + argot，法语的蜗牛和 @ | 艾斯卡狗；外号「蜗牛」 | 9 | 停在初筛：8 字母，Samsung 有同名 JS 引擎，意译成普通的蜗牛 |
| Miuku | `miuku` | 芬兰语里 @ 的俗称「喵喵」 | 喵酷 | 9 | 停在初筛：英文读法不稳，猫伙伴撞得多 |
| Dotsam | `dotsam` | dot + flotsam（漂流物） | 多桑（台湾 / 日语里是「爸爸」） | 9 | 停在初筛：裸名是活跃用户（dot + Sam），flotsam 是生词 |
| Nodel | `nodel` | no + DEL，没有删除键 | 诺德，或被看成 noodle | 6 | 停在初筛：三种读法，长不出角色 |
| Charling | `charling` | char + -ling，字符也是角色 | 查令 | 8 | 停在初筛：char 读法有争议，像人名，贴近 Codex Pets |
| Tildy | `tildy` | ~（主目录，也是水波） | 缇迪；外号「小波浪」 | 9 | 停在初筛：和 tidy 只差一个字母，像整理工具 |
| Bitern | `bitern` | bit + tern | 比特恩，想到比特币 | 6 | 停在初筛：像 bittern 拼错，bitter 负面 |
| Curlee | `curlee` | curl + ee，接近 curlew | 柯莉，像女孩名 | 6 | 停在初筛：`cur` 补全和 curl 抢，像发廊名 |
| Ditty | `ditty` | dither + tty，合成「小曲」 | 外号「弟弟」 | 3（致命 1） | 停在初筛：「弟弟」是嘲讽人菜的黑话 |
| Dithron | `dithron` | dither + heron 的尾音 | 迪斯伦 / 滴龙 | 6 | 停在初筛：th 难念，-ron 像机甲名 |
| Grepup | `grepup` | grep + up，也能断成 pup | 格瑞帕普 | 6 | 停在初筛：npm 同名是 AI 代码迁移工具 |
| Glyphowl | `glyphowl` | glyph + owl | 外号「猫头鹰」 | 6 | 停在初筛：猫头鹰滑向「智慧 / 第二大脑」，-lyph 辅音群 |
| Awklet | `awklet` | awk + auklet（小海雀） | 傲客；外号「尴尬鸟」 | 6 | 停在初筛：auklet 冷门，两层梗都要解释 |
| Murkat | `murkat` | murk + meerkat | 墨卡；外号「丁满」 | 6 | 停在初筛：技术双关弱，murk 阴郁，狐獴形象被广告用过 |
| Kukac | `kukac` | 匈牙利语的 @，本义虫子、蛆 | 库卡茨 / 苦卡 | 0（致命 1） | 停在初筛：本义是蛆 |
| Rastern | `rastern` | raster + tern | 拉斯腾 | 4 | 停在初筛：两个 r，像 eastern / western，梗弱 |
| Kernling | `kernling` | kern / kernel + -ling | 克林（《龙珠》角色） | 6 | 停在初筛：和 Grepling 同一个 -ling 套路，只需留一个 |
| Waldo | `waldo` | WAL + do；《Where's Waldo?》 | 沃尔多 | 4（致命 2） | 停在初筛：借版权角色名，wally 是英式「傻瓜」 |
| Dotheron | `dotheron` | dot + heron | 多特鹭 | 5 | 停在初筛：描述型，断字读法不唯一 |
| Pixheron | `pixheron` | pixel + heron | 皮克斯鹭 | 1（致命 1） | 停在初筛：中文第一反应是皮克斯，像蹭 Pixar |
| Glyphon | `glyphon` | glyph + gryphon | 格利丰 | 5 | 停在初筛：字形近 glyphosate（草甘膦），Rust 有同名文本渲染库 |

**流外的立场**（25 个）：把「在推荐流外、回头看不用再进推荐流、判断留在自己手里」拆成两组：一组从行为切（上岸、钉子户、潜水、蹲、涉水、截弯、泥滩捡东西、占、驮回家），读进 `<名字> add` 都像工具；另一组是带倔劲的口头禅（Nah、Hmph、Wontfix、Fie、Naysay、Prefernot），能配具体角色的才留。反用平台词只留 Stale、Nofyp 两个，Nofyp 没过 add 测试，作对照。中文那层（上岸、钉子户、潜水、蹲、拿、哼）都是英文本义之外送的。这个方向是真实英文词，GitHub 裸名基本全被占，25 个里只有 stale、nofyp 返回 404，而 stale 可能是保留名。

| 名字 | 命令 | 含义 | 中文读法 | 初筛分 | 去留原因 |
|---|---|---|---|---|---|
| Ashore | `ashore` | 上岸、在岸上，从流里出来 | 阿硕；外号「上岸」 | 11 | 细查判红，0 票：已有中文、开源、本地优先的备考项目 Ashore（上岸） |
| Holdout | `holdout` | 钉子户；机器学习的留出集 | 霍道特；外号「钉子户」 | 12 | 入围，10 票 |
| Lurk | `lurk` | 潜伏旁观，网络「潜水」 | 勒克；外号「潜水」 | 9 | 停在初筛：常用词，lurker 带偷窥感，Rust 已有 lurk 命令 |
| Noop | `noop` | no-op，也像 nope | 努普 / 怒扑 | 8 | 停在初筛：代码里到处是 noop，搜索和 Go 包名都混 |
| Mudlark | `mudlark` | 河岸泥滩上捡旧物的人 | 马德拉克（≈「妈的」） | 2（致命 1） | 停在初筛：粗口谐音 |
| Oxbow | `oxbow` | 牛轭湖，被截下的一段河 | 奥克斯包（撞空调品牌） | 8 | 停在初筛：组织名被活跃占用，有服装和宠物食品品牌 |
| Dibs | `dibs` | 「占了！」 | 迪布斯；外号「占坑」 | 9 | 停在初筛：抢的语气和不争不催冲突，常用俚语 |
| Abide | `abide` | The Dude abides | 阿拜德 | 7 | 停在初筛：电影梗小众，外号长不出 |
| Mule | `mule` | 骡子，倔，驮东西回家 | 外号「电驴」 | 2（致命 1） | 停在初筛：撞 eMule 电驴和 MuleSoft，还有运毒「骡子」义 |
| Lowkey | `lowkey` | 低调、不声张 | 外号「低调」「洛基」 | 8 | 停在初筛：常见俚语搜不到，读音近 Grafana Loki |
| Sayso | `sayso` | say-so，我说了算 | 塞搜 | 7 | 停在初筛：去掉连字符后认不出，首音离 See- 近 |
| Nah | `nah` | 「不了」，中文读「拿」 | 拿 / 呐 | 10 | 细查判红，0 票：coding-agent 护栏的命令就叫 `nah` |
| Hmph | `hmph` | 「哼」 | 哼 | 6 | 停在初筛：英文没元音，念不出也传不开 |
| Wontfix | `wontfix` | GitHub 默认标签「不会再做」 | 万特 fix；外号「不修」 | 8 | 停在初筛：外行读成不维护了，npm 有同名 Agent 台账 |
| Naysay | `naysay` | 唱反调，nay 和 neigh 同音 | 内塞 | 6 | 停在初筛：naysayer 偏扫兴，说教 |
| Prefernot | `prefernot` | 巴托比的 I would prefer not to | 普瑞佛闹特 | 4 | 停在初筛：9 字母，两词粘连，底色发暗 |
| Honk | `honk` | 鹅叫，《无题大鹅模拟》 | 外号「大鹅」 | 7 | 停在初筛：撞 Block 的 goose（同走 MCP），无题大鹅是现成角色 |
| Ratel | `ratel` | 蜜獾 | 外号「平头哥」 | 6 | 停在初筛：撞阿里平头哥半导体，Dgraph 有 Ratel |
| Nopelican | `nopelican` | nope + pelican | 外号「鹈鹕」 | 5 | 停在初筛：9 字母，撞 Pelican 静态站点生成器 |
| Fie | `fie` | 古英语的「呸」 | 飞 / 肥 | 7 | 停在初筛：古词没人认识，npm 同名是阿里前端工具 |
| Stale | `stale` | 不新鲜，反用新鲜度 | 外号「过期」 | 2（致命 1） | 停在初筛：本义负面，开发者先想到关 issue 的 stale bot |
| Nofyp | `nofyp` | no + For You Page | 只能逐字母读 | 3 | 停在初筛：像口号，念不出 |
| Chalant | `chalant` | 反着 nonchalant，在乎 | 沙朗 | 6 | 停在初筛：读法两种，网络玩笑词易过时 |
| Squat | `squat` | 蹲，占着不走 | 外号「蹲」 | 8 | 停在初筛：先想到健身，技术圈 typosquatting 负面 |
| Wade | `wade` | 涉水，鹭是涉禽 | 韦德 | 6 | 停在初筛：像人名（球星、死侍） |

**雾与浅水**（24 个）：不用 fog / mist / haze 原词，从三处下手：雾里的具体东西和现象，挑带点不正经的（fogdog、Brocken、groaner、pogonip、haar）；站在浅水里不动的动物（bittern、shoebill、hern、blenny、goby、dugong，加猫头鹰环蝶 caligo）；有第二层意思的少见外语「雾」（meh、nebulo、udu、caligo、dimma）。另有两个浅水用具 galosh、coracle。筛掉的：没有第二层的外语雾词（brume、nebbia、neblina、laino 等）、fog + X 生造词、punt（「拖延」就是稍后读）、egret（白鹭引擎）、rime（Rime 输入法）、toktokkie（像 TikTok）、wisp（鬼火引人迷路），以及 8 个字母以上的 lightship 等。查重时这个方向的 25 个名字 GitHub 裸名全被占，多数是 0–3 个仓库的休眠账户。

| 名字 | 命令 | 含义 | 中文读法 | 初筛分 | 去留原因 |
|---|---|---|---|---|---|
| Bittern | `bittern` | 麻鳽（鹭科），藏 bit；叫声像雾笛 | 比特恩；麻鳽生僻 | 9 | 停在初筛：字形贴 bitter，同一只鸟的 Butterbump 更好笑 |
| Fogdog | `fogdog` | 雾中看见的一道光 | 外号「雾狗」 | 10 | 入围，15 票，备选第二 |
| Caligo | `caligo` | 拉丁语的雾；猫头鹰环蝶属 | 卡利哥；外号「咖喱狗」 | 10 | 细查判黄，0 票：裸名和 `-app` 被占，英文还是眼病名，像 Caligula |
| Shoebill | `shoebill` | 鲸头鹳，「不动的鸟」 | 会被意译成「鲸头鹳」 | 9 | 停在初筛：意译丢名，有蹭热度感，裸名被组织占 |
| Brocken | `brocken` | 布罗肯幽灵，雾里放大的是自己 | 布罗肯 / 「坏了」 | 7 | 停在初筛：和 broken 同音，被当成拼错 |
| Fogey | `fogey` | 老顽固 | 福吉 | 8 | 停在初筛：三种拼法，带年龄贬义，带 fog 原词 |
| Groaner | `groaner` | 雾中哨笛浮标；烂谐音梗 | 格罗纳 | 6 | 停在初筛：第一义是抱怨、呻吟 |
| Nebulo | `nebulo` | 世界语的雾；拉丁语的无赖 | 内布罗 | 8 | 停在初筛：会被当成 nebula 拼错 |
| Galosh | `galosh` | 套在鞋外的胶鞋 | 嘎洛什；外号「胶鞋」 | 9 | 停在初筛：词义平，梗全靠吉祥物，裸名被研究机构组织占 |
| Nixie | `nixie` | 水妖；辉光管 | 近「你写」 | 7 | 停在初筛：Nix 生态已有同名 CLI nixie-dev/nixie |
| Udu | `udu` | 爱沙尼亚语的雾；伊博陶罐鼓 | 乌都 / 呜嘟 | 7 | 停在初筛：3 个字母搜不到，有「胡话」义 |
| Bunyip | `bunyip` | 澳洲传说里的沼泽怪 | 外号「笨鸭」 | 8 | 停在初筛：原住民传说借用有争议，bun 开头像蹭 Bun |
| Pogonip | `pogonip` | 美国西部的冰雾，原样冻住 | 波戈尼普 / 「破锅泥」 | 6 | 停在初筛：三音节，结尾 nip 有冒犯俚语 |
| Hern | `hern` | 方言的鹭；角落 | 读成「哼」 | 7 | 停在初筛：像 hernia 开头，4 字母搜不到 |
| Meh | `meh` | 波斯语的雾；英语「无所谓」 | 咩 | 5（致命 2） | 停在初筛：第一眼是「无所谓」，和定位相反 |
| Blenny | `blenny` | 鳚，从洞里探头张望 | 布莱尼 | 8 | 停在初筛：小众，bl- 加双写 n 难拼 |
| Goby | `goby` | 虾虎鱼；go by | 高比 / 戈比 | 3（致命 1） | 停在初筛：撞 Go 写的 Goby 语言和安全圈的 Goby 工具 |
| Coracle | `coracle` | 能背着走的小圆船，藏 oracle | 可乐壳 | 7 | 停在初筛：像蹭 Oracle，有同名 Nostr 客户端 |
| Dimma | `dimma` | 瑞典语的雾，藏 dim | 迪玛 | 8 | 停在初筛：只有 dim 一层，没故事 |
| Haar | `haar` | 英国东海岸的海雾 | 哈尔（《哈尔的移动城堡》） | 7 | 停在初筛：撞 Haar 级联和小波，像蹭吉卜力 |
| Tule | `tule` | 莎草；tule fog | 读成「吐了」 | 6 | 停在初筛：英文读法不唯一，「吐了」会成默认外号 |
| Mizzle | `mizzle` | 雾一样的细雨 | 米佐 | 7 | 停在初筛：-izzle 说唱梗盖过本意，有「搞糊涂」义 |
| Neap | `neap` | 小潮 | 尼普 | 7 | 停在初筛：冷门，没现成角色，有「搁浅」义 |
| Dugong | `dugong` | 儒艮 | 嘟公 / 近「督工」 | 9 | 停在初筛：「督工」和不催相反，和雾关系弱 |

**念起来好玩**（24 个）：先找念出来就好玩的词，再看它能不能长出角色和一句话来历。结果最好的是两类：鹭科鸟自己的叫声和民间叫法（Frank、Kwak、Butterbump、Dunkadoo），以及美国伐木工传说里的 fearsome critters（Squonk、Gillygaloo、Teakettler），属公有领域。纯拟声短词和网络梗 GitHub 裸名全被占；本轮查了约 100 个名字，短英文词没有一个空着。没收录的：poof（冒犯同性恋男性的英式俚语）、foom（AI 圈「硬起飞」黑话）、murmur（撞本地语音转文字 murmure 和 murmur3 哈希）、splat（撞 Gaussian Splatting）、noggin（大脑词）、mofo（粗口）、gawky（中文读出来像「搞基」）。不管产品最后叫什么，那只鹭都可以叫 Frank：东英格兰方言里灰鹭就叫 Frank，因为它一直在喊 fraaank（Moor《Suffolk Words》1823、BTO）。

| 名字 | 命令 | 含义 | 中文读法 | 初筛分 | 去留原因 |
|---|---|---|---|---|---|
| Squonk | `squonk` | 被逼急就化成一摊眼泪的传说动物 | 斯宽克；外号「哭包」 | 9 | 停在初筛：sq / nk 难念，npm 被在更新的 SQL 解析器占 |
| Gillygaloo | `gillygaloo` | 下方形蛋的传说鸟 | 吉利嘎噜，像「叽里咕噜」 | 7 | 停在初筛：10 字母，结尾 loo 和 Seelu 同病 |
| Kwak | `kwak` | 荷兰语的夜鹭，模仿叫声 | 「夸克」 | 3（致命 3） | 停在初筛：读成「夸克」，同类目撞名 |
| Frank | `frank` | 东英格兰方言里的灰鹭 | 弗兰克 | 6 | 停在初筛：常用人名搜不到；适合当吉祥物名 |
| Dunkadoo | `dunkadoo` | 美洲麻鳽的民间叫法 | 外号「嘟嘟」 | 7 | 停在初筛：doo-doo 是儿语便便，裸组织名被占 |
| Teakettler | `teakettler` | 叫声像烧开水壶的传说小兽 | 外号「茶壶」 | 6 | 停在初筛：10 字母，-tler 难念 |
| Fillyloo | `fillyloo` | 倒着飞的传说鸟 | 「非礼噜」 | 0（致命 1） | 停在初筛：「非礼」谐音，loo 结尾 |
| Borogove | `borogove` | 卡罗尔诗里像拖把的鸟 | 外号「菠萝」 | 8 | 停在初筛：卡罗尔造词被用滥，npm 是聊天 SDK |
| Upupa | `upupa` | 戴胜学名 | 乌噗啪；俗名「臭姑姑」 | 6 | 停在初筛：含 poo，读法不唯一 |
| Blorbo | `blorbo` | 粉圈俚语「你偏爱的那个」 | 外号「菠萝包」 | 11 | 细查判黄，1 票：npm 刚被活跃库占，中文用户不认识这个梗 |
| Pitapat | `pitapat` | 心怦怦跳、雨点啪嗒 | 噼嗒啪 | 8 | 停在初筛：角色不在名字里 |
| Bouba | `bouba` | bouba / kiki 效应 | 布巴 / 宝爸 | 8 | 停在初筛：读音同 booba（胸部表情、俄罗斯动画） |
| Hmmbird | `hmmbird` | hummingbird 的双关 | 嗯鸟 | 6 | 停在初筛：HMM 是隐马尔可夫模型，拼写不定 |
| Hmm | `hmm` | 想一想时的声音 | 嗯 | 4 | 停在初筛：拼法不定，搜不到，撞 HMM |
| Plip | `plip` | 水滴落进水面的轻声 | 普利普 | 6 | 停在初筛：太短，撞病理 AI 模型 PLIP |
| Plipo | `plipo` | plip 的两音节造词 | 扑哩啵 | 7 | 停在初筛：纯造词，梗全靠来历 |
| Dingbat | `dingbat` | 排版装饰符号；傻子 | 顶霸 | 8 | 停在初筛：英文轻度骂人，绑定 Zapf Dingbats |
| Longboi | `longboi` | 约克大学的高个鸭子 Long Boi | 外号「龙宝」 | 7 | 停在初筛：借已故名鸭，boi 网络腔易过时 |
| Pompom | `pompom` | 啦啦队彩球、绒球 | 蓬蓬 | 6 | 停在初筛：撞《星穹铁道》帕姆、布丁狗、同赛道终端宠物 |
| Whiffle | `whiffle` | 雁为快速下降翻身侧滑 | 威否 | 6 | 停在初筛：whiffy 有臭味义 |
| Bimble | `bimble` | 英式俚语，慢悠悠闲逛 | 宾宝 / 「病宝」 | 6 | 停在初筛：「病」谐音，近 bimbo |
| Boop | `boop` | 轻点鼻头 | 啵普 | 3（致命 1） | 停在初筛：撞开发者熟知的 macOS 应用 Boop |
| Yoink | `yoink` | 一把抢过来 | 哟因克 | 2（致命 1） | 停在初筛：等于自认在偷，已有终端视频下载工具 yoinks |
| Sploot | `sploot` | 动物肚皮贴地趴平 | 斯普鲁特 | 6 | 停在初筛：spl 难念，同名代码工具多 |

**藏着中文的英文名**（25 个）：只收 Yazi 式的整词拼音，即一个完整、土、日常的中文词，拼写里没有 q、x、zh、c、ü，并尽量带第二层意思。这个方向偏弱，适合当备选：查了约 55 个完整拼音词，GitHub 裸名一个都拿不到（大多是 2008–2014 年中国用户的闲置号）；npm 上的同名多是 2022 年前后的占位包；中文用户一眼认出中文词，名字在中文圈其实就是正式中文名，「中英两套身份」的问题换成了土味版本，和这一轮只定英文名冲突。如果想留一点拼音彩蛋，更合适的位置是吉祥物名、获取插件组织名或命令别名（推断）。查过但没收的：bage、hudie、gezi、lusi、lihua、shadiao、mogu、mohu、woniu、yumi，以及 malatang 等 8 个字母以上的词。

| 名字 | 命令 | 含义 | 中文读法 | 初筛分 | 去留原因 |
|---|---|---|---|---|---|
| Buding | `buding` | 布丁；近音补丁 | 布丁 | 9 | 停在初筛：像宠物名，英文读成 budding；拼音和只定英文名冲突 |
| Moyu | `moyu` | 墨鱼；近音摸鱼 | 摸鱼 | 8 | 停在初筛：搜索被摸鱼工具和魔方品牌 MoYu 淹没，近上一轮的 moyo |
| Gugugu | `gugugu` | 咕咕咕，网络用语「鸽了」 | 咕咕；外号「鸽子」 | 10 | 细查判黄，0 票：拼音，和只定英文名冲突；维护一慢就被吐槽「又咕了」 |
| Haita | `haita` | 海獭，腋下揣着最喜欢的石头 | 海獭 / 「害他」 | 8 | 停在初筛：名字本身没双关，听着像 hater |
| Lizi | `lizi` | 栗子；同音粒子、例子 | 栗子 | 8 | 停在初筛：英文读成 Lizzy，-zi 结尾像跟 Yazi |
| Juzi | `juzi` | 橘子；同音句子 | 「局子」 | 6 | 停在初筛：「进局子」同音，撞句子互动 |
| Digu | `digu` | 嘀咕；近音低估 | 嘀咕 | 7 | 停在初筛：`dig` 补全撞，2008 年有同名微博 |
| Doufu | `doufu` | 豆腐；缺字方框 tofu | 豆腐 | 7 | 停在初筛：「吃豆腐」俚语，doofus 联想 |
| Panta | `panta` | 盘它，盘出包浆 | 盘它 | 7 | 停在初筛：2019 年旧梗，偶带攻击性，裸名在活跃账号手里 |
| Laodeng | `laodeng` | 苍鹭的北方俗名「老等」 | 「老登」 | 3（致命 2） | 停在初筛：「老登」是爹味说教，和产品气质相反 |
| Zimu | `zimu` | 字幕；近音字母 | 字幕 | 7 | 停在初筛：描述型，会被当字幕下载器 |
| Mihu | `mihu` | 米糊；近音迷糊 | 小迷糊 | 7 | 停在初筛：让工具显得不靠谱，有冷门隐语 |
| Tuding | `tuding` | 图钉；近音秃顶 | 图钉 | 6 | 停在初筛：已有两个叫「图钉」的中文 App |
| Dabao | `dabao` | 打包带走 | 「大宝」 | 6 | 停在初筛：撞护肤品牌，意象偏囤积 |
| Miao | `miao` | 喵；同音瞄、秒、妙 | 喵 | 7 | 停在初筛：搜不到，英文 Miao 是苗族 |
| Wowo | `wowo` | 窝窝头；自己的窝 | 窝窝 | 7 | 停在初筛：npm 是安全占位，woe-woe 读法偏悲 |
| Nanie | `nanie` | 拿捏 | 拿捏 | 7 | 停在初筛：英文读成 nanny，「拿捏人」有控制义 |
| Dasuan | `dasuan` | 大蒜；近音打算 | 蒜蒜 | 6 | 停在初筛：会被当段子，suan 英文难念 |
| Ganhai | `ganhai` | 赶海 | 赶海 / 「干嗨」 | 6 | 停在初筛：赶海本身是网红短视频题材，「干嗨」谐音 |
| Huntun | `huntun` | 馄饨；近音混沌 | 馄饨 | 7 | 停在初筛：混沌偏古典哲学，英文带 hunt |
| Sanhua | `sanhua` | 三花猫 | 三花 | 2（致命 1） | 停在初筛：撞上市公司三花智控，「三花聚顶」偏玄学 |
| Hetun | `hetun` | 河豚，受惊鼓成刺球 | 河豚 | 6 | 停在初筛：「拼死吃河豚」带死，英文读法不稳 |
| Digua | `digua` | 地瓜；英文前半是 dig | 地瓜 | 6 | 停在初筛：接到小红书的「小红薯」，`dig` 补全撞 |
| Yingwu | `yingwu` | 鹦鹉，学人说话 | 鹦鹉 | 2（致命 1） | 停在初筛：「鹦鹉学舌」和自己的判断相反，还会想到 stochastic parrots |
| Mili | `mili` | 米粒；近音迷离 | 米粒 | 5 | 停在初筛：搜不到，是常见网名 |

**一个小动作**（21 个）：拿产品里或鹭身上的一个小动作当名字，让命令读起来像一句话（`guddle add`、`scry bottom`），中文侧能长出「X 一下」或现成外号。常用动词基本都被占，所以往三类词里找：苏格兰、英式、澳新的方言俚语，拟声和网络词，带技术或游戏双关的词（dither 是点阵抖动算法，scry 是万智牌的「占卜」）。按五个动作分配：收下、瞄一眼、拨开雾、沉底或放着、圈住或认领。筛掉的：suss（近音「萨斯」）、espy / descry（含 spy）、keek（和 Keep 只差一个字母）、gander（鹅撞 goose，已有同名 1.3k 星项目）、thunk（redux-thunk）、boop（Homebrew cask）、wheesht（看成 wee + shit）。Crane、Noodle、Lasso 列进来但标了不推荐，记下被撞掉的原因。

| 名字 | 命令 | 含义 | 中文读法 | 初筛分 | 去留原因 |
|---|---|---|---|---|---|
| Guddle | `guddle` | 苏格兰词，伸手到石头底下徒手摸鱼 | 嘎豆；外号「摸鱼」 | 12 | 入围，22 票，推荐 |
| Scry | `scry` | 用水晶球看；万智牌「占卜」 | 斯快；外号「占卜」 | 11 | 细查判红，0 票：nixpkgs / AUR 有 scry 二进制，USPTO 有第 42 类注册 |
| Dither | `dither` | 犹豫；点阵抖动算法 | 迪泽 | 8 | 停在初筛：th 难念，Go 生态有知名同名库 |
| Plowter | `plowter` | 在水里扑腾、瞎忙活 | 扑捞特 | 6 | 停在初筛：两种拼法，ow 读法不定；转作 Guddle 的插件组织备选，见 [§9](#9-获取插件组织的名字) |
| Gley | `gley` | 斜眼看；看漏、放过 | 格雷，像 grey | 8 | 停在初筛：口头会被拼成 grey，潜育土是发臭的泥 |
| Dowse | `dowse` | 拿探水棍找水 | 道斯 | 8 | 停在初筛：探水是伪科学，和 douse 同音 |
| Fossick | `fossick` | 翻找、在废矿里淘金 | 佛系克 | 8 | 停在初筛：藏着 sick，有同名 MCP 服务 |
| Snaffle | `snaffle` | 一把抓过来 | 斯纳佛 | 6 | 停在初筛：像爬虫，安全圈先想到 Snaffler |
| Dekko | `dekko` | 英式俚语，看一眼 | 德扣 | 9 | 停在初筛：梗淡，npm 和 Ubuntu Touch 旧应用已占 |
| Shufty | `shufty` | 匆匆看一眼 | 舒芙蒂 | 8 | 停在初筛：两种拼法，有同名 KYC 核验公司 |
| Squizz | `squizz` | 澳新俚语，打量 | 斯奎兹 | 6 | 停在初筛：squ 加双写 z 难拼，撞 Squiz |
| Glisk | `glisk` | 闪一下、瞥一眼 | 格力斯 | 6 | 停在初筛：同为本地桌面工具的 glisk 正在更新 |
| Steep | `steep` | 把茶泡在水里 | 意译成「泡」 | 2（致命 1） | 停在初筛：Ruby 的类型检查器命令就叫 `steep` |
| Marinate | `marinate` | 腌、浸泡入味 | 外号「腌」 | 7 | 停在初筛：8 字母三音节 |
| Bagsy | `bagsy` | 口头宣布「这个归我」 | 八哥 / 「白给」 | 7 | 停在初筛：「白给」带输，bag 贴近囤积 |
| Tootle | `tootle` | 嘟一声；漫无目的地溜达 | 「秃头」 | 7 | 停在初筛：秃头梗有人烦，方向和上一轮的 amble 重复 |
| Oho | `oho` | 「哦？」发现时的一声 | 哦吼 | 8 | 停在初筛：3 字母搜不到，像鸿蒙的 ohos |
| Smudge | `smudge` | 在起雾的玻璃上抹开一块 | 斯马吉 | 6 | 停在初筛：借猫表情包，裸名被活跃用户占 |
| Crane | `crane` | 伸长脖子看；鹤 | 意译成「鹤」 | 3（致命 2） | 停在初筛：Homebrew 已有 `crane`，落进国风 |
| Noodle | `noodle` | 琢磨；徒手抓鲶鱼；面条 | 面条 | 4（致命 2） | 停在初筛：Homebrew 已有 `noodle`，同在效率类的 noodle 有 1.2 万星 |
| Lasso | `lasso` | 套索、圈住 | 拉索 / 「啰嗦」 | 4（致命 2） | 停在初筛：Lasso 回归是 ML 术语，Homebrew 已有 `lasso` |

<a id="8-中文里怎么叫它"></a>
## 8. 中文里怎么叫它

**中文用户称呼英文名工具的几种方式：**

1. 原样念英文：名字短、好念时最常见，如 Zed、Bun、Deno、fish（推断，未找到系统性证据）。
2. 首字母：两词名会被缩成字母，CSDN 文章开头即写「Claude Code（下文简称 cc）」。
3. 意译：英文是实物词时直接翻译，Obsidian →「黑曜石」（CSDN、下载站）。
4. 译名迁移：「油猴」原是 Greasemonkey 的译名，Tampermonkey 流行后外号转给了它（掘金、少数派）；同类还有「暴力猴」。
5. 按图标：Shadowrocket 因图标叫「小火箭」（Wikipedia）。
6. 按吉祥物：OpenClaw 被叫「龙虾」，部署叫「养龙虾」，因为图标是红色机械龙虾（IT之家、中国日报）；Ollama 在中文教程里写作「羊驼」（CSDN）。
7. 名字本身是拼音：Yazi 的意思就是「鸭子」（README），中文用户不需要外号。

**外号什么时候长出来**（推断，基于上面的样本）：英文名长或难念（Tampermonkey、Shadowrocket），同时图标或吉祥物具体可画（猴、火箭、龙虾、羊驼），再加上有一个动作让大家一起做（部署 = 养），外号就会长出来。外号多半在中文教程和社群里扩散时出现，比如教程标题直接写「Ollama, 羊驼」。英文名短且好念时，外号一般不长，大家原样念。想要好外号，就让角色能用一两个汉字叫出来。

**不起正式中文名，丢掉和得到的：**

- 丢掉：对中文称呼的控制权（社区可能起一个你不喜欢的）；在小红书、B 站等中文平台的话题搜索里少一个现成的中文关键词；中文用户第一次见面不确定怎么念。
- 得到：避开国风和「NPC 感」；不再有中文名和英文名两套身份互相拖累（思鹭的问题正是英文名寄生在中文名上，见 [§1](#1-思鹭为什么不行)）；避开中文谐音雷（思路 / 死路）；社区自己起的外号更亲近（养龙虾比任何官方中文名都好传）。

**文案里中文怎么写：**

- 名字在中文文案里一律写英文原文，不加书名号，不配官方中文名。
- 第一次出现时用括号补一句读音和角色，例如「Guddle（读作「嘎豆」，一只伸手到石头底下摸鱼的鹭）」；之后只写 Guddle。
- README 中文版写一句汉字读音提示，只给一种；常见误读不在官方材料里列为备用读音。
- 来历只陈述事实（「guddle 是苏格兰词，意思是徒手摸鱼」），不在官方材料里钦定外号，也不写「摸鱼神器」一类的话。
- 吉祥物设计成一个具体、能用两三个汉字叫出来的角色（「小鹭」「那只鹭」），给外号留路。

**要不要、什么时候给官方中文外号：** 先不给。等社区里出现一个稳定的叫法（中文教程标题、社群里反复出现、搜索里能搜到），并且它不负面、不撞别人的名字，再在 README 里顺着用，像 OpenClaw 接住「龙虾」那样；社区长出的叫法不合适时，官方不跟着用就行，不去纠正。上一轮的叙事写法见 [调研 07 §10](07-positioning-and-naming.md#10-叙事与介绍定名后第二版)，定名后要按这一节重写。

**推荐名和备选各一句：**

- **Guddle**：原样念「嘎豆」，外号大概率是名字直译过来的「摸鱼」，动作现成（「guddle 一下」「摸一条进来」），不用官方钦定；官方只要别把它往「划水工具」上带。
- **Grepling**：开发者原样念「格瑞普林」并说「grepling 一下」；普通用户读不出 grep 的梗，外号只能从角色上长，大概是「小鹭」，所以吉祥物要比名字更早出现在中文材料里。
- **Fogdog**：两个闭音节好念好拼，外号「雾狗」几乎一定会长出来，能接上狗头文化；要提防简称成太泛的「狗子」。
- **Butterbump**：没人会念原名，外号「黄油包」会立刻接管名字；风险在被再缩成「黄油」，中文材料里最好一开始就把吉祥物写全、写成「一只黄油色的大麻鳽」，减少往「黄油」那边滑。

<a id="9-获取插件组织的名字"></a>
## 9. 获取插件组织的名字

照 Mihon 和 Keiyoushi 的关系选：两者都是日语普通名词，彼此不讲同一个故事，只是语域相同。所以每个主名配同一语域的名字：Guddle 配苏格兰口语，Grepling 配「把 Unix 命令藏在自然词里」的造法，Fogdog 配带梗的普通英文词。九个名字互不重复，都不含主名。GitHub 一栏是 2026-11-01 用 `gh api users/<名字>` 和 `gh search repos` 只读查到的；404 只说明今天没有这个账号或组织，不保证以后还空着。所有读音和中文外号都是推断，没有找中文用户试读过。

**配 Guddle**（倾向 plowter 或 trouttickle）：

| 名字 | 含义 | GitHub | 备注 |
|---|---|---|---|
| plowter | 苏格兰词，plouter 的另一种写法：在湿的东西里扑腾、拨弄，也指瞎忙活（Wiktionary）；guddle 是伸手摸鱼，plowter 是在水里扑腾 | `plowter` 404；主写法 `plouter` 被一个 0 仓库的 User 占（2026-10-03 创建） | 梗最直白，中文顺口叫「扑腾」；只是异体写法，英文读者可能先看成 plotter；7 个字母，三个里最短 |
| trouttickle | 来自 trout tickling：用手指揉鳟鱼肚皮，鱼进入恍惚后捞起（Wikipedia），和 guddle 同类；插件对平台也是轻轻挠，等它交出内容 | `trouttickle`、`troutticklers` 都是 404，没有同名仓库 | 梗最多，中文可叫「挠鳟鱼」；意思和 guddle 贴得太近，可能被当成主名的英格兰译法；11 个字母。Wiktionary 里 trout 作动词有「拿湿鳟鱼抽人」的比喻义，是否源自 IRC 待核实 |
| burnstanes | 苏格兰词拼的造词（推断，没查到有人这样用过）：burn 是溪流，stane 是 stone 的方言写法（Wiktionary），溪里的石头就是 guddle 时伸手去摸的地方；每个插件是一块石头 | `burnstanes`、`burnstane` 都是 404，没有同名仓库 | 读音近「伯恩斯坦斯」，可能被叫成「伯恩斯坦」（推断），让人以为和某个人有关；10 个字母 |

备选：fullcreel（满篓；creel 是钓鱼人装鱼的柳条篓，Wiktionary），查询 404，故事清楚但梗弱。

**配 Grepling**（倾向 pipewort 或 sedges）：

| 名字 | 含义 | GitHub | 备注 |
|---|---|---|---|
| pipewort | 真词，谷精草属（Eriocaulon）的几种水生植物（Wiktionary；中文属名「谷精草」是推断）；pipe 正好是 Unix 管道符，插件把内容经管道交给主程序，写出来就是 `pipewort \| grepling`；长在鹭站的浅水湿地里 | `pipewort` 404，没有同名仓库 | 造法和 Grepling 一样，语域一致又不含主名，三个里最不容易撞名；读音大约「派普沃特」，外号可能是「管子草」（推断） |
| sedges | 真词，三层：莎草科、薹草属植物；集合名词「一群鹭、鹤或麻鳽」（a sedge of herons，Wiktionary）；sed 是 Unix 流编辑器，常和 grep 在同一条管道里 | `sedges` 404；单数 `sedge` 被一个 71 仓库、2026-10 还在更新的 User 占 | 和鹭最贴，「a sedge of herons」本身是冷知识梗；必须用复数；NethermindEth/sedge（194 星，PoS 验证节点部署工具）在 GitHub 搜 sedge 时排在前面；程序员可能叫它「sed 哥」（推断） |
| awklets | 造词：awk 加 -let，读音和 auklet（小型海雀，Wiktionary）一样，和 grepling 是同一窝的兄弟 | `awklets`、`awklet` 都是 404；已有 0 星仓库 WasabiFan/awklet | 结构对称；中文用户多半先想到 awkward，外号变成「小尴尬」（推断）；海雀不是鹭，离主名的世界稍远 |

**配 Fogdog**（倾向 fetchquest）：

| 名字 | 含义 | GitHub | 备注 |
|---|---|---|---|
| fetchquest | 真词组，游戏里的跑腿任务：拿到某件东西再交回给发任务的人（Wiktionary）；狗的 fetch 加跑腿任务，每个插件去一个平台把内容叼回来 | `fetchquest`、`fetchquests` 都是 404；已有小仓库 gmuGADIG/FetchQuest（7 星，狗主题小游戏）、FluidXR/fetchquest（7 星，把 Meta Quest 截图同步到本地）、genc-murat/fetchquest（2 星，Rust HTTP 客户端） | 语域一致，带点自嘲；中文外号直接是「跑腿」（推断）；三个里最推荐 |
| scentwork | 犬类嗅探运动，靠气味找目标；Wiktionary 收的是同义词 nosework，scentwork 本身查不到条目，是否英国叫法待核实 | `scentwork` 404；`nosework` 被一个 0 仓库的 User 占 | 中文直译「嗅探」，技术圈里是抓包、嗅探器，容易被误解成爬虫或监听工具（推断），正是用户不想要的误解；英国可能有同名犬类运动组织（待核实） |
| brockens | 来自 Brocken spectre：观察者站在太阳和云雾之间时，影子被放大投在雾上（Wiktionary 记作 mountain spectre 的同义词）；和 fogdog 是同一类雾中光学现象，一个是光，一个是影 | `brockens` 404；单数 `brocken` 被一个 0 仓库的 User 占（2010 年） | 会被当成 broken 拼错（推断），爬取类插件本来就常因平台改版失效，可能让人真以为坏了，排第三；不想冒险可换 fetchyard（造词，查询 404，梗弱） |

**排除掉的名字和原因：**

- turnstone(s)（翻石鹬，翻石头找吃的，本来很配 Guddle）：turnstonelabs/turnstone 是 1653 星、今天还在更新的 AI 编排平台，同在 Agent 圈，算蹭。
- tiddlers（英式说法里的小鱼苗）：TiddlyWiki 把每条笔记叫 tiddler，撞上用户不想被当成笔记软件这一点。
- peasoupers（伦敦浓雾的俚语，查询 404）：Wiktionary 上第二个义项是对法裔加拿大人的称呼，有冒犯风险。
- 裸名已被占用的：creel、kelpie、riffle、sedge、auk、tarn、headwaters、haar、kennel、lamppost、zoomies、sniffari、puppers、goodbois、awoo 等。

<a id="10-待核实清单"></a>
## 10. 待核实清单

定名前要补的，排在前面的可能推翻推荐。上一轮的清单见 [调研 07 §9](07-positioning-and-naming.md#9-待核实清单)。

1. **正式商标检索。** 在 USPTO、EUIPO、UKIPO、CNIPA 按第 9 类（软件）和第 42 类（软件服务）查 GUDDLE、GREPLING、FOGDOG；Grepling 顺带看 Greptile 的商标范围有没有覆盖 GREP- 开头的词；Fogdog 查美国注册号 5,101,851 的现状。
2. **国内平台同名。** 小程序、国内安卓应用商店、小红书、B 站、抖音、公众号里有没有叫 Guddle / Grepling / Fogdog 的产品或账号；中文搜索「guddle 摸鱼」现在能搜到什么。
3. **社交账号名。** X、Bluesky、Mastodon、小红书、B 站、即刻、Discord 上推荐名和备选的用户名是否可用（本轮没查）。
4. **GitHub 组织名。** Guddle 的裸名 `guddle` 在一个仍有零星活动的个人号手里：是联系转让，还是直接用 `guddlehq`。名字定下后，组织名要由用户本人尽快注册，现在空着不代表一直空着（`dotty-app` 就是 2026-09 才被人注册的）。GitHub 近期是否还受理释放闲置用户名，社区回复说商标纠纷以外不再受理（GitHub Community #197291，回复日期未确认）。
5. **Guddle 的同名产品。** AlternativeTo 上那个 PRINCE2 项目管理工具 Guddle 还在不在运营；guddle.com 是否在 Afternic 挂牌（只作同名信号）。
6. **找真人试读。** 找几个中文用户和非开发者实际念一遍、听一遍：Guddle 读成什么、会不会想到「摸鱼」、能不能听一遍就拼出来；Grepling 在非开发者那里能不能叫出口。本篇所有读音和外号判断都是推断。
7. **Guddle 的词义细节。** Urban Dictionary 那几条性暗示的分量；Wiktionary 以外日、西、德、法、韩语里有没有同形词（只查了 Wiktionary）。
8. **App Store 分区。** Holdout、Blorbo、Scry、Spoony、Cuttle、Nah 的被占应用分别在哪一区（[§5](#5-可用性检查) 里中国区写了「未知」）。
9. **Potoo 的 npm。** 已于 2026-11-01 复查：被一个 2017 年发布的包占着（见 [§5](#5-可用性检查)），这一项不用再查。
10. **Butterbump 的出处。** 「诺福克」「1671 年首见」两说只有二手出处（wehd.com 转引 Skinner 词典），原文待核实；中文 ACG 圈「黄油 = 成人游戏」的流行程度只有一条评论佐证。
11. **其他细查名的缺口。** Grepling 的其他语言含义；Fogdog 的中国商标和 Fogdog Cold Brew 是否仍在经营；Holdout 在 PyPI 是否带命令行入口；Stilly、Spoony、Caligo 的商标搜索；Bernat 的中国商标；`stale` 是否为 GitHub 保留名。
12. **插件组织名的词义。** scentwork 是否英国叫法、有没有同名犬类运动组织；trout 作动词的比喻义是否源自 IRC；pipewort 的中文属名「谷精草」。
13. **参考软件的来历。** [§2](#2-有梗的独立软件名字) 表中标「未核实」「待核实」的各项（Ollama、Clawd、Zed、Jellyfin、Immich、Raycast、Excalidraw、tldraw、Zellij、goose 的来历，Karakeep 改名是否因为商标纠纷，以及 Ghostty、Bun、Deno 等的中文叫法）。
