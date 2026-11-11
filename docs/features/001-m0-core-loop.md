# 001 — 第一条线：收下、存好、变成字、搜到、在窗口里看到

状态：in-progress（2026-11-11 重写。这份写给实现的模型和以后查，不等用户批，见 [CLAUDE.md](../../CLAUDE.md) 开发流程）
关联：[PRD §7](../PRD.md#7-功能) · [SPEC 第 0、1 层](../SPEC.md#0-约定第-0-层) · [插件协议](../plugin-protocol.md) · [ADR-004](../decisions/ADR-004-rust-tauri.md)

## 目标

从粘贴一条链接，到在窗口里看见它、搜到它里面的字，整条线跑通。之后每一条线（平台、转换、思路、写作）都接在这条线上。

## 范围

### 做

- 资料库：照 SPEC 第 0、1 层读写。id 和路径、条目文件夹、manifest、快照、`content.md`、你写的那层（笔记、划线、主题、卡片、成稿）、改动记录、回收站。
- 插件宿主：读插件清单，调 `match`、`canonicalize`、`fetch`、`list`，管超时和输出目录。
- 预装的通用网页获取插件，不含平台知识。
- 任务：收下以后在后台取内容，能重试。
- 索引和搜索：中文分词加拼音，按平台、种类筛。
- 本机接口：收下、列出、读一条、搜索、取媒体（支持 Range）、事件推送。
- 命令行：`usuyo init`、`add`、`get`、`search`、`daemon`、`reindex`。
- 窗口：Tauri 壳加前端骨架，有收下、列表、条目页、搜索四处。

### 不做（后面的线）

- 平台插件和登录、导入收藏：002。
- 口播转文字、图里的字、画面：003。
- 主题、地图、把收藏拼成思路、多角度分析：004。
- 在界面里写笔记、主题页、卡片、成稿、引用：005。
- MCP 和 Agent：006。导出和读回：007。同步：008。

## 仓库怎么分

| 位置 | 管什么 | 依赖 |
|---|---|---|
| `crates/usuyo-contract` | 资料库的类型和 JSON Schema（MIT） | 无 |
| `crates/usuyo-library` | 文件就是真相的那一层：id、路径、读写、改动记录、回收站、op | contract |
| `crates/usuyo-plugin` | 插件协议的类型和宿主（MIT 的协议类型放这里的 `protocol` 模块） | 无 |
| `crates/usuyo-index` | 能重建的索引和搜索 | contract、library |
| `crates/usuyo-core` | 把上面几样串起来：收下、任务、转换的调度 | 上面全部 |
| `crates/usuyo-api` | 本机 HTTP 和事件 | core |
| `crates/usuyo-cli` | `usuyo` 这个可执行文件 | core、api |
| `plugins/web` | 预装的通用网页获取插件 | plugin 的协议类型 |
| `apps/desktop/src-tauri` | 壳 | 无（只启动 daemon） |
| `apps/desktop/web` | 前端，照 [ADR-002](../decisions/ADR-002-frontend-stack.md) | 本机接口 |

平台插件不进这个仓库，放在旁边的 `usuyo-fetch/` 下，一个插件一个仓库（[CLAUDE.md](../../CLAUDE.md) 原则 1）。

## 约定

- 库用 `thiserror` 定义自己的错误类型，可执行文件的入口用 `anyhow` 收口，错误只在边界处理一次。
- 不用 `unsafe`，不压警告；`just check` 要过。
- 每个 crate 只编自己的：`cargo test -p <crate>`。
- 依赖用最新的稳定版。
- 文档和代码对不上时，以 SPEC 为准；SPEC 有缺口就照 [open-decisions §1.8](../open-decisions.md#18-grill-回答) 里用户说过的方向补，并把补的地方写回 SPEC。

## 验收

1. `usuyo init <目录>` 建出 SPEC §1.1 的目录树。
2. `usuyo add <一篇公开网页的链接>` 马上回条目 id；后台取完以后，条目文件夹里有 `manifest.json`、`raw/<snapshotId>/`、`content.md`。
3. 同一条链接再收一次，不新建条目，只多一项 `origins`。
4. `usuyo search <正文里的词>` 和按拼音都能搜到它。
5. 删掉索引目录，`usuyo reindex` 以后搜索结果不变。
6. 打开窗口：能粘贴链接收下，列表里出现，点进去看到正文，搜得到。
7. `just check` 通过。

## 桌面应用（11-11 补）

- 壳管内核：先找在跑的（`daemon.json` 加带令牌的 `/health`），没有就起包里带的 `usuyo daemon`，等它答应了才把地址和令牌经 `src/shell/` 交给前端；退出只停自己起的那个（SIGTERM，5 秒后强停）。内核收到 SIGTERM 和 Ctrl-C 一样收尾。
- 第一次打开：`usuyo daemon --first-library <目录>` 在没有资料库时建一个，壳给的是「文稿」下的 `usuyo`；设了 `USUYO_HOME` 时是 `$USUYO_HOME/library`。预装的网页插件作为资源打进包里，经 `--plugins` 找到。
- 门：除了 CORS 预检，每个请求都带令牌；`Origin` 只用来挡，名单里只有桌面壳的 webview。开发时经 Vite 的代理，代理在服务端带令牌。
- 契约（`endpoints.ts`）：`POST /topics {name}` 按名字建主题（同名回已有的）；`stored` 多了 `partial`，缺的写在 `missing`；`GET /health` 多了 `library`；事件流用 fetch 读，`LibraryEvent` 列出五种事件。
- 怎么出包、怎么开发：[apps/desktop/README.md](../../apps/desktop/README.md)。

## 测试

- library：id 的固定样例（SPEC §0.4 的例子）、文件名规则、front matter 往返后字节相同、回收站进出、op 做了再撤销。
- plugin：用假插件测超时、坏输出、越界路径。
- index：二字词、拼音、筛选。
- 手动：验收 2–6 各跑一遍。
