# apps/desktop/

桌面窗口：`web/` 是前端（React + Vite），`src-tauri/` 是 Tauri 2 壳。前端只经本机 HTTP 找内核，不用 Tauri 的 `invoke`；Tauri 接口只许出现在 `web/src/shell/`。决策见 [ADR-002](../../docs/decisions/ADR-002-frontend-stack.md)、[ADR-004](../../docs/decisions/ADR-004-rust-tauri.md)。

## 运行

```
cd apps/desktop/web
pnpm install
pnpm dev          # 只开前端，浏览器打开 http://localhost:5173
pnpm tauri dev    # 带窗口运行（要先装 Rust）
pnpm typecheck && pnpm lint && pnpm test && pnpm build
```

内核地址默认 `http://127.0.0.1:47615`，用环境变量 `VITE_USUYO_API` 改。内核没启动时页面只显示「内核没在运行」。

## 还没做

- 壳里启动 `usuyo daemon` 作为 sidecar、登录窗口：`src-tauri/src/main.rs` 里留了 TODO。
- 接口路径和类型还没定稿，都在 `web/src/api/endpoints.ts`。
- 没接 Paraglide、CodeMirror、画布、Playwright；视觉样式是占位，图标是灰色占位图。
