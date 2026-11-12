//! `ffprobe` 看一个文件：有哪些流、多长、什么编码、多大。输出是 JSON，当作不可信的文字读：
//! 缺的字段是 `None`，时长不是有限的非负数就当没有。

use std::ffi::OsString;
use std::path::Path;
use std::time::Duration;

use serde::Deserialize;
use usuyo_plugin_kit::protocol::MediaKind;

use crate::exec::{self, ExecError, Spec, Stdout, Supervisor};
use crate::tools::{Tools, input_args};

/// ffprobe 一次最多跑多久（它只读文件头，大文件也该很快）。
const PROBE_TIMEOUT: Duration = Duration::from_secs(60);

/// 只问要的字段：流的序号、类型、编码、宽高、时长、是不是封面图，和文件的总时长。
const SHOW_ENTRIES: &str = "stream=index,codec_type,codec_name,width,height,duration:stream_disposition=attached_pic:format=duration";

#[derive(Debug, thiserror::Error)]
pub enum ProbeError {
    #[error(transparent)]
    Run(#[from] ExecError),
    #[error("path: {0}")]
    Path(#[from] std::io::Error),
    #[error("ffprobe printed JSON this plugin cannot read: {0}")]
    Json(#[from] serde_json::Error),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Stream {
    pub index: u32,
    pub kind: MediaKind,
    pub codec: Option<String>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    /// mp3、m4a 里的封面图：ffprobe 也把它算成一条画面流，这里单独标出来。
    pub attached_picture: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MediaInfo {
    /// 文件的总时长；没有就取各条流里最长的。
    pub duration_sec: Option<f64>,
    pub streams: Vec<Stream>,
}

impl MediaInfo {
    /// 真正的画面流（不含封面图）。
    pub fn pictures(&self) -> impl Iterator<Item = &Stream> {
        self.streams
            .iter()
            .filter(|s| s.kind == MediaKind::Video && !s.attached_picture)
    }

    pub fn sounds(&self) -> impl Iterator<Item = &Stream> {
        self.streams.iter().filter(|s| s.kind == MediaKind::Audio)
    }
}

pub fn probe(tools: &Tools, path: &Path, sup: &dyn Supervisor) -> Result<MediaInfo, ProbeError> {
    let mut args: Vec<OsString> = ["-hide_banner", "-loglevel", "error", "-of", "json"]
        .map(OsString::from)
        .into();
    args.extend(["-show_entries", SHOW_ENTRIES].map(OsString::from));
    args.extend(input_args(path)?);
    let spec = Spec {
        program: &tools.ffprobe.path,
        args,
        timeout: PROBE_TIMEOUT,
        stdout: Stdout::Capture,
        total: 0.0,
        unit: "sec",
    };
    let finished = exec::run(&spec, sup, |_| false)?;
    Ok(parse_probe(&finished.stdout)?)
}

#[derive(Deserialize)]
struct RawProbe {
    #[serde(default)]
    streams: Vec<RawStream>,
    format: Option<RawFormat>,
}

#[derive(Deserialize)]
struct RawFormat {
    duration: Option<Number>,
}

#[derive(Deserialize)]
struct RawStream {
    index: Option<u32>,
    codec_type: Option<String>,
    codec_name: Option<String>,
    width: Option<u32>,
    height: Option<u32>,
    duration: Option<Number>,
    disposition: Option<RawDisposition>,
}

#[derive(Deserialize)]
struct RawDisposition {
    attached_pic: Option<u8>,
}

/// ffprobe 把时长写成字符串；也认数字。
#[derive(Deserialize)]
#[serde(untagged)]
enum Number {
    Text(String),
    Value(f64),
}

impl Number {
    fn seconds(&self) -> Option<f64> {
        let v = match self {
            Number::Text(t) => t.trim().parse().ok()?,
            Number::Value(v) => *v,
        };
        (v.is_finite() && v >= 0.0).then_some(v)
    }
}

pub fn parse_probe(json: &[u8]) -> Result<MediaInfo, serde_json::Error> {
    let raw: RawProbe = serde_json::from_slice(json)?;
    let longest_stream = raw
        .streams
        .iter()
        .filter_map(|s| s.duration.as_ref()?.seconds())
        .fold(None, |best: Option<f64>, d| {
            Some(best.map_or(d, |b| b.max(d)))
        });
    let duration_sec = raw
        .format
        .and_then(|f| f.duration?.seconds())
        .or(longest_stream);
    let streams = raw
        .streams
        .into_iter()
        .enumerate()
        .map(|(position, s)| Stream {
            index: s
                .index
                .unwrap_or(u32::try_from(position).unwrap_or(u32::MAX)),
            kind: match s.codec_type.as_deref() {
                Some("video") => MediaKind::Video,
                Some("audio") => MediaKind::Audio,
                Some("subtitle") => MediaKind::Subtitle,
                _ => MediaKind::Other,
            },
            codec: s
                .codec_name
                .filter(|c| !c.is_empty() && c.len() <= 64 && c.chars().all(|c| !c.is_control())),
            width: s.width,
            height: s.height,
            attached_picture: s
                .disposition
                .and_then(|d| d.attached_pic)
                .is_some_and(|v| v == 1),
        })
        .collect();
    Ok(MediaInfo {
        duration_sec,
        streams,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn video_with_one_stream() {
        let info = parse_probe(
            br#"{"programs":[],"streams":[{"index":0,"codec_name":"h264","codec_type":"video","width":320,"height":180,"duration":"7.520000","disposition":{"attached_pic":0}}],"format":{"duration":"7.520000"}}"#,
        )
        .unwrap();
        assert_eq!(info.duration_sec, Some(7.52));
        assert_eq!(info.pictures().count(), 1);
        assert_eq!(info.sounds().count(), 0);
        let s = &info.streams[0];
        assert_eq!((s.width, s.height), (Some(320), Some(180)));
        assert_eq!(s.codec.as_deref(), Some("h264"));
    }

    #[test]
    fn cover_art_is_not_a_picture_and_missing_fields_are_none() {
        let info = parse_probe(
            br#"{"streams":[{"index":0,"codec_type":"audio","codec_name":"mp3"},{"index":1,"codec_type":"video","codec_name":"mjpeg","disposition":{"attached_pic":1}},{"codec_type":"data"}],"format":{"duration":"N/A"}}"#,
        )
        .unwrap();
        assert_eq!(info.pictures().count(), 0);
        assert_eq!(info.sounds().count(), 1);
        assert_eq!(info.duration_sec, None);
        assert_eq!(info.streams[2].kind, MediaKind::Other);
        assert_eq!(info.streams[2].index, 2);
    }

    #[test]
    fn duration_falls_back_to_the_longest_stream_and_rejects_nonsense() {
        let info = parse_probe(
            br#"{"streams":[{"codec_type":"video","duration":"3.5"},{"codec_type":"audio","duration":"9"}],"format":{"duration":"-1"}}"#,
        )
        .unwrap();
        assert_eq!(info.duration_sec, Some(9.0));
        let info = parse_probe(br#"{"streams":[],"format":{"duration":"inf"}}"#).unwrap();
        assert_eq!(info.duration_sec, None);
        assert!(parse_probe(b"not json").is_err());
        assert_eq!(parse_probe(b"{}").unwrap().streams, []);
    }
}
