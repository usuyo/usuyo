//! 一次 `keyframes` 转录里的 `keyframes.json`（SPEC §1.2.2）。插件写，宿主读。

use serde::{Deserialize, Serialize};

use crate::{Extra, is_plain_relative};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Keyframes {
    /// 画面所在的快照。
    pub snapshot: String,
    pub duration_sec: f64,
    #[serde(default)]
    pub frames: Vec<Keyframe>,
    /// 画面停住的时间加起来多少秒。
    pub frozen_sec: f64,
    #[serde(flatten)]
    pub extra: Extra,
}

/// 一帧；`path` 相对转录文件夹，`keyframes/<毫秒，8 位补零>.jpg`。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Keyframe {
    pub sec: f64,
    pub path: String,
    pub sha256: String,
    pub width: u32,
    pub height: u32,
    /// `scdet` 给这一处镜头变化打的分；不是镜头变化处取的帧为 `null`。
    pub scene: Option<f64>,
    #[serde(flatten)]
    pub extra: Extra,
}

impl Keyframes {
    /// 第一个不是普通相对路径的帧路径。
    pub fn bad_path(&self) -> Option<&str> {
        self.frames
            .iter()
            .map(|f| f.path.as_str())
            .find(|p| !is_plain_relative(p))
    }
}
