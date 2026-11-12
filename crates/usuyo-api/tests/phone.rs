//! 「手机」那一页的路由，对着假的一切：本机起的假 release 服务器、假的「mirroir serve」（MCP over HTTP）、
//! 当 `sw_vers`、`pgrep`、`open`、剪贴板计数和 `adb` 用的 sh 脚本。不下真的驱动、不起真的驱动、
//! 不打开系统设置、不调真的 adb。
#![cfg(unix)]

use std::collections::HashMap;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use axum::Router;
use axum::body::Body;
use axum::http::{Request, StatusCode, header};
use http_body_util::BodyExt;
use serde_json::{Value, json};
use tempfile::TempDir;
use tower::ServiceExt;
use usuyo_api::{ApiConfig, default_origins, router};
use usuyo_core::view::Event;
use usuyo_core::{Asset, Downloads, Engine, EngineConfig, PhoneConfig, Release};

const PORT: u16 = 47615;
const TOKEN: &str = "test-token";
const SERVE_TOKEN: &str = "serve-token-0123456789-0123456789-abcdef";

// —— 假的 release 服务器 ——

#[derive(Clone)]
enum Reply {
    Bytes(Vec<u8>),
    Stall(Vec<u8>, usize),
    Status(u16),
}

fn release_server(routes: Vec<(&str, Reply)>) -> String {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let base = format!("http://{}", listener.local_addr().unwrap());
    let routes: HashMap<String, Reply> =
        routes.into_iter().map(|(p, r)| (p.to_owned(), r)).collect();
    std::thread::spawn(move || {
        for stream in listener.incoming() {
            let Ok(mut stream) = stream else { return };
            let routes = routes.clone();
            std::thread::spawn(move || {
                let (path, _, _) = read_request(&mut stream);
                let head = |status: &str, len: usize| {
                    format!(
                        "HTTP/1.1 {status}\r\nContent-Length: {len}\r\nConnection: close\r\n\r\n"
                    )
                };
                match routes.get(&path) {
                    Some(Reply::Bytes(b)) => {
                        let _ = stream.write_all(head("200 OK", b.len()).as_bytes());
                        let _ = stream.write_all(b);
                    }
                    Some(Reply::Stall(b, n)) => {
                        let _ = stream.write_all(head("200 OK", b.len()).as_bytes());
                        let _ = stream.write_all(&b[..*n]);
                        let _ = stream.flush();
                        std::thread::sleep(Duration::from_secs(3));
                    }
                    Some(Reply::Status(code)) => {
                        let _ = stream.write_all(head(&format!("{code} X"), 0).as_bytes());
                    }
                    None => {
                        let _ = stream.write_all(head("404 Not Found", 0).as_bytes());
                    }
                }
            });
        }
    });
    base
}

/// 请求行的路径、头（小写名字）、请求体。
fn read_request(stream: &mut TcpStream) -> (String, HashMap<String, String>, Vec<u8>) {
    let mut reader = BufReader::new(stream.try_clone().unwrap());
    let mut line = String::new();
    reader.read_line(&mut line).unwrap();
    let path = line.split(' ').nth(1).unwrap_or_default().to_owned();
    let mut headers = HashMap::new();
    loop {
        let mut h = String::new();
        reader.read_line(&mut h).unwrap();
        if h.trim().is_empty() {
            break;
        }
        if let Some((k, v)) = h.split_once(':') {
            headers.insert(k.trim().to_lowercase(), v.trim().to_owned());
        }
    }
    let len = headers
        .get("content-length")
        .and_then(|l| l.parse::<usize>().ok())
        .unwrap_or(0);
    let mut body = vec![0u8; len];
    reader.read_exact(&mut body).unwrap();
    (path, headers, body)
}

fn tar_gz(entries: &[(&str, &[u8])]) -> Vec<u8> {
    let mut b = tar::Builder::new(flate2::write::GzEncoder::new(
        Vec::new(),
        flate2::Compression::fast(),
    ));
    for (path, data) in entries {
        let mut h = tar::Header::new_gnu();
        h.set_size(data.len() as u64);
        h.set_mode(0o644);
        h.set_entry_type(tar::EntryType::Regular);
        h.set_path(path).unwrap();
        h.set_cksum();
        b.append(&h, *data).unwrap();
    }
    b.into_inner().unwrap().finish().unwrap()
}

fn sha256(bytes: &[u8]) -> String {
    use sha2::Digest;
    sha2::Sha256::digest(bytes)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

// —— 假的「mirroir serve」 ——

/// 它现在怎么答：服务版本、`status` 的话、`check_health` 的话、`describe_screen` 的话（`None` 是 `isError`）。
#[derive(Clone)]
struct Mirror {
    version: String,
    status: String,
    health: String,
    screen: Option<String>,
    /// 收到的工具调用，按顺序。
    calls: Vec<String>,
}

type Shared = Arc<Mutex<Mirror>>;

fn mirror_server(state: Shared) -> String {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let url = format!("http://{}/mcp", listener.local_addr().unwrap());
    std::thread::spawn(move || {
        for stream in listener.incoming() {
            let Ok(mut stream) = stream else { return };
            let state = Arc::clone(&state);
            std::thread::spawn(move || {
                let (path, headers, body) = read_request(&mut stream);
                let reply = |status: &str, body: &str| {
                    format!(
                        "HTTP/1.1 {status}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                        body.len()
                    )
                };
                if path != "/mcp"
                    || headers.get("authorization").map(String::as_str)
                        != Some(&format!("Bearer {SERVE_TOKEN}"))
                {
                    let _ =
                        stream.write_all(reply("401 Unauthorized", "Send the header").as_bytes());
                    return;
                }
                let req: Value = serde_json::from_slice(&body).unwrap();
                let mut s = state.lock().unwrap();
                let result = match req["method"].as_str() {
                    Some("initialize") => json!({
                        "protocolVersion": "2025-11-25",
                        "serverInfo": {"name": "mirroir-mcp", "version": s.version},
                        "capabilities": {"tools": {}},
                    }),
                    Some("tools/call") => {
                        let name = req["params"]["name"].as_str().unwrap().to_owned();
                        s.calls.push(name.clone());
                        let text = match name.as_str() {
                            "status" => Some(s.status.clone()),
                            "check_health" => Some(s.health.clone()),
                            "describe_screen" => {
                                assert_eq!(
                                    req["params"]["arguments"],
                                    json!({"text_only": true, "omit_screenshot": true})
                                );
                                s.screen.clone()
                            }
                            other => panic!("unexpected tool {other}"),
                        };
                        match text {
                            Some(t) => {
                                json!({"content": [{"type": "text", "text": t}], "isError": false})
                            }
                            None => {
                                json!({"content": [{"type": "text", "text": "Target 'iPhone Mirroring' is not running"}], "isError": true})
                            }
                        }
                    }
                    other => panic!("unexpected method {other:?}"),
                };
                let body = json!({"jsonrpc": "2.0", "id": req["id"], "result": result}).to_string();
                let _ = stream.write_all(reply("200 OK", &body).as_bytes());
            });
        }
    });
    url
}

// —— 假程序 ——

fn script(dir: &Path, name: &str, body: &str) -> PathBuf {
    let path = dir.join(name);
    std::fs::write(&path, format!("#!/bin/sh\n{body}\n")).unwrap();
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).unwrap();
    path
}

const IPHONE_PLUGIN: &str = r#"protocol: 1
id: fake/phone
kind: fetch
name: 手机的
version: 1.0.0
license: MIT
maintainers: [someone]
platforms: [example]
match: ["example.test/*"]
methods:
  - id: page
    name: 公开页
    access: public-web
    login: none
    account: none
    capabilities:
      fetch: {outputs: [text]}
  - id: iphone
    name: iPhone 镜像
    built: false
    access: vision-ui
    login: required
    account: own-phone
    drives: iphone
permissions:
  fs: {write: ["${OUT}"]}
run:
  command: ["${PLUGIN_DIR}/plugin.sh"]
"#;

const FLOW: &str = r#"{"name": "例子 · 喜欢", "steps": [
  {"expect": ["首页", "我"], "tap": ["我"]},
  {"expect": ["喜欢", "作品"], "tap": ["喜欢"], "allow": ["喜欢"]},
  {"expect": ["分享", "复制链接"], "tap": ["复制链接"]}
]}"#;

struct Fixture {
    tmp: TempDir,
    engine: Engine,
    app: Router,
    mirror: Shared,
    bin: PathBuf,
    driver_dir: PathBuf,
}

struct Options {
    /// 假 release 服务器上包的路径和包本身；sha256 照 `sha` 钉（`None` 是包本身的）。
    asset: Option<(String, Vec<u8>, Option<String>)>,
    arch: &'static str,
    /// 「mirroir serve」在跑（令牌文件在）。
    serving: bool,
    adb: bool,
}

impl Default for Options {
    fn default() -> Self {
        Options {
            asset: None,
            arch: "aarch64",
            serving: false,
            adb: true,
        }
    }
}

fn fixture(o: Options) -> Fixture {
    let tmp = tempfile::tempdir().unwrap();
    let machine = tmp.path().join("machine");
    let bin = tmp.path().join("bin");
    std::fs::create_dir_all(&bin).unwrap();
    // 装一个驱动 iPhone 的插件，带一个流程
    let plugin = machine.join("plugins/fake/phone/1.0.0");
    std::fs::create_dir_all(plugin.join("flows")).unwrap();
    std::fs::write(plugin.join("usuyo-plugin.yaml"), IPHONE_PLUGIN).unwrap();
    script(
        &plugin,
        "plugin.sh",
        "echo '{\"ok\": true, \"ready\": true, \"protocol\": 1, \"version\": \"1.0.0\", \"checks\": []}'",
    );
    std::fs::write(plugin.join("flows/like.json"), FLOW).unwrap();

    let sw_vers = script(&bin, "sw_vers", "echo 15.6.1");
    // pgrep：`mirroring-running` 这个文件在就是在跑
    let pgrep = script(
        &bin,
        "pgrep",
        &format!("test -e '{}/mirroring-running'", bin.display()),
    );
    // open：记下参数；`open-fails` 在就失败
    let open = script(
        &bin,
        "open",
        &format!(
            "echo \"$1\" >> '{0}/opened'; test ! -e '{0}/open-fails'",
            bin.display()
        ),
    );
    // 剪贴板计数：读文件；`count` 不在就失败
    std::fs::write(bin.join("count"), "41").unwrap();
    let pasteboard = script(
        &bin,
        "pasteboard",
        &format!("cat '{}/count'", bin.display()),
    );
    let adb_dir = tmp.path().join("adb");
    std::fs::create_dir_all(&adb_dir).unwrap();
    if o.adb {
        script(
            &adb_dir,
            "adb",
            r#"case "$1" in
version) printf 'Android Debug Bridge version 1.0.41\nVersion 36.0.0-13206524\n' ;;
devices) printf 'List of devices attached\nR58N70ABC3K2A          unauthorized usb:1-1 transport_id:1\n2B1F0C7E9A4D           device usb:1-2 product:husky model:Pixel_8 device:husky transport_id:2\n\n' ;;
*) exit 2 ;;
esac"#,
        );
    }
    let mirroring_app = tmp.path().join("iPhone Mirroring.app");
    std::fs::create_dir_all(&mirroring_app).unwrap();

    let mirror = Arc::new(Mutex::new(Mirror {
        version: "0.47.0".into(),
        status: "Connected — mirroring active (window: 326x720 pt, pos=(10,20), portrait)".into(),
        health: "All checks passed\n\n[ok] Screen capture working (40 ms)".into(),
        screen: Some(
            "Screen elements (tap coordinates in points):\n- \"喜欢\"\n- \"作品\"\n- \"某人\"\n\n_meta: element_count=3".into(),
        ),
        calls: Vec::new(),
    }));
    let serve_url = mirror_server(Arc::clone(&mirror));
    let serve_token = tmp.path().join("serve-token");
    if o.serving {
        std::fs::write(&serve_token, SERVE_TOKEN).unwrap();
    }

    let (path, pack, sha) = o
        .asset
        .unwrap_or_else(|| ("/none".into(), b"x".to_vec(), None));
    // 后面的同名路径盖过前面的：404 和断流的那两条总是照它们自己的样子答
    let base = release_server(vec![
        (path.as_str(), Reply::Bytes(pack.clone())),
        ("/gone.tar.gz", Reply::Status(404)),
        ("/slow.tar.gz", Reply::Stall(pack.clone(), 4)),
    ]);
    let asset = Asset {
        arch: "aarch64".into(),
        name: "mirroir-mcp-darwin-arm64.tar.gz".into(),
        url: format!("{base}{path}"),
        size: pack.len() as u64,
        sha256: sha.unwrap_or_else(|| sha256(&pack)),
    };
    let release = Release {
        name: "mirroir-mcp".into(),
        version: "0.47.0".into(),
        license: "Apache-2.0".into(),
        page: "https://example.test/releases/tag/v0.47.0".into(),
        binary: "mirroir-mcp".into(),
        assets: vec![asset],
    };
    let driver_dir = machine.join("drivers/mirroir-mcp");
    let phone = PhoneConfig {
        driver_dir: driver_dir.clone(),
        release,
        arch: o.arch.into(),
        downloads: Downloads::Loopback {
            stall: Duration::from_millis(400),
        },
        serve_url,
        serve_token,
        sw_vers,
        mirroring_app,
        pgrep,
        open,
        pasteboard: vec![pasteboard.display().to_string()],
        adb_path: Some(adb_dir.clone().into_os_string()),
        adb_dirs: Vec::new(),
        clipboard_wait: Duration::from_secs(1),
        clipboard_every: Duration::from_millis(50),
    };
    let lib = Engine::init_library(&tmp.path().join("lib"), &machine).unwrap();
    let mut config = EngineConfig::new(lib, &machine);
    config.phone = Some(phone);
    let engine = Engine::open(config).unwrap();
    let app = api(&engine);
    Fixture {
        tmp,
        engine,
        app,
        mirror,
        bin,
        driver_dir,
    }
}

fn api(engine: &Engine) -> Router {
    router(
        engine.clone(),
        ApiConfig {
            port: PORT,
            token: TOKEN.into(),
            origins: default_origins(),
            clients: None,
        },
    )
    .unwrap()
}

fn req(method: &str, path: &str, body: Option<Value>) -> Request<Body> {
    let b = Request::builder()
        .method(method)
        .uri(path)
        .header(header::HOST, format!("127.0.0.1:{PORT}"))
        .header(header::AUTHORIZATION, format!("Bearer {TOKEN}"));
    match body {
        Some(v) => b
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(v.to_string()))
            .unwrap(),
        None => b.body(Body::empty()).unwrap(),
    }
}

async fn call(app: &Router, method: &str, path: &str, body: Option<Value>) -> (StatusCode, Value) {
    let res = app.clone().oneshot(req(method, path, body)).await.unwrap();
    let status = res.status();
    let bytes = res.into_body().collect().await.unwrap().to_bytes();
    let v = if bytes.is_empty() {
        Value::Null
    } else {
        serde_json::from_slice(&bytes).unwrap()
    };
    (status, v)
}

async fn view(app: &Router) -> Value {
    let (s, v) = call(app, "GET", "/phone", None).await;
    assert_eq!(s, StatusCode::OK, "{v}");
    v
}

/// 等 `phone-updated`，直到 `done` 对这一页成立。
async fn until(f: &Fixture, done: impl Fn(&Value) -> bool) -> Value {
    let mut events = f.engine.subscribe();
    let deadline = tokio::time::Instant::now() + Duration::from_secs(10);
    loop {
        let v = view(&f.app).await;
        if done(&v) {
            return v;
        }
        let left = deadline.saturating_duration_since(tokio::time::Instant::now());
        assert!(!left.is_zero(), "timed out; last view {v}");
        match tokio::time::timeout(left, events.recv()).await {
            Ok(Ok(Event::PhoneUpdated)) | Err(_) => {}
            Ok(_) => {}
        }
    }
}

fn nothing_left(f: &Fixture) {
    let parent = f.driver_dir.parent().unwrap();
    let left: Vec<_> = match std::fs::read_dir(parent) {
        Ok(rd) => rd.map(|e| e.unwrap().path()).collect(),
        Err(_) => Vec::new(),
    };
    assert!(left.is_empty(), "{left:?}");
}

#[tokio::test]
async fn without_a_phone_config_every_route_is_not_found() {
    let tmp = tempfile::tempdir().unwrap();
    let machine = tmp.path().join("machine");
    let lib = Engine::init_library(&tmp.path().join("lib"), &machine).unwrap();
    let engine = Engine::open(EngineConfig::new(lib, &machine)).unwrap();
    let app = api(&engine);
    for (m, p, b) in [
        ("GET", "/phone", None),
        ("POST", "/phone/ios/driver/install", None),
        ("POST", "/phone/ios/driver/update", None),
        ("POST", "/phone/ios/driver/remove", None),
        (
            "POST",
            "/phone/open-settings",
            Some(json!({"pane": "accessibility"})),
        ),
        ("POST", "/phone/ios/clipboard-test", None),
        ("POST", "/phone/ios/dry-run", None),
    ] {
        let (s, v) = call(&app, m, p, b).await;
        assert_eq!(s, StatusCode::NOT_FOUND, "{p}");
        assert_eq!(v["code"], "not-found");
        assert_eq!(v["hint"], "这台电脑上的 usuyo 内核不管手机。");
    }
}

#[tokio::test]
async fn the_page_reads_this_machine_with_nothing_installed() {
    let f = fixture(Options::default());
    let v = view(&f.app).await;
    assert_eq!(
        v,
        json!({
            "ios": {
                "system": {"macos": "15.6.1", "mirroring": true},
                "driver": {
                    "state": "absent",
                    "available": {
                        "version": "0.47.0",
                        "size": 1,
                        "source": "https://example.test/releases/tag/v0.47.0",
                    },
                },
                "grants": {"to": "mirroir serve", "screen": "unknown", "accessibility": "unknown"},
                "mirroring": "not-set-up",
            },
            "android": {
                "adb": {"path": f.tmp.path().join("adb/adb").display().to_string(), "version": "36.0.0"},
                "devices": [
                    {"serial": "R58N…3K2A", "state": "unauthorized"},
                    {"serial": "2B1F…9A4D", "model": "Pixel 8", "state": "device"},
                ],
            },
        })
    );
    // iPhone 镜像开着、驱动不在：离开着
    std::fs::write(f.bin.join("mirroring-running"), "").unwrap();
    assert_eq!(view(&f.app).await["ios"]["mirroring"], "away");
    assert!(f.mirror.lock().unwrap().calls.is_empty());
}

#[tokio::test]
async fn no_adb_and_no_mirroring_app() {
    let f = fixture(Options {
        adb: false,
        ..Options::default()
    });
    std::fs::remove_dir(f.tmp.path().join("iPhone Mirroring.app")).unwrap();
    let v = view(&f.app).await;
    assert_eq!(v["android"], json!({"adb": null, "devices": []}));
    assert_eq!(v["ios"]["system"]["mirroring"], false);
    assert_eq!(v["ios"]["mirroring"], "not-set-up");
}

#[tokio::test]
async fn install_then_register_then_remove() {
    let pack = tar_gz(&[("mirroir-mcp", b"#!/bin/sh\necho fake driver\n")]);
    let f = fixture(Options {
        asset: Some(("/pack.tar.gz".into(), pack, None)),
        ..Options::default()
    });
    let (s, v) = call(&f.app, "POST", "/phone/ios/driver/install", None).await;
    assert_eq!(s, StatusCode::OK, "{v}");
    assert!(
        ["installing", "present"].contains(&v["ios"]["driver"]["state"].as_str().unwrap()),
        "{v}"
    );
    let v = until(&f, |v| v["ios"]["driver"]["state"] == "present").await;
    let driver = &v["ios"]["driver"];
    assert_eq!(driver["version"], "0.47.0");
    assert!(driver.get("available").is_none(), "{driver}");
    // 「mirroir serve」没在跑：要你登记它
    let binary = f.driver_dir.join("mirroir-mcp");
    assert_eq!(
        driver["step"]["commands"],
        json!([
            format!("'{}' install-agent", binary.display()),
            "launchctl bootstrap gui/$(id -u) ~/Library/LaunchAgents/dev.mirroir.serve.plist",
        ])
    );
    assert_eq!(
        std::fs::metadata(&binary).unwrap().permissions().mode() & 0o777,
        0o755
    );
    // 再点一次装：已经是这一版，不再下
    let (_, v) = call(&f.app, "POST", "/phone/ios/driver/update", None).await;
    assert_eq!(v["ios"]["driver"]["state"], "present");

    // 登记好了、连着：授权都给了，不用再做什么
    std::fs::write(f.tmp.path().join("serve-token"), format!("{SERVE_TOKEN}\n")).unwrap();
    let v = view(&f.app).await;
    assert_eq!(v["ios"]["mirroring"], "connected");
    assert_eq!(
        v["ios"]["grants"],
        json!({"to": "mirroir serve", "screen": "given", "accessibility": "given"})
    );
    assert!(v["ios"]["driver"].get("step").is_none());
    assert_eq!(f.mirror.lock().unwrap().calls, ["status"]);

    // 在跑的还是旧的那一版：要你重新登记
    f.mirror.lock().unwrap().version = "0.46.0".into();
    let v = view(&f.app).await;
    assert_eq!(
        v["ios"]["driver"]["step"]["commands"][1],
        "launchctl kickstart -k gui/$(id -u)/dev.mirroir.serve"
    );

    // 卸掉：目录没了，「mirroir serve」还在跑，要你停掉它
    let (s, v) = call(&f.app, "POST", "/phone/ios/driver/remove", None).await;
    assert_eq!(s, StatusCode::OK);
    assert_eq!(v["ios"]["driver"]["state"], "absent");
    assert_eq!(
        v["ios"]["driver"]["step"]["commands"],
        json!(["launchctl bootout gui/$(id -u)/dev.mirroir.serve"])
    );
    nothing_left(&f);
    let (s, _) = call(&f.app, "POST", "/phone/ios/driver/remove", None).await;
    assert_eq!(s, StatusCode::OK);
}

/// 装不上的几种：页面上「没装上：」后面那一句，什么都没留下。
#[tokio::test]
async fn failed_installs_say_why_and_leave_nothing() {
    let pack = tar_gz(&[("mirroir-mcp", b"program")]);
    for (path, sha, sentence) in [
        (
            "/pack.tar.gz",
            Some("0".repeat(64)),
            "下载下来的包和钉住的 SHA256 对不上，已经丢掉了，没有放进这台电脑。",
        ),
        (
            "/gone.tar.gz",
            None,
            "官方 release 上没有这一版的包，什么都没放进这台电脑。",
        ),
        (
            "/slow.tar.gz",
            None,
            "网络不通，或者下载断了，什么都没放进这台电脑。",
        ),
    ] {
        let mut options = Options {
            asset: Some(("/pack.tar.gz".into(), pack.clone(), sha.clone())),
            ..Options::default()
        };
        if path != "/pack.tar.gz" {
            // 包的地址换成 404 或断流的那一个，大小和 sha256 照好的包
            options.asset = Some((path.into(), pack.clone(), None));
        }
        let f = fixture(options);
        call(&f.app, "POST", "/phone/ios/driver/install", None).await;
        let v = until(&f, |v| v["ios"]["driver"]["state"] == "failed").await;
        assert_eq!(v["ios"]["driver"]["hint"], sentence, "{path}");
        assert_eq!(v["ios"]["driver"]["available"]["version"], "0.47.0");
        nothing_left(&f);
    }
}

#[tokio::test]
async fn an_unsupported_machine_is_refused_with_one_sentence() {
    let f = fixture(Options {
        arch: "x86_64",
        ..Options::default()
    });
    let (s, v) = call(&f.app, "POST", "/phone/ios/driver/install", None).await;
    assert_eq!(s, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(v["code"], "unsupported-input");
    assert_eq!(
        v["hint"],
        "mirroir 只给 Apple 芯片的 Mac 出了包，这台电脑装不了。"
    );
    assert!(
        view(&f.app).await["ios"]["driver"]
            .get("available")
            .is_none()
    );
    nothing_left(&f);
}

#[tokio::test]
async fn grants_come_from_mirroir_serve_itself() {
    let f = fixture(Options {
        serving: true,
        ..Options::default()
    });
    let set = |status: &str, health: &str| {
        let mut m = f.mirror.lock().unwrap();
        m.status = status.into();
        m.health = health.into();
    };
    set(
        "Paused — the window says iPhone in Use.",
        "Issues detected\n\n[FAIL] Screen capture failed — turn on mirroir serve",
    );
    let v = view(&f.app).await;
    assert_eq!(v["ios"]["mirroring"], "away");
    assert_eq!(v["ios"]["grants"]["screen"], "missing");
    assert_eq!(v["ios"]["grants"]["accessibility"], "given");
    // 十秒里不再截第二张
    view(&f.app).await;
    assert_eq!(
        f.mirror.lock().unwrap().calls,
        ["status", "check_health", "status"]
    );

    let f = fixture(Options {
        serving: true,
        ..Options::default()
    });
    {
        let mut m = f.mirror.lock().unwrap();
        m.status = "No window — app is running but no mirroring window found".into();
        m.health = "Issues detected\n\n[ok] Screen capture working (40 ms)".into();
    }
    let v = view(&f.app).await;
    assert_eq!(v["ios"]["grants"]["screen"], "given");
    assert_eq!(v["ios"]["grants"]["accessibility"], "missing");

    {
        let mut m = f.mirror.lock().unwrap();
        m.status = "Not running — 'iPhone Mirroring' is not open.".into();
    }
    let v = view(&f.app).await;
    assert_eq!(v["ios"]["mirroring"], "not-set-up");
    assert_eq!(v["ios"]["grants"]["screen"], "unknown");
}

#[tokio::test]
async fn a_wrong_token_reads_as_not_answering() {
    let f = fixture(Options::default());
    std::fs::write(f.tmp.path().join("serve-token"), "someone-elses-token").unwrap();
    std::fs::write(f.bin.join("mirroring-running"), "").unwrap();
    let v = view(&f.app).await;
    assert_eq!(v["ios"]["mirroring"], "away");
    assert_eq!(v["ios"]["grants"]["screen"], "unknown");
    let (s, v) = call(&f.app, "POST", "/phone/ios/dry-run", None).await;
    assert_eq!(s, StatusCode::SERVICE_UNAVAILABLE);
    assert_eq!(v["code"], "unavailable");
    assert_eq!(v["hint"], "「mirroir serve」回的话看不懂。");
}

#[tokio::test]
async fn open_settings_opens_only_the_three_panes() {
    let f = fixture(Options::default());
    for pane in ["screen-recording", "accessibility", "iphone-mirroring"] {
        let (s, v) = call(
            &f.app,
            "POST",
            "/phone/open-settings",
            Some(json!({"pane": pane})),
        )
        .await;
        assert_eq!(s, StatusCode::NO_CONTENT, "{v}");
    }
    let opened = std::fs::read_to_string(f.bin.join("opened")).unwrap();
    assert_eq!(
        opened.lines().collect::<Vec<_>>(),
        [
            "x-apple.systempreferences:com.apple.preference.security?Privacy_ScreenCapture",
            "x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility",
            &f.tmp
                .path()
                .join("iPhone Mirroring.app")
                .display()
                .to_string(),
        ]
    );
    for bad in [
        json!({"pane": "general"}),
        json!({}),
        json!({"pane": "accessibility", "x": 1}),
    ] {
        let (s, v) = call(&f.app, "POST", "/phone/open-settings", Some(bad)).await;
        assert_eq!(s, StatusCode::BAD_REQUEST);
        assert_eq!(v["code"], "invalid-args");
    }
    std::fs::write(f.bin.join("open-fails"), "").unwrap();
    let (s, v) = call(
        &f.app,
        "POST",
        "/phone/open-settings",
        Some(json!({"pane": "accessibility"})),
    )
    .await;
    assert_eq!(s, StatusCode::INTERNAL_SERVER_ERROR);
    assert_eq!(v["hint"], "没能打开系统设置。");
}

#[tokio::test]
async fn the_clipboard_test_watches_only_the_change_count() {
    let f = fixture(Options::default());
    let (s, v) = call(&f.app, "POST", "/phone/ios/clipboard-test", None).await;
    assert_eq!(s, StatusCode::OK);
    assert_eq!(v["ios"]["clipboard"]["state"], "waiting");
    assert_eq!(v["ios"]["clipboard"]["waitSec"], 1);
    assert!(v["ios"]["clipboard"]["at"].as_str().unwrap().ends_with('Z'));
    // 点第二次：还在等，不另起一次
    let (_, v2) = call(&f.app, "POST", "/phone/ios/clipboard-test", None).await;
    assert_eq!(v2["ios"]["clipboard"], v["ios"]["clipboard"]);
    std::fs::write(f.bin.join("count"), "42").unwrap();
    let v = until(&f, |v| v["ios"]["clipboard"]["state"] != "waiting").await;
    assert_eq!(v["ios"]["clipboard"]["state"], "arrived");

    // 等满了没变
    let (_, _) = call(&f.app, "POST", "/phone/ios/clipboard-test", None).await;
    let v = until(&f, |v| v["ios"]["clipboard"]["state"] != "waiting").await;
    assert_eq!(v["ios"]["clipboard"]["state"], "unchanged");

    // 读不出这个数
    std::fs::remove_file(f.bin.join("count")).unwrap();
    let (s, v) = call(&f.app, "POST", "/phone/ios/clipboard-test", None).await;
    assert_eq!(s, StatusCode::INTERNAL_SERVER_ERROR);
    assert_eq!(v["hint"], "看不了这台 Mac 的剪贴板变没变。");
}

#[tokio::test]
async fn the_dry_run_reads_once_and_matches_the_plugins_flows() {
    let f = fixture(Options {
        serving: true,
        ..Options::default()
    });
    let (s, v) = call(&f.app, "POST", "/phone/ios/dry-run", None).await;
    assert_eq!(s, StatusCode::OK, "{v}");
    assert_eq!(
        v,
        json!({
            "read": 3,
            "match": {
                "flow": "例子 · 喜欢",
                "step": 2,
                "expect": {"matched": 2, "total": 2},
                "targets": [{"label": "喜欢", "found": true}],
            },
        })
    );
    assert_eq!(f.mirror.lock().unwrap().calls, ["describe_screen"]);

    f.mirror.lock().unwrap().screen =
        Some("Screen elements (tap coordinates in points):\n(no text detected)".into());
    let (_, v) = call(&f.app, "POST", "/phone/ios/dry-run", None).await;
    assert_eq!(v, json!({"read": 0}));

    // 镜像没连上：工具报错
    f.mirror.lock().unwrap().screen = None;
    let (s, v) = call(&f.app, "POST", "/phone/ios/dry-run", None).await;
    assert_eq!(s, StatusCode::FAILED_DEPENDENCY);
    assert_eq!(v["code"], "dependency-missing");
    assert_eq!(
        v["hint"],
        "读不了这一屏：iPhone 镜像没连上，或者授权还没给。"
    );
    // 回的话里不带它说的字
    assert!(!v.to_string().contains("not running"));

    // 「mirroir serve」不在
    std::fs::remove_file(f.tmp.path().join("serve-token")).unwrap();
    let (s, v) = call(&f.app, "POST", "/phone/ios/dry-run", None).await;
    assert_eq!(s, StatusCode::FAILED_DEPENDENCY);
    assert_eq!(
        v["hint"],
        "「mirroir serve」没在这台 Mac 上跑：先照「驱动」那一行把它登记好。"
    );
}

/// 装着旧的一版时给「更新」；换新成了是新的那版，没成时旧的那份还在、原因照样给。
#[tokio::test]
async fn update_replaces_an_older_driver_or_keeps_it_with_the_reason() {
    let pack = tar_gz(&[("mirroir-mcp", b"#!/bin/sh\necho new\n")]);
    for (sha, after, hint) in [
        (None, "0.47.0", None),
        (
            Some("0".repeat(64)),
            "0.46.0",
            Some("下载下来的包和钉住的 SHA256 对不上，已经丢掉了，没有放进这台电脑。"),
        ),
    ] {
        let f = fixture(Options {
            asset: Some(("/pack.tar.gz".into(), pack.clone(), sha)),
            ..Options::default()
        });
        std::fs::create_dir_all(&f.driver_dir).unwrap();
        std::fs::write(f.driver_dir.join("mirroir-mcp"), b"old").unwrap();
        std::fs::write(
            f.driver_dir.join("driver.json"),
            br#"{"version": "0.46.0", "asset": "a", "sha256": "b"}"#,
        )
        .unwrap();
        let v = view(&f.app).await;
        assert_eq!(v["ios"]["driver"]["version"], "0.46.0");
        assert_eq!(v["ios"]["driver"]["available"]["version"], "0.47.0");
        let (s, v) = call(&f.app, "POST", "/phone/ios/driver/update", None).await;
        assert_eq!(s, StatusCode::OK, "{v}");
        until(&f, |v| v["ios"]["driver"]["state"] == "present").await;
        let v = if hint.is_some() {
            until(&f, |v| v["ios"]["driver"].get("hint").is_some()).await
        } else {
            until(&f, |v| v["ios"]["driver"]["version"] == after).await
        };
        assert_eq!(v["ios"]["driver"]["version"], after, "{v}");
        assert_eq!(v["ios"]["driver"].get("hint").and_then(Value::as_str), hint);
        // 驱动目录以外什么都没留下
        let parent = f.driver_dir.parent().unwrap();
        assert_eq!(std::fs::read_dir(parent).unwrap().count(), 1);
    }
}
