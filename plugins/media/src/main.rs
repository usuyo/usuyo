//! 插件进程的入口：找 ffmpeg、ffprobe，跑 plugin-kit 的 JSON-RPC 循环，直到宿主关 stdin。
//!
//! 宿主给的：`USUYO_PLUGIN_DIR`（安装目录，里面有 `ffmpeg`、`ffprobe`）。不从 `PATH` 找。

use std::path::PathBuf;
use std::process::ExitCode;

use anyhow::Context as _;
use usuyo_plugin_kit::transform;
use usuyo_plugin_media::plugin::Media;
use usuyo_plugin_media::tools::Tools;

fn main() -> ExitCode {
    match serve() {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("media: {e:#}");
            ExitCode::FAILURE
        }
    }
}

fn serve() -> anyhow::Result<()> {
    let plugin_dir = std::env::var_os("USUYO_PLUGIN_DIR")
        .filter(|v| !v.is_empty())
        .map(PathBuf::from)
        .context("USUYO_PLUGIN_DIR is not set")?;
    let tools = Tools::locate(&plugin_dir).context("the ffmpeg assets are missing or broken")?;
    let version = env!("CARGO_PKG_VERSION")
        .parse()
        .context("the package version is not semver")?;
    let plugin = Media::new(tools);
    let describe = plugin.describe(version);
    transform::serve(describe, plugin)?;
    Ok(())
}
