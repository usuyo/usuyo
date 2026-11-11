//! 通用网页插件走一遍宿主：清单 → match → canonicalize → fetch → 校验过的输出目录。
//! 对着本机的 HTTP 服务跑，不碰真网络。

use std::io::{BufRead, BufReader, Write};
use std::net::TcpListener;
use std::path::PathBuf;

use tempfile::TempDir;
use usuyo_plugin::protocol::{
    Canonicalized, Completeness, ErrorCode, Identity, NotFoundReason, OutputKind,
};
use usuyo_plugin::{
    CallLimits, CallOptions, FetchInput, FetchRequest, FetchedSnapshot, HostConfig, PluginError,
    PluginHost, discover,
};

const ARTICLE: &str = r#"<!DOCTYPE html>
<html lang="zh"><head>
<meta charset="utf-8">
<title>手冲咖啡：三段式注水 - 咖啡日常</title>
<meta name="author" content="咖啡日常">
<meta property="article:published_time" content="2026-10-01T20:00:00+08:00">
<meta property="og:description" content="三段式注水的做法和要点">
<meta property="og:image" content="/img/lead.png">
<link rel="canonical" href="/posts/pour-over">
</head><body>
<nav><a href="/">首页</a> <a href="/about">关于</a></nav>
<article>
<h1>手冲咖啡：三段式注水</h1>
<p>三段式注水是一种常见的手冲做法。第一段用 40 克水闷蒸 30 秒，让咖啡粉充分排气，这一步决定了后面萃取是否均匀，水温控制在 92 度左右比较合适，粉的研磨度接近细砂糖。</p>
<p><img src="/img/a.png" alt="闷蒸"></p>
<p>第二段注水到 150 克，水流保持细而稳定，从中心向外画圈，不要冲到滤纸边缘，否则水会绕过粉层直接流下去，导致萃取不足，味道发酸发薄，这是新手最容易犯的错误。</p>
<p><img src="/img/missing.png" alt="缺的图"></p>
<p>第三段注水到 225 克，总时间控制在两分半左右。如果流速太慢，下次把粉磨粗一点；太快就磨细一点。记录每次的参数，慢慢就能找到自己喜欢的味道，这比照搬别人的配方更可靠。</p>
</article>
<footer>版权所有</footer>
</body></html>"#;

const PNG: &[u8] = b"\x89PNG\r\n\x1a\nfake";

/// 起一个只回固定内容的 HTTP 服务，返回它的地址。
fn serve() -> String {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = format!("http://{}", listener.local_addr().unwrap());
    std::thread::spawn(move || {
        for stream in listener.incoming() {
            let Ok(mut stream) = stream else { continue };
            std::thread::spawn(move || {
                let mut reader = BufReader::new(stream.try_clone().unwrap());
                let mut line = String::new();
                reader.read_line(&mut line).unwrap();
                let path = line.split_whitespace().nth(1).unwrap_or("/").to_owned();
                loop {
                    let mut header = String::new();
                    if reader.read_line(&mut header).unwrap_or(0) == 0 || header == "\r\n" {
                        break;
                    }
                }
                let (status, ctype, body, extra): (&str, &str, Vec<u8>, &str) = match path.as_str()
                {
                    "/article?utm_source=feed" | "/article" => {
                        ("200 OK", "text/html; charset=utf-8", ARTICLE.into(), "")
                    }
                    "/redirect" => (
                        "302 Found",
                        "text/plain",
                        Vec::new(),
                        "Location: /article\r\n",
                    ),
                    "/gbk" => {
                        let page = ARTICLE.replace("utf-8", "gbk");
                        let bytes = encoding_rs::GBK.encode(&page).0.into_owned();
                        ("200 OK", "text/html", bytes, "")
                    }
                    "/img/a.png" | "/img/lead.png" => ("200 OK", "image/png", PNG.to_vec(), ""),
                    "/gone" => ("410 Gone", "text/html", b"gone".to_vec(), ""),
                    "/limited" => (
                        "429 Too Many Requests",
                        "text/html",
                        Vec::new(),
                        "Retry-After: 120\r\n",
                    ),
                    "/pdf" => ("200 OK", "application/pdf", b"%PDF-1.7".to_vec(), ""),
                    "/big" => ("200 OK", "text/html", vec![b'a'; 17 << 20], ""),
                    _ => ("404 Not Found", "text/html", b"nope".to_vec(), ""),
                };
                let head = format!(
                    "HTTP/1.1 {status}\r\nContent-Type: {ctype}\r\nContent-Length: {}\r\nConnection: close\r\n{extra}\r\n",
                    body.len()
                );
                stream.write_all(head.as_bytes()).ok();
                stream.write_all(&body).ok();
            });
        }
    });
    addr
}

struct Fixture {
    tmp: TempDir,
    host: PluginHost,
    base: String,
}

/// 照预装的样子装好：`<root>/builtin/web/<version>/`，清单就是仓库里那份。
fn install() -> Fixture {
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp.path().join("plugins/builtin/web/0.0.0");
    std::fs::create_dir_all(&dir).unwrap();
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("usuyo-plugin.yaml");
    std::fs::copy(manifest, dir.join("usuyo-plugin.yaml")).unwrap();
    std::fs::copy(
        env!("CARGO_BIN_EXE_usuyo-plugin-web"),
        dir.join("usuyo-plugin-web"),
    )
    .unwrap();
    let found = discover(&tmp.path().join("plugins")).unwrap();
    assert!(found.rejected.is_empty(), "{:?}", found.rejected);
    let host = PluginHost::new(
        HostConfig {
            work_dir: tmp.path().join("work"),
            cache_dir: tmp.path().join("cache"),
            locale: "zh-CN".into(),
            limits: CallLimits::default(),
        },
        found.fetch,
    );
    Fixture {
        tmp,
        host,
        base: serve(),
    }
}

impl Fixture {
    async fn fetch(
        &self,
        path: &str,
        want: Option<&[OutputKind]>,
    ) -> Result<FetchedSnapshot, PluginError> {
        let url = url::Url::parse(&format!("{}{path}", self.base)).unwrap();
        let plugin = self.host.candidates(&url)[0];
        let request = FetchRequest {
            input: FetchInput::Url(url),
            want: want.map(|w| w.iter().copied().collect()),
            input_file: None,
        };
        self.host
            .fetch(plugin, request, &CallOptions::default())
            .await
    }
}

fn failure(result: Result<FetchedSnapshot, PluginError>) -> usuyo_plugin::protocol::CallFailure {
    match result {
        Err(PluginError::Call { failure, .. }) => *failure,
        other => panic!("expected a failure, got {other:?}"),
    }
}

#[tokio::test]
async fn fetches_an_article() {
    let f = install();
    let input = format!("{}/article?utm_source=feed", f.base);
    let url = url::Url::parse(&input).unwrap();
    let plugin = f.host.candidates(&url)[0];
    assert_eq!(plugin.id().as_str(), "builtin/web");

    let canonical = f
        .host
        .canonicalize(plugin, &url, &CallOptions::default())
        .await
        .unwrap();
    let Canonicalized::Match {
        identity,
        resolved_url: None,
    } = canonical
    else {
        panic!("{canonical:?}");
    };
    assert_eq!(
        identity,
        Identity::Web {
            url: format!("{}/article", f.base)
        }
    );

    let snap = f.fetch("/article?utm_source=feed", None).await.unwrap();
    let out = &snap.output;
    // fetch 给的身份和 canonicalize 一样，同一条链接再收不会变成两条
    assert_eq!(out.identity.as_ref(), Some(&identity));
    assert_eq!(out.completeness, Completeness::Full, "{:?}", out.missing);
    assert_eq!(
        snap.obtained,
        [
            OutputKind::Text,
            OutputKind::Html,
            OutputKind::Images,
            OutputKind::Meta
        ]
        .into()
    );
    let text = out.text.as_deref().unwrap();
    assert!(text.contains("第一段用 40 克水闷蒸 30 秒"), "{text}");
    assert!(!text.contains("版权所有"), "{text}");
    assert!(
        out.title
            .as_deref()
            .unwrap()
            .starts_with("手冲咖啡：三段式注水")
    );
    assert_eq!(out.author.as_ref().unwrap().name, "咖啡日常");
    assert_eq!(
        out.published_at.as_ref().unwrap().as_str(),
        "2026-10-01T20:00:00+08:00"
    );
    assert_eq!(out.evidence.status, Some(200));

    let dir = snap.out_dir();
    assert_eq!(
        std::fs::read_to_string(dir.join("page.html")).unwrap(),
        ARTICLE
    );
    assert_eq!(out.images.len(), 1);
    assert_eq!(
        std::fs::read(dir.join(out.images[0].as_str())).unwrap(),
        PNG
    );
    let meta: serde_json::Value =
        serde_json::from_slice(&std::fs::read(dir.join("meta.json")).unwrap()).unwrap();
    assert_eq!(meta["description"], "三段式注水的做法和要点");
    assert_eq!(meta["leadImage"], format!("{}/img/lead.png", f.base));
    assert_eq!(meta["canonicalUrl"], format!("{}/posts/pour-over", f.base));
    assert_eq!(
        meta["failedImages"][0],
        format!("{}/img/missing.png", f.base)
    );
    let files: Vec<_> = snap.files.iter().map(|f| f.path.as_str()).collect();
    assert_eq!(files, ["images/001.png", "meta.json", "page.html"]);
    drop(f.tmp);
}

#[tokio::test]
async fn want_limits_what_is_written() {
    let f = install();
    let snap = f
        .fetch("/article", Some(&[OutputKind::Text]))
        .await
        .unwrap();
    assert!(snap.output.html.is_none());
    assert!(snap.output.images.is_empty());
    assert!(snap.files.is_empty(), "{:?}", snap.files);
    assert_eq!(snap.obtained, [OutputKind::Text].into());
}

#[tokio::test]
async fn follows_redirects_and_keeps_the_input_identity() {
    let f = install();
    let snap = f.fetch("/redirect", None).await.unwrap();
    assert_eq!(
        snap.output.identity,
        Some(Identity::Web {
            url: format!("{}/redirect", f.base)
        })
    );
    let meta: serde_json::Value =
        serde_json::from_slice(&std::fs::read(snap.out_dir().join("meta.json")).unwrap()).unwrap();
    assert_eq!(meta["finalUrl"], format!("{}/article", f.base));
}

#[tokio::test]
async fn decodes_gbk() {
    let f = install();
    let snap = f
        .fetch("/gbk", Some(&[OutputKind::Text, OutputKind::Meta]))
        .await
        .unwrap();
    assert!(snap.output.text.as_deref().unwrap().contains("闷蒸"));
}

#[tokio::test]
async fn maps_http_failures() {
    let f = install();
    let gone = failure(f.fetch("/gone", None).await);
    assert_eq!(
        (gone.code, gone.reason),
        (ErrorCode::NotFound, Some(NotFoundReason::Deleted))
    );
    assert_eq!(
        failure(f.fetch("/nothing", None).await).code,
        ErrorCode::NotFound
    );
    let limited = failure(f.fetch("/limited", None).await);
    assert_eq!(
        (limited.code, limited.retry_after),
        (ErrorCode::RateLimited, Some(120))
    );
    assert_eq!(
        failure(f.fetch("/pdf", None).await).code,
        ErrorCode::UnsupportedInput
    );
    assert_eq!(
        failure(f.fetch("/big", None).await).code,
        ErrorCode::UnsupportedInput
    );
}

#[tokio::test]
async fn refuses_non_http_links() {
    let f = install();
    let url = url::Url::parse("ftp://example.com/a").unwrap();
    assert!(f.host.candidates(&url).is_empty());
}
