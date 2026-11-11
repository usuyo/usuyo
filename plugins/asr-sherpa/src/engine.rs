//! 干活的那部分：找模型文件、核 VAD、建识别器、按 VAD 段转写。插件（`plugin`）和量速度的
//! `examples/bench.rs` 都用它。

use std::io;
use std::path::{Path, PathBuf};

use sha2::{Digest, Sha256};
use sherpa_onnx::{
    OfflineRecognizer, OfflineRecognizerConfig, OfflineSenseVoiceModelConfig, SileroVadModelConfig,
    VadModelConfig, VoiceActivityDetector,
};

use crate::cues::{Cue, Recognized, segment_cues};
use crate::options::Options;
use crate::wav::{Pcm16k, SAMPLE_RATE};

/// 清单 `models[].name`，也是 `${MODELS}` 下的目录名。
pub const MODEL_NAME: &str = "sense-voice-int8-2024-07-17";
/// 压缩包解开后的顶层目录。插件要宿主把压缩包原样解进 `${MODELS}/<name>/`、不去掉这一层（宿主的下载还没做，协议也没写明）；
/// 压缩包由 sha256 锁定，这一层不会变。
const MODEL_ARCHIVE_DIR: &str = "sherpa-onnx-sense-voice-zh-en-ja-ko-yue-int8-2024-07-17";
/// 要用的两个文件和它们的字节数；大小对不上算没下完。
const MODEL_FILES: [(&str, u64); 2] = [("model.int8.onnx", 239_233_841), ("tokens.txt", 315_894)];

/// 资产里的 VAD 文件名，在 `${PLUGIN_DIR}` 下。
pub const VAD_FILE: &str = "silero_vad_v5.onnx";
/// `silero_vad_v5.onnx`（2,313,101 字节）的 sha256。`silero_vad.onnx` 是另一个模型。
pub const VAD_SHA256: &str = "6b99cbfd39246b6706f98ec13c7c50c6b299181f2474fa05cbc8046acc274396";

/// VAD 每次喂多少个采样（silero 在 16 kHz 下的窗口）。
const VAD_WINDOW: usize = 512;
/// 一次从文件读多少个采样：一秒。
const READ_BLOCK: usize = SAMPLE_RATE as usize;
/// 没有段落下来时，每扫过这么多秒的音频也报一次。
const TICK_EVERY_SEC: f64 = 30.0;

#[derive(Debug, thiserror::Error)]
pub enum EngineError {
    #[error("read the audio: {0}")]
    Read(#[from] io::Error),
    #[error("sherpa-onnx could not create the {0}")]
    Create(&'static str),
    #[error("sherpa-onnx returned no result for a segment")]
    NoResult,
    #[error("stopped")]
    Stopped,
}

/// 模型不在或不全。
#[derive(Debug, thiserror::Error)]
#[error("model `{MODEL_NAME}` is not in {dir}: {why}")]
pub struct ModelMissing {
    dir: PathBuf,
    why: String,
}

/// 下好的 SenseVoice。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelFiles {
    model: String,
    tokens: String,
}

impl ModelFiles {
    /// 在模型根目录（`${MODELS}`）下找这个模型；不在、大小不对都算缺。
    pub fn locate(models_dir: &Path) -> Result<Self, ModelMissing> {
        let dir = models_dir.join(MODEL_NAME).join(MODEL_ARCHIVE_DIR);
        let [model, tokens] = MODEL_FILES;
        Ok(Self {
            model: model_file(&dir, model)?,
            tokens: model_file(&dir, tokens)?,
        })
    }
}

/// 一个模型文件在、大小对，回它的路径。
fn model_file(dir: &Path, (name, size): (&str, u64)) -> Result<String, ModelMissing> {
    let missing = |why: String| ModelMissing {
        dir: dir.to_owned(),
        why,
    };
    let path = dir.join(name);
    let len = path
        .metadata()
        .map_err(|e| missing(format!("{name}: {e}")))?
        .len();
    if len != size {
        return Err(missing(format!("{name} is {len} bytes, expected {size}")));
    }
    path.into_os_string()
        .into_string()
        .map_err(|_| missing(format!("{name}: the path is not UTF-8")))
}

/// VAD 资产不在或被改过。
#[derive(Debug, thiserror::Error)]
pub enum VadError {
    #[error("read {path}: {source}")]
    Read { path: PathBuf, source: io::Error },
    #[error("{path} has sha256 {actual}, expected {VAD_SHA256}")]
    Mismatch { path: PathBuf, actual: String },
    #[error("path is not UTF-8: {0}")]
    Path(PathBuf),
}

/// 核过 sha256 的 VAD 模型路径。
#[derive(Debug, Clone)]
pub struct VadModel(String);

impl VadModel {
    pub fn verify(path: &Path) -> Result<Self, VadError> {
        let read = |source| VadError::Read {
            path: path.to_owned(),
            source,
        };
        let bytes = std::fs::read(path).map_err(read)?;
        let actual: String = Sha256::digest(&bytes)
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect();
        if actual != VAD_SHA256 {
            return Err(VadError::Mismatch {
                path: path.to_owned(),
                actual,
            });
        }
        path.to_str()
            .map(|p| Self(p.to_owned()))
            .ok_or_else(|| VadError::Path(path.to_owned()))
    }
}

/// 建识别器时定下、之后改不了的那几个参数。参数变了要重建。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecognizerKey {
    model: ModelFiles,
    language: &'static str,
    itn: bool,
    threads: u32,
}

impl RecognizerKey {
    pub fn new(model: ModelFiles, options: &Options) -> Self {
        Self {
            model,
            language: options.language.as_str(),
            itn: options.itn,
            threads: options.threads,
        }
    }
}

/// 加载 SenseVoice；约 1 秒。
pub fn load_recognizer(key: &RecognizerKey) -> Result<OfflineRecognizer, EngineError> {
    let mut config = OfflineRecognizerConfig::default();
    config.model_config.sense_voice = OfflineSenseVoiceModelConfig {
        model: Some(key.model.model.clone()),
        language: Some(key.language.to_owned()),
        use_itn: key.itn,
    };
    config.model_config.tokens = Some(key.model.tokens.clone());
    // 不给就是 1；这台 2 个比 1 个快将近一倍，再多反而慢。
    config.model_config.num_threads = key.threads as i32;
    OfflineRecognizer::create(&config).ok_or(EngineError::Create("SenseVoice recognizer"))
}

/// 一次转写的结果。
#[derive(Debug)]
pub struct Transcript {
    pub duration_sec: f64,
    pub speech_sec: f64,
    pub segments: usize,
    pub cues: Vec<Cue>,
    /// 所有段的字连起来，猜语言用。
    pub text: String,
}

/// 读完整段音频：VAD 切段，每段落下来就转，转完就丢，内存不随时长涨。
/// 每转完一段、每扫过 30 秒音频调一次 `tick(扫到第几秒)`；它回 `false` 就停，回 [`EngineError::Stopped`]。
pub fn transcribe(
    recognizer: &OfflineRecognizer,
    vad_model: &VadModel,
    audio: &mut Pcm16k,
    options: &Options,
    mut tick: impl FnMut(f64) -> bool,
) -> Result<Transcript, EngineError> {
    let vad_config = VadModelConfig {
        silero_vad: SileroVadModelConfig {
            model: Some(vad_model.0.clone()),
            threshold: options.vad_threshold as f32,
            min_silence_duration: 0.5,
            min_speech_duration: 0.25,
            window_size: VAD_WINDOW as i32,
            max_speech_duration: 30.0,
        },
        sample_rate: SAMPLE_RATE as i32,
        num_threads: 1,
        ..Default::default()
    };
    // 每次新建：阈值每次可能不同，上一条的状态也不该带过来。
    let vad = VoiceActivityDetector::create(&vad_config, 60.0).ok_or(EngineError::Create("VAD"))?;
    let rate = f64::from(SAMPLE_RATE);
    let mut t = Transcript {
        duration_sec: audio.duration_sec(),
        speech_sec: 0.0,
        segments: 0,
        cues: Vec::new(),
        text: String::new(),
    };
    let mut fed = 0usize;
    let mut next_tick = TICK_EVERY_SEC;
    let mut block = Vec::with_capacity(READ_BLOCK);
    loop {
        audio.read_block(&mut block, READ_BLOCK)?;
        if block.is_empty() {
            vad.flush();
            drain(recognizer, &vad, options, &mut t, &mut tick)?;
            return Ok(t);
        }
        for window in block.chunks(VAD_WINDOW) {
            vad.accept_waveform(window);
            drain(recognizer, &vad, options, &mut t, &mut tick)?;
        }
        fed += block.len();
        let at = fed as f64 / rate;
        if at >= next_tick {
            next_tick = at + TICK_EVERY_SEC;
            if !tick(at) {
                return Err(EngineError::Stopped);
            }
        }
    }
}

/// 把 VAD 已经切好的段都转掉。
fn drain(
    recognizer: &OfflineRecognizer,
    vad: &VoiceActivityDetector,
    options: &Options,
    t: &mut Transcript,
    tick: &mut impl FnMut(f64) -> bool,
) -> Result<(), EngineError> {
    let rate = f64::from(SAMPLE_RATE);
    while let Some(seg) = vad.front() {
        let start = seg.start() as f64 / rate;
        let samples = seg.samples();
        let dur = samples.len() as f64 / rate;
        let stream = recognizer.create_stream();
        stream.accept_waveform(SAMPLE_RATE as i32, samples);
        recognizer.decode(&stream);
        let result = stream.get_result().ok_or(EngineError::NoResult)?;
        vad.pop();
        t.speech_sec += dur;
        t.segments += 1;
        t.text.push_str(&result.text);
        t.cues.extend(segment_cues(
            start,
            dur,
            &Recognized {
                text: &result.text,
                tokens: &result.tokens,
                timestamps: result.timestamps.as_deref(),
            },
            options.max_cue_sec,
        ));
        if !tick(start + dur) {
            return Err(EngineError::Stopped);
        }
    }
    Ok(())
}
