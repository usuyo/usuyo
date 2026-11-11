//! 投递令牌（协议 §5.1）：给浏览器扩展、油猴脚本、手机快捷指令各发一个，只能交链接，读不了资料库。
//!
//! 令牌写成 `<id>.<秘密>`：`id` 不是秘密，握手时客户端只发 `id`，内核用整个令牌当密钥回证明，
//! 令牌本身不上线路。存在按机器目录的 `deliver-tokens.json`（0600），不进资料库；撤销马上生效。

use std::path::{Path, PathBuf};
use std::sync::{PoisonError, RwLock};

use serde::{Deserialize, Serialize};

use crate::handshake::random_hex;

/// 入口类型，决定条目 `origins[].entry`。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ClientKind {
    Extension,
    Mobile,
    Api,
}

impl ClientKind {
    pub fn via(self) -> usuyo_core::Via {
        match self {
            ClientKind::Extension => usuyo_core::Via::Extension,
            ClientKind::Mobile => usuyo_core::Via::Mobile,
            ClientKind::Api => usuyo_core::Via::Api,
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Stored {
    id: String,
    name: String,
    kind: ClientKind,
    secret: String,
    created_at: String,
}

/// 一个投递端，不带秘密。
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Client {
    pub id: String,
    pub name: String,
    pub kind: ClientKind,
    pub created_at: String,
}

impl From<&Stored> for Client {
    fn from(s: &Stored) -> Client {
        Client {
            id: s.id.clone(),
            name: s.name.clone(),
            kind: s.kind,
            created_at: s.created_at.clone(),
        }
    }
}

pub(crate) struct Clients {
    path: Option<PathBuf>,
    list: RwLock<Vec<Stored>>,
}

fn io(path: &Path, source: std::io::Error) -> crate::Error {
    crate::Error::Io {
        path: path.to_owned(),
        source,
    }
}

impl Clients {
    /// 读 `path`（没有就是空的）；`None` 只在内存里，测试用。
    pub fn open(path: Option<PathBuf>) -> Result<Clients, crate::Error> {
        let list = match &path {
            Some(p) => match std::fs::read(p) {
                Ok(b) => serde_json::from_slice(&b)
                    .map_err(|e| io(p, std::io::Error::new(std::io::ErrorKind::InvalidData, e)))?,
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => Vec::new(),
                Err(e) => return Err(io(p, e)),
            },
            None => Vec::new(),
        };
        Ok(Clients {
            path,
            list: RwLock::new(list),
        })
    }

    fn save(&self, list: &[Stored]) -> Result<(), crate::Error> {
        let Some(path) = &self.path else {
            return Ok(());
        };
        let bytes =
            serde_json::to_vec_pretty(list).map_err(|e| io(path, std::io::Error::other(e)))?;
        crate::write_private(path, &bytes)
    }

    pub fn list(&self) -> Vec<Client> {
        self.list
            .read()
            .unwrap_or_else(PoisonError::into_inner)
            .iter()
            .map(Client::from)
            .collect()
    }

    /// 发一个新的；回它和整个令牌（只这一次能看到）。
    pub fn create(&self, name: &str, kind: ClientKind) -> Result<(Client, String), crate::Error> {
        let id = random_hex().map_err(crate::Error::Random)?[..16].to_owned();
        let secret = random_hex().map_err(crate::Error::Random)?;
        let stored = Stored {
            id: id.clone(),
            name: name.to_owned(),
            kind,
            secret: secret.clone(),
            created_at: jiff::Timestamp::now().to_string(),
        };
        let mut list = self.list.write().unwrap_or_else(PoisonError::into_inner);
        list.push(stored.clone());
        self.save(&list)?;
        Ok((Client::from(&stored), format!("{id}.{secret}")))
    }

    /// 撤销；没有这个 id 回 `false`。
    pub fn revoke(&self, id: &str) -> Result<bool, crate::Error> {
        let mut list = self.list.write().unwrap_or_else(PoisonError::into_inner);
        let before = list.len();
        list.retain(|c| c.id != id);
        if list.len() == before {
            return Ok(false);
        }
        self.save(&list)?;
        Ok(true)
    }

    /// 这个令牌是谁的；不认识（或撤销了）回 `None`。比较用常数时间。
    pub fn check(&self, token: &str) -> Option<Client> {
        let (id, secret) = token.split_once('.')?;
        let list = self.list.read().unwrap_or_else(PoisonError::into_inner);
        let c = list.iter().find(|c| c.id == id)?;
        crate::guard::same(secret, &c.secret).then(|| Client::from(c))
    }

    /// 握手用的密钥：这个投递端的整个令牌。
    pub fn token_of(&self, id: &str) -> Option<String> {
        let list = self.list.read().unwrap_or_else(PoisonError::into_inner);
        list.iter()
            .find(|c| c.id == id)
            .map(|c| format!("{}.{}", c.id, c.secret))
    }
}
