//! 驱动的下载和解开对着本机起的假 release 服务器（好的包、sha256 不对、404、断流、包里不对），
//! 和 mirroir、adb 回答的解析、演练的比对。不连外网，不跑真的程序。

use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::time::Duration;

use super::adb;
use super::driver::{self, Asset, InstallError, Release};
use super::flows;
use super::serve::{self, Window};
use crate::models::{ModelError, Net};
use crate::view::{DeviceState, GrantState, Mirroring};

#[derive(Clone)]
enum Reply {
    Bytes(Vec<u8>),
    /// 写前 `n` 个字节以后不动了。
    Stall(Vec<u8>, usize),
    Status(u16),
}

fn serve(routes: Vec<(&str, Reply)>) -> String {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let base = format!("http://{}", listener.local_addr().unwrap());
    let routes: HashMap<String, Reply> =
        routes.into_iter().map(|(p, r)| (p.to_owned(), r)).collect();
    std::thread::spawn(move || {
        for stream in listener.incoming() {
            let Ok(stream) = stream else { return };
            let routes = routes.clone();
            std::thread::spawn(move || answer(stream, &routes));
        }
    });
    base
}

fn answer(mut stream: TcpStream, routes: &HashMap<String, Reply>) {
    let mut reader = BufReader::new(stream.try_clone().unwrap());
    let mut line = String::new();
    reader.read_line(&mut line).unwrap();
    let path = line.split(' ').nth(1).unwrap_or_default().to_owned();
    loop {
        let mut h = String::new();
        reader.read_line(&mut h).unwrap();
        if h.trim().is_empty() {
            break;
        }
    }
    let head = |status: &str, len: usize| {
        format!("HTTP/1.1 {status}\r\nContent-Length: {len}\r\nConnection: close\r\n\r\n")
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
}

/// tar.gz：`entries` 照写，模式 0644（解开以后要变成 0755）。
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
        let name = &mut h.as_gnu_mut().unwrap().name;
        name[..path.len()].copy_from_slice(path.as_bytes());
        h.set_cksum();
        b.append(&h, *data).unwrap();
    }
    b.into_inner().unwrap().finish().unwrap()
}

fn release(base: &str, path: &str, bytes: &[u8]) -> (Release, Asset) {
    let asset = Asset {
        arch: "aarch64".into(),
        name: "mirroir-mcp-darwin-arm64.tar.gz".into(),
        url: format!("{base}{path}"),
        size: bytes.len() as u64,
        sha256: usuyo_library::sha256_hex(bytes),
    };
    let release = Release {
        name: "mirroir-mcp".into(),
        version: "0.47.0".into(),
        license: "Apache-2.0".into(),
        page: format!("{base}/releases/tag/v0.47.0"),
        binary: "mirroir-mcp".into(),
        assets: vec![asset.clone()],
    };
    (release, asset)
}

fn net() -> Net {
    Net::loopback(Duration::from_millis(400)).unwrap()
}

/// 驱动目录的父目录里还有什么（除了驱动目录本身）。
fn leftovers(dir: &Path) -> Vec<PathBuf> {
    let parent = dir.parent().unwrap();
    match std::fs::read_dir(parent) {
        Ok(rd) => rd.map(|e| e.unwrap().path()).filter(|p| p != dir).collect(),
        Err(_) => Vec::new(),
    }
}

fn driver_dir(tmp: &tempfile::TempDir) -> PathBuf {
    tmp.path().join("machine/drivers/mirroir-mcp")
}

#[test]
fn the_pin_parses_and_names_one_arm64_asset() {
    let r = Release::pinned().unwrap();
    assert_eq!(r.version, "0.47.0");
    assert_eq!(r.binary, "mirroir-mcp");
    assert!(
        r.page
            .starts_with("https://github.com/jfarcand/mirroir-mcp/releases/tag/v")
    );
    let [a] = r.assets.as_slice() else {
        panic!("one asset")
    };
    assert_eq!(a.arch, "aarch64");
    assert!(
        a.url
            .starts_with("https://github.com/jfarcand/mirroir-mcp/releases/download/v0.47.0/")
    );
    assert!(a.url.ends_with(&a.name));
    assert_eq!(a.sha256.len(), 64);
    assert!(
        a.sha256
            .bytes()
            .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
    );
    assert!(a.size > 0);
}

#[tokio::test]
async fn installs_the_program_executable_with_its_record_and_nothing_else() {
    use std::os::unix::fs::PermissionsExt;
    let tmp = tempfile::tempdir().unwrap();
    let dir = driver_dir(&tmp);
    let pack = tar_gz(&[
        ("._mirroir-mcp", b"meta"),
        ("mirroir-mcp", b"#!/bin/sh\necho fake\n"),
    ]);
    let base = serve(vec![("/good.tar.gz", Reply::Bytes(pack.clone()))]);
    let (r, a) = release(&base, "/good.tar.gz", &pack);
    driver::install(&net(), &dir, &r, &a).await.unwrap();
    assert_eq!(
        driver::installed(&dir, "mirroir-mcp").as_deref(),
        Some("0.47.0")
    );
    let bin = dir.join("mirroir-mcp");
    assert_eq!(std::fs::read(&bin).unwrap(), b"#!/bin/sh\necho fake\n");
    assert_eq!(
        std::fs::metadata(&bin).unwrap().permissions().mode() & 0o777,
        0o755
    );
    let mut names: Vec<_> = std::fs::read_dir(&dir)
        .unwrap()
        .map(|e| e.unwrap().file_name().into_string().unwrap())
        .collect();
    names.sort();
    assert_eq!(names, ["driver.json", "mirroir-mcp"]);
    assert!(leftovers(&dir).is_empty(), "{:?}", leftovers(&dir));

    // 换一份：旧的整个换掉
    let newer = tar_gz(&[("./mirroir-mcp", b"v2")]);
    let base = serve(vec![("/v2.tar.gz", Reply::Bytes(newer.clone()))]);
    let (r, a) = release(&base, "/v2.tar.gz", &newer);
    driver::install(&net(), &dir, &r, &a).await.unwrap();
    assert_eq!(std::fs::read(&bin).unwrap(), b"v2");
    assert!(leftovers(&dir).is_empty());

    driver::remove(&dir).unwrap();
    assert!(!dir.exists());
    assert!(leftovers(&dir).is_empty());
    assert_eq!(driver::installed(&dir, "mirroir-mcp"), None);
    driver::remove(&dir).unwrap();
}

/// 装不上的每一种：错误、给人看的那一句、什么都没留下（原来那份不动）。
#[tokio::test]
async fn every_failed_install_leaves_nothing_and_says_one_sentence() {
    let pack = tar_gz(&[("mirroir-mcp", b"program")]);
    let extra = tar_gz(&[("mirroir-mcp", b"program"), ("other", b"x")]);
    let empty = tar_gz(&[("README", b"x")]);
    let base = serve(vec![
        ("/good.tar.gz", Reply::Bytes(pack.clone())),
        ("/extra.tar.gz", Reply::Bytes(extra.clone())),
        ("/empty.tar.gz", Reply::Bytes(empty.clone())),
        ("/slow.tar.gz", Reply::Stall(pack.clone(), 10)),
        ("/gone.tar.gz", Reply::Status(404)),
        ("/broken.tar.gz", Reply::Status(500)),
    ]);
    let zeros = "0".repeat(64);
    let cases: Vec<(&str, Vec<u8>, Option<&str>, &str)> = vec![
        (
            "/good.tar.gz",
            pack.clone(),
            Some(&zeros),
            "下载下来的包和钉住的 SHA256 对不上，已经丢掉了，没有放进这台电脑。",
        ),
        (
            "/gone.tar.gz",
            pack.clone(),
            None,
            "官方 release 上没有这一版的包，什么都没放进这台电脑。",
        ),
        (
            "/broken.tar.gz",
            pack.clone(),
            None,
            "网络不通，或者下载断了，什么都没放进这台电脑。",
        ),
        (
            "/slow.tar.gz",
            pack.clone(),
            None,
            "网络不通，或者下载断了，什么都没放进这台电脑。",
        ),
        (
            "/extra.tar.gz",
            extra.clone(),
            None,
            "下载下来的包里不是要的那个程序，已经丢掉了，没有放进这台电脑。",
        ),
        (
            "/empty.tar.gz",
            empty.clone(),
            None,
            "下载下来的包里不是要的那个程序，已经丢掉了，没有放进这台电脑。",
        ),
    ];
    for (path, bytes, sha, sentence) in cases {
        let tmp = tempfile::tempdir().unwrap();
        let dir = driver_dir(&tmp);
        // 已经装着一份：没装上时它不动
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("mirroir-mcp"), b"old").unwrap();
        std::fs::write(
            dir.join("driver.json"),
            br#"{"version": "0.46.0", "asset": "a", "sha256": "b"}"#,
        )
        .unwrap();
        let (r, mut a) = release(&base, path, &bytes);
        if let Some(s) = sha {
            a.sha256 = s.to_owned();
        }
        let e = driver::install(&net(), &dir, &r, &a).await.unwrap_err();
        assert_eq!(driver::hint(&e), sentence, "{path}: {e}");
        assert_eq!(
            driver::installed(&dir, "mirroir-mcp").as_deref(),
            Some("0.46.0"),
            "{path}"
        );
        assert_eq!(std::fs::read(dir.join("mirroir-mcp")).unwrap(), b"old");
        assert!(leftovers(&dir).is_empty(), "{path}: {:?}", leftovers(&dir));
    }
}

#[test]
fn the_other_failure_sentences() {
    let refused = InstallError::Download(ModelError::Refused("proxy".into()));
    assert_eq!(
        driver::hint(&refused),
        "连不到 GitHub（看看设置里的代理），什么都没放进这台电脑。"
    );
    let io = InstallError::Io {
        path: "/x".into(),
        source: std::io::Error::other("full"),
    };
    assert_eq!(
        driver::hint(&io),
        "存不进这台电脑的驱动目录，什么都没留下。"
    );
}

#[test]
fn the_steps_quote_the_program_path() {
    let s = driver::register_step(Path::new(
        "/Users/a/Library/Application Support/usuyo/drivers/mirroir-mcp/mirroir-mcp",
    ));
    assert_eq!(
        s.commands,
        [
            "'/Users/a/Library/Application Support/usuyo/drivers/mirroir-mcp/mirroir-mcp' install-agent",
            "launchctl bootstrap gui/$(id -u) ~/Library/LaunchAgents/dev.mirroir.serve.plist",
        ]
    );
    assert!(s.text.contains("钥匙串"));
    assert_eq!(
        driver::renew_step(Path::new("/d/it's")).commands[0],
        r"'/d/it'\''s' install-agent"
    );
    assert_eq!(
        driver::stop_step().commands,
        ["launchctl bootout gui/$(id -u)/dev.mirroir.serve"]
    );
}

#[test]
fn reads_mirroir_status_and_health_lines() {
    // InfoTools.swift 的 `status` 回答的开头
    assert_eq!(
        serve::window("Connected — mirroring active (window: 326x720 pt, pos=(10,20), portrait)"),
        Window::Connected
    );
    assert_eq!(
        serve::window("Paused — the window says Unlock iPhone."),
        Window::Paused
    );
    assert_eq!(
        serve::window("Not running — 'iPhone Mirroring' is not open."),
        Window::NotRunning
    );
    assert_eq!(
        serve::window("No window — app is running but no mirroring window found"),
        Window::Gone
    );
    assert_eq!(
        serve::window("Window minimized — 'iPhone Mirroring' is running"),
        Window::Minimized
    );
    assert_eq!(serve::window("the Mac's screen is locked"), Window::Other);

    assert_eq!(serve::mirroring(Window::Connected), Mirroring::Connected);
    assert_eq!(serve::mirroring(Window::NotRunning), Mirroring::NotSetUp);
    assert_eq!(serve::mirroring(Window::Paused), Mirroring::Away);
    assert_eq!(serve::mirroring(Window::Other), Mirroring::Away);

    use GrantState::{Given, Missing, Unknown};
    assert_eq!(serve::grants(Window::Connected, None), (Given, Given));
    assert_eq!(serve::grants(Window::Paused, Some(false)), (Missing, Given));
    assert_eq!(serve::grants(Window::Paused, None), (Unknown, Given));
    assert_eq!(serve::grants(Window::Gone, Some(true)), (Given, Missing));
    assert_eq!(serve::grants(Window::Gone, Some(false)), (Unknown, Unknown));
    assert_eq!(serve::grants(Window::NotRunning, None), (Unknown, Unknown));
    assert_eq!(
        serve::grants(Window::Minimized, Some(true)),
        (Unknown, Unknown)
    );

    let ok = "Issues detected\n\n[ok] 'iPhone Mirroring' process is running\n[FAIL] 'iPhone Mirroring' running but no window found\n[ok] Screen capture working (42 ms)\n[ok] Text recognition working (30 ms)";
    assert_eq!(serve::capture_ok(ok), Some(true));
    let fail = "Issues detected\n\n[FAIL] Screen capture failed — open System Settings and turn on mirroir serve.";
    assert_eq!(serve::capture_ok(fail), Some(false));
    assert_eq!(serve::capture_ok("All checks passed"), None);
}

#[test]
fn reads_describe_screen_text_lines() {
    let answer = "Screen elements (tap coordinates in points):\n- \"收藏\"\n- \"赞过\" [tree]\n- \"他说 \"好\"\"\n- \"\"\n\n_meta: ocr_time_ms=12 element_count=4";
    assert_eq!(serve::texts(answer), ["收藏", "赞过", "他说 \"好\""]);
}

#[test]
fn reads_adb_answers_and_shortens_serials() {
    let v = "Android Debug Bridge version 1.0.41\nVersion 36.0.0-13206524\nInstalled as /opt/homebrew/bin/adb\n";
    assert_eq!(adb::version(v).as_deref(), Some("36.0.0"));
    assert_eq!(
        adb::version("Android Debug Bridge version 1.0.41\n").as_deref(),
        Some("1.0.41")
    );
    let out = "* daemon not running; starting now at tcp:5037\n* daemon started successfully\nList of devices attached\nR58N70ABC3K2A          unauthorized usb:1-1 transport_id:1\n2B1F0C7E9A4D           device usb:1-2 product:husky model:Pixel_8 device:husky transport_id:2\nemulator-5554          offline transport_id:3\nZY22ABCDEF             recovery transport_id:4\n\n";
    let d = adb::devices(out);
    assert_eq!(d.len(), 3);
    assert_eq!(d[0].serial, "R58N…3K2A");
    assert_eq!(d[0].state, DeviceState::Unauthorized);
    assert_eq!(d[0].model, None);
    assert_eq!(d[1].serial, "2B1F…9A4D");
    assert_eq!(d[1].model.as_deref(), Some("Pixel 8"));
    assert_eq!(d[1].state, DeviceState::Device);
    assert_eq!(
        d[2].serial,
        "emulator-5554".chars().take(4).collect::<String>() + "…" + "5554"
    );
    assert_eq!(d[2].state, DeviceState::Offline);
    assert_eq!(adb::short("abc"), "abc");
}

fn testdata_flows() -> Vec<flows::Flow> {
    flows::load(&[Path::new(env!("CARGO_MANIFEST_DIR")).join("testdata/phone/flows")])
}

#[test]
fn the_dry_run_finds_the_step_a_screen_looks_like() {
    let all = testdata_flows();
    assert_eq!(all.len(), 3);
    let screen: Vec<String> = ["关注", "笔记", "收藏", "赞过", "某某的名字"]
        .map(String::from)
        .to_vec();
    let m = flows::best(&all, &screen).unwrap();
    assert_eq!(m.flow, "小红书 · 收藏");
    assert_eq!(m.step, 2);
    assert_eq!((m.expect.matched, m.expect.total), (3, 3));
    assert_eq!(m.targets.len(), 1);
    assert_eq!(m.targets[0].label, "收藏");
    assert!(m.targets[0].found);

    // 变体：同一步另一种写法
    let screen: Vec<String> = ["赞过", "收藏", "作品"].map(String::from).to_vec();
    let m = flows::best(&all, &screen).unwrap();
    assert_eq!(m.flow, "视频号 · 赞和收藏");
    assert_eq!(m.step, 4);
    assert_eq!(m.targets[0].label, "赞过");

    // 要点的字没找到
    let screen: Vec<String> = ["分享", "举报"].map(String::from).to_vec();
    let m = flows::best(&all, &screen).unwrap();
    assert_eq!((m.flow.as_str(), m.step), ("小红书 · 收藏", 4));
    assert_eq!((m.expect.matched, m.expect.total), (1, 2));
    assert!(!m.targets[0].found);

    let none: Vec<String> = ["天气", "日历"].map(String::from).to_vec();
    assert!(flows::best(&all, &none).is_none());
    assert!(flows::best(&[], &screen).is_none());
}

#[test]
fn broken_flow_files_are_skipped() {
    let tmp = tempfile::tempdir().unwrap();
    std::fs::write(tmp.path().join("a.json"), b"{not json").unwrap();
    std::fs::write(
        tmp.path().join("b.json"),
        br#"{"name": "n", "steps": [{"expect": ["x"]}]}"#,
    )
    .unwrap();
    std::fs::write(tmp.path().join("c.txt"), b"ignored").unwrap();
    let got = flows::load(&[tmp.path().to_owned(), tmp.path().join("missing")]);
    assert_eq!(got.len(), 1);
    assert_eq!(got[0].name, "n");
}
