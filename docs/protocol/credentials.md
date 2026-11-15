<a id="6-凭据"></a>
## 6. 凭据

登录状态只存在 usuyo 自己的地方：登录窗口和站点窗口各自那份网页数据，和系统钥匙串（Linux 上暂时是按机器目录里 0600 的 `secrets.json`，[按机器的目录](../spec/machine.md#按机器的目录)）；只在调用时交给获取策略选中的、列了那个站点的方法（[§2.3.2](manifest.md#232-获取策略)）；不从别的浏览器读，也不导出。

做到的见 [§3.13](fetch.md#313-宿主和-plugin-kit-现在做到的11-11)：`credential: cookies` 的站点有登录窗口和 cookie 文件两种来源（下表），只传这种站点；`credentialsUpdate` 写回这次交出去的站点，存进 `usuyo.fetch.json` 前去掉。

- **只由宿主在调用时传入**，只传清单 `sites` 里声明、这次的方法在这次调用的平台上列了的站点（[§2.3.1](manifest.md#231-方法-methods)：`perPlatform` 写了这个平台就照它的 `login`、`sites`；平台认不出时照方法所有层合起来；列清单、列来源、按 `ref` 取照插件的第一个平台）；`login: none` 的方法不传任何平台账号凭据，`canonicalize`、`health`（不带 `--probe`）也不传。插件自己的密钥设置不算平台凭据，另经 `USUYO_SECRETS` 传（[§2.11](manifest.md#211-插件设置-settings)）。
- **访客身份不是账号凭据**（[§2.5.1](manifest.md#251-访客身份-visitors)）：只传给列了它的方法，经 `USUYO_VISITOR`，文件和 `USUYO_CREDENTIALS` 同一个形状、同样 0600、调用结束即删；和同一主机的账号分开存、分开传，`credentialsUpdate` 不写回它。
- **这些限制靠沙箱的文件读权限落实。** 在文件读取只能「仅告知」的平台上（Windows，以及 Linux 上 bubblewrap 和 Landlock 都不可用时），任何插件都可能读到别的调用的凭据文件、别的站点的资料目录和日常浏览器的 cookie 库，这几条只是约定，不是隔离；界面的权限页按 [§7](isolation.md#7-权限与隔离) 的三档如实标出。
- **形式**：`USUYO_CREDENTIALS` 指向一个临时 JSON 文件（权限 0600，放在这次调用的临时目录里，调用结束即删除）；`credential: cookies` 时另有 `USUYO_COOKIES_TXT`，内容相同、格式是 Netscape cookies.txt。常驻模式下路径放在每个请求的 `context` 里。

```json
{
  "sites": [{"site": "www.douyin.com", "source": "login-window"}],
  "storageState": {
    "cookies": [{"name": "sessionid", "value": "…", "domain": ".douyin.com", "path": "/", "expires": 1793000000, "httpOnly": true, "secure": true, "sameSite": "Lax"}],
    "origins": [{"origin": "https://www.douyin.com", "localStorage": [{"name": "…", "value": "…"}]}]
  }
}
```

`sites[].source` 是这份登录状态从哪来：`login-window`（登录窗口）或 `cookie-file`（你导入的 cookie 文件）。`storageState` 用 Playwright storageState 的形状，Playwright、chromedp 都能直接装进浏览器。只传和 `site` 做域名匹配的 cookie（站点本身和它的上级域名，不含公共后缀），加上 `domains` 显式列出的（[§2.5](manifest.md#25-登录站点-sites)）。

- **插件不自己读浏览器 cookie 库，不持久保存凭据**：不写进 `${CACHE}` 或 `${HOME}`，不写日志，不发到声明以外的主机。
- **会话刷新。** 插件不主动调平台的会话刷新接口，被包装的工具有自动刷新的要关掉；有的平台刷新后旧 cookie 就失效，例如 B站 的 Web 端 Cookie 刷新（推断，待核实），刷新了又被宿主丢掉，用户存的登录态就没用了。调用期间平台自己换了会话 cookie 时，信封可以带 `credentialsUpdate`（`storageState` 的形状，只含声明站点的 cookie），宿主只写回钥匙串里存着的那份（来源照旧记着），不写回你导入的那个文件，也不写回别的浏览器。`USUYO_COOKIES_TXT` 是可写的（yt-dlp 会把 cookie 写回 `--cookies` 指向的文件），调用结束后丢弃，宿主不从它读回。
- **凭据从哪来**（[06 §11](../research/06-platform-access-and-account-risk.md#11-agent-接入层)）：默认是 usuyo 的登录窗口；`cookies` 站点也可以由你从文件导入。**usuyo 不读其他浏览器的数据**：不读你日常浏览器的 cookie 库、资料目录和钥匙串里它们的密钥，所以没有「复用日常浏览器」这种来源（2026-11-15 定）；日常浏览器里的登录状态要用，由你自己导出成文件再导入，读哪些、交出哪些都在你手里。

| 来源 | 宿主怎么做 | 已知问题 |
|---|---|---|
| 专用资料目录 | 宿主为这个站点建一个浏览器资料目录，用普通模式的 Chrome 打开 `loginUrl` 让用户登录，按 `sites.cookies` 判断登录完成，再经 CDP 导出 cookie：CDP 走 `--remote-debugging-pipe`，不开 TCP 端口（开了的话本机任何进程都能连上去接管会话），导出后立即关闭这个浏览器；`browser-profile` 类插件直接拿到这个目录 | 要单独登录一次；同一目录同一时间只给一个进程用，宿主加锁（Easel 的资料目录锁），交接见下 |
| cookie 文件（11-15 做了） | `credential: cookies` 站点的登录卡上「从文件导入登录状态…」：你自己选一个 Netscape 格式的 cookies.txt（Get-cookies.txt-LOCALLY 一类扩展导出的），窗口把全文交给 `PUT /sites/{site}/login/file`（至多 256 KiB）。宿主在边界上读：七栏、`#HttpOnly_` 开头的照 HttpOnly 收，读不了的说是第几行、哪一栏，不带那一行的字；只留属于这个站点（站点本身、上级域名和 `domains`）、没过期的 cookie；一个都没有、或缺了 `sites[].cookies` 里登录后才有的那几个（导出时没登录、已过期）就不存。存进和登录窗口同一处（钥匙串里 `site:<站点>` 那一项，索引记来源是文件和导入的时间），之后和在登录窗口里登录的一样交给按 cookie 调用的方法，同一张登录卡管理，退出登录即删。文件本身不留副本，值不进日志和报错 | 过期要你重新导出；`browser-profile` 站点（站点窗口自己的网页数据）不认文件，导入的登录状态进不了那份网页数据，在站点窗口里读的方法用不上它 |
| usuyo 的站点窗口（11-14 做了） | 在站点窗口里读的方法（[§2.3.1](manifest.md#231a-在站点窗口里读)）：壳给这个 `browser-profile` 站点开一个窗口，用它自己那份网页数据（macOS 上是 WebKit 的数据仓库，别的系统上是按机器目录 `webviews/` 下的一个目录），你在里面登录。cookie 不读出来、不进钥匙串、不交给插件进程，内核只记登录好的时间和上一回哪种方法用它、找到几条；退出登录时壳删掉那份网页数据，内核删掉记录 | 页面里跑着插件的脚本，和站点自己的代码在同一个页面；脚本只能交链接（只进不出的那一条），读不到 usuyo 的任何东西 |

<a id="61-站点窗口的网页数据与凭据形式"></a>
### 6.1 站点窗口的网页数据与凭据形式

- **一个站点只有一种凭据形式。** 装着的插件对同一个站点（`site`）写的 `credential` 不能一个是 `cookies`、一个是 `browser-profile`。宿主按插件目录的顺序（先 `owner` 后 `repo`，几个插件目录时按目录顺序）收：后面那个和前面已加载的冲突，就不加载，插件列表里写明原因，两边的插件 id、站点、两种形式都在里面。`app` 站点宿主什么都不存，不在这条里。
- **网页数据按名字分，名字不会撞。** 壳里每份网页数据有自己的名字，名字算出标识（macOS 上是数据仓库标识，别的系统上是 `webviews/` 下的目录）：

  | 名字 | 谁用 | cookie 去哪 |
  |---|---|---|
  | `<站点>` | `cookies` 站点的登录窗口 | 读出来，存进钥匙串，调用时交给插件 |
  | `visitor:<站点>` | 访客身份的窗口 | 读出来，存进钥匙串 |
  | `profile:<插件 id>:<站点>` | `browser-profile` 站点，在站点窗口里读的方法 | 不读出来，不交给任何插件 |

  读 cookie 的只有前两种；没有任何一处从 `profile:` 的那份里读 cookie。`site` 是小写主机名，里面没有 `:`，所以三种名字互不相同。
- **一个插件一份。** `browser-profile` 的网页数据属于一个插件：同一个站点上第二个插件声明了 `browser-profile`，得到它自己的空的一份，它的脚本不会在第一个插件已登录的页面里跑，用户要为它再登录一次。内核的登录记录、「退出登录」（先清壳里 `profile:<插件 id>:<站点>` 那份，再忘记录）也按插件分：退出一个，另一个不动。窗口能去的域名（`domains`）只来自拥有这个窗口的插件自己的清单，同一站点别的插件写的 `domains` 不算。
- **已有的数据不迁移。** 在这一条之前，站点窗口用的是站点名那一份，已存在的只有开发机上用 `127.0.0.1` 测试留下的；那份和它在内核里的记录不迁移，留在原处不再用，要的话重新登录一次。

- **资料目录按站点建，同一站点的插件共用**，用户每个站点只登录一次。锁由用到它的进程持有：一次一进程的调用到进程结束；常驻进程从第一次用到起，调用之间可以让浏览器开着。宿主自己导出 cookie 也要拿这把锁。运行器换到需要同一目录的下一个插件（或宿主自己要用）时，先让持有锁的常驻进程退出：关闭 stdin，5 秒后杀进程组；最多等锁 10 秒，等不到就给这个插件记宿主错误 `profile-busy`，换下一个。每次把目录交给插件前，宿主检查并清掉目录里新出现的浏览器扩展，免得一个插件留下的扩展影响下一个插件和用户在这个目录里的登录。
- 首次登录不在自动化控制的浏览器里做：有的站点不接受在 Playwright 控制的 Chrome 里登录，macOS 上自动化启动的 Chrome 也读不到钥匙串加密的 cookie（kali-publish 的记录，见 [06 §3](../research/06-platform-access-and-account-risk.md#3-用户点名的项目)）。
- `credential: app` 的站点（驱动已登录的微信客户端、手机 App），宿主什么都不传；界面照样写「会用到你在微信里登录的账号」。`site` 写这个平台账号的站点（例如 `weixin.qq.com`、`www.douyin.com`），和浏览器类插件写同一个 `site` 没关系，`credential` 不同；登录状态由插件在 `health` 的 `login[]` 里报（[§3.10](fetch.md#310-health)）。
