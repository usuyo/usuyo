//! 本机接口的门：机器上任何网页都能试着访问 127.0.0.1，所以每个请求都要过这里。
//!
//! - `Host` 必须是本机地址加这个端口（挡 DNS rebinding）。
//! - 带 `Origin` 的（浏览器发的）只认名单里的：桌面壳的 webview。`Origin` 谁都能伪造，只用来挡，不用来放行。
//! - 每个请求都要带令牌 `Authorization: Bearer <token>`；令牌在只有自己能读的文件里。
//! - 只有 CORS 预检（带名单里的 `Origin`）和 `POST /handshake` 不带令牌也放行：
//!   预检不碰资料库，握手只回一个证明（见 [`crate::handshake`]）。Host、Origin 的规则照样管它们。
//! - `<img>`、`<video>` 带不了令牌，`/media/<id>/` 下认这一条的 `k`：守护进程每次启动随机生成一个
//!   密钥，按条目 id 算出来。漏出去一个媒体地址，只看得到那一条的图和视频。
//! - `/v1/` 是投递接口（协议 §5.1）：`Origin` 只认浏览器扩展的（`chrome-extension://` 一类）或者不带；
//!   握手（`/handshake`）也认扩展的来源；
//!   投递令牌只能进 [`DELIVER_PATHS`] 这几条，别的一律 401（名单以外都不放，不是挡几条）。

use std::sync::Arc;

use axum::extract::{Request, State};
use axum::http::{HeaderMap, Method, header};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};

use crate::clients::{Client, Clients};
use crate::handshake;
use crate::problem::Problem;

/// 投递令牌能进的路径，只有这几条。
pub(crate) const DELIVER_PATHS: [&str; 4] =
    ["/v1/ping", "/v1/deliver", "/v1/batches", "/v1/file-imports"];

/// 浏览器扩展的来源（协议 §5.1）：只有它们能带 `Origin` 访问 `/v1/`。
const EXTENSION_SCHEMES: [&str; 3] = [
    "chrome-extension://",
    "moz-extension://",
    "safari-web-extension://",
];

pub(crate) struct Gate {
    pub port: u16,
    pub token: String,
    pub media_secret: String,
    pub origins: Vec<String>,
    pub clients: Clients,
}

/// 谁在调：带主令牌的（壳、命令行、开发代理），或一个投递端。处理函数从请求的扩展里拿。
#[derive(Debug, Clone)]
pub(crate) enum Caller {
    Owner,
    Deliver(Client),
    /// 没带令牌也放行的：预检、握手、带 `k` 的媒体。
    Anonymous,
}

pub(crate) async fn check(State(gate): State<Arc<Gate>>, mut req: Request, next: Next) -> Response {
    match gate.admit(&req) {
        Ok(caller) => {
            req.extensions_mut().insert(caller);
            next.run(req).await
        }
        Err(p) => p.into_response(),
    }
}

impl Gate {
    /// 条目 `id` 的媒体地址带的 `k`。
    pub fn media_key(&self, id: &str) -> String {
        handshake::hmac_hex(&self.media_secret, &format!("usuyo-media/v1\n{id}"))
    }

    fn admit(&self, req: &Request) -> Result<Caller, Problem> {
        let headers = req.headers();
        let host = headers
            .get(header::HOST)
            .and_then(|h| h.to_str().ok())
            .unwrap_or_default();
        let port = self.port;
        let local = [
            format!("127.0.0.1:{port}"),
            format!("localhost:{port}"),
            format!("[::1]:{port}"),
        ];
        if !local.iter().any(|h| h.eq_ignore_ascii_case(host)) {
            return Err(Problem::new("forbidden", "unexpected Host header"));
        }
        let path = req.uri().path();
        let deliver = path.starts_with("/v1/");
        let origin = headers.get(header::ORIGIN);
        if let Some(origin) = origin {
            let origin = origin.to_str().unwrap_or_default();
            let extension = EXTENSION_SCHEMES.iter().any(|s| origin.starts_with(s));
            let window = self.origins.iter().any(|o| o == origin);
            // 握手两边都要：扩展先确认对面是内核，再发投递令牌
            let allowed = if deliver {
                extension
            } else if path == handshake::PATH {
                extension || window
            } else {
                window
            };
            if !allowed {
                return Err(Problem::new("forbidden", "origin not allowed"));
            }
            if !deliver
                && req.method() == Method::OPTIONS
                && headers.contains_key(header::ACCESS_CONTROL_REQUEST_METHOD)
            {
                return Ok(Caller::Anonymous);
            }
        }
        if req.method() == Method::POST && path == handshake::PATH {
            return Ok(Caller::Anonymous);
        }
        if let Some(t) = bearer(headers) {
            if same(t, &self.token) {
                return Ok(Caller::Owner);
            }
            if DELIVER_PATHS.contains(&path)
                && let Some(c) = self.clients.check(t)
            {
                return Ok(Caller::Deliver(c));
            }
        }
        if let Some(id) = req
            .uri()
            .path()
            .strip_prefix("/media/")
            .and_then(|p| p.split('/').next())
            && media_key(req).is_some_and(|k| same(k, &self.media_key(id)))
        {
            return Ok(Caller::Anonymous);
        }
        Err(Problem::new("unauthorized", "missing or wrong token"))
    }
}

fn bearer(headers: &HeaderMap) -> Option<&str> {
    headers
        .get(header::AUTHORIZATION)?
        .to_str()
        .ok()?
        .strip_prefix("Bearer ")
}

fn media_key(req: &Request) -> Option<&str> {
    req.uri()
        .query()?
        .split('&')
        .find_map(|kv| kv.strip_prefix("k="))
}

/// 比较不因先错的那一位提早结束，免得靠时间猜令牌。
pub(crate) fn same(a: &str, b: &str) -> bool {
    a.len() == b.len()
        && a.bytes()
            .zip(b.bytes())
            .fold(0u8, |acc, (x, y)| acc | (x ^ y))
            == 0
}
