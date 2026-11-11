//! 转录记录 `derived/runs/<runId>/run.json`（SPEC §1.2.2）。宿主写，插件不写；写好以后不再改。
//!
//! 词表（`job`、`trigger`、输出的 `kind`、`reason`、标记的 `code`）是字符串：
//! 别的设备上新版本写的转录，这一版照样读得进来、照样写回。

use serde::{Deserialize, Serialize};

use crate::{Extra, is_plain_relative};

/// 一次转换。值为空的字段写 `null`，数组总是出现。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RunRecord {
    /// UUIDv7，等于文件夹名。
    pub id: String,
    /// `play` `asr` `keyframes` `ocr` `frames`
    pub job: String,
    /// 插件 id；宿主自己判出来的是 `host`。
    pub by: String,
    /// 插件的版本；`by: host` 时是内核的版本。
    pub version: String,
    /// 插件里干活的程序，插件在 `run` 的结果里报；宿主写的为 `null`。
    pub tool: Option<Tool>,
    /// 用了模型时：名字和宿主核过的 sha256。
    pub model: Option<ModelRef>,
    /// 这次实际用的参数，含插件补上的默认值；重做时原样交回。
    #[serde(default)]
    pub options: serde_json::Map<String, serde_json::Value>,
    pub at: String,
    pub device: String,
    /// `auto` `open` `redo`
    pub trigger: String,
    /// 点「再转一次」「换一个转」时当时眼前那一份的 id。
    pub redo_of: Option<String>,
    #[serde(default)]
    pub inputs: Vec<RunInput>,
    #[serde(default)]
    pub outputs: Vec<RunOutput>,
    /// 没有可产出的东西；这时 `outputs` 为空、`reason` 有值。
    #[serde(default)]
    pub empty: bool,
    /// `no-speech` `no-audio-track` `no-text`
    pub reason: Option<String>,
    #[serde(default)]
    pub marks: Vec<Mark>,
    #[serde(flatten)]
    pub extra: Extra,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Tool {
    pub name: String,
    pub version: String,
    #[serde(flatten)]
    pub extra: Extra,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModelRef {
    pub name: String,
    pub sha256: String,
    #[serde(flatten)]
    pub extra: Extra,
}

/// 一个输入：相对条目文件夹的路径（`raw/<snapshotId>/…`，或 `frames` 的
/// `derived/runs/<runId>/keyframes/…`）和它所在的快照。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RunInput {
    pub path: String,
    pub sha256: String,
    pub snapshot: String,
    /// 从这个输入到插件真正吃的东西之间的每一步中间转换。
    #[serde(default)]
    pub via: Vec<ViaStep>,
    #[serde(flatten)]
    pub extra: Extra,
}

/// 一步中间转换；产物不进资料库，只记来历。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ViaStep {
    pub by: String,
    pub version: String,
    /// 产物种类，例如 `audio:pcm16k`、`probe`。
    pub op: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool: Option<Tool>,
    /// 产物的 sha256；`probe` 这类不出文件的没有。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sha256: Option<String>,
    #[serde(flatten)]
    pub extra: Extra,
}

/// 转录文件夹里的一个产物；路径相对这个文件夹。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RunOutput {
    pub path: String,
    /// 产物种类：`play` `asr` `keyframes` `ocr`；关键帧图片算在 `keyframes` 里，不另列。
    pub kind: String,
    pub size: u64,
    pub sha256: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lang: Option<String>,
    #[serde(flatten)]
    pub extra: Extra,
}

/// 很明显的标记（feature 003 §6）：只提示，不藏任何东西。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Mark {
    /// `no-audio-track` `no-speech` `static-picture` `no-text`
    pub code: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub speech_sec: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub duration_sec: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub frozen_sec: Option<f64>,
    #[serde(flatten)]
    pub extra: Extra,
}

impl RunRecord {
    /// 第一个不是普通相对路径的输入或输出路径；读 `run.json` 的一方拿到 `Some` 就当它坏了。
    pub fn bad_path(&self) -> Option<&str> {
        self.inputs
            .iter()
            .map(|i| i.path.as_str())
            .chain(self.outputs.iter().map(|o| o.path.as_str()))
            .find(|p| !is_plain_relative(p))
    }
}
