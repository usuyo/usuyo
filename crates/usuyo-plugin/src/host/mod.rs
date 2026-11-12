//! 插件的宿主：发现、挑选、调用、校验（协议 §2.8、§3、§4）。
//!
//! 获取插件一次一进程：宿主只负责「调一次」，起进程、管时长和环境、把插件交回的每个字节校验一遍。
//! 按用户排的顺序换插件、平台节奏、暂停和各种锁由调用方（内核的编排）做，
//! 判断需要的信息都在返回值里：错误码的 `tries_next`、`retryable`、`pause`，清单的 `pacing`。
//! 转换插件常驻，见 `transform` 模块的 [`ResidentTransform`]。

mod credentials;
mod envelope_text;
mod network;
mod outdir;
mod process;
mod transform;

use std::collections::{BTreeMap, BTreeSet};
use std::ffi::{OsStr, OsString};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use serde::de::DeserializeOwned;
use serde_json::Value;
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

pub use credentials::{DeclaredSite, DeclaredVisitor, LoginState, Vault};
pub use network::{ProxySetting, behind_fake_ip_proxy, resolve_proxy};
pub use outdir::OutFile;
use process::{Ending, Finished, Spawn};
pub use transform::{
    ResidentTransform, TransformConfig, TransformLimits, TransformOutput, TransformRequest,
};

use crate::PluginError;
use crate::protocol::{
    CallFailure, Canonicalized, CapabilityUse, Completeness, ErrorCode, FetchInputKind,
    FetchOutput, FetchSpec, Identity, ListPage, MANIFEST_FILE, NetworkMode, OutputKind,
    PLUGIN_DIR_PLACEHOLDER, PluginEvent, PluginHeader, PluginId, PluginKind, PluginManifest,
    Rfc3339, Site, SourceInfo, SourcesOutput, StorageState, TransformSpec,
};

/// 每次调用的上限。默认值照协议 §3.2、§3.4、§3.6；用户的「调用超时加长」「输出大小上限」改这里。
#[derive(Debug, Clone)]
pub struct CallLimits {
    pub canonicalize: Duration,
    /// 能力没写 `timeout` 时 fetch 的总时长。
    pub fetch: Duration,
    pub sources: Duration,
    /// list 每页的总时长。
    pub list: Duration,
    /// 发过 progress 之后，多久没有新的 progress 算卡住。
    pub idle: Duration,
    /// 发出 waiting 之后最长等多久。
    pub waiting: Duration,
    /// SIGTERM 到 SIGKILL 的宽限。
    pub grace: Duration,
    pub stdout_bytes: u64,
    pub stderr_lines: usize,
    /// fetch 输出目录的总大小上限。
    pub fetch_dir_bytes: u64,
}

impl Default for CallLimits {
    fn default() -> Self {
        Self {
            canonicalize: Duration::from_secs(10),
            fetch: Duration::from_secs(300),
            sources: Duration::from_secs(30),
            list: Duration::from_secs(60),
            idle: Duration::from_secs(120),
            waiting: Duration::from_secs(600),
            grace: Duration::from_secs(5),
            stdout_bytes: 32 << 20,
            stderr_lines: 200,
            fetch_dir_bytes: 8 << 30,
        }
    }
}

#[derive(Debug, Clone)]
pub struct HostConfig {
    /// 每次调用的暂存目录建在这里；和资料库在同一个文件系统上，移进 `raw/` 才是改名。
    pub work_dir: PathBuf,
    /// `${CACHE}` 的上级，按插件 id 分子目录。
    pub cache_dir: PathBuf,
    /// 用户界面语言，例如 `zh-CN`，插件按它写 `hint`。
    pub locale: String,
    pub limits: CallLimits,
}

/// 一个装好的获取插件。
#[derive(Debug, Clone)]
pub struct FetchPlugin {
    pub header: PluginHeader,
    pub spec: FetchSpec,
    /// 安装目录 `${PLUGIN_DIR}`，绝对路径。
    pub dir: PathBuf,
}

impl FetchPlugin {
    pub fn id(&self) -> &PluginId {
        &self.header.id
    }
}

/// 一个装好的转换插件。WP6（`usuyo-core` 的编排）拿它建 [`ResidentTransform`]；
/// WP4（模型下载）从 `spec.models` 读它声明的模型。
#[derive(Debug, Clone)]
pub struct TransformPlugin {
    pub header: PluginHeader,
    pub spec: TransformSpec,
    /// 安装目录 `${PLUGIN_DIR}`，绝对路径。
    pub dir: PathBuf,
}

impl TransformPlugin {
    pub fn id(&self) -> &PluginId {
        &self.header.id
    }
}

/// 装好的导出插件；宿主现在只列出它们，不调用。
#[derive(Debug, Clone)]
pub struct InstalledPlugin {
    pub manifest: PluginManifest,
    pub dir: PathBuf,
}

/// 没加载的插件和原因，显示在插件列表里（§0.1）。
#[derive(Debug)]
pub struct Rejected {
    pub dir: PathBuf,
    pub error: PluginError,
}

#[derive(Debug, Default)]
pub struct Discovery {
    pub fetch: Vec<FetchPlugin>,
    pub transform: Vec<TransformPlugin>,
    pub other: Vec<InstalledPlugin>,
    pub rejected: Vec<Rejected>,
}

/// 找出装在 `root` 下的插件：`<root>/<owner>/<repo>/<version>/usuyo-plugin.yaml`（§2.8）。
///
/// 每个插件只看版本号最高的那个目录；它不合规就整个不加载，不退回旧版本。
/// 签名和来源证明（§8.3）在安装时核，这里信任已经装进来的目录。
pub fn discover(root: &Path) -> Result<Discovery, PluginError> {
    let io = |path: &Path| {
        let path = path.to_owned();
        move |source| PluginError::Io { path, source }
    };
    let root = root.canonicalize().map_err(io(root))?;
    let mut found = Discovery::default();
    for owner in sorted_dirs(&root).map_err(io(&root))? {
        for repo in sorted_dirs(&owner).map_err(io(&owner))? {
            let mut versions = Vec::new();
            for dir in sorted_dirs(&repo).map_err(io(&repo))? {
                let name = dir.file_name().and_then(OsStr::to_str).unwrap_or_default();
                match semver::Version::parse(name) {
                    Ok(v) => versions.push((v, dir)),
                    Err(e) => found.rejected.push(Rejected {
                        error: PluginError::Manifest {
                            field: "version".into(),
                            message: format!("install directory `{name}` is not a version: {e}"),
                        },
                        dir,
                    }),
                }
            }
            let Some((version, dir)) = versions.into_iter().max_by(|a, b| a.0.cmp(&b.0)) else {
                continue;
            };
            let expected_id = format!(
                "{}/{}",
                owner
                    .file_name()
                    .and_then(OsStr::to_str)
                    .unwrap_or_default(),
                repo.file_name().and_then(OsStr::to_str).unwrap_or_default()
            );
            match load(&dir, &expected_id, &version) {
                Ok(manifest) => match manifest.kind {
                    PluginKind::Fetch(spec) => found.fetch.push(FetchPlugin {
                        header: manifest.header,
                        spec: *spec,
                        dir,
                    }),
                    PluginKind::Transform(spec) => found.transform.push(TransformPlugin {
                        header: manifest.header,
                        spec,
                        dir,
                    }),
                    kind => found.other.push(InstalledPlugin {
                        manifest: PluginManifest {
                            header: manifest.header,
                            kind,
                        },
                        dir,
                    }),
                },
                Err(error) => found.rejected.push(Rejected { dir, error }),
            }
        }
    }
    Ok(found)
}

fn sorted_dirs(dir: &Path) -> std::io::Result<Vec<PathBuf>> {
    let mut dirs = Vec::new();
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        if entry.file_type()?.is_dir() {
            dirs.push(entry.path());
        }
    }
    dirs.sort();
    Ok(dirs)
}

fn load(
    dir: &Path,
    expected_id: &str,
    version: &semver::Version,
) -> Result<PluginManifest, PluginError> {
    let path = dir.join(MANIFEST_FILE);
    let text = std::fs::read_to_string(&path).map_err(|source| PluginError::Io { path, source })?;
    let parsed = PluginManifest::from_yaml(&text)?;
    for field in &parsed.unknown_fields {
        tracing::warn!(dir = %dir.display(), "unknown manifest field `{field}` ignored");
    }
    let manifest = parsed.value;
    if manifest.header.id.as_str() != expected_id {
        return Err(PluginError::Manifest {
            field: "id".into(),
            message: format!(
                "`{}` does not match its install directory `{expected_id}`",
                manifest.header.id
            ),
        });
    }
    if &manifest.header.version != version {
        return Err(PluginError::Manifest {
            field: "version".into(),
            message: format!(
                "`{}` does not match its install directory `{version}`",
                manifest.header.version
            ),
        });
    }
    Ok(manifest)
}

/// 一次调用的选项。
#[derive(Debug, Clone, Default)]
pub struct CallOptions {
    /// 取消：先 SIGTERM，宽限后杀整个进程组，记 `cancelled`。
    pub cancel: Option<CancellationToken>,
    /// 插件的 progress、waiting 事件转到这里。
    pub events: Option<mpsc::UnboundedSender<PluginEvent>>,
}

/// fetch 的第一个参数（§3.6）：收下时的原链接（有 `resolvedUrl` 时用它），或 list 给的 `ref`。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FetchInput {
    Url(url::Url),
    Ref(String),
}

#[derive(Debug, Clone)]
pub struct FetchRequest {
    pub input: FetchInput,
    /// 想要的输出，`outputs` 的子集；`None` 是全要。
    pub want: Option<BTreeSet<OutputKind>>,
    /// 导入文件的副本（§3.9），传成 `USUYO_INPUT`。
    pub input_file: Option<PathBuf>,
}

#[derive(Debug, Clone)]
pub struct ListRequest {
    pub source: String,
    pub cursor: Option<String>,
    /// 这个来源上次跟着收跑完的时间。
    pub since: Option<jiff::Timestamp>,
    /// 导入文件的副本（§3.9），传成 `USUYO_INPUT`。
    pub input_file: Option<PathBuf>,
}

/// 一次成功的 fetch：校验过的输出和它的目录。
///
/// 调用方把 [`out_dir`](Self::out_dir) 整个改名移进 `raw/<snapshotId>/`；
/// 这个值丢掉时，暂存目录里剩下的（没移走的输出、`${TMP}`）一起删掉。
#[derive(Debug)]
pub struct FetchedSnapshot {
    /// `completeness`、`missing` 已按实际输出改正（§3.1）。
    pub output: FetchOutput,
    /// 实际拿到的输出集合；比完整度只比它。
    pub obtained: BTreeSet<OutputKind>,
    /// 输出目录里的全部文件，按路径排好。
    pub files: Vec<OutFile>,
    call_dir: tempfile::TempDir,
}

impl FetchedSnapshot {
    pub fn out_dir(&self) -> PathBuf {
        self.call_dir.path().join("out")
    }
}

pub struct PluginHost {
    config: HostConfig,
    plugins: Vec<FetchPlugin>,
    /// 插件 id → 用户给它的非密钥设置（§2.11），调用时写成 `USUYO_SETTINGS` 指的文件。
    settings: BTreeMap<String, Value>,
    logins: Arc<credentials::Logins>,
    /// 要联网的插件走哪个代理。
    proxy: ProxySetting,
}

impl PluginHost {
    /// `plugins` 的顺序就是用户排的顺序（§3.1）。
    pub fn new(config: HostConfig, plugins: Vec<FetchPlugin>) -> Self {
        let logins = Arc::new(credentials::Logins::none());
        Self {
            config,
            plugins,
            settings: BTreeMap::new(),
            logins,
            proxy: ProxySetting::Direct,
        }
    }

    /// 要联网的插件经哪个代理出去（按机器的 `network.proxy`）。没调这个时直连。
    /// 调用方：`usuyo-core` 的 `Engine::open`。
    pub fn with_proxy(mut self, proxy: ProxySetting) -> Self {
        self.proxy = proxy;
        self
    }

    /// 凭据和密钥存在 `vault`（§6）。`private_dir` 是按机器的目录（不在资料库里）：
    /// 交给插件的凭据文件建在它的 `run/` 下，登录索引是 `logins.json`。
    /// 没调这个时插件拿不到任何凭据，存凭据报错。
    pub fn with_vault(mut self, vault: Arc<dyn Vault>, private_dir: &Path) -> Self {
        self.logins = Arc::new(credentials::Logins::new(private_dir, vault));
        self
    }

    /// 清掉上次没删掉的凭据目录（进程被杀时会剩下）。
    pub fn sweep_credentials(&self) -> Result<(), PluginError> {
        self.logins.sweep()
    }

    pub fn plugin(&self, id: &str) -> Option<&FetchPlugin> {
        self.plugins.iter().find(|p| p.id().as_str() == id)
    }

    /// 装着的插件声明过的登录站点和登录状态。只读索引，不碰钥匙串。
    pub fn sites(&self) -> Result<Vec<DeclaredSite>, PluginError> {
        self.logins.declared(&self.plugins)
    }

    /// 存下登录窗口读出的 cookie（只留属于这个站点的）。写钥匙串，会阻塞。
    pub fn save_site(&self, site: &str, state: &StorageState) -> Result<DeclaredSite, PluginError> {
        self.logins.save(&self.plugins, site, state)
    }

    /// 忘掉一个站点的登录。
    pub fn forget_site(&self, site: &str) -> Result<(), PluginError> {
        self.logins.forget(site)
    }

    /// 装着的插件声明过的访客身份和有没有存着（§2.5.1）。只读索引，不碰钥匙串。
    pub fn visitors(&self) -> Result<Vec<DeclaredVisitor>, PluginError> {
        self.logins.visitors(&self.plugins)
    }

    /// 存下访客窗口读出的 cookie（只留属于这个访客身份的）。写钥匙串，会阻塞。
    pub fn save_visitor(
        &self,
        site: &str,
        state: &StorageState,
    ) -> Result<DeclaredVisitor, PluginError> {
        self.logins.save_visitor(&self.plugins, site, state)
    }

    /// 去掉一个访客身份。同一主机的账号不动。
    pub fn forget_visitor(&self, site: &str) -> Result<(), PluginError> {
        self.logins.forget_visitor(site)
    }

    /// 填（或用空值去掉）一个插件的密钥设置（§2.11）。
    pub fn set_secret(
        &self,
        plugin: &FetchPlugin,
        key: &str,
        value: &str,
    ) -> Result<(), PluginError> {
        self.logins.set_secret(plugin, key, value)
    }

    /// 这个插件填了值的密钥设置。
    pub fn secrets_set(&self, plugin: &PluginId) -> Result<BTreeSet<String>, PluginError> {
        self.logins.secrets_set(plugin)
    }

    /// 带上各插件的设置（插件 id → JSON 对象）。没有设置的插件不传 `USUYO_SETTINGS`。
    pub fn with_settings(mut self, settings: BTreeMap<String, Value>) -> Self {
        self.settings = settings;
        self
    }

    pub fn plugins(&self) -> &[FetchPlugin] {
        &self.plugins
    }

    /// `match` 命中这条链接的插件：平台插件按顺序在前，`web` 一类排在最后（§3.1）。
    pub fn candidates(&self, url: &url::Url) -> Vec<&FetchPlugin> {
        let (web, platform): (Vec<_>, Vec<_>) = self
            .plugins
            .iter()
            .filter(|p| p.spec.matches(url))
            .partition(|p| p.spec.is_web());
        platform.into_iter().chain(web).collect()
    }

    /// 认出这条链接的身份（§3.5）。`NoMatch` 不是错误。
    pub async fn canonicalize(
        &self,
        plugin: &FetchPlugin,
        url: &url::Url,
        opts: &CallOptions,
    ) -> Result<Canonicalized, PluginError> {
        let call = self.start_call(plugin, false)?;
        let args = vec!["canonicalize".into(), url.as_str().into()];
        let (finished, _) = self
            .run(
                plugin,
                &call,
                Invocation {
                    args,
                    input_file: None,
                    cap: None,
                    total: self.config.limits.canonicalize,
                },
                opts,
            )
            .await?;
        let result: Canonicalized = call.decode(plugin, finished)?;
        if let Canonicalized::Match { identity, .. } = &result {
            check_identity(plugin, identity)
                .map_err(|m| call.failed(plugin, ErrorCode::InvalidOutput, m))?;
        }
        Ok(result)
    }

    /// 取一条（§3.6、§3.7）。
    pub async fn fetch(
        &self,
        plugin: &FetchPlugin,
        request: FetchRequest,
        opts: &CallOptions,
    ) -> Result<FetchedSnapshot, PluginError> {
        let unsupported = |message: String| PluginError::Unsupported {
            plugin: plugin.id().clone(),
            message,
        };
        let cap = plugin
            .spec
            .capabilities
            .fetch
            .as_ref()
            .ok_or_else(|| unsupported("no `fetch` capability".into()))?;
        let (kind, input) = match &request.input {
            FetchInput::Url(u) => (FetchInputKind::Url, u.as_str().to_owned()),
            FetchInput::Ref(r) => (FetchInputKind::Ref, r.clone()),
        };
        if !cap.input.contains(&kind) {
            return Err(unsupported(format!("`fetch` does not take {kind:?} input")));
        }
        if let Some(want) = &request.want {
            if want.is_empty() {
                return Err(unsupported("`want` is empty".into()));
            }
            if let Some(extra) = want.iter().find(|w| !cap.outputs.contains(w)) {
                return Err(unsupported(format!("`{extra}` is not among its `outputs`")));
            }
        }
        let wanted = request.want.clone().unwrap_or_else(|| cap.outputs.clone());

        let call = self.start_call(plugin, true)?;
        let out = call.dir.path().join("out");
        let mut args: Vec<OsString> = vec![
            "fetch".into(),
            input.into(),
            "--out".into(),
            out.clone().into(),
        ];
        if let Some(want) = &request.want {
            let joined: Vec<&str> = want.iter().map(|w| w.as_str()).collect();
            args.extend(["--want".into(), joined.join(",").into()]);
        }
        let total = cap
            .uses
            .timeout
            .map_or(self.config.limits.fetch, |t| t.get());
        let (finished, handed) = self
            .run(
                plugin,
                &call,
                Invocation {
                    args,
                    input_file: request.input_file.as_deref(),
                    cap: Some(&cap.uses),
                    total,
                },
                opts,
            )
            .await?;
        let mut output: FetchOutput = call.decode(plugin, finished)?;
        // 换了的会话 cookie 写回钥匙串，不随快照进资料库（§6）。
        if let Some(update) = output.credentials_update.take() {
            let logins = Arc::clone(&self.logins);
            let written = tokio::task::spawn_blocking(move || logins.update(&handed, &update))
                .await
                .map_err(|e| PluginError::Vault(format!("credentials update: {e}")))?;
            if let Err(e) = written {
                tracing::warn!(plugin = %plugin.id(), call = %call.id, error = %e, "credentialsUpdate not saved");
            }
        }
        envelope_text::fetch(&mut output);
        let invalid = |m: String| call.failed(plugin, ErrorCode::InvalidOutput, m);

        if let Some(identity) = &output.identity {
            check_identity(plugin, identity).map_err(invalid)?;
        }
        if let Some(c) = &output.comments
            && c.as_str() != FetchOutput::COMMENTS_FILE
        {
            return Err(invalid(format!(
                "`comments` must be `{}`",
                FetchOutput::COMMENTS_FILE
            )));
        }
        let max = self.config.limits.fetch_dir_bytes;
        let dir = out.clone();
        let files = tokio::task::spawn_blocking(move || {
            outdir::inspect(&dir, max, outdir::FETCH_PATH_UTF16)
        })
        .await
        .map_err(|e| invalid(format!("inspect output directory: {e}")))?
        .map_err(invalid)?;
        let present: BTreeSet<_> = files.iter().map(|f| &f.path).collect();
        if let Some(missing) = output.referenced_files().find(|p| !present.contains(p)) {
            return Err(invalid(format!(
                "`{missing}` is referenced but is not a file in the output"
            )));
        }
        // 一个文件只担一种角色：不然「只留文字」挪走图片时会带走评论文件。
        let mut seen = BTreeSet::new();
        if let Some(twice) = output.referenced_files().find(|p| !seen.insert(*p)) {
            return Err(invalid(format!("`{twice}` is referenced more than once")));
        }

        // 完整度以宿主算的为准（§3.1）：缺的 = 想要的 − 实际拿到的；一样都没拿到算只认出了链接
        let obtained = output.obtained();
        let missing: Vec<OutputKind> = wanted.difference(&obtained).copied().collect();
        let completeness = if obtained.is_disjoint(&wanted) {
            Completeness::Link
        } else if missing.is_empty() {
            Completeness::Full
        } else {
            Completeness::Partial
        };
        if completeness != output.completeness || missing != output.missing {
            tracing::warn!(
                plugin = %plugin.id(),
                claimed = ?output.completeness,
                actual = ?completeness,
                "plugin completeness does not match its output; using the actual one"
            );
        }
        if completeness == Completeness::Link {
            return Err(call.failed(
                plugin,
                ErrorCode::UnsupportedInput,
                "plugin returned none of the wanted outputs (completeness: link)".into(),
            ));
        }
        output.completeness = completeness;
        output.missing = missing;

        Ok(FetchedSnapshot {
            output,
            obtained,
            files,
            call_dir: call.dir,
        })
    }

    /// 列出用户能选的来源（§3.8）。
    pub async fn sources(
        &self,
        plugin: &FetchPlugin,
        opts: &CallOptions,
    ) -> Result<Vec<SourceInfo>, PluginError> {
        let cap =
            plugin
                .spec
                .capabilities
                .list
                .as_ref()
                .ok_or_else(|| PluginError::Unsupported {
                    plugin: plugin.id().clone(),
                    message: "no `list` capability".into(),
                })?;
        let call = self.start_call(plugin, false)?;
        let total = cap
            .uses
            .timeout
            .map_or(self.config.limits.sources, |t| t.get());
        let (finished, _) = self
            .run(
                plugin,
                &call,
                Invocation {
                    args: vec!["sources".into()],
                    input_file: None,
                    cap: Some(&cap.uses),
                    total,
                },
                opts,
            )
            .await?;
        let mut out: SourcesOutput = call.decode(plugin, finished)?;
        envelope_text::sources(&mut out.sources);
        Ok(out.sources)
    }

    /// 列一页（§3.8）。翻页和节奏由调用方管，这里只调一次。
    pub async fn list(
        &self,
        plugin: &FetchPlugin,
        request: ListRequest,
        opts: &CallOptions,
    ) -> Result<ListPage, PluginError> {
        let cap =
            plugin
                .spec
                .capabilities
                .list
                .as_ref()
                .ok_or_else(|| PluginError::Unsupported {
                    plugin: plugin.id().clone(),
                    message: "no `list` capability".into(),
                })?;
        let call = self.start_call(plugin, false)?;
        let mut args: Vec<OsString> = vec!["list".into(), request.source.into()];
        if let Some(c) = request.cursor {
            args.extend(["--cursor".into(), c.into()]);
        }
        if let Some(since) = request.since {
            args.extend([
                "--since".into(),
                Rfc3339::from_timestamp(since).as_str().into(),
            ]);
        }
        let total = cap
            .uses
            .timeout
            .map_or(self.config.limits.list, |t| t.get());
        let (finished, _) = self
            .run(
                plugin,
                &call,
                Invocation {
                    args,
                    input_file: request.input_file.as_deref(),
                    cap: Some(&cap.uses),
                    total,
                },
                opts,
            )
            .await?;
        let mut page: ListPage = call.decode(plugin, finished)?;
        envelope_text::list(&mut page);
        for (i, item) in page.items.iter().enumerate() {
            let check = || -> Result<(), String> {
                match (&item.url, &item.reference) {
                    (None, None) => return Err("has neither `url` nor `ref`".into()),
                    (Some(u), _) => {
                        crate::protocol::check_http_url(u)?;
                    }
                    _ => {}
                }
                if let Some(identity) = &item.identity {
                    check_identity(plugin, identity)?;
                }
                Ok(())
            };
            check().map_err(|m| {
                call.failed(plugin, ErrorCode::InvalidOutput, format!("items[{i}]: {m}"))
            })?;
        }
        Ok(page)
    }

    fn start_call(&self, plugin: &FetchPlugin, with_out: bool) -> Result<Call, PluginError> {
        let io = |path: &Path| {
            let path = path.to_owned();
            move |source| PluginError::Io { path, source }
        };
        let id = uuid::Uuid::now_v7().to_string();
        std::fs::create_dir_all(&self.config.work_dir).map_err(io(&self.config.work_dir))?;
        let dir = tempfile::Builder::new()
            .prefix(&format!("usuyo-call-{id}-"))
            .tempdir_in(&self.config.work_dir)
            .map_err(io(&self.config.work_dir))?;
        let mut make = vec!["tmp/home/.config", "tmp/bin"];
        if with_out {
            make.push("out");
        }
        for sub in make {
            let p = dir.path().join(sub);
            std::fs::create_dir_all(&p).map_err(io(&p))?;
        }
        let cache = self
            .config
            .cache_dir
            .join(plugin.id().owner())
            .join(plugin.id().repo());
        std::fs::create_dir_all(&cache).map_err(io(&cache))?;
        Ok(Call { id, dir, cache })
    }

    /// 起一次进程；回结局和交了凭据的站点。
    async fn run(
        &self,
        plugin: &FetchPlugin,
        call: &Call,
        inv: Invocation<'_>,
        opts: &CallOptions,
    ) -> Result<(Finished, Vec<Site>), PluginError> {
        let Invocation {
            args,
            input_file,
            cap,
            total,
        } = inv;
        let (program, mut command_args) = resolve_command(&plugin.header, &plugin.dir)
            .map_err(|failure| call.fail(plugin, failure, Vec::new()))?;
        command_args.extend(args);
        let mut env = call_env(&self.config, plugin, call, input_file);
        // 凭据和密钥（§6、§2.11）：读钥匙串会阻塞，放到阻塞线程上；文件随 `handover` 在进程结束时删掉。
        let handover = {
            let logins = Arc::clone(&self.logins);
            let plugin = plugin.clone();
            let cap = cap.cloned();
            tokio::task::spawn_blocking(move || logins.handover(&plugin, cap.as_ref()))
                .await
                .map_err(|e| PluginError::Vault(format!("credentials: {e}")))??
        };
        env.extend(handover.env.iter().cloned());
        if let Some(settings) = self.settings.get(plugin.id().as_str()) {
            let path = call.dir.path().join("settings.json");
            std::fs::write(&path, settings.to_string()).map_err(|e| PluginError::Io {
                path: path.clone(),
                source: e,
            })?;
            env.push(("USUYO_SETTINGS".into(), path.into()));
        }
        // 这台机器开着假 IP 模式的代理时告诉插件，不然它会把每个网站都当成内网地址拒掉。
        if tokio::task::spawn_blocking(network::behind_fake_ip_proxy)
            .await
            .unwrap_or(false)
        {
            env.push(("USUYO_NETWORK".into(), "fake-ip".into()));
        }
        // 要联网的插件走和浏览器一样的路：宿主定好代理，插件的 HTTP 客户端照它连（不经 `HTTP_PROXY`）。
        if plugin.header.permissions.network.mode != NetworkMode::None {
            let setting = self.proxy.clone();
            if let Some(proxy) =
                tokio::task::spawn_blocking(move || network::resolve_proxy(&setting))
                    .await
                    .map_err(|e| PluginError::Staging {
                        what: "proxy".into(),
                        source: std::io::Error::other(e),
                    })?
            {
                env.push(("USUYO_PROXY".into(), proxy.into()));
            }
        }
        let spawn = Spawn {
            program,
            args: command_args,
            env,
            cwd: call.dir.path().join("tmp"),
        };
        let mut finished = process::run(spawn, total, &self.config.limits, opts)
            .await
            .map_err(|e| {
                call.failed(
                    plugin,
                    ErrorCode::Crashed,
                    format!("could not run the plugin: {e}"),
                )
            })?;
        for line in &mut finished.stderr {
            credentials::scrub(line, &handover.values);
        }
        finished.scrub = handover.values.clone();
        let sites = handover.sites.clone();
        drop(handover);
        Ok((finished, sites))
    }
}

/// 一次进程要的：参数、导入文件、这次用的能力（决定交哪些站点的凭据，`canonicalize` 没有）、总时长。
struct Invocation<'a> {
    args: Vec<OsString>,
    input_file: Option<&'a Path>,
    cap: Option<&'a CapabilityUse>,
    total: Duration,
}

/// 一次调用的 id、暂存目录和缓存目录。
struct Call {
    id: String,
    dir: tempfile::TempDir,
    cache: PathBuf,
}

impl Call {
    fn fail(&self, plugin: &FetchPlugin, failure: CallFailure, stderr: Vec<String>) -> PluginError {
        PluginError::Call {
            plugin: plugin.id().clone(),
            call_id: self.id.clone(),
            failure: Box::new(failure),
            stderr,
        }
    }

    fn failed(&self, plugin: &FetchPlugin, code: ErrorCode, message: String) -> PluginError {
        self.fail(plugin, CallFailure::new(code, message), Vec::new())
    }

    /// 把进程的结局和 stdout 变成结果（§3.2、§3.3）：成败以信封为准，退出码只是兜底。
    fn decode<T: DeserializeOwned>(
        &self,
        plugin: &FetchPlugin,
        finished: Finished,
    ) -> Result<T, PluginError> {
        let Finished {
            ending,
            stdout,
            stderr,
            scrub,
        } = finished;
        let host = |code: ErrorCode, message: String| CallFailure::new(code, message);
        let status = match ending {
            Ending::TimedOut => {
                return Err(self.fail(
                    plugin,
                    host(ErrorCode::Timeout, "timed out and was killed".into()),
                    stderr,
                ));
            }
            Ending::Cancelled => {
                return Err(self.fail(
                    plugin,
                    host(ErrorCode::Cancelled, "cancelled".into()),
                    stderr,
                ));
            }
            Ending::StdoutOverflow => {
                return Err(self.fail(
                    plugin,
                    host(
                        ErrorCode::InvalidOutput,
                        "stdout exceeded its size limit".into(),
                    ),
                    stderr,
                ));
            }
            Ending::Exited(status) => status,
        };
        let value: Value = match serde_json::from_slice(&stdout) {
            Ok(v) => v,
            Err(e) => {
                let failure = if status.code() == Some(2) {
                    host(
                        ErrorCode::InvalidArgs,
                        "usage error: exit status 2 without an envelope".into(),
                    )
                } else {
                    host(
                        ErrorCode::Crashed,
                        format!("{status} without a parseable envelope: {e}"),
                    )
                };
                return Err(self.fail(plugin, failure, stderr));
            }
        };
        let Value::Object(mut map) = value else {
            return Err(self.fail(
                plugin,
                host(
                    ErrorCode::InvalidOutput,
                    "stdout is not a JSON object".into(),
                ),
                stderr,
            ));
        };
        let ok = map.remove("ok");
        let mut unknown = Vec::new();
        // 只说是哪一栏不对：解析器的原话会引用那一栏的值，值里可能夹着交出去的凭据。
        let invalid = |(field, _): (String, String)| {
            host(
                ErrorCode::InvalidOutput,
                format!("envelope at `{field}` has the wrong type or shape"),
            )
        };
        let result = match ok {
            Some(Value::Bool(true)) => {
                crate::protocol::deserialize_tracked::<T>(Value::Object(map), &mut unknown)
                    .map_err(invalid)
            }
            Some(Value::Bool(false)) => {
                #[derive(serde::Deserialize)]
                struct Failed {
                    error: CallFailure,
                }
                match crate::protocol::deserialize_tracked::<Failed>(
                    Value::Object(map),
                    &mut unknown,
                ) {
                    Ok(Failed { mut error }) => Err(match error.check_from_plugin() {
                        Ok(()) => {
                            // 先洗再截：截短以后被切开的值就认不出来了
                            credentials::scrub(&mut error.message, &scrub);
                            if let Some(h) = &mut error.hint {
                                credentials::scrub(h, &scrub);
                            }
                            envelope_text::failure(&mut error);
                            error
                        }
                        Err(why) => host(ErrorCode::InvalidOutput, why),
                    }),
                    Err(e) => Err(invalid(e)),
                }
            }
            _ => Err(host(
                ErrorCode::InvalidOutput,
                "envelope has no boolean `ok`".into(),
            )),
        };
        for field in unknown {
            // 字段名来自插件：按 Debug 写，换行进不了日志
            tracing::warn!(plugin = %plugin.id(), call = %self.id, field = ?field, "unknown output field ignored");
        }
        result.map_err(|mut failure| {
            // 宿主拼的错误里也可能带着插件给的字（栏名、平台名），在这里再过一遍。
            credentials::scrub(&mut failure.message, &scrub);
            if let Some(hint) = &mut failure.hint {
                credentials::scrub(hint, &scrub);
            }
            self.fail(plugin, failure, stderr)
        })
    }
}

/// 身份的平台要是这个插件声明过的；`file` 总是可以（§3.7）。
fn check_identity(plugin: &FetchPlugin, identity: &Identity) -> Result<(), String> {
    match identity {
        Identity::File => Ok(()),
        other
            if plugin
                .spec
                .platforms
                .iter()
                .any(|p| p.as_str() == other.platform()) =>
        {
            Ok(())
        }
        other => Err(format!(
            "identity platform `{}` is not among the plugin's `platforms`",
            other.platform()
        )),
    }
}

/// 展开 `run.command`：`${PLUGIN_DIR}` 换成安装目录；不在安装目录里的第一项按运行时名字找。
fn resolve_command(
    header: &PluginHeader,
    plugin_dir: &Path,
) -> Result<(OsString, Vec<OsString>), CallFailure> {
    let dir = plugin_dir.to_string_lossy();
    let mut parts = header
        .run
        .command
        .iter()
        .map(|part| OsString::from(part.replace(PLUGIN_DIR_PLACEHOLDER, &dir)));
    let first = parts.next().unwrap_or_default();
    let program = if header.run.command[0].starts_with(PLUGIN_DIR_PLACEHOLDER) {
        first
    } else {
        find_runtime(&first).ok_or_else(|| {
            let mut f = CallFailure::new(
                ErrorCode::DependencyMissing,
                format!("runtime `{}` was not found", first.to_string_lossy()),
            );
            if let Ok(need) =
                crate::protocol::Need::try_from(format!("runtime:{}", first.to_string_lossy()))
            {
                f.needs.push(need);
            }
            f
        })?
    };
    Ok((program, parts.collect()))
}

/// 按 §2.13 的顺序找运行时：宿主的 `PATH`，再加 Homebrew 和 `~/.local/bin`。
/// 用户指定路径、运行时表、版本检查和 `${TMP}/bin` 的链接还没做。
fn find_runtime(name: &OsStr) -> Option<OsString> {
    let mut dirs: Vec<PathBuf> = std::env::var_os("PATH")
        .map(|p| std::env::split_paths(&p).collect())
        .unwrap_or_default();
    dirs.extend([
        PathBuf::from("/opt/homebrew/bin"),
        PathBuf::from("/usr/local/bin"),
    ]);
    if let Some(home) = std::env::var_os("HOME") {
        dirs.push(PathBuf::from(home).join(".local/bin"));
    }
    dirs.into_iter()
        .map(|d| d.join(name))
        .find(|p| is_executable(p))
        .map(PathBuf::into_os_string)
}

fn is_executable(path: &Path) -> bool {
    let Ok(meta) = std::fs::metadata(path) else {
        return false;
    };
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        meta.is_file() && meta.permissions().mode() & 0o111 != 0
    }
    #[cfg(not(unix))]
    {
        meta.is_file()
    }
}

/// 清空后只传这些（§3.2）。凭据（§6）、设置文件、本机代理（§3.2 网络出口）、设备变量接在这里。
fn call_env(
    config: &HostConfig,
    plugin: &FetchPlugin,
    call: &Call,
    input_file: Option<&Path>,
) -> Vec<(OsString, OsString)> {
    let tmp = call.dir.path().join("tmp");
    let mut env = plugin_env(
        &plugin.header,
        &PluginDirs {
            plugin: &plugin.dir,
            cache: &call.cache,
            home: &tmp.join("home"),
            tmp: &tmp,
            bin: &tmp.join("bin"),
        },
        &config.locale,
    );
    env.insert("USUYO_CALL_ID".into(), call.id.clone().into());
    if let Some(input) = input_file {
        env.insert("USUYO_INPUT".into(), input.into());
    }
    env.into_iter().collect()
}

/// 一个插件进程看到的几个目录。
struct PluginDirs<'a> {
    plugin: &'a Path,
    cache: &'a Path,
    home: &'a Path,
    tmp: &'a Path,
    bin: &'a Path,
}

/// 获取插件和转换插件都有的那部分环境（§3.2、§4.1）：清单声明要透传的，再加宿主设的，同名以宿主为准。
fn plugin_env(
    header: &PluginHeader,
    dirs: &PluginDirs<'_>,
    locale: &str,
) -> BTreeMap<OsString, OsString> {
    let mut env: BTreeMap<OsString, OsString> = BTreeMap::new();
    let mut set = |k: &str, v: OsString| {
        env.insert(k.into(), v);
    };
    // 先放清单声明要透传的，宿主自己设的随后覆盖
    for name in &header.permissions.env {
        if let Some(v) = std::env::var_os(name) {
            set(name, v);
        }
    }
    set("USUYO_PROTOCOL", "1".into());
    set("USUYO_PLUGIN_DIR", dirs.plugin.into());
    set("USUYO_CACHE_DIR", dirs.cache.into());
    set("USUYO_LOCALE", locale.into());
    set("HOME", dirs.home.into());
    set("XDG_CONFIG_HOME", dirs.home.join(".config").into());
    set("XDG_CACHE_HOME", dirs.cache.into());
    for k in ["TMPDIR", "TMP", "TEMP"] {
        set(k, dirs.tmp.into());
    }
    let mut path = vec![dirs.bin.to_owned()];
    #[cfg(unix)]
    path.extend(["/usr/bin", "/bin", "/usr/sbin", "/sbin"].map(PathBuf::from));
    #[cfg(windows)]
    {
        for k in ["SystemRoot", "windir"] {
            if let Some(v) = std::env::var_os(k) {
                set(k, v.clone());
                path.push(PathBuf::from(v).join("System32"));
            }
        }
    }
    match std::env::join_paths(path) {
        Ok(joined) => set("PATH", joined),
        Err(e) => tracing::warn!("plugin PATH not set: {e}"),
    }
    env
}
