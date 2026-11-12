<a id="2-清单"></a>
## 2. 清单

<a id="21-文件名与位置"></a>
### 2.1 文件名与位置

- 文件名 `usuyo-plugin.yaml`，放仓库根目录。这样叫的理由：三类插件都对得上；带产品前缀的名字在被包装的上游仓库里不会和别人的 `plugin.yaml`（Helm、Krew 都用这个名字）撞；GitHub 代码搜索能直接找到。
- 仓库里的是**源清单**：除 `install` 外都要写，`version` 必须等于要发的 tag（`v0.3.1` 对应 `0.3.1`）。
- 发版时由 plugin-kit 的发版工作流或 `usuyo plugin pack` 补上 `install`（各平台资产名与 sha256），作为 release 资产 `usuyo-plugin.yaml` 上传（[§8.2](distribution.md#82-发版与-release-资产)）。**客户端只读 release 资产里的清单**，不读默认分支上的文件。
- 格式 YAML 1.2；宿主转成 JSON 后按 schema 校验。字段名用 camelCase；时长写 `8s`、`10m`，大小写 `512MiB`、`1GiB`。

<a id="22-共有字段"></a>
### 2.2 共有字段

| 字段 | 类型 | 必填 | 含义 | 例子 |
|---|---|---|---|---|
| `protocol` | 整数 | 是 | 插件协议版本 | `1` |
| `id` | 字符串 | 是 | 等于发布它的 GitHub 仓库 `owner/repo`，全小写；客户端核对，不一致就拒装。`builtin/` 前缀留给随安装包预装的插件（[§8.4](distribution.md#84-三种来源)） | `someone/usuyo-fetch-douyin` |
| `kind` | 枚举 | 是 | `fetch`、`transform`、`export` | `fetch` |
| `name` | 字符串 | 是 | 界面上显示的名字 | `抖音` |
| `description` | 字符串 | 否 | 一句话说明 | `列出我的抖音收藏夹，按分享页取单条` |
| `version` | 字符串 | 是 | semver，等于 tag 去掉 `v`；写进条目快照的 `pluginVersion` | `0.3.1` |
| `license` | 字符串 | 是 | 插件自身的 SPDX 许可证 | `MIT` |
| `homepage` | URL | 否 | 默认是仓库主页 | `https://github.com/someone/usuyo-fetch-douyin` |
| `maintainers` | 字符串数组 | 是 | 维护者的 GitHub 登录名，至少一个；不经本人同意不写别人（RSSHub 的规则，[02 §15.1](../research/02-adapter-ecosystems.md#151-声明与匹配)） | `[someone]` |
| `upstream` | 对象数组 | 否 | 被包装或 fork 的上游，只写出处和许可证：`name`、`url`、`license`、`paid`（收费服务时为 `true`）；版本约束只写在 `requires.runtime` | 见下 |
| `requires.runtime` | 字符串数组 | 否 | 要用户自己装的运行时和外部程序，写成「名字加版本约束」；宿主安装前列出来，并检查能不能找到、版本对不对（[§2.13](#213-运行时解析)） | `["chrome>=136", "yt-dlp>=2026.10"]` |
| `settings` | 对象数组 | 否 | 插件设置，见 [§2.11](#211-插件设置-settings) | |
| `permissions` | 对象 | 是 | 权限声明，见 [§2.7](#27-权限-permissions) | |
| `install` | 对象 | 发版时生成 | 各平台 release 资产与 sha256，见 [§2.8](#28-安装与启动) | |
| `run` | 对象 | 是 | 启动命令，见 [§2.8](#28-安装与启动) | |

```yaml
upstream:
  - name: yt-dlp
    url: https://github.com/yt-dlp/yt-dlp
    license: Unlicense
```

<a id="23-获取插件的字段"></a>
### 2.3 获取插件的字段

| 字段 | 类型 | 必填 | 含义 | 例子 |
|---|---|---|---|---|
| `platforms` | 字符串数组 | 是 | 平台 id，取自 plugin-kit 里的平台列表（`douyin`、`xiaohongshu`、`bilibili`、`wechat-mp`、`wechat-channels`、`x`、`web`……）；新平台向 plugin-kit 提 PR 加名字。平台列表的每一项除了 id，还规定 `contentId` 的格式（正则）和规范链接的模板，例如 `douyin` 是十进制作品 ID，`bilibili` 用 BV 号、分 P 写成 `BV…:p2`；plugin-kit 为每个平台附一份共用的 `testdata/canonicalize.json`。同一平台的所有插件共用节奏和去重，所以 id 和 `contentId` 的写法都不能各起各的 | `[douyin]` |
| `match` | 字符串数组 | `fetch.input` 含 `url` 时必填 | 粗筛链接用的 `host/path` glob，不含 scheme；`*.` 前缀匹配子域但不含裸域；最终身份由 `canonicalize` 判定（[02 §15.1](../research/02-adapter-ecosystems.md#151-声明与匹配)） | `["v.douyin.com/*", "www.douyin.com/video/*"]` |
| `access` | 枚举 | 是 | 所有能力里最高的一层，见 [§2.4](#24-access-与-login) | `browser-session` |
| `login` | 枚举 | 是 | 所有能力里最高的一档：`none` < `optional` < `required` | `required` |
| `sites` | 对象数组 | 有登录时必填 | 会用到哪些账号，见 [§2.5](#25-登录站点-sites) | |
| `visitors` | 对象数组 | 否 | 会用到哪些站点的访客身份（不是账号），见 [§2.5.1](#251-访客身份-visitors) | |
| `capabilities` | 对象 | 是 | 能力名到能力声明的映射，见下表 | |
| `pacing` | 对象 | 否 | 按平台给的节奏建议：`batch`（`false` 表示只接受用户一条条收下）、`interval`（同一平台上两次调用的最短间隔）、`daily`（每天的调用次数上限）。宿主取同平台所有已装插件里最严的作默认值，用户可以改；计哪些调用见 [§3.1](fetch.md#31-运行器与节奏) | `{douyin: {batch: true, interval: 8s, daily: 300}}` |

每个能力的声明：

| 字段 | 适用 | 必填 | 含义 | 例子 |
|---|---|---|---|---|
| `access` | 全部 | 是 | 这个能力实际用到的那一层 | `public-web` |
| `login` | 全部 | 是 | `none`、`optional`、`required` | `none` |
| `sites` | 全部 | 有登录时必填 | 用到 `sites` 里的哪几个，写 `site` 值 | `[www.douyin.com]` |
| `visitors` | 全部 | 否 | 用到 `visitors` 里的哪几个，写 `site` 值；和 `login` 无关，`login: none` 也可以写（[§2.5.1](#251-访客身份-visitors)） | `[www.example.com]` |
| `timeout` | 全部 | 否 | 覆盖 [§3.4](fetch.md#34-超时与取消) 的默认总时长；获取插件只用这一处，不用 `permissions.limits.timeout` | `2h` |
| `perPlatform` | 全部 | 否 | 多平台插件里某个平台走的路不同时，按平台覆盖 `access`、`login`、`sites` | `{bilibili: {access: private-api, login: optional}}` |
| `outputs` | `fetch` | 是 | 能取到哪些东西，固定词表：`text`、`html`、`images`、`media:video`、`media:audio`、`subtitle`、`meta`、`comments` | `[text, images, media:video]` |
| `input` | `fetch` | 否 | 接什么：`[url]`（默认）、`[ref]`（只接 `list` 返回的 `ref`，导入插件这样写）或 `[url, ref]` | `[ref]` |
| `order` | `list` | 是 | `newest-first`（按加入时间从新到旧）或 `unordered`，决定跟着收怎么停（[§3.8](fetch.md#38-list-与跟着收)） | `newest-first` |
| `sources` | `list` | 否 | 固定的来源，界面在登录前就能显示；运行时以 `sources` 子命令为准 | `[{source: "douyin:favorites", title: 我的收藏夹, action: favorite}]` |
| `files` | `list` | 导入插件必填 | 能读的导出文件名 glob（[§3.9](fetch.md#39-从文件导入)） | `["twitter-*.zip"]` |
| `interaction` | 全部 | 否，默认 `background` | 这次调用用户看不看得见：`background` 只读，不弹菜单、不改剪贴板、不抢焦点、不动指针、不占手机屏幕；`foreground` 是上面任何一样会发生。宿主只拿它排程和加锁（[§3.1](fetch.md#31-运行器与节奏)），不拦、不弹确认 | `foreground` |
| `provider` | 全部 | 否，默认 `none` | 这个能力要不要模型：`none`、`optional`、`required`，含义同 [§2.9](#29-转换插件的字段)；不是 `none` 时经宿主代发（[§4.3](transform-export.md#43-模型与密钥由宿主代发)） | `optional` |

一个按 [06 §1.3](../research/06-platform-access-and-account-risk.md#13-混合路径视觉层拿链接公开页面取内容) 混合路径写的抖音插件，源清单是这样（cookie 名是示例；`install` 发版时生成，见 [§2.8](#28-安装与启动)）：

```yaml
protocol: 1
id: someone/usuyo-fetch-douyin
kind: fetch
name: 抖音
description: 列出我的抖音收藏夹，按分享页取单条
version: 0.3.1
license: MIT
maintainers: [someone]
upstream:
  - name: parse-video
    url: https://github.com/wujunwei928/parse-video
    license: MIT
platforms: [douyin]
match: ["v.douyin.com/*", "www.douyin.com/video/*", "www.iesdouyin.com/share/video/*"]
access: browser-session
login: required
sites:
  - site: www.douyin.com
    name: 抖音
    credential: browser-profile
    loginUrl: https://www.douyin.com/
    cookies: [sessionid]
capabilities:
  fetch:
    access: public-web
    login: none
    outputs: [text, images, media:video, meta]
  list:
    access: browser-session
    login: required
    sites: [www.douyin.com]
    order: newest-first
    sources:
      - {source: "douyin:favorites", title: 我的收藏夹, action: favorite}
pacing:
  douyin: {batch: true, interval: 8s, daily: 300}
permissions:
  fs:
    write: ["${OUT}"]
  network:
    mode: allowlist
    hosts: ["*.douyin.com", "*.douyinvod.com", "*.douyinpic.com", "*.iesdouyin.com"]
  env: []
  devices: []
  gpu: false
  limits: {memory: 1GiB}
requires:
  runtime: ["chrome>=136"]
run:
  command: ["${PLUGIN_DIR}/usuyo-fetch-douyin"]
  serve: true
```

<a id="24-access-与-login"></a>
### 2.4 access 与 login

`access` 取七层之一，含义照 [06 约定](../research/06-platform-access-and-account-risk.md#约定) 与 [06 §1.1](../research/06-platform-access-and-account-risk.md#11-七个层级)，从低到高：

| 值 | 怎么拿数据 |
|---|---|
| `official-export` | 平台官方的数据导出、用户授权的官方开放接口，或读用户已经导出的文件 |
| `public-web` | 不登录，读公开页面或分享链接，不自己复现平台的请求签名 |
| `vision-ui` | 只读屏幕上渲染出来的内容，用系统级输入驱动真实客户端 |
| `browser-session` | 驱动浏览器，带登录态读 DOM 或页面自己发出的请求 |
| `session-api` | 代码里带用户的 cookie 或 token 调平台接口，不复现签名 |
| `private-api` | 在官方客户端之外复现签名、加密参数、设备指纹，或模拟 App |
| `client-hook` | 注入或 Hook 官方客户端、解密本地数据库、装根证书做中间人 |

规则：

- **按能力分别声明，顶层取最高。** 列表走登录侧、单条走公开页这类混合路径要分开写；顶层 `access`、`login` 是所有能力（含 `perPlatform`）里最高的一档，用于列表展示。
- **按实际默认路径填，不按宣传填。** 一条路径用到多层时取最高的一层；上游工具有不同模式时，按插件实际调用的模式填。
- **读界面的插件怎么填。** 只读无障碍树、控件树或截图，动作走无障碍接口或系统输入的，填 `vision-ui`：读用户日常浏览器里渲染出来的网页、经 ADB 或手机上的程序读安卓控件树再发输入，都算；开后台虚拟屏、以 shell 身份起助手进程不改变层级。经 CDP、远程调试、扩展或页面脚本读 DOM 或请求的，填 `browser-session`。带调试端口重启官方客户端、在它的渲染进程里执行脚本（例如对抖音、B站这类 Electron 客户端用 agent-desktop 的 `launch --cdp`），或注入、Hook 目标 App 的进程、读它的私有数据，填 `client-hook`（[调研 10](../research/10-desktop-and-android-agents.md)）。
- **它们是给用户挑选用的标签，不是闸门。** 宿主不据此推风险档、不拦截、不弹单独的确认；运行器只按用户排的顺序试（[§3.1](fetch.md#31-运行器与节奏)）。
- **`login: none` 的能力拿不到任何平台账号凭据**：宿主不传 cookie、站点存储和浏览器资料目录（[§6](credentials.md#6-凭据)）。插件自己的密钥设置照 [§2.11](#211-插件设置-settings) 另行传入，访客身份照 [§2.5.1](#251-访客身份-visitors) 另行传入，都和 `login` 无关。`optional` 表示没有凭据也能跑；用户登录过也不带，见下。
- **新版本调高 `access` 或 `login`** 时，客户端在更新说明里标出，不停用这个版本（[§8.5](distribution.md#85-添加来源更新与权限扩大)）。

`optional` 表示没有凭据也能跑；不带账号能拿到的一律不带，用户登录过也不带，账号只用在非它不可的那几样上。

<a id="25-登录站点-sites"></a>
### 2.5 登录站点 sites

`sites` 写明会用到哪个站点的账号。界面据 `name` 写「会用到你的抖音账号」；视频号借腾讯元宝登录态的插件写 `yuanbao.tencent.com`，界面就写「会用到你的腾讯元宝账号」（[06 §1.3](../research/06-platform-access-and-account-risk.md#13-混合路径视觉层拿链接公开页面取内容)）。

| 字段 | 类型 | 必填 | 含义 | 例子 |
|---|---|---|---|---|
| `site` | 字符串 | 是 | 账号所属站点的主机名；凭据只按它做域名匹配后传出（[§6](credentials.md#6-凭据)） | `yuanbao.tencent.com` |
| `name` | 字符串 | 是 | 界面上的账号名 | `腾讯元宝` |
| `credential` | 枚举 | 是 | 插件要哪种形式：`cookies`（宿主传 cookie 与站点存储）、`browser-profile`（宿主传一个专用浏览器资料目录）、`app`（插件驱动用户已登录的客户端或手机，宿主什么都不传） | `cookies` |
| `loginUrl` | URL | `cookies`、`browser-profile` 必填 | 宿主让用户登录时打开的页面 | `https://yuanbao.tencent.com/` |
| `cookies` | 字符串数组 | 否 | 登录后才有的 cookie 名；宿主只看名字在不在、过没过期，判断登录是否完成，不读值 | `["<登录后才有的 cookie 名>"]` |
| `domains` | 字符串数组 | 否 | 登录态还分布在别的域名时显式列出，宿主一并传出。每项必须和 `site` 属于同一个可注册域名（eTLD+1），不能是公共后缀；按 cookie 的 `domain` 属性匹配这个值及其子域 | `[".tencent.com"]` |

视频号借元宝登录态的插件照 [06 §17.3](../research/06-platform-access-and-account-risk.md#173-首批自己适配的插件) 移植 qiaomu-clipper 的做法，在元宝页内取凭证，登录态不出浏览器，所以记 `browser-session`。只用 cookie、在页外能不能换到视频地址，待核实。

```yaml
platforms: [wechat-channels]
access: browser-session
login: required
sites:
  - site: yuanbao.tencent.com
    name: 腾讯元宝
    credential: browser-profile
    loginUrl: https://yuanbao.tencent.com/
    cookies: ["<登录后才有的 cookie 名>"]
capabilities:
  fetch:
    access: browser-session
    login: required
    sites: [yuanbao.tencent.com]
    outputs: [text, media:video, meta]
requires:
  runtime: ["chrome>=136"]
```

一个能力最多声明一个 `credential: browser-profile` 的站点，因为一次调用只交出一个资料目录。这个目录（`${PROFILE}`）不用在 `permissions.fs` 里声明：`sites` 本身就是声明，宿主只在用到这个站点的那次调用（或那个常驻进程，[§3.12](fetch.md#312-常驻模式-serve)）里放行读写。

视频号借元宝登录态的插件在 README 里多写一句「元宝可以用微信登录，这时这个元宝账号和你的微信号关联」；两边是否共用风控没查到，不写进界面。

<a id="251-访客身份-visitors"></a>
### 2.5.1 访客身份 visitors

有的站点不登录也要一份访客身份才取得稳：任何没登录的访客打开一页，站点都会给一组 cookie。它不是账号，所以不写在 `sites` 里：宿主另开一个不登录的窗口，打开 `url`，存下页面自己生成的 cookie，经 `USUYO_VISITOR` 交给列了它的能力（[§3.2](fetch.md#32-调用约定)、[§6](credentials.md#6-凭据)）。

| 字段 | 类型 | 必填 | 含义 | 例子 |
|---|---|---|---|---|
| `site` | 字符串 | 是 | 主机名，小写；一个清单里不重复 | `www.example.com` |
| `name` | 字符串 | 是 | 界面上写「开一次 X 首页」的 X | `例子` |
| `url` | URL | 是 | 宿主打开的页面，只能是 https | `https://www.example.com/` |
| `cookies` | 字符串数组 | 否 | 这些名字都出现了就算取到；不写时等名字连着几秒不再变 | `["<访客 cookie 名>"]` |
| `domains` | 字符串数组 | 否 | 和 [§2.5](#25-登录站点-sites) 的 `domains` 同一套规则：和 `site` 同一个可注册域名，不能是公共后缀 | `[".example.com"]` |

```yaml
access: public-web
login: none
visitors:
  - site: www.example.com
    name: 例子
    url: https://www.example.com/
capabilities:
  fetch:
    access: public-web
    login: none
    visitors: [www.example.com]
    outputs: [text, images, meta]
```

- **和同一主机的账号是两回事。** 访客身份和账号各存一份（钥匙串里分开的两项），登录窗口和访客窗口各用一份网页数据；以后登录这个主机，不会和访客身份混在一起。只列了访客身份的调用拿不到账号的 cookie，只列了 `sites` 的调用拿不到访客身份。
- **能力里的每一项都要在顶层 `visitors` 里声明**，否则清单不合格。`login: none` 的能力可以列访客身份，照样不能列 `sites`。
- 页面要访客身份而没有交来、或交来的页面仍不收时，插件回 `visitor-required`（不是 `login-required`），在 `message` 里分清是哪一种；界面据此写「要先取访客身份」或「换一份」，不会让人去找一个不存在的登录。
- 访客身份只存 cookie，不存 localStorage；`credentialsUpdate` 只写回账号站点（[§6](credentials.md#6-凭据)），不改访客身份。
- 界面只说有没有、什么时候存下的、最早哪个 cookie 什么时候过期；不出现在登录站点的列表里，也不算「还没登录」。

<a id="26-账号写操作"></a>
### 2.6 账号写操作

对平台账号做写操作的能力要单独声明。v1 只定义一个：`unfavorite`，在平台上取消收藏（[回答表 D3](../answers.md)）。

```yaml
capabilities:
  unfavorite:
    access: browser-session
    login: required
    sites: [www.douyin.com]
```

- 默认关。用户在插件详情里打开后才会被调用；每次批量执行前，宿主列出要取消的条目让用户确认。
- 只有声明了 `unfavorite` 的插件会收到这个调用；清单里没有的写操作一律不存在。包装别人的工具时，要关掉发帖、点赞、私信这类命令（[06 §1.1](../research/06-platform-access-and-account-risk.md#11-七个层级)）。
- 以后要加别的写操作，照同样的规则：单独的能力名、默认关、执行前确认。

有插件实现 `unfavorite` 时，入口放在导入结果页和跟着收设置里，执行前列清单确认；收下后不自动在平台上取消。

<a id="27-权限-permissions"></a>
### 2.7 权限 permissions

沿用 [ADR-003 §4](../decisions/ADR-003-plugins.md#4-权限与隔离) 的结构，加上设备类。路径一律写宿主展开的占位符。

| 字段 | 类型 | 默认 | 含义 | 例子 |
|---|---|---|---|---|
| `fs.read` | 占位符数组 | `[]` | 可读的路径 | `["${INPUT}"]` |
| `fs.write` | 占位符数组 | `[]` | 可写的路径 | `["${OUT}"]` |
| `network.mode` | 枚举 | `none` | `none` 或 `allowlist` | `allowlist` |
| `network.hosts` | 字符串数组 | `[]` | 允许出站的主机，可带端口，不带端口指 80 与 443；`*.` 匹配子域；单写 `"*"` 表示任意公开主机（通用网页一类插件用，权限页写「可以连任何公开主机」）；可以引用设置 `${setting.<key>}`，宿主取其中的主机和端口。回环和内网地址只在显式写了带端口的地址时放行（[§3.2](fetch.md#32-调用约定)） | `["*.douyin.com", "127.0.0.1:5556"]` |
| `exec` | 字符串数组 | `requires.runtime` 里的名字 | 允许再起的程序名，管整个进程树：被包装的工具再起的程序也要算进来。`${PLUGIN_DIR}` 里的可执行文件（插件自带的工具和它们再起的 helper）总是可以执行，不用写。缺省等于 `requires.runtime` 里的名字，只有额外的程序（例如系统自带的 `osascript`）才另写；解析见 [§2.13](#213-运行时解析) | `[osascript]` |
| `env` | 字符串数组 | `[]` | 允许从用户环境透传的变量名；不能用来传凭据，名字里带 `TOKEN`、`COOKIE`、`KEY`、`SECRET`、`PASSWORD` 的，`usuyo plugin check` 判失败。宿主在加载时就拒掉这个插件：名字要合 `[A-Za-z_][A-Za-z0-9_]*`；不能带上面五个词和 `AUTH`、`SOCK`（不分大小写）；不能是宿主自己设的变量：`HOME`、`PATH`、`TMPDIR`、`TMP`、`TEMP`、`SYSTEMROOT`、`WINDIR`，以及 `XDG_`、`USUYO_` 开头的 | `[LANG]` |
| `gpu` | 布尔 | `false` | 要不要 GPU | `false` |
| `devices` | 枚举数组 | `[]` | 设备类权限，见下 | `[ios-device, clipboard]` |
| `limits.memory` | 大小 | `1GiB` | 内存上限，超了宿主杀进程 | `4GiB` |
| `limits.timeout` | 时长 | 按调用 | 转换、导出插件单次 `run` 的总时长；获取插件改用能力里的 `timeout`（[§3.4](fetch.md#34-超时与取消)） | `4h` |

占位符：

| 占位符 | 指什么 | 读写 |
|---|---|---|
| `${OUT}` | 这次调用的输出目录，宿主新建、空、绝对路径 | 写 |
| `${INPUT}` | 这次调用的输入：导入文件的副本（[§3.9](fetch.md#39-从文件导入)）、转换与导出的条目文件 | 读 |
| `${CACHE}` | 这个插件的缓存目录，宿主随时可以清；不能放凭据 | 读写 |
| `${PROFILE}` | `browser-profile` 站点的专用浏览器资料目录；不用声明，宿主只在用到这个站点的调用里放行（[§2.5](#25-登录站点-sites)） | 读写 |
| `${TARGET}` | 导出的目标文件夹，例如整个 Obsidian vault；只给 `readback: true` 的导出插件，`readback` 用（[§4.4](transform-export.md#44-导出的-run-与-readback)、[§7.1](isolation.md#71-常驻进程的路径)）；`readback: true` 的导出插件要写进 `fs.read`，没写的不放行，`usuyo plugin check` 查这条（推断）；更新时多了它就算扩大了权限（[§8.5](distribution.md#85-添加来源更新与权限扩大)） | 读 |
| `${MODELS}` | 转换插件的模型目录，由宿主按 `models` 下载 | 读 |
| `${PLUGIN_DIR}` | 安装目录，总是只读可见，不用声明 | 读 |
| `${TMP}` | 这次调用的临时目录，总是可写，不用声明 | 读写 |
| `${HOME}` | 插件的私有主目录：一次一进程时是 `${TMP}/home`，常驻时是进程暂存目录下的 `home/`（[§7.1](isolation.md#71-常驻进程的路径)）；总是可写，不用声明，调用或进程结束时删除 | 读写 |

`devices` 的取值：

| 值 | 用来做什么 |
|---|---|
| `adb` | 经宿主管的 adb 服务操作用户选定的一台 Android 设备（USB 或无线调试，[§7](isolation.md#7-权限与隔离)）；adb shell 能做的都能做：读屏幕和控件树、截图、模拟点按、读剪贴板、开虚拟屏、推送并运行程序、装卸 App。权限页照这句写 |
| `ios-device` | 经 usbmux 一类通道驱动 iPhone（WebDriverAgent、go-ios） |
| `screen-capture` | 截屏、录屏，包括读 iPhone 镜像窗口 |
| `accessibility` | 用辅助功能、UI Automation 读界面和发送输入 |
| `audio-capture` | 录系统声音，例如录下视频号的口播再转写 |
| `clipboard` | 读写剪贴板，例如「分享 → 复制链接」后读链接 |

- `accessibility`、`screen-capture`、`clipboard`、`audio-capture` 指电脑本机。安卓上的读屏、截图、剪贴板都经 `adb`，不另声明这几项。
- 不为虚拟屏、合成输入另加取值：开不开后台虚拟屏是插件的做法，做成插件设置（[§2.11](#211-插件设置-settings)）；会不会动指针、弹菜单、占屏幕，由能力的 `interaction` 说（[§2.3](#23-获取插件的字段)）。
- 插件推到手机上的文件、装的 App 或输入法、改的设置，要在 README 列出，并在进程退出前还原。

各平台能强制到什么程度见 [§7](isolation.md#7-权限与隔离)。

<a id="28-安装与启动"></a>
### 2.8 安装与启动

```yaml
install:
  assets:
    darwin-arm64:
      name: usuyo-fetch-douyin_0.3.1_darwin-arm64.tar.gz
      sha256: 4b227777d4dd1fc61c6f884f48641d02b4d121d3fd328cb08b5531fcacdabf8a
    linux-amd64:
      name: usuyo-fetch-douyin_0.3.1_linux-amd64.tar.gz
      sha256: ef2d127de37b942baad06145e54b0c619a1f22327b2ebbcfbec78f5564afe39d
    windows-amd64:
      name: usuyo-fetch-douyin_0.3.1_windows-amd64.zip
      sha256: e7f6c011776e8db7cd330b54174fd76f7d0216b612387a5ffcfb81e6f0919683
run:
  command: ["${PLUGIN_DIR}/usuyo-fetch-douyin"]
  serve: true
```

| 字段 | 类型 | 必填 | 含义 |
|---|---|---|---|
| `install.assets` | 对象 | 发版时生成 | 键是 `<os>-<arch>`（`darwin`、`linux`、`windows` 配 `amd64`、`arm64`），或不分平台的 `any`；值是资产名与 sha256。资产是单个可执行文件、`.tar.gz` 或 `.zip`，宿主解到 `${PLUGIN_DIR}`。资产名以这里写的 `name` 为准 |
| `run.command` | 字符串数组 | 是 | 启动命令，可以用 `${PLUGIN_DIR}`；脚本类插件写运行时，例如 `["node", "${PLUGIN_DIR}/dist/main.js"]`。第一个元素不在 `${PLUGIN_DIR}` 下时，它必须写进 `requires.runtime`，按 [§2.13](#213-运行时解析) 解析，不用另写进 `exec` |
| `run.serve` | 布尔 | 否 | 获取插件是否支持常驻模式（[§3.12](fetch.md#312-常驻模式-serve)）；转换、导出插件总是常驻 |

- 宿主安装时只下载 release 资产，不调用 npm、pip 一类包管理器。脚本类插件把依赖打进资产，或做成单个可执行文件（`bun build --compile`、`deno compile`、PyInstaller 一类）；要的运行时写进 `requires.runtime`。
- **插件自带的第三方程序**（例如视觉层插件带的 agent-desktop，推到手机上的 scrcpy-server 或自己写的 `app_process` 助手）放进插件自己的资产，只打包当前平台那一份，锁定版本，不让用户另装（agent-desktop 不走 `npm i -g`）。插件仓库的 CI 先核上游发布的 sha256 和来源证明，agent-desktop 是 `gh attestation verify <archive> --repo vercel-labs/agent-desktop`，通过了才打进资产；上游的 LICENSE 一并带上，agent-desktop 的 npm 包和 release 压缩包里都没有，要从它的仓库拿。运行时不另下载，推到手机上的文件只从 `${PLUGIN_DIR}` 推；GPL 的手机端组件只让用户自装（[§9.6](wrapping.md#96-许可证)）。
- 安装目录按版本分开：`<本机配置目录>/plugins/<owner>/<repo>/<version>/`，保留上一个版本便于回退。
- 插件的启用状态、设置放本机配置目录，密钥放系统钥匙串（[§2.11](#211-插件设置-settings)），都不放进资料库（[ADR-003 §4](../decisions/ADR-003-plugins.md#4-权限与隔离)）。

<a id="29-转换插件的字段"></a>
### 2.9 转换插件的字段

| 字段 | 类型 | 必填 | 含义 | 例子 |
|---|---|---|---|---|
| `consumes` | 数组 | 是 | 吃什么：`media:video`、`media:audio`、`audio:pcm16k`（别的插件抽出的 16 kHz 单声道 WAV）、`image`、`html`、`text`、`subtitle`、`derived:asr`、`derived:ocr`、`derived:vision`、`derived:keyframes`。`text` 指原文那一栏：主快照的 `text`，没有就是 `derived/readable.md`（[SPEC §1.2.3](../spec/library.md#123-contentmd-和各栏)）。只吃特定格式时写成对象 `{kind, mime}`，宿主编排时先让抽音轨一类插件转成要的格式 | `[{kind: media:audio, mime: [audio/wav]}]` |
| `produces` | 字符串数组 | 是 | 产出哪几种东西，写种类不写路径：`play`（分开的画面和音轨合起来能播的文件）、`audio:pcm16k`（给转写用的中间音频，不进资料库）、`probe`（只回流的信息，不出文件）、`asr`、`keyframes`、`ocr`；以后 `readable`、`vision`、`summary`、`embeddings`。不重复。路径由宿主定在 `derived/runs/<runId>/` 下（[SPEC §1.2.2](../spec/library.md#122-raw-和-derived)）；`thumb.jpg`、`fingerprints.json` 由宿主产出，不进 `produces` | `[asr]` |
| `provider` | 枚举 | 是 | 要不要模型 provider：`none`、`optional`、`required`（[§4.3](transform-export.md#43-模型与密钥由宿主代发)） | `none` |
| `models` | 对象数组 | 否 | 要宿主下载的模型，放进 `${MODELS}/<name>/`：`name`（成为目录名：小写字母、数字、`.`、`_`、`-`，不以 `.` 开头，最多 100 字节，不重名）、`url`、`mirrors`（可选，同一个 sha256 的别的地址，`url` 不行时按顺序试）、`sha256`（整个下载文件）、`size`（整个下载文件的字节数，大于 0）、`format`（`file`、`tar.bz2`、`tar.gz`、`zip`；压缩包由宿主原样解到 `${MODELS}/<name>/`，包里的顶层文件夹留着，例如 `${MODELS}/sense-voice-int8-2024-07-17/sherpa-onnx-sense-voice-zh-en-ja-ko-yue-int8-2024-07-17/model.int8.onnx`）、`license`（下载前给你看），这些都必填。可选 `setting`、`value`：只在用户把这个设置选成这个值之后才用到这个模型。什么时候下见 feature 003 [§4.3](../features/003-transforms.md#43-ffmpeg-和模型怎么到这台电脑上)：第一次用到时问一句，点了才下，宿主下，插件不联网 | 见下 |

```yaml
kind: transform
consumes: [audio:pcm16k]
produces: [asr]
provider: none
models:
  - name: sense-voice-int8-2024-07-17
    url: https://github.com/k2-fsa/sherpa-onnx/releases/download/asr-models/sherpa-onnx-sense-voice-zh-en-ja-ko-yue-int8-2024-07-17.tar.bz2
    sha256: 7d1efa2138a65b0b488df37f8b89e3d91a60676e416f515b952358d83dfd347e
    size: 163002883
    format: tar.bz2
    license: FunASR Model Open Source License（可商用，署名并写明模型名）
    setting: model
    value: sense-voice
permissions:
  fs:
    read: ["${INPUT}", "${MODELS}"]
    write: ["${OUT}"]
  gpu: true
  limits: {memory: 4GiB, timeout: 1800s}
```

宿主按 `consumes`、`produces` 把转换插件串起来，例如抽音轨、转写、摘要（[ADR-003 §1](../decisions/ADR-003-plugins.md#1-插件分三类)）。

<a id="210-导出插件的字段"></a>
### 2.10 导出插件的字段

| 字段 | 类型 | 必填 | 含义 | 例子 |
|---|---|---|---|---|
| `target` | 字符串 | 是 | 目标软件 | `obsidian` |
| `writes` | 枚举 | 是 | `files`（结果交给宿主写进目标文件夹）或 `api`（经宿主代发云端调用，界面标「离开本机」） | `files` |
| `readback` | 布尔 | 是 | 支不支持读回用户改动 | `true` |

<a id="211-插件设置-settings"></a>
### 2.11 插件设置 settings

包装用户自己跑的服务、选画质这类事要设置项。

| 字段 | 类型 | 必填 | 含义 |
|---|---|---|---|
| `key` | 字符串 | 是 | 小写字母和数字，几段用 `-` 连起来，不重复 |
| `label` | 字符串 | 是 | 界面文字 |
| `type` | 枚举 | 是 | `string`、`bool`、`int`、`enum`；没有小数，带小数的参数（例如转写插件的 VAD 阈值）不做成设置，只经 `run` 的 `options` 传 |
| `options` | 字符串数组 | `enum` 必填 | 可选值 |
| `default` | 任意 | 否 | 默认值 |
| `secret` | 布尔 | 否 | 是密钥时为 `true`，存系统钥匙串 |
| `inject` | 对象 | 转换、导出的密钥必填 | `{host, header, format}`：宿主在代发时把密钥填进这个请求头，插件拿不到明文（[§4.3](transform-export.md#43-模型与密钥由宿主代发)） |

```yaml
settings:
  - key: endpoint
    label: XHS-Downloader 服务地址
    type: string
    default: http://127.0.0.1:5556
  - key: xhs-token
    label: XHS-Downloader 访问令牌
    type: string
    secret: true
```

获取插件的非密钥设置经 `USUYO_SETTINGS` 指向的 JSON 文件传入；密钥设置经 `USUYO_SECRETS` 另传（[§3.2](fetch.md#32-调用约定)），和平台账号凭据分开，和 `login` 无关。转换、导出插件的非密钥设置经 `run` 的 `options` 传，密钥只经宿主代发。

<a id="213-运行时解析"></a>
### 2.13 运行时解析

`requires.runtime` 和 `exec` 里写的是名字，宿主负责把名字变成这台电脑上的程序。三类插件都一样。

- **运行时表。** plugin-kit 维护一张表：名字 → 各系统上的可执行文件名和常见安装位置、取版本的命令和正则、版本怎么比较（semver、日期版本、只比主版本号）。例如 `chrome` 在 macOS 上是 `/Applications/Google Chrome.app/Contents/MacOS/Google Chrome`，版本靠 `--version` 取；`yt-dlp>=2026.10` 按日期版本比较。表外的名字只检查能不能找到，不检查版本。
- **查找顺序。** 用户在设置里给这个名字指定的路径 → 用户登录 shell 的 `PATH` → `/opt/homebrew/bin`、`/usr/local/bin`、`~/.local/bin` → 表里的常见安装位置。由 launchd 或 Finder 启动的 usuyo，`PATH` 里没有 Homebrew 的目录，所以不能只看自己进程的 `PATH`。
- **按名字调用。** 每个进程开始时，宿主建 `${TMP}/bin/`（常驻进程建在它的暂存目录下），为每个名字放一个指向解析结果的符号链接或转发脚本，并把这个目录放在 `PATH` 最前。插件和被包装的工具一律按声明的名字调用，不自己找路径。
- **管整个进程树。** `exec` 管插件和它所有子孙进程：yt-dlp 合并音视频要起 ffmpeg、解 YouTube 要起 deno，这两个也要在 `requires.runtime` 或 `exec` 里。
- **自动放行只读。** 解析出的程序、它带 shebang 时的解释器（Homebrew 装的 yt-dlp 是 Python 脚本），以及它们的安装目录，宿主自动放行只读和执行，不用写进 `fs.read`。
- **找不到或版本不对**时，宿主不调用插件，在插件那一行列出缺的东西，和 `dependency-missing` 的 `needs` 用同样的写法（[§3.3](fetch.md#33-信封与错误码)）。
