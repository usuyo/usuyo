//! 测试用的假转换插件，用 plugin-kit 的循环：`tests/transforms.rs` 把它装成三个插件，第一个参数说是哪个。
//!
//! 输入文件的内容就是它「有什么」：`v:7.5` 是 7.5 秒、只有画面；`a:7.4` 只有声音；`va:5` 都有。
//!
//! - `media`：`probe` 照内容报流；`play` 把两个输入接起来；`audio:pcm16k` 没有声音时回 `no-audio-track`；
//!   `keyframes` 出一帧和 `keyframes.json`。`options.fail` 为真时回 `internal`。
//! - `asr`：`${MODELS}/fake-voice/model.json` 不在就回 `dependency-missing`（`model:fake-voice`）；
//!   在就写一条含「咖啡」的 cue。
//! - `ocr`：每个输入一张图一行字。

use std::path::Path;
use std::process::ExitCode;

use serde_json::{Value, json};
use usuyo_plugin_kit::protocol::{
    CallFailure, Describe, EmptyReason, ErrorCode, MediaKind, ModelUsed, Need, Probe, ProbeStream,
    Produce, RelPath, RunParams, RunProduct, RunResult, Seconds, UpstreamVersion,
};
use usuyo_plugin_kit::transform::{self, RunContext};

fn fail(code: ErrorCode, msg: impl Into<String>) -> CallFailure {
    CallFailure::new(code, msg)
}

fn internal(e: impl std::fmt::Display) -> CallFailure {
    fail(ErrorCode::Internal, e.to_string())
}

fn result(tool: &str, options: serde_json::Map<String, Value>) -> RunResult {
    RunResult {
        outputs: Vec::new(),
        tool: UpstreamVersion {
            name: tool.into(),
            version: "1".into(),
        },
        options,
        model: None,
        marks: Vec::new(),
        empty: false,
        reason: None,
        probe: Vec::new(),
    }
}

fn product(kind: Produce, path: &str) -> Result<RunProduct, CallFailure> {
    Ok(RunProduct {
        kind,
        path: RelPath::try_from(path).map_err(internal)?,
        lang: None,
    })
}

/// 内容 → (有画面, 有声音, 秒数)。
fn what(path: &Path) -> Result<(bool, bool, f64), CallFailure> {
    let text = std::fs::read_to_string(path).map_err(internal)?;
    let (kinds, sec) = text.trim().split_once(':').unwrap_or(("v", "5"));
    Ok((
        kinds.contains('v'),
        kinds.contains('a'),
        sec.parse().unwrap_or(5.0),
    ))
}

fn sha256(bytes: &[u8]) -> String {
    use sha2::Digest;
    sha2::Sha256::digest(bytes)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

fn media(p: RunParams) -> Result<RunResult, CallFailure> {
    if p.options.get("fail").and_then(Value::as_bool) == Some(true) {
        return Err(fail(ErrorCode::Internal, "asked to fail"));
    }
    let want = p.want[0];
    let mut r = result("fake-ffmpeg", p.options.clone());
    match want {
        Produce::Probe => {
            for (i, input) in p.inputs.iter().enumerate() {
                let (v, a, sec) = what(&input.path)?;
                let mut streams = Vec::new();
                if v {
                    streams.push(ProbeStream {
                        kind: MediaKind::Video,
                        codec: Some("h264".into()),
                    });
                }
                if a {
                    streams.push(ProbeStream {
                        kind: MediaKind::Audio,
                        codec: Some("aac".into()),
                    });
                }
                r.probe.push(Probe {
                    input: i as u32,
                    duration_sec: Some(Seconds::try_from(sec).map_err(internal)?),
                    streams,
                });
            }
        }
        Produce::Play => {
            let mut bytes = Vec::new();
            for input in &p.inputs {
                bytes.extend(std::fs::read(&input.path).map_err(internal)?);
            }
            std::fs::write(p.out.join("play.mp4"), bytes).map_err(internal)?;
            r.outputs.push(product(Produce::Play, "play.mp4")?);
        }
        Produce::AudioPcm16k => {
            let input = &p.inputs[0];
            let (_, a, _) = what(&input.path)?;
            if a {
                let bytes = std::fs::read(&input.path).map_err(internal)?;
                std::fs::write(
                    p.out.join("audio.wav"),
                    [b"pcm:".as_slice(), &bytes].concat(),
                )
                .map_err(internal)?;
                r.outputs.push(product(Produce::AudioPcm16k, "audio.wav")?);
            } else {
                r.empty = true;
                r.reason = Some(EmptyReason::NoAudioTrack);
            }
        }
        Produce::Keyframes => {
            let input = &p.inputs[0];
            let (_, _, sec) = what(&input.path)?;
            std::fs::create_dir_all(p.out.join("keyframes")).map_err(internal)?;
            let frame = format!("frame of {}", input.path.display());
            std::fs::write(p.out.join("keyframes/00000500.jpg"), &frame).map_err(internal)?;
            let index = json!({
                "snapshot": input.snapshot,
                "durationSec": sec,
                "frames": [{"sec": 0.5, "path": "keyframes/00000500.jpg", "sha256": sha256(frame.as_bytes()),
                            "width": 8, "height": 8, "scene": null}],
                "frozenSec": 0.0
            });
            std::fs::write(p.out.join("keyframes.json"), index.to_string()).map_err(internal)?;
            r.outputs
                .push(product(Produce::Keyframes, "keyframes.json")?);
        }
        other => return Err(fail(ErrorCode::InvalidArgs, format!("cannot {other}"))),
    }
    Ok(r)
}

fn asr(p: RunParams) -> Result<RunResult, CallFailure> {
    let models = std::env::var_os("USUYO_MODELS_DIR").ok_or_else(|| internal("no models dir"))?;
    if !Path::new(&models).join("fake-voice/model.json").is_file() {
        let mut f = fail(ErrorCode::DependencyMissing, "model fake-voice is missing");
        f.needs
            .push(Need::try_from("model:fake-voice".to_owned()).map_err(internal)?);
        return Err(f);
    }
    let input = std::fs::read_to_string(&p.inputs[0].path).map_err(internal)?;
    let vtt = format!(
        "WEBVTT\n\n00:00.000 --> 00:01.000\n手冲咖啡 {}\n",
        input.trim()
    );
    std::fs::write(p.out.join("asr.vtt"), vtt).map_err(internal)?;
    let mut r = result("fake-sherpa", p.options.clone());
    r.model = Some(ModelUsed {
        name: "fake-voice".into(),
    });
    r.outputs.push(RunProduct {
        lang: Some("zh".into()),
        ..product(Produce::Asr, "asr.vtt")?
    });
    Ok(r)
}

fn ocr(p: RunParams) -> Result<RunResult, CallFailure> {
    let images: Vec<Value> = p
        .inputs
        .iter()
        .enumerate()
        .map(|(i, input)| {
            let text = std::fs::read_to_string(&input.path).unwrap_or_default();
            json!({"input": i, "imageKey": input.image_key, "frameSec": input.frame_sec,
                   "width": 8, "height": 8,
                   "lines": [{"text": format!("粉水比 {}", text.trim()), "bbox": [0.1, 0.1, 0.5, 0.1], "confidence": 1.0}]})
        })
        .collect();
    std::fs::write(
        p.out.join("ocr.json"),
        json!({"images": images}).to_string(),
    )
    .map_err(internal)?;
    let mut r = result("fake-vision", p.options.clone());
    r.outputs.push(product(Produce::Ocr, "ocr.json")?);
    Ok(r)
}

fn main() -> ExitCode {
    let role = std::env::args().nth(1).unwrap_or_default();
    let run = move |p: RunParams, ctx: &RunContext| -> Result<RunResult, CallFailure> {
        ctx.check_cancelled()?;
        match role.as_str() {
            "media" => media(p),
            "asr" => asr(p),
            "ocr" => ocr(p),
            other => Err(fail(ErrorCode::Internal, format!("unknown role {other}"))),
        }
    };
    let Ok(version) = "1.0.0".parse() else {
        return ExitCode::FAILURE;
    };
    match transform::serve(Describe::new(version, Vec::new()), run) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("{e}");
            ExitCode::FAILURE
        }
    }
}
