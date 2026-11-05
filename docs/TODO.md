<a id="todo"></a>
# TODO

内部文档：写了平台名和首批计划，公开前按 [O2](open-decisions.md#13-开源与发布) 处理。

每条末尾写：谁做 · 多大 · 挡住什么。文档会话指写这些文档的会话。多大是推断。

<a id="1-等你回答"></a>
## 1. 等你回答

一次问一条，顺序照 [discussion](discussion.md)。

- [x] [1. 什么东西进资料库](discussion.md#1-什么东西进资料库)：按订阅各自设置，只收挑的或也收命中关注的（11-04）
- [x] [2. 地图上一直清楚的是什么](discussion.md#2-地图上一直清楚的是什么)：关注的是主题，命中可按关键词或相似度（11-04）；动作叫「关注」，雾的理解确认（11-05）
- [x] [3. 知识的单位](discussion.md#3-知识的单位)：主题页，加卡片和成稿的链路（11-04）
- [x] [4. 删掉以后留下什么](discussion.md#4-删掉以后留下什么)：回收站加删除记录（11-04）
- [x] [5. 名字](discussion.md#5-名字)：usuyo（11-05），两轮复查没有同类软件；组织 `usuyo`、`usuyo-fetch`、`usuyo-fetch-hook`（11-05）
- [x] [6. 桌面壳](discussion.md#6-桌面壳)、[7. 内核语言](discussion.md#7-内核语言)：Go 加 MyGo（11-04）
- [x] [8. 插件怎么校验](discussion.md#8-插件怎么校验)：照 10-28 做 GitHub App，无服务器中转加 Actions 跑校验（11-04）
- [ ] 把 App 的部署（无服务器中转接 webhook，官方组织的 Actions 跑契约测试、写回 check run）写进 [ADR-003 §6](decisions/ADR-003-plugins.md#6-分发与校验) 和[插件协议 §8.3](plugin-protocol.md#83-校验与来源证明) — 文档会话 · 小 · 挡发版流程
- [x] [9. 自己的获取插件组织收什么](discussion.md#9-自己的获取插件组织收什么)：七层都收，client-hook 单独一个组织（11-04）
- [x] [10. 首批平台](discussion.md#10-首批平台)：视频号、YouTube、抖音、小红书、B站、X（11-04）
- [x] [11. 默认用不用账号](discussion.md#11-默认用不用账号)：最后要用账号，扫码登录，先做完不用账号的（11-04）
- [x] [12. 能不能读浏览器的登录状态](discussion.md#12-能不能读浏览器的登录状态)：扫码为默认，也能用日常浏览器的登录（11-04）
- [x] [13. 占屏幕的插件能不能自己跑](discussion.md#13-占屏幕的插件能不能自己跑)：每个插件单独设置，默认按声明（11-04）
- [x] [14. 资料库能不能放进同步盘](discussion.md#14-资料库能不能放进同步盘)：同步学 Hatoba，进第一版，细节照调研 14（11-04、11-05）
- [x] [15. Agent 能不能直接收下](discussion.md#15-agent-能不能直接收下)：直接生效，可撤销（11-04）
- [x] [16. 哪些文档公开](discussion.md#16-哪些文档公开)：开发期全部私有，公开前再定（11-04）
- [x] [17. 兼容和开创各指什么](discussion.md#17-兼容和开创各指什么)：17b 分五层；17a 你另提了蒸馏链路（已记）；17c 第一版不往外给 feed（11-04、11-05）

<a id="2-文档和接口设计"></a>
## 2. 文档和接口设计

接口分五层，见 [overview](overview.md#5-接口做到哪了)。除了界面端口的 `Origin`、daemon 在不在壳里，都不等语言。

- [x] 第 0 层：版本策略、错误模型（RFC 9457）、ID 和时间格式 — 草案写好，等你过目：[SPEC §0](SPEC.md#0-约定第-0-层)（11-05）
- [x] 第 1 层：[SPEC](SPEC.md) 改成资料库 v1，加改动提议和改动记录的格式；补快照字段、`private`、`web` 身份、`jobs` 取值 — 草案写好，等你过目：[SPEC §1](SPEC.md#1-资料库-v1第-1-层)，v0 附录 18 处的去向在 [SPEC 附](SPEC.md#附v0-各节和-18-处的去向)（11-05）
- [ ] 第 2 层：一份内核服务契约，派生 HTTP（OpenAPI 3.2）、MCP、CLI；含搜索语法、SSE 事件、端口 — 文档会话 · 一到两天 · 挡界面对接、MCP
  - 第 0、1 层留给这一层的：`settings.json` 的完整清单、Agent 连接名从哪来、MCP 收下工具的 `pickedBy`、MCP 的撤销工具（能撤哪些已定，[SPEC §1.10 第 18 条](SPEC.md#110-要你看一眼)）、MCP 错误和输出 schema 怎么配、游标有效期、各处的间隔、上限和文案；重写时替换 [SPEC §2](SPEC.md#2-留着的-v0第-2-层重写前保留) 留着的 v0
- [ ] 第 3 层：plugin-kit 的 schema、拆 `pkg/contract`、SDK 的形状 — 文档会话 · 中 · 挡插件运行器
- [ ] 第 4 层：投递、导出文件形状、分发、订阅源 — 文档会话 · 中 · 挡对应功能
- 等[第 1 条](discussion.md#1-什么东西进资料库)答了再补协议：
  - [ ] 订阅：`resolve-source`（带 `owner`）、`sourceItemId`、url 与 ref 的先后、第一次拉只记不收、`origin.subscription`、`feed:` 按主机算节奏；OPML 订阅对象只在选 B 时 — 文档会话 · 中 · 挡跟着收个人来源
  - [ ] `sites[].site` 能引用设置项 — 文档会话 · 小 · 挡跟着阅读器的星标
  - [ ] fetch 输出加原文出处标记 `textOrigin` — 文档会话 · 小 · 挡读 Horizon 的插件
- [ ] 权限项加连本机 Unix socket，只在第三方写包装时要 — 文档会话 · 小 · 不挡
- [ ] `builtin/` 的身份和发版对上[插件协议 §8.4](plugin-protocol.md#84-三种来源)；`web` 免 `match`、按主机分节奏 — 文档会话 · 中 · 挡预装的通用网页获取插件
- [ ] 照建议把 [L5–L7](open-decisions.md#4-借鉴其他软件)、[R2、R3](open-decisions.md#12-声明与责任) 写进协议 — 文档会话 · 小 · 挡首批插件
- [x] [CLAUDE.md](../CLAUDE.md)：一句话、文档地图、原则 1、2、6、技术栈、第一版范围（11-04，你点头后改；名字等定名再换）
- [ ] [03](research/03-agent-protocols.md#13-现状2026-07-28-修订要点) 第 80 行加注：rmcp 3.0.0 已在 2026-07-28 发正式版（[crates.io](https://crates.io/crates/rmcp/versions)，11-04 查） — 文档会话 · 一行 · 不挡
- [ ] [04 §6.4](research/04-tech-options.md#64-桌面壳mygo-与回退路径) 的版本说法过时：MyGo 0.7.4、Wails v3 正式版，出处见[调研 12](research/12-shell-and-core-language.md#25-壳的成熟度) — 文档会话 · 小 · 不挡
- [ ] [ADR-002](decisions/ADR-002-frontend-stack.md#和内核的接口) 第 37 行：接受 SSE 就把 WebSocket 换掉 — 文档会话 · 一句 · 不挡
- [ ] ADR-003 改 accepted：先定 O3、O9、G2 — 你定，文档会话改 · 小 · 不挡代码
- [x] 全仓改名为 usuyo（11-05）：入口文档、CLAUDE.md、README、SPEC、插件协议、ADR、feature 001、product-model、致谢；调研和 open-decisions 留旧名作记录；代码见下一条
- [x] 代码改名（11-05）：`go.mod` 模块 `github.com/usuyo/usuyo`、`cmd/usuyo`、import、Makefile；`make check` 通过

- [ ] 插件协议 [§8.4](plugin-protocol.md#84-三种来源)、[§8.8](plugin-protocol.md#88-获取方式组织的仓库结构)：client-hook 插件放 `usuyo-fetch-hook`，添加来源时能一次加两个（11-04） — 文档会话 · 小 · 挡获取插件组织的结构
- [ ] [插件协议 §13](plugin-protocol.md#13-待定) 第 26–28 项：按关键词和创作者主页列、可选的评论、扫码登录（11-04） — 文档会话 · 中 · 挡首批插件
- [x] [调研 14](research/14-sync-like-hatoba.md#0-结论)：学 Hatoba 的同步（11-04）
- [x] 资料库 v1 加「成稿」对象（素材 → 卡片 → 主题页 → 成稿，每句带引用，11-04） — 草案写好，等你过目：[SPEC §1.4](SPEC.md#14-你写的那一层)（11-05）
- [x] 资料库 v1 照调研 14 改四处：记录 id 和目录名由身份算、后台任务状态搬出 manifest、改动记录按设备分段、按机器的状态搬出资料库目录 — 草案写好，等你过目：[SPEC §0.4](SPEC.md#04-id)、[§1.1](SPEC.md#11-目录树)、[§1.3](SPEC.md#13-条目的四个维度)、[§1.6](SPEC.md#16-改动记录)（11-05）
- [ ] 你过目 SPEC 第 0、1 层「要你看一眼」的 9 条还没定，都在 [§1.10](SPEC.md#110-要你看一眼)（[§0.9 第 1 条](SPEC.md#09-要你看一眼) 11-04 已定、[第 2–4 条](SPEC.md#09-要你看一眼)不再问；[§1.10 第 2–5 条](SPEC.md#110-要你看一眼)不再问，[第 1、8、9、10、12、18、19 条](SPEC.md#110-要你看一眼)已定，[第 20 条](SPEC.md#110-要你看一眼)是 11-05 新加的） — 你 · 十几分钟 · 挡 feature 001
- [x] [CLAUDE.md](../CLAUDE.md)：原则 2 照 [PRD 原则 2](PRD.md#4-原则) 改措辞（`content.md` 是宿主生成的只读副本）；原则 3「删掉 `.usuyo/index/`」改成「删掉索引目录」（[SPEC §1.10 第 10 条](SPEC.md#110-要你看一眼)）；文档地图里 SPEC 那行改成 v1 草案（11-05 你点头，改了）
- [ ] 跟着第 1 层改别的文档：[ADR-003 §7](decisions/ADR-003-plugins.md#7-导出与读回)（在[修订 2026-11-05](decisions/ADR-003-plugins.md#修订-2026-11-05) 一节补上：导出记录搬到按机器目录；读回的改动记 `actor: external`，仍不走提议、能撤销；读回的层已写进那一节；[§1](decisions/ADR-003-plugins.md#1-插件分三类) 的 `images/keyframes/`、`origin.entry` 跟着改成 `derived/keyframes/`、`origins[].entry`）；[product-model §1](product-model.md#1-一条视频从收下到放下)（照 [M1](open-decisions.md#6-地图与界面) 重写时「磁盘上发生什么」一栏换成 v1 路径，或删掉这一栏指向 [SPEC §1.2](SPEC.md#12-条目文件夹)）、[§3](product-model.md#3-词汇表)（删除记录去掉「记的一句」）、[§6](product-model.md#6-知识层)（规则 3 第 165 行「要求 Obsidian 打开时可读」改成不开 usuyo 也能读；主题页按标题起名，记一句是 `notes.md` 头部的 `takeaway`；卡片 `cards/<标题>.md`、模型或 Agent 只能起草；加成稿一行；规则 4 第 166 行：头部不写成员和修改时间，不在 Obsidian 里改，改走导出再读回）；[调研 14 §3](research/14-sync-like-hatoba.md#3-存在哪和花多少)、[§5](research/14-sync-like-hatoba.md#5-改动和冲突)（`content.md` 归机器层；改到同一处时先推上服务器的当现在这份；删除记录按 `itemId` 存；改动记录的段和提议放 `blobs`；`derived/` 先推上的留下；输掉的那份进回收站，不放 `sync.db`；[§2 第 1 条](research/14-sync-like-hatoba.md#2-本产品要同步什么)改成文件夹不按年月；第 23、145、167、181 行的「Obsidian 能打开」「Obsidian 同时在改」改成不开 usuyo 也能读、云盘上的另一台或手改）；[调研 11 §4.2](research/11-subscriptions-compat-and-wx-cli.md#42-标准) 第 526 行「条目的 `content.md`」改成「主题页、卡片、成稿、导出」 — 文档会话 · 小 · 不挡
- [ ] [插件协议 §13](plugin-protocol.md#13-待定) 第 29–34 项：资料库 v1 带来的协议改动（web 身份、`version:`、时间带偏移、评论、`usuyo.` 文件名、`skipped`、导出读回带主题 id 和卡片、成稿、划线；第 34 项 11-05 照资料库的新形状重写了） — 文档会话 · 小 · 挡插件运行器和导出
- [x] [overview §5](overview.md#5-接口做到哪了) 表上面那句（「18 处对不上」）和插件协议那行的待定项数跟着改（11-05）
- [ ] feature 001 时把 manifest、changeset、journal-entry 和 topic、card、draft、notes 的 schema 写进 `pkg/contract`，类型和 `testdata/` 照 [SPEC §1.9](SPEC.md#19-schema-和-pkgcontract) 改 — 主会话 · 中 · 挡资料库读写
- [ ] 代码里引 SPEC 节号的注释跟着改：`cmd/usuyo/main.go`、`main_test.go`、`internal/*/doc.go`、`pkg/contract/*.go` 还按 v0 的节号写（`SPEC §9`、`SPEC §3` 一类），`pkg/contract/README.md`、`adapters/README.md` 的链接文字同理；引 v0 [§8](SPEC.md#8-适配器协议-v0) 的改指[插件协议](plugin-protocol.md)，引 [§12](SPEC.md#12-m0-范围) 的去掉，其余改成 [SPEC §1](SPEC.md#1-资料库-v1第-1-层)、[§2](SPEC.md#2-留着的-v0第-2-层重写前保留) 里新的节号 — 主会话 · 小 · 随 feature 001

<a id="3-原型和量测"></a>
## 3. 原型和量测

- [ ] 迷雾画布原型：点阵雾、口子、所有变化走一个时钟；顺带试壳 — 主会话做，你看 · 大 · 挡地图界面
- [ ] `go.mod` 从 Go 1.26.0 升到 MyGo 要求的 1.27.3 以上（CLAUDE.md 已写 1.27.3+） — 主会话 · 小 · 挡接壳
- [ ] 可选：agent-desktop 给 Mac 微信收藏页拍只读快照（[10 §6 第 1 条](research/10-desktop-and-android-agents.md#6-待核实清单)），要你同意并给终端授辅助功能 — 主会话 · 十分钟 · 让 P10 有实测
- [ ] 补 [06 §18](research/06-platform-access-and-account-risk.md#18-待核实清单) 里首批平台的事实（抖音、小红书、元宝、视频号） — 文档会话 · 中 · 挡对应插件

<a id="4-开始写代码"></a>
## 4. 开始写代码

第一版的范围（11-04 定）：收下和取内容、变成文字和搜索、地图和主题页、Agent 和导出、文字同步，见 [PRD §12](PRD.md#12-开发顺序)。

- [ ] 定开工从哪一段开始（[open-decisions §15.1 第 2 条](open-decisions.md#151-挡住开工m0)），建议无界面的最小闭环；这是第一段代码，不是一期发布（O8 不分期） — 你 · 一句 · 挡 feature 001 的范围
- [ ] 重写 [feature 001](features/001-m0-core-loop.md) 为 `draft`：获取协议的子集、预装的通用网页获取插件、`pkg/contract` 迁 v1、验收用不属于任何平台的网页 — 文档会话 · 中 · 挡下一条
- [ ] 你过目 feature 001 的要点，没确认不写代码 — 你 · 十几分钟 · 挡写代码
- [ ] 开工：先资料库读写和身份，再索引和改动记录，再运行器和预装插件，最后 MCP — 主会话 · 大 · 无

<a id="5-发布之前"></a>
## 5. 发布之前

- [ ] 定哪些文档公开（[第 16 条](discussion.md#16-哪些文档公开)的选项，O2、O7、P13） — 你 · 一次回复 · 挡第一次公开
- [ ] [O4、O6、O7](open-decisions.md#13-开源与发布)：要不要付费签名、开不开捐赠、致谢列哪些 — 你 · 各一句 · 挡发布
- [ ] [G1](open-decisions.md#14-法律意见) 找不找法律意见 — 你 · 一句 · 挡 GPL 工具随包
- [ ] [N6](open-decisions.md#2-命名与组织) 用哪个域名 — 你 · 一句 · 挡 bundle id
- [ ] 注册：GitHub 组织 `usuyo`、`usuyo-fetch`、`usuyo-fetch-hook`，账号 @usuyoapp，占 npm、PyPI、crates 的 `usuyo`（11-05 都空着，404 不保证能注册，尽早）；公开前查商标第 9、42 类（J-PlatPat、USPTO、中国商标网） — 你 · 半小时 · 挡远端仓库和公开
