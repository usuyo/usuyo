//! 本 crate 唯一的错误类型；`code()` 给边界映射成 SPEC §0.3 的错误码。

use std::path::PathBuf;

use usuyo_plugin::PluginError;

use crate::words::Said;

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
    #[error(transparent)]
    Model(#[from] crate::models::ModelError),
    #[error(transparent)]
    Phone(#[from] crate::phone::PhoneError),
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
    /// 正在做的事挡着（这个转换正在跑、这个模型正在下）。
    #[error("conflict: {0}")]
    Conflict(String),
    /// 后台线程没跑完（panic 或被取消）。
    #[error("background task failed: {0}")]
    Task(String),
    /// 带一句给人看的话（两种语言，[`Said`]）：界面照 `hint` 显示，`source` 的英文进日志。
    #[error("{source}")]
    Hinted { source: Box<Error>, hint: Said },
}

impl Error {
    /// SPEC §0.3 的错误码。
    pub fn code(&self) -> &'static str {
        match self {
            Error::Hinted { source, .. } => source.code(),
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
            Error::Plugin(PluginError::Credentials { .. }) => "invalid-args",
            Error::Plugin(_) => "internal",
            Error::Jobs(_) | Error::Json(_) | Error::Io { .. } | Error::Task(_) => "internal",
            Error::Model(crate::models::ModelError::Busy) | Error::Conflict(_) => "conflict",
            Error::Model(_) => "internal",
            Error::Phone(e) => e.code(),
            Error::InvalidArgs(_) => "invalid-args",
            Error::NotFound(_) => "not-found",
            Error::UnsupportedInput(_) => "unsupported-input",
        }
    }

    /// 插件报的错：哪个插件。边界据此回 502 并带上 `plugin`。
    pub fn plugin(&self) -> Option<&str> {
        match self {
            Error::Hinted { source, .. } => source.plugin(),
            Error::Plugin(PluginError::Call { plugin, .. }) => Some(plugin.as_str()),
            _ => None,
        }
    }

    /// 给人看的一句话：插件给了 `hint` 就用它（只有一种语言，照原样）。
    pub fn hint(&self) -> Option<Said> {
        match self {
            Error::Hinted { hint, .. } => Some(hint.clone()),
            Error::Plugin(PluginError::Call { failure, .. }) => {
                failure.hint.as_deref().map(Said::from_stored)
            }
            Error::Phone(e) => Some(e.hint()),
            _ => None,
        }
    }

    /// 配上一句给人看的话（`words.rs` 里的一句）。
    pub fn hinted(self, hint: Said) -> Error {
        Error::Hinted {
            source: Box::new(self),
            hint,
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
