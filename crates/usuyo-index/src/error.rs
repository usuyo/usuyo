use std::path::PathBuf;

/// 索引 crate 的统一错误。
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("目录不是 usuyo 索引且不为空：{0}")]
    ForeignDirectory(PathBuf),
    #[error("索引正被另一个进程写入")]
    Busy,
    #[error("搜索词为空，或没有可搜的文字")]
    EmptyQuery,
    #[error("搜索词太长：最多 {max_terms} 个词、每个词 {max_chars} 个字符")]
    QueryTooLong { max_terms: usize, max_chars: usize },
    #[error("翻页游标无效")]
    BadCursor,
    #[error("层名 {0:?} 不可用：空，或是内部保留名")]
    InvalidLayer(String),
    #[error("读写索引目录失败：{0}")]
    Io(#[from] std::io::Error),
    #[error("索引标记文件无法读写：{0}")]
    Stamp(#[from] serde_json::Error),
    #[error("打开索引目录失败：{0}")]
    OpenDirectory(#[from] tantivy::directory::error::OpenDirectoryError),
    #[error("索引引擎出错：{0}")]
    Engine(#[from] tantivy::TantivyError),
}

pub type Result<T> = std::result::Result<T, Error>;
