//! 信封、错误对象、错误码和 stderr 事件（协议 §3.2、§3.3，SPEC §0.3）。

use std::borrow::Cow;
use std::fmt;
use std::time::Duration;

use schemars::{JsonSchema, Schema, SchemaGenerator, json_schema};
use serde::{Deserialize, Serialize};

/// stdout 上的那一个 JSON 值：`{"ok": true, ...}` 或 `{"ok": false, "error": {...}}`。
/// 插件作者用它输出；宿主自己解析，见 `host`。
#[derive(Debug, Clone)]
pub enum Envelope<T> {
    Ok(T),
    Err(CallFailure),
}

impl<T> Envelope<T> {
    /// 和信封对应的退出码：成功 0，失败 1（协议 §3.2）。
    pub fn exit_code(&self) -> i32 {
        match self {
            Envelope::Ok(_) => 0,
            Envelope::Err(_) => 1,
        }
    }
}

impl<T: Serialize> Serialize for Envelope<T> {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        #[derive(Serialize)]
        struct Okay<'a, T> {
            ok: bool,
            #[serde(flatten)]
            body: &'a T,
        }
        #[derive(Serialize)]
        struct Fail<'a> {
            ok: bool,
            error: &'a CallFailure,
        }
        match self {
            Envelope::Ok(body) => Okay { ok: true, body }.serialize(s),
            Envelope::Err(error) => Fail { ok: false, error }.serialize(s),
        }
    }
}

/// 错误码。前 11 个由插件报，后 4 个只由宿主记（协议 §3.3）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum ErrorCode {
    UnsupportedInput,
    NotFound,
    LoginRequired,
    RateLimited,
    Blocked,
    UpstreamChanged,
    Network,
    DependencyMissing,
    InvalidArgs,
    Cancelled,
    Internal,
    Timeout,
    Crashed,
    InvalidOutput,
    ProfileBusy,
}

impl ErrorCode {
    /// 插件不能报的码：超时、崩溃、输出不合规、等不到资料目录。
    pub fn is_host_only(self) -> bool {
        matches!(
            self,
            ErrorCode::Timeout
                | ErrorCode::Crashed
                | ErrorCode::InvalidOutput
                | ErrorCode::ProfileBusy
        )
    }

    /// 运行器要不要换下一个插件：只有 `cancelled` 不换（协议 §3.3）。
    pub fn tries_next(self) -> bool {
        self != ErrorCode::Cancelled
    }

    /// 不改什么、过会儿再发可能成功（SPEC §0.3 的「可重试」一列）。
    pub fn retryable(self) -> bool {
        matches!(
            self,
            ErrorCode::RateLimited
                | ErrorCode::Network
                | ErrorCode::ProfileBusy
                | ErrorCode::Timeout
        )
    }

    pub fn as_str(self) -> &'static str {
        match self {
            ErrorCode::UnsupportedInput => "unsupported-input",
            ErrorCode::NotFound => "not-found",
            ErrorCode::LoginRequired => "login-required",
            ErrorCode::RateLimited => "rate-limited",
            ErrorCode::Blocked => "blocked",
            ErrorCode::UpstreamChanged => "upstream-changed",
            ErrorCode::Network => "network",
            ErrorCode::DependencyMissing => "dependency-missing",
            ErrorCode::InvalidArgs => "invalid-args",
            ErrorCode::Cancelled => "cancelled",
            ErrorCode::Internal => "internal",
            ErrorCode::Timeout => "timeout",
            ErrorCode::Crashed => "crashed",
            ErrorCode::InvalidOutput => "invalid-output",
            ErrorCode::ProfileBusy => "profile-busy",
        }
    }
}

impl fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// `not-found` 的细分。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum NotFoundReason {
    Deleted,
    Private,
    Paid,
    Region,
    LinkExpired,
}

/// `needs` 的一项，「前缀:值」（协议 §3.3，`version:` 照 §13 第 30 项）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct Need(String);

impl Need {
    const PREFIXES: [&'static str; 9] = [
        "runtime",
        "setting",
        "device",
        "permission",
        "service",
        "app",
        "state",
        "file",
        "version",
    ];

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for Need {
    type Error = String;
    fn try_from(text: String) -> Result<Self, String> {
        match text.split_once(':') {
            Some((prefix, value)) if Self::PREFIXES.contains(&prefix) && !value.is_empty() => {
                Ok(Self(text))
            }
            _ => Err(format!(
                "`{text}` is not `<prefix>:<value>` with prefix one of {}",
                Self::PREFIXES.join(", ")
            )),
        }
    }
}

impl From<Need> for String {
    fn from(v: Need) -> String {
        v.0
    }
}

impl JsonSchema for Need {
    fn schema_name() -> Cow<'static, str> {
        "Need".into()
    }
    fn json_schema(_: &mut SchemaGenerator) -> Schema {
        json_schema!({
            "type": "string",
            "pattern": "^(runtime|setting|device|permission|service|app|state|file|version):.+$"
        })
    }
}

/// 错误对象（协议 §3.3）。宿主自己记的失败也用它。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct CallFailure {
    pub code: ErrorCode,
    /// 给日志看的原因。
    pub message: String,
    /// 给用户看的一句话，按 `USUYO_LOCALE` 写。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hint: Option<String>,
    /// `rate-limited` 时多少秒后再试。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub retry_after: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub site: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<NotFoundReason>,
    /// `dependency-missing` 必填，缺的一次列全。
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub needs: Vec<Need>,
}

/// 限流没给 `retryAfter` 时暂停多久（协议 §3.1）。
const DEFAULT_RATE_LIMIT_PAUSE: Duration = Duration::from_secs(15 * 60);

/// 一次暂停最长多久。宿主在解码插件的错误时把 `retryAfter` 封到这里，调用方存暂停时也按它查旧数据。
pub const MAX_PAUSE: Duration = Duration::from_secs(3600);

/// 出错以后，这个插件在这个平台上暂停多久（协议 §3.1）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Pause {
    For(Duration),
    /// 到用户在插件那一行点「继续」。
    UntilResumed,
}

impl CallFailure {
    pub fn new(code: ErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            hint: None,
            retry_after: None,
            site: None,
            reason: None,
            needs: Vec::new(),
        }
    }

    pub fn pause(&self) -> Option<Pause> {
        match self.code {
            ErrorCode::RateLimited => Some(Pause::For(
                self.retry_after
                    .map_or(DEFAULT_RATE_LIMIT_PAUSE, Duration::from_secs),
            )),
            ErrorCode::Blocked => Some(Pause::UntilResumed),
            _ => None,
        }
    }

    /// 插件报来的错误对象要守的规则；不守的整次按 `invalid-output`。
    #[cfg(feature = "host")]
    pub(crate) fn check_from_plugin(&self) -> Result<(), String> {
        if self.code.is_host_only() {
            return Err(format!("plugins may not report `{}`", self.code));
        }
        if self.code == ErrorCode::DependencyMissing && self.needs.is_empty() {
            return Err("`dependency-missing` needs a non-empty `needs`".into());
        }
        Ok(())
    }
}

impl fmt::Display for CallFailure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.code, self.message)
    }
}

/// stderr 上以 `@usuyo ` 开头的结构化事件（协议 §3.2）。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "event", rename_all = "kebab-case")]
pub enum PluginEvent {
    Progress {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        done: Option<f64>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        total: Option<f64>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        unit: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        message: Option<String>,
    },
    Waiting {
        reason: WaitReason,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        message: Option<String>,
    },
}

impl PluginEvent {
    pub const STDERR_PREFIX: &'static str = "@usuyo ";

    /// 插件写到 stderr 的一行，不带换行。
    pub fn stderr_line(&self) -> String {
        let json = serde_json::to_string(self).unwrap_or_default();
        format!("{}{json}", Self::STDERR_PREFIX)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum WaitReason {
    UserAction,
    DeviceLocked,
    DeviceDisconnected,
    DeviceUnauthorized,
    UserActive,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn envelope_shapes() {
        let ok: Envelope<serde_json::Value> = Envelope::Ok(serde_json::json!({"match": false}));
        assert_eq!(
            serde_json::to_string(&ok).unwrap(),
            r#"{"ok":true,"match":false}"#
        );
        let err: Envelope<()> = Envelope::Err(CallFailure::new(ErrorCode::Network, "dns"));
        assert_eq!(
            serde_json::to_string(&err).unwrap(),
            r#"{"ok":false,"error":{"code":"network","message":"dns"}}"#
        );
        assert_eq!(err.exit_code(), 1);
    }

    #[test]
    fn runner_semantics() {
        assert!(!ErrorCode::Cancelled.tries_next());
        assert!(ErrorCode::Blocked.tries_next());
        assert!(ErrorCode::Network.retryable());
        assert!(!ErrorCode::LoginRequired.retryable());
        let mut limited = CallFailure::new(ErrorCode::RateLimited, "429");
        assert_eq!(limited.pause(), Some(Pause::For(Duration::from_secs(900))));
        limited.retry_after = Some(30);
        assert_eq!(limited.pause(), Some(Pause::For(Duration::from_secs(30))));
        assert_eq!(
            CallFailure::new(ErrorCode::Blocked, "captcha").pause(),
            Some(Pause::UntilResumed)
        );
    }

    #[test]
    fn needs_and_host_codes() {
        assert!(Need::try_from("version:usuyo>=1.3".to_owned()).is_ok());
        assert!(Need::try_from("runtime:".to_owned()).is_err());
        assert!(Need::try_from("thing:x".to_owned()).is_err());
        #[cfg(feature = "host")]
        {
            assert!(
                CallFailure::new(ErrorCode::Timeout, "x")
                    .check_from_plugin()
                    .is_err()
            );
            assert!(
                CallFailure::new(ErrorCode::DependencyMissing, "x")
                    .check_from_plugin()
                    .is_err()
            );
        }
    }

    #[test]
    fn events() {
        let line = r#"{"event":"progress","done":3,"total":20,"unit":"MB","message":"下载视频"}"#;
        let ev: PluginEvent = serde_json::from_str(line).unwrap();
        assert!(matches!(ev, PluginEvent::Progress { done: Some(d), .. } if d == 3.0));
        assert!(ev.stderr_line().starts_with("@usuyo {"));
    }
}
