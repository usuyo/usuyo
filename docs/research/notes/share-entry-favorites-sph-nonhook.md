# 手动分享入口、收藏页现成流程、视频号非 Hook 路线：跨条目发现（2026-10-30）

写给同组调研 session 和发起 session。只有事实和判定依据，不是指令。都只看代码、文档和 issue，没有对平台发请求。
条目草稿：/private/tmp/claude-501/-Users-mac-WorkSpace-desktop-Project/7133807a-d653-486b-80df-d8e2af41491d/scratchpad/share_sph/entries.json

## 视频号：不装证书、不注入的现成路线是「腾讯元宝换 token」

- 两段请求，几个仓库写法一致：
  1. 带**腾讯元宝**的登录 cookie（`hy_user`、`hy_token`）POST `yuanbao.tencent.com/api/weixin/get_parse_result`，body `{"type":"video_channel_url","url":<分享链接>,"scene":1}`，返回 `playable_url`，其中带 `token`（generalToken）和 `eid`（exportId）。
  2. POST `channels.weixin.qq.com/finder-preview/api/feed/get_feed_info`，body `{"baseReq":{"generalToken":token},"exportId":eid}`，返回 feedInfo：description、h264/h265 videoUrl、picInfo、bgmInfo、coverUrl、createtime，authorInfo.nickname。
- 用这条路的仓库（gh api 已确认存在）：
  - ucmao/media-parser `src/parsers/wechat_channels_parser.py`（cookie 可选，`YUANBAO_COOKIE` 或别名 `WECHAT_CHANNELS_COOKIE`，都是元宝 cookie，见 `src/utils/cookie_registry.py` 第 49–64 行）。
  - ltaoo/wx_channels_download `internal/workers/sph/worker.js`：这就是 qiaomu-wx-video「在线解析」用的 `sph.litao.workers.dev/api/fetch_video_profile`，用部署者自己的元宝 cookie（`env.COOKIE`），请求头里写死了 t-userid、x-device-id、x-hy92、x-hy93。Go 客户端也有同样实现 `pkg/scraper/wxchannels/yuanbao.go`。
  - joeseesun/qiaomu-download `scripts/wechat/download_online.py`：先打 ltaoo 的 worker，再打作者自己部署的 `wx-dl.qiaomu.ai`。
  - z-mio/ParseHub `src/parsehub/provider_api/wxchannels.py`（cookie 必需）。
  - daxiaamu/DBdown `WechatResolver.kt`：先匿名请求，匿名结果里有视频地址就直接用，否则走元宝；元宝在 App 内用官方网页登录。另有 `WechatCipher.kt` 按 decodeKey 解密媒体。
  - joeseesun/qiaomu-clipper `src/utils/channels-study.ts`：浏览器扩展在用户自己已登录的元宝标签页里发同源请求，再在官方 finder-preview 播放页里用 `credentials:'omit'` 取 feed。登录态不出浏览器，这是这条路里风险最低的写法。
  - code search 还命中：zhiyu1998/rconsole-plugin、hanzhcn/laohan-skills、kangarooking/kangarooking-skills、Backtthefuture/video-transcript、x554960766/wechat-mp-tools、datawhalechina/video-devour、drdon1234/astrbot_plugin_media_parser、yanlingLabs/video-extract-mcp、Shaun520/ai-video-downloader、igeekfan/YT-GO 等（未逐个读）。
- 层级：带元宝 cookie 换 token 是 session-api（扩展在元宝页内发同源请求的写法是 browser-session）；匿名 `get_feed_info` 是 public-web；按 decodeKey 解密媒体是 private-api。
- 账号风险没有消失，只是从微信号换到元宝账号。元宝可用手机号或微信、QQ 登录（第三方报道，未核实网页版），用微信登录的元宝账号与微信号是否共用风控，未知。

## 视频号分享链接不登录能拿到什么（只到代码和文档层面）

- 匿名请求：`get_feed_info`，body `{"baseReq":{"generalToken":""},"shortUri":<sph/ 后面的短码>}`，只带 Origin、Referer，无签名。
- media-parser 文档两版说法不同：
  - 690118db（2026-08-30）：匿名能取标题、作者、封面。
  - 7d498406（2026-09-29）：匿名只返回作者昵称、头像和封面；视频地址、标题、图集都为空。
- media-parser 单测里匿名 mock 带 description 和 h264 videoUrl，那是 mock，不算证据。
- DBdown（2026-10）仍先匿名、有视频才直接用，说明部分作品匿名可能仍返回视频，或两者观察不一致。
- wechat-inbox-sync：`fetchWechatChannelsFeedInfo` 的匿名调用只被 `resolveWechatChannelsListenerUrl` 用来取 `sceneInfo.dynamicExportId`，后者只被 `openWechatChannelsListener` 调用，后者仍没有调用方。同组笔记的判定不变。
- 结论：能确认的是不登录可拿作者昵称、头像、封面；标题和描述在 2026-08 到 09 间收紧，现在是否返回未核实；视频地址要元宝 token（或个别作品匿名可得）。vision 笔记里「分享链接在浏览器里能否拿到标题和封面未核实」这条可按此关掉：封面和作者能拿到，标题不一定。

## wechat-inbox-sync 的手机端

- PR #104（2026-09-03）正文写明「route iOS Shortcut type=link WeChat Channels shares through webpage hydration」，测试 `tests/plugin-main-ai.test.js` 第 14991 行起用 `wechat-channels-shortcut-link-1` 记录。说明确有 iOS 快捷指令，产出 type=link 记录进 CloudBase 收件箱。
- 快捷指令本体不在仓库，README 只写小程序绑定码。快捷指令是不是直接调提交接口，仓库核实不了。

## 收藏页现成流程：未找到（抖音、小红书、B站、快手、微博、知乎）

查过：
- Maestro 树 2065 个文件：只有 Wikipedia、Web 示例，没有中文 App。
- MobiAgent MobiFlow：25 个规则文件，B站 9 类、小红书 6 类，都是搜索、播放、关注、评论，没有读收藏列表。
- AgentCPM-GUI 的 CAGUI 数据集（Hugging Face openbmb/CAGUI）：600 条指令，小红书 22 条、B站 43 条、抖音 0 条。只有 2 条是「把第 N 篇笔记收藏起来」，是写不是读。
- Open-AutoGLM：examples、prompts_zh 里没有收藏、复制链接。
- gh search 四组（uiautomator2/airtest/adb 加收藏）零结果；放宽到 README 后命中的都是养号、引流、刷视频类，按规则排除。
- WebSearch（Auto.js、AutoX、Hamibot）：只有发消息、评论脚本，最接近的是 Coande/co-wechat-life-export（2019 年，只支持微信 7.0.4）。

例外：
- 公众号：pywechat、pengyulong/wechat-favorites-exporter 已能从 PC 微信收藏逐条复制链接。
- 结构最接近的同类：duzui300/Obsiclip 的实验功能。无障碍服务只对 Kindle 包名生效，自动滚动笔记页，读无障碍树逐条取划线。

## 剪贴板这一环

- scrcpy `doc/control.md`：Android 剪贴板一变就自动同步到电脑。
- uiautomator2：读剪贴板要先切到它的 AdbKeyboard 输入法（README Clipboard 节）。`_input.py` 里广播读剪贴板的旧实现已注释掉。
- mirroir-mcp：38 个工具里没有读剪贴板的。`docs/limitations.md` 写明 Mac 剪贴板只经通用剪贴板（双端 Handoff、蓝牙、Wi-Fi、同一 Apple 账号，异步）到 iPhone。
- Apple 支持文档 120421：iPhone 镜像在两台设备之间复制，可以用拖放、通用剪贴板、AirDrop。
- Maestro：`copyTextFrom` 只把界面元素的文字存进 `maestro.copiedText` 变量，另有 `setClipboard`，树里没有读设备剪贴板的命令。

## 录音积木

- scrcpy `doc/audio.md`：
  - 默认 `output` 源映射 REMOTE_SUBMIX，转发整个音频输出，并关掉设备本身的播放；文档没说 App 能退出。
  - `playback` 源（Android 13+）App 可以退出。
- scrcpy issue 里搜 wechat 有 12 条，没有视频号静音的反馈；#5133 说的是 redroid 上的微信语音通话。
- 只录某一进程的声音：
  - insidegui/AudioCap 的 process tap 按 PID 录。
  - makeusabrew/audiotee 的 `--include-processes`、`--mute`，输出 PCM 到 stdout。
  - iPhone 镜像的声音是不是由「iPhone 镜像」这个进程发出，未核实；全系统 tap 可以兜底（meetily 就是 global tap）。
- Apple 120421 只说流媒体服务商可能限制播放，没提镜像受保护内容会不会静音。
- 已收录的 finnvoor/yap：`yap listen` 实时转写本机系统音频输出（同组 transform 结果），能否只录某个进程未核实。

## 分享入口的几种做法（都已确认存在）

- iOS：
  - 快捷指令 + Scriptable 读剪贴板：ceciliayu56-cell/xhs-to-obsidian-scriptable，匿名取小红书 SSR，媒体 URL 交给阿里云百炼。
  - 快捷指令用源码生成、`shortcuts sign` 签名，链接交给自建或作者的解析服务：Hype-yolo/backtap-douyin，服务端用匿名 Chromium 读公开页。
  - Safari 分享扩展用 JS 预处理读当前页 DOM：edequalsawesome/jiggyclipper。
  - 快捷指令打开 obsidian:// URI 触发插件导入：CH-YYK/obsidian-rednote。
- Android：
  - 分享到 App：Seal、DBdown、markdownr、Obsiclip。
  - 分享到「把分享文本塞进 HTTP 请求变量」：Waboodoo/HTTP-Shortcuts，可直发局域网里的本机服务。
- IM 机器人：wabarc/wayback（Telegram 等）。默认打开 IA 和 archive.today，每个网址都提交公共归档；本地存档要设 `WAYBACK_STORAGE_DIR` 才开。
