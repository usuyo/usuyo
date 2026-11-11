//! 转换插件那一侧的 JSON-RPC 循环（协议 §4.1、§4.2）。
//!
//! 作者只写 `run` 处理函数（实现 [`Transform`]，或直接交一个闭包）和一份 [`Describe`]；
//! 分帧、`describe`、`health`、`cancel`、`progress`、错误码都由这里管：
//!
//! ```no_run
//! use usuyo_plugin_kit::protocol::{Describe, RunParams, RunResult};
//! use usuyo_plugin_kit::transform::{self, RunContext};
//! use usuyo_plugin_kit::protocol::CallFailure;
//!
//! fn run(params: RunParams, ctx: &RunContext) -> Result<RunResult, CallFailure> {
//!     ctx.check_cancelled()?;
//!     todo!("read params.inputs, write into params.out")
//! }
//!
//! fn main() -> std::process::ExitCode {
//!     let version = env!("CARGO_PKG_VERSION").parse().expect("Cargo version is semver");
//!     match transform::serve(Describe::new(version, Vec::new()), run) {
//!         Ok(()) => std::process::ExitCode::SUCCESS,
//!         Err(e) => {
//!             eprintln!("{e}");
//!             std::process::ExitCode::FAILURE
//!         }
//!     }
//! }
//! ```
//!
//! - 一行一条消息，最长 1 MiB；更长的回 `-32600`，丢掉这一行接着读。
//! - 同一时间只跑一个 `run`，在自己的线程里；`describe`、`health`、`cancel` 跑着的时候照样收。
//! - 收到 `cancel` 以后，处理函数返回什么都回 `cancelled`。
//! - stdin 关了（宿主要插件退出）：跑着的 `run` 收到取消，最多等 4 秒，然后返回。

use std::io::{self, BufRead, Write};
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver, RecvTimeoutError};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
use std::thread;
use std::time::Duration;

use serde::Serialize;
use serde_json::{Value, json};
use usuyo_plugin::protocol::{
    CallFailure, Cancel, Describe, ErrorCode, Health, METHOD_CANCEL, METHOD_DESCRIBE,
    METHOD_HEALTH, METHOD_PROGRESS, METHOD_RUN, Progress, RpcError, RpcId, RunParams, RunResult,
};

/// 一条消息最多多少字节（不算换行）。`run` 的参数只有路径和设置，用不了这么多。
const MAX_FRAME: usize = 1 << 20;

/// stdin 关了以后等跑着的 `run` 收尾多久。宿主关 stdin 后等 5 秒才发 SIGTERM（协议 §4.1）。
const STOP_GRACE: Duration = Duration::from_secs(4);

/// 一个转换插件。闭包 `Fn(RunParams, &RunContext) -> Result<RunResult, CallFailure>` 也算。
pub trait Transform: Send + Sync + 'static {
    /// 处理一条。只写 `params.out`；长的活隔一阵查一次 [`RunContext::check_cancelled`]。
    fn run(&self, params: RunParams, ctx: &RunContext) -> Result<RunResult, CallFailure>;

    /// `health` 的回答；默认什么都不查、能接调用。
    fn health(&self) -> Health {
        Health {
            ready: true,
            checks: Vec::new(),
        }
    }
}

impl<F> Transform for F
where
    F: Fn(RunParams, &RunContext) -> Result<RunResult, CallFailure> + Send + Sync + 'static,
{
    fn run(&self, params: RunParams, ctx: &RunContext) -> Result<RunResult, CallFailure> {
        self(params, ctx)
    }
}

/// 循环自己出的错：读写 stdio 失败，或 stdin 关了以后 `run` 不肯停。
#[derive(Debug, thiserror::Error)]
pub enum ServeError {
    #[error("stdio: {0}")]
    Io(#[from] io::Error),
    #[error("stdin closed and the running `run` did not stop within {0:?}")]
    RunDidNotStop(Duration),
}

/// 一次 `run` 拿到的：取消标志和发 `progress` 的口子。
pub struct RunContext {
    id: RpcId,
    cancelled: Arc<AtomicBool>,
    out: Arc<Output>,
}

impl RunContext {
    /// 宿主发过 `cancel`，或 stdin 已经关了。
    pub fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::SeqCst)
    }

    /// 取消了就回 `Err(cancelled)`，处理函数用 `?` 收尾。
    pub fn check_cancelled(&self) -> Result<(), CallFailure> {
        if self.is_cancelled() {
            Err(CallFailure::new(ErrorCode::Cancelled, "cancelled"))
        } else {
            Ok(())
        }
    }

    /// 发一条 `progress`。宿主只拿它判断卡住（120 秒没有新的算卡住），不往界面报。
    /// 写不出去时不打扰处理函数：错误留着，循环结束时由 [`serve`] 返回。
    pub fn progress(&self, done: f64, total: f64, unit: &str) {
        let progress = Progress {
            id: self.id.clone(),
            done: Some(done),
            total: Some(total),
            unit: Some(unit.to_owned()),
            message: None,
        };
        let note = json!({"jsonrpc": "2.0", "method": METHOD_PROGRESS, "params": progress});
        if let Err(e) = self.out.send(&note) {
            self.out.remember(e);
        }
    }
}

/// 在 stdin、stdout 上跑循环，直到 stdin 关掉。
pub fn serve(describe: Describe, plugin: impl Transform) -> Result<(), ServeError> {
    serve_on(describe, plugin, io::stdin().lock(), io::stdout())
}

/// 同 [`serve`]，读写给定的流；插件自己的测试用内存里的管道驱动它。
pub fn serve_on<R, W>(
    describe: Describe,
    plugin: impl Transform,
    mut input: R,
    output: W,
) -> Result<(), ServeError>
where
    R: BufRead,
    W: Write + Send + 'static,
{
    let plugin: Arc<dyn Transform> = Arc::new(plugin);
    let out = Arc::new(Output {
        writer: Mutex::new(Box::new(output)),
        failed: Mutex::new(None),
    });
    let running: Arc<Mutex<Option<Running>>> = Arc::new(Mutex::new(None));
    // 最近一个处理线程写完回答时发来的信号。
    let mut last_done: Option<Receiver<()>> = None;
    loop {
        let frame = read_frame(&mut input, MAX_FRAME)?;
        out.take_failure()?;
        let line = match frame {
            Frame::Eof => return stop(&running, last_done, &out),
            Frame::TooLong => {
                out.send(&error(
                    None,
                    RpcError::protocol(
                        RpcError::INVALID_REQUEST,
                        format!("message longer than {MAX_FRAME} bytes"),
                    ),
                ))?;
                continue;
            }
            Frame::Line(line) => line,
        };
        if line.iter().all(u8::is_ascii_whitespace) {
            continue;
        }
        let message = match parse(&line) {
            Ok(m) => m,
            Err((id, e)) => {
                out.send(&error(id.as_ref(), e))?;
                continue;
            }
        };
        let Some(id) = message.id else {
            // 通知没有回答；只认 `cancel`。
            if message.method == METHOD_CANCEL
                && let Ok(c) = serde_json::from_value::<Cancel>(message.params)
                && let Some(r) = lock(&running).as_ref()
                && c.id == r.id
            {
                r.cancelled.store(true, Ordering::SeqCst);
            }
            continue;
        };
        let reply = match message.method.as_str() {
            METHOD_DESCRIBE => ok(&id, &describe),
            METHOD_HEALTH => match catch_unwind(AssertUnwindSafe(|| plugin.health())) {
                Ok(h) => ok(&id, &h),
                Err(_) => failure(&id, internal("the health handler panicked")),
            },
            METHOD_RUN => match start_run(&id, message.params, &plugin, &out, &running) {
                Ok(done) => {
                    last_done = Some(done);
                    continue;
                }
                Err(e) => error(Some(&id), e),
            },
            METHOD_CANCEL => error(
                Some(&id),
                RpcError::protocol(RpcError::INVALID_REQUEST, "`cancel` is a notification"),
            ),
            other => error(
                Some(&id),
                RpcError::protocol(
                    RpcError::METHOD_NOT_FOUND,
                    format!("no method `{}`", truncate(other)),
                ),
            ),
        };
        out.send(&reply)?;
    }
}

/// 正在跑的那个 `run`。处理线程在写回答之前把它清掉，宿主收到回答后马上发的下一个 `run` 不会撞上它。
struct Running {
    id: RpcId,
    cancelled: Arc<AtomicBool>,
}

/// 起一个处理线程；回的是它写完回答时的信号。
fn start_run(
    id: &RpcId,
    params: Value,
    plugin: &Arc<dyn Transform>,
    out: &Arc<Output>,
    running: &Arc<Mutex<Option<Running>>>,
) -> Result<Receiver<()>, RpcError> {
    let params: RunParams = serde_json::from_value(params)
        .map_err(|e| RpcError::protocol(RpcError::INVALID_PARAMS, format!("run params: {e}")))?;
    if params.want.is_empty() {
        return Err(RpcError::protocol(
            RpcError::INVALID_PARAMS,
            "run params: `want` is empty",
        ));
    }
    let mut slot = lock(running);
    if slot.is_some() {
        return Err(RpcError::failure(CallFailure::new(
            ErrorCode::InvalidArgs,
            "a `run` is already in progress; send one at a time",
        )));
    }
    let cancelled = Arc::new(AtomicBool::new(false));
    let ctx = RunContext {
        id: id.clone(),
        cancelled: Arc::clone(&cancelled),
        out: Arc::clone(out),
    };
    let (tx, done) = mpsc::channel();
    let plugin = Arc::clone(plugin);
    let out = Arc::clone(out);
    let reply_id = id.clone();
    let finished = Arc::clone(running);
    thread::Builder::new()
        .name("usuyo-transform-run".into())
        .spawn(move || {
            let want = params.want.clone();
            let inputs = params.inputs.len();
            let outcome = catch_unwind(AssertUnwindSafe(|| plugin.run(params, &ctx)));
            let reply = match outcome {
                _ if ctx.is_cancelled() => failure(
                    &reply_id,
                    CallFailure::new(ErrorCode::Cancelled, "cancelled"),
                ),
                Ok(Ok(result)) => match result.problem(&want, inputs) {
                    None => ok(&reply_id, &result),
                    Some(why) => failure(&reply_id, internal(&format!("run result: {why}"))),
                },
                Ok(Err(f)) => failure(&reply_id, from_handler(f)),
                Err(_) => failure(&reply_id, internal("the run handler panicked")),
            };
            lock(&finished).take();
            if let Err(e) = out.send(&reply) {
                out.remember(e);
            }
            // 循环等不及已经返回时没人收这个信号；回答已经写过了。
            let _ = tx.send(());
        })
        .map_err(|e| RpcError::failure(internal(&format!("start the run thread: {e}"))))?;
    *slot = Some(Running {
        id: id.clone(),
        cancelled,
    });
    Ok(done)
}

/// stdin 关了：取消跑着的，等最近一个回答写完。
fn stop(
    running: &Mutex<Option<Running>>,
    last_done: Option<Receiver<()>>,
    out: &Output,
) -> Result<(), ServeError> {
    if let Some(r) = lock(running).as_ref() {
        r.cancelled.store(true, Ordering::SeqCst);
    }
    if let Some(done) = last_done {
        match done.recv_timeout(STOP_GRACE) {
            Ok(()) | Err(RecvTimeoutError::Disconnected) => {}
            Err(RecvTimeoutError::Timeout) => return Err(ServeError::RunDidNotStop(STOP_GRACE)),
        }
    }
    out.take_failure()?;
    Ok(())
}

/// 锁里只有简单的值，处理线程的 panic 已经接住，锁坏了照样用里面的值。
fn lock<T>(m: &Mutex<T>) -> MutexGuard<'_, T> {
    m.lock().unwrap_or_else(PoisonError::into_inner)
}

/// 处理函数报的错照协议给宿主：宿主才用的码、缺 `needs` 的 `dependency-missing` 改成 `internal`，
/// 免得宿主把整次当成插件输出不合规。
fn from_handler(f: CallFailure) -> CallFailure {
    if f.code.is_host_only() {
        return internal(&format!(
            "the handler reported host-only code `{}`: {}",
            f.code, f.message
        ));
    }
    if f.code == ErrorCode::DependencyMissing && f.needs.is_empty() {
        return internal(&format!(
            "the handler reported `dependency-missing` without `needs`: {}",
            f.message
        ));
    }
    f
}

fn internal(message: &str) -> CallFailure {
    CallFailure::new(ErrorCode::Internal, message)
}

/// 一条读进来的消息。
struct Message {
    id: Option<RpcId>,
    method: String,
    params: Value,
}

/// 解析一行。出错时带上能认出的 `id`（认不出是 `None`，回 `null`）。
fn parse(line: &[u8]) -> Result<Message, (Option<RpcId>, RpcError)> {
    let value: Value = serde_json::from_slice(line).map_err(|e| {
        (
            None,
            RpcError::protocol(RpcError::PARSE_ERROR, format!("not JSON: {e}")),
        )
    })?;
    let invalid = |id: Option<RpcId>, m: &str| {
        (
            id,
            RpcError::protocol(RpcError::INVALID_REQUEST, m.to_owned()),
        )
    };
    let Value::Object(mut map) = value else {
        return Err(invalid(
            None,
            "a message is one JSON object; batches are not supported",
        ));
    };
    let id = match map.remove("id") {
        None => None,
        Some(raw) => match serde_json::from_value::<RpcId>(raw) {
            Ok(id) => Some(id),
            Err(_) => return Err(invalid(None, "`id` must be an integer or a string")),
        },
    };
    if map.get("jsonrpc").and_then(Value::as_str) != Some("2.0") {
        return Err(invalid(id, "`jsonrpc` must be \"2.0\""));
    }
    let Some(Value::String(method)) = map.remove("method") else {
        return Err(invalid(id, "no string `method`"));
    };
    Ok(Message {
        id,
        method,
        params: map.remove("params").unwrap_or(Value::Null),
    })
}

fn ok<T: Serialize>(id: &RpcId, result: &T) -> Value {
    match serde_json::to_value(result) {
        Ok(result) => json!({"jsonrpc": "2.0", "id": id, "result": result}),
        Err(e) => failure(id, internal(&format!("encode the result: {e}"))),
    }
}

fn failure(id: &RpcId, f: CallFailure) -> Value {
    error(Some(id), RpcError::failure(f))
}

fn error(id: Option<&RpcId>, e: RpcError) -> Value {
    json!({"jsonrpc": "2.0", "id": id, "error": e})
}

/// 回给宿主的话里引用它发来的名字时只留一段。
fn truncate(s: &str) -> String {
    s.chars().take(64).collect()
}

/// stdout：一次写一整行，几个线程轮流用。
struct Output {
    writer: Mutex<Box<dyn Write + Send>>,
    /// `progress` 写失败时记下的第一个错误。
    failed: Mutex<Option<io::Error>>,
}

impl Output {
    fn send(&self, value: &Value) -> io::Result<()> {
        let mut line = serde_json::to_vec(value)?;
        line.push(b'\n');
        let mut w = lock(&self.writer);
        w.write_all(&line)?;
        w.flush()
    }

    fn remember(&self, e: io::Error) {
        lock(&self.failed).get_or_insert(e);
    }

    fn take_failure(&self) -> io::Result<()> {
        lock(&self.failed).take().map_or(Ok(()), Err)
    }
}

enum Frame {
    Line(Vec<u8>),
    /// 超过上限的一行，已经读掉丢了。
    TooLong,
    Eof,
}

/// 读到换行或 EOF；超过 `max` 的部分不留在内存里。
fn read_frame(r: &mut impl BufRead, max: usize) -> io::Result<Frame> {
    let mut line = Vec::new();
    let mut too_long = false;
    let mut read_any = false;
    loop {
        let buf = match r.fill_buf() {
            Ok(b) => b,
            Err(e) if e.kind() == io::ErrorKind::Interrupted => continue,
            Err(e) => return Err(e),
        };
        if buf.is_empty() {
            return Ok(match (read_any, too_long) {
                (false, _) => Frame::Eof,
                (true, true) => Frame::TooLong,
                (true, false) => Frame::Line(line),
            });
        }
        read_any = true;
        let newline = buf.iter().position(|&b| b == b'\n');
        let chunk = &buf[..newline.unwrap_or(buf.len())];
        if !too_long {
            if line.len() + chunk.len() > max {
                too_long = true;
                line = Vec::new();
            } else {
                line.extend_from_slice(chunk);
            }
        }
        let used = newline.map_or(buf.len(), |i| i + 1);
        r.consume(used);
        if newline.is_some() {
            if line.last() == Some(&b'\r') {
                line.pop();
            }
            return Ok(if too_long {
                Frame::TooLong
            } else {
                Frame::Line(line)
            });
        }
    }
}

#[cfg(test)]
mod tests;
