//! usuyo 的引擎：打开着的一个资料库、它的索引、本机任务队列和插件宿主，串成「收下 → 取内容 → 存好 → 搜到」。
//!
//! 接口（HTTP、命令行）只经 [`Engine`]；返回的视图形状照前端的 `endpoints.ts`。

mod content;
mod engine;
mod error;
mod fetch;
mod index_doc;
mod jobs;
mod layout;
pub mod view;

pub use engine::{Engine, EngineConfig, Links, RetryPolicy, Via};
pub use error::{Error, Result};
pub use layout::{HashLayout, Layout, LayoutItem, Placed};
pub use usuyo_plugin::CallLimits;
