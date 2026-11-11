//! usuyo 资料库：文件就是真相的那一层（SPEC 第 0、1 层）。
//!
//! 逻辑写成纯函数（id、文本规则、front matter、划线、引用、三方合并、manifest 重算），
//! 读写文件集中在 [`Library`] 的方法里。

mod citation;
mod doc;
mod error;
mod frontmatter;
mod fsx;
mod hash;
mod highlights;
mod id;
mod item;
mod journal;
mod library;
mod link;
pub mod merge;
mod ops;
mod proposals;
mod segments;
mod text;
pub mod time;
mod trash;
mod writing;

pub use citation::{Citation, citations};
pub use doc::{Doc, Paragraphs, Splice, apply_hunks, revert_splices};
pub use error::{Error, Result};
pub use fsx::TextStyle;
pub use hash::{hash_tag, sha256_hex};
pub use highlights::Highlight;
pub use id::{ItemId, block_id, identity_key, is_block_id, is_uuid7, normalize_web_url, uuid7};
pub use item::{
    AddOutcome, Capture, Item, LayerSource, NewOrigin, NewSnapshot, hash_staged, outputs_of,
    refresh, render_content,
};
pub use journal::Batch;
pub use library::{FORMAT_VERSION, Library};
pub use link::{FORGOTTEN, Link, column_label, parse_time_fragment, text_fragment, time_fragment};
pub use ops::{Applied, UndoReport};
pub use text::{nfc, sanitize};
pub use trash::{PurgeReport, expires_at};
pub use writing::{DocKind, Notes, Scan, Titled, Written};
