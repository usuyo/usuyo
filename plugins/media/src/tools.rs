//! 插件自带的 ffmpeg、ffprobe：在 `${PLUGIN_DIR}` 下找、问版本、拼命令行。
//!
//! 命令行的规矩：
//! - 输入输出路径都变成绝对路径加 `file:` 前缀，所以不会被读成选项（以 `-` 开头的文件名），
//!   也不会被读成协议（`http:`、`concat:` 一类的文件名）；
//! - 输入前加 `-protocol_whitelist file` 和 `-format_whitelist <常见封装>`，文件里写的别的地址、
//!   `ffconcat` 之类的清单不会被跟着读。

use std::ffi::OsString;
use std::io;
use std::path::{Path, PathBuf};
use std::time::Duration;

use usuyo_plugin_kit::protocol::UpstreamVersion;

use crate::exec::{self, ExecError, Spec, Stdout, Unsupervised};

pub const FFMPEG_FILE: &str = "ffmpeg";
pub const FFPROBE_FILE: &str = "ffprobe";

/// 精简构建里有的拆封装器（feature 003 §4.3）加上 flac、ogg；输入的封装不在里面就不读。
const INPUT_FORMATS: &str = "mov,mp4,m4a,3gp,3g2,mj2,matroska,webm,flv,mpegts,aac,mp3,wav,flac,ogg";

/// 问 `-version` 最多等多久。
const VERSION_TIMEOUT: Duration = Duration::from_secs(10);

#[derive(Debug, thiserror::Error)]
pub enum ToolsError {
    #[error("{0} is not a file")]
    Missing(PathBuf),
    #[error(transparent)]
    Run(#[from] ExecError),
    #[error("{program} printed an unexpected first line for -version: {line:?}")]
    Version { program: &'static str, line: String },
}

/// 一个程序：路径，和它自己报的版本。
#[derive(Debug, Clone)]
pub struct Tool {
    pub path: PathBuf,
    pub version: String,
}

impl Tool {
    fn locate(dir: &Path, file: &'static str) -> Result<Self, ToolsError> {
        let path = dir.join(file);
        if !path.is_file() {
            return Err(ToolsError::Missing(path));
        }
        let version = ask_version(&path, file)?;
        Ok(Self { path, version })
    }

    /// 给 `run` 的结果里的 `tool`。
    pub fn upstream(&self, name: &str) -> UpstreamVersion {
        UpstreamVersion {
            name: name.to_owned(),
            version: self.version.clone(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Tools {
    pub ffmpeg: Tool,
    pub ffprobe: Tool,
}

impl Tools {
    pub fn locate(plugin_dir: &Path) -> Result<Self, ToolsError> {
        Ok(Self {
            ffmpeg: Tool::locate(plugin_dir, FFMPEG_FILE)?,
            ffprobe: Tool::locate(plugin_dir, FFPROBE_FILE)?,
        })
    }

    /// 两个文件还在。
    pub fn present(&self) -> bool {
        self.ffmpeg.path.is_file() && self.ffprobe.path.is_file()
    }
}

fn ask_version(path: &Path, program: &'static str) -> Result<String, ToolsError> {
    let spec = Spec {
        program: path,
        args: vec!["-version".into()],
        timeout: VERSION_TIMEOUT,
        stdout: Stdout::Capture,
        total: 0.0,
        unit: "sec",
    };
    let finished = exec::run(&spec, &Unsupervised, |_| false)?;
    let text = String::from_utf8_lossy(&finished.stdout);
    let line = text.lines().next().unwrap_or_default();
    parse_version(program, line).ok_or_else(|| ToolsError::Version {
        program,
        line: line.chars().take(200).collect(),
    })
}

/// `ffmpeg version 9.0.1 Copyright …` 里的 `9.0.1`。版本号只留字母、数字和 `. _ + - ~`，最长 64 个字符。
pub fn parse_version(program: &str, first_line: &str) -> Option<String> {
    let rest = first_line
        .strip_prefix(program)?
        .strip_prefix(" version ")?;
    let token = rest.split_whitespace().next()?;
    let ok = token.len() <= 64
        && token
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || ".+-_~".contains(c));
    ok.then(|| token.to_owned())
}

/// `file:<绝对路径>`。
pub fn file_arg(path: &Path) -> io::Result<OsString> {
    let mut arg = OsString::from("file:");
    arg.push(std::path::absolute(path)?);
    Ok(arg)
}

/// 输入文件前面的几个选项加 `-i file:<路径>`（ffmpeg 和 ffprobe 都用这几个）。
pub fn input_args(path: &Path) -> io::Result<Vec<OsString>> {
    Ok(vec![
        "-protocol_whitelist".into(),
        "file".into(),
        "-format_whitelist".into(),
        INPUT_FORMATS.into(),
        "-i".into(),
        file_arg(path)?,
    ])
}

/// ffmpeg 命令行的开头：不读 stdin、不打横幅和统计；`progress` 时进度走 stdout 的 `-progress`。
pub fn ffmpeg_prefix(loglevel: &str, progress: bool) -> Vec<OsString> {
    let mut args: Vec<OsString> = [
        "-nostdin",
        "-hide_banner",
        "-nostats",
        "-loglevel",
        loglevel,
    ]
    .map(OsString::from)
    .into();
    if progress {
        args.extend(["-progress", "pipe:1"].map(OsString::from));
    }
    args
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn versions() {
        assert_eq!(
            parse_version("ffmpeg", "ffmpeg version 9.0.1 Copyright (c) 2000-2026"),
            Some("9.0.1".into())
        );
        assert_eq!(
            parse_version("ffprobe", "ffprobe version N-123456-gabcdef Copyright"),
            Some("N-123456-gabcdef".into())
        );
        assert_eq!(parse_version("ffmpeg", "ffprobe version 9.0.1"), None);
        assert_eq!(parse_version("ffmpeg", "ffmpeg version"), None);
        assert_eq!(parse_version("ffmpeg", "ffmpeg version $(rm)x"), None);
        let long = format!("ffmpeg version {}", "1".repeat(65));
        assert_eq!(parse_version("ffmpeg", &long), None);
    }

    #[test]
    fn paths_cannot_become_options_or_protocols() {
        let cwd = std::env::current_dir().unwrap();
        for name in ["-i", "-loglevel", "concat:a|b", "http:x.mp4", "pipe:0"] {
            let arg = file_arg(Path::new(name)).unwrap();
            let expected = format!("file:{}/{name}", cwd.display());
            assert_eq!(arg, OsString::from(expected));
        }
        let abs = file_arg(Path::new("/tmp/-x.mp4")).unwrap();
        assert_eq!(abs, OsString::from("file:/tmp/-x.mp4"));
    }
}
