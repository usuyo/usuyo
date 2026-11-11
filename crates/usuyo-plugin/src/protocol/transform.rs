//! 转换插件的 JSON-RPC 消息（协议 §4.1、§4.2）：stdio 上一行一条，JSON-RPC 2.0。
//!
//! 宿主 → 插件：`describe`、`health`、`run`，通知 `cancel`；插件 → 宿主：通知 `progress`。
//! 插件一侧的循环在 plugin-kit 的 `transform` 模块；宿主一侧在 `host`（还没做）。

use std::collections::BTreeSet;
use std::fmt;
use std::path::PathBuf;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use super::envelope::CallFailure;
use super::fetch::{MediaKind, UpstreamVersion};
use super::manifest::{ConsumeKind, Produce};
use super::values::{RelPath, Seconds};

/// 方法名。
pub const METHOD_DESCRIBE: &str = "describe";
pub const METHOD_HEALTH: &str = "health";
pub const METHOD_RUN: &str = "run";
pub const METHOD_CANCEL: &str = "cancel";
pub const METHOD_PROGRESS: &str = "progress";

/// JSON-RPC 请求的 `id`：数字或字符串，原样回。
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(untagged)]
pub enum RpcId {
    Number(i64),
    Text(String),
}

impl fmt::Display for RpcId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RpcId::Number(n) => write!(f, "{n}"),
            RpcId::Text(t) => write!(f, "{t:?}"),
        }
    }
}

/// JSON-RPC 的 `error`。协议错误用标准码；插件自己的失败用 [`RpcError::PLUGIN_FAILURE`]，
/// `data` 是错误对象（协议 §3.3）。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct RpcError {
    pub code: i64,
    pub message: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub data: Option<Box<CallFailure>>,
}

impl RpcError {
    pub const PARSE_ERROR: i64 = -32700;
    pub const INVALID_REQUEST: i64 = -32600;
    pub const METHOD_NOT_FOUND: i64 = -32601;
    pub const INVALID_PARAMS: i64 = -32602;
    pub const PLUGIN_FAILURE: i64 = -32000;

    pub fn protocol(code: i64, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            data: None,
        }
    }

    pub fn failure(failure: CallFailure) -> Self {
        Self {
            code: Self::PLUGIN_FAILURE,
            message: failure.code.as_str().to_owned(),
            data: Some(Box::new(failure)),
        }
    }
}

/// `describe` 的结果。宿主核对 `protocol` 和清单的 `version`，对不上不加载。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct Describe {
    pub protocol: u32,
    #[schemars(with = "String")]
    pub version: semver::Version,
    /// 实际可用的模型或语言。
    #[serde(default)]
    pub models: Vec<String>,
}

impl Describe {
    /// 这一版协议的 `describe`。
    pub fn new(version: semver::Version, models: Vec<String>) -> Self {
        Self {
            protocol: 1,
            version,
            models,
        }
    }
}

/// `health` 的结果（协议 §3.10 的 `ready`、`checks`）。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct Health {
    /// 所有必要检查都通过、能接 `run`。
    pub ready: bool,
    #[serde(default)]
    pub checks: Vec<HealthCheck>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct HealthCheck {
    pub name: String,
    pub ok: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

/// `run` 的参数（协议 §4.2）。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct RunParams {
    pub item: RunItem,
    /// 这次要哪几种产出；取清单 `produces` 的子集，不为空。
    pub want: Vec<Produce>,
    pub inputs: Vec<RunInput>,
    /// 这次调用的输出目录，绝对路径，进程可写；插件只写这里。
    #[schemars(with = "String")]
    pub out: PathBuf,
    /// 这个插件的非密钥设置。
    #[serde(default)]
    pub options: Map<String, Value>,
    /// 清单的 `provider` 不是 `none`、你配了模型时才有（协议 §4.3）。
    #[serde(default)]
    pub provider: Option<ProviderGrant>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct RunItem {
    pub id: String,
    /// 条目的 `kind`（`video`、`post`、`article`…）。
    pub kind: String,
    pub title: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lang: Option<String>,
}

/// 一个输入：放进进程暂存目录后的绝对路径，只读。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct RunInput {
    pub kind: ConsumeKind,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mime: Option<String>,
    #[schemars(with = "String")]
    pub path: PathBuf,
    /// 所在的快照。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub snapshot: Option<String>,
    /// 图文的图、封面：`imageKey`（SPEC §1.2.2）。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub image_key: Option<String>,
    /// 关键帧：在视频的第几秒。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub frame_sec: Option<f64>,
    /// 合起来能播（`play`）时，哪个是画面、哪个是音轨。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub role: Option<InputRole>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum InputRole {
    Video,
    Audio,
}

/// 宿主代发的模型参数（协议 §4.3）。`token` 只对这一次调用有效，`Debug` 不打印它。
#[derive(Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ProviderGrant {
    pub base_url: String,
    pub token: String,
    pub model: String,
}

impl fmt::Debug for ProviderGrant {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ProviderGrant")
            .field("base_url", &self.base_url)
            .field("token", &"<redacted>")
            .field("model", &self.model)
            .finish()
    }
}

/// `run` 的结果。宿主照它写 `run.json`（SPEC §1.2.2）。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct RunResult {
    /// `out` 里的产物，每种一项；`empty: true` 时为空。
    #[serde(default)]
    pub outputs: Vec<RunProduct>,
    /// 插件里干活的程序和版本，例如 `{name: sherpa-onnx, version: 2.0.1}`。
    pub tool: UpstreamVersion,
    /// 这次实际用的参数，含插件补上的默认值；宿主原样记下，重做时原样交回。
    #[serde(default)]
    pub options: Map<String, Value>,
    /// 用了模型时是哪一个；sha256 由宿主自己核。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: Option<ModelUsed>,
    #[serde(default)]
    pub marks: Vec<Mark>,
    /// 这条没有可产出的东西；宿主不再往下游编排。
    #[serde(default)]
    pub empty: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<EmptyReason>,
    /// `want` 有 `probe` 时：每个输入的流。
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub probe: Vec<Probe>,
}

/// 一个产物；`path` 相对 `out`。`keyframes` 的 `path` 是 `keyframes.json`，图片在它旁边的 `keyframes/` 里。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct RunProduct {
    pub kind: Produce,
    pub path: RelPath,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lang: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ModelUsed {
    pub name: String,
}

/// `empty: true` 的原因。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum EmptyReason {
    NoSpeech,
    NoAudioTrack,
    NoText,
}

/// 很明显的标记，量出来的，只提示（feature 003 §6）。`duplicate-images` 由宿主算，不在这里。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Mark {
    pub code: MarkCode,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub speech_sec: Option<Seconds>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub duration_sec: Option<Seconds>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub frozen_sec: Option<Seconds>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum MarkCode {
    NoAudioTrack,
    NoSpeech,
    StaticPicture,
    NoText,
}

/// 一个输入里有哪些流、多长（`want: [probe]`）。宿主用它配对画面和音轨、判没有声音轨。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Probe {
    /// `inputs` 的下标。
    pub input: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub duration_sec: Option<Seconds>,
    pub streams: Vec<ProbeStream>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ProbeStream {
    pub kind: MediaKind,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub codec: Option<String>,
}

impl RunResult {
    /// 这个结果和这次的 `want`、输入个数对不对得上；不对时回原因。plugin-kit 回之前查，宿主收到时再查。
    pub fn problem(&self, want: &[Produce], inputs: usize) -> Option<String> {
        if self.empty {
            if !self.outputs.is_empty() {
                return Some("`empty: true` with outputs".into());
            }
            if self.reason.is_none() {
                return Some("`empty: true` needs a `reason`".into());
            }
        } else if self.reason.is_some() {
            return Some("`reason` is only for `empty: true`".into());
        }
        let mut kinds = BTreeSet::new();
        for o in &self.outputs {
            if !want.contains(&o.kind) {
                return Some(format!("output `{}` was not wanted", o.kind));
            }
            if !o.kind.writes_files() {
                return Some(format!("`{}` writes no file", o.kind));
            }
            if !kinds.insert(o.kind) {
                return Some(format!("output `{}` is listed twice", o.kind));
            }
        }
        let wants_files = want.iter().any(|w| w.writes_files());
        if wants_files && !self.empty && self.outputs.is_empty() {
            return Some("no outputs and not `empty`".into());
        }
        if !self.probe.is_empty() && !want.contains(&Produce::Probe) {
            return Some("`probe` was not wanted".into());
        }
        if let Some(p) = self.probe.iter().find(|p| p.input as usize >= inputs) {
            return Some(format!("`probe` names input {} of {inputs}", p.input));
        }
        None
    }
}

/// 通知 `progress` 的参数。只用来判断卡住，不往界面报（协议 §4.1）。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct Progress {
    /// 哪个 `run`。
    pub id: RpcId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub done: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub total: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub unit: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

/// 通知 `cancel` 的参数；插件仍要回那个 `run`，错误码 `cancelled`。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct Cancel {
    pub id: RpcId,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn result(json: Value) -> RunResult {
        serde_json::from_value(json).unwrap()
    }

    #[test]
    fn run_round_trips() {
        let params = serde_json::json!({
            "item": {"id": "bilibili-4plrcxk343onmyxibmrr672kga", "kind": "video", "title": "手冲咖啡", "lang": "zh"},
            "want": ["play"],
            "inputs": [
                {"kind": "media:video", "mime": "video/mp4", "path": "/tmp/p/in/7/raw/s/media/video.m4s", "snapshot": "019a6f2b-4a10-7c3e-8b21-0d9e8f7a6b5c", "role": "video"},
                {"kind": "media:audio", "path": "/tmp/p/in/7/raw/s/media/audio.m4s", "snapshot": "019a6f2b-4a10-7c3e-8b21-0d9e8f7a6b5c", "role": "audio"},
                {"kind": "derived:keyframes", "path": "/tmp/p/in/7/k/00012040.jpg", "frameSec": 12.04},
                {"kind": "image", "path": "/tmp/p/in/7/raw/s/images/001.jpg", "imageKey": "3f2a9c0d1e4b5a67"}
            ],
            "out": "/tmp/p/out/7",
            "options": {"model": "sense-voice"},
            "provider": null
        });
        let typed: RunParams = serde_json::from_value(params.clone()).unwrap();
        assert_eq!(typed.inputs[0].role, Some(InputRole::Video));
        let mut back = serde_json::to_value(&typed).unwrap();
        back.as_object_mut().unwrap().remove("provider");
        let mut expected = params;
        expected.as_object_mut().unwrap().remove("provider");
        assert_eq!(back, expected);

        let r = serde_json::json!({
            "outputs": [{"kind": "asr", "path": "asr.vtt", "lang": "zh"}],
            "tool": {"name": "sherpa-onnx", "version": "2.0.1"},
            "options": {"language": "auto", "vadThreshold": 0.5},
            "model": {"name": "sense-voice-int8-2025-09-09"},
            "marks": [],
            "empty": false
        });
        let typed = result(r.clone());
        assert_eq!(serde_json::to_value(&typed).unwrap(), r);
        assert_eq!(typed.problem(&[Produce::Asr], 1), None);
    }

    #[test]
    fn result_rules() {
        let tool = serde_json::json!({"name": "ffmpeg", "version": "9.0.2"});
        let empty = result(serde_json::json!({
            "tool": tool, "empty": true, "reason": "no-speech",
            "marks": [{"code": "no-speech", "speechSec": 0.4, "durationSec": 5.0}]
        }));
        assert_eq!(empty.problem(&[Produce::Asr], 1), None);
        let no_reason = result(serde_json::json!({"tool": tool, "empty": true}));
        assert!(no_reason.problem(&[Produce::Asr], 1).is_some());
        let nothing = result(serde_json::json!({"tool": tool}));
        assert!(nothing.problem(&[Produce::Asr], 1).is_some());
        let unwanted = result(serde_json::json!({
            "tool": tool, "outputs": [{"kind": "play", "path": "play.mp4"}]
        }));
        assert!(unwanted.problem(&[Produce::Asr], 1).is_some());
        let probe = result(serde_json::json!({
            "tool": tool,
            "probe": [{"input": 0, "durationSec": 7.5, "streams": [{"kind": "video", "codec": "h264"}]}]
        }));
        assert_eq!(probe.problem(&[Produce::Probe], 1), None);
        assert!(probe.problem(&[Produce::Probe], 0).is_some());
        assert!(probe.problem(&[Produce::Asr], 1).is_some());
        for bad in [
            serde_json::json!({"tool": tool, "empty": true, "reason": "no-music"}),
            serde_json::json!({"tool": tool, "marks": [{"code": "duplicate-images"}]}),
            serde_json::json!({"tool": tool, "marks": [{"code": "no-speech", "speechSec": -1.0}]}),
            serde_json::json!({"tool": tool, "outputs": [{"kind": "derived/asr.vtt", "path": "asr.vtt"}]}),
            serde_json::json!({"tool": tool, "outputs": [{"kind": "asr", "path": "../asr.vtt"}]}),
        ] {
            assert!(
                serde_json::from_value::<RunResult>(bad.clone()).is_err(),
                "{bad}"
            );
        }
    }

    #[test]
    fn provider_token_stays_out_of_debug() {
        let grant = ProviderGrant {
            base_url: "http://127.0.0.1:47822/v1".into(),
            token: "q8Zr2VbN4kTxW1mYc7HdLs0JpEa5Gu9F".into(),
            model: "summary".into(),
        };
        assert!(!format!("{grant:?}").contains("q8Zr2"));
    }
}
