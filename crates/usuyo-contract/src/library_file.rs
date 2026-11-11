//! `.usuyo/library.json`、`settings.json`、`devices/<deviceId>.json`（SPEC §1.1）。

use serde::{Deserialize, Serialize};

use crate::Extra;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LibraryFile {
    pub format_version: u32,
    #[serde(default)]
    pub extensions: Extra,
    pub library_id: String,
    pub created_at: String,
    #[serde(default)]
    pub sync: Option<serde_json::Value>,
    #[serde(flatten)]
    pub extra: Extra,
}

/// 跟着资料库走的设置；这一层只定 `trashDays`（`null` 是「一直」）。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    #[serde(default = "default_trash_days")]
    pub trash_days: Option<u32>,
    #[serde(flatten)]
    pub extra: Extra,
}

fn default_trash_days() -> Option<u32> {
    Some(30)
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            trash_days: default_trash_days(),
            extra: Extra::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceRecord {
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub revoked_at: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_seen_at: Option<String>,
    #[serde(flatten)]
    pub extra: Extra,
}
