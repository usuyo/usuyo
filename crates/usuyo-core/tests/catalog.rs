//! 订阅源和从订阅源装插件，对着本机起的假订阅源和假 release 资产：装上、更新、删除、不重启就载入，
//! 校验值不对、没有这台电脑的包、比声明的大、包里有往外跑的路径、开发时装的不被覆盖、ETag。不连外网。
#![cfg(unix)]

use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use sha2::{Digest, Sha256};
use tempfile::TempDir;
use usuyo_core::view::{CatalogPlugin, InstallOrigin};
use usuyo_core::{Engine, EngineConfig};

const ID: &str = "fake/hello";

/// 收到的每个请求：路径和 If-None-Match。
type Seen = Arc<Mutex<Vec<(String, Option<String>)>>>;

/// 路径 → (内容, ETag)。可以在跑着的时候换。
type Routes = Arc<Mutex<HashMap<String, (Vec<u8>, Option<String>)>>>;

struct Server {
    base: String,
    routes: Routes,
    /// 收到的每个请求：路径和 If-None-Match。
    seen: Seen,
}

impl Server {
    fn start() -> Server {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let base = format!("http://{}", listener.local_addr().unwrap());
        let routes: Routes = Arc::default();
        let seen: Seen = Arc::default();
        let (r, s) = (Arc::clone(&routes), Arc::clone(&seen));
        std::thread::spawn(move || {
            for stream in listener.incoming() {
                let Ok(stream) = stream else { return };
                let (r, s) = (Arc::clone(&r), Arc::clone(&s));
                std::thread::spawn(move || answer(stream, &r, &s));
            }
        });
        Server { base, routes, seen }
    }

    fn put(&self, path: &str, body: Vec<u8>, etag: Option<&str>) {
        self.routes
            .lock()
            .unwrap()
            .insert(path.to_owned(), (body, etag.map(str::to_owned)));
    }

    fn url(&self, path: &str) -> String {
        format!("{}{path}", self.base)
    }
}

fn answer(mut stream: TcpStream, routes: &Routes, seen: &Seen) {
    let mut reader = BufReader::new(stream.try_clone().unwrap());
    let mut line = String::new();
    if reader.read_line(&mut line).is_err() {
        return;
    }
    let path = line.split(' ').nth(1).unwrap_or_default().to_owned();
    let mut if_none = None;
    loop {
        let mut h = String::new();
        if reader.read_line(&mut h).is_err() || h.trim().is_empty() {
            break;
        }
        if let Some((k, v)) = h.split_once(':')
            && k.eq_ignore_ascii_case("if-none-match")
        {
            if_none = Some(v.trim().to_owned());
        }
    }
    seen.lock().unwrap().push((path.clone(), if_none.clone()));
    let found = routes.lock().unwrap().get(&path).cloned();
    let reply = match found {
        Some((_, Some(tag))) if if_none.as_deref() == Some(tag.as_str()) => {
            format!("HTTP/1.1 304 Not Modified\r\nETag: {tag}\r\nContent-Length: 0\r\nConnection: close\r\n\r\n").into_bytes()
        }
        Some((body, tag)) => {
            let tag = tag.map(|t| format!("ETag: {t}\r\n")).unwrap_or_default();
            let mut out = format!(
                "HTTP/1.1 200 OK\r\nContent-Length: {}\r\n{tag}Connection: close\r\n\r\n",
                body.len()
            )
            .into_bytes();
            out.extend_from_slice(&body);
            out
        }
        None => b"HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\nConnection: close\r\n\r\n".to_vec(),
    };
    let _ = stream.write_all(&reply);
}

fn manifest(version: &str) -> String {
    format!(
        r#"protocol: 1
id: {ID}
kind: fetch
name: 你好
version: {version}
license: MIT
maintainers: [someone]
platforms: [hello]
match: ["hello.test/*"]
methods:
  - id: page
    name: 公开页
    access: public-web
    login: none
    account: none
    capabilities:
      fetch:
        outputs: [text, meta]
permissions:
  fs: {{write: ["${{OUT}}"]}}
run:
  command: ["${{PLUGIN_DIR}}/hello"]
"#
    )
}

/// 一个 tar.gz：`entries` 是 (路径, 内容, 模式)。路径照原样写进头里，坏路径也写得进去。
fn tar_gz(entries: &[(&str, &[u8], u32)]) -> Vec<u8> {
    let mut b = tar::Builder::new(flate2::write::GzEncoder::new(
        Vec::new(),
        flate2::Compression::fast(),
    ));
    for (path, body, mode) in entries {
        let mut h = tar::Header::new_gnu();
        h.set_size(body.len() as u64);
        h.set_mode(*mode);
        let name = path.as_bytes();
        h.as_old_mut().name[..name.len()].copy_from_slice(name);
        h.set_cksum();
        b.append(&h, *body).unwrap();
    }
    b.into_inner().unwrap().finish().unwrap()
}

fn good_package(version: &str) -> Vec<u8> {
    tar_gz(&[
        ("usuyo-plugin.yaml", manifest(version).as_bytes(), 0o644),
        ("hello", b"#!/bin/sh\necho '{\"ok\": true, \"ready\": true, \"protocol\": 1, \"version\": \"1\"}'\n", 0o644),
    ])
}

fn sha(b: &[u8]) -> String {
    Sha256::digest(b)
        .iter()
        .map(|x| format!("{x:02x}"))
        .collect()
}

fn target() -> &'static str {
    match (std::env::consts::OS, std::env::consts::ARCH) {
        ("macos", "aarch64") => "macos-arm64",
        ("macos", "x86_64") => "macos-x86_64",
        _ => "linux-x86_64",
    }
}

/// 索引：这个插件的一版，包在 `asset_path`，记的大小和 sha256 由调用方给。
fn index(
    server: &Server,
    version: &str,
    asset_path: &str,
    size: usize,
    sha256: &str,
    asset_target: &str,
) -> Vec<u8> {
    serde_json::to_vec(&serde_json::json!({
        "format": 1,
        "generatedAt": "2026-11-13T00:00:00.000Z",
        "platforms": {"hello": {"zh-Hans": "你好", "en": "Hello"}},
        "plugins": [{
            "id": ID,
            "kind": "fetch",
            "name": {"zh-Hans": "你好", "en": "Hello"},
            "description": {"zh-Hans": "测试用", "en": "For tests"},
            "platforms": ["hello"],
            "repository": "https://github.com/fake/hello",
            "license": "MIT",
            "methods": [{"id": "page", "name": {"zh-Hans": "公开页", "en": "Public page"}, "built": true,
                         "access": "public-web", "login": "none", "account": "none"}],
            "release": {
                "version": version,
                "tag": format!("v{version}"),
                "publishedAt": "2026-11-13T00:00:00.000Z",
                "assets": [{"target": asset_target, "name": format!("hello_{version}_{asset_target}.tar.gz"),
                            "url": server.url(asset_path), "size": size, "sha256": sha256}]
            }
        }, {
            "id": "fake/unreleased",
            "kind": "fetch",
            "name": {"zh-Hans": "未发布"},
            "platforms": ["hello"],
            "repository": "https://github.com/fake/unreleased",
            "license": "MIT",
            "release": null
        }]
    }))
    .unwrap()
}

struct Fixture {
    tmp: TempDir,
    library: PathBuf,
    server: Server,
}

impl Fixture {
    fn new() -> Fixture {
        let tmp = tempfile::tempdir().unwrap();
        let library =
            Engine::init_library(&tmp.path().join("lib"), &tmp.path().join("machine")).unwrap();
        Fixture {
            tmp,
            library,
            server: Server::start(),
        }
    }

    fn machine(&self) -> PathBuf {
        self.tmp.path().join("machine")
    }

    fn source(&self) -> String {
        format!("{}/src", self.server.base)
    }

    fn engine(&self) -> Engine {
        let mut config = EngineConfig::new(&self.library, self.machine());
        config.default_sources = vec![self.source()];
        config.dev_loopback_sources = true;
        Engine::open(config).unwrap()
    }

    /// 放好一版：包和索引。
    fn publish(&self, version: &str, package: &[u8]) {
        let path = format!("/dl/{version}.tar.gz");
        self.server.put(&path, package.to_vec(), None);
        let idx = index(
            &self.server,
            version,
            &path,
            package.len(),
            &sha(package),
            target(),
        );
        self.server
            .put("/src/index.json", idx, Some(&format!("\"{version}\"")));
    }

    fn plugin_dir(&self) -> PathBuf {
        self.machine().join("plugins").join(ID)
    }
}

fn entry(engine: &Engine, id: &str) -> CatalogPlugin {
    engine.catalog_plugin(id).unwrap()
}

/// 等这个插件手上的事做完。
async fn settled(engine: &Engine, id: &str) -> CatalogPlugin {
    for _ in 0..200 {
        let e = entry(engine, id);
        if e.action.is_none() {
            return e;
        }
        tokio::time::sleep(Duration::from_millis(25)).await;
    }
    panic!("{id} did not settle");
}

fn versions(dir: &Path) -> Vec<String> {
    let mut v: Vec<String> = std::fs::read_dir(dir)
        .map(|r| {
            r.filter_map(Result::ok)
                .map(|e| e.file_name().to_string_lossy().into_owned())
                .collect()
        })
        .unwrap_or_default();
    v.sort();
    v
}

#[tokio::test(flavor = "multi_thread")]
async fn install_update_and_remove_without_restarting() {
    let f = Fixture::new();
    f.publish("0.1.0", &good_package("0.1.0"));
    let engine = f.engine();
    engine.refresh_plugin_sources(None).await.unwrap();
    let sources = engine.source_list();
    assert_eq!(sources[0].plugins, Some(2));
    assert!(sources[0].error.is_none());

    let e = entry(&engine, ID);
    assert!(e.installed.is_none());
    assert!(e.blocked.is_none(), "{:?}", e.blocked);
    assert_eq!(e.name, "你好");
    assert_eq!(
        entry(&engine, "fake/unreleased").blocked.unwrap().code,
        "no-release"
    );
    assert!(engine.install_plugin("fake/unreleased").is_err());

    engine.install_plugin(ID).unwrap();
    let e = settled(&engine, ID).await;
    assert!(e.failure.is_none(), "{:?}", e.failure);
    let installed = e.installed.unwrap();
    assert_eq!(installed.version, "0.1.0");
    assert_eq!(installed.origin, InstallOrigin::Source);
    assert_eq!(installed.source.as_deref(), Some(f.source().as_str()));
    assert!(
        engine.plugin_ids().contains(&ID.to_owned()),
        "loaded without a restart"
    );
    let bin = f.plugin_dir().join("0.1.0/hello");
    use std::os::unix::fs::PermissionsExt;
    assert_eq!(
        std::fs::metadata(&bin).unwrap().permissions().mode() & 0o111,
        0o111
    );
    assert!(engine.install_plugin(ID).is_err(), "installed twice");
    assert!(engine.update_plugin(ID).is_err(), "nothing newer");
    assert!(
        std::fs::read_dir(f.machine().join("plugins/.staging"))
            .unwrap()
            .next()
            .is_none()
    );

    f.publish("0.2.0", &good_package("0.2.0"));
    engine.refresh_plugin_sources(None).await.unwrap();
    assert!(entry(&engine, ID).update);
    engine.update_plugin(ID).unwrap();
    let e = settled(&engine, ID).await;
    assert!(e.failure.is_none(), "{:?}", e.failure);
    assert_eq!(e.installed.unwrap().version, "0.2.0");
    assert!(!e.update);
    assert_eq!(versions(&f.plugin_dir()), vec!["0.2.0"]);

    let view = engine.remove_plugin(ID).await.unwrap();
    let e = view.plugins.iter().find(|p| p.id == ID).unwrap();
    assert!(e.installed.is_none());
    assert!(!f.plugin_dir().exists());
    assert!(!engine.plugin_ids().contains(&ID.to_owned()));
}

#[tokio::test(flavor = "multi_thread")]
async fn a_wrong_checksum_installs_nothing() {
    let f = Fixture::new();
    let package = good_package("0.1.0");
    f.server.put("/dl/x.tar.gz", package.clone(), None);
    let idx = index(
        &f.server,
        "0.1.0",
        "/dl/x.tar.gz",
        package.len(),
        &"0".repeat(64),
        target(),
    );
    f.server.put("/src/index.json", idx, None);
    let engine = f.engine();
    engine.refresh_plugin_sources(None).await.unwrap();
    engine.install_plugin(ID).unwrap();
    let e = settled(&engine, ID).await;
    assert!(e.installed.is_none());
    assert!(e.failure.unwrap().contains("校验值"));
    assert!(!f.plugin_dir().exists());
    assert!(
        std::fs::read_dir(f.machine().join("plugins/.staging"))
            .unwrap()
            .next()
            .is_none()
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn no_package_for_this_computer() {
    let f = Fixture::new();
    let package = good_package("0.1.0");
    f.server.put("/dl/x.tar.gz", package.clone(), None);
    let other = if target() == "linux-x86_64" {
        "windows-x86_64"
    } else {
        "linux-x86_64"
    };
    let idx = index(
        &f.server,
        "0.1.0",
        "/dl/x.tar.gz",
        package.len(),
        &sha(&package),
        other,
    );
    f.server.put("/src/index.json", idx, None);
    let engine = f.engine();
    engine.refresh_plugin_sources(None).await.unwrap();
    let e = entry(&engine, ID);
    assert_eq!(e.blocked.unwrap().code, "no-asset");
    assert!(e.latest.unwrap().asset.is_none());
    assert!(engine.install_plugin(ID).is_err());
}

#[tokio::test(flavor = "multi_thread")]
async fn a_package_larger_than_declared_is_dropped() {
    let f = Fixture::new();
    let package = good_package("0.1.0");
    f.server.put("/dl/x.tar.gz", package.clone(), None);
    let idx = index(
        &f.server,
        "0.1.0",
        "/dl/x.tar.gz",
        package.len() - 10,
        &sha(&package),
        target(),
    );
    f.server.put("/src/index.json", idx, None);
    let engine = f.engine();
    engine.refresh_plugin_sources(None).await.unwrap();
    engine.install_plugin(ID).unwrap();
    let e = settled(&engine, ID).await;
    assert!(e.installed.is_none());
    assert!(e.failure.is_some());
    assert!(!f.plugin_dir().exists());
}

#[tokio::test(flavor = "multi_thread")]
async fn a_hostile_entry_installs_nothing() {
    let f = Fixture::new();
    let package = tar_gz(&[
        ("usuyo-plugin.yaml", manifest("0.1.0").as_bytes(), 0o644),
        ("hello", b"#!/bin/sh\n", 0o755),
        ("../../escaped", b"x", 0o644),
    ]);
    f.publish("0.1.0", &package);
    let engine = f.engine();
    engine.refresh_plugin_sources(None).await.unwrap();
    engine.install_plugin(ID).unwrap();
    let e = settled(&engine, ID).await;
    assert!(e.installed.is_none());
    assert!(e.failure.unwrap().contains("不允许"));
    assert!(!f.plugin_dir().exists());
    assert!(!f.machine().join("escaped").exists());
    assert!(!f.machine().join("plugins/escaped").exists());
}

#[tokio::test(flavor = "multi_thread")]
async fn a_package_whose_manifest_disagrees_installs_nothing() {
    let f = Fixture::new();
    // 包里是 0.9.0，索引说 0.1.0
    let package = good_package("0.9.0");
    f.publish("0.1.0", &package);
    let engine = f.engine();
    engine.refresh_plugin_sources(None).await.unwrap();
    engine.install_plugin(ID).unwrap();
    let e = settled(&engine, ID).await;
    assert!(e.installed.is_none());
    assert!(e.failure.unwrap().contains("清单"));
}

#[tokio::test(flavor = "multi_thread")]
async fn a_development_install_is_never_overwritten() {
    let f = Fixture::new();
    // 照 just dev-plugins 的样子放一版，没有安装记录
    let dev = f.plugin_dir().join("0.0.1");
    std::fs::create_dir_all(&dev).unwrap();
    std::fs::write(dev.join("usuyo-plugin.yaml"), manifest("0.0.1")).unwrap();
    std::fs::write(dev.join("hello"), "#!/bin/sh\n").unwrap();
    f.publish("0.1.0", &good_package("0.1.0"));
    let engine = f.engine();
    engine.refresh_plugin_sources(None).await.unwrap();
    let e = entry(&engine, ID);
    let installed = e.installed.unwrap();
    assert_eq!(installed.origin, InstallOrigin::Development);
    assert_eq!(e.blocked.unwrap().code, "development");
    assert!(!e.update);
    assert!(engine.update_plugin(ID).is_err());
    assert!(engine.install_plugin(ID).is_err());
    assert!(engine.remove_plugin(ID).await.is_err());
    assert_eq!(versions(&f.plugin_dir()), vec!["0.0.1"]);
}

#[tokio::test(flavor = "multi_thread")]
async fn sources_are_cached_with_their_etag_and_can_be_added_and_removed() {
    let f = Fixture::new();
    f.publish("0.1.0", &good_package("0.1.0"));
    let engine = f.engine();
    engine.refresh_plugin_sources(None).await.unwrap();
    engine.refresh_plugin_sources(None).await.unwrap();
    let asked: Vec<Option<String>> = f
        .server
        .seen
        .lock()
        .unwrap()
        .iter()
        .filter(|(p, _)| p == "/src/index.json")
        .map(|(_, t)| t.clone())
        .collect();
    assert_eq!(asked, vec![None, Some("\"0.1.0\"".to_owned())]);
    drop(engine);

    // 重开：没联网前就从缓存里有目录
    let engine = f.engine();
    assert_eq!(engine.source_list()[0].plugins, Some(2));
    assert!(entry(&engine, ID).latest.is_some());

    let err = engine.add_plugin_source("not a source").await.unwrap_err();
    assert_eq!(err.code(), "invalid-args");
    assert_eq!(
        engine
            .add_plugin_source(&f.source())
            .await
            .unwrap_err()
            .code(),
        "conflict"
    );
    let missing = format!("{}/none", f.server.base);
    let list = engine.add_plugin_source(&missing).await.unwrap();
    assert_eq!(list.len(), 2);
    assert!(
        list[1].error.is_some(),
        "a source without index.json says so"
    );
    let settings: serde_json::Value =
        serde_json::from_slice(&std::fs::read(f.machine().join("settings.json")).unwrap()).unwrap();
    assert_eq!(settings["pluginSources"].as_array().unwrap().len(), 2);

    let list = engine.remove_plugin_source(&f.source()).await.unwrap();
    assert_eq!(list.len(), 1);
    assert!(engine.catalog_plugin(ID).is_err(), "gone with its source");
}

/// 包里的清单的 `run.command` 指向插件文件夹外面：装不上，什么都不留下。
async fn refused_with_command(command: &str, extra: &[(&str, &[u8], u32)]) -> CatalogPlugin {
    let f = Fixture::new();
    let text = manifest("0.1.0").replace("${PLUGIN_DIR}/hello", command);
    let mut entries: Vec<(&str, &[u8], u32)> = vec![("usuyo-plugin.yaml", text.as_bytes(), 0o644)];
    entries.extend_from_slice(extra);
    f.publish("0.1.0", &tar_gz(&entries));
    let engine = f.engine();
    engine.refresh_plugin_sources(None).await.unwrap();
    engine.install_plugin(ID).unwrap();
    let e = settled(&engine, ID).await;
    assert!(e.installed.is_none(), "{command}");
    assert!(!f.plugin_dir().exists(), "{command}");
    assert!(
        std::fs::read_dir(f.machine().join("plugins/.staging"))
            .unwrap()
            .next()
            .is_none()
    );
    e
}

#[tokio::test(flavor = "multi_thread")]
async fn a_program_outside_the_plugin_folder_is_refused() {
    let e = refused_with_command(
        "${PLUGIN_DIR}/../hello",
        &[("hello", b"#!/bin/sh\n", 0o755)],
    )
    .await;
    assert!(e.failure.is_some());
    let e =
        refused_with_command("${PLUGIN_DIR}//bin/sh", &[("hello", b"#!/bin/sh\n", 0o755)]).await;
    assert!(e.failure.is_some());
}

#[tokio::test(flavor = "multi_thread")]
async fn a_link_in_the_package_is_refused() {
    let f = Fixture::new();
    let mut b = tar::Builder::new(flate2::write::GzEncoder::new(
        Vec::new(),
        flate2::Compression::fast(),
    ));
    let text = manifest("0.1.0");
    let mut h = tar::Header::new_gnu();
    h.set_size(text.len() as u64);
    h.set_mode(0o644);
    h.set_path("usuyo-plugin.yaml").unwrap();
    h.set_cksum();
    b.append(&h, text.as_bytes()).unwrap();
    let mut l = tar::Header::new_gnu();
    l.set_entry_type(tar::EntryType::Symlink);
    l.set_size(0);
    l.set_path("hello").unwrap();
    l.set_link_name("/bin/sh").unwrap();
    l.set_cksum();
    b.append(&l, std::io::empty()).unwrap();
    let package = b.into_inner().unwrap().finish().unwrap();
    f.publish("0.1.0", &package);
    let engine = f.engine();
    engine.refresh_plugin_sources(None).await.unwrap();
    engine.install_plugin(ID).unwrap();
    let e = settled(&engine, ID).await;
    assert!(e.installed.is_none());
    assert!(e.failure.unwrap().contains("不允许"));
    assert!(!f.plugin_dir().exists());
}
