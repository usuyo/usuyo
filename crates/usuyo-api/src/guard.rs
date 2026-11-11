//! 本机接口的门：机器上任何网页都能试着访问 127.0.0.1，所以每个请求都要过这里。
//!
//! - `Host` 必须是本机地址加这个端口（挡 DNS rebinding）。
//! - 带 `Origin` 的（浏览器发的）只认名单里的：Tauri 的 webview 和 Vite 开发服务器。
//! - 不带 `Origin` 的（命令行、别的程序）要带令牌 `Authorization: Bearer <token>`。
//! - `<img>`、`<video>` 两样都不带，媒体地址里带守护进程每次启动随机生成的 `k`。

use std::sync::Arc;

use axum::extract::{Request, State};
use axum::http::{HeaderMap, header};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};

use crate::problem::Problem;

pub(crate) struct Gate {
    pub port: u16,
    pub token: String,
    pub media_key: String,
    pub origins: Vec<String>,
}

pub(crate) async fn check(State(gate): State<Arc<Gate>>, req: Request, next: Next) -> Response {
    match gate.admit(&req) {
        Ok(()) => next.run(req).await,
        Err(p) => p.into_response(),
    }
}

impl Gate {
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
        if let Some(origin) = headers.get(header::ORIGIN) {
            let origin = origin.to_str().unwrap_or_default();
            return if self.origins.iter().any(|o| o == origin) {
                Ok(())
            } else {
                Err(Problem::new("forbidden", "origin not allowed"))
            };
        }
        if bearer(headers).is_some_and(|t| same(t, &self.token)) {
            return Ok(());
        }
        if req.uri().path().starts_with("/media/")
            && media_key(req).is_some_and(|k| same(k, &self.media_key))
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

/// 32 字节随机数的十六进制。
pub(crate) fn random_secret() -> Result<String, getrandom::Error> {
    let mut b = [0u8; 32];
    getrandom::fill(&mut b)?;
    Ok(b.iter().map(|x| format!("{x:02x}")).collect())
}
