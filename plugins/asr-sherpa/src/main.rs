//! 插件进程的入口：核 VAD 资产，跑 plugin-kit 的 JSON-RPC 循环，直到宿主关 stdin。
//!
//! 宿主给的：`USUYO_PLUGIN_DIR`（安装目录，里面有 `silero_vad_v5.onnx`）、
//! `USUYO_MODELS_DIR`（这个插件的 `${MODELS}`，模型在 `<它>/sense-voice-int8-2024-07-17/`）。

use std::path::PathBuf;
use std::process::ExitCode;

use anyhow::Context as _;
use usuyo_plugin_asr_sherpa::engine::{VAD_FILE, VadModel};
use usuyo_plugin_asr_sherpa::plugin::AsrSherpa;
use usuyo_plugin_kit::transform;

fn main() -> ExitCode {
    match serve() {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("asr-sherpa: {e:#}");
            ExitCode::FAILURE
        }
    }
}

fn serve() -> anyhow::Result<()> {
    let plugin_dir = env_path("USUYO_PLUGIN_DIR")?;
    let models_dir = env_path("USUYO_MODELS_DIR")?;
    let vad = VadModel::verify(&plugin_dir.join(VAD_FILE))
        .context("the VAD asset is missing or changed")?;
    let version = env!("CARGO_PKG_VERSION")
        .parse()
        .context("the package version is not semver")?;
    let plugin = AsrSherpa::new(vad, models_dir);
    let describe = plugin.describe(version);
    transform::serve(describe, plugin)?;
    Ok(())
}

fn env_path(name: &str) -> anyhow::Result<PathBuf> {
    std::env::var_os(name)
        .filter(|v| !v.is_empty())
        .map(PathBuf::from)
        .with_context(|| format!("{name} is not set"))
}
