//! usuyo 的引擎：打开着的一个资料库、它的索引、本机任务队列和插件宿主，串成「收下 → 取内容 → 存好 → 转成文字 → 搜到」。
//!
//! 接口（HTTP、命令行）只经 [`Engine`]；返回的视图形状照前端的 `endpoints.ts`。

mod content;
mod engine;
mod error;
mod facets;
mod fetch;
mod file_imports;
mod imports;
mod inbox;
mod index_doc;
mod jobs;
mod layout;
mod methods;
mod models;
mod owner_only;
mod pace;
mod phone;
mod registry;
mod reveal;
mod settings;
mod share_text;
mod transcripts;
mod transform;
mod trash;
mod vault;
pub mod view;
mod words;

pub use engine::{Engine, EngineConfig, Links, RetryPolicy, Via};
pub use error::{Error, Result};
pub use file_imports::{FILE_IMPORT_MAX, FileUpload, HANDOVER_MAX};
pub use imports::{BATCH_MAX, DeliveredLink, ImportChoice};
pub use inbox::InboxReport;
pub use jobs::{TransformJob, TransformJobKind, TransformState};
pub use layout::{HashLayout, Layout, LayoutItem, Placed};
pub use models::{ModelError, ModelState};
pub use owner_only::write_private;
pub use phone::{Asset, Downloads, DriverAction, PhoneConfig, PhoneError, Release};
pub use settings::{
    Language, NetworkMode, NetworkPatch, SettingsPatch, TransformsPatch, language, proxy_setting,
    remember_library, transforms_auto,
};
pub use share_text::{link_count, link_in};
pub use transform::{Power, PowerSource, SystemPower};
pub use trash::{ClockGuard, Expiry};
pub use usuyo_plugin::{CallLimits, ProxySetting, TransformLimits, Vault};
pub use vault::{FileVault, system_vault};
pub use words::{Lang, Said, Words};
