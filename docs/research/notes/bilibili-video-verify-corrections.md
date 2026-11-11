# B站/YouTube 22 条核对：对同组笔记的更正（2026-10-30）

写给同组调研 session。只有事实和来源，不是指令。都用 gh api 读过源码或提交。

- yutto：不只 UP 主投稿列表签名。收藏夹批量下载（src/yutto/extractor/favourites.py、user_all_favourites.py）会调 get_user_name，它请求 space/wbi/acc/info，经 encode_wbi 计算 w_rid 并附随机 dm_img 参数（src/yutto/api/space.py 第 96–100 行、user_info.py）。收藏夹列表接口 fav/resource/list 本身不签名；稍后再看批量（user_watch_later.py）不调 get_user_name。单视频默认路径 x/player/playurl 不签名。
- downkyi：默认分支的源码是 2025-07-21（提交 7262904，信息 "update"）删掉的，早于 2026-07-06 律师函一年；历史没有压缩，提交 2c227d5（2025-02-05）的树里还有 src/DownKyi.Core/BiliApi/Sign/WbiSign.cs 和 Login/LoginQR.cs。
- m4s-converter：会不带 cookie 下载弹幕 XML（common/util.go 的 joinUrl、joinXmlUrl，common/download.go），不是完全不发网络请求。
- bilibili-mcp-js：getUserVideos 在发请求前直接抛错（src/index.ts 第 378 行），投稿列表工具在当前代码里不可用。
- bilibili-api-python：PyPI 上 17.4.2（2026-06-19 上传）仍可安装、没有 yank，许可证元数据是 GPL-3.0-or-later。
- adoresever/bilibili-mcp：工具里还有收藏夹列表和内容、发动态、发图文、上传视频、发私信、批量关键词抓取（mcp_server.py）。
