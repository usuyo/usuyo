<a id="13-待定"></a>
## 13. 待定

需要用户在产品层面拍板的问题都在 [回答表](../answers.md)，选项、建议和影响以那里为准。本文和别的文档引用本节时写的「第 N 项」就是下面的编号，编号固定，不重排，定了的项从这里去掉、写进正文。第 33、34 项不是产品问题，是资料库（[SPEC 第 1 层](../spec/library.md#1-资料库-v1第-1-层)）要的协议改动，还没做；第 35 项起是写插件时发现的空缺。

- **1.** 两个组织的名字和权利人联系地址：[N1、N2](../BUILD.md#7-只有用户能答的和能验的)，[R4](distribution.md#85-添加来源更新与权限扩大)。
- **6.** 手机投递默认走哪条：[C2](../PRD.md#71-收下)。
- **9.** 插件详情里附不附「这一层的已知情况」：[P13](../PRD.md#72-获取插件)。
- **10.** 声明文案（[附录 A](distribution.md#附录-a-声明模板)）怎么定稿、公开前请不请人看：[R1](distribution.md#85-添加来源更新与权限扩大)。
- **13.** `usuyo-fetch` 的发版用谁的工作流（[§8.8](distribution.md#88-获取方式组织的仓库结构)）：[O9](../BUILD.md#7-只有用户能答的和能验的)。
- **14.** 主仓库公开前，`docs/research` 里按平台写取法的部分放在哪，附录 A 的风险证据链接指向哪：[O2](../BUILD.md#7-只有用户能答的和能验的)。
- **17.** 电脑端视觉层先做什么，微信收藏读不到界面时退不退到截图加 OCR（[§9.5](wrapping.md#95-视觉层流程) 例子一）：[P10](../PRD.md#72-获取插件)。
- **18.** 安卓要不要做、做到哪（[§7](isolation.md#7-权限与隔离) 的 adb 管理、[§9.5](wrapping.md#95-视觉层流程) 例子二）：[P18](../PRD.md#72-获取插件)。
- **20.** `usuyo-fetch` 收不收包装自带模型的第三方手机 Agent 的获取插件（[§4.3](transform-export.md#43-模型与密钥由宿主代发)）：[R11](distribution.md#85-添加来源更新与权限扩大)。
- **21.** 用户自己填的来源参数：`sources` 只能列插件自己知道的来源，填不进一个 feed 地址或一个账号（[§3.8](fetch.md#38-list-与跟着收)）。提议加子命令 `resolve-source <输入>`，返回 `{match, source, title, order, owner}`；先例是 RSSHub 路由的 `:param`：[调研 11 §6](../research/11-subscriptions-compat-and-wx-cli.md#6-对插件协议的影响)。
- **22.** feed 条目的身份和第一次拉：`guid`、`atom:id` 是任意字符串，套不进 plugin-kit 要求的 `contentId` 正则；别人的来源第一次拉只记下已有条目、不收，以后只收新出现的（[§3.8](fetch.md#38-list-与跟着收)）：[调研 11 §6](../research/11-subscriptions-compat-and-wx-cli.md#6-对插件协议的影响)。答了一半：宿主不查 `contentId` 的格式，只要非空、不超过 256 字节、没有控制字符，`guid` 放得进；第一次拉还没做。
- **23.** OPML 导进来的是订阅，不是条目：[§3.9](fetch.md#39-从文件导入) 的导入插件只会列出条目，要有地方建一批跟着收的订阅。只在 [回答表第 1 条](../answers.md#1-什么东西进资料库)选 B 时需要：[调研 11 §6](../research/11-subscriptions-compat-and-wx-cli.md#6-对插件协议的影响)。
- **24.** 上游 AI 产物放哪：fetch 输出加一个可选的 `derived` 字段，放 Horizon 这类上游已经算好的摘要、打分和标签，宿主照转换插件的规矩移进 `derived/`；另加一个原文来源标记（例如 `textOrigin: platform | machine`），界面据此标出原文是机器写的。`derived` 那半要不要做，随 [回答表第 1 条](../answers.md#1-什么东西进资料库)一起答：[调研 11 §6](../research/11-subscriptions-compat-and-wx-cli.md#6-对插件协议的影响)。
- **25.** 连本机 Unix socket 的权限：包装常驻进程类的工具（例如 wx-cli 的 `daemon.sock`）时，`fs.read`、`network.hosts` 都表达不了「连本机某个 socket」，要么加一个权限项，要么要求这类插件走本机 HTTP（[§2.7](manifest.md#27-权限-permissions)）。只在第三方写包装常驻进程的插件时用得到：[调研 11 §6](../research/11-subscriptions-compat-and-wx-cli.md#6-对插件协议的影响)。
- **26.** 按关键词搜、按创作者主页列：现有 `list` 只列用户自己的收藏夹和列表，没有带查询词的搜索，也没有「某个创作者的主页」这种来源；结果只列出、不进库，用户挑了才 `fetch`（用户要的，[回答表](../answers.md)）。
- **27.** 评论：`fetch` 输出没有评论这一类；要可选、默认不取，按平台或按条打开，能设上限（同上）。评论放在输出目录根下的 `comments.json`（[§3.7](fetch.md#37-fetch-输出)）；JSON 的形状、上限怎么交给插件（`fetch` 现在只有 `--want`）还没定。
- **28.** 扫码登录：要账号的平台在内置浏览器里打开登录页、用户扫码，登录态按站点存本机；和 [§6](credentials.md#6-凭据) 的凭据怎么接、过期后怎么提示重扫（同上）。
- **33.** 投递遇到不收的条目（[§5.1](delivery.md#51-本机-http-接口)）：`/v1/deliver` 返回的 `state` 加 `skipped`，这时不回 `id`；只在 `api` 令牌交来的链接是私密条目时这样回（[SPEC §1.7](../spec/library.md#17-删掉)），又私密又在回收站里的也这样回。其余在回收站里的，哪种令牌交来都拿回来（记 `restore`，能撤销），不提醒；已经到期移除的，就是普通的收下。收不收等身份算出来再判；只有短链自己的 `web:` 身份时（[§3.1](fetch.md#31-运行器与节奏)），照常建条目、回 `id`，并进平台 id 以后再查：`api` 令牌交来的，目标私密时并进那一条，已回的 id 成了别名；别的照 [SPEC §0.4](../spec/conventions.md#04-id) 收下（推断）。
- **34.** 导出和读回跟上资料库 v1（[§4.4](transform-export.md#44-导出的-run-与-readback)，照 [SPEC 第 1 层](../spec/library.md#1-资料库-v1第-1-层) 的形状）：
  - `run` 的 `items[]` 不再给 `manifest`、`content` 的路径，每项是 `{id, title, author, publishedAt, publishedAtOriginal, addedAt, platform, kind, url, tags: {user, platform}, topics: [{id, title}], takeaway, highlights: [{id, layer, snapshot, fragment, quote, caption}], text, notes, derived, files}`：`text` 是主快照文字的路径，`notes` 是 `notes.md` 去掉 front matter 的正文。
  - `topics[]`、`cards[]`、`drafts[]` 是 `{id, title, createdAt, folder, path}`，`path` 是只有正文的副本，私密条目的脚注已换掉（[SPEC §1.3](../spec/library.md#13-条目的四个维度)）；主题另有 `members`（索引给的，私密的不列），卡片另有 `topics`，成稿另有 `kind`。
  - `readback` 返回 `run` 的用户层键，宿主照 [SPEC §1.4](../spec/library.md#14-你写的那一层)、[§1.8](../spec/library.md#18-同步) 合并，照 [§1.5.1](../spec/library.md#151-op-词表) 记 op，`actor: external`、`via: readback:<插件 id>`（[§4.4](transform-export.md#44-导出的-run-与-readback)）。

- **35.** 一类输出只拿到一部分（插件作者写插件时报的，37、39 同）：`outputs`、`missing` 只说一类有没有，没有字段说「这一类拿到了一部分」；web 插件现在写进 `missingReason`。
- **37.** 短码的别名：同一条内容的短码没有地方记成身份的别名（推断的后果：用短码和用完整 id 收下的会成两条）。
- **39.** 按语言的提示：`hint` 一次只按 `USUYO_LOCALE` 写一种语言，没有按语言分开给的写法。
- **40.** 只有登录才看得到：`not-found` 的 `reason` 里没有这一种。X 上成人内容和受保护账号的帖子不登录看不到，插件现在回 `reason: private`，界面分不出「作者设了私密」和「登录了就看得到」（[platforms/x.md](../platforms/x.md#3-取单条)）。
- **41.** 哪条路取到的：一个插件在一个能力里有几条路（X 的帖子接口和嵌入用的 syndication 接口），`evidence` 没有字段写这次是哪一条答的，出了错、改了版对不上是哪条路。
- **42.** 只有某几样要账号（已定，还没做）：能力加可选字段 `accountOutputs`，值是这个能力 `outputs` 的子集，列出只有带账号才拿得到的几样；宿主只在 `--want` 和它有交集时交凭据，别的调用照 `login: none` 办；`login: optional` 而没写 `accountOutputs` 的，当作每一样都可能用到账号。视频号写 `accountOutputs: [media:video]`（[platforms](../platforms/README.md#1-怎么读这一页)）。现在宿主只看能力的 `login` 不是 `none`、站点存着登录，就交凭据。

调研 11 建议的其余几处还没编号：`owner` 的含义、同时带 `url` 和 `ref` 时先用哪个、`sites[].site` 引用设置项、`feed:` 来源按主机算节奏（`origin.subscription` 写在 [§3.8](fetch.md#38-list-与跟着收)），见[调研 11 §6.2](../research/11-subscriptions-compat-and-wx-cli.md#62-要补的地方)。
