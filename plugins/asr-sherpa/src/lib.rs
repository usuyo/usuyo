//! `builtin/asr-sherpa`：16 kHz 单声道 WAV 进，带时间点的 `asr.vtt` 出（feature 003 §4.1、§5.3）。
//!
//! - [`plugin`]：转换协议的 `run`、`health`，交给 plugin-kit 的循环。
//! - [`engine`]：silero VAD 切段、sherpa-onnx 跑 SenseVoice；`examples/bench.rs` 直接用它量速度。
//! - [`cues`]：cue 切分、去标签、猜语言、写 WebVTT。
//! - [`options`]、[`wav`]：参数和输入格式。

pub mod cues;
pub mod engine;
pub mod options;
pub mod plugin;
pub mod wav;
