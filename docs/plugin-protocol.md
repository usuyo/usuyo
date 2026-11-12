<a id="思鹭插件协议-v1草案"></a>
<a id="usuyo-插件协议-v1草案"></a>
# usuyo 插件协议 v1（草案）

状态：草案 · 2026-11-04 · `protocol: 1`。11-11 有了第一版宿主和 plugin-kit，做到哪见 [§3.13](#313-宿主和-plugin-kit-现在做到的11-11)、[§10.1](#101-plugin-kit-的-http-客户端11-11)。

把自己的工具接进 usuyo，读这一份就够：清单怎么写、宿主怎么调用、输出什么 JSON、凭据和权限怎么给、怎么发版和分发。三类插件（获取、转换、导出）都在这里；投递接口给浏览器扩展、油猴脚本、手机快捷指令这类不能由宿主启动的工具用。

<a id="0-读这份文档之前"></a>
## 0. 读这份文档之前

<a id="01-读者与状态"></a>
### 0.1 读者与状态

- **读者**：插件作者（包装现成工具或自己写）、实现插件运行器的人、在 [§13](#13-待定) 拍板产品问题的人。
- **状态**：草案。2026-11-03 按 [调研 10](research/10-desktop-and-android-agents.md) 补了视觉层的两种做法（电脑端读无障碍树、安卓经 ADB，[§9.5](#95-视觉层流程)）和获取插件经宿主代发用模型（[§4.3](#43-模型与密钥由宿主代发)）。字段名、子命令、方法名、错误码按这里写；实现插件运行器时如有改动，在本文修订并同步 plugin-kit 的 schema（[§10](#10-契约测试与-plugin-kit)）。
- **版本**：清单里的 `protocol` 是整数，本文是 `1`。宿主只加载 `protocol` 在支持集合里的插件，拒载原因显示在插件列表里（做法同 Mihon 的 `extensionLib`，见 [02 §15.1](research/02-adapter-ecosystems.md#151-声明与匹配)）。同一个 `protocol` 内只加可选字段；插件不得输出它声明的版本里没有的字段，宿主遇到未知字段时忽略并记日志，`usuyo plugin check` 把未知字段当失败。SPEC v0 的适配器协议没有发布过，不做兼容。主版本怎么升、旧主版本再加载多久，照 [SPEC §0.2](SPEC.md#02-版本与兼容)。

<a id="02-和其他文档的关系"></a>
### 0.2 和其他文档的关系

- [ADR-003](decisions/ADR-003-plugins.md) 定了插件的种类、进程模型、权限与隔离、仓库拆分、分发与校验、导出与读回；本文是它的展开。2026-11-01 的修订见 [ADR-003 修订](decisions/ADR-003-plugins.md#5-仓库拆分)。
- SPEC v0 的适配器协议已删，以本文为准（[SPEC 附](SPEC.md#附v0-各节和-18-处的去向)）。资料库目录、`manifest.json`、`derived/` 文件以 [SPEC 第 1 层](SPEC.md#1-资料库-v1第-1-层)为准（[§1.1](SPEC.md#11-目录树)、[§1.2.1](SPEC.md#121-manifestjson-v1)、[§1.2.2](SPEC.md#122-raw-和-derived)），身份和 id 以 [SPEC §0.4](SPEC.md#04-id) 为准，去重仍以 [SPEC §2.3](SPEC.md#23-去重)（v0）为准。
- 11-11 起内核是 Rust（[ADR-004](decisions/ADR-004-rust-tauri.md)）。本文的类型在 `crates/usuyo-plugin`（清单、信封、宿主），插件作者用 MIT 的 `crates/usuyo-plugin-kit`，它转出这些类型，另带 HTTP 客户端（[§10.1](#101-plugin-kit-的-http-客户端11-11)）；资料库的类型在 `crates/usuyo-contract`（原 `pkg/contract`）。插件的 JSON Schema 文件还没有（[§10](#10-契约测试与-plugin-kit)）。下面和 v0 schema 的对照留着查。和 `fetch-output.schema.json` 的差异列在 [§3.7](#37-fetch-输出)，和 `adapter.schema.json` 的对照在 [§2.12](#212-旧字段对照)。
- 依据：适配器生态 [调研 02](research/02-adapter-ecosystems.md)、Agent 协议 [调研 03](research/03-agent-protocols.md)、笔记软件与插件 [调研 05](research/05-note-apps-and-plugins.md)、平台获取与账号 [调研 06](research/06-platform-access-and-account-risk.md)。
- 需要用户拍板的产品问题汇总在 [回答表](answers.md)，本文相关的见 [§13](#13-待定)。

<a id="03-名字"></a>
### 0.3 名字

产品名 usuyo，命令 `usuyo`（2026-11-05 定，[回答表](answers.md)）。本文的标识符：清单 `usuyo-plugin.yaml`、环境变量前缀 `USUYO_`、资料库隐藏目录 `.usuyo/`、GitHub topic `usuyo-plugin`、导出文件里的 `usuyo_id`、URL scheme `usuyo://`。2026-11-05 之前本文用过暂名 `seelu`（思鹭），调研和 open-decisions 里还留着它和更早的 `xiecun`（歇存），作为记录不改。

本文说的「本机配置目录」「按机器目录」都是 [SPEC §0.1](SPEC.md#01-命名) 的按机器的目录。

GitHub 组织（2026-11-05 定，用户自己注册）：官方组织 `usuyo`，获取方式组织 `usuyo-fetch`，client-hook 插件放 `usuyo-fetch-hook`。权利人联系地址写 `{contact}`，两个组织用不用同一个地址见 [§13](#13-待定) 第 1 项。

<a id="04-需要同步的文档"></a>
### 0.4 需要同步的文档

本文新增或改动了下面这些设置，[PRD §8.1](PRD.md#711-设置) 的设置清单要跟着改。格式照那张表；最后一列写和现有行的关系。

| 分组 | 设置 | 默认 | 离开本机 | 和现有行的关系 |
|---|---|---|---|---|
| 收下 | 收件文件夹里的导入文件：直接收下 / 进导入预览（[§5.2](#52-监听文件夹)） | 待定，见 [§13](#13-待定) 第 15 项 | | 新增 |
| 平台与获取方式 | 每个平台的节奏上限：只接受一条条收下、两次调用的间隔、每日上限（[§3.1](#31-运行器与节奏)） | 取已装插件建议里最严的一组；你改过的不再被覆盖 | | 新增 |
| 平台与获取方式 | 每个平台的上游代理（取代 v0 的 `requires.proxy`） | 无 | | 新增 |
| 平台与获取方式 | 登录方式：不需要 / 复用日常浏览器 / 专用资料目录 / cookie 文件（[§6](#6-凭据)） | usuyo 的登录窗口（[PRD 7.2](PRD.md#72-获取插件)） | | 替换「不需要 / 用浏览器里登录着的 / cookie 文件」 |
| 平台与获取方式 | `login: optional` 的获取方式带不带账号（[§2.4](#24-access-与-login)） | 不要账号能拿到的不带，账号只用在非它不可的那几样上 | | 新增 |
| 平台与获取方式 | 获取方式来源：组织名或 `owner/repo`；每个来源显示「由 `<owner>/<repo>` 的发版工作流构建」或「没有来源证明」（[§8.3](#83-校验与来源证明)、[§8.4](#84-三种来源)） | 空 | | 替换「获取方式来源列表；官方不内置任何来源，显示签名指纹」 |
| 平台与获取方式 | 插件那一行：「现在再试」（去掉这个插件的暂停，11-11 做了；`blocked` 暂停一小时，[§3.1](#31-运行器与节奏)）和「先别试」（到点「现在再试」为止不调它）；连续崩溃后停用再按「继续」恢复还没做、「在平台上取消收藏」开关（[§2.6](#26-账号写操作)） | 取消收藏关 | | 新增 |
| 平台与获取方式 | 外部程序的路径：chrome、yt-dlp、ffmpeg 等（[§2.13](#213-运行时解析)） | 自动查找 | | 新增 |
| 平台与获取方式 | 调用超时加长（[§3.4](#34-超时与取消)） | 按插件声明 | | 新增 |
| 平台与获取方式 | 一次获取的输出大小上限（[§3.6](#36-fetch-与输出目录)） | 8 GiB | | 新增 |
| 平台与获取方式 | GitHub 令牌（只读公开仓库，用来提高查询限额，[§8.7](#87-github-api-限额与缓存)） | 无 | | 新增 |
| 平台与获取方式 | 视觉层调用（会占屏幕、剪贴板或手机）在跟着收、批量导入里能不能自己跑（[§3.1](#31-运行器与节奏)） | 待定，见 [§13](#13-待定) 第 19 项 | | 新增 |
| 平台与获取方式 | Android 设备：用哪一台、无线调试配对（[§7](#7-权限与隔离)） | 无 | | 新增 |
| 隐私 | 本地接口的访问令牌：按投递端分别生成（名字、入口类型），可单独撤销（[§5.1](#51-本机-http-接口)） | 无 | | 替换「本地接口的访问令牌：重新生成 · 自动生成」 |

<a id="1-概念"></a>
## 1. 概念

| 类 | 做什么 | 输入 | 输出 | 控制协议 |
|---|---|---|---|---|
| 获取 `fetch` | 从平台或导出文件取回内容；导入是获取的一种，来源是文件 | 链接、列表来源、导出文件 | 输出目录里的原始文件加一份 JSON | 子命令，stdout 输出 JSON（[§3](#3-获取插件协议)） |
| 转换 `transform` | 把条目里已有的文件变成新的一层 | 条目文件的只读路径 | `derived/` 下的约定文件，含 `derived/keyframes/` | stdio 上换行分隔的 JSON-RPC 2.0（[§4](#4-转换与导出插件协议)） |
| 导出 `export` | 把条目和主题页写到笔记软件，可选读回用户改动 | 条目与主题页的只读路径 | 交给宿主写的文件，或经宿主代发的云端调用；读回时返回用户层的值 | 同转换 |

几条贯穿全文的规则：

- **宿主**指 usuyo 的内核进程（`usuyo daemon` 或直接跑的 `usuyo` 命令），负责启动插件、落实权限、传凭据、写资料库。
- **宿主是资料库唯一的写方。** 插件只写宿主分配的临时目录；宿主校验后移进 `raw/<snapshotId>/`、`derived/` 或导出目标，失败就整个丢掉。插件拿不到资料库根目录。
- **插件都是独立进程**，不进内核（[ADR-004](decisions/ADR-004-rust-tauri.md)）。宿主只认这一份协议；包装命令行、本机 HTTP 服务、MCP 服务、浏览器的差别都在插件内部消化（[§9](#9-适配现有工具)）。
- **入库内容是不可信输入。** 插件输出的文字、HTML、元数据都按数据处理，不当指令执行（[03 §5.3](research/03-agent-protocols.md#53-对歇存的意义)）。
- **能力**（capability）是获取插件能做的一件事：`fetch`（取单条）、`list`（列出一个来源里的条目）、`unfavorite`（在平台上取消收藏）。`canonicalize` 和 `health` 每个获取插件都必须有，不算能力。

<a id="2-清单"></a>
## 2. 清单

<a id="21-文件名与位置"></a>
### 2.1 文件名与位置

- 文件名 `usuyo-plugin.yaml`，放仓库根目录。ADR-003 原先说在 `adapter.yaml` 上扩展；改名的理由：「adapter」只对得上获取一类；带产品前缀的名字在被包装的上游仓库里不会和别人的 `plugin.yaml`（Helm、Krew 都用这个名字）撞；GitHub 代码搜索能直接找到。
- 仓库里的是**源清单**：除 `install` 外都要写，`version` 必须等于要发的 tag（`v0.3.1` 对应 `0.3.1`）。
- 发版时由 plugin-kit 的发版工作流或 `usuyo plugin pack` 补上 `install`（各平台资产名与 sha256），作为 release 资产 `usuyo-plugin.yaml` 上传（[§8.2](#82-发版与-release-资产)）。**客户端只读 release 资产里的清单**，不读默认分支上的文件。
- 格式 YAML 1.2；宿主转成 JSON 后按 schema 校验。字段名用 camelCase；时长写 `8s`、`10m`，大小写 `512MiB`、`1GiB`。

<a id="22-共有字段"></a>
### 2.2 共有字段

| 字段 | 类型 | 必填 | 含义 | 例子 |
|---|---|---|---|---|
| `protocol` | 整数 | 是 | 插件协议版本 | `1` |
| `id` | 字符串 | 是 | 等于发布它的 GitHub 仓库 `owner/repo`，全小写；客户端核对，不一致就拒装。`builtin/` 前缀留给随安装包预装的插件（[§8.4](#84-三种来源)） | `someone/usuyo-fetch-douyin` |
| `kind` | 枚举 | 是 | `fetch`、`transform`、`export` | `fetch` |
| `name` | 字符串 | 是 | 界面上显示的名字 | `抖音` |
| `description` | 字符串 | 否 | 一句话说明 | `列出我的抖音收藏夹，按分享页取单条` |
| `version` | 字符串 | 是 | semver，等于 tag 去掉 `v`；写进条目快照的 `pluginVersion` | `0.3.1` |
| `license` | 字符串 | 是 | 插件自身的 SPDX 许可证 | `MIT` |
| `homepage` | URL | 否 | 默认是仓库主页 | `https://github.com/someone/usuyo-fetch-douyin` |
| `maintainers` | 字符串数组 | 是 | 维护者的 GitHub 登录名，至少一个；不经本人同意不写别人（RSSHub 的规则，[02 §15.1](research/02-adapter-ecosystems.md#151-声明与匹配)） | `[someone]` |
| `upstream` | 对象数组 | 否 | 被包装或 fork 的上游，只写出处和许可证：`name`、`url`、`license`、`paid`（收费服务时为 `true`）；版本约束只写在 `requires.runtime` | 见下 |
| `requires.runtime` | 字符串数组 | 否 | 要用户自己装的运行时和外部程序，写成「名字加版本约束」；宿主安装前列出来，并检查能不能找到、版本对不对（[§2.13](#213-运行时解析)） | `["chrome>=136", "yt-dlp>=2026.10"]` |
| `settings` | 对象数组 | 否 | 插件设置，见 [§2.11](#211-插件设置-settings) | |
| `permissions` | 对象 | 是 | 权限声明，见 [§2.7](#27-权限-permissions) | |
| `install` | 对象 | 发版时生成 | 各平台 release 资产与 sha256，见 [§2.8](#28-安装与启动) | |
| `run` | 对象 | 是 | 启动命令，见 [§2.8](#28-安装与启动) | |

```yaml
upstream:
  - name: yt-dlp
    url: https://github.com/yt-dlp/yt-dlp
    license: Unlicense
```

<a id="23-获取插件的字段"></a>
### 2.3 获取插件的字段

| 字段 | 类型 | 必填 | 含义 | 例子 |
|---|---|---|---|---|
| `platforms` | 字符串数组 | 是 | 平台 id，取自 plugin-kit 里的平台列表（`douyin`、`xiaohongshu`、`bilibili`、`wechat-mp`、`wechat-channels`、`x`、`web`……）；新平台向 plugin-kit 提 PR 加名字。平台列表的每一项除了 id，还规定 `contentId` 的格式（正则）和规范链接的模板，例如 `douyin` 是十进制作品 ID，`bilibili` 用 BV 号、分 P 写成 `BV…:p2`；plugin-kit 为每个平台附一份共用的 `testdata/canonicalize.json`。同一平台的所有插件共用节奏和去重，所以 id 和 `contentId` 的写法都不能各起各的 | `[douyin]` |
| `match` | 字符串数组 | `fetch.input` 含 `url` 时必填 | 粗筛链接用的 `host/path` glob，不含 scheme；`*.` 前缀匹配子域但不含裸域；最终身份由 `canonicalize` 判定（[02 §15.1](research/02-adapter-ecosystems.md#151-声明与匹配)） | `["v.douyin.com/*", "www.douyin.com/video/*"]` |
| `access` | 枚举 | 是 | 所有能力里最高的一层，见 [§2.4](#24-access-与-login) | `browser-session` |
| `login` | 枚举 | 是 | 所有能力里最高的一档：`none` < `optional` < `required` | `required` |
| `sites` | 对象数组 | 有登录时必填 | 会用到哪些账号，见 [§2.5](#25-登录站点-sites) | |
| `visitors` | 对象数组 | 否 | 会用到哪些站点的访客身份（不是账号），见 [§2.5.1](#251-访客身份-visitors) | |
| `capabilities` | 对象 | 是 | 能力名到能力声明的映射，见下表 | |
| `pacing` | 对象 | 否 | 按平台给的节奏建议：`batch`（`false` 表示只接受用户一条条收下）、`interval`（同一平台上两次调用的最短间隔）、`daily`（每天的调用次数上限）。宿主取同平台所有已装插件里最严的作默认值，用户可以改；计哪些调用见 [§3.1](#31-运行器与节奏) | `{douyin: {batch: true, interval: 8s, daily: 300}}` |

每个能力的声明：

| 字段 | 适用 | 必填 | 含义 | 例子 |
|---|---|---|---|---|
| `access` | 全部 | 是 | 这个能力实际用到的那一层 | `public-web` |
| `login` | 全部 | 是 | `none`、`optional`、`required` | `none` |
| `sites` | 全部 | 有登录时必填 | 用到 `sites` 里的哪几个，写 `site` 值 | `[www.douyin.com]` |
| `visitors` | 全部 | 否 | 用到 `visitors` 里的哪几个，写 `site` 值；和 `login` 无关，`login: none` 也可以写（[§2.5.1](#251-访客身份-visitors)） | `[www.example.com]` |
| `timeout` | 全部 | 否 | 覆盖 [§3.4](#34-超时与取消) 的默认总时长；获取插件只用这一处，不用 `permissions.limits.timeout` | `2h` |
| `perPlatform` | 全部 | 否 | 多平台插件里某个平台走的路不同时，按平台覆盖 `access`、`login`、`sites` | `{bilibili: {access: private-api, login: optional}}` |
| `outputs` | `fetch` | 是 | 能取到哪些东西，固定词表：`text`、`html`、`images`、`media:video`、`media:audio`、`subtitle`、`meta`、`comments`（11-11 照 [§13](#13-待定) 第 31 项加了 `comments`） | `[text, images, media:video]` |
| `input` | `fetch` | 否 | 接什么：`[url]`（默认）、`[ref]`（只接 `list` 返回的 `ref`，导入插件这样写）或 `[url, ref]` | `[ref]` |
| `order` | `list` | 是 | `newest-first`（按加入时间从新到旧）或 `unordered`，决定跟着收怎么停（[§3.8](#38-list-与跟着收)） | `newest-first` |
| `sources` | `list` | 否 | 固定的来源，界面在登录前就能显示；运行时以 `sources` 子命令为准 | `[{source: "douyin:favorites", title: 我的收藏夹, action: favorite}]` |
| `files` | `list` | 导入插件必填 | 能读的导出文件名 glob（[§3.9](#39-从文件导入)） | `["twitter-*.zip"]` |
| `interaction` | 全部 | 否，默认 `background` | 这次调用用户看不看得见：`background` 只读，不弹菜单、不改剪贴板、不抢焦点、不动指针、不占手机屏幕；`foreground` 是上面任何一样会发生。宿主只拿它排程和加锁（[§3.1](#31-运行器与节奏)），不拦、不弹确认 | `foreground` |
| `provider` | 全部 | 否，默认 `none` | 这个能力要不要模型：`none`、`optional`、`required`，含义同 [§2.9](#29-转换插件的字段)；不是 `none` 时经宿主代发（[§4.3](#43-模型与密钥由宿主代发)） | `optional` |

一个按 [06 §1.3](research/06-platform-access-and-account-risk.md#13-混合路径视觉层拿链接公开页面取内容) 混合路径写的抖音插件，源清单是这样（cookie 名是示例；`install` 发版时生成，见 [§2.8](#28-安装与启动)）：

```yaml
protocol: 1
id: someone/usuyo-fetch-douyin
kind: fetch
name: 抖音
description: 列出我的抖音收藏夹，按分享页取单条
version: 0.3.1
license: MIT
maintainers: [someone]
upstream:
  - name: parse-video
    url: https://github.com/wujunwei928/parse-video
    license: MIT
platforms: [douyin]
match: ["v.douyin.com/*", "www.douyin.com/video/*", "www.iesdouyin.com/share/video/*"]
access: browser-session
login: required
sites:
  - site: www.douyin.com
    name: 抖音
    credential: browser-profile
    loginUrl: https://www.douyin.com/
    cookies: [sessionid]
capabilities:
  fetch:
    access: public-web
    login: none
    outputs: [text, images, media:video, meta]
  list:
    access: browser-session
    login: required
    sites: [www.douyin.com]
    order: newest-first
    sources:
      - {source: "douyin:favorites", title: 我的收藏夹, action: favorite}
pacing:
  douyin: {batch: true, interval: 8s, daily: 300}
permissions:
  fs:
    write: ["${OUT}"]
  network:
    mode: allowlist
    hosts: ["*.douyin.com", "*.douyinvod.com", "*.douyinpic.com", "*.iesdouyin.com"]
  env: []
  devices: []
  gpu: false
  limits: {memory: 1GiB}
requires:
  runtime: ["chrome>=136"]
run:
  command: ["${PLUGIN_DIR}/usuyo-fetch-douyin"]
  serve: true
```

<a id="24-access-与-login"></a>
### 2.4 access 与 login

`access` 取七层之一，含义照 [06 约定](research/06-platform-access-and-account-risk.md#约定) 与 [06 §1.1](research/06-platform-access-and-account-risk.md#11-七个层级)，从低到高：

| 值 | 怎么拿数据 |
|---|---|
| `official-export` | 平台官方的数据导出、用户授权的官方开放接口，或读用户已经导出的文件 |
| `public-web` | 不登录，读公开页面或分享链接，不自己复现平台的请求签名 |
| `vision-ui` | 只读屏幕上渲染出来的内容，用系统级输入驱动真实客户端 |
| `browser-session` | 驱动浏览器，带登录态读 DOM 或页面自己发出的请求 |
| `session-api` | 代码里带用户的 cookie 或 token 调平台接口，不复现签名 |
| `private-api` | 在官方客户端之外复现签名、加密参数、设备指纹，或模拟 App |
| `client-hook` | 注入或 Hook 官方客户端、解密本地数据库、装根证书做中间人 |

规则：

- **按能力分别声明，顶层取最高。** 列表走登录侧、单条走公开页这类混合路径要分开写；顶层 `access`、`login` 是所有能力（含 `perPlatform`）里最高的一档，用于列表展示。
- **按实际默认路径填，不按宣传填。** 一条路径用到多层时取最高的一层；上游工具有不同模式时，按插件实际调用的模式填。
- **读界面的插件怎么填。** 只读无障碍树、控件树或截图，动作走无障碍接口或系统输入的，填 `vision-ui`：读用户日常浏览器里渲染出来的网页、经 ADB 或手机上的程序读安卓控件树再发输入，都算；开后台虚拟屏、以 shell 身份起助手进程不改变层级。经 CDP、远程调试、扩展或页面脚本读 DOM 或请求的，填 `browser-session`。带调试端口重启官方客户端、在它的渲染进程里执行脚本（例如对抖音、B站这类 Electron 客户端用 agent-desktop 的 `launch --cdp`），或注入、Hook 目标 App 的进程、读它的私有数据，填 `client-hook`（[调研 10](research/10-desktop-and-android-agents.md)）。
- **它们是给用户挑选用的标签，不是闸门。** 宿主不据此推风险档、不拦截、不弹单独的确认；运行器只按用户排的顺序试（[§3.1](#31-运行器与节奏)）。
- **`login: none` 的能力拿不到任何平台账号凭据**：宿主不传 cookie、站点存储和浏览器资料目录（[§6](#6-凭据)）。插件自己的密钥设置照 [§2.11](#211-插件设置-settings) 另行传入，访客身份照 [§2.5.1](#251-访客身份-visitors) 另行传入，都和 `login` 无关。`optional` 表示没有凭据也能跑；用户登录过也不带，见下。
- **新版本调高 `access` 或 `login`** 时，客户端在更新说明里标出，不停用这个版本（[§8.5](#85-添加来源更新与权限扩大)）。

`optional` 表示没有凭据也能跑；不带账号能拿到的一律不带，用户登录过也不带，账号只用在非它不可的那几样上。

<a id="25-登录站点-sites"></a>
### 2.5 登录站点 sites

`sites` 写明会用到哪个站点的账号。界面据 `name` 写「会用到你的抖音账号」；视频号借腾讯元宝登录态的插件写 `yuanbao.tencent.com`，界面就写「会用到你的腾讯元宝账号」（[06 §1.3](research/06-platform-access-and-account-risk.md#13-混合路径视觉层拿链接公开页面取内容)）。

| 字段 | 类型 | 必填 | 含义 | 例子 |
|---|---|---|---|---|
| `site` | 字符串 | 是 | 账号所属站点的主机名；凭据只按它做域名匹配后传出（[§6](#6-凭据)） | `yuanbao.tencent.com` |
| `name` | 字符串 | 是 | 界面上的账号名 | `腾讯元宝` |
| `credential` | 枚举 | 是 | 插件要哪种形式：`cookies`（宿主传 cookie 与站点存储）、`browser-profile`（宿主传一个专用浏览器资料目录）、`app`（插件驱动用户已登录的客户端或手机，宿主什么都不传） | `cookies` |
| `loginUrl` | URL | `cookies`、`browser-profile` 必填 | 宿主让用户登录时打开的页面 | `https://yuanbao.tencent.com/` |
| `cookies` | 字符串数组 | 否 | 登录后才有的 cookie 名；宿主只看名字在不在、过没过期，判断登录是否完成，不读值 | `["<登录后才有的 cookie 名>"]` |
| `domains` | 字符串数组 | 否 | 登录态还分布在别的域名时显式列出，宿主一并传出。每项必须和 `site` 属于同一个可注册域名（eTLD+1），不能是公共后缀；按 cookie 的 `domain` 属性匹配这个值及其子域 | `[".tencent.com"]` |

视频号借元宝登录态的插件照 [06 §17.3](research/06-platform-access-and-account-risk.md#173-首批自己适配的插件) 移植 qiaomu-clipper 的做法，在元宝页内取凭证，登录态不出浏览器，所以记 `browser-session`。只用 cookie、在页外能不能换到视频地址，待核实。

```yaml
platforms: [wechat-channels]
access: browser-session
login: required
sites:
  - site: yuanbao.tencent.com
    name: 腾讯元宝
    credential: browser-profile
    loginUrl: https://yuanbao.tencent.com/
    cookies: ["<登录后才有的 cookie 名>"]
capabilities:
  fetch:
    access: browser-session
    login: required
    sites: [yuanbao.tencent.com]
    outputs: [text, media:video, meta]
requires:
  runtime: ["chrome>=136"]
```

一个能力最多声明一个 `credential: browser-profile` 的站点，因为一次调用只交出一个资料目录。这个目录（`${PROFILE}`）不用在 `permissions.fs` 里声明：`sites` 本身就是声明，宿主只在用到这个站点的那次调用（或那个常驻进程，[§3.12](#312-常驻模式-serve)）里放行读写。

视频号借元宝登录态的插件在 README 里多写一句「元宝可以用微信登录，这时这个元宝账号和你的微信号关联」；两边是否共用风控没查到，不写进界面。

<a id="251-访客身份-visitors"></a>
### 2.5.1 访客身份 visitors

有的站点不登录也要一份访客身份才取得稳：任何没登录的访客打开一页，站点都会给一组 cookie。它不是账号，所以不写在 `sites` 里：宿主另开一个不登录的窗口，打开 `url`，存下页面自己生成的 cookie，经 `USUYO_VISITOR` 交给列了它的能力（[§3.2](#32-调用约定)、[§6](#6-凭据)）。

| 字段 | 类型 | 必填 | 含义 | 例子 |
|---|---|---|---|---|
| `site` | 字符串 | 是 | 主机名，小写；一个清单里不重复 | `www.example.com` |
| `name` | 字符串 | 是 | 界面上写「开一次 X 首页」的 X | `例子` |
| `url` | URL | 是 | 宿主打开的页面，只能是 https | `https://www.example.com/` |
| `cookies` | 字符串数组 | 否 | 这些名字都出现了就算取到；不写时等名字连着几秒不再变 | `["<访客 cookie 名>"]` |
| `domains` | 字符串数组 | 否 | 和 [§2.5](#25-登录站点-sites) 的 `domains` 同一套规则：和 `site` 同一个可注册域名，不能是公共后缀 | `[".example.com"]` |

```yaml
access: public-web
login: none
visitors:
  - site: www.example.com
    name: 例子
    url: https://www.example.com/
capabilities:
  fetch:
    access: public-web
    login: none
    visitors: [www.example.com]
    outputs: [text, images, meta]
```

- **和同一主机的账号是两回事。** 访客身份和账号各存一份（钥匙串里分开的两项），登录窗口和访客窗口各用一份网页数据；以后登录这个主机，不会和访客身份混在一起。只列了访客身份的调用拿不到账号的 cookie，只列了 `sites` 的调用拿不到访客身份。
- **能力里的每一项都要在顶层 `visitors` 里声明**，否则清单不合格。`login: none` 的能力可以列访客身份，照样不能列 `sites`。
- 页面要访客身份而没有交来、或交来的页面仍不收时，插件回 `visitor-required`（不是 `login-required`），在 `message` 里分清是哪一种；界面据此写「要先取访客身份」或「换一份」，不会让人去找一个不存在的登录。
- 访客身份只存 cookie，不存 localStorage；`credentialsUpdate` 只写回账号站点（[§6](#6-凭据)），不改访客身份。
- 界面只说有没有、什么时候存下的、最早哪个 cookie 什么时候过期；不出现在登录站点的列表里，也不算「还没登录」。

<a id="26-账号写操作"></a>
### 2.6 账号写操作

对平台账号做写操作的能力要单独声明。v1 只定义一个：`unfavorite`，在平台上取消收藏（[回答表 D3](answers.md)）。

```yaml
capabilities:
  unfavorite:
    access: browser-session
    login: required
    sites: [www.douyin.com]
```

- 默认关。用户在插件详情里打开后才会被调用；每次批量执行前，宿主列出要取消的条目让用户确认。
- 只有声明了 `unfavorite` 的插件会收到这个调用；清单里没有的写操作一律不存在。包装别人的工具时，要关掉发帖、点赞、私信这类命令（[06 §1.1](research/06-platform-access-and-account-risk.md#11-七个层级)）。
- 以后要加别的写操作，照同样的规则：单独的能力名、默认关、执行前确认。

有插件实现 `unfavorite` 时，入口放在导入结果页和跟着收设置里，执行前列清单确认；收下后不自动在平台上取消。

<a id="27-权限-permissions"></a>
### 2.7 权限 permissions

沿用 [ADR-003 §4](decisions/ADR-003-plugins.md#4-权限与隔离) 的结构，加上设备类。路径一律写宿主展开的占位符。

| 字段 | 类型 | 默认 | 含义 | 例子 |
|---|---|---|---|---|
| `fs.read` | 占位符数组 | `[]` | 可读的路径 | `["${INPUT}"]` |
| `fs.write` | 占位符数组 | `[]` | 可写的路径 | `["${OUT}"]` |
| `network.mode` | 枚举 | `none` | `none` 或 `allowlist` | `allowlist` |
| `network.hosts` | 字符串数组 | `[]` | 允许出站的主机，可带端口，不带端口指 80 与 443；`*.` 匹配子域；单写 `"*"` 表示任意公开主机（通用网页一类插件用，权限页写「可以连任何公开主机」）；可以引用设置 `${setting.<key>}`，宿主取其中的主机和端口。回环和内网地址只在显式写了带端口的地址时放行（[§3.2](#32-调用约定)） | `["*.douyin.com", "127.0.0.1:5556"]` |
| `exec` | 字符串数组 | `requires.runtime` 里的名字 | 允许再起的程序名，管整个进程树：被包装的工具再起的程序也要算进来。`${PLUGIN_DIR}` 里的可执行文件（插件自带的工具和它们再起的 helper）总是可以执行，不用写。缺省等于 `requires.runtime` 里的名字，只有额外的程序（例如系统自带的 `osascript`）才另写；解析见 [§2.13](#213-运行时解析) | `[osascript]` |
| `env` | 字符串数组 | `[]` | 允许从用户环境透传的变量名；不能用来传凭据，名字里带 `TOKEN`、`COOKIE`、`KEY`、`SECRET`、`PASSWORD` 的，`usuyo plugin check` 判失败。11-11 代码在加载时就拒掉这个插件：名字要合 `[A-Za-z_][A-Za-z0-9_]*`；不能带上面五个词和 `AUTH`、`SOCK`（不分大小写）；不能是宿主自己设的变量：`HOME`、`PATH`、`TMPDIR`、`TMP`、`TEMP`、`SYSTEMROOT`、`WINDIR`，以及 `XDG_`、`USUYO_` 开头的 | `[LANG]` |
| `gpu` | 布尔 | `false` | 要不要 GPU | `false` |
| `devices` | 枚举数组 | `[]` | 设备类权限，见下 | `[ios-device, clipboard]` |
| `limits.memory` | 大小 | `1GiB` | 内存上限，超了宿主杀进程 | `4GiB` |
| `limits.timeout` | 时长 | 按调用 | 转换、导出插件单次 `run` 的总时长；获取插件改用能力里的 `timeout`（[§3.4](#34-超时与取消)） | `4h` |

占位符：

| 占位符 | 指什么 | 读写 |
|---|---|---|
| `${OUT}` | 这次调用的输出目录，宿主新建、空、绝对路径 | 写 |
| `${INPUT}` | 这次调用的输入：导入文件的副本（[§3.9](#39-从文件导入)）、转换与导出的条目文件 | 读 |
| `${CACHE}` | 这个插件的缓存目录，宿主随时可以清；不能放凭据 | 读写 |
| `${PROFILE}` | `browser-profile` 站点的专用浏览器资料目录；不用声明，宿主只在用到这个站点的调用里放行（[§2.5](#25-登录站点-sites)） | 读写 |
| `${TARGET}` | 导出的目标文件夹，例如整个 Obsidian vault；只给 `readback: true` 的导出插件，`readback` 用（[§4.4](#44-导出的-run-与-readback)、[§7.1](#71-常驻进程的路径)）；`readback: true` 的导出插件要写进 `fs.read`，没写的不放行，`usuyo plugin check` 查这条（推断）；更新时多了它就算扩大了权限（[§8.5](#85-添加来源更新与权限扩大)） | 读 |
| `${MODELS}` | 转换插件的模型目录，由宿主按 `models` 下载 | 读 |
| `${PLUGIN_DIR}` | 安装目录，总是只读可见，不用声明 | 读 |
| `${TMP}` | 这次调用的临时目录，总是可写，不用声明 | 读写 |
| `${HOME}` | 插件的私有主目录：一次一进程时是 `${TMP}/home`，常驻时是进程暂存目录下的 `home/`（[§7.1](#71-常驻进程的路径)）；总是可写，不用声明，调用或进程结束时删除 | 读写 |

`devices` 的取值：

| 值 | 用来做什么 |
|---|---|
| `adb` | 经宿主管的 adb 服务操作用户选定的一台 Android 设备（USB 或无线调试，[§7](#7-权限与隔离)）；adb shell 能做的都能做：读屏幕和控件树、截图、模拟点按、读剪贴板、开虚拟屏、推送并运行程序、装卸 App。权限页照这句写 |
| `ios-device` | 经 usbmux 一类通道驱动 iPhone（WebDriverAgent、go-ios） |
| `screen-capture` | 截屏、录屏，包括读 iPhone 镜像窗口 |
| `accessibility` | 用辅助功能、UI Automation 读界面和发送输入 |
| `audio-capture` | 录系统声音，例如录下视频号的口播再转写 |
| `clipboard` | 读写剪贴板，例如「分享 → 复制链接」后读链接 |

- `accessibility`、`screen-capture`、`clipboard`、`audio-capture` 指电脑本机。安卓上的读屏、截图、剪贴板都经 `adb`，不另声明这几项。
- 不为虚拟屏、合成输入另加取值：开不开后台虚拟屏是插件的做法，做成插件设置（[§2.11](#211-插件设置-settings)）；会不会动指针、弹菜单、占屏幕，由能力的 `interaction` 说（[§2.3](#23-获取插件的字段)）。
- 插件推到手机上的文件、装的 App 或输入法、改的设置，要在 README 列出，并在进程退出前还原。

各平台能强制到什么程度见 [§7](#7-权限与隔离)。

<a id="28-安装与启动"></a>
### 2.8 安装与启动

```yaml
install:
  assets:
    darwin-arm64:
      name: usuyo-fetch-douyin_0.3.1_darwin-arm64.tar.gz
      sha256: 4b227777d4dd1fc61c6f884f48641d02b4d121d3fd328cb08b5531fcacdabf8a
    linux-amd64:
      name: usuyo-fetch-douyin_0.3.1_linux-amd64.tar.gz
      sha256: ef2d127de37b942baad06145e54b0c619a1f22327b2ebbcfbec78f5564afe39d
    windows-amd64:
      name: usuyo-fetch-douyin_0.3.1_windows-amd64.zip
      sha256: e7f6c011776e8db7cd330b54174fd76f7d0216b612387a5ffcfb81e6f0919683
run:
  command: ["${PLUGIN_DIR}/usuyo-fetch-douyin"]
  serve: true
```

| 字段 | 类型 | 必填 | 含义 |
|---|---|---|---|
| `install.assets` | 对象 | 发版时生成 | 键是 `<os>-<arch>`（`darwin`、`linux`、`windows` 配 `amd64`、`arm64`），或不分平台的 `any`；值是资产名与 sha256。资产是单个可执行文件、`.tar.gz` 或 `.zip`，宿主解到 `${PLUGIN_DIR}`。资产名以这里写的 `name` 为准 |
| `run.command` | 字符串数组 | 是 | 启动命令，可以用 `${PLUGIN_DIR}`；脚本类插件写运行时，例如 `["node", "${PLUGIN_DIR}/dist/main.js"]`。第一个元素不在 `${PLUGIN_DIR}` 下时，它必须写进 `requires.runtime`，按 [§2.13](#213-运行时解析) 解析，不用另写进 `exec` |
| `run.serve` | 布尔 | 否 | 获取插件是否支持常驻模式（[§3.12](#312-常驻模式-serve)）；转换、导出插件总是常驻 |

- 宿主安装时只下载 release 资产，不调用 npm、pip 一类包管理器。脚本类插件把依赖打进资产，或做成单个可执行文件（`bun build --compile`、`deno compile`、PyInstaller 一类）；要的运行时写进 `requires.runtime`。
- **插件自带的第三方程序**（例如视觉层插件带的 agent-desktop，推到手机上的 scrcpy-server 或自己写的 `app_process` 助手）放进插件自己的资产，只打包当前平台那一份，锁定版本，不让用户另装（agent-desktop 不走 `npm i -g`）。插件仓库的 CI 先核上游发布的 sha256 和来源证明，agent-desktop 是 `gh attestation verify <archive> --repo vercel-labs/agent-desktop`，通过了才打进资产；上游的 LICENSE 一并带上，agent-desktop 的 npm 包和 release 压缩包里都没有，要从它的仓库拿。运行时不另下载，推到手机上的文件只从 `${PLUGIN_DIR}` 推；GPL 的手机端组件只让用户自装（[§9.6](#96-许可证)）。
- 安装目录按版本分开：`<本机配置目录>/plugins/<owner>/<repo>/<version>/`，保留上一个版本便于回退。
- 插件的启用状态、设置放本机配置目录，密钥放系统钥匙串（[§2.11](#211-插件设置-settings)），都不放进资料库（[ADR-003 §4](decisions/ADR-003-plugins.md#4-权限与隔离)）。

<a id="29-转换插件的字段"></a>
### 2.9 转换插件的字段

| 字段 | 类型 | 必填 | 含义 | 例子 |
|---|---|---|---|---|
| `consumes` | 数组 | 是 | 吃什么：`media:video`、`media:audio`、`audio:pcm16k`（别的插件抽出的 16 kHz 单声道 WAV）、`image`、`html`、`text`、`subtitle`、`derived:asr`、`derived:ocr`、`derived:vision`、`derived:keyframes`。`text` 指原文那一栏：主快照的 `text`，没有就是 `derived/readable.md`（[SPEC §1.2.3](SPEC.md#123-contentmd-和各栏)）。只吃特定格式时写成对象 `{kind, mime}`，宿主编排时先让抽音轨一类插件转成要的格式 | `[{kind: media:audio, mime: [audio/wav]}]` |
| `produces` | 字符串数组 | 是 | 产出哪几种东西，写种类不写路径：`play`（分开的画面和音轨合起来能播的文件）、`audio:pcm16k`（给转写用的中间音频，不进资料库）、`probe`（只回流的信息，不出文件）、`asr`、`keyframes`、`ocr`；以后 `readable`、`vision`、`summary`、`embeddings`。不重复。路径由宿主定在 `derived/runs/<runId>/` 下（[SPEC §1.2.2](SPEC.md#122-raw-和-derived)）；`thumb.jpg`、`fingerprints.json` 由宿主产出，不进 `produces` | `[asr]` |
| `provider` | 枚举 | 是 | 要不要模型 provider：`none`、`optional`、`required`（[§4.3](#43-模型与密钥由宿主代发)） | `none` |
| `models` | 对象数组 | 否 | 要宿主下载的模型，放进 `${MODELS}/<name>/`：`name`（成为目录名：小写字母、数字、`.`、`_`、`-`，不以 `.` 开头，最多 100 字节，不重名）、`url`、`mirrors`（可选，同一个 sha256 的别的地址，`url` 不行时按顺序试）、`sha256`（整个下载文件）、`size`（整个下载文件的字节数，大于 0）、`format`（`file`、`tar.bz2`、`tar.gz`、`zip`；压缩包由宿主原样解到 `${MODELS}/<name>/`，包里的顶层文件夹留着，例如 `${MODELS}/sense-voice-int8-2024-07-17/sherpa-onnx-sense-voice-zh-en-ja-ko-yue-int8-2024-07-17/model.int8.onnx`）、`license`（下载前给你看），这些都必填。可选 `setting`、`value`：只在用户把这个设置选成这个值之后才用到这个模型。什么时候下见 feature 003 [§4.3](features/003-transforms.md#43-ffmpeg-和模型怎么到这台电脑上)：第一次用到时问一句，点了才下，宿主下，插件不联网 | 见下 |

```yaml
kind: transform
consumes: [audio:pcm16k]
produces: [asr]
provider: none
models:
  - name: sense-voice-int8-2024-07-17
    url: https://github.com/k2-fsa/sherpa-onnx/releases/download/asr-models/sherpa-onnx-sense-voice-zh-en-ja-ko-yue-int8-2024-07-17.tar.bz2
    sha256: 7d1efa2138a65b0b488df37f8b89e3d91a60676e416f515b952358d83dfd347e
    size: 163002883
    format: tar.bz2
    license: FunASR Model Open Source License（可商用，署名并写明模型名）
    setting: model
    value: sense-voice
permissions:
  fs:
    read: ["${INPUT}", "${MODELS}"]
    write: ["${OUT}"]
  gpu: true
  limits: {memory: 4GiB, timeout: 1800s}
```

宿主按 `consumes`、`produces` 把转换插件串起来，例如抽音轨、转写、摘要（[ADR-003 §1](decisions/ADR-003-plugins.md#1-插件分三类)）。

<a id="210-导出插件的字段"></a>
### 2.10 导出插件的字段

| 字段 | 类型 | 必填 | 含义 | 例子 |
|---|---|---|---|---|
| `target` | 字符串 | 是 | 目标软件 | `obsidian` |
| `writes` | 枚举 | 是 | `files`（结果交给宿主写进目标文件夹）或 `api`（经宿主代发云端调用，界面标「离开本机」） | `files` |
| `readback` | 布尔 | 是 | 支不支持读回用户改动 | `true` |

<a id="211-插件设置-settings"></a>
### 2.11 插件设置 settings

包装用户自己跑的服务、选画质这类事要设置项。

| 字段 | 类型 | 必填 | 含义 |
|---|---|---|---|
| `key` | 字符串 | 是 | 小写字母和数字，几段用 `-` 连起来，不重复 |
| `label` | 字符串 | 是 | 界面文字 |
| `type` | 枚举 | 是 | `string`、`bool`、`int`、`enum`；没有小数，带小数的参数（例如转写插件的 VAD 阈值）不做成设置，只经 `run` 的 `options` 传 |
| `options` | 字符串数组 | `enum` 必填 | 可选值 |
| `default` | 任意 | 否 | 默认值 |
| `secret` | 布尔 | 否 | 是密钥时为 `true`，存系统钥匙串 |
| `inject` | 对象 | 转换、导出的密钥必填 | `{host, header, format}`：宿主在代发时把密钥填进这个请求头，插件拿不到明文（[§4.3](#43-模型与密钥由宿主代发)） |

```yaml
settings:
  - key: endpoint
    label: XHS-Downloader 服务地址
    type: string
    default: http://127.0.0.1:5556
  - key: xhs-token
    label: XHS-Downloader 访问令牌
    type: string
    secret: true
```

获取插件的非密钥设置经 `USUYO_SETTINGS` 指向的 JSON 文件传入；密钥设置经 `USUYO_SECRETS` 另传（[§3.2](#32-调用约定)），和平台账号凭据分开，和 `login` 无关。转换、导出插件的非密钥设置经 `run` 的 `options` 传，密钥只经宿主代发。

<a id="212-旧字段对照"></a>
### 2.12 旧字段对照

SPEC v0 的适配器协议（[SPEC 附](SPEC.md#附v0-各节和-18-处的去向)）与 `pkg/contract/schema/adapter.schema.json` 的字段到 v1 的去向：

| v0（`adapter.yaml`） | v1（`usuyo-plugin.yaml`） | 说明 |
|---|---|---|
| 文件名 `adapter.yaml` | `usuyo-plugin.yaml` | [§2.1](#21-文件名与位置) |
| `id`（`<namespace>/<name>`） | `id`（`<owner>/<repo>`） | 和仓库对得上，客户端核对；`builtin/` 前缀留给随安装包预装的插件（[§8.4](#84-三种来源)） |
| `platforms` | `platforms`，另加能力里的 `perPlatform` | 平台 id 取自 plugin-kit 的列表 |
| `match` | `match` | 语义写清：`host/path` glob、`*.` 不含裸域、只作粗筛 |
| `capabilities: [fetch, list, watch]` | `capabilities` 映射：`fetch`、`list`、`unfavorite` | `watch` 去掉，由宿主定时调 `list`（[02 §15.5](research/02-adapter-ecosystems.md#155-批量导入与同步)） |
| `method: cli \| local-service \| mcp \| http-api \| browser` | 去掉 | 宿主只认一种协议；`browser` 由 `access: browser-session` 和 `credential: browser-profile` 表达；其余是插件内部的事（[§9](#9-适配现有工具)） |
| `upstream: {project, license, install}` | `upstream[]: {name, url, license, paid}` 加顶层 `install` | `install` 只认 release 资产；版本约束写在 `requires.runtime` |
| `requires.cookie` | `login` 加 `sites` | 写明哪个站点、哪种凭据 |
| `requires.proxy` | 去掉 | 代理由用户按平台设，宿主经本机代理出站时一并处理（[§3.2](#32-调用约定)） |
| `cost` | `upstream[].paid` | 只在上游是收费服务时标 |
| `outputs: [text, raw-html, images]` | `capabilities.fetch.outputs` | 固定词表，`raw-html` 改叫 `html` |
| `limits`（自由文本） | `pacing` 加 `permissions.limits` | 节奏结构化；资源上限归权限 |
| 无 | `protocol`、`kind`、`name`、`description`、`version`、`license`、`homepage`、`maintainers`、`access`、`login`、`sites`、`settings`、`permissions`、`install`、`run`、`requires.runtime` | 新增 |

<a id="213-运行时解析"></a>
### 2.13 运行时解析

`requires.runtime` 和 `exec` 里写的是名字，宿主负责把名字变成这台电脑上的程序。三类插件都一样。

- **运行时表。** plugin-kit 维护一张表：名字 → 各系统上的可执行文件名和常见安装位置、取版本的命令和正则、版本怎么比较（semver、日期版本、只比主版本号）。例如 `chrome` 在 macOS 上是 `/Applications/Google Chrome.app/Contents/MacOS/Google Chrome`，版本靠 `--version` 取；`yt-dlp>=2026.10` 按日期版本比较。表外的名字只检查能不能找到，不检查版本。
- **查找顺序。** 用户在设置里给这个名字指定的路径 → 用户登录 shell 的 `PATH` → `/opt/homebrew/bin`、`/usr/local/bin`、`~/.local/bin` → 表里的常见安装位置。由 launchd 或 Finder 启动的 usuyo，`PATH` 里没有 Homebrew 的目录，所以不能只看自己进程的 `PATH`。
- **按名字调用。** 每个进程开始时，宿主建 `${TMP}/bin/`（常驻进程建在它的暂存目录下），为每个名字放一个指向解析结果的符号链接或转发脚本，并把这个目录放在 `PATH` 最前。插件和被包装的工具一律按声明的名字调用，不自己找路径。
- **管整个进程树。** `exec` 管插件和它所有子孙进程：yt-dlp 合并音视频要起 ffmpeg、解 YouTube 要起 deno，这两个也要在 `requires.runtime` 或 `exec` 里。
- **自动放行只读。** 解析出的程序、它带 shebang 时的解释器（Homebrew 装的 yt-dlp 是 Python 脚本），以及它们的安装目录，宿主自动放行只读和执行，不用写进 `fs.read`。
- **找不到或版本不对**时，宿主不调用插件，在插件那一行列出缺的东西，和 `dependency-missing` 的 `needs` 用同样的写法（[§3.3](#33-信封与错误码)）。

<a id="3-获取插件协议"></a>
## 3. 获取插件协议

<a id="31-运行器与节奏"></a>
### 3.1 运行器与节奏

宿主怎么用获取插件，插件作者要知道的部分：

- **身份**：收下一个链接时，宿主在 `match` 命中的插件上按用户排的顺序调 `canonicalize`，取第一个 `match: true` 的结果作为身份（[SPEC §0.4](SPEC.md#04-id)）。`identity.contentId` 要符合平台列表里的格式（[§2.3](#23-获取插件的字段)），不符合的按 `invalid-output` 处理（11-11 代码：平台列表还没有，只查非空、不超过 256 字节、没有控制字符）。
- **顺序**：每个平台按用户排的顺序依次调 `fetch`，失败换下一个，全部失败停在 `link` 并记下各自的原因。默认顺序见本节末尾。不论 `access` 高低都照这个顺序换。平台插件和 `platforms` 含 `web` 的插件都命中时，先把平台插件全部试完，再轮到 `web` 插件。
- **暂停单个插件**：插件返回 `rate-limited` 时，宿主在 `retryAfter` 之内（没给就 15 分钟）不再在这个平台上调它；返回 `blocked` 时，暂停一小时，用户在插件那一行点「现在再试」可以更早去掉（2026-11-11 定；原先设想的「暂停到用户点继续」不做了），这一条不因这次暂停重试。11-11 代码：`retryAfter` 最多算一小时（更长的按一小时）；`platforms` 含 `web` 的插件按链接的主机暂停，别的按平台；`jobs.db` 打开时丢掉比现在晚一小时以上的旧暂停。别的插件照常按顺序试，包括用同一个账号的插件：平台的限流常按接口分开计，换一个插件、换一个接口可能还取得到（推断），这是按 D-c 和实用优先定的技术做法。
- **完整度只升不降**：宿主给每个快照记下实际拿到的输出集合（`outputs` 词表里的值）。新快照算不算升级、用户改了「只留文字」之类的设置后要不要再取，比的是这个集合，不比 `full`、`partial` 这两个字样。失败不降级。

**节奏。** 每个平台一组上限，该平台所有获取插件共用（D-e，目的是不被限流、能一直用）：

- **计入节奏的调用**：`fetch`、`list` 的一页、`sources`、`unfavorite`。同一平台同一时间只跑其中一个；`interval` 是上一次调用结束到下一次调用开始的最短间隔。一次调用内部的请求怎么排，由插件自己控制，例如视觉层插件在 `list` 的一页里逐条复制链接的间隔。
- **`daily`** 按上面这些调用的次数计，只限批量导入、跟着收和取消收藏；用户手动收下的单条不计入 `daily`，但照样排队、照样隔 `interval`。
- **`batch: false`**：只接受用户一条条收下，不做批量导入和跟着收。
- **不计入节奏的**：`canonicalize` 不占单调用锁，不计 `interval` 和 `daily`，展开短链另有每平台每分钟上限（默认 30 次），所以正在下大视频时，新贴的链接照样几秒内拿到身份；`health` 不计（`--probe` 只在用户点「检查登录」时发一次）；`access: official-export` 且 `network.mode: none` 的能力只读本机文件，不受节奏限制，可以并行。
- **队列顺序**：用户手动收下 > 跟着收 > 批量导入。
- **整机锁和设备锁。** 平台锁之外，宿主再加三把锁，不分平台、不分插件：声明了 `interaction: foreground` 的调用，全机同一时间只跑一个；声明了 `devices: [clipboard]` 的调用，全机同一时间只跑一个；同一台安卓设备同一时间只跑一个调用。原因：「复制链接」一类流程就算全走无障碍接口，也会弹菜单、改剪贴板，两个视觉层插件同时跑会互相弄坏（[§9.5](#95-视觉层流程)）。
- **视觉层调用什么时候跑。** `interaction: foreground` 的调用和用到安卓设备的调用，能不能在跟着收、批量导入里自己跑：2026-11-04 定为每个插件单独设置，三种模式 `anytime`、`idle`（用户没在用时跑，一动键鼠就停）、`manual`（只在用户点开始时跑）。默认按 `interaction` 定：`background` 为 `anytime`，`foreground` 为 `idle`，用到安卓设备的为 `manual`；声明了 `devices: [clipboard]` 的调用，宿主在调用前存下剪贴板、结束后放回（[回答表](answers.md)；字段名在实现运行器时定稿）。插件在 `foreground` 的步骤之间发现用户在动键鼠，就发 `waiting`（`user-active`），等用户停手，等不到就回 `blocked`；分辨是不是用户的输入由 plugin-kit 的视觉层 SDK 做，插件自己合成的事件会不会被算进去，待核实。安卓设备没连、没解锁、没授权时，插件回 `dependency-missing`，跟着收跳过这一轮，不算失败；宿主不存手机的解锁密码，不替用户解锁。
- **默认值**取已装插件 `pacing` 里最严的一组，用户在设置里可以改。用户改过的值不会被新装插件的建议覆盖，设置里只提示「有插件建议更慢」。

**几种情况运行器怎么做：**

| 情况 | 宿主怎么做 |
|---|---|
| 插件回 `ok: true`、`completeness: partial` | 记下这个快照，每个成功的快照都保留。顺序里后面的插件 `outputs` 能补上 `missing` 的，宿主可以带 `--want=<missing>` 去试它，后面的插件只取缺的部分。默认自动接着试，见本节末尾。插件拿到一部分就回 `partial`，不要回错误 |
| 插件回 `ok: true`、`completeness: link` | 按 `unsupported-input` 处理，换下一个 |
| 插件自报的 `completeness` 和实际输出对不上 | 宿主按 `text`、`html`、`images`、`media[].kind` 实际出现了哪些，对照 `--want` 重算，以宿主算的为准，并记日志 |
| 退出码 `2` 且没有信封 | 记 `invalid-args`，当 bug 记下，换下一个 |
| 没有插件的 `match` 命中，或 `canonicalize` 全部回 `match: false` | 交给 `platforms` 含 `web` 的插件（如果装了）；没装就由宿主照 [SPEC §0.4](SPEC.md#04-id) 自己算出 `web:` 身份建条目，停在 `link` |
| `match` 命中的插件都没回 `match: true`，且有一个 `canonicalize` 因 `network` 或 `timeout` 失败（例如离线时收下的短链） | 不交给 `web` 插件。宿主拿这条链接本身算 `web:` 身份，马上建只有链接的条目（`completeness: link`，还没有快照；之后的快照照记 `inputUrl`），重试记进本机 `jobs.db`（[SPEC §1.1](SPEC.md#11-目录树)）；之后再调 `canonicalize`，拿到平台身份就照 [SPEC §0.4](SPEC.md#04-id) 并过去，`web-` id 留作别名；跟着收的不建，这一次不算跑完，联网后整次重跑 |
| 常驻进程崩溃 | 记 `crashed`，下一次调用时重启；连续崩溃 3 次就停用这个插件，直到用户在插件那一行点「继续」 |

默认顺序：先试不用账号的，再试用账号的；同一类里按 `access` 从低到高，相同的按安装顺序；用户拖动排过的平台照用户排的。拿到一部分时默认接着试：顺序里后面能补上 `missing` 的插件，宿主带 `--want=<missing>` 接着调，直到拿全或试完，每个成功的快照都留。

你正一条条收下时碰到 `blocked`，宿主打开 usuyo 里的窗口显示那一页，你当场完成验证后接着收这一条；跟着收和批量导入时不弹，平台那一行写「需要你完成一次验证」，点了再打开那一页。不自动识别验证码。

<a id="32-调用约定"></a>
### 3.2 调用约定

```
<command> canonicalize <url>
<command> fetch <url-or-ref> --out <dir> [--want text,images,media:video]
<command> sources
<command> list <source> [--cursor <c>] [--since <time>]
<command> health [--probe]
<command> unfavorite <url> --source <source>
<command> serve
```

`<command>` 是 `run.command` 展开后的命令。各子命令的输入输出见 [§3.5](#35-canonicalize-与身份) 到 [§3.12](#312-常驻模式-serve)。

**环境变量。** 宿主清空环境后只传下面这些，加上 `permissions.env` 声明的变量（11-11 哪些已经传了见 [§3.13](#313-宿主和-plugin-kit-现在做到的11-11)）：

| 变量 | 什么时候有 | 含义 |
|---|---|---|
| `USUYO_PROTOCOL` | 总是 | `1` |
| `USUYO_PLUGIN_DIR` | 总是 | 安装目录 |
| `USUYO_CACHE_DIR` | 总是 | `${CACHE}` |
| `USUYO_MODELS_DIR` | 转换插件的常驻进程 | `${MODELS}`，按机器目录的 `models/<owner>/<repo>/` |
| `USUYO_CALL_ID` | 一次一进程的调用；常驻进程没有，一个进程接好几次调用 | 这次调用的 id，写日志用 |
| `USUYO_LOCALE` | 总是 | 用户界面语言，例如 `zh-CN`；`hint` 按它写 |
| `USUYO_NETWORK` | 宿主查到这台机器开着假 IP 模式的代理时 | 值是 `fake-ip`。这时系统给每个域名都回 `198.18.0.0/15` 里的地址；plugin-kit 的客户端据此放行查出来全在这一段的域名，没有这个变量就照旧拒绝。是不是这种环境由宿主拿固定的公网域名去查（11-11） |
| `USUYO_SETTINGS` | 有设置时 | 非密钥设置的 JSON 文件路径（11-11 已传，[§3.13](#313-宿主和-plugin-kit-现在做到的11-11)） |
| `USUYO_SECRETS` | 插件声明了 `secret: true` 的设置、且用户填了值时，和 `login` 无关 | 一个 0600 的临时文件，内容只有 `{"<key>": "<value>"}`，调用结束即删 |
| `USUYO_INPUT` | 导入调用（[§3.9](#39-从文件导入)） | `${INPUT}`，导入文件的副本 |
| `USUYO_CREDENTIALS` | 这次调用要带账号、且用户已登录时（[§2.4](#24-access-与-login)） | 凭据 JSON 文件路径（[§6](#6-凭据)） |
| `USUYO_COOKIES_TXT` | 同上，`credential: cookies` | 同一批 cookie 的 Netscape 格式文件，给 yt-dlp 一类工具的 `--cookies` 用；可写，调用结束后丢弃 |
| `USUYO_VISITOR` | 这次的能力列了 `visitors`、且存着那份访客身份时，和 `login` 无关（[§2.5.1](#251-访客身份-visitors)） | 访客身份的 JSON 文件路径，形状和 `USUYO_CREDENTIALS` 一样，`source` 是 `visitor-window`、没有 localStorage；0600，调用结束即删 |
| `USUYO_BROWSER_PROFILE` | `credential: browser-profile` | `${PROFILE}` |
| `USUYO_PROVIDER` | 能力的 `provider` 不是 `none`、用户配了模型时（[§4.3](#43-模型与密钥由宿主代发)） | 这次调用的代发参数，JSON 文件路径，内容是 `{baseUrl, token, model}` |
| `ADB_SERVER_SOCKET` | 声明了 `devices: [adb]` | 宿主管的 adb 服务，`tcp:127.0.0.1:<端口>`（[§7](#7-权限与隔离)）；adb 认这个变量 |
| `ANDROID_SERIAL` | 同上 | 用户选定的那台设备的序列号；adb 认这个变量 |
| `HOME` | 总是 | `${HOME}`（[§2.7](#27-权限-permissions)），不指向用户主目录：被包装的工具不会读到用户自己的配置，例如 yt-dlp 配置里的 `--cookies-from-browser` |
| `XDG_CONFIG_HOME`、`XDG_CACHE_HOME` | 总是 | `${HOME}/.config`、`${CACHE}` |
| `TMPDIR`、`TMP`、`TEMP` | 总是 | `${TMP}` |
| `HTTP_PROXY`、`HTTPS_PROXY`、`ALL_PROXY` | 有网络权限时 | 指向宿主的本机代理，见下面的「网络出口」 |
| `NO_PROXY` | 有网络权限时 | `localhost,127.0.0.1,::1` |
| `USUYO_PROXY` | 有网络权限时 | 宿主本机代理的 `host:port`，给浏览器的 `--proxy-server` 用 |
| `NODE_USE_ENV_PROXY` | 有网络权限时 | `1`，让 Node 内置的 `fetch` 读代理变量（Node 哪个版本起支持，待核实） |
| `PATH` | 总是 | `${TMP}/bin`（常驻进程是暂存目录的 `bin/`，[§2.13](#213-运行时解析)）在最前，再加系统目录 |

**网络出口。**

- 非回环地址的出站只经宿主的本机代理。代理先解析域名再按 `network.hosts` 判断，连接时用解析出来的那个地址，防止 DNS 重绑定；解析结果是回环、RFC 1918 内网、链路本地、ULA 或云元数据地址（169.254.169.254 一类）的一律拒绝，除非 `hosts` 里显式写了带端口的这个地址。
- `hosts` 里显式写的回环地址（例如 `127.0.0.1:5556`）和宿主自己的本机端点（[§4.3](#43-模型与密钥由宿主代发)），由沙箱直接放行到声明的端口，不经代理。原因：Go 的 `http.ProxyFromEnvironment` 对 `localhost` 和回环地址一律不走代理，改 `NO_PROXY` 也没用（已用 Go 1.26 的 `go doc` 和 `httpproxy.useProxy` 源码核对）。
- 浏览器不读代理环境变量（推断）：插件启动 Chrome 时带 `--proxy-server=$USUYO_PROXY`。Node 插件靠 `NODE_USE_ENV_PROXY` 或 SDK 的代理设置。plugin-kit 的 SDK 默认把这两件事做好。
- 用户给这个平台设了上游代理时，由宿主的代理转发。
- 代理记下这次调用被拒的主机。调用失败时，界面随错误一起显示「被拦下的主机：…」；`usuyo plugin try` 打印同样的列表，作者照着补 `hosts`。浏览器类插件打开的页面会连很多 CDN 和统计域名，靠这个列表补全，或者写 `"*"`。

**标准流。**

- stdin：一次一进程时宿主立即关闭，插件不要等输入。要用户操作的事（扫码、短信验证）由宿主的登录流程处理，不经过插件。
- stdout：只输出一个 JSON 值（[§3.3](#33-信封与错误码)），UTF-8，最多 32 MiB；不要往 stdout 打日志。
- stderr：日志，任意文本，宿主保留最后 200 行作诊断，不当作失败信号。以 `@usuyo ` 开头的行是结构化事件，后面跟一个单行 JSON 对象：

```
@usuyo {"event":"progress","done":3,"total":20,"unit":"MB","message":"下载视频"}
@usuyo {"event":"waiting","reason":"user-action","message":"请在弹出的浏览器窗口里完成验证"}
```

| 事件 | 字段 | 用途 |
|---|---|---|
| `progress` | `done`、`total`、`unit`、`message` | 进度条 |
| `waiting` | `reason`（`user-action`、`device-locked`、`device-disconnected`、`device-unauthorized`（手机上还没点「允许 USB 调试」）、`user-active`（用户正在用键鼠，[§3.1](#31-运行器与节奏)））、`message` | 插件在等用户在可见的窗口或手机上操作；等不到就在超时前返回 `blocked` 或 `dependency-missing` |

**退出码。** `0`：stdout 是 `ok: true` 的信封；`1`：stdout 是 `ok: false` 的信封；`2`：用法错误（不认识的子命令或参数），可以没有信封，宿主记为 `invalid-args`、当 bug 记下，然后换下一个（宿主只调用声明过的能力，所以这里多半是包装脚本自己出错，例如 `set -e` 下 curl 失败）。成败以信封为准，退出码只是兜底。

<a id="33-信封与错误码"></a>
### 3.3 信封与错误码

成功时信封顶层是 `ok: true` 加该子命令的字段；失败时：

```json
{
  "ok": false,
  "error": {
    "code": "login-required",
    "message": "favorites request returned 401",
    "hint": "抖音的登录已过期，请在设置 › 平台与获取方式里重新登录",
    "site": "www.douyin.com"
  }
}
```

| 字段 | 必填 | 含义 |
|---|---|---|
| `code` | 是 | 下表之一 |
| `message` | 是 | 给日志看的原因，不限语言 |
| `hint` | 否 | 给用户看的一句话，按 `USUYO_LOCALE` 写 |
| `retryAfter` | 否 | `rate-limited` 时多少秒后再试 |
| `site` | 否 | 和哪个登录站点有关 |
| `reason` | 否 | `not-found` 的细分：`deleted`、`private`、`paid`、`region`、`link-expired` |
| `needs` | `dependency-missing` 必填 | 缺的东西一次列全，例如 `["runtime:yt-dlp", "setting:endpoint", "device:ios-device"]`。不叫 `missing`，免得和 fetch 输出里「缺的输出」混在一起 |

`needs` 的每一项是「前缀:值」：

| 前缀 | 值 | 例子 |
|---|---|---|
| `runtime:` | `requires.runtime` 或 `exec` 里的名字 | `runtime:ffmpeg` |
| `setting:` | 设置的 `key` | `setting:xhs-token` |
| `device:` | `devices` 的值，指设备没连上 | `device:ios-device` |
| `permission:` | 系统没给 usuyo 的授权：`accessibility`、`screen-capture`、`audio-capture` | `permission:accessibility` |
| `service:` | 设置里指向的本机服务连不上，值是那个设置的 `key` | `service:endpoint` |
| `app:` | 要开着的客户端 | `app:wechat` |
| `state:` | 要用户处理的状态：`unlocked`（屏幕或手机没解锁）、`logged-in`（客户端里没登录）、`debugging`（手机没开开发者选项或 USB 调试）、`authorized`（手机还没允许这台电脑调试） | `state:unlocked` |
| `file:` | 导入文件的副本已经不在 | `file:twitter-2026-10-30.zip` |
| `model:` | 转换插件要的模型（清单 `models[].name`）不在 `${MODELS}` 里 | `model:sense-voice-int8-2024-07-17` |

11-11 照 [§13](#13-待定) 第 30 项加了前缀 `version:`，例 `version:usuyo>=1.3`。

插件返回的错误码：

| `code` | 含义 | 运行器怎么办 |
|---|---|---|
| `unsupported-input` | 不是这个插件能处理的链接、文件或来源（例如直播） | 换下一个 |
| `not-found` | 内容不存在、已删除或当前身份看不到；能分清时填 `reason` | 换下一个。只有全部是 `not-found` 且 `reason` 都是 `deleted` 时，条目才记「平台上已看不到」；有 `link-expired` 时提示用户重新分享或重新导出链接；其余记「取不到」并列出原因 |
| `login-required` | 没登录，或登录已过期 | 换下一个；插件那一行提示重新登录 `site` |
| `visitor-required` | 站点要一份访客身份（[§2.5.1](#251-访客身份-visitors)）：没有交来，或交来的页面仍不收 | 换下一个；界面写成要取或要换访客身份，不说登录；和 `login-required` 一样不自己暂停 |
| `rate-limited` | 被平台限流 | 换下一个；这个插件在这个平台暂停到 `retryAfter` |
| `blocked` | 平台出现验证页、风控提示或拦截 | 换下一个；这个插件在这个平台暂停一小时，用户点「现在再试」可更早解开（2026-11-11 定，[§3.1](#31-运行器与节奏)） |
| `upstream-changed` | 页面或接口改版，解析不出 | 换下一个；记进插件的最近失败 |
| `network` | 网络不通、DNS、TLS 或插件自己的请求超时 | 换下一个 |
| `dependency-missing` | 缺运行时、外部程序、设备、设置、系统授权、本机服务，或要用户处理的状态 | 换下一个；界面列出 `needs` |
| `invalid-args` | 参数不对，多半是宿主或插件的 bug | 换下一个；记 bug |
| `cancelled` | 收到取消 | 不再换 |
| `internal` | 插件内部错误 | 换下一个 |

宿主自己记的四种：`timeout`（超时被杀）、`crashed`（退出了但没有可解析的信封）、`invalid-output`（信封不符合 schema、输出目录不合规，[§3.6](#36-fetch-与输出目录)）、`profile-busy`（等不到浏览器资料目录，[§6](#6-凭据)）。四种都换下一个。

这套码的来源：[02 §15.3](research/02-adapter-ecosystems.md#153-调用错误与产物) 的建议；wechat-inbox-sync 的失败分类（需要登录、风控、内容不可用、不支持、网络）对应 `login-required`、`rate-limited` 与 `blocked`、`not-found`、`unsupported-input`、`network`；ChubbySkills 的缺依赖、cookie 问题、风控限流、来源失效对应 `dependency-missing`、`login-required`、`rate-limited`、`upstream-changed`；Agent-Reach 把 `NEED_LOGIN`、`RATE_LIMITED`、`PLATFORM_BLOCKED` 当终止码，这里只暂停出错的那个插件，不停止换下一个（[06 §3](research/06-platform-access-and-account-risk.md#3-用户点名的项目)）。`dependency-missing` 一次列全缺的东西，照 MoneyPrinterTurbo 退出码 10 的做法，让用户一次补齐。

界面把失败固定成 7 类，每类一句话加一个动作：要重新登录（去登录）、平台暂时限制（几点恢复）、要你完成验证（打开）、平台上已看不到（留着链接）、暂不支持这个链接（换获取插件或存下网页）、网络问题（重试）、获取插件要更新（检查更新）。插件原话放详情里；每类都给「存下这个网页」兜底；没有口播的视频写「这段没有口播」。

<a id="34-超时与取消"></a>
### 3.4 超时与取消

| 调用 | 默认总时长 |
|---|---|
| `canonicalize` | 10 s |
| `fetch` | 300 s |
| `sources` | 30 s |
| `list`（每页） | 60 s |
| `health` | 10 s；`--probe` 30 s |
| `unfavorite` | 30 s |
| 转换的 `run` | 3600 s；清单写了 `permissions.limits.timeout` 时换成它 |

- 两种计时同时生效。**总时长**：默认照上表，获取插件用能力里的 `timeout` 改，转换、导出插件用 `permissions.limits.timeout` 改；用户可以在设置里再加长，不设硬上限，下一个大视频、在 CPU 上转写几小时的播客都放得下（11-11 代码：能力里的 `timeout` 超过 2 小时按 2 小时算，设置还没有）。**无进度时长**：插件发过 `progress` 之后，120 s 内没有新的 `progress` 就算超时，用来发现卡住的调用；从没发过 `progress` 的调用只受总时长约束。
- 插件发出 `waiting` 后，两种计时都暂停，直到下一个 `progress` 事件，最长等 10 分钟（11-11 代码：一次调用里所有 `waiting` 的时间加起来算这 10 分钟）；等用户在弹出的窗口里过验证时不会被杀。
- 取消与超时：宿主先发 SIGTERM（Windows 上向进程组发 `CTRL_BREAK_EVENT`），插件应在 5 秒内收尾退出，能输出 `cancelled` 信封就输出；5 秒后宿主杀掉整个进程组（Windows 上终止 Job Object），插件启动的浏览器一并结束。
- 被取消或超时的调用，输出目录整个丢掉。

<a id="35-canonicalize-与身份"></a>
### 3.5 canonicalize 与身份

```
<command> canonicalize "https://v.douyin.com/iRNBho6u/"
```

```json
{"ok": true, "match": true, "identity": {"platform": "douyin", "contentId": "7431234567890123456", "url": "https://www.douyin.com/video/7431234567890123456"}, "resolvedUrl": "https://www.iesdouyin.com/share/video/7431234567890123456/?<分享参数>"}
```

```json
{"ok": true, "match": false}
```

- 展开短链、去掉跟踪参数、取出平台内容 ID（[SPEC §0.4](SPEC.md#04-id)）。不是自己的链接回 `match: false`，退出码 0，这不是错误。
- 只为展开短链才发请求，只跟随重定向，不带凭据；这个请求不计入平台节奏，另受每平台每分钟的上限（[§3.1](#31-运行器与节奏)）。
- `identity` 的形状和 `fetch` 输出里的一样。`identity.contentId` 照 plugin-kit 平台列表里的格式写（[§2.3](#23-获取插件的字段)）。`identity.url` 写规范化后的链接，去掉跟踪参数和会话 token 一类参数，只用于去重和显示。`platform: web` 的身份不带 `contentId`，必带 `url`，宿主再照 [SPEC §0.4](SPEC.md#04-id) 规范化一遍（11-11 照 [§13](#13-待定) 第 29 项做了）。
- `resolvedUrl` 可选：展开短链后的完整链接，保留全部参数。宿主把用户给的原链接另存（快照里记 `inputUrl`，[SPEC §1.2.1](SPEC.md#121-manifestjson-v1)），有 `resolvedUrl` 时一并记下，交给 `fetch` 的是这两者之一，不是 `identity.url`（[§3.6](#36-fetch-与输出目录)）。小红书的链接要带 `xsec_token` 才取得到（[06 §1.3](research/06-platform-access-and-account-risk.md#13-混合路径视觉层拿链接公开页面取内容)），去掉参数就取不到了。

<a id="36-fetch-与输出目录"></a>
### 3.6 fetch 与输出目录

```
<command> fetch "https://v.douyin.com/iRNBho6u/" --out /private/var/folders/xx/usuyo-call-01/out --want text,images,media:video
```

- 第一个参数是收下时拿到的原链接（`canonicalize` 给了 `resolvedUrl` 时用它，同样不去参数），或 `list` 返回的 `ref`（[§3.9](#39-从文件导入)）；接哪种由 `fetch.input` 声明。`identity.url` 只用于去重和显示，不交给 `fetch`。
- `--want` 是宿主想要的东西，取 `outputs` 的子集；不给时按 `outputs` 全取。例如用户开了「收下后直接只留文字」，宿主就不要 `media:video`，插件不必下载视频。
- `--out` 是宿主新建的空目录，绝对路径。插件只在里面写文件，不依赖当前目录；输出 JSON 里的路径都相对它。
- 成功后宿主把整个目录移成 `raw/<snapshotId>/`，没被 JSON 引用的文件（例如平台原始返回）也原样留下；失败就整个丢掉。
- **输出目录的规则**（[§4.2](#42-转换的-run)、[§4.4](#44-导出的-run-与-readback)、[§5.1](#51-本机-http-接口)、[§5.2](#52-监听文件夹) 都照这条）：宿主在移动前遍历整个目录，不只看 JSON 引用的条目。只接受普通文件和目录；遇到符号链接、`nlink > 1` 的硬链接、FIFO、套接字、设备文件，或名字含 `..`、以 `/` 开头、含控制字符的条目，整次判 `invalid-output` 并丢弃。名字另照 [§13](#13-待定) 第 32 项查（11-11 做了）：不能以 `usuyo.` 开头（不分大小写），不能有 `< > : " / \ | ? *`，不以 `.` 开头、不以空格或 `.` 结尾，要是 NFC，不能是 Windows 保留名（带扩展名也算），大小写折叠后不重名，每段不超过 200 字节，相对路径不超过 99 个 UTF-16 单位（条目文件夹以内的 140 减去 `raw/<snapshotId>/`）。整个目录最多 10,000 项（文件加目录）。JSON 里同一个文件只能引用一次，引用的文件要在。JSON 里的路径同样不能有 `..` 或绝对路径。整个目录有大小上限，`fetch` 默认 8 GiB，转换、导出和投递默认 1 GiB，用户可以在设置里改，超了也判 `invalid-output`。原因：插件在沙箱里读不了 `~/.ssh`，但能在 `out` 里建一个指向它的符号链接；目录移进资料库后，索引、导出和 MCP 会以宿主的身份读到目标内容。
- 建议的目录布局：

```
<out>/
  page.html
  meta.json
  images/001.jpg
  media/video.mp4
  media/subtitle.zh.vtt
```

<a id="37-fetch-输出"></a>
### 3.7 fetch 输出

```json
{
  "ok": true,
  "identity": {"platform": "douyin", "contentId": "7431234567890123456", "url": "https://www.douyin.com/video/7431234567890123456"},
  "kind": "video",
  "title": "手冲咖啡：三段式注水",
  "author": {"name": "咖啡日常", "platformId": "MS4wLjABAAAAsample", "url": "https://www.douyin.com/user/MS4wLjABAAAAsample"},
  "publishedAt": "2026-10-01T12:00:00Z",
  "text": "三段式注水：第一段 40 克闷蒸 30 秒……",
  "textFormat": "plain",
  "html": "page.html",
  "images": ["images/001.jpg"],
  "media": [
    {"path": "media/video.mp4", "kind": "video", "mime": "video/mp4", "durationSec": 183},
    {"path": "media/subtitle.zh.vtt", "kind": "subtitle", "lang": "zh"}
  ],
  "platformTags": ["手冲", "咖啡日常"],
  "completeness": "full",
  "missing": [],
  "evidence": {"fetchedAt": "2026-11-01T03:00:01Z", "access": "public-web", "status": 200}
}
```

和 `pkg/contract/schema/fetch-output.schema.json`（v0）的差异：

| 字段 | v0 | v1 |
|---|---|---|
| `ok` | 无 | 必填，成功为 `true` |
| `identity` | 必填，`platform` 必填 | 没有平台内容 ID 的条目（聊天记录、转发的图片一类）可以省略，或写 `{"platform": "file"}` 不带 `contentId`，省略等于 `{"platform": "file"}`；这样的条目不参与身份层，去重只走内容层（[SPEC §0.4](SPEC.md#04-id)） |
| `textFormat` | 无 | 可选，`plain`（默认）或 `markdown` |
| `media` | 路径字符串数组 | 对象数组：`path`、`kind`（`video`、`audio`、`subtitle`、`other`）必填，`mime`、`lang`、`durationSec`、`group` 可选。转换插件按 `kind` 编排，靠扩展名猜不可靠。`group`：同一个字符串表示同一段时间线拆开的几个文件（例如分开存的画面和音轨），1–64 个字母、数字或 `. _ : -`，不合的整次 `invalid-output`；没有 `group` 时宿主照 feature 003 [§4.1](features/003-transforms.md#41-四个转换插件) 按内容配（[§13](#13-待定) 第 38 项） |
| `missing` 的取值 | 自由字符串 | 用 `outputs` 的词表，例如 `media:video` |
| `missingReason` | 无（[SPEC §1.2.1](SPEC.md#121-manifestjson-v1) 的 manifest 有） | 可选，给用户看的原因，例如「分享页没有给出视频地址」 |
| `evidence.access` | 无 | 必填，这次实际用到的那一层。快照里另有一个值 `deliver`，由宿主给投递来的内容填（[§5.1](#51-本机-http-接口)），不出现在清单的 `access` 里 |
| `credentialsUpdate` | 无 | 可选，调用期间平台换了会话 cookie 时交回新值（[§6](#6-凭据)） |
| `evidence.site` | 无 | 用了登录时填站点 |
| `evidence.upstream` | 无 | 可选，`{name, version}`，被包装工具的实际版本 |
| `evidence.status` | 可选 | 只在 HTTP 类请求时填 |
| 其余字段 | | 不变：`kind`、`title`、`author`、`publishedAt`、`text`、`html`、`images`、`platformTags`、`completeness`、`missing`、`evidence.fetchedAt`；时间一律 RFC 3339 带偏移，宿主把 `publishedAt` 换成 UTC 存（11-11 照 [§13](#13-待定) 第 30 项做了） |

- `completeness`：`full` 是 `--want` 里的都拿到了；`partial` 是缺了一部分，缺什么写进 `missing`；`link` 是只认出了身份。宿主会按实际输出重算，比较完整度时比的是实际拿到的输出集合（[§3.1](#31-运行器与节奏)）。
- `text` 是平台原文的文字，宿主据此生成 `content.md`：只读副本，总照主快照重写；原样在 `raw/`（[SPEC §1.2.3](SPEC.md#123-contentmd-和各栏)）。
- 数组字段总是出现，可以为空。

<a id="38-list-与跟着收"></a>
### 3.8 list 与跟着收

```
<command> sources
```

```json
{"ok": true, "sources": [
  {"source": "bilibili:fav:50948568", "title": "默认收藏夹", "action": "favorite", "count": 128},
  {"source": "bilibili:toview", "title": "稍后再看", "action": "favorite"}
]}
```

```
<command> list "bilibili:fav:50948568" --cursor "eyJwbiI6Mn0" --since 2026-10-31T00:00:00Z
```

```json
{
  "ok": true,
  "items": [
    {"url": "https://www.bilibili.com/video/BV1xx411c7mD", "identity": {"platform": "bilibili", "contentId": "BV1xx411c7mD"}, "title": "手冲咖啡入门", "action": "favorite", "actedAt": "2026-10-31T22:14:00+08:00"}
  ],
  "nextCursor": "eyJwbiI6M30",
  "evidence": {"fetchedAt": "2026-11-01T03:00:01Z", "access": "browser-session", "site": "www.bilibili.com"}
}
```

- `source` 是插件自己定的字符串，建议写 `<platform>:<kind>[:<id>]`；`file:` 开头的留给导入（[§3.9](#39-从文件导入)）。`sources` 列出用户能选的来源，要登录时也要先登录。
- 每页由宿主单独调用，页与页之间按节奏等待；不传 `--cursor` 是第一页，`nextCursor: null` 是最后一页。游标是插件私有的不透明字符串。
- 条目字段：`url` 或 `ref` 至少一个；`identity`、`title`、`author`、`publishedAt`、`action`（`favorite`、`like`、`share`、`view`、`unknown`）、`actedAt` 能给就给，时间 RFC 3339 带偏移（11-11 照 [§13](#13-待定) 第 30 项做了）。拿不到加入时间时 `actedAt` 留空，不用抓取时间冒充（[02 §15.5](research/02-adapter-ecosystems.md#155-批量导入与同步)）。
- 只有 `url` 的条目，宿主按 [§3.1](#31-运行器与节奏) 的顺序去取内容；带 `ref` 的条目，宿主回到这个插件调 `fetch <ref>`。`ref` 由插件自己定，对宿主不透明；宿主为每个 `ref` 记下它来自哪个插件、哪个 `source`，调 `fetch <ref>` 时按这个 `source` 设好 `${INPUT}` 一类的上下文。
- **跟着收（增量）**：每个订阅记下自己列过的身份，只存身份的哈希、不存标题，按机器记在 `jobs.db`，和翻页游标放在一起（推断，[SPEC §1.1](SPEC.md#11-目录树)）。宿主只收这个订阅没列过的身份：回收站里的跳过，资料库里已有的不新建、只多一项 `origins`（[SPEC §1.7](SPEC.md#17-删掉)）。所以它列过、你删掉的不会被它带回来，回收站到期移除以后也一样；它没列过的（比如你自己粘贴过、删掉、已经到期移除的），以后它第一次列出来就照新的收（11-06，这是代价）。`order: newest-first` 的来源，宿主从第一页往后翻，遇到一整页都是这个订阅列过的就停；`--since` 是这个来源上次跟着收成功跑完的时间，插件拿得到 `actedAt` 时可以据此提前结束、返回 `nextCursor: null`。`unordered` 的来源每次全量翻，频率由用户的订阅设置定。
- 一台机器第一次跑一个订阅，只记下列出来的身份，不收（[PRD 7.3](PRD.md#73-导入和跟着收) 对个人来源这样建议，这里所有订阅都照它，推断）。这一次跑完才把列出来的记成列过；还没跑完过一次的，下一次仍照第一次跑（推断）。换了 `fetchDevice`，新的那台也是第一次：上一台最后一次跑完到新的那台第一次跑之间新出现的，会漏掉（推断）。
- 批量导入的结果先进预览（新增、已有、可以升级完整度；回收站里的算已有，确认后拿回来，推断），用户确认后才按节奏去取内容（[PRD](PRD.md) 的批量导入）；跟着收是用户订阅过的来源，新条目直接收下，不再逐次预览。
- 跟着收收下的条目，`origins[]` 里那一项记 `entry: subscription`，另记 `subscription`（订阅 id）；批量导入（`/v1/batches` 和对列表来源的一次性导入）记 `import`，同一批共用一个 `importBatch`（[SPEC §1.2.1](SPEC.md#121-manifestjson-v1)）。

`sources` 里的 `count` 是平台页面上显示的数字，拿不到就不填；导入结果里写一次「平台显示多少条，取到多少条」，并说明差额可能是已删除或看不到的，不变成常驻的差额。

<a id="39-从文件导入"></a>
### 3.9 从文件导入

导入插件读用户已经导出的文件（`access: official-export`、`login: none`、`network.mode: none`），复用 `list` 和 `fetch`：

```yaml
capabilities:
  list:
    access: official-export
    login: none
    order: unordered
    files: ["twitter-*.zip"]
  fetch:
    access: official-export
    login: none
    input: [ref]
    outputs: [text, images, media:video, meta]
permissions:
  fs:
    read: ["${INPUT}"]
    write: ["${OUT}"]
  network: {mode: none}
```

```
USUYO_INPUT=<副本路径> <command> list "file:twitter-2026-10-30.zip"
```

```json
{"ok": true, "items": [
  {"ref": "bookmarks/1849000000000000001", "url": "https://x.com/someone/status/1849000000000000001", "title": "…", "action": "favorite", "actedAt": "2026-10-12T08:30:00Z"}
], "nextCursor": "500", "evidence": {"fetchedAt": "2026-11-01T03:00:01Z", "access": "official-export"}}
```

- 用户拖进一个文件或点「导入」时，宿主按 `files` 找能读它的插件；都对不上时让用户手选，插件不认识就回 `unsupported-input`。
- 调 `list` 之前，宿主把导入文件复制到按机器目录（[SPEC §1.1](SPEC.md#11-目录树)）下的 `libraries/<libraryId>/imports/<importBatch>/`（能用 clonefile、reflink 或硬链接就不真的复制），`${INPUT}` 和 `USUYO_INPUT` 指向这份副本；来源写 `file:<原文件名>`。导入要先进预览，用户可能几天后才确认，原文件这期间被移走也不影响；副本在这批导入结束或取消后清掉。用户下载目录的绝对路径不进 `ref`，也不随快照写进资料库。
- `ref` 由插件自己定，对宿主不透明，不含绝对路径。宿主记住每个 `ref` 来自哪个来源，`fetch <ref>` 时放行同一个 `${INPUT}`。副本已经不在时，宿主不调用插件，条目记 `dependency-missing`，`needs` 写 `file:<原文件名>`。
- 导出包里有内容的（X 归档、flomo、BiliNote 的 `note_results`），条目带 `ref`，宿主再调 `fetch <ref>` 取出内容和文件；只有链接的（Takeout 的稍后观看、书签），条目只带 `url`，内容交给平台插件去取，取不到就停在 `link`。
- 大文件可以在 `${CACHE}` 里留解压结果，或用常驻模式只打开一次。
- 条目在 `origins[]` 里那一项记 `entry: import`，同一批共用一个 `importBatch`。

<a id="310-health"></a>
### 3.10 health

```
<command> health
```

```json
{
  "ok": true,
  "ready": true,
  "protocol": 1,
  "version": "0.3.1",
  "upstream": [{"name": "chrome", "version": "…"}],
  "login": [{"site": "www.douyin.com", "state": "present"}],
  "checks": [{"name": "chrome", "ok": true, "message": "找到 Chrome"}]
}
```

- `ready`：所有必要检查都通过、能接调用；登录缺失不影响 `ready`，只影响要登录的能力。
- 登录状态谁来判断：
  - `credential: cookies`、`browser-profile` 的站点，`present`（`sites.cookies` 都在且没过期）、`missing`、`expired` 由宿主按 `sites.cookies` 自己算，插件不填；不带 `--probe` 的 `health` 也拿不到凭据。
  - `credential: app` 的站点（微信这类）宿主判断不了，由插件在 `login[]` 里报。
  - `health --probe` 时，宿主传入所有声明站点的凭据或资料目录，插件发一次轻量请求核对，回 `verified` 或 `rejected`。
- `credential: app` 的站点回 `login-required` 时，宿主不打开 `loginUrl`，只显示插件给的 `hint`。
- 默认不向平台发请求，只看本机的程序、设置、设备和凭据（ChubbySkills 的做法，避免为了检查去碰平台）。`--probe` 只在用户点「检查登录」时用。
- `protocol`、`version` 和清单不一致时，宿主不加载这个版本。

平台那一行显示登录有效（带上次成功的时间）、已过期、没登录、不确定，默认只按上次结果判断，点「检查登录」才查一次；登录过程中跟着插件显示现在在等什么：等扫码、要短信验证码。插件能不能用只看本机：最近的失败和 `health`，不往任何服务上报。获取插件组织可以用 Actions 定时对公开链接跑不登录的取法，把结果公开成一页、插件详情里链过去，页面写明只代表不登录、从云主机访问时的结果。

<a id="311-unfavorite"></a>
### 3.11 unfavorite

```
<command> unfavorite "https://www.douyin.com/video/7431234567890123456" --source "douyin:favorites"
```

- `--source` 是这条内容从哪个来源列出来的（[§3.8](#38-list-与跟着收)）：同一条内容可能同时在好几个收藏夹里，插件只从这个来源取消。没有来源时插件回 `invalid-args`。

```json
{"ok": true, "verified": true, "evidence": {"fetchedAt": "2026-11-01T03:00:01Z", "access": "browser-session", "site": "www.douyin.com"}}
```

- 只在用户打开了这个能力、并确认了这一批之后调用（[§2.6](#26-账号写操作)）。
- `verified: true` 表示拿到了平台侧的确认（例如重新读一次已不在收藏夹里）；平台只回了成功、没有确认时写 `false`。照 Easel 读回分档的做法：没有平台侧证据不算完成。
- 已经不在收藏夹里时也回 `ok: true`，`verified: true`。

<a id="312-常驻模式-serve"></a>
### 3.12 常驻模式 serve

要开浏览器或连设备的插件，每次调用都冷启动要好几秒，还会抢同一个资料目录。`run.serve: true` 的插件支持 `<command> serve`：宿主启动一次，经 stdio 发换行分隔的 JSON-RPC 2.0，分帧和 [§4.1](#41-分帧与生命周期) 相同。

| 方法 | 参数 | 结果 |
|---|---|---|
| `canonicalize` | `{input}` | 同 [§3.5](#35-canonicalize-与身份) 的信封去掉 `ok` |
| `fetch` | `{input, out, want, context}` | 同 [§3.7](#37-fetch-输出) |
| `sources` | `{context}` | 同 [§3.8](#38-list-与跟着收) |
| `list` | `{source, cursor, since, context}` | 同上 |
| `health` | `{probe}` | 同 [§3.10](#310-health) |
| `unfavorite` | `{input, source, context}` | 同 [§3.11](#311-unfavorite) |

```json
{"jsonrpc":"2.0","id":3,"method":"list","params":{"source":"douyin:favorites","cursor":null,"since":null,"context":{"credentials":"/private/var/folders/xx/usuyo-proc-07/cred/3/credentials.json","cookiesTxt":null,"browserProfile":"/Users/me/Library/Application Support/usuyo/profiles/www.douyin.com","settings":null,"secrets":null,"input":null}}}
```

- `context` 里是这一次调用的文件路径，对应一次一进程时的环境变量：`credentials` ↔ `USUYO_CREDENTIALS`、`cookiesTxt` ↔ `USUYO_COOKIES_TXT`、`browserProfile` ↔ `USUYO_BROWSER_PROFILE`、`settings` ↔ `USUYO_SETTINGS`、`secrets` ↔ `USUYO_SECRETS`、`input` ↔ `USUYO_INPUT`、`provider` ↔ `USUYO_PROVIDER`（这一个是代发参数本身，不是路径）。这些路径都在进程的暂存目录里（[§7.1](#71-常驻进程的路径)），凭据和密钥文件在这次响应之后删除。`HOME`、代理等进程级的变量在启动时给定。
- **按站点集合分进程。** `permissions` 按插件声明，沙箱按进程生效，凭据按能力发。所以宿主按「这次调用用到的站点集合」分别起常驻进程，沙箱也按这个集合生成：`login: none` 的能力只发给不带站点的那个进程。抖音的例子里，`fetch` 和 `list` 各一个进程，`list` 那个进程里开着带登录的浏览器，`fetch` 那个进程碰不到它。
- **资料目录的交接。** 带 `browser-profile` 站点的常驻进程从第一次用到这个目录起持有它的锁，调用之间可以让浏览器开着。别的进程要用同一个目录时，宿主先让持有它的进程退出，再交出去（[§6](#6-凭据)）。
- 失败返回 JSON-RPC `error`：`code: -32000`，`data` 是 [§3.3](#33-信封与错误码) 的错误对象。
- 宿主同一时间只向一个进程发一个请求；节奏照 [§3.1](#31-运行器与节奏)。常驻进程正忙时，`canonicalize` 改用一次一进程的方式调用，不排在它后面。
- 进度和等待用 `progress`、`waiting` 通知，参数同 stderr 事件，另带 `id`。取消用 `cancel` 通知。
- 空闲 10 分钟后宿主关闭 stdin，插件 5 秒内退出。进程崩溃怎么处理见 [§3.1](#31-运行器与节奏) 的表。
- 一次一进程的子命令仍然必须有；两种方式结果相同，SDK 从同一套处理函数生成。

<a id="313-宿主和-plugin-kit-现在做到的11-11"></a>
### 3.13 宿主和 plugin-kit 现在做到的（11-11）

照 b574487 的 `crates/usuyo-plugin`、`crates/usuyo-core` 写。和上面不一样的以这里为准，直到正文改过来。

- **找插件**：按机器目录的 `plugins/<owner>/<repo>/<版本>/usuyo-plugin.yaml`（内核可以配几个根目录，同一个 id 以先找到的为准；开发版另把 web 插件放进 `dev-plugins/builtin/web/<版本>/`），同一个插件只用最高版本，最高版本不合格整个不加载，不退回旧版本；`id`、`version` 要和目录对上。清单里不认识的字段只记警告。签名和来源证明还不查（[§8.3](#83-校验与来源证明)）。只有一次一进程的调用，`serve` 还没做。
- **环境变量**：先清空，再放 `permissions.env` 里列的、宿主自己环境里有的，再放宿主设的（同名的以宿主为准，清单也不许列它们，[§2.7](#27-权限-permissions)）。宿主设：`USUYO_PROTOCOL=1`、`USUYO_PLUGIN_DIR`、`USUYO_CACHE_DIR`（按机器目录的 `cache/plugins/<owner>/<repo>/`）、`USUYO_CALL_ID`（UUIDv7）、`USUYO_LOCALE`、导入时的 `USUYO_INPUT`、`HOME`、`XDG_CONFIG_HOME`、`XDG_CACHE_HOME`、`TMPDIR`、`TMP`、`TEMP`、`PATH`（`${TMP}/bin` 加系统目录）。工作目录是 `${TMP}`，stdin 是空的。
- **`USUYO_SETTINGS`**：按机器 `settings.json` 里 `plugins.<插件 id>.settings` 有值时，宿主把它写成这次调用文件夹里的 `settings.json`，变量指向它；没有就不设。
- **`USUYO_NETWORK`**：宿主查 `example.com`、`example.org` 两个名字（两秒内），两个都有回答、回答全在 `198.18.0.0/15` 里时设成 `fake-ip`，结果缓存一分钟；查不到、超时都算不是。不看要取的那个域名自己的回答。
- **节奏做了一部分**：[§3.1](#31-运行器与节奏) 的 `interval`、`daily`、`batch: false`、每平台一个调用的锁有了（取最严的一组，记账在 `jobs.db`，重启后照旧）；整机锁和设备锁还没做；任务一次只跑一个。「现在再试」「先别试」有了（`POST /plugins/{owner}/{repo}/try-now`、`…/hold`），用户改节奏的设置还没做。
- **凭据和密钥**：`credential: cookies` 的站点经 `USUYO_CREDENTIALS`、`USUYO_COOKIES_TXT`，密钥设置经 `USUYO_SECRETS`，访客身份经 `USUYO_VISITOR`（[§2.5.1](#251-访客身份-visitors)）。值在系统钥匙串里（没有钥匙串的系统上是按机器目录的 0600 文件），按机器目录的 `logins.json` 只记名字、域名和过期时间；文件放在按机器目录 `run/` 下这次调用自己的 0700 目录里，0600，进程结束就删；交出去的值从 stderr、插件的错误和宿主拼的错误里洗掉（包括 JSON、Debug 转义过的写法）。plugin-kit 的 `http::Cookie::visitor` 把访客身份读成只发给给定主机、只走 https 的 Cookie。
- **还没传的**：`USUYO_BROWSER_PROFILE`、`USUYO_PROVIDER`、`ADB_SERVER_SOCKET`、`ANDROID_SERIAL`、各个代理变量。本机代理和沙箱（[§7](#7-权限与隔离)）都还没做；插件的出站现在靠 plugin-kit 的客户端自己把关（[§10.1](#101-plugin-kit-的-http-客户端11-11)）。
- **调用文件夹**：资料库的 `.usuyo/tmp/usuyo-call-<UUIDv7>-<随机>/`，里面 `tmp/`、`tmp/home/`、`tmp/bin/`，`fetch` 另有 `out/`；用完整个删掉，崩溃留下的在下次打开库时清掉。
- **时长**：`canonicalize` 10 s、`fetch` 300 s、`sources` 30 s、`list` 每页 60 s；发过 `progress` 以后 120 s 没有新的算卡住；一次调用里 `waiting` 加起来最多 600 s，等的时候另两个时钟停；能力的 `timeout` 最多按 2 小时算。超时或取消先给进程组 SIGTERM，5 秒后 SIGKILL；正常退出后也给进程组发一次 SIGKILL，留下的子进程一起结束。Windows 上只结束这一个进程（还没有 Job Object）。
- **stdout、stderr**：stdout 最多 32 MiB，超了 `invalid-output`；stderr 留最后 200 行，每行最多 8 KiB；解析不了的 `@usuyo ` 行忽略。
- **信封里的字**：标题、作者的 `name`、`platformId`、`url`，错误的 `message`、`hint`、`site`，`missingReason`、`evidence.site`，`list` 条目的标题和作者，`sources` 的标题：各截到 1024 字节（按字符边界），控制字符（含换行）换成空格。`platformTags` 最多 64 个，每个截到 100 字节。正文 `text` 不动。这些是截断，不拒。
- **会拒的**：插件给的链接（`identity.url`、`resolvedUrl`、`list` 的 `items[].url`）超过 4096 字节或有控制字符；`contentId` 空、超过 256 字节或有控制字符；身份的 `platform` 不在清单的 `platforms` 里；`comments` 不是 `comments.json`；同一个文件引用两次、引用的文件不在；输出目录不合 [§3.6](#36-fetch-与输出目录)（最多 10,000 项，默认 8 GiB）；插件报宿主才用的码（`timeout`、`crashed`、`invalid-output`、`profile-busy`）；`dependency-missing` 不带 `needs`。都记 `invalid-output`。
- **完整度**：宿主照 `--want` 和实际输出重算，和插件报的不一样记警告；算出来是 `link` 按 `unsupported-input`。
- **暂停**：`rate-limited` 按 `retryAfter`，没给 15 分钟，最多一小时；`blocked` 一小时；通用网页插件按主机暂停，别的按平台（[§3.1](#31-运行器与节奏)）。重试按 30 秒乘 4 的次方退后，最多 6 小时、6 次。

<a id="4-转换与导出插件协议"></a>
## 4. 转换与导出插件协议

<a id="41-分帧与生命周期"></a>
### 4.1 分帧与生命周期

- stdio 上换行分隔的 JSON-RPC 2.0：一行一条消息，消息里不能有换行；stdout 只放协议消息，日志写 stderr；分帧和 MCP stdio 相同（[03 无状态模型与 stdio](research/03-agent-protocols.md#无状态模型与-stdio)）。
- 宿主用 `run.command` 启动进程，先调 `describe`，核对 `protocol` 和 `version`，对不上不加载；可以常驻，模型加载一次处理多条。`describe` 的回答在进程活着的时候不变，宿主只在起进程时问一次；它不列能产出哪几种，种类照清单在每次请求和每个结果上核。宿主同一时间向一个进程只发一个 `run`；`gpu: true` 的插件同一时间只开一个进程，其余的按「同时处理几个」的设置并行开进程。
- [§3.2](#32-调用约定) 的环境变量表同样适用于转换、导出插件，只有 `USUYO_SETTINGS`、`USUYO_SECRETS`、`USUYO_INPUT`、`USUYO_CREDENTIALS`、`USUYO_COOKIES_TXT`、`USUYO_VISITOR`、`USUYO_BROWSER_PROFILE` 不给：这两类插件的非密钥设置只经 `run` 的 `options` 传，密钥只经宿主代发。常驻的转换进程拿到 `USUYO_PLUGIN_DIR`（安装目录）和 `USUYO_MODELS_DIR`（`${MODELS}`），没有 `USUYO_CALL_ID`：一个进程接好几次 `run`，哪一次看请求的 `id`。标准流按本节。进程能看到哪些路径见 [§7.1](#71-常驻进程的路径)。
- 退出：宿主关闭 stdin，插件 5 秒内退出；否则 SIGTERM，再 5 秒后杀进程组。
- 转换进程闲了（120 秒没有 `run`）宿主可以关掉，放掉模型占的内存，下次要用再起。
- `progress` 只用来判断卡住（[§3.4](#34-超时与取消) 的无进度时长），不往界面报：界面上没有百分比。卡住的时钟从这次 `run` 的第一个 `progress` 起算，从没发过的只受总时长约束。
- stdout 上每一行都要是回答或通知，最长 1 MiB。不是 JSON 对象、`jsonrpc` 不是 `"2.0"`、插件发来请求、回答里 `result` 和 `error` 不是恰好一个、回答的 `id` 不是正在等的那个，或者一行太长，宿主记 `invalid-output`，关掉整个进程组，下次要用再起。认不出的通知忽略。两次调用之间写到 stdout 的，宿主在下一次调用前换一个进程。
- 错误：协议错误用 JSON-RPC 标准码（`-32700`、`-32600`、`-32601`、`-32602`）；插件自己的失败用 `-32000`，`data` 是错误对象，`code` 取 [§3.3](#33-信封与错误码) 里适用的值（多是 `unsupported-input`、`dependency-missing`、`network`、`cancelled`、`internal`）。

| 方向 | 方法 | 用途 |
|---|---|---|
| 宿主 → 插件 | `describe` | 返回 `{protocol, version, models?}`，`models` 是实际可用的模型或语言 |
| 宿主 → 插件 | `health` | 同 [§3.10](#310-health) 的 `ready`、`checks`，例如模型在不在、GPU 能不能用、后端服务通不通 |
| 宿主 → 插件 | `run` | 处理一条（转换）或一批（导出） |
| 宿主 → 插件 | `readback` | 只有导出插件，`readback: true` 时 |
| 宿主 → 插件（通知） | `cancel` | `{id}`；插件仍要回这个请求，错误码 `cancelled`（LSP 的规则） |
| 插件 → 宿主（通知） | `progress` | `{id, done, total, unit, message}`，只用来判断卡住 |

<a id="42-转换的-run"></a>
### 4.2 转换的 run

```json
{"jsonrpc":"2.0","id":7,"method":"run","params":{"item":{"id":"bilibili-4plrcxk343onmyxibmrr672kga","kind":"video","title":"手冲咖啡：三段式注水","lang":"zh"},"want":["asr"],"inputs":[{"kind":"audio:pcm16k","mime":"audio/wav","path":"/private/var/folders/xx/usuyo-proc-03/in/7/0/audio.wav","snapshot":"019a6f2b-4a10-7c3e-8b21-0d9e8f7a6b5c"}],"out":"/private/var/folders/xx/usuyo-proc-03/out/7","options":{"vadThreshold":0.4},"provider":null}}
```

```json
{"jsonrpc":"2.0","id":7,"result":{"outputs":[{"kind":"asr","path":"asr.vtt","lang":"zh"}],"tool":{"name":"sherpa-onnx","version":"2.0.1"},"options":{"language":"auto","itn":true,"vad":"silero-vad-v5","vadThreshold":0.4,"maxCueSec":15},"model":{"name":"sense-voice-int8-2024-07-17"},"marks":[],"empty":false}}
```

```json
{"jsonrpc":"2.0","id":8,"result":{"outputs":[],"tool":{"name":"sherpa-onnx","version":"2.0.1"},"options":{"vadThreshold":0.5},"marks":[{"code":"no-speech","speechSec":0.4,"durationSec":5.0}],"empty":true,"reason":"no-speech"}}
```

- `want` 是这次要哪几种产出，取清单 `produces` 的子集、不为空、不重复；一个插件能做几种时（抽音轨的插件做 `play`、`audio:pcm16k`、`probe`、`keyframes`）靠它分。宿主只核这几条，一次可以要几种；预装的 `media`、`asr-sherpa`、`ocr-vision` 一次只收一种，`want` 多于一项回 `invalid-args`。
- `inputs` 是条目文件放进进程暂存目录后的绝对路径，只读（`${INPUT}`，怎么放进去见 [§7.1](#71-常驻进程的路径)）：`in/<callId>/<下标>/<原文件名>`，下标是它在 `inputs` 里的位置，原文件名留着扩展名；和 `out` 指的 `out/<callId>/` 在同一个暂存目录下。大块数据不走管道。`consumes` 写了 `mime` 的，宿主只给符合格式的文件，必要时先让抽音轨一类插件转好。每个输入另有可选的 `snapshot`（所在快照）、`imageKey`（图文的图和封面）、`frameSec`（关键帧在第几秒）、`role`（`play` 时哪个是 `video`、哪个是 `audio`）。
- `options` 是这个插件的非密钥设置。
- 插件只写 `out`。结果里：
  - `outputs[]` 每项 `{kind, path, lang?}`：`kind` 取 `want` 里会出文件的种类，每种一项；`path` 相对 `out`；`lang` 要像语言标签（字母和数字，用 `-` 分段，每段最多 8 个，整个最多 35 个）。`keyframes` 的 `path` 是 `keyframes.json`，帧图片在它旁边的 `keyframes/` 里。文件格式见 [SPEC §1.2.2](SPEC.md#122-raw-和-derived)。
  - `tool` 必填：插件里干活的程序和版本 `{name, version}`，两项都不能空，例如 `ffmpeg 9.0.2`、`sherpa-onnx 2.0.1`；宿主不猜。
  - `options`：这次实际用的参数，含插件补上的默认值；宿主原样记下，重做时原样交回。
  - `model`：用了模型时 `{name}`，`name` 要是清单 `models[].name` 之一；sha256 由宿主自己核。
  - `marks[]`：很明显的标记 `{code, speechSec?, durationSec?, frozenSec?}`，`code` 是 `no-audio-track`、`no-speech`、`static-picture`、`no-text`（feature 003 [§6](features/003-transforms.md#6-很明显的标记)）；秒数有限、不小于 0。
  - `probe[]`：`want` 有 `probe` 时，每个输入一项 `{input, durationSec?, streams: [{kind, codec?}]}`，`input` 是 `inputs` 的下标，`kind` 是 `video`、`audio`、`subtitle`、`other`；每个流只报种类和编码。宿主用它配对画面和音轨、判出没有声音轨。
- `out` 照 [§3.6](#36-fetch-与输出目录) 的输出目录规则检查；宿主核对后连同自己算的哈希写成一次转录 `derived/runs/<runId>/`，替插件写 `run.json`（[SPEC §1.2.2](SPEC.md#122-raw-和-derived)），不再写 `<文件>.meta.json`；插件不用自己写。
- `empty: true` 表示这条没有可产出的东西，这时 `outputs` 为空、`reason` 必填：`no-speech`（没听到人说话）、`no-audio-track`（没有声音轨）、`no-text`（没认出字）；`empty: false` 时不带 `reason`。宿主不再往下游编排摘要，免得模型对着空文稿编造（AI-Video-Transcriber 的 `no_speech` 做法）。不合这几条的结果、`kind` 不在 `want` 里或重复的，宿主按 `invalid-output`。
- 缺模型回 `dependency-missing`，`needs: ["model:<名字>"]`（[§3.3](#33-信封与错误码)）。
- 11-11：这些消息的类型在 `crates/usuyo-plugin` 的 `protocol::transform`，JSON Schema 是 `crates/usuyo-plugin/schema/transform-*.schema.json`；Rust 插件的循环在 plugin-kit 的 `transform` 模块（[§10](#10-契约测试与-plugin-kit)）；宿主一侧是 `crates/usuyo-plugin` 的 `host::transform`。

<a id="43-模型与密钥由宿主代发"></a>
### 4.3 模型与密钥由宿主代发

- 模型 provider 是宿主的一份配置（名称、OpenAI 兼容端点、模型、密钥），密钥在系统钥匙串里（[ADR-003 §1](decisions/ADR-003-plugins.md#1-插件分三类)）。
- `provider` 不是 `none` 的插件，`run` 参数里会有：

```json
{"provider": {"baseUrl": "http://127.0.0.1:47822/v1", "token": "q8Zr2VbN4kTxW1mYc7HdLs0JpEa5Gu9F", "model": "summary"}}
```

- 插件按 OpenAI 兼容接口请求 `baseUrl`（`/chat/completions`、`/audio/transcriptions`），带 `Authorization: Bearer <token>`。这个 token 是至少 128 位的随机值，只对这一次调用、这个插件、宿主的本机端点有效，调用结束即作废；宿主比对时用常数时间比较，因为本机任何进程都能连这个端口；宿主换上真正的密钥转发给用户配的 provider，记下这次请求是否离开本机。宿主自动把这个端点加进插件的网络许可。
- **获取插件也可以用。** 能力声明了 `provider`（[§2.3](#23-获取插件的字段)）、用户配了模型时，一次一进程的调用经 `USUYO_PROVIDER` 拿到同样的 `{baseUrl, token, model}`，常驻进程在请求的 `context.provider` 里拿到；token 的规则不变：一次调用、一个插件、宿主的本机端点。视觉层插件用看图模型找控件时走这里，界面照样标「离开本机」（[§9.5](#95-视觉层流程)）。
- 包装自带模型的第三方 Agent（Mika 的 MCP 或 HTTP 接口一类）不在这一条之内：它的模型调用在手机上、由它自己发，宿主标不了「离开本机」，节奏也管不到。这类插件 `usuyo-fetch` 收不收是待定项（[§13](#13-待定) 第 20 项）。
- 带 `inject` 的密钥设置（导出到 Notion 这类云端目标）用同一个办法：宿主在 `run` 参数的 `endpoints` 里给一个本机地址，例如 `{"notion": "http://127.0.0.1:47823/"}`，转发时按 `inject` 填请求头。插件始终拿不到密钥明文。

自带模型、每次走第三方模型的手机 Agent 插件 `usuyo-fetch` 不收：宿主的节奏和「离开本机」的标注都管不到。你自己装的第三方插件照常能用；手机上导出的清单走导入插件读。

<a id="44-导出的-run-与-readback"></a>
### 4.4 导出的 run 与 readback

资料库的形状按 usuyo 自己的需要定（[SPEC 第 1 层](SPEC.md#1-资料库-v1第-1-层)）；导出插件把宿主给的副本转成目标软件的样子。读回是 `run` 反过来做：把目标里的文件按 `usuyo_id`（主题、卡片、成稿按它们的 id）转回 `run` 收到的用户层，交回值，不是改动；宿主拿它和底本、库里现在的那份照 [SPEC §1.4](SPEC.md#14-你写的那一层)「读进来」、[§1.8](SPEC.md#18-同步) 三方合并，同步、在 usuyo 外改文件、导出读回用的是同一个合并。插件不读资料库文件，宿主不解析目标的语法。

```json
{"jsonrpc":"2.0","id":2,"method":"run","params":{"items":[{"id":"douyin-4plrcxk343onmyxibmrr672kga","manifest":"/…/in/2/douyin-4plrcxk343onmyxibmrr672kga/manifest.json","content":"/…/content.md","notes":"/…/notes.md","derived":{"derived/summary.md":"/…/derived/summary.md","derived/asr.vtt":"/…/derived/asr.vtt","derived/keyframes/":["/…/derived/keyframes/0012.jpg","/…/derived/keyframes/0047.jpg"]},"files":[{"path":"/…/raw/<snapshotId>/images/001.jpg","kind":"image","mime":"image/jpeg"},{"path":"/…/raw/<snapshotId>/media/video.mp4","kind":"video","mime":"video/mp4"}]}],"topics":[{"slug":"手冲咖啡","path":"/…/in/2/topics/手冲咖啡.md"}],"options":{"folder":"usuyo"},"previous":{"douyin-4plrcxk343onmyxibmrr672kga":{"target":"usuyo/手冲咖啡：三段式注水.md"}},"endpoints":{},"out":"/…/out/2"}}
```

```json
{"jsonrpc":"2.0","id":2,"result":{"files":[{"path":"items/douyin-4plrcxk343onmyxibmrr672kga.md","target":"usuyo/手冲咖啡：三段式注水.md","usuyoId":"douyin-4plrcxk343onmyxibmrr672kga"},{"from":"item:raw/<snapshotId>/images/001.jpg","target":"usuyo/附件/douyin-4plrcxk343onmyxibmrr672kga-001.jpg","usuyoId":"douyin-4plrcxk343onmyxibmrr672kga"}]}}
```

- `items[].derived` 的键是 `produces` 里的值（`derived/asr.vtt`、`derived/ocr/*.json`、`derived/keyframes/` 等），值是路径，目录或通配的值是路径数组。`items[].files` 列出条目的原图和媒体：`path`、`kind`、`mime`；是各快照里还在的那些，同一 `sha256` 只列一次（先取主快照那份）。
- `writes: files`：插件把文件写进 `out`，结果里写明每个文件要放到目标的哪个相对路径；`out` 照 [§3.6](#36-fetch-与输出目录) 的输出目录规则检查。原图、视频这类不用改的文件，结果里写 `{from: "item:<相对条目目录的路径>", target, usuyoId}`（例如 `item:raw/<snapshotId>/images/001.jpg`），由宿主从资料库直接复制，不经过 `out`。宿主写进目标文件夹、记哈希，导出记录放按机器目录的 `libraries/<libraryId>/exports/<target>.json`（每个对象 `{path, hash, base}`，见下面的 `readback`），导出路径每台各管（[SPEC §1.1](SPEC.md#11-目录树)）。`run` 时插件不碰目标文件夹，`readback` 时只读它（[ADR-003 §7](decisions/ADR-003-plugins.md#7-导出与读回)、[§7.1](#71-常驻进程的路径)）。
- `target` 必须是规范化后的相对路径，不含 `..`、不是绝对路径，解析后仍在目标文件夹内；宿主写入时不跟随目标侧已有的符号链接。目标路径上已经有文件、而文件里没有对得上的 `usuyo_id` 时，宿主不覆盖，记进这次导出的问题列表让用户处理。
- `previous` 是导出过的对象的目标路径 `{id: {target}}`，取读回找到的路径，没找到的取导出记录里的 `path`（推断）。插件写回这个路径，写主题链接也照它；只有第一次导出的对象才按标题算路径。
- `writes: api`：插件经 `endpoints` 调云端接口，结果里返回 `records: [{usuyoId, remoteId}]`。
- 每个导出文件带 `usuyo_id`。**Obsidian 导出照这样转**（宿主给的字段照 [§13 第 34 项](#13-待定)，待定）：
  - 条目一篇笔记。属性是 Web Clipper 的 `title` `source` `author` `published` `created` `description` `tags`（`description` 取摘要层，没有不写；`tags` 取 `tags.user`），加 `usuyo_id` `platform` `kind` `usuyo_takeaway` `usuyo_topics`（指向主题笔记的 wikilink，写成相对 vault 的路径，例如 `[[usuyo/主题/手冲咖啡]]`，不和你自己的笔记撞名，推断）；YAML 标量一律双引号。`published`、`created` 写 `YYYY-MM-DD`：`published` 有 `publishedAtOriginal` 就按它的偏移取日期，没有按导出这台的本地日期；`created` 取 `addedAt`（主题、卡片、成稿取 `createdAt`）的本地日期。正文先是你的笔记（`notes.md` 的正文），接一行分隔线 `%% usuyo：这行以下每次导出重写 %%`，下面是划线（照 [SPEC §1.4.2](SPEC.md#142-卡片成稿记一句和划线) 的写法，行末 ` ^<id>` 本来就是 Obsidian 的块 id）、摘要、原文（宿主给的 `text`）、口播等各层。`notes` 里有一行正好等于分隔线，就转义，readback 时还原。通用 Markdown 文件夹用同一条分隔线，写成 HTML 注释。
  - 主题、卡片、成稿各一篇：`usuyo_id`、`title`、`created`，卡片另加 `usuyo_topics`；正文就是宿主给的副本，没有标记。主题页不写成员列表：条目笔记的 `usuyo_topics` 会出现在 Obsidian 的反向链接里，那就是成员。
  - 文件名照 [SPEC §0.6](SPEC.md#06-文本和文件)，另把 Obsidian 链接、标题、块用的 `# ^ [ ] |` 换掉；目标文件夹里撞名的按 id 顺序加「 2」，记下目标路径；`folder` 变成子文件夹。在 vault 里改名、挪文件夹不碍事，读回按 `usuyo_id` 认；再导出时写回读回找到的路径。
  - 脚注、` ^<id>`、`usuyo://` 链接照原样过去（私密条目的脚注宿主给副本时已换掉，`usuyo://item/forgotten` 照留），不另加 wikilink。
  - readback 照这样转回来：
    - 属性：`tags` → `tags.user`，`usuyo_takeaway` → `takeaway`，`usuyo_topics` → `topics`（先由 wikilink 找到那篇笔记，再换成它的 `usuyo_id`）。卡片只有 `usuyo_topics` 这一项。
    - 条目正文：front matter 和分隔线之间的部分 → `notes`。
    - 主题、卡片、成稿：front matter 之后的整篇 → `body`。
    - 划线、分隔线以下的各层、日期、标题、文件名和别的属性不读回。
  - 待核实：Obsidian 留不留带偏移的日期时间属性（要加一个带时刻的属性，照它认的写法）。
- `readback`：插件只读整个目标文件夹，在里面按 `usuyo_id` 认出导出过的文件，挪过、改过名的也认得；不看以 `.` 开头的文件夹（`.trash`、`.obsidian`），那里的文件当作不在（推断）；把每份转回 `run` 收到的用户层，交回值：

```json
{"jsonrpc":"2.0","id":5,"method":"readback","params":{"vault":"/…/Obsidian/笔记","options":{"folder":"usuyo"}}}
```

```json
{"jsonrpc":"2.0","id":5,"result":{"objects":[{"id":"douyin-4plrcxk343onmyxibmrr672kga","kind":"item","path":"usuyo/手冲咖啡：三段式注水.md","hash":"sha256:<读到的那份的哈希>","takeaway":"闷蒸 30 秒比水温要紧","notes":"第一段水量记得称。\n\n周末试 1:15 粉水比","tags":{"user":["咖啡","周末要试"]},"topics":[{"id":"0193f2a4-7c1e-7a3b-9d2e-5f0a1b2c3d4e"}]},{"id":"0193f2a4-7c1e-7a3b-9d2e-5f0a1b2c3d4e","kind":"topic","path":"usuyo/主题/手冲咖啡入门.md","hash":"sha256:<读到的那份的哈希>","body":"……"}],"problems":[{"path":"usuyo/读书笔记.md","reason":"unparseable"}]}}
```

- `readback` 的参数：`vault` 是目标文件夹的绝对路径（`${TARGET}`，只读，[§7.1](#71-常驻进程的路径)），`options` 同 `run`。
- `objects[]` 里 `id`、`kind`、`path`、`hash` 是读回自己的键：`kind` 是对象种类（`item` `topic` `card` `draft`），不是条目或成稿的 `kind`；主题和成稿笔记的属性一样，插件分不出，`kind` 由宿主按 id 查，插件可以不给。`path` 相对目标文件夹，不是 `run` 里副本的 `path`。用户层的键照 `run`（[§13 第 34 项](#13-待定)）：`takeaway`、`notes`、`tags`（只有 `user`）、`topics`（每项只有 `id`）；主题、卡片、成稿的正文叫 `body`，是 `run` 里 `path` 那份副本的内容（推断）。`hash` 是读到的那份文件的 sha256。
- `problems[]` 是 `{path, reason, id?}`：属性坏了也按行找 `usuyo_id:`，找得到就带上 `id`，带 `id` 的对象这次不重新生成、不写；找不到 `usuyo_id` 的文件不算导出过的，不报（推断）。`reason` 只有两种，这份文件整篇不读：
  - `unparseable`：front matter 坏了、分隔线没有或不止一条、两个文件同一个 `usuyo_id`。
  - `unknown-topic`：`usuyo_topics` 里有链接指不到带 `usuyo_id` 的笔记。指到的不是库里的主题（成稿、卡片、条目），由宿主照 `unknown-topic` 办，那份不读也不写（推断）。
- `readback(run(V))` 按值等于 V 的用户层：字符串逐字节相同，集合相等。readback 就按这条定义，`usuyo plugin check` 跑这条往返测试（[§10](#10-契约测试与-plugin-kit)）。这里说的用户层只有 `takeaway`、`notes`、`tags.user`、`topics`，主题、卡片、成稿再加 `body`；划线、标题不算。`base`、往返测试、「和 `base` 一样」都只比这几项。
- 导出记录每个对象存 `{path, hash, base}`。`base` 是上次和 vault 对上的那份用户层：写出去了就是交给 `run` 的那份，没写出去就是那次读回的值（下面第 3、4 步）；私密条目的脚注已换成占位（[SPEC §1.3](SPEC.md#13-条目的四个维度)）；占位行在底本和 vault 里一模一样，三方合并自然留下库里的真值。不另调一次 `readback` 读写出去的文件来取底本：每次导出只读回一次，转回去再转回来不变由往返测试保证。
- 导出只在你点「现在导出」时跑，先不做自动、定时导出（11-06 定，[回答表](answers.md)）。`readback: true` 的导出插件，宿主每次导出做四步：
  1. 调 `readback`。
  2. 逐个对象处理：
     - 库里没有，或在回收站里：列成 `item-deleted`（[SPEC §1.7](SPEC.md#17-删掉)）。
     - 和 `base` 一样：跳过。
     - 其余做 merge3(`base`, 库里现在的, vault)，照 [SPEC §1.8](SPEC.md#18-同步)：集合两边各自加的都留，一边去掉的就去掉；标量只有同一项两边都改了才出卡片；正文按行合并，只有改到重叠的行才算合不上，相邻的不算。
     - 合不上的：库里那份留下，vault 那份进回收站（`reason: conflict`，`kind` 用已有的 `notes` `topic` `card` `draft`），出「两边都改过的」卡片，vault 里这篇照库里的重写。
     - 差异照 [SPEC §1.4](SPEC.md#14-你写的那一层)「读进来」记成已有的 op（`set-tags` `set-topics` `set-takeaway` `edit-notes` `edit-topic-page` `edit-card` `edit-draft`，[SPEC §1.5.1](SPEC.md#151-op-词表)），记 `actor: external`、`via: readback:<插件 id>`。一次导出算一批，能整批撤销。
     - 没有底本的（导出记录丢了，或另一台第一次导出到同一个 vault）：照 [SPEC §1.8](SPEC.md#18-同步)「云盘的冲突副本」那条：集合取并集，标量和正文两边不一样就算合不上，库里那份留下，vault 那份进回收站、出卡片。两台轮流导出到同一个 vault 时，底本按机器存，一台写出去的在另一台看来就是在 Obsidian 里改的：之后在库里去掉的会被加回来，改到同一段会出卡片（都记进改动记录，能整批撤销）。一个 vault 建议只由一台导出（推断）。
  3. 按合并后的库调 `run`，写到读回找到的路径。写之前再比一次文件哈希：和读回时的 `hash` 不一样（比如你正在 Obsidian 里打字），这一份这次不写，下次导出再读；它的 `base` 换成这次读回的值（推断：换成交给 `run` 的那份，下次会把没写过去的库里改动当成 vault 那边删掉的）。这次没交给 `run` 的（比如导出后设成私密的）同样，`base` 换成这次读回的值（推断）。
  4. 写出去的那些，`base` 换成这次交给 `run` 的那份用户层；没写出去的（`problems`、`item-deleted`）`base` 不动。
- 出了问题的文件（`problems`、`item-deleted`）不读也不写，列进这次导出的问题列表。在 vault 里删掉的笔记不算删掉，下次导出重新生成。

<a id="5-投递接口"></a>
## 5. 投递接口

很多现成工具不是宿主能启动的进程：浏览器扩展、油猴脚本、iOS 快捷指令、Android 分享、Agent Skill。它们走投递接口，把链接或已经取到的内容交给宿主。内容的形状就是 [§3.7](#37-fetch-输出) 的 fetch 输出；来源由宿主按令牌标明，投递方自己不能改。不带令牌的入口（`usuyo://` 链接一类）都要用户在窗口里确认一次才进队列。

<a id="51-本机-http-接口"></a>
### 5.1 本机 HTTP 接口

对应 PRD 的浏览器扩展、本地 HTTP API，约定照 [03 §6.3](research/03-agent-protocols.md#63-对歇存的意义)。11-11 代码做了其中一部分：`/v1/ping`、`/v1/deliver`（只收链接）、`/v1/batches` 和按投递端发的令牌，和窗口用的路由在同一个监听口上，规则见 [SPEC 本机接口](SPEC.md#本机接口第-2-层11-11-照代码)；下面没做的部分标了「还没做」。

- 只绑 `127.0.0.1`，端口 `47615`，和给窗口用的路由是同一个监听口，不另开端口（11-11 代码；原先设想的 `47821` 不用）。
- 每个请求带 `Authorization: Bearer <令牌>`。令牌按投递端分别生成：名字（例如「xhs-vault 脚本」）、入口类型（`extension`、`mobile`、`api`），可单独撤销。11-11 代码：在应用的导入页里发（`POST /deliver-tokens`，主令牌调），格式 `<id>.<秘密>`，整个令牌只在发的那一次回给调用方；存在按机器目录的 `deliver-tokens.json`，不放进资料库。投递令牌只能进 `/v1/ping`、`/v1/deliver`、`/v1/batches`。
- 除 `chrome-extension://`、`moz-extension://`、`safari-web-extension://` 以外，带任何 `Origin` 头的请求都拒绝，包括 `Origin: null`（沙箱 iframe、`file://` 页面会带它），也不回 CORS 头：普通网页发不进来，也读不到响应。没有 `Origin` 的请求放行。宿主另外核对 `Host` 头是 `127.0.0.1:<端口>`、`localhost:<端口>` 或 `[::1]:<端口>`。各油猴脚本管理器的 `GM_xmlhttpRequest` 实际带什么 `Origin` 待核实。11-11 代码：投递令牌发出去以前先握手：`POST /handshake`，请求 `{nonce, client}`，`client` 是令牌 `.` 前面的 id，内核用整个令牌当密钥回 HMAC 证明，令牌本身不上线路（[SPEC 本机接口](SPEC.md#本机接口第-2-层11-11-照代码)）。`/v1/…` 不回 CORS 头，也不答 CORS 预检（Safari 的 Userscripts 没有网站访问权限时会先发预检，还没做，[总纲](BUILD.md#5-功能线)）。
- 条目在 `origins[]` 里那一项的 `entry` 取令牌的入口类型。`extension` 令牌交来已取到的内容时，快照的 `plugin` 记 `deliver:<令牌名>`；只交链接的，快照记去取它的那个插件。投递来的内容同样按不可信输入处理。
- **令牌能交什么。** `api` 和 `mobile` 类令牌只接受链接（`url`、`text` 和 `/v1/batches`），不接受已取到的内容；已取到的内容只接受 `extension` 类令牌和收件文件夹（[§5.2](#52-监听文件夹)）（11-11 还没做：`/v1/deliver` 现在只收链接，没有 multipart，任何令牌都不能交已取到的内容）。Agent 用 `api` 令牌交已取好的内容，等于没经过改动提议就改了已有条目，和「Agent 写入先提议后生效」的原则冲突。
- **身份由宿主重新算。** 宿主不采用投递方给的 `identity`，而是用 `identity.url`（没有就用 `url`）重新调 `canonicalize` 得出身份；拿不到身份的（没有网址的；http(s) 链接马上就有身份，最差是这条链接的 `web:` 身份，离线时的短链也一样，之后可能并进平台 id，见 [§3.1](#31-运行器与节奏)），按无平台 ID 的条目处理（[SPEC §0.4](SPEC.md#04-id)）。不然一个持有令牌的脚本报出某个已有条目的身份、再带上 `completeness: full`，就能覆盖那个条目。

| 方法与路径 | 用途 | 返回 |
|---|---|---|
| `GET /v1/ping` | 检查令牌和连通 | `{"name":"usuyo","version":"…","protocol":1,"client":"xhs-vault 脚本"}` |
| `POST /v1/deliver` | 收下一条：只有链接或分享文字，或已取到的内容（已取到的内容 11-11 还没做；请求体只认 `url`、`text`，别的字段包括 `action` 都被忽略） | `202`，身份一算出就回条目 id（一般 3 秒内）；`canonicalize` 因网络失败或超时（例如离线时的短链）时，回这条链接的 `web-` id，联网后可能并进平台 id，旧 id 留作别名（[SPEC §0.4](SPEC.md#04-id)） |
| `POST /v1/batches` | 交一批链接，进导入预览 | `202`，批次 id；新增、已有的数目在导入预览里随身份算出逐步出现 |

只有链接时，宿主照 [§3.1](#31-运行器与节奏) 去取内容；`text` 是手机分享出来的整段文字，宿主从里面找出第一个链接：

```json
{"url": "https://v.douyin.com/iRNBho6u/", "action": "favorite"}
```

```json
{"text": "7.94 复制打开抖音，看看【咖啡日常的作品】手冲咖啡：三段式注水 https://v.douyin.com/iRNBho6u/ 12/30 abc:/"}
```

返回（不收的条目怎么回要改，见 [§13](#13-待定) 第 33 项；11-11 代码：私密条目被跳过时现在回的和「收下过」一样，`duplicate: true`，没有单独的 `skipped`）：

```json
{"id": "douyin-4plrcxk343onmyxibmrr672kga", "state": "accepted", "duplicate": false}
```

已经取到内容的（例如在页面里拿到了正文和图片），用 `multipart/form-data`（还没做，下面是设计）：`item` 部分是 fetch 输出 JSON；每个文件部分的**字段名**就是 JSON 里引用的相对路径，宿主按字段名找文件，`filename` 只作参考。原因：有的 multipart 库取文件名时只留最后一段（Go 的 `Part.FileName()` 会先过一遍 `filepath.Base`，11-05 用 Go 1.26 的 `go doc` 核对过），`images/001.jpg` 会变成 `001.jpg`；各家浏览器扩展对带斜杠的 `filename` 处理也不一致（推断）。字段名照 [§3.6](#36-fetch-与输出目录) 的输出目录规则校验，一次投递默认最多 1 GiB。`item` 按 fetch 输出的 schema 校验，只有 `evidence.access` 可以省略，宿主在快照里记成 `deliver`。

```
curl -X POST http://127.0.0.1:47615/v1/deliver \
  -H "Authorization: Bearer $USUYO_TOKEN" \
  -F 'item=@item.json;type=application/json' \
  -F 'images/001.jpg=@images/001.jpg'
```

一批链接（例如油猴脚本导出的小红书收藏夹清单）进导入预览，用户确认后按节奏去取：

```json
{"name": "小红书收藏夹 2026-11-01", "action": "favorite", "links": [
  {"url": "https://www.xiaohongshu.com/explore/671234567890abcdef012345?xsec_token=…", "title": "…", "actedAt": "2026-10-30T21:00:00+08:00"}
]}
```

<a id="52-监听文件夹"></a>
### 5.2 监听文件夹

不开网络端口的办法，适合手机经用户自己的云盘（iCloud 云盘等）投递，对应 PRD 的「自有云盘监听目录」。用户在设置里选一个收件文件夹，宿主处理里面的（11-11 代码：收件文件夹固定是按机器目录里的 `inbox/`，不能选，设置里还没有；只读顶层的 `.txt`，下表别的行都还没做）：

| 放进去的东西 | 当作 |
|---|---|
| `.txt`、`.url`、`.webloc` | 链接，`.txt` 里每个链接一条，也可以是整段分享文字 |
| `<名字>.usuyo/` 文件夹，里面一个 `item.json` 加它引用的文件 | 已取到的内容；`item.json` 最后写 |
| 文件名命中某个已装导入插件的 `files`（例如 WeChatBridge 的合并转发包） | 交给那个插件的 `list`（[§3.9](#39-从文件导入)）；多个插件都命中时，用用户排在最前的那个 |
| 其他文件（截图、PDF、视频） | 文件条目，`kind: file` 或 `image` |

- 一个条目 10 秒没有变化、里面没有云盘的占位文件，才开始处理。
- 收件文件夹里的条目照 [§3.6](#36-fetch-与输出目录) 的输出目录规则校验，例如放进来的符号链接一律不收；不合规的移到 `.usuyo-failed/`。
- 从收件文件夹来的导入，是直接收下，还是进导入预览等你确认：收件文件夹没人值守，放进去本身就是一次确认；默认哪种见 [§13](#13-待定) 第 15 项。
- `<名字>.usuyo/` 里的 `item.json` 和投递接口一样，身份由宿主重新算（[§5.1](#51-本机-http-接口)）；没有平台内容 ID 的（聊天记录、转发的图片）可以不写 `identity`（[§3.7](#37-fetch-输出)）。
- 处理完移到 `.usuyo-done/<日期>/`，失败的移到 `.usuyo-failed/`，旁边放一个 `.error.json`。
- 条目在 `origins[]` 里那一项的 `entry` 记 `mobile`，收件文件夹由用户指定入口类型时照指定的记；放进来的是已取到的内容（`<名字>.usuyo/`）或其他文件时，快照的 `plugin` 记 `deliver:inbox`，链接和交给导入插件的文件，快照照常记取它的那个插件。

从收件文件夹来的导入直接收下，处理完提示一次这次收下了多少条；设置里能改成先进导入预览。只有链接、要联网去取的条目照批量导入的节奏排队。

<a id="53-各种入口怎么接"></a>
### 5.3 各种入口怎么接

| 入口 | 走哪条 | `origins[].entry` |
|---|---|---|
| usuyo 自己的浏览器扩展「保存当前页」 | `POST /v1/deliver`，带正文和文件 | `extension` |
| 别人的扩展和油猴脚本（xhs-vault、Weibo-archiver 一类） | `POST /v1/batches` 交清单；或导出 JSON 文件，再用导入插件读（[§3.9](#39-从文件导入)） | `extension` 或 `import` |
| iOS 快捷指令 | 写进监听文件夹；或开了局域网直传时 `POST` 到电脑 | `mobile` |
| Android 分享（HTTP Shortcuts 一类） | 局域网直传 | `mobile` |
| Agent Skill（用户自己的 Agent 操作完把链接交过来） | `usuyo add <url>`，或用自己的 `api` 令牌 `POST /v1/deliver`；只交链接，内容由宿主照 [§3.1](#31-运行器与节奏) 去取 | `cli` 或 `api` |
| Raycast、Alfred | 优先调用 `usuyo add <url>`；`usuyo://add?url=<链接>` 只打开收下窗口并填好链接，用户点了「收下」才进队列，因为任何网页都能触发这个链接 | `cli` 或 `api` |
| 别的工具的产物（BiliNote 的 `note_results`） | 导入插件，或监听文件夹 | `import` |
| 照搬 qiaomu-clipper 一类做成的浏览器扩展（例如视频号在元宝页内取） | 扩展只能经 `POST /v1/deliver` 往宿主推内容，不在运行器的顺序里；手机分享过来的链接，宿主没法交给扩展去取。要让扩展也当获取方式，就给 `extension` 令牌加一个领任务的接口：例如 `GET /v1/jobs?platform=wechat-channels&wait=30` 长轮询领链接，交回时 `POST /v1/deliver` 带上 `jobId`。做不做见 [§13](#13-待定) 第 12 项 | `extension` |

**局域网直传**（还没做）是唯一听非回环地址的入口，默认关。打开后另起一个端口，手机扫电脑上的二维码拿到地址和这台手机专用的令牌。局域网上的明文 HTTP 会让同一网络里的人看到令牌和内容；手机端快捷指令对自签证书的支持待核实，所以默认推荐监听文件夹（默认走哪条见 [§13](#13-待定) 第 6 项）。

浏览器扩展不当获取方式、不领任务，只做「保存当前页」和投递；要在页面里取的平台做成带专用资料目录的插件，进运行器的顺序，手机分享来的链接也取得到。

<a id="6-凭据"></a>
## 6. 凭据

11-11 做到的见 [§3.13](#313-宿主和-plugin-kit-现在做到的11-11)：只有登录窗口这一种来源、只传 `credential: cookies` 的站点；`credentialsUpdate` 写回这次交出去的站点，存进 `usuyo.fetch.json` 前去掉。

- **只由宿主在调用时传入**，只传清单 `sites` 里声明、这个能力用到的站点；`login: none` 的能力不传任何平台账号凭据。插件自己的密钥设置不算平台凭据，另经 `USUYO_SECRETS` 传（[§2.11](#211-插件设置-settings)）。
- **访客身份不是账号凭据**（[§2.5.1](#251-访客身份-visitors)）：只传给列了它的能力，经 `USUYO_VISITOR`，文件和 `USUYO_CREDENTIALS` 同一个形状、同样 0600、调用结束即删；和同一主机的账号分开存、分开传，`credentialsUpdate` 不写回它。
- **这些限制靠沙箱的文件读权限落实。** 在文件读取只能「仅告知」的平台上（Windows，以及 Linux 上 bubblewrap 和 Landlock 都不可用时），任何插件都可能读到别的调用的凭据文件、别的站点的资料目录和日常浏览器的 cookie 库，这几条只是约定，不是隔离；界面的权限页按 [§7](#7-权限与隔离) 的三档如实标出。
- **形式**：`USUYO_CREDENTIALS` 指向一个临时 JSON 文件（权限 0600，放在这次调用的临时目录里，调用结束即删除）；`credential: cookies` 时另有 `USUYO_COOKIES_TXT`，内容相同、格式是 Netscape cookies.txt。常驻模式下路径放在每个请求的 `context` 里。

```json
{
  "sites": [{"site": "www.douyin.com", "source": "daily-browser"}],
  "storageState": {
    "cookies": [{"name": "sessionid", "value": "…", "domain": ".douyin.com", "path": "/", "expires": 1793000000, "httpOnly": true, "secure": true, "sameSite": "Lax"}],
    "origins": [{"origin": "https://www.douyin.com", "localStorage": [{"name": "…", "value": "…"}]}]
  }
}
```

`storageState` 用 Playwright storageState 的形状，Playwright、chromedp 都能直接装进浏览器。只传和 `site` 做域名匹配的 cookie（站点本身和它的上级域名，不含公共后缀），加上 `domains` 显式列出的（[§2.5](#25-登录站点-sites)）。

- **插件不自己读浏览器 cookie 库，不持久保存凭据**：不写进 `${CACHE}` 或 `${HOME}`，不写日志，不发到声明以外的主机。
- **会话刷新。** 插件不主动调平台的会话刷新接口，被包装的工具有自动刷新的要关掉；有的平台刷新后旧 cookie 就失效，例如 B站 的 Web 端 Cookie 刷新（推断，待核实），刷新了又被宿主丢掉，用户存的登录态就没用了。调用期间平台自己换了会话 cookie 时，信封可以带 `credentialsUpdate`（`storageState` 的形状，只含声明站点的 cookie），宿主只写回专用资料目录或导入的 cookie 文件，不写回用户的日常浏览器。`USUYO_COOKIES_TXT` 是可写的（yt-dlp 会把 cookie 写回 `--cookies` 指向的文件），调用结束后丢弃，宿主不从它读回。
- **凭据从哪来，用户按站点选**（[06 §11](research/06-platform-access-and-account-risk.md#11-agent-接入层)；默认是 usuyo 的登录窗口，日常浏览器的登录状态要用户给这个站点选了才读，[PRD 7.2](PRD.md#72-获取插件)）：

| 来源 | 宿主怎么做 | 已知问题 |
|---|---|---|
| 复用日常浏览器 | 调用时从用户选的浏览器读这个站点的 cookie，宿主不另外保存，只写进这次调用的凭据文件，调用结束即删；macOS 上首次读 Chrome 要用户在钥匙串弹窗里允许 | 浏览器和插件同机同 IP，最接近用户平时的样子；`browser-profile` 类插件用不了日常资料目录（Chrome 136 起自动化工具连不上默认资料目录），宿主把 cookie 装进专用资料目录时，可能被判为同一账号出现在另一套指纹上 |
| 专用资料目录 | 宿主为这个站点建一个浏览器资料目录，用普通模式的 Chrome 打开 `loginUrl` 让用户登录，按 `sites.cookies` 判断登录完成，再经 CDP 导出 cookie：CDP 走 `--remote-debugging-pipe`，不开 TCP 端口（开了的话本机任何进程都能连上去接管会话），导出后立即关闭这个浏览器；`browser-profile` 类插件直接拿到这个目录 | 要单独登录一次；同一目录同一时间只给一个进程用，宿主加锁（Easel 的资料目录锁），交接见下 |
| cookie 文件 | 用户导入 Get-cookies.txt-LOCALLY 一类扩展导出的文件 | 过期要用户重新导出 |

- **资料目录按站点建，同一站点的插件共用**，用户每个站点只登录一次。锁由用到它的进程持有：一次一进程的调用到进程结束；常驻进程从第一次用到起，调用之间可以让浏览器开着。宿主自己导出 cookie 也要拿这把锁。运行器换到需要同一目录的下一个插件（或宿主自己要用）时，先让持有锁的常驻进程退出：关闭 stdin，5 秒后杀进程组；最多等锁 10 秒，等不到就给这个插件记宿主错误 `profile-busy`，换下一个。每次把目录交给插件前，宿主检查并清掉目录里新出现的浏览器扩展，免得一个插件留下的扩展影响下一个插件和用户在这个目录里的登录。
- 首次登录不在自动化控制的浏览器里做：有的站点不接受在 Playwright 控制的 Chrome 里登录，macOS 上自动化启动的 Chrome 也读不到钥匙串加密的 cookie（kali-publish 的记录，见 [06 §3](research/06-platform-access-and-account-risk.md#3-用户点名的项目)）。
- `credential: app` 的站点（驱动已登录的微信客户端、手机 App），宿主什么都不传；界面照样写「会用到你在微信里登录的账号」。`site` 写这个平台账号的站点（例如 `weixin.qq.com`、`www.douyin.com`），和浏览器类插件写同一个 `site` 没关系，`credential` 不同；登录状态由插件在 `health` 的 `login[]` 里报（[§3.10](#310-health)）。

<a id="7-权限与隔离"></a>
## 7. 权限与隔离

11-11 还没做：沙箱和宿主的本机代理都没有；插件的进程放在自己的进程组里，环境照 [§3.13](#313-宿主和-plugin-kit-现在做到的11-11)。代理做出来时，plugin-kit 的客户端要能经宿主的代理出站（另用一个专门的变量，不读 `HTTP_PROXY`），地址检查跟着挪到宿主一侧。

沿用 [ADR-003 §4](decisions/ADR-003-plugins.md#4-权限与隔离)：macOS 用 Seatbelt，Linux 用 bubblewrap（不可用时 Landlock），Windows 先只强制 Job Object 与环境变量；超时杀进程组；`allowlist` 网络只经宿主代理出站；界面按平台、按声明项显示「强制 / 部分 / 仅告知」，不一律写「已沙箱」。各声明项在各平台的落实程度见 [05 §4.5](research/05-note-apps-and-plugins.md#45-权限声明与各平台落实程度)。

补设备类：

| `devices` | macOS | Linux | Windows |
|---|---|---|---|
| `adb` | 部分：插件只连宿主管的 adb 服务端口，不自己起 adb 服务 | 部分：同左 | 仅告知 |
| `ios-device` | 待核实：能否用 Seatbelt 只对声明的插件放行 usbmux 套接字 | 部分：bind usbmux 套接字 | 仅告知 |
| `screen-capture` | 仅告知 | 仅告知 | 仅告知 |
| `accessibility` | 仅告知 | 仅告知 | 仅告知 |
| `audio-capture` | 仅告知 | 仅告知 | 仅告知 |
| `clipboard` | 待核实：能否用 Seatbelt 拦下未声明插件的剪贴板服务 | 仅告知 | 仅告知 |

- macOS 的屏幕录制、辅助功能、系统录音权限是授给 usuyo 本身的。子进程按「负责进程」用这些授权：第三方工具 agent-desktop 的权限引导代码和故障文档都说授权记在启动它的 App 上，也说系统设置有时会单独列出子程序、要一起打开（第三方旁证，没有查到 Apple 文档）。所以 usuyo 一旦拿到，未声明的插件也可能用得上；不做按插件分开授权（[§11](#11-设计取舍)），先按「仅告知」显示。
- **adb 服务由宿主管。** 宿主自己起一个 adb 服务，用私有端口；这个服务进程的 `HOME` 指向本机配置目录下的一个子目录，让 adb 的密钥（`$HOME/.android/adbkey`）固定下来。不这样的话，插件的私有 `${HOME}` 每次调用都会生成新密钥，手机每次都要重新点授权（推断）。adb 密钥算凭据，放本机配置目录、权限 0600，不放 `${CACHE}`。插件只拿到 `ADB_SERVER_SOCKET` 和 `ANDROID_SERIAL`（[§3.2](#32-调用约定)），沙箱只放行这个端口；无线调试的配对和连接在宿主界面里做，插件拿不到手机的局域网地址。连着几台设备时，插件经同一个服务仍能点名别的设备，所以是「部分」。插件用 scrcpy 时，宿主给它 `ADB=<解析出的 adb>`，免得两个版本的 adb 互相重启服务端（推断，待核实）。
- 用到 `devices` 的插件，安装时在权限页单独列出这几项；第一次真正用到时，宿主先请用户在系统设置里授权 usuyo。
- 剪贴板：Apple 给通用剪贴板加了「程序读之前先问」（[NSPasteboard.AccessBehavior](https://developer.apple.com/documentation/appkit/nspasteboard/accessbehavior-swift.enum)，macOS 26 上还是开发者预览），以后默认开不开待核实；开了以后弹窗大概率记在 usuyo 上（推断，和上一条同理），宿主在第一次用到 `clipboard` 时先跟用户说清楚：跑的时候会动剪贴板，跑完写回原来的内容。
- 声明了 `accessibility`、`screen-capture`、`audio-capture`、`clipboard` 的插件要连系统服务（推断）。宿主在这个插件的沙箱里放行对应的系统服务，具体放行哪些规则待核实，实现时先在 Seatbelt profile 里跑一次 agent-desktop 的只读快照试出来；做不到的话，这几项在 macOS 上不进 Seatbelt，界面显示「仅告知」。要紧的是声明了的插件不被挡住，而不是拦下未声明的插件。
- 宿主不代用户安装根证书、不向别的进程注入；需要这些的插件让用户自己装、自己负责。

<a id="71-常驻进程的路径"></a>
### 7.1 常驻进程的路径

Seatbelt 的 profile、bubblewrap 的 bind 和 Landlock 的规则都在进程启动时定下，之后不能再放行新路径。常驻的转换、导出进程和 `serve` 模式的获取进程每次调用拿到的却是新的条目路径、`out` 目录和凭据文件。为此宿主给每个常驻进程建一个暂存目录：

```
<tmp>/usuyo-proc-<id>/
  in/<callId>/<下标>/<文件名>  输入：条目文件、导入文件的副本；进程只读
  out/<callId>/                这次调用的输出目录；进程可写
  cred/<callId>/               凭据、cookies.txt、密钥、设置文件；进程可读写
  home/                        ${HOME}
  bin/                         运行时的符号链接或转发脚本
  tmp/                         ${TMP}，也是进程的工作目录
```

`bin/` 的做法见 [§2.13](#213-运行时解析)。

- 进程启动时，沙箱只放行这个暂存目录，加上已声明的 `${CACHE}`、`${MODELS}`，以及这个进程的站点集合里 `browser-profile` 站点的 `${PROFILE}`（[§3.12](#312-常驻模式-serve)）；`readback: true` 的导出插件另放行目标文件夹 `${TARGET}`，只读，给 `readback` 用（规则启动时就定下，所以整个进程都看得到；`run` 照 [§4.4](#44-导出的-run-与-readback) 不碰它）。不放行整个 `items/` 或临时目录的上级，插件拿不到资料库根目录（[§1](#1-概念)）。目标文件夹是资料库、在资料库里或包含资料库时，宿主不放行，导出设置里也选不了（推断）。
- 每次调用，宿主把输入放进 `in/<callId>/<下标>/`，文件名照原样：先在同一文件系统上建硬链接（`in/` 对进程只读，改不到原文件），跨文件系统时复制。插件在 `out/<callId>/` 旁边的 `in/<callId>/` 找到这次的输入。转换进程的 `<callId>` 就是 `run` 的 JSON-RPC `id`。没有沙箱时（11-11 还没有，[§7](#7-权限与隔离)），硬链接进 `in/` 挡不住插件改到原文件：原样被改过靠 `usuyo.fetch.json` 里的哈希查出来（同步前和「检查与修复」时，[SPEC §1.2.2](SPEC.md#122-raw-和-derived)），不靠挡；有沙箱以后照这一条；所以宿主尽量把暂存目录建在和资料库同一个文件系统上。新建 `out/<callId>/`，把凭据写进 `cred/<callId>/`。`run`、`fetch` 参数和 `context` 里的路径都指向暂存目录。
- 调用结束后，宿主删掉 `in/<callId>/` 和 `cred/<callId>/`，`out/<callId>/` 照 [§3.6](#36-fetch-与输出目录) 检查后移走。进程退出时整个暂存目录删除。
- 一个常驻进程能看到它自己正在进行的各次调用的暂存内容，看不到别的进程的。宿主同一时间只向一个进程发一个请求，所以实际只有一份。
- 一次一进程的调用不需要这一层：启动时就知道这次的路径，`${TMP}` 下的布局照此类推。

<a id="8-打包发布与分发"></a>
## 8. 打包、发布与分发

<a id="81-仓库命名"></a>
### 8.1 仓库命名

| 来源 | 仓库名 | 例子 |
|---|---|---|
| 官方组织 `usuyo` | 应用 `usuyo`、`plugin-kit`、转换与导出插件 `<kind>-<name>`，不加 `usuyo-` 前缀 | `usuyo/transform-asr-sherpa`、`usuyo/export-obsidian` |
| 获取方式组织 `usuyo-fetch` | 只放获取插件，`<platform>` 或 `<platform>-<取法>`，不加前缀 | `usuyo-fetch/douyin`、`usuyo-fetch/xiaohongshu-share`、`usuyo-fetch/takeout` |
| 个人仓库 | `usuyo-fetch-<name>`、`usuyo-transform-<name>`、`usuyo-export-<name>` | `someone/usuyo-fetch-douyin` |

所有插件仓库都加 GitHub topic `usuyo-plugin`。插件的 `id` 就是仓库的 `owner/repo`。

<a id="82-发版与-release-资产"></a>
### 8.2 发版与 release 资产

一个 release 里必须有：

| 资产 | 内容 |
|---|---|
| `usuyo-plugin.yaml` | 补好 `install` 的清单 |
| 插件本体，名字以清单里的 `install.assets[].name` 为准；建议写成 `<repo>_<version>_<os>-<arch>.tar.gz`（Windows 用 `.zip`）或 `<repo>_<version>_any.tar.gz` | 插件本体 |
| `provenance.sigstore.json` | 构建来源证明，覆盖上面所有资产；官方组织必须有，`usuyo-fetch` 自己约定都有，个人仓库建议有 |

- tag 写 `v<version>`；仓库开 GitHub immutable releases：发布后资产不能改、tag 不能移（2025-10-28 起正式可用，组织级可以统一打开）。所以要先建草稿、传齐资产、再发布。
- 用 plugin-kit 的可复用发版工作流：校验源清单、核对 tag 与 `version`、算 sha256 写进 `install`、生成来源证明、建草稿、上传、发布。插件仓库只写构建这一段：

```yaml
name: release
on:
  push:
    tags: ["v*"]
permissions:
  contents: read
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@<完整 commit SHA>
      - run: make dist
      - uses: actions/upload-artifact@<完整 commit SHA>
        with: {name: dist, path: dist/}
  release:
    needs: build
    permissions:
      contents: write
      id-token: write
      attestations: write
    uses: "usuyo/plugin-kit/.github/workflows/release.yml@<完整 commit SHA>"
```

所有 action 和可复用工作流都按完整 commit SHA 引用，由 Dependabot 升级。`usuyo-fetch` 是引用这个工作流，还是复制一份放进自己组织，见 [§8.8](#88-获取方式组织的仓库结构) 和 [§13](#13-待定) 第 13 项。

不用这个工作流的（GoReleaser 或手工发版），用 `usuyo plugin pack --version 0.3.1 dist/*`：校验源清单、算 sha256，写出补好 `install` 的 `usuyo-plugin.yaml`，由作者自己连同资产上传。

<a id="83-校验与来源证明"></a>
### 8.3 校验与来源证明

客户端安装或更新一个版本，依次做：

0. **认准仓库。** 安装时记下 GitHub 的数字仓库 id 和所有者 id：来源是组织时从 `GET /orgs/{org}/repos` 的列表结果取，单独添加 `owner/repo` 时调一次 `GET /repos/{o}/{r}`。更新时，有来源证明的比对证书里的 Source Repository Identifier 和 Source Repository Owner Identifier；没有来源证明的，在检查更新时比对列表结果（单个仓库调一次 `GET /repos/{o}/{r}`）。下载清单时不跟随指向别的 `owner/repo` 的重定向。名字或数字 id 对不上时（仓库改名、转移，或旧名字被别人重新注册），标「来源已变：现在是 <新名字>」，停止更新，既不拒装也不自动装，让用户重新添加来源。新版本的 `version` 不高于已装版本时不更新，作者把旧版本标成 latest 也不会让客户端退回去。
1. 查屏蔽清单（[§8.6](#86-屏蔽清单与下架)），命中就拒装。
2. 下载 release 资产 `usuyo-plugin.yaml`，按 schema 校验；`id` 等于仓库、`version` 等于 tag，否则拒装。
3. 下载本机平台的资产，sha256 和清单不一致就拒装（[ADR-003 §6](decisions/ADR-003-plugins.md#6-分发与校验)）。
4. 有 `provenance.sigstore.json` 时，用 Sigstore 的客户端库验证（原写 sigstore-go；内核 11-11 换成 Rust，用哪个库实现时定），不经 GitHub API（Sigstore 的信任根由客户端按 TUF 定期更新）：签名有效、证书里的源仓库是这个仓库、源 ref 是这个 tag。签发者（构建它的工作流，按仓库和路径，不含 ref）的要求分两种：
   - **官方组织**：客户端内置，签发者必须是 `usuyo/plugin-kit` 的发版工作流；没有来源证明或验证失败就拒装。
   - **用户添加的来源**（包括 `usuyo-fetch`）：添加时固定签发者。[附录 A.4](#a4-添加第三方来源时的确认) 的确认框显示这个来源现有插件的签发工作流，例如「由 `usuyo/plugin-kit` 的 `release.yml` 构建」，或「没有来源证明」；用户添加后客户端记下。以后这个来源里的新版本，签发工作流和记下的不同，或者原来有来源证明现在没有，就不更新并标出来。添加时记下的是「没有来源证明」的，不做这项比对，只逐个版本显示有没有。验证失败一律拒装。这和 Mihon 的做法同一个思路：信任来自用户添加 store 时接受的签名指纹（[02 §15.7](research/02-adapter-ecosystems.md#157-适配器源信任与治理)），客户端不用认识 `usuyo-fetch`。
5. 读 GitHub App 写在 tag 提交上的 check run，显示「校验通过 / 没通过 / 没有校验」（[ADR-003 §6](decisions/ADR-003-plugins.md#6-分发与校验)）。只认 `app.id` 等于 usuyo GitHub App 的 check run，客户端内置这个 id：任何人都能用 Actions 的 `GITHUB_TOKEN` 或自己的 App 写一个同名、结论为 success 的 check run。官方组织的插件没通过就拒装；读不到 check run 时（App 还没跑完、被限额挡住）推迟这次更新，下次检查时再读，不安装。第三方插件读不到时显示「没有校验」；没通过时只标出，安装前多一步确认。
6. 展示权限，用户同意后安装（[§8.5](#85-添加来源更新与权限扩大)）。权限页列出每个站点的 `site`、`credential` 和 `domains`，以及每个密钥设置会被注入到哪个 `inject.host`。

分工：sha256 防下载途中被换；immutable releases 防发布后被换；来源证明说明资产是这个仓库在这个 tag 上由哪个工作流构建的，替代自己管签名密钥（Mihon 用 store 的签名指纹，[02 §15.7](research/02-adapter-ecosystems.md#157-适配器源信任与治理)）；GitHub App 的 check run 说明清单和契约测试过没过。界面上把 [PRD §8.1](PRD.md#711-设置) 里的「显示签名指纹」换成「由 `<owner>/<repo>` 的发版工作流构建」。

GitHub App 的检查加三条：`env` 里有凭据类变量判失败；`access` 或 `login` 比上一版高时写进检查结果，供更新说明显示；被包装工具里有发帖、点赞、私信一类命令时只作提示。App 不执行插件，具体分工见 [§10](#10-契约测试与-plugin-kit)。

第三方插件校验没通过时只标出，安装前多一步确认；屏蔽清单命中的照旧拒装。

<a id="84-三种来源"></a>
### 8.4 三种来源

| 来源 | 客户端里怎么来 | 标签 |
|---|---|---|
| 官方组织 `usuyo` | 客户端内置，列出组织里带 topic `usuyo-plugin` 的仓库；认 `kind: transform`、`export`，`kind: fetch` 只认下面说的两类预装插件，别的不列 | 官方 |
| 用户添加的组织，包括 `usuyo-fetch` | 用户在设置里输入组织名；列出组织里带 topic 的仓库 | 第三方来源 · 组织名 |
| 用户添加的单个仓库 | 用户输入 `owner/repo` | 第三方 · 未经 usuyo 审核 |

- 随安装包预装两类不含任何平台知识的获取插件（2026-11-01 用户同意，[PRD P1、P3](PRD.md#72-获取插件)）：通用网页获取（只做 `public-web`、不登录、没有站点专用规则，`network.hosts` 写 `"*"`，宿主代理照样挡住内网和云元数据地址），和读别的工具导出文件的导入（Netscape HTML 书签、Pocket、Omnivore、Readwise、Instapaper、Wallabag、Obsidian Web Clipper、flomo 一类，`network: none`）。它们的源码在官方组织，`id` 用 `builtin/` 前缀，跟着客户端一起发版和更新。平台官方导出包（抖音个人信息下载、Google Takeout、X 归档一类）的导入插件仍放 `usuyo-fetch`。
- 除了这两类，客户端不预置任何获取来源，`usuyo-fetch` 也要用户手动添加，代码里不对它特殊处理；对它的要求和别的用户来源一样，靠添加时固定的签发者（[§8.3](#83-校验与来源证明) 第 4 步）。
- 浏览第三方插件靠 GitHub 搜索 topic `usuyo-plugin`，客户端里给一个打开搜索结果的链接，不做商店。
- 每个平台的插件列表显示：名字、`access`、会用到哪个账号（`sites[].name`，没有就写「不用账号」）、来源标签、校验结果、来源证明、最近一次失败的原因。

插件列表里 `access` 显示一句白话，英文标识放进详情：official-export「读你导出的文件」，public-web「不登录，读公开页面」，vision-ui「看屏幕，替你点 App」，browser-session「在登录着的浏览器页面里读」，session-api「带你的登录信息直接请求平台」，private-api「模仿平台客户端发请求」，client-hook「读取或改动平台客户端本身」。

<a id="85-添加来源更新与权限扩大"></a>
### 8.5 添加来源、更新与权限扩大

- **添加第三方来源时确认一次声明**（文案见 [附录 A.4](#a4-添加第三方来源时的确认)），之后装这个来源里的插件不再逐个弹窗。每个来源只确认一次。
- **安装**：展示权限，用户按资料库同意（[ADR-003 §4](decisions/ADR-003-plugins.md#4-权限与隔离)）。
- **更新**：沿用「自动更新获取方式」的设置。新版本先过 [§8.3](#83-校验与来源证明) 的校验，再和已装版本比：
  - **扩大了权限**的，这个版本先停用，等用户重新同意，不静默更新。按集合和已装版本比，出现已装版本没有的值就算：`fs` 多了占位符；`network` 从 `none` 变 `allowlist`，或多了主机（包括 `"*"`）；`exec` 多了程序（包括 `requires.runtime` 新增的名字）；`env` 多了变量；`gpu` 从关到开；`devices` 多了项；`sites` 的 (`site`, `credential`, `domains`) 三元组里出现新值，例如多了站点、已有站点加了 `domains`、`credential` 从 `cookies` 改成 `browser-profile`；`settings` 里多了 `secret: true` 的键，或已有密钥的 `inject.host`、`inject.header` 改了。这几种改动都会把凭据或密钥交到新的地方：在「复用日常浏览器」下，多一项 `domains` 等于宿主替插件从日常浏览器多读一批 cookie；`browser-profile` 交出的是整个可写的资料目录。这是防经自动更新下发恶意代码的安全规则。
  - **调高了 `access` 或 `login`**、新增了 `unfavorite` 的，照常更新，只在更新说明里标出；`unfavorite` 仍默认关。
- **只从来源更新**：插件只从安装它的那个仓库更新（Mihon v0.21 的做法）。仓库改名、转移或被别人重新注册时怎么办，见 [§8.3](#83-校验与来源证明) 第 0 步。

自动更新默认开，不自动降级；用户可以在插件详情里手动回到上一版。声明带版本号：已添加的来源不重新确认；声明有实质变化时，只在下一次添加新来源时确认新版，并在应用的更新说明里写一句。

<a id="86-屏蔽清单与下架"></a>
### 8.6 屏蔽清单与下架

- 屏蔽清单 `security/plugin-blocklist.json` 放在主仓库，只列恶意的仓库和版本（[ADR-003 §6](decisions/ADR-003-plugins.md#6-分发与校验)）。客户端在启动、安装、更新前读取并缓存；命中的拒装，已装的停用。
- 地址 `https://raw.githubusercontent.com/usuyo/usuyo/main/security/plugin-blocklist.json`，带 ETag 缓存，最多每 24 小时刷新一次。读不到时（离线、GitHub 不可达）用缓存；连缓存也没有时照常启动，安装前提示「屏蔽清单读不到」。
- `versions` 用 semver 范围语法（同 npm 的 node-semver），省略表示所有版本。条目带 `repoId`，按数字仓库 id 匹配，`repo` 只供人看（[§8.3](#83-校验与来源证明) 第 0 步）。
- 命中已装版本时，宿主结束它的常驻进程，取消它排队中的调用，再停用。

```json
{
  "updatedAt": "2026-11-01T00:00:00Z",
  "entries": [
    {"repo": "someone/usuyo-fetch-example", "repoId": 912345678, "versions": "<=0.2.3", "reason": "版本里带了上传 cookie 的代码", "at": "2026-10-31"}
  ]
}
```

- 来源里已经没有的插件（仓库被下架、移出组织、删除），客户端标「来源里已没有」，不再更新，不自动卸载；已经存下的快照不受影响（Mihon 的 orphaned）。
- 获取方式组织下架一个插件后，记进置顶 issue「已下架的获取方式」，写明原因和日期。因权利人通知下架的不再加回来；因维护原因下架的写明原因，以后可以重新提交（Keiyoushi 的置顶 issue 就是这样分的，[02 §1](research/02-adapter-ecosystems.md#1-tachiyomi-与-mihon-扩展生态)）。

<a id="87-github-api-限额与缓存"></a>
### 8.7 GitHub API 限额与缓存

不登录时 REST API 每个 IP 每小时 60 次；带 ETag 的条件请求回 304 时不计次，但只对带 `Authorization` 的请求成立。所以客户端尽量少用 API：

| 要读的 | 怎么读 | 计不计 API 次数 |
|---|---|---|
| 组织里的插件仓库 | `GET /orgs/{org}/repos?per_page=100`，结果里带 `topics`，客户端自己筛 | 计，每个组织每 100 个仓库一次 |
| 最新版本的清单 | `https://github.com/<owner>/<repo>/releases/latest/download/usuyo-plugin.yaml`，走网页下载的重定向 | 不计 API |
| 资产和来源证明 | 清单里的资产名拼 `releases/download/v<version>/<name>` | 不计 API |
| check run | `GET /repos/{o}/{r}/commits/{tag}/check-runs` | 计；不登录能不能读到待核实（ADR-003 待核实） |
| 单独添加的仓库的数字 id | `GET /repos/{o}/{r}`，安装时和检查更新时各一次（[§8.3](#83-校验与来源证明) 第 0 步） | 计 |

- 组织列表缓存 24 小时；清单按版本缓存，immutable release 的同一版本永不过期；检查更新最多每 6 小时一次，另有手动刷新。
- 可选：用户在设置里填一个只读公开仓库的 GitHub 令牌，限额升到每小时 5000 次。不要求登录 GitHub。

<a id="88-获取方式组织的仓库结构"></a>
### 8.8 获取方式组织的仓库结构

选**每个插件一个仓库**，不学 keiyoushi/extensions-source 的大仓库。

| 方面 | 每个插件一个仓库 | 一个大仓库 |
|---|---|---|
| 下架粒度 | 投诉或律师函只影响那一个仓库 | 一个插件被投诉，处理对象是整个仓库；GitHub 的 DMCA 流程给所有者改动的机会，改不了就禁用仓库（细节待核实） |
| 发现与发布 | 客户端按 topic 列仓库、读各自的最新 release，不要索引文件 | 要按插件打 tag，还要一份索引告诉客户端有哪些插件，这正是 ADR-003 放弃的东西 |
| 共享代码 | 平台中立的放进 plugin-kit；同一平台不同取法之间的少量重复可以接受 | 放 `lib/` 里直接共享，Keiyoushi 的 `lib-multisrc` |
| CI | 每个仓库一行引用 plugin-kit 的可复用工作流；组织级规则统一保护分支和 tag；升级靠 Dependabot 批量提 PR | 一处配置；但每次改动要做路径过滤，一个插件的构建坏了会挡住别的插件 |
| 维护权限 | 每个仓库一个 team，可以单独交给某个维护者 | 要靠 CODEOWNERS 细分 |

代价是仓库多了以后，统一的改动要批量提 PR；可复用工作流把这件事压到改一个 SHA。

改造上游项目时，新建仓库再推送它的历史（保留许可证和版权声明），不用 GitHub 的 fork 按钮：fork 会进上游的派生网络。派生超过 100 个、且通知方主张派生同样侵权时，GitHub 按整个派生网络处理，被屏蔽的仓库返回 451（[06 §16.3](research/06-platform-access-and-account-risk.md#163-下架停更与法律风险)）；派生不到 100 个时怎么处理，待核实。

**和 Keiyoushi 的不同。** Keiyoushi 用自己的 CI 和自己的签名密钥构建、发布，Mihon 不拥有、也不关联任何扩展仓库（[02 §1](research/02-adapter-ecosystems.md#1-tachiyomi-与-mihon-扩展生态)）。本文现在的写法里，`usuyo-fetch` 的发版引用官方组织 `usuyo/plugin-kit` 的可复用工作流，来源证明里的签发者因此是官方组织的工作流，官方组织进了获取插件的发版链；usuyo 的 GitHub App 装在 `usuyo-fetch` 上只做读和校验，和任何第三方作者装它一样，不参与发版。另一种做法是把工作流复制进 `usuyo-fetch/.github`，签发者变成 `usuyo-fetch` 自己的工作流，代价是 plugin-kit 改了工作流要再同步一次。选哪种见 [§13](#13-待定) 第 13 项。两个组织的 owner 目前是同一个人，这一点两种做法都改变不了（[§8.9](#89-组织设置清单)）。

<a id="89-组织设置清单"></a>
### 8.9 组织设置清单

两个组织都要设，标「获取」的只在 `usuyo-fetch`：

- [ ] `.github` 仓库的 `profile/README.md`：组织主页。`usuyo-fetch` 用 [附录 A.2](#a2-获取方式组织主页)。
- [ ] `.github` 仓库放默认的社区文件：`SECURITY.md`（漏洞走 GitHub 私密漏洞报告）、`CONTRIBUTING.md`（清单写法、`maintainers` 要本人同意、不收写操作和推广内容）、issue 模板（插件坏了、请求新平台；获取：权利人通知）、PR 模板。
- [ ] 获取：置顶 issue「已下架的获取方式」。
- [ ] 成员必须开双重验证；基础权限只读；只有 owner 能建仓库、转入仓库、改可见性。
- [ ] 组织级规则：`main` 禁止强推和删除、合并要走 PR；`v*` tag 只有发版的人能建，不能删也不能移。
- [ ] 组织级打开 immutable releases。
- [ ] Actions：只允许 GitHub 官方的 action 和发版用的可复用工作流（`usuyo/plugin-kit` 的，或按 [§13](#13-待定) 第 13 项复制进 `usuyo-fetch/.github` 的），按完整 SHA 引用；`GITHUB_TOKEN` 默认只读；只有发版工作流拿 `contents: write`、`id-token: write`、`attestations: write`。
- [ ] 谁能发版：每个仓库一个 team，名单和清单的 `maintainers` 一致；能推 `v*` tag 的就是这个 team 加 owner。维护者多于一人后，发版用的 environment 要求另一人批准。
- [ ] 装上 usuyo 的 GitHub App（公开的 App，只读和写 check run，不执行插件）。
- [ ] 打开 secret scanning、push protection 和 Dependabot。
- [ ] 联系方式：权利人通知写信到 `{contact}`；两个组织用不用同一个地址、用不用你域名下的地址，见 [§13](#13-待定) 第 1 项。
- [ ] 两个组织的 owner 目前是同一个人，隔离只到组织和仓库这一层（[06 §17.4](research/06-platform-access-and-account-risk.md#174-fork-与许可证)）。声明里凡是说谁在维护的句子，都按组织的实际成员如实写（[附录 A](#附录-a-声明模板)）。

`usuyo-fetch` 不用用户的域名：不放文档、不用 `$id`、不链个人主页，README 只链官方仓库里的协议文档；插件清单照样引用官方 schema。

<a id="9-适配现有工具"></a>
## 9. 适配现有工具

上游只有 HTTP 接口或 MCP 的，插件里写一层薄的命令行转接，对宿主仍是 [§3](#3-获取插件协议) 的子命令。本节的清单片段只列和包装方式有关的字段，完整清单照 [§2](#2-清单) 补齐。

<a id="91-包装命令行"></a>
### 9.1 包装命令行

例子：不登录取 YouTube 单条，包装用户自装的 yt-dlp。按 [06 §17.3](research/06-platform-access-and-account-risk.md#173-首批自己适配的插件)，yt-dlp 在 YouTube 上记 `private-api`，不传 cookie 就没有账号风险。

```yaml
id: someone/usuyo-fetch-youtube
kind: fetch
platforms: [youtube]
match: ["www.youtube.com/watch*", "youtu.be/*", "www.youtube.com/shorts/*"]
access: private-api
login: none
capabilities:
  fetch:
    access: private-api
    login: none
    outputs: [text, media:video, subtitle, meta]
requires:
  runtime: ["yt-dlp>=2026.10", "deno>=2", "ffmpeg"]
permissions:
  fs:
    write: ["${OUT}"]
  network:
    mode: allowlist
    hosts: ["*.youtube.com", "*.googlevideo.com", "*.ytimg.com", "youtu.be"]
```

`exec` 没写，缺省等于 `requires.runtime` 里的 yt-dlp、deno、ffmpeg（[§2.7](#27-权限-permissions)）：yt-dlp 解 YouTube 要起 deno，B站、YouTube 的高画质音视频分开下，要 ffmpeg 合并。

插件的 `fetch` 只调一次 yt-dlp，同一条内容不向平台解析两遍：

```
yt-dlp --ignore-config --no-playlist --no-progress --write-info-json --write-subs --sub-langs "zh.*,en.*" -o "$OUT/media/%(id)s.%(ext)s" -- "$URL"
```

- `--ignore-config`：私有 `HOME` 挡不住 `/etc/yt-dlp.conf` 这类系统级配置，要显式不读。
- `--`：链接前加上，免得以 `-` 开头的链接被当成选项（[06 §3](research/06-platform-access-and-account-risk.md#3-用户点名的项目) 引的 hypit #452）。
- 元数据从写出的 `.info.json` 读：填 `identity`、`title`、`author`、`publishedAt`，`media` 列出视频和字幕，`evidence.upstream` 写 yt-dlp 的版本。
- 不要用 `--cookies-from-browser`；这个插件是 `login: none`。有登录的插件用 `--cookies "$USUYO_COOKIES_TXT"`。
- yt-dlp 自己的输出不要转到插件的 stdout；要进度就解析它的输出，转成 `@usuyo` 的 `progress` 事件。下大视频时在能力里把 `timeout` 设长（[§3.4](#34-超时与取消)）。

另一种做法是把上游当库编进插件：parse-video 是 Go、MIT，可以直接调它的 `parser.ParseVideoShareUrl` 做抖音的 `public-web` 单条；不给它设 cookie，设了它会改调自算签名的接口（[06 §1.3](research/06-platform-access-and-account-risk.md#13-混合路径视觉层拿链接公开页面取内容)）。

<a id="92-包装本机-http-服务"></a>
### 9.2 包装本机 HTTP 服务

例子：小红书单条，调用户自己跑的 XHS-Downloader 服务（它用 `python main.py api` 启动，文档在 `http://127.0.0.1:5556/docs`，接口 `POST /xhs/detail`，要 `Authorization: Bearer` 令牌）。XHS-Downloader 是 GPL-3.0，只能用户自装，插件经 HTTP 调用。

```yaml
settings:
  - {key: endpoint, label: XHS-Downloader 服务地址, type: string, default: "http://127.0.0.1:5556"}
  - {key: xhs-token, label: XHS-Downloader 访问令牌, type: string, secret: true}
capabilities:
  fetch:
    access: public-web
    login: none
    outputs: [text, images, media:video, meta]
permissions:
  network:
    mode: allowlist
    hosts: ["${setting.endpoint}", "*.xhscdn.com"]
  fs:
    write: ["${OUT}"]
```

- `fetch` 把链接（带 `xsec_token` 的原链接，[§3.6](#36-fetch-与输出目录)）交给服务，请求体固定写 `download: false`（不让服务下载）和 `check_record: false`（它默认是 `true`，会跳过有下载记录的作品）。拿到元数据和文件地址后，由插件自己把文件写进 `--out`。
- 访问令牌是 `secret: true` 的设置，经 `USUYO_SECRETS` 传入，插件按 `Authorization: Bearer` 带上；它不是平台账号凭据，所以 `login: none` 也拿得到（[§2.11](#211-插件设置-settings)）。
- XHS-Downloader 在请求的 `cookie` 参数缺省时用自己配置文件里的 cookie，传空字符串多半也会退回配置文件（推断，待核实）。所以插件 README 和服务连不上时的 `hint` 都写明「请清空 XHS-Downloader 配置里的 cookie」，否则插件声明的 `login: none` 和实际不符。
- 图片和视频地址在小红书的 CDN 上，例子里的 `*.xhscdn.com` 是示例，实际用到哪些主机待核实，照 `usuyo plugin try` 打印的被拦主机补（[§3.2](#32-调用约定)）。

<a id="93-包装-mcp-服务"></a>
### 9.3 包装 MCP 服务

插件作为 MCP 客户端，经 stdio 启动上游的 MCP 服务（要 `exec` 权限），把子命令映射成 `tools/call`：

| 子命令 | 映射 |
|---|---|
| `fetch <url>` | 调上游按链接取单条的工具，把 `structuredContent` 转成 fetch 输出，文件写进 `--out` |
| `list <source>` | 调上游列收藏的工具，把分页参数放进游标 |
| `health` | 只启动服务、读工具列表，不调会请求平台的工具 |

工具名照上游的文档填；上游返回的内容同样是不可信数据，只取字段，不执行其中的文字。上游如果还暴露发帖、点赞、私信一类工具，插件不调用，并在 README 写明。

<a id="94-浏览器扩展与油猴脚本"></a>
### 9.4 浏览器扩展与油猴脚本

在用户自己的浏览器里跑的工具（xhs-vault、Weibo-archiver、bilibili-favorites-exporter）有两条路：

- 改一行，把导出的清单直接交给投递接口（[§5.1](#51-本机-http-接口)）：

```js
// ==UserScript==
// @name        xhs-favorites-to-usuyo
// @match       https://www.xiaohongshu.com/*
// @grant       GM_xmlhttpRequest
// @grant       GM_getValue
// @connect     127.0.0.1
// ==/UserScript==
function sendTousuyo(links) {
  GM_xmlhttpRequest({
    method: "POST",
    url: "http://127.0.0.1:47615/v1/batches",
    headers: {"Authorization": "Bearer " + GM_getValue("usuyoToken"), "Content-Type": "application/json"},
    data: JSON.stringify({name: "小红书收藏夹", action: "favorite", links: links}),
  });
}
```

- 不改原工具：它照旧导出 JSON 文件，再写一个导入插件读这个文件（[§3.9](#39-从文件导入)）。导入插件按自己做的事填 `access: official-export`、`login: none`，和 [§3.9](#39-从文件导入) 一致；文件怎么来的写进 `description` 和插件 README，例如「读 xhs-vault 导出的清单；清单由你在登录的浏览器里导出」。手机上的 Agent（Mika 一类）导出的清单同样处理，`description` 写「清单由你手机上的某某导出，导出时用的是它自己的模型」。

令牌由用户粘贴进脚本设置；小红书链接里的 `xsec_token` 只在会话内有效，清单交过来后宜尽快取（[06 §1.3](research/06-platform-access-and-account-risk.md#13-混合路径视觉层拿链接公开页面取内容)）。

<a id="95-视觉层流程"></a>
### 9.5 视觉层流程

视觉层插件读屏幕上渲染出来的界面、替用户点已登录的客户端。`list` 一页处理几条，只返回 `url`，不带 `ref`；内容由别的插件按 [§3.1](#31-运行器与节奏) 去取。底层有两种做法（[调研 10](research/10-desktop-and-android-agents.md)）：

- **电脑端读系统的无障碍树。** 例如 agent-desktop（Vercel Labs，Apache-2.0）：读 macOS AX、Windows UI Automation、Linux AT-SPI 的树，给可交互的元素编号，动作默认走无障碍接口，不聚焦、不动指针；命令行不要模型、不联网。
- **安卓经 ADB。** 电脑上的获取插件经宿主管的 adb 服务连用户的手机（[§7](#7-权限与隔离)），读控件树、截图、按坐标点，在手机侧经 adb shell 读剪贴板；手机上不装 App、不常开无障碍服务。

两种都拿不到链接本身：无障碍树和控件树里没有 URL，链接要「分享 → 复制链接」后读剪贴板。

**例子一：Mac 上读微信收藏里的公众号文章链接**，再交给公众号的 `public-web` 插件取正文（[06 §17.3](research/06-platform-access-and-account-risk.md#173-首批自己适配的插件)）。

```yaml
platforms: [wechat-mp]
access: vision-ui
login: required
sites:
  - {site: weixin.qq.com, name: 微信, credential: app}
upstream:
  - {name: agent-desktop, url: https://github.com/vercel-labs/agent-desktop, license: Apache-2.0}
capabilities:
  list:
    access: vision-ui
    login: required
    sites: [weixin.qq.com]
    order: newest-first
    interaction: foreground
    timeout: 10m
    sources:
      - {source: "wechat:favorites:articles", title: 微信收藏里的文章, action: favorite}
pacing:
  wechat-mp: {batch: true, interval: 6s, daily: 30}
permissions:
  devices: [accessibility, clipboard, screen-capture]   # screen-capture 只给截图加 OCR 的退路用
  network: {mode: none}
```

- `exec` 不写：agent-desktop 和它的 helper 在 `${PLUGIN_DIR}` 里（[§2.7](#27-权限-permissions)）。
- 流程：找到「收藏」点开；对列表容器取快照；每一行先往剪贴板写一个哨兵值，再右键、读菜单、点「复制链接」、读剪贴板；滚一屏，前后两屏有重叠，按 `canonicalize` 后的身份去重，不按整行文字比；连续两屏没有新条目就停。每处理一条发一次 `progress`，取快照之前也先发一次（一次快照可能要几十秒），能力里的 `timeout` 设成分钟级，卡住靠 120 秒无进度发现（[§3.4](#34-超时与取消)）。游标记最后一条的链接和已读条数，下一页先在可见的几行里找它，找不到就回到顶部重新滚。`title` 原样保留界面文字，拿不到收藏时间就让 `actedAt` 留空。
- 为什么是 `foreground`：macOS 上走无障碍接口的右键（`AXShowMenu`）照样把菜单弹到屏幕上，读链接要改剪贴板；Windows 上右键只能用物理输入，要抢焦点、动鼠标。
- **受版本限制。** Mac 微信 4.1.x 在 macOS 26 上只给浅层窗口（报告测的是聊天界面，收藏页同样读不到是推断）；PC 微信 4.1 起没开过无障碍模式的新账号拿不到树。读不到时回 `upstream-changed`，`hint` 写「这个版本的微信不给读界面」；要不要退到「截窗口 → 本机 OCR 找行 → 按坐标点」，是待定项（[§13](#13-待定) 第 17 项）。Windows 版另写一份。
- 改造 wechat-favorites-exporter 的做法时保留它的 MIT 版权声明（[§9.6](#96-许可证)）；流程总共几步，自己写比 fork 省事。
- `pacing` 是示意值：每条大约四次 agent-desktop 调用，加 0.5 秒左右的等待，没有微信这边的阈值数据（推断）。

**例子二：安卓手机上的小红书收藏，经 ADB**（包名 `com.xingin.xhs` 出自 Mika 的 App 知识文件）。

```yaml
platforms: [xiaohongshu]
access: vision-ui
login: required
sites:
  - {site: www.xiaohongshu.com, name: 小红书, credential: app}
capabilities:
  list:
    access: vision-ui
    login: required
    sites: [www.xiaohongshu.com]
    order: newest-first
    interaction: foreground
    timeout: 10m
    sources:
      - {source: "xiaohongshu:favorites", title: 我在手机上收藏的笔记, action: favorite}
pacing:
  xiaohongshu: {batch: true, interval: 10s, daily: 60}
settings:
  - {key: display, label: 在哪块屏上跑, type: enum, options: [main, virtual], default: main}
permissions:
  devices: [adb]
  network: {mode: none}
requires:
  runtime: ["adb"]
run:
  command: ["${PLUGIN_DIR}/usuyo-fetch-xiaohongshu-android"]
  serve: true
```

- 默认在主屏上跑：跑的时候手机要连着、解锁、亮屏，不能用。后台虚拟屏是插件设置（`display`），默认关：它多出一块公开屏，App 自己就查得到不在 0 号屏上（推断）。微信在 Mika 的虚拟屏上整屏读不到、点按没反应（Mika #237，只在虚拟屏上测过，主屏上行不行待核实），所以微信和视频号不用虚拟屏。
- 读树用 `uiautomator dump`，它要等屏幕安静一秒，十秒内等不到就失败；自动播放视频的页面改用截图加本机 OCR。
- 链接：每条点开 → 分享 → 复制链接，在手机侧经 adb shell 读剪贴板；不靠 scrcpy 的剪贴板同步，那会盖掉电脑上的剪贴板。不用屏上位置当 `ref`，新收藏会把位置往后挤。小红书分享链接里的 `xsec_token` 只在会话内有效，宜尽快取（[§9.4](#94-浏览器扩展与油猴脚本)）。
- 流程格式可以参考 Mika 的路线回放（Route、Selector、Anchors，Apache-2.0，借代码要保留 LICENSE、NOTICE 和改动说明）：选择器记角色、名字、资源 ID、位置、同类序号；每步核对前台包名和几段稳定文字，对不上就停（`upstream-changed`），碰到验证或风控提示回 `blocked`。
- `pacing`、`requires.runtime` 都是示意值，没有证据支撑具体数字。一次调用内部的节奏可以参考 Mika 给 `careful` 一类 App 的数：换一屏停 2.5–5 秒，按屏上字数每字加 15 ms，一次至多换 24 屏；`interval`、`daily` 另按平台给。

两种做法共用的规则：

- **流程固定下来**（Maestro 的 YAML、mirroir-mcp 的录制回放、Mika 的路线一类），不每次让模型现场决定；客户端改版就要改流程。运行时默认不用模型，对不上就停。能力声明了 `provider: optional` 的，用户打开后可以让看图模型找一个控件，找到后再在树里核对；模型经 [§4.3](#43-模型与密钥由宿主代发) 代发，本机和云端都行，云端的界面标「离开本机」。
- 屏幕或手机没解锁、窗口被挡住时发 `waiting` 事件；等不到就返回 `dependency-missing`，`needs` 照 [§3.3](#33-信封与错误码) 的前缀写，例如 `["state:unlocked"]`、`["permission:accessibility"]`、`["app:wechat"]`、`["state:authorized"]`。
- **底层工具的错误怎么翻**（plugin-kit 的视觉层 SDK 做）：没给系统授权 → `dependency-missing` 加 `permission:accessibility` 或 `permission:screen-capture`；客户端没开 → `dependency-missing` 加 `app:`；窗口里是登录或扫码页 → `login-required`；树是空的或太浅、找不到列表 → `upstream-changed`；界面上出现验证、风控或下线提示 → `blocked`；元素过期、对不上 → 插件自己重新快照再试，还不行回 `upstream-changed`；快照没取全不算错，回已经读到的条目，`nextCursor` 指回这一屏。
- **隐私**（视觉层 SDK 做，契约测试也查）：
  - agent-desktop 的状态目录设在 `${TMP}`（常驻时设在 `${HOME}`），不用 `${CACHE}`：它的元素映射里存着界面上的文字。不开会话、不开 trace、不用 `--screenshots`。
  - 进了列表以后每次快照都限在列表容器上（`--root`），整窗快照只在导航时做一次：微信主窗口里有联系人名字和消息预览。
  - 截图只截目标窗口，写在 `${TMP}`，本机 OCR 用完马上删，不进 `${OUT}`。
  - 剪贴板：一页开始前把原内容读进内存，一页结束写回；全程不写日志、不写盘。剪贴板管理工具会记下客户端复制出的每条链接，宿主拦不住；第一次用到 `clipboard` 时界面说清楚。
  - stderr 不写屏幕上的文字、标题、链接，只写计数和错误码。
- **包 agent-desktop 时**：只按名字调 `${PLUGIN_DIR}` 里那份固定版本；检查信封的版本号，主版本和锁定的不一样就回 `internal`；只有 `interaction: foreground` 的能力才用 `--headed`；每条命令都带 `--app`（不带时它读的是前台 App），最好也带 `--window-id`；不用 `launch --cdp`（那等于在官方客户端里执行代码，按 [§2.4](#24-access-与-login) 要记 `client-hook`），不用 `skills`，也不用它仓库里的 JEV 脚本（它把屏幕描述发给第三方的模型服务，不经 [§4.3](#43-模型与密钥由宿主代发)）。

<a id="96-许可证"></a>
### 9.6 许可证

按 [06 §17.4](research/06-platform-access-and-account-risk.md#174-fork-与许可证)：MIT、Apache、BSD 一类可以 fork、可以打进插件资产，保留版权声明；GPL 系只做用户自装、插件以独立进程或 HTTP 调用，不随资产分发（[ADR-003 §8](decisions/ADR-003-plugins.md#8-gpl-工具)）；非商业或源码可见许可不打包、不进两个组织；没有许可证的只参考做法。插件 README 写明上游、许可证和改动，`upstream` 字段照填。上游的发布包里没有 LICENSE 文件时（agent-desktop 就是这样），从上游仓库取一份放进资产。

自己写的获取插件用 MIT，和 plugin-kit 一致。两个组织里的插件不调用用户自装的非商业许可工具；别人在自己仓库里这样做不管。

<a id="10-契约测试与-plugin-kit"></a>
## 10. 契约测试与 plugin-kit

插件仓库里放：

| 文件 | 内容 | 什么时候跑 |
|---|---|---|
| `testdata/canonicalize.json` | `[{"input": "<链接>", "expect": {"match": true, "identity": {…}}}, {"input": "https://example.com/", "expect": {"match": false}}]`；要展开短链的标 `"live": true` | 离线部分每次 |
| `testdata/outputs/*.json` | 录下的 `fetch`、`list`、`sources`、`health` 输出；提交前换成样例数据，`usuyo plugin try --record` 默认把标题、作者和各种 ID 换成占位值，因为这些来自作者自己的账号，放进公开仓库就公开了 | 每次，只校验 schema 与路径 |
| `testdata/live.json` | 真实链接，给维护者在自己电脑上跑 | 只在本机加 `--live` 时 |

```
usuyo plugin check .
usuyo plugin check . --live
usuyo plugin try fetch "https://v.douyin.com/iRNBho6u/"
usuyo plugin pack --version 0.3.1 dist/*
```

`usuyo plugin check` 做：清单 schema；`id` 和 git 远端一致；占位符和 `${setting.*}` 都能解析；`access`、`login` 顶层是不是各能力里的最高档；`login: none` 的能力没有 `sites`；`fetch.input` 含 `url` 时有 `match`；`sites[].domains` 每项和 `site` 同属一个可注册域名、不是公共后缀；`env` 里没有凭据类变量；`outputs`、`produces` 用的是词表里的值；样例输出过 schema、路径不越界；在禁网的沙箱里跑离线的 `canonicalize` 样例，连同 plugin-kit 为每个声明的平台附的共用样例，`identity.contentId` 要符合平台列表里的格式；跑 `health`；`run.serve: true` 时两种方式结果一致；`readback: true` 的导出插件跑往返测试，`readback(run(V))` 按值等于 V 的用户层，字符串逐字节相同，集合相等（[§4.4](#44-导出的-run-与-readback)）。`usuyo plugin try` 在真实的沙箱和凭据下跑一次，打印信封、事件和被代理拦下的主机。`usuyo plugin pack` 给不用 plugin-kit 发版工作流的仓库生成发布清单（[§8.2](#82-发版与-release-资产)）。

- 契约测试不在 CI 里请求平台：CI 在机房 IP 上跑，结果不稳，也平白多出请求。
- GitHub App 不执行插件，只做不用跑插件的检查：清单 schema、tag 和 `version`、资产 sha256、来源证明、`testdata/outputs` 过 schema，结果写成 check run。要执行插件的部分（`health`、禁网沙箱里的 `canonicalize` 样例、两种方式结果一致、导出插件的往返测试）放进 plugin-kit 的可复用工作流，在插件仓库自己的 Actions 里跑；App 只读那次运行的结论。否则 App 要在自己的服务器上执行任何装了它的仓库发布的二进制。
- plugin-kit（MIT）里有：清单、信封、JSON-RPC 消息和 `derived/` 文件的 JSON Schema；平台 id 列表；`usuyo plugin check` 的实现（主程序内嵌同一份代码，另有给 CI 用的 action）；发版可复用工作流；SDK。11-11：plugin-kit 是 Rust crate `crates/usuyo-plugin-kit`，转出协议类型，另带 HTTP 客户端（[§10.1](#101-plugin-kit-的-http-客户端11-11)）；TypeScript、Python 的 SDK 还没有。SDK 管参数解析、信封和错误码、stderr 事件、常驻模式的循环、凭据文件读取、输出目录的路径检查、转换与导出的 JSON-RPC 循环；作者只写处理函数。
- schema 以 plugin-kit 为准；内核经 cargo 依赖引用，`crates/usuyo-contract` 只留资料库自己的类型（[§11](#11-设计取舍)）。11-11 还没有 schema 文件，也没有 `usuyo plugin check`、`try`、`pack`。

<a id="101-plugin-kit-的-http-客户端11-11"></a>
### 10.1 plugin-kit 的 HTTP 客户端（11-11）

`crates/usuyo-plugin-kit` 的 `http::Client`（MIT）。获取插件的每个请求都走它，包括接口调用、展开短链、下图片和视频，不另引别的 HTTP 库：在沙箱和宿主的代理做出来以前（[§7](#7-权限与隔离)），这是挡住插件连到本机和内网的那一层。它做的：

- **只连公网地址。** 只认 `http`、`https`；带用户名、密码的链接不连；主机要在调用方给的名单里。解析出的每个地址都要是公网：IPv4 拒 `0/8`、`10/8`、`100.64/10`、`127/8`、`169.254/16`、`172.16/12`、`192.0.0/24`、`192.0.2/24`、`192.88.99/24`、`192.168/16`、`198.18/15`、`198.51.100/24`、`203.0.113/24`、`224/4`、`240/4`；IPv6 只认 `2000::/3`，再去掉 `2001::/23`、`2001:db8::/32`、`2002::/16`、`3fff::/20`；IPv4 映射和 `64:ff9b::/96` 按里面的 IPv4 判。连接只连查过的那个地址，不再查一次 DNS。DNS 算在总时限里。
- **假 IP 代理。** 宿主设了 `USUYO_NETWORK=fake-ip`（[§3.13](#313-宿主和-plugin-kit-现在做到的11-11)）时，回答全在 `198.18.0.0/15` 的域名照常连；单段名字和 `.localhost`、`.local`、`.internal`、`.lan`、`.home`、`.home.arpa`、`.corp`、`.intranet`、`.private` 结尾的照旧拒；直接写成这一段 IP 的链接一律拒。插件用 `Policy::from_host()`（`Client::new` 就是），测试才自己拼策略。
- **重定向默认不跟。** 调用方开了才跟（只跟 301、302、303、307、308），每一跳重新查主机名单和地址。换了主机，或从 https 跳到 http，只留 `Accept`、`Accept-Language`，调用方加的别的头不带过去。
- **cookie** 不能自己写进 `Cookie` 头，要用 `Cookie::new(值, 主机)`：只发给列出的主机，只走 https。没有 cookie 罐，`Set-Cookie` 不带到下一跳。
- **大小和时间。** 读正文时调用方必须给上限，按解压后的字节算，到上限就停（`TooLarge`）；默认总时长 30 秒（含所有跳和读正文），连接 15 秒。不读系统的代理设置。

通用网页插件（`plugins/web`）另外：图片的 `Referer` 只给页面的源加 `/`，https 页面上的 http 图片不给；正文里有图片没下下来时在 `missingReason` 里写几张没下下来，一张都没下下来才把 `images` 记进 `missing`。设置 `allow-private-network`（默认关）只放开你给的那条链接的第一跳，图片和重定向照旧只连公网。

<a id="11-设计取舍"></a>
## 11. 设计取舍

技术上替用户定下的做法：

| 做法 | 理由 | 放弃了什么 |
|---|---|---|
| 清单叫 `usuyo-plugin.yaml` | 三类插件都对得上；在被包装的仓库里不和别人的 `plugin.yaml` 撞；搜得到 | ADR-003 原来「在 `adapter.yaml` 上扩展」的说法 |
| 客户端只认 release 资产里的清单 | 和资产、sha256 同一次发布，immutable 之后不会再变 | 不能只改默认分支上的清单就生效 |
| `id` 等于 `owner/repo` | 不会有两个插件抢一个 id；来源一眼能看出 | 仓库转移后 id 会变，要重新确认 |
| 获取插件保留子命令，另加可选的 `serve` | 子命令在终端里就能调试，包装 shell 脚本最省事；浏览器和设备类插件靠常驻省掉冷启动、守住资料目录 | 作者多一种方式要支持，靠 SDK 从同一套处理函数生成 |
| stdout 只放一个信封，退出码只分 0、1、2 | 成败只看一处；错误分类全在 `code` 里，不用两套对照表 | 不给 shell 用户按退出码细分错误 |
| 进度和等待走 stderr 的 `@usuyo ` 行 | 不另开文件描述符，Windows 上也能用；普通日志照写 | 日志里刚好以这几个字开头的行会被当成事件 |
| 每种错误都换下一个，只有 `cancelled` 不换；限流和拦截只暂停出错的那个插件 | 照 D-c（[§12](#12-调研-06-提议的采纳情况)）；不同插件的出口、接口和做法不同，一个被拦不代表别的也会被拦，一直撞同一个拦截页也没用 | 不做平台级的统一冷却；用同一个账号的插件可能被一起限住，接着试它会多碰一次限流 |
| `health` 默认不请求平台 | 打开设置页、启动时都会跑，不该每次碰平台 | 登录是否真有效要用户点「检查登录」才知道 |
| 跟着收靠 `newest-first` 加「一整页都是这个订阅列过的就停」，`--since` 只是提示 | 不要插件记跟着收的进度；平台不按时间排时退回全量 | `unordered` 的来源每次都要全量翻 |
| 导入复用 `list` 加 `fetch <ref>` | 预览、确认、续传和网络来源走同一条路 | 大的导出包要靠缓存或常驻模式避免反复打开 |
| 凭据用临时文件，`storageState` 加 cookies.txt；`HOME` 指向插件私有的 `${HOME}` | 两种格式能直接喂给浏览器和 yt-dlp 一类工具；私有 `HOME` 让被包装的工具读不到用户自己的配置 | 同一用户的其他进程在调用期间读得到这个 0600 文件 |
| 节奏由插件给建议、宿主取最严、用户能改；同一平台同一时间一个调用，`canonicalize` 不占这把锁 | 内核不写平台知识也能有合理的默认值；所有插件共用一组上限；新贴的链接不用等正在下的大视频 | 一个插件写得过严会拖慢同平台的其他插件，用户要自己调 |
| `sites` 的 (`site`, `credential`, `domains`) 出现新值、密钥的注入目标改了，都算扩大权限 | 等于把另一批凭据或密钥交到新的地方，和多了网络主机同一类 | 合理地加一个登录站点也要用户点一次同意 |
| 模型和云端密钥经宿主本机端点代发；获取插件的密钥设置经 `USUYO_SECRETS` 另传 | 照 ADR-003，转换、导出插件没有理由拿密钥；获取插件的密钥设置是用户为这个插件填的，只交给这个插件，和平台账号凭据分开，`login: none` 的插件也拿得到 | 两类插件拿密钥的办法不同 |
| 常驻进程按站点集合分开；资料目录按站点共用，交接前先关掉持有它的进程 | 凭据按能力发而沙箱按进程生效，分开进程才守得住 `login: none`；用户每个站点只登录一次 | 同一个插件可能开两个常驻进程；换插件时要等上一个进程退出 |
| 运行时由宿主按名字解析，放进 `PATH` 最前 | launchd 启动的进程看不到 Homebrew 的目录；插件不用各自找程序 | plugin-kit 要维护一张运行时表 |
| 输出目录整体检查，只收普通文件和目录 | 符号链接、硬链接能让宿主替插件读到沙箱外的文件 | 插件不能用链接省空间 |
| 用户添加的来源在添加时固定签发者；按数字仓库 id 认仓库 | 和 Mihon 认 store 签名指纹同一个思路，客户端不用认识 `usuyo-fetch`；改名、转移、旧名被重新注册都能发现 | 来源换了发版方式或仓库转移后，要用户重新添加 |
| 投递来的身份由宿主重新算；`api` 令牌只交链接 | 投递方自报的身份是不可信输入；Agent 交内容等于绕开改动提议 | Agent 不能直接交它已经取好的内容 |
| 获取方式组织每个插件一个仓库 | 下架只影响一个仓库；不要索引文件 | 共享代码和统一改动不如大仓库方便（[§8.8](#88-获取方式组织的仓库结构)） |
| 来源证明用 immutable releases 加构建来源证明，作为 release 资产发 | 不自己管签名密钥；不靠 API 就能下载、离线验证 | 依赖 GitHub 和 Sigstore 的公共服务；个人仓库只能建议，不能要求 |
| 安装不调用 npm、pip | 装上的就是 sha256 核过的那一份，不在安装时临时拉依赖 | 脚本类插件要自己打包依赖 |
| schema 以 plugin-kit 为准 | 插件作者只依赖 MIT 的 plugin-kit，不用拉整个内核模块；只有一份不会漂移 | 内核改契约时要先给 plugin-kit 发版 |
| 投递接口按投递端发令牌，来源由令牌决定 | 撤销一个脚本不影响别的；来源不靠投递方自称 | 每接一个工具要建一个令牌 |
| 改造上游不用 GitHub 的 fork 按钮 | 不进上游的派生网络；派生多的网络会被整体处理（[§8.8](#88-获取方式组织的仓库结构)） | GitHub 上看不到「forked from」，要在 README 写明上游 |
| 能力加 `interaction`；宿主加两把整机锁（占屏幕的、用剪贴板的），每台安卓设备一把锁 | 「复制链接」类流程走无障碍接口也会弹菜单、改剪贴板，两个视觉层插件同时跑会互相弄坏；`devices` 只说要什么权限，说不出会不会打扰用户 | 不同平台的视觉层插件不能并行 |
| 不为虚拟屏、合成输入加 `devices` 取值；安卓上的读屏、截图、剪贴板都归 `adb` | 虚拟屏是插件的做法，做成插件设置；会不会动指针由 `interaction` 说；电脑本机的三项权限和手机上的不是一回事 | 权限页看不出插件开不开虚拟屏，要看插件设置和 README |
| 视觉层插件自带固定版本的底层工具（例如 agent-desktop），不让用户 `npm i -g`；`${PLUGIN_DIR}` 里的程序不用写进 `exec` | 安装时不跑 npm 的安装脚本，流程锁在一个信封版本上，照样按 sha256 和来源证明核过；底层工具会自己起 helper | 每个插件多几 MB，底层工具升级要跟着发插件版本；`exec` 看不出包里带了哪些程序，要看 `upstream` 和 README。三个以上插件都带同一个工具时，再考虑做成宿主管的共用运行时 |
| 不做按插件分开的 macOS 授权 | 要让插件进程自己当负责进程，得用私有接口；ad-hoc 签名的插件每次升级可能都要重新授权（推断） | 宿主拿到辅助功能、屏幕录制授权以后，没声明的插件也用得上，只能「仅告知」 |
| 宿主自己管 adb 服务和密钥，插件只拿到端口和设备序列号 | 密钥固定下来，手机只授权一次；插件拿不到手机的局域网地址 | 宿主多管一个进程；连着几台设备时插件仍能点名别的设备 |
| 安卓视觉层默认在主屏上跑，后台虚拟屏是插件设置、默认关 | 和人用手机一样，不多出一块别的 App 查得到的公开屏；微信在虚拟屏上读不到、点不动 | 跑的时候手机不能用；主屏上微信行不行还没有实测 |
| 获取插件也能经宿主代发用模型；视觉层默认不用模型，对不上就停 | 只靠回放的流程可预期，也不会把收藏页上的字发出去；要兜底时照转换插件的规矩，插件拿不到密钥，界面能标「离开本机」 | 客户端小改版也会让流程停下，要等插件更新 |
| 信封不加「投递」字段（agent-desktop 有 `disposition`） | v1 的 `list`、`fetch` 只读，重试或换下一个都安全；唯一的写操作 `unfavorite` 是幂等的 | 以后加不幂等的写操作时要改协议 |

<a id="12-调研-06-提议的采纳情况"></a>
## 12. 调研 06 提议的采纳情况

[06 §17](research/06-platform-access-and-account-risk.md#17-对歇存的启示) 的提议，按用户 2026-11-01 的方向处理。「原因」一列引用的用户决定：

- D-a：获取插件的清单保留 `access`、`login` 两个描述性必填字段，给用户挑选用，不是闸门。
- D-b：不做宿主推导的风险档和平台修正表，不做「失败后不往上换」和单独的确认弹窗；调高 `access`、`login` 只在更新说明里标出。
- D-c：运行器按用户排的顺序依次试，失败换下一个，全部失败停在 `link`。
- D-d：用户添加第三方来源时确认一次声明，不做逐个插件的弹窗。
- D-e：每个平台一组节奏上限，该平台所有获取方式共用。
- D-f：自己维护的获取插件放单独的 GitHub 组织，客户端不预置任何获取来源；11-01 加了两个例外，通用网页获取和别的工具的导入随安装包预装（P1、P3）。
- D-g：清单能声明用的是哪个站点的登录，界面据此写「会用到你的某某账号」。


| 提议 | 出处 | 结论 | 原因 |
|---|---|---|---|
| 获取插件声明 `access`，七层之一 | [§17.1](research/06-platform-access-and-account-risk.md#171-清单里声明获取层级) | 采纳 | 给用户挑选用的标签（D-a） |
| 声明 `login`，取代 `requires.cookie` | [§17.1](research/06-platform-access-and-account-risk.md#171-清单里声明获取层级) | 采纳 | 另加 `sites`，写明用哪个站点的账号（D-a、D-g） |
| `risk` 由宿主按层级和登录推出 | [§17.1](research/06-platform-access-and-account-risk.md#171-清单里声明获取层级) | 不采纳 | 风险由用户自己判断（D-b） |
| 平台风险修正表随屏蔽清单发布 | [§17.1](research/06-platform-access-and-account-risk.md#171-清单里声明获取层级) | 不采纳 | 同上（D-b） |
| 按能力分别声明，顶层取最高 | [§17.1](research/06-platform-access-and-account-risk.md#171-清单里声明获取层级) | 采纳 | 另加 `perPlatform` 给多平台插件 |
| `login: none` 拿不到凭据，`env` 不能声明凭据变量 | [§17.1](research/06-platform-access-and-account-risk.md#171-清单里声明获取层级) | 采纳并放宽到所有能力 | 凭据只走凭据文件，任何能力的 `env` 都不能传凭据 |
| 凭据只由宿主传，插件不读浏览器 cookie 库 | [§17.1](research/06-platform-access-and-account-risk.md#171-清单里声明获取层级) | 采纳 | [§6](#6-凭据) |
| 只读导出文件的插件 `network: none` | [§17.1](research/06-platform-access-and-account-risk.md#171-清单里声明获取层级) | 采纳 | [§3.9](#39-从文件导入) 的例子照此写 |
| 加设备类权限 | [§17.1](research/06-platform-access-and-account-risk.md#171-清单里声明获取层级) | 采纳 | `permissions.devices` |
| `client-hook`：宿主不代装证书、不注入，单独确认，不自动更新 | [§17.1](research/06-platform-access-and-account-risk.md#171-清单里声明获取层级) | 改写 | 宿主不代装、不注入照旧，这是宿主自己不做的事；不单独确认，更新规则和其他插件一样（D-b） |
| 调高 `access` 或 `login` 按扩大权限处理 | [§17.1](research/06-platform-access-and-account-risk.md#171-清单里声明获取层级) | 不采纳 | 只在更新说明里标出（D-b） |
| GitHub App 加静态检查 | [§17.1](research/06-platform-access-and-account-risk.md#171-清单里声明获取层级) | 改写 | 凭据变量判失败；层级调高只写进结果；写命令只作提示（[§8.3](#83-校验与来源证明)） |
| 每个平台一张列表，显示层级、账号、风险、来源、校验、最近失败 | [§17.2](research/06-platform-access-and-account-risk.md#172-设置里怎么让用户选) | 改写 | 去掉风险一列，加来源证明（[§8.4](#84-三种来源)） |
| 默认按风险从低到高排序 | [§17.2](research/06-platform-access-and-account-risk.md#172-设置里怎么让用户选) | 待定 | 默认顺序由用户定（D-c）；先试不用账号的、同类里按 `access` 从低到高，是 [PRD P4](PRD.md#72-获取插件) 的建议 |
| 高风险层和微信视觉层单独确认 | [§17.2](research/06-platform-access-and-account-risk.md#172-设置里怎么让用户选) | 不采纳 | 改为添加来源时确认一次声明（D-b、D-d） |
| 自动换方式时不往更高的层换 | [§17.2](research/06-platform-access-and-account-risk.md#172-设置里怎么让用户选) | 不采纳 | 按用户的顺序依次换（D-c） |
| 被拦截后这个平台整体冷却 | [§17.2](research/06-platform-access-and-account-risk.md#172-设置里怎么让用户选) | 改写 | 只暂停出错的插件在这个平台的调用（[§3.1](#31-运行器与节奏)） |
| 每个平台一组节奏上限，所有获取方式共用 | [§17.2](research/06-platform-access-and-account-risk.md#172-设置里怎么让用户选) | 采纳 | D-e；插件可以给建议值 |
| 跟着收用到要登录的方式时默认更低频并提示 | [§17.2](research/06-platform-access-and-account-risk.md#172-设置里怎么让用户选) | 不采纳 | 频率只看平台节奏和订阅设置 |
| 「复用日常浏览器」和「专用资料目录」都给 | [§17.2](research/06-platform-access-and-account-risk.md#172-设置里怎么让用户选) | 采纳 | 另加 cookie 文件；默认哪种待定（[PRD P8](PRD.md#72-获取插件)） |
| 首批自己适配的插件 | [§17.3](research/06-platform-access-and-account-risk.md#173-首批自己适配的插件) | 采纳 | 仓库名按 [§8.1](#81-仓库命名) |
| 上游只有 HTTP 或 MCP 时写一层命令行转接 | [§17.3](research/06-platform-access-and-account-risk.md#173-首批自己适配的插件) | 采纳 | [§9](#9-适配现有工具) |
| 通用视觉层插件：设备通道、固定流程、按人的节奏 | [§17.3](research/06-platform-access-and-account-risk.md#173-首批自己适配的插件) | 采纳 | 设备权限、`waiting` 事件、节奏建议（[§9.5](#95-视觉层流程)） |
| 只提供 Skill，让用户的 Agent 操作完交给本地接口 | [§17.3](research/06-platform-access-and-account-risk.md#173-首批自己适配的插件) | 改写 | 能走投递接口，不作为视觉层的默认 |
| fork 与许可证的分档 | [§17.4](research/06-platform-access-and-account-risk.md#174-fork-与许可证) | 采纳 | [§9.6](#96-许可证) |
| 自己维护的获取方式放单独组织，客户端不预置 | [§17.4](research/06-platform-access-and-account-risk.md#174-fork-与许可证) | 采纳 | D-f；通用网页获取和别的工具的导入例外，随安装包预装（[§8.4](#84-三种来源)） |
| fork 时要做的事：保留许可证、去掉写操作和推广、锁定外部二进制 | [§17.4](research/06-platform-access-and-account-risk.md#174-fork-与许可证) | 采纳 | 另加：不用 fork 按钮（[§8.8](#88-获取方式组织的仓库结构)） |
| 官方组织收转换和导出 | [§17.4](research/06-platform-access-and-account-risk.md#174-fork-与许可证) | 采纳 | 平台中立的共用代码进 plugin-kit |
| 只读导出文件的导入插件放哪个组织 | [§17.4](research/06-platform-access-and-account-risk.md#174-fork-与许可证) | 已定 | 11-01 用户选 B：别的工具的导出进官方组织并预装，平台官方导出包放 `usuyo-fetch`（[PRD P3](PRD.md#72-获取插件)、[§8.4](#84-三种来源)） |
| 法律意见 | [§17.4](research/06-platform-access-and-account-risk.md#174-fork-与许可证) | 保留 | 仍待核实 |

<a id="13-待定"></a>
## 13. 待定

需要用户在产品层面拍板的问题都在 [回答表](answers.md)，选项、建议和影响以那里为准。本文和别的文档引用本节时写的「第 N 项」就是下面的编号，编号固定，不重排；open-decisions 的表格行没有单独的锚点，链接指向条目所在的节，条目编号写在链接文字里。第 29–34 项例外：不是产品问题，是 SPEC v1 草案（[第 0 层](SPEC.md#0-约定第-0-层)、[第 1 层](SPEC.md#1-资料库-v1第-1-层)）带来的协议改动，用户过目 SPEC 后再写进正文（[总纲](BUILD.md#5-功能线)）；正文里受影响的地方标了「要改」。11-11 代码照第 29–32 项做了，这几处正文跟着改了；第 33、34 项还没做。第 35–39 项是 11-11 写插件时发现的空缺，还没定。

- **1.** 两个组织的名字和权利人联系地址：[N1、N2](BUILD.md#7-只有用户能答的和能验的)，[R4](#85-添加来源更新与权限扩大)。
- **2.** 只读导出文件的导入插件放哪个组织、要不要预装：[P3](PRD.md#72-获取插件)。11-01 已定：别的工具的导出进官方组织并预装，平台官方导出包放 `usuyo-fetch`（[§8.4](#84-三种来源)）。
- **5.** 通用网页获取放哪里（内核内置、官方组织出插件随安装包预装、只放获取方式组织），`builtin/` 前缀用不用：[P1](PRD.md#72-获取插件)。11-01 已定：官方组织出插件，随安装包预装，用 `builtin/` 前缀（[§8.4](#84-三种来源)）。
- **6.** 手机投递默认走哪条：[C2](PRD.md#71-收下)。
- **9.** 插件详情里附不附「这一层的已知情况」：[P13](PRD.md#72-获取插件)。
- **10.** 声明文案（[附录 A](#附录-a-声明模板)）怎么定稿、公开前请不请人看：[R1](#85-添加来源更新与权限扩大)。
- **12.** 浏览器扩展要不要也能当获取方式（[§5.3](#53-各种入口怎么接) 最后一行）：[P17](PRD.md#72-获取插件)。
- **13.** `usuyo-fetch` 的发版用谁的工作流（[§8.8](#88-获取方式组织的仓库结构)）：[O9](BUILD.md#7-只有用户能答的和能验的)。
- **14.** 主仓库公开前，`docs/research` 里按平台写取法的部分放在哪，附录 A 的风险证据链接指向哪：[O2](BUILD.md#7-只有用户能答的和能验的)。
- **15.** 收件文件夹里的导入文件直接收下，还是进导入预览：[C7](PRD.md#71-收下)。
- **17.** 电脑端视觉层先做什么，微信收藏读不到界面时退不退到截图加 OCR（[§9.5](#95-视觉层流程) 例子一）：[P10](PRD.md#72-获取插件)。
- **18.** 安卓要不要做、做到哪（[§7](#7-权限与隔离) 的 adb 管理、[§9.5](#95-视觉层流程) 例子二）：[P18](PRD.md#72-获取插件)。
- **19.** ~~视觉层调用能不能自己跑~~：2026-11-04 已定，见 [§3.1](#31-运行器与节奏)（[P19](PRD.md#72-获取插件)）。
- **20.** `usuyo-fetch` 收不收包装自带模型的第三方手机 Agent 的获取插件（[§4.3](#43-模型与密钥由宿主代发)）：[R11](#85-添加来源更新与权限扩大)。
- **21.** 用户自己填的来源参数：`sources` 只能列插件自己知道的来源，填不进一个 feed 地址或一个账号（[§3.8](#38-list-与跟着收)）。提议加子命令 `resolve-source <输入>`，返回 `{match, source, title, order, owner}`；先例是 RSSHub 路由的 `:param`：[调研 11 §6](research/11-subscriptions-compat-and-wx-cli.md#6-对插件协议的影响)。
- **22.** feed 条目的身份和第一次拉：`guid`、`atom:id` 是任意字符串，套不进 plugin-kit 要求的 `contentId` 正则；别人的来源第一次拉只记下已有条目、不收，以后只收新出现的（[§3.8](#38-list-与跟着收)）：[调研 11 §6](research/11-subscriptions-compat-and-wx-cli.md#6-对插件协议的影响)。11-11 答了一半：宿主不查 `contentId` 的格式，只要非空、不超过 256 字节、没有控制字符，`guid` 放得进；第一次拉还没做。
- **23.** OPML 导进来的是订阅，不是条目：[§3.9](#39-从文件导入) 的导入插件只会列出条目，要有地方建一批跟着收的订阅。只在 [回答表第 1 条](answers.md#1-什么东西进资料库)选 B 时需要：[调研 11 §6](research/11-subscriptions-compat-and-wx-cli.md#6-对插件协议的影响)。
- **24.** 上游 AI 产物放哪：fetch 输出加一个可选的 `derived` 字段，放 Horizon 这类上游已经算好的摘要、打分和标签，宿主照转换插件的规矩移进 `derived/`；另加一个原文来源标记（例如 `textOrigin: platform | machine`），界面据此标出原文是机器写的。`derived` 那半要不要做，随 [回答表第 1 条](answers.md#1-什么东西进资料库)一起答：[调研 11 §6](research/11-subscriptions-compat-and-wx-cli.md#6-对插件协议的影响)。
- **25.** 连本机 Unix socket 的权限：包装常驻进程类的工具（例如 wx-cli 的 `daemon.sock`）时，`fs.read`、`network.hosts` 都表达不了「连本机某个 socket」，要么加一个权限项，要么要求这类插件走本机 HTTP（[§2.7](#27-权限-permissions)）。只在第三方写包装常驻进程的插件时用得到：[调研 11 §6](research/11-subscriptions-compat-and-wx-cli.md#6-对插件协议的影响)。
- **26.** 按关键词搜、按创作者主页列：现有 `list` 只列用户自己的收藏夹和列表，没有带查询词的搜索，也没有「某个创作者的主页」这种来源；结果只列出、不进库，用户挑了才 `fetch`（2026-11-04 用户要，[回答表](answers.md)）。
- **27.** 评论：`fetch` 输出没有评论这一类；要可选、默认不取，按平台或按条打开，能设上限（同上）。评论 JSON 的形状、上限怎么交给插件（`fetch` 现在只有 `--want`）还没定。11-11：放哪照第 31 项做了，形状和上限还没定。
- **28.** 扫码登录：要账号的平台在内置浏览器里打开登录页、用户扫码，登录态按站点存本机；和 [§6](#6-凭据) 的凭据怎么接、过期后怎么提示重扫（同上）。
- **29.** web 身份的插件侧：`platform: web` 的 `identity` 不带 `contentId`，`identity.url` 必填（[§3.5](#35-canonicalize-与身份)）；宿主再照 [SPEC §0.4](SPEC.md#04-id) 规范化一遍，算出条目 id（2026-11-05 资料库 v1 草案带来的，下同）。**11-11 已答**：照这样做了，正文 [§3.5](#35-canonicalize-与身份) 已改。
- **30.** 照 SPEC 第 0 层补的两条：`needs` 加前缀 `version:`，例 `version:usuyo>=1.3`（[§3.3](#33-信封与错误码)，[SPEC §0.3](SPEC.md#03-错误模型)）；插件给的时间（`publishedAt`、`actedAt`、`evidence.fetchedAt`）一律 RFC 3339 带偏移（[§3.7](#37-fetch-输出)、[§3.8](#38-list-与跟着收)，[SPEC §0.5](SPEC.md#05-时间)）。**11-11 已答**：两条都照做了，`publishedAt` 存成 UTC，偏移不是 `Z` 的另存 `publishedAtOriginal`。
- **31.** 评论放哪：[§2.3](#23-获取插件的字段) 的 `outputs` 词表加 `comments`；插件把评论写在输出目录根下的 `comments.json`，[§3.7](#37-fetch-输出) 加可选字段 `comments`，值固定是 `comments.json`；整个目录照 [§3.6](#36-fetch-与输出目录) 移进 `raw/<snapshotId>/`，宿主不改名（[SPEC §1.2.3](SPEC.md#123-contentmd-和各栏)）。了结第 27 项「放哪」那一半。**11-11 已答**：照这样做了，`comments` 不是 `comments.json` 判 `invalid-output`；一个文件只能担一个角色。
- **32.** 输出目录里的名字（[§3.6](#36-fetch-与输出目录)）：`usuyo.` 开头的文件名留给宿主（宿主在快照里存 `usuyo.fetch.json`），插件写了判 `invalid-output`；宿主移动前按 [SPEC §0.6](SPEC.md#06-文本和文件) 查名字（长度、保留名带扩展名也算、大小写折叠后重名、NFC），不合格判 `invalid-output`。**11-11 已答**：照这样做了，规则写进了 [§3.6](#36-fetch-与输出目录)。
- **33.** 投递遇到不收的条目（[§5.1](#51-本机-http-接口)）：`/v1/deliver` 返回的 `state` 加 `skipped`，这时不回 `id`；只在 `api` 令牌交来的链接是私密条目时这样回（[SPEC §1.7](SPEC.md#17-删掉)），又私密又在回收站里的也这样回。其余在回收站里的，哪种令牌交来都拿回来（记 `restore`，能撤销），不提醒；已经到期移除的，就是普通的收下（11-06）。收不收等身份算出来再判；只有短链自己的 `web:` 身份时（[§3.1](#31-运行器与节奏)），照常建条目、回 `id`，并进平台 id 以后再查：`api` 令牌交来的，目标私密时并进那一条，已回的 id 成了别名；别的照 [SPEC §0.4](SPEC.md#04-id) 收下（推断）。
- **34.** 导出和读回跟上资料库 v1（[§4.4](#44-导出的-run-与-readback)；11-05 照 [SPEC 第 1 层](SPEC.md#1-资料库-v1第-1-层) 的新形状重写）：
  - `run` 的 `items[]` 不再给 `manifest`、`content` 的路径，每项是 `{id, title, author, publishedAt, publishedAtOriginal, addedAt, platform, kind, url, tags: {user, platform}, topics: [{id, title}], takeaway, highlights: [{id, layer, snapshot, fragment, quote, caption}], text, notes, derived, files}`：`text` 是主快照文字的路径，`notes` 是 `notes.md` 去掉 front matter 的正文。
  - `topics[]`、`cards[]`、`drafts[]` 是 `{id, title, createdAt, folder, path}`，`path` 是只有正文的副本，私密条目的脚注已换掉（[SPEC §1.3](SPEC.md#13-条目的四个维度)）；主题另有 `members`（索引给的，私密的不列），卡片另有 `topics`，成稿另有 `kind`。
  - `readback` 返回 `run` 的用户层键，宿主照 [SPEC §1.4](SPEC.md#14-你写的那一层)、[§1.8](SPEC.md#18-同步) 合并，照 [§1.5.1](SPEC.md#151-op-词表) 记 op，`actor: external`、`via: readback:<插件 id>`（11-06 定，[§4.4](#44-导出的-run-与-readback)）。

- **35.** 一类输出只拿到一部分（2026-11-11 两个插件作者写插件时报的，35–39 同）：`outputs`、`missing` 只说一类有没有，没有字段说「这一类拿到了一部分」；web 插件现在写进 `missingReason`。
- **36.** 访客凭据：有的平台不登录也要一份访客 cookie 才取得到，[§6](#6-凭据) 的凭据里没有这一种；小红书插件现在做成插件设置，让你自己粘。**11-11 已答**：加了顶层 `visitors` 和能力的 `visitors`（[§2.5.1](#251-访客身份-visitors)），宿主开不登录的窗口取，经 `USUYO_VISITOR` 交；小红书插件改用它，去掉了手填的设置。
- **37.** 短码的别名：同一条内容的短码没有地方记成身份的别名（推断的后果：用短码和用完整 id 收下的会成两条）。
- **38.** 音频和视频分开给：`media[]` 说不出两个文件是同一段的音轨和画面、要合起来看。**11-11 已答**：`media[]` 加可选 `group`（[§3.7](#37-fetch-输出)），同一个字符串表示同一段时间线拆开的几个文件；没有 `group` 时宿主照 feature 003 [§4.1](features/003-transforms.md#41-四个转换插件) 的内容规则配（一个画面、一个音轨、时长差不超过 1 秒或 2%）。B 站插件交 `video.m4s`、`audio.m4s` 时写同一个 `group`（`cid:<这一 P 的 cid>`）。
- **39.** 按语言的提示：`hint` 一次只按 `USUYO_LOCALE` 写一种语言，没有按语言分开给的写法。
- **40.** 只有登录才看得到：`not-found` 的 `reason` 里没有这一种。X 上成人内容和受保护账号的帖子不登录看不到，插件现在回 `reason: private`，界面分不出「作者设了私密」和「登录了就看得到」（[platforms/x.md](platforms/x.md#3-取单条)）。
- **41.** 哪条路取到的：一个插件在一个能力里有几条路（X 的帖子接口和嵌入用的 syndication 接口），`evidence` 没有字段写这次是哪一条答的，出了错、改了版对不上是哪条路。
- **42.** 只有某几样要账号（已定，还没做）：能力加可选字段 `accountOutputs`，值是这个能力 `outputs` 的子集，列出只有带账号才拿得到的几样；宿主只在 `--want` 和它有交集时交凭据，别的调用照 `login: none` 办；`login: optional` 而没写 `accountOutputs` 的，当作每一样都可能用到账号。视频号写 `accountOutputs: [media:video]`（[platforms](platforms/README.md#1-怎么读这一页)）。现在宿主只看能力的 `login` 不是 `none`、站点存着登录，就交凭据。

调研 11 建议的其余几处还没编号：`owner` 的含义、同时带 `url` 和 `ref` 时先用哪个、`sites[].site` 引用设置项、`feed:` 来源按主机算节奏（`origin.subscription` 已按 SPEC v1 写进 [§3.8](#38-list-与跟着收)），见[调研 11 §6.2](research/11-subscriptions-compat-and-wx-cli.md#62-要补的地方)。

<a id="附录-a-声明模板"></a>
## 附录 A. 声明模板

以下都是草稿，未经法律意见。`usuyo`、`usuyo-fetch`、`{contact}` 换成实际值；A.2、A.3 里的 `{contact}` 是获取方式组织用的地址，和 A.1 是不是同一个见 [§13](#13-待定) 第 1 项。链接写成完整 URL，复制到别处也能打开。

<a id="a1-主仓库-readme"></a>
### A.1 主仓库 README

```markdown
**用途与责任**

usuyo 是在你自己电脑上运行的开源软件，用来做个人备份、离线阅读和注意力整理：把你在各平台存下的内容收回到本机的资料库里，变成能搜、能整理的文字。

- usuyo 本身不含任何平台的获取代码，也不托管、不转发任何平台内容。从平台取内容的是获取插件。它们是独立的程序，放在单独的仓库和组织里，不随 usuyo 分发，要你自己添加。随 usuyo 一起装好的只有两类不含任何平台知识的插件：通用网页获取，和读别的工具导出文件（书签、Pocket、Readwise 一类）的导入。
- 用不用某个获取插件、用哪个账号，由你决定。平台可能因此限制或封禁你的账号；这些后果和相关的法律责任由你自己承担。各种获取方式的已知情况见 [调研 06 第 16 节](https://github.com/usuyo/usuyo/blob/main/docs/research/06-platform-access-and-account-risk.md#16-账号风险的证据)。
- 请遵守各平台的用户协议，尊重内容作者和版权人。收回来的内容只供你自己查看，不要再发布或分发。
- usuyo 不提供默认开启的平台写操作。插件声明的「在平台上取消收藏」默认关闭，每次执行前要你确认。
- 权利人如认为本仓库或官方组织里的内容侵犯了权利，请写信到 {contact}，写明仓库、文件和理由。我们核实后处理，并回复处理结果。

本段是草稿，未经法律意见。
```

<a id="a2-获取方式组织主页"></a>
### A.2 获取方式组织主页

```markdown
**usuyo-fetch：usuyo 的获取插件**

这里每个仓库是一个平台上的一种取法，用来把你自己存下的内容做个人备份、离线阅读和注意力整理。

- 本组织和 usuyo 官方组织 usuyo 是两个独立的组织，仓库、发版和下架各自处理。usuyo 客户端不预置这里的任何插件；要用的话，在 usuyo 的设置里手动添加组织名 `usuyo-fetch`。
- 每个插件的清单写明获取层级（access）、要不要登录（login），以及会用到哪个站点的账号。挑哪个、用哪个账号由你决定；平台对账号的限制和相关的法律后果由你自己承担。已知情况见 [调研 06 第 16 节](https://github.com/usuyo/usuyo/blob/main/docs/research/06-platform-access-and-account-risk.md#16-账号风险的证据)。
- 插件只读取你自己有权查看的内容。没有默认开启的写操作；包装别的工具时去掉了发帖、点赞、私信一类命令。
- 本组织不托管任何平台内容，release 里只有插件程序和清单。
- 请遵守各平台的用户协议，尊重内容作者和版权人。
- 权利人通知：写信到 {contact}，或用 issue 模板「权利人通知」（不想公开时用邮件），写明仓库、文件和理由。核实后我们下架对应的仓库，记进置顶 issue「已下架的获取方式」；因权利人通知下架的不再加回来。
- 安全问题请按 SECURITY.md 私下报告。

本段是草稿，未经法律意见。
```

<a id="a3-插件-readme-模板"></a>
### A.3 插件 README 模板

```markdown
**{插件名}**

- 平台：{平台}
- 获取层级：`{access}`；登录：`{login}`；会用到：{站点名}账号（`{site}`）。没有登录时写「不用账号」。
- 上游：{上游项目}（{许可证}）。本仓库的改动见 CHANGES.md。
- 用途：个人备份、离线阅读、注意力整理。只取你自己有权查看的内容，不发帖、不点赞、不私信。
- 账号与责任：平台可能限制或封禁账号。用不用、用哪个账号由你决定，后果由你自己承担。已知情况见 [调研 06 第 16 节](https://github.com/usuyo/usuyo/blob/main/docs/research/06-platform-access-and-account-risk.md#16-账号风险的证据)。
- 平台写操作：{没有 / 「在平台上取消收藏」，默认关闭，每次执行前确认}。
- 不托管任何平台内容。请遵守平台协议，尊重作者和版权人。权利人通知：{contact}。

本段是草稿，未经法律意见。
```

<a id="a4-添加第三方来源时的确认"></a>
### A.4 添加第三方来源时的确认

客户端在用户添加组织名或 `owner/repo` 时显示，每个来源一次：

```text
标题：添加第三方来源 {来源}

{来源} 不由 usuyo 审核。里面的插件可能出错，也可能含有恶意代码；usuyo 会显示每个版本的校验结果、来源证明和要用的权限。

这个来源现有的插件：{由 <owner>/<repo> 的 <工作流> 构建 / 没有来源证明}。以后的新版本换了构建方式，usuyo 会停止更新并告诉你。

获取插件会用你的网络，有的还会用你的平台账号去取内容。请只用来做个人备份、离线阅读和注意力整理。平台对账号的限制和相关的法律后果，由你自己承担。（链接：各种获取方式的已知情况）

请遵守平台的用户协议，尊重作者和版权人，不要再发布收回来的内容。

[ ] 我读过了，使用这个来源的后果由我自己承担

按钮：添加 · 取消
```

「添加」在勾选后才能点。「各种获取方式的已知情况」链到 [06 §16](research/06-platform-access-and-account-risk.md#16-账号风险的证据)。本段是草稿，未经法律意见。
