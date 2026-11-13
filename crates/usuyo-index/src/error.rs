use std::path::PathBuf;

/// 索引 crate 的统一错误。文字是英文，只进日志和 `detail`；给人看的话由上层按码配（SPEC §0.3）。
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("not a usuyo index and not empty: {0}")]
    ForeignDirectory(PathBuf),
    #[error("the index is being written by another process")]
    Busy,
    #[error("the query is empty or has nothing searchable")]
    EmptyQuery,
    #[error("the query is too long: at most {max_terms} terms of {max_chars} characters each")]
    QueryTooLong { max_terms: usize, max_chars: usize },
    #[error("invalid cursor")]
    BadCursor,
    #[error("layer name {0:?} is empty or reserved")]
    InvalidLayer(String),
    #[error("index directory: {0}")]
    Io(#[from] std::io::Error),
    #[error("index stamp file: {0}")]
    Stamp(#[from] serde_json::Error),
    #[error("opening the index directory: {0}")]
    OpenDirectory(#[from] tantivy::directory::error::OpenDirectoryError),
    #[error("index engine: {0}")]
    Engine(#[from] tantivy::TantivyError),
}

pub type Result<T> = std::result::Result<T, Error>;
