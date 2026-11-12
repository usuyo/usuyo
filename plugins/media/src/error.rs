//! 插件内部的错误：在 `run` 的入口统一换成协议的错误对象（`CallFailure`），只在那里处理一次。

use std::io;

use usuyo_plugin_kit::protocol::{CallFailure, ErrorCode};

use crate::exec::ExecError;
use crate::options::OptionError;
use crate::probe::ProbeError;

#[derive(Debug, thiserror::Error)]
pub enum MediaError {
    /// `want`、`inputs`、`options` 不合这个插件的规矩。
    #[error("{0}")]
    BadArgs(String),
    #[error(transparent)]
    Option(#[from] OptionError),
    /// 输入不是这次要的那种文件（例如 `play` 收到已经带声音的画面）。
    #[error("{0}")]
    Refused(String),
    #[error("{path}: {source}")]
    Probe {
        path: String,
        #[source]
        source: ProbeError,
    },
    #[error(transparent)]
    Exec(#[from] ExecError),
    #[error("{context}: {source}")]
    Io {
        context: String,
        #[source]
        source: io::Error,
    },
    /// ffmpeg 退出正常，但产物不见了或不合格。
    #[error("{0}")]
    Output(String),
}

impl MediaError {
    pub fn io(context: impl Into<String>, source: io::Error) -> Self {
        MediaError::Io {
            context: context.into(),
            source,
        }
    }
}

impl From<MediaError> for CallFailure {
    fn from(e: MediaError) -> Self {
        let code = match &e {
            MediaError::BadArgs(_) | MediaError::Option(_) => ErrorCode::InvalidArgs,
            MediaError::Refused(_) => ErrorCode::UnsupportedInput,
            // ffprobe 认不出来：文件坏了或不是音视频。
            MediaError::Probe {
                source: ProbeError::Run(ExecError::Exited { .. }) | ProbeError::Json(_),
                ..
            } => ErrorCode::UnsupportedInput,
            MediaError::Probe {
                source: ProbeError::Run(ExecError::Cancelled),
                ..
            }
            | MediaError::Exec(ExecError::Cancelled) => ErrorCode::Cancelled,
            _ => ErrorCode::Internal,
        };
        CallFailure::new(code, e.to_string())
    }
}
