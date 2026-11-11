//! 守护进程在跑时，命令经它的 HTTP 接口走，带 0600 文件里的令牌。

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
    /// `daemon.json` 在、令牌对、`/health` 回 200 才算守护进程在跑。
    pub fn find(root: &Path) -> Result<Option<Client>> {
        let Ok(bytes) = std::fs::read(machine::daemon_path(root)) else {
            return Ok(None);
        };
        let info: Value = serde_json::from_slice(&bytes).context("bad daemon.json")?;
        let Some(port) = info.get("port").and_then(Value::as_u64) else {
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
        let client = Client {
            base: format!("http://127.0.0.1:{port}"),
            token: token.trim().to_owned(),
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
