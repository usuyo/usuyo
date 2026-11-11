//! 守护进程在跑时，命令经它的 HTTP 接口走，带 0600 文件里的令牌。
//! 发令牌以前先握手（[`usuyo_api::handshake`]）：对面拿不出这次启动的证明，就不是守护进程。

use std::path::Path;
use std::time::Duration;

use anyhow::{Context, Result, bail};
use serde_json::Value;

use crate::machine;

pub struct Client {
    base: String,
    token: String,
    agent: ureq::Agent,
}

impl Client {
    /// `daemon.json` 在、对面握手核得上、`/health` 回 200 才算守护进程在跑。
    pub fn find(root: &Path) -> Result<Option<Client>> {
        let Ok(bytes) = std::fs::read(machine::daemon_path(root)) else {
            return Ok(None);
        };
        let info: Value = serde_json::from_slice(&bytes).context("bad daemon.json")?;
        let Some(port) = info
            .get("port")
            .and_then(Value::as_u64)
            .and_then(|p| u16::try_from(p).ok())
        else {
            return Ok(None);
        };
        let Ok(token) = std::fs::read_to_string(machine::token_path(root)) else {
            return Ok(None);
        };
        let agent: ureq::Agent = ureq::Agent::config_builder()
            .timeout_global(Some(Duration::from_secs(30)))
            .timeout_connect(Some(Duration::from_secs(2)))
            .http_status_as_error(false)
            .proxy(None)
            .build()
            .into();
        let token = token.trim().to_owned();
        let probe: ureq::Agent = ureq::Agent::config_builder()
            .timeout_global(Some(Duration::from_secs(2)))
            .http_status_as_error(false)
            .proxy(None)
            .build()
            .into();
        if let Err(e) = usuyo_api::handshake::prove(&probe, port, &token) {
            // 没人在听，或者占着端口的不是守护进程：照不在跑办，令牌不发出去。
            tracing::debug!(error = %e, "no daemon");
            return Ok(None);
        }
        let client = Client {
            base: format!("http://127.0.0.1:{port}"),
            token,
            agent,
        };
        match client.get("/health", &[]) {
            Ok(_) => Ok(Some(client)),
            // 连不上：上次没正常退出留下的 daemon.json，照不在跑办。
            Err(_) => Ok(None),
        }
    }

    pub fn get(&self, path: &str, query: &[(&str, &str)]) -> Result<Value> {
        let url = format!("{}{path}", self.base);
        let mut req = self.agent.get(&url).header("Authorization", self.auth());
        for (k, v) in query {
            req = req.query(*k, *v);
        }
        Self::read(req.call()?)
    }

    pub fn post(&self, path: &str, body: Option<&Value>) -> Result<Value> {
        let url = format!("{}{path}", self.base);
        let req = self.agent.post(&url).header("Authorization", self.auth());
        Self::read(match body {
            Some(b) => req
                .header("Content-Type", "application/json")
                .send(b.to_string())?,
            None => req.send_empty()?,
        })
    }

    fn auth(&self) -> String {
        format!("Bearer {}", self.token)
    }

    fn read(mut res: ureq::http::Response<ureq::Body>) -> Result<Value> {
        let status = res.status();
        let text = res.body_mut().read_to_string()?;
        let value: Value = serde_json::from_str(&text)
            .with_context(|| format!("daemon sent non-JSON ({status})"))?;
        if !status.is_success() {
            let code = value["code"].as_str().unwrap_or("internal");
            let detail = value["detail"].as_str().unwrap_or_default();
            bail!("{code}: {detail}");
        }
        Ok(value)
    }
}

#[cfg(test)]
mod tests {
    use std::io::{Read, Write};
    use std::net::TcpListener;
    use std::sync::{Arc, Mutex};

    use super::*;

    /// 占着端口、什么都答 200 的程序；记下它收到的全部字节。
    fn impostor() -> (u16, Arc<Mutex<Vec<u8>>>) {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        let seen = Arc::new(Mutex::new(Vec::new()));
        let log = Arc::clone(&seen);
        std::thread::spawn(move || {
            for stream in listener.incoming() {
                let Ok(mut s) = stream else { return };
                s.set_read_timeout(Some(Duration::from_millis(300)))
                    .unwrap();
                let mut buf = [0u8; 4096];
                while let Ok(n) = s.read(&mut buf) {
                    if n == 0 {
                        break;
                    }
                    log.lock().unwrap().extend_from_slice(&buf[..n]);
                    if buf[..n].windows(4).any(|w| w == b"\r\n\r\n") {
                        break;
                    }
                }
                let body = format!(r#"{{"status":"ok","proof":"{}"}}"#, "0".repeat(64));
                let _ = write!(
                    s,
                    "HTTP/1.1 200 OK\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{body}",
                    body.len()
                );
            }
        });
        (port, seen)
    }

    #[test]
    fn impostor_never_gets_the_token() {
        let root = tempfile::tempdir().unwrap();
        let (port, seen) = impostor();
        let token = "secret-token-for-this-test";
        std::fs::write(machine::token_path(root.path()), token).unwrap();
        std::fs::write(
            machine::daemon_path(root.path()),
            format!(r#"{{"port":{port},"pid":1}}"#),
        )
        .unwrap();

        assert!(Client::find(root.path()).unwrap().is_none());
        let seen = String::from_utf8_lossy(&seen.lock().unwrap()).to_lowercase();
        assert!(seen.contains("post /handshake"), "{seen}");
        assert!(!seen.contains(token), "{seen}");
        assert!(!seen.contains("authorization"), "{seen}");
    }
}
