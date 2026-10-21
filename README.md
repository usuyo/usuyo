# 歇存 Xiecun

开源的注意力回收工具。把散落在各平台收藏夹里的内容，变成你自己的、可搜索、可交给 Agent 处理的文本资料库。

- 产品定义：[docs/PRD.md](docs/PRD.md)
- 技术规格：[docs/SPEC.md](docs/SPEC.md)
- 功能文档：[docs/features/](docs/features/)
- 架构决策：[docs/decisions/](docs/decisions/)
- 开发约定：[CLAUDE.md](CLAUDE.md)

```
make init     # 安装依赖
make check    # lint + test
make build    # 产出 bin/xiecun
```

## 许可证

内核以 [AGPL-3.0](LICENSE) 发布；对外契约 [pkg/contract](pkg/contract/) 以 [MIT](pkg/contract/LICENSE) 发布，供适配器与 SDK 自由使用。
