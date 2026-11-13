//! 转换插件的常驻进程（协议 §4.1、§4.2、§7.1；feature 003 §4.2）。
//!
//! 一个插件一个 [`ResidentTransform`]。第一次要用时起进程、核 `describe`；之后同一进程一次只发一个
//! `run`；超时、卡住、退出、发来坏消息就关掉整个进程组，下次要用再起；闲了 `idle` 也关掉，
//! 放掉模型占的内存。关的顺序：关 stdin，等 `stop`；再 SIGTERM 进程组，等 `grace`；最后 SIGKILL 进程组。
//!
//! 进程发来的每一行都不可信：消息在这里解析、核对一次，结果核对 `want`、清单、输出目录，
//! 往里交的是校验过的 [`RunResult`] 和输出目录里的文件。宿主自己拼的错误只写相对这次调用的名字；
//! 插件给的字里的令牌和几个目录换成占位符。

use std::collections::{BTreeSet, VecDeque};
use std::io;
use std::path::{Path, PathBuf};
use std::process::{ExitStatus, Stdio};
use std::sync::{Arc, Mutex as StdMutex, PoisonError, Weak};
use std::time::Duration;

use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::{Map, Value};
use tokio::io::{AsyncBufRead, AsyncBufReadExt, AsyncRead, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, ChildStdin, Command};
use tokio::sync::{Mutex, mpsc};
use tokio::task::JoinHandle;
use tokio::time::{Instant, sleep_until, timeout, timeout_at};
use tokio_util::sync::CancellationToken;

use super::outdir::{self, OutFile};
use super::process::stop_group;
use super::{PluginDirs, TransformPlugin, credentials, envelope_text, plugin_env, resolve_command};
use crate::PluginError;
use crate::protocol::{
    CallFailure, Describe, ErrorCode, Health, METHOD_CANCEL, METHOD_DESCRIBE, METHOD_HEALTH,
    METHOD_PROGRESS, METHOD_RUN, Produce, Progress, ProviderGrant, ProviderNeed, RpcError, RpcId,
    RunInput, RunItem, RunParams, RunResult, SUPPORTED_PROTOCOLS,
};

/// stderr 一行最多留多少字节，免得没有换行的输出撑爆内存。
const STDERR_LINE_BYTES: usize = 8 * 1024;
/// 读 stdout 的任务最多攒几条没取走的消息；满了它就不读，插件写 stdout 会停住。
const INCOMING_BACKLOG: usize = 64;

/// 常驻进程的各种计时和上限。默认值照协议 §3.4、§4.1；测试把它们调到毫秒。
#[derive(Debug, Clone)]
pub struct TransformLimits {
    /// 起进程到 `describe` 答回来。
    pub describe: Duration,
    pub health: Duration,
    /// 清单没写 `permissions.limits.timeout` 时一次 `run` 的总时长。
    pub run: Duration,
    /// 一次 `run` 发过 `progress` 以后，多久没有新的算卡住（协议 §3.4）；从没发过的只受总时长约束。
    pub stuck: Duration,
    /// 多久没有调用就关掉进程（协议 §4.1）。
    pub idle: Duration,
    /// 关 stdin 以后等进程自己退出多久。
    pub stop: Duration,
    /// SIGTERM 到 SIGKILL 的宽限；发了 `cancel` 以后等那个 `run` 回答也等这么久。
    pub grace: Duration,
    /// 一条消息最多多少字节（不算换行），两个方向都按它；plugin-kit 收的上限是 1 MiB。
    pub frame_bytes: usize,
    /// 报错时带上 stderr 的最后几行。
    pub stderr_lines: usize,
    /// 一次 `run` 的输出目录总大小上限。
    pub out_bytes: u64,
}

impl Default for TransformLimits {
    fn default() -> Self {
        Self {
            describe: Duration::from_secs(10),
            health: Duration::from_secs(10),
            run: Duration::from_secs(3600),
            stuck: Duration::from_secs(120),
            idle: Duration::from_secs(120),
            stop: Duration::from_secs(5),
            grace: Duration::from_secs(5),
            frame_bytes: 1 << 20,
            stderr_lines: 200,
            out_bytes: 8 << 30,
        }
    }
}

#[derive(Debug, Clone)]
pub struct TransformConfig {
    /// 进程的暂存目录 `usuyo-proc-<UUIDv7>-<随机>/` 建在这里。和资料库在同一个文件系统上，
    /// 输入才能硬链接进去、输出才能改名移走。
    pub work_dir: PathBuf,
    /// `${CACHE}` 的上级，按插件 id 分子目录。
    pub cache_dir: PathBuf,
    /// `${MODELS}` 的上级，按插件 id 分子目录（`<models_dir>/<owner>/<repo>/`），经 `USUYO_MODELS_DIR`
    /// 交给进程。模型由调用方下好放进去，这里不建、不下。
    pub models_dir: PathBuf,
    /// 用户界面语言（`zh-Hans`、`en`），和获取插件共用一个，设置里换了马上换。
    pub locale: super::Locale,
    pub limits: TransformLimits,
}

/// 一次 `run` 要的（协议 §4.2）。`out` 由宿主定。
#[derive(Debug, Clone)]
pub struct TransformRequest {
    pub item: RunItem,
    /// 这次要哪几种产出：清单 `produces` 的子集，不为空、不重复。
    pub want: Vec<Produce>,
    /// `path` 是资料库里的文件（绝对路径）。宿主把它链接进 `in/<callId>/`，交给插件的是那里的路径。
    pub inputs: Vec<RunInput>,
    pub options: Map<String, Value>,
    pub provider: Option<ProviderGrant>,
}

/// 一次成功的 `run`：校验过的结果和输出目录。
///
/// 输出目录在插件回答以后就从 `out/<callId>/` 改名挪到工作目录里一个只有宿主知道的地方
/// （`<work_dir>/<UUIDv7>/`，不在进程的暂存目录里），核对的和交出去的都是挪过的那一份：
/// 回答以后还在跑的插件照它知道的路径写，写不进来。
/// 调用方（WP5 的转录写入）把 [`out_dir`](Self::out_dir) 整个改名移进转录文件夹；这个值丢掉时，
/// 目录里剩下的一起删掉。进程关了、换了，它照样在。
#[derive(Debug)]
pub struct TransformOutput {
    pub result: RunResult,
    /// 输出目录里的全部文件，按路径排好；每个都是 `result.outputs` 报过的，或 `keyframes/` 里的帧。
    pub files: Vec<OutFile>,
    out: HostDir,
}

impl TransformOutput {
    pub fn out_dir(&self) -> &Path {
        &self.out.path
    }
}

/// 一个转换插件的常驻进程。
///
/// WP6（`usuyo-core` 排任务、编排一条链）每个插件拿一个，`run` 一条一条地发，用取消令牌取消；
/// 进程什么时候起、什么时候关由这里管，调用方不管。几个调用一起来时排着，一个进程同一时间只有一个
/// `run`（协议 §4.1）。丢掉它时进程组被 SIGKILL。
pub struct ResidentTransform {
    inner: Arc<Inner>,
}

struct Inner {
    plugin: TransformPlugin,
    config: TransformConfig,
    state: Mutex<State>,
}

struct State {
    live: Option<Live>,
    /// 下一个 JSON-RPC `id`，也是 `in/<callId>/`、`out/<callId>/` 的名字；换了进程也接着往上数。
    next_id: i64,
    /// 起过几个进程。闲置计时只管起它的那一个。
    generation: u64,
    last_used: Instant,
}

impl State {
    fn take_id(&mut self) -> RpcId {
        let id = self.next_id;
        self.next_id += 1;
        RpcId::Number(id)
    }
}

impl ResidentTransform {
    /// 不起进程；第一次 [`run`](Self::run) 或 [`health`](Self::health) 时才起。
    pub fn new(plugin: TransformPlugin, config: TransformConfig) -> Self {
        Self {
            inner: Arc::new(Inner {
                plugin,
                config,
                state: Mutex::new(State {
                    live: None,
                    next_id: 1,
                    generation: 0,
                    last_used: Instant::now(),
                }),
            }),
        }
    }

    pub fn plugin(&self) -> &TransformPlugin {
        &self.inner.plugin
    }

    /// 问插件能不能接 `run`（协议 §3.10）：模型在不在、GPU 能不能用。`ready: false` 不是错误。
    pub async fn health(&self) -> Result<Health, PluginError> {
        let mut state = self.inner.state.lock().await;
        self.health_on(&mut state).await
    }

    /// 同 [`health`](Self::health)，但这个进程正在跑一条 `run` 时不等，回 `None`（它在干活）。
    /// 调用方是 `usuyo-core` 的 `GET /transforms`：一条转写可能跑好几分钟。
    pub async fn health_if_idle(&self) -> Option<Result<Health, PluginError>> {
        let mut state = self.inner.state.try_lock().ok()?;
        Some(self.health_on(&mut state).await)
    }

    async fn health_on(&self, state: &mut State) -> Result<Health, PluginError> {
        let inner = &self.inner;
        ensure_live(inner, state, None).await?;
        let id = state.take_id();
        let frame = inner.encode(&id, METHOD_HEALTH, None::<()>)?;
        let limits = &inner.config.limits;
        let ended = match state.live.as_mut() {
            Some(live) => {
                exchange(live, &id, &frame, limits.health, None, None, limits.grace).await
            }
            None => Ended::Exited(None),
        };
        state.last_used = Instant::now();
        let value = inner.settle(state, &id, METHOD_HEALTH, ended).await?;
        let mut health: Health = inner
            .decode(value, "health")
            .map_err(|m| inner.host_error(state, &id, ErrorCode::InvalidOutput, m))?;
        envelope_text::health(&mut health);
        Ok(health)
    }

    /// 处理一条。要停下一条用 `cancel`：丢掉这个 future 也行，但那个进程随后被当成坏了，下次重起。
    /// 成功时输出目录交给调用方；失败时输出目录删掉，错误码照协议 §3.3：
    /// 插件报的照转（`dependency-missing` 带 `needs`），宿主记的是 `timeout`（超时、卡住）、`crashed`、
    /// `invalid-output`、`cancelled`。超时、卡住、退出、坏消息之后进程已经关掉，下一次重起。
    pub async fn run(
        &self,
        request: TransformRequest,
        cancel: &CancellationToken,
    ) -> Result<TransformOutput, PluginError> {
        let inner = &self.inner;
        inner.check_request(&request)?;
        let mut state = tokio::select! {
            state = inner.state.lock() => state,
            () = cancel.cancelled() => {
                return Err(inner.error(
                    "-".into(),
                    CallFailure::new(ErrorCode::Cancelled, "cancelled before it started"),
                    Vec::new(),
                    None,
                ));
            }
        };
        ensure_live(inner, &mut state, Some(cancel)).await?;
        let result = inner.run_on(&mut state, request, cancel).await;
        state.last_used = Instant::now();
        result
    }

    /// 关掉进程：关 stdin，再 SIGTERM、SIGKILL 进程组。没起着就什么都不做。
    /// 调用方是 WP6：守护进程退出、插件被换掉或删掉时。
    pub async fn shutdown(&self) {
        let mut state = self.inner.state.lock().await;
        if let Some(live) = state.live.take() {
            stop(live, &self.inner.config.limits, Stop::CloseStdin).await;
        }
    }
}

/// 没有活着的进程就起一个；有，但它退出了、上一个请求被调用方丢下了，或闲着时往 stdout 写了东西，
/// 就关掉换一个。
async fn ensure_live(
    inner: &Arc<Inner>,
    state: &mut State,
    cancel: Option<&CancellationToken>,
) -> Result<(), PluginError> {
    if let Some(live) = &mut state.live {
        let exited = !matches!(live.child.try_wait(), Ok(None));
        let stray = live.take_stray();
        if !exited && !live.busy && stray.is_none() {
            return Ok(());
        }
        tracing::warn!(
            plugin = %inner.plugin.id(),
            exited,
            abandoned = live.busy,
            stray = ?stray,
            "transform process is not usable; starting a new one"
        );
        if let Some(live) = state.live.take() {
            stop(live, &inner.config.limits, Stop::Terminate).await;
        }
    }
    start(inner, state, cancel).await
}

/// 起进程、核 `describe`（协议 §4.1）：`protocol` 要是宿主会说的，`version` 要和清单一样，不然不加载。
async fn start(
    inner: &Arc<Inner>,
    state: &mut State,
    cancel: Option<&CancellationToken>,
) -> Result<(), PluginError> {
    let plugin = &inner.plugin;
    let config = &inner.config;
    let proc_id = uuid::Uuid::now_v7().to_string();
    let staging = |what: &str| {
        let what = what.to_owned();
        move |source| PluginError::Staging { what, source }
    };
    std::fs::create_dir_all(&config.work_dir).map_err(staging("work directory"))?;
    let dir = tempfile::Builder::new()
        .prefix(&format!("usuyo-proc-{proc_id}-"))
        .tempdir_in(&config.work_dir)
        .map_err(staging("process directory"))?;
    for sub in ["in", "out", "home/.config", "bin", "tmp"] {
        std::fs::create_dir_all(dir.path().join(sub)).map_err(staging(sub))?;
    }
    let cache = config
        .cache_dir
        .join(plugin.id().owner())
        .join(plugin.id().repo());
    std::fs::create_dir_all(&cache).map_err(staging("cache directory"))?;
    let models = config
        .models_dir
        .join(plugin.id().owner())
        .join(plugin.id().repo());

    let (program, args) = resolve_command(&plugin.header, &plugin.dir)
        .map_err(|f| inner.error(proc_id.clone(), f, Vec::new(), None))?;
    let mut env = plugin_env(
        &plugin.header,
        &PluginDirs {
            plugin: &plugin.dir,
            cache: &cache,
            home: &dir.path().join("home"),
            tmp: &dir.path().join("tmp"),
            bin: &dir.path().join("bin"),
        },
        &config.locale.get(),
    );
    env.insert("USUYO_MODELS_DIR".into(), models.clone().into());

    let mut cmd = Command::new(&program);
    cmd.args(&args)
        .env_clear()
        .envs(&env)
        .current_dir(dir.path().join("tmp"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true);
    #[cfg(unix)]
    cmd.process_group(0);
    // §7 的沙箱接在这里：启动时只放行暂存目录、`${CACHE}`、`${MODELS}`。
    let mut child = cmd.spawn().map_err(|e| {
        inner.error(
            proc_id.clone(),
            CallFailure::new(
                ErrorCode::Crashed,
                format!("could not start the plugin: {e}"),
            ),
            Vec::new(),
            None,
        )
    })?;
    let group = child.id();
    let pipes = (child.stdin.take(), child.stdout.take(), child.stderr.take());
    let (Some(stdin), Some(stdout), Some(stderr)) = pipes else {
        return Err(inner.error(
            proc_id,
            CallFailure::new(ErrorCode::Crashed, "the plugin's stdio is not piped"),
            Vec::new(),
            None,
        ));
    };
    let limits = &config.limits;
    let (tx, incoming) = mpsc::channel(INCOMING_BACKLOG);
    let tail = Arc::new(StdMutex::new(VecDeque::new()));
    let tasks = [
        tokio::spawn(read_stdout(stdout, limits.frame_bytes, tx)),
        tokio::spawn(read_stderr(stderr, limits.stderr_lines, Arc::clone(&tail))),
    ];
    let mut live = Live {
        id: proc_id,
        child,
        group,
        stdin: Some(stdin),
        incoming,
        stderr: tail,
        scrub: Scrub {
            token: None,
            dirs: [
                (dir.path().to_owned(), "${PROC}"),
                (models, "${MODELS}"),
                (cache, "${CACHE}"),
                (plugin.dir.clone(), "${PLUGIN_DIR}"),
            ]
            .into_iter()
            .flat_map(|(d, name)| {
                let real = d.canonicalize().ok();
                std::iter::once((d, name)).chain(real.map(|r| (r, name)))
            })
            .collect(),
        },
        dir: Arc::new(dir),
        tasks,
        busy: false,
    };

    let id = state.take_id();
    let frame = inner.encode(&id, METHOD_DESCRIBE, None::<()>)?;
    let ended = exchange(
        &mut live,
        &id,
        &frame,
        limits.describe,
        None,
        cancel,
        limits.grace,
    )
    .await;
    state.live = Some(live);
    // 没核过 `describe` 的进程不留，哪怕它在取消以后才答回来。
    let value = match inner.settle(state, &id, METHOD_DESCRIBE, ended).await {
        Ok(value) => value,
        Err(e) => {
            if let Some(live) = state.live.take() {
                stop(live, limits, Stop::Terminate).await;
            }
            return Err(e);
        }
    };
    let checked = inner.decode::<Describe>(value, "describe").and_then(|d| {
        if !SUPPORTED_PROTOCOLS.contains(&d.protocol) {
            return Err(format!(
                "describe: protocol {} is not one the host speaks ({SUPPORTED_PROTOCOLS:?})",
                d.protocol
            ));
        }
        if d.version != plugin.header.version {
            return Err(format!(
                "describe: version {} does not match the manifest's {}",
                d.version, plugin.header.version
            ));
        }
        Ok(())
    });
    if let Err(message) = checked {
        let failure = inner.host_error(state, &id, ErrorCode::InvalidOutput, message);
        if let Some(live) = state.live.take() {
            stop(live, limits, Stop::CloseStdin).await;
        }
        return Err(failure);
    }
    state.generation += 1;
    state.last_used = Instant::now();
    watch_idle(inner, state.generation);
    Ok(())
}

/// 闲了 `idle` 就关掉这一代进程。处理着 `run` 时拿不到锁，处理完 `last_used` 是新的，接着等。
fn watch_idle(inner: &Arc<Inner>, generation: u64) {
    let weak: Weak<Inner> = Arc::downgrade(inner);
    let idle = inner.config.limits.idle;
    tokio::spawn(async move {
        let mut wait = idle;
        loop {
            tokio::time::sleep(wait).await;
            let Some(inner) = weak.upgrade() else { return };
            let mut state = inner.state.lock().await;
            if state.generation != generation || state.live.is_none() {
                return;
            }
            let quiet = state.last_used.elapsed();
            if quiet < idle {
                wait = idle - quiet;
                continue;
            }
            if let Some(live) = state.live.take() {
                tracing::debug!(plugin = %inner.plugin.id(), "transform process idle; stopping it");
                stop(live, &inner.config.limits, Stop::CloseStdin).await;
            }
            return;
        }
    });
}

impl Inner {
    /// 调用方给的请求对不对得上清单：在起进程、放输入之前查。
    fn check_request(&self, request: &TransformRequest) -> Result<(), PluginError> {
        let spec = &self.plugin.spec;
        let unsupported = |message: String| PluginError::Unsupported {
            plugin: self.plugin.id().clone(),
            message,
        };
        if request.want.is_empty() {
            return Err(unsupported("`want` is empty".into()));
        }
        let mut seen = BTreeSet::new();
        for w in &request.want {
            if !spec.produces.contains(w) {
                return Err(unsupported(format!("`{w}` is not among its `produces`")));
            }
            if !seen.insert(*w) {
                return Err(unsupported(format!("`{w}` is wanted twice")));
            }
        }
        for (i, input) in request.inputs.iter().enumerate() {
            let fits = spec.consumes.iter().any(|c| {
                c.kind == input.kind
                    && (c.mime.is_empty()
                        || input.mime.as_ref().is_some_and(|m| c.mime.contains(m)))
            });
            if !fits {
                return Err(unsupported(format!(
                    "input {i} ({:?}, mime {:?}) is not among its `consumes`",
                    input.kind, input.mime
                )));
            }
        }
        match (spec.provider, &request.provider) {
            (ProviderNeed::Required, None) => Err(unsupported(
                "it needs a model provider and none was given".into(),
            )),
            (ProviderNeed::None, Some(_)) => {
                Err(unsupported("it declares `provider: none`".into()))
            }
            _ => Ok(()),
        }
    }

    async fn run_on(
        &self,
        state: &mut State,
        mut request: TransformRequest,
        cancel: &CancellationToken,
    ) -> Result<TransformOutput, PluginError> {
        let limits = &self.config.limits;
        let id = state.take_id();
        let token = request.provider.as_ref().map(|p| p.token.clone());
        let Some(live) = state.live.as_mut() else {
            return Err(self.host_error(
                state,
                &id,
                ErrorCode::Crashed,
                "the process is not running".into(),
            ));
        };
        live.scrub.token = token;
        live.clear_stderr();
        let proc_dir = Arc::clone(&live.dir);
        let name = id.to_string();
        let in_dir = CallDir::create(proc_dir.path().join("in").join(&name), &proc_dir).map_err(
            |source| PluginError::Staging {
                what: format!("in/{name}"),
                source,
            },
        )?;
        let out_dir = CallDir::create(proc_dir.path().join("out").join(&name), &proc_dir).map_err(
            |source| PluginError::Staging {
                what: format!("out/{name}"),
                source,
            },
        )?;

        let inputs = std::mem::take(&mut request.inputs);
        let staged_into = in_dir.path.clone();
        let plugin_id = self.plugin.id().clone();
        let inputs = tokio::task::spawn_blocking(move || stage(inputs, &staged_into, &plugin_id))
            .await
            .map_err(|e| PluginError::Staging {
                what: format!("in/{name}"),
                source: io::Error::other(e),
            })??;
        let want = request.want.clone();
        let input_count = inputs.len();
        let params = RunParams {
            item: request.item,
            want: request.want,
            inputs,
            out: out_dir.path.clone(),
            options: request.options,
            provider: request.provider,
        };
        let frame = self.encode(&id, METHOD_RUN, Some(&params))?;
        if frame.len() > limits.frame_bytes {
            return Err(PluginError::Unsupported {
                plugin: self.plugin.id().clone(),
                message: format!(
                    "the `run` request is {} bytes, over the {}-byte message limit",
                    frame.len(),
                    limits.frame_bytes
                ),
            });
        }
        let total = self
            .plugin
            .header
            .permissions
            .limits
            .timeout
            .map_or(limits.run, |t| t.get());
        let ended = exchange(
            live,
            &id,
            &frame,
            total,
            Some(limits.stuck),
            Some(cancel),
            limits.grace,
        )
        .await;
        drop(in_dir);
        // 取消了的，哪怕插件回了成功也按 `cancelled`，输出目录随 `out_dir` 删掉。
        let value = self.settle(state, &id, METHOD_RUN, ended).await?;
        let invalid = |state: &State, message: String| {
            self.host_error(state, &id, ErrorCode::InvalidOutput, message)
        };
        // 先挪出插件够得着的地方，再核对。
        let out_dir = HostDir::take(out_dir, &self.config.work_dir).map_err(|source| {
            PluginError::Staging {
                what: format!("out/{name}"),
                source,
            }
        })?;
        let mut result: RunResult = self
            .decode(value, "run result")
            .map_err(|m| invalid(state, m))?;
        envelope_text::run(&mut result);
        self.check_result(&result, &want, input_count)
            .map_err(|m| invalid(state, m))?;
        let max = limits.out_bytes;
        let dir = out_dir.path.clone();
        let files =
            tokio::task::spawn_blocking(move || outdir::inspect(&dir, max, outdir::RUN_PATH_UTF16))
                .await
                .map_err(|e| invalid(state, format!("inspect `out`: {e}")))?
                .map_err(|m| invalid(state, m))?;
        check_reported(&result, &files).map_err(|m| invalid(state, m))?;
        Ok(TransformOutput {
            result,
            files,
            out: out_dir,
        })
    }

    /// `run` 的结果对不对得上这次的 `want`、输入和清单。
    fn check_result(&self, r: &RunResult, want: &[Produce], inputs: usize) -> Result<(), String> {
        if let Some(why) = r.problem(want, inputs) {
            return Err(format!("run result: {why}"));
        }
        if r.tool.name.is_empty() || r.tool.version.is_empty() {
            return Err("run result: `tool` needs a name and a version".into());
        }
        if let Some(m) = &r.model
            && !self.plugin.spec.models.iter().any(|s| s.name == m.name)
        {
            return Err(format!(
                "run result: model `{}` is not among the manifest's `models`",
                m.name
            ));
        }
        if let Some(o) = r
            .outputs
            .iter()
            .find(|o| o.lang.as_deref().is_some_and(|l| !is_language_tag(l)))
        {
            return Err(format!(
                "run result: output `{}` has a `lang` that is not a language tag",
                o.kind
            ));
        }
        Ok(())
    }

    /// 一次交换的结局变成值或错误；要关进程的在这里关。`method` 写进宿主拼的错误。
    async fn settle(
        &self,
        state: &mut State,
        id: &RpcId,
        method: &str,
        ended: Ended,
    ) -> Result<Value, PluginError> {
        let host = |code: ErrorCode, message: String| CallFailure::new(code, message);
        let (failure, how) = match ended {
            Ended::Answered(Ok(value)) => {
                // 回答以后进程自己退了：照用回答，进程收掉（连同它留下的子进程）。
                let exited = state
                    .live
                    .as_mut()
                    .is_some_and(|l| !matches!(l.child.try_wait(), Ok(None)));
                if exited && let Some(live) = state.live.take() {
                    stop(live, &self.config.limits, Stop::Terminate).await;
                }
                return Ok(value);
            }
            Ended::Answered(Err(error)) => (self.plugin_failure(error), None),
            Ended::TimedOut(d) => (
                host(
                    ErrorCode::Timeout,
                    format!("`{method}` took longer than {d:?}; the process was stopped"),
                ),
                Some(Stop::Terminate),
            ),
            Ended::Stuck(d) => (
                host(
                    ErrorCode::Timeout,
                    format!(
                        "`{method}` sent no `progress` for {d:?}; it is stuck and the process was stopped"
                    ),
                ),
                Some(Stop::Terminate),
            ),
            Ended::Cancelled { answered } => (
                host(ErrorCode::Cancelled, "cancelled".into()),
                (!answered).then_some(Stop::Terminate),
            ),
            Ended::Exited(status) => (
                host(
                    ErrorCode::Crashed,
                    match status {
                        Some(s) => format!("the plugin exited during `{method}` ({s})"),
                        None => format!("the plugin closed its stdout during `{method}`"),
                    },
                ),
                Some(Stop::Terminate),
            ),
            Ended::Broken(why) => (host(ErrorCode::InvalidOutput, why), Some(Stop::Terminate)),
        };
        let error = self.failure_on(state, id, failure);
        if let Some(how) = how
            && let Some(live) = state.live.take()
        {
            stop(live, &self.config.limits, how).await;
        }
        Err(error)
    }

    /// 插件回的 JSON-RPC `error`（协议 §4.1）：`-32000` 带错误对象是插件自己的失败，照转；
    /// 标准码是它不认这个请求；别的都算输出不合规。
    fn plugin_failure(&self, error: Value) -> CallFailure {
        let error: RpcError = match self.decode(error, "error") {
            Ok(e) => e,
            Err(m) => return CallFailure::new(ErrorCode::InvalidOutput, m),
        };
        match (error.code, error.data) {
            (RpcError::PLUGIN_FAILURE, Some(f)) => match f.check_from_plugin() {
                Ok(()) => *f,
                Err(why) => CallFailure::new(ErrorCode::InvalidOutput, why),
            },
            (RpcError::PLUGIN_FAILURE, None) => CallFailure::new(
                ErrorCode::InvalidOutput,
                "`-32000` without an error object in `data`",
            ),
            (
                code @ (RpcError::PARSE_ERROR
                | RpcError::INVALID_REQUEST
                | RpcError::METHOD_NOT_FOUND
                | RpcError::INVALID_PARAMS),
                _,
            ) => CallFailure::new(
                ErrorCode::InvalidArgs,
                format!("the plugin refused the request ({code}): {}", error.message),
            ),
            (code, _) => CallFailure::new(
                ErrorCode::InvalidOutput,
                format!("unknown JSON-RPC error code {code}"),
            ),
        }
    }

    fn encode<P: Serialize>(
        &self,
        id: &RpcId,
        method: &str,
        params: Option<P>,
    ) -> Result<Vec<u8>, PluginError> {
        request_frame(id, method, params).map_err(|e| PluginError::Unsupported {
            plugin: self.plugin.id().clone(),
            message: format!("could not encode the `{method}` request: {e}"),
        })
    }

    /// 按类型读插件给的值。错了只说是哪一栏：解析器的原话会引用值。
    fn decode<T: DeserializeOwned>(&self, value: Value, what: &str) -> Result<T, String> {
        let mut unknown = Vec::new();
        let decoded = crate::protocol::deserialize_tracked::<T>(value, &mut unknown)
            .map_err(|(field, _)| format!("{what} at `{field}` has the wrong type or shape"))?;
        for field in unknown {
            // 字段名来自插件：按 Debug 写，换行进不了日志
            tracing::warn!(plugin = %self.plugin.id(), field = ?field, "unknown {what} field ignored");
        }
        Ok(decoded)
    }

    fn host_error(
        &self,
        state: &State,
        id: &RpcId,
        code: ErrorCode,
        message: String,
    ) -> PluginError {
        self.failure_on(state, id, CallFailure::new(code, message))
    }

    /// 这次调用的错误，带上进程 stderr 的最后几行。
    fn failure_on(&self, state: &State, id: &RpcId, failure: CallFailure) -> PluginError {
        match &state.live {
            Some(live) => self.error(
                format!("{}/{id}", live.id),
                failure,
                live.stderr_tail(),
                Some(&live.scrub),
            ),
            None => self.error(format!("-/{id}"), failure, Vec::new(), None),
        }
    }

    /// 收口：令牌和目录换掉，一行的字截短。
    fn error(
        &self,
        call_id: String,
        mut failure: CallFailure,
        mut stderr: Vec<String>,
        scrub: Option<&Scrub>,
    ) -> PluginError {
        if let Some(scrub) = scrub {
            scrub.apply(&mut failure.message);
            if let Some(hint) = &mut failure.hint {
                scrub.apply(hint);
            }
            for line in &mut stderr {
                scrub.apply(line);
            }
        }
        envelope_text::failure(&mut failure);
        PluginError::Call {
            plugin: self.plugin.id().clone(),
            call_id,
            failure: Box::new(failure),
            stderr,
        }
    }
}

/// 结果里报的每个产物都在 `out` 里；`out` 里每个文件都有人报过。`keyframes` 的帧图片在
/// `keyframes.json` 旁边的 `keyframes/` 里，不逐个报（协议 §4.2）。
fn check_reported(result: &RunResult, files: &[OutFile]) -> Result<(), String> {
    let present: BTreeSet<&str> = files.iter().map(|f| f.path.as_str()).collect();
    let mut frame_dirs = Vec::new();
    for o in &result.outputs {
        if !present.contains(o.path.as_str()) {
            return Err(format!(
                "output `{}` names `{}`, which is not a file in `out`",
                o.kind, o.path
            ));
        }
        if o.kind == Produce::Keyframes {
            frame_dirs.push(match o.path.as_str().rsplit_once('/') {
                Some((parent, _)) => format!("{parent}/keyframes/"),
                None => "keyframes/".to_owned(),
            });
        }
    }
    let reported: BTreeSet<&str> = result.outputs.iter().map(|o| o.path.as_str()).collect();
    match files.iter().find(|f| {
        !reported.contains(f.path.as_str())
            && !frame_dirs.iter().any(|d| f.path.as_str().starts_with(d))
    }) {
        Some(stray) => Err(format!(
            "`{}` is in `out` but no output reports it",
            stray.path
        )),
        None => Ok(()),
    }
}

/// BCP 47 的样子：字母、数字和 `-`，最多 35 个字符。
fn is_language_tag(tag: &str) -> bool {
    (1..=35).contains(&tag.len())
        && tag.split('-').all(|part| {
            !part.is_empty() && part.len() <= 8 && part.bytes().all(|b| b.is_ascii_alphanumeric())
        })
}

/// 把输入复制进 `in/<callId>/<下标>/<原文件名>`（协议 §7.1）：插件改了它也碰不到资料库里的原样。
/// APFS 上 `std::fs::copy` 是写时复制的克隆，大文件也不费空间。
/// 下标分开同名的文件；原文件名留着扩展名，插件按它认格式。
fn stage(
    mut inputs: Vec<RunInput>,
    dir: &Path,
    plugin: &crate::protocol::PluginId,
) -> Result<Vec<RunInput>, PluginError> {
    for (i, input) in inputs.iter_mut().enumerate() {
        let staging = |source| PluginError::Staging {
            what: format!("input {i}"),
            source,
        };
        let unsupported = |message: String| PluginError::Unsupported {
            plugin: plugin.clone(),
            message,
        };
        let name = input
            .path
            .file_name()
            .ok_or_else(|| unsupported(format!("input {i} has no file name")))?
            .to_owned();
        let meta = std::fs::symlink_metadata(&input.path).map_err(staging)?;
        if !meta.is_file() {
            return Err(unsupported(format!("input {i} is not a regular file")));
        }
        let sub = dir.join(i.to_string());
        std::fs::create_dir(&sub).map_err(staging)?;
        let dest = sub.join(name);
        std::fs::copy(&input.path, &dest).map_err(staging)?;
        input.path = dest;
    }
    Ok(inputs)
}

/// 一次调用的 `in/<callId>/` 或 `out/<callId>/`。丢掉时删掉；拿着进程暂存目录的引用，进程关了它也在。
#[derive(Debug)]
struct CallDir {
    path: PathBuf,
    _proc: Arc<tempfile::TempDir>,
}

impl CallDir {
    fn create(path: PathBuf, proc_dir: &Arc<tempfile::TempDir>) -> io::Result<Self> {
        std::fs::create_dir(&path)?;
        Ok(Self {
            path,
            _proc: Arc::clone(proc_dir),
        })
    }
}

impl Drop for CallDir {
    fn drop(&mut self) {
        match std::fs::remove_dir_all(&self.path) {
            Ok(()) => {}
            Err(e) if e.kind() == io::ErrorKind::NotFound => {}
            Err(e) => tracing::warn!(error = %e, "could not remove a transform call directory"),
        }
    }
}

/// 回答以后挪出来的输出目录：`<work_dir>/<UUIDv7>/`，丢掉时删掉。资料库打开时清掉以 UUIDv7 命名的暂存，
/// 宿主中途退出也不会留下。
#[derive(Debug)]
struct HostDir {
    path: PathBuf,
}

impl HostDir {
    fn take(from: CallDir, work_dir: &Path) -> io::Result<HostDir> {
        let path = work_dir.join(uuid::Uuid::now_v7().to_string());
        std::fs::rename(&from.path, &path)?;
        Ok(HostDir { path })
    }
}

impl Drop for HostDir {
    fn drop(&mut self) {
        match std::fs::remove_dir_all(&self.path) {
            Ok(()) => {}
            Err(e) if e.kind() == io::ErrorKind::NotFound => {}
            Err(e) => tracing::warn!(error = %e, "could not remove a transform output directory"),
        }
    }
}

/// 一个活着的进程。
struct Live {
    /// UUIDv7，也在暂存目录的名字里；错误的 `call_id` 是 `<它>/<JSON-RPC id>`。
    id: String,
    child: Child,
    /// 进程组 id（就是进程的 pid）；收过尾以后为 `None`，丢掉时不再发信号。
    group: Option<u32>,
    stdin: Option<ChildStdin>,
    incoming: mpsc::Receiver<Incoming>,
    stderr: Arc<StdMutex<VecDeque<String>>>,
    scrub: Scrub,
    dir: Arc<tempfile::TempDir>,
    tasks: [JoinHandle<()>; 2],
    /// 一次交换发出去了、还没收尾。调用方丢掉 `run` 的 future 时它留着 `true`：那个请求还在进程里，
    /// 这个进程不能再用。
    busy: bool,
}

impl Live {
    /// 两次调用之间插件写到 stdout 的：没人等的回答、坏消息、stdout 关了。都说明这个进程不能再用。
    fn take_stray(&mut self) -> Option<String> {
        loop {
            match self.incoming.try_recv() {
                Ok(Incoming::Progress(_) | Incoming::Ignored) => {}
                Ok(Incoming::Reply { .. }) => return Some("a reply nobody asked for".into()),
                Ok(Incoming::Broken(why)) => return Some(why),
                Err(mpsc::error::TryRecvError::Empty) => return None,
                Err(mpsc::error::TryRecvError::Disconnected) => {
                    return Some("stdout is closed".into());
                }
            }
        }
    }

    fn clear_stderr(&self) {
        self.stderr
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .clear();
    }

    fn stderr_tail(&self) -> Vec<String> {
        self.stderr
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .iter()
            .cloned()
            .collect()
    }
}

impl Drop for Live {
    fn drop(&mut self) {
        for task in &self.tasks {
            task.abort();
        }
        if let Some(group) = self.group.take() {
            kill_group(group);
        }
    }
}

/// 插件给的字里要换掉的：这次 `run` 的令牌，和进程看得到的几个目录（换成协议的占位符）。
struct Scrub {
    token: Option<String>,
    dirs: Vec<(PathBuf, &'static str)>,
}

impl Scrub {
    fn apply(&self, text: &mut String) {
        if let Some(token) = &self.token {
            credentials::scrub(text, std::slice::from_ref(token));
        }
        for (dir, name) in &self.dirs {
            let dir = dir.to_string_lossy();
            if !dir.is_empty() && text.contains(dir.as_ref()) {
                *text = text.replace(dir.as_ref(), name);
            }
        }
    }
}

#[derive(Clone, Copy)]
enum Stop {
    /// 关 stdin，等 `stop`；没退再 SIGTERM。
    CloseStdin,
    /// 进程不守规矩：直接 SIGTERM。
    Terminate,
}

/// 收掉一个进程：最后总是 SIGKILL 整个进程组，插件留下的子进程一并结束。
async fn stop(mut live: Live, limits: &TransformLimits, how: Stop) {
    drop(live.stdin.take());
    let grace = match how {
        Stop::CloseStdin => match timeout(limits.stop, live.child.wait()).await {
            Ok(_) => None,
            Err(_) => Some(limits.grace),
        },
        Stop::Terminate => Some(limits.grace),
    };
    stop_group(&mut live.child, live.group.take(), grace).await;
}

#[cfg(unix)]
fn kill_group(group: u32) {
    use nix::sys::signal::{Signal, killpg};
    use nix::unistd::Pid;

    let Ok(pid) = i32::try_from(group) else {
        return;
    };
    match killpg(Pid::from_raw(pid), Signal::SIGKILL) {
        // 进程组已经没有成员
        Ok(()) | Err(nix::errno::Errno::ESRCH) => {}
        Err(e) => tracing::warn!("kill transform process group: {e}"),
    }
}

#[cfg(not(unix))]
fn kill_group(group: u32) {
    // Windows 上终止 Job Object 的做法接在这里；现在 `kill_on_drop` 只结束插件本身
    let _ = group;
}

/// 一次交换怎么结束的。
enum Ended {
    /// 插件回了：`result` 或 `error` 原样的值。
    Answered(Result<Value, Value>),
    TimedOut(Duration),
    Stuck(Duration),
    /// 发了 `cancel`；`answered` 是宽限期内回没回。
    Cancelled {
        answered: bool,
    },
    /// 进程退出（有退出状态）或关了 stdout。
    Exited(Option<ExitStatus>),
    /// stdout 上来了不合协议的东西。
    Broken(String),
}

/// 发一个请求，等它的回答。`stuck` 有值时，收到这个 `id` 的 `progress` 以后开始算卡住。
async fn exchange(
    live: &mut Live,
    id: &RpcId,
    frame: &[u8],
    total: Duration,
    stuck: Option<Duration>,
    cancel: Option<&CancellationToken>,
    grace: Duration,
) -> Ended {
    live.busy = true;
    let ended = converse(live, id, frame, total, stuck, cancel, grace).await;
    live.busy = false;
    ended
}

async fn converse(
    live: &mut Live,
    id: &RpcId,
    frame: &[u8],
    total: Duration,
    stuck: Option<Duration>,
    cancel: Option<&CancellationToken>,
    grace: Duration,
) -> Ended {
    let total_deadline = Instant::now() + total;
    let Some(stdin) = live.stdin.as_mut() else {
        return Ended::Exited(None);
    };
    match timeout_at(total_deadline, write_frame(stdin, frame)).await {
        Err(_) => return Ended::TimedOut(total),
        Ok(Err(_)) => return Ended::Exited(live.child.try_wait().ok().flatten()),
        Ok(Ok(())) => {}
    }
    let mut stuck_deadline: Option<Instant> = None;
    let mut cancel_deadline: Option<Instant> = None;
    loop {
        let deadline = [Some(total_deadline), stuck_deadline, cancel_deadline]
            .into_iter()
            .flatten()
            .min()
            .unwrap_or(total_deadline);
        // 先看时限和取消（插件不停地发 progress 也挡不住它们），再看已经读到的消息，最后看进程退没退：
        // 退出前写下的坏消息照样按坏消息算。
        tokio::select! {
            biased;
            () = sleep_until(deadline) => {
                let now = Instant::now();
                if cancel_deadline.is_some_and(|d| now >= d) {
                    return Ended::Cancelled { answered: false };
                }
                if let Some(stuck) = stuck && stuck_deadline.is_some_and(|d| now >= d) {
                    return Ended::Stuck(stuck);
                }
                return Ended::TimedOut(total);
            }
            () = cancelled(cancel), if cancel_deadline.is_none() => {
                cancel_deadline = Some(Instant::now() + grace);
                match notification_frame(METHOD_CANCEL, &crate::protocol::Cancel { id: id.clone() }) {
                    Ok(note) => {
                        if let Some(stdin) = live.stdin.as_mut()
                            && !matches!(timeout(grace, write_frame(stdin, &note)).await, Ok(Ok(())))
                        {
                            tracing::debug!("could not send `cancel`; waiting for the process anyway");
                        }
                    }
                    Err(e) => tracing::warn!("could not encode `cancel`: {e}; waiting for the process anyway"),
                }
            }
            message = live.incoming.recv() => match message {
                Some(Incoming::Reply { id: got, answer }) if &got == id => {
                    return match cancel_deadline {
                        Some(_) => Ended::Cancelled { answered: true },
                        None => Ended::Answered(answer),
                    };
                }
                Some(Incoming::Reply { .. }) => {
                    return Ended::Broken("a reply names an id that is not waiting".into());
                }
                Some(Incoming::Progress(p)) => {
                    if let Some(stuck) = stuck && &p == id {
                        stuck_deadline = Some(Instant::now() + stuck);
                    }
                }
                Some(Incoming::Ignored) => {}
                Some(Incoming::Broken(why)) => return Ended::Broken(why),
                None => {
                    let status = timeout(grace, live.child.wait()).await.ok().and_then(Result::ok);
                    return Ended::Exited(status);
                }
            },
            status = live.child.wait() => {
                // 退出前写的回答可能还在管道里：读到 stdout 关上，最多等 `grace`
                // （进程留下的子进程可能还拿着 stdout）。
                let until = Instant::now() + grace;
                loop {
                    match timeout_at(until, live.incoming.recv()).await {
                        Ok(Some(Incoming::Reply { id: got, answer })) if &got == id => {
                            return match cancel_deadline {
                                Some(_) => Ended::Cancelled { answered: true },
                                None => Ended::Answered(answer),
                            };
                        }
                        Ok(Some(Incoming::Reply { .. })) => {
                            return Ended::Broken("a reply names an id that is not waiting".into());
                        }
                        Ok(Some(Incoming::Broken(why))) => return Ended::Broken(why),
                        Ok(Some(Incoming::Progress(_) | Incoming::Ignored)) => {}
                        Ok(None) | Err(_) => return Ended::Exited(status.ok()),
                    }
                }
            }
        }
    }
}

async fn cancelled(cancel: Option<&CancellationToken>) {
    match cancel {
        Some(token) => token.cancelled().await,
        None => std::future::pending().await,
    }
}

async fn write_frame(stdin: &mut ChildStdin, frame: &[u8]) -> io::Result<()> {
    stdin.write_all(frame).await?;
    stdin.flush().await
}

#[derive(Serialize)]
struct Request<'a, P: Serialize> {
    jsonrpc: &'static str,
    id: &'a RpcId,
    method: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    params: Option<P>,
}

#[derive(Serialize)]
struct Notification<'a, P: Serialize> {
    jsonrpc: &'static str,
    method: &'a str,
    params: &'a P,
}

/// 一行请求，带换行。字段顺序固定：`jsonrpc`、`id`、`method`、`params`。
/// 编不出来的只有不是 UTF-8 的路径。
fn request_frame<P: Serialize>(
    id: &RpcId,
    method: &str,
    params: Option<P>,
) -> serde_json::Result<Vec<u8>> {
    line(&Request {
        jsonrpc: "2.0",
        id,
        method,
        params,
    })
}

fn notification_frame<P: Serialize>(method: &str, params: &P) -> serde_json::Result<Vec<u8>> {
    line(&Notification {
        jsonrpc: "2.0",
        method,
        params,
    })
}

fn line(value: &impl Serialize) -> serde_json::Result<Vec<u8>> {
    let mut bytes = serde_json::to_vec(value)?;
    bytes.push(b'\n');
    Ok(bytes)
}

/// stdout 上读到的一条。
enum Incoming {
    /// 某个请求的回答：`result` 或 `error` 原样的值。
    Reply {
        id: RpcId,
        answer: Result<Value, Value>,
    },
    Progress(RpcId),
    /// 认不出的通知，忽略。
    Ignored,
    /// 不合协议；读的任务随后停下。
    Broken(String),
}

/// 一直读 stdout，直到关上或读到不合协议的一行。超长的行不留在内存里。
async fn read_stdout(stdout: impl AsyncRead + Unpin, max: usize, tx: mpsc::Sender<Incoming>) {
    let mut reader = BufReader::new(stdout);
    loop {
        let incoming = match read_frame(&mut reader, max).await {
            Ok(Frame::Eof) => return,
            Ok(Frame::TooLong) => Incoming::Broken(format!("a message is longer than {max} bytes")),
            Ok(Frame::Line(line)) if line.iter().all(u8::is_ascii_whitespace) => continue,
            Ok(Frame::Line(line)) => parse_incoming(&line),
            Err(e) => Incoming::Broken(format!("read stdout: {e}")),
        };
        let broken = matches!(incoming, Incoming::Broken(_));
        if tx.send(incoming).await.is_err() || broken {
            return;
        }
    }
}

enum Frame {
    Line(Vec<u8>),
    TooLong,
    Eof,
}

async fn read_frame(r: &mut (impl AsyncBufRead + Unpin), max: usize) -> io::Result<Frame> {
    let mut line = Vec::new();
    loop {
        let buf = r.fill_buf().await?;
        if buf.is_empty() {
            return Ok(if line.is_empty() {
                Frame::Eof
            } else {
                Frame::Line(line)
            });
        }
        let newline = buf.iter().position(|&b| b == b'\n');
        let chunk = &buf[..newline.unwrap_or(buf.len())];
        if line.len() + chunk.len() > max {
            return Ok(Frame::TooLong);
        }
        line.extend_from_slice(chunk);
        let used = newline.map_or(buf.len(), |i| i + 1);
        r.consume(used);
        if newline.is_some() {
            if line.last() == Some(&b'\r') {
                line.pop();
            }
            return Ok(Frame::Line(line));
        }
    }
}

/// 一行 stdout 是哪种消息。插件只能发回答和 `progress` 通知（协议 §4.1）。
fn parse_incoming(line: &[u8]) -> Incoming {
    let broken = |why: &str| Incoming::Broken(why.to_owned());
    let Ok(Value::Object(mut map)) = serde_json::from_slice::<Value>(line) else {
        return broken("a stdout line is not a JSON object");
    };
    if map.get("jsonrpc").and_then(Value::as_str) != Some("2.0") {
        return broken("a message's `jsonrpc` is not \"2.0\"");
    }
    match (map.remove("method"), map.remove("id")) {
        (Some(Value::String(method)), None) => {
            if method != METHOD_PROGRESS {
                tracing::debug!(method = ?method.chars().take(64).collect::<String>(), "unknown notification ignored");
                return Incoming::Ignored;
            }
            match serde_json::from_value::<Progress>(map.remove("params").unwrap_or(Value::Null)) {
                Ok(p) => Incoming::Progress(p.id),
                Err(_) => broken("`progress` has the wrong shape"),
            }
        }
        (Some(_), Some(_)) => broken("the plugin sent a request; only notifications go this way"),
        (Some(_), None) => broken("a notification's `method` is not a string"),
        (None, Some(id)) => {
            let Ok(id) = serde_json::from_value::<RpcId>(id) else {
                return broken("a reply's `id` is not an integer or a string");
            };
            match (map.remove("result"), map.remove("error")) {
                (Some(result), None) => Incoming::Reply {
                    id,
                    answer: Ok(result),
                },
                (None, Some(error)) => Incoming::Reply {
                    id,
                    answer: Err(error),
                },
                _ => broken("a reply needs exactly one of `result` and `error`"),
            }
        }
        (None, None) => broken("a message is neither a reply nor a notification"),
    }
}

/// 一直读 stderr，只留最后 `keep` 行。读错了就停在那里。
async fn read_stderr(
    mut reader: impl AsyncRead + Unpin,
    keep: usize,
    tail: Arc<StdMutex<VecDeque<String>>>,
) {
    let mut line = Vec::new();
    let mut chunk = [0u8; 8192];
    let finish = |line: &mut Vec<u8>| {
        let text = String::from_utf8_lossy(line)
            .trim_end_matches('\r')
            .to_owned();
        line.clear();
        if keep == 0 {
            return;
        }
        let mut tail = tail.lock().unwrap_or_else(PoisonError::into_inner);
        if tail.len() >= keep {
            tail.pop_front();
        }
        tail.push_back(text);
    };
    loop {
        let n = match reader.read(&mut chunk).await {
            Ok(0) => break,
            Ok(n) => n,
            Err(e) => {
                tracing::debug!("read transform stderr: {e}");
                break;
            }
        };
        for &b in &chunk[..n] {
            if b == b'\n' {
                finish(&mut line);
            } else if line.len() < STDERR_LINE_BYTES {
                line.push(b);
            }
        }
    }
    if !line.is_empty() {
        finish(&mut line);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn language_tags() {
        for ok in ["zh", "en-US", "zh-Hans-CN", "yue"] {
            assert!(is_language_tag(ok), "{ok}");
        }
        for bad in [
            "",
            "zh_CN",
            "zh--CN",
            "-zh",
            "e\nn",
            &"a".repeat(36),
            "toolongpart",
        ] {
            assert!(!is_language_tag(bad), "{bad}");
        }
    }

    #[test]
    fn messages_are_sorted_out() {
        let reply = parse_incoming(br#"{"jsonrpc":"2.0","id":3,"result":{"ready":true}}"#);
        assert!(matches!(
            reply,
            Incoming::Reply {
                id: RpcId::Number(3),
                answer: Ok(_)
            }
        ));
        let error = parse_incoming(br#"{"jsonrpc":"2.0","id":"a","error":{"code":-32000}}"#);
        assert!(matches!(error, Incoming::Reply { answer: Err(_), .. }));
        let progress =
            parse_incoming(br#"{"jsonrpc":"2.0","method":"progress","params":{"id":3,"done":1}}"#);
        assert!(matches!(progress, Incoming::Progress(RpcId::Number(3))));
        let other = parse_incoming(br#"{"jsonrpc":"2.0","method":"log","params":{}}"#);
        assert!(matches!(other, Incoming::Ignored));
        for bad in [
            &b"hello"[..],
            br#"[1]"#,
            br#"{"id":1,"result":1}"#,
            br#"{"jsonrpc":"2.0","id":1,"result":1,"error":{}}"#,
            br#"{"jsonrpc":"2.0","id":1,"method":"run"}"#,
            br#"{"jsonrpc":"2.0","id":null,"error":{"code":-32700}}"#,
            br#"{"jsonrpc":"2.0","method":"progress","params":{"done":1}}"#,
        ] {
            assert!(
                matches!(parse_incoming(bad), Incoming::Broken(_)),
                "{}",
                String::from_utf8_lossy(bad)
            );
        }
    }

    #[test]
    fn frames_have_a_fixed_field_order() {
        let frame = request_frame(&RpcId::Number(7), METHOD_HEALTH, None::<()>).unwrap();
        assert_eq!(
            frame,
            b"{\"jsonrpc\":\"2.0\",\"id\":7,\"method\":\"health\"}\n"
        );
    }
}
