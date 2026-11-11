# 微信生态调研：跨条目发现（2026-10-30）

完整结果（39 条 + 42 条 lead）已通过 StructuredOutput 交回工作流，副本在
/private/tmp/claude-501/-Users-mac-WorkSpace-desktop-Project/7133807a-d653-486b-80df-d8e2af41491d/scratchpad/wx/result.json 。
这里只记其他方向也会用到的事实。

## 腾讯对「解密本地数据库」类仓库的 DMCA

- 来源：github/dmca 仓库，用 `gh api repos/github/dmca/contents/<年>/<月>` 列出文件名含 wechat、tencent 的通知。
- 2026-01-08-tencent.md：以 §1201 为由，列 29 个仓库，按 4,195 个仓库的派生网络整体处理（wechat-dump-rs、SharpWxDump、wechatDataBackup、wechat-backup、wechat-dump、WechatMoments、TC999/WeChatMsg 等）。
- 2026-06-29 至 2026-10-13：36 份通知，每份点名一个仓库，其中 8 份按派生网络处理。wx-cli（1,464 个仓库）、wx_key、WeFlow、ylytdeng/wechat-decrypt（2,352 个仓库）、chatlog_alpha 等都在内。
- 2025-10-20：PyWxDump、chatlog 收到律师函或函件后自删代码。
- 视频号代理下载类、UI 自动化类、iOS 备份解析类（BlueMatthew/WechatExporter）至今没被点名。
- WeFlow 被点名后改为「不内置解密组件，由用户在设置里指定外部动态库」，仓库仍可访问。

## 公众号接口关闭

- 2026-07-30 公众号后台「搜索其他公众号文章」接口关闭（wechat-article-exporter #200）。之后的批量路径都转向微信读书：we-mp-rss 的 weread_mp 模式用网页 cookie，wallechfox/wechat-mp-archive 模拟微信读书 App 登录。
- we-mp-rss v1.5.3 的 RSS template 参数被当作 eval 执行，已被利用来装挖矿程序（#479、#480），看不到修复提交。

## 微信读书官方 Gateway

- Tencent/WeChatReading（Apache-2.0）：2026-05-17 起可用个人 API Key（wrk-）调 `POST https://i.weread.qq.com/api/agent/gateway`，属 official-export。
- obsidian-weread-plugin、weread2notion 已改用这个 Gateway。书架里的「文章收藏」只给入口，不给文章内容。

## 微信收藏

- 官方没有导出功能。不解密的开源实现只找到 pengyulong/wechat-favorites-exporter：用 macOS 辅助功能在收藏页右键「复制链接」，输出 CSV（收藏时间、标题、链接），MIT。
- 视频号的「我的收藏、赞」列表：ltaoo/wx_channels_download（MCP）和 nobiyou/wx_channel 支持，二者都属 client-hook。

## 层级口径

- Camoufox（jackwener/wechat-article-to-markdown）和 curl_cffi 模拟 TLS 指纹（tmwgsicp/wechat-download-api）都按「模拟设备指纹」记 private-api，与 CloakBrowser 的判法一致。
- Bwkyd/url-md 用的是普通 reqwest 加 UA，记 public-web。
