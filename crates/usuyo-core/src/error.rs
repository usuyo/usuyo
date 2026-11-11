//! 本 crate 唯一的错误类型；`code()` 给边界映射成 SPEC §0.3 的错误码。

use std::path::PathBuf;

use usuyo_plugin::PluginError;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Library(#[from] usuyo_library::Error),
    #[error(transparent)]
    Index(#[from] usuyo_index::Error),
    #[error(transparent)]
    Plugin(#[from] PluginError),
    #[error("json: {0}")]
    Json(#[from] serde_json::Error),
    #[error("jobs database: {0}")]
    Jobs(#[from] rusqlite::Error),
    #[error("io error at {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("invalid argument: {0}")]
    InvalidArgs(String),
    #[error("not found: {0}")]
    NotFound(String),
    #[error("unsupported input: {0}")]
    UnsupportedInput(String),
    /// 后台线程没跑完（panic 或被取消）。
    #[error("background task failed: {0}")]
    Task(String),
}

impl Error {
    /// SPEC §0.3 的错误码。
    pub fn code(&self) -> &'static str {
        match self {
            Error::Library(e) => e.code(),
            Error::Index(e) => match e {
                usuyo_index::Error::EmptyQuery | usuyo_index::Error::QueryTooLong { .. } => {
                    "invalid-args"
                }
                usuyo_index::Error::BadCursor => "cursor-invalid",
                usuyo_index::Error::Busy => "library-busy",
                _ => "internal",
            },
            Error::Plugin(PluginError::Call { failure, .. }) => failure.code.as_str(),
            Error::Plugin(PluginError::Unsupported { .. }) => "unsupported-input",
            Error::Plugin(_) => "internal",
            Error::Jobs(_) | Error::Json(_) | Error::Io { .. } | Error::Task(_) => "internal",
            Error::InvalidArgs(_) => "invalid-args",
            Error::NotFound(_) => "not-found",
            Error::UnsupportedInput(_) => "unsupported-input",
        }
    }

    /// 插件报的错：哪个插件。边界据此回 502 并带上 `plugin`。
    pub fn plugin(&self) -> Option<&str> {
        match self {
            Error::Plugin(PluginError::Call { plugin, .. }) => Some(plugin.as_str()),
            _ => None,
        }
    }

    /// 给人看的一句话：插件给了 `hint` 就用它。
    pub fn hint(&self) -> Option<&str> {
        match self {
            Error::Plugin(PluginError::Call { failure, .. }) => failure.hint.as_deref(),
            _ => None,
        }
    }

    pub(crate) fn io(path: impl Into<PathBuf>, source: std::io::Error) -> Error {
        Error::Io {
            path: path.into(),
            source,
        }
    }
}

pub type Result<T, E = Error> = std::result::Result<T, E>;
