//! 出错时回 RFC 9457 的 problem+json（SPEC §0.3）。

use axum::extract::{Request, State};
use axum::http::{HeaderValue, StatusCode, header};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use serde::Serialize;
use usuyo_core::{Engine, Lang, Said};

tokio::task_local! {
    /// 这次请求的 `hint` 用哪种语言：请求进来时照内核设置的语言定（[`speak`]）。
    static LANG: Lang;
}

/// 最外层的一道：这次请求里写出的 `hint` 都用内核现在的语言。
pub(crate) async fn speak(State(engine): State<Engine>, req: Request, next: Next) -> Response {
    LANG.scope(engine.lang(), next.run(req)).await
}

/// 不在请求里（测试直接转换）时用简体中文。
fn lang() -> Lang {
    LANG.try_with(|l| *l).unwrap_or_default()
}

/// 一个错误：码、状态、给人看的一句话；`type`、`title`、`instance` 在写出时补上。
#[derive(Debug, Clone)]
pub struct Problem {
    pub code: &'static str,
    pub status: u16,
    pub detail: String,
    pub plugin: Option<String>,
    /// 给人看的一句，两种语言都在，写出时照这次请求的语言挑；`detail` 是给日志的原因。
    pub hint: Option<Said>,
}

#[derive(Serialize)]
struct Wire<'a> {
    #[serde(rename = "type")]
    kind: String,
    title: &'static str,
    status: u16,
    detail: &'a str,
    instance: String,
    /// 客户端只按它分支。
    code: &'a str,
    retryable: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    plugin: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    hint: Option<&'a str>,
}

impl Problem {
    pub fn new(code: &'static str, detail: impl Into<String>) -> Problem {
        Problem {
            code,
            status: status_of(code).as_u16(),
            detail: detail.into(),
            plugin: None,
            hint: None,
        }
    }

    /// 配上一句给人看的话（`words.rs` 里的一句）。
    pub fn with_hint(mut self, hint: Said) -> Problem {
        self.hint = Some(hint);
        self
    }

    fn retryable(&self) -> bool {
        matches!(
            self.code,
            "rate-limited"
                | "network"
                | "library-busy"
                | "unavailable"
                | "profile-busy"
                | "timeout"
        )
    }
}

impl From<usuyo_core::Error> for Problem {
    fn from(e: usuyo_core::Error) -> Problem {
        let code = e.code();
        if code == "internal" {
            tracing::error!(error = %e, "request failed");
        }
        // 插件报的错照旧把它的一句话放进 `detail`；内核自己配的话只进 `hint`，`detail` 留英文给日志。
        let detail = match &e {
            usuyo_core::Error::Hinted { .. } => e.to_string(),
            _ => e.hint().map_or_else(|| e.to_string(), |h| h.en),
        };
        let mut p = Problem::new(code, detail);
        p.hint = e.hint();
        // 插件报的错一律 502、带插件 id，免得平台的 login-required 像令牌不对。
        if let Some(plugin) = e.plugin() {
            p.status = 502;
            p.plugin = Some(plugin.to_owned());
        }
        p
    }
}

impl IntoResponse for Problem {
    fn into_response(self) -> Response {
        let status = StatusCode::from_u16(self.status).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
        let retryable = self.retryable();
        let wire = Wire {
            kind: format!("urn:usuyo:problem:{}", self.code),
            title: title_of(self.code),
            status: self.status,
            detail: &self.detail,
            instance: format!("urn:uuid:{}", call_id()),
            code: self.code,
            retryable,
            plugin: self.plugin.as_deref(),
            hint: self.hint.as_ref().map(|h| h.pick(lang())),
        };
        let body = serde_json::to_vec(&wire).unwrap_or_default();
        let mut res = (status, body).into_response();
        res.headers_mut().insert(
            header::CONTENT_TYPE,
            HeaderValue::from_static("application/problem+json"),
        );
        if retryable {
            res.headers_mut()
                .insert(header::RETRY_AFTER, HeaderValue::from_static("5"));
        }
        res
    }
}

fn status_of(code: &str) -> StatusCode {
    match code {
        "invalid-args" | "cursor-invalid" => StatusCode::BAD_REQUEST,
        "unauthorized" => StatusCode::UNAUTHORIZED,
        "forbidden" => StatusCode::FORBIDDEN,
        "not-found" => StatusCode::NOT_FOUND,
        "gone" => StatusCode::GONE,
        "conflict" | "cancelled" | "library-version" => StatusCode::CONFLICT,
        "too-large" => StatusCode::PAYLOAD_TOO_LARGE,
        "unsupported-input" => StatusCode::UNPROCESSABLE_ENTITY,
        "dependency-missing" => StatusCode::FAILED_DEPENDENCY,
        "rate-limited" => StatusCode::TOO_MANY_REQUESTS,
        "library-busy" | "unavailable" | "profile-busy" => StatusCode::SERVICE_UNAVAILABLE,
        "timeout" => StatusCode::GATEWAY_TIMEOUT,
        "internal" => StatusCode::INTERNAL_SERVER_ERROR,
        // crashed、invalid-output 和只由插件报的码
        _ => StatusCode::BAD_GATEWAY,
    }
}

fn title_of(code: &str) -> &'static str {
    match code {
        "invalid-args" => "Invalid arguments",
        "cursor-invalid" => "Invalid cursor",
        "unauthorized" => "Unauthorized",
        "forbidden" => "Forbidden",
        "not-found" => "Not found",
        "gone" => "Gone",
        "conflict" => "Conflict",
        "cancelled" => "Cancelled",
        "library-version" => "Library version not supported",
        "too-large" => "Too large",
        "unsupported-input" => "Unsupported input",
        "dependency-missing" => "Dependency missing",
        "rate-limited" => "Rate limited",
        "library-busy" => "Library busy",
        "unavailable" => "Unavailable",
        "timeout" => "Timeout",
        "internal" => "Internal error",
        _ => "Plugin error",
    }
}

/// UUIDv4 形状的随机 id，只用来在日志里对上一次请求。
fn call_id() -> String {
    let mut b = [0u8; 16];
    if getrandom::fill(&mut b).is_err() {
        return "00000000-0000-4000-8000-000000000000".into();
    }
    b[6] = (b[6] & 0x0f) | 0x40;
    b[8] = (b[8] & 0x3f) | 0x80;
    let h: String = b.iter().map(|x| format!("{x:02x}")).collect();
    format!(
        "{}-{}-{}-{}-{}",
        &h[0..8],
        &h[8..12],
        &h[12..16],
        &h[16..20],
        &h[20..32]
    )
}
