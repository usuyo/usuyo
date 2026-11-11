//! 用内存里的管道驱动循环：一端当宿主写请求，另一端读回答。

use std::io::{BufRead, BufReader, PipeWriter, Write};
use std::sync::mpsc::{self, Receiver};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use serde_json::{Value, json};
use usuyo_plugin::protocol::{
    CallFailure, Describe, ErrorCode, Need, Produce, RelPath, RunParams, RunProduct, RunResult,
    UpstreamVersion,
};

use super::{MAX_FRAME, RunContext, STOP_GRACE, ServeError, Transform, serve_on};

/// 等一条回答最多多久；测试里没有哪一步该这么慢。
const WAIT: Duration = Duration::from_secs(10);

struct Host {
    to_plugin: Option<PipeWriter>,
    from_plugin: Receiver<Value>,
    serving: JoinHandle<Result<(), ServeError>>,
}

impl Host {
    fn start(plugin: impl Transform) -> Self {
        let (in_read, in_write) = std::io::pipe().unwrap();
        let (out_read, out_write) = std::io::pipe().unwrap();
        let describe = Describe::new("0.1.0".parse().unwrap(), vec!["sense-voice".into()]);
        let serving =
            thread::spawn(move || serve_on(describe, plugin, BufReader::new(in_read), out_write));
        let (tx, from_plugin) = mpsc::channel();
        thread::spawn(move || {
            for line in BufReader::new(out_read).lines() {
                let Ok(line) = line else { break };
                let value: Value = serde_json::from_str(&line).expect("plugin wrote JSON");
                if tx.send(value).is_err() {
                    break;
                }
            }
        });
        Host {
            to_plugin: Some(in_write),
            from_plugin,
            serving,
        }
    }

    fn send_raw(&mut self, bytes: &[u8]) {
        let w = self.to_plugin.as_mut().unwrap();
        w.write_all(bytes).unwrap();
        w.flush().unwrap();
    }

    fn send(&mut self, message: Value) {
        let mut line = serde_json::to_vec(&message).unwrap();
        line.push(b'\n');
        self.send_raw(&line);
    }

    fn next(&self) -> Value {
        self.from_plugin
            .recv_timeout(WAIT)
            .expect("no message from the plugin in time")
    }

    /// 关掉 stdin，等循环返回。
    fn close(mut self) -> Result<(), ServeError> {
        drop(self.to_plugin.take());
        let deadline = Instant::now() + WAIT;
        while !self.serving.is_finished() {
            assert!(Instant::now() < deadline, "the loop did not end after EOF");
            thread::sleep(Duration::from_millis(10));
        }
        self.serving.join().unwrap()
    }
}

fn params(want: &str) -> Value {
    json!({
        "item": {"id": "bilibili-4plrcxk343onmyxibmrr672kga", "kind": "video", "title": "手冲咖啡"},
        "want": [want],
        "inputs": [{"kind": "audio:pcm16k", "mime": "audio/wav", "path": "/tmp/p/in/1/audio.wav", "snapshot": "019a6f2b-4a10-7c3e-8b21-0d9e8f7a6b5c"}],
        "out": "/tmp/p/out/1",
        "options": {"vadThreshold": 0.4},
        "provider": null
    })
}

fn run(id: Value, want: &str) -> Value {
    json!({"jsonrpc": "2.0", "id": id, "method": "run", "params": params(want)})
}

fn asr_result(p: &RunParams) -> RunResult {
    let mut options = p.options.clone();
    options.insert("language".into(), json!("auto"));
    RunResult {
        outputs: vec![RunProduct {
            kind: Produce::Asr,
            path: RelPath::try_from("asr.vtt").unwrap(),
            lang: Some("zh".into()),
        }],
        tool: UpstreamVersion {
            name: "sherpa-onnx".into(),
            version: "2.0.1".into(),
        },
        options,
        model: None,
        marks: Vec::new(),
        empty: false,
        reason: None,
        probe: Vec::new(),
    }
}

fn plugin_failure(reply: &Value) -> CallFailure {
    assert_eq!(reply["error"]["code"], json!(-32000), "{reply}");
    serde_json::from_value(reply["error"]["data"].clone()).unwrap()
}

#[test]
fn describe_health_run_result() {
    let mut host = Host::start(|p: RunParams, _: &RunContext| Ok(asr_result(&p)));
    host.send(json!({"jsonrpc": "2.0", "id": 1, "method": "describe"}));
    let d = host.next();
    assert_eq!(d["id"], json!(1));
    assert_eq!(
        d["result"],
        json!({"protocol": 1, "version": "0.1.0", "models": ["sense-voice"]})
    );
    host.send(json!({"jsonrpc": "2.0", "id": "h", "method": "health", "params": {}}));
    assert_eq!(
        host.next(),
        json!({"jsonrpc": "2.0", "id": "h", "result": {"ready": true, "checks": []}})
    );
    host.send(run(json!(7), "asr"));
    let r = host.next();
    assert_eq!(r["id"], json!(7));
    assert_eq!(r["result"]["outputs"][0]["kind"], json!("asr"));
    assert_eq!(r["result"]["tool"]["name"], json!("sherpa-onnx"));
    assert_eq!(
        r["result"]["options"],
        json!({"vadThreshold": 0.4, "language": "auto"})
    );
    // 宿主收到回答马上发下一个：不算正在跑
    host.send(run(json!(8), "asr"));
    assert!(host.next()["result"].is_object());
    host.close().unwrap();
}

#[test]
fn cancel_during_a_run() {
    let plugin = |_: RunParams, ctx: &RunContext| {
        let deadline = Instant::now() + WAIT;
        let mut done = 0.0;
        while Instant::now() < deadline {
            ctx.check_cancelled()?;
            ctx.progress(done, 100.0, "%");
            done += 1.0;
            thread::sleep(Duration::from_millis(20));
        }
        Err(CallFailure::new(ErrorCode::Internal, "never cancelled"))
    };
    let mut host = Host::start(plugin);
    host.send(run(json!(3), "asr"));
    let progress = host.next();
    assert_eq!(progress["method"], json!("progress"));
    assert_eq!(progress["params"]["id"], json!(3));
    assert!(progress.get("id").is_none(), "progress is a notification");

    // 跑着的时候 describe 照样回；第二个 run 不收
    host.send(json!({"jsonrpc": "2.0", "id": 4, "method": "describe"}));
    host.send(run(json!(5), "asr"));
    let mut replies = Vec::new();
    while replies.len() < 2 {
        let m = host.next();
        if m.get("id").is_some() {
            replies.push(m);
        }
    }
    assert_eq!(replies[0]["id"], json!(4));
    assert_eq!(replies[1]["id"], json!(5));
    assert_eq!(plugin_failure(&replies[1]).code, ErrorCode::InvalidArgs);

    // 别的 id 的 cancel 不算
    host.send(json!({"jsonrpc": "2.0", "method": "cancel", "params": {"id": 99}}));
    host.send(json!({"jsonrpc": "2.0", "method": "cancel", "params": {"id": 3}}));
    let reply = loop {
        let m = host.next();
        if m.get("id").is_some() {
            break m;
        }
    };
    assert_eq!(reply["id"], json!(3));
    assert_eq!(plugin_failure(&reply).code, ErrorCode::Cancelled);
    host.close().unwrap();
}

#[test]
fn a_cancelled_run_answers_cancelled_even_when_it_finishes() {
    let (started, wait_started) = mpsc::channel();
    let plugin = move |p: RunParams, ctx: &RunContext| {
        started.send(()).unwrap();
        while !ctx.is_cancelled() {
            thread::sleep(Duration::from_millis(5));
        }
        Ok(asr_result(&p))
    };
    let mut host = Host::start(plugin);
    host.send(run(json!(1), "asr"));
    wait_started.recv_timeout(WAIT).unwrap();
    host.send(json!({"jsonrpc": "2.0", "method": "cancel", "params": {"id": 1}}));
    assert_eq!(plugin_failure(&host.next()).code, ErrorCode::Cancelled);
    host.close().unwrap();
}

#[test]
fn malformed_and_oversized_frames() {
    let mut host = Host::start(|p: RunParams, _: &RunContext| Ok(asr_result(&p)));
    let code = |m: &Value| m["error"]["code"].as_i64().unwrap();

    host.send_raw(b"not json\n");
    let m = host.next();
    assert_eq!((code(&m), &m["id"]), (-32700, &Value::Null));

    host.send_raw(b"\xff\xfe\n");
    assert_eq!(code(&host.next()), -32700);

    host.send(json!([{"jsonrpc": "2.0", "id": 1, "method": "describe"}]));
    assert_eq!(code(&host.next()), -32600);

    host.send(json!({"jsonrpc": "1.0", "id": 2, "method": "describe"}));
    let m = host.next();
    assert_eq!((code(&m), &m["id"]), (-32600, &json!(2)));

    host.send(json!({"jsonrpc": "2.0", "id": {"x": 1}, "method": "describe"}));
    assert_eq!(code(&host.next()), -32600);

    host.send(json!({"jsonrpc": "2.0", "id": 3}));
    assert_eq!(code(&host.next()), -32600);

    host.send(json!({"jsonrpc": "2.0", "id": 4, "method": "readback", "params": {}}));
    assert_eq!(code(&host.next()), -32601);

    host.send(json!({"jsonrpc": "2.0", "id": 5, "method": "run", "params": {"want": []}}));
    assert_eq!(code(&host.next()), -32602);

    let mut empty_want = run(json!(6), "asr");
    empty_want["params"]["want"] = json!([]);
    host.send(empty_want);
    assert_eq!(code(&host.next()), -32602);

    let mut old_kind = run(json!(7), "asr");
    old_kind["params"]["want"] = json!(["derived/asr.vtt"]);
    host.send(old_kind);
    assert_eq!(code(&host.next()), -32602);

    host.send(json!({"jsonrpc": "2.0", "id": 8, "method": "cancel", "params": {"id": 1}}));
    assert_eq!(code(&host.next()), -32600);

    // 不认识的通知、空行不回
    host.send(json!({"jsonrpc": "2.0", "method": "shutdown"}));
    host.send_raw(b"\n\r\n");

    let mut big = vec![b' '; MAX_FRAME + 1];
    big.push(b'\n');
    host.send_raw(&big);
    let m = host.next();
    assert_eq!((code(&m), &m["id"]), (-32600, &Value::Null));

    // 之后照常
    host.send(json!({"jsonrpc": "2.0", "id": 9, "method": "describe"}));
    assert_eq!(host.next()["id"], json!(9));

    // 最后一行没有换行也算一条，随后 EOF
    host.send_raw(br#"{"jsonrpc": "2.0", "id": 10, "method": "describe"}"#);
    let m = {
        drop(host.to_plugin.take());
        host.next()
    };
    assert_eq!(m["id"], json!(10));
    host.close().unwrap();
}

#[test]
fn handler_errors_become_protocol_errors() {
    let plugin = |p: RunParams, _: &RunContext| -> Result<RunResult, CallFailure> {
        match p.options.get("case").and_then(Value::as_str) {
            Some("model") => {
                let mut f = CallFailure::new(ErrorCode::DependencyMissing, "no model file");
                f.needs =
                    vec![Need::try_from("model:sense-voice-int8-2025-09-09".to_owned()).unwrap()];
                Err(f)
            }
            Some("host-code") => Err(CallFailure::new(ErrorCode::Timeout, "took too long")),
            Some("no-needs") => Err(CallFailure::new(ErrorCode::DependencyMissing, "?")),
            Some("panic") => panic!("handler bug"),
            Some("bad-result") => {
                let mut r = asr_result(&p);
                r.empty = true;
                Ok(r)
            }
            _ => Err(CallFailure::new(ErrorCode::UnsupportedInput, "not audio")),
        }
    };
    let mut host = Host::start(plugin);
    let mut call = |id: i64, case: &str| {
        let mut m = run(json!(id), "asr");
        m["params"]["options"] = json!({"case": case});
        host.send(m);
        let reply = host.next();
        assert_eq!(reply["id"], json!(id));
        plugin_failure(&reply)
    };
    let f = call(1, "model");
    assert_eq!(f.code, ErrorCode::DependencyMissing);
    assert_eq!(f.needs[0].as_str(), "model:sense-voice-int8-2025-09-09");
    assert_eq!(call(2, "other").code, ErrorCode::UnsupportedInput);
    for (id, case) in [
        (3, "host-code"),
        (4, "no-needs"),
        (5, "panic"),
        (6, "bad-result"),
    ] {
        assert_eq!(call(id, case).code, ErrorCode::Internal, "{case}");
    }
    host.close().unwrap();
}

#[test]
fn eof_ends_the_loop() {
    let host = Host::start(|p: RunParams, _: &RunContext| Ok(asr_result(&p)));
    host.close().unwrap();

    // 跑着的时候关了 stdin：取消它，等它回完
    let (started, wait_started) = mpsc::channel();
    let plugin = move |_: RunParams, ctx: &RunContext| {
        started.send(()).unwrap();
        loop {
            ctx.check_cancelled()?;
            thread::sleep(Duration::from_millis(5));
        }
    };
    let mut host = Host::start(plugin);
    host.send(run(json!(1), "asr"));
    wait_started.recv_timeout(WAIT).unwrap();
    drop(host.to_plugin.take());
    assert_eq!(plugin_failure(&host.next()).code, ErrorCode::Cancelled);
    host.close().unwrap();
}

#[test]
fn a_run_that_ignores_cancel_does_not_hold_the_loop() {
    let (started, wait_started) = mpsc::channel();
    let plugin = move |p: RunParams, _: &RunContext| {
        started.send(()).unwrap();
        thread::sleep(STOP_GRACE + Duration::from_secs(2));
        Ok(asr_result(&p))
    };
    let mut host = Host::start(plugin);
    host.send(run(json!(1), "asr"));
    wait_started.recv_timeout(WAIT).unwrap();
    let begun = Instant::now();
    assert!(matches!(host.close(), Err(ServeError::RunDidNotStop(_))));
    assert!(begun.elapsed() < STOP_GRACE + Duration::from_secs(1));
}
