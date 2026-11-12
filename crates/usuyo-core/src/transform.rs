//! 转换：排任务、配对、编排一条链、写转录（feature 003 §4.1、§4.4、§5）。
//!
//! 一个快照落进资料库（收下、再取一次、导入、交进来的）以后，看它有什么，在 `transform_jobs` 里排上要做的：
//! 有画面和分开的音轨就排 `play`，有画面或声音就排 `asr`，有画面就排 `keyframes`，有图就排 `ocr`；
//! `keyframes` 落地以后排 `frames`。一行只说「这一样还没做好」，具体用哪几个输入、做好了没有，轮到它时再看。
//!
//! 整台机器一次只跑一个转换，和取内容是两条道。自动的只在接着电源时跑；点名的（打开了这条、
//! 再转一次）在电池上也跑。缺模型的等模型，不算失败；失败照取内容的退避再试。
//!
//! 插件的产出都是不可信输入：`ResidentTransform` 核过消息和输出目录，资料库写转录时再核一遍文件。

use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, PoisonError};
use std::time::Duration;

use serde_json::{Map, Value};
use tokio::sync::Notify;
use usuyo_contract::{FetchRecord, ItemState, Keyframes, Manifest, RunRecord};
use usuyo_library::{ItemId, Library, NewRun, hash_run_products};
use usuyo_plugin::protocol::{
    CallFailure, ConsumeKind, ErrorCode, InputRole, ModelSpec, Probe, Produce, RunInput, RunItem,
    RunResult,
};
use usuyo_plugin::{
    CancellationToken, PluginError, ResidentTransform, TransformOutput, TransformPlugin,
    TransformRequest,
};

use crate::engine::Engine;
use crate::fetch::now_ms;
use crate::jobs::{TransformJob, TransformJobKind, TransformRedo, TransformState};
use crate::models::{Begin, Models};
use crate::view::Event;
use crate::{Error, Result};

/// 先后（feature 003 §4.4）：点名的、你手动收下的、导入和交进来的。
pub(crate) const OPEN: i64 = 0;
pub(crate) const BY_HAND: i64 = 1;
pub(crate) const LATER: i64 = 2;

/// 没有到点的任务时，多久再看一次电源和模型。
const IDLE: Duration = Duration::from_secs(30);

/// 接没接电源。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Power {
    Mains,
    Battery,
}

/// 读电源状态。守护进程用 [`SystemPower`]，测试换成自己能拨的。
pub trait PowerSource: Send + Sync + std::fmt::Debug {
    /// 读不出来回 `None`。
    fn read(&self) -> Option<Power>;
}

/// 这台机器的电池：有一块在放电就算在电池上；没有电池（台式机）算接着电。
/// debug 构建里认 `USUYO_TEST_POWER=battery|mains`，验收时模拟拔电（feature 003 §11 第 9 条）。
#[derive(Debug, Default)]
pub struct SystemPower;

impl PowerSource for SystemPower {
    fn read(&self) -> Option<Power> {
        #[cfg(debug_assertions)]
        match std::env::var("USUYO_TEST_POWER").as_deref() {
            Ok("battery") => return Some(Power::Battery),
            Ok("mains") => return Some(Power::Mains),
            _ => {}
        }
        let manager = starship_battery::Manager::new().ok()?;
        let mut discharging = false;
        for battery in manager.batteries().ok()? {
            discharging |= battery.ok()?.state() == starship_battery::State::Discharging;
        }
        Some(if discharging {
            Power::Battery
        } else {
            Power::Mains
        })
    }
}

/// 引擎里转换那一路的东西。
pub(crate) struct Transforms {
    /// 发现的顺序；同一种活有几个插件能做时，默认用最先的那个。
    plugins: Vec<TransformPlugin>,
    resident: HashMap<String, ResidentTransform>,
    pub models: Models,
    /// 按机器的 `settings.json` 里的 `transforms.auto`；设置页改了马上换。
    auto: AtomicBool,
    power: Arc<dyn PowerSource>,
    /// 读不出电源状态时只记一次日志。
    power_unread: AtomicBool,
    pub wake: Notify,
    /// 正在跑的那一个：条目 id 和取消令牌。进回收站、只留文字时取消它。
    running: Mutex<Option<(String, CancellationToken)>>,
    /// `<按机器目录>/cache/transform/`：一条链的中间产物。
    cache: PathBuf,
    /// 插件 id → 用户给它的设置，`run` 的 `options` 从它开始。
    settings: BTreeMap<String, Value>,
}

/// 建 [`Transforms`] 要的。
pub(crate) struct TransformSetup {
    pub plugins: Vec<TransformPlugin>,
    pub config: usuyo_plugin::TransformConfig,
    pub models: Models,
    pub auto: bool,
    pub power: Arc<dyn PowerSource>,
    pub cache: PathBuf,
    pub settings: BTreeMap<String, Value>,
}

impl Transforms {
    /// 上次没跑完的链留下的中间产物在这里清掉：链不跨重启。
    pub fn new(s: TransformSetup) -> Result<Transforms> {
        match std::fs::remove_dir_all(&s.cache) {
            Ok(()) => {}
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(e) => return Err(Error::io(&s.cache, e)),
        }
        let resident = s
            .plugins
            .iter()
            .map(|p| {
                (
                    p.id().to_string(),
                    ResidentTransform::new(p.clone(), s.config.clone()),
                )
            })
            .collect();
        Ok(Transforms {
            plugins: s.plugins,
            resident,
            models: s.models,
            auto: AtomicBool::new(s.auto),
            power: s.power,
            power_unread: AtomicBool::new(false),
            wake: Notify::new(),
            running: Mutex::new(None),
            cache: s.cache,
            settings: s.settings,
        })
    }

    fn on_mains(&self) -> bool {
        match self.power.read() {
            Some(p) => p == Power::Mains,
            None => {
                if !self.power_unread.swap(true, Ordering::Relaxed) {
                    tracing::warn!("power state is not readable; transforms run as if on mains");
                }
                true
            }
        }
    }

    /// 能做这一样的插件：给了 id 就只看它。
    fn plugin_for(&self, job: TransformJobKind, id: Option<&str>) -> Option<&TransformPlugin> {
        let (produce, eats) = needs_of(job);
        self.plugins
            .iter()
            .filter(|p| id.is_none_or(|id| p.id().as_str() == id))
            .find(|p| {
                p.spec.produces.contains(&produce) && p.spec.consumes.iter().any(|c| c.kind == eats)
            })
    }

    /// 做中间一步（`probe`、`audio:pcm16k`）的插件。
    fn helper(&self, produce: Produce) -> Option<&TransformPlugin> {
        self.plugins
            .iter()
            .find(|p| p.spec.produces.contains(&produce))
    }

    fn resident(&self, plugin: &TransformPlugin) -> Result<&ResidentTransform> {
        self.resident
            .get(plugin.id().as_str())
            .ok_or_else(|| Error::NotFound(format!("transform plugin {}", plugin.id())))
    }

    /// 这次要用的模型：清单里 `setting` 对得上这次参数的那一个（没写 `setting` 的总要）。
    fn model_needed<'a>(
        &self,
        plugin: &'a TransformPlugin,
        options: &Map<String, Value>,
    ) -> Option<&'a ModelSpec> {
        plugin
            .spec
            .models
            .iter()
            .find(|m| match (&m.setting, &m.value) {
                (Some(key), Some(value)) => {
                    let chosen = options.get(key).cloned().or_else(|| {
                        plugin
                            .header
                            .settings
                            .iter()
                            .find(|s| &s.key == key)
                            .and_then(|s| s.default.clone())
                    });
                    chosen.as_ref().and_then(Value::as_str) == Some(value.as_str())
                }
                _ => true,
            })
    }

    /// 交给插件的参数：用户给它的设置，再盖上点名时给的。
    fn options(&self, plugin: &TransformPlugin, row: &TransformJob) -> Result<Map<String, Value>> {
        let mut out = match self.settings.get(plugin.id().as_str()) {
            Some(Value::Object(m)) => m.clone(),
            _ => Map::new(),
        };
        if let Some(text) = &row.options {
            let given: Map<String, Value> = serde_json::from_str(text)?;
            out.extend(given);
        }
        Ok(out)
    }

    fn running(&self) -> std::sync::MutexGuard<'_, Option<(String, CancellationToken)>> {
        self.running.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

/// 一样活要插件产出什么、吃什么。
fn needs_of(job: TransformJobKind) -> (Produce, ConsumeKind) {
    match job {
        TransformJobKind::Play => (Produce::Play, ConsumeKind::Video),
        TransformJobKind::Asr => (Produce::Asr, ConsumeKind::AudioPcm16k),
        TransformJobKind::Keyframes => (Produce::Keyframes, ConsumeKind::Video),
        TransformJobKind::Ocr => (Produce::Ocr, ConsumeKind::Image),
        TransformJobKind::Frames => (Produce::Ocr, ConsumeKind::Keyframes),
    }
}

/// 只留文字以后不该再出现的：带二进制产物的那几样。
fn writes_binaries(job: TransformJobKind) -> bool {
    matches!(job, TransformJobKind::Play | TransformJobKind::Keyframes)
}

/// 快照里一个媒体文件或一张图。
#[derive(Debug, Clone)]
struct Source {
    /// 相对条目文件夹，`raw/<snapshotId>/…` 或 `derived/runs/<runId>/keyframes/…`。
    rel: String,
    abs: PathBuf,
    sha256: String,
    snapshot: String,
    mime: Option<String>,
    group: Option<String>,
}

/// 主快照里转换要看的东西（feature 003 §4.1）。
#[derive(Debug, Clone, Default)]
struct Sources {
    video: Option<Source>,
    audio: Option<Source>,
    /// 画面和音轨是不是一段时间线：两边的 `group` 一样就是；都没写 `group` 时 `None`，要量过才知道。
    grouped: Option<bool>,
    /// 图文的图和视频的封面，按 `imageKey` 去重。
    images: Vec<(String, Source)>,
}

impl Sources {
    /// 转口播用哪个：分开的音轨，没有就用画面（看它有没有声音轨）。
    fn speech(&self) -> Option<&Source> {
        self.audio.as_ref().or(self.video.as_ref())
    }

    /// 有没有可能配成一对：恰好一个画面、一个音轨，`group` 没写或写的一样。
    fn may_pair(&self) -> bool {
        self.video.is_some() && self.audio.is_some() && self.grouped != Some(false)
    }
}

fn sources(lib: &Library, id: &ItemId, m: &Manifest) -> Result<Option<Sources>> {
    let Some(snapshot) = m.primary_snapshot.clone() else {
        return Ok(None);
    };
    let rec = lib.fetch_record(id, &snapshot)?;
    let dir = lib.root().join(id.rel_dir());
    Ok(Some(sources_of(&rec, &dir, &snapshot)))
}

fn sources_of(rec: &FetchRecord, dir: &Path, snapshot: &str) -> Sources {
    let source = |path: &str, mime: Option<String>, group: Option<String>| {
        let f = rec.files.iter().find(|f| f.path == path)?;
        let rel = format!("raw/{snapshot}/{path}");
        let abs = dir.join(&rel);
        // 只留文字以后媒体在回收站里，不在这里了。
        abs.is_file().then(|| Source {
            rel,
            abs,
            sha256: f.sha256.clone(),
            snapshot: snapshot.to_owned(),
            mime,
            group,
        })
    };
    let media = |kind: &'static str| rec.media.iter().filter(move |m| m.kind == kind);
    let first = |kind: &'static str| {
        media(kind).find_map(|m| {
            let mime = m
                .extra
                .get("mime")
                .and_then(Value::as_str)
                .map(str::to_owned);
            source(&m.path, mime, m.group.clone())
        })
    };
    let mut s = Sources {
        video: first("video"),
        audio: first("audio"),
        ..Sources::default()
    };
    let one_each = media("video").count() == 1 && media("audio").count() == 1;
    s.grouped = match (&s.video, &s.audio) {
        (Some(v), Some(a)) if v.group.is_some() || a.group.is_some() => {
            let g = &v.group;
            let same = g.is_some()
                && *g == a.group
                && rec.media.iter().filter(|m| m.group == *g).count() == 2;
            Some(same)
        }
        (Some(_), Some(_)) if !one_each => Some(false),
        _ => None,
    };
    let mut seen = BTreeSet::new();
    for path in &rec.images {
        let Some(img) = source(path, None, None) else {
            continue;
        };
        let Some(key) = img.sha256.get(..16).map(str::to_owned) else {
            continue;
        };
        if seen.insert(key.clone()) {
            s.images.push((key, img));
        }
    }
    s
}

/// 画面和音轨能不能配成一对（feature 003 §4.1 的内容规则）：画面文件没有音轨，音频文件没有画面
/// （封面图不算），两者时长差不超过 1 秒或 2%（取大的）。
fn pairs(video: &Probe, audio: &Probe) -> bool {
    use usuyo_plugin::protocol::MediaKind;
    let cover = |codec: &Option<String>| {
        codec
            .as_deref()
            .is_some_and(|c| matches!(c, "mjpeg" | "png" | "bmp" | "gif" | "webp"))
    };
    let video_ok = video.streams.iter().any(|s| s.kind == MediaKind::Video)
        && !video.streams.iter().any(|s| s.kind == MediaKind::Audio);
    let audio_ok = audio.streams.iter().any(|s| s.kind == MediaKind::Audio)
        && !audio
            .streams
            .iter()
            .any(|s| s.kind == MediaKind::Video && !cover(&s.codec));
    let (Some(dv), Some(da)) = (video.duration_sec, audio.duration_sec) else {
        return false;
    };
    let (dv, da) = (dv.get(), da.get());
    let slack = (dv.max(da) * 0.02).max(1.0);
    video_ok && audio_ok && (dv - da).abs() <= slack
}

/// 有没有音频流。
fn has_audio(p: &Probe) -> bool {
    p.streams
        .iter()
        .any(|s| s.kind == usuyo_plugin::protocol::MediaKind::Audio)
}

/// 这一条现在还有哪几样没做好（只看手边的哈希，不量媒体；配不配得上留到轮到 `play` 时）。
fn not_done(lib: &Library, id: &ItemId, m: &Manifest) -> Result<Vec<TransformJobKind>> {
    let mut out = Vec::new();
    if m.state != ItemState::Normal {
        return Ok(out);
    }
    let Some(s) = sources(lib, id, m)? else {
        return Ok(out);
    };
    let done = |job: TransformJobKind, needed: &[String]| lib.job_done(id, job.as_str(), needed);
    if s.may_pair()
        && let (Some(v), Some(a)) = (&s.video, &s.audio)
        && !done(
            TransformJobKind::Play,
            &[v.sha256.clone(), a.sha256.clone()],
        )?
    {
        out.push(TransformJobKind::Play);
    }
    if let Some(src) = s.speech()
        && !done(TransformJobKind::Asr, std::slice::from_ref(&src.sha256))?
    {
        out.push(TransformJobKind::Asr);
    }
    if let Some(v) = &s.video
        && !done(TransformJobKind::Keyframes, std::slice::from_ref(&v.sha256))?
    {
        out.push(TransformJobKind::Keyframes);
    }
    for (_, img) in &s.images {
        if !done(TransformJobKind::Ocr, std::slice::from_ref(&img.sha256))? {
            out.push(TransformJobKind::Ocr);
            break;
        }
    }
    if let Some((_, frames)) = newest_frames(lib, id)?
        && !frames.is_empty()
        && !done(
            TransformJobKind::Frames,
            &frames.iter().map(|f| f.sha256.clone()).collect::<Vec<_>>(),
        )?
    {
        out.push(TransformJobKind::Frames);
    }
    Ok(out)
}

/// 最新一次 `keyframes` 转录和它的帧（文件还在的）。
fn newest_frames(lib: &Library, id: &ItemId) -> Result<Option<(RunRecord, Vec<Source>)>> {
    let Some(run) = lib.job_runs(id, "keyframes")?.into_iter().next() else {
        return Ok(None);
    };
    let index = match lib.run_output(id, &run.id, "keyframes") {
        Ok(p) => p,
        Err(usuyo_library::Error::NotFound(_)) => return Ok(None),
        Err(e) => return Err(e.into()),
    };
    let bytes = std::fs::read(&index).map_err(|e| Error::io(&index, e))?;
    let kf: Keyframes = serde_json::from_slice(&bytes)?;
    if let Some(bad) = kf.bad_path() {
        return Err(Error::InvalidArgs(format!(
            "keyframes of run {} name `{bad}`",
            run.id
        )));
    }
    let dir = lib.root().join(id.rel_dir());
    let snapshot = run
        .inputs
        .first()
        .map(|i| i.snapshot.clone())
        .unwrap_or_default();
    let frames = kf
        .frames
        .iter()
        .filter_map(|f| {
            let rel = format!("derived/runs/{}/{}", run.id, f.path);
            let abs = dir.join(&rel);
            abs.is_file().then(|| Source {
                rel,
                abs,
                sha256: f.sha256.clone(),
                snapshot: snapshot.clone(),
                mime: Some("image/jpeg".into()),
                group: None,
            })
        })
        .collect();
    Ok(Some((run, frames)))
}

/// 一次跑完以后怎样。
enum Outcome {
    /// 写了一次转录（或者用不着做，例如配不上对、已经做好了）。
    Done {
        landed: bool,
    },
    WaitModel(String),
    Failed {
        failure: CallFailure,
        retry: bool,
    },
}

/// 转口播那一步：转写插件、抽音频的插件、原样的那个文件、中间 WAV 放哪、参数。
struct AsrStep<'a> {
    plugin: &'a TransformPlugin,
    media: &'a TransformPlugin,
    src: &'a Source,
    cache: &'a Path,
    options: Map<String, Value>,
}

/// 一次插件调用要的。
struct Call<'a> {
    plugin: &'a TransformPlugin,
    want: Produce,
    inputs: Vec<RunInput>,
    options: Map<String, Value>,
}

fn input(kind: ConsumeKind, s: &Source) -> RunInput {
    RunInput {
        kind,
        mime: s.mime.clone(),
        path: s.abs.clone(),
        snapshot: Some(s.snapshot.clone()),
        image_key: None,
        frame_sec: None,
        role: None,
    }
}

fn media_kind(s: &Sources, src: &Source) -> ConsumeKind {
    if s.audio.as_ref().is_some_and(|a| a.rel == src.rel) {
        ConsumeKind::Audio
    } else {
        ConsumeKind::Video
    }
}

fn record_input(s: &Source, via: Vec<usuyo_contract::ViaStep>) -> usuyo_contract::RunInput {
    usuyo_contract::RunInput {
        path: s.rel.clone(),
        sha256: s.sha256.clone(),
        snapshot: s.snapshot.clone(),
        via,
        extra: Default::default(),
    }
}

fn via_step(
    plugin: &TransformPlugin,
    op: &str,
    r: &RunResult,
    sha256: Option<String>,
) -> usuyo_contract::ViaStep {
    usuyo_contract::ViaStep {
        by: plugin.id().to_string(),
        version: plugin.header.version.to_string(),
        op: op.into(),
        tool: Some(usuyo_contract::Tool {
            name: r.tool.name.clone(),
            version: r.tool.version.clone(),
            extra: Default::default(),
        }),
        sha256,
        extra: Default::default(),
    }
}

/// 要写的一次转录：插件做的带着输出目录，宿主自己判的没有。
struct Landing {
    job: TransformJobKind,
    run_id: String,
    by: String,
    version: String,
    result: Option<RunResult>,
    out: Option<TransformOutput>,
    inputs: Vec<usuyo_contract::RunInput>,
    model: Option<usuyo_contract::ModelRef>,
    /// 宿主判的 `empty` 和标记（`no-audio-track`）。
    host_empty: Option<(String, Vec<usuyo_contract::Mark>)>,
}

impl Engine {
    pub(crate) fn transforms(&self) -> &Transforms {
        &self.inner.transforms
    }

    /// 自动排不排在这台：`transforms.auto` 开着，而且这条最早是在这台收下的（SPEC §1.2.1 的 `origins`）。
    fn transforms_here(&self, lib: &Library, m: &Manifest) -> bool {
        self.transforms().auto.load(Ordering::Relaxed)
            && m.origins
                .first()
                .is_some_and(|o| o.device == lib.device_id())
    }

    /// 一个快照落地以后（收下、再取一次、导入、交进来的）：排上还没做好的。`by_hand` 是你在窗口、终端里收的。
    /// 失败过的重新开始：新快照的输入可能不一样了。
    pub(crate) fn schedule_after_snapshot(
        &self,
        lib: &Library,
        id: &ItemId,
        by_hand: bool,
    ) -> Result<()> {
        let item = lib.item(id)?;
        if !self.transforms_here(lib, &item.manifest) {
            return Ok(());
        }
        let priority = if by_hand { BY_HAND } else { LATER };
        let now = now_ms();
        for job in not_done(lib, id, &item.manifest)? {
            self.inner
                .jobs
                .schedule_transform(id.as_str(), job, priority, "auto", true, now)?;
        }
        self.transforms().wake.notify_one();
        Ok(())
    }

    /// 你在窗口、终端里收过这一条没有（不是只经导入、扩展、手机交进来的）。
    pub(crate) fn taken_by_hand(m: &Manifest) -> bool {
        m.origins
            .iter()
            .any(|o| matches!(o.entry.as_str(), "paste" | "cli"))
    }

    /// 打开了这一条：还没做好的挪到最前，在电池上也转；`transforms.auto` 关着时也转（点名）。
    ///
    /// 调用方：WP8 的 `POST /items/{id}/transform {"jobs": "pending"}`（条目页打开时）。
    pub fn transform_now(&self, id: &str) -> Result<()> {
        let id = ItemId::parse(id)?;
        let lib = self.lib();
        let item = lib.item(&id)?;
        let now = now_ms();
        for job in not_done(&lib, &id, &item.manifest)? {
            self.inner
                .jobs
                .schedule_transform(id.as_str(), job, OPEN, "open", false, now)?;
        }
        drop(lib);
        self.transforms().wake.notify_one();
        Ok(())
    }

    /// 再转一次，或换一个插件、换一组参数转：哪怕做好了也再做一次，新的一次和旧的并排，`redoOf`
    /// 指现在最新的那一次。这一样正在跑回 `conflict`；插件做不了这一样回 `invalid-args`；
    /// 媒体已经移进回收站（只留文字）回 `unsupported-input`。
    ///
    /// 调用方：WP8 的 `POST /items/{id}/transform {job, plugin?, options?}`。
    pub fn redo_transform(
        &self,
        id: &str,
        job: &str,
        plugin: Option<&str>,
        options: Map<String, Value>,
    ) -> Result<()> {
        let id = ItemId::parse(id)?;
        let job = TransformJobKind::parse(job)
            .ok_or_else(|| Error::InvalidArgs(format!("no transform job `{job}`")))?;
        if self.transforms().plugin_for(job, plugin).is_none() {
            return Err(Error::InvalidArgs(match plugin {
                Some(p) => format!("{p} cannot do `{}`", job.as_str()),
                None => format!("no installed plugin can do `{}`", job.as_str()),
            }));
        }
        let lib = self.lib();
        let item = lib.item(&id)?;
        if item.manifest.state != ItemState::Normal {
            return Err(Error::UnsupportedInput(
                "the media of this item is in the trash".into(),
            ));
        }
        let redo_of = lib
            .job_runs(&id, job.as_str())?
            .into_iter()
            .next()
            .map(|r| r.id);
        drop(lib);
        let options = (!options.is_empty())
            .then(|| serde_json::to_string(&options))
            .transpose()?;
        let queued = self.inner.jobs.redo_transform(
            &TransformRedo {
                item_id: id.as_str(),
                job,
                plugin,
                options: options.as_deref(),
                redo_of: redo_of.as_deref(),
            },
            now_ms(),
        )?;
        if !queued {
            return Err(Error::Conflict(format!(
                "`{}` of {id} is running",
                job.as_str()
            )));
        }
        self.transforms().wake.notify_one();
        Ok(())
    }

    /// 这一条的转换任务（还没做好的那几样），给条目页那一面写一句（feature 003 §4.4 的表）。
    ///
    /// 调用方：WP7 的条目页（`pending`）。
    pub fn transform_jobs(&self, id: &str) -> Result<Vec<TransformJob>> {
        let id = ItemId::parse(id)?;
        self.inner.jobs.transforms_of(id.as_str())
    }

    /// 这台现在接没接着电源（读不出来算接着）。
    ///
    /// 调用方：WP7 的条目页（排着的那一面写「接上电源时会转」还是「会慢慢转」）。
    pub fn on_mains_power(&self) -> bool {
        self.transforms().on_mains()
    }

    /// 不再为这一条转：任务删掉，正在跑的取消。进回收站、只留文字、清掉时。
    pub(crate) fn stop_transforms(&self, id: &str) -> Result<()> {
        self.inner.jobs.drop_transforms(id)?;
        if let Some((item, token)) = self.transforms().running().as_ref()
            && item == id
        {
            token.cancel();
        }
        Ok(())
    }

    /// 启动时补排：在这台收下的、还有没做好的，按导入那一档排；已经有的一行不动。
    fn backfill_transforms(&self) -> Result<()> {
        let now = now_ms();
        // 上次缺插件、缺程序停下的：这次可能装上了。
        self.inner.jobs.revive_missing(now)?;
        let lib = self.lib();
        for id in lib.item_ids()? {
            let item = match lib.item(&id) {
                Ok(item) => item,
                Err(e) => {
                    tracing::warn!(item = %id, error = %e, "item not readable; not scheduled");
                    continue;
                }
            };
            if !self.transforms_here(&lib, &item.manifest) {
                continue;
            }
            match not_done(&lib, &id, &item.manifest) {
                Ok(jobs) => {
                    for job in jobs {
                        self.inner
                            .jobs
                            .backfill_transform(id.as_str(), job, LATER, now)?;
                    }
                }
                Err(e) => tracing::warn!(item = %id, error = %e, "not scheduled"),
            }
        }
        Ok(())
    }

    /// 转换那一路：一直跑。先补排，再一个一个做；没有到点的就睡，有新任务、模型到了时被叫醒。
    pub(crate) async fn run_transforms(&self) {
        let this = self.clone();
        match tokio::task::spawn_blocking(move || this.backfill_transforms()).await {
            Ok(Ok(())) => {}
            Ok(Err(e)) => tracing::error!(error = %e, "transform backfill"),
            Err(e) => tracing::error!(error = %e, "transform backfill"),
        }
        loop {
            if let Err(e) = self.run_transforms_due().await {
                tracing::error!(error = %e, "transform jobs");
            }
            let now = now_ms();
            let wait = match self.inner.jobs.next_transform_due_after(now) {
                Ok(Some(at)) => {
                    Duration::from_millis(u64::try_from(at - now).unwrap_or(0)).min(IDLE)
                }
                Ok(None) => IDLE,
                Err(e) => {
                    tracing::error!(error = %e, "jobs database");
                    IDLE
                }
            };
            tokio::select! {
                () = self.transforms().wake.notified() => {}
                () = tokio::time::sleep(wait) => {}
            }
        }
    }

    /// 把现在能做的转换一个一个做完，回做了几个。每轮先照电源和模型改一遍等着的。
    pub async fn run_transforms_due(&self) -> Result<usize> {
        let mut n = 0;
        loop {
            let on_mains = self.transforms().on_mains();
            self.inner.jobs.gate_power(on_mains)?;
            self.resume_present_models()?;
            let Some(row) = self.inner.jobs.next_transform(now_ms())? else {
                return Ok(n);
            };
            if !self.inner.jobs.claim_transform(&row.item_id, row.job)? {
                continue;
            }
            n += 1;
            // 条目页那一面改成「在转」。
            self.emit(Event::ItemUpdated {
                id: row.item_id.clone(),
            });
            self.transform_one(row).await?;
        }
    }

    /// 等着的模型现在在了（下好了，或者你自己放进去了）：等它的回到排着。
    fn resume_present_models(&self) -> Result<()> {
        let t = self.transforms();
        for name in self.inner.jobs.waited_models()? {
            let here = t.plugins.iter().any(|p| {
                p.spec
                    .models
                    .iter()
                    .any(|m| m.name == name && t.models.record(p.id(), m).is_some())
            });
            if here {
                for id in self.inner.jobs.model_arrived(&name, now_ms())? {
                    self.emit(Event::ItemUpdated { id });
                }
            }
        }
        Ok(())
    }

    /// 跑一行，照结局记账。资料库、任务库出的错往上交；插件的失败记在任务上。
    async fn transform_one(&self, row: TransformJob) -> Result<()> {
        let token = CancellationToken::new();
        *self.transforms().running() = Some((row.item_id.clone(), token.clone()));
        let outcome = self.attempt(&row, &token).await;
        *self.transforms().running() = None;
        let jobs = &self.inner.jobs;
        let outcome = match outcome {
            Ok(o) => o,
            Err(Error::Library(usuyo_library::Error::NotFound(_))) => {
                jobs.finish_transform(&row.item_id, row.job)?;
                return Ok(());
            }
            Err(e @ (Error::Library(_) | Error::Json(_))) => Outcome::Failed {
                failure: CallFailure::new(ErrorCode::Internal, e.to_string()),
                retry: false,
            },
            Err(e) => return Err(e),
        };
        if token.is_cancelled() {
            // 进了回收站、只留文字：这一行已经删了。
            return Ok(());
        }
        match outcome {
            Outcome::Done { landed } => {
                jobs.finish_transform(&row.item_id, row.job)?;
                if landed {
                    self.after_landing(&row)?;
                }
            }
            Outcome::WaitModel(name) => {
                tracing::info!(item = %row.item_id, job = row.job.as_str(), model = %name, "transform waits for a model");
                jobs.wait_for_model(&row.item_id, row.job, &name)?;
            }
            Outcome::Failed { failure, retry } => {
                let hint = failure
                    .hint
                    .clone()
                    .unwrap_or_else(|| failure_hint(failure.code).to_owned());
                let hint = Some(hint);
                tracing::info!(item = %row.item_id, job = row.job.as_str(), failure = %failure, "transform failed");
                let retry_policy = self.inner.retry;
                if retry && row.attempts + 1 < retry_policy.max_attempts {
                    let backoff = i64::try_from(retry_policy.delay(row.attempts).as_millis())
                        .unwrap_or(i64::MAX);
                    jobs.retry_transform(
                        &row.item_id,
                        row.job,
                        now_ms().saturating_add(backoff),
                        failure.code.as_str(),
                        hint.as_deref(),
                    )?;
                } else {
                    jobs.fail_transform(
                        &row.item_id,
                        row.job,
                        failure.code.as_str(),
                        hint.as_deref(),
                    )?;
                }
            }
        }
        // 那一面换成转出来的、等模型的、没转出来的，或者不再写什么。
        self.emit(Event::ItemUpdated { id: row.item_id });
        Ok(())
    }

    /// 一次转录落地以后：`keyframes` 的接着排 `frames`；重建这一条的索引。
    fn after_landing(&self, row: &TransformJob) -> Result<()> {
        let id = ItemId::parse(&row.item_id)?;
        if row.job == TransformJobKind::Keyframes {
            let trigger = if row.trigger == "redo" {
                "open"
            } else {
                row.trigger.as_str()
            };
            self.inner.jobs.schedule_transform(
                id.as_str(),
                TransformJobKind::Frames,
                row.priority,
                trigger,
                true,
                now_ms(),
            )?;
        }
        let lib = self.lib();
        self.reindex_item(&lib, &id)?;
        Ok(())
    }

    /// 做一行：看现在的输入，要做就调插件、写转录。
    async fn attempt(&self, row: &TransformJob, cancel: &CancellationToken) -> Result<Outcome> {
        let id = ItemId::parse(&row.item_id)?;
        let (item, s, frames) = {
            let lib = self.lib();
            let item = lib.item(&id)?;
            if item.manifest.state != ItemState::Normal {
                return Ok(Outcome::Done { landed: false });
            }
            let s = sources(&lib, &id, &item.manifest)?.unwrap_or_default();
            let frames = if row.job == TransformJobKind::Frames {
                newest_frames(&lib, &id)?
            } else {
                None
            };
            (item, s, frames)
        };
        let redo = row.trigger == "redo";
        let t = self.transforms();
        let Some(plugin) = t.plugin_for(row.job, row.plugin.as_deref()) else {
            return Ok(missing_plugin(
                format!("no installed plugin does `{}`", row.job.as_str()),
                row.job,
            ));
        };
        let options = t.options(plugin, row)?;
        let m = &item.manifest;
        let run_item = RunItem {
            id: id.to_string(),
            kind: m.kind.clone(),
            title: m.title.clone(),
            lang: m.lang.clone(),
        };
        let done = |needed: Vec<String>| -> Result<bool> {
            Ok(!redo && self.lib().job_done(&id, row.job.as_str(), &needed)?)
        };
        let run_id = usuyo_library::uuid7();
        let landing = |result: RunResult, out: TransformOutput, inputs, model| Landing {
            job: row.job,
            run_id: run_id.clone(),
            by: plugin.id().to_string(),
            version: plugin.header.version.to_string(),
            result: Some(result),
            out: Some(out),
            inputs,
            model,
            host_empty: None,
        };
        let landed = match row.job {
            TransformJobKind::Play => {
                let (Some(v), Some(a)) = (&s.video, &s.audio) else {
                    return Ok(Outcome::Done { landed: false });
                };
                if !s.may_pair() || done(vec![v.sha256.clone(), a.sha256.clone()])? {
                    return Ok(Outcome::Done { landed: false });
                }
                if s.grouped.is_none() {
                    let probed = match self
                        .probe(
                            &run_item,
                            &[(ConsumeKind::Video, v), (ConsumeKind::Audio, a)],
                            cancel,
                        )
                        .await?
                    {
                        Ok((p, _, _)) => p,
                        Err(o) => return Ok(o),
                    };
                    if !pairs(&probed[0], &probed[1]) {
                        return Ok(Outcome::Done { landed: false });
                    }
                }
                let mut vi = input(ConsumeKind::Video, v);
                vi.role = Some(InputRole::Video);
                let mut ai = input(ConsumeKind::Audio, a);
                ai.role = Some(InputRole::Audio);
                let call = Call {
                    plugin,
                    want: Produce::Play,
                    inputs: vec![vi, ai],
                    options,
                };
                match self.invoke(&run_item, call, cancel).await? {
                    Ok(out) => landing(
                        out.result.clone(),
                        out,
                        vec![record_input(v, Vec::new()), record_input(a, Vec::new())],
                        None,
                    ),
                    Err(o) => return Ok(o),
                }
            }
            TransformJobKind::Asr => {
                let Some(src) = s.speech().cloned() else {
                    return Ok(Outcome::Done { landed: false });
                };
                if done(vec![src.sha256.clone()])? {
                    return Ok(Outcome::Done { landed: false });
                }
                let kind = media_kind(&s, &src);
                if kind == ConsumeKind::Video {
                    let (probed, prober, result) =
                        match self.probe(&run_item, &[(kind, &src)], cancel).await? {
                            Ok(p) => p,
                            Err(o) => return Ok(o),
                        };
                    if !has_audio(&probed[0]) {
                        let via = via_step(&prober, "probe", &result, None);
                        return self
                            .land(&id, no_audio_track(row.job, run_id, &src, via))
                            .await;
                    }
                }
                if let Some(spec) = t.model_needed(plugin, &options)
                    && t.models.record(plugin.id(), spec).is_none()
                {
                    return Ok(Outcome::WaitModel(spec.name.clone()));
                }
                let Some(media) = t.helper(Produce::AudioPcm16k) else {
                    return Ok(missing_plugin(
                        "no installed plugin extracts audio".into(),
                        TransformJobKind::Play,
                    ));
                };
                let call = Call {
                    plugin: media,
                    want: Produce::AudioPcm16k,
                    inputs: vec![input(kind, &src)],
                    options: Map::new(),
                };
                let pcm = match self.invoke(&run_item, call, cancel).await? {
                    Ok(out) => out,
                    Err(o) => return Ok(o),
                };
                if pcm.result.empty {
                    let via = via_step(media, "audio:pcm16k", &pcm.result, None);
                    return self
                        .land(&id, no_audio_track(row.job, run_id, &src, via))
                        .await;
                }
                let cache = t.cache.join(id.as_str()).join(&run_id);
                let step = AsrStep {
                    plugin,
                    media,
                    src: &src,
                    cache: &cache,
                    options,
                };
                let chained = self.asr_from(&run_item, step, pcm, cancel).await;
                remove_cache(&cache, &t.cache.join(id.as_str()));
                match chained? {
                    Ok((out, via)) => {
                        let model = self.model_ref(plugin, &out.result);
                        landing(
                            out.result.clone(),
                            out,
                            vec![record_input(&src, vec![via])],
                            model,
                        )
                    }
                    Err(o) => return Ok(o),
                }
            }
            TransformJobKind::Keyframes => {
                let Some(v) = &s.video else {
                    return Ok(Outcome::Done { landed: false });
                };
                if done(vec![v.sha256.clone()])? {
                    return Ok(Outcome::Done { landed: false });
                }
                let call = Call {
                    plugin,
                    want: Produce::Keyframes,
                    inputs: vec![input(ConsumeKind::Video, v)],
                    options,
                };
                match self.invoke(&run_item, call, cancel).await? {
                    Ok(out) => landing(
                        out.result.clone(),
                        out,
                        vec![record_input(v, Vec::new())],
                        None,
                    ),
                    Err(o) => return Ok(o),
                }
            }
            TransformJobKind::Ocr => {
                let mut todo = Vec::new();
                for (key, img) in &s.images {
                    if !done(vec![img.sha256.clone()])? {
                        todo.push((key.clone(), img.clone()));
                    }
                }
                if todo.is_empty() {
                    return Ok(Outcome::Done { landed: false });
                }
                let inputs = todo
                    .iter()
                    .map(|(key, img)| RunInput {
                        image_key: Some(key.clone()),
                        ..input(ConsumeKind::Image, img)
                    })
                    .collect();
                let call = Call {
                    plugin,
                    want: Produce::Ocr,
                    inputs,
                    options,
                };
                match self.invoke(&run_item, call, cancel).await? {
                    Ok(out) => {
                        let rec = todo
                            .iter()
                            .map(|(_, i)| record_input(i, Vec::new()))
                            .collect();
                        landing(out.result.clone(), out, rec, None)
                    }
                    Err(o) => return Ok(o),
                }
            }
            TransformJobKind::Frames => {
                let Some((kf_run, frames)) = frames else {
                    return Ok(Outcome::Done { landed: false });
                };
                if frames.is_empty() || done(frames.iter().map(|f| f.sha256.clone()).collect())? {
                    return Ok(Outcome::Done { landed: false });
                }
                let secs = self.frame_secs(&id, &kf_run)?;
                let inputs = frames
                    .iter()
                    .map(|f| RunInput {
                        frame_sec: secs.get(&f.rel).copied(),
                        ..input(ConsumeKind::Keyframes, f)
                    })
                    .collect();
                let call = Call {
                    plugin,
                    want: Produce::Ocr,
                    inputs,
                    options,
                };
                match self.invoke(&run_item, call, cancel).await? {
                    Ok(out) => {
                        let rec = frames.iter().map(|f| record_input(f, Vec::new())).collect();
                        landing(out.result.clone(), out, rec, None)
                    }
                    Err(o) => return Ok(o),
                }
            }
        };
        if cancel.is_cancelled() {
            return Ok(Outcome::Done { landed: false });
        }
        self.land(&id, landed).await
    }

    /// 帧的相对路径 → 秒数，取自那次 `keyframes` 的 `keyframes.json`。
    fn frame_secs(&self, id: &ItemId, run: &RunRecord) -> Result<HashMap<String, f64>> {
        let lib = self.lib();
        let index = lib.run_output(id, &run.id, "keyframes")?;
        let bytes = std::fs::read(&index).map_err(|e| Error::io(&index, e))?;
        let kf: Keyframes = serde_json::from_slice(&bytes)?;
        Ok(kf
            .frames
            .iter()
            .map(|f| (format!("derived/runs/{}/{}", run.id, f.path), f.sec))
            .collect())
    }

    /// 让做 `probe` 的插件量一遍这些文件；回每个输入的流、是哪个插件量的和它的结果。
    async fn probe(
        &self,
        item: &RunItem,
        files: &[(ConsumeKind, &Source)],
        cancel: &CancellationToken,
    ) -> Result<std::result::Result<(Vec<Probe>, TransformPlugin, RunResult), Outcome>> {
        let Some(prober) = self.transforms().helper(Produce::Probe) else {
            return Ok(Err(missing_plugin(
                "no installed plugin probes media".into(),
                TransformJobKind::Play,
            )));
        };
        let call = Call {
            plugin: prober,
            want: Produce::Probe,
            inputs: files.iter().map(|(k, s)| input(*k, s)).collect(),
            options: Map::new(),
        };
        let out = match self.invoke(item, call, cancel).await? {
            Ok(out) => out,
            Err(o) => return Ok(Err(o)),
        };
        let mut probed = Vec::new();
        for i in 0..files.len() {
            match out.result.probe.iter().find(|p| p.input as usize == i) {
                Some(p) => probed.push(p.clone()),
                None => {
                    return Ok(Err(Outcome::Failed {
                        failure: CallFailure::new(
                            ErrorCode::InvalidOutput,
                            format!("probe has nothing for input {i}"),
                        ),
                        retry: false,
                    }));
                }
            }
        }
        Ok(Ok((probed, prober.clone(), out.result.clone())))
    }

    /// 16 kHz 的 WAV 挪进按机器缓存，再交给转写插件；回转写的输出和抽音频那一步的来历。
    async fn asr_from(
        &self,
        item: &RunItem,
        step: AsrStep<'_>,
        pcm: TransformOutput,
        cancel: &CancellationToken,
    ) -> Result<std::result::Result<(TransformOutput, usuyo_contract::ViaStep), Outcome>> {
        let AsrStep {
            plugin,
            media,
            src,
            cache,
            options,
        } = step;
        let Some(product) = pcm
            .result
            .outputs
            .iter()
            .find(|o| o.kind == Produce::AudioPcm16k)
        else {
            return Ok(Err(Outcome::Failed {
                failure: CallFailure::new(ErrorCode::InvalidOutput, "no audio:pcm16k output"),
                retry: false,
            }));
        };
        let from = pcm.out_dir().join(product.path.as_str());
        let wav = cache.join("audio.wav");
        let (f, w, c) = (from.clone(), wav.clone(), cache.to_owned());
        let sha256 = tokio::task::spawn_blocking(move || -> Result<String> {
            std::fs::create_dir_all(&c).map_err(|e| Error::io(&c, e))?;
            move_file(&f, &w)?;
            let bytes = std::fs::read(&w).map_err(|e| Error::io(&w, e))?;
            Ok(usuyo_library::sha256_hex(&bytes))
        })
        .await
        .map_err(|e| Error::Task(e.to_string()))??;
        let via = via_step(media, "audio:pcm16k", &pcm.result, Some(sha256));
        drop(pcm);
        let call = Call {
            plugin,
            want: Produce::Asr,
            inputs: vec![RunInput {
                kind: ConsumeKind::AudioPcm16k,
                mime: Some("audio/wav".into()),
                path: wav,
                snapshot: Some(src.snapshot.clone()),
                image_key: None,
                frame_sec: None,
                role: None,
            }],
            options,
        };
        Ok(self.invoke(item, call, cancel).await?.map(|out| (out, via)))
    }

    /// 调一次插件。插件的失败变成 [`Outcome`]：缺模型的等模型，别的按能不能再试分。
    async fn invoke(
        &self,
        item: &RunItem,
        call: Call<'_>,
        cancel: &CancellationToken,
    ) -> Result<std::result::Result<TransformOutput, Outcome>> {
        let resident = self.transforms().resident(call.plugin)?;
        let request = TransformRequest {
            item: item.clone(),
            want: vec![call.want],
            inputs: call.inputs,
            options: call.options,
            provider: None,
        };
        match resident.run(request, cancel).await {
            Ok(out) => Ok(Ok(out)),
            Err(PluginError::Call { failure, .. }) => {
                if failure.code == ErrorCode::DependencyMissing
                    && let Some(name) = failure
                        .needs
                        .iter()
                        .find_map(|n| n.as_str().strip_prefix("model:"))
                {
                    return Ok(Err(Outcome::WaitModel(name.to_owned())));
                }
                let retry = !matches!(
                    failure.code,
                    ErrorCode::InvalidArgs
                        | ErrorCode::UnsupportedInput
                        | ErrorCode::NotFound
                        | ErrorCode::DependencyMissing
                        | ErrorCode::Cancelled
                );
                Ok(Err(Outcome::Failed {
                    failure: *failure,
                    retry,
                }))
            }
            Err(e @ PluginError::Unsupported { .. }) => Ok(Err(Outcome::Failed {
                failure: CallFailure::new(ErrorCode::UnsupportedInput, e.to_string()),
                retry: false,
            })),
            Err(e) => Ok(Err(Outcome::Failed {
                failure: CallFailure::new(ErrorCode::Internal, e.to_string()),
                retry: true,
            })),
        }
    }

    /// 用了模型时记名字和宿主核过的 sha256（`model.json` 里的）。
    fn model_ref(
        &self,
        plugin: &TransformPlugin,
        r: &RunResult,
    ) -> Option<usuyo_contract::ModelRef> {
        let used = r.model.as_ref()?;
        let spec = plugin.spec.models.iter().find(|m| m.name == used.name)?;
        let record = self.transforms().models.record(plugin.id(), spec)?;
        Some(usuyo_contract::ModelRef {
            name: record.name,
            sha256: record.sha256,
            extra: Default::default(),
        })
    }

    /// 写一次转录：输出目录挪进资料库的暂存，算好哈希，拿着锁核对条目还在、还能收这一样，再写。
    async fn land(&self, id: &ItemId, l: Landing) -> Result<Outcome> {
        let this = self.clone();
        let id = id.clone();
        tokio::task::spawn_blocking(move || this.land_blocking(&id, l))
            .await
            .map_err(|e| Error::Task(e.to_string()))?
    }

    fn land_blocking(&self, id: &ItemId, l: Landing) -> Result<Outcome> {
        let staged = self.lib().staging_dir()?;
        if let Some(out) = &l.out {
            for f in &out.files {
                move_file(
                    &out.out_dir().join(f.path.as_str()),
                    &staged.join(f.path.as_str()),
                )?;
            }
        }
        let files = hash_run_products(&staged)?;
        let size_of = |path: &str| files.iter().find(|f| f.path == path);
        let (outputs, marks, empty, reason, options, tool) = match (&l.result, l.host_empty) {
            (Some(r), _) => {
                let mut outputs = Vec::new();
                for p in &r.outputs {
                    let f = size_of(p.path.as_str()).ok_or_else(|| {
                        Error::InvalidArgs(format!("output `{}` is missing", p.path.as_str()))
                    })?;
                    outputs.push(usuyo_contract::RunOutput {
                        path: f.path.clone(),
                        kind: p.kind.as_str().to_owned(),
                        size: f.size,
                        sha256: f.sha256.clone(),
                        lang: p.lang.clone(),
                        extra: Default::default(),
                    });
                }
                let marks: Vec<usuyo_contract::Mark> =
                    serde_json::from_value(serde_json::to_value(&r.marks)?)?;
                let reason = r
                    .reason
                    .map(serde_json::to_value)
                    .transpose()?
                    .and_then(|v| v.as_str().map(str::to_owned));
                let tool = usuyo_contract::Tool {
                    name: r.tool.name.clone(),
                    version: r.tool.version.clone(),
                    extra: Default::default(),
                };
                (
                    outputs,
                    marks,
                    r.empty,
                    reason,
                    r.options.clone(),
                    Some(tool),
                )
            }
            (None, Some((reason, marks))) => {
                (Vec::new(), marks, true, Some(reason), Map::new(), None)
            }
            (None, None) => {
                return Err(Error::InvalidArgs("a run needs a result".into()));
            }
        };
        let mut lib = self.lib();
        let item = lib.item(id)?;
        // 跑着的时候只留文字了：带图和视频的那几样不再落进来。
        if item.manifest.state != ItemState::Normal && writes_binaries(l.job) {
            drop(lib);
            if let Err(e) = std::fs::remove_dir_all(&staged) {
                tracing::warn!(path = %staged.display(), error = %e, "staged run not removed");
            }
            return Ok(Outcome::Done { landed: false });
        }
        let row = self.inner.jobs.transforms_of(id.as_str())?;
        let row = row.iter().find(|r| r.job == l.job);
        let record = RunRecord {
            id: l.run_id,
            job: l.job.as_str().to_owned(),
            by: l.by,
            version: l.version,
            tool,
            model: l.model,
            options,
            at: usuyo_library::time::now(),
            device: lib.device_id().to_owned(),
            trigger: row.map_or("auto", |r| r.trigger.as_str()).to_owned(),
            redo_of: row.and_then(|r| r.redo_of.clone()),
            inputs: l.inputs,
            outputs,
            empty,
            reason,
            marks,
            extra: Default::default(),
        };
        lib.write_run(
            id,
            NewRun {
                staged,
                files,
                record,
            },
        )?;
        Ok(Outcome::Done { landed: true })
    }

    /// 下这个模型（只在你点了以后）：马上回，在后台下；下好以后等它的任务接着转。
    /// 回 `true` 是已经在了。插件或模型不在回 `not-found`。
    ///
    /// 调用方：WP8 的 `POST /transforms/{owner}/{repo}/models/{name}/download`、条目页的「下载」。
    pub fn download_model(&self, plugin: &str, name: &str) -> Result<bool> {
        let (p, spec) = self.model_spec(plugin, name)?;
        match self.transforms().models.begin(p.id(), &spec) {
            Begin::Present => return Ok(true),
            Begin::Downloading => return Ok(false),
            Begin::Started => {}
        }
        self.emit(Event::TransformsUpdated);
        let this = self.clone();
        tokio::spawn(async move {
            let t = this.transforms();
            let result = t.models.fetch(p.id(), &spec).await;
            t.models.end(p.id(), &spec, &result);
            match &result {
                Ok(()) => {
                    tracing::info!(plugin = %p.id(), model = %spec.name, "model downloaded");
                    // 进程启动时报过手上有哪些模型；换一个进程，让它重新看。
                    if let Ok(r) = t.resident(&p) {
                        r.shutdown().await;
                    }
                    match this.inner.jobs.model_arrived(&spec.name, now_ms()) {
                        Ok(ids) => {
                            for id in ids {
                                this.emit(Event::ItemUpdated { id });
                            }
                        }
                        Err(e) => tracing::error!(error = %e, "jobs database"),
                    }
                    t.wake.notify_one();
                }
                Err(e) => {
                    tracing::warn!(plugin = %p.id(), model = %spec.name, error = %e, "model not downloaded")
                }
            }
            this.emit(Event::TransformsUpdated);
        });
        Ok(false)
    }

    /// 删掉这个模型（按机器目录里的，不是资料库）；之后要它的任务等模型。正在下回 `conflict`。
    /// 插件的进程在后台关掉（正在转的那一条转完再关），下次起来就看不到这个模型了。
    ///
    /// 调用方：`POST /transforms/{owner}/{repo}/models/{name}/remove`。
    pub fn remove_model(&self, plugin: &str, name: &str) -> Result<bool> {
        let (p, spec) = self.model_spec(plugin, name)?;
        let removed = self.transforms().models.remove(p.id(), &spec)?;
        let this = self.clone();
        tokio::spawn(async move {
            if let Ok(r) = this.transforms().resident(&p) {
                r.shutdown().await;
            }
        });
        self.emit(Event::TransformsUpdated);
        Ok(removed)
    }

    fn model_spec(&self, plugin: &str, name: &str) -> Result<(TransformPlugin, ModelSpec)> {
        let p = self
            .transforms()
            .plugins
            .iter()
            .find(|p| p.id().as_str() == plugin)
            .ok_or_else(|| Error::NotFound(format!("transform plugin {plugin}")))?;
        let spec = p
            .spec
            .models
            .iter()
            .find(|m| m.name == name)
            .ok_or_else(|| Error::NotFound(format!("model {name} of {plugin}")))?;
        Ok((p.clone(), spec.clone()))
    }

    /// 装着的转换插件、各做哪几样、自己报的能不能用、模型和它们现在怎样，和 `transforms.auto`。
    /// 能不能用要问插件（没起着的会起一个）；正在转一条的不等它，算能用。
    ///
    /// 调用方：`GET /transforms`、获取插件页的转换插件和模型那一栏。
    pub async fn transforms_view(&self) -> crate::view::TransformsView {
        use crate::view::{Bundled, Health, ModelView, TransformPluginView};
        let t = self.transforms();
        // 几个插件同时问，一个起得慢不拖住别的。
        let asks: Vec<_> = t
            .plugins
            .iter()
            .map(|p| {
                let (this, p) = (self.clone(), p.clone());
                tokio::spawn(async move { this.health_of(&p).await })
            })
            .collect();
        let mut plugins = Vec::new();
        for (p, ask) in t.plugins.iter().zip(asks) {
            let answer = ask
                .await
                .map_err(|e| Error::Task(e.to_string()))
                .and_then(|r| r);
            let (health, hint) = match answer {
                Ok(h) => h,
                Err(e) => {
                    tracing::warn!(plugin = %p.id(), error = %e, "transform plugin health");
                    (Health::Unavailable, Some("这个插件起不来".to_owned()))
                }
            };
            plugins.push(TransformPluginView {
                id: p.id().to_string(),
                name: p.header.name.clone(),
                version: p.header.version.to_string(),
                jobs: TransformJobKind::ALL
                    .into_iter()
                    .filter(|j| t.plugin_for(*j, Some(p.id().as_str())).is_some())
                    .map(|j| j.as_str().to_owned())
                    .collect(),
                bundled: p
                    .header
                    .upstream
                    .iter()
                    .map(|u| Bundled {
                        name: u.name.clone(),
                        license: u.license.clone(),
                    })
                    .collect(),
                models: p
                    .spec
                    .models
                    .iter()
                    .map(|m| {
                        let state = t.models.state(p.id(), m);
                        let hint = match &state {
                            crate::ModelState::Failed(h) => Some((*h).to_owned()),
                            _ => None,
                        };
                        ModelView {
                            name: m.name.clone(),
                            size: m.size,
                            license: m.license.clone(),
                            state,
                            hint,
                        }
                    })
                    .collect(),
                health,
                hint,
            });
        }
        crate::view::TransformsView {
            plugins,
            auto: t.auto.load(Ordering::Relaxed),
        }
    }

    /// 插件的 `health`：没过的检查的话（插件给的）当 `hint`。
    async fn health_of(
        &self,
        p: &TransformPlugin,
    ) -> Result<(crate::view::Health, Option<String>)> {
        use crate::view::Health;
        let Some(h) = self.transforms().resident(p)?.health_if_idle().await else {
            return Ok((Health::Ok, None));
        };
        let h = h?;
        if h.ready {
            return Ok((Health::Ok, None));
        }
        // 缺的是宿主管的模型：模型那一栏已经写着、给着「下载」，这里不转述插件的英文（里面有路径）。
        let t = self.transforms();
        if p.spec
            .models
            .iter()
            .any(|m| t.models.record(p.id(), m).is_none())
        {
            return Ok((Health::Unavailable, Some("要先下载模型".to_owned())));
        }
        let why: Vec<String> = h
            .checks
            .iter()
            .filter(|c| !c.ok)
            .map(|c| c.message.clone().unwrap_or_else(|| c.name.clone()))
            .collect();
        Ok((
            Health::Unavailable,
            (!why.is_empty()).then(|| why.join("；")),
        ))
    }

    /// 打开或关上 `transforms.auto`：关上时自动排的（没在跑的）都删掉，之后只在打开条目、再转一次时转；
    /// 打开时照启动那样补排。调用方：设置（`PATCH /settings`）。
    pub(crate) fn set_transforms_auto(&self, on: bool) -> Result<()> {
        let was = self.transforms().auto.swap(on, Ordering::Relaxed);
        if was == on {
            return Ok(());
        }
        if on {
            self.backfill_transforms()?;
            self.transforms().wake.notify_one();
        } else {
            for id in self.inner.jobs.drop_automatic()? {
                self.emit(Event::ItemUpdated { id });
            }
        }
        Ok(())
    }

    /// 条目页那一面写的一句（feature 003 §4.4 的表）：每一面一句，挑最要紧的那一行
    /// （在转、等模型、没转出来、等电源、排着）。`play`、`keyframes` 都算画面那一面。
    pub(crate) fn pending(&self, id: &str) -> Result<Vec<crate::view::Pending>> {
        use crate::view::{Pending, PendingModel, PendingState};
        let rows = self.inner.jobs.transforms_of(id)?;
        if rows.is_empty() {
            return Ok(Vec::new());
        }
        let on_mains = self.on_mains_power();
        let t = self.transforms();
        let mut out: Vec<Pending> = Vec::new();
        for r in rows {
            let layer = match r.job {
                TransformJobKind::Play | TransformJobKind::Keyframes => "video",
                TransformJobKind::Asr => "asr",
                TransformJobKind::Ocr => "ocr",
                TransformJobKind::Frames => "frames",
            };
            let mut model = None;
            let state = match r.state {
                TransformState::Running => PendingState::Now,
                TransformState::Failed => PendingState::Failed,
                TransformState::Waiting => {
                    match r.wait.as_deref().and_then(|w| w.strip_prefix("model:")) {
                        Some(name) => {
                            model = t.plugins.iter().find_map(|p| {
                                let m = p.spec.models.iter().find(|m| m.name == name)?;
                                Some(PendingModel {
                                    plugin: p.id().to_string(),
                                    name: m.name.clone(),
                                    size: m.size,
                                    license: m.license.clone(),
                                })
                            });
                            PendingState::NeedsModel
                        }
                        None => PendingState::OnPower,
                    }
                }
                TransformState::Queued if !on_mains && r.priority > OPEN => PendingState::OnPower,
                TransformState::Queued => PendingState::Later,
            };
            let hint = (state == PendingState::Failed)
                .then_some(r.error_hint)
                .flatten();
            let p = Pending {
                layer,
                state,
                model,
                hint,
            };
            match out.iter_mut().find(|x| x.layer == layer) {
                Some(x) if p.state < x.state => *x = p,
                Some(_) => {}
                None => out.push(p),
            }
        }
        let rank = |l: &str| {
            ["video", "asr", "ocr", "frames"]
                .iter()
                .position(|x| *x == l)
        };
        out.sort_by_key(|p| rank(p.layer));
        Ok(out)
    }

    /// 守护进程退出时：关掉所有转换插件的进程（关 stdin，再 SIGTERM、SIGKILL 进程组）。
    ///
    /// 调用方：`usuyo-cli` 的 `daemon`（接口停下以后）。
    pub async fn shutdown(&self) {
        if let Some((_, token)) = self.transforms().running().as_ref() {
            token.cancel();
        }
        for r in self.transforms().resident.values() {
            r.shutdown().await;
        }
    }
}

/// `no-audio-track`：画面里没有音频流，宿主自己写一次 `asr` 转录（feature 003 §5.2）。
fn no_audio_track(
    job: TransformJobKind,
    run_id: String,
    src: &Source,
    via: usuyo_contract::ViaStep,
) -> Landing {
    Landing {
        job,
        run_id,
        by: "host".into(),
        version: env!("CARGO_PKG_VERSION").into(),
        result: None,
        out: None,
        inputs: vec![record_input(src, vec![via])],
        model: None,
        host_empty: Some((
            "no-audio-track".into(),
            vec![usuyo_contract::Mark {
                code: "no-audio-track".into(),
                speech_sec: None,
                duration_sec: None,
                frozen_sec: None,
                extra: Default::default(),
            }],
        )),
    }
}

/// 这台电脑上没有能做这一样的插件（或做中间一步的插件）：不再自动试，启动时再看装了没有。
fn missing_plugin(message: String, job: TransformJobKind) -> Outcome {
    let mut failure = CallFailure::new(ErrorCode::DependencyMissing, message);
    failure.hint = Some(
        match job {
            TransformJobKind::Asr => "这台电脑上还没有能转口播的插件",
            TransformJobKind::Ocr | TransformJobKind::Frames => {
                "这台电脑上还没有能认图里的字的插件"
            }
            TransformJobKind::Play | TransformJobKind::Keyframes => {
                "这台电脑上还没有处理音视频的插件"
            }
        }
        .into(),
    );
    Outcome::Failed {
        failure,
        retry: false,
    }
}

/// 插件没给 `hint` 时，条目页那一面「没转出来：」后面写的。插件的 `message` 是英文、给日志的。
fn failure_hint(code: ErrorCode) -> &'static str {
    match code {
        ErrorCode::Timeout => "转得太久，停下了",
        ErrorCode::Crashed => "转换插件中途退出了",
        ErrorCode::InvalidOutput => "转换插件交回来的东西不对",
        ErrorCode::UnsupportedInput => "这个文件转不了",
        ErrorCode::DependencyMissing => "转换插件缺了要用的东西",
        _ => "转换插件出了错",
    }
}

/// 改名过去；跨文件系统时复制再删。
fn move_file(from: &Path, to: &Path) -> Result<()> {
    if let Some(parent) = to.parent() {
        std::fs::create_dir_all(parent).map_err(|e| Error::io(parent, e))?;
    }
    match std::fs::rename(from, to) {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == std::io::ErrorKind::CrossesDevices => {
            std::fs::copy(from, to).map_err(|e| Error::io(to, e))?;
            std::fs::remove_file(from).map_err(|e| Error::io(from, e))
        }
        Err(e) => Err(Error::io(to, e)),
    }
}

/// 一条链跑完：中间产物删掉，这一条的缓存文件夹空了也删掉。
fn remove_cache(dir: &Path, item_dir: &Path) {
    match std::fs::remove_dir_all(dir) {
        Ok(()) => {}
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
        Err(e) => tracing::warn!(path = %dir.display(), error = %e, "transform cache not removed"),
    }
    // 还有别的在用时删不掉，不算错。
    let _ = std::fs::remove_dir(item_dir);
}

#[cfg(test)]
mod tests {
    use super::*;
    use usuyo_contract::{FileHash, MediaFile};
    use usuyo_plugin::protocol::{MediaKind, ProbeStream, Seconds};

    fn probe(streams: &[(MediaKind, &str)], dur: f64) -> Probe {
        Probe {
            input: 0,
            duration_sec: Some(Seconds::try_from(dur).unwrap()),
            streams: streams
                .iter()
                .map(|(k, c)| ProbeStream {
                    kind: *k,
                    codec: Some((*c).into()),
                })
                .collect(),
        }
    }

    #[test]
    fn pairing_rule() {
        let v = |d| probe(&[(MediaKind::Video, "h264")], d);
        let a = |d| probe(&[(MediaKind::Audio, "aac")], d);
        assert!(pairs(&v(7.52), &a(7.43)));
        assert!(pairs(&v(10.0), &a(11.0)), "刚好差 1 秒");
        assert!(!pairs(&v(10.0), &a(11.01)), "刚好超出");
        assert!(pairs(&v(100.0), &a(102.0)), "长的按 2%");
        assert!(!pairs(&v(100.0), &a(102.1)));
        let with_sound = probe(
            &[(MediaKind::Video, "h264"), (MediaKind::Audio, "aac")],
            7.5,
        );
        assert!(!pairs(&with_sound, &a(7.5)), "画面自己带声音的不配");
        let cover = probe(
            &[(MediaKind::Audio, "aac"), (MediaKind::Video, "mjpeg")],
            7.5,
        );
        assert!(pairs(&v(7.5), &cover), "音频里的封面图不算画面");
        let film = probe(
            &[(MediaKind::Audio, "aac"), (MediaKind::Video, "h264")],
            7.5,
        );
        assert!(!pairs(&v(7.5), &film));
    }

    fn media(path: &str, kind: &str, group: Option<&str>) -> MediaFile {
        MediaFile {
            path: path.into(),
            kind: kind.into(),
            group: group.map(str::to_owned),
            lang: None,
            extra: Default::default(),
        }
    }

    fn record(media_files: Vec<MediaFile>, dir: &Path) -> FetchRecord {
        let mut rec = FetchRecord {
            media: media_files,
            ..FetchRecord::default()
        };
        for m in &rec.media {
            let p = dir.join("raw/s").join(&m.path);
            std::fs::create_dir_all(p.parent().unwrap()).unwrap();
            std::fs::write(&p, &m.path).unwrap();
        }
        rec.files = rec
            .media
            .iter()
            .map(|m| FileHash {
                path: m.path.clone(),
                size: 1,
                sha256: usuyo_library::sha256_hex(m.path.as_bytes()),
            })
            .collect();
        rec
    }

    #[test]
    fn group_first_then_the_content_rule() {
        let tmp = tempfile::tempdir().unwrap();
        let d = tmp.path();
        let s = sources_of(
            &record(
                vec![media("v.m4s", "video", None), media("a.m4s", "audio", None)],
                d,
            ),
            d,
            "s",
        );
        assert!(s.may_pair() && s.grouped.is_none(), "没写 group：量过再说");
        assert_eq!(s.speech().unwrap().rel, "raw/s/a.m4s", "口播用分开的音轨");
        let s = sources_of(
            &record(
                vec![
                    media("v.m4s", "video", Some("g")),
                    media("a.m4s", "audio", Some("g")),
                ],
                d,
            ),
            d,
            "s",
        );
        assert_eq!(s.grouped, Some(true));
        let s = sources_of(
            &record(
                vec![
                    media("v.m4s", "video", Some("g")),
                    media("a.m4s", "audio", Some("h")),
                ],
                d,
            ),
            d,
            "s",
        );
        assert!(!s.may_pair(), "group 不一样的不配");
        let s = sources_of(
            &record(
                vec![
                    media("v1.mp4", "video", None),
                    media("v2.mp4", "video", None),
                    media("a.m4a", "audio", None),
                ],
                d,
            ),
            d,
            "s",
        );
        assert!(!s.may_pair(), "两个画面一个音轨不配");
        let s = sources_of(&record(vec![media("v.mp4", "video", None)], d), d, "s");
        assert!(!s.may_pair());
        assert_eq!(
            s.speech().unwrap().rel,
            "raw/s/v.mp4",
            "没有分开的音轨就看画面里的"
        );
    }
}
