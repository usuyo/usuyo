//! 转换插件声明的模型（feature 003 §4.3、SPEC §1.1）：宿主下载、校验、解开，插件不联网。
//!
//! 放在按机器目录的 `models/<owner>/<repo>/<名字>/`：压缩包原样解开（留着它自己的顶层文件夹），
//! 单个文件按链接里的文件名放；旁边写 `model.json`。`model.json` 在、名字和 sha256 都和清单一样，
//! 才算这个模型在。只在你点了下载时下，从不自己下。
//!
//! 下载：先试 `url`，不行按顺序试 `mirrors`；只走 https，每一跳先照插件发请求那套规则查地址
//! （只连公网，[`Policy::vet`]），只连查过的地址，跳转一跳一跳地跟；连接 15 秒，60 秒没收到字节算断，
//! 写下来的不超过清单的 `size`，断了下次从断处接着（Range）。下完按整个文件的 sha256 核对，
//! 对不上整个丢掉。压缩包、服务器的回答都不可信：在这里查一次。

use std::collections::HashMap;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, PoisonError};
use std::time::Duration;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tokio::io::AsyncWriteExt;
use url::Url;
use usuyo_plugin::ProxySetting;
use usuyo_plugin::protocol::{ModelFormat, ModelSpec, PluginId};
use usuyo_plugin_kit::http::{Policy, Proxy};

const CONNECT: Duration = Duration::from_secs(15);
const STALL: Duration = Duration::from_secs(60);
const MAX_REDIRECTS: u32 = 5;
/// 压缩包里最多几项。
const MAX_ENTRIES: usize = 100_000;
/// 解开以后最多多少字节：下载大小的 8 倍，至少 1 GiB。
fn unpacked_cap(size: u64) -> u64 {
    size.saturating_mul(8).max(1 << 30)
}

/// 一个模型现在怎样（`GET /transforms` 的 `state`）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ModelState {
    Absent,
    Downloading,
    Present,
    /// 上次下载没成：给人看的一句（两种语言；英文的原因只进日志）。
    Failed(crate::words::Words),
}

/// `model.json`：`{name, sha256, size, license, files: [{path, size, sha256}]}`，`path` 相对模型文件夹。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct ModelRecord {
    pub name: String,
    pub sha256: String,
    pub size: u64,
    pub license: String,
    pub files: Vec<ModelFile>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct ModelFile {
    pub path: String,
    pub size: u64,
    pub sha256: String,
}

#[derive(Debug, thiserror::Error)]
pub enum ModelError {
    #[error("bad model address `{0}`")]
    Url(String),
    #[error("`{0}` is not https")]
    NotHttps(String),
    #[error("refused before connecting: {0}")]
    Refused(String),
    #[error("http status {0}")]
    Status(u16),
    #[error("network: {0}")]
    Network(String),
    #[error("no bytes for {0:?}; the download stalled")]
    Stall(Duration),
    #[error("the server sent more than the declared {0} bytes")]
    TooLarge(u64),
    #[error("the download ended at {got} of {size} bytes")]
    Short { got: u64, size: u64 },
    #[error("more than {MAX_REDIRECTS} redirects")]
    TooManyRedirects,
    #[error("sha256 is {got}, the manifest says {expected}")]
    Sha256 { expected: String, got: String },
    #[error("archive: {0}")]
    Archive(String),
    #[error("io error at {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("the model is downloading")]
    Busy,
    #[error("background task failed: {0}")]
    Task(String),
}

impl ModelError {
    /// 给人看的一句，写在模型那一栏「没下成」后面。
    fn hint(&self) -> crate::words::Words {
        use crate::words as w;
        match self {
            ModelError::Network(_) | ModelError::Stall(_) | ModelError::Status(_) => {
                w::MODEL_NETWORK
            }
            ModelError::Refused(_) => w::MODEL_REFUSED,
            ModelError::Sha256 { .. } | ModelError::TooLarge(_) | ModelError::Short { .. } => {
                w::MODEL_MISMATCH
            }
            ModelError::Io { .. } => w::MODEL_IO,
            _ => w::MODEL_FAILED,
        }
    }
}

fn io(path: &Path) -> impl FnOnce(std::io::Error) -> ModelError + '_ {
    move |source| ModelError::Io {
        path: path.to_owned(),
        source,
    }
}

fn archive(msg: impl Into<String>) -> ModelError {
    ModelError::Archive(msg.into())
}

/// 下载怎么连。出厂只有一种：[`Net::system`]；测试换成连本机 http 测试服务器的。
#[derive(Clone)]
pub(crate) struct Net {
    /// 和获取插件同一条路：按机器的 `network.proxy`；设置页改了马上换。
    route: Arc<std::sync::RwLock<ProxySetting>>,
    /// 测试固定的地址策略和代理地址；为空时每次下载照这台机器现在的网络环境取严格策略
    /// （假 IP 代理照 `behind_fake_ip_proxy`，代理照 `route`）。
    policy: Option<(Policy, Option<String>)>,
    https_only: bool,
    connect: Duration,
    stall: Duration,
    tls: Arc<rustls::ClientConfig>,
}

impl Net {
    pub fn system(route: ProxySetting) -> Result<Net, ModelError> {
        let provider = Arc::new(rustls::crypto::ring::default_provider());
        let roots = rustls::RootCertStore {
            roots: webpki_roots::TLS_SERVER_ROOTS.to_vec(),
        };
        let tls = rustls::ClientConfig::builder_with_provider(provider)
            .with_safe_default_protocol_versions()
            .map_err(|e| ModelError::Network(format!("tls: {e}")))?
            .with_root_certificates(roots)
            .with_no_client_auth();
        Ok(Net {
            route: Arc::new(std::sync::RwLock::new(route)),
            policy: None,
            https_only: true,
            connect: CONNECT,
            stall: STALL,
            tls: Arc::new(tls),
        })
    }

    /// 只连本机回环地址、可以走明文 http 的下载（测试对着本机起的假 release 服务器用）；
    /// 断流 `stall` 就算断。
    pub(crate) fn loopback(stall: Duration) -> Result<Net, ModelError> {
        let mut n = Net::system(ProxySetting::Direct)?;
        n.policy = Some((
            Policy {
                public: |ip| ip.is_loopback(),
                ..Policy::strict()
            },
            None,
        ));
        n.https_only = false;
        n.stall = stall;
        Ok(n)
    }

    /// 这次下载的地址策略和代理（`None` 是直连）。
    async fn policy(&self) -> Result<(Policy, Option<String>), ModelError> {
        if let Some(fixed) = &self.policy {
            return Ok(fixed.clone());
        }
        let route = self
            .route
            .read()
            .unwrap_or_else(PoisonError::into_inner)
            .clone();
        let (fake, proxy) = tokio::task::spawn_blocking(move || {
            (
                usuyo_plugin::behind_fake_ip_proxy(),
                usuyo_plugin::resolve_proxy(&route),
            )
        })
        .await
        .map_err(|e| ModelError::Task(e.to_string()))?;
        let interned = proxy
            .as_deref()
            .map(Proxy::intern)
            .transpose()
            .map_err(|e| ModelError::Refused(e.to_string()))?;
        Ok((
            Policy {
                fake_ip_names: fake,
                proxy: interned,
                ..Policy::strict()
            },
            proxy,
        ))
    }

    /// 发一个 GET，一跳一跳地跟跳转（每一跳先查地址，只连查过的地址），回第一个不是跳转的回答。
    async fn send(
        &self,
        url: &Url,
        headers: &[(reqwest::header::HeaderName, String)],
    ) -> Result<reqwest::Response, ModelError> {
        let (policy, proxy) = self.policy().await?;
        let mut url = url.clone();
        for _ in 0..=MAX_REDIRECTS {
            if self.https_only && url.scheme() != "https" {
                return Err(ModelError::NotHttps(redact(&url)));
            }
            let vet_url = url.clone();
            let connect = self.connect;
            let vet_policy = policy;
            let addrs = tokio::task::spawn_blocking(move || vet_policy.vet(&vet_url, connect))
                .await
                .map_err(|e| ModelError::Task(e.to_string()))?
                .map_err(|e| ModelError::Refused(e.to_string()))?;
            let mut builder = reqwest::Client::builder()
                .no_proxy()
                .redirect(reqwest::redirect::Policy::none())
                .connect_timeout(self.connect)
                .user_agent(concat!("usuyo/", env!("CARGO_PKG_VERSION")))
                .tls_backend_preconfigured((*self.tls).clone());
            match &proxy {
                // 经代理：只连代理，目标的名字交给它查（socks5 照 socks5h 用）
                Some(p) => {
                    let p = match p.strip_prefix("socks5://") {
                        Some(rest) => format!("socks5h://{rest}"),
                        None => p.clone(),
                    };
                    let p = reqwest::Proxy::all(&p).map_err(|_| {
                        ModelError::Refused("the proxy address is not usable".into())
                    })?;
                    builder = builder.proxy(p);
                }
                None => {
                    if let Some(url::Host::Domain(name)) = url.host() {
                        builder = builder.resolve_to_addrs(name, &addrs);
                    }
                }
            }
            let client = builder
                .build()
                .map_err(|e| ModelError::Network(e.to_string()))?;
            let mut request = client
                .get(url.as_str())
                .header(reqwest::header::ACCEPT_ENCODING, "identity");
            for (name, value) in headers {
                request = request.header(name, value);
            }
            let response = tokio::time::timeout(self.stall, request.send())
                .await
                .map_err(|_| ModelError::Stall(self.stall))?
                .map_err(|e| ModelError::Network(e.without_url().to_string()))?;
            if matches!(response.status().as_u16(), 301 | 302 | 303 | 307 | 308) {
                url = response
                    .headers()
                    .get(reqwest::header::LOCATION)
                    .and_then(|l| l.to_str().ok())
                    .and_then(|l| url.join(l).ok())
                    .ok_or_else(|| ModelError::Network("redirect without a location".into()))?;
                continue;
            }
            return Ok(response);
        }
        Err(ModelError::TooManyRedirects)
    }

    /// 取一个小文件（来源的索引）放进内存：最多 `limit` 字节；带上次的 `etag` 时服务器可以回 304。
    pub(crate) async fn get_small(
        &self,
        url: &Url,
        limit: u64,
        etag: Option<&str>,
    ) -> Result<Small, ModelError> {
        let headers: Vec<_> = etag
            .map(|e| (reqwest::header::IF_NONE_MATCH, e.to_owned()))
            .into_iter()
            .collect();
        let mut response = self.send(url, &headers).await?;
        match response.status().as_u16() {
            304 => return Ok(Small::NotModified),
            200 => {}
            other => return Err(ModelError::Status(other)),
        }
        if response.content_length().is_some_and(|len| len > limit) {
            return Err(ModelError::TooLarge(limit));
        }
        let etag = response
            .headers()
            .get(reqwest::header::ETAG)
            .and_then(|v| v.to_str().ok())
            .map(str::to_owned);
        let mut body = Vec::new();
        loop {
            let chunk = tokio::time::timeout(self.stall, response.chunk())
                .await
                .map_err(|_| ModelError::Stall(self.stall))?
                .map_err(|e| ModelError::Network(e.without_url().to_string()))?;
            let Some(bytes) = chunk else { break };
            if body.len() as u64 + bytes.len() as u64 > limit {
                return Err(ModelError::TooLarge(limit));
            }
            body.extend_from_slice(&bytes);
        }
        Ok(Small::Body { bytes: body, etag })
    }

    /// 把 `url` 下到 `part`：`part` 里已有的当成前面那一段，用 Range 接着下；服务器不认 Range 就从头来。
    /// 中途断了 `part` 留着，下次接着；多出清单大小的整个删掉。每收到一段，`progress` 拿到已有的字节数。
    pub(crate) async fn download(
        &self,
        url: &Url,
        part: &Path,
        size: u64,
        progress: &(dyn Fn(u64) + Send + Sync),
    ) -> Result<(), ModelError> {
        let mut have = match tokio::fs::metadata(part).await {
            Ok(m) => m.len(),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => 0,
            Err(e) => return Err(io(part)(e)),
        };
        if have > size {
            tokio::fs::remove_file(part).await.map_err(io(part))?;
            have = 0;
        }
        if have == size {
            return Ok(());
        }
        let headers: Vec<_> = (have > 0)
            .then(|| (reqwest::header::RANGE, format!("bytes={have}-")))
            .into_iter()
            .collect();
        let mut response = self.send(url, &headers).await?;
        let append = match response.status().as_u16() {
            206 if have > 0 => {
                let range = response
                    .headers()
                    .get(reqwest::header::CONTENT_RANGE)
                    .and_then(|v| v.to_str().ok())
                    .unwrap_or_default();
                if !range_continues(range, have, size) {
                    return Err(ModelError::Network(format!(
                        "unexpected content range `{range}`"
                    )));
                }
                true
            }
            200 => {
                have = 0;
                false
            }
            other => return Err(ModelError::Status(other)),
        };
        if response
            .content_length()
            .is_some_and(|len| have.saturating_add(len) > size)
        {
            return Err(ModelError::TooLarge(size));
        }
        let mut file = tokio::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .append(append)
            .truncate(!append)
            .open(part)
            .await
            .map_err(io(part))?;
        progress(have);
        loop {
            let chunk = tokio::time::timeout(self.stall, response.chunk())
                .await
                .map_err(|_| ModelError::Stall(self.stall))?;
            let bytes = match chunk {
                Ok(Some(b)) => b,
                Ok(None) => break,
                Err(e) => {
                    file.flush().await.map_err(io(part))?;
                    return Err(ModelError::Network(e.without_url().to_string()));
                }
            };
            let len = bytes.len() as u64;
            if have.saturating_add(len) > size {
                drop(file);
                tokio::fs::remove_file(part).await.map_err(io(part))?;
                return Err(ModelError::TooLarge(size));
            }
            file.write_all(&bytes).await.map_err(io(part))?;
            have += len;
            progress(have);
        }
        file.flush().await.map_err(io(part))?;
        file.sync_all().await.map_err(io(part))?;
        if have == size {
            Ok(())
        } else {
            Err(ModelError::Short { got: have, size })
        }
    }
}

/// [`Net::get_small`] 的回答。
pub(crate) enum Small {
    NotModified,
    Body {
        bytes: Vec<u8>,
        etag: Option<String>,
    },
}

/// 日志里只写主机和路径，不写查询串。
pub(crate) fn redact(url: &Url) -> String {
    format!(
        "{}://{}{}",
        url.scheme(),
        url.host_str().unwrap_or_default(),
        url.path()
    )
}

/// `Content-Range: bytes <have>-<size-1>/<size>`（总数可以是 `*`）。
fn range_continues(range: &str, have: u64, size: u64) -> bool {
    let Some(rest) = range.trim().strip_prefix("bytes ") else {
        return false;
    };
    let Some((span, total)) = rest.split_once('/') else {
        return false;
    };
    let Some((start, end)) = span.split_once('-') else {
        return false;
    };
    let total_ok = total == "*" || total.parse::<u64>() == Ok(size);
    start.parse::<u64>() == Ok(have) && end.parse::<u64>() == Ok(size - 1) && total_ok
}

/// 按机器目录里的模型，和正在下的、下失败的。
pub(crate) struct Models {
    root: PathBuf,
    net: Net,
    busy: Mutex<HashMap<(String, String), ModelState>>,
}

/// [`Models::begin`] 的结果。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Begin {
    Started,
    Present,
    Downloading,
}

impl Models {
    /// `root` 是按机器目录的 `models/`。
    pub fn new(root: PathBuf, net: Net) -> Models {
        Models {
            root,
            net,
            busy: Mutex::new(HashMap::new()),
        }
    }

    fn key(plugin: &PluginId, name: &str) -> (String, String) {
        (plugin.to_string(), name.to_owned())
    }

    fn busy(&self) -> std::sync::MutexGuard<'_, HashMap<(String, String), ModelState>> {
        self.busy.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// 之后的下载走这个代理。
    pub fn set_route(&self, route: ProxySetting) {
        *self
            .net
            .route
            .write()
            .unwrap_or_else(PoisonError::into_inner) = route;
    }

    /// 插件的 `${MODELS}`：`models/<owner>/<repo>/`。
    fn plugin_dir(&self, plugin: &PluginId) -> PathBuf {
        self.root.join(plugin.owner()).join(plugin.repo())
    }

    /// 这个模型的 `model.json`；不在、读不了、和清单对不上回 `None`。
    pub fn record(&self, plugin: &PluginId, spec: &ModelSpec) -> Option<ModelRecord> {
        let path = self.plugin_dir(plugin).join(&spec.name).join("model.json");
        let bytes = match std::fs::read(&path) {
            Ok(b) => b,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return None,
            Err(e) => {
                tracing::warn!(path = %path.display(), error = %e, "model.json not readable");
                return None;
            }
        };
        match serde_json::from_slice::<ModelRecord>(&bytes) {
            Ok(r) if r.name == spec.name && r.sha256 == spec.sha256 => Some(r),
            Ok(_) => None,
            Err(e) => {
                tracing::warn!(path = %path.display(), error = %e, "model.json is not a model record");
                None
            }
        }
    }

    pub fn state(&self, plugin: &PluginId, spec: &ModelSpec) -> ModelState {
        if let Some(s) = self.busy().get(&Self::key(plugin, &spec.name)) {
            return s.clone();
        }
        if self.record(plugin, spec).is_some() {
            ModelState::Present
        } else {
            ModelState::Absent
        }
    }

    /// 记下「在下」；已经在或在下就不再下。
    pub fn begin(&self, plugin: &PluginId, spec: &ModelSpec) -> Begin {
        let mut busy = self.busy();
        let key = Self::key(plugin, &spec.name);
        if busy.get(&key) == Some(&ModelState::Downloading) {
            return Begin::Downloading;
        }
        if self.record(plugin, spec).is_some() {
            busy.remove(&key);
            return Begin::Present;
        }
        busy.insert(key, ModelState::Downloading);
        Begin::Started
    }

    /// 下完（成没成都算）：成了去掉记号，没成记下原因。
    pub fn end(&self, plugin: &PluginId, spec: &ModelSpec, result: &Result<(), ModelError>) {
        let mut busy = self.busy();
        let key = Self::key(plugin, &spec.name);
        match result {
            Ok(()) => {
                busy.remove(&key);
            }
            Err(e) => {
                busy.insert(key, ModelState::Failed(e.hint()));
            }
        }
    }

    /// 下载、核对、解开。调用方先 [`Models::begin`]，完了 [`Models::end`]。
    pub async fn fetch(&self, plugin: &PluginId, spec: &ModelSpec) -> Result<(), ModelError> {
        let dir = self.plugin_dir(plugin);
        tokio::fs::create_dir_all(&dir).await.map_err(io(&dir))?;
        let part = dir.join(format!(".{}.part", spec.name));
        let mut last = None;
        for source in std::iter::once(&spec.url).chain(&spec.mirrors) {
            let Ok(url) = Url::parse(source) else {
                last = Some(ModelError::Url(source.clone()));
                continue;
            };
            if let Err(e) = self.net.download(&url, &part, spec.size, &|_| {}).await {
                tracing::warn!(model = %spec.name, from = %redact(&url), error = %e, "model download did not finish");
                last = Some(e);
                continue;
            }
            let p = part.clone();
            let got = tokio::task::spawn_blocking(move || hash_file(&p))
                .await
                .map_err(|e| ModelError::Task(e.to_string()))??;
            if got != spec.sha256 {
                tokio::fs::remove_file(&part).await.map_err(io(&part))?;
                let e = ModelError::Sha256 {
                    expected: spec.sha256.clone(),
                    got,
                };
                tracing::warn!(model = %spec.name, from = %redact(&url), error = %e, "model discarded");
                last = Some(e);
                continue;
            }
            let (p, d, s, file_name) = (
                part.clone(),
                dir.clone(),
                spec.clone(),
                file_name(&url, &spec.name),
            );
            let unpacked = tokio::task::spawn_blocking(move || unpack(&p, &d, &s, &file_name))
                .await
                .map_err(|e| ModelError::Task(e.to_string()))?;
            tokio::fs::remove_file(&part).await.map_err(io(&part))?;
            return unpacked;
        }
        Err(last.unwrap_or_else(|| ModelError::Url(String::new())))
    }

    /// 删掉这个模型（按机器目录里的）和下了一半的。在下时回 [`ModelError::Busy`]。回原来有没有。
    pub fn remove(&self, plugin: &PluginId, spec: &ModelSpec) -> Result<bool, ModelError> {
        let mut busy = self.busy();
        let key = Self::key(plugin, &spec.name);
        if busy.get(&key) == Some(&ModelState::Downloading) {
            return Err(ModelError::Busy);
        }
        busy.remove(&key);
        let dir = self.plugin_dir(plugin);
        let part = dir.join(format!(".{}.part", spec.name));
        match std::fs::remove_file(&part) {
            Ok(()) => {}
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(e) => return Err(io(&part)(e)),
        }
        let target = dir.join(&spec.name);
        if std::fs::symlink_metadata(&target).is_err() {
            return Ok(false);
        }
        // 先整个改名挪开再删：插件看到的要么是整个模型，要么是没有。
        let aside = dir.join(format!(
            ".{}.removing-{}",
            spec.name,
            usuyo_library::uuid7()
        ));
        std::fs::rename(&target, &aside).map_err(io(&target))?;
        std::fs::remove_dir_all(&aside).map_err(io(&aside))?;
        Ok(true)
    }
}

pub(crate) fn hash_file(path: &Path) -> Result<String, ModelError> {
    let mut f = std::fs::File::open(path).map_err(io(path))?;
    let mut h = Sha256::new();
    let mut buf = vec![0u8; 1 << 16];
    loop {
        let n = f.read(&mut buf).map_err(io(path))?;
        if n == 0 {
            break;
        }
        h.update(&buf[..n]);
    }
    Ok(hex(&h.finalize()))
}

fn hex(bytes: &[u8]) -> String {
    use std::fmt::Write as _;
    bytes.iter().fold(String::new(), |mut s, b| {
        let _ = write!(s, "{b:02x}");
        s
    })
}

/// 单个文件的模型放成什么名字：链接路径的最后一段，不合规时用模型名。
fn file_name(url: &Url, name: &str) -> String {
    url.path_segments()
        .and_then(|mut s| s.next_back())
        .filter(|s| {
            !s.is_empty()
                && *s != "."
                && *s != ".."
                && *s != "model.json"
                && !s.contains(['/', '\\', '%'])
        })
        .map_or_else(|| name.to_owned(), str::to_owned)
}

/// 解进 `<dir>/.<名字>.unpack-<UUIDv7>/`，写好 `model.json`，再整个改名成 `<dir>/<名字>/`。
/// 出错时暂存的文件夹删掉，`<名字>/` 不动。
fn unpack(archive_path: &Path, dir: &Path, spec: &ModelSpec, file: &str) -> Result<(), ModelError> {
    let stage = dir.join(format!(".{}.unpack-{}", spec.name, usuyo_library::uuid7()));
    std::fs::create_dir(&stage).map_err(io(&stage))?;
    let result = unpack_into(archive_path, &stage, spec, file).and_then(|()| {
        let target = dir.join(&spec.name);
        if std::fs::symlink_metadata(&target).is_ok() {
            // 清单换了模型（sha256 不同）时旧的那份：挪开再删。
            let aside = dir.join(format!(".{}.old-{}", spec.name, usuyo_library::uuid7()));
            std::fs::rename(&target, &aside).map_err(io(&target))?;
            std::fs::remove_dir_all(&aside).map_err(io(&aside))?;
        }
        std::fs::rename(&stage, &target).map_err(io(&target))
    });
    if result.is_err()
        && let Err(e) = std::fs::remove_dir_all(&stage)
        && e.kind() != std::io::ErrorKind::NotFound
    {
        tracing::warn!(path = %stage.display(), error = %e, "unpack folder not removed");
    }
    result
}

fn unpack_into(
    archive_path: &Path,
    stage: &Path,
    spec: &ModelSpec,
    file: &str,
) -> Result<(), ModelError> {
    use crate::unpack::{Format, Limits, Unpacked, unpack};
    let mut out = Unpacked::new(
        stage,
        Limits {
            entries: MAX_ENTRIES,
            bytes: unpacked_cap(spec.size),
            reserved: "model.json",
        },
    );
    match spec.format {
        ModelFormat::File => out.file(
            Path::new(file),
            0,
            std::fs::File::open(archive_path).map_err(io(archive_path))?,
        )?,
        ModelFormat::TarBz2 => unpack(&mut out, archive_path, Format::TarBz2)?,
        ModelFormat::TarGz => unpack(&mut out, archive_path, Format::TarGz)?,
        ModelFormat::Zip => unpack(&mut out, archive_path, Format::Zip)?,
    }
    let record = ModelRecord {
        name: spec.name.clone(),
        sha256: spec.sha256.clone(),
        size: spec.size,
        license: spec.license.clone(),
        files: out
            .into_files()
            .into_iter()
            .map(|f| ModelFile {
                path: f.path,
                size: f.size,
                sha256: f.sha256,
            })
            .collect(),
    };
    let path = stage.join("model.json");
    let mut bytes =
        serde_json::to_vec_pretty(&record).map_err(|e| archive(format!("model.json: {e}")))?;
    bytes.push(b'\n');
    let mut f = std::fs::File::create(&path).map_err(io(&path))?;
    f.write_all(&bytes).map_err(io(&path))?;
    f.sync_all().map_err(io(&path))
}

impl From<crate::unpack::UnpackError> for ModelError {
    fn from(e: crate::unpack::UnpackError) -> ModelError {
        match e {
            crate::unpack::UnpackError::Archive(m) => ModelError::Archive(m),
            crate::unpack::UnpackError::Io { path, source } => ModelError::Io { path, source },
        }
    }
}

#[cfg(test)]
mod tests;
