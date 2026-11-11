//! 索引目录里的版本标记：分词器或字段变了，旧索引要重建；
//! 重建没做完（`complete == false`）也要重建。

use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::analyzer::ANALYZER_VERSION;
use crate::error::Result;

pub(crate) const FILE_NAME: &str = "usuyo-index.json";
/// 字段布局变了就加一。
const SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct Stamp {
    schema: u32,
    analyzer: u32,
    pub complete: bool,
}

impl Stamp {
    pub fn current(complete: bool) -> Self {
        Self {
            schema: SCHEMA_VERSION,
            analyzer: ANALYZER_VERSION,
            complete,
        }
    }

    pub fn is_current(&self) -> bool {
        self.schema == SCHEMA_VERSION && self.analyzer == ANALYZER_VERSION
    }

    /// 没有这个文件返回 `None`；文件内容读不懂也算版本不对，返回 `Some(Err)` 交给调用方重建。
    pub fn read(dir: &Path) -> Result<Option<std::result::Result<Self, serde_json::Error>>> {
        match fs::read(dir.join(FILE_NAME)) {
            Ok(bytes) => Ok(Some(serde_json::from_slice(&bytes))),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    /// 先写临时文件再改名，半截的标记不会出现。
    pub fn write(&self, dir: &Path) -> Result<()> {
        let tmp = dir.join(format!("{FILE_NAME}.tmp"));
        fs::write(&tmp, serde_json::to_vec(self)?)?;
        fs::rename(tmp, dir.join(FILE_NAME))?;
        Ok(())
    }
}
