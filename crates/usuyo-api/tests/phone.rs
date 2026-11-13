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
                let mut reader = BufReader::new(stream.try_clone().unwrap());
                let Some((path, _, _)) = read_request(&mut reader) else {
                    return;
                };
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

/// 一个请求的路径、头（小写名字）、请求体；对面没发请求就关了（或连接断了）是 `None`。
fn read_request(
    reader: &mut BufReader<TcpStream>,
) -> Option<(String, HashMap<String, String>, Vec<u8>)> {
    let mut line = String::new();
    if reader.read_line(&mut line).unwrap_or(0) == 0 {
        return None;
    }
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
    Some((path, headers, body))
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
    /// 收到的每一个请求（`initialize`、令牌不对被拒的都算）。
    requests: usize,
    /// 接下的连接。
    connections: usize,
    /// 每答完一个请求就关掉连接（`Connection: close`）。
    close: bool,
    /// 一接下连接就关掉，什么都不读。
    hang_up: bool,
}

type Shared = Arc<Mutex<Mirror>>;

/// 每接下一条连接，把对面（守护进程那一头）的端口写进 `accepted`：假的 `lsof` 照它答这条连接的另一头。
fn mirror_server(state: Shared, accepted: PathBuf) -> String {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let url = format!("http://{}/mcp", listener.local_addr().unwrap());
    std::thread::spawn(move || {
        for stream in listener.incoming() {
            let Ok(stream) = stream else { return };
            let hang_up = {
                let mut s = state.lock().unwrap();
                s.connections += 1;
                s.hang_up
            };
            if hang_up {
                drop(stream);
                continue;
            }
            let ours = stream.peer_addr().unwrap().port();
            let fresh = accepted.with_extension("new");
            std::fs::write(&fresh, ours.to_string()).unwrap();
            std::fs::rename(&fresh, &accepted).unwrap();
            let state = Arc::clone(&state);
            std::thread::spawn(move || serve_mirror(&state, stream));
        }
    });
    url
}

/// 一条连接上的请求，一个一个答，直到对面关掉（或 `close` 时答完一个）。
fn serve_mirror(state: &Shared, mut stream: TcpStream) {
    let mut reader = BufReader::new(stream.try_clone().unwrap());
    while let Some((path, headers, body)) = read_request(&mut reader) {
        let close = {
            let mut s = state.lock().unwrap();
            s.requests += 1;
            s.close
        };
        let reply = |status: &str, body: &str| {
            format!(
                "HTTP/1.1 {status}\r\nContent-Type: application/json\r\nContent-Length: {}\r\n{}\r\n{body}",
                body.len(),
                if close { "Connection: close\r\n" } else { "" },
            )
        };
        if path != "/mcp"
            || headers.get("authorization").map(String::as_str)
                != Some(&format!("Bearer {SERVE_TOKEN}"))
        {
            let _ = stream.write_all(reply("401 Unauthorized", "Send the header").as_bytes());
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
        drop(s);
        let body = json!({"jsonrpc": "2.0", "id": req["id"], "result": result}).to_string();
        if stream.write_all(reply("200 OK", &body).as_bytes()).is_err() || close {
            return;
        }
    }
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
    /// 假的 adb 服务收到的请求。
    adb_server: Arc<Mutex<Vec<String>>>,
}

impl Fixture {
    /// 假 `adb` 每次被调的参数，一行一次。
    fn adb_calls(&self) -> Vec<String> {
        std::fs::read_to_string(self.tmp.path().join("adb/calls"))
            .unwrap_or_default()
            .lines()
            .map(str::to_owned)
            .collect()
    }

    fn requests(&self) -> usize {
        self.mirror.lock().unwrap().requests
    }

    fn connections(&self) -> usize {
        self.mirror.lock().unwrap().connections
    }
}

const ADB_LIST: &str = "R58N70ABC3K2A          unauthorized usb:1-1 transport_id:1\n2B1F0C7E9A4D           device usb:1-2 product:husky model:Pixel_8 device:husky transport_id:2\n";

/// 假的 adb 服务：每个连接读一个 `<四位十六进制长度><服务名>`，记下服务名，答 `OKAY` 加设备表。
fn adb_server(seen: Arc<Mutex<Vec<String>>>) -> std::net::SocketAddr {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();
    std::thread::spawn(move || {
        for stream in listener.incoming() {
            let Ok(mut stream) = stream else { return };
            let mut len = [0u8; 4];
            stream.read_exact(&mut len).unwrap();
            let n = usize::from_str_radix(std::str::from_utf8(&len).unwrap(), 16).unwrap();
            let mut service = vec![0u8; n];
            stream.read_exact(&mut service).unwrap();
            seen.lock()
                .unwrap()
                .push(String::from_utf8(service).unwrap());
            let _ = stream.write_all(format!("OKAY{:04x}{ADB_LIST}", ADB_LIST.len()).as_bytes());
        }
    });
    addr
}

/// 一个没人听的本机端口。
fn closed_port() -> std::net::SocketAddr {
    TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap()
}

struct Options {
    /// 假 release 服务器上包的路径和包本身；sha256 照 `sha` 钉（`None` 是包本身的）。
    asset: Option<(String, Vec<u8>, Option<String>)>,
    arch: &'static str,
    /// 「mirroir serve」在跑（令牌文件在）。
    serving: bool,
    adb: bool,
    /// adb 自己的服务已经在跑（本机起一个答 `host:devices-l` 的假服务）。
    adb_running: bool,
}

impl Default for Options {
    fn default() -> Self {
        Options {
            asset: None,
            arch: "aarch64",
            serving: false,
            adb: true,
            adb_running: false,
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
            &(format!("echo \"$*\" >> '{}/calls'\n", adb_dir.display())
                + r#"case "$1" in
version) printf 'Android Debug Bridge version 1.0.41\nVersion 36.0.0-13206524\n' ;;
devices) printf 'List of devices attached\nR58N70ABC3K2A          unauthorized usb:1-1 transport_id:1\n2B1F0C7E9A4D           device usb:1-2 product:husky model:Pixel_8 device:husky transport_id:2\n\n' ;;
*) exit 2 ;;
esac"#),
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
        requests: 0,
        connections: 0,
        close: false,
        hang_up: false,
    }));
    let serve_url = mirror_server(Arc::clone(&mirror), bin.join("accepted"));
    // 连接的另一头在谁手里：`lsof` 照假服务写下的 `accepted`（守护进程那一头的端口）和 `peers`（每行
    // `<pid> <uid>`，空的是这个用户看不见）答，先列守护进程自己那一头；`-d txt` 照 `program` 答。
    // `launchctl print` 照 `launchd` 答（不在是没登记），`id -u` 是 501。默认是 launchd 里登记的
    // 「mirroir serve」接下了连接。端口上一直有驱动在听（`-sTCP:LISTEN` 答的那一行）。
    let port = serve_url
        .rsplit(':')
        .next()
        .unwrap()
        .trim_end_matches("/mcp");
    std::fs::write(bin.join("peers"), "4242 501\n").unwrap();
    std::fs::write(
        bin.join("launchd"),
        "gui/501/dev.mirroir.serve = {\n\tstate = running\n\tpid = 4242\n}\n",
    )
    .unwrap();
    std::fs::write(
        bin.join("program"),
        "p4242\nftxt\nn/Users/a/.mirroir-mcp/mirroir serve.app/Contents/MacOS/mirroir-mcp\n",
    )
    .unwrap();
    let b = bin.display();
    let lsof = script(
        &bin,
        "lsof",
        &format!(
            r#"case "$*" in
*-iTCP@127.0.0.1:{port}\ *-sTCP:ESTABLISHED*)
  test -e '{b}/accepted' || exit 1
  ours=$(cat '{b}/accepted')
  printf 'p999\nu501\nf20\nn127.0.0.1:%s->127.0.0.1:{port}\n' "$ours"
  while read -r pid uid; do
    printf 'p%s\nu%s\nf9\nn127.0.0.1:{port}->127.0.0.1:%s\n' "$pid" "$uid" "$ours"
  done < '{b}/peers' ;;
*-iTCP:{port}\ *-sTCP:LISTEN*) printf 'p4242\nu501\nf9\nn*:{port}\n' ;;
*-iTCP*) exit 1 ;;
*txt*) cat '{b}/program' ;;
*) exit 2 ;;
esac"#
        ),
    );
    let launchctl = script(
        &bin,
        "launchctl",
        &format!(
            "test \"$1 $2\" = 'print gui/501/dev.mirroir.serve' && test -e '{b}/launchd' || exit 113; cat '{b}/launchd'"
        ),
    );
    let id = script(&bin, "id", "echo 501");
    let seen = Arc::new(Mutex::new(Vec::new()));
    let adb_addr = if o.adb_running {
        adb_server(Arc::clone(&seen))
    } else {
        closed_port()
    };
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
        lsof,
        launchctl,
        id,
        program_limit: Duration::from_secs(30),
        sw_vers,
        mirroring_app,
        pgrep,
        open,
        pasteboard: vec![pasteboard.display().to_string()],
        adb_path: Some(adb_dir.clone().into_os_string()),
        adb_dirs: Vec::new(),
        adb_server: adb_addr,
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
        adb_server: seen,
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
        ("POST", "/phone/android/devices", None),
    ] {
        let (s, v) = call(&app, m, p, b).await;
        assert_eq!(s, StatusCode::NOT_FOUND, "{p}");
        assert_eq!(v["code"], "not-found");
        assert_eq!(v["hint"], "此版本的 usuyo 服务不支持手机。");
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
                "adb": {"path": f.tmp.path().join("adb/adb").display().to_string(), "version": "36.0.0", "running": false},
                "devices": [],
            },
        })
    );
    // 看这一页只问了 adb 的版本，没跑 `adb devices`；没有令牌文件，什么也没发给端口上的服务
    assert_eq!(f.adb_calls(), ["version"]);
    assert_eq!(f.requests(), 0);
    assert_eq!(f.connections(), 0);
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
            "下载的安装包与固定的 SHA256 不一致，已丢弃。这台电脑上没有放入任何文件。",
        ),
        (
            "/gone.tar.gz",
            None,
            "官方发布中没有这一版的安装包。这台电脑上没有放入任何文件。",
        ),
        (
            "/slow.tar.gz",
            None,
            "网络连接失败，或下载中断。这台电脑上没有放入任何文件。",
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
        "mirroir 只提供 Apple 芯片 Mac 的版本，无法安装在这台电脑上。"
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
    // 三十秒里不再截第二张
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
    assert_eq!(v["hint"], "无法识别「mirroir serve」的回复。");
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
    assert_eq!(v["hint"], "无法打开系统设置。");
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
    assert_eq!(v["hint"], "无法检查这台 Mac 的剪贴板是否变化。");
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
    assert_eq!(v["hint"], "无法读取这一屏：iPhone 镜像未连接，或尚未授权。");
    // 回的话里不带它说的字
    assert!(!v.to_string().contains("not running"));

    // 「mirroir serve」不在
    std::fs::remove_file(f.tmp.path().join("serve-token")).unwrap();
    let (s, v) = call(&f.app, "POST", "/phone/ios/dry-run", None).await;
    assert_eq!(s, StatusCode::FAILED_DEPENDENCY);
    assert_eq!(
        v["hint"],
        "「mirroir serve」未在这台 Mac 上运行。先按「驱动」一行中的步骤完成登记。"
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
            Some("下载的安装包与固定的 SHA256 不一致，已丢弃。这台电脑上没有放入任何文件。"),
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

/// 只有点了「找一下连着的手机」才跑 `adb devices`；回的页面里有设备，和怎么停掉留在后台的 adb 服务。
#[tokio::test]
async fn only_the_look_action_runs_adb_devices() {
    let f = fixture(Options::default());
    view(&f.app).await;
    view(&f.app).await;
    assert!(
        f.adb_calls().iter().all(|c| c == "version"),
        "{:?}",
        f.adb_calls()
    );
    let (s, v) = call(&f.app, "POST", "/phone/android/devices", None).await;
    assert_eq!(s, StatusCode::OK, "{v}");
    assert_eq!(
        f.adb_calls().iter().filter(|c| *c == "devices -l").count(),
        1
    );
    let adb = f.tmp.path().join("adb/adb");
    assert_eq!(
        v["android"],
        json!({
            "adb": {"path": adb.display().to_string(), "version": "36.0.0", "running": true},
            "devices": [
                {"serial": "R58N…3K2A", "state": "unauthorized"},
                {"serial": "2B1F…9A4D", "model": "Pixel 8", "state": "device"},
            ],
            "step": {
                "text": "adb 服务在这台电脑的后台运行，会一直保留，usuyo 不会停止它。要停止它，在「终端」中运行：",
                "commands": [format!("'{}' kill-server", adb.display())],
            },
        })
    );
    // 动作不发 `phone-updated`，回的就是整页
    assert_eq!(v["ios"]["system"]["macos"], "15.6.1");
}

/// adb 的服务已经在跑：经它的口子读设备，不跑 `adb devices`。
#[tokio::test]
async fn a_running_adb_server_is_asked_over_its_port() {
    let f = fixture(Options {
        adb_running: true,
        ..Options::default()
    });
    let v = view(&f.app).await;
    assert_eq!(v["android"]["adb"]["running"], true);
    assert_eq!(
        v["android"]["devices"],
        json!([
            {"serial": "R58N…3K2A", "state": "unauthorized"},
            {"serial": "2B1F…9A4D", "model": "Pixel 8", "state": "device"},
        ])
    );
    assert_eq!(
        v["android"]["step"]["commands"][0],
        format!("'{}' kill-server", f.tmp.path().join("adb/adb").display())
    );
    assert_eq!(f.adb_calls(), ["version"]);
    assert_eq!(*f.adb_server.lock().unwrap(), ["host:devices-l"]);
}

#[tokio::test]
async fn looking_for_phones_without_adb_says_so() {
    let f = fixture(Options {
        adb: false,
        ..Options::default()
    });
    let (s, v) = call(&f.app, "POST", "/phone/android/devices", None).await;
    assert_eq!(s, StatusCode::FAILED_DEPENDENCY);
    assert_eq!(v["code"], "dependency-missing");
    assert_eq!(v["hint"], "这台电脑上找不到 adb。");
}

/// 令牌只发给认得出的那一条连接：连接的另一头要是你自己跑的驱动。看不见的（别的用户的）、别的用户的、
/// 别的程序的、两个进程拿着的、认完以前就断了的，一个请求都不发。
#[tokio::test]
async fn the_token_goes_only_over_a_connection_the_driver_holds() {
    let f = fixture(Options {
        serving: true,
        ..Options::default()
    });
    // 装着驱动：认不出时页面说要你登记「mirroir serve」
    std::fs::create_dir_all(&f.driver_dir).unwrap();
    std::fs::write(f.driver_dir.join("mirroir-mcp"), b"driver").unwrap();
    std::fs::write(
        f.driver_dir.join("driver.json"),
        br#"{"version": "0.47.0", "asset": "a", "sha256": "b"}"#,
    )
    .unwrap();
    let not_sent = |v: &Value| {
        assert_eq!(
            v["ios"]["grants"],
            json!({"to": "mirroir serve", "screen": "unknown", "accessibility": "unknown"})
        );
        assert_eq!(v["ios"]["mirroring"], "not-set-up");
        assert_eq!(
            v["ios"]["driver"]["step"]["commands"][1],
            "launchctl bootstrap gui/$(id -u) ~/Library/LaunchAgents/dev.mirroir.serve.plist"
        );
    };
    let peers = |p: &str| std::fs::write(f.bin.join("peers"), p).unwrap();

    // 另一头这个用户看不见（别的用户占着同一个地址），端口上照样有驱动在听
    peers("");
    not_sent(&view(&f.app).await);
    // 另一头是别的用户的
    peers("4242 0\n");
    not_sent(&view(&f.app).await);
    // 自己的用户、不是登记的那一项、程序也不是装着的驱动
    peers("777 501\n");
    std::fs::write(f.bin.join("program"), "p777\nftxt\nn/tmp/catch-the-token\n").unwrap();
    not_sent(&view(&f.app).await);
    let (s, _) = call(&f.app, "POST", "/phone/ios/dry-run", None).await;
    assert_eq!(s, StatusCode::FAILED_DEPENDENCY);
    // 两个进程都拿着另一头
    peers("4242 501\n777 501\n");
    not_sent(&view(&f.app).await);
    // 认完以前服务就关了连接：平平地当它没在跑
    peers("4242 501\n");
    f.mirror.lock().unwrap().hang_up = true;
    let (s, v) = call(&f.app, "GET", "/phone", None).await;
    assert_eq!(s, StatusCode::OK, "{v}");
    not_sent(&v);
    assert_eq!(f.requests(), 0);
    assert!(f.connections() >= 6, "{}", f.connections());

    // launchd 里登记的那一项接下了连接：发，一次看这一页（`initialize` 和 `status`）只用一条连接
    f.mirror.lock().unwrap().hang_up = false;
    let before = f.connections();
    let v = view(&f.app).await;
    assert_eq!(v["ios"]["mirroring"], "connected");
    assert_eq!(f.requests(), 2);
    assert_eq!(f.connections(), before + 1);

    // 服务每答完一个就关：另开的那条从头再认，认不出就不发
    f.mirror.lock().unwrap().close = true;
    let v = view(&f.app).await;
    assert_eq!(v["ios"]["mirroring"], "connected");
    assert_eq!(f.requests(), 4);
    assert_eq!(f.connections(), before + 3);

    // 另一头是装着的那一份驱动（没登记进 launchd，自己起的）：发
    std::fs::remove_file(f.bin.join("launchd")).unwrap();
    peers("777 501\n");
    std::fs::write(
        f.bin.join("program"),
        format!(
            "p777\nftxt\nn{}\n",
            f.driver_dir.join("mirroir-mcp").display()
        ),
    )
    .unwrap();
    let v = view(&f.app).await;
    assert_eq!(v["ios"]["mirroring"], "connected");
    assert!(v["ios"]["driver"].get("step").is_none(), "{v}");
    assert_eq!(f.requests(), 6);
}
