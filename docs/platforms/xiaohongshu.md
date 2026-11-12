<a id="小红书获取设计"></a>
# 小红书：获取设计

文中「协议」指[插件协议](../protocol/README.md)，「06」指[调研 06](../research/06-platform-access-and-account-risk.md)。各平台的总表、要不要账号的几个说法、怎么相对安全地试，在 [platforms/README](README.md)；这里不重复。

<a id="0-结论"></a>
## 0. 结论

| 平台 | 首个实现 | 兜底 | 做不到的 |
|---|---|---|---|
| 小红书 | `fetch` 走公开笔记页（带 `xsec_token` 的长链，解析 `__INITIAL_STATE__`），不签名、不登录，被送回首页时带一份访客身份再试一次；清单由你在日常浏览器里用油猴脚本交给 [`/v1/batches`](../protocol/delivery.md#51-本机-http-接口)，插件没有 `list` | 你自装的 XHS-Downloader，按 [协议 §9.2](../protocol/wrapping.md#92-包装本机-http-服务) 包装 | 不带任何 cookie 取单条不稳定：2026-10 有报告空 cookie 的长链被重定向到首页，带一份未登录的访客 cookie 才成功（[§2.2](#22-取单条)）；访客身份由清单的 `visitors` 声明，usuyo 开一个不登录的窗口取（[协议 §2.5.1](../protocol/manifest.md#251-访客身份-visitors)） |

最大的风险在小红书：2026 年只读也有警告、封号的报告（[06 §16.2](../research/06-platform-access-and-account-risk.md#162-issue-里的账号问题)），匿名批量取还牵连过复制链接的账号（[XHS-Downloader #305](https://github.com/JoeanAmier/XHS-Downloader/issues/305)）。

<a id="2-小红书"></a>
## 2. 小红书

<a id="21-列出你存过的"></a>
### 2.1 列出你存过的

- **官方导出**：没有。[隐私政策](https://agree.xiaohongshu.com/h5/terms/ZXXY20220509001/-1)只列 App 内查看「笔记/收藏/赞过」，其他按邮件申请（official-export 笔记（同组笔记 `official-export-findings.md`，不在仓库里））。可以发邮件试，结果不能当计划。
- **日常浏览器里的脚本**（`browser-session`，低到中）：[wkwunju/xhs-vault](https://github.com/wkwunju/xhs-vault)（MIT，2026-08-18）的 `userscript/xhs-export.user.js` 改写 `XMLHttpRequest`，只截获收藏页自己发出的 `collect`、`board` 请求，取出 note id、`xsec_token`、标题；另有一个按钮自动滚到底。自动滚动属于节奏异常，去掉，改成你自己往下翻。「赞过」页同理，接口名待核实。usuyo 用的脚本（`usuyo-fetch/xiaohongshu` 的 `userscript/`）就是这样改的：只记你自己的「收藏」「赞过」（请求里的 `user_id` 要是页面上登录的那个人），记下笔记 id、`xsec_token` 和标题，12 小时后作废；你点「交给 usuyo」时，先和本机的 usuyo 握手（发一个随机数和投递令牌的 id，核对对面用整个令牌签回的证明，[SPEC 本机接口](../spec/local-api.md#本机接口第-2-层11-11-照代码)），核对过了才带令牌把两批交给 `POST /v1/batches`，核对不过什么都不发。这条也不是零风险：小红书网页会对数据采集类扩展弹「有害插件」提示（[XHS-Downloader #283](https://github.com/JoeanAmier/XHS-Downloader/issues/283)），经扩展驱动用户自己已登录的 Chrome 读小红书也被判过违规（[OpenCLI #842](https://github.com/jackwener/OpenCLI/issues/842)）。只截获、不发额外请求的油猴脚本没有找到处罚报告。
- **自动化浏览器**（`browser-session`，高）：Playwright 打开要登录的页面返回 300012（[web-image-sourcing](https://github.com/danzey0425-lgtm/web-image-sourcing)）；[xiaohongshu-mcp](https://github.com/xpzouying/xiaohongshu-mcp) 的用户一两天内被警告、封号（[#720](https://github.com/xpzouying/xiaohongshu-mcp/issues/720)、[#726](https://github.com/xpzouying/xiaohongshu-mcp/issues/726)、[#728](https://github.com/xpzouying/xiaohongshu-mcp/issues/728)），10 月还有搜索被限（[#877](https://github.com/xpzouying/xiaohongshu-mcp/issues/877)）。它的「收藏」工具是收藏动作，不是列清单。
- **本地签名**（`private-api`，高）：[redbook](https://github.com/lucasygu/redbook)、[MediaCrawler](https://github.com/NanmiCoder/MediaCrawler)、[Spider_XHS](https://github.com/cv-cat/Spider_XHS)，签名库是 [xhshow](https://github.com/Cloxl/xhshow)，有永久封禁记录（[MediaCrawler #865](https://github.com/NanmiCoder/MediaCrawler/issues/865)）。
- **翻页**：收藏页是游标翻页，每页约 20–30 条（推断）。`xsec_token` 只在会话内有效，有项目实测几小时后链接 404（06 [§1.3](../research/06-platform-access-and-account-risk.md#13-混合路径视觉层拿链接公开页面取内容)），所以清单交来后要在当天取完。

<a id="22-取单条"></a>
### 2.2 取单条

- **页面**：`https://www.xiaohongshu.com/explore/<id>?xsec_token=…`（`discovery/item/`、`user/profile/<作者>/<id>` 同样），HTML 里的 `window.__INITIAL_STATE__` 有标题、正文、图片列表、视频流、封面、作者、发布时间、标签、互动数。解析前要把 JS 的 `undefined`、`new Map([])`、`new Set([])` 换成 JSON；`new Set` 是 10-15 前后新出现的，XHS-Downloader 当时整批失败（[#497](https://github.com/JoeanAmier/XHS-Downloader/issues/497)）。
- **这个月的变化**：[#495](https://github.com/JoeanAmier/XHS-Downloader/issues/495)（10-14）实测「长链加空 cookie」被重定向到首页，「长链加未登录访客 cookie」成功；短链（`xhslink.cn`）跳转时 cookie 没带上会失败，维护者说只有部分链接这样。[#499](https://github.com/JoeanAmier/XHS-Downloader/issues/499)（10-22）也是设了 cookie 才取到。[ParseHub](https://github.com/z-mio/ParseHub) 10-14 给小红书加了游客 session 和移动端 UA 回退，10-27 适配了新的图片 CDN 地址；它取游客 session 要用 xhshow 签名，属 `private-api`，不照搬。
- **要登录的部分**：没有 cookie 时视频只有低清（XHS-Downloader README）；评论要调签名接口，不取。
- **分享链接、不登录**：手机分享出的短链展开后带跟踪参数。#305 的用户没填 cookie、批量取约 108 条后，复制链接的账号被下线。哪些参数能去掉、去掉后还取不取得到，要你实测（[§2.5](#25-测试)）。

<a id="23-基础与打包"></a>
### 2.3 基础与打包

| 项目 | 许可证 / 语言 | 最近推送 | 用法 |
|---|---|---|---|
| [wkwunju/xhs-vault](https://github.com/wkwunju/xhs-vault) | MIT / JS、Python | 2026-08-18 | fork 油猴脚本：留 XHR 截获，删自动滚动和导出 `cookies.json`，改成 `POST /v1/batches`；`scraper/fetch.py` 是取单条的参照 |
| [chenjin-cmd/xhs-account-teardown](https://github.com/chenjin-cmd/xhs-account-teardown) | MIT / Python | 2026-09-29 | 单条脚本的参照 |
| [z-mio/ParseHub](https://github.com/z-mio/ParseHub) `provider_api/xhs.py` | MIT / Python | 2026-11-09 | 读它的 `new Set` 处理、CDN 地址换算、移动端回退；不用签名部分 |
| [JoeanAmier/XHS-Downloader](https://github.com/JoeanAmier/XHS-Downloader) | GPL-3.0 / Python | 2026-11-07 | 兜底：你自装，插件经 HTTP 调；[#502](https://github.com/JoeanAmier/XHS-Downloader/pull/502) 正在加扫码登录和「风控暂停」 |

清单：`platforms: [xiaohongshu]`，`match` 是上面四种长链和两种短链，`access: public-web`、`login: none`，顶层 `visitors` 声明 `www.xiaohongshu.com` 的访客身份（窗口打开 `https://www.xiaohongshu.com/`），`fetch` 列 `visitors: [www.xiaohongshu.com]`，`outputs: [text, images, media:video, meta]`。`canonicalize` 取 24 位十六进制 note id 作 `contentId`，`identity.url` 写 `https://www.xiaohongshu.com/explore/<id>`，`resolvedUrl` 保留 `xsec_token`（[协议 §3.5](../protocol/fetch.md#35-canonicalize-与身份)）。节奏 `batch: true`、`interval: 15s`、`daily: 100`。

<a id="24-插件做成的样子"></a>
### 2.4 插件做成的样子

插件在 `usuyo-fetch/xiaohongshu`，照上面的做法：油猴脚本交清单，插件用访客身份取公开页。兜底仍是 XHS-Downloader 包装插件，没写。三个月后最可能变的是 SSR 字段和访客 cookie 的要求，所以解析只在 `src/state.rs` 一个文件里，字段缺了回 `upstream-changed`。

- **`canonicalize`**：四种长链（`explore/<id>`、`search_result/<id>`、`discovery/item/<id>`、`user/profile/<作者>/<id>`）和 `xhslink.com`、`xhslink.cn` 短链；短链只请求短链主机、只看它跳去哪里，不带 cookie。
- **`fetch`**：用普通桌面 UA 打开长链，先不带 cookie；请求里只带 `xsec_token`、`xsec_source`，手机分享链接里指向分享人的参数不发出去。被送回首页或登录页时，宿主交了访客身份（`USUYO_VISITOR`）就带上再试一次，还不行、或没交，回 `visitor-required`；访客 cookie 只发给 `www.xiaohongshu.com` 的笔记页、只走 https。清洗后解析 `__INITIAL_STATE__`，图片、视频只下页面给的、在 `*.xhscdn.com` 上的地址，写进 `--out`；缺的写进 `missing`、`missingReason`。验证页回 `blocked`，链接里没有 `xsec_token` 回 `not-found` 加 `reason: link-expired`。
- **`health`**：不发请求，`--probe` 也不发：笔记链接里的 `xsec_token` 几小时就失效，没有一篇长期可用的笔记能拿来试。
- **油猴脚本**（`userscript/usuyo-xhs-saved.user.js`，改写自 xhs-vault）：见 [§2.1](#21-列出你存过的)。
- **测试**：全离线，对着假笔记页、假短链服务和手写的页面样例（含 `undefined`、`new Map`、`new Set`）；脚本对着假的小红书列表接口和一次性的 usuyo。
- 还没有的：plugin-kit 的平台列表（`xiaohongshu` 的 `contentId` 正则 `^[0-9a-f]{24}$` 和共用的 `testdata/canonicalize.json`）；宿主现在只查 `contentId` 非空、不超过 256 字节、没有控制字符。

<a id="25-测试"></a>
### 2.5 测试

- **不用账号能测**：`canonicalize` 全部；解析器对保存的 HTML；在一个从没登录过的浏览器里打开公开笔记页，拿访客 cookie 跑 `fetch` 冒烟，每次几条、间隔十几秒。注意开发机和你的账号可能同一 IP（推断有关联）。
- **只有你能测**：收藏页、赞过页的截获和完成判定；手机分享链接去掉哪些参数仍可取；取一批（几十条）后账号有没有提示；邮件申请导出的结果。
