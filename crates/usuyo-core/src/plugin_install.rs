//! 插件目录（`GET /catalog`）和从订阅源安装、更新、删除插件（协议 §8.3）。
//!
//! 装：从订阅源的索引取这台电脑的包，下进按机器的 `plugins/.staging/`，大小和 sha256 都和索引一样才往下走；
//! 解开（只许普通文件和文件夹、普通的相对路径，项数和总大小有上限），包里的 `usuyo-plugin.yaml` 的 `id`、
//! `version` 要和索引一样，`run.command` 指的程序要在；写上安装记录 `.usuyo-install.json`，整个改名成
//! `plugins/<owner>/<repo>/<version>/`，再重新找一遍插件换上，不重启。哪一步不成都不留下东西。
//!
//! 更新：只从记录里的那个订阅源，只往高的版本；新版本放好以后关掉这个插件的常驻进程，删掉旧版本的文件夹。
//! 删除：关掉常驻进程，删掉 `plugins/<owner>/<repo>/`。
//!
//! 没有安装记录的（`just dev-plugins` 放进来的、`builtin/` 的、别的插件目录里的）是开发版本：
//! 订阅源不覆盖、不更新、不删它；同一个 id 下只要有一个版本文件夹没有记录就算。

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, PoisonError};

use serde::{Deserialize, Serialize};
use usuyo_plugin::protocol::{MANIFEST_FILE, PluginHeader, PluginManifest};

/// `run.command` 里插件文件夹的占位（协议 §2.8）。
const PLUGIN_DIR_PLACEHOLDER: &str = "${PLUGIN_DIR}";

use crate::catalog::{
    ASSET_LIMIT, Index, IndexAsset, IndexPlugin, IndexRelease, pick, this_target,
};
use crate::engine::Engine;
use crate::models::{ModelError, hash_file};
use crate::unpack::{Format, Limits, UnpackError, Unpacked, unpack};
use crate::view::{
    CatalogAction, CatalogAsset, CatalogBlocked, CatalogInstalled, CatalogMethod, CatalogPlatform,
    CatalogPlugin, CatalogRelease, CatalogView, Event, InstallOrigin,
};
use crate::words::{self, Lang, Said};
use crate::{Error, Result};

/// 安装记录：每个从订阅源装的版本文件夹里一份。
pub(crate) const RECORD: &str = ".usuyo-install.json";
/// 解开以后最多多少字节、几项。
const UNPACKED_LIMIT: u64 = 512 << 20;
const MAX_ENTRIES: usize = 10_000;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Record {
    source: String,
    version: String,
    asset: String,
    sha256: String,
    installed_at: String,
}

/// 正在做的和上一次没成的，按插件 id。
#[derive(Default)]
pub(crate) struct Actions {
    running: Mutex<HashMap<String, CatalogAction>>,
    failed: Mutex<HashMap<String, Said>>,
}

impl Actions {
    fn running(&self) -> std::sync::MutexGuard<'_, HashMap<String, CatalogAction>> {
        self.running.lock().unwrap_or_else(PoisonError::into_inner)
    }
    fn failed(&self) -> std::sync::MutexGuard<'_, HashMap<String, Said>> {
        self.failed.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

/// 装着的一个插件（任何一种）。
struct Have {
    header: PluginHeader,
    kind: &'static str,
    platforms: Vec<String>,
    methods: Vec<CatalogMethod>,
    dir: PathBuf,
}

fn wire<T: Serialize>(x: &T) -> String {
    serde_json::to_value(x)
        .ok()
        .and_then(|v| v.as_str().map(str::to_owned))
        .unwrap_or_default()
}

fn hinted(source: Error, hint: Said) -> Error {
    Error::Hinted {
        source: Box::new(source),
        hint,
    }
}

fn read_record(dir: &Path) -> Option<Record> {
    let path = dir.join(RECORD);
    let bytes = match std::fs::read(&path) {
        Ok(b) => b,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return None,
        Err(e) => {
            tracing::warn!(path = %path.display(), error = %e, "install record not readable");
            return None;
        }
    };
    serde_json::from_slice(&bytes)
        .inspect_err(
            |e| tracing::warn!(path = %path.display(), error = %e, "install record not readable"),
        )
        .ok()
}

/// 一次安装或更新要的：从哪个订阅源、哪一版、哪个包。
struct Plan {
    id: String,
    source: String,
    release: IndexRelease,
    asset: IndexAsset,
    update: bool,
}

/// 安装没成：日志里的英文和给人看的一句。
#[derive(Debug)]
struct Failed {
    error: String,
    hint: Said,
}

impl From<ModelError> for Failed {
    fn from(e: ModelError) -> Failed {
        let hint = match &e {
            ModelError::Status(404) => words::INSTALL_NOT_ON_RELEASE,
            ModelError::TooLarge(_) | ModelError::Short { .. } | ModelError::Sha256 { .. } => {
                words::INSTALL_MISMATCH
            }
            ModelError::Io { .. } => words::INSTALL_IO,
            ModelError::Archive(_) => words::INSTALL_ARCHIVE,
            _ => words::INSTALL_NETWORK,
        };
        Failed {
            error: e.to_string(),
            hint: hint.said(),
        }
    }
}

impl From<UnpackError> for Failed {
    fn from(e: UnpackError) -> Failed {
        let hint = match &e {
            UnpackError::Archive(_) => words::INSTALL_ARCHIVE,
            UnpackError::Io { .. } => words::INSTALL_IO,
        };
        Failed {
            error: e.to_string(),
            hint: hint.said(),
        }
    }
}

fn io_failed(path: &Path, e: std::io::Error) -> Failed {
    Failed {
        error: format!("io error at {}: {e}", path.display()),
        hint: words::INSTALL_IO.said(),
    }
}

fn manifest_failed(reason: impl Into<String>) -> Failed {
    Failed {
        error: reason.into(),
        hint: words::INSTALL_MANIFEST.said(),
    }
}

fn remove_dir(path: &Path) -> std::io::Result<()> {
    match std::fs::remove_dir_all(path) {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(e),
    }
}

impl Engine {
    /// 从订阅源装的插件放在这里（按机器的 `plugins/`）。
    fn install_root(&self) -> PathBuf {
        self.inner.machine_root.join("plugins")
    }

    /// 上次没装完留下的暂存文件夹，打开引擎时清掉。
    pub(crate) fn sweep_staging(machine_root: &Path) -> Result<()> {
        let stage = machine_root.join("plugins/.staging");
        remove_dir(&stage).map_err(|e| Error::io(&stage, e))
    }

    fn haves(&self) -> Vec<Have> {
        let loaded = self.loaded();
        let mut out: Vec<Have> = Vec::new();
        for p in loaded.host.plugins() {
            out.push(Have {
                header: p.header.clone(),
                kind: "fetch",
                platforms: p
                    .spec
                    .platforms
                    .iter()
                    .map(|x| x.as_str().to_owned())
                    .collect(),
                methods: p
                    .spec
                    .methods
                    .iter()
                    .map(|m| CatalogMethod {
                        id: m.id.clone(),
                        name: m.name.clone(),
                        built: m.built,
                        access: wire(&m.access),
                        login: wire(&m.login),
                        account: wire(&m.account),
                    })
                    .collect(),
                dir: p.dir.clone(),
            });
        }
        for p in self.transforms().plugins().iter() {
            out.push(Have {
                header: p.header.clone(),
                kind: "transform",
                platforms: Vec::new(),
                methods: Vec::new(),
                dir: p.dir.clone(),
            });
        }
        for p in &loaded.other {
            out.push(Have {
                header: p.manifest.header.clone(),
                kind: "export",
                platforms: Vec::new(),
                methods: Vec::new(),
                dir: p.dir.clone(),
            });
        }
        out
    }

    /// 装着的这一版从哪来：有记录、在按机器的插件目录里、同一个 id 下的版本文件夹都有记录，才算从订阅源装的。
    fn origin_of(&self, id: &str, dir: &Path) -> (InstallOrigin, Option<Record>) {
        // 找插件时目录取的是规范路径（macOS 上 /var 是 /private/var）：比之前也规范一下
        let root = self.install_root();
        let root = root.canonicalize().unwrap_or(root);
        let in_root = dir.starts_with(&root);
        let all_recorded = std::fs::read_dir(root.join(id))
            .map(|entries| {
                entries
                    .filter_map(std::result::Result::ok)
                    .filter(|e| e.file_type().is_ok_and(|t| t.is_dir()))
                    .filter(|e| !e.file_name().to_string_lossy().starts_with('.'))
                    .all(|e| e.path().join(RECORD).is_file())
            })
            .unwrap_or(false);
        match read_record(dir) {
            Some(r) if in_root && all_recorded && !id.starts_with("builtin/") => {
                (InstallOrigin::Source, Some(r))
            }
            _ => (InstallOrigin::Development, None),
        }
    }

    /// 订阅源里第一个列着这个插件的。
    fn listing(&self, id: &str) -> Option<(String, Arc<Index>, IndexPlugin)> {
        self.catalog()
            .indexes()
            .into_iter()
            .find_map(|(source, index)| {
                let p = index.plugins.iter().find(|p| p.id == id)?.clone();
                Some((source, index, p))
            })
    }

    /// 插件目录：订阅源里的和装着的合在一起，照订阅源的顺序，后面是只装着、不在订阅源里的。
    pub fn plugin_catalog(&self) -> CatalogView {
        let lang = self.lang();
        let haves = self.haves();
        let mut plugins = Vec::new();
        let mut listed = std::collections::BTreeSet::new();
        for (source, index) in self.catalog().indexes() {
            for p in &index.plugins {
                if !listed.insert(p.id.clone()) {
                    continue;
                }
                let have = haves.iter().find(|h| h.header.id.as_str() == p.id);
                plugins.push(self.entry(lang, Some((&source, &index, p)), have));
            }
        }
        for h in &haves {
            if !listed.contains(h.header.id.as_str()) {
                plugins.push(self.entry(lang, None, Some(h)));
            }
        }
        CatalogView {
            target: this_target().map(str::to_owned),
            sources: self.source_list(),
            plugins,
        }
    }

    /// 目录里的一项。
    pub fn catalog_plugin(&self, id: &str) -> Result<CatalogPlugin> {
        self.plugin_catalog()
            .plugins
            .into_iter()
            .find(|p| p.id == id)
            .ok_or_else(|| {
                hinted(
                    Error::NotFound(format!("plugin {id}")),
                    words::PLUGIN_NOT_IN_SOURCE.said(),
                )
            })
    }

    fn entry(
        &self,
        lang: Lang,
        listed: Option<(&String, &Arc<Index>, &IndexPlugin)>,
        have: Option<&Have>,
    ) -> CatalogPlugin {
        let id = listed
            .map(|(_, _, p)| p.id.clone())
            .or_else(|| have.map(|h| h.header.id.to_string()))
            .unwrap_or_default();
        let platform_name = |pid: &str| {
            listed
                .and_then(|(_, index, _)| index.platforms.get(pid))
                .and_then(|n| pick(n, lang))
                .unwrap_or_else(|| self.platform(pid).name)
        };
        let installed = have.map(|h| {
            let (origin, record) = self.origin_of(&id, &h.dir);
            CatalogInstalled {
                version: h.header.version.to_string(),
                origin,
                source: record.map(|r| r.source),
                dir: h.dir.to_string_lossy().into_owned(),
            }
        });
        let release = listed.and_then(|(_, _, p)| p.release.as_ref());
        let latest = release.map(|r| CatalogRelease {
            version: r.version.to_string(),
            published_at: r.published_at.clone(),
            url: r.url.clone(),
            asset: r.asset().map(|a| CatalogAsset {
                name: a.name.clone(),
                size: a.size,
            }),
        });
        let blocked = self.catalog_blocked(
            &id,
            listed.map(|(s, _, _)| s.as_str()),
            release,
            installed.as_ref(),
        );
        let update = match (&installed, release) {
            (Some(i), Some(r)) => {
                blocked.is_none()
                    && i.origin == InstallOrigin::Source
                    && semver::Version::parse(&i.version).is_ok_and(|v| r.version > v)
            }
            _ => false,
        };
        let (name, description, platforms, methods, kind, repository, license) = match listed {
            Some((_, _, p)) => (
                pick(&p.name, lang).unwrap_or_else(|| p.id.clone()),
                pick(&p.description, lang),
                p.platforms.clone(),
                p.methods
                    .iter()
                    .map(|m| CatalogMethod {
                        id: m.id.clone(),
                        name: pick(&m.name, lang).unwrap_or_else(|| m.id.clone()),
                        built: m.built,
                        access: m.access.clone(),
                        login: m.login.clone(),
                        account: m.account.clone(),
                    })
                    .collect(),
                p.kind.clone(),
                Some(p.repository.clone()),
                Some(p.license.clone()),
            ),
            None => {
                let h = have.expect("an entry is listed or installed");
                (
                    h.header.name.clone(),
                    h.header.description.clone(),
                    h.platforms.clone(),
                    h.methods.clone(),
                    h.kind.to_owned(),
                    h.header.homepage.clone(),
                    Some(h.header.license.clone()),
                )
            }
        };
        let actions = &self.inner.actions;
        CatalogPlugin {
            platforms: platforms
                .iter()
                .map(|pid| CatalogPlatform {
                    id: pid.clone(),
                    name: platform_name(pid),
                })
                .collect(),
            action: actions.running().get(&id).cloned(),
            failure: actions.failed().get(&id).map(|s| s.pick(lang).to_owned()),
            id,
            kind,
            name,
            description,
            repository,
            license,
            methods,
            source: listed.map(|(s, _, _)| s.clone()),
            latest,
            installed,
            update,
            blocked: blocked.map(|(code, hint)| CatalogBlocked {
                code: code.to_owned(),
                hint: hint.pick(lang).to_owned(),
            }),
        }
    }

    /// 为什么装不了（没装着时）或为什么不从订阅源更新、删除（装着时）。
    fn catalog_blocked(
        &self,
        id: &str,
        source: Option<&str>,
        release: Option<&IndexRelease>,
        installed: Option<&CatalogInstalled>,
    ) -> Option<(&'static str, Said)> {
        if id.starts_with("builtin/") {
            return Some(("builtin", words::PLUGIN_BUILTIN.said()));
        }
        if let Some(i) = installed {
            if i.origin == InstallOrigin::Development {
                return Some(("development", words::PLUGIN_DEVELOPMENT.said()));
            }
            if let (Some(from), Some(listed)) = (i.source.as_deref(), source)
                && from != listed
            {
                return Some((
                    "other-source",
                    words::PLUGIN_OTHER_SOURCE.with(&[("source", &from)]),
                ));
            }
        }
        source?;
        let Some(release) = release else {
            return Some(("no-release", words::PLUGIN_NOT_RELEASED.said()));
        };
        if release.asset().is_none() {
            let target = this_target().unwrap_or(std::env::consts::OS);
            return Some((
                "no-asset",
                words::PLUGIN_NO_ASSET.with(&[("target", &target)]),
            ));
        }
        None
    }

    /// 开始从订阅源装一个插件；在后台下载、核对、解开、载入，进展经 `catalog-updated` 事件。
    pub fn install_plugin(&self, id: &str) -> Result<CatalogPlugin> {
        let entry = self.catalog_plugin(id)?;
        if entry.installed.is_some() || self.install_root().join(id).exists() {
            return Err(hinted(
                Error::Conflict(format!("{id} is installed")),
                words::PLUGIN_ALREADY.said(),
            ));
        }
        let plan = self.plan(id, &entry, false)?;
        self.start(plan)
    }

    /// 开始从装它的那个订阅源更新一个插件。
    pub fn update_plugin(&self, id: &str) -> Result<CatalogPlugin> {
        let entry = self.catalog_plugin(id)?;
        let Some(installed) = &entry.installed else {
            return Err(hinted(
                Error::Conflict(format!("{id} is not installed")),
                words::PLUGIN_NOT_INSTALLED.said(),
            ));
        };
        if let Some(b) = &entry.blocked {
            return Err(hinted(
                Error::Conflict(format!("{id}: {}", b.code)),
                Said::plain(b.hint.clone()),
            ));
        }
        if installed.source.as_deref() != entry.source.as_deref() || entry.source.is_none() {
            return Err(hinted(
                Error::NotFound(format!("{id} is not in its source")),
                words::PLUGIN_NOT_IN_SOURCE.said(),
            ));
        }
        if !entry.update {
            return Err(hinted(
                Error::Conflict(format!("{id} is up to date")),
                words::PLUGIN_UP_TO_DATE.said(),
            ));
        }
        let plan = self.plan(id, &entry, true)?;
        self.start(plan)
    }

    fn plan(&self, id: &str, entry: &CatalogPlugin, update: bool) -> Result<Plan> {
        if let Some(b) = &entry.blocked {
            return Err(hinted(
                Error::Conflict(format!("{id}: {}", b.code)),
                Said::plain(b.hint.clone()),
            ));
        }
        let (source, _, p) = self.listing(id).ok_or_else(|| {
            hinted(
                Error::NotFound(format!("plugin {id}")),
                words::PLUGIN_NOT_IN_SOURCE.said(),
            )
        })?;
        let release = p.release.ok_or_else(|| {
            hinted(
                Error::Conflict(format!("{id} has no release")),
                words::PLUGIN_NOT_RELEASED.said(),
            )
        })?;
        let asset = release.asset().cloned().ok_or_else(|| {
            let target = this_target().unwrap_or(std::env::consts::OS);
            hinted(
                Error::UnsupportedInput(format!("{id} has no asset for {target}")),
                words::PLUGIN_NO_ASSET.with(&[("target", &target)]),
            )
        })?;
        Ok(Plan {
            id: id.to_owned(),
            source,
            release,
            asset,
            update,
        })
    }

    /// 记下「正在做」，起后台任务。同一个插件同时只做一件事。
    fn start(&self, plan: Plan) -> Result<CatalogPlugin> {
        self.begin(
            &plan.id,
            if plan.update { "update" } else { "install" },
            "downloading",
        )?;
        let this = self.clone();
        let id = plan.id.clone();
        tokio::spawn(async move {
            let result = this.place(&plan).await;
            this.finish(&plan.id, result).await;
        });
        self.catalog_plugin(&id)
    }

    fn begin(&self, id: &str, kind: &str, phase: &str) -> Result<()> {
        let mut running = self.inner.actions.running();
        if running.contains_key(id) {
            return Err(hinted(
                Error::Conflict(format!("{id} is busy")),
                words::PLUGIN_BUSY.said(),
            ));
        }
        running.insert(
            id.to_owned(),
            CatalogAction {
                kind: kind.to_owned(),
                phase: phase.to_owned(),
                received: None,
                total: None,
            },
        );
        self.inner.actions.failed().remove(id);
        drop(running);
        self.emit(Event::CatalogUpdated);
        Ok(())
    }

    fn phase(&self, id: &str, phase: &str) {
        if let Some(a) = self.inner.actions.running().get_mut(id) {
            a.phase = phase.to_owned();
        }
        self.emit(Event::CatalogUpdated);
    }

    async fn finish(&self, id: &str, result: std::result::Result<(), Failed>) {
        let result = match result {
            Ok(()) => self.reload_plugins().await.map_err(|e| Failed {
                error: e.to_string(),
                hint: words::INSTALL_IO.said(),
            }),
            Err(f) => Err(f),
        };
        self.inner.actions.running().remove(id);
        match result {
            Ok(()) => tracing::info!(plugin = id, "plugin placed and loaded"),
            Err(f) => {
                tracing::warn!(plugin = id, error = %f.error, "plugin not installed");
                self.inner.actions.failed().insert(id.to_owned(), f.hint);
            }
        }
        self.emit(Event::CatalogUpdated);
    }

    /// 下载、核对、解开、查清单、放好；更新时再关掉常驻进程、删掉旧版本。不成就清掉暂存。
    async fn place(&self, plan: &Plan) -> std::result::Result<(), Failed> {
        let staging = self.install_root().join(".staging");
        std::fs::create_dir_all(&staging).map_err(|e| io_failed(&staging, e))?;
        let name = usuyo_library::uuid7();
        let part = staging.join(format!("{name}.part"));
        let stage = staging.join(&name);
        let result = self.place_from(plan, &part, &stage).await;
        for leftover in [&part] {
            if let Err(e) = std::fs::remove_file(leftover)
                && e.kind() != std::io::ErrorKind::NotFound
            {
                tracing::warn!(path = %leftover.display(), error = %e, "download not removed");
            }
        }
        if let Err(e) = remove_dir(&stage) {
            tracing::warn!(path = %stage.display(), error = %e, "staging folder not removed");
        }
        result
    }

    async fn place_from(
        &self,
        plan: &Plan,
        part: &Path,
        stage: &Path,
    ) -> std::result::Result<(), Failed> {
        let total = plan.asset.size;
        let step = AtomicU64::new(u64::MAX);
        let this = self.clone();
        let id = plan.id.clone();
        let progress = move |have: u64| {
            // 每过 2% 说一次
            let now = have.saturating_mul(50) / total.max(1);
            if step.swap(now, Ordering::Relaxed) != now {
                if let Some(a) = this.inner.actions.running().get_mut(&id) {
                    a.received = Some(have);
                    a.total = Some(total);
                }
                this.emit(Event::CatalogUpdated);
            }
        };
        self.catalog()
            .net()
            .download(&plan.asset.url, part, total, &progress)
            .await?;
        self.phase(&plan.id, "verifying");
        let (part_owned, stage_owned) = (part.to_owned(), stage.to_owned());
        let plan_asset = plan.asset.clone();
        let release = plan.release.clone();
        let id = plan.id.clone();
        let source = plan.source.clone();
        let this = self.clone();
        let root = self.install_root();
        let dest = tokio::task::spawn_blocking(move || {
            let got = hash_file(&part_owned)?;
            if got != plan_asset.sha256 {
                return Err(Failed::from(ModelError::Sha256 {
                    expected: plan_asset.sha256.clone(),
                    got,
                }));
            }
            this.phase(&id, "unpacking");
            std::fs::create_dir(&stage_owned).map_err(|e| io_failed(&stage_owned, e))?;
            let mut out = Unpacked::new(
                &stage_owned,
                Limits {
                    entries: MAX_ENTRIES,
                    bytes: UNPACKED_LIMIT.max(ASSET_LIMIT),
                    reserved: RECORD,
                },
            );
            let format = if plan_asset.name.ends_with(".zip") {
                Format::Zip
            } else {
                Format::TarGz
            };
            unpack(&mut out, &part_owned, format)?;
            check_manifest(&stage_owned, &id, &release.version)?;
            let record = Record {
                source,
                version: release.version.to_string(),
                asset: plan_asset.name.clone(),
                sha256: plan_asset.sha256.clone(),
                installed_at: usuyo_library::time::now(),
            };
            let path = stage_owned.join(RECORD);
            let bytes =
                serde_json::to_vec_pretty(&record).map_err(|e| manifest_failed(e.to_string()))?;
            std::fs::write(&path, bytes).map_err(|e| io_failed(&path, e))?;
            let parent = root.join(&id);
            std::fs::create_dir_all(&parent).map_err(|e| io_failed(&parent, e))?;
            let dest = parent.join(release.version.to_string());
            if dest.exists() {
                return Err(io_failed(
                    &dest,
                    std::io::Error::new(std::io::ErrorKind::AlreadyExists, "version folder exists"),
                ));
            }
            std::fs::rename(&stage_owned, &dest).map_err(|e| io_failed(&dest, e))?;
            Ok(dest)
        })
        .await
        .map_err(|e| Failed {
            error: e.to_string(),
            hint: words::INSTALL_IO.said(),
        })??;
        self.phase(&plan.id, "loading");
        if plan.update {
            self.transforms().stop_plugin(&plan.id).await;
            let parent = self.install_root().join(&plan.id);
            let old: Vec<PathBuf> = std::fs::read_dir(&parent)
                .map_err(|e| io_failed(&parent, e))?
                .filter_map(std::result::Result::ok)
                .map(|e| e.path())
                .filter(|p| p.is_dir() && p != &dest)
                .collect();
            for dir in old {
                remove_dir(&dir).map_err(|e| io_failed(&dir, e))?;
            }
        }
        Ok(())
    }

    /// 删掉一个从订阅源装的插件：关掉常驻进程，删掉它的文件夹，换上新的一份插件。
    pub async fn remove_plugin(&self, id: &str) -> Result<CatalogView> {
        let entry = self.catalog_plugin(id)?;
        let Some(installed) = &entry.installed else {
            return Err(hinted(
                Error::Conflict(format!("{id} is not installed")),
                words::PLUGIN_NOT_INSTALLED.said(),
            ));
        };
        if id.starts_with("builtin/") {
            return Err(hinted(
                Error::Conflict(format!("{id} is builtin")),
                words::PLUGIN_BUILTIN.said(),
            ));
        }
        if installed.origin == InstallOrigin::Development {
            return Err(hinted(
                Error::Conflict(format!("{id} is a development install")),
                words::PLUGIN_DEVELOPMENT.said(),
            ));
        }
        self.begin(id, "remove", "removing")?;
        self.transforms().stop_plugin(id).await;
        let dir = self.install_root().join(id);
        let removed =
            tokio::task::spawn_blocking(move || remove_dir(&dir).map_err(|e| Error::io(&dir, e)))
                .await
                .map_err(|e| Error::Task(e.to_string()))
                .and_then(|r| r);
        let result = match removed {
            Ok(()) => self.reload_plugins().await,
            Err(e) => Err(e),
        };
        self.inner.actions.running().remove(id);
        if let Err(e) = &result {
            tracing::warn!(plugin = id, error = %e, "plugin not removed");
            self.inner
                .actions
                .failed()
                .insert(id.to_owned(), words::INSTALL_IO.said());
        }
        self.emit(Event::CatalogUpdated);
        result?;
        Ok(self.plugin_catalog())
    }
}

/// 包里的清单：读得出、合规，`id`、`version` 和索引一样，`run.command` 指的程序在包里（unix 上设成可执行）。
fn check_manifest(
    stage: &Path,
    id: &str,
    version: &semver::Version,
) -> std::result::Result<(), Failed> {
    let path = stage.join(MANIFEST_FILE);
    let text = std::fs::read_to_string(&path)
        .map_err(|e| manifest_failed(format!("{}: {e}", path.display())))?;
    let parsed = PluginManifest::from_yaml(&text).map_err(|e| manifest_failed(e.to_string()))?;
    let header = parsed.value.header;
    if header.id.as_str() != id {
        return Err(manifest_failed(format!(
            "manifest id `{}` is not `{id}`",
            header.id
        )));
    }
    if &header.version != version {
        return Err(manifest_failed(format!(
            "manifest version `{}` is not `{version}`",
            header.version
        )));
    }
    let command = header.run.command.first().cloned().unwrap_or_default();
    if let Some(rest) = command.strip_prefix(PLUGIN_DIR_PLACEHOLDER) {
        let rel = rest.strip_prefix('/').ok_or_else(|| {
            manifest_failed(format!(
                "run.command `{command}` is not a path in the plugin folder"
            ))
        })?;
        let exe = format!("{rel}.exe");
        let mut program = None;
        for r in [rel, exe.as_str()] {
            let p = inside(stage, r)?;
            if p.is_file() {
                program = Some(p);
                break;
            }
        }
        let program =
            program.ok_or_else(|| manifest_failed(format!("`{rel}` is not in the package")))?;
        make_executable(&program).map_err(|e| io_failed(&program, e))?;
    }
    // 交链接的方法写的脚本文件（协议 §2.3.1）：同样要在包里、在插件文件夹里面
    if let usuyo_plugin::protocol::PluginKind::Fetch(spec) = &parsed.value.kind {
        for script in spec.methods.iter().filter_map(|m| m.script.as_deref()) {
            let file = inside(stage, script)?;
            if !file.is_file() {
                return Err(manifest_failed(format!(
                    "script `{script}` is not in the package"
                )));
            }
        }
    }
    Ok(())
}

/// 清单里写的一个相对路径在包里的位置：每一段都是普通的名字（没有根、盘符、`.`、`..`、空段），
/// 解开链接以后还在暂存文件夹里面。不存在的路径照原样回（调用方再看是不是文件）。
fn inside(stage: &Path, rel: &str) -> std::result::Result<PathBuf, Failed> {
    let bad = || {
        manifest_failed(format!(
            "path `{rel}` in the manifest leaves the plugin folder"
        ))
    };
    if rel.is_empty() || rel.contains('\\') {
        return Err(bad());
    }
    let mut out = stage.to_owned();
    for part in rel.split('/') {
        let normal = matches!(
            Path::new(part).components().collect::<Vec<_>>().as_slice(),
            [std::path::Component::Normal(_)]
        );
        if !normal {
            return Err(bad());
        }
        out.push(part);
    }
    let Ok(real) = out.canonicalize() else {
        return Ok(out);
    };
    let root = stage.canonicalize().map_err(|e| io_failed(stage, e))?;
    if !real.starts_with(&root) {
        return Err(bad());
    }
    Ok(real)
}

#[cfg(unix)]
fn make_executable(path: &Path) -> std::io::Result<()> {
    use std::os::unix::fs::PermissionsExt;
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o755))
}

#[cfg(not(unix))]
fn make_executable(_path: &Path) -> std::io::Result<()> {
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::inside;

    #[test]
    fn manifest_paths_stay_in_the_plugin_folder() {
        let tmp = tempfile::tempdir().unwrap();
        let stage = tmp.path().join("stage");
        std::fs::create_dir_all(stage.join("bin")).unwrap();
        std::fs::write(stage.join("bin/run"), "x").unwrap();
        assert!(inside(&stage, "bin/run").unwrap().is_file());
        for bad in [
            "../x",
            "/etc/passwd",
            "",
            "a//b",
            "a/./b",
            "bin/../../x",
            "c:\\x",
        ] {
            assert!(inside(&stage, bad).is_err(), "{bad}");
        }
        #[cfg(unix)]
        {
            std::fs::write(tmp.path().join("outside"), "x").unwrap();
            std::os::unix::fs::symlink(tmp.path().join("outside"), stage.join("link")).unwrap();
            assert!(inside(&stage, "link").is_err());
        }
    }
}
