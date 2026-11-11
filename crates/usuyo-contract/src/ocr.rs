//! 一次 `ocr` 或 `frames` 转录里的 `ocr.json`（SPEC §1.2.2）。插件写，宿主读。
//! 框是 `[x, y, w, h]`，0–1，按图的宽高算，原点在左上角。

use serde::{Deserialize, Serialize};

use crate::Extra;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Ocr {
    #[serde(default)]
    pub images: Vec<OcrImage>,
    #[serde(flatten)]
    pub extra: Extra,
}

/// 一张图上认出的字。图文的图和封面带 `imageKey`，关键帧带 `frameSec`。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OcrImage {
    /// `run.json` 里 `inputs` 的下标。
    pub input: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub image_key: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub frame_sec: Option<f64>,
    pub width: u32,
    pub height: u32,
    #[serde(default)]
    pub lines: Vec<OcrLine>,
    #[serde(flatten)]
    pub extra: Extra,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OcrLine {
    pub text: String,
    pub bbox: [f64; 4],
    /// 引擎自己的分，不跨引擎比。
    pub confidence: f64,
    #[serde(flatten)]
    pub extra: Extra,
}
