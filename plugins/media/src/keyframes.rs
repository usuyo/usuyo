//! `keyframes`：两遍。第一遍低清扫一遍（`fps`、`scale`、`scdet`、`freezedetect`），拿到镜头变化和停住的区间；
//! 选帧见 [`crate::select`]；第二遍逐个时间点取一张 JPEG。写 `keyframes.json`（契约里的 `Keyframes`）。

use std::ffi::OsString;
use std::fs;
use std::io::ErrorKind;
use std::path::Path;
use std::time::Duration;

use serde::Serialize;
use sha2::{Digest, Sha256};
use usuyo_plugin_kit::protocol::{
    Mark, MarkCode, Produce, RelPath, RunProduct, RunResult, Seconds,
};

use crate::budget;
use crate::error::MediaError;
use crate::exec::{self, Spec, Stdout, Supervisor};
use crate::jpeg;
use crate::options::KeyframeOptions;
use crate::probe::probe;
use crate::scan::{is_scan_line, parse_scan};
use crate::select::select;
use crate::tools::{Tools, ffmpeg_prefix, file_arg, input_args};

const JSON_FILE: &str = "keyframes.json";
const FRAME_DIR: &str = "keyframes";
/// 取一帧最多等多久。
const FRAME_TIMEOUT: Duration = Duration::from_secs(60);

/// `keyframes.json`：字段和 `urn:usuyo:schema:keyframes:1` 一致。
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct KeyframesFile {
    snapshot: String,
    duration_sec: f64,
    frames: Vec<FrameEntry>,
    frozen_sec: f64,
}

#[derive(Serialize)]
struct FrameEntry {
    sec: f64,
    path: String,
    sha256: String,
    width: u32,
    height: u32,
    scene: Option<f64>,
}

/// 毫秒取整，写进 JSON 的秒数最多 3 位小数。
fn rounded(sec: f64) -> f64 {
    (sec * 1000.0).round() / 1000.0
}

fn seconds(sec: f64) -> Result<Seconds, MediaError> {
    Seconds::try_from(rounded(sec)).map_err(MediaError::Output)
}

pub fn run(
    tools: &Tools,
    sup: &dyn Supervisor,
    input: &Path,
    snapshot: &str,
    out: &Path,
    options: &KeyframeOptions,
) -> Result<RunResult, MediaError> {
    let path_text = input.display().to_string();
    let info = probe(tools, input, sup).map_err(|source| MediaError::Probe {
        path: path_text.clone(),
        source,
    })?;
    let picture = info
        .pictures()
        .next()
        .ok_or_else(|| MediaError::Refused(format!("{path_text} has no video stream")))?;
    let duration = info
        .duration_sec
        .filter(|d| *d > 0.0)
        .ok_or_else(|| MediaError::Refused(format!("{path_text} has no usable duration")))?;
    let stream = picture.index;

    sup.progress(0.0, duration, "sec");
    let scan = scan(tools, sup, input, stream, duration, options)?;
    let frozen = scan.frozen_sec(duration);
    let picks = select(duration, &scan.scenes, options);

    let frames_dir = out.join(FRAME_DIR);
    fs::create_dir(&frames_dir)
        .map_err(|e| MediaError::io(format!("create {}", frames_dir.display()), e))?;
    let mut frames = Vec::with_capacity(picks.len());
    for (done, pick) in picks.iter().enumerate() {
        sup.progress(done as f64, picks.len() as f64, "frames");
        let name = format!("{:08}.jpg", pick.ms);
        let target = frames_dir.join(&name);
        extract(tools, sup, input, stream, pick.ms, &target, options)?;
        // 取不到（时间点在最后一帧之后）的帧不算，留一行日志；全取不到才是错。
        let bytes = match fs::read(&target) {
            Ok(bytes) => bytes,
            Err(e) if e.kind() == ErrorKind::NotFound => {
                eprintln!("media: no frame at {} ms in {path_text}", pick.ms);
                continue;
            }
            Err(e) => return Err(MediaError::io(format!("read {}", target.display()), e)),
        };
        let (width, height) = jpeg::dimensions(&bytes)
            .ok_or_else(|| MediaError::Output(format!("{name} is not a readable JPEG")))?;
        frames.push(FrameEntry {
            sec: rounded(pick.ms as f64 / 1000.0),
            path: format!("{FRAME_DIR}/{name}"),
            sha256: Sha256::digest(&bytes)
                .iter()
                .map(|b| format!("{b:02x}"))
                .collect(),
            width,
            height,
            scene: pick.scene,
        });
    }
    if frames.is_empty() {
        return Err(MediaError::Output(format!(
            "no frame could be taken from {path_text}"
        )));
    }
    sup.progress(picks.len() as f64, picks.len() as f64, "frames");

    let file = KeyframesFile {
        snapshot: snapshot.to_owned(),
        duration_sec: rounded(duration),
        frames,
        frozen_sec: rounded(frozen),
    };
    let json = serde_json::to_vec_pretty(&file)
        .map_err(|e| MediaError::Output(format!("write {JSON_FILE}: {e}")))?;
    fs::write(out.join(JSON_FILE), json)
        .map_err(|e| MediaError::io(format!("write {JSON_FILE}"), e))?;

    let mut result = RunResult {
        outputs: vec![RunProduct {
            kind: Produce::Keyframes,
            path: RelPath::try_from(JSON_FILE).map_err(MediaError::Output)?,
            lang: None,
        }],
        tool: tools.ffmpeg.upstream("ffmpeg"),
        options: options.to_map(),
        model: None,
        marks: Vec::new(),
        empty: false,
        reason: None,
        probe: Vec::new(),
    };
    if is_static(frozen, duration, options.static_ratio) {
        result.marks.push(Mark {
            code: MarkCode::StaticPicture,
            speech_sec: None,
            duration_sec: Some(seconds(duration)?),
            frozen_sec: Some(seconds(frozen)?),
        });
    }
    Ok(result)
}

/// 停住的时间到时长的 `ratio`（§6 是 90%）就算「画面几乎不动」。
pub fn is_static(frozen_sec: f64, duration_sec: f64, ratio: f64) -> bool {
    duration_sec > 0.0 && frozen_sec >= duration_sec * ratio
}

fn scan(
    tools: &Tools,
    sup: &dyn Supervisor,
    input: &Path,
    stream: u32,
    duration: f64,
    o: &KeyframeOptions,
) -> Result<crate::scan::Scan, MediaError> {
    let mut args = ffmpeg_prefix("info", true);
    args.extend(input_args(input).map_err(|e| MediaError::io("input path", e))?);
    let filters = format!(
        "fps={},scale=-2:{},scdet=threshold={},freezedetect=n={}:d={}",
        o.scan_fps, o.scan_height, o.scene_threshold, o.freeze_noise, o.freeze_min_sec
    );
    args.extend(
        [
            "-map".to_owned(),
            format!("0:{stream}"),
            "-an".into(),
            "-sn".into(),
            "-dn".into(),
            "-vf".into(),
            filters,
            "-f".into(),
            "null".into(),
            "-".into(),
        ]
        .map(OsString::from),
    );
    let spec = Spec {
        program: &tools.ffmpeg.path,
        args,
        timeout: budget::timeout(Some(duration), 120.0, 2.0),
        stdout: Stdout::FfmpegProgress,
        total: duration,
        unit: "sec",
    };
    let finished = exec::run(&spec, sup, is_scan_line)?;
    Ok(parse_scan(&finished.kept))
}

/// 在 `ms` 毫秒处取一帧，长边缩到不超过 `maxSide`（不放大），写成 JPEG。
fn extract(
    tools: &Tools,
    sup: &dyn Supervisor,
    input: &Path,
    stream: u32,
    ms: u64,
    target: &Path,
    o: &KeyframeOptions,
) -> Result<(), MediaError> {
    let mut args = ffmpeg_prefix("error", false);
    args.extend([
        "-n".into(),
        "-ss".into(),
        format!("{}.{:03}", ms / 1000, ms % 1000).into(),
    ]);
    args.extend(input_args(input).map_err(|e| MediaError::io("input path", e))?);
    let side = o.max_side;
    let scale = format!(
        "scale='if(gt(iw,ih),min({side},iw),-2)':'if(gt(iw,ih),-2,min({side},ih))',format=yuvj420p"
    );
    args.extend(
        [
            "-map".to_owned(),
            format!("0:{stream}"),
            "-frames:v".into(),
            "1".into(),
            "-vf".into(),
            scale,
            "-q:v".into(),
            o.jpeg_quality.to_string(),
            "-f".into(),
            "image2".into(),
        ]
        .map(OsString::from),
    );
    args.push(file_arg(target).map_err(|e| MediaError::io("output path", e))?);
    let spec = Spec {
        program: &tools.ffmpeg.path,
        args,
        timeout: FRAME_TIMEOUT,
        stdout: Stdout::Capture,
        total: 1.0,
        unit: "frames",
    };
    exec::run(&spec, sup, |_| false)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn static_means_ninety_percent_frozen() {
        assert!(is_static(9.0, 10.0, 0.9));
        assert!(is_static(10.0, 10.0, 0.9));
        assert!(!is_static(8.9, 10.0, 0.9));
        assert!(!is_static(0.0, 0.0, 0.9));
    }

    #[test]
    fn json_numbers_have_at_most_three_decimals() {
        assert_eq!(rounded(12.0400000001), 12.04);
        assert_eq!(rounded(63.2), 63.2);
    }
}
