//! 写 usuyo 获取插件用的依赖。
//!
//! - [`protocol`]：插件协议 v1 的类型，原样转自 `usuyo-plugin`。
//! - [`http`]：发请求的客户端。链接和平台回答里的地址都不可信，插件的每个请求都走它。

pub mod http;

pub use usuyo_plugin::protocol;
