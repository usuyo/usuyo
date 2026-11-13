//! 转出来的文字（feature 003 §5.3、§7）：平台字幕和每次 `asr`、`ocr`、`frames` 转录，读成带原样坐标的段。
//! 条目页和索引都从这里读，`reindex` 前后一样。
//!
//! 字幕和插件的产出都是不可信输入：只当纯文字往外给。读不了的一份跳过、记日志，不挡住别的。

use std::collections::HashMap;
use std::path::Path;

use usuyo_contract::{Keyframes, Ocr, RunRecord};
use usuyo_library::{ItemId, Library};

use crate::words::{self, Words};
use crate::{Error, Result};

/// 三层转出来的文字（和转换任务同名）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TextLayer {
    /// 口播：平台字幕和机器转的。
    Asr,
    /// 图文的图、封面上的字。
    Ocr,
    /// 关键帧上的字。
    Frames,
}

impl TextLayer {
    pub fn as_str(self) -> &'static str {
        match self {
            TextLayer::Asr => "asr",
            TextLayer::Ocr => "ocr",
            TextLayer::Frames => "frames",
        }
    }
}

/// 一段在原样里的哪一处。框是 `[x, y, w, h]`，0–1，原点在左上角。
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Spot {
    Time {
        sec: f64,
        end: Option<f64>,
        snapshot: String,
        xywh: Option<[f64; 4]>,
        /// 认出这一行的那一帧的媒体层：`frame/<keyframes 的 runId>/<毫秒>`；帧文件不在了（只留文字）时没有。
        frame: Option<String>,
    },
    Image {
        key: String,
        xywh: [f64; 4],
    },
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Piece {
    pub text: String,
    pub spot: Spot,
}

/// 机器转的那一次。
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Machine {
    pub run_id: String,
    pub at: String,
    /// 显示的工具名：用了模型时是模型名，不然是插件报的程序名；宿主自己判的没有。
    pub tool: Option<String>,
    /// 做这次转录的转换插件（`run.json` 的 `by`）；宿主自己判的没有。
    pub plugin: Option<String>,
}

/// 一份转出来的文字：一份平台字幕，或一次转录。
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Transcript {
    pub layer: TextLayer,
    /// 索引里的 `source`：转录的 `runId`，平台字幕是 `sub:<snapshotId>:<文件名>`。
    pub source: String,
    pub snapshot: String,
    /// 没有是平台给的。
    pub machine: Option<Machine>,
    pub lang: Option<String>,
    pub pieces: Vec<Piece>,
    /// 很明显的标记（feature 003 §6）写成的一句。
    pub remark: Option<Words>,
}

/// 最新一次 `keyframes`：给画面那一面的帧和标记。
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct KeyframeSet {
    pub run_id: String,
    /// 帧文件还在的那些帧的秒数。
    pub secs: Vec<f64>,
    pub still: bool,
}

/// 标记码 → 那一面头上写的一句（feature 003 §6 的表）。
pub(crate) fn remark_of(code: &str) -> Option<Words> {
    Some(match code {
        "no-audio-track" => words::REMARK_NO_AUDIO,
        "no-speech" => words::REMARK_NO_SPEECH,
        "static-picture" => words::REMARK_STATIC,
        "no-text" => words::REMARK_NO_TEXT,
        "duplicate-images" => words::REMARK_DUPLICATES,
        _ => return None,
    })
}

/// 一次转录的那一句：先看 `marks`，再看 `empty` 的 `reason`。
fn run_remark(r: &RunRecord) -> Option<Words> {
    r.marks
        .iter()
        .find_map(|m| remark_of(&m.code))
        .or_else(|| r.reason.as_deref().and_then(remark_of))
}

/// 一条的全部转出来的文字：口播（平台字幕在前，机器的新的在前）、图里的字、画面上的字。
pub(crate) fn read(lib: &Library, id: &ItemId) -> Result<(Vec<Transcript>, Option<KeyframeSet>)> {
    let dir = lib.root().join(id.rel_dir());
    let mut out = Vec::new();
    for sub in lib.platform_subtitles(id)? {
        let text = match std::fs::read(&sub.file) {
            Ok(b) => String::from_utf8_lossy(&b).into_owned(),
            Err(e) => {
                tracing::warn!(item = %id, path = %sub.file.display(), error = %e, "subtitle not readable; skipped");
                continue;
            }
        };
        let name = sub.path.rsplit('/').next().unwrap_or(&sub.path);
        out.push(Transcript {
            layer: TextLayer::Asr,
            source: format!("sub:{}:{name}", sub.snapshot),
            snapshot: sub.snapshot.clone(),
            machine: None,
            lang: sub.lang.clone(),
            pieces: timed(&sub.snapshot, &text),
            remark: None,
        });
    }
    let runs = lib.runs(id)?;
    let mut machine: Vec<Transcript> = Vec::new();
    for r in &runs {
        let layer = match r.job.as_str() {
            "asr" => TextLayer::Asr,
            "ocr" => TextLayer::Ocr,
            "frames" => TextLayer::Frames,
            _ => continue,
        };
        match transcript(lib, id, &dir, r, layer) {
            Ok(t) => machine.push(t),
            Err(e) => {
                tracing::warn!(item = %id, run = %r.id, error = %e, "run not readable; skipped")
            }
        }
    }
    // 同一层里新的在前（`runs` 已经是新的在前），层按口播、图里的字、画面上的字排。
    for layer in [TextLayer::Asr, TextLayer::Ocr, TextLayer::Frames] {
        out.extend(machine.iter().filter(|t| t.layer == layer).cloned());
    }
    let keyframes = match runs.iter().find(|r| r.job == "keyframes") {
        Some(r) => match keyframe_set(lib, id, &dir, r) {
            Ok(k) => k,
            Err(e) => {
                tracing::warn!(item = %id, run = %r.id, error = %e, "keyframes not readable; skipped");
                None
            }
        },
        None => None,
    };
    Ok((out, keyframes))
}

fn transcript(
    lib: &Library,
    id: &ItemId,
    dir: &Path,
    r: &RunRecord,
    layer: TextLayer,
) -> Result<Transcript> {
    let snapshot = r
        .inputs
        .first()
        .map(|i| i.snapshot.clone())
        .unwrap_or_default();
    let kind = if layer == TextLayer::Asr {
        "asr"
    } else {
        "ocr"
    };
    let pieces = if r.empty {
        Vec::new()
    } else {
        let path = lib.run_output(id, &r.id, kind)?;
        let bytes = std::fs::read(&path).map_err(|e| Error::io(&path, e))?;
        match layer {
            TextLayer::Asr => timed(&snapshot, &String::from_utf8_lossy(&bytes)),
            TextLayer::Ocr => image_lines(&serde_json::from_slice(&bytes)?),
            TextLayer::Frames => frame_lines(dir, r, &serde_json::from_slice(&bytes)?),
        }
    };
    let lang = r
        .outputs
        .iter()
        .find(|o| o.kind == kind)
        .and_then(|o| o.lang.clone());
    Ok(Transcript {
        layer,
        source: r.id.clone(),
        snapshot,
        machine: Some(Machine {
            run_id: r.id.clone(),
            at: r.at.clone(),
            tool: r
                .model
                .as_ref()
                .map(|m| m.name.clone())
                .or_else(|| r.tool.as_ref().map(|t| t.name.clone())),
            plugin: (r.by != "host").then(|| r.by.clone()),
        }),
        lang,
        pieces,
        remark: run_remark(r),
    })
}

fn timed(snapshot: &str, vtt: &str) -> Vec<Piece> {
    vtt_cues(vtt)
        .into_iter()
        .map(|(sec, end, text)| Piece {
            text,
            spot: Spot::Time {
                sec,
                end: Some(end),
                snapshot: snapshot.to_owned(),
                xywh: None,
                frame: None,
            },
        })
        .collect()
}

/// 图文的图和封面：每一行一段，带 `imageKey` 和框。没有 `imageKey` 的图找不回原样，不出段。
fn image_lines(ocr: &Ocr) -> Vec<Piece> {
    let mut out = Vec::new();
    for img in &ocr.images {
        let Some(key) = &img.image_key else { continue };
        for l in &img.lines {
            if l.text.trim().is_empty() {
                continue;
            }
            out.push(Piece {
                text: l.text.clone(),
                spot: Spot::Image {
                    key: key.clone(),
                    xywh: l.bbox,
                },
            });
        }
    }
    out
}

/// 关键帧上的字：按帧的先后，连着几帧一模一样的一行合成一段，`end` 写到最后出现的那一帧。
/// 这是压缩，不是挑：每一行都在。
fn frame_lines(dir: &Path, r: &RunRecord, ocr: &Ocr) -> Vec<Piece> {
    let mut frames: Vec<_> = ocr
        .images
        .iter()
        .filter_map(|img| Some((img.frame_sec?, img)))
        .collect();
    frames.sort_by(|a, b| a.0.total_cmp(&b.0));
    let mut out: Vec<Piece> = Vec::new();
    // 一行字 → 上一帧里它那一段的下标。
    let mut open: HashMap<String, usize> = HashMap::new();
    for (sec, img) in frames {
        let input = r.inputs.get(img.input as usize);
        let snapshot = input.map(|i| i.snapshot.clone()).unwrap_or_default();
        let frame = input.and_then(|i| frame_layer(dir, &i.path, sec));
        let mut now: HashMap<String, usize> = HashMap::new();
        for l in &img.lines {
            if l.text.trim().is_empty() || now.contains_key(&l.text) {
                continue;
            }
            let at = match open.get(&l.text) {
                Some(&i) => {
                    if let Spot::Time { end, .. } = &mut out[i].spot {
                        *end = Some(sec);
                    }
                    i
                }
                None => {
                    out.push(Piece {
                        text: l.text.clone(),
                        spot: Spot::Time {
                            sec,
                            end: None,
                            snapshot: snapshot.clone(),
                            xywh: Some(l.bbox),
                            frame: frame.clone(),
                        },
                    });
                    out.len() - 1
                }
            };
            now.insert(l.text.clone(), at);
        }
        open = now;
    }
    out
}

/// `derived/runs/<runId>/keyframes/<文件>` 这一帧还在时，它的媒体层 `frame/<runId>/<毫秒>`。
fn frame_layer(dir: &Path, input: &str, sec: f64) -> Option<String> {
    let run = input
        .strip_prefix("derived/runs/")?
        .split('/')
        .next()
        .filter(|r| usuyo_library::is_uuid7(r))?;
    dir.join(input)
        .is_file()
        .then(|| format!("frame/{run}/{}", (sec * 1000.0).round() as u64))
}

fn keyframe_set(
    lib: &Library,
    id: &ItemId,
    dir: &Path,
    r: &RunRecord,
) -> Result<Option<KeyframeSet>> {
    let still = r.marks.iter().any(|m| m.code == "static-picture");
    let path = match lib.run_output(id, &r.id, "keyframes") {
        Ok(p) => p,
        Err(usuyo_library::Error::NotFound(_)) => return Ok(None),
        Err(e) => return Err(e.into()),
    };
    let bytes = std::fs::read(&path).map_err(|e| Error::io(&path, e))?;
    let kf: Keyframes = serde_json::from_slice(&bytes)?;
    if let Some(bad) = kf.bad_path() {
        return Err(Error::InvalidArgs(format!("keyframe path `{bad}`")));
    }
    let base = dir.join("derived/runs").join(&r.id);
    Ok(Some(KeyframeSet {
        run_id: r.id.clone(),
        secs: kf
            .frames
            .iter()
            .filter(|f| base.join(&f.path).is_file())
            .map(|f| f.sec)
            .collect(),
        still,
    }))
}

/// WebVTT 的 cue：`(起, 止, 字)`，秒。头、`NOTE`、`STYLE`、`REGION` 块和时间写坏了的 cue 跳过；
/// 字里的标签（`<c>`、`<00:01.000>`、`<v 名字>`）去掉，几个常见的字符引用换回来。
pub(crate) fn vtt_cues(text: &str) -> Vec<(f64, f64, String)> {
    let text = text.replace("\r\n", "\n").replace('\r', "\n");
    let mut out = Vec::new();
    for block in text.split("\n\n") {
        let lines: Vec<&str> = block.lines().collect();
        let Some(at) = lines.iter().position(|l| l.contains("-->")) else {
            continue;
        };
        let Some((start, end)) = cue_times(lines[at]) else {
            continue;
        };
        let body = lines[at + 1..]
            .iter()
            .map(|l| strip_tags(l))
            .map(|l| l.trim().to_owned())
            .filter(|l| !l.is_empty())
            .collect::<Vec<_>>()
            .join("\n");
        if !body.is_empty() {
            out.push((start, end.max(start), body));
        }
    }
    out
}

fn cue_times(line: &str) -> Option<(f64, f64)> {
    let (a, b) = line.split_once("-->")?;
    let start = timestamp(a.trim())?;
    let end = timestamp(b.split_whitespace().next()?)?;
    Some((start, end))
}

/// `hh:mm:ss.ttt` 或 `mm:ss.ttt`。
fn timestamp(s: &str) -> Option<f64> {
    let parts: Vec<&str> = s.split(':').collect();
    let (h, m, sec) = match parts.as_slice() {
        [h, m, s] => (h.parse::<u64>().ok()?, m.parse::<u64>().ok()?, *s),
        [m, s] => (0, m.parse::<u64>().ok()?, *s),
        _ => return None,
    };
    let sec: f64 = sec.parse().ok()?;
    if !sec.is_finite() || sec < 0.0 || m >= 60 {
        return None;
    }
    Some((h * 3600 + m * 60) as f64 + sec)
}

fn strip_tags(line: &str) -> String {
    let mut out = String::with_capacity(line.len());
    let mut in_tag = false;
    for c in line.chars() {
        match c {
            '<' => in_tag = true,
            '>' if in_tag => in_tag = false,
            _ if !in_tag => out.push(c),
            _ => {}
        }
    }
    out.replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&nbsp;", " ")
        .replace("&amp;", "&")
}

#[cfg(test)]
mod tests {
    use super::*;
    use usuyo_contract::{OcrImage, OcrLine};

    #[test]
    fn reads_vtt_cues() {
        let vtt = "WEBVTT\nKind: captions\n\nNOTE zh\n\n1\n00:00:01.000 --> 00:00:03.500 align:start\n<v 主播>大家好，</v>\n今天讲 <c.x>咖啡</c> &amp; 茶\n\n01:02.5 --> 01:04.000\n水温九十二度\n\nbad --> 00:01.000\nx\n\n00:05.000 --> 00:06.000\n\n";
        assert_eq!(
            vtt_cues(vtt),
            [
                (1.0, 3.5, "大家好，\n今天讲 咖啡 & 茶".to_owned()),
                (62.5, 64.0, "水温九十二度".to_owned())
            ]
        );
        assert!(vtt_cues("WEBVTT\r\n").is_empty());
    }

    fn line(text: &str) -> OcrLine {
        OcrLine {
            text: text.into(),
            bbox: [0.1, 0.2, 0.3, 0.04],
            confidence: 1.0,
            extra: Default::default(),
        }
    }

    fn frame(input: u32, sec: f64, lines: &[&str]) -> OcrImage {
        OcrImage {
            input,
            image_key: None,
            frame_sec: Some(sec),
            width: 960,
            height: 540,
            lines: lines.iter().map(|t| line(t)).collect(),
            extra: Default::default(),
        }
    }

    #[test]
    fn same_line_on_consecutive_frames_is_one_piece() {
        let inputs: Vec<serde_json::Value> = (0..3)
            .map(|i| {
                serde_json::json!({
                    "path": format!("derived/runs/019a6f3e-0000-7b2a-9e44-5d0c1a2b3c4d/keyframes/{i}.jpg"),
                    "sha256": "s", "snapshot": "snap"})
            })
            .collect();
        let run: RunRecord = serde_json::from_value(serde_json::json!({
            "id": "019a6f3e-8c1d-7b2a-9e44-5d0c1a2b3c4d", "job": "frames", "by": "x", "version": "1",
            "tool": null, "model": null, "at": "2026-11-12T00:00:00Z", "device": "d", "trigger": "auto",
            "redoOf": null, "reason": null, "inputs": inputs
        }))
        .unwrap();
        let ocr = Ocr {
            images: vec![
                frame(2, 12.0, &["标题", "第三帧才有"]),
                frame(0, 0.5, &["标题", "只在第一帧"]),
                frame(1, 4.0, &["标题"]),
            ],
            extra: Default::default(),
        };
        let dir = tempfile::tempdir().unwrap();
        let pieces = frame_lines(dir.path(), &run, &ocr);
        let got: Vec<(&str, f64, Option<f64>)> = pieces
            .iter()
            .map(|p| match &p.spot {
                Spot::Time { sec, end, .. } => (p.text.as_str(), *sec, *end),
                Spot::Image { .. } => unreachable!(),
            })
            .collect();
        assert_eq!(
            got,
            [
                ("标题", 0.5, Some(12.0)),
                ("只在第一帧", 0.5, None),
                ("第三帧才有", 12.0, None)
            ]
        );
        // 帧文件不在了：不给帧的地址
        assert!(matches!(&pieces[0].spot, Spot::Time { frame: None, .. }));
    }
}
