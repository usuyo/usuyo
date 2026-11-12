<a id="按机器的目录"></a>
## 按机器的目录

**留在每台机器上的**放 [§0.1](conventions.md#01-命名) 的按机器目录；缓存放系统缓存目录（`~/Library/Caches/usuyo/`、`%LOCALAPPDATA%\usuyo\cache\`、`$XDG_CACHE_HOME/usuyo/`），删了不丢东西（11-11 代码：插件的缓存先放在按机器根目录的 `cache/plugins/` 下）。

| 在哪 | 放什么 |
|---|---|
| `libraries/<libraryId>/` | `device.json`（`deviceId`、库路径）、`index/`、`clock-mark`（回收站到期用的时钟记录，见 [§1.7](library.md#17-删掉)）、`jobs.db`（任务状态、转换任务 `transform_jobs`（feature 003，还没做）、跟着收列过的身份、翻页游标和 `--since`、还没展开的短链（重试）、上次写的 `content.md` 的哈希）、`sync.db`（游标、底本（你写的那层和 manifest 上次见过的那份，没开同步也存，[§1.4](library.md#14-你写的那一层) 外面改了靠它合并）、上次见过的路径）、`activity/`（Agent 搜过、读过什么，一行一次，不给 Agent、不同步；格式和留多久归第 2 层，推断）、`exports/<target>.json`（每个对象 `{path, hash, base}`，`base` 是上次和 vault 对上的那份用户层：写出去了就是交给导出插件的那份，没写出去就是那次读回的值，私密脚注已换成占位；[§1.4](library.md#14-你写的那一层) 导出读回靠它合并）、`imports/<importBatch>/`（导入文件副本）、`backups/` |
| 根目录 | `settings.json`（外观、模型端点、插件启用和设置，`transforms.auto`（自动转换，默认 `true`），`network.proxy`（出站的路线：`system`、`direct` 或一个代理地址，没写是 `system`，见[协议 §3.13](../protocol/fetch.md#313-宿主和-plugin-kit-现在做到的11-11)）；11-12 代码用 `library`、`plugins.<插件 id>.settings`、`plugins.<插件 id>.methods.<方法 id>.enabled`（这种方法开着没有，没写照默认，[协议 §2.3.1](../protocol/manifest.md#231-方法-methods)）、`transforms.auto`、`network.proxy`）、`plugins/`、`profiles/`、`models/<owner>/<repo>/<名字>/`（转换插件的模型，宿主下载、校验后放这里，旁边写 `model.json`：`{name, sha256, size, license, files: [{path, size, sha256}]}`）、`cache/plugins/<owner>/<repo>/`、`cache/transform/<itemId>/<runId>/`（一条转换链的中间产物，例如给转写用的 16 kHz WAV，链跑完就删）、本机接口令牌 `api-token`、投递令牌 `deliver-tokens.json`、`daemon.json`、登录索引 `logins.json`、`run/`、`inbox/`、日志 `logs/daemon.log`、获取插件的失败日志 `logs/fetch-failures.jsonl`（一次失败一行，过 1 MiB 换成 `fetch-failures.1.jsonl`，[协议 §2.3.1](../protocol/manifest.md#231-方法-methods)）（各项见下面「11-11 代码」） |
| 系统钥匙串 | 同步令牌和库钥匙、模型和插件密钥、平台登录态（[协议 §6](../protocol/credentials.md#6-凭据)） |

11-11 代码：

- 按机器的根目录可以用 `USUYO_HOME` 换成别处（测试用）；Linux 没设 `$XDG_STATE_HOME` 时是 `~/.local/state/usuyo/`。根目录、`libraries/<libraryId>/` 和 `logs/` 只给本人（0700），`jobs.db`、`api-token`、`deliver-tokens.json`、`daemon.json`、`logs/daemon.log`、`logs/fetch-failures.jsonl` 是 0600。
- `api-token` 存令牌本身，不存哈希：握手要拿它当密钥（[本机接口](local-api.md#本机接口第-2-层11-11-照代码)）。每次启动守护进程换一个，正常退出时连同 `daemon.json`（`{port, pid}`）一起删掉。
- `device.json` 是 `{deviceId, libraryPath}`。`jobs.db`（建表分几版，`user_version` 现在是 3）里有：取内容的任务（每条一行，按 `itemId` 去重，带优先级和所属导入）、插件的暂停（按插件、方法和范围，范围是平台，web 插件按主机；你点的「先别试」方法和范围都是 `*`；记错误码、原因、到什么时候；第 3 版重建了这张表，旧的暂停不搬）、每种方法上一次调用（时间、错误码和一句话，成了的错误码是空）、上次写的 `content.md` 的哈希、平台的节奏记账（每个平台上次调用结束的时间、今天批量调用了几次，重启后照旧）、导入（`source` 类记它属于哪种方法）、导入的清单（每个导入里每条链接一行，记身份、在库里的哪里、收下没有）、转换的任务。失败的任务清掉链接，只留错误码和原因。任务一次跑一个，同一平台同一时间只跑一个调用。
- `daemon.json` 是 `{port, pid}`。`api-token`、`daemon.json` 的来去见[本机接口](local-api.md#本机接口第-2-层11-11-照代码)。根目录 `settings.json` 现在用到的键见上表；设置页（[本机接口](local-api.md#本机接口第-2-层11-11-照代码)的 `PATCH /settings`）改 `network.proxy`、`transforms.auto`，先写临时文件再改名，别的键原样留着。
- `deliver-tokens.json`：投递令牌的清单，一个数组，每项 `{id, name, kind, secret, createdAt}`，`kind` 取 `extension`、`mobile`、`api`。`secret` 是明文（读文件的权限只有本人），令牌整个只在发的那一次回给调用方；撤销马上生效。
- `logins.json`：登录站点、访客身份和密钥设置的索引，不含任何值。站点和访客身份各按主机记存下的时间和每个 cookie 的名字、域名、过期时间；密钥设置记每个插件填了哪几项。看登录状态、过期没有只读它，不碰钥匙串。
- `run/`：一次调用要交凭据给插件时，在这里建这次自己的文件夹 `usuyo-cred-<随机>`（0700），文件 0600：`credentials.json` 和 `cookies.txt`（账号站点）、`visitor.json`（访客身份）、`secrets.json`（密钥设置），路径经 `USUYO_CREDENTIALS`、`USUYO_COOKIES_TXT`、`USUYO_VISITOR`、`USUYO_SECRETS` 传给插件（[协议 §6](../protocol/credentials.md#6-凭据)）；进程结束整个文件夹删掉，守护进程启动时再清一遍上次被杀留下的 `usuyo-cred-*`。没有可交的就不建。
- `inbox/`：收件文件夹，手机经云盘放进来。只认顶层的 `.txt` 普通文件；点开头的、别的扩展名、子文件夹都不碰；每 5 秒看一轮，一个文件 10 秒没变才处理；最多读 1 MiB、收 500 条链接，文件里每个 http(s) 链接收下一条（整段分享文字也行），入口记 `mobile`，放进去本身就是确认，不进导入预览。全收成了移到 `.usuyo-done/<日期>/`；没有链接、有链接没收成、或是符号链接的移到 `.usuyo-failed/`，旁边一个 `<名字>.error.json`（`{errors: […]}`，原因只写主机和错误码）。`GET /inbox` 回它在哪。
- `libraries/<libraryId>/clock-mark`：回收站到期检查记下的最大时间，见 [§1.7](library.md#17-删掉)。
- 钥匙串（macOS、Windows）：服务名 `usuyo`，账户名是 `<12 位十六进制>/<键>`，前一段是按机器根目录（取规范路径）的 SHA-256 前 12 位，所以不同的 `USUYO_HOME` 各管各的，开发用的另一套不会盖掉正式那套。键有三类：`site:<主机>`（这个站点的登录，storageState 的 JSON）、`visitor:<主机>`（这个站点的访客身份，不是账号，和同一主机的 `site:` 分开）、`secret:<插件 id>:<设置键>`（一个密钥设置）。Linux 还没接 Secret Service，换成根目录里 0600 的 `secrets.json`，键一样，是替身：挡得住别的用户，挡不住以你的身份跑的程序。壳的登录窗口在 macOS 上用系统的网页数据仓库，Windows、Linux 上放在根目录的 `webviews/` 下，见 [apps/desktop/README.md](../../apps/desktop/README.md#登录窗口)。
- 以写方式打开库（拿到 `.usuyo/lock`）时，清掉 `.usuyo/tmp/` 里崩溃留下的调用文件夹（`usuyo-call-*`）和以 UUIDv7 命名的暂存文件夹，别的不动。
- 扫描（启动、重建索引、地图、收下）遇到读不了的条目、回收站项、主题、卡片、成稿就跳过，写日志；条目另外列进结果的 `unreadable`（`{id, reason}`），一个坏文件不挡住别的。界面还不显示它们。

为什么搬出来：库进同步盘时 sqlite 和索引会被写坏；密钥不落普通文件；库拷到另一台自然有新 `deviceId`。索引不放缓存目录，macOS 磁盘紧时会清 Caches。同一台上同一个 `libraryId` 出现在第二个路径，问「这是拷贝吗」，是就换新 `libraryId`、`sync` 置 `null`，免得同步进同一个云端。
