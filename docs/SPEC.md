<a id="歇存-xiecun--技术规格-v01m0"></a>
# 歇存 Xiecun — 技术规格 v0.1（M0）

状态：草案 · 2026-10-21。契约字段一旦 M0 落地即视为稳定，变更需 ADR。

<a id="1-仓库布局monorepo"></a>
## 1. 仓库布局（monorepo）

```
xiecun/
  cmd/xiecun/            # CLI 入口（也负责启动 daemon 和 mcp 子命令）
  internal/
    core/                # 内容包读写、生命周期状态机、去重
    index/               # bleve 索引（可重建）
    jobs/                # 任务队列（fetch / normalize / ocr / asr / embed）
    adapter/             # 适配器协议 + 运行器（子进程）
    mcp/                 # MCP 服务
    api/                 # 本地 HTTP/JSON-RPC（给桌面壳、扩展、手机桥）
    normalize/           # 通用正文抽取（go-readability 等）
  pkg/contract/          # 对外契约类型（manifest、适配器输入输出），MIT
  adapters/              # 仅放参考适配器与 SDK 示例，不放平台抓取逻辑；平台适配器在社区仓库
  apps/desktop/          # MyGo 桌面壳；apps/desktop/web 为 TS 前端（M3 才启用）
  docs/
  Makefile
```

<a id="2-资料库目录真相源"></a>
## 2. 资料库目录（真相源）

```
<library>/
  items/YYYY/MM/<platform>-<id8>-<rand4>/
    manifest.json
    content.md
    raw/<snapshotId>/...
    images/
    images/keyframes/
    media/
    derived/
      ocr/<imageName>.json
      asr.vtt
      vision.md
      summary.md
      attention.json
      embeddings.bin
    notes.md
  threads/<slug>.md        # 线索（画布分组/主题），frontmatter 列出条目 id
  thoughts/<slug>.md       # 念头，frontmatter 列出出处条目 id
  canvas.json              # 仅位置与视图状态
  .xiecun/
    index/                 # bleve，可删除重建
    jobs.db                # 任务队列（sqlite，纯 Go 驱动 modernc）
    journal/YYYY-MM.jsonl  # 只追加操作日志（撤销与审计）
    config.json
  .trash/                  # 删除先进这里，30 天后清除
```

目录名：`<platform>` 为适配器声明的平台 id（`douyin`、`xiaohongshu`、`bilibili`、`wechat-mp`、`x`、`web`、`file`）；`<id8>` 为平台内容 ID 的前 8 位（无平台 ID 时为内容哈希前 8 位）；`<rand4>` 防撞。目录名创建后不变，标题改了也不改。

<a id="3-manifestjson"></a>
## 3. manifest.json

```json
{
  "schema": 1,
  "id": "douyin-7f3a2c1e-k9x2",
  "identities": [
    {"platform": "douyin", "contentId": "7f3a2c1e9d...", "url": "https://..."}
  ],
  "kind": "video",                       // article | post | image | video | file
  "title": "...",
  "author": {"name": "...", "platformId": "...", "url": "..."},
  "publishedAt": "2026-10-01T12:00:00Z",
  "capturedAt": "2026-10-20T03:00:00Z",
  "origin": {                            // 来自哪条入口/行为
    "entry": "paste",                    // paste | extension | mobile | import | sync | mcp | cli | api
    "action": "favorite",                // 平台上的行为：favorite | like | share | view | unknown
    "importBatch": null
  },
  "completeness": "partial",             // link | partial | full（只升不降）
  "missing": ["media:video"],            // 缺什么，原因写 reason
  "missingReason": "平台未提供原文件",
  "snapshots": [
    {"id": "s1", "adapter": "community/douyin-api", "adapterVersion": "5.1.4",
     "at": "...", "completeness": "partial", "rawDir": "raw/s1"}
  ],
  "tags": {
    "platform": ["手冲", "咖啡日常"],
    "engine": [{"tag": "咖啡", "by": "qwen3-4b", "at": "...", "confidence": 0.82}],
    "user": ["周末要试"]
  },
  "fingerprints": {"text": "simhash:...", "images": ["phash:..."], "video": "dur:183|phash:..."},
  "sameAs": [],                          // 同源组：其他条目 id
  "state": "normal",                     // normal | slimmed | trashed
  "jobs": {"normalize": "done", "ocr": "pending", "asr": "failed", "embed": "pending"}
}
```

<a id="4-contentmd"></a>
## 4. content.md

frontmatter 复制 manifest 里的标题/作者/平台/时间（便于 grep 与 Obsidian），正文为平台原文的规范化文本。机器永不改写正文；需要修正时产生新快照。

```
---
title: ...
platform: douyin
author: ...
published: 2026-10-01
url: https://...
---
<原文>
```

<a id="5-derived-文件约定"></a>
## 5. derived/ 文件约定

- 每个文件头部或同名 `.meta.json` 记录 `{by, version, at, input}`。
- `ocr/<image>.json`：`[{text, bbox:[x,y,w,h], confidence}]`
- `asr.vtt`：WebVTT，分段带时间戳
- `vision.md`：关键帧画面描述，每段注明帧时间
- `attention.json`：`{opens, lastOpenedAt, referencedBy: [thoughtSlug...], state: "unseen|noticed|digested|released"}`

<a id="6-标签规则"></a>
## 6. 标签规则

- `platform`：原样保留，不编辑。
- `engine`：模型/规则建议，带来源与置信度；只显示给用户确认用。
- `user`：用户确认或手打；Agent 通过 propose/apply 进入这里。
- 搜索三种都索引；画布默认只显示 `user`。

<a id="7-去重"></a>
## 7. 去重

1. 身份层：适配器实现 `canonicalize(url) -> identity`（展开短链、去追踪参数、提取平台 ID）。同身份不新建条目，追加快照；新快照更完整则升级 `completeness`。无平台 ID 的条目不参与身份层。
2. 内容层：文本 SimHash（海明距离 ≤ 3）、图片 pHash、视频时长 + 关键帧 pHash。命中写入 `sameAs`，不自动合并。
3. 用户裁决：合并 / 保持 / 忽略。

并发：条目创建对 identity 加唯一约束（index 层）+ 任务队列按 identity 去重。

<a id="8-适配器协议-v0"></a>
## 8. 适配器协议 v0

v0 由 [思鹭插件协议 v1（草案）](plugin-protocol.md) 取代，字段、调用约定、错误码、凭据、权限和分发都以那份文档为准。v0 没有发布过，不做兼容。概要：

- 插件分获取、转换、导出三类，都是独立进程，共用一份清单 `seelu-plugin.yaml`（取代 `adapter.yaml`）。旧字段的去向见 [plugin-protocol §2.12](plugin-protocol.md#212-旧字段对照)。
- 获取插件用子命令 `canonicalize`、`fetch`、`sources`、`list`、`health`、`unfavorite`，stdout 只输出一个 JSON 信封；支持可选的常驻模式（[plugin-protocol §3](plugin-protocol.md#3-获取插件协议)）。fetch 输出和 `pkg/contract/schema/fetch-output.schema.json` 的差异见 [plugin-protocol §3.7](plugin-protocol.md#37-fetch-输出)。
- 转换、导出插件走 stdio 上换行分隔的 JSON-RPC 2.0（[plugin-protocol §4](plugin-protocol.md#4-转换与导出插件协议)）。
- 运行器：每个平台按用户排的顺序依次试，失败换下一个，全部失败停在 `link`（[plugin-protocol §3.1](plugin-protocol.md#31-运行器与节奏)）。
- 契约测试：插件仓库放 `testdata/`，作者用 `seelu plugin check` 做离线检查，用 `seelu plugin try` 在真实的沙箱和凭据下跑一次，不用 plugin-kit 发版工作流的用 `seelu plugin pack` 生成发布清单（[plugin-protocol §10](plugin-protocol.md#10-契约测试与-plugin-kit)）。
- `pkg/contract` 的 Go 类型、JSON Schema 和 `testdata/` 仍是 v0；实现插件运行器时，插件相关的 schema 迁到 plugin-kit，`pkg/contract` 只留资料库自己的类型（[plugin-protocol §10](plugin-protocol.md#10-契约测试与-plugin-kit)）。

<a id="9-clim0"></a>
## 9. CLI（M0）

```
xiecun init [dir]                 # 建库
xiecun add <url|file|->           # 收下；立即返回条目 id，后台获取
xiecun get <id> [--layer original|derived|notes|all]
xiecun search <query> [--src douyin] [--in asr] [--kind video] [--json]
xiecun tag <id> --add a,b --rm c
xiecun jobs [--retry <id>]
xiecun reindex
xiecun daemon                     # 常驻：任务队列 + 本地 API + 文件监听
xiecun mcp                        # stdio MCP 服务（转发到 daemon，daemon 不在则内嵌）
```

<a id="10-mcp-工具m0"></a>
## 10. MCP 工具（M0）

- `search(query, filters) -> [{id, title, hitLayer, snippet, untrusted:true, source}]`
- `get_item(id, layers=[original|derived|notes])`
- `propose_changes(changes:[{itemId, op:"add_tag"|"rm_tag"|"set_thread", value}]) -> changesetId`
- `apply_changeset(changesetId)`（需用户确认策略：M0 为 CLI 交互确认）
- `undo_changeset(changesetId)`
- 不提供：delete、export、open_url。
- 标记 `private: true` 的条目和 `notes.md` 默认不返回。

按 MCP 2026-07-28 规范实现为无状态；changesetId 为显式句柄。

<a id="11-索引"></a>
## 11. 索引

bleve（纯 Go）。中文用 CJK 双字分析器（解决二字词），同时索引拼音字段（go-pinyin）。字段：title、body、ocr、asr、vision、summary、notes、tags.*、platform、kind、author、publishedAt。向量检索 M1 再加（候选：bleve vector 或 sqlite-vec）。

<a id="12-m0-范围"></a>
## 12. M0 范围

做：第 2–11 节全部，但适配器只内置一个 `web-generic`（通用网页：抓取 + go-readability 抽正文 + 图片下载），用来验证整条链路；OCR/ASR 留接口不实现。
验收：
1. `xiecun add https://mp.weixin.qq.com/s/xxx` 3 秒内返回 id；`items/` 下出现内容包，`raw/` 有原 HTML，`content.md` 有正文。
2. `xiecun search 咖啡` 命中；`--in title` 过滤生效；二字词可搜。
3. 在 Claude Code 配置 `xiecun mcp` 后，能 `search` 并 `get_item`，返回带 `untrusted: true`。
4. `propose_changes` → CLI 确认 → `apply` → `manifest.json` 的 `tags.user` 更新 → `undo` 回滚，journal 有记录。
5. 删除 `.xiecun/index/` 后 `xiecun reindex` 恢复搜索。

不做：GUI、扩展、手机桥、平台适配器、OCR/ASR、同步。
