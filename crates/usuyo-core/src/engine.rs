//! 引擎：一个打开着的资料库、它的索引、任务队列和插件宿主。
//!
//! 读写资料库的方法是同步的（文件操作），异步那边放进 `spawn_blocking` 调；
//! 取内容（[`Engine::take`]、后台任务）是异步的，等插件时不拿着资料库的锁。

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
use std::time::Duration;

use tokio::sync::{Notify, broadcast};
use usuyo_contract::{ItemState, Manifest, Op, OpTarget, SetChange, TopicMeta};
use usuyo_index::{Filter, SearchIndex, SearchRequest, Visibility};
use usuyo_library::{Batch, ItemId, Library};
use usuyo_plugin::{CallLimits, HostConfig, PluginHost, discover};

use crate::content::{self, Content};
use crate::index_doc::index_doc;
use crate::jobs::{JobState, Jobs};
use crate::layout::{HashLayout, Layout, LayoutItem};
use crate::view::{
    Comment, Event, Facet, HitLayer, ImageRef, ItemSummary, ItemView, MapPoint, MapView, Platform,
    SearchHit, Segment, Stored, Topic,
};
use crate::{Error, Result};

/// 失败以后多久再试：`base × 4^已试次数`，最多 6 小时；试满 `max_attempts` 次就停。
#[derive(Debug, Clone, Copy)]
pub struct RetryPolicy {
    pub base: Duration,
    pub max_attempts: u32,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            base: Duration::from_secs(30),
            max_attempts: 6,
        }
    }
}

impl RetryPolicy {
    pub(crate) fn delay(&self, attempts: u32) -> Duration {
        let factor = 4u32.saturating_pow(attempts.min(10));
        self.base
            .saturating_mul(factor)
            .min(Duration::from_secs(6 * 3600))
    }
}

#[derive(Debug, Clone)]
pub struct EngineConfig {
    pub library: PathBuf,
    /// 按机器的根目录（SPEC §0.1）。
    pub machine_root: PathBuf,
    /// 装插件的目录，按顺序找；同一个插件 id 先找到的算。
    pub plugin_roots: Vec<PathBuf>,
    pub limits: CallLimits,
    pub retry: RetryPolicy,
    pub locale: String,
    /// 插件 id → 用户给它的设置（协议 §2.11），来自按机器的 `settings.json`。
    pub plugin_settings: BTreeMap<String, serde_json::Value>,
}

impl EngineConfig {
    /// 插件默认在 `<machine_root>/plugins`。
    pub fn new(library: impl Into<PathBuf>, machine_root: impl Into<PathBuf>) -> Self {
        let machine_root = machine_root.into();
        Self {
            library: library.into(),
            plugin_roots: vec![machine_root.join("plugins")],
            machine_root,
            limits: CallLimits::default(),
            retry: RetryPolicy::default(),
            locale: "zh-CN".into(),
            plugin_settings: BTreeMap::new(),
        }
    }
}

/// 谁从哪里动的手：决定 `origins[].entry` 和改动记录的 `via`。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Via {
    /// 窗口里粘贴。
    App,
    Cli,
    /// 带令牌调本机接口的别的程序。
    Api,
}

impl Via {
    pub(crate) fn entry(self) -> &'static str {
        match self {
            Via::App => "paste",
            Via::Cli => "cli",
            Via::Api => "api",
        }
    }

    pub(crate) fn via(self) -> &'static str {
        match self {
            Via::App => "app",
            Via::Cli => "cli",
            Via::Api => "api",
        }
    }

    pub(crate) fn batch(self) -> Result<Batch> {
        Ok(Batch::new("user", self.via())?)
    }
}

/// 媒体地址怎么拼：`<base>/media/<itemId>/<layer>?k=<key>`。
/// `key` 是守护进程每次启动时随机生成的，`<img>`、`<video>` 带不了令牌，靠它。
#[derive(Debug, Clone, Default)]
pub struct Links {
    pub base: String,
    pub key: Option<String>,
}

impl Links {
    pub fn url(&self, item: &str, layer: &str) -> String {
        match &self.key {
            Some(k) => format!("{}/media/{item}/{layer}?k={k}", self.base),
            None => format!("{}/media/{item}/{layer}", self.base),
        }
    }
}

pub(crate) struct Inner {
    pub library: Mutex<Library>,
    pub index: SearchIndex,
    pub jobs: Jobs,
    pub host: PluginHost,
    /// 平台 id → 显示名，取自装着的获取插件。
    pub platforms: BTreeMap<String, String>,
    pub events: broadcast::Sender<Event>,
    pub wake: Notify,
    pub retry: RetryPolicy,
    pub layout: Box<dyn Layout>,
}

#[derive(Clone)]
pub struct Engine {
    pub(crate) inner: Arc<Inner>,
}

impl Engine {
    /// 建一个新资料库（SPEC §1.1 的目录树）。
    pub fn init_library(root: &Path, machine_root: &Path) -> Result<PathBuf> {
        std::fs::create_dir_all(root).map_err(|e| Error::io(root, e))?;
        let lib = Library::init(root, machine_root)?;
        Ok(lib.root().to_owned())
    }

    /// 打开资料库、索引（新建的或没建完的先从文件重建）、任务队列，找出装着的插件。
    pub fn open(config: EngineConfig) -> Result<Engine> {
        let library = Library::open(&config.library, &config.machine_root)?;
        let machine_dir = library.machine_dir().to_owned();
        std::fs::create_dir_all(&machine_dir).map_err(|e| Error::io(&machine_dir, e))?;
        let index = SearchIndex::open_or_create(&machine_dir.join("index"))?;
        let jobs = Jobs::open(&machine_dir.join("jobs.db"))?;

        let mut plugins: Vec<usuyo_plugin::FetchPlugin> = Vec::new();
        for root in config.plugin_roots.iter().filter(|r| r.is_dir()) {
            let found = discover(root)?;
            for r in &found.rejected {
                tracing::warn!(dir = %r.dir.display(), error = %r.error, "plugin not loaded");
            }
            for p in found.fetch {
                if !plugins.iter().any(|q| q.id() == p.id()) {
                    plugins.push(p);
                }
            }
        }
        let mut platforms = BTreeMap::new();
        // 平台插件的名字优先，通用网页插件的排后。
        for p in plugins
            .iter()
            .filter(|p| !p.spec.is_web())
            .chain(plugins.iter().filter(|p| p.spec.is_web()))
        {
            for platform in &p.spec.platforms {
                platforms
                    .entry(platform.as_str().to_owned())
                    .or_insert_with(|| p.header.name.clone());
            }
        }
        platforms
            .entry("file".into())
            .or_insert_with(|| "文件".into());

        // 插件的输出目录要和资料库在同一个文件系统上，移进 raw/ 才是改名。
        let work_dir = library.root().join(".usuyo/tmp");
        let host = PluginHost::new(
            HostConfig {
                work_dir,
                cache_dir: config.machine_root.join("cache/plugins"),
                locale: config.locale,
                limits: config.limits,
            },
            plugins,
        )
        .with_settings(config.plugin_settings);
        let (events, _) = broadcast::channel(256);
        let engine = Engine {
            inner: Arc::new(Inner {
                library: Mutex::new(library),
                index,
                jobs,
                host,
                platforms,
                events,
                wake: Notify::new(),
                retry: config.retry,
                layout: Box::new(HashLayout),
            }),
        };
        if engine.inner.index.needs_rebuild() {
            engine.reindex()?;
        }
        Ok(engine)
    }

    pub(crate) fn lib(&self) -> MutexGuard<'_, Library> {
        self.inner
            .library
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
    }

    pub fn library_root(&self) -> PathBuf {
        self.lib().root().to_owned()
    }

    /// 已加载的获取插件 id。
    pub fn plugin_ids(&self) -> Vec<String> {
        self.inner
            .host
            .plugins()
            .iter()
            .map(|p| p.id().to_string())
            .collect()
    }

    pub fn subscribe(&self) -> broadcast::Receiver<Event> {
        self.inner.events.subscribe()
    }

    pub(crate) fn emit(&self, e: Event) {
        // 没有人在听时发不出去，不算错。
        let _ = self.inner.events.send(e);
    }

    pub(crate) fn platform(&self, id: &str) -> Platform {
        let name = self
            .inner
            .platforms
            .get(id)
            .cloned()
            .unwrap_or_else(|| id.to_owned());
        Platform {
            id: id.to_owned(),
            name,
        }
    }

    pub(crate) fn summary(&self, m: &Manifest) -> ItemSummary {
        let platform = m.id.rsplit_once('-').map_or("", |(p, _)| p);
        ItemSummary {
            id: m.id.clone(),
            title: m.title.clone(),
            platform: self.platform(platform),
            author: m.author.as_ref().map(|a| a.name.clone()),
            taken_at: m.added_at.clone(),
            topic_ids: m.topics.clone(),
        }
    }

    /// 改了以后照文件重写索引里的这一条。
    pub(crate) fn reindex_item(&self, lib: &Library, id: &ItemId) -> Result<()> {
        let item = lib.item(id)?;
        let c = content::read(lib, &item)?;
        self.inner.index.upsert(&index_doc(&item.manifest, &c))?;
        self.inner.index.commit()?;
        Ok(())
    }

    /// 从资料库重建整个索引，回条目数。
    pub fn reindex(&self) -> Result<usize> {
        let lib = self.lib();
        let mut docs = Vec::new();
        for id in lib.item_ids()? {
            let item = lib.item(&id)?;
            let c = content::read(&lib, &item)?;
            docs.push(index_doc(&item.manifest, &c));
        }
        let n = docs.len();
        self.inner.index.rebuild(docs)?;
        Ok(n)
    }

    /// 读一条；在回收站里的也回，带 `trashedAt`，不带各个侧面。
    pub fn item(&self, id: &str, links: &Links) -> Result<ItemView> {
        let id = ItemId::parse(id)?;
        let lib = self.lib();
        self.view(&lib, &id, links)
    }

    pub(crate) fn view(&self, lib: &Library, id: &ItemId, links: &Links) -> Result<ItemView> {
        match lib.item(id) {
            Ok(item) => {
                let c = content::read(lib, &item)?;
                self.view_of(&item.manifest, &c, links)
            }
            Err(usuyo_library::Error::NotFound(_)) => match lib.trashed_item(id)? {
                Some((entry, m, dir)) => {
                    let c = content::read_trashed(&dir, &m)?;
                    let mut v = self.view_of(&m, &c, links)?;
                    v.trashed_at = Some(entry.trashed_at);
                    Ok(v)
                }
                None => Err(Error::NotFound(id.to_string())),
            },
            Err(e) => Err(e.into()),
        }
    }

    fn view_of(&self, m: &Manifest, c: &Content, links: &Links) -> Result<ItemView> {
        let id = m.id.as_str();
        let mut facets = Vec::new();
        if let Some(v) = &c.video {
            facets.push(Facet::Video {
                snapshot_id: v.snapshot.clone(),
                duration_sec: v.duration_sec.unwrap_or(0.0),
                url: Some(links.url(id, "video")),
                frames: Vec::new(),
            });
        }
        if !c.images.is_empty() {
            let images = c
                .images
                .iter()
                .map(|im| {
                    let (width, height) = imagesize::size(&im.path)
                        .map(|s| (s.width as u32, s.height as u32))
                        .unwrap_or((0, 0));
                    ImageRef {
                        key: im.key.clone(),
                        url: links.url(id, &format!("image/{}", im.key)),
                        width,
                        height,
                    }
                })
                .collect();
            facets.push(Facet::Image { images });
        }
        if let Some((_, paras)) = &c.text {
            facets.push(Facet::Text {
                segments: paras.iter().map(|t| Segment { text: t.clone() }).collect(),
            });
        }
        if let Some((_, comments)) = &c.comments {
            facets.push(Facet::Comments {
                comments: comments
                    .iter()
                    .map(|x| Comment {
                        author: x.author.clone(),
                        text: x.text.clone(),
                        likes: x.likes,
                    })
                    .collect(),
            });
        }
        let failed = self
            .inner
            .jobs
            .job(id)?
            .filter(|j| j.state == JobState::Failed);
        let missing = match failed {
            Some(j) => Some(format!(
                "取不到内容：{}",
                j.error_hint
                    .or(j.error_code)
                    .unwrap_or_else(|| "原因不明".into())
            )),
            None => describe_missing(m),
        };
        Ok(ItemView {
            summary: self.summary(m),
            url: link_of(m),
            stored: stored(m),
            missing,
            note: c.takeaway.clone(),
            facets,
            trashed_at: None,
        })
    }

    /// 搜索：每条一个命中，带命中的那一层。
    pub fn search(&self, query: &str, limit: usize) -> Result<Vec<SearchHit>> {
        let page = self.inner.index.search(&SearchRequest {
            query: query.to_owned(),
            filter: Filter {
                visibility: Visibility::IncludePrivate,
                ..Filter::default()
            },
            limit,
            segments_per_item: 1,
            cursor: None,
        })?;
        Ok(page
            .items
            .into_iter()
            .filter_map(|hit| {
                let seg = hit.segments.into_iter().next()?;
                let first = seg.snippet.highlights.first().cloned().unwrap_or(0..0);
                let layer = match seg.layer.as_str() {
                    "comments" => HitLayer::Comments,
                    "takeaway" | "notes" => HitLayer::Note,
                    "meta" => meta_layer(&seg.snippet, &hit.author),
                    _ => HitLayer::Text,
                };
                let utf16 = |byte: usize| {
                    seg.snippet
                        .text
                        .get(..byte)
                        .map_or(0, |s| s.encode_utf16().count())
                };
                Some(SearchHit {
                    matched: [utf16(first.start), utf16(first.end)],
                    platform: self.platform(&hit.platform),
                    item_id: hit.item_id,
                    title: hit.title,
                    layer,
                    snippet: seg.snippet.text,
                })
            })
            .collect())
    }

    pub fn topics(&self) -> Result<Vec<Topic>> {
        let lib = self.lib();
        topics_of(&lib)
    }

    /// 地图：所有在 `items/` 里的条目。
    pub fn map(&self) -> Result<MapView> {
        let lib = self.lib();
        let topics = topics_of(&lib)?;
        let mut manifests = Vec::new();
        for id in lib.item_ids()? {
            manifests.push(lib.item(&id)?.manifest);
        }
        drop(lib);
        let items: Vec<LayoutItem> = manifests
            .iter()
            .map(|m| LayoutItem {
                id: m.id.clone(),
                topic_ids: m.topics.clone(),
            })
            .collect();
        let placed = self.inner.layout.place(&topics, &items);
        let points = manifests
            .iter()
            .zip(placed.points)
            .map(|(m, (x, y))| {
                let s = self.summary(m);
                MapPoint {
                    item_id: s.id,
                    x,
                    y,
                    title: s.title,
                    platform: s.platform,
                    topic_ids: s.topic_ids,
                    taken_at: s.taken_at,
                }
            })
            .collect();
        Ok(MapView {
            regions: placed.regions,
            points,
        })
    }

    /// 媒体文件在哪：只认 `image/<key>`、`video`、`audio`，经资料库解析，出不了条目文件夹。
    pub fn media(&self, id: &str, layer: &str) -> Result<PathBuf> {
        let ok = layer == "video" || layer == "audio" || layer.starts_with("image/");
        if !ok {
            return Err(Error::InvalidArgs(format!("no media layer `{layer}`")));
        }
        let id = ItemId::parse(id)?;
        match self.lib().resolve_layer(&id, layer, None)? {
            usuyo_library::LayerSource::File(p) => Ok(p),
            usuyo_library::LayerSource::Text(_) => Err(Error::NotFound(layer.to_owned())),
        }
    }

    // —— 你的动作：都经资料库的 op，记进改动记录，能撤销 ——

    fn apply(&self, lib: &mut Library, via: Via, kind: &str, id: &str, op: Op) -> Result<()> {
        let target = OpTarget {
            kind: kind.into(),
            id: id.into(),
        };
        lib.apply(&via.batch()?, &target, op)?;
        Ok(())
    }

    /// 改了条目以后：重建它的索引、发事件、回最新的样子。
    fn after(&self, lib: &Library, id: &ItemId, links: &Links) -> Result<ItemView> {
        self.reindex_item(lib, id)?;
        self.emit(Event::ItemUpdated { id: id.to_string() });
        self.view(lib, id, links)
    }

    pub fn follow(&self, topic_id: &str, followed: bool, via: Via) -> Result<Topic> {
        let mut lib = self.lib();
        self.apply(
            &mut lib,
            via,
            "topic",
            topic_id,
            Op::SetFollowed { value: followed },
        )?;
        let t = lib.find::<TopicMeta>(topic_id)?.doc.meta;
        self.emit(Event::TopicUpdated { id: t.id.clone() });
        Ok(Topic {
            id: t.id,
            name: t.title,
            followed: t.followed,
        })
    }

    /// 记一句；空的就是去掉。
    pub fn set_note(&self, id: &str, text: &str, via: Via, links: &Links) -> Result<ItemView> {
        let id = ItemId::parse(id)?;
        let text = text.trim();
        let mut lib = self.lib();
        let op = Op::SetTakeaway {
            text: (!text.is_empty()).then(|| text.to_owned()),
        };
        self.apply(&mut lib, via, "item", id.as_str(), op)?;
        self.after(&lib, &id, links)
    }

    pub fn set_topic(
        &self,
        id: &str,
        topic_id: &str,
        member: bool,
        via: Via,
        links: &Links,
    ) -> Result<ItemView> {
        let id = ItemId::parse(id)?;
        let mut lib = self.lib();
        lib.find::<TopicMeta>(topic_id)?;
        let change = if member {
            SetChange {
                add: vec![topic_id.into()],
                remove: vec![],
            }
        } else {
            SetChange {
                add: vec![],
                remove: vec![topic_id.into()],
            }
        };
        self.apply(&mut lib, via, "item", id.as_str(), Op::SetTopics(change))?;
        self.after(&lib, &id, links)
    }

    /// 删掉：整条进回收站，索引里去掉，排着的取内容任务也不做了。
    pub fn trash(&self, id: &str, via: Via, links: &Links) -> Result<ItemView> {
        let id = ItemId::parse(id)?;
        let mut lib = self.lib();
        let op = Op::MoveToTrash {
            reason: "delete".into(),
        };
        self.apply(&mut lib, via, "item", id.as_str(), op)?;
        self.inner.jobs.finish(id.as_str())?;
        self.inner.index.delete(id.as_str())?;
        self.inner.index.commit()?;
        self.emit(Event::ItemTrashed { id: id.to_string() });
        self.view(&lib, &id, links)
    }

    /// 整条从回收站拿回来。
    pub fn restore(&self, id: &str, via: Via, links: &Links) -> Result<ItemView> {
        let id = ItemId::parse(id)?;
        let mut lib = self.lib();
        let (entry, _, _) = lib
            .trashed_item(&id)?
            .ok_or_else(|| Error::NotFound(format!("{id} is not in the trash")))?;
        self.apply(&mut lib, via, "trash", &entry.id, Op::Restore {})?;
        self.after(&lib, &id, links)
    }

    /// 只留文字以后，把移进回收站的图和视频拿回来。
    pub fn restore_media(&self, id: &str, via: Via, links: &Links) -> Result<ItemView> {
        let id = ItemId::parse(id)?;
        let mut lib = self.lib();
        let entry = lib
            .trash_entries()?
            .into_iter()
            .rev()
            .find(|e| e.kind == "item" && e.target == id.as_str() && e.reason == "keep-text")
            .ok_or_else(|| Error::NotFound(format!("{id} has no media in the trash")))?;
        self.apply(&mut lib, via, "trash", &entry.id, Op::Restore {})?;
        self.after(&lib, &id, links)
    }

    /// 只留文字：图、视频、音频进回收站。
    pub fn text_only(&self, id: &str, via: Via, links: &Links) -> Result<ItemView> {
        let id = ItemId::parse(id)?;
        let mut lib = self.lib();
        self.apply(&mut lib, via, "item", id.as_str(), Op::KeepText {})?;
        self.after(&lib, &id, links)
    }

    /// 从原帖再取一次；存着的不动，取到了多一份快照。
    pub fn refetch(&self, id: &str, links: &Links) -> Result<ItemView> {
        let id = ItemId::parse(id)?;
        let lib = self.lib();
        let item = lib.item(&id)?;
        let url = fetch_url(&item.manifest)
            .ok_or_else(|| Error::UnsupportedInput(format!("{id} has no link to fetch")))?;
        self.inner
            .jobs
            .enqueue(id.as_str(), &url, crate::fetch::now_ms())?;
        self.inner.wake.notify_one();
        self.view(&lib, &id, links)
    }
}

fn topics_of(lib: &Library) -> Result<Vec<Topic>> {
    let scan = lib.scan::<TopicMeta>()?;
    for (path, why) in &scan.unreadable {
        tracing::warn!(path, why, "topic file not readable");
    }
    let mut out: Vec<Topic> = scan
        .docs
        .into_iter()
        .map(|w| Topic {
            id: w.doc.meta.id,
            name: w.doc.meta.title,
            followed: w.doc.meta.followed,
        })
        .collect();
    out.sort_by(|a, b| a.id.cmp(&b.id));
    Ok(out)
}

/// 索引把标题、作者（和标签、主题）合成一段、一行一样。命中字多的那一行是作者行就算作者，否则算标题。
fn meta_layer(snippet: &usuyo_index::Snippet, author: &str) -> HitLayer {
    let (mut on_author, mut elsewhere) = (0, 0);
    let mut start = 0;
    for line in snippet.text.split('\n') {
        let end = start + line.len();
        let marked: usize = snippet
            .highlights
            .iter()
            .map(|h| h.end.min(end).saturating_sub(h.start.max(start)))
            .sum();
        let bare = line.trim_matches('…');
        if !author.is_empty() && !bare.is_empty() && author.contains(bare) {
            on_author += marked;
        } else {
            elsewhere += marked;
        }
        start = end + 1;
    }
    if on_author > elsewhere {
        HitLayer::Author
    } else {
        HitLayer::Title
    }
}

fn stored(m: &Manifest) -> Stored {
    if m.state == ItemState::TextOnly {
        Stored::TextOnly
    } else if m.snapshots.is_empty() {
        Stored::Link
    } else {
        Stored::Complete
    }
}

/// 原帖链接：主快照的 `resolvedUrl`、`inputUrl`，再没有就身份的 `url`（同 `content.md` 第三行）。
fn link_of(m: &Manifest) -> String {
    let primary = m
        .snapshots
        .iter()
        .find(|s| Some(&s.id) == m.primary_snapshot.as_ref());
    primary
        .and_then(|s| s.resolved_url.clone().or_else(|| s.input_url.clone()))
        .or_else(|| m.identities.iter().find_map(|i| i.url.clone()))
        .unwrap_or_default()
}

/// 再取一次用的链接：最近一次快照的原链接，没有就身份的 `url`。
fn fetch_url(m: &Manifest) -> Option<String> {
    m.snapshots
        .iter()
        .rev()
        .find_map(|s| s.resolved_url.clone().or_else(|| s.input_url.clone()))
        .or_else(|| m.identities.iter().find_map(|i| i.url.clone()))
}

/// 「没存下来：图片、视频（原因）」
fn describe_missing(m: &Manifest) -> Option<String> {
    if m.missing.is_empty() {
        return None;
    }
    let names: Vec<&str> = m
        .missing
        .iter()
        .map(|o| match o.as_str() {
            "text" => "正文",
            "html" => "网页原文",
            "images" => "图片",
            "media:video" => "视频",
            "media:audio" => "音频",
            "subtitle" => "字幕",
            "meta" => "标题和作者",
            "comments" => "评论",
            other => other,
        })
        .collect();
    let mut s = format!("没存下来：{}", names.join("、"));
    if let Some(r) = &m.missing_reason {
        s.push('（');
        s.push_str(r);
        s.push('）');
    }
    Some(s)
}
