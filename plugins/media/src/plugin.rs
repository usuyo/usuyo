//! `builtin/media` 的 `run` 和 `health`（协议 §4.2）；JSON-RPC 循环在 plugin-kit。
//!
//! 一次 `run` 做一种产物（`want` 只有一项）：`probe`、`play`、`audio:pcm16k`、`keyframes`。
//! 查完 `want`、`inputs`、`options` 才起 ffmpeg；错误在这里统一换成协议的错误对象。

use std::path::Path;

use usuyo_plugin_kit::protocol::{
    CallFailure, ConsumeKind, Describe, Health, HealthCheck, Produce, RunInput, RunParams,
    RunResult,
};
use usuyo_plugin_kit::semver;
use usuyo_plugin_kit::transform::{RunContext, Transform};

use crate::error::MediaError;
use crate::exec::Supervisor;
use crate::keyframes;
use crate::options::{KeyframeOptions, expect_none};
use crate::products;
use crate::tools::Tools;

pub struct Media {
    tools: Tools,
}

/// `RunContext` 给子进程循环用：取消标志和进度口子。
struct Context<'a>(&'a RunContext);

impl Supervisor for Context<'_> {
    fn is_cancelled(&self) -> bool {
        self.0.is_cancelled()
    }
    fn progress(&self, done: f64, total: f64, unit: &str) {
        self.0.progress(done, total, unit);
    }
}

impl Media {
    pub fn new(tools: Tools) -> Self {
        Self { tools }
    }

    /// `describe` 的回答：没有模型。
    pub fn describe(&self, version: semver::Version) -> Describe {
        Describe::new(version, Vec::new())
    }

    fn make(&self, params: RunParams, ctx: &RunContext) -> Result<RunResult, MediaError> {
        let sup = Context(ctx);
        let [want] = params.want.as_slice() else {
            return Err(MediaError::BadArgs(
                "`want` must name exactly one product".into(),
            ));
        };
        let media_inputs = || -> Result<Vec<&RunInput>, MediaError> {
            if params.inputs.is_empty() {
                return Err(MediaError::BadArgs("`inputs` is empty".into()));
            }
            for input in &params.inputs {
                if !matches!(input.kind, ConsumeKind::Video | ConsumeKind::Audio) {
                    return Err(MediaError::BadArgs(
                        "inputs must be `media:video` or `media:audio`".into(),
                    ));
                }
            }
            Ok(params.inputs.iter().collect())
        };
        let one = |inputs: Vec<&RunInput>, kinds: &[ConsumeKind]| -> Result<RunInput, MediaError> {
            match inputs.as_slice() {
                [input] if kinds.contains(&input.kind) => Ok((*input).clone()),
                _ => Err(MediaError::BadArgs(format!(
                    "`{want}` takes exactly one input of kind {kinds:?}"
                ))),
            }
        };
        let out = params.out.as_path();
        match want {
            Produce::Probe => {
                expect_none(&params.options)?;
                let paths: Vec<&Path> = media_inputs()?.iter().map(|i| i.path.as_path()).collect();
                products::probe_all(&self.tools, &sup, &paths)
            }
            Produce::Play => {
                expect_none(&params.options)?;
                let inputs = media_inputs()?;
                let (video, audio) = match inputs.as_slice() {
                    [a, b] if a.kind == ConsumeKind::Video && b.kind == ConsumeKind::Audio => {
                        (a, b)
                    }
                    [a, b] if a.kind == ConsumeKind::Audio && b.kind == ConsumeKind::Video => {
                        (b, a)
                    }
                    _ => {
                        return Err(MediaError::BadArgs(
                            "`play` takes one `media:video` and one `media:audio` input".into(),
                        ));
                    }
                };
                products::play(&self.tools, &sup, &video.path, &audio.path, out)
            }
            Produce::AudioPcm16k => {
                expect_none(&params.options)?;
                let input = one(media_inputs()?, &[ConsumeKind::Video, ConsumeKind::Audio])?;
                products::audio_pcm16k(&self.tools, &sup, &input.path, out)
            }
            Produce::Keyframes => {
                let options = KeyframeOptions::from_map(&params.options)?;
                let input = one(media_inputs()?, &[ConsumeKind::Video])?;
                let snapshot = input.snapshot.as_deref().ok_or_else(|| {
                    MediaError::BadArgs("`keyframes` needs the video input's `snapshot`".into())
                })?;
                keyframes::run(&self.tools, &sup, &input.path, snapshot, out, &options)
            }
            other => Err(MediaError::BadArgs(format!(
                "this plugin does not produce `{other}`"
            ))),
        }
    }
}

impl Transform for Media {
    fn run(&self, params: RunParams, ctx: &RunContext) -> Result<RunResult, CallFailure> {
        let result = self.make(params, ctx)?;
        ctx.check_cancelled()?;
        Ok(result)
    }

    fn health(&self) -> Health {
        let check = |name: &str, tool: &crate::tools::Tool| {
            let ok = tool.path.is_file();
            HealthCheck {
                name: name.to_owned(),
                ok,
                message: Some(if ok {
                    format!("{name} {}", tool.version)
                } else {
                    format!("{} is missing", tool.path.display())
                }),
            }
        };
        Health {
            ready: self.tools.present(),
            checks: vec![
                check("ffmpeg", &self.tools.ffmpeg),
                check("ffprobe", &self.tools.ffprobe),
            ],
        }
    }
}
