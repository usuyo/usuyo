<a id="思鹭插件协议-v1草案"></a>
<a id="usuyo-插件协议-v1草案"></a>
# usuyo 插件协议 v1（草案）

状态：草案 · `protocol: 1`。宿主和 plugin-kit 做到哪见 [§3.13](fetch.md#313-宿主和-plugin-kit-现在做到的11-11)、[§10.1](plugin-kit.md#101-plugin-kit-的-http-客户端11-11)。

把自己的工具接进 usuyo，读这一份就够：清单怎么写、宿主怎么调用、输出什么 JSON、凭据和权限怎么给、怎么发版和分发。三类插件（获取、转换、导出）都在这里；投递接口给浏览器扩展、油猴脚本、手机快捷指令这类不能由宿主启动的工具用。

<a id="0-读这份文档之前"></a>
## 0. 读这份文档之前

<a id="01-读者与状态"></a>
### 0.1 读者与状态

- **读者**：插件作者（包装现成工具或自己写）、实现插件运行器的人、在 [§13](open.md#13-待定) 拍板产品问题的人。
- **状态**：草案。视觉层的两种做法（电脑端读无障碍树、安卓经 ADB，[§9.5](wrapping.md#95-视觉层流程)）和获取插件经宿主代发用模型（[§4.3](transform-export.md#43-模型与密钥由宿主代发)）照[调研 10](../research/10-desktop-and-android-agents.md)。字段名、子命令、方法名、错误码按这里写；实现插件运行器时如有改动，在本文修订并同步 plugin-kit 的 schema（[§10](plugin-kit.md#10-契约测试与-plugin-kit)）。
- **版本**：清单里的 `protocol` 是整数，本文是 `1`。宿主只加载 `protocol` 在支持集合里的插件，拒载原因显示在插件列表里（做法同 Mihon 的 `extensionLib`，见 [02 §15.1](../research/02-adapter-ecosystems.md#151-声明与匹配)）。同一个 `protocol` 内只加可选字段；插件不得输出它声明的版本里没有的字段，宿主遇到未知字段时忽略并记日志，`usuyo plugin check` 把未知字段当失败。主版本怎么升、旧主版本再加载多久，照 [SPEC §0.2](../spec/conventions.md#02-版本与兼容)。

<a id="02-和其他文档的关系"></a>
### 0.2 和其他文档的关系

- [ADR-003](../decisions/ADR-003-plugins.md) 定了插件的种类、进程模型、权限与隔离、仓库拆分、分发与校验、导出与读回；本文是它的展开。
- 资料库目录、`manifest.json`、`derived/` 文件以 [SPEC 第 1 层](../spec/library.md#1-资料库-v1第-1-层)为准（[§1.1](../spec/library.md#11-目录树)、[§1.2.1](../spec/library.md#121-manifestjson-v1)、[§1.2.2](../spec/library.md#122-raw-和-derived)），身份和 id 以 [SPEC §0.4](../spec/conventions.md#04-id) 为准。
- 内核是 Rust（[ADR-004](../decisions/ADR-004-rust-tauri.md)）。本文的类型在 `crates/usuyo-plugin`（清单、信封、宿主），JSON Schema 在 `crates/usuyo-plugin/schema/`；插件作者用 MIT 的 `crates/usuyo-plugin-kit`，它转出这些类型，另带 HTTP 客户端（[§10.1](plugin-kit.md#101-plugin-kit-的-http-客户端11-11)）；资料库的类型在 `crates/usuyo-contract`。
- 依据：适配器生态 [调研 02](../research/02-adapter-ecosystems.md)、Agent 协议 [调研 03](../research/03-agent-protocols.md)、笔记软件与插件 [调研 05](../research/05-note-apps-and-plugins.md)、平台获取与账号 [调研 06](../research/06-platform-access-and-account-risk.md)。
- 需要用户拍板的产品问题汇总在 [回答表](../answers.md)，本文相关的见 [§13](open.md#13-待定)。

<a id="03-名字"></a>
### 0.3 名字

产品名 usuyo，命令 `usuyo`（[回答表](../answers.md)）。本文的标识符：清单 `usuyo-plugin.yaml`、环境变量前缀 `USUYO_`、资料库隐藏目录 `.usuyo/`、GitHub topic `usuyo-plugin`、导出文件里的 `usuyo_id`、URL scheme `usuyo://`。调研和回答表里的 `seelu`（思鹭）、`xiecun`（歇存）是旧暂名，作为记录不改。

本文说的「本机配置目录」「按机器目录」都是 [SPEC §0.1](../spec/conventions.md#01-命名) 的按机器的目录。

GitHub 组织（用户自己注册）：官方组织 `usuyo`，获取方式组织 `usuyo-fetch`，client-hook 插件放 `usuyo-fetch-hook`。权利人联系地址写 `{contact}`，两个组织用不用同一个地址见 [§13](open.md#13-待定) 第 1 项。

<a id="1-概念"></a>
## 1. 概念

| 类 | 做什么 | 输入 | 输出 | 控制协议 |
|---|---|---|---|---|
| 获取 `fetch` | 从平台或导出文件取回内容；导入是获取的一种，来源是文件 | 链接、列表来源、导出文件 | 输出目录里的原始文件加一份 JSON | 子命令，stdout 输出 JSON（[§3](fetch.md#3-获取插件协议)） |
| 转换 `transform` | 把条目里已有的文件变成新的一层 | 条目文件的只读路径 | `derived/` 下的约定文件，含 `derived/keyframes/` | stdio 上换行分隔的 JSON-RPC 2.0（[§4](transform-export.md#4-转换与导出插件协议)） |
| 导出 `export` | 把条目和主题页写到笔记软件，可选读回用户改动 | 条目与主题页的只读路径 | 交给宿主写的文件，或经宿主代发的云端调用；读回时返回用户层的值 | 同转换 |

几条贯穿全文的规则：

- **宿主**指 usuyo 的内核进程（`usuyo daemon` 或直接跑的 `usuyo` 命令），负责启动插件、落实权限、传凭据、写资料库。
- **宿主是资料库唯一的写方。** 插件只写宿主分配的临时目录；宿主校验后移进 `raw/<snapshotId>/`、`derived/` 或导出目标，失败就整个丢掉。插件拿不到资料库根目录。
- **插件都是独立进程**，不进内核（[ADR-004](../decisions/ADR-004-rust-tauri.md)）。宿主只认这一份协议；包装命令行、本机 HTTP 服务、MCP 服务、浏览器的差别都在插件内部消化（[§9](wrapping.md#9-适配现有工具)）。
- **入库内容是不可信输入。** 插件输出的文字、HTML、元数据都按数据处理，不当指令执行（[03 §5.3](../research/03-agent-protocols.md#53-对歇存的意义)）。
- **能力**（capability）是获取插件能做的一件事：`fetch`（取单条）、`list`（列出一个来源里的条目）、`unfavorite`（在平台上取消收藏）。`canonicalize` 和 `health` 每个获取插件都必须有，不算能力。

<a id="各节在哪"></a>
## 各节在哪

- [§2 清单](manifest.md#2-清单)：`usuyo-plugin.yaml` 的字段、权限、设置、运行时解析
- [§3 获取插件协议](fetch.md#3-获取插件协议)：运行器与节奏、调用约定、信封与错误码、各子命令、宿主做到哪
- [§4 转换与导出插件协议](transform-export.md#4-转换与导出插件协议)：JSON-RPC、`run`、模型与密钥代发、读回
- [§5 投递接口](delivery.md#5-投递接口)：本机 HTTP、收件文件夹、各种入口
- [§6 凭据](credentials.md#6-凭据)
- [§7 权限与隔离](isolation.md#7-权限与隔离)：沙箱、设备、常驻进程的路径
- [§8 打包、发布与分发](distribution.md#8-打包发布与分发)，附录 A 声明模板
- [§9 适配现有工具](wrapping.md#9-适配现有工具)：包装命令行、本机服务、MCP、浏览器、视觉层
- [§10 契约测试与 plugin-kit](plugin-kit.md#10-契约测试与-plugin-kit)
- [§11 设计取舍](tradeoffs.md#11-设计取舍)
- [§13 待定](open.md#13-待定)
