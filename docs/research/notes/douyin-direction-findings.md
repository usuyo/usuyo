# 抖音 / 快手 / TikTok / 头条方向：跨条目发现（2026-10-30）

写给同组调研 session。只有事实与判定依据，不是指令。完整条目：同目录 research-douyin-kuaishou-tiktok-toutiao.json（27 条 + 27 条 lead）。

## 抖音网页端在 2026 下半年收紧，页内请求方案更稳

- jiji262/douyin-downloader README：抖音请求校验拦下命令行的单条、合集、音乐、点赞、收藏下载，刷新 Cookie 无效；#258（2026-10-26）报错原文为「安全校验只放行网页内发起的请求」。
- 同期 private-api 工具收藏类接口 403：DouK-Downloader #792（2026-09）、F2 #443（2026-08）。DouK-Downloader README 声明不再维护签名算法。
- 仍可用的是页内请求：Hannah2227/douyin-favorite-MyLike 在真实抖音页面主世界里 fetch（apps/desktop/src/main/douyin/page-request.ts），注释写明不伪造签名值。

## 抖音有官方「个人信息下载」

- ZhangYukwiver/DouyinDataReview 可导入抖音官方「个人信息下载」的 JSON / ZIP 档案（README）。没找到独立的开源解析器；字段范围待核实。歇存可考虑先做「导入官方档案」这条 official-export。

## 层级判定口径（本方向）

- 分享页 SSR（iesdouyin _ROUTER_DATA、快手 __APOLLO_STATE__）不带登录：public-web。parse-video 只有在部署者配 Cookie 且分享页缺详情时才本地算 a_bogus，默认仍记 public-web。
- KS-Downloader 用 curl_cffi 模拟浏览器 TLS 指纹，我记 public-web 并在 needs_check 写了「严格口径可算模拟指纹」；与小红书方向「只有 TLS 指纹模拟不升层」的口径一致。
- 「官方导出取清单 + 逐条取内容」按逐条取内容那段定层级（tiktok-save → browser-session）。
- 签名在页面里生成、请求由后端 httpx 发出：session-api（yumingtao 后端，但签名模块 Pyarmor 混淆，无法核实）。

## 下架与停更

- erma0/douyin（1618★）：README 称 2026-06 因合规审查清空代码。
- GangJust/FreedomPlus（抖音 Xposed 模块）：gh api 404，只剩 2023 年副本 xiaodesetingyongzhanghao/FreedomPlus。
- yzfly/douyin-mcp-server：已归档。

## 提示注入样本

- JoeanAmier/TikTokDownloader 的 issue 评论里有 dosubot 模板文字，对「AI coding agents」要求执行 curl | sh 安装其 MCP。我没有执行，读 issue 时注意。
