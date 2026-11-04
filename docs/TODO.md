<a id="todo"></a>
# TODO

内部文档：写了平台名和首批计划，公开前按 [O2](open-decisions.md#13-开源与发布) 处理。

每条末尾写：谁做 · 多大 · 挡住什么。文档会话指写这些文档的会话。多大是推断。

<a id="1-等你回答"></a>
## 1. 等你回答

一次问一条，顺序照 [discussion](discussion.md)。

- [ ] [1. 什么东西进资料库](discussion.md#1-什么东西进资料库) — 你 · 一句 · 挡订阅相关的协议字段
- [ ] [2. 地图上一直清楚的是什么](discussion.md#2-地图上一直清楚的是什么) — 你 · 一句 · 挡迷雾原型
- [ ] [3. 知识的单位](discussion.md#3-知识的单位) — 你 · 一句 · 挡资料库目录、MCP 的 `op`
- [ ] [4. 删掉以后留下什么](discussion.md#4-删掉以后留下什么) — 你 · 一句 · 挡条目的 `state`、回收站
- [ ] [5. 名字](discussion.md#5-名字) — 你 · 想一想 · 挡写进磁盘的标识、注册组织
- [x] [6. 桌面壳](discussion.md#6-桌面壳)、[7. 内核语言](discussion.md#7-内核语言)：Go 加 MyGo（11-04）
- [x] [8. 插件怎么校验](discussion.md#8-插件怎么校验)：照 10-28 做 GitHub App，无服务器中转加 Actions 跑校验（11-04）
- [ ] 把 App 的部署（无服务器中转接 webhook，官方组织的 Actions 跑契约测试、写回 check run）写进 [ADR-003 §6](decisions/ADR-003-plugins.md#6-分发与校验) 和[插件协议 §8.3](plugin-protocol.md#83-校验与来源证明) — 文档会话 · 小 · 挡发版流程
- [ ] [9. 自己的获取插件组织收什么](discussion.md#9-自己的获取插件组织收什么) — 你 · 想一想 · 挡首批插件的范围
- [ ] [10. 首批平台](discussion.md#10-首批平台) — 你 · 给三个事实 · 挡 `{fetch-org}` 的第一批仓库
- [ ] [11. 默认用不用账号](discussion.md#11-默认用不用账号) — 你 · 一句 · 挡运行器的默认顺序
- [ ] [12. 能不能读浏览器的登录状态](discussion.md#12-能不能读浏览器的登录状态) — 你 · 一句 · 挡宿主读不读 cookie 库
- [ ] [13. 占屏幕的插件能不能自己跑](discussion.md#13-占屏幕的插件能不能自己跑) — 你 · 一句 · 挡视觉层调用的排法
- [ ] [14. 资料库能不能放进同步盘](discussion.md#14-资料库能不能放进同步盘) — 你 · 一句 · 挡隐藏目录的结构
- [ ] [15. Agent 能不能直接收下](discussion.md#15-agent-能不能直接收下) — 你 · 一句 · 挡 MCP 工具表
- [ ] [16. 哪些文档公开](discussion.md#16-哪些文档公开) — 你 · 一句 · 挡声明里的链接
- [ ] [17. 兼容和开创各指什么](discussion.md#17-兼容和开创各指什么) — 你 · 三问 · 挡 PRD 这两句定稿

<a id="2-文档和接口设计"></a>
## 2. 文档和接口设计

接口分五层，见 [overview](overview.md#5-接口做到哪了)。除了界面端口的 `Origin`、daemon 在不在壳里，都不等语言。

- [ ] 第 0 层：`{name}` 占位、版本策略、错误模型（RFC 9457）、ID 和时间格式 — 文档会话 · 半天 · 挡第 1–4 层
- [ ] 第 1 层：[SPEC](SPEC.md) 改成资料库 v1，加改动提议和改动记录的格式；补快照字段、`private`、`web` 身份、`jobs` 取值 — 文档会话 · 一天 · 挡 feature 001
- [ ] 第 2 层：一份内核服务契约，派生 HTTP（OpenAPI 3.2）、MCP、CLI；含搜索语法、SSE 事件、端口 — 文档会话 · 一到两天 · 挡界面对接、MCP
- [ ] 第 3 层：plugin-kit 的 schema、拆 `pkg/contract`、SDK 的形状 — 文档会话 · 中 · 挡插件运行器
- [ ] 第 4 层：投递、导出文件形状、分发、订阅源 — 文档会话 · 中 · 挡对应功能
- 等[第 1 条](discussion.md#1-什么东西进资料库)答了再补协议：
  - [ ] 订阅：`resolve-source`（带 `owner`）、`sourceItemId`、url 与 ref 的先后、第一次拉只记不收、`origin.subscription`、`feed:` 按主机算节奏；OPML 订阅对象只在选 B 时 — 文档会话 · 中 · 挡跟着收个人来源
  - [ ] `sites[].site` 能引用设置项 — 文档会话 · 小 · 挡跟着阅读器的星标
  - [ ] fetch 输出加原文出处标记 `textOrigin` — 文档会话 · 小 · 挡读 Horizon 的插件
- [ ] 权限项加连本机 Unix socket，只在第三方写包装时要 — 文档会话 · 小 · 不挡
- [ ] `builtin/` 的身份和发版对上[插件协议 §8.4](plugin-protocol.md#84-三种来源)；`web` 免 `match`、按主机分节奏 — 文档会话 · 中 · 挡预装的通用网页获取插件
- [ ] 照建议把 [L5–L7](open-decisions.md#4-借鉴其他软件)、[R2、R3](open-decisions.md#12-声明与责任) 写进协议 — 文档会话 · 小 · 挡首批插件
- [ ] [CLAUDE.md](../CLAUDE.md)：改文档地图、原则 1、原则 2（11-04 K5 A 的措辞）、技术栈、里程碑、一句话；中英文你定 — 文档会话拟稿，你点头 · 小 · 不改则新会话照 M0 开工
- [ ] [03](research/03-agent-protocols.md#13-现状2026-07-28-修订要点) 第 80 行加注：rmcp 3.0.0 已在 2026-07-28 发正式版（[crates.io](https://crates.io/crates/rmcp/versions)，11-04 查） — 文档会话 · 一行 · 不挡
- [ ] [04 §6.4](research/04-tech-options.md#64-桌面壳mygo-与回退路径) 的版本说法过时：MyGo 0.7.4、Wails v3 正式版，出处见[调研 12](research/12-shell-and-core-language.md#25-壳的成熟度) — 文档会话 · 小 · 不挡
- [ ] [ADR-002](decisions/ADR-002-frontend-stack.md#和内核的接口) 第 37 行：接受 SSE 就把 WebSocket 换掉 — 文档会话 · 一句 · 不挡
- [ ] ADR-003 改 accepted：先定 O3、O9、G2 — 你定，文档会话改 · 小 · 不挡代码
- [ ] 定名后全仓改名：旧占位 `xiecun`（代码）、`seelu`（插件协议）和 `{name}` 一起换，清单见 [07 §8](research/07-positioning-and-naming.md#8-改名要动的地方) — 主会话 · 中 · 挡标识定稿

- [ ] 插件协议 [§8.4](plugin-protocol.md#84-三种来源)、[§8.8](plugin-protocol.md#88-获取方式组织的仓库结构)：client-hook 插件放 `{fetch-org}-hook`，添加来源时能一次加两个（11-04） — 文档会话 · 小 · 挡获取插件组织的结构
- [ ] 调研 14：学 Hatoba 的同步（Worker、D1、加密、冲突、设备撤销），写本产品的同步设计和 R2 放媒体的做法 — 文档会话 · 中 · 挡资料库 v1 的隐藏目录和改动记录格式

<a id="3-原型和量测"></a>
## 3. 原型和量测

- [ ] 迷雾画布原型：点阵雾、口子、所有变化走一个时钟；顺带试壳 — 主会话做，你看 · 大 · 挡地图界面
- [ ] `go.mod` 从 Go 1.26.0 升到 MyGo 要求的 1.27.3 以上，CLAUDE.md 的「Go 1.23+」跟着改 — 主会话 · 小 · 挡接壳
- [ ] 可选：agent-desktop 给 Mac 微信收藏页拍只读快照（[10 §6 第 1 条](research/10-desktop-and-android-agents.md#6-待核实清单)），要你同意并给终端授辅助功能 — 主会话 · 十分钟 · 让 P10 有实测
- [ ] 补 [06 §18](research/06-platform-access-and-account-risk.md#18-待核实清单) 里首批平台的事实（抖音、小红书、元宝、视频号） — 文档会话 · 中 · 挡对应插件

<a id="4-开始写代码"></a>
## 4. 开始写代码

- [ ] 定开工从哪一段开始（[open-decisions §15.1 第 2 条](open-decisions.md#151-挡住开工m0)），建议无界面的最小闭环；这是第一段代码，不是一期发布（O8 不分期） — 你 · 一句 · 挡 feature 001 的范围
- [ ] 重写 [feature 001](features/001-m0-core-loop.md) 为 `draft`：获取协议的子集、预装的通用网页获取插件、`pkg/contract` 迁 v1、验收用不属于任何平台的网页 — 文档会话 · 中 · 挡下一条
- [ ] 你过目 feature 001 的要点，没确认不写代码 — 你 · 十几分钟 · 挡写代码
- [ ] 开工：先资料库读写和身份，再索引和改动记录，再运行器和预装插件，最后 MCP — 主会话 · 大 · 无

<a id="5-发布之前"></a>
## 5. 发布之前

- [ ] [O4、O6、O7](open-decisions.md#13-开源与发布)：要不要付费签名、开不开捐赠、致谢列哪些 — 你 · 各一句 · 挡发布
- [ ] [G1](open-decisions.md#14-法律意见) 找不找法律意见 — 你 · 一句 · 挡 GPL 工具随包
- [ ] [N6](open-decisions.md#2-命名与组织) 用哪个域名 — 你 · 一句 · 挡 bundle id
- [ ] 注册组织：定名后查国内商标和同名，你本人注册两个组织，占 npm、PyPI、crates；文档会话当天只读复查 — 你 · 半小时 · 挡远端仓库
