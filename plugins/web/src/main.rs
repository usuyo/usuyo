//! 预装的通用网页获取插件 `builtin/web`（协议 §8.4）：只读公开网页，不登录，
//! 没有任何站点专用的规则。子命令照协议 §3.2：`canonicalize`、`fetch`、`health`。

mod page;

use std::fmt;
use std::path::PathBuf;
use std::process::ExitCode;

use serde::{Deserialize, Serialize};
use usuyo_plugin_kit::http::Policy;
use usuyo_plugin_kit::protocol::{
    CallFailure, Canonicalized, Envelope, ErrorCode, Identity, OutputKind,
};

/// 插件能报的失败，带着协议的错误对象穿过 `anyhow`，在入口变回信封。
#[derive(Debug)]
struct Fail(CallFailure);

impl fmt::Display for Fail {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl std::error::Error for Fail {}

fn fail(code: ErrorCode, message: impl Into<String>) -> anyhow::Error {
    Fail(CallFailure::new(code, message)).into()
}

const USAGE: &str = "usage: usuyo-plugin-web canonicalize <url> | fetch <url> --out <dir> [--want a,b] | health [--probe]";

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let printed = match args.first().map(String::as_str) {
        Some("canonicalize") if args.len() == 2 => print(canonicalize(&args[1])),
        Some("fetch") => match parse_fetch(&args[1..]) {
            Some((url, out, want)) => print(settings().and_then(|s| {
                page::fetch(
                    &url,
                    &out,
                    want.as_ref(),
                    s.allow_private_network,
                    &Policy::strict(),
                )
            })),
            None => usage(),
        },
        Some("health") if args.len() <= 2 && args.get(1).is_none_or(|a| a == "--probe") => {
            print(Ok(serde_json::json!({
                "ready": true,
                "protocol": 1,
                "version": env!("CARGO_PKG_VERSION"),
                "upstream": [],
                "login": [],
                "checks": []
            })))
        }
        _ => usage(),
    };
    ExitCode::from(printed)
}

/// 插件设置（协议 §2.11），宿主经 `USUYO_SETTINGS` 指向的 JSON 文件传入；没有这个变量就全用默认值。
#[derive(Debug, Default, Deserialize)]
struct Settings {
    /// 要取的链接本身可以是本机或内网地址。图片和跳转目标不受影响。
    #[serde(rename = "allow-private-network", default)]
    allow_private_network: bool,
}

fn settings() -> anyhow::Result<Settings> {
    let Some(path) = std::env::var_os("USUYO_SETTINGS") else {
        return Ok(Settings::default());
    };
    let bytes = std::fs::read(&path).map_err(|e| {
        fail(
            ErrorCode::InvalidArgs,
            format!(
                "cannot read USUYO_SETTINGS {}: {e}",
                PathBuf::from(&path).display()
            ),
        )
    })?;
    serde_json::from_slice(&bytes)
        .map_err(|e| fail(ErrorCode::InvalidArgs, format!("bad USUYO_SETTINGS: {e}")))
}

fn usage() -> u8 {
    eprintln!("{USAGE}");
    2
}

/// 把结果写成 stdout 上的那一个信封，返回退出码。
fn print<T: Serialize>(result: anyhow::Result<T>) -> u8 {
    let envelope = match result {
        Ok(body) => Envelope::Ok(body),
        Err(e) => match e.downcast::<Fail>() {
            Ok(Fail(failure)) => Envelope::Err(failure),
            Err(e) => Envelope::Err(CallFailure::new(ErrorCode::Internal, format!("{e:#}"))),
        },
    };
    match serde_json::to_string(&envelope) {
        Ok(json) => {
            println!("{json}");
            if envelope.exit_code() == 0 { 0 } else { 1 }
        }
        Err(e) => {
            eprintln!("cannot write the envelope: {e}");
            1
        }
    }
}

fn parse_fetch(args: &[String]) -> Option<(String, PathBuf, Option<Vec<OutputKind>>)> {
    let (url, rest) = args.split_first()?;
    let mut out = None;
    let mut want = None;
    let mut it = rest.iter();
    while let Some(flag) = it.next() {
        match flag.as_str() {
            "--out" => out = Some(PathBuf::from(it.next()?)),
            "--want" => {
                want = Some(
                    it.next()?
                        .split(',')
                        .map(OutputKind::parse)
                        .collect::<Option<Vec<_>>>()?,
                );
            }
            _ => return None,
        }
    }
    Some((url.clone(), out?, want))
}

/// 认身份：任何 http(s) 链接都是自己的。不发请求：通用网页分不出哪些是短链。
fn canonicalize(input: &str) -> anyhow::Result<Canonicalized> {
    Ok(match web_identity(input) {
        Some(identity) => Canonicalized::Match {
            identity,
            resolved_url: None,
        },
        None => Canonicalized::NoMatch,
    })
}

/// 跟踪参数：不分站点的通用那几种。
const TRACKING_PARAMS: [&str; 16] = [
    "fbclid", "gclid", "dclid", "gbraid", "wbraid", "msclkid", "yclid", "twclid", "ttclid",
    "igshid", "mc_cid", "mc_eid", "_hsenc", "_hsmi", "_ga", "_gl",
];

/// `web:` 身份的链接：去掉 `#` 片段和跟踪参数，其余查询串原样留着（SPEC §0.4）。
/// fetch 用同一个函数，所以两边给的身份总是一样。
pub(crate) fn web_identity(input: &str) -> Option<Identity> {
    let mut url = url::Url::parse(input).ok()?;
    if !matches!(url.scheme(), "http" | "https") || url.host_str().is_none() {
        return None;
    }
    url.set_fragment(None);
    if let Some(query) = url.query() {
        let kept: Vec<&str> = query
            .split('&')
            .filter(|pair| {
                let key = pair.split('=').next().unwrap_or_default();
                !(key.is_empty() || key.starts_with("utm_") || TRACKING_PARAMS.contains(&key))
            })
            .collect();
        let kept = kept.join("&");
        url.set_query((!kept.is_empty()).then_some(kept.as_str()));
    }
    Some(Identity::Web { url: url.into() })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn url_of(input: &str) -> Option<String> {
        match web_identity(input)? {
            Identity::Web { url } => Some(url),
            _ => None,
        }
    }

    #[test]
    fn identities() {
        assert_eq!(
            url_of("HTTPS://Example.COM:443/a?utm_source=x&id=1&fbclid=2#top").as_deref(),
            Some("https://example.com/a?id=1")
        );
        assert_eq!(
            url_of("https://example.com/a?utm_source=x").as_deref(),
            Some("https://example.com/a")
        );
        assert_eq!(
            url_of("https://example.com/s?q=a+b%20c").as_deref(),
            Some("https://example.com/s?q=a+b%20c")
        );
        assert_eq!(url_of("ftp://example.com/a"), None);
        assert_eq!(url_of("not a url"), None);
    }

    #[test]
    fn fetch_args() {
        let args: Vec<String> = ["https://a.b/", "--out", "/tmp/o", "--want", "text,images"]
            .map(String::from)
            .to_vec();
        let (_, out, want) = parse_fetch(&args).unwrap();
        assert_eq!(out, PathBuf::from("/tmp/o"));
        assert_eq!(want, Some(vec![OutputKind::Text, OutputKind::Images]));
        let bad: Vec<String> = ["https://a.b/", "--want", "video"]
            .map(String::from)
            .to_vec();
        assert!(parse_fetch(&bad).is_none());
    }
}
