//! 通用网页插件走一遍宿主：清单 → match → canonicalize → fetch → 校验过的输出目录。
//! 对着本机的 HTTP 服务跑，不碰真网络。
//!
//! 本机服务在回环地址上，插件默认不取。宿主还没把设置传给插件（`call_env`），
//! 所以要打开「允许取内网网页」的用例直接起插件进程，用 `USUYO_SETTINGS` 传设置。

use std::io::{BufRead, BufReader, Write};
use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::process::Command;

use tempfile::TempDir;
use usuyo_plugin::protocol::{
    CallFailure, Canonicalized, Completeness, ErrorCode, FetchOutput, Identity, NotFoundReason,
    OutputKind,
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

/// 每一种不放行的地址，各取一个例子。都在连接前被拒，测试不会真的去连。
const BLOCKED: [&str; 29] = [
    "http://127.0.0.1:9/",
    "http://127.8.9.10/",
    "http://localhost/",
    "http://[::1]:9/",
    "http://0.0.0.0/",
    "http://[::]/",
    "http://10.1.2.3/",
    "http://172.16.0.1/",
    "http://192.168.1.1/",
    "http://169.254.169.254/latest/meta-data/",
    "http://[fe80::1]/",
    "http://[fc00::1]/",
    "http://[fd12:3456::1]/",
    "http://100.64.0.1/",
    "http://224.0.0.1/",
    "http://[ff02::1]/",
    "http://255.255.255.255/",
    "http://240.0.0.1/",
    "http://192.0.2.1/",
    "http://198.51.100.1/",
    "http://203.0.113.1/",
    "http://198.18.0.1/",
    "http://[2001:db8::1]/",
    "http://[::ffff:127.0.0.1]/",
    "http://[::ffff:a9fe:a9fe]/",
    "http://[64:ff9b::7f00:1]/",
    "http://2130706433/",
    "http://0x7f.0.0.1/",
    "http://0177.0.0.1/",
];

/// 正文里的图片全是不放行的地址。
fn ranges_page() -> String {
    let images: String = BLOCKED
        .iter()
        .map(|u| format!("<p><img src=\"{u}img.png\" alt=\"图\"></p>\n"))
        .collect();
    ARTICLE.replace("<p><img src=\"/img/a.png\" alt=\"闷蒸\"></p>", &images)
}

/// 起一个只回固定内容的 HTTP 服务，返回它的地址。
fn serve() -> String {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    let addr = format!("http://127.0.0.1:{port}");
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
                let redirect = |to: String| {
                    (
                        "302 Found",
                        "text/plain",
                        Vec::new(),
                        format!("Location: {to}\r\n"),
                    )
                };
                let (status, ctype, body, extra): (&str, &str, Vec<u8>, String) = match path
                    .as_str()
                {
                    "/article?utm_source=feed" | "/article" => (
                        "200 OK",
                        "text/html; charset=utf-8",
                        ARTICLE.into(),
                        String::new(),
                    ),
                    "/ranges" => ("200 OK", "text/html", ranges_page().into(), String::new()),
                    "/redirect" => redirect("/article".into()),
                    "/redirect-localhost" => redirect(format!("http://localhost:{port}/article")),
                    "/redirect-hex" => redirect(format!("http://0x7f000001:{port}/article")),
                    p if p.starts_with("/to/") => {
                        let n: usize = p["/to/".len()..].parse().unwrap();
                        redirect(BLOCKED[n].into())
                    }
                    "/gbk" => {
                        let page = ARTICLE.replace("utf-8", "gbk");
                        let bytes = encoding_rs::GBK.encode(&page).0.into_owned();
                        ("200 OK", "text/html", bytes, String::new())
                    }
                    "/img/a.png" | "/img/lead.png" => {
                        ("200 OK", "image/png", PNG.to_vec(), String::new())
                    }
                    "/gone" => ("410 Gone", "text/html", b"gone".to_vec(), String::new()),
                    "/limited" => (
                        "429 Too Many Requests",
                        "text/html",
                        Vec::new(),
                        "Retry-After: 120\r\n".into(),
                    ),
                    "/pdf" => (
                        "200 OK",
                        "application/pdf",
                        b"%PDF-1.7".to_vec(),
                        String::new(),
                    ),
                    "/big" => ("200 OK", "text/html", vec![b'a'; 17 << 20], String::new()),
                    _ => (
                        "404 Not Found",
                        "text/html",
                        b"nope".to_vec(),
                        String::new(),
                    ),
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
    async fn fetch(&self, path: &str) -> Result<FetchedSnapshot, PluginError> {
        let url = url::Url::parse(&format!("{}{path}", self.base)).unwrap();
        let plugin = self.host.candidates(&url)[0];
        let request = FetchRequest {
            input: FetchInput::Url(url),
            want: None,
            input_file: None,
        };
        self.host
            .fetch(plugin, request, &CallOptions::default())
            .await
    }
}

/// 不经宿主，直接起一次 `fetch`。`settings` 是 `USUYO_SETTINGS` 文件的内容。
struct Run {
    out: TempDir,
    result: Result<FetchOutput, CallFailure>,
}

fn run(url: &str, want: Option<&str>, settings: Option<&str>) -> Run {
    let out = tempfile::tempdir().unwrap();
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_usuyo-plugin-web"));
    cmd.args(["fetch", url, "--out"])
        .arg(out.path())
        .env_remove("USUYO_SETTINGS");
    if let Some(want) = want {
        cmd.args(["--want", want]);
    }
    let settings_dir = tempfile::tempdir().unwrap();
    if let Some(settings) = settings {
        let file = settings_dir.path().join("settings.json");
        std::fs::write(&file, settings).unwrap();
        cmd.env("USUYO_SETTINGS", file);
    }
    let done = cmd.output().unwrap();
    let envelope: serde_json::Value = serde_json::from_slice(&done.stdout)
        .unwrap_or_else(|e| panic!("{e}: {}", String::from_utf8_lossy(&done.stderr)));
    let result = if envelope["ok"] == true {
        Ok(serde_json::from_value(envelope).unwrap())
    } else {
        Err(serde_json::from_value(envelope["error"].clone()).unwrap())
    };
    Run { out, result }
}

const ALLOW: Option<&str> = Some(r#"{"allow-private-network": true}"#);

fn ok(run: &Run) -> &FetchOutput {
    run.result.as_ref().unwrap()
}

fn err(run: Run) -> CallFailure {
    match run.result {
        Err(failure) => failure,
        Ok(out) => panic!("expected a failure, got {out:?}"),
    }
}

fn meta(dir: &Path) -> serde_json::Value {
    serde_json::from_slice(&std::fs::read(dir.join("meta.json")).unwrap()).unwrap()
}

fn files(dir: &Path) -> Vec<String> {
    let mut names: Vec<String> = std::fs::read_dir(dir)
        .unwrap()
        .map(|e| e.unwrap().file_name().into_string().unwrap())
        .collect();
    names.sort();
    names
}

#[tokio::test]
async fn host_finds_the_plugin_and_it_refuses_loopback_by_default() {
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

    // 宿主还不传设置，默认关着：本机地址不取，提示里说怎么打开
    match f.fetch("/article").await {
        Err(PluginError::Call { failure, .. }) => {
            assert_eq!(failure.code, ErrorCode::UnsupportedInput);
            assert!(failure.hint.unwrap().contains("允许取内网网页"));
        }
        other => panic!("expected a failure, got {other:?}"),
    }
    drop(f.tmp);
}

#[tokio::test]
async fn refuses_non_http_links() {
    let f = install();
    let url = url::Url::parse("ftp://example.com/a").unwrap();
    assert!(f.host.candidates(&url).is_empty());
}

#[test]
fn fetches_an_article_but_not_its_loopback_images() {
    let base = serve();
    let run = run(&format!("{base}/article?utm_source=feed"), None, ALLOW);
    let out = ok(&run);
    assert_eq!(
        out.identity,
        Some(Identity::Web {
            url: format!("{base}/article")
        })
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
    // 设置只放开页面本身：同一台机器上的图片照样不取
    assert!(out.images.is_empty());
    assert_eq!(out.completeness, Completeness::Partial);
    assert_eq!(out.missing, [OutputKind::Images]);

    let dir = run.out.path();
    assert_eq!(
        std::fs::read_to_string(dir.join("page.html")).unwrap(),
        ARTICLE
    );
    let meta = meta(dir);
    assert_eq!(meta["description"], "三段式注水的做法和要点");
    assert_eq!(meta["leadImage"], format!("{base}/img/lead.png"));
    assert_eq!(meta["canonicalUrl"], format!("{base}/posts/pour-over"));
    assert_eq!(
        meta["failedImages"],
        serde_json::json!([
            format!("{base}/img/a.png"),
            format!("{base}/img/missing.png")
        ])
    );
    assert_eq!(files(dir), ["meta.json", "page.html"]);
}

#[test]
fn want_limits_what_is_written() {
    let base = serve();
    let run = run(&format!("{base}/article"), Some("text"), ALLOW);
    let out = ok(&run);
    assert!(out.text.is_some());
    assert!(out.html.is_none());
    assert!(out.images.is_empty());
    assert!(files(run.out.path()).is_empty());
}

#[test]
fn decodes_gbk() {
    let base = serve();
    let run = run(&format!("{base}/gbk"), Some("text,meta"), ALLOW);
    assert!(ok(&run).text.as_deref().unwrap().contains("闷蒸"));
}

#[test]
fn maps_http_failures() {
    let base = serve();
    let code = |path: &str| err(run(&format!("{base}{path}"), None, ALLOW));
    let gone = code("/gone");
    assert_eq!(
        (gone.code, gone.reason),
        (ErrorCode::NotFound, Some(NotFoundReason::Deleted))
    );
    assert_eq!(code("/nothing").code, ErrorCode::NotFound);
    let limited = code("/limited");
    assert_eq!(
        (limited.code, limited.retry_after),
        (ErrorCode::RateLimited, Some(120))
    );
    assert_eq!(code("/pdf").code, ErrorCode::UnsupportedInput);
    assert_eq!(code("/big").code, ErrorCode::UnsupportedInput);
}

#[test]
fn bad_settings_are_invalid_args() {
    let base = serve();
    let failure = err(run(
        &format!("{base}/article"),
        None,
        Some(r#"{"allow-private-network": "yes"}"#),
    ));
    assert_eq!(failure.code, ErrorCode::InvalidArgs);
}

#[test]
fn refuses_every_private_range_as_the_page() {
    for url in BLOCKED {
        let failure = err(run(url, None, None));
        assert_eq!(failure.code, ErrorCode::UnsupportedInput, "{url}");
        assert!(failure.hint.is_some(), "{url}");
    }
}

#[test]
fn refuses_credentials_even_when_private_is_allowed() {
    let base = serve().replace("http://", "http://user:pw@");
    let failure = err(run(&format!("{base}/article"), None, ALLOW));
    assert_eq!(failure.code, ErrorCode::UnsupportedInput);
    assert!(failure.hint.is_none());
}

#[test]
fn refuses_every_private_range_as_an_image() {
    let base = serve();
    let run = run(&format!("{base}/ranges"), None, ALLOW);
    let out = ok(&run);
    assert!(out.images.is_empty());
    assert!(out.text.is_some(), "a refused image never fails the fetch");
    let failed = meta(run.out.path())["failedImages"].clone();
    for url in BLOCKED {
        let src = url::Url::parse(&format!("{url}img.png")).unwrap();
        assert!(
            failed.as_array().unwrap().iter().any(|f| f == src.as_str()),
            "{src} not in {failed}"
        );
    }
}

#[test]
fn refuses_every_private_range_as_a_redirect_target() {
    let base = serve();
    for (n, url) in BLOCKED.iter().enumerate() {
        // 第一跳是放开的本机地址，跳转目标不放开
        let failure = err(run(&format!("{base}/to/{n}"), None, ALLOW));
        assert_eq!(failure.code, ErrorCode::UnsupportedInput, "{url}");
        assert!(
            failure.message.contains("redirect refused"),
            "{url}: {}",
            failure.message
        );
        assert!(failure.hint.is_none(), "{url}");
    }
}

#[test]
fn the_setting_does_not_reach_redirect_targets() {
    let base = serve();
    for path in ["/redirect", "/redirect-localhost", "/redirect-hex"] {
        let failure = err(run(&format!("{base}{path}"), None, ALLOW));
        assert_eq!(failure.code, ErrorCode::UnsupportedInput, "{path}");
        assert!(failure.message.contains("redirect refused"), "{path}");
    }
}
