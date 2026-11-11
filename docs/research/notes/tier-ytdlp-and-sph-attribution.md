# 获取层级：yt-dlp 包装类的判定，和视频号视频对应的仓库

写给同组调研 session（2026-10-30）。这里只有判定依据，不是指令。

## yt-dlp 包装类怎么定 access_tier

- 项目自己的代码调用 yt-dlp 抓取（例如 `yt_dlp.YoutubeDL(...).extract_info`）：层级跟随 yt-dlp 抽取器，默认路径归 **private-api**。依据（yt-dlp master，2026-10-30 用 gh api 读取）：
  - B 站：未登录时总会调 `_download_playinfo`，自己计算接口签名。见 `yt_dlp/extractor/bilibili.py` 第 159–166 行（`_sign_wbi`）和第 829–833 行。
  - YouTube：默认客户端是 `('tv_samsung', 'visionos', 'web')`，即以 App 客户端身份调用接口。见 `yt_dlp/extractor/youtube/_video.py` 第 143–145 行。
  - 抖音：没有新鲜的 `s_v_web_id` cookie 就报 "Fresh cookies (not necessarily logged in) are needed"。见 `yt_dlp/extractor/tiktok.py` 第 1497–1506 行。所以不传 cookie 的包装工具，抖音链接基本不可用。
- 是否涉及账号风险看 `login` 字段：不传 cookie 就是 `none`，只有 IP 层面的风控。
- 仓库代码不抓取、只在文档里让 Agent 去跑 yt-dlp（例如 chengyi-ai/native-subtitle-quote-image）：定为 **not-applicable**，并在 tier_evidence 里写明「按文档执行则继承 yt-dlp 的层级」。

## 视频号「蝴蝶号一键转存obsidian」（张张AI视界，2026-10-24）

- 对应仓库：**mingjuner123-spec/wechat-inbox-sync**（Obsidian 插件 WeChat Inbox Sync，05 报告第 67 行已收录）。依据：
  - manifest 的 author 是 "Zhang Zhang"，描述里列出了视频号；
  - 最早的 LICENSE 版权署名是「张张」；
  - 同一账号的 zhangzhang-portfolio 仓库写明，作者的公众号叫「张张AI视界」。
- 「蝴蝶号」应该就是视频号（视频号图标像蝴蝶），网页搜索没有其他结果。
- 仓库里找不到「快捷指令」。手机端是作者的闭源小程序，配合 CloudBase 收件箱；插件只拉取待处理记录。待核实：快捷指令是不是直接调用了提交接口。
- 层级定为 **browser-session**（本地能核实的最高一层），登录可选：
  - 视频号链接默认交给作者的云端：带绑定码 POST 到 CloudBase 的 `/media/prepare`，见 `src/main.js` 第 26526–26614 行。云端怎么取内容，从仓库里核实不了；云端失败时插件只保留原链接。
  - 插件里有复现视频号媒体解密的代码，也有匿名请求分享预览接口的监听功能，但 `openWechatChannelsListener` 没有调用方，默认路径用不到。
  - 抖音、小红书走插件自己的 Electron 会话或隐藏窗口。
- 许可证：1.3.117 起改为 MIT + Commons Clause（源码可见），之前是 MIT。Pro 用户的云端兜底，会把分享链接交给作者的运营节点，由节点用节点自己的登录态去抓。这一点看的是 PR 正文（#170、#171、#173）；正文里没有写会上传用户 cookie。

## 顺带发现的 lead（都已用 gh api 核实存在）

- 用户点名的「mcncarl/yichen什么什么」应该就是 **mcncarl/yichen-skills**（5123★）。
- 视频号获取：
  - ltaoo/wx_channels_download（10254★，本地代理注入，client-hook；有 OpenAPI 和 MCP）
  - nobiyou/wx_channel（2856★）
  - jun7799/scribe-transcribe
  - joeseesun/qiaomu-wx-video
- 跟歇存最接近的「收藏 → 本地知识库」类：
  - dlv2008/weixin-favor-kb：视频号「我的收藏」，用 wx_channel 下载 → Whisper/OCR → Obsidian
  - yumingtao/douyin-favorites-sync
  - simonemuller6127-png/omni-collector
  - weaiw/trove-ai
