//! usuyo 的插件协议和获取插件的宿主。
//!
//! - [`protocol`]：协议 v1 的类型（清单、信封、错误码、各子命令的输出、JSON Schema），
//!   插件作者关掉默认 feature 就只依赖这一块。
//! - 宿主（`host` feature，默认开）：发现装好的插件、按 `match` 挑、调用
//!   `canonicalize` `fetch` `sources` `list`，管超时、环境、输出目录的校验；
//!   转换插件的常驻进程（[`ResidentTransform`]）。

mod error;
pub mod protocol;

#[cfg(feature = "host")]
mod host;

pub use error::PluginError;

#[cfg(feature = "host")]
pub use host::{
    CallLimits, CallOptions, DeclaredSite, DeclaredVisitor, Discovery, FetchInput, FetchPlugin,
    FetchRequest, FetchedSnapshot, HostConfig, InstalledPlugin, ListRequest, LoginState, OutFile,
    PluginHost, ProxySetting, Rejected, ResidentTransform, TransformConfig, TransformLimits,
    TransformOutput, TransformPlugin, TransformRequest, Vault, behind_fake_ip_proxy, discover,
    resolve_proxy,
};
#[cfg(feature = "host")]
pub use tokio_util::sync::CancellationToken;
