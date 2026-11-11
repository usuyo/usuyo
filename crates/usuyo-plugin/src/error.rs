use std::path::PathBuf;

use crate::protocol::{CallFailure, PluginId};

/// 这个 crate 的错误，一个枚举。插件报的失败和宿主记的失败都在 `Call` 里，按 `failure.code` 分支。
#[derive(Debug, thiserror::Error)]
pub enum PluginError {
    /// 清单不合规：`field` 是出错的字段路径，例如 `capabilities.fetch.outputs[1]`。
    #[error("invalid plugin manifest at `{field}`: {message}")]
    Manifest { field: String, message: String },

    #[error("{}: {source}", path.display())]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    /// 调用方要的这件事插件没声明：没有这个能力、`--want` 超出 `outputs`、输入种类不对。
    #[error("plugin {plugin} cannot take this call: {message}")]
    Unsupported { plugin: PluginId, message: String },

    /// 一次调用失败了。`stderr` 是最后几行日志，给诊断用。
    #[error("plugin {plugin} call {call_id} failed: {failure}")]
    Call {
        plugin: PluginId,
        call_id: String,
        failure: Box<CallFailure>,
        stderr: Vec<String>,
    },
}
