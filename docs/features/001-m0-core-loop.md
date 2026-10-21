# 001 — M0 最小闭环：add → 内容包 → search → MCP

状态：approved
关联：PRD §8 M0 · SPEC §2–12

## 目标
不带 GUI 跑通整条链路：粘贴一条链接变成内容包，CLI 能搜到，Claude Code 通过 MCP 能读能提议标签。从这一天起作者自己开始使用。

## 用户故事
- 作为作者，我想 `xiecun add <url>` 后立刻拿到 id 并继续手头的事，以便不被"先整理"打断。
- 作为作者，我想在 Claude Code 里直接搜我的资料库，以便让 Agent 基于我存的材料写东西。

## 范围
### 做
- 资料库初始化与目录结构（SPEC §2）
- manifest / content.md 读写（SPEC §3–4）
- 适配器协议 v0 + 运行器 + 内置 `web-generic` 适配器（SPEC §8）
- 任务队列（fetch、normalize）与 daemon
- bleve 索引与 `search`（含 CJK 双字、拼音字段、--in/--src/--kind 过滤）
- 身份层去重（SPEC §7.1）
- MCP：search / get_item / propose_changes / apply_changeset / undo_changeset；journal
- `reindex`
### 不做
- OCR / ASR / 关键帧 / 向量
- 任何平台适配器
- 内容层去重（SimHash/pHash）留接口
- GUI、扩展、手机桥、同步

## 数据契约变更
首次定义，见 SPEC §3–5。

## 接口变更
首次定义，见 SPEC §9–10。

## 实现要点
- Go 1.23，`cmd/xiecun` 用 cobra；配置 `~/.config/xiecun/config.json` 指向当前库。
- 通用正文抽取：`go-shiori/go-readability`；HTML 快照原样存 `raw/s1/page.html`。
- 索引：`blevesearch/bleve/v2`，自定义 analyzer：CJK bigram + 拼音（`mozillazg/go-pinyin`）。
- 队列：`modernc.org/sqlite`（纯 Go），表 `jobs(id, itemId, type, state, attempts, nextAt, error)`。
- MCP：`modelcontextprotocol/go-sdk`（若其尚未支持 2026-07-28 无状态语义，先按其当前稳定版实现，changesetId 仍作显式参数）。
- journal：`.xiecun/journal/YYYY-MM.jsonl`，每行 `{at, actor, op, itemId, before, after, changesetId}`。
- 所有返回给 MCP 的正文包在 `{untrusted: true, source: {...}, content}` 信封里。

## 验收标准
见 SPEC §12 的 5 条，逐条可执行。

## 测试方案
- `internal/core`：manifest 读写往返、目录名生成、身份层去重并发测试。
- `internal/adapter`：用 `testdata/wechat-mp-sample.html` 跑 web-generic，校验 fetch 输出符合契约。
- `internal/index`：二字词、拼音、过滤器用例。
- `internal/mcp`：propose → apply → undo 往返，journal 行数与内容断言。
- 手动：对一篇真实公众号文章跑验收 1–5。

## 风险与待定
- bleve 的 CJK 分析器对混排中英文的效果需实测；不理想则换 gse 分词。
- go-sdk 对最新 MCP 规范的支持进度。
- MyGo 仅在 M3 引入；M0 不碰前端。
