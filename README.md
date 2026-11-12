# usuyo

在推荐流外，维护一套你自己的思路。

- 总纲（做到哪、要做的事）：[docs/BUILD.md](docs/BUILD.md)
- 产品定义：[docs/PRD.md](docs/PRD.md)
- 技术规格：[docs/spec/](docs/spec/README.md)
- 插件协议：[docs/protocol/](docs/protocol/README.md)
- 各平台的获取方案：[docs/platforms/](docs/platforms/)
- 用户逐条的回答：[docs/answers.md](docs/answers.md)
- 功能文档：[docs/features/](docs/features/)
- 架构决策：[docs/decisions/](docs/decisions/)
- 开发约定：[CLAUDE.md](CLAUDE.md)

```
just init     # 装前端依赖
just check    # lint + test
just build    # 产出 target/release/usuyo
```

## 许可证

内核以 [AGPL-3.0](LICENSE) 发布；对外契约 [crates/usuyo-contract](crates/usuyo-contract/) 以 [MIT](crates/usuyo-contract/LICENSE) 发布，供适配器与 SDK 自由使用。
