<a id="adr-002-桌面前端技术栈"></a>
# ADR-002 桌面前端技术栈

日期：2026-10-23 · 状态：proposed · 2026-11-11 照代码修订

本条不涉及桌面壳。壳 11-11 换成 Tauri 2，见 [ADR-004](ADR-004-rust-tauri.md)（取代 [ADR-001](ADR-001-stack.md) 的 MyGo）；Tauri 的接口只许出现在 `apps/desktop/web/src/shell/`。

<a id="背景"></a>
## 背景

前端是桌面应用的 web 部分，跑在系统 webview 里，只通过本地接口和内核通信。它要承载：上万条的列表与图墙、视频与口播文字联动、Markdown 主题页编辑、画布、默认双语。这个项目主要由 Coding Agent 编写。界面对象与动作见 [产品模型 §10](../BUILD.md#4-界面清单)。

选型原则：每一层选最新的稳定版本和最符合需求的方案，不为兼容旧工具链降级。界面的最终效果与形式在设计阶段一起定，本条只定底层。

<a id="决定"></a>
## 决定

| 层 | 选择 | 定下它的约束 | 放弃的 |
|---|---|---|---|
| 应用形态 | React 19 + React Compiler + Vite 8 单页应用，TypeScript 7，pnpm workspace，放在 `apps/desktop/web`。React Compiler 11-11 这样接：`@vitejs/plugin-react` 的 `reactCompilerPreset()` 交给 `@rolldown/plugin-babel` 跑（`vite.config.ts`） | 桌面不需要服务端渲染；单页应用不绑定任何壳 | Next.js：看看收藏用它，但服务端渲染和路由约定在桌面里是负担。Svelte 5、Solid：体积更小，但 react-konva、React Aria、CodeMirror 封装和 Agent 的熟练度都偏向 React |
| Markdown 编辑 | CodeMirror 6 | 文件是真相，编辑器必须原样读写任意 Markdown。基于 ProseMirror 的编辑器加载时会把文档规范化，用户在 Obsidian 里写的格式会被改掉 | TipTap、Milkdown、Lexical |
| 交互组件 | React Aria Components | 默认多语言：组件自带多语言文案，日期、数字按地区格式化，支持从右到左布局；键盘与读屏支持完整 | Base UI、Radix：可访问性好，但本地化要自己补 |
| 样式 | Tailwind CSS 4 + CSS 变量设计令牌 | 令牌在 `:root` 定义浅色、深色两套，用 `@theme` 暴露成工具类 | CSS Modules：可行，但组件多了令牌引用会分散 |
| 画布 | 自建数据模型；文件格式以 JSON Canvas 1.0 为基础；渲染层在设计阶段按要的效果定，候选 Konva 与 PixiJS v8 的 WebGPU 渲染 | 一条可以属于多个相交的分组，现成画布库的分组都是树，见调研 [04 §6.1](../research/04-tech-options.md#61-画布库对比) | tldraw：专有许可、要 key、带遥测。Excalidraw、React Flow：单父分组 |
| 服务端状态 | TanStack Query | 本地接口的读取、缓存、失效 | 自写 fetch 缓存 |
| 界面状态 | Zustand | 选中、视图、面板这类本地状态 | Redux |
| 长列表 | TanStack Virtual | 上万条的列表与图墙 | 无 |
| 路由 | TanStack Router | 页面不多，但搜索参数要类型安全 | React Router |
| 文案 | Paraglide JS 2；日期、数字交给 `Intl` | 编译期生成类型安全的消息函数，只打包用到的消息 | i18next，作为回退 |
| 图标 | lucide-react | 全局约定 | 自绘图标 |
| 动效 | Motion，遵守系统的「减少动态效果」 | 无 | 无 |
| 测试 | Vitest + Playwright | 单元与组件测试；端到端跑在开发服务器上 | 无 |
| 静态检查 | oxlint，带类型检查（`oxlint --type-aware`，靠 `oxlint-tsgolint`；11-11） | 快；类型感知的规则走 tsgolint，和 TypeScript 7 对得上（推断，代码里没写理由） | ESLint |

<a id="和内核的接口"></a>
## 和内核的接口

1. 前端只通过本地 HTTP 接口和事件流与内核通信。应用代码里不调用壳生成的绑定。事件流是 server-sent events（`GET /events`）：要带 `Authorization` 头，`EventSource` 带不了，所以用 `fetch` 读流，断了按 1 秒到 30 秒退后重连，重连上时把数据全部重拉一次（11-11，原写 WebSocket）。
2. 窗口、菜单、托盘、全局快捷键、文件对话框这些壳能力，集中在一个薄的 `shell/` 模块后面。换壳只改这一处。
3. 媒体由 daemon 通过 localhost 提供，支持 HTTP Range 请求，不用 `file://`。原因：WKWebView 在 http 源下读本地文件受限，视频拖动进度也需要 Range。这是交给内核接口的要求。
4. 本地接口与 MCP 工具共用一份契约定义，见调研 [01 启示 8](../research/01-comparable-products.md#4-对歇存的启示)。11-11 这份是 `apps/desktop/web/src/api/endpoints.ts`：路径和类型只写在这一个文件里，改接口先改它，Rust 那边照它做（[SPEC 本机接口](../SPEC.md#本机接口第-2-层11-11-照代码)）；MCP 还没做。

<a id="待验证"></a>
## 待验证

- 在画布上编辑中文：文字编辑切到 DOM 覆盖层，检查输入法组合事件。
- 视频在 WKWebView、WebView2、WebKitGTK 里通过 localhost Range 播放。

<a id="版本"></a>
## 版本

2026-10-23 与 2026-10-28 从 npm 查到的最新版本（11-11 实际用的版本以 `apps/desktop/web/package.json` 为准）：

| 包 | 版本 |
|---|---|
| react | 19.3.1 |
| babel-plugin-react-compiler | 1.0.0 |
| @vitejs/plugin-react | 6.1.2 |
| vite | 8.3.5 |
| typescript | 7.0.2 |
| tailwindcss | 4.3.3 |
| react-aria-components | 1.22.1 |
| @tanstack/react-query | 5.104.5 |
| @tanstack/react-router | 1.170.46 |
| @tanstack/react-virtual | 3.14.22 |
| zustand | 5.0.16 |
| konva / react-konva | 10.9.0 / 19.3.2 |
| @codemirror/view | 6.43.15 |
| @inlang/paraglide-js | 2.29.1 |
| lucide-react | 1.61.0 |
| motion | 14.6.0 |
| vitest | 5.0.5 |
| @playwright/test | 1.64.1 |

<a id="影响"></a>
## 影响

- CLAUDE.md 里「画布库待定」改为指向本条。
- 壳的那条 ADR（[ADR-004](ADR-004-rust-tauri.md)）要确认壳能提供 `shell/` 模块需要的能力：全局快捷键、托盘、拖入文件、原生菜单。11-11 `shell/` 有了：问壳要接口地址和令牌、只开 `http`、`https` 的「打开原帖」；托盘、全局快捷键、登录窗口还没做（`apps/desktop/README.md`）。
- 11-11 还没接：Paraglide、CodeMirror、画布、Motion、Playwright。
