<a id="3-获取插件协议"></a>
## 3. 获取插件协议

<a id="31-运行器与节奏"></a>
### 3.1 运行器与节奏

宿主怎么用获取插件，插件作者要知道的部分：

- **身份**：收下一个链接时，宿主在 `match` 命中的插件上按用户排的顺序调 `canonicalize`，取第一个 `match: true` 的结果作为身份（[SPEC §0.4](../spec/conventions.md#04-id)）。`identity.contentId` 要符合平台列表里的格式（[§2.3](manifest.md#23-获取插件的字段)），不符合的按 `invalid-output` 处理（平台列表还没有，宿主现在只查非空、不超过 256 字节、没有控制字符）。
- **顺序**：每个平台按用户排的顺序依次调 `fetch`，失败换下一个，全部失败停在 `link` 并记下各自的原因。默认顺序见本节末尾。不论 `access` 高低都照这个顺序换。平台插件和 `platforms` 含 `web` 的插件都命中时，先把平台插件全部试完，再轮到 `web` 插件。
- **暂停单个插件**：插件返回 `rate-limited` 时，宿主在 `retryAfter` 之内（没给就 15 分钟）不再在这个平台上调它；返回 `blocked` 时，暂停一小时，用户在插件那一行点「现在再试」可以更早去掉，这一条不因这次暂停重试。`retryAfter` 最多算一小时（更长的按一小时）；`platforms` 含 `web` 的插件按链接的主机暂停，别的按平台；`jobs.db` 打开时丢掉比现在晚一小时以上的旧暂停。别的插件照常按顺序试，包括用同一个账号的插件：平台的限流常按接口分开计，换一个插件、换一个接口可能还取得到（推断）。
- **完整度只升不降**：宿主给每个快照记下实际拿到的输出集合（`outputs` 词表里的值）。新快照算不算升级、用户改了「只留文字」之类的设置后要不要再取，比的是这个集合，不比 `full`、`partial` 这两个字样。失败不降级。

**节奏。** 每个平台一组上限，该平台所有获取插件共用（目的是不被限流、能一直用）：

- **计入节奏的调用**：`fetch`、`list` 的一页、`sources`、`unfavorite`。同一平台同一时间只跑其中一个；`interval` 是上一次调用结束到下一次调用开始的最短间隔。一次调用内部的请求怎么排，由插件自己控制，例如视觉层插件在 `list` 的一页里逐条复制链接的间隔。
- **`daily`** 按上面这些调用的次数计，只限批量导入、跟着收和取消收藏；用户手动收下的单条不计入 `daily`，但照样排队、照样隔 `interval`。
- **`batch: false`**：只接受用户一条条收下，不做批量导入和跟着收。
- **不计入节奏的**：`canonicalize` 不占单调用锁，不计 `interval` 和 `daily`，展开短链另有每平台每分钟上限（默认 30 次），所以正在下大视频时，新贴的链接照样几秒内拿到身份；`health` 不计（`--probe` 只在用户点「检查登录」时发一次）；`access: official-export` 且 `network.mode: none` 的能力只读本机文件，不受节奏限制，可以并行。
- **队列顺序**：用户手动收下 > 跟着收 > 批量导入。
- **整机锁和设备锁。** 平台锁之外，宿主再加三把锁，不分平台、不分插件：声明了 `interaction: foreground` 的调用，全机同一时间只跑一个；声明了 `devices: [clipboard]` 的调用，全机同一时间只跑一个；同一台安卓设备同一时间只跑一个调用。原因：「复制链接」一类流程就算全走无障碍接口，也会弹菜单、改剪贴板，两个视觉层插件同时跑会互相弄坏（[§9.5](wrapping.md#95-视觉层流程)）。
- **视觉层调用什么时候跑。** `interaction: foreground` 的调用和用到安卓设备的调用，能不能在跟着收、批量导入里自己跑，每个插件单独设置，三种模式 `anytime`、`idle`（用户没在用时跑，一动键鼠就停）、`manual`（只在用户点开始时跑）。默认按 `interaction` 定：`background` 为 `anytime`，`foreground` 为 `idle`，用到安卓设备的为 `manual`；声明了 `devices: [clipboard]` 的调用，宿主在调用前存下剪贴板、结束后放回（[回答表](../answers.md)；字段名在实现运行器时定）。插件在 `foreground` 的步骤之间发现用户在动键鼠，就发 `waiting`（`user-active`），等用户停手，等不到就回 `blocked`；分辨是不是用户的输入由 plugin-kit 的视觉层 SDK 做，插件自己合成的事件会不会被算进去，待核实。安卓设备没连、没解锁、没授权时，插件回 `dependency-missing`，跟着收跳过这一轮，不算失败；宿主不存手机的解锁密码，不替用户解锁。
- **默认值**取已装插件 `pacing` 里最严的一组，用户在设置里可以改。用户改过的值不会被新装插件的建议覆盖，设置里只提示「有插件建议更慢」。

**几种情况运行器怎么做：**

| 情况 | 宿主怎么做 |
|---|---|
| 插件回 `ok: true`、`completeness: partial` | 记下这个快照，每个成功的快照都保留。顺序里后面的插件 `outputs` 能补上 `missing` 的，宿主可以带 `--want=<missing>` 去试它，后面的插件只取缺的部分。默认自动接着试，见本节末尾。插件拿到一部分就回 `partial`，不要回错误 |
| 插件回 `ok: true`、`completeness: link` | 按 `unsupported-input` 处理，换下一个 |
| 插件自报的 `completeness`、`missing` 和实际输出对不上 | 宿主照 [§3.7](#37-fetch-输出) 的规则重算，以宿主算的为准，并记日志 |
| 退出码 `2` 且没有信封 | 记 `invalid-args`，当 bug 记下，换下一个 |
| 没有插件的 `match` 命中，或 `canonicalize` 全部回 `match: false` | 交给 `platforms` 含 `web` 的插件（如果装了）；没装就由宿主照 [SPEC §0.4](../spec/conventions.md#04-id) 自己算出 `web:` 身份建条目，停在 `link` |
| `match` 命中的插件都没回 `match: true`，且有一个 `canonicalize` 因 `network` 或 `timeout` 失败（例如离线时收下的短链） | 不交给 `web` 插件。宿主拿这条链接本身算 `web:` 身份，马上建只有链接的条目（`completeness: link`，还没有快照；之后的快照照记 `inputUrl`），重试记进本机 `jobs.db`（[SPEC §1.1](../spec/library.md#11-目录树)）；之后再调 `canonicalize`，拿到平台身份就照 [SPEC §0.4](../spec/conventions.md#04-id) 并过去，`web-` id 留作别名；跟着收的不建，这一次不算跑完，联网后整次重跑 |
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

**环境变量。** 宿主清空环境后只传下面这些，加上 `permissions.env` 声明的变量（哪些已经传了见 [§3.13](#313-宿主和-plugin-kit-现在做到的11-11)）：

| 变量 | 什么时候有 | 含义 |
|---|---|---|
| `USUYO_PROTOCOL` | 总是 | `1` |
| `USUYO_PLUGIN_DIR` | 总是 | 安装目录 |
| `USUYO_CACHE_DIR` | 总是 | `${CACHE}` |
| `USUYO_MODELS_DIR` | 转换插件的常驻进程 | `${MODELS}`，按机器目录的 `models/<owner>/<repo>/` |
| `USUYO_CALL_ID` | 一次一进程的调用；常驻进程没有，一个进程接好几次调用 | 这次调用的 id，写日志用 |
| `USUYO_LOCALE` | 总是 | 用户界面语言，例如 `zh-CN`；`hint` 按它写 |
| `USUYO_NETWORK` | 宿主查到这台机器开着假 IP 模式的代理时 | 值是 `fake-ip`。这时系统给每个域名都回 `198.18.0.0/15` 里的地址；plugin-kit 的客户端据此放行查出来全在这一段的域名，没有这个变量就照旧拒绝。是不是这种环境由宿主拿固定的公网域名去查 |
| `USUYO_SETTINGS` | 有设置时 | 非密钥设置的 JSON 文件路径（[§3.13](#313-宿主和-plugin-kit-现在做到的11-11)） |
| `USUYO_SECRETS` | 插件声明了 `secret: true` 的设置、且用户填了值时，和 `login` 无关 | 一个 0600 的临时文件，内容只有 `{"<key>": "<value>"}`，调用结束即删 |
| `USUYO_INPUT` | 导入调用（[§3.9](#39-从文件导入)） | `${INPUT}`，导入文件的副本 |
| `USUYO_CREDENTIALS` | 这次调用要带账号、且用户已登录时（[§2.4](manifest.md#24-access-与-login)） | 凭据 JSON 文件路径（[§6](credentials.md#6-凭据)） |
| `USUYO_COOKIES_TXT` | 同上，`credential: cookies` | 同一批 cookie 的 Netscape 格式文件，给 yt-dlp 一类工具的 `--cookies` 用；可写，调用结束后丢弃 |
| `USUYO_VISITOR` | 这次的能力列了 `visitors`、且存着那份访客身份时，和 `login` 无关（[§2.5.1](manifest.md#251-访客身份-visitors)） | 访客身份的 JSON 文件路径，形状和 `USUYO_CREDENTIALS` 一样，`source` 是 `visitor-window`、没有 localStorage；0600，调用结束即删 |
| `USUYO_BROWSER_PROFILE` | `credential: browser-profile` | `${PROFILE}` |
| `USUYO_PROVIDER` | 能力的 `provider` 不是 `none`、用户配了模型时（[§4.3](transform-export.md#43-模型与密钥由宿主代发)） | 这次调用的代发参数，JSON 文件路径，内容是 `{baseUrl, token, model}` |
| `ADB_SERVER_SOCKET` | 声明了 `devices: [adb]` | 宿主管的 adb 服务，`tcp:127.0.0.1:<端口>`（[§7](isolation.md#7-权限与隔离)）；adb 认这个变量 |
| `ANDROID_SERIAL` | 同上 | 用户选定的那台设备的序列号；adb 认这个变量 |
| `HOME` | 总是 | `${HOME}`（[§2.7](manifest.md#27-权限-permissions)），不指向用户主目录：被包装的工具不会读到用户自己的配置，例如 yt-dlp 配置里的 `--cookies-from-browser` |
| `XDG_CONFIG_HOME`、`XDG_CACHE_HOME` | 总是 | `${HOME}/.config`、`${CACHE}` |
| `TMPDIR`、`TMP`、`TEMP` | 总是 | `${TMP}` |
| `HTTP_PROXY`、`HTTPS_PROXY`、`ALL_PROXY` | 有网络权限时 | 指向宿主的本机代理，见下面的「网络出口」 |
| `NO_PROXY` | 有网络权限时 | `localhost,127.0.0.1,::1` |
| `USUYO_PROXY` | 有网络权限时 | 宿主本机代理的 `host:port`，给浏览器的 `--proxy-server` 用 |
| `NODE_USE_ENV_PROXY` | 有网络权限时 | `1`，让 Node 内置的 `fetch` 读代理变量（Node 哪个版本起支持，待核实） |
| `PATH` | 总是 | `${TMP}/bin`（常驻进程是暂存目录的 `bin/`，[§2.13](manifest.md#213-运行时解析)）在最前，再加系统目录 |

**网络出口。**

- 非回环地址的出站只经宿主的本机代理。代理先解析域名再按 `network.hosts` 判断，连接时用解析出来的那个地址，防止 DNS 重绑定；解析结果是回环、RFC 1918 内网、链路本地、ULA 或云元数据地址（169.254.169.254 一类）的一律拒绝，除非 `hosts` 里显式写了带端口的这个地址。
- `hosts` 里显式写的回环地址（例如 `127.0.0.1:5556`）和宿主自己的本机端点（[§4.3](transform-export.md#43-模型与密钥由宿主代发)），由沙箱直接放行到声明的端口，不经代理。原因：Go 的 `http.ProxyFromEnvironment` 对 `localhost` 和回环地址一律不走代理，改 `NO_PROXY` 也没用（已用 Go 1.26 的 `go doc` 和 `httpproxy.useProxy` 源码核对）。
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

还有前缀 `version:`，值是宿主要满足的版本，例 `version:usuyo>=1.3`（[SPEC §0.3](../spec/conventions.md#03-错误模型)）。

插件返回的错误码：

| `code` | 含义 | 运行器怎么办 |
|---|---|---|
| `unsupported-input` | 不是这个插件能处理的链接、文件或来源（例如直播） | 换下一个 |
| `not-found` | 内容不存在、已删除或当前身份看不到；能分清时填 `reason` | 换下一个。只有全部是 `not-found` 且 `reason` 都是 `deleted` 时，条目才记「平台上已看不到」；有 `link-expired` 时提示用户重新分享或重新导出链接；其余记「取不到」并列出原因 |
| `login-required` | 没登录，或登录已过期 | 换下一个；插件那一行提示重新登录 `site` |
| `visitor-required` | 站点要一份访客身份（[§2.5.1](manifest.md#251-访客身份-visitors)）：没有交来，或交来的页面仍不收 | 换下一个；界面写成要取或要换访客身份，不说登录；和 `login-required` 一样不自己暂停 |
| `rate-limited` | 被平台限流 | 换下一个；这个插件在这个平台暂停到 `retryAfter` |
| `blocked` | 平台出现验证页、风控提示或拦截 | 换下一个；这个插件在这个平台暂停一小时，用户点「现在再试」可更早解开（[§3.1](#31-运行器与节奏)） |
| `upstream-changed` | 页面或接口改版，解析不出 | 换下一个；记进插件的最近失败 |
| `network` | 网络不通、DNS、TLS 或插件自己的请求超时 | 换下一个 |
| `dependency-missing` | 缺运行时、外部程序、设备、设置、系统授权、本机服务，或要用户处理的状态 | 换下一个；界面列出 `needs` |
| `invalid-args` | 参数不对，多半是宿主或插件的 bug | 换下一个；记 bug |
| `cancelled` | 收到取消 | 不再换 |
| `internal` | 插件内部错误 | 换下一个 |

宿主自己记的四种：`timeout`（超时被杀）、`crashed`（退出了但没有可解析的信封）、`invalid-output`（信封不符合 schema、输出目录不合规，[§3.6](#36-fetch-与输出目录)）、`profile-busy`（等不到浏览器资料目录，[§6](credentials.md#6-凭据)）。四种都换下一个。

这套码的来源：[02 §15.3](../research/02-adapter-ecosystems.md#153-调用错误与产物) 的建议；wechat-inbox-sync 的失败分类（需要登录、风控、内容不可用、不支持、网络）对应 `login-required`、`rate-limited` 与 `blocked`、`not-found`、`unsupported-input`、`network`；ChubbySkills 的缺依赖、cookie 问题、风控限流、来源失效对应 `dependency-missing`、`login-required`、`rate-limited`、`upstream-changed`；Agent-Reach 把 `NEED_LOGIN`、`RATE_LIMITED`、`PLATFORM_BLOCKED` 当终止码，这里只暂停出错的那个插件，不停止换下一个（[06 §3](../research/06-platform-access-and-account-risk.md#3-用户点名的项目)）。`dependency-missing` 一次列全缺的东西，照 MoneyPrinterTurbo 退出码 10 的做法，让用户一次补齐。

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

- 两种计时同时生效。**总时长**：默认照上表，获取插件用能力里的 `timeout` 改，转换、导出插件用 `permissions.limits.timeout` 改；用户可以在设置里再加长，不设硬上限，下一个大视频、在 CPU 上转写几小时的播客都放得下（宿主现在把能力里超过 2 小时的 `timeout` 按 2 小时算，设置还没有）。**无进度时长**：插件发过 `progress` 之后，120 s 内没有新的 `progress` 就算超时，用来发现卡住的调用；从没发过 `progress` 的调用只受总时长约束。
- 插件发出 `waiting` 后，两种计时都暂停，直到下一个 `progress` 事件，最长等 10 分钟（一次调用里所有 `waiting` 的时间加起来算）；等用户在弹出的窗口里过验证时不会被杀。
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

- 展开短链、去掉跟踪参数、取出平台内容 ID（[SPEC §0.4](../spec/conventions.md#04-id)）。不是自己的链接回 `match: false`，退出码 0，这不是错误。
- 只为展开短链才发请求，只跟随重定向，不带凭据；这个请求不计入平台节奏，另受每平台每分钟的上限（[§3.1](#31-运行器与节奏)）。
- `identity` 的形状和 `fetch` 输出里的一样。`identity.contentId` 照 plugin-kit 平台列表里的格式写（[§2.3](manifest.md#23-获取插件的字段)）。`identity.url` 写规范化后的链接，去掉跟踪参数和会话 token 一类参数，只用于去重和显示。`platform: web` 的身份不带 `contentId`，必带 `url`，宿主再照 [SPEC §0.4](../spec/conventions.md#04-id) 规范化一遍。
- `resolvedUrl` 可选：展开短链后的完整链接，保留全部参数。宿主把用户给的原链接另存（快照里记 `inputUrl`，[SPEC §1.2.1](../spec/library.md#121-manifestjson-v1)），有 `resolvedUrl` 时一并记下，交给 `fetch` 的是这两者之一，不是 `identity.url`（[§3.6](#36-fetch-与输出目录)）。小红书的链接要带 `xsec_token` 才取得到（[06 §1.3](../research/06-platform-access-and-account-risk.md#13-混合路径视觉层拿链接公开页面取内容)），去掉参数就取不到了。

<a id="36-fetch-与输出目录"></a>
### 3.6 fetch 与输出目录

```
<command> fetch "https://v.douyin.com/iRNBho6u/" --out /private/var/folders/xx/usuyo-call-01/out --want text,images,media:video
```

- 第一个参数是收下时拿到的原链接（`canonicalize` 给了 `resolvedUrl` 时用它，同样不去参数），或 `list` 返回的 `ref`（[§3.9](#39-从文件导入)）；接哪种由 `fetch.input` 声明。`identity.url` 只用于去重和显示，不交给 `fetch`。
- `--want` 是宿主想要的东西，取 `outputs` 的子集；不给时按 `outputs` 全取。例如用户开了「收下后直接只留文字」，宿主就不要 `media:video`，插件不必下载视频。
- `--out` 是宿主新建的空目录，绝对路径。插件只在里面写文件，不依赖当前目录；输出 JSON 里的路径都相对它。
- 成功后宿主把整个目录移成 `raw/<snapshotId>/`，没被 JSON 引用的文件（例如平台原始返回）也原样留下；失败就整个丢掉。
- **输出目录的规则**（[§4.2](transform-export.md#42-转换的-run)、[§4.4](transform-export.md#44-导出的-run-与-readback)、[§5.1](delivery.md#51-本机-http-接口)、[§5.2](delivery.md#52-监听文件夹) 都照这条）：宿主在移动前遍历整个目录，不只看 JSON 引用的条目。只接受普通文件和目录；遇到符号链接、`nlink > 1` 的硬链接、FIFO、套接字、设备文件，或名字含 `..`、以 `/` 开头、含控制字符的条目，整次判 `invalid-output` 并丢弃。名字另照 [SPEC §0.6](../spec/conventions.md#06-文本和文件) 查：不能以 `usuyo.` 开头（不分大小写），不能有 `< > : " / \ | ? *`，不以 `.` 开头、不以空格或 `.` 结尾，要是 NFC，不能是 Windows 保留名（带扩展名也算），大小写折叠后不重名，每段不超过 200 字节，相对路径不超过 99 个 UTF-16 单位（条目文件夹以内的 140 减去 `raw/<snapshotId>/`）。整个目录最多 10,000 项（文件加目录）。JSON 里同一个文件只能引用一次，引用的文件要在。JSON 里的路径同样不能有 `..` 或绝对路径。整个目录有大小上限，`fetch` 默认 8 GiB，转换、导出和投递默认 1 GiB，用户可以在设置里改，超了也判 `invalid-output`。原因：插件在沙箱里读不了 `~/.ssh`，但能在 `out` 里建一个指向它的符号链接；目录移进资料库后，索引、导出和 MCP 会以宿主的身份读到目标内容。
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

各字段：

| 字段 | 含义 |
|---|---|
| `ok` | 必填，成功为 `true` |
| `identity` | 没有平台内容 ID 的条目（聊天记录、转发的图片一类）可以省略，或写 `{"platform": "file"}` 不带 `contentId`，省略等于 `{"platform": "file"}`；这样的条目不参与身份层，去重只走内容层（[SPEC §0.4](../spec/conventions.md#04-id)） |
| `textFormat` | 可选，`plain`（默认）或 `markdown` |
| `media` | 对象数组：`path`、`kind`（`video`、`audio`、`subtitle`、`other`）必填，`mime`、`lang`、`durationSec`、`group` 可选。转换插件按 `kind` 编排，靠扩展名猜不可靠。`group`：同一个字符串表示同一段时间线拆开的几个文件（例如分开存的画面和音轨），1–64 个字母、数字或 `. _ : -`，不合的整次 `invalid-output`；没有 `group` 时宿主照 feature 003 [§4.1](../features/003-transforms.md#41-四个转换插件) 按内容配 |
| `missing` 的取值 | 用 `outputs` 的词表，例如 `media:video` |
| `missingReason` | 可选，给用户看的原因，例如「分享页没有给出视频地址」 |
| `evidence.access` | 必填，这次实际用到的那一层。快照里另有一个值 `deliver`，由宿主给投递来的内容填（[§5.1](delivery.md#51-本机-http-接口)），不出现在清单的 `access` 里 |
| `credentialsUpdate` | 可选，调用期间平台换了会话 cookie 时交回新值（[§6](credentials.md#6-凭据)） |
| `evidence.site` | 用了登录时填站点 |
| `evidence.upstream` | 可选，`{name, version}`，被包装工具的实际版本 |
| `evidence.status` | 只在 HTTP 类请求时填 |
| 其余字段 | `kind`、`title`、`author`、`publishedAt`、`text`、`html`、`images`、`platformTags`、`completeness`、`missing`、`evidence.fetchedAt`；时间一律 RFC 3339 带偏移，宿主把 `publishedAt` 换成 UTC 存，偏移不是 `Z` 的另存 `publishedAtOriginal` |

- `completeness` 和 `missing`：`missing` 只写这一条有、这次想要（`--want`）却没交来的；这一条本来就没有的那一样（只有字的帖子没有图和视频）不是缺，不写。`full` 是没有缺的，`partial` 是有缺的，`link` 是只认出了身份。哪一样这一条本来就没有，只有插件知道，宿主不照「想要的 − 交来的」猜；一样可以交来一部分、还缺一部分（九张图下到六张），照样写进 `missing`，原因写进 `missingReason`。宿主只去掉 `missing` 里不在 `--want` 里的，再照剩下的定 `completeness`；按 `text`、`html`、`images`、`media[].kind` 实际交来了哪些看，想要的一样都没交来算 `link`。比较完整度时比的是实际拿到的输出集合（[§3.1](#31-运行器与节奏)）。
- `text` 是平台原文的文字，宿主据此生成 `content.md`：只读副本，总照主快照重写；原样在 `raw/`（[SPEC §1.2.3](../spec/library.md#123-contentmd-和各栏)）。
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
- 条目字段：`url` 或 `ref` 至少一个；`identity`、`title`、`author`、`publishedAt`、`action`（`favorite`、`like`、`share`、`view`、`unknown`）、`actedAt` 能给就给，时间 RFC 3339 带偏移。拿不到加入时间时 `actedAt` 留空，不用抓取时间冒充（[02 §15.5](../research/02-adapter-ecosystems.md#155-批量导入与同步)）。
- 只有 `url` 的条目，宿主按 [§3.1](#31-运行器与节奏) 的顺序去取内容；带 `ref` 的条目，宿主回到这个插件调 `fetch <ref>`。`ref` 由插件自己定，对宿主不透明；宿主为每个 `ref` 记下它来自哪个插件、哪个 `source`，调 `fetch <ref>` 时按这个 `source` 设好 `${INPUT}` 一类的上下文。
- **跟着收（增量）**：每个订阅记下自己列过的身份，只存身份的哈希、不存标题，按机器记在 `jobs.db`，和翻页游标放在一起（推断，[SPEC §1.1](../spec/library.md#11-目录树)）。宿主只收这个订阅没列过的身份：回收站里的跳过，资料库里已有的不新建、只多一项 `origins`（[SPEC §1.7](../spec/library.md#17-删掉)）。所以它列过、你删掉的不会被它带回来，回收站到期移除以后也一样；它没列过的（比如你自己粘贴过、删掉、已经到期移除的），以后它第一次列出来就照新的收（这是代价）。`order: newest-first` 的来源，宿主从第一页往后翻，遇到一整页都是这个订阅列过的就停；`--since` 是这个来源上次跟着收成功跑完的时间，插件拿得到 `actedAt` 时可以据此提前结束、返回 `nextCursor: null`。`unordered` 的来源每次全量翻，频率由用户的订阅设置定。
- 一台机器第一次跑一个订阅，只记下列出来的身份，不收（[PRD 7.3](../PRD.md#73-导入和跟着收) 对个人来源这样建议，这里所有订阅都照它，推断）。这一次跑完才把列出来的记成列过；还没跑完过一次的，下一次仍照第一次跑（推断）。换了 `fetchDevice`，新的那台也是第一次：上一台最后一次跑完到新的那台第一次跑之间新出现的，会漏掉（推断）。
- 批量导入的结果先进预览（新增、已有、可以升级完整度；回收站里的算已有，确认后拿回来，推断），用户确认后才按节奏去取内容（[PRD](../PRD.md) 的批量导入）；跟着收是用户订阅过的来源，新条目直接收下，不再逐次预览。
- 跟着收收下的条目，`origins[]` 里那一项记 `entry: subscription`，另记 `subscription`（订阅 id）；批量导入（`/v1/batches` 和对列表来源的一次性导入）记 `import`，同一批共用一个 `importBatch`（[SPEC §1.2.1](../spec/library.md#121-manifestjson-v1)）。

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
- 调 `list` 之前，宿主把导入文件复制到按机器目录（[SPEC §1.1](../spec/library.md#11-目录树)）下的 `libraries/<libraryId>/imports/<importBatch>/`（能用 clonefile、reflink 或硬链接就不真的复制），`${INPUT}` 和 `USUYO_INPUT` 指向这份副本；来源写 `file:<原文件名>`。导入要先进预览，用户可能几天后才确认，原文件这期间被移走也不影响；副本在这批导入结束或取消后清掉。用户下载目录的绝对路径不进 `ref`，也不随快照写进资料库。
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

- 只在用户打开了这个能力、并确认了这一批之后调用（[§2.6](manifest.md#26-账号写操作)）。
- `verified: true` 表示拿到了平台侧的确认（例如重新读一次已不在收藏夹里）；平台只回了成功、没有确认时写 `false`。照 Easel 读回分档的做法：没有平台侧证据不算完成。
- 已经不在收藏夹里时也回 `ok: true`，`verified: true`。

<a id="312-常驻模式-serve"></a>
### 3.12 常驻模式 serve

要开浏览器或连设备的插件，每次调用都冷启动要好几秒，还会抢同一个资料目录。`run.serve: true` 的插件支持 `<command> serve`：宿主启动一次，经 stdio 发换行分隔的 JSON-RPC 2.0，分帧和 [§4.1](transform-export.md#41-分帧与生命周期) 相同。

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

- `context` 里是这一次调用的文件路径，对应一次一进程时的环境变量：`credentials` ↔ `USUYO_CREDENTIALS`、`cookiesTxt` ↔ `USUYO_COOKIES_TXT`、`browserProfile` ↔ `USUYO_BROWSER_PROFILE`、`settings` ↔ `USUYO_SETTINGS`、`secrets` ↔ `USUYO_SECRETS`、`input` ↔ `USUYO_INPUT`、`provider` ↔ `USUYO_PROVIDER`（这一个是代发参数本身，不是路径）。这些路径都在进程的暂存目录里（[§7.1](isolation.md#71-常驻进程的路径)），凭据和密钥文件在这次响应之后删除。`HOME`、代理等进程级的变量在启动时给定。
- **按站点集合分进程。** `permissions` 按插件声明，沙箱按进程生效，凭据按能力发。所以宿主按「这次调用用到的站点集合」分别起常驻进程，沙箱也按这个集合生成：`login: none` 的能力只发给不带站点的那个进程。抖音的例子里，`fetch` 和 `list` 各一个进程，`list` 那个进程里开着带登录的浏览器，`fetch` 那个进程碰不到它。
- **资料目录的交接。** 带 `browser-profile` 站点的常驻进程从第一次用到这个目录起持有它的锁，调用之间可以让浏览器开着。别的进程要用同一个目录时，宿主先让持有它的进程退出，再交出去（[§6](credentials.md#6-凭据)）。
- 失败返回 JSON-RPC `error`：`code: -32000`，`data` 是 [§3.3](#33-信封与错误码) 的错误对象。
- 宿主同一时间只向一个进程发一个请求；节奏照 [§3.1](#31-运行器与节奏)。常驻进程正忙时，`canonicalize` 改用一次一进程的方式调用，不排在它后面。
- 进度和等待用 `progress`、`waiting` 通知，参数同 stderr 事件，另带 `id`。取消用 `cancel` 通知。
- 空闲 10 分钟后宿主关闭 stdin，插件 5 秒内退出。进程崩溃怎么处理见 [§3.1](#31-运行器与节奏) 的表。
- 一次一进程的子命令仍然必须有；两种方式结果相同，SDK 从同一套处理函数生成。

<a id="313-宿主和-plugin-kit-现在做到的11-11"></a>
### 3.13 宿主和 plugin-kit 现在做到的（11-11）

照 `crates/usuyo-plugin`、`crates/usuyo-core` 写。和上面不一样的以这里为准。

- **找插件**：按机器目录的 `plugins/<owner>/<repo>/<版本>/usuyo-plugin.yaml`（内核可以配几个根目录，同一个 id 以先找到的为准；开发版另把 web 插件放进 `dev-plugins/builtin/web/<版本>/`），同一个插件只用最高版本，最高版本不合格整个不加载，不退回旧版本；`id`、`version` 要和目录对上。清单里不认识的字段只记警告。签名和来源证明还不查（[§8.3](distribution.md#83-校验与来源证明)）。只有一次一进程的调用，`serve` 还没做。
- **环境变量**：先清空，再放 `permissions.env` 里列的、宿主自己环境里有的，再放宿主设的（同名的以宿主为准，清单也不许列它们，[§2.7](manifest.md#27-权限-permissions)）。宿主设：`USUYO_PROTOCOL=1`、`USUYO_PLUGIN_DIR`、`USUYO_CACHE_DIR`（按机器目录的 `cache/plugins/<owner>/<repo>/`）、`USUYO_CALL_ID`（UUIDv7）、`USUYO_LOCALE`、导入时的 `USUYO_INPUT`、`HOME`、`XDG_CONFIG_HOME`、`XDG_CACHE_HOME`、`TMPDIR`、`TMP`、`TEMP`、`PATH`（`${TMP}/bin` 加系统目录）。工作目录是 `${TMP}`，stdin 是空的。
- **`USUYO_SETTINGS`**：按机器 `settings.json` 里 `plugins.<插件 id>.settings` 有值时，宿主把它写成这次调用文件夹里的 `settings.json`，变量指向它；没有就不设。
- **`USUYO_NETWORK`**：宿主查 `example.com`、`example.org` 两个名字（两秒内），两个都有回答、回答全在 `198.18.0.0/15` 里时设成 `fake-ip`，结果缓存一分钟；查不到、超时都算不是。不看要取的那个域名自己的回答。
- **节奏做了一部分**：[§3.1](#31-运行器与节奏) 的 `interval`、`daily`、`batch: false`、每平台一个调用的锁有了（取最严的一组，记账在 `jobs.db`，重启后照旧）；整机锁和设备锁还没做；任务一次只跑一个。「现在再试」「先别试」有了（`POST /plugins/{owner}/{repo}/try-now`、`…/hold`），用户改节奏的设置还没做。
- **凭据和密钥**：`credential: cookies` 的站点经 `USUYO_CREDENTIALS`、`USUYO_COOKIES_TXT`，密钥设置经 `USUYO_SECRETS`，访客身份经 `USUYO_VISITOR`（[§2.5.1](manifest.md#251-访客身份-visitors)）。值在系统钥匙串里（没有钥匙串的系统上是按机器目录的 0600 文件），按机器目录的 `logins.json` 只记名字、域名和过期时间；文件放在按机器目录 `run/` 下这次调用自己的 0700 目录里，0600，进程结束就删；交出去的值从 stderr、插件的错误和宿主拼的错误里洗掉（包括 JSON、Debug 转义过的写法）。plugin-kit 的 `http::Cookie::visitor` 把访客身份读成只发给给定主机、只走 https 的 Cookie。
- **`USUYO_PROXY`**（11-12）：现在交的是用户的代理，本机代理做出来以后改成指向本机代理、用户的代理由它转发（上面的「网络出口」）。路线是按机器设置 `network.proxy`：`system`（没写时）、`direct`、或一个代理地址。`system` 在 macOS 上读 `scutil --proxy`（两秒内，结果留一分钟），先取 https 的，再 SOCKS，再 http 的；别的系统暂时读环境变量；读不到就直连，记一条日志。只有清单里有网络权限的插件拿到这个变量；宿主自己下模型走同一条路。
- **还没传的**：`USUYO_BROWSER_PROFILE`、`USUYO_PROVIDER`、`ADB_SERVER_SOCKET`、`ANDROID_SERIAL`、`HTTP_PROXY` 一类的代理变量。本机代理和沙箱（[§7](isolation.md#7-权限与隔离)）都还没做；插件的出站现在靠 plugin-kit 的客户端自己把关（[§10.1](plugin-kit.md#101-plugin-kit-的-http-客户端11-11)）。
- **调用文件夹**：资料库的 `.usuyo/tmp/usuyo-call-<UUIDv7>-<随机>/`，里面 `tmp/`、`tmp/home/`、`tmp/bin/`，`fetch` 另有 `out/`；用完整个删掉，崩溃留下的在下次打开库时清掉。
- **时长**：`canonicalize` 10 s、`fetch` 300 s、`sources` 30 s、`list` 每页 60 s；发过 `progress` 以后 120 s 没有新的算卡住；一次调用里 `waiting` 加起来最多 600 s，等的时候另两个时钟停；能力的 `timeout` 最多按 2 小时算。超时或取消先给进程组 SIGTERM，5 秒后 SIGKILL；正常退出后也给进程组发一次 SIGKILL，留下的子进程一起结束。Windows 上只结束这一个进程（还没有 Job Object）。
- **stdout、stderr**：stdout 最多 32 MiB，超了 `invalid-output`；stderr 留最后 200 行，每行最多 8 KiB；解析不了的 `@usuyo ` 行忽略。
- **信封里的字**：标题、作者的 `name`、`platformId`、`url`，错误的 `message`、`hint`、`site`，`missingReason`、`evidence.site`，`list` 条目的标题和作者，`sources` 的标题：各截到 1024 字节（按字符边界），控制字符（含换行）换成空格。`platformTags` 最多 64 个，每个截到 100 字节。正文 `text` 不动。这些是截断，不拒。
- **会拒的**：插件给的链接（`identity.url`、`resolvedUrl`、`list` 的 `items[].url`）超过 4096 字节或有控制字符；`contentId` 空、超过 256 字节或有控制字符；身份的 `platform` 不在清单的 `platforms` 里；`comments` 不是 `comments.json`；同一个文件引用两次、引用的文件不在；输出目录不合 [§3.6](#36-fetch-与输出目录)（最多 10,000 项，默认 8 GiB）；插件报宿主才用的码（`timeout`、`crashed`、`invalid-output`、`profile-busy`）；`dependency-missing` 不带 `needs`。都记 `invalid-output`。
- **完整度**：宿主照 `--want` 和实际输出重算，和插件报的不一样记警告；算出来是 `link` 按 `unsupported-input`。
- **暂停**：`rate-limited` 按 `retryAfter`，没给 15 分钟，最多一小时；`blocked` 一小时；通用网页插件按主机暂停，别的按平台（[§3.1](#31-运行器与节奏)）。重试按 30 秒乘 4 的次方退后，最多 6 小时、6 次。
