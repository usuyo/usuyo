<a id="调研-13--第四轮像-yohaku-的名字"></a>
# 调研 13 — 第四轮：像 Yohaku 的名字

状态：草案 · 2026-11-04 · 2026-11-04 补 [§6](#6-11-04-复查-usuyo)

回答你 11-04 的一句：「dokokara备选，然后我想再找个yohaku这种的」（记在 [open-decisions §1.8](../open-decisions.md#18-grill-回答)，对应 [discussion 第 5 条](../discussion.md#5-名字)）。推荐和要你回答的一问在 [§0](#0-结论)；挑名字的尺子在 [§1](#1-这一轮按什么挑)；查过的 101 个名字在 [§2](#2-候选和可用性)；前三名和 Dokokara 的细查在 [§3](#3-入围细查)；怀疑者改了什么在 [§4](#4-怀疑者复核)；定名前还要查的在 [§5](#5-待核实清单)。本篇不改别的文档。

<a id="约定"></a>
## 约定

- **查证日期**：全文事实都是 2026-11-04 查的；Dokokara 的几条取自 [09 §12.4](09-fog-and-name.md#124-dokokara-复查和实际问题)，是 2026-11-03 查的。
- **流程**：四个方向生成候选（日语审美词、日语排版和书画词、雾与缝、其他语言）；101 个查占用；其中 24 个细查词义、误读和同名产品；一位终评把这 24 个加上参照 Dokokara 打分排名；一位怀疑者复核前三名。各步的原始记录在流程笔记里，不进仓库。
- **怎么查的**：GitHub 用 `gh api`（`users/<名>`、`<名>-app`、`<名>hq`、`search/repositories`、`search/users`）；npm 用 `npm view`；PyPI JSON、crates.io API、Homebrew formula 和 cask JSON、iTunes Search API（cn、jp、us，iPhone 和 Mac）、kotobank、Jisho、Wiktionary、汉典、萌典 用 `curl --max-time 30 -A "research-client/1.0"`，不带邮箱或任何个人信息。只读：没有注册、提交表单、star、fork 或评论，没有安装或运行第三方工具。
- **词典**：日语每个词至少两部，kotobank 上的 デジタル大辞泉（下称大辞泉）和 精選版 日本国語大辞典（下称日国），另有字通、日本大百科全書、世界大百科事典；Jisho 作旁证。
- **「空着」**只说明查的时候没被占。GitHub 一般不释放休眠账号，裸名被休眠号占着时，Go 模块路径要挂在 `<名>hq` 或别的名下（推断）。商标没有正式检索；.com 只当同名信号，你不买域名。
- **分数**是终评按 [§1](#1-这一轮按什么挑) 的八条打的，1 最差、5 最好，满分 40，不是你定的。怀疑者改过的分见 [§4](#4-怀疑者复核)，正文用改后的分。
- **推断**：中英文读者怎么念、会想到什么，都是推断，没有找真人试读。
- **名字**：正文写「本产品」，照 [overview](../overview.md#整体设计供-review) 的占位规则。

<a id="0-结论"></a>
## 0. 结论

**推荐 usuyo（薄様，うすよう），命令 `usuyo`。** 怀疑者复核后它 32 分，和 hihaku 并列；没碰到门槛的名字里，除了不算审美或版面词的 mabara（33），它分最高。

- 词：薄的鸟之子纸、雁皮纸；大辞泉第三义是袭的色目，同色几件叠穿，由外到内渐浅，最下两件是白（[kotobank 薄様](https://kotobank.jp/word/%E8%96%84%E6%A7%98-439972)）。是和纸、平安书写那一圈的审美词。
- 声音：5 个字母，u-su-yo 三个开音节，是这一轮最像 yo-ha-ku 的。
- 占用：GitHub 裸名、`-app`、`hq` 全 404（usuyou 也是），Go 模块路径多半能写 `github.com/usuyo`（推断，404 分不出没人用和被保留）；npm、PyPI、crates、Homebrew 全空；同名仓库 0；App Store 中日美 0。
- 中文：薄样 汉典、萌典都没有词条，读不回现成中文词，不会长出中文名。
- 让步：留白要讲两层（先讲薄纸，再讲最下两件是白），第一眼没有意思；usuyo.com 是一个在运营的立陶宛语软件推荐站，品牌就叫 Usuyo；日本人习惯拼 usuyou。

**另一条路：hihaku（飛白，ひはく），32 分。** 留白就在词义里：笔画里没沾墨的白一段段露出来，字通引《书断》字要轻、不要满；占用也干净，只有裸名被 2021 年的休眠号占着。卡在门槛 G2：README 一露汉字，中文读者就会叫它「飞白」，这是汉典收的书法和国画词，等于有了中文名。怀疑者还查到，辞典写 飛白「筆勢が飛動」「勢いよく飛翔」，是宫殿题额的书体，不安静。你能接受「飞白」这个外号，就换成 hihaku。

**第三：rangai（欄外，らんがい），29–30 分。** 终评原推荐它（33 分），理由是「让步最少」。怀疑者查到：拼音能切成 rang-ai，读成「让爱」「让 AI」；英文会看成 Rang AI；印地语 रंगाई 是「染色」。这条理由不再成立，本篇把推荐换成 usuyo（[§4](#4-怀疑者复核)）。

**没有一个名字到满意的程度**（终评的判断）：25 个里没有一个同时做到词义本身是留白、6 个字母以内且声音软、读不回中文词、不撞名。mabara（疎ら）33 分最高，但它的意思是「稀疏」，不是留白，也不是审美或版面词，你第一条不过，怀疑者也没复查，只在你放宽第一条时考虑。Dokokara 仍是你的备选；按这一轮的尺子它 24 分：不是留白义，8 个字母 4 个音节。

| 名字 | 原文和意思 | 字母 / 音节 | 分 | 撞名 | 中英怎么念 | 首屏一句 | 弱点 |
|---|---|---|---|---|---|---|---|
| usuyo | 薄様：薄的鸟之子纸、雁皮纸；袭的色目里由外到内渐浅，最下两件是白（大辞泉第三义、日国第二义） | 5 / 3（日本人拼 usuyou，6） | 32（终评 33） | GitHub 三处 404；注册表全空；同名仓库 0；App Store 0；usuyo.com 是在运营的「Usuyo」软件站 | 中「乌苏哟」，先想到乌苏啤酒；英 oo-SOO-yo 或 YOU-su-yo（推断） | Layered like thin paper, the patch you follow shows through; the rest can stay in the fog. ／ 像几层薄纸叠着，你跟着的那一片透出来，其余的留在雾里也没关系。 | 留白要讲两层；冷僻；文案写「淡到白」会读成雾在退（推断）；两种拼法；日本读者可能读成 うすよ（推断，弱） |
| hihaku | 飛白：汉字书体，刷子状的笔写出枯笔，笔画里的白一段段露出来；字要轻、不要满（大辞泉、日国、字通、日本大百科） | 6 / 3 | 32（终评 34）；G2 | 裸名是 2021 年休眠个人号，`-app`、`hq` 404；注册表全空；精确同名 0；App Store 0；hihaku.com 2016 年起被注册，内容读不出 | 中「嘿哈库」，露汉字后叫「飞白」；英 hee-HAH-koo，也可能念成 high-（推断） | Like flying-white brushwork, white runs through every stroke; the rest can stay in the fog. ／ 像飞白那一笔，笔画里本来就露着白；其余的像一层轻云，留在雾里。 | 读回「飞白」（G2）；和「被爆 hibaku」差一个字母；和 Innei 的「余白 / Yohaku」同落「X白」，可能被说蹭（推断）；词义偏笔势飞动；多数日本人把 飛白 读成 かすり（推断） |
| rangai | 欄外：印刷部分或框线以外的地方，例「欄外の余白」「欄外に注記する」（大辞泉）；日国第一义是栏杆外 | 6 / 2 | 29–30（终评 33） | 裸名是 2012 年个人号，末次推送 2022，资料 2024-11 改过；`-app`、`hq` 404；注册表全空；精确同名 1（0 星）；App Store 0；rang.ai 是在运营的俄语 SaaS | 中「然盖」，或切成「让爱」「让 AI」；英 RANG-guy、ran-GUY，或 Rang AI（推断） | Your own line goes outside the frame; the rest can stay in the fog. ／ 你自己的那句话写在框外，其余的留在雾里也没关系。 | 「让 AI」和「机器写的排在你的话下面」正反着（推断）；印地语「染色」；ng 接 g 偏硬；语感偏表格、课本（推断）；会被叫「栏外」 |
| Dokokara（参照） | どこから：「从哪儿？」；答句是零食袋边的「こちら側のどこからでも切れます」（[09 §12.4](09-fog-and-name.md#124-dokokara-复查和实际问题)） | 8 / 4 | 24；G1、G4 | 拉丁写法 GitHub、各注册表、App Store 五区全空；Docker Hub 用户名被占；日区有「どこから片付け？AI」 | 中要四个字标音；英 doh-koh-KAH-rah（推断） | Saved hundreds of things? Open them from any corner. ／ 收了几百条，不知道从哪看起？从哪一角打开都行。 | 不是留白义；8 个字母 4 个音节；问句旁边永远要跟答句；「说能撕开、其实撕不开」的反讽；近音 ココカラ |

**要你回答的一问**

> hihaku（飛白）的留白就在词义里，书法里笔画露白、字不写满，占用也干净；但 README 一露汉字，中文读者就会叫它「飞白」，这是中国书法和国画的现成雅词，等于有了一个中文名。你能接受吗？

- **能 → hihaku。** 还要一并接受：和「被爆 hibaku」只差一个字母；和 Innei 的余白同落「X白」，可能被说蹭（推断）；飛白 的本义是笔势飞动，不如 yohaku 安静。
- **不能 → usuyo。** 留白要讲两层，名字冷僻；搜「usuyo」可能先出那个立陶宛软件站（推断）。
- **usuyo 也嫌冷僻 → rangai。** 要接受被读成「让 AI」「Rang AI」，和被叫成「栏外」（栏外 不是汉典词）。

<a id="1-这一轮按什么挑"></a>
## 1. 这一轮按什么挑

你 11-04 的原话（逐字）：

> 「dokokara备选，然后我想再找个yohaku这种的」

问「像 Yohaku」像哪几点时，四个选项你都选了（选项文字照 [open-decisions §1.8](../open-decisions.md#18-grill-回答) 的记录）：「日语里的美学或版面词」「意思：留白、不填满」「发音短、安静、字母少」「不限日语，只要气质像」。

这一轮把它读成四条（我们的读法）：

1. 日语审美词或版面词；
2. 意思是留白、不填满；
3. 短、安静、字母少：2–3 个音节，最多 6 个字母，命令就是名字本身；
4. 不限日语，气质像就行。

Yohaku 本身不用：GitHub 裸名、`-app`、`hq` 全被占，同名仓库 304 个，Innei 有还在更新的「余白 / Yohaku」，Mac 上有「YOHAKU – liquid notes」（[09 §12.3](09-fog-and-name.md#123-yohaku-细查)，2026-11-03 查）。

原有规矩照旧：只用拉丁字母，不起中文名（[08 §1](08-english-name.md#1-思鹭为什么不行)）；不撞名、不常见；这一轮只收词典里的真词，不收造词；中英好念；不制造压力，不带清空、进度、治好的意思；不被误解（中文近音、英文俚语、日语语感都查）；前几轮否过或选过的不再用，除非这一轮明显更合适，并标出当时的判词。开头就排除：yohaku、dokokara（只当参照）、toedip、pertugio、anazuri、guddle、grepling、fogdog、butterbump、seelu、xiecun。

**终评的尺子**：八条等权，每条 1–5 分。

1. 像 Yohaku：审美或版面词，词典义本身是留白，语感安静；
2. 和雾、首屏：名字的画面落在点阵雾里，首屏一句不认识名字也成立，不承诺清雾；
3. 不撞名：GitHub 三处、四个注册表、App Store；
4. 真实可核实：两部以上辞典收，罗马字是通行写法；
5. 短而安静：≤6 个字母，2–3 个音节，开音节，一种读法；
6. 中英好念；
7. 不制造压力；
8. 不被误解：负面义、近音、读回现成中文词。

**门槛**（不进分数，碰到就不推荐）：G1 超过 6 个字母；G2 README 露出汉字后读回汉典收的现成中文词，社区就有了中文名（余白 汉典没有，所以 Yohaku 本身不算）；G3 [09 §3](09-fog-and-name.md#3-命名标准第三版) 的「直接叫雾」，或名字本身是压力词；G4 八条里有一条 1 分。

<a id="2-候选和可用性"></a>
## 2. 候选和可用性

**细查过的 24 个和 Dokokara。** 分数是终评的，前三名是怀疑者改后的（[§4](#4-怀疑者复核)），其余没复核。

| 名字 | 原词 | 分 | 门槛 | 判定和主要原因 |
|---|---|---|---|---|
| usuyo | 薄様 | 32（原 33） | — | **推荐**。占用全空，声音最像；留白只在大辞泉第三义，要讲两层 |
| hihaku | 飛白 | 32（原 34） | G2 | **你接受「飞白」时推荐**。笔画露白；读回 飞白；形近 被爆 |
| rangai | 欄外 | 29–30（原 33） | — | **第三**。版面词；让爱、让 AI、Rang AI |
| mabara | 疎ら | 33（未复核） | — | 放宽你第一条时考虑。稀疏不是留白；日常偏冷清（まばらな拍手） |
| kasuri | 絣 | 30 | — | 不推荐。模糊不是留白；同音 掠り 有回扣义；英文搜索被香料 kasuri methi 占满 |
| tenbyo | 点描 | 29 | — | 不推荐。点不是白；裸名 2026-10-07 刚被不明账号建；日本人拼 tenbyou（7） |
| amiten | 網点 | 29 | G2 | 点不是白；读回「网点」（快递、银行）；裸名是活跃小组织 |
| shiroji | 白地 | 28 | G1、G2 | 还没写的白，意思贴；7 个字母；前缀撞 Innei/Shiro |
| kasuka | 幽か | 28 | — | 隐约，是雾本身不是留白；副义寒酸；同名练习仓库 27 个 |
| gengai | 言外 | 27 | G2、G4 | 拼音近「更改」，正撞「原文不改」；中区有「言外」读书摘句应用 |
| honoka | 仄か | 27 | G4 | 常见女名；npm 被占；Bootstrap 主题 Honoka 656 星 |
| jisage | 字下げ | 26 | — | 释义里有「空白」，但像排版术语；Obsidian 社区插件叫 Jisage |
| oboro | 朧 | 26 | G3 | 词典归为夜里的霞，就是直接叫雾 |
| yochi | 余地 | 26 | G2、G4 | 近音「幼齿」；同音 予知；npm 占位包 |
| nijimi | 滲み | 26 | G4 | 晕开、渗，不是留白；「血のにじむような努力」 |
| ichigu | 一隅 | 25 | G2 | 「一隅を照らす」是劝勉格言；天台宗官方应用；看成 ichigo |
| nagori | 名残 | 25 | G3 | 残留；英文开头 nag；mhiro2/nagori 是本地优先剪贴板工具 |
| gyokan | 行間 | 24 | G2 | Mac 有「GyoKan Select - 行間」；gyo 中英都难念 |
| ikkaku | 一角 | 24 | G2 | 最常用搭配「冰山一角」；搜索被《BLEACH》占满 |
| kasure | 掠れ | 24 | — | 日常是打印掉色、嗓子哑；GitHub 有同类「かすれのあるノート」 |
| tenchi | 天地 | 24 | G2、G4 | 中文常用大词；PyPI 被活跃框架占 |
| Dokokara | どこから | 24 | G1、G4 | 参照。不是留白义；8 个字母 4 个音节 |
| kehai | 気配 | 23 | G4 | 同类目同名：Mac 效率应用 Kehai 11-03 刚发 0.6.8 |
| sukashi | 透かし | 22 | G1、G4 | 7 个字母；suka 在俄、英、波兰语里是脏话；すかしっ屁 |
| kaigyo | 改行 | 21 | G2 | App Store 已有「Kaigyo」；读回「改行」（转行） |

**只查了占用的 77 个。** 判定按占用那一步的规则：clean 是裸名空或休眠、npm 和 crates 空、没有同类产品；usable 只有轻微占用；crowded 是很多同名，或有同类产品但不活跃；taken 是有活跃的同类产品或知名品牌。

| 判定 | 名字 |
|---|---|
| clean（17） | pousio、mezera、gilid、ledig、luang、glipa、egwyl、reunus、imeall、brache、puwang、okasa、taema、usubi、kumoma、kyufu、soupir |
| usable（11） | kugeki、vacat、lihi、lapang、yeoyu、nafasi、fursat、tauko、andrum、kenon、reglet |
| crowded（31） | awai、kinari、hazama、kuhaku、maai、yoin、kasumi、aki、amari、nodo、nagomi、yojo、fuchi、boten、folga、karumi、orihon、hamish、vahe、pengo、jeda、sosta、virama、otium、pausa、sunya、spatium、kireji、nagame、kajo、postil |
| taken（18） | yoyu（Mac 笔记「Markdown YOYU」）、sukima（「Brain Dump: Sukima」）、kiji（笔记应用和本地 RSS 阅读器）、nokori（PyPI 上的 AI 规则记忆）、asobi（crates 上的 AI agent 知识图谱）、yutori（AI agent 公司）、aida（AIDA64）、masume（npm 带同名命令）、sumi（local-first 的 macOS 应用）、hanshi（macOS Markdown 笔记）、yeobaek（韩区多款 여백 读书记录应用）、okraj（11-02 上架的笔记应用）、kenar（10-22 上架的 AI 稍后读）、chora（crates 上的 Markdown 笔记命令行）、lacuna（三个注册表被占）、margo（Margo 组织和 Markdown 编辑器）、tacet（同类目活跃应用）、elpa（GNU ELPA） |

clean 和 usable 的 28 个没有细查词义、误读和同名产品，不算出局（[§5](#5-待核实清单)）。字面就是「留白、页边」的词（kuhaku 空白、yeobaek 여백、okraj、kenar、margo、lacuna）都撞名，其中四个撞的是 2026 年刚上架或刚更新的笔记、稍后读应用；推断这个意象最近很多人在用。

**前几轮出现过的**（在 [07 §7](07-positioning-and-naming.md#7-全部候选)、[08 §7](08-english-name.md#7-全部候选)、[09 §9](09-fog-and-name.md#9-全部候选) 按词搜过）：

- **ichigu**：09 初筛 4 分出局，理由「天台宗语录加国风雅词，属第一轮被否那一类」；[09 §12.2](09-fog-and-name.md#122-理解) 承认「属第一轮被否那一类」推断过头。这一轮按新查到的另判：劝勉义、天台宗应用、ichigo 混淆、G2，仍不推荐。
- **sukashi**：09 列在「查过但没进候选表」里，没写原因。这一轮查到大辞泉名词义「留出缝隙」，比当时贴，但 7 个字母加 suka 脏话义，仍出局。
- **sukima**：09 初筛 8 分存活，理由「中文 ACG 圈『隙间』被东方 Project 占，npm 被占」；这一轮又查到「Brain Dump: Sukima」，taken。
- **yoin**：07「余音 / Yoin」初筛 7 分，原因「『三日不绝』会被读成放不下；yoin 读音不定」；这一轮 App Store 日区 9 个同名，crowded。
- **postil**：和 07 的「按语 / Postilla」（初筛 7 分）同根，crowded。

其余 96 个在 07、08、09 里都没出现过。

<a id="3-入围细查"></a>
## 3. 入围细查

<a id="31-usuyo薄様"></a>
### 3.1 usuyo（薄様）

- **词义**：大辞泉 ① 薄的鸟之子纸、雁皮纸，泛指薄和纸；② 上浓下淡的晕染，也叫曙染；③ 袭的色目，「下の2枚を白にする」。日国 ① 雁皮薄纸，最早书证《宇津保物語》，平安中期以后用于情书和亲密的赠答，叠成「薄様重ね」；② 袭的色目，「最後の一枚を白にする」。世界大百科：染色的薄様叠起来，下面的颜色透上来（[kotobank 薄様](https://kotobank.jp/word/%E8%96%84%E6%A7%98-439972)）。Jisho 不标常用词；Wiktionary 没有 usuyo、usuyou、薄様 词条。
- **留白在哪**：只在大辞泉第三义（日国第二义）的「最下两件是白」，第二义讲渐变，不讲白。所以终评这一条给 3：要先讲纸，再讲白。和雾的关系在世界大百科那句「叠起来，下面的透上来」。
- **占用**：见 [§0](#0-结论) 的表；另外 GitHub 登录名含 usuyo 的只有 usuyong、usuyoshi，topics 0。usuyo.com 2025-12-16 注册，立陶宛语，自称「Usuyo portale」，是应用和游戏推荐、软件评测站，列表更新到 2026 年 7 月。
- **误读**：中文「乌苏哟」，偏好玩，不负面（推断）；英文可能拆成 use yo(u)（推断，弱）；日语和「嘘よ」差一个元音（推断，弱）；Jisho 搜 うすよ 返回 薄様、薄葉紙、薄汚れる，日本读者看到 usuyo 可能联想到「薄汚れ」（推断，弱）。
- **当标识**：`usuyo`、`.usuyo/`、`USUYO_`、`usuyo://` 都短（推断）。README 要说明 usuyo、usuyou 两种拼法。

<a id="32-hihaku飛白"></a>
### 3.2 hihaku（飛白）

- **词义**：大辞泉 ① 汉字书体，用刷子状的笔写出枯笔，相传后汉蔡邕所创，用于匾额；② 絣；③ 同双钩。日国最早书证是空海《性霊集》（835 年前后）。字通「字画に空処が多い」，引《书断》字要轻、不要满。日本大百科：笔画里没沾墨的白「とびとびに現れる」。同页也写「勢いよく飛翔している独特の書体」，マイペディア「筆勢が飛動するところから名付けられ」，代表作是唐太宗、高宗、则天武后的题额（[kotobank 飛白](https://kotobank.jp/word/%E9%A3%9B%E7%99%BD-120608)）。
- **中文**：[汉典 飞白](https://zdic.net/hans/%E9%A3%9E%E7%99%BD) 三义：书法字体；国画枯笔露白，书证《世说新语》「飞白拂其上，使如轻云之蔽日」；修辞里故意写白字。萌典 飛白 也有词条。第二义正好接上雾，第三义可能被拿来开「错别字」的玩笑（推断，轻）。
- **日语读法**：Jisho 搜 ひはく 只返回「非白人」，搜 飛白 只给 かすり 一个读法；同音 菲薄 是「粗陋、才能贫乏」，书面语，轻。推断多数日本读者看到 飛白 先读 かすり，ひはく 是书法圈读法。
- **形近**：hibaku 被爆（原子弹受害）在日本是很重的词，打错、扫一眼都可能看成它（推断）。
- **占用**：裸名 HiHaku 是 2021-11-21 建的个人号，1 个仓库，之后没动；登录名含 hihaku 的 2 个；topics 0。hihaku.com 2016-07-13 经 Onamae.com 注册，whois 记录 2026-06-27 更新过，正文读不出。

<a id="33-rangai欄外"></a>
### 3.3 rangai（欄外）

- **词义**：大辞泉「書籍・新聞・印刷物などの、印刷部分の外。また、罫で囲まれた部分の外」，例「欄外の余白」「欄外に注記する」；日国 ① 栏杆外（《米欧回覧実記》1877），② 印刷物框外（中江兆民《一年有半》1901）（[kotobank 欄外](https://kotobank.jp/word/%E6%AC%84%E5%A4%96-656645)）。Jisho：margin，不标常用词。大辞泉定义的是一个位置，「余白」是例句里的另一个词。
- **和产品**：「欄外に注記する」是在框外写一句，正文不动，对得上「写下哪句话」（[overview §2](../overview.md#2-产品是什么)）；但你第二条要的是「留着空」，rangai 对上的是「原文不动」，有张力（推断）。
- **误读**：
  - 拼音切成 rang-ai，读「让爱」「让 AI」，中等：App Store 中国区搜 rangai 唯一返回「让爱成书」，说明搜索接受这种切法，不说明它更常见。「让 AI 做某事」是 2026 年中文技术圈的常用句式；本产品说机器写的排在你的话下面，名字读成「让 AI」正反着（推断）。
  - 英文看成 Rang + AI，轻到中等：GitHub 有 safdarmustafa/Rang-ai（「Rang AI」，AI 修图安卓应用，2026-10-15 推送）、Timures/rang-ai-test（「test task for Rang.ai company」）；rang.ai 是在运营的俄语 SaaS「Ранг」。
  - 印地语、乌尔都语 रंगाई rangāī「染色、上色的工钱」（[Rekhta Dictionary](https://www.rekhtadictionary.com/meaning-of-rangaaii?lang=hi)），意思是涂满颜色，和留白相反，轻。
  - 葡萄牙语 rangar（吱嘎作响）的命令式，弱；Urban Dictionary 唯一词条把 Rangai 当女孩名；GitHub 登录名含 rangai 的 73 个，多是南印度人名 Rangaiah 一类。
- **占用**：裸名 rangai 是 2012-12-13 建的个人号，12 个仓库，末次推送 2022-05-28，90 天公开事件 0，但资料 2024-11-16 改过，不是弃号；`rangaihq` 404。rangai.com 挂牌出售。

<a id="34-mabara-和-dokokara"></a>
### 3.4 mabara 和 Dokokara

- **mabara（疎ら）**：大辞泉 ① 东西少、中间空着，例「人通りも―な住宅街」；② 零零散散（[kotobank 疎ら](https://kotobank.jp/word/%E7%96%8E%E3%82%89-635560)）。Jisho 标常用词。名字就是稀疏的点阵，好念，注册表和 App Store 三区全空，读不回中文词。但它是日常描写词，不是审美或版面词，意思是稀不是留，日常偏冷清（「まばらな拍手」）。怀疑者没复查，33 分照终评。
- **Dokokara**：事实见 [09 §12.4](09-fog-and-name.md#124-dokokara-复查和实际问题)，09 的比较见 [09 §12.5](09-fog-and-name.md#125-按第三版标准逐条对比) 和 [09 §12.9](09-fog-and-name.md#129-终评推荐和决定胜负的那一条)。按这一轮的尺子重打：1/4/4/4/1/4/3/3 = 24。长处（概念、首屏、梗）在条 2，短处正落在你这次新要的第一、三条上。这不改 09 对它和 Yohaku 的比较，只说明它不是「Yohaku 这种的」。

<a id="4-怀疑者复核"></a>
## 4. 怀疑者复核

**结论：削弱，没有推翻。** 怀疑者只复核前三名（hihaku、rangai、usuyo）和推荐；mabara 和其余 21 个没复查。下面每条后面写本篇怎么改的。

1. **命名空间事实全部复现**：三个名字的 npm、PyPI、crates、Homebrew formula 和 cask 都是 404；GitHub `-app`、`hq`、`app`、`-dev`、`-hq` 都是 404；App Store 中日美三区 iPhone 和 Mac 都没有整词同名；GitHub topics 都是 0。汉典 栏外、薄样 404，飞白 200，G2 成立。→ 照用。
2. **rangai 漏了「让爱」「让 AI」**（中等，推断），证据是 App Store 中国区。→ 写进 [§3.3](#33-rangai欄外) 和 [§0](#0-结论) 的表。
3. **rangai 漏了「Rang AI」**：GitHub 三个 Rang-ai 仓库，rang.ai 在运营，rangai.com 挂牌出售。→ 同上。
4. **rangai 漏了印地语 रंगाई「染色」**（轻）。→ 同上。
5. **rangai 按终评自己的尺子给高了**：条 1 从 4 到 3（大辞泉定义的是位置，留白来自例句；日国第一义是栏杆外）；条 6 从 4 到 3（中英两边念法都不定）；条 8 从 4 到 3（一处中等加几处轻度）；条 5 可能从 4 到 3。合计 29–30，不是 33。「让步最少」不再成立；「没有一条低于 3」仍成立。→ 推荐从 rangai 换成 usuyo，rangai 排第三。
6. **rangai 裸名**：按规则算休眠，但资料 2024-11-16 改过；登录名含 rangai 的 73 个。→ 写进 [§3.3](#33-rangai欄外)。
7. **hihaku 条 1 从 5 到 4**：kotobank 写的是「筆勢が飛動」「勢いよく飛翔」、宫殿题额，不安静；**条 6 从 5 到 4**：英文 hi- 可能念 high（推断）。合计约 32，G2 照旧。hihaku.com 2016 年起被注册。→ [§0](#0-结论) 不再说它「最像 Yohaku」，改说留白在词义里、但不安静。
8. **usuyo 留白义的位置写错了**：终评写「在第 2、3 义里」，大辞泉第二义是曙染，不讲白，只有第三义（日国第二义）讲白。条 1 仍是 3。→ 照改。
9. **usuyo.com 是在运营的「Usuyo」软件推荐站**：条 3 从 5 到 4，约 32。日本读者可能读成 うすよ（弱，推断）。→ 写进让步和弱点。
10. **排序**（怀疑者的判断）：按这些改动，你不要汉字中文名时，usuyo 排在 rangai 前面。→ 本篇采用，推荐 usuyo；问你的那一问不变，只改「不能」那一支。
11. **没核实成的**：Product Hunt 对脚本 403，网页搜索没有；少数派站内搜索对已知词也返回 0，不可用；hihaku.com 内容；GB/T 16159 隔音符号条是凭记忆引的；所有读法都是推断。V2EX（sov2ex）三个拉丁名都是 0 条。→ 进 [§5](#5-待核实清单)。

<a id="5-待核实清单"></a>
## 5. 待核实清单

1. **真人试读**：中英文用户念 usuyo、hihaku、rangai 的实际读法；hihaku 会不会被叫「飞白」、被说和 Innei 的余白有关；日本读者会不会把 hihaku 看成 hibaku、把 usuyo 读成 うすよ；rangai 会不会被读成「让 AI」。
2. **正式商标检索**：三个名字都没查 J-PlatPat、USPTO、EUIPO、CNIPA 第 9、42 类。
3. **国内平台**：小红书、B 站、抖音、国内安卓商店搜 usuyo、hihaku、rangai、薄样、飞白、栏外 会出什么，没查。
4. **Product Hunt 和少数派**：一个对脚本 403，一个站内搜索不可用，都没核实。
5. **usuyo.com**：搜「usuyo app」第一页会不会先出这个立陶宛软件站，是推断。
6. **hihaku.com**：HTTP 200 但正文读不出，持有人和用途不明。
7. **GB/T 16159**：「rang'ai 要加隔音符，不加时切成 ran-gai」是怀疑者凭记忆引的，没抓标准原文。
8. **没细查的 28 个**（[§2](#2-候选和可用性) clean、usable 两行）：只查了占用，没查词义、误读和同名产品。
9. **没复查的**：mabara 和其余 21 个的分只有终评一人打过。
10. **tenbyo**：GitHub 裸名 2026-10-07 刚被建，用途不明；要改选它先看账号是谁的。
11. **域名信号**：三个名字的 `.app`、`.dev` 没有 NS 记录，只说明多半没注册，没查注册局。

<a id="6-11-04-复查-usuyo"></a>
## 6. 11-04 复查 usuyo

你 11-04 的原话（逐字）：

> 「usuyo感觉很合适看看是否有重名或者什么，然后就不用中文，就纯英文这个，然后我觉得也符合这个软件，你觉得呢」

查了三步：撞名、词义和误读、一位怀疑者复核前两步。事实都是 2026-11-04 查的，方法照 [约定](#约定)，另用了 RDAP、whois、Bing、DuckDuckGo 和各平台的公开接口。下面已按怀疑者改过。

**占用**

| 面 | 结果 |
|---|---|
| GitHub | `usuyo`、`usuyou`、`usuyo-app`、`usuyohq`、`getusuyo` 等 13 个登录名，`users/` 和 `orgs/` 都 404；同名仓库 0；最近的是 nonQualities/usyuo（拼法不同，待办工具，5 星，README 说名字借了 薄様） |
| 注册表和扩展 | npm（含作用域）、PyPI、crates、pkg.go.dev、Homebrew 两种、Docker Hub、VS Code、Open VSX、Obsidian 插件和主题、Raycast 全空 |
| 其他托管和商店 | GitLab、Codeberg、Hugging Face、SourceForge、Snap、Flathub、AUR、F-Droid、winget 0；App Store 15 区 iPhone 和 Mac，usuyo、usuyou 整词 0；Google Play 包名 404（只查美区）；Steam 0 |
| 社交裸名 | X @usuyo 是 2011 年的休眠号（0 帖）；TikTok @usuyo 是在运营的西语游戏主播，33.87 万粉丝，Twitch usuyo 是同一人；YouTube @usuyo 是个人频道；B站 usuyo（1,554 粉丝）、微博 usuyo（二手，搜索摘要）有人。空着：X @usuyoapp、@getusuyo、@usuyohq，YouTube @usuyoapp，Bluesky usuyo.bsky.social |
| 同名非软件 | 東洋佐々木ガラス 在售的 USUYO 玻璃杯线（薄葉 うすよう），英文官网写「named after Japanese paper」（[官网](https://www.toyo.sasaki.co.jp/e/)），美国有零售，英、瑞典、挪威、韩国也有（二手，搜索摘要）；TALAS、Hiromi Paper 的 Usuyo 雁皮纸；高田みどり 2022 年的曲「Usuyo (Futa-Aya-Asobi)」；杜鹃品种 Usuyo |
| 域名 | usuyo.com 是在运营的立陶宛语软件推荐站，2025-12-16 注册，一年期（[RDAP](https://rdap.verisign.com/com/v1/domain/usuyo.com)）；.net、.org、.io、.app、.dev 未注册 |
| 搜裸词第一页 | Bing 美区、日区和 DuckDuckGo：主播和玻璃杯，usuyo.com 不在。Google 没测，百度停在验证页 |
| 商标 | 没查成：USPTO 接口 405，TMview 被 WAF 拦，WIPO 要令牌。第三方聚合站只见近似的 USYO、YUSUUO，都不在第 9、42 类 |

**判定：crowded，不是 taken**（怀疑者）。crowded 是裸词搜索首页和多数裸名社交账号被几个在运营、有真实受众的非同类主体占着；taken 要有同类产品或拦路的商标。你想按 usable 算，改这条规则就行，数据不变。

**词义和误读**

- **薄様**：大辞泉 3 义、日国 4 义（[kotobank 薄様](https://kotobank.jp/word/%E8%96%84%E6%A7%98-439972)）。日国第一义写它也用来包东西；语誌说平安中期起用于情书和亲密的赠答。几部辞典里只有世界大百科明说叠起来下面的颜色「透けて」。
- **罗马字**：令和7年内閣告示第4号（2025-12-22）的规范写法是 usuyō 或 usuyou；去长音的 usuyo 和 Tokyo 同类，告示允许团体沿用惯用写法（[文化庁 Q&A](https://www.bunka.go.jp/seisaku/kokugo_nihongo/kokugo_shisaku/pdf/94340701_01.pdf)）。高田みどり、TALAS、東洋佐々木 都写 Usuyo。
- **负面义**：没找到。Urban Dictionary 没有词条，Wiktionary 22 个语言版没有词条。
- **弱的误读**：土耳其语 üşüyo「在发冷」，不打变音符就写成 usuyo(r)；日本读者可能先读 うすよ，想到 薄汚れる、嘘よ（推断）；《龍が如く 極3》（2026-02-12 发售）有同伴角色 サチ・ウスヨ（[game8](https://game8.jp/ryu-ga-gotoku-kiwami3/761849)）；中文念「乌苏哟」，可能被开「大乌苏」的玩笑（推断）；英文念 oo-SOO-yoh 或 YOO-soo-yoh，字形近 usury（推断）。
- **中文名**：薄样 汉典、萌典都没有，品牌又不露汉字，不会长出中文名，和你说的「纯英文」对得上。
- **合不合这个软件**（你问的「你觉得呢」）：合，但要讲两步，先讲薄纸叠起来下面透上来，再讲最里两件是白；适合放在 README 一屏以下讲开，不当口号。文案不写「fades to white」「clears」，会读成雾在退（推断）。

**怀疑者改了什么**

1. 撞名一步说「网页上没有叫 usuyo 的产品或和纸品牌」，不成立：東洋佐々木 的 USUYO 线和你同词源、讲同一句故事，2026-09 还在推（日期只来自搜索摘要）。
2. 「只有 X 和 usuyo.com 两处轻度占用」不成立：TikTok、Twitch 的主播是这个裸词在网上最大的使用者。
3. 「搜 usuyo 可能先出 usuyo.com」测了，不成立，见上表。
4. 词义一步说「没找到叫 Usuyo 的角色」，部分不成立：サチ・ウスヨ。
5. 判定从 usable 改成 crowded。仍成立的：同类软件 0，开发者命名空间全空，没有负面义。

本节更正 [§0](#0-结论) 让步里「搜「usuyo」可能先出那个立陶宛软件站」一句和 [§3.1](#31-usuyo薄様) 的占用一段：第一页是主播和玻璃杯；App Store 这次查了 15 区。[§5](#5-待核实清单) 里 usuyo.com 那一条已测完，国内平台那一条只查了 B站、微博。

**结论：可以定 usuyo。** 同类软件、GitHub、注册表、应用商店全空，没有负面义；代价是社交账号要带后缀，短期内搜裸词找不到你，「名字来自薄和纸」这句 東洋佐々木 已经用英文讲过。

**还挡着或要注意的**

1. **商标没查成**，现在只有这一项可能推翻结论：你本人在 J-PlatPat 查 USUYO、薄葉、うすよう（東洋佐々木 可能注册了第 21 类，推断；第 21 类不拦软件，推断），再在 USPTO、TMview、WIPO、CNIPA 查第 9、42 类。
2. **组织名尽快注册**：GitHub 不提供预留，404 也不保证能注册。`users/` 404 也不等于空着：这次 `tesuki` 的 `users/` 和网页都 404，`orgs/` 却是 2024-06-08 建的组织，所以下面的名字都加查了 `orgs/`。
3. **没测成的**：首批平台里的小红书、抖音、快手、知乎、视频号、贴吧要登录；Google、Product Hunt、Chrome 网上应用店、Instagram @usuyo 也没测成。
4. **两种拼法**：README 写一句「usuyo (oo-soo-yoh), from the Japanese usuyō」，不露汉字（推断）。
5. **没找真人试读**，念法都是推断。

**组织名**（`users/`、`orgs/`、网页都 404）：

| 用途 | 名字 |
|---|---|
| 官方组织 `{official-org}` | `usuyo`，Go 路径 `github.com/usuyo/usuyo` |
| 获取插件 `{fetch-org}` | `usuyo-sources` |
| client-hook `{fetch-org}-hook` | `usuyo-sources-hook` |

这一组是 [N2](../open-decisions.md#2-命名与组织) 的 A（带产品名）。N2 的建议是 B：和产品名没有字面关系，像 Keiyoushi 不带 Mihon 的名字，你还没回。选 B 时有一个候选 `kamiire`（紙入れ，Jisho 释作 wallet、billfold）加 `kamiire-hook`，也都 404；只查了一部词典，「把收下的装进纸里」是推断，起名留到下一轮。
