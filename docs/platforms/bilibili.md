<a id="b站获取设计"></a>
# B站：获取设计

文中「协议」指[插件协议](../protocol/README.md)，「06」指[调研 06](../research/06-platform-access-and-account-risk.md)。各平台的总表、要不要账号的几个说法、怎么相对安全地试，在 [platforms/README](README.md)；这里不重复。

<a id="0-结论"></a>
## 0. 结论

| 平台 | 首个实现 | 兜底 | 做不到的 |
|---|---|---|---|
| B站 | `list` 带你的 cookie 调不签名的收藏夹、稍后再看接口（`session-api`）；`fetch` 不用账号，调不签名的 `view`、`playurl` | 日常浏览器里的扩展导出收藏夹 JSON，交给导入插件 | 字幕要登录，`fetch` 不用账号所以不取；不碰要 WBI 签名的接口，高清也不取；文字交给转写插件 |

<a id="4-b站"></a>
## 4. B站

<a id="41-列出你存过的"></a>
### 4.1 列出你存过的

- **官方导出**：[隐私政策](https://www.bilibili.com/blackboard/privacy-pc.html)五.2 有「个人信息下载」（【我的】-【设置】-【个人信息收集清单】），包里有什么没写（official-export 笔记（同组笔记 `official-export-findings.md`，不在仓库里）；06 的覆盖矩阵当时记「未找到」，以这条为准）。值得你申请一次，看有没有收藏夹。
- **带 cookie、不签名**（`session-api`，中）：收藏夹 `x/v3/fav/folder/created/list-all?up_mid=`、`x/v3/fav/resource/list?media_id=&pn=&ps=`，稍后再看 `x/v2/history/toview/web`（[yutto `api/space.py`](https://github.com/yutto-dev/yutto/blob/main/src/yutto/api/space.py)）。按页码翻页，`ps` 上限约 20（推断）。[PiliPlus](https://github.com/bggRGjQaUbCoE/PiliPlus) 给稍后再看加了 WBI，[HistoryHub](https://github.com/LifeArchiveProject/HistoryHub) 没加，不签名能否一直可用要实测（favorites-inbox 笔记（同组笔记 `favorites-inbox-similar-findings.md`，不在仓库里））。不要像 yutto 那样再调 `space/wbi/acc/info` 取用户名，那一步要签名（bilibili-video-verify 笔记（同组笔记 `bilibili-video-verify-corrections.md`，不在仓库里））。
- **点赞**：没有找到列出自己点赞视频的现成工具；空间页有「最近点赞的视频」，接口和条数上限未核实，不做。
- **公开收藏夹**：收藏夹设为公开时，不带 cookie 可能也列得出（推断，要测）。
- **日常浏览器扩展**（`browser-session`）：[ayasa520/bilibili-favorites-exporter](https://github.com/ayasa520/bilibili-favorites-exporter)（MIT，2025-09 后没更新）导出 JSON。
- **平台会做什么**：带登录态批量下载的账号有长期风控，[bili-sync #756](https://github.com/amtoaer/bili-sync/issues/756)（10-20）主账号半年下载都提示风控；一例大会员被冻结（[yutto #452](https://github.com/yutto-dev/yutto/issues/452)，因果不明）；缺指纹参数的部分接口返回 412（[BiliNote #397](https://github.com/JefferyHcool/BiliNote/issues/397)）。2026 年的律师函针对签名、登录态、付费内容和弹幕反查（[06 §16.3](../research/06-platform-access-and-account-risk.md#163-下架停更与法律风险)）。只列清单、不批量下高清视频，风险最小。

<a id="42-取单条"></a>
### 4.2 取单条

- **不登录**：`x/web-interface/view?bvid=` 给标题、简介、作者、发布时间、封面、分 P；`x/player/playurl`（不签名）给低清音视频，要 ffmpeg 合并（[yutto `api/ugc_video.py`](https://github.com/yutto-dev/yutto/blob/main/src/yutto/api/ugc_video.py)）。
- **要登录**：高清；字幕。yutto 不签名调 `x/player/wbi/v2` 取字幕，带 cookie 才有，标 `need_login_subtitle` 的不登录拿不到。拿不到字幕时文字交给转写插件。
- **评论、弹幕**：不取。
- **分享链接、不登录**：`b23.tv` 短链展开后是普通视频页，能拿到上面「不登录」的全部。

<a id="43-基础与打包"></a>
### 4.3 基础与打包

| 项目 | 许可证 / 语言 | 最近推送 | 用法 |
|---|---|---|---|
| [yutto-dev/yutto](https://github.com/yutto-dev/yutto) | GPL-3.0 / Python | 2026-11-09 | 只读，看接口和参数，不抄代码；也可做你自装的包装插件 |
| [amtoaer/bili-sync](https://github.com/amtoaer/bili-sync) | MIT / Rust | 2026-09-10 | 可借收藏夹请求代码（`fav_time` 的处理在 `crates/bili_sync/src/utils/convert.rs`，`DedeUserID` 在 `bilibili/credential.rs`）；它会签名、按网页流程刷新 cookie，这两部分不要 |
| [ayasa520/bilibili-favorites-exporter](https://github.com/ayasa520/bilibili-favorites-exporter) | MIT / JS | 2025-09-09 | 兜底：扩展导出 JSON，写导入插件读 |

清单：`platforms: [bilibili]`，`match: ["www.bilibili.com/video/*", "m.bilibili.com/video/*", "b23.tv/*"]`；`fetch` 记 `public-web`、`login: none`，`timeout: 30m`，`outputs: [text, images, media:video, media:audio, meta]`；`list` 记 `session-api`、`login: required`、`order: newest-first`，清单里写死「稍后再看」这一个来源（`bilibili:toview`），收藏夹由 `sources` 现列；`sites` 写 `www.bilibili.com`、`credential: cookies`、`domains: [".bilibili.com"]`，登录判定 cookie `SESSDATA`。插件不调 cookie 刷新接口（[协议 §6](../protocol/credentials.md#6-凭据)）。`contentId` 用 BV 号，分 P 写 `BV…:p2`。节奏 `batch: true`、`interval: 5s`、`daily: 500`。

<a id="44-插件做成的样子"></a>
### 4.4 插件做成的样子

插件在 `usuyo-fetch/bilibili`：`list` 调不签名的收藏夹、稍后再看接口，`fetch` 不用账号调 `view` 和 `playurl`。平时只取文字、元数据和封面（`--want text,meta,images`），视频按需要再取。兜底仍是扩展导出 JSON 加导入插件，再不行用官方的个人信息下载。

- **`canonicalize`**：BV 号、av 号、`b23.tv` 短链和分 P；av 号离线换成 BV 号，只有短链发请求，只跟重定向，不带 cookie。
- **`sources`**：列你建的收藏夹和「稍后再看」；`mid` 取自 cookie `DedeUserID`，不调签名接口。
- **`list`**：一页一调，收藏夹按页码翻页，`actedAt` 用收藏时间 `fav_time`；稍后再看一次给完，按加入时间从新到旧。
- **`fetch`**：`view` 的原样回答、封面、按 `--want` 的最低画质视频流和音频流（两个文件，`media[]` 里带同一个 `group`：`cid:<这一 P 的 cid>`，合起来交给宿主的转换）。即使你登录过，也不读凭据、不带 cookie，所以不取字幕。拿不到的写进 `missing`、`missingReason`。
- **cookie 只用在 `sources`、`list`**，只发往 `api.bilibili.com`；封面和视频流只连 `*.hdslb.com`、`*.bilivideo.com`、`*.bilivideo.cn`，也不带 cookie。不带 `bili_jct`。
- **错误码**（推断，照抓到的响应核对）：`-412`、`-352`（以及 HTTP 412、回答里带 `v_voucher`）回 `blocked`，`-101` 回 `login-required`，`-799`、`-509`、HTTP 429 回 `rate-limited`。
- **`health`**：不发请求；`--probe` 才用登录信息请求一次 `nav`。
- **测试**：全离线，对着本机的假服务器；`view.json` 是一次真实的公开视频回答（封面地址换成本机），其余照公开项目的代码手写。
- 登录窗口（扫码）由宿主管，插件只读宿主交来的 `USUYO_CREDENTIALS`。

<a id="45-测试"></a>
### 4.5 测试

- **不用账号能测**：`canonicalize`；公开视频的 `view`、低清 `playurl`、下载合并；公开收藏夹能否匿名列出。
- **只有你能测**：私密收藏夹和稍后再看、字幕、高清；连续列几百条后有没有 `-352`；个人信息下载包里有什么。
