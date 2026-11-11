//! 本机接口的门：机器上任何网页都能试着访问 127.0.0.1，所以每个请求都要过这里。
//!
//! - `Host` 必须是本机地址加这个端口（挡 DNS rebinding）。
//! - 带 `Origin` 的（浏览器发的）只认名单里的：桌面壳的 webview。`Origin` 谁都能伪造，只用来挡，不用来放行。
//! - 每个请求都要带令牌 `Authorization: Bearer <token>`；令牌在只有自己能读的文件里。
//! - 只有 CORS 预检（带名单里的 `Origin`）和 `POST /handshake` 不带令牌也放行：
//!   预检不碰资料库，握手只回一个证明（见 [`crate::handshake`]）。Host、Origin 的规则照样管它们。
//! - `<img>`、`<video>` 带不了令牌，`/media/<id>/` 下认这一条的 `k`：守护进程每次启动随机生成一个
//!   密钥，按条目 id 算出来。漏出去一个媒体地址，只看得到那一条的图和视频。

use std::sync::Arc;

use axum::extract::{Request, State};
use axum::http::{HeaderMap, Method, header};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};

use crate::handshake;
use crate::problem::Problem;

pub(crate) struct Gate {
    pub port: u16,
    pub token: String,
    pub media_secret: String,
    pub origins: Vec<String>,
}

pub(crate) async fn check(State(gate): State<Arc<Gate>>, req: Request, next: Next) -> Response {
    match gate.admit(&req) {
        Ok(()) => next.run(req).await,
        Err(p) => p.into_response(),
    }
}

impl Gate {
    /// 条目 `id` 的媒体地址带的 `k`。
    pub fn media_key(&self, id: &str) -> String {
        handshake::hmac_hex(&self.media_secret, &format!("usuyo-media/v1\n{id}"))
    }

    fn admit(&self, req: &Request) -> Result<(), Problem> {
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
        let origin = headers.get(header::ORIGIN);
        if let Some(origin) = origin {
            let origin = origin.to_str().unwrap_or_default();
            if !self.origins.iter().any(|o| o == origin) {
                return Err(Problem::new("forbidden", "origin not allowed"));
            }
            if req.method() == Method::OPTIONS
                && headers.contains_key(header::ACCESS_CONTROL_REQUEST_METHOD)
            {
                return Ok(());
            }
        }
        if req.method() == Method::POST && req.uri().path() == handshake::PATH {
            return Ok(());
        }
        if bearer(headers).is_some_and(|t| same(t, &self.token)) {
            return Ok(());
        }
        if let Some(id) = req
            .uri()
            .path()
            .strip_prefix("/media/")
            .and_then(|p| p.split('/').next())
            && media_key(req).is_some_and(|k| same(k, &self.media_key(id)))
        {
            return Ok(());
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
fn same(a: &str, b: &str) -> bool {
    a.len() == b.len()
        && a.bytes()
            .zip(b.bytes())
            .fold(0u8, |acc, (x, y)| acc | (x ^ y))
            == 0
}
