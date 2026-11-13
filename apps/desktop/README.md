<a id="appsdesktop"></a>
# apps/desktop/

桌面窗口：`web/` 是前端（React + Vite），`src-tauri/` 是 Tauri 2 壳。前端只经本机 HTTP 找内核，不用 Tauri 的 `invoke`；Tauri 接口只许出现在 `web/src/shell/`。决策见 [ADR-002](../../docs/decisions/ADR-002-frontend-stack.md)、[ADR-004](../../docs/decisions/ADR-004-rust-tauri.md)。

<a id="出一个能双击打开的应用"></a>
## 出一个能双击打开的应用

```
just app
# 等同于：bun run build
```

出来的是 `apps/desktop/src-tauri/target/release/bundle/macos/usuyo.app`，没有签名，只在编它的这台 Mac 上直接双击打开。要先装好 Rust、bun（`bun install` 装前端依赖）和 `cargo install tauri-cli`。

这一条命令会先跑 `src-tauri/stage-core.sh release`：编出内核 `usuyo` 和预装的网页插件，放进 `src-tauri/binaries/`、`src-tauri/core-plugins/`（都不进 git），再编前端，最后打包。`cargo clippy` 之类直接编壳的命令也要先跑一次这个脚本。

<a id="打开以后"></a>
## 打开以后

- 壳先找这台机器上已经在跑的内核（`daemon.json` 加握手，见下），有就用它；没有就起包里带的 `usuyo daemon`，等它答应了才放前端连。端口 47615 被别的程序占着时不再起一个，窗口里直接说「端口 47615 被别的程序占着，usuyo 的后台没能启动」。日志在按机器目录的 `logs/daemon.log`（目录 0700，文件 0600）。
- 前端连不上内核或令牌不对（内核重启过）时再问壳要一次，壳重新核对、给新的令牌。
- 退出应用时只停自己起的那个内核（先 SIGTERM，5 秒不走再强停）；本来就在跑的不动。壳被信号杀掉（`kill`、崩溃）时不会走到这一步，它起的内核留着继续跑；下一次启动壳照 `daemon.json` 加握手找到它、接着用，不另起一个。内核没有「壳不在了就自己退」的逻辑。
- 主窗口默认 1280×800，最小 720×480。
- 还没有资料库时建在「文稿」下的 `usuyo` 文件夹，空的那一页右下角写着路径。设了 `USUYO_HOME`（另一套按机器目录，比如测试）时建在 `$USUYO_HOME/library`，不碰「文稿」。
- 内核端口固定 47615：窗口的 CSP 只放行它。

<a id="开发"></a>
## 开发

```
bun install                              # 仓库根目录，装前端依赖
bun run dev                              # 带窗口的开发窗口：编守护进程、起 Vite、开窗口
just dev                                 # 或：一个终端起内核
bun run --cwd apps/desktop/web dev       # 另一个终端：前端，浏览器打开 http://localhost:5173
bun run check                            # Rust lint + test，前端 typecheck + lint + test
```

开发时前端不直接找内核：页面请求同源的 `/api/...`，Vite 的代理转给 `http://127.0.0.1:47615`（`USUYO_PORT` 可改），在服务端带上令牌（读按机器目录里的 `api-token`，认 `USUYO_HOME`），去掉浏览器的 `Origin`。代理只接这个开发服务器自己页面发的同源请求，令牌换了就先握手（`proxy/proof.ts`），端口上不是内核时回 502 并说明，不带令牌过去。内核没在跑时代理回 502，前端换成样例库（`VITE_USUYO_MOCK=1` 一直用样例库，地址栏带 `?mock=empty` 从空的开始）。

<a id="和内核之间的门"></a>
## 和内核之间的门

- 除了 CORS 预检和握手，每个请求都要带 `Authorization: Bearer <令牌>`；令牌在按机器目录的 `api-token`（0600）。内核每次启动生成一个新的，正常退出时连同 `daemon.json` 一起去掉；漏出去的令牌跟着那一次启动作废。打包的应用里由壳读出来，经 `src/shell/` 交给前端。
- 发令牌以前先握手，确认端口上是这次启动的内核，令牌不上线路：客户端往 `POST /handshake` 发 `{"nonce": "<64 位十六进制随机数>"}`，内核回 `{"proof": "<十六进制>"}`，是以令牌为密钥、对 `usuyo-daemon-proof/v1\n127.0.0.1:<端口>\n<随机数>` 做的 HMAC-SHA256；客户端用自己读到的令牌核对。壳、命令行（`usuyo_api::handshake::prove`，同一份代码）和 Vite 代理（`proxy/proof.ts`）都这样做。握手也守 Host、Origin 的规则（扩展的来源也认），请求体最多 256 字节，只回证明。投递端（浏览器扩展、油猴脚本）另带 `client`，值是它的投递令牌 `<id>.<秘密>` 里 `.` 前面的 id，证明用整个投递令牌当密钥算；不带 `client` 就是用主令牌。
- 带 `Origin` 的还得在名单里：只有桌面壳的 webview（`tauri://localhost`，Windows 是 `http://tauri.localhost`）。开发服务器的来源不在名单里。同一个监听口上的投递接口（`/v1/…`）反过来：只认浏览器扩展的来源（`chrome-extension://`、`moz-extension://`、`safari-web-extension://`）或不带 `Origin`，窗口的来源在那里是 403；投递令牌只能进 `/v1/ping`、`/v1/deliver`、`/v1/batches`。完整的规则见 [SPEC 本机接口](../../docs/spec/local-api.md#本机接口第-2-层11-11-照代码)。
- `<img>`、`<video>` 带不了请求头，媒体地址自带 `k`，只对 `/media/<这一条>/` 有效：守护进程每次启动随机生成一个密钥，按条目 id 算出各条的 `k`，漏出去一个地址只看得到那一条。窗口和开发页面对 `127.0.0.1` 都是跨站，所以媒体的 `Cross-Origin-Resource-Policy` 留着 `cross-origin`。事件流 `GET /events` 用 `fetch` 读，好带令牌。
- 「打开原帖」只开 `http`、`https`：前端先查（`src/shell/`），打包的应用里经 opener 插件交给系统浏览器，`capabilities/default.json` 也只放行这两种。
- 窗口的 CSP 在 `src-tauri/tauri.conf.json`：只许自己的资源和 `http://127.0.0.1:47615`。脚本只许自己的文件；样式另外放行行内的（`style-src` 带 `'unsafe-inline'`），因为 React Aria 这类组件库运行时会往页面里插 `<style>`，不放行会被挡掉。平台来的字只当文字显示，不会变成样式。

<a id="登录窗口"></a>
## 登录窗口

- 「获取插件」页点「登录」，前端只把站点名交给壳（`src/shell/` 的 `openLogin`）。壳自己问内核 `GET /sites`，开一个只装那个站点的窗口：上面一条是我们的页面（`login-bar.html`），下面是站点的登录页。每个站点自己一份网页数据：macOS 上是按站点名算的数据仓库标识（要 macOS 14 以上），Windows、Linux 上是按机器目录下 `webviews/<标识>/`。
- 窗口默认 480×780（最小 360×480），上面那条 52 像素高。你在窗口里自己登录。壳在后台线程每秒读一次这个站点的 cookie（含 HttpOnly；Windows 上在主线程读会卡死，所以不在主线程）；声明的 cookie 名都出现了、值不空（没声明名字的站点等你点上面那条的「登录好了」），就只留属于这个站点（站点本身、它的上级域名，加清单里声明的 `domains`）且没过期的，自己 `PUT /sites/{site}/login` 交给内核，成了就关窗口。交不成时那一条上写一句，10 秒后再试。内核收到以后还会再筛一遍。交过去的每个 cookie 有名字、值、域名、路径、过期时间、`httpOnly`、`secure`、`sameSite`。前端从内核的 `plugins-updated` 事件知道登录好了；cookie 的值不经过前端，壳不写日志。
- 「退出登录」：前端 `DELETE /sites/{site}/login`，再让壳关掉那个站点的窗口、删掉它那份网页数据。
- 访客身份（站点给任何没登录的访客的 cookie，不是账号）在插件清单的 `visitors` 里声明（站点、名字、要打开的 https 页、要等的 cookie 名、域名）；前端「取访客身份」只把站点名交给壳（`openVisitor`），壳问内核 `GET /visitors` 拿到那一页。用同一套窗口，不登录，网页数据另放一份 `visitor:<站点>`，和同一主机的账号窗口分开：macOS 上是另一个数据仓库标识，Windows、Linux 上是另一个目录，一个站点的账号和访客身份互相看不到对方的 cookie。声明了 cookie 名的，都出现且值不空就算取到；没声明名字的，等这个站点的 cookie 名连着 3 次（每秒一次）不变、且至少有一个，就算页面生成完了。取到后壳 `PUT /visitors/{site}` 交给内核（内核存成钥匙串里的 `visitor:<站点>`，和账号的 `site:` 分开），成了关窗口；不用点「登录好了」。「换一份」时前端给 `replace`，壳先关掉开着的窗口、清掉那份网页数据，页面才会给新的一份。
- 站点的网页拿不到壳的命令：Tauri 只让本地页面调自定义命令。
- 调试版对着本机测试页试：`USUYO_HOME=<临时目录> USUYO_PORT=<别的端口>` 起内核和壳，`USUYO_OPEN_LOGIN=<站点>` 一启动就开那个站点的登录窗口，`USUYO_CLEAR_LOGIN=<站点>` 清掉它的网页数据。

<a id="还没做"></a>
## 还没做

- 托盘、全局快捷键。
- 壳被杀以后它起的内核不会自己退出（见上）。
- Windows 上退出时直接强停内核（还没有 SIGTERM 那样的收尾）。
- 没接 Paraglide、CodeMirror、画布、Playwright；图标是灰色占位图。
