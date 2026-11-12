//! 「mirroir serve」的 MCP 客户端（协议 §9.3 的包装，经它的 Streamable HTTP：一次 POST 一个 JSON-RPC，
//! 一个 JSON 回答，带 `Authorization: Bearer`，令牌在 `~/.mirroir-mcp/serve-token`）。
//!
//! 只调三个只读的工具：`status`（连没连上）、`check_health`（截一张图试屏幕录制）、
//! `describe_screen`（只要字，不要截图和坐标）。认的字样照 mirroir-mcp v0.47.0 的源码
//! （`InfoTools.swift`、`DescribeScreenFilter.swift`），认不出的一律当「看不出来」。

use std::path::Path;
use std::time::Duration;

use serde_json::{Value, json};

use super::PhoneError;
use crate::models::Net;
use crate::view::{GrantState, Mirroring};

/// 用 `initialize` 那一代的协议：v0.47.0 两代都认（`MCPServer.legacyProtocolVersions`）。
const PROTOCOL: &str = "2025-11-25";
const STATUS_TIMEOUT: Duration = Duration::from_secs(5);
const HEALTH_TIMEOUT: Duration = Duration::from_secs(15);
const DESCRIBE_TIMEOUT: Duration = Duration::from_secs(25);
/// 回答最多多大；只要字的 `describe_screen` 也就几 KiB。
const ANSWER_CAP: usize = 1 << 20;
const TOKEN_CAP: u64 = 4096;

pub(crate) struct Service {
    url: String,
    token_file: std::path::PathBuf,
    net: Net,
}

/// `status` 问到的：服务报的版本（`initialize` 的 `serverInfo.version`）和 `status` 的那句话。
pub(crate) struct Status {
    pub version: Option<String>,
    pub status: String,
}

/// `status` 那句话开头说的镜像窗口（`InfoTools.swift` 的 `status`）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Window {
    Connected,
    Paused,
    NotRunning,
    Gone,
    Minimized,
    Other,
}

pub(crate) fn window(status: &str) -> Window {
    let s = status.trim_start();
    if s.starts_with("Connected") {
        Window::Connected
    } else if s.starts_with("Paused") {
        Window::Paused
    } else if s.starts_with("Not running") {
        Window::NotRunning
    } else if s.starts_with("No window") {
        Window::Gone
    } else if s.starts_with("Window minimized") {
        Window::Minimized
    } else {
        Window::Other
    }
}

pub(crate) fn mirroring(w: Window) -> Mirroring {
    match w {
        Window::Connected => Mirroring::Connected,
        Window::NotRunning => Mirroring::NotSetUp,
        _ => Mirroring::Away,
    }
}

/// (屏幕录制, 辅助功能)。mirroir-mcp 的说明（docs/cli.md「Serve」）：两项都给了以前 `status` 说
/// 「No window」、截图失败。窗口经辅助功能找，所以连着、暂停着都说明辅助功能给了；
/// 截图经屏幕录制，所以截到了说明屏幕录制给了。`captured` 是 `check_health` 截没截到（没问是 `None`）。
pub(crate) fn grants(w: Window, captured: Option<bool>) -> (GrantState, GrantState) {
    use GrantState::{Given, Missing, Unknown};
    match (w, captured) {
        (Window::Connected, _) => (Given, Given),
        (Window::Paused, Some(true)) => (Given, Given),
        (Window::Paused, Some(false)) => (Missing, Given),
        (Window::Paused, None) => (Unknown, Given),
        // 截得到却找不到窗口：缺的是辅助功能
        (Window::Gone, Some(true)) => (Given, Missing),
        _ => (Unknown, Unknown),
    }
}

/// `check_health` 的回答里截图那一行（`[ok] Screen capture working` / `[FAIL] Screen capture failed`）。
pub(crate) fn capture_ok(health: &str) -> Option<bool> {
    health.lines().find_map(|l| {
        let l = l.trim();
        if l.starts_with("[ok] Screen capture") {
            Some(true)
        } else if l.starts_with("[FAIL] Screen capture") {
            Some(false)
        } else {
            None
        }
    })
}

/// `describe_screen` 的 `text_only` 回答里每一段字：`- "<字>"`，后面可以跟 ` [tree]`。
pub(crate) fn texts(answer: &str) -> Vec<String> {
    answer
        .lines()
        .filter_map(|l| {
            let rest = l.trim().strip_prefix("- \"")?;
            let end = rest.rfind('"')?;
            let text = rest[..end].trim();
            (!text.is_empty()).then(|| text.to_owned())
        })
        .collect()
}

impl Service {
    pub(crate) fn new(url: &str, token_file: &Path, net: &Net) -> Service {
        Service {
            url: url.to_owned(),
            token_file: token_file.to_owned(),
            net: net.clone(),
        }
    }

    fn token(&self) -> Result<String, PhoneError> {
        use std::io::Read;
        let file = std::fs::File::open(&self.token_file).map_err(|e| {
            PhoneError::ServiceDown(format!("token file {}: {e}", self.token_file.display()))
        })?;
        let mut token = String::new();
        file.take(TOKEN_CAP)
            .read_to_string(&mut token)
            .map_err(|e| PhoneError::ServiceDown(format!("token file: {e}")))?;
        let token = token.trim().to_owned();
        if token.is_empty() {
            return Err(PhoneError::ServiceDown("the token file is empty".into()));
        }
        Ok(token)
    }

    /// 一次 JSON-RPC：回 `result`。
    async fn rpc(
        &self,
        method: &str,
        params: Value,
        timeout: Duration,
    ) -> Result<Value, PhoneError> {
        let token = self.token()?;
        let client = self
            .net
            .local_client(timeout)
            .map_err(|e| PhoneError::Service(e.to_string()))?;
        let body = json!({"jsonrpc": "2.0", "id": 1, "method": method, "params": params});
        let mut response = client
            .post(&self.url)
            .bearer_auth(token)
            .header("MCP-Protocol-Version", PROTOCOL)
            .header(
                reqwest::header::ACCEPT,
                "application/json, text/event-stream",
            )
            .header(reqwest::header::CONTENT_TYPE, "application/json")
            .body(body.to_string())
            .send()
            .await
            .map_err(|e| {
                if e.is_timeout() {
                    PhoneError::ServiceTimeout
                } else if e.is_connect() {
                    PhoneError::ServiceDown(e.without_url().to_string())
                } else {
                    PhoneError::Service(e.without_url().to_string())
                }
            })?;
        let status = response.status();
        if !status.is_success() {
            return Err(PhoneError::Service(format!(
                "http status {}",
                status.as_u16()
            )));
        }
        let mut bytes = Vec::new();
        while let Some(chunk) = response.chunk().await.map_err(|e| {
            if e.is_timeout() {
                PhoneError::ServiceTimeout
            } else {
                PhoneError::Service(e.without_url().to_string())
            }
        })? {
            if bytes.len() + chunk.len() > ANSWER_CAP {
                return Err(PhoneError::Service(format!(
                    "answer over {ANSWER_CAP} bytes"
                )));
            }
            bytes.extend_from_slice(&chunk);
        }
        let mut answer: Value = serde_json::from_slice(&bytes)
            .map_err(|e| PhoneError::Service(format!("answer is not JSON: {e}")))?;
        if let Some(err) = answer.get("error") {
            let code = err.get("code").and_then(Value::as_i64).unwrap_or_default();
            return Err(PhoneError::Service(format!("json-rpc error {code}")));
        }
        Ok(answer["result"].take())
    }

    /// 调一个工具，回它的字；`isError` 的回 [`PhoneError::Tool`]（它的话里可能有屏上的字，不往外带）。
    async fn tool(
        &self,
        tool: &'static str,
        args: Value,
        timeout: Duration,
    ) -> Result<String, PhoneError> {
        let result = self
            .rpc(
                "tools/call",
                json!({"name": tool, "arguments": args}),
                timeout,
            )
            .await?;
        if result.get("isError").and_then(Value::as_bool) == Some(true) {
            return Err(PhoneError::Tool { tool });
        }
        let text = result
            .get("content")
            .and_then(Value::as_array)
            .ok_or_else(|| PhoneError::Service(format!("{tool}: no content")))?
            .iter()
            .filter(|c| c.get("type").and_then(Value::as_str) == Some("text"))
            .filter_map(|c| c.get("text").and_then(Value::as_str))
            .collect::<Vec<_>>()
            .join("\n");
        Ok(text)
    }

    /// 服务报的版本和 `status`。
    pub(crate) async fn status(&self) -> Result<Status, PhoneError> {
        let init = self
            .rpc(
                "initialize",
                json!({
                    "protocolVersion": PROTOCOL,
                    "capabilities": {},
                    "clientInfo": {"name": "usuyo", "version": env!("CARGO_PKG_VERSION")},
                }),
                STATUS_TIMEOUT,
            )
            .await?;
        let version = init
            .pointer("/serverInfo/version")
            .and_then(Value::as_str)
            .map(str::to_owned);
        let status = self.tool("status", json!({}), STATUS_TIMEOUT).await?;
        Ok(Status { version, status })
    }

    /// `check_health` 截没截到图；回答里没有那一行时是服务的回答看不懂。
    pub(crate) async fn health(&self) -> Result<bool, PhoneError> {
        let text = self.tool("check_health", json!({}), HEALTH_TIMEOUT).await?;
        capture_ok(&text).ok_or_else(|| PhoneError::Service("check_health: no capture line".into()))
    }

    /// 这一屏上的字，从上到下；不要截图、不要坐标。
    pub(crate) async fn screen_texts(&self) -> Result<Vec<String>, PhoneError> {
        let text = self
            .tool(
                "describe_screen",
                json!({"text_only": true, "omit_screenshot": true}),
                DESCRIBE_TIMEOUT,
            )
            .await?;
        Ok(texts(&text))
    }
}
