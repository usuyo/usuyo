//! 模型下载对着本机起的 http 测试服务器：续传、镜像依次试、sha256 不对整个丢掉、跳转、断流、
//! 超出声明的大小、压缩包里的坏路径和链接、解开的文件和 `model.json`。不连外网。

use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};

use std::sync::{Arc, Mutex};
use std::time::Duration;
use usuyo_plugin::ProxySetting;
use usuyo_plugin_kit::http::Proxy;

use super::*;

/// 测试服务器怎么答一个路径。
#[derive(Clone)]
enum Reply {
    /// 认 Range。
    Bytes(Vec<u8>),
    /// 报整个的长度，只写前 `n` 个字节就断开；不认 Range。
    Cut(Vec<u8>, usize),
    /// 写前 `n` 个字节以后不动了。
    Stall(Vec<u8>, usize),
    Status(u16),
    Redirect(String),
}

/// 收到的每个请求：路径和 Range 头。
type Seen = Arc<Mutex<Vec<(String, Option<String>)>>>;

struct Server {
    base: String,
    /// 收到的每个请求：路径和 Range 头。
    seen: Seen,
}

fn serve(routes: Vec<(&str, Reply)>) -> Server {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let base = format!("http://{}", listener.local_addr().unwrap());
    let routes: HashMap<String, Reply> =
        routes.into_iter().map(|(p, r)| (p.to_owned(), r)).collect();
    let seen = Arc::new(Mutex::new(Vec::new()));
    let log = Arc::clone(&seen);
    std::thread::spawn(move || {
        for stream in listener.incoming() {
            let Ok(stream) = stream else { return };
            let routes = routes.clone();
            let log = Arc::clone(&log);
            std::thread::spawn(move || answer(stream, &routes, &log));
        }
    });
    Server { base, seen }
}

fn answer(mut stream: TcpStream, routes: &HashMap<String, Reply>, log: &Seen) {
    let mut reader = BufReader::new(stream.try_clone().unwrap());
    let mut line = String::new();
    reader.read_line(&mut line).unwrap();
    let path = line.split(' ').nth(1).unwrap_or_default().to_owned();
    let mut range = None;
    loop {
        let mut h = String::new();
        reader.read_line(&mut h).unwrap();
        if h.trim().is_empty() {
            break;
        }
        if let Some((k, v)) = h.split_once(':')
            && k.eq_ignore_ascii_case("range")
        {
            range = Some(v.trim().to_owned());
        }
    }
    log.lock().unwrap().push((path.clone(), range.clone()));
    let head = |status: &str, extra: &str, len: usize| {
        format!("HTTP/1.1 {status}\r\nContent-Length: {len}\r\nConnection: close\r\n{extra}\r\n")
    };
    match routes.get(&path) {
        Some(Reply::Bytes(b)) => {
            let from = range
                .as_deref()
                .and_then(|r| r.strip_prefix("bytes="))
                .and_then(|r| r.strip_suffix('-'))
                .and_then(|n| n.parse::<usize>().ok());
            match from {
                Some(from) => {
                    let extra = format!(
                        "Content-Range: bytes {from}-{}/{}\r\n",
                        b.len() - 1,
                        b.len()
                    );
                    let _ = stream
                        .write_all(head("206 Partial Content", &extra, b.len() - from).as_bytes());
                    let _ = stream.write_all(&b[from..]);
                }
                None => {
                    let _ = stream.write_all(head("200 OK", "", b.len()).as_bytes());
                    let _ = stream.write_all(b);
                }
            }
        }
        Some(Reply::Cut(b, n)) => {
            let _ = stream.write_all(head("200 OK", "", b.len()).as_bytes());
            let _ = stream.write_all(&b[..*n]);
        }
        Some(Reply::Stall(b, n)) => {
            let _ = stream.write_all(head("200 OK", "", b.len()).as_bytes());
            let _ = stream.write_all(&b[..*n]);
            let _ = stream.flush();
            std::thread::sleep(Duration::from_secs(5));
        }
        Some(Reply::Status(code)) => {
            let _ = stream.write_all(head(&format!("{code} X"), "", 0).as_bytes());
        }
        Some(Reply::Redirect(to)) => {
            let _ =
                stream.write_all(head("302 Found", &format!("Location: {to}\r\n"), 0).as_bytes());
        }
        None => {
            let _ = stream.write_all(head("404 Not Found", "", 0).as_bytes());
        }
    }
}

fn net() -> Net {
    let mut n = Net::system(ProxySetting::Direct).unwrap();
    n.policy = Some((
        Policy {
            public: |_| true,
            ..Policy::strict()
        },
        None,
    ));
    n.https_only = false;
    n.connect = Duration::from_secs(5);
    n.stall = Duration::from_millis(400);
    n
}

fn plugin() -> PluginId {
    PluginId::try_from("builtin/asr-sherpa".to_owned()).unwrap()
}

fn spec(
    server: &Server,
    path: &str,
    mirrors: &[&str],
    bytes: &[u8],
    format: ModelFormat,
) -> ModelSpec {
    ModelSpec {
        name: "fake-model".into(),
        url: format!("{}{path}", server.base),
        mirrors: mirrors
            .iter()
            .map(|m| format!("{}{m}", server.base))
            .collect(),
        sha256: usuyo_library::sha256_hex(bytes),
        size: bytes.len() as u64,
        format,
        license: "MIT".into(),
        setting: None,
        value: None,
    }
}

/// tar.bz2：一个顶层文件夹，里面两个文件。
fn tar_bz2(entries: &[(&str, &[u8])]) -> Vec<u8> {
    let mut b = tar::Builder::new(bzip2::write::BzEncoder::new(
        Vec::new(),
        bzip2::Compression::fast(),
    ));
    for (path, data) in entries {
        let mut h = tar::Header::new_gnu();
        h.set_size(data.len() as u64);
        h.set_mode(0o644);
        h.set_entry_type(tar::EntryType::Regular);
        // 原样写进名字一栏，`../`、`/` 开头的也照写（`set_path` 会拒）。
        let name = &mut h.as_gnu_mut().unwrap().name;
        name[..path.len()].copy_from_slice(path.as_bytes());
        h.set_cksum();
        b.append(&h, *data).unwrap();
    }
    b.into_inner().unwrap().finish().unwrap()
}

struct Env {
    _tmp: tempfile::TempDir,
    root: PathBuf,
    models: Models,
}

fn env() -> Env {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path().join("models");
    Env {
        models: Models::new(root.clone(), net()),
        root,
        _tmp: tmp,
    }
}

impl Env {
    async fn fetch(&self, spec: &ModelSpec) -> Result<(), ModelError> {
        assert_eq!(self.models.begin(&plugin(), spec), Begin::Started);
        let r = self.models.fetch(&plugin(), spec).await;
        self.models.end(&plugin(), spec, &r);
        r
    }

    fn dir(&self) -> PathBuf {
        self.root.join("builtin/asr-sherpa")
    }

    /// 插件目录下的全部名字（含隐藏的），排好。
    fn names(&self) -> Vec<String> {
        let mut v: Vec<String> = match std::fs::read_dir(self.dir()) {
            Ok(d) => d
                .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
                .collect(),
            Err(_) => Vec::new(),
        };
        v.sort();
        v
    }
}

#[tokio::test]
async fn downloads_unpacks_keeps_the_top_folder_and_writes_model_json() {
    let archive = tar_bz2(&[
        ("sense/model.int8.onnx", b"weights"),
        ("sense/tokens.txt", b"a b c"),
    ]);
    let s = serve(vec![("/m.tar.bz2", Reply::Bytes(archive.clone()))]);
    let e = env();
    let spec = spec(&s, "/m.tar.bz2", &[], &archive, ModelFormat::TarBz2);
    assert_eq!(e.models.state(&plugin(), &spec), ModelState::Absent);
    e.fetch(&spec).await.unwrap();
    assert_eq!(e.names(), vec!["fake-model"], "暂存和下了一半的都不剩");
    let m = e.dir().join("fake-model");
    assert_eq!(std::fs::read(m.join("sense/tokens.txt")).unwrap(), b"a b c");
    let rec = e.models.record(&plugin(), &spec).unwrap();
    assert_eq!(rec.sha256, spec.sha256);
    assert_eq!(rec.size, archive.len() as u64);
    assert_eq!(
        rec.files,
        vec![
            ModelFile {
                path: "sense/model.int8.onnx".into(),
                size: 7,
                sha256: usuyo_library::sha256_hex(b"weights"),
            },
            ModelFile {
                path: "sense/tokens.txt".into(),
                size: 5,
                sha256: usuyo_library::sha256_hex(b"a b c"),
            },
        ]
    );
    assert_eq!(e.models.state(&plugin(), &spec), ModelState::Present);
    assert_eq!(
        e.models.begin(&plugin(), &spec),
        Begin::Present,
        "在了不再下"
    );
    // 清单换了一个模型（sha256 不同）：原来那份不算在
    let mut other = spec.clone();
    other.sha256 = "0".repeat(64);
    assert_eq!(e.models.state(&plugin(), &other), ModelState::Absent);

    assert!(e.models.remove(&plugin(), &spec).unwrap());
    assert_eq!(e.names(), Vec::<String>::new());
    assert_eq!(e.models.state(&plugin(), &spec), ModelState::Absent);
    assert!(!e.models.remove(&plugin(), &spec).unwrap());
}

#[tokio::test]
async fn a_cut_download_resumes_from_the_next_mirror_with_range() {
    let bytes: Vec<u8> = (0..50_000u32).map(|i| (i % 251) as u8).collect();
    let s = serve(vec![
        ("/a", Reply::Cut(bytes.clone(), 20_000)),
        ("/b", Reply::Bytes(bytes.clone())),
    ]);
    let e = env();
    let spec = spec(&s, "/a", &["/b"], &bytes, ModelFormat::File);
    e.fetch(&spec).await.unwrap();
    let seen = s.seen.lock().unwrap().clone();
    assert_eq!(seen[0], ("/a".into(), None));
    assert_eq!(
        seen[1],
        ("/b".into(), Some("bytes=20000-".into())),
        "接着断处下"
    );
    assert_eq!(std::fs::read(e.dir().join("fake-model/b")).unwrap(), bytes);
}

#[tokio::test]
async fn a_cut_download_keeps_its_part_for_next_time() {
    let bytes = vec![7u8; 10_000];
    let s = serve(vec![("/a", Reply::Cut(bytes.clone(), 4_000))]);
    let e = env();
    let spec = spec(&s, "/a", &[], &bytes, ModelFormat::File);
    let err = e.fetch(&spec).await.unwrap_err();
    // 连接断在中途：读体出错（或干脆读完了也不够长），留下的那一段等下次接着
    assert!(
        matches!(err, ModelError::Network(_) | ModelError::Short { .. }),
        "{err}"
    );
    assert_eq!(e.names(), vec![".fake-model.part"]);
    assert_eq!(
        std::fs::metadata(e.dir().join(".fake-model.part"))
            .unwrap()
            .len(),
        4_000
    );
    assert!(matches!(
        e.models.state(&plugin(), &spec),
        ModelState::Failed(_)
    ));
}

#[tokio::test]
async fn mirrors_are_tried_in_order() {
    let bytes = b"model bytes".to_vec();
    let s = serve(vec![
        ("/gone", Reply::Status(404)),
        ("/err", Reply::Status(500)),
        ("/ok", Reply::Bytes(bytes.clone())),
    ]);
    let e = env();
    let spec = spec(&s, "/gone", &["/err", "/ok"], &bytes, ModelFormat::File);
    e.fetch(&spec).await.unwrap();
    let paths: Vec<String> = s.seen.lock().unwrap().iter().map(|r| r.0.clone()).collect();
    assert_eq!(paths, vec!["/gone", "/err", "/ok"]);
}

#[tokio::test]
async fn a_wrong_sha256_is_thrown_away_whole() {
    let bytes = b"model bytes".to_vec();
    let s = serve(vec![("/m", Reply::Bytes(b"other bytes".to_vec()))]);
    let e = env();
    let spec = spec(&s, "/m", &[], &bytes, ModelFormat::File);
    let err = e.fetch(&spec).await.unwrap_err();
    assert!(matches!(err, ModelError::Sha256 { .. }), "{err}");
    assert_eq!(e.names(), Vec::<String>::new(), "下的整个丢掉");
}

#[tokio::test]
async fn more_than_the_declared_size_is_refused() {
    let s = serve(vec![("/m", Reply::Bytes(vec![1u8; 2_000]))]);
    let e = env();
    let mut spec = spec(&s, "/m", &[], &[1u8; 1_000], ModelFormat::File);
    spec.size = 1_000;
    let err = e.fetch(&spec).await.unwrap_err();
    assert!(matches!(err, ModelError::TooLarge(1_000)), "{err}");
    assert_eq!(e.names(), Vec::<String>::new());
}

#[tokio::test]
async fn a_stall_ends_the_attempt() {
    let bytes = vec![3u8; 5_000];
    let s = serve(vec![("/m", Reply::Stall(bytes.clone(), 1_000))]);
    let e = env();
    let spec = spec(&s, "/m", &[], &bytes, ModelFormat::File);
    let err = e.fetch(&spec).await.unwrap_err();
    assert!(matches!(err, ModelError::Stall(_)), "{err}");
}

#[tokio::test]
async fn redirects_are_followed_hop_by_hop_and_https_is_required() {
    let bytes = b"model bytes".to_vec();
    let s = serve(vec![
        ("/r", Reply::Redirect("/m".into())),
        ("/m", Reply::Bytes(bytes.clone())),
    ]);
    let e = env();
    let spec = spec(&s, "/r", &[], &bytes, ModelFormat::File);
    e.fetch(&spec).await.unwrap();
    // 单个文件按清单里那个地址的文件名放，不按跳转到的
    assert_eq!(std::fs::read(e.dir().join("fake-model/r")).unwrap(), bytes);
    assert_eq!(s.seen.lock().unwrap().len(), 2);

    // 出厂的设置：http 的不连，本机地址不连
    let real = Models::new(
        e.root.join("other"),
        Net::system(ProxySetting::Direct).unwrap(),
    );
    let err = real.fetch(&plugin(), &spec).await.unwrap_err();
    assert!(matches!(err, ModelError::NotHttps(_)), "{err}");
    let mut local = spec.clone();
    local.url = local.url.replace("http://", "https://");
    let err = real.fetch(&plugin(), &local).await.unwrap_err();
    assert!(matches!(err, ModelError::Refused(_)), "{err}");
}

#[tokio::test]
async fn unsafe_archives_are_refused_and_nothing_lands() {
    let cases: Vec<Vec<u8>> = vec![
        tar_bz2(&[("../evil", b"x")]),
        tar_bz2(&[("/abs/evil", b"x")]),
        tar_bz2(&[("model.json", b"{}")]),
        {
            let mut b = tar::Builder::new(bzip2::write::BzEncoder::new(
                Vec::new(),
                bzip2::Compression::fast(),
            ));
            let mut h = tar::Header::new_gnu();
            h.set_entry_type(tar::EntryType::Symlink);
            h.set_size(0);
            h.set_path("top/link").unwrap();
            h.set_link_name("/etc/passwd").unwrap();
            h.set_cksum();
            b.append(&h, std::io::empty()).unwrap();
            b.into_inner().unwrap().finish().unwrap()
        },
    ];
    for archive in cases {
        let s = serve(vec![("/m", Reply::Bytes(archive.clone()))]);
        let e = env();
        let spec = spec(&s, "/m", &[], &archive, ModelFormat::TarBz2);
        let err = e.fetch(&spec).await.unwrap_err();
        assert!(matches!(err, ModelError::Archive(_)), "{err}");
        assert_eq!(e.names(), Vec::<String>::new(), "{err}");
        assert!(!e.root.parent().unwrap().join("evil").exists());
    }
}

#[test]
fn unpacking_stops_at_the_size_cap() {
    let tmp = tempfile::tempdir().unwrap();
    let archive = tmp.path().join("a.tar.gz");
    let mut b = tar::Builder::new(flate2::write::GzEncoder::new(
        Vec::new(),
        flate2::Compression::fast(),
    ));
    let big = vec![0u8; 3 << 20];
    let mut h = tar::Header::new_gnu();
    h.set_size(big.len() as u64);
    h.set_path("top/zeros").unwrap();
    h.set_cksum();
    b.append(&h, big.as_slice()).unwrap();
    std::fs::write(&archive, b.into_inner().unwrap().finish().unwrap()).unwrap();
    let mut out = Unpacked {
        root: tmp.path().join("out"),
        total: 0,
        cap: 1 << 20,
        files: Vec::new(),
        entries: 0,
    };
    std::fs::create_dir(&out.root).unwrap();
    let r = untar(
        &mut out,
        flate2::read::GzDecoder::new(std::fs::File::open(&archive).unwrap()),
    );
    assert!(matches!(r, Err(ModelError::Archive(_))), "{r:?}");
}

#[test]
fn zip_entries_are_checked_too() {
    let tmp = tempfile::tempdir().unwrap();
    let write = |name: &str, f: &dyn Fn(&mut zip::ZipWriter<std::fs::File>)| {
        let path = tmp.path().join(name);
        let mut z = zip::ZipWriter::new(std::fs::File::create(&path).unwrap());
        f(&mut z);
        z.finish().unwrap();
        path
    };
    let opts =
        zip::write::SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored);
    let good = write("good.zip", &|z| {
        z.add_directory("top/", opts).unwrap();
        z.start_file("top/a.bin", opts).unwrap();
        z.write_all(b"abc").unwrap();
    });
    let link = write("link.zip", &|z| {
        z.add_symlink("top/l", "/etc/passwd", opts).unwrap();
    });
    let up = write("up.zip", &|z| {
        z.start_file("../x", opts).unwrap();
        z.write_all(b"abc").unwrap();
    });
    let run = |path: &Path| {
        let root = tmp.path().join(format!("out-{}", usuyo_library::uuid7()));
        std::fs::create_dir(&root).unwrap();
        let mut out = Unpacked {
            root,
            total: 0,
            cap: 1 << 20,
            files: Vec::new(),
            entries: 0,
        };
        unzip(&mut out, std::fs::File::open(path).unwrap()).map(|()| out.files)
    };
    let files = run(&good).unwrap();
    assert_eq!(files.len(), 1);
    assert_eq!(files[0].path, "top/a.bin");
    assert!(matches!(run(&link), Err(ModelError::Archive(_))));
    assert!(matches!(run(&up), Err(ModelError::Archive(_))));
}

#[test]
fn content_range_must_continue_where_the_part_ends() {
    assert!(range_continues("bytes 100-199/200", 100, 200));
    assert!(range_continues("bytes 100-199/*", 100, 200));
    assert!(!range_continues("bytes 0-199/200", 100, 200));
    assert!(!range_continues("bytes 100-198/200", 100, 200));
    assert!(!range_continues("bytes 100-199/300", 100, 200));
    assert!(!range_continues("", 100, 200));
}

/// 真的 SenseVoice 压缩包（`USUYO_TEST_SENSEVOICE_ARCHIVE` 指过去，不设就跳过）照清单解开，
/// 布局和 asr-sherpa 要的一样。
#[test]
fn the_real_sense_voice_archive_unpacks_where_asr_sherpa_looks() {
    let Some(archive) = std::env::var_os("USUYO_TEST_SENSEVOICE_ARCHIVE") else {
        eprintln!("skipped: USUYO_TEST_SENSEVOICE_ARCHIVE is not set");
        return;
    };
    let out = std::env::var_os("USUYO_TEST_SENSEVOICE_OUT").map(PathBuf::from);
    let tmp = tempfile::tempdir().unwrap();
    let dir = out.unwrap_or_else(|| tmp.path().to_owned());
    std::fs::create_dir_all(&dir).unwrap();
    let spec = ModelSpec {
        name: "sense-voice-int8-2024-07-17".into(),
        url: "https://example.com/m.tar.bz2".into(),
        mirrors: Vec::new(),
        sha256: "7d1efa2138a65b0b488df37f8b89e3d91a60676e416f515b952358d83dfd347e".into(),
        size: 163_002_883,
        format: ModelFormat::TarBz2,
        license: "FunASR Model Open Source License（可商用，署名并写明模型名）".into(),
        setting: None,
        value: None,
    };
    assert_eq!(hash_file(Path::new(&archive)).unwrap(), spec.sha256);
    unpack(Path::new(&archive), &dir, &spec, "m.tar.bz2").unwrap();
    let top = dir
        .join(&spec.name)
        .join("sherpa-onnx-sense-voice-zh-en-ja-ko-yue-int8-2024-07-17");
    assert!(top.join("model.int8.onnx").is_file());
    assert!(top.join("tokens.txt").is_file());
    assert!(dir.join(&spec.name).join("model.json").is_file());
}

/// 本机的 HTTP 代理：`CONNECT` 照接通；普通的 `GET http://…` 改成只带路径转给 `target`。记下被要求去的地方。
fn http_proxy(target: u16) -> (u16, Arc<Mutex<Vec<String>>>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    let asked: Arc<Mutex<Vec<String>>> = Arc::default();
    let log = Arc::clone(&asked);
    std::thread::spawn(move || {
        for stream in listener.incoming() {
            let Ok(client) = stream else { return };
            let log = Arc::clone(&log);
            std::thread::spawn(move || {
                let mut reader = BufReader::new(client.try_clone().unwrap());
                let mut line = String::new();
                reader.read_line(&mut line).unwrap();
                let mut head = Vec::new();
                loop {
                    let mut h = String::new();
                    reader.read_line(&mut h).unwrap();
                    if h.trim().is_empty() {
                        break;
                    }
                    head.push(h);
                }
                let mut parts = line.split_whitespace();
                let (method, to) = (
                    parts.next().unwrap_or_default(),
                    parts.next().unwrap_or_default(),
                );
                log.lock().unwrap().push(format!("{method} {to}"));
                let mut upstream = TcpStream::connect(("127.0.0.1", target)).unwrap();
                let mut client = client;
                if method == "CONNECT" {
                    client.write_all(b"HTTP/1.1 200 OK\r\n\r\n").unwrap();
                } else {
                    let path = Url::parse(to).unwrap().path().to_owned();
                    let mut req = format!("{method} {path} HTTP/1.1\r\n");
                    for h in &head {
                        req.push_str(h);
                    }
                    req.push_str("\r\n");
                    upstream.write_all(req.as_bytes()).unwrap();
                }
                let (mut a, mut b) = (reader.into_inner(), upstream.try_clone().unwrap());
                std::thread::spawn(move || {
                    let _ = std::io::copy(&mut a, &mut b);
                });
                let _ = std::io::copy(&mut upstream, &mut client);
            });
        }
    });
    (port, asked)
}

#[tokio::test]
async fn downloads_go_through_the_proxy_the_plugins_use() {
    let bytes = b"model bytes".to_vec();
    let s = serve(vec![("/m", Reply::Bytes(bytes.clone()))]);
    let port = s.base.rsplit(':').next().unwrap().parse().unwrap();
    let (pport, asked) = http_proxy(port);
    let tmp = tempfile::tempdir().unwrap();
    let mut n = net();
    let proxy = format!("http://127.0.0.1:{pport}");
    // 明文 http 经代理时本机要查得到公网地址（出厂只下 https，这里为了用本机测试服务）
    fn lookup(_: &str, port: u16) -> std::io::Result<Vec<std::net::SocketAddr>> {
        Ok(vec![std::net::SocketAddr::from(([93, 184, 216, 34], port))])
    }
    n.policy = Some((
        Policy {
            lookup,
            proxy: Some(Proxy::intern(&proxy).unwrap()),
            ..Policy::strict()
        },
        Some(proxy),
    ));
    let models = Models::new(tmp.path().join("models"), n);
    let mut spec = spec(&s, "/m", &[], &bytes, ModelFormat::File);
    spec.url = format!("http://models.test:{port}/m");
    models.fetch(&plugin(), &spec).await.unwrap();
    assert_eq!(
        *asked.lock().unwrap(),
        vec![format!("GET http://models.test:{port}/m")],
        "the proxy is asked for the name"
    );
    // 经代理时也不去内网：数字的内网地址在问代理之前就拒了
    let mut inner = spec.clone();
    inner.url = format!("http://10.0.0.1:{port}/m");
    let err = models.fetch(&plugin(), &inner).await.unwrap_err();
    assert!(matches!(err, ModelError::Refused(_)), "{err}");
    assert_eq!(asked.lock().unwrap().len(), 1);
}
