//! 插件协议 v1 的类型（`docs/protocol/`）。MIT，插件作者可以单独依赖：
//! 关掉默认的 `host` feature 就只剩这里。

mod credentials;
mod envelope;
mod fetch;
mod identity;
mod manifest;
mod schema;
mod transform;
mod values;

pub use credentials::{
    Cookie, CredentialSite, Credentials, OriginState, SameSite, StorageEntry, StorageState,
};
pub use envelope::{
    CallFailure, Envelope, ErrorCode, MAX_PAUSE, Need, NotFoundReason, Pause, PluginEvent,
    WaitReason,
};
pub use fetch::{
    Author, Canonicalized, Completeness, Evidence, FetchHealth, FetchOutput, ItemKind, ListAction,
    ListItem, ListPage, LoginReport, LoginReportState, Media, MediaKind, MethodCheck, OutputKind,
    SourceInfo, SourcesOutput, TextFormat, UpstreamVersion,
};
pub use identity::Identity;
pub use manifest::{
    Access, Account, Asset, Capabilities, CapabilityUse, Consume, ConsumeKind, Credential,
    Delivers, Device, ExportSpec, ExportWrites, FetchCapability, FetchInputKind, FetchSpec,
    FsPermissions, Inject, Install, Interaction, Limits, ListCapability, ListOrder, Login,
    MANIFEST_FILE, MatchPattern, Method, ModelFormat, ModelSpec, NetworkMode, NetworkPermissions,
    Pacing, Permissions, PlatformUse, PluginHeader, PluginKind, PluginKindTag, PluginManifest,
    Produce, ProviderNeed, Risk, RiskSource, Run, RuntimeRequirement, SUPPORTED_PROTOCOLS, Setting,
    SettingType, Site, TransformSpec, Upstream, Visitor,
};
pub use schema::schemas;
pub use transform::{
    Cancel, Describe, EmptyReason, Health, HealthCheck, InputRole, METHOD_CANCEL, METHOD_DESCRIBE,
    METHOD_HEALTH, METHOD_PROGRESS, METHOD_RUN, Mark, MarkCode, ModelUsed, Probe, ProbeStream,
    Progress, ProviderGrant, RpcError, RpcId, RunInput, RunItem, RunParams, RunProduct, RunResult,
};
pub use values::{
    ByteSize, DurationSpec, MediaGroup, PlatformId, PluginId, RelPath, Rfc3339, Seconds,
};

#[cfg(feature = "host")]
pub(crate) use manifest::PLUGIN_DIR_PLACEHOLDER;
#[cfg(feature = "host")]
pub(crate) use values::check_http_url;

/// 读出来的值，加上不认识的字段（宿主忽略并记日志，`usuyo plugin check` 当失败，§0.1）。
#[derive(Debug, Clone)]
pub struct Parsed<T> {
    pub value: T,
    pub unknown_fields: Vec<String>,
}

/// 按类型读一个 JSON 值：错了报出字段路径，不认识的字段记进 `unknown`。
pub(crate) fn deserialize_tracked<T: serde::de::DeserializeOwned>(
    value: serde_json::Value,
    unknown: &mut Vec<String>,
) -> Result<T, (String, String)> {
    let mut track = |path: serde_ignored::Path<'_>| unknown.push(path.to_string());
    let de = serde_ignored::Deserializer::new(value, &mut track);
    serde_path_to_error::deserialize(de).map_err(|e| {
        let field = e.path().to_string();
        (field, e.into_inner().to_string())
    })
}
