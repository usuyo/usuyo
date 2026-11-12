//! `keyframes` 的参数（feature 003 §5.3）：读进来、补默认值、原样回显。宿主重做时把回显的那份原样交回，
//! 所以回显的每个键这里都认；不认识的键和越界的值都报错。
//!
//! 清单没有 `settings`：设置的键只能是小写加 `-`，也没有小数类型，这些只经 `run` 的 `options` 交进来。
//! `play`、`audio:pcm16k`、`probe` 没有参数，给了任何键都报错。

use serde_json::{Map, Value, json};

#[derive(Debug, Clone, PartialEq)]
pub struct KeyframeOptions {
    /// 第一遍低清扫描每秒取几帧。
    pub scan_fps: f64,
    /// 第一遍缩到多高（宽按比例，取偶数）。
    pub scan_height: u32,
    /// `scdet` 的阈值：分数不低于它算镜头变化。
    pub scene_threshold: f64,
    /// `freezedetect` 的噪声容限。
    pub freeze_noise: f64,
    /// 停住多久才算一段。
    pub freeze_min_sec: f64,
    /// 总要取的那一帧的时间。
    pub first_sec: f64,
    /// 镜头变化处取的帧，和已选的任何一帧至少隔多少秒。
    pub min_gap_sec: f64,
    pub max_per_minute: u32,
    pub max_frames: u32,
    /// 选出来的帧少于 `min(minFrames, 时长 / minFrameEverySec)` 时按时长均分补齐。
    pub min_frames: u32,
    pub min_frame_every_sec: f64,
    /// JPEG 长边最多多少像素。
    pub max_side: u32,
    /// `-q:v`，1 最好，31 最差。
    pub jpeg_quality: u32,
    /// 停住的时间加起来到时长的这个比例，标 `static-picture`（§6）。
    pub static_ratio: f64,
}

impl Default for KeyframeOptions {
    fn default() -> Self {
        Self {
            scan_fps: 4.0,
            scan_height: 360,
            scene_threshold: 10.0,
            freeze_noise: 0.003,
            freeze_min_sec: 2.0,
            first_sec: 0.5,
            min_gap_sec: 2.0,
            max_per_minute: 6,
            max_frames: 60,
            min_frames: 3,
            min_frame_every_sec: 10.0,
            max_side: 960,
            jpeg_quality: 4,
            static_ratio: 0.9,
        }
    }
}

#[derive(Debug, thiserror::Error)]
#[error("option `{key}`: {why}")]
pub struct OptionError {
    key: String,
    why: String,
}

fn bad(key: &str, why: impl Into<String>) -> OptionError {
    OptionError {
        key: key.to_owned(),
        why: why.into(),
    }
}

/// 没有参数的产物：给了键就报错。
pub fn expect_none(map: &Map<String, Value>) -> Result<(), OptionError> {
    match map.keys().next() {
        Some(key) => Err(bad(key, "this product takes no options")),
        None => Ok(()),
    }
}

fn float(key: &str, v: &Value, min: f64, max: f64) -> Result<f64, OptionError> {
    let n = v.as_f64().ok_or_else(|| bad(key, "must be a number"))?;
    if n.is_finite() && (min..=max).contains(&n) {
        Ok(n)
    } else {
        Err(bad(key, format!("must be between {min} and {max}")))
    }
}

fn int(key: &str, v: &Value, min: u32, max: u32) -> Result<u32, OptionError> {
    let n = v
        .as_u64()
        .ok_or_else(|| bad(key, "must be a whole number"))?;
    match u32::try_from(n) {
        Ok(n) if (min..=max).contains(&n) => Ok(n),
        _ => Err(bad(key, format!("must be between {min} and {max}"))),
    }
}

impl KeyframeOptions {
    /// 读 `run` 的 `options`；没给的取默认。
    pub fn from_map(map: &Map<String, Value>) -> Result<Self, OptionError> {
        let mut o = Self::default();
        for (key, v) in map {
            match key.as_str() {
                "scanFps" => o.scan_fps = float(key, v, 0.1, 30.0)?,
                "scanHeight" => o.scan_height = int(key, v, 16, 2160)?,
                "sceneThreshold" => o.scene_threshold = float(key, v, 0.0, 100.0)?,
                "freezeNoise" => o.freeze_noise = float(key, v, 0.0, 1.0)?,
                "freezeMinSec" => o.freeze_min_sec = float(key, v, 0.1, 3600.0)?,
                "firstSec" => o.first_sec = float(key, v, 0.0, 3600.0)?,
                "minGapSec" => o.min_gap_sec = float(key, v, 0.0, 3600.0)?,
                "maxPerMinute" => o.max_per_minute = int(key, v, 1, 600)?,
                "maxFrames" => o.max_frames = int(key, v, 1, 1000)?,
                "minFrames" => o.min_frames = int(key, v, 0, 100)?,
                "minFrameEverySec" => o.min_frame_every_sec = float(key, v, 0.1, 3600.0)?,
                "maxSide" => o.max_side = int(key, v, 16, 4096)?,
                "jpegQuality" => o.jpeg_quality = int(key, v, 1, 31)?,
                "staticRatio" => o.static_ratio = float(key, v, 0.01, 1.0)?,
                _ => return Err(bad(key, "unknown option")),
            }
        }
        if o.min_frames > o.max_frames {
            return Err(bad("minFrames", "must not exceed maxFrames"));
        }
        Ok(o)
    }

    /// 回显用：实际用的每个参数。
    pub fn to_map(&self) -> Map<String, Value> {
        let Value::Object(map) = json!({
            "scanFps": self.scan_fps,
            "scanHeight": self.scan_height,
            "sceneThreshold": self.scene_threshold,
            "freezeNoise": self.freeze_noise,
            "freezeMinSec": self.freeze_min_sec,
            "firstSec": self.first_sec,
            "minGapSec": self.min_gap_sec,
            "maxPerMinute": self.max_per_minute,
            "maxFrames": self.max_frames,
            "minFrames": self.min_frames,
            "minFrameEverySec": self.min_frame_every_sec,
            "maxSide": self.max_side,
            "jpegQuality": self.jpeg_quality,
            "staticRatio": self.static_ratio,
        }) else {
            unreachable!("json! of an object literal is an object")
        };
        map
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn map(v: Value) -> Map<String, Value> {
        v.as_object().unwrap().clone()
    }

    #[test]
    fn defaults_are_the_documented_ones_and_round_trip() {
        let o = KeyframeOptions::from_map(&Map::new()).unwrap();
        assert_eq!(o, KeyframeOptions::default());
        assert_eq!(o.scan_fps, 4.0);
        assert_eq!(o.scan_height, 360);
        assert_eq!(o.scene_threshold, 10.0);
        assert_eq!(o.freeze_noise, 0.003);
        assert_eq!(o.freeze_min_sec, 2.0);
        assert_eq!((o.first_sec, o.min_gap_sec), (0.5, 2.0));
        assert_eq!((o.max_per_minute, o.max_frames), (6, 60));
        assert_eq!((o.min_frames, o.min_frame_every_sec), (3, 10.0));
        assert_eq!((o.max_side, o.jpeg_quality), (960, 4));
        assert_eq!(o.static_ratio, 0.9);
        assert_eq!(KeyframeOptions::from_map(&o.to_map()).unwrap(), o);
    }

    #[test]
    fn bad_options_are_refused() {
        for bad in [
            json!({"nope": 1}),
            json!({"maxFrames": 0}),
            json!({"maxFrames": 2.5}),
            json!({"maxFrames": "60"}),
            json!({"scanFps": 0}),
            json!({"staticRatio": 1.5}),
            json!({"jpegQuality": 99}),
            json!({"minFrames": 5, "maxFrames": 4}),
        ] {
            assert!(
                KeyframeOptions::from_map(&map(bad.clone())).is_err(),
                "{bad}"
            );
        }
        assert!(expect_none(&map(json!({}))).is_ok());
        assert!(expect_none(&map(json!({"x": 1}))).is_err());
    }
}
