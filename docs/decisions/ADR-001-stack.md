# ADR-001 技术栈：Go 内核 + MyGo 壳 + TS 前端

日期：2026-10-21 · 状态：accepted

2026-11-04：重新讨论后确认本 ADR：Go 内核加 MyGo 壳。只做桌面三系统，手机以后原生重写，不要求共用内核（「其实就桌面端吧，手机端其实可以考虑native去重写，不用考虑手机上的跨平台，所以go感觉也是够的」；[open-decisions §1.8](../open-decisions.md#18-grill-回答)，对比见 [调研 12](../research/12-shell-and-core-language.md#0-结论)）。

2026-11-04：下文的版本说法停在 2026-10-21，已过时：MyGo 最新是 0.7.4（2026-11-01 发布，[releases](https://github.com/egoist/mygo/releases)），不是 0.5.x，要求 Go 1.27.3+（[调研 12 §4.1](../research/12-shell-and-core-language.md#41-选项-a)）；Wails v3 在 2026-10-26 发了正式版 v3.0.0（[发布说明](https://github.com/wailsapp/wails/releases/tag/v3.0.0)）。

<a id="壳的退出条件2026-11-04"></a>
## 壳的退出条件（2026-11-04）

MyGo 仓库 2026-09-26 才建，几乎只有作者一人提交，10-28 还有破坏性变更（[调研 12](../research/12-shell-and-core-language.md#25-壳的成熟度)）。为此守两条约束，定四个换壳信号；信号是建议值，可以改。

- 约束：前端只经本机 HTTP 和事件找内核（[ADR-002](ADR-002-frontend-stack.md#和内核的接口)）；壳相关代码只放在 `shell/` 一处，内核不 import 壳。
- 换壳信号，出现任一条就评估换到 Wails v3（Go 栈不变）：
  1. 60 天没有新版本，或阻塞问题两周没有回应；
  2. 一次升级要改壳代码超过一天，连续两次；
  3. Windows 或 Linux 上打包、webview 有阻塞问题，两周修不好；
  4. 许可证变更。

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
