//! 写 usuyo 插件用的依赖。
//!
//! - [`protocol`]：插件协议 v1 的类型，原样转自 `usuyo-plugin`。
//! - [`fetch`]：获取插件的子命令和参数，作者照它分派到每种方法；`health` 的回答用 `protocol::FetchHealth`。
//! - [`http`]：获取插件发请求的客户端。链接和平台回答里的地址都不可信，插件的每个请求都走它。
//! - [`transform`]：转换插件的 JSON-RPC 循环，作者只写 `run` 处理函数。
//! - [`semver`]：`Describe` 里的版本号类型。

pub mod fetch;
pub mod http;
pub mod transform;

pub use semver;
pub use usuyo_plugin::protocol;
