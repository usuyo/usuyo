//! `probe`、`play`、`audio:pcm16k` 三种产物。`keyframes` 在 [`crate::keyframes`]。

use std::ffi::OsString;
use std::path::Path;

use usuyo_plugin_kit::protocol::{
    EmptyReason, Mark, MarkCode, Probe, ProbeStream, Produce, RelPath, RunProduct, RunResult,
    Seconds,
};

use crate::budget;
use crate::error::MediaError;
use crate::exec::{self, Spec, Stdout, Supervisor};
use crate::probe::{MediaInfo, probe};
use crate::tools::{Tools, ffmpeg_prefix, file_arg, input_args};

const PLAY_FILE: &str = "play.mp4";
const PCM_FILE: &str = "audio.wav";

fn probed(tools: &Tools, sup: &dyn Supervisor, path: &Path) -> Result<MediaInfo, MediaError> {
    probe(tools, path, sup).map_err(|source| MediaError::Probe {
        path: path.display().to_string(),
        source,
    })
}

fn empty_result(tools: &Tools, name: &str) -> RunResult {
    let tool = match name {
        "ffprobe" => tools.ffprobe.upstream(name),
        _ => tools.ffmpeg.upstream(name),
    };
    RunResult {
        outputs: Vec::new(),
        tool,
        options: Default::default(),
        model: None,
        marks: Vec::new(),
        empty: false,
        reason: None,
        probe: Vec::new(),
    }
}

fn product(kind: Produce, file: &str) -> Result<RunProduct, MediaError> {
    Ok(RunProduct {
        kind,
        path: RelPath::try_from(file).map_err(MediaError::Output)?,
        lang: None,
    })
}

fn seconds(sec: f64) -> Result<Seconds, MediaError> {
    Seconds::try_from((sec * 1000.0).round() / 1000.0).map_err(MediaError::Output)
}

/// 每个输入一项：有哪些流、多长、什么编码。
pub fn probe_all(
    tools: &Tools,
    sup: &dyn Supervisor,
    inputs: &[&Path],
) -> Result<RunResult, MediaError> {
    let mut result = empty_result(tools, "ffprobe");
    for (i, path) in inputs.iter().enumerate() {
        let info = probed(tools, sup, path)?;
        result.probe.push(Probe {
            input: u32::try_from(i).map_err(|e| MediaError::Output(e.to_string()))?,
            duration_sec: info.duration_sec.map(seconds).transpose()?,
            streams: info
                .streams
                .iter()
                .map(|s| ProbeStream {
                    kind: s.kind,
                    codec: s.codec.clone(),
                })
                .collect(),
        });
    }
    Ok(result)
}

/// 一个只有画面的文件和一个只有声音的文件，不重编码合成 `play.mp4`（§5.3）。
pub fn play(
    tools: &Tools,
    sup: &dyn Supervisor,
    video: &Path,
    audio: &Path,
    out: &Path,
) -> Result<RunResult, MediaError> {
    let v = probed(tools, sup, video)?;
    let a = probed(tools, sup, audio)?;
    if v.pictures().next().is_none() || v.sounds().next().is_some() {
        return Err(MediaError::Refused(format!(
            "{} is not a video-only file (one picture stream, no audio)",
            video.display()
        )));
    }
    if a.sounds().next().is_none() || a.pictures().next().is_some() {
        return Err(MediaError::Refused(format!(
            "{} is not an audio-only file (audio, no picture stream)",
            audio.display()
        )));
    }
    let duration = match (v.duration_sec, a.duration_sec) {
        (Some(x), Some(y)) => Some(x.max(y)),
        _ => None,
    };
    fresh(out, PLAY_FILE)?;
    let mut args = ffmpeg_prefix("error", true);
    args.push("-n".into());
    args.extend(input_args(video).map_err(|e| MediaError::io("video path", e))?);
    args.extend(input_args(audio).map_err(|e| MediaError::io("audio path", e))?);
    args.extend(
        [
            "-map",
            "0:v:0",
            "-map",
            "1:a:0",
            "-c",
            "copy",
            "-movflags",
            "+faststart",
            "-f",
            "mp4",
        ]
        .map(OsString::from),
    );
    args.push(file_arg(&out.join(PLAY_FILE)).map_err(|e| MediaError::io("output path", e))?);
    let spec = Spec {
        program: &tools.ffmpeg.path,
        args,
        timeout: budget::timeout(duration, 120.0, 0.2),
        stdout: Stdout::FfmpegProgress,
        total: duration.unwrap_or(0.0),
        unit: "sec",
    };
    exec::run(&spec, sup, |_| false)?;
    let mut result = empty_result(tools, "ffmpeg");
    result
        .outputs
        .push(finished_output(Produce::Play, out, PLAY_FILE)?);
    Ok(result)
}

/// 16 kHz 单声道 16 位 WAV，取输入的第一条音轨；没有音轨回 `empty` / `no-audio-track`。
pub fn audio_pcm16k(
    tools: &Tools,
    sup: &dyn Supervisor,
    input: &Path,
    out: &Path,
) -> Result<RunResult, MediaError> {
    let info = probed(tools, sup, input)?;
    let mut result = empty_result(tools, "ffmpeg");
    let Some(track) = info.sounds().next() else {
        result.empty = true;
        result.reason = Some(EmptyReason::NoAudioTrack);
        result.marks.push(Mark {
            code: MarkCode::NoAudioTrack,
            speech_sec: None,
            duration_sec: None,
            frozen_sec: None,
        });
        return Ok(result);
    };
    fresh(out, PCM_FILE)?;
    let mut args = ffmpeg_prefix("error", true);
    args.push("-n".into());
    args.extend(input_args(input).map_err(|e| MediaError::io("input path", e))?);
    args.extend(
        [
            "-map".to_owned(),
            format!("0:{}", track.index),
            "-vn".into(),
            "-sn".into(),
            "-dn".into(),
            "-ac".into(),
            "1".into(),
            "-ar".into(),
            "16000".into(),
            "-c:a".into(),
            "pcm_s16le".into(),
            "-f".into(),
            "wav".into(),
        ]
        .map(OsString::from),
    );
    args.push(file_arg(&out.join(PCM_FILE)).map_err(|e| MediaError::io("output path", e))?);
    let spec = Spec {
        program: &tools.ffmpeg.path,
        args,
        timeout: budget::timeout(info.duration_sec, 120.0, 0.5),
        stdout: Stdout::FfmpegProgress,
        total: info.duration_sec.unwrap_or(0.0),
        unit: "sec",
    };
    exec::run(&spec, sup, |_| false)?;
    result
        .outputs
        .push(finished_output(Produce::AudioPcm16k, out, PCM_FILE)?);
    Ok(result)
}

/// ffmpeg 用 `-n`（不覆盖）：先确认产物还不在，免得把旧文件当成这次的。
fn fresh(out: &Path, file: &str) -> Result<(), MediaError> {
    match std::fs::symlink_metadata(out.join(file)) {
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(MediaError::io(format!("check {file}"), e)),
        Ok(_) => Err(MediaError::Output(format!(
            "{file} already exists in the output directory"
        ))),
    }
}

/// ffmpeg 用 `-n`：文件已经在时它不报错也不写。产物不在就是没做成。
fn finished_output(kind: Produce, out: &Path, file: &str) -> Result<RunProduct, MediaError> {
    let path = out.join(file);
    let meta = std::fs::symlink_metadata(&path)
        .map_err(|e| MediaError::Output(format!("ffmpeg left no {file}: {e}")))?;
    if !meta.is_file() || meta.len() == 0 {
        return Err(MediaError::Output(format!("{file} is empty or not a file")));
    }
    product(kind, file)
}
