# apps/desktop/

桌面窗口：`web/` 是前端（React + Vite），`src-tauri/` 是 Tauri 2 壳。前端只经本机 HTTP 找内核，不用 Tauri 的 `invoke`；Tauri 接口只许出现在 `web/src/shell/`。决策见 [ADR-002](../../docs/decisions/ADR-002-frontend-stack.md)、[ADR-004](../../docs/decisions/ADR-004-rust-tauri.md)。

## 出一个能双击打开的应用

```
just app
# 等同于：cd apps/desktop/src-tauri && cargo tauri build --bundles app
```

出来的是 `apps/desktop/src-tauri/target/release/bundle/macos/usuyo.app`，没有签名，只在编它的这台 Mac 上直接双击打开。要先装好 Rust、pnpm（`just init` 装前端依赖）和 `cargo install tauri-cli`。

这一条命令会先跑 `src-tauri/stage-core.sh release`：编出内核 `usuyo` 和预装的网页插件，放进 `src-tauri/binaries/`、`src-tauri/core-plugins/`（都不进 git），再编前端，最后打包。`cargo clippy` 之类直接编壳的命令也要先跑一次这个脚本。

## 打开以后

- 壳先找这台机器上已经在跑的内核（`daemon.json` 加握手，见下），有就用它；没有就起包里带的 `usuyo daemon`，等它答应了才放前端连。端口 47615 被别的程序占着时不再起一个，窗口里直接说「端口 47615 被别的程序占着，usuyo 的后台没能启动」。日志在按机器目录的 `logs/daemon.log`（目录 0700，文件 0600）。
- 前端连不上内核或令牌不对（内核重启过）时再问壳要一次，壳重新核对、给新的令牌。
- 退出应用时只停自己起的那个内核（先 SIGTERM，5 秒不走再强停）；本来就在跑的不动。
- 还没有资料库时建在「文稿」下的 `usuyo` 文件夹，空的那一页右下角写着路径。设了 `USUYO_HOME`（另一套按机器目录，比如测试）时建在 `$USUYO_HOME/library`，不碰「文稿」。
- 内核端口固定 47615：窗口的 CSP 只放行它。

## 开发

```
just dev          # 一个终端：内核
cd apps/desktop/web
pnpm install
pnpm dev          # 另一个终端：前端，浏览器打开 http://localhost:5173
pnpm tauri dev    # 或者带窗口运行（壳自己找或起内核）
pnpm typecheck && pnpm lint && pnpm test && pnpm build
```

开发时前端不直接找内核：页面请求同源的 `/api/...`，Vite 的代理转给 `http://127.0.0.1:47615`（`USUYO_PORT` 可改），在服务端带上令牌（读按机器目录里的 `api-token`，认 `USUYO_HOME`），去掉浏览器的 `Origin`。代理只接这个开发服务器自己页面发的同源请求，令牌换了就先握手（`proxy/proof.ts`），端口上不是内核时回 502 并说明，不带令牌过去。内核没在跑时代理回 502，前端换成样例库（`VITE_USUYO_MOCK=1` 一直用样例库，地址栏带 `?mock=empty` 从空的开始）。

## 和内核之间的门

- 除了 CORS 预检和握手，每个请求都要带 `Authorization: Bearer <令牌>`；令牌在按机器目录的 `api-token`（0600）。内核每次启动生成一个新的，正常退出时连同 `daemon.json` 一起去掉；漏出去的令牌跟着那一次启动作废。打包的应用里由壳读出来，经 `src/shell/` 交给前端。
- 发令牌以前先握手，确认端口上是这次启动的内核，令牌不上线路：客户端往 `POST /handshake` 发 `{"nonce": "<64 位十六进制随机数>"}`，内核回 `{"proof": "<十六进制>"}`，是以令牌为密钥、对 `usuyo-daemon-proof/v1\n127.0.0.1:<端口>\n<随机数>` 做的 HMAC-SHA256；客户端用自己读到的令牌核对。壳、命令行（`usuyo_api::handshake::prove`，同一份代码）和 Vite 代理（`proxy/proof.ts`）都这样做。握手也守 Host、Origin 的规则，请求体最多 256 字节，只回证明。
- 带 `Origin` 的还得在名单里：只有桌面壳的 webview（`tauri://localhost`，Windows 是 `http://tauri.localhost`）。开发服务器的来源不在名单里。
- `<img>`、`<video>` 带不了请求头，媒体地址自带 `k`，只对 `/media/<这一条>/` 有效：守护进程每次启动随机生成一个密钥，按条目 id 算出各条的 `k`，漏出去一个地址只看得到那一条。窗口和开发页面对 `127.0.0.1` 都是跨站，所以媒体的 `Cross-Origin-Resource-Policy` 留着 `cross-origin`。事件流 `GET /events` 用 `fetch` 读，好带令牌。
- 「打开原帖」只开 `http`、`https`：前端先查（`src/shell/`），打包的应用里经 opener 插件交给系统浏览器，`capabilities/default.json` 也只放行这两种。
- 窗口的 CSP 在 `src-tauri/tauri.conf.json`：只许自己的资源和 `http://127.0.0.1:47615`。脚本只许自己的文件；样式另外放行行内的（`style-src` 带 `'unsafe-inline'`），因为 React Aria 这类组件库运行时会往页面里插 `<style>`，不放行会被挡掉。平台来的字只当文字显示，不会变成样式。

## 还没做

- 登录窗口、托盘、全局快捷键：`src-tauri/src/main.rs` 里留了 TODO。
- Windows 上退出时直接强停内核（还没有 SIGTERM 那样的收尾）。
- 没接 Paraglide、CodeMirror、画布、Playwright；图标是灰色占位图。
