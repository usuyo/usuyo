<a id="6-凭据"></a>
## 6. 凭据

做到的见 [§3.13](fetch.md#313-宿主和-plugin-kit-现在做到的11-11)：只有登录窗口这一种来源、只传 `credential: cookies` 的站点；`credentialsUpdate` 写回这次交出去的站点，存进 `usuyo.fetch.json` 前去掉。

- **只由宿主在调用时传入**，只传清单 `sites` 里声明、这个能力用到的站点；`login: none` 的能力不传任何平台账号凭据。插件自己的密钥设置不算平台凭据，另经 `USUYO_SECRETS` 传（[§2.11](manifest.md#211-插件设置-settings)）。
- **访客身份不是账号凭据**（[§2.5.1](manifest.md#251-访客身份-visitors)）：只传给列了它的能力，经 `USUYO_VISITOR`，文件和 `USUYO_CREDENTIALS` 同一个形状、同样 0600、调用结束即删；和同一主机的账号分开存、分开传，`credentialsUpdate` 不写回它。
- **这些限制靠沙箱的文件读权限落实。** 在文件读取只能「仅告知」的平台上（Windows，以及 Linux 上 bubblewrap 和 Landlock 都不可用时），任何插件都可能读到别的调用的凭据文件、别的站点的资料目录和日常浏览器的 cookie 库，这几条只是约定，不是隔离；界面的权限页按 [§7](isolation.md#7-权限与隔离) 的三档如实标出。
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

`storageState` 用 Playwright storageState 的形状，Playwright、chromedp 都能直接装进浏览器。只传和 `site` 做域名匹配的 cookie（站点本身和它的上级域名，不含公共后缀），加上 `domains` 显式列出的（[§2.5](manifest.md#25-登录站点-sites)）。

- **插件不自己读浏览器 cookie 库，不持久保存凭据**：不写进 `${CACHE}` 或 `${HOME}`，不写日志，不发到声明以外的主机。
- **会话刷新。** 插件不主动调平台的会话刷新接口，被包装的工具有自动刷新的要关掉；有的平台刷新后旧 cookie 就失效，例如 B站 的 Web 端 Cookie 刷新（推断，待核实），刷新了又被宿主丢掉，用户存的登录态就没用了。调用期间平台自己换了会话 cookie 时，信封可以带 `credentialsUpdate`（`storageState` 的形状，只含声明站点的 cookie），宿主只写回专用资料目录或导入的 cookie 文件，不写回用户的日常浏览器。`USUYO_COOKIES_TXT` 是可写的（yt-dlp 会把 cookie 写回 `--cookies` 指向的文件），调用结束后丢弃，宿主不从它读回。
- **凭据从哪来，用户按站点选**（[06 §11](../research/06-platform-access-and-account-risk.md#11-agent-接入层)；默认是 usuyo 的登录窗口，日常浏览器的登录状态要用户给这个站点选了才读，[PRD 7.2](../PRD.md#72-获取插件)）：

| 来源 | 宿主怎么做 | 已知问题 |
|---|---|---|
| 复用日常浏览器 | 调用时从用户选的浏览器读这个站点的 cookie，宿主不另外保存，只写进这次调用的凭据文件，调用结束即删；macOS 上首次读 Chrome 要用户在钥匙串弹窗里允许 | 浏览器和插件同机同 IP，最接近用户平时的样子；`browser-profile` 类插件用不了日常资料目录（Chrome 136 起自动化工具连不上默认资料目录），宿主把 cookie 装进专用资料目录时，可能被判为同一账号出现在另一套指纹上 |
| 专用资料目录 | 宿主为这个站点建一个浏览器资料目录，用普通模式的 Chrome 打开 `loginUrl` 让用户登录，按 `sites.cookies` 判断登录完成，再经 CDP 导出 cookie：CDP 走 `--remote-debugging-pipe`，不开 TCP 端口（开了的话本机任何进程都能连上去接管会话），导出后立即关闭这个浏览器；`browser-profile` 类插件直接拿到这个目录 | 要单独登录一次；同一目录同一时间只给一个进程用，宿主加锁（Easel 的资料目录锁），交接见下 |
| cookie 文件 | 用户导入 Get-cookies.txt-LOCALLY 一类扩展导出的文件 | 过期要用户重新导出 |

- **资料目录按站点建，同一站点的插件共用**，用户每个站点只登录一次。锁由用到它的进程持有：一次一进程的调用到进程结束；常驻进程从第一次用到起，调用之间可以让浏览器开着。宿主自己导出 cookie 也要拿这把锁。运行器换到需要同一目录的下一个插件（或宿主自己要用）时，先让持有锁的常驻进程退出：关闭 stdin，5 秒后杀进程组；最多等锁 10 秒，等不到就给这个插件记宿主错误 `profile-busy`，换下一个。每次把目录交给插件前，宿主检查并清掉目录里新出现的浏览器扩展，免得一个插件留下的扩展影响下一个插件和用户在这个目录里的登录。
- 首次登录不在自动化控制的浏览器里做：有的站点不接受在 Playwright 控制的 Chrome 里登录，macOS 上自动化启动的 Chrome 也读不到钥匙串加密的 cookie（kali-publish 的记录，见 [06 §3](../research/06-platform-access-and-account-risk.md#3-用户点名的项目)）。
- `credential: app` 的站点（驱动已登录的微信客户端、手机 App），宿主什么都不传；界面照样写「会用到你在微信里登录的账号」。`site` 写这个平台账号的站点（例如 `weixin.qq.com`、`www.douyin.com`），和浏览器类插件写同一个 `site` 没关系，`credential` 不同；登录状态由插件在 `health` 的 `login[]` 里报（[§3.10](fetch.md#310-health)）。
