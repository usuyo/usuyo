//! usuyo 的全文索引：可随时从文件重建，放在每台机器自己的应用数据目录里。
//!
//! 调用方把资料库里的条目映射成 [`IndexDoc`]，经 [`SearchIndex`] 写入和搜索；
//! 搜索结果按条目分组，每个命中带层、调用方给的定位串和高亮片段。

mod analyzer;
mod collect;
mod doc;
mod error;
mod filter;
mod query;
mod schema;
mod search;
mod snippet;
mod stamp;
mod store;

pub use doc::{IndexDoc, LayerText, Segment};
pub use error::{Error, Result};
pub use filter::{DateRange, Filter, Visibility};
pub use search::{ItemHit, SearchPage, SearchRequest, SegmentHit};
pub use snippet::Snippet;
pub use store::SearchIndex;
