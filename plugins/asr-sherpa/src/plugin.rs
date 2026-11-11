//! `builtin/asr-sherpa` 的 `run` 和 `health`（协议 §4.2）；JSON-RPC 循环在 plugin-kit。
//!
//! 一次 `run`：查输入和参数 → 读 WAV 头 → 找模型（不在回 `dependency-missing`）→ 识别器还在就接着用，
//! 参数变了才重建 → 转写，每段之间查取消、发 `progress` → 人声太少回 `empty` / `no-speech`，
//! 否则写 `asr.vtt`。错误在这里统一换成协议的错误对象。

use std::path::PathBuf;
use std::sync::{Mutex, PoisonError};

use sherpa_onnx::OfflineRecognizer;
use usuyo_plugin_kit::protocol::{
    CallFailure, ConsumeKind, Describe, EmptyReason, ErrorCode, Health, HealthCheck, Mark,
    MarkCode, ModelUsed, Need, Produce, RelPath, RunParams, RunProduct, RunResult, Seconds,
    UpstreamVersion,
};
use usuyo_plugin_kit::semver;
use usuyo_plugin_kit::transform::{RunContext, Transform};

use crate::cues::{guess_language, is_no_speech, write_vtt};
use crate::engine::{
    EngineError, MODEL_NAME, ModelFiles, RecognizerKey, VadModel, load_recognizer, transcribe,
};
use crate::options::Options;
use crate::wav::Pcm16k;

/// 产物的文件名，相对 `out`。
const OUTPUT_FILE: &str = "asr.vtt";

/// 这个插件：核过的 VAD、模型根目录，和上一次用过的识别器。
pub struct AsrSherpa {
    vad: VadModel,
    models_dir: PathBuf,
    /// 同一进程里的几次 `run` 共用；`run` 一次只有一个，锁只是为了 `Sync`。
    loaded: Mutex<Option<(RecognizerKey, OfflineRecognizer)>>,
}

impl AsrSherpa {
    pub fn new(vad: VadModel, models_dir: PathBuf) -> Self {
        Self {
            vad,
            models_dir,
            loaded: Mutex::new(None),
        }
    }

    /// `describe` 的回答：`models` 是现在就在的模型。
    pub fn describe(&self, version: semver::Version) -> Describe {
        let present = ModelFiles::locate(&self.models_dir).is_ok();
        Describe::new(
            version,
            present.then(|| MODEL_NAME.to_owned()).into_iter().collect(),
        )
    }

    fn transcribe_one(
        &self,
        params: RunParams,
        ctx: &RunContext,
    ) -> Result<RunResult, CallFailure> {
        if params.want != [Produce::Asr] {
            return Err(failure(
                ErrorCode::InvalidArgs,
                "this plugin only produces `asr`",
            ));
        }
        let [input] = params.inputs.as_slice() else {
            return Err(failure(
                ErrorCode::InvalidArgs,
                format!("expected one input, got {}", params.inputs.len()),
            ));
        };
        if input.kind != ConsumeKind::AudioPcm16k {
            return Err(failure(
                ErrorCode::InvalidArgs,
                "the input must be `audio:pcm16k`",
            ));
        }
        let options = Options::from_map(&params.options)
            .map_err(|e| failure(ErrorCode::InvalidArgs, e.to_string()))?;
        let mut audio = Pcm16k::open(&input.path).map_err(|e| {
            let code = if e.is_io() {
                ErrorCode::Internal
            } else {
                ErrorCode::UnsupportedInput
            };
            failure(code, format!("{}: {e}", input.path.display()))
        })?;
        let duration = audio.duration_sec();
        let model = match ModelFiles::locate(&self.models_dir) {
            Ok(model) => model,
            Err(e) => {
                let mut f = failure(ErrorCode::DependencyMissing, e.to_string());
                f.needs = vec![
                    Need::try_from(format!("model:{MODEL_NAME}"))
                        .map_err(|e| failure(ErrorCode::Internal, e))?,
                ];
                return Err(f);
            }
        };
        ctx.progress(0.0, duration, "sec");

        let key = RecognizerKey::new(model, &options);
        let mut loaded = self.loaded.lock().unwrap_or_else(PoisonError::into_inner);
        let recognizer = match &mut *loaded {
            Some((k, r)) if *k == key => &*r,
            slot => {
                // 先放掉旧的，免得两份模型同时占着内存。
                *slot = None;
                let r = load_recognizer(&key).map_err(engine_failure)?;
                &slot.insert((key, r)).1
            }
        };
        ctx.check_cancelled()?;

        let transcript = transcribe(recognizer, &self.vad, &mut audio, &options, |at| {
            ctx.progress(at, duration, "sec");
            !ctx.is_cancelled()
        })
        .map_err(engine_failure)?;
        ctx.check_cancelled()?;

        let mut result = RunResult {
            outputs: Vec::new(),
            tool: UpstreamVersion {
                name: "sherpa-onnx".into(),
                version: sherpa_onnx::version().into(),
            },
            options: options.to_map(),
            model: Some(ModelUsed {
                name: MODEL_NAME.into(),
            }),
            marks: Vec::new(),
            empty: false,
            reason: None,
            probe: Vec::new(),
        };
        if is_no_speech(transcript.speech_sec, transcript.duration_sec) {
            result.empty = true;
            result.reason = Some(EmptyReason::NoSpeech);
            result.marks.push(Mark {
                code: MarkCode::NoSpeech,
                speech_sec: Some(seconds(transcript.speech_sec)?),
                duration_sec: Some(seconds(transcript.duration_sec)?),
                frozen_sec: None,
            });
            return Ok(result);
        }
        let lang = options
            .language
            .fixed()
            .unwrap_or_else(|| guess_language(&transcript.text));
        std::fs::write(
            params.out.join(OUTPUT_FILE),
            write_vtt(lang, &transcript.cues),
        )
        .map_err(|e| failure(ErrorCode::Internal, format!("write {OUTPUT_FILE}: {e}")))?;
        result.outputs.push(RunProduct {
            kind: Produce::Asr,
            path: RelPath::try_from(OUTPUT_FILE).map_err(|e| failure(ErrorCode::Internal, e))?,
            lang: Some(lang.into()),
        });
        Ok(result)
    }
}

impl Transform for AsrSherpa {
    fn run(&self, params: RunParams, ctx: &RunContext) -> Result<RunResult, CallFailure> {
        self.transcribe_one(params, ctx)
    }

    fn health(&self) -> Health {
        let model = ModelFiles::locate(&self.models_dir);
        Health {
            ready: model.is_ok(),
            checks: vec![HealthCheck {
                name: format!("model:{MODEL_NAME}"),
                ok: model.is_ok(),
                message: model.err().map(|e| e.to_string()),
            }],
        }
    }
}

fn failure(code: ErrorCode, message: impl Into<String>) -> CallFailure {
    CallFailure::new(code, message)
}

fn engine_failure(e: EngineError) -> CallFailure {
    match e {
        EngineError::Stopped => failure(ErrorCode::Cancelled, "cancelled"),
        other => failure(ErrorCode::Internal, other.to_string()),
    }
}

/// 秒数最多 3 位小数（SPEC §0.5）。
fn seconds(v: f64) -> Result<Seconds, CallFailure> {
    Seconds::try_from((v * 1000.0).round() / 1000.0).map_err(|e| failure(ErrorCode::Internal, e))
}
