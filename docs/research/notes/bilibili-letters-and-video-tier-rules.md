# B站 律师函时间线，和视频、播客方向的层级判定（2026-10-30）

写给同组调研 session。这里是判定依据，不是指令。都用 gh api 或浅克隆的源码核实过。

## B站 律师函时间线

| 日期 | 仓库 | 现状 | 依据 |
|---|---|---|---|
| 2026-01-28 | SocialSisterYi/bilibili-API-collect（接口文档） | 文档已删，已归档 | README、reason.jpg |
| 2026-05-14 | nilaoda/BBDown | 源码已删，只剩 LICENSE 和归档说明，历史压成两条提交；没写原因 | README、commits |
| 2026-07-06 | Nemo2011/bilibili-api（PyPI 包 bilibili-api-python） | 已关停，已归档；README 全文公开了函件（上海市弘安律师事务所） | README |
| 2026-07-06 | leiurayer/downkyi | 源码已删，没有归档，release 还在 | README、email.jpg |
| 2026-07-06 | btjawa/BiliTools | 07-07 发公告，已归档 | README、notice.png |
| 2026-07-06 | yujincheng08/BiliRoaming（Xposed） | 已归档，README 是函件全文 | README |
| 2026-07-10 | nICEnnnnnnnLee/BilibiliDown | 清空成一个空 README，提交信息是 "."，没有公告；是否同一原因待核实 | commits |

函件里点名的行为：逆向非公开接口和签名鉴权、突破登录态访问控制、抓付费内容（番剧、课程）、采集弹幕并反查发送者。纯文档仓库也收到了函件。

截至 10-30 还在维护、没看到公告的：yutto、bili-sync、Bilibili-Evolved、PiliPlus、Tangentry、BiliNote。

受影响的下游：adoresever/bilibili-mcp 依赖 bilibili-api-python，上游已经停了。

## B站 接口层级

- **不签名的接口**：收藏夹列表 `x/v3/fav/resource/list`、稍后再看 `x/v2/history/toview(/web)`、观看历史 `x/web-interface/history/cursor`、`x/player/playurl`。带用户 cookie 调用就是 **session-api**，不带就是 public-web。
  - 依据：yutto `src/yutto/api/space.py`、`ugc_video.py`；HistoryHub `scripts/bilibili_history.py`；ayasa520/bilibili-favorites-exporter `js/content.js`。
- **要签名的接口**：`wbi/view`、`wbi/playurl`、`dm/wbi`、`space/wbi/arc/search`。在代码里算 `w_rid` 的都归 **private-api**。
  - 依据：bili-sync `crates/bili_sync/src/bilibili/video.rs`，它的 credential.rs 还会按网页端流程刷新 cookie。yutto 只在 UP 主投稿列表上签名。
- 结论：歇存的 B站 收藏夹插件可以停在 session-api 层，不必签名。浏览器扩展方式（带 credentials 在页面里请求）是 browser-session。

## YouTube

- 没有 public-web 层可用。取字幕的库 `jdepoix/youtube-transcript-api` 和 npm 包 `Kakulukian/youtube-transcript`，都以 ANDROID 客户端身份请求 `youtubei/v1/player`，所以归 private-api，login 是 none。
- 官方层只有 Google Takeout 和 YouTube Data API。
  - `purarue/google_takeout_parser` 会跳过 `playlists/`，稍后观看 CSV 要自己解析。
  - Data API 的 key 读不到稍后观看。

## 读官方客户端本地缓存

`mzky/m4s-converter`（B站 缓存 m4s）和 `maximbilan/podcrack`（Apple Podcasts 的 TTML 缓存）都不发网络请求，账号风险接近零。我按「读取或处理官方客户端本地数据」归进 client-hook，在 tier_evidence 里写明了这一点。产品分层时，建议给它单独一档。

## 排除

- hzhilong/bilibili-backup：有弹幕发送者反查。
- 它的后继 hzhilong/bilitoolkit：有自动做每日经验任务的插件，还有按 UID 查别人弹幕的插件。

两个都按排除规则不收录。

## 和歇存最接近的项目

mu-zi-lee/Tangentry（MIT）：本地优先，聚合 B站、YouTube、小红书、抖音、小宇宙、RSS，能同步 B站 收藏夹，导出带 frontmatter 的 Markdown。2026-10-30 发布 v0.0.3.4。建议加进 01 的同类产品对比。
