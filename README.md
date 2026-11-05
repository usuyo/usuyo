# usuyo

在推荐流外，维护一套你自己的思路。

- 整体设计（先读）：[docs/overview.md](docs/overview.md)
- 产品定义：[docs/PRD.md](docs/PRD.md)
- 待讨论：[docs/discussion.md](docs/discussion.md)
- 要做的事：[docs/TODO.md](docs/TODO.md)
- 技术规格：[docs/SPEC.md](docs/SPEC.md)
- 功能文档：[docs/features/](docs/features/)
- 架构决策：[docs/decisions/](docs/decisions/)
- 开发约定：[CLAUDE.md](CLAUDE.md)

```
make init     # 安装依赖
make check    # lint + test
make build    # 产出 bin/usuyo
```

## 许可证

内核以 [AGPL-3.0](LICENSE) 发布；对外契约 [pkg/contract](pkg/contract/) 以 [MIT](pkg/contract/LICENSE) 发布，供适配器与 SDK 自由使用。
