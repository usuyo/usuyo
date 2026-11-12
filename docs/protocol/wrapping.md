<a id="9-适配现有工具"></a>
## 9. 适配现有工具

上游只有 HTTP 接口或 MCP 的，插件里写一层薄的命令行转接，对宿主仍是 [§3](fetch.md#3-获取插件协议) 的子命令。本节的清单片段只列和包装方式有关的字段，完整清单照 [§2](manifest.md#2-清单) 补齐。

<a id="91-包装命令行"></a>
### 9.1 包装命令行

例子：不登录取 YouTube 单条，包装用户自装的 yt-dlp。按 [06 §17.3](../research/06-platform-access-and-account-risk.md#173-首批自己适配的插件)，yt-dlp 在 YouTube 上记 `private-api`，不传 cookie 就没有账号风险。

```yaml
id: someone/usuyo-fetch-youtube
kind: fetch
platforms: [youtube]
match: ["www.youtube.com/watch*", "youtu.be/*", "www.youtube.com/shorts/*"]
access: private-api
login: none
capabilities:
  fetch:
    access: private-api
    login: none
    outputs: [text, media:video, subtitle, meta]
requires:
  runtime: ["yt-dlp>=2026.10", "deno>=2", "ffmpeg"]
permissions:
  fs:
    write: ["${OUT}"]
  network:
    mode: allowlist
    hosts: ["*.youtube.com", "*.googlevideo.com", "*.ytimg.com", "youtu.be"]
```

`exec` 没写，缺省等于 `requires.runtime` 里的 yt-dlp、deno、ffmpeg（[§2.7](manifest.md#27-权限-permissions)）：yt-dlp 解 YouTube 要起 deno，B站、YouTube 的高画质音视频分开下，要 ffmpeg 合并。

插件的 `fetch` 只调一次 yt-dlp，同一条内容不向平台解析两遍：

```
yt-dlp --ignore-config --no-playlist --no-progress --write-info-json --write-subs --sub-langs "zh.*,en.*" -o "$OUT/media/%(id)s.%(ext)s" -- "$URL"
```

- `--ignore-config`：私有 `HOME` 挡不住 `/etc/yt-dlp.conf` 这类系统级配置，要显式不读。
- `--`：链接前加上，免得以 `-` 开头的链接被当成选项（[06 §3](../research/06-platform-access-and-account-risk.md#3-用户点名的项目) 引的 hypit #452）。
- 元数据从写出的 `.info.json` 读：填 `identity`、`title`、`author`、`publishedAt`，`media` 列出视频和字幕，`evidence.upstream` 写 yt-dlp 的版本。
- 不要用 `--cookies-from-browser`；这个插件是 `login: none`。有登录的插件用 `--cookies "$USUYO_COOKIES_TXT"`。
- yt-dlp 自己的输出不要转到插件的 stdout；要进度就解析它的输出，转成 `@usuyo` 的 `progress` 事件。下大视频时在能力里把 `timeout` 设长（[§3.4](fetch.md#34-超时与取消)）。

另一种做法是把上游当库编进插件：parse-video 是 Go、MIT，可以直接调它的 `parser.ParseVideoShareUrl` 做抖音的 `public-web` 单条；不给它设 cookie，设了它会改调自算签名的接口（[06 §1.3](../research/06-platform-access-and-account-risk.md#13-混合路径视觉层拿链接公开页面取内容)）。

<a id="92-包装本机-http-服务"></a>
### 9.2 包装本机 HTTP 服务

例子：小红书单条，调用户自己跑的 XHS-Downloader 服务（它用 `python main.py api` 启动，文档在 `http://127.0.0.1:5556/docs`，接口 `POST /xhs/detail`，要 `Authorization: Bearer` 令牌）。XHS-Downloader 是 GPL-3.0，只能用户自装，插件经 HTTP 调用。

```yaml
settings:
  - {key: endpoint, label: XHS-Downloader 服务地址, type: string, default: "http://127.0.0.1:5556"}
  - {key: xhs-token, label: XHS-Downloader 访问令牌, type: string, secret: true}
capabilities:
  fetch:
    access: public-web
    login: none
    outputs: [text, images, media:video, meta]
permissions:
  network:
    mode: allowlist
    hosts: ["${setting.endpoint}", "*.xhscdn.com"]
  fs:
    write: ["${OUT}"]
```

- `fetch` 把链接（带 `xsec_token` 的原链接，[§3.6](fetch.md#36-fetch-与输出目录)）交给服务，请求体固定写 `download: false`（不让服务下载）和 `check_record: false`（它默认是 `true`，会跳过有下载记录的作品）。拿到元数据和文件地址后，由插件自己把文件写进 `--out`。
- 访问令牌是 `secret: true` 的设置，经 `USUYO_SECRETS` 传入，插件按 `Authorization: Bearer` 带上；它不是平台账号凭据，所以 `login: none` 也拿得到（[§2.11](manifest.md#211-插件设置-settings)）。
- XHS-Downloader 在请求的 `cookie` 参数缺省时用自己配置文件里的 cookie，传空字符串多半也会退回配置文件（推断，待核实）。所以插件 README 和服务连不上时的 `hint` 都写明「请清空 XHS-Downloader 配置里的 cookie」，否则插件声明的 `login: none` 和实际不符。
- 图片和视频地址在小红书的 CDN 上，例子里的 `*.xhscdn.com` 是示例，实际用到哪些主机待核实，照 `usuyo plugin try` 打印的被拦主机补（[§3.2](fetch.md#32-调用约定)）。

<a id="93-包装-mcp-服务"></a>
### 9.3 包装 MCP 服务

插件作为 MCP 客户端，经 stdio 启动上游的 MCP 服务（要 `exec` 权限），把子命令映射成 `tools/call`：

| 子命令 | 映射 |
|---|---|
| `fetch <url>` | 调上游按链接取单条的工具，把 `structuredContent` 转成 fetch 输出，文件写进 `--out` |
| `list <source>` | 调上游列收藏的工具，把分页参数放进游标 |
| `health` | 只启动服务、读工具列表，不调会请求平台的工具 |

工具名照上游的文档填；上游返回的内容同样是不可信数据，只取字段，不执行其中的文字。上游如果还暴露发帖、点赞、私信一类工具，插件不调用，并在 README 写明。

<a id="94-浏览器扩展与油猴脚本"></a>
### 9.4 浏览器扩展与油猴脚本

在用户自己的浏览器里跑的工具（xhs-vault、Weibo-archiver、bilibili-favorites-exporter）有两条路：

- 改一行，把导出的清单直接交给投递接口（[§5.1](delivery.md#51-本机-http-接口)）：

```js
// ==UserScript==
// @name        xhs-favorites-to-usuyo
// @match       https://www.xiaohongshu.com/*
// @grant       GM_xmlhttpRequest
// @grant       GM_getValue
// @connect     127.0.0.1
// ==/UserScript==
function sendTousuyo(links) {
  GM_xmlhttpRequest({
    method: "POST",
    url: "http://127.0.0.1:47615/v1/batches",
    headers: {"Authorization": "Bearer " + GM_getValue("usuyoToken"), "Content-Type": "application/json"},
    data: JSON.stringify({name: "小红书收藏夹", action: "favorite", links: links}),
  });
}
```

- 不改原工具：它照旧导出 JSON 文件，再写一个导入插件读这个文件（[§3.9](fetch.md#39-从文件导入)）。导入插件按自己做的事填 `access: official-export`、`login: none`，和 [§3.9](fetch.md#39-从文件导入) 一致；文件怎么来的写进 `description` 和插件 README，例如「读 xhs-vault 导出的清单；清单由你在登录的浏览器里导出」。手机上的 Agent（Mika 一类）导出的清单同样处理，`description` 写「清单由你手机上的某某导出，导出时用的是它自己的模型」。

令牌由用户粘贴进脚本设置；小红书链接里的 `xsec_token` 只在会话内有效，清单交过来后宜尽快取（[06 §1.3](../research/06-platform-access-and-account-risk.md#13-混合路径视觉层拿链接公开页面取内容)）。

<a id="95-视觉层流程"></a>
### 9.5 视觉层流程

视觉层插件读屏幕上渲染出来的界面、替用户点已登录的客户端。`list` 一页处理几条，只返回 `url`，不带 `ref`；内容由别的插件按 [§3.1](fetch.md#31-运行器与节奏) 去取。底层有两种做法（[调研 10](../research/10-desktop-and-android-agents.md)）：

- **电脑端读系统的无障碍树。** 例如 agent-desktop（Vercel Labs，Apache-2.0）：读 macOS AX、Windows UI Automation、Linux AT-SPI 的树，给可交互的元素编号，动作默认走无障碍接口，不聚焦、不动指针；命令行不要模型、不联网。
- **安卓经 ADB。** 电脑上的获取插件经宿主管的 adb 服务连用户的手机（[§7](isolation.md#7-权限与隔离)），读控件树、截图、按坐标点，在手机侧经 adb shell 读剪贴板；手机上不装 App、不常开无障碍服务。

两种都拿不到链接本身：无障碍树和控件树里没有 URL，链接要「分享 → 复制链接」后读剪贴板。

**例子一：Mac 上读微信收藏里的公众号文章链接**，再交给公众号的 `public-web` 插件取正文（[06 §17.3](../research/06-platform-access-and-account-risk.md#173-首批自己适配的插件)）。

```yaml
platforms: [wechat-mp]
access: vision-ui
login: required
sites:
  - {site: weixin.qq.com, name: 微信, credential: app}
upstream:
  - {name: agent-desktop, url: https://github.com/vercel-labs/agent-desktop, license: Apache-2.0}
capabilities:
  list:
    access: vision-ui
    login: required
    sites: [weixin.qq.com]
    order: newest-first
    interaction: foreground
    timeout: 10m
    sources:
      - {source: "wechat:favorites:articles", title: 微信收藏里的文章, action: favorite}
pacing:
  wechat-mp: {batch: true, interval: 6s, daily: 30}
permissions:
  devices: [accessibility, clipboard, screen-capture]   # screen-capture 只给截图加 OCR 的退路用
  network: {mode: none}
```

- `exec` 不写：agent-desktop 和它的 helper 在 `${PLUGIN_DIR}` 里（[§2.7](manifest.md#27-权限-permissions)）。
- 流程：找到「收藏」点开；对列表容器取快照；每一行先往剪贴板写一个哨兵值，再右键、读菜单、点「复制链接」、读剪贴板；滚一屏，前后两屏有重叠，按 `canonicalize` 后的身份去重，不按整行文字比；连续两屏没有新条目就停。每处理一条发一次 `progress`，取快照之前也先发一次（一次快照可能要几十秒），能力里的 `timeout` 设成分钟级，卡住靠 120 秒无进度发现（[§3.4](fetch.md#34-超时与取消)）。游标记最后一条的链接和已读条数，下一页先在可见的几行里找它，找不到就回到顶部重新滚。`title` 原样保留界面文字，拿不到收藏时间就让 `actedAt` 留空。
- 为什么是 `foreground`：macOS 上走无障碍接口的右键（`AXShowMenu`）照样把菜单弹到屏幕上，读链接要改剪贴板；Windows 上右键只能用物理输入，要抢焦点、动鼠标。
- **受版本限制。** Mac 微信 4.1.x 在 macOS 26 上只给浅层窗口（报告测的是聊天界面，收藏页同样读不到是推断）；PC 微信 4.1 起没开过无障碍模式的新账号拿不到树。读不到时回 `upstream-changed`，`hint` 写「这个版本的微信不给读界面」；要不要退到「截窗口 → 本机 OCR 找行 → 按坐标点」，是待定项（[§13](open.md#13-待定) 第 17 项）。Windows 版另写一份。
- 改造 wechat-favorites-exporter 的做法时保留它的 MIT 版权声明（[§9.6](#96-许可证)）；流程总共几步，自己写比 fork 省事。
- `pacing` 是示意值：每条大约四次 agent-desktop 调用，加 0.5 秒左右的等待，没有微信这边的阈值数据（推断）。

**例子二：安卓手机上的小红书收藏，经 ADB**（包名 `com.xingin.xhs` 出自 Mika 的 App 知识文件）。

```yaml
platforms: [xiaohongshu]
access: vision-ui
login: required
sites:
  - {site: www.xiaohongshu.com, name: 小红书, credential: app}
capabilities:
  list:
    access: vision-ui
    login: required
    sites: [www.xiaohongshu.com]
    order: newest-first
    interaction: foreground
    timeout: 10m
    sources:
      - {source: "xiaohongshu:favorites", title: 我在手机上收藏的笔记, action: favorite}
pacing:
  xiaohongshu: {batch: true, interval: 10s, daily: 60}
settings:
  - {key: display, label: 在哪块屏上跑, type: enum, options: [main, virtual], default: main}
permissions:
  devices: [adb]
  network: {mode: none}
requires:
  runtime: ["adb"]
run:
  command: ["${PLUGIN_DIR}/usuyo-fetch-xiaohongshu-android"]
  serve: true
```

- 默认在主屏上跑：跑的时候手机要连着、解锁、亮屏，不能用。后台虚拟屏是插件设置（`display`），默认关：它多出一块公开屏，App 自己就查得到不在 0 号屏上（推断）。微信在 Mika 的虚拟屏上整屏读不到、点按没反应（Mika #237，只在虚拟屏上测过，主屏上行不行待核实），所以微信和视频号不用虚拟屏。
- 读树用 `uiautomator dump`，它要等屏幕安静一秒，十秒内等不到就失败；自动播放视频的页面改用截图加本机 OCR。
- 链接：每条点开 → 分享 → 复制链接，在手机侧经 adb shell 读剪贴板；不靠 scrcpy 的剪贴板同步，那会盖掉电脑上的剪贴板。不用屏上位置当 `ref`，新收藏会把位置往后挤。小红书分享链接里的 `xsec_token` 只在会话内有效，宜尽快取（[§9.4](#94-浏览器扩展与油猴脚本)）。
- 流程格式可以参考 Mika 的路线回放（Route、Selector、Anchors，Apache-2.0，借代码要保留 LICENSE、NOTICE 和改动说明）：选择器记角色、名字、资源 ID、位置、同类序号；每步核对前台包名和几段稳定文字，对不上就停（`upstream-changed`），碰到验证或风控提示回 `blocked`。
- `pacing`、`requires.runtime` 都是示意值，没有证据支撑具体数字。一次调用内部的节奏可以参考 Mika 给 `careful` 一类 App 的数：换一屏停 2.5–5 秒，按屏上字数每字加 15 ms，一次至多换 24 屏；`interval`、`daily` 另按平台给。

两种做法共用的规则：

- **流程固定下来**（Maestro 的 YAML、mirroir-mcp 的录制回放、Mika 的路线一类），不每次让模型现场决定；客户端改版就要改流程。运行时默认不用模型，对不上就停。能力声明了 `provider: optional` 的，用户打开后可以让看图模型找一个控件，找到后再在树里核对；模型经 [§4.3](transform-export.md#43-模型与密钥由宿主代发) 代发，本机和云端都行，云端的界面标「离开本机」。
- 屏幕或手机没解锁、窗口被挡住时发 `waiting` 事件；等不到就返回 `dependency-missing`，`needs` 照 [§3.3](fetch.md#33-信封与错误码) 的前缀写，例如 `["state:unlocked"]`、`["permission:accessibility"]`、`["app:wechat"]`、`["state:authorized"]`。
- **底层工具的错误怎么翻**（plugin-kit 的视觉层 SDK 做）：没给系统授权 → `dependency-missing` 加 `permission:accessibility` 或 `permission:screen-capture`；客户端没开 → `dependency-missing` 加 `app:`；窗口里是登录或扫码页 → `login-required`；树是空的或太浅、找不到列表 → `upstream-changed`；界面上出现验证、风控或下线提示 → `blocked`；元素过期、对不上 → 插件自己重新快照再试，还不行回 `upstream-changed`；快照没取全不算错，回已经读到的条目，`nextCursor` 指回这一屏。
- **隐私**（视觉层 SDK 做，契约测试也查）：
  - agent-desktop 的状态目录设在 `${TMP}`（常驻时设在 `${HOME}`），不用 `${CACHE}`：它的元素映射里存着界面上的文字。不开会话、不开 trace、不用 `--screenshots`。
  - 进了列表以后每次快照都限在列表容器上（`--root`），整窗快照只在导航时做一次：微信主窗口里有联系人名字和消息预览。
  - 截图只截目标窗口，写在 `${TMP}`，本机 OCR 用完马上删，不进 `${OUT}`。
  - 剪贴板：一页开始前把原内容读进内存，一页结束写回；全程不写日志、不写盘。剪贴板管理工具会记下客户端复制出的每条链接，宿主拦不住；第一次用到 `clipboard` 时界面说清楚。
  - stderr 不写屏幕上的文字、标题、链接，只写计数和错误码。
- **包 agent-desktop 时**：只按名字调 `${PLUGIN_DIR}` 里那份固定版本；检查信封的版本号，主版本和锁定的不一样就回 `internal`；只有 `interaction: foreground` 的能力才用 `--headed`；每条命令都带 `--app`（不带时它读的是前台 App），最好也带 `--window-id`；不用 `launch --cdp`（那等于在官方客户端里执行代码，按 [§2.4](manifest.md#24-access-与-login) 要记 `client-hook`），不用 `skills`，也不用它仓库里的 JEV 脚本（它把屏幕描述发给第三方的模型服务，不经 [§4.3](transform-export.md#43-模型与密钥由宿主代发)）。

<a id="96-许可证"></a>
### 9.6 许可证

按 [06 §17.4](../research/06-platform-access-and-account-risk.md#174-fork-与许可证)：MIT、Apache、BSD 一类可以 fork、可以打进插件资产，保留版权声明；GPL 系只做用户自装、插件以独立进程或 HTTP 调用，不随资产分发（[ADR-003 §8](../decisions/ADR-003-plugins.md#8-gpl-工具)）；非商业或源码可见许可不打包、不进两个组织；没有许可证的只参考做法。插件 README 写明上游、许可证和改动，`upstream` 字段照填。上游的发布包里没有 LICENSE 文件时（agent-desktop 就是这样），从上游仓库取一份放进资产。

自己写的获取插件用 MIT，和 plugin-kit 一致。两个组织里的插件不调用用户自装的非商业许可工具；别人在自己仓库里这样做不管。
