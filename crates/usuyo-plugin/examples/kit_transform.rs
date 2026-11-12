//! 测试用的转换插件，用 plugin-kit 的 JSON-RPC 循环：`tests/transform.rs` 从宿主一侧起它，
//! 核对两边说的是同一套消息。把第一个输入拷成 `asr.vtt`；`options.wait` 为真时等到被取消。

use std::process::ExitCode;
use std::time::Duration;

use usuyo_plugin_kit::protocol::{
    CallFailure, Describe, ErrorCode, Produce, RelPath, RunParams, RunProduct, RunResult,
    UpstreamVersion,
};
use usuyo_plugin_kit::transform::{self, RunContext};

fn run(params: RunParams, ctx: &RunContext) -> Result<RunResult, CallFailure> {
    ctx.progress(0.0, 1.0, "files");
    if params.options.get("wait").and_then(|v| v.as_bool()) == Some(true) {
        while !ctx.is_cancelled() {
            std::thread::sleep(Duration::from_millis(10));
        }
        ctx.check_cancelled()?;
    }
    let input = params
        .inputs
        .first()
        .ok_or_else(|| CallFailure::new(ErrorCode::UnsupportedInput, "no input"))?;
    let bytes = std::fs::read(&input.path)
        .map_err(|e| CallFailure::new(ErrorCode::Internal, format!("read input: {e}")))?;
    std::fs::write(params.out.join("asr.vtt"), bytes)
        .map_err(|e| CallFailure::new(ErrorCode::Internal, format!("write output: {e}")))?;
    let path =
        RelPath::try_from("asr.vtt").map_err(|e| CallFailure::new(ErrorCode::Internal, e))?;
    Ok(RunResult {
        outputs: vec![RunProduct {
            kind: Produce::Asr,
            path,
            lang: Some("zh".into()),
        }],
        tool: UpstreamVersion {
            name: "kit-example".into(),
            version: "1".into(),
        },
        options: params.options,
        model: None,
        marks: Vec::new(),
        empty: false,
        reason: None,
        probe: Vec::new(),
    })
}

fn main() -> ExitCode {
    let Ok(version) = env!("CARGO_PKG_VERSION").parse() else {
        eprintln!("the crate version is not semver");
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
