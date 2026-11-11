# 各获取层级的账号后果：GitHub issue 证据（2026-10-30）

写给同组调研 session。只有事实和判定依据，不是指令。完整结果（51 条 finding，含 summary_zh、needs_check）：
/private/tmp/claude-501/-Users-mac-WorkSpace-desktop-Project/7133807a-d653-486b-80df-d8e2af41491d/scratchpad/issue_risk/structured_output.json
逐条分类（读过的 183 条 issue，按 a 实现失效 / b IP 或频率 / c 验证码 / d 警告下线限制 / e 禁言封号 / f 下架关停 / g 提问）：同目录 classification.json。

## 范围

- 34 个仓库共 12585 条 issue 标题，任务关键词命中 197 条；读 183 条正文，确认 80 条是账号层后果，31 条是禁言或封号。样本按关键词挑选，不能当发生率。
- 读不到：nilaoda/BBDown、Nemo2011/bilibili-api、cluic/wxauto 的 issue 已关闭。
- 只抓了部分：jackwener/OpenCLI 最近 2500 条；yt-dlp 只取标题含 bilibili、douyin、xiaohongshu、weibo 的 287 条。

## 和「风险分层」假设冲突的三点

1. **public-web 在小红书上不是零账号风险。** XHS-Downloader #305：不填 cookie 批量下载约 108 条后，复制链接的账号被下线；作者说链接带跟踪参数，#309 说换 IP 或账号要重新取链接。「视觉层拿分享链接、再匿名取详情」的混合路径要考虑这一点。
2. **browser-session 在小红书 2026 年并不低风险。** xiaohongshu-mcp 读 15 条有 13 条是账号后果（9 条禁言或封号），一次登录、两次采集、一天低频只读都有人被警告或封（#720、#726、#728）；复用用户自己 Chrome 的 OpenCLI 也有只读被判违规（#842）；2026-07-27 换内置指纹浏览器后仍有报告（#877、#720 下 10-30 的评论）。另一面：MediaCrawler 维护者（#865）认为连接真实浏览器比自动化更难被识别。
3. **vision-ui 的后果是强制下线。** pywechat：UI 读消息、夜间、云服务器、虚拟机、RDP 下会被强制下线、要扫码重登；维护者说没见过因此封号。手机和 iPhone 镜像类通用 Agent 没有报告（只说明没有报告）。

## 各层一句话

- session-api：公众号后台是频率控制（ret=200013）和会话 2–4 天过期，微信读书是小黑屋和半天到一天失效，没有微信号处罚报告；B站 有一例 SESSDATA 下载私有收藏夹后大会员冻结（yutto #452，因果不明）。
- private-api：小红书最重；抖音多为 1–12 小时按功能的限制，维护者称一般不封号；B站 个别账号持续半年（bili-sync #756）；X 老主号低量也被封；Instagram 有备份自己 150 条帖子后永久封号（instaloader #2680）。
- client-hook：WeChatFerry 警告→封 7 天→警告永久封，按实名牵连大号，只读监听也会被处理（#126、#422）；本地代理类没有微信账号处罚报告，风险在共用根证书私钥（wx_channels_download #586）。

## 判读注意

- 「account blocked」「461」「账号异常」常是签名或代码失效（MediaCrawler #895、#757），不能直接当处罚计数。
- 任务关键词漏掉了真正报处罚的词：警告、禁言、下线、退出登录、300011/300017、checkpoint、please wait、feedback_required、locked。
- Dosu 机器人的总结数字没有采用；所有数字出自人类评论或正文。
