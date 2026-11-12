//! 起插件的可执行文件，在 stdio 上说 JSON-RPC，和宿主一样；ffmpeg、ffprobe 用 Homebrew 那份
//! （环境变量 `USUYO_TEST_FFMPEG`、`USUYO_TEST_FFPROBE` 可以换成别的构建，例如 CI 编的精简版）。
//! 没有 ffmpeg 就跳过并打印原因。样例在 `crates/usuyo-core/testdata/transform/`。

use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, Command, ExitStatus, Stdio};
use std::sync::mpsc::{self, Receiver};
use std::time::{Duration, Instant};

use serde_json::{Value, json};
use sha2::{Digest, Sha256};

const HOMEBREW_FFMPEG: &str = "/opt/homebrew/bin/ffmpeg";
const HOMEBREW_FFPROBE: &str = "/opt/homebrew/bin/ffprobe";
/// 一次回答最多等多久。
const WAIT: Duration = Duration::from_secs(120);

fn samples() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../crates/usuyo-core/testdata/transform")
}

fn sample(name: &str) -> PathBuf {
    samples().join(name)
}

fn tool_path(var: &str, fallback: &str) -> PathBuf {
    std::env::var_os(var)
        .filter(|v| !v.is_empty())
        .map_or_else(|| PathBuf::from(fallback), PathBuf::from)
}

/// 一个测试的环境：临时目录、插件目录（里面是指向 ffmpeg、ffprobe 的链接）。
struct Env {
    tmp: tempfile::TempDir,
    plugin_dir: PathBuf,
    ffmpeg: PathBuf,
    ffprobe: PathBuf,
}

impl Env {
    /// 没有 ffmpeg 时打印原因，回 `None`。`synthesizes` 的测试还要 Homebrew 的 ffmpeg 来合成样例。
    fn new(test: &str) -> Option<Self> {
        Self::with(test, false)
    }

    fn new_synthesizing(test: &str) -> Option<Self> {
        Self::with(test, true)
    }

    fn with(test: &str, synthesizes: bool) -> Option<Self> {
        if synthesizes && !Path::new(HOMEBREW_FFMPEG).is_file() {
            eprintln!("skip {test}: no {HOMEBREW_FFMPEG} to synthesize the clips");
            return None;
        }
        let ffmpeg = tool_path("USUYO_TEST_FFMPEG", HOMEBREW_FFMPEG);
        let ffprobe = tool_path("USUYO_TEST_FFPROBE", HOMEBREW_FFPROBE);
        for tool in [&ffmpeg, &ffprobe] {
            if !tool.is_file() {
                eprintln!("skip {test}: no {}", tool.display());
                return None;
            }
        }
        let tmp = tempfile::tempdir().unwrap();
        let plugin_dir = tmp.path().join("plugin");
        std::fs::create_dir_all(&plugin_dir).unwrap();
        // 开发机上插件目录里的 ffmpeg 就是指向 Homebrew 的链接。
        std::os::unix::fs::symlink(&ffmpeg, plugin_dir.join("ffmpeg")).unwrap();
        std::os::unix::fs::symlink(&ffprobe, plugin_dir.join("ffprobe")).unwrap();
        Some(Self {
            tmp,
            plugin_dir,
            ffmpeg,
            ffprobe,
        })
    }

    fn path(&self, name: &str) -> PathBuf {
        self.tmp.path().join(name)
    }

    fn out(&self, name: &str) -> PathBuf {
        let dir = self.path(name);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn start(&self) -> Plugin {
        Plugin::start(&self.plugin_dir, self.tmp.path())
    }

    /// 用 ffmpeg 合成测试文件。
    /// 合成用的 ffmpeg 一直是 Homebrew 那份：精简构建没有 `lavfi`、`color`、`testsrc2` 和 x264，
    /// `USUYO_TEST_FFMPEG` 换的只是被测的那一份。
    fn synthesize(&self, args: &[&str]) {
        let status = Command::new(HOMEBREW_FFMPEG)
            .args(["-nostdin", "-hide_banner", "-loglevel", "error", "-y"])
            .args(args)
            .status()
            .expect("run ffmpeg");
        assert!(status.success(), "ffmpeg {args:?}");
    }

    /// 用 ffprobe 看产物的流。
    fn streams(&self, path: &Path) -> Vec<Value> {
        let out = Command::new(&self.ffprobe)
            .args(["-v", "error", "-of", "json", "-show_streams"])
            .arg(path)
            .output()
            .expect("run ffprobe");
        assert!(out.status.success(), "ffprobe {}", path.display());
        let v: Value = serde_json::from_slice(&out.stdout).unwrap();
        v["streams"].as_array().unwrap().clone()
    }

    fn version_line(&self, tool: &Path) -> String {
        let out = Command::new(tool).arg("-version").output().unwrap();
        String::from_utf8_lossy(&out.stdout)
            .lines()
            .next()
            .unwrap()
            .to_owned()
    }
}

fn sha256(path: &Path) -> String {
    Sha256::digest(std::fs::read(path).unwrap())
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

/// 起着的插件进程。
struct Plugin {
    child: Child,
    stdin: Option<ChildStdin>,
    lines: Receiver<Result<Value, String>>,
    next_id: i64,
}

impl Plugin {
    fn start(plugin_dir: &Path, cwd: &Path) -> Self {
        let mut child = Command::new(env!("CARGO_BIN_EXE_usuyo-plugin-media"))
            .env_clear()
            .env("USUYO_PLUGIN_DIR", plugin_dir)
            .current_dir(cwd)
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

    fn pid(&self) -> u32 {
        self.child.id()
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

fn input(kind: &str, path: &Path) -> Value {
    json!({"kind": kind, "path": path})
}

fn run_params(want: &str, inputs: Vec<Value>, out: &Path, options: Value) -> Value {
    json!({
        "item": {"id": "fixture-local-0001", "kind": "video", "title": "样例"},
        "want": [want],
        "inputs": inputs,
        "out": out,
        "options": options,
    })
}

const SNAPSHOT: &str = "019a6f2b-4a10-7c3e-8b21-0d9e8f7a6b5c";

fn video_with_snapshot(path: &Path) -> Value {
    json!({"kind": "media:video", "path": path, "snapshot": SNAPSHOT})
}

fn error_code(reply: &Value) -> &str {
    assert_eq!(reply["error"]["code"], -32000, "{reply}");
    reply["error"]["data"]["code"].as_str().unwrap()
}

#[test]
fn describe_health_and_probe_each_sample() {
    let Some(env) = Env::new("describe_health_and_probe_each_sample") else {
        return;
    };
    let mut p = env.start();
    let d = p.call("describe", json!({}));
    assert_eq!(d["result"]["protocol"], 1, "{d}");
    assert_eq!(d["result"]["version"], env!("CARGO_PKG_VERSION"));
    assert_eq!(d["result"]["models"], json!([]));
    let h = p.call("health", json!({}));
    assert_eq!(h["result"]["ready"], true, "{h}");

    let out = env.out("out");
    let inputs = vec![
        input("media:audio", &sample("speech.m4a")),
        input("media:video", &sample("video-only.mp4")),
        input("media:video", &sample("silent.mp4")),
        input("media:video", &sample("quiet.mp4")),
    ];
    let reply = p.call("run", run_params("probe", inputs, &out, json!({})));
    let r = &reply["result"];
    assert_eq!(r["empty"], false, "{reply}");
    assert_eq!(r["outputs"], json!([]));
    assert_eq!(r["options"], json!({}));
    let version = env.version_line(&env.ffprobe);
    assert_eq!(r["tool"]["name"], "ffprobe");
    assert!(
        version.starts_with(&format!(
            "ffprobe version {}",
            r["tool"]["version"].as_str().unwrap()
        )),
        "{version} vs {r}"
    );
    let probe = r["probe"].as_array().unwrap();
    let kinds = |i: usize| -> Vec<(String, String)> {
        probe[i]["streams"]
            .as_array()
            .unwrap()
            .iter()
            .map(|s| {
                (
                    s["kind"].as_str().unwrap().to_owned(),
                    s["codec"].as_str().unwrap().to_owned(),
                )
            })
            .collect()
    };
    let secs = |i: usize| probe[i]["durationSec"].as_f64().unwrap();
    assert_eq!(probe.len(), 4);
    for (i, p) in probe.iter().enumerate() {
        assert_eq!(p["input"], i);
    }
    assert_eq!(kinds(0), [("audio".into(), "aac".into())]);
    assert!((secs(0) - 7.43).abs() < 0.05);
    assert_eq!(kinds(1), [("video".into(), "h264".into())]);
    assert!((secs(1) - 7.52).abs() < 0.05);
    assert_eq!(kinds(2), [("video".into(), "h264".into())]);
    assert!((secs(2) - 5.0).abs() < 0.05);
    assert_eq!(
        kinds(3),
        [
            ("video".into(), "h264".into()),
            ("audio".into(), "aac".into())
        ]
    );

    // 图片不在可读的封装里：回 unsupported-input，进程接着用。
    let reply = p.call(
        "run",
        run_params(
            "probe",
            vec![input("media:video", &sample("card.png"))],
            &out,
            json!({}),
        ),
    );
    assert_eq!(error_code(&reply), "unsupported-input", "{reply}");
    assert!(p.close().success());
}

#[test]
fn play_remuxes_without_touching_the_inputs_and_refuses_other_pairs() {
    let Some(env) = Env::new("play_remuxes_without_touching_the_inputs_and_refuses_other_pairs")
    else {
        return;
    };
    let video = sample("video-only.mp4");
    let audio = sample("speech.m4a");
    let before = (sha256(&video), sha256(&audio));
    let mut p = env.start();

    let out = env.out("play");
    let id = p.request(
        "run",
        run_params(
            "play",
            vec![input("media:video", &video), input("media:audio", &audio)],
            &out,
            json!({}),
        ),
    );
    let (reply, progress) = p.answer(id);
    let r = &reply["result"];
    assert_eq!(
        r["outputs"],
        json!([{"kind": "play", "path": "play.mp4"}]),
        "{reply}"
    );
    assert_eq!(r["tool"]["name"], "ffmpeg");
    assert_eq!(r["options"], json!({}));
    assert!(!progress.is_empty(), "progress was sent");
    let streams = env.streams(&out.join("play.mp4"));
    let kinds: Vec<_> = streams
        .iter()
        .map(|s| s["codec_type"].as_str().unwrap())
        .collect();
    assert_eq!(kinds, ["video", "audio"]);
    // 不重编码：编码和输入一样。
    assert_eq!(streams[0]["codec_name"], "h264");
    assert_eq!(streams[1]["codec_name"], "aac");
    assert_eq!(
        (streams[0]["width"].as_u64(), streams[0]["height"].as_u64()),
        (Some(320), Some(180))
    );
    assert_eq!(
        (sha256(&video), sha256(&audio)),
        before,
        "inputs are untouched"
    );

    // 顺序反过来给也认。
    let out2 = env.out("play-swapped");
    let reply = p.call(
        "run",
        run_params(
            "play",
            vec![input("media:audio", &audio), input("media:video", &video)],
            &out2,
            json!({}),
        ),
    );
    assert_eq!(reply["result"]["outputs"][0]["path"], "play.mp4", "{reply}");

    // 画面文件已经带声音：拒绝，什么也不写。
    let out3 = env.out("play-both");
    let reply = p.call(
        "run",
        run_params(
            "play",
            vec![
                input("media:video", &sample("quiet.mp4")),
                input("media:audio", &audio),
            ],
            &out3,
            json!({}),
        ),
    );
    assert_eq!(error_code(&reply), "unsupported-input", "{reply}");
    assert_eq!(std::fs::read_dir(&out3).unwrap().count(), 0);
    // 「音轨」文件其实是画面：拒绝。
    let reply = p.call(
        "run",
        run_params(
            "play",
            vec![input("media:video", &video), input("media:audio", &video)],
            &out3,
            json!({}),
        ),
    );
    assert_eq!(error_code(&reply), "unsupported-input", "{reply}");
    // 两个都是视频：参数不对。
    let reply = p.call(
        "run",
        run_params(
            "play",
            vec![input("media:video", &video), input("media:video", &video)],
            &out3,
            json!({}),
        ),
    );
    assert_eq!(error_code(&reply), "invalid-args", "{reply}");
    assert!(p.close().success());
    assert_eq!((sha256(&video), sha256(&audio)), before);
}

#[test]
fn audio_pcm16k_from_audio_and_from_a_video_track() {
    let Some(env) = Env::new("audio_pcm16k_from_audio_and_from_a_video_track") else {
        return;
    };
    let mut p = env.start();
    let wav_facts = |path: &Path| {
        let s = &env.streams(path)[0];
        (
            s["codec_name"].as_str().unwrap().to_owned(),
            s["sample_rate"].as_str().unwrap().to_owned(),
            s["channels"].as_u64().unwrap(),
        )
    };
    for (kind, name, secs) in [
        ("media:audio", "speech.m4a", 7.43),
        ("media:video", "quiet.mp4", 5.0),
    ] {
        let out = env.out(&format!("pcm-{name}"));
        let reply = p.call(
            "run",
            run_params(
                "audio:pcm16k",
                vec![input(kind, &sample(name))],
                &out,
                json!({}),
            ),
        );
        let r = &reply["result"];
        assert_eq!(
            r["outputs"],
            json!([{"kind": "audio:pcm16k", "path": "audio.wav"}]),
            "{reply}"
        );
        assert_eq!(r["empty"], false);
        assert_eq!(
            wav_facts(&out.join("audio.wav")),
            ("pcm_s16le".into(), "16000".into(), 1),
            "{name}"
        );
        // 16 kHz、16 位、单声道：每秒 32000 字节加 44 字节的头。
        let size = std::fs::metadata(out.join("audio.wav")).unwrap().len() as f64;
        assert!(
            ((size - 44.0) / 32000.0 - secs).abs() < 0.1,
            "{name}: {size}"
        );
    }

    // 没有音轨：不是错误，回 empty / no-audio-track，什么也不写。
    let out = env.out("pcm-none");
    let reply = p.call(
        "run",
        run_params(
            "audio:pcm16k",
            vec![input("media:video", &sample("video-only.mp4"))],
            &out,
            json!({}),
        ),
    );
    let r = &reply["result"];
    assert_eq!(r["empty"], true, "{reply}");
    assert_eq!(r["reason"], "no-audio-track");
    assert_eq!(r["outputs"], json!([]));
    assert_eq!(r["marks"], json!([{"code": "no-audio-track"}]));
    assert_eq!(std::fs::read_dir(&out).unwrap().count(), 0);
    assert!(p.close().success());
}

/// 读 `keyframes.json`，按契约的 schema 和类型都校验一遍。
fn read_keyframes(out: &Path) -> Value {
    let text = std::fs::read_to_string(out.join("keyframes.json")).unwrap();
    let value: Value = serde_json::from_str(&text).unwrap();
    let schema: Value = serde_json::from_slice(
        &std::fs::read(
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../crates/usuyo-contract/schema/keyframes.schema.json"),
        )
        .unwrap(),
    )
    .unwrap();
    let validator = jsonschema::validator_for(&schema).unwrap();
    let errors: Vec<String> = validator
        .iter_errors(&value)
        .map(|e| e.to_string())
        .collect();
    assert!(errors.is_empty(), "{errors:?}\n{text}");
    let typed: usuyo_contract::Keyframes = serde_json::from_value(value.clone()).unwrap();
    assert_eq!(typed.snapshot, SNAPSHOT);
    assert_eq!(typed.bad_path(), None);
    assert!(typed.extra.is_empty(), "unknown fields in keyframes.json");
    value
}

/// 每一帧的文件在、哈希和宽高对得上、文件名是毫秒。
fn check_frame_files(out: &Path, kf: &Value) {
    for f in kf["frames"].as_array().unwrap() {
        let path = out.join(f["path"].as_str().unwrap());
        let ms = (f["sec"].as_f64().unwrap() * 1000.0).round() as u64;
        assert_eq!(
            path.file_name().unwrap().to_str().unwrap(),
            format!("{ms:08}.jpg")
        );
        assert_eq!(sha256(&path), f["sha256"].as_str().unwrap());
        let bytes = std::fs::read(&path).unwrap();
        assert_eq!(&bytes[..2], [0xFF, 0xD8], "{} is a JPEG", path.display());
        assert!(
            f["width"]
                .as_u64()
                .unwrap()
                .max(f["height"].as_u64().unwrap())
                <= 960
        );
    }
}

fn frame_secs(kf: &Value) -> Vec<f64> {
    kf["frames"]
        .as_array()
        .unwrap()
        .iter()
        .map(|f| f["sec"].as_f64().unwrap())
        .collect()
}

#[test]
fn keyframes_static_picture_and_moving_picture() {
    let Some(env) = Env::new("keyframes_static_picture_and_moving_picture") else {
        return;
    };
    let mut p = env.start();

    // 静止的画面：标 static-picture，frozenSec 约等于时长。
    let out = env.out("kf-silent");
    let id = p.request(
        "run",
        run_params(
            "keyframes",
            vec![video_with_snapshot(&sample("silent.mp4"))],
            &out,
            json!({}),
        ),
    );
    let (reply, progress) = p.answer(id);
    let r = &reply["result"];
    assert_eq!(
        r["outputs"],
        json!([{"kind": "keyframes", "path": "keyframes.json"}]),
        "{reply}"
    );
    assert!(!progress.is_empty());
    let mark = &r["marks"][0];
    assert_eq!(mark["code"], "static-picture", "{reply}");
    assert!(
        (mark["durationSec"].as_f64().unwrap() - 5.0).abs() < 0.05,
        "{mark}"
    );
    assert!(mark["frozenSec"].as_f64().unwrap() >= 4.5, "{mark}");
    // 所有参数都带着默认值回显。
    assert_eq!(
        r["options"],
        json!({"scanFps": 4.0, "scanHeight": 360, "sceneThreshold": 10.0, "freezeNoise": 0.003,
               "freezeMinSec": 2.0, "firstSec": 0.5, "minGapSec": 2.0, "maxPerMinute": 6,
               "maxFrames": 60, "minFrames": 3, "minFrameEverySec": 10.0, "maxSide": 960,
               "jpegQuality": 4, "staticRatio": 0.9})
    );
    let kf = read_keyframes(&out);
    assert_eq!(frame_secs(&kf), [0.5]);
    assert_eq!(kf["frames"][0]["scene"], Value::Null);
    assert_eq!(kf["frames"][0]["path"], "keyframes/00000500.jpg");
    assert_eq!(
        (
            kf["frames"][0]["width"].as_u64(),
            kf["frames"][0]["height"].as_u64()
        ),
        (Some(540), Some(720))
    );
    assert!(kf["frozenSec"].as_f64().unwrap() >= 4.5);
    check_frame_files(&out, &kf);

    // 动着的画面：没有这个标记，frozenSec 是 0。
    let out = env.out("kf-moving");
    let reply = p.call(
        "run",
        run_params(
            "keyframes",
            vec![video_with_snapshot(&sample("video-only.mp4"))],
            &out,
            json!({}),
        ),
    );
    assert_eq!(reply["result"]["marks"], json!([]), "{reply}");
    let kf = read_keyframes(&out);
    assert_eq!(kf["frozenSec"], 0.0);
    assert!((kf["durationSec"].as_f64().unwrap() - 7.52).abs() < 0.05);
    check_frame_files(&out, &kf);

    // 参数能改：staticRatio 调到 1 以上不收；maxFrames 比 minFrames 小不收。
    let reply = p.call(
        "run",
        run_params(
            "keyframes",
            vec![video_with_snapshot(&sample("silent.mp4"))],
            &env.out("bad"),
            json!({"staticRatio": 2}),
        ),
    );
    assert_eq!(error_code(&reply), "invalid-args", "{reply}");
    let reply = p.call(
        "run",
        run_params(
            "keyframes",
            vec![json!({"kind": "media:video", "path": sample("silent.mp4")})],
            &env.out("bad2"),
            json!({}),
        ),
    );
    assert_eq!(error_code(&reply), "invalid-args", "{reply}");
    assert!(p.close().success());
}

#[test]
fn keyframes_follow_scene_changes_and_respect_the_caps() {
    let Some(env) = Env::new_synthesizing("keyframes_follow_scene_changes_and_respect_the_caps")
    else {
        return;
    };
    // 红、测试图、蓝各 3 秒：镜头变化在 3 秒和 6 秒。
    let cuts = env.path("cuts.mp4");
    env.synthesize(
        &[
            "-f",
            "lavfi",
            "-i",
            "color=c=red:s=320x180:r=25:d=3",
            "-f",
            "lavfi",
            "-i",
            "testsrc2=s=320x180:r=25:d=3",
            "-f",
            "lavfi",
            "-i",
            "color=c=blue:s=320x180:r=25:d=3",
            "-filter_complex",
            "[0][1][2]concat=n=3:v=1:a=0",
            "-c:v",
            "libx264",
            "-pix_fmt",
            "yuv420p",
        ]
        .iter()
        .copied()
        .chain([cuts.to_str().unwrap()])
        .collect::<Vec<_>>(),
    );
    let mut p = env.start();
    let out = env.out("kf-cuts");
    let reply = p.call(
        "run",
        run_params(
            "keyframes",
            vec![video_with_snapshot(&cuts)],
            &out,
            json!({}),
        ),
    );
    assert_eq!(reply["result"]["marks"], json!([]), "{reply}");
    let kf = read_keyframes(&out);
    let secs = frame_secs(&kf);
    assert_eq!(secs.len(), 3, "{secs:?}");
    assert!((secs[0] - 0.5).abs() < 1e-9);
    assert!(
        (secs[1] - 3.0).abs() < 0.3 && (secs[2] - 6.0).abs() < 0.3,
        "{secs:?}"
    );
    assert_eq!(kf["frames"][0]["scene"], Value::Null);
    assert!(kf["frames"][1]["scene"].as_f64().unwrap() >= 10.0);
    check_frame_files(&out, &kf);

    // 参数生效：maxFrames 2 只留 2 帧（0.5 秒那帧加分数最高的一处）。
    let out = env.out("kf-cuts-2");
    let reply = p.call(
        "run",
        run_params(
            "keyframes",
            vec![video_with_snapshot(&cuts)],
            &out,
            json!({"maxFrames": 2, "minFrames": 1}),
        ),
    );
    assert_eq!(reply["result"]["options"]["maxFrames"], 2);
    assert_eq!(frame_secs(&read_keyframes(&out)).len(), 2);

    // 红 4 秒、蓝 4 秒首尾相接 17 遍（136 秒），每 4 秒一处镜头变化：
    // 每分钟最多 6 帧，镜头变化处的帧彼此至少隔 2 秒，总数不超过 60。
    let pair = env.path("red-blue.mp4");
    env.synthesize(&[
        "-f",
        "lavfi",
        "-i",
        "color=c=red:s=160x90:r=10:d=4",
        "-f",
        "lavfi",
        "-i",
        "color=c=blue:s=160x90:r=10:d=4",
        "-filter_complex",
        "[0][1]concat=n=2:v=1:a=0",
        "-c:v",
        "libx264",
        "-pix_fmt",
        "yuv420p",
        pair.to_str().unwrap(),
    ]);
    let long = env.path("loop.mp4");
    env.synthesize(&[
        "-stream_loop",
        "16",
        "-i",
        pair.to_str().unwrap(),
        "-c",
        "copy",
        long.to_str().unwrap(),
    ]);
    let out = env.out("kf-loop");
    let reply = p.call(
        "run",
        run_params(
            "keyframes",
            vec![video_with_snapshot(&long)],
            &out,
            json!({}),
        ),
    );
    assert!(reply["result"]["outputs"].is_array(), "{reply}");
    let kf = read_keyframes(&out);
    let frames = kf["frames"].as_array().unwrap();
    assert!(frames.len() <= 60);
    let mut per_minute = std::collections::BTreeMap::<u64, usize>::new();
    for f in frames {
        *per_minute
            .entry((f["sec"].as_f64().unwrap() / 60.0) as u64)
            .or_default() += 1;
    }
    assert!(per_minute.values().all(|n| *n <= 6), "{per_minute:?}");
    assert!(per_minute.len() >= 2, "{per_minute:?}");
    let scene_times: Vec<f64> = frames
        .iter()
        .filter(|f| !f["scene"].is_null())
        .map(|f| f["sec"].as_f64().unwrap())
        .collect();
    assert!(scene_times.len() >= 4, "{scene_times:?}");
    let all = frame_secs(&kf);
    for t in &scene_times {
        for other in &all {
            assert!(
                t == other || (t - other).abs() >= 2.0 - 1e-9,
                "{t} vs {other} in {all:?}"
            );
        }
    }
    check_frame_files(&out, &kf);
    assert!(p.close().success());
}

/// `pgrep -P <父进程>` 找到的子进程数；`pattern` 非空时只数命令行里有它的。
fn children_of(parent: u32, pattern: &str) -> usize {
    let mut cmd = Command::new("pgrep");
    cmd.arg("-P").arg(parent.to_string());
    if !pattern.is_empty() {
        cmd.args(["-f", "--", pattern]);
    }
    let out = cmd.output().expect("run pgrep");
    String::from_utf8_lossy(&out.stdout).lines().count()
}

#[test]
fn cancel_stops_ffmpeg_and_leaves_no_child() {
    let Some(env) = Env::new_synthesizing("cancel_stops_ffmpeg_and_leaves_no_child") else {
        return;
    };
    // 样例首尾相接 400 遍（约 50 分钟），拷流不重编码，第一遍扫描要几十秒。
    let long = env.path("long.mp4");
    env.synthesize(&[
        "-stream_loop",
        "399",
        "-i",
        sample("video-only.mp4").to_str().unwrap(),
        "-c",
        "copy",
        long.to_str().unwrap(),
    ]);
    let mut p = env.start();
    let out = env.out("kf-long");
    let id = p.request(
        "run",
        run_params(
            "keyframes",
            vec![video_with_snapshot(&long)],
            &out,
            json!({}),
        ),
    );
    // 等扫描的 ffmpeg 起来再取消，确认取消时它真的在跑。只有 ffmpeg 的命令行里有 `-progress`；
    // 不能按名字找：Homebrew 的 ffprobe 的真实路径里也有 `ffmpeg`（Cellar/ffmpeg/…）。
    let deadline = Instant::now() + Duration::from_secs(30);
    while children_of(p.pid(), "-progress") == 0 {
        assert!(Instant::now() < deadline, "ffmpeg never started");
        std::thread::sleep(Duration::from_millis(50));
    }
    let cancelled_at = Instant::now();
    p.send(&json!({"jsonrpc": "2.0", "method": "cancel", "params": {"id": id}}));
    let (reply, progress) = p.answer(id);
    assert_eq!(error_code(&reply), "cancelled", "{reply}");
    // 扫描真的停在半路：报过的位置离片尾还远（片长约 3000 秒）。
    let total = progress
        .iter()
        .filter_map(|p| p["total"].as_f64())
        .fold(0.0, f64::max);
    let reached = progress
        .iter()
        .filter(|p| p["unit"] == "sec")
        .filter_map(|p| p["done"].as_f64())
        .fold(0.0, f64::max);
    assert!(total > 2000.0, "{progress:?}");
    assert!(reached < total / 2.0, "scan reached {reached} of {total}");
    assert!(
        cancelled_at.elapsed() < Duration::from_secs(5),
        "{:?}",
        cancelled_at.elapsed()
    );
    assert_eq!(children_of(p.pid(), ""), 0, "a child process is left");
    assert!(!out.join("keyframes.json").exists());

    // 取消以后这个进程照样接下一个。
    let reply = p.call(
        "run",
        run_params(
            "probe",
            vec![input("media:video", &sample("silent.mp4"))],
            &env.out("after"),
            json!({}),
        ),
    );
    assert!(reply["result"]["probe"].is_array(), "{reply}");
    assert!(p.close().success());
}

#[test]
fn file_names_that_look_like_options_or_protocols_are_files() {
    let Some(env) = Env::new("file_names_that_look_like_options_or_protocols_are_files") else {
        return;
    };
    let dir = env.out("odd");
    let names = ["-loglevel.mp4", "-i", "concat:video-only.mp4", "http:x.mp4"];
    for name in names {
        std::fs::copy(sample("quiet.mp4"), dir.join(name)).unwrap();
    }
    let mut p = Plugin::start(&env.plugin_dir, &dir);
    for name in names {
        // 绝对路径和相对于插件工作目录的相对路径。
        for path in [dir.join(name), PathBuf::from(name)] {
            let out = env.out(&format!("odd-out-{}", name.replace([':', '/'], "_")));
            let reply = p.call(
                "run",
                run_params("probe", vec![input("media:video", &path)], &out, json!({})),
            );
            let streams = reply["result"]["probe"][0]["streams"]
                .as_array()
                .unwrap_or_else(|| panic!("{name}: {reply}"));
            assert_eq!(streams.len(), 2, "{name}: {reply}");
        }
        let out = env.out(&format!("odd-pcm-{}", name.replace([':', '/'], "_")));
        let reply = p.call(
            "run",
            run_params(
                "audio:pcm16k",
                vec![input("media:video", Path::new(name))],
                &out,
                json!({}),
            ),
        );
        assert_eq!(
            reply["result"]["outputs"][0]["path"], "audio.wav",
            "{name}: {reply}"
        );
        let out = env.out(&format!("odd-kf-{}", name.replace([':', '/'], "_")));
        let reply = p.call(
            "run",
            run_params(
                "keyframes",
                vec![video_with_snapshot(Path::new(name))],
                &out,
                json!({}),
            ),
        );
        assert_eq!(
            reply["result"]["outputs"][0]["path"], "keyframes.json",
            "{name}: {reply}"
        );
    }
    assert!(p.close().success());
}

#[test]
fn a_file_cannot_pull_in_other_files() {
    let Some(env) = Env::new("a_file_cannot_pull_in_other_files") else {
        return;
    };
    // 内容是 ffconcat 清单：不在可读的封装里，不跟着读清单里写的文件。
    let dir = env.out("sneaky");
    std::fs::copy(sample("video-only.mp4"), dir.join("real.mp4")).unwrap();
    let list = dir.join("list.mp4");
    std::fs::write(&list, "ffconcat version 1.0\nfile 'real.mp4'\n").unwrap();
    let mut p = env.start();
    let reply = p.call(
        "run",
        run_params(
            "probe",
            vec![input("media:video", &list)],
            &env.out("sneaky-out"),
            json!({}),
        ),
    );
    assert_eq!(error_code(&reply), "unsupported-input", "{reply}");
    assert!(p.close().success());
}

#[test]
fn bad_requests_are_invalid_args_and_a_missing_tool_stops_the_start() {
    let Some(env) = Env::new("bad_requests_are_invalid_args_and_a_missing_tool_stops_the_start")
    else {
        return;
    };
    let mut p = env.start();
    let out = env.out("bad");
    let video = input("media:video", &sample("silent.mp4"));
    let cases = [
        // 一次只做一种。
        json!({"want": ["probe", "play"], "inputs": [video], "out": out}),
        // 这个插件不出 asr。
        json!({"want": ["asr"], "inputs": [video], "out": out}),
        // 没有输入。
        json!({"want": ["probe"], "inputs": [], "out": out}),
        // 输入不是音视频。
        json!({"want": ["probe"], "inputs": [{"kind": "image", "path": sample("card.png")}], "out": out}),
        // probe、play、audio:pcm16k 没有参数。
        json!({"want": ["probe"], "inputs": [video], "out": out, "options": {"x": 1}}),
    ];
    for mut params in cases {
        params["item"] = json!({"id": "a", "kind": "video", "title": "t"});
        let reply = p.call("run", params.clone());
        assert_eq!(error_code(&reply), "invalid-args", "{params}: {reply}");
    }
    assert!(p.close().success());

    // 插件目录里没有 ffmpeg：起不来，stderr 说明原因。
    let empty = env.out("no-assets");
    let started = Command::new(env!("CARGO_BIN_EXE_usuyo-plugin-media"))
        .env_clear()
        .env("USUYO_PLUGIN_DIR", &empty)
        .stdin(Stdio::null())
        .output()
        .unwrap();
    assert!(!started.status.success());
    assert!(started.stdout.is_empty());
    assert!(String::from_utf8_lossy(&started.stderr).contains("ffmpeg"));
    let unset = Command::new(env!("CARGO_BIN_EXE_usuyo-plugin-media"))
        .env_clear()
        .stdin(Stdio::null())
        .output()
        .unwrap();
    assert!(String::from_utf8_lossy(&unset.stderr).contains("USUYO_PLUGIN_DIR is not set"));
}

#[test]
fn tool_versions_come_from_the_binary() {
    let Some(env) = Env::new("tool_versions_come_from_the_binary") else {
        return;
    };
    let mut p = env.start();
    let out = env.out("ver");
    let reply = p.call(
        "run",
        run_params(
            "audio:pcm16k",
            vec![input("media:audio", &sample("speech.m4a"))],
            &out,
            json!({}),
        ),
    );
    let tool = &reply["result"]["tool"];
    assert_eq!(tool["name"], "ffmpeg");
    let line = env.version_line(&env.ffmpeg);
    assert!(
        line.starts_with(&format!(
            "ffmpeg version {}",
            tool["version"].as_str().unwrap()
        )),
        "{line} vs {tool}"
    );
    assert!(p.close().success());
}

#[test]
fn max_side_shrinks_the_frames() {
    let Some(env) = Env::new("max_side_shrinks_the_frames") else {
        return;
    };
    let mut p = env.start();
    let out = env.out("kf-small");
    let reply = p.call(
        "run",
        run_params(
            "keyframes",
            vec![video_with_snapshot(&sample("video-only.mp4"))],
            &out,
            json!({"maxSide": 200}),
        ),
    );
    assert_eq!(reply["result"]["options"]["maxSide"], 200, "{reply}");
    let kf = read_keyframes(&out);
    let f = &kf["frames"][0];
    // 320x180 缩到长边 200：200x112（宽高取偶数）。
    assert_eq!(
        (f["width"].as_u64(), f["height"].as_u64()),
        (Some(200), Some(112)),
        "{f}"
    );
    assert!(p.close().success());
}

#[test]
fn fragmented_segments_like_the_ones_a_fetch_plugin_stores_work() {
    let Some(env) =
        Env::new_synthesizing("fragmented_segments_like_the_ones_a_fetch_plugin_stores_work")
    else {
        return;
    };
    // 分段的 mp4（没有总的 moov 时长表），文件名是 `.m4s`。
    let video = env.path("video.m4s");
    let audio = env.path("audio.m4s");
    for (src, dst) in [("video-only.mp4", &video), ("speech.m4a", &audio)] {
        env.synthesize(&[
            "-i",
            sample(src).to_str().unwrap(),
            "-c",
            "copy",
            "-movflags",
            "+frag_keyframe+empty_moov+default_base_moof",
            "-f",
            "mp4",
            dst.to_str().unwrap(),
        ]);
    }
    let mut p = env.start();
    let reply = p.call(
        "run",
        run_params(
            "probe",
            vec![input("media:video", &video), input("media:audio", &audio)],
            &env.out("frag-probe"),
            json!({}),
        ),
    );
    let probe = &reply["result"]["probe"];
    assert!(
        (probe[0]["durationSec"].as_f64().unwrap() - 7.52).abs() < 0.05,
        "{reply}"
    );
    assert!(
        (probe[1]["durationSec"].as_f64().unwrap() - 7.5).abs() < 0.1,
        "{reply}"
    );

    let out = env.out("frag-play");
    let reply = p.call(
        "run",
        run_params(
            "play",
            vec![input("media:video", &video), input("media:audio", &audio)],
            &out,
            json!({}),
        ),
    );
    assert_eq!(reply["result"]["outputs"][0]["path"], "play.mp4", "{reply}");
    assert_eq!(env.streams(&out.join("play.mp4")).len(), 2);

    let out = env.out("frag-kf");
    let reply = p.call(
        "run",
        run_params(
            "keyframes",
            vec![video_with_snapshot(&video)],
            &out,
            json!({}),
        ),
    );
    assert_eq!(
        reply["result"]["outputs"][0]["path"], "keyframes.json",
        "{reply}"
    );
    check_frame_files(&out, &read_keyframes(&out));
    assert!(p.close().success());
}
