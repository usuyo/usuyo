# usuyo-contract

usuyo 资料库磁盘上各记录的 Rust 类型和 JSON Schema（draft 2020-12）：`manifest.json` v1、改动记录的一行、改动提议，主题、卡片、成稿、`notes.md` 的 front matter 对象，转录记录 `run.json` 和转换产物 `keyframes.json`、`ocr.json`；另有 op 词表、回收站的 `entry.json`、`library.json`、`usuyo.fetch.json` 的类型。

本目录以 [MIT](LICENSE) 单独授权，与仓库其余部分的 AGPL-3.0 不同；插件和别的工具可直接依赖或复制其中的 schema。

- 规范来源：[SPEC §1.9](../../docs/spec/library.md#19-schema-和-pkgcontract)，各记录的字段见 SPEC 第 1 层。
- Schema：`schema/*.schema.json`，`$id` 为 `urn:usuyo:schema:<名字>:1`。`adapter.*`、`fetch-output.*` 还是 v0，归插件协议。
- 固定样例：`testdata/`；`testdata/invalid/` 是应当校验失败的样例。
- 读到不认识的字段不报错，写回时原样留着（每个类型的 `extra`）。
- 记录里的相对路径（`usuyo.fetch.json`、`run.json`、`keyframes.json`、`entry.json`）读的一方先用 `is_plain_relative` 或各类型的 `bad_path` 查过再拼。
