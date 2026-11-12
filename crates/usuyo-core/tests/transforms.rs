//! 转换的调度和一条链对着假插件跑：获取插件是 sh 脚本，转换插件是 `examples/fake_transform`
//! （plugin-kit 的循环）。电源由测试拨，退避是零，什么都不等墙上的钟。
#![cfg(unix)]

use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use serde_json::{Map, Value, json};
use tempfile::TempDir;
use usuyo_core::{
    Engine, EngineConfig, Links, Power, PowerSource, RetryPolicy, TransformJobKind, TransformState,
    Via,
};

/// 测试拨的电源。
#[derive(Debug, Default)]
struct Switch(AtomicBool);

impl PowerSource for Switch {
    fn read(&self) -> Option<Power> {
        Some(if self.0.load(Ordering::SeqCst) {
            Power::Battery
        } else {
            Power::Mains
        })
    }
}

/// 取到的条目：`$2` 是链接，最后一段说是哪一种。
const FETCH: &str = r#"
case "$2" in
  */dash)
    mkdir -p "$out/media"
    printf 'v:7.5' > "$out/media/video.m4s"; printf 'a:7.4' > "$out/media/audio.m4s"
    printf 'WEBVTT\n' > "$out/media/subtitle.zh.vtt"
    MEDIA='[{"path": "media/video.m4s", "kind": "video"}, {"path": "media/audio.m4s", "kind": "audio"},
            {"path": "media/subtitle.zh.vtt", "kind": "subtitle", "lang": "zh"}]'
    IMAGES='[]'; KIND=video ;;
  */grouped)
    mkdir -p "$out/media"
    printf 'v:7.5' > "$out/media/video.m4s"; printf 'a:60' > "$out/media/audio.m4s"
    MEDIA='[{"path": "media/video.m4s", "kind": "video", "group": "g"}, {"path": "media/audio.m4s", "kind": "audio", "group": "g"}]'
    IMAGES='[]'; KIND=video ;;
  */apart)
    mkdir -p "$out/media"
    printf 'v:7.5' > "$out/media/video.m4s"; printf 'a:60' > "$out/media/audio.m4s"
    MEDIA='[{"path": "media/video.m4s", "kind": "video"}, {"path": "media/audio.m4s", "kind": "audio"}]'
    IMAGES='[]'; KIND=video ;;
  */silent)
    mkdir -p "$out/media"
    printf 'v:5' > "$out/media/video.mp4"
    MEDIA='[{"path": "media/video.mp4", "kind": "video"}]'
    IMAGES='[]'; KIND=video ;;
  */note)
    mkdir -p "$out/images"
    printf 'card' > "$out/images/001.png"; printf 'card' > "$out/images/002.png"
    MEDIA='[]'; IMAGES='["images/001.png", "images/002.png"]'; KIND=post ;;
esac
cat <<JSON
{"ok": true, "identity": {"platform": "web", "url": "$2"}, "kind": "$KIND", "title": "手冲咖啡",
 "text": "三段式注水", "images": $IMAGES, "media": $MEDIA, "platformTags": [],
 "completeness": "full", "missing": [],
 "evidence": {"fetchedAt": "2026-11-01T03:00:01Z", "access": "public-web"}}
JSON
"#;

const FETCH_MANIFEST: &str = r#"protocol: 1
id: fake/web
kind: fetch
name: 假的
version: 1.0.0
license: MIT
maintainers: [someone]
platforms: [web]
match: ["*"]
access: public-web
login: none
capabilities:
  fetch:
    access: public-web
    login: none
    outputs: [text, images, media:video, media:audio, subtitle, meta]
permissions:
  fs: {write: ["${OUT}"]}
run:
  command: ["${PLUGIN_DIR}/plugin.sh"]
"#;

fn transform_manifest(repo: &str, consumes: &str, produces: &str, models: &str) -> String {
    format!(
        r#"protocol: 1
id: fake/{repo}
kind: transform
name: 假的 {repo}
version: 1.0.0
license: MIT
maintainers: [someone]
consumes: [{consumes}]
produces: [{produces}]
provider: none
{models}permissions:
  fs: {{read: ["${{INPUT}}", "${{MODELS}}"], write: ["${{OUT}}"]}}
run:
  command: ["${{PLUGIN_DIR}}/fake_transform", "{repo}"]
"#
    )
}

const VOICE: &str = r#"models:
  - name: fake-voice
    url: https://example.com/fake-voice.tar.bz2
    sha256: "1111111111111111111111111111111111111111111111111111111111111111"
    size: 10
    format: tar.bz2
    license: MIT
"#;

/// `cargo test` 顺带编好的 `examples/fake_transform`。
fn fake_transform() -> PathBuf {
    let exe = std::env::current_exe().unwrap();
    let profile = exe.parent().and_then(Path::parent).unwrap();
    let path = profile.join("examples/fake_transform");
    assert!(
        path.is_file(),
        "{} is missing: `cargo test` builds the examples unless it is narrowed with `--test`; add `--example fake_transform`",
        path.display()
    );
    path
}

struct Fixture {
    tmp: TempDir,
    library: PathBuf,
    battery: Arc<Switch>,
}

impl Fixture {
    fn new() -> Fixture {
        let tmp = tempfile::tempdir().unwrap();
        let library =
            Engine::init_library(&tmp.path().join("lib"), &tmp.path().join("machine")).unwrap();
        let f = Fixture {
            tmp,
            library,
            battery: Arc::default(),
        };
        let fetch = f.machine().join("plugins/fake/web/1.0.0");
        std::fs::create_dir_all(&fetch).unwrap();
        std::fs::write(fetch.join("usuyo-plugin.yaml"), FETCH_MANIFEST).unwrap();
        let script = fetch.join("plugin.sh");
        std::fs::write(
            &script,
            format!(
                "#!/bin/sh\nout=\"\"\nif [ \"$1\" = canonicalize ]; then echo '{{\"ok\": true, \"match\": false}}'; exit 0; fi\nif [ \"$1\" = fetch ]; then out=\"$4\"; fi\n{FETCH}\n"
            ),
        )
        .unwrap();
        std::fs::set_permissions(&script, std::fs::Permissions::from_mode(0o755)).unwrap();
        for (repo, consumes, produces, models) in [
            (
                "media",
                "media:video, media:audio",
                "play, audio:pcm16k, probe, keyframes",
                "",
            ),
            ("asr", "audio:pcm16k", "asr", VOICE),
            ("ocr", "image, derived:keyframes", "ocr", ""),
        ] {
            let dir = f.machine().join("plugins/fake").join(repo).join("1.0.0");
            std::fs::create_dir_all(&dir).unwrap();
            std::fs::write(
                dir.join("usuyo-plugin.yaml"),
                transform_manifest(repo, consumes, produces, models),
            )
            .unwrap();
            std::fs::copy(fake_transform(), dir.join("fake_transform")).unwrap();
        }
        f
    }

    fn machine(&self) -> PathBuf {
        self.tmp.path().join("machine")
    }

    fn engine_with(&self, tune: impl FnOnce(&mut EngineConfig)) -> Engine {
        let mut config = EngineConfig::new(&self.library, self.machine());
        config.retry = RetryPolicy {
            base: Duration::ZERO,
            max_attempts: 3,
        };
        config.power = self.battery.clone();
        tune(&mut config);
        Engine::open(config).unwrap()
    }

    fn engine(&self) -> Engine {
        self.engine_with(|_| {})
    }

    /// 收下、取到内容；回条目 id。
    async fn take(&self, engine: &Engine, name: &str) -> String {
        let t = engine
            .take(&format!("https://example.test/{name}"), "link", Via::App)
            .await
            .unwrap();
        assert_eq!(engine.run_due().await.unwrap(), 1);
        t.id
    }

    fn item_dir(&self, id: &str) -> PathBuf {
        self.library.join("items/web").join(id)
    }

    /// 一条的转录，新的在前。
    fn runs(&self, id: &str) -> Vec<Value> {
        let dir = self.item_dir(id).join("derived/runs");
        let mut names: Vec<String> = match std::fs::read_dir(&dir) {
            Ok(d) => d
                .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
                .collect(),
            Err(_) => Vec::new(),
        };
        names.sort();
        names.reverse();
        names
            .iter()
            .map(|n| {
                serde_json::from_slice(&std::fs::read(dir.join(n).join("run.json")).unwrap())
                    .unwrap()
            })
            .collect()
    }

    fn jobs_of(&self, runs: &[Value], job: &str) -> Vec<Value> {
        runs.iter().filter(|r| r["job"] == job).cloned().collect()
    }

    /// 把模型照宿主下载以后的样子放进按机器目录。
    fn place_voice(&self) {
        let dir = self.machine().join("models/fake/asr/fake-voice");
        std::fs::create_dir_all(dir.join("top")).unwrap();
        std::fs::write(dir.join("top/weights"), "w").unwrap();
        std::fs::write(
            dir.join("model.json"),
            json!({"name": "fake-voice", "sha256": "1".repeat(64), "size": 10, "license": "MIT",
                   "files": [{"path": "top/weights", "size": 1, "sha256": "x"}]})
            .to_string(),
        )
        .unwrap();
    }
}

fn state(
    engine: &Engine,
    id: &str,
    job: TransformJobKind,
) -> Option<(TransformState, Option<String>, u32)> {
    engine
        .transform_jobs(id)
        .unwrap()
        .into_iter()
        .find(|j| j.job == job)
        .map(|j| (j.state, j.wait, j.attempts))
}

#[tokio::test]
async fn a_split_video_becomes_play_keyframes_frames_and_asr_waits_for_its_model() {
    let f = Fixture::new();
    let engine = f.engine();
    let mut events = engine.subscribe();
    let id = f.take(&engine, "dash").await;
    let raw_before: Vec<(PathBuf, Vec<u8>)> = walk(&f.item_dir(&id).join("raw"));

    engine.run_transforms_due().await.unwrap();
    let runs = f.runs(&id);
    let play = f.jobs_of(&runs, "play");
    assert_eq!(play.len(), 1, "{runs:?}");
    assert_eq!(play[0]["trigger"], "auto");
    assert_eq!(play[0]["by"], "fake/media");
    assert_eq!(play[0]["outputs"][0]["path"], "play.mp4");
    let play_inputs: Vec<&str> = play[0]["inputs"]
        .as_array()
        .unwrap()
        .iter()
        .map(|i| i["path"].as_str().unwrap().rsplit('/').next().unwrap())
        .collect();
    assert_eq!(play_inputs, ["video.m4s", "audio.m4s"]);
    let kf = f.jobs_of(&runs, "keyframes");
    assert_eq!(kf.len(), 1);
    let frames = f.jobs_of(&runs, "frames");
    assert_eq!(frames.len(), 1, "keyframes 落地以后接着认帧上的字");
    let kf_id = kf[0]["id"].as_str().unwrap();
    assert_eq!(
        frames[0]["inputs"][0]["path"],
        format!("derived/runs/{kf_id}/keyframes/00000500.jpg")
    );
    let ocr: Value = serde_json::from_slice(
        &std::fs::read(
            f.item_dir(&id)
                .join("derived/runs")
                .join(frames[0]["id"].as_str().unwrap())
                .join("ocr.json"),
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(ocr["images"][0]["frameSec"], 0.5);

    // 口播：没有模型，等着，不算一次失败，也没有转录
    assert!(f.jobs_of(&runs, "asr").is_empty());
    assert_eq!(
        state(&engine, &id, TransformJobKind::Asr),
        Some((TransformState::Waiting, Some("model:fake-voice".into()), 0))
    );
    assert_eq!(
        engine.run_transforms_due().await.unwrap(),
        0,
        "等着的不再跑"
    );

    // 自己把模型放进去：下一轮看到了，接着转
    f.place_voice();
    assert_eq!(engine.run_transforms_due().await.unwrap(), 1);
    let runs = f.runs(&id);
    let asr = f.jobs_of(&runs, "asr");
    assert_eq!(asr.len(), 1);
    let a = &asr[0];
    assert_eq!(
        a["inputs"][0]["path"].as_str().unwrap().rsplit('/').next(),
        Some("audio.m4s"),
        "分开的音轨"
    );
    assert_eq!(a["inputs"][0]["via"][0]["op"], "audio:pcm16k");
    assert_eq!(a["inputs"][0]["via"][0]["by"], "fake/media");
    assert!(a["inputs"][0]["via"][0]["sha256"].is_string());
    assert_eq!(a["model"]["name"], "fake-voice");
    assert_eq!(a["model"]["sha256"], "1".repeat(64));
    let vtt = std::fs::read_to_string(
        f.item_dir(&id)
            .join("derived/runs")
            .join(a["id"].as_str().unwrap())
            .join("asr.vtt"),
    )
    .unwrap();
    assert!(vtt.contains("咖啡"), "{vtt}");
    assert!(engine.transform_jobs(&id).unwrap().is_empty(), "都做好了");
    // 中间的 WAV 不留
    let cache = f.machine().join("cache/transform");
    assert!(!cache.exists() || walk(&cache).is_empty());
    // 原样没动
    assert_eq!(walk(&f.item_dir(&id).join("raw")), raw_before);
    // 落地时发了这一条的事件
    let mut updated = false;
    while let Ok(e) = events.try_recv() {
        updated |= matches!(e, usuyo_core::view::Event::ItemUpdated { id: ref x } if *x == id);
    }
    assert!(updated);

    // 再转一次：并排一份新的，`redoOf` 指旧的
    let mut options = Map::new();
    options.insert("vadThreshold".into(), json!(0.4));
    engine
        .redo_transform(&id, "asr", Some("fake/asr"), options)
        .unwrap();
    assert_eq!(engine.run_transforms_due().await.unwrap(), 1);
    let asr = f.jobs_of(&f.runs(&id), "asr");
    assert_eq!(asr.len(), 2);
    assert_eq!(asr[0]["trigger"], "redo");
    assert_eq!(asr[0]["redoOf"], asr[1]["id"]);
    assert_eq!(asr[0]["options"]["vadThreshold"], 0.4);
    assert!(matches!(
        engine.redo_transform(&id, "asr", Some("fake/media"), Map::new()),
        Err(usuyo_core::Error::InvalidArgs(_))
    ));

    // 同一份内容再取一次：都做好了，不再排
    engine.refetch(&id, &Links::default()).unwrap();
    assert_eq!(engine.run_due().await.unwrap(), 1);
    assert!(engine.transform_jobs(&id).unwrap().is_empty());
}

fn walk(dir: &Path) -> Vec<(PathBuf, Vec<u8>)> {
    let mut out = Vec::new();
    let Ok(entries) = std::fs::read_dir(dir) else {
        return out;
    };
    for e in entries {
        let p = e.unwrap().path();
        if p.is_dir() {
            out.extend(walk(&p));
        } else {
            out.push((p.clone(), std::fs::read(&p).unwrap()));
        }
    }
    out.sort();
    out
}

#[tokio::test]
async fn group_pairs_without_measuring_and_unpaired_files_are_not_merged() {
    let f = Fixture::new();
    let engine = f.engine();
    f.place_voice();
    // 写了同一个 group：时长差得远也照它配
    let grouped = f.take(&engine, "grouped").await;
    // 没写 group、时长差得远：不合，各自照常转
    let apart = f.take(&engine, "apart").await;
    engine.run_transforms_due().await.unwrap();
    assert_eq!(f.jobs_of(&f.runs(&grouped), "play").len(), 1);
    let runs = f.runs(&apart);
    assert!(f.jobs_of(&runs, "play").is_empty(), "{runs:?}");
    assert_eq!(f.jobs_of(&runs, "asr").len(), 1);
    assert_eq!(f.jobs_of(&runs, "keyframes").len(), 1);
    assert!(
        engine.transform_jobs(&apart).unwrap().is_empty(),
        "配不上的不留着排"
    );
}

#[tokio::test]
async fn a_video_without_sound_gets_a_host_written_no_audio_track_run() {
    let f = Fixture::new();
    let engine = f.engine();
    let id = f.take(&engine, "silent").await;
    engine.run_transforms_due().await.unwrap();
    let asr = f.jobs_of(&f.runs(&id), "asr");
    assert_eq!(asr.len(), 1, "不用等模型");
    let a = &asr[0];
    assert_eq!(a["by"], "host");
    assert_eq!(a["empty"], true);
    assert_eq!(a["reason"], "no-audio-track");
    assert_eq!(a["marks"][0]["code"], "no-audio-track");
    assert_eq!(a["tool"], Value::Null);
    assert_eq!(a["inputs"][0]["via"][0]["op"], "probe");
    assert_eq!(a["inputs"][0]["via"][0]["tool"]["name"], "fake-ffmpeg");
}

#[tokio::test]
async fn images_are_read_once_per_image_key() {
    let f = Fixture::new();
    let engine = f.engine();
    let id = f.take(&engine, "note").await;
    engine.run_transforms_due().await.unwrap();
    let ocr = f.jobs_of(&f.runs(&id), "ocr");
    assert_eq!(ocr.len(), 1);
    assert_eq!(
        ocr[0]["inputs"].as_array().unwrap().len(),
        1,
        "两张一样的图只认一次"
    );
    assert!(engine.transform_jobs(&id).unwrap().is_empty());
}

#[tokio::test]
async fn on_battery_only_what_you_opened_runs() {
    let f = Fixture::new();
    f.battery.0.store(true, Ordering::SeqCst);
    let engine = f.engine();
    let note = f.take(&engine, "note").await;
    let silent = f.take(&engine, "silent").await;
    assert_eq!(engine.run_transforms_due().await.unwrap(), 0);
    assert_eq!(
        state(&engine, &note, TransformJobKind::Ocr),
        Some((TransformState::Waiting, Some("power".into()), 0))
    );
    assert!(!engine.on_mains_power());
    // 打开了这一条：在电池上也转
    engine.transform_now(&note).unwrap();
    assert_eq!(engine.run_transforms_due().await.unwrap(), 1);
    assert_eq!(f.jobs_of(&f.runs(&note), "ocr").len(), 1);
    assert!(f.runs(&silent).is_empty());
    // 接上电：剩下的接着
    f.battery.0.store(false, Ordering::SeqCst);
    engine.run_transforms_due().await.unwrap();
    assert!(!f.jobs_of(&f.runs(&silent), "keyframes").is_empty());
}

#[tokio::test]
async fn failures_back_off_and_stop_after_the_last_try() {
    let f = Fixture::new();
    let engine = f.engine();
    let id = f.take(&engine, "silent").await;
    let mut options = Map::new();
    options.insert("fail".into(), json!(true));
    engine.run_transforms_due().await.unwrap();
    engine
        .redo_transform(&id, "keyframes", None, options)
        .unwrap();
    assert_eq!(engine.run_transforms_due().await.unwrap(), 3, "试满三次");
    let (s, _, attempts) = state(&engine, &id, TransformJobKind::Keyframes).unwrap();
    assert_eq!((s, attempts), (TransformState::Failed, 2));
    let job = engine
        .transform_jobs(&id)
        .unwrap()
        .into_iter()
        .find(|j| j.job == TransformJobKind::Keyframes)
        .unwrap();
    assert_eq!(job.error_code.as_deref(), Some("internal"));
}

#[tokio::test]
async fn trash_and_keep_text_drop_the_jobs() {
    let f = Fixture::new();
    let engine = f.engine();
    let a = f.take(&engine, "dash").await;
    let b = f.take(&engine, "silent").await;
    assert!(!engine.transform_jobs(&a).unwrap().is_empty());
    engine.trash(&a, Via::App, &Links::default()).unwrap();
    assert!(engine.transform_jobs(&a).unwrap().is_empty());
    engine.text_only(&b, Via::App, &Links::default()).unwrap();
    assert!(engine.transform_jobs(&b).unwrap().is_empty());
    assert_eq!(engine.run_transforms_due().await.unwrap(), 0);
    assert!(f.runs(&b).is_empty());
    // 拿回来：又排上
    engine.restore(&a, Via::App, &Links::default()).unwrap();
    assert!(!engine.transform_jobs(&a).unwrap().is_empty());
}

#[tokio::test]
async fn with_auto_off_only_opening_transforms() {
    let f = Fixture::new();
    let engine = f.engine_with(|c| c.transforms_auto = false);
    let id = f.take(&engine, "note").await;
    assert!(engine.transform_jobs(&id).unwrap().is_empty());
    assert_eq!(engine.run_transforms_due().await.unwrap(), 0);
    engine.transform_now(&id).unwrap();
    assert_eq!(engine.run_transforms_due().await.unwrap(), 1);
    let ocr = f.jobs_of(&f.runs(&id), "ocr");
    assert_eq!(ocr[0]["trigger"], "open");
}

#[tokio::test]
async fn restart_backfills_what_is_not_done() {
    let f = Fixture::new();
    let id = {
        let engine = f.engine_with(|c| c.transforms_auto = false);
        f.take(&engine, "note").await
    };
    // 下次打开时 auto 开着：补排，按导入那一档
    let engine = f.engine();
    let worker = {
        let e = engine.clone();
        tokio::spawn(async move { e.run_worker().await })
    };
    for _ in 0..200 {
        if !f.jobs_of(&f.runs(&id), "ocr").is_empty() {
            break;
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    worker.abort();
    engine.shutdown().await;
    assert_eq!(f.jobs_of(&f.runs(&id), "ocr").len(), 1);
}
