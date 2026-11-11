//! 本 crate 唯一的错误类型；`code()` 给边界映射成 SPEC §0.3 的错误码。

use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("io error at {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("bad json in {path}: {source}")]
    Json {
        path: PathBuf,
        #[source]
        source: serde_json::Error,
    },
    /// 文件不合约定的写法（front matter、划线行、JSON 形状），多半在 usuyo 外改坏了。
    #[error("malformed {path}: {reason}")]
    Malformed { path: PathBuf, reason: String },
    #[error("invalid argument: {0}")]
    InvalidArgs(String),
    #[error("not found: {0}")]
    NotFound(String),
    #[error("conflict: {0}")]
    Conflict(String),
    #[error("too large: {0}")]
    TooLarge(String),
    #[error("library is busy (another writer holds the lock)")]
    Busy,
    /// `reason`：`too-new`、`unknown-extension`、`migrating`。
    #[error("library version: {reason}")]
    LibraryVersion { reason: &'static str },
    #[error("system random source failed: {0}")]
    Random(String),
    #[error(transparent)]
    Contract(#[from] usuyo_contract::ContractError),
}

impl Error {
    /// SPEC §0.3 的错误码。
    pub fn code(&self) -> &'static str {
        match self {
            Error::Io { .. } | Error::Random(_) => "internal",
            Error::Json { .. } | Error::Malformed { .. } => "internal",
            Error::InvalidArgs(_) | Error::Contract(_) => "invalid-args",
            Error::NotFound(_) => "not-found",
            Error::Conflict(_) => "conflict",
            Error::TooLarge(_) => "too-large",
            Error::Busy => "library-busy",
            Error::LibraryVersion { .. } => "library-version",
        }
    }

    pub(crate) fn io(path: impl Into<PathBuf>, source: std::io::Error) -> Error {
        Error::Io {
            path: path.into(),
            source,
        }
    }

    pub(crate) fn malformed(path: impl Into<PathBuf>, reason: impl Into<String>) -> Error {
        Error::Malformed {
            path: path.into(),
            reason: reason.into(),
        }
    }
}

pub type Result<T, E = Error> = std::result::Result<T, E>;
