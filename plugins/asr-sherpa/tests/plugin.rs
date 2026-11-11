//! 起插件的可执行文件，在 stdio 上说 JSON-RPC，和宿主一样。
//!
//! 要模型的测试找环境变量 `USUYO_TEST_SHERPA_DIR` 指的目录，照宿主的样子摆：
//! `<目录>/sense-voice-int8-2024-07-17/sherpa-onnx-sense-voice-zh-en-ja-ko-yue-int8-2024-07-17/`（压缩包原样解开）
//! 和 `<目录>/silero_vad_v5.onnx`。没设就跳过，打印原因。样例在 `crates/usuyo-core/testdata/transform/`，
//! 用 Homebrew 的 ffmpeg 转成 WAV；没有 ffmpeg 也跳过。

use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, Command, ExitStatus, Stdio};
use std::sync::mpsc::{self, Receiver};
use std::time::Duration;

use serde_json::{Value, json};
use usuyo_plugin_asr_sherpa::engine::{MODEL_NAME, VAD_FILE};

const FFMPEG: &str = "/opt/homebrew/bin/ffmpeg";
const DIR_VAR: &str = "USUYO_TEST_SHERPA_DIR";
/// 一次回答最多等多久；转一段 2 分钟的音频在这台要十来秒。
const WAIT: Duration = Duration::from_secs(120);

fn samples() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../crates/usuyo-core/testdata/transform")
}

/// 测试要的模型目录；没设时打印原因，回 `None`。
fn model_dir(test: &str) -> Option<PathBuf> {
    match std::env::var_os(DIR_VAR).filter(|v| !v.is_empty()) {
        Some(dir) => Some(PathBuf::from(dir)),
        None => {
            eprintln!("skip {test}: {DIR_VAR} is not set (see the README)");
            None
        }
    }
}

fn have_ffmpeg(test: &str) -> bool {
    let ok = Path::new(FFMPEG).is_file();
    if !ok {
        eprintln!("skip {test}: no ffmpeg at {FFMPEG}");
    }
    ok
}

/// 用 ffmpeg 把样例转成 WAV；`before` 放在 `-i` 前面（例如 `-stream_loop`）。
fn to_wav(src: &str, dst: &Path, before: &[&str], rate: u32) {
    let status = Command::new(FFMPEG)
        .args(["-hide_banner", "-loglevel", "error", "-y"])
        .args(before)
        .arg("-i")
        .arg(samples().join(src))
        .args(["-ac", "1", "-ar", &rate.to_string(), "-c:a", "pcm_s16le"])
        .arg(dst)
        .status()
        .expect("run ffmpeg");
    assert!(status.success(), "ffmpeg {src}");
}

/// 一个插件目录，里面放一份测试目录的 VAD。
fn plugin_dir(root: &Path, vad: &Path) -> PathBuf {
    let dir = root.join("plugin");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::copy(vad, dir.join(VAD_FILE)).unwrap();
    dir
}

/// 起着的插件进程。
struct Plugin {
    child: Child,
    stdin: Option<ChildStdin>,
    lines: Receiver<Result<Value, String>>,
    next_id: i64,
}

impl Plugin {
    fn start(plugin_dir: &Path, models_dir: &Path) -> Self {
        let mut child = Command::new(env!("CARGO_BIN_EXE_usuyo-plugin-asr-sherpa"))
            .env_clear()
            .env("USUYO_PLUGIN_DIR", plugin_dir)
            .env("USUYO_MODELS_DIR", models_dir)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()
            .expect("start the plugin");
        let stdout = child.stdout.take().unwrap();
        let (tx, lines) = mpsc::channel();
        std::thread::spawn(move || {
            for line in BufReader::new(stdout).lines() {
                let parsed = match line {
                    Ok(l) => serde_json::from_str::<Value>(&l)
                        .map_err(|e| format!("stdout line is not JSON ({e}): {l}"))
                        .and_then(|v| {
                            if v["jsonrpc"] == "2.0" {
                                Ok(v)
                            } else {
                                Err(format!("not JSON-RPC 2.0: {l}"))
                            }
                        }),
                    Err(e) => Err(format!("read stdout: {e}")),
                };
                if tx.send(parsed).is_err() {
                    return;
                }
            }
        });
        Self {
            stdin: child.stdin.take(),
            child,
            lines,
            next_id: 1,
        }
    }

    fn send(&mut self, message: &Value) {
        let stdin = self.stdin.as_mut().expect("stdin open");
        writeln!(stdin, "{message}").unwrap();
        stdin.flush().unwrap();
    }

    fn next_line(&self) -> Value {
        self.lines
            .recv_timeout(WAIT)
            .expect("an answer within the wait")
            .unwrap_or_else(|e| panic!("{e}"))
    }

    fn request(&mut self, method: &str, params: Value) -> i64 {
        let id = self.next_id;
        self.next_id += 1;
        self.send(&json!({"jsonrpc": "2.0", "id": id, "method": method, "params": params}));
        id
    }

    /// 等这个 `id` 的回答；中间的 `progress` 收进第二个返回值。
    fn answer(&self, id: i64) -> (Value, Vec<Value>) {
        let mut progress = Vec::new();
        loop {
            let line = self.next_line();
            if line["method"] == "progress" {
                assert_eq!(line["params"]["id"], id, "{line}");
                progress.push(line["params"].clone());
                continue;
            }
            assert_eq!(line["id"], id, "{line}");
            return (line, progress);
        }
    }

    fn call(&mut self, method: &str, params: Value) -> Value {
        let id = self.request(method, params);
        self.answer(id).0
    }

    /// 关 stdin，等进程退出。
    fn close(mut self) -> ExitStatus {
        self.stdin.take();
        self.child.wait().unwrap()
    }
}

fn run_params(wav: &Path, out: &Path, options: Value) -> Value {
    std::fs::create_dir_all(out).unwrap();
    json!({
        "item": {"id": "fixture-local-0001", "kind": "video", "title": "样例"},
        "want": ["asr"],
        "inputs": [{"kind": "audio:pcm16k", "mime": "audio/wav", "path": wav}],
        "out": out,
        "options": options,
    })
}

/// `asr.vtt` 里每条 cue 的长度（秒）和字。
fn cues(vtt: &str) -> Vec<(f64, String)> {
    let sec = |t: &str| {
        let p: Vec<f64> = t.split(':').map(|x| x.parse().unwrap()).collect();
        p[0] * 3600.0 + p[1] * 60.0 + p[2]
    };
    let lines: Vec<&str> = vtt.lines().collect();
    lines
        .iter()
        .enumerate()
        .filter_map(|(i, l)| {
            let (a, b) = l.split_once(" --> ")?;
            Some((sec(b) - sec(a), lines[i + 1].to_string()))
        })
        .collect()
}

#[test]
fn speech_quiet_and_wrong_rate() {
    let test = "speech_quiet_and_wrong_rate";
    let Some(dir) = model_dir(test) else { return };
    if !have_ffmpeg(test) {
        return;
    }
    let tmp = tempfile::tempdir().unwrap();
    let speech = tmp.path().join("speech.wav");
    let quiet = tmp.path().join("quiet.wav");
    let narrow = tmp.path().join("speech-8k.wav");
    to_wav("speech.m4a", &speech, &[], 16_000);
    to_wav("quiet.mp4", &quiet, &[], 16_000);
    to_wav("speech.m4a", &narrow, &[], 8_000);
    let mut p = Plugin::start(&plugin_dir(tmp.path(), &dir.join(VAD_FILE)), &dir);

    let d = p.call("describe", json!({}));
    assert_eq!(d["result"]["protocol"], 1, "{d}");
    assert_eq!(d["result"]["version"], env!("CARGO_PKG_VERSION"));
    assert_eq!(d["result"]["models"], json!([MODEL_NAME]));
    let h = p.call("health", json!({}));
    assert_eq!(h["result"]["ready"], true, "{h}");

    // 说话的那段：有「咖啡」「注水」，没有一条超过 15 秒。
    let out = tmp.path().join("out/1");
    let id = p.request("run", run_params(&speech, &out, json!({})));
    let (reply, progress) = p.answer(id);
    let r = &reply["result"];
    assert_eq!(
        r["outputs"],
        json!([{"kind": "asr", "path": "asr.vtt", "lang": "zh"}]),
        "{reply}"
    );
    assert_eq!(
        r["tool"],
        json!({"name": "sherpa-onnx", "version": "2.0.1"})
    );
    assert_eq!(r["model"], json!({"name": MODEL_NAME}));
    assert_eq!(r["empty"], false);
    assert_eq!(
        r["options"],
        json!({"model": "sense-voice", "language": "auto", "itn": true, "vad": "silero-vad-v5",
               "vadThreshold": 0.5, "maxCueSec": 15.0, "threads": 2})
    );
    assert!(!progress.is_empty(), "progress was sent");
    let vtt = std::fs::read_to_string(out.join("asr.vtt")).unwrap();
    assert!(vtt.starts_with("WEBVTT\n\nNOTE language: zh\n"), "{vtt}");
    let cs = cues(&vtt);
    assert!(!cs.is_empty(), "{vtt}");
    let text: String = cs.iter().map(|(_, t)| t.as_str()).collect();
    assert!(text.contains("咖啡") && text.contains("注水"), "{vtt}");
    assert!(cs.iter().all(|(len, _)| *len <= 15.0), "{vtt}");
    assert!(!text.contains("<|"), "{vtt}");

    // 宿主重做时把回显的参数原样交回；同一个进程里接着用已经加载的模型。
    let echoed = r["options"].clone();
    let again = tmp.path().join("out/2");
    let reply = p.call("run", run_params(&speech, &again, echoed));
    assert_eq!(reply["result"]["empty"], false, "{reply}");
    assert_eq!(std::fs::read_to_string(again.join("asr.vtt")).unwrap(), vtt);

    // 只有静音：不写文件，empty / no-speech，标记带秒数。
    let out = tmp.path().join("out/3");
    let reply = p.call("run", run_params(&quiet, &out, json!({})));
    let r = &reply["result"];
    assert_eq!(r["empty"], true, "{reply}");
    assert_eq!(r["reason"], "no-speech");
    assert_eq!(r["outputs"], json!([]));
    assert_eq!(r["marks"][0]["code"], "no-speech");
    assert!(r["marks"][0]["speechSec"].as_f64().unwrap() < 1.0);
    let duration = r["marks"][0]["durationSec"].as_f64().unwrap();
    assert!((duration - 5.0).abs() < 0.1, "{reply}");
    assert_eq!(std::fs::read_dir(&out).unwrap().count(), 0);

    // 8 kHz 不收。
    let out = tmp.path().join("out/4");
    let reply = p.call("run", run_params(&narrow, &out, json!({})));
    assert_eq!(reply["error"]["code"], -32000, "{reply}");
    assert_eq!(reply["error"]["data"]["code"], "unsupported-input");
    assert!(
        reply["error"]["data"]["message"]
            .as_str()
            .unwrap()
            .contains("8000 Hz")
    );

    // 参数不对回 invalid-args。
    let reply = p.call("run", run_params(&speech, &out, json!({"vadThreshold": 2})));
    assert_eq!(reply["error"]["data"]["code"], "invalid-args", "{reply}");

    assert!(p.close().success());
}

#[test]
fn missing_or_incomplete_model_is_dependency_missing() {
    let test = "missing_or_incomplete_model_is_dependency_missing";
    let Some(dir) = model_dir(test) else { return };
    if !have_ffmpeg(test) {
        return;
    }
    let tmp = tempfile::tempdir().unwrap();
    let speech = tmp.path().join("speech.wav");
    to_wav("speech.m4a", &speech, &[], 16_000);
    let plugin = plugin_dir(tmp.path(), &dir.join(VAD_FILE));

    // 一个空的模型目录，和一个只下了一半的。
    let empty = tmp.path().join("models-empty");
    let half = tmp.path().join("models-half");
    let half_model = half
        .join(MODEL_NAME)
        .join("sherpa-onnx-sense-voice-zh-en-ja-ko-yue-int8-2024-07-17");
    std::fs::create_dir_all(&empty).unwrap();
    std::fs::create_dir_all(&half_model).unwrap();
    std::fs::write(half_model.join("model.int8.onnx"), b"partial").unwrap();
    std::fs::write(half_model.join("tokens.txt"), b"partial").unwrap();

    for models in [empty, half] {
        let mut p = Plugin::start(&plugin, &models);
        let d = p.call("describe", json!({}));
        assert_eq!(d["result"]["models"], json!([]), "{d}");
        let h = p.call("health", json!({}));
        assert_eq!(h["result"]["ready"], false, "{h}");
        let out = tmp.path().join("out");
        let reply = p.call("run", run_params(&speech, &out, json!({})));
        let e = &reply["error"]["data"];
        assert_eq!(e["code"], "dependency-missing", "{reply}");
        assert_eq!(e["needs"], json!([format!("model:{MODEL_NAME}")]));
        assert!(p.close().success());
    }
}

#[test]
fn cancel_stops_between_segments() {
    let test = "cancel_stops_between_segments";
    let Some(dir) = model_dir(test) else { return };
    if !have_ffmpeg(test) {
        return;
    }
    let tmp = tempfile::tempdir().unwrap();
    // 样例接 20 遍，约 2.5 分钟，二十来个段。
    let long = tmp.path().join("long.wav");
    to_wav("speech.m4a", &long, &["-stream_loop", "19"], 16_000);
    let quiet = tmp.path().join("quiet.wav");
    to_wav("quiet.mp4", &quiet, &[], 16_000);
    let mut p = Plugin::start(&plugin_dir(tmp.path(), &dir.join(VAD_FILE)), &dir);

    let out = tmp.path().join("out/1");
    let id = p.request("run", run_params(&long, &out, json!({})));
    // 等第一段转完（`done` 大于 0）再取消。
    loop {
        let line = p.next_line();
        assert_eq!(
            line["method"], "progress",
            "finished before the cancel: {line}"
        );
        if line["params"]["done"].as_f64().unwrap() > 0.0 {
            break;
        }
    }
    p.send(&json!({"jsonrpc": "2.0", "method": "cancel", "params": {"id": id}}));
    let (reply, after) = p.answer(id);
    assert_eq!(reply["error"]["data"]["code"], "cancelled", "{reply}");
    assert!(!out.join("asr.vtt").exists());
    // 真的停在半路：取消以后最多再转完正在转的那一段，扫到的位置离结尾还远。
    let total = after.last().map_or(0.0, |p| p["total"].as_f64().unwrap());
    let reached = after
        .iter()
        .map(|p| p["done"].as_f64().unwrap())
        .fold(0.0, f64::max);
    assert!(after.len() <= 2, "kept going after the cancel: {after:?}");
    assert!(total == 0.0 || reached < total / 2.0, "{after:?}");

    // 取消以后这个进程照样接下一个。
    let reply = p.call(
        "run",
        run_params(&quiet, &tmp.path().join("out/2"), json!({})),
    );
    assert_eq!(reply["result"]["reason"], "no-speech", "{reply}");
    assert!(p.close().success());
}

#[test]
fn refuses_to_start_without_a_matching_vad() {
    let tmp = tempfile::tempdir().unwrap();
    let plugin = tmp.path().join("plugin");
    std::fs::create_dir_all(&plugin).unwrap();
    std::fs::write(plugin.join(VAD_FILE), b"not the model").unwrap();
    let bin = env!("CARGO_BIN_EXE_usuyo-plugin-asr-sherpa");
    let changed = Command::new(bin)
        .env_clear()
        .env("USUYO_PLUGIN_DIR", &plugin)
        .env("USUYO_MODELS_DIR", tmp.path())
        .stdin(Stdio::null())
        .output()
        .unwrap();
    assert!(!changed.status.success());
    assert!(changed.stdout.is_empty());
    assert!(String::from_utf8_lossy(&changed.stderr).contains("sha256"));

    let unset = Command::new(bin)
        .env_clear()
        .env("USUYO_PLUGIN_DIR", &plugin)
        .stdin(Stdio::null())
        .output()
        .unwrap();
    assert!(!unset.status.success());
    assert!(String::from_utf8_lossy(&unset.stderr).contains("USUYO_MODELS_DIR is not set"));
}
