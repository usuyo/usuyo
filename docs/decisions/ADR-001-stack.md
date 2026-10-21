# ADR-001 技术栈：Go 内核 + MyGo 壳 + TS 前端

日期：2026-10-21 · 状态：accepted

## 决定
- 内核、CLI、MCP、daemon 用 Go（单二进制、纯 Go 依赖优先、无 cgo）。
- 桌面壳用 MyGo，web 前端 TypeScript + Vite + pnpm。
- 重计算（ffmpeg、OCR、ASR）与平台适配器均为独立进程，不进内核。
- 索引用 bleve（纯 Go，可重建）；任务队列用 modernc sqlite。

## 理由
- 单人 + Coding Agent 开发，希望单二进制、启动快、内存小；MyGo 约 7MB/44MB。
- 文件为真相源，内核主要是 IO 与编排，Go 足够。
- 适配器外置后，内核语言与适配器生态（多为 JS/Python）无关。

## 放弃的方案
- Rust + Tauri：团队产品的选择，单人速度不占优。
- Electron + TS 内核：生态最顺但包体与内存大；若 MyGo 成熟度成问题，这是回退方案（UI 只通过本地 API 与内核通信，壳可换）。

## 影响
- MyGo 仍在 0.5.x，M3 之前不绑定前端；若届时不稳，回退 Electron 不影响内核。
