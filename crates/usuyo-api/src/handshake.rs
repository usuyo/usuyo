//! 找内核的一方先确认对面真是内核，再把令牌发过去。
//!
//! 客户端往 `POST /handshake` 发一个新的随机数 `{"nonce": "<64 位十六进制>"}`，内核回
//! `{"proof": "<十六进制>"}`：以令牌为密钥，对 [`message`] 做 HMAC-SHA256。客户端用自己读到的令牌核对。
//! 令牌本身不上线路；端口算在消息里，别的端口上的程序把随机数转给真内核也拿不到能用的回答。

use hmac::{Hmac, KeyInit, Mac};
use sha2::Sha256;

pub const PATH: &str = "/handshake";
/// 随机数的长度（十六进制字符），32 字节。
pub const NONCE_LEN: usize = 64;
const DOMAIN: &str = "usuyo-daemon-proof/v1";

type HmacSha256 = Hmac<Sha256>;

/// 签的是哪几个字节：`usuyo-daemon-proof/v1\n127.0.0.1:<port>\n<nonce>`。
pub fn message(port: u16, nonce: &str) -> String {
    format!("{DOMAIN}\n127.0.0.1:{port}\n{nonce}")
}

fn mac(key: &str) -> HmacSha256 {
    // HMAC 接受任意长度的密钥，这里不会失败。
    <HmacSha256 as KeyInit>::new_from_slice(key.as_bytes()).expect("HMAC takes keys of any length")
}

/// 内核这边：回给客户端的证明。
pub fn proof(token: &str, port: u16, nonce: &str) -> String {
    let mut m = mac(token);
    m.update(message(port, nonce).as_bytes());
    hex(&m.finalize().into_bytes())
}

/// 客户端这边：对面回的证明对不对；比较用常数时间。
pub fn verify(token: &str, port: u16, nonce: &str, proof: &str) -> bool {
    let Some(bytes) = unhex(proof) else {
        return false;
    };
    let mut m = mac(token);
    m.update(message(port, nonce).as_bytes());
    m.verify_slice(&bytes).is_ok()
}

/// 新的随机数。
pub fn new_nonce() -> Result<String, getrandom::Error> {
    random_hex()
}

/// 合格的随机数：正好 64 位小写十六进制。
pub fn valid_nonce(nonce: &str) -> bool {
    nonce.len() == NONCE_LEN
        && nonce
            .bytes()
            .all(|b| matches!(b, b'0'..=b'9' | b'a'..=b'f'))
}

/// 32 字节随机数的十六进制。
pub(crate) fn random_hex() -> Result<String, getrandom::Error> {
    let mut b = [0u8; 32];
    getrandom::fill(&mut b)?;
    Ok(hex(&b))
}

/// 以 `key` 为密钥对 `msg` 做 HMAC-SHA256，十六进制。
#[cfg(feature = "server")]
pub(crate) fn hmac_hex(key: &str, msg: &str) -> String {
    let mut m = mac(key);
    m.update(msg.as_bytes());
    hex(&m.finalize().into_bytes())
}

fn hex(b: &[u8]) -> String {
    b.iter().map(|x| format!("{x:02x}")).collect()
}

fn unhex(s: &str) -> Option<Vec<u8>> {
    if !s.len().is_multiple_of(2) {
        return None;
    }
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(s.get(i..i + 2)?, 16).ok())
        .collect()
}

/// 核对失败的两种：对面没人，或者对面不是内核。
#[cfg(feature = "client")]
#[derive(Debug, thiserror::Error)]
pub enum ProveError {
    #[error("nothing answers on 127.0.0.1:{port}: {source}")]
    Unreachable {
        port: u16,
        #[source]
        source: Box<ureq::Error>,
    },
    #[error("the program on 127.0.0.1:{port} is not the usuyo daemon: {why}")]
    Stranger { port: u16, why: String },
    #[error("system random source failed: {0}")]
    Random(getrandom::Error),
}

/// 问 127.0.0.1:`port` 要证明并核对。只发随机数，不发令牌；`agent` 由调用方定超时。
#[cfg(feature = "client")]
pub fn prove(agent: &ureq::Agent, port: u16, token: &str) -> Result<(), ProveError> {
    let nonce = new_nonce().map_err(ProveError::Random)?;
    let stranger = |why: String| ProveError::Stranger { port, why };
    let body = serde_json::json!({ "nonce": nonce }).to_string();
    let res = agent
        .post(format!("http://127.0.0.1:{port}{PATH}"))
        .header("Content-Type", "application/json")
        .send(body);
    let mut res = match res {
        Ok(r) => r,
        Err(e) if is_transport(&e) => {
            return Err(ProveError::Unreachable {
                port,
                source: Box::new(e),
            });
        }
        Err(e) => return Err(stranger(e.to_string())),
    };
    if !res.status().is_success() {
        return Err(stranger(format!("answered {}", res.status())));
    }
    let text = res
        .body_mut()
        .with_config()
        .limit(1024)
        .read_to_string()
        .map_err(|e| stranger(e.to_string()))?;
    let value: serde_json::Value =
        serde_json::from_str(&text).map_err(|_| stranger("answer is not JSON".into()))?;
    let proof = value["proof"].as_str().unwrap_or_default();
    if verify(token, port, &nonce, proof) {
        Ok(())
    } else {
        Err(stranger("wrong proof".into()))
    }
}

/// 连不上（没人在听、超时）和对面答错了分开。
#[cfg(feature = "client")]
fn is_transport(e: &ureq::Error) -> bool {
    matches!(
        e,
        ureq::Error::Io(_) | ureq::Error::ConnectionFailed | ureq::Error::Timeout(_)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 和 `apps/desktop/web/proxy/proof.test.ts` 用同一组值（也和 `openssl dgst -sha256 -hmac` 对过）。
    #[test]
    fn fixed_vector() {
        assert_eq!(
            proof("test-token", 47615, &"0".repeat(64)),
            "fc597e72ae56d3d9ba293b2e557a6df8a9d0736968d89fed4b7bc79571475776"
        );
    }

    #[test]
    fn right_token_verifies_wrong_token_fails() {
        let nonce = new_nonce().unwrap();
        assert!(valid_nonce(&nonce));
        let p = proof("right", 47615, &nonce);
        assert!(verify("right", 47615, &nonce, &p));
        assert!(!verify("wrong", 47615, &nonce, &p));
        assert!(!verify("right", 47616, &nonce, &p), "port is bound in");
        assert!(!verify("right", 47615, &new_nonce().unwrap(), &p));
        assert!(!verify("right", 47615, &nonce, "zz"));
        assert!(!verify("right", 47615, &nonce, ""));
    }

    #[test]
    fn nonce_shape() {
        assert!(!valid_nonce("abc"));
        assert!(!valid_nonce(&"A".repeat(64)));
        assert!(!valid_nonce(&"0".repeat(65)));
    }
}
