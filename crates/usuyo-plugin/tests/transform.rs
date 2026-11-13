//! 常驻转换进程对着假插件（sh 脚本，说换行分隔的 JSON-RPC）跑：`describe` 核对、`run` 和输出目录、
//! 超时、卡住、取消、闲了关、进程组、坏消息、stderr、报错里没有令牌和绝对路径。
//! 计时都调到毫秒；插件那一侧等的时间远长于宿主的上限，结果不靠运气。
#![cfg(all(unix, feature = "host"))]

use std::os::unix::fs::{MetadataExt, PermissionsExt};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use serde_json::{Map, Value, json};
use tempfile::TempDir;
use usuyo_plugin::protocol::{ConsumeKind, ErrorCode, Produce, ProviderGrant, RunInput, RunItem};
use usuyo_plugin::{
    CancellationToken, PluginError, ResidentTransform, TransformConfig, TransformLimits,
    TransformRequest, discover,
};

const MANIFEST: &str = r#"protocol: 1
id: fake/transform
kind: transform
name: Fake
version: 1.0.0
license: MIT
maintainers: [someone]
consumes: [audio:pcm16k, {kind: image, mime: [image/png]}]
produces: [asr, keyframes]
provider: optional
models:
  - name: fake-model
    url: https://example.com/fake-model.tar.bz2
    sha256: 0000000000000000000000000000000000000000000000000000000000000000
    size: 10
    format: tar.bz2
    license: MIT
permissions:
  fs: {read: ["${INPUT}", "${MODELS}"], write: ["${OUT}"]}
run:
  command: ["${PLUGIN_DIR}/plugin.sh"]
"#;

/// 假插件的骨架：每收到一行，认出 `id`、`method`，`run` 时另认出 `out` 和第一个输入 `in0`。
/// `reply`、`fail`、`progress` 往 stdout 写一行；每起一次进程往 `${CACHE}/pids` 记一行 pid。
const SCRIPT: &str = r#"#!/bin/sh
echo $$ >> "$USUYO_CACHE_DIR/pids"
reply() { printf '{"jsonrpc":"2.0","id":%s,"result":%s}\n' "$id" "$1"; }
fail() { printf '{"jsonrpc":"2.0","id":%s,"error":%s}\n' "$id" "$1"; }
progress() { printf '{"jsonrpc":"2.0","method":"progress","params":{"id":%s,"done":1,"total":2}}\n' "$id"; }
OK='{"outputs":[{"kind":"asr","path":"asr.vtt","lang":"zh"}],"tool":{"name":"fake","version":"1"},"options":{"x":1},"marks":[],"empty":false}'
while IFS= read -r line; do
  id=$(printf '%s' "$line" | sed -n 's/^{"jsonrpc":"2.0","id":\([0-9]*\),.*/\1/p')
  method=$(printf '%s' "$line" | sed -n 's/^{"jsonrpc":"2.0",\("id":[0-9]*,\)\{0,1\}"method":"\([a-z]*\)".*/\2/p')
  case "$method" in
    describe)
      :
      @DESCRIBE@
      ;;
    health) reply '{"ready":true,"checks":[{"name":"model","ok":true}]}' ;;
    run)
      out=$(printf '%s' "$line" | sed -n 's/.*"out":"\([^"]*\)".*/\1/p')
      in0=$(printf '%s' "$line" | sed -n 's/.*"inputs":\[{[^}]*"path":"\([^"]*\)".*/\1/p')
      :
      @RUN@
      ;;
    cancel)
      id=$(printf '%s' "$line" | sed -n 's/.*"params":{"id":\([0-9]*\)}.*/\1/p')
      :
      @CANCEL@
      ;;
  esac
done
:
@AFTER@
"#;

const DESCRIBE_OK: &str = r#"reply '{"protocol":1,"version":"1.0.0","models":["fake-model"]}'"#;
const RUN_OK: &str = r#"cat "$in0" > "$out/asr.vtt"; reply "$OK""#;
const TOKEN: &str = "tok-q8Zr2VbN4kTxW1mYc7HdLs0J";

#[derive(Clone, Copy)]
struct Script<'a> {
    describe: &'a str,
    run: &'a str,
    cancel: &'a str,
    after: &'a str,
}

impl Default for Script<'_> {
    fn default() -> Self {
        Self {
            describe: DESCRIBE_OK,
            run: RUN_OK,
            cancel: "",
            after: "",
        }
    }
}

struct Fixture {
    tmp: TempDir,
    transform: ResidentTransform,
    /// 资料库里的输入文件。
    input: PathBuf,
}

fn run_script(run: &str) -> Fixture {
    fixture(
        Script {
            run,
            ..Script::default()
        },
        |_| {},
    )
}

fn fixture(script: Script<'_>, tune: impl FnOnce(&mut TransformLimits)) -> Fixture {
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp.path().join("plugins/fake/transform/1.0.0");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("usuyo-plugin.yaml"), MANIFEST).unwrap();
    let body = SCRIPT
        .replace("@DESCRIBE@", script.describe)
        .replace("@RUN@", script.run)
        .replace("@CANCEL@", script.cancel)
        .replace("@AFTER@", script.after);
    let path = dir.join("plugin.sh");
    std::fs::write(&path, body).unwrap();
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).unwrap();

    let input = tmp.path().join("library/items/x/raw/s/media/audio.wav");
    std::fs::create_dir_all(input.parent().unwrap()).unwrap();
    std::fs::write(&input, "pcm-bytes").unwrap();

    let found = discover(&tmp.path().join("plugins")).unwrap();
    assert!(found.rejected.is_empty(), "{:?}", found.rejected);
    let plugin = found
        .transform
        .into_iter()
        .next()
        .expect("a transform plugin");
    // 起进程和 `describe` 只在成功时等满；机器忙时起一个新写的脚本要好几秒
    let mut limits = TransformLimits {
        describe: Duration::from_secs(30),
        health: Duration::from_secs(30),
        run: Duration::from_secs(10),
        stuck: Duration::from_secs(10),
        idle: Duration::from_secs(60),
        stop: Duration::from_millis(500),
        grace: Duration::from_millis(300),
        frame_bytes: 64 << 10,
        stderr_lines: 20,
        out_bytes: 1 << 20,
    };
    tune(&mut limits);
    let config = TransformConfig {
        work_dir: tmp.path().join("library/.usuyo/tmp"),
        cache_dir: tmp.path().join("cache"),
        models_dir: tmp.path().join("models"),
        locale: usuyo_plugin::Locale::new("zh-Hans"),
        limits,
    };
    Fixture {
        transform: ResidentTransform::new(plugin, config),
        input,
        tmp,
    }
}

impl Fixture {
    fn request(&self) -> TransformRequest {
        TransformRequest {
            item: RunItem {
                id: "fixture-1".into(),
                kind: "video".into(),
                title: "手冲咖啡".into(),
                lang: Some("zh".into()),
            },
            want: vec![Produce::Asr],
            inputs: vec![RunInput {
                kind: ConsumeKind::AudioPcm16k,
                mime: None,
                path: self.input.clone(),
                snapshot: Some("019a6f2b-4a10-7c3e-8b21-0d9e8f7a6b5c".into()),
                image_key: None,
                frame_sec: None,
                role: None,
            }],
            options: Map::new(),
            provider: None,
        }
    }

    async fn run(&self) -> Result<usuyo_plugin::TransformOutput, PluginError> {
        self.transform
            .run(self.request(), &CancellationToken::new())
            .await
    }

    fn cache(&self) -> PathBuf {
        self.tmp.path().join("cache/fake/transform")
    }

    fn pids(&self) -> Vec<String> {
        std::fs::read_to_string(self.cache().join("pids"))
            .unwrap_or_default()
            .lines()
            .map(str::to_owned)
            .collect()
    }

    fn pid(&self) -> String {
        self.pids().last().cloned().expect("the plugin started")
    }

    /// 进程暂存目录里还在的 `in/`、`out/` 子目录。
    fn call_dirs(&self) -> Vec<PathBuf> {
        let mut found = Vec::new();
        let work = self.tmp.path().join("library/.usuyo/tmp");
        for proc in std::fs::read_dir(work).into_iter().flatten().flatten() {
            for side in ["in", "out"] {
                for call in std::fs::read_dir(proc.path().join(side))
                    .into_iter()
                    .flatten()
                    .flatten()
                {
                    found.push(call.path());
                }
            }
        }
        found
    }

    /// 报错里不许有令牌，也不许有这台机器上的绝对路径（插件给的字里的目录换成了占位符）。
    fn assert_clean(&self, e: &PluginError) {
        let mut texts = vec![e.to_string(), format!("{e:?}")];
        if let PluginError::Call {
            stderr, failure, ..
        } = e
        {
            texts.extend(stderr.iter().cloned());
            texts.extend(failure.hint.clone());
        }
        let root = self.tmp.path().to_string_lossy().into_owned();
        let real = self
            .tmp
            .path()
            .canonicalize()
            .unwrap()
            .to_string_lossy()
            .into_owned();
        for t in texts {
            assert!(!t.contains(TOKEN), "token leaked: {t}");
            assert!(
                !t.contains(&root) && !t.contains(&real),
                "absolute path leaked: {t}"
            );
        }
    }
}

fn failure_of(result: Result<impl std::fmt::Debug, PluginError>) -> (PluginError, ErrorCode) {
    match result {
        Err(e @ PluginError::Call { .. }) => {
            let PluginError::Call { failure, .. } = &e else {
                unreachable!()
            };
            let code = failure.code;
            (e, code)
        }
        other => panic!("expected a call failure, got {other:?}"),
    }
}

fn message(e: &PluginError) -> String {
    match e {
        PluginError::Call { failure, .. } => failure.message.clone(),
        other => other.to_string(),
    }
}

fn alive(pid: &str) -> bool {
    std::process::Command::new("kill")
        .args(["-0", pid.trim()])
        .stderr(std::process::Stdio::null())
        .status()
        .unwrap()
        .success()
}

/// 等这个进程没了：没了就回，最多等 10 秒（宿主那一侧的宽限都在 1 秒以内）。
/// 异步地等，宿主的闲置计时照样走。
async fn assert_gone(pid: &str) {
    let deadline = Instant::now() + Duration::from_secs(10);
    while alive(pid) && Instant::now() < deadline {
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    assert!(!alive(pid), "process {pid} survived");
}

/// 插件进了 `run` 以后取消：等它在 `${CACHE}` 里写下 `running`。
fn cancel_once_running(f: &Fixture) -> CancellationToken {
    let cancel = CancellationToken::new();
    let trigger = cancel.clone();
    let marker = f.cache().join("running");
    tokio::spawn(async move {
        let deadline = Instant::now() + Duration::from_secs(10);
        while !marker.exists() && Instant::now() < deadline {
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
        trigger.cancel();
    });
    cancel
}

fn read(path: &Path) -> String {
    std::fs::read_to_string(path).unwrap()
}

#[tokio::test]
async fn describe_must_match_the_manifest() {
    for (describe, says) in [
        (r#"reply '{"protocol":1,"version":"2.0.0"}'"#, "version"),
        (r#"reply '{"protocol":9,"version":"1.0.0"}'"#, "protocol"),
        (r#"reply '{"protocol":"one"}'"#, "describe at `protocol`"),
    ] {
        let f = fixture(
            Script {
                describe,
                ..Script::default()
            },
            |_| {},
        );
        let (e, code) = failure_of(f.run().await);
        assert_eq!(code, ErrorCode::InvalidOutput);
        assert!(message(&e).contains(says), "{e}");
        f.assert_clean(&e);
        assert_gone(&f.pid()).await;
        // health 也一样不加载；每次都重起、重核
        let (_, code) = failure_of(f.transform.health().await);
        assert_eq!(code, ErrorCode::InvalidOutput);
        assert_eq!(f.pids().len(), 2);
    }
}

#[tokio::test]
async fn requests_outside_the_manifest_are_refused_before_starting() {
    let f = run_script(RUN_OK);
    let cancel = CancellationToken::new();
    let mut wrong_want = f.request();
    wrong_want.want = vec![Produce::Play];
    let mut twice = f.request();
    twice.want = vec![Produce::Asr, Produce::Asr];
    let mut nothing = f.request();
    nothing.want.clear();
    let mut wrong_kind = f.request();
    wrong_kind.inputs[0].kind = ConsumeKind::Video;
    let mut wrong_mime = f.request();
    wrong_mime.inputs[0].kind = ConsumeKind::Image;
    wrong_mime.inputs[0].mime = Some("image/jpeg".into());
    for request in [wrong_want, twice, nothing, wrong_kind, wrong_mime] {
        let result = f.transform.run(request, &cancel).await;
        assert!(
            matches!(result, Err(PluginError::Unsupported { .. })),
            "{result:?}"
        );
    }
    assert!(f.pids().is_empty(), "the process was started");

    let mut png = f.request();
    png.inputs[0].kind = ConsumeKind::Image;
    png.inputs[0].mime = Some("image/png".into());
    f.transform.run(png, &cancel).await.unwrap();
}

#[tokio::test]
async fn a_run_returns_its_files_and_keeps_the_process() {
    let f = run_script(
        r#"ls -i "$in0" | awk '{print $1}' > "$USUYO_CACHE_DIR/inode"
printf '%s\n' "$in0" > "$USUYO_CACHE_DIR/in0"
env > "$USUYO_CACHE_DIR/env"
pwd > "$USUYO_CACHE_DIR/cwd"
cat "$in0" > "$out/asr.vtt"; reply "$OK""#,
    );
    let health = f.transform.health().await.unwrap();
    assert!(health.ready);
    let first = f.run().await.unwrap();
    assert_eq!(first.result.outputs[0].kind, Produce::Asr);
    assert_eq!(first.result.tool.name, "fake");
    assert_eq!(first.result.options["x"], json!(1));
    let paths: Vec<_> = first.files.iter().map(|f| f.path.as_str()).collect();
    assert_eq!(paths, ["asr.vtt"]);
    assert_eq!(read(&first.out_dir().join("asr.vtt")), "pcm-bytes");

    // 输入是复制进 `in/<callId>/<下标>/` 的，插件看不到资料库里的路径；跑完 `in/` 删掉
    let in0 = read(&f.cache().join("in0"));
    assert!(in0.trim_end().ends_with("/in/3/0/audio.wav"), "{in0}");
    assert!(!in0.contains("library/items"), "{in0}");
    assert!(!Path::new(in0.trim_end()).exists());
    let inode = read(&f.cache().join("inode"));
    assert_ne!(
        inode.trim(),
        std::fs::metadata(&f.input).unwrap().ino().to_string(),
        "a copy, not a hard link"
    );
    // 交出去的输出目录不在进程的暂存目录里
    assert!(!first.out_dir().to_string_lossy().contains("usuyo-proc-"));

    let env = read(&f.cache().join("env"));
    let models = f.tmp.path().join("models/fake/transform");
    assert!(
        env.lines()
            .any(|l| l == format!("USUYO_MODELS_DIR={}", models.display())),
        "{env}"
    );
    assert!(env.lines().any(|l| l == "USUYO_PROTOCOL=1"), "{env}");
    assert!(!env.contains("USUYO_CALL_ID"), "{env}");
    assert!(!env.contains("CARGO_"), "host environment leaked: {env}");
    assert!(read(&f.cache().join("cwd")).contains("usuyo-proc-"));

    // 同一个进程接着处理下一条
    let second = f.run().await.unwrap();
    assert_eq!(f.pids().len(), 1);
    assert_ne!(first.out_dir(), second.out_dir());

    // 输出目录交给调用方；丢掉时删掉，进程关了也不影响还拿着的
    let out = first.out_dir().to_owned();
    f.transform.shutdown().await;
    assert_gone(&f.pid()).await;
    assert!(out.join("asr.vtt").is_file());
    drop(first);
    assert!(!out.exists());
    drop(second);
    assert!(f.call_dirs().is_empty(), "{:?}", f.call_dirs());
}

#[tokio::test]
async fn writing_to_an_input_leaves_the_original_alone() {
    let f = run_script(r#"printf evil > "$in0"; cat "$in0" > "$out/asr.vtt"; reply "$OK""#);
    let out = f.run().await.unwrap();
    assert_eq!(read(&out.out_dir().join("asr.vtt")), "evil");
    assert_eq!(read(&f.input), "pcm-bytes");
}

#[tokio::test]
async fn writes_after_the_reply_do_not_reach_the_output() {
    // 回答以后接着改输出：等测试放出 `go` 再写，写完留下 `done`
    let f = run_script(
        r#"cat "$in0" > "$out/asr.vtt"; reply "$OK"
( while [ ! -f "$USUYO_CACHE_DIR/go" ]; do sleep 0.02; done
  printf swapped > "$out/asr.vtt" 2>/dev/null; printf more > "$out/extra" 2>/dev/null
  touch "$USUYO_CACHE_DIR/done" ) &"#,
    );
    let out = f.run().await.unwrap();
    std::fs::write(f.cache().join("go"), "").unwrap();
    let deadline = Instant::now() + Duration::from_secs(10);
    while !f.cache().join("done").exists() {
        assert!(Instant::now() < deadline, "the plugin never tried to write");
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    assert_eq!(read(&out.out_dir().join("asr.vtt")), "pcm-bytes");
    assert!(!out.out_dir().join("extra").exists());
    let paths: Vec<_> = out.files.iter().map(|f| f.path.as_str()).collect();
    assert_eq!(paths, ["asr.vtt"]);
}

#[tokio::test]
async fn keyframes_images_ride_along() {
    let f = run_script(
        r#"mkdir -p "$out/keyframes"
printf '{}' > "$out/keyframes.json"
printf 'jpg' > "$out/keyframes/00000500.jpg"
reply '{"outputs":[{"kind":"keyframes","path":"keyframes.json"}],"tool":{"name":"ffmpeg","version":"9.0.2"},"marks":[{"code":"static-picture","frozenSec":4.5,"durationSec":5}]}'"#,
    );
    let mut request = f.request();
    request.want = vec![Produce::Keyframes];
    let out = f
        .transform
        .run(request, &CancellationToken::new())
        .await
        .unwrap();
    let paths: Vec<_> = out.files.iter().map(|f| f.path.as_str()).collect();
    assert_eq!(paths, ["keyframes.json", "keyframes/00000500.jpg"]);
}

#[tokio::test]
async fn out_must_hold_exactly_what_was_reported() {
    for (run, says) in [
        (
            r#"ln -s /etc/hosts "$out/asr.vtt"; reply "$OK""#,
            "symbolic links",
        ),
        (
            r#"printf 'x' > "$out/asr.vtt"; printf 'y' > "$out/extra.txt"; reply "$OK""#,
            "`extra.txt` is in `out` but no output reports it",
        ),
        (
            r#"printf 'x' > "$out/asr.vtt"; mkdir "$out/keyframes"; printf 'y' > "$out/keyframes/1.jpg"; reply "$OK""#,
            "`keyframes/1.jpg` is in `out`",
        ),
        (r#"reply "$OK""#, "not a file in `out`"),
        (
            r#"printf 'x' > "$out/asr.vtt"; ln "$out/asr.vtt" "$out/copy.vtt"; reply "$OK""#,
            "hard links",
        ),
        (
            r#"head -c 2000000 /dev/zero > "$out/asr.vtt"; reply "$OK""#,
            "larger than",
        ),
        (
            r#"printf 'x' > "$out/asr.vtt"; reply '{"outputs":[{"kind":"asr","path":"asr.vtt"}],"tool":{"name":"fake","version":"1"},"model":{"name":"other"}}'"#,
            "model `other`",
        ),
        (
            r#"printf 'x' > "$out/asr.vtt"; reply '{"outputs":[{"kind":"asr","path":"asr.vtt","lang":"z h"}],"tool":{"name":"fake","version":"1"}}'"#,
            "language tag",
        ),
        (
            r#"reply '{"outputs":[{"kind":"keyframes","path":"k.json"}],"tool":{"name":"fake","version":"1"}}'"#,
            "was not wanted",
        ),
        (
            r#"reply '{"outputs":"asr.vtt","tool":{"name":"fake","version":"1"}}'"#,
            "run result at `outputs`",
        ),
    ] {
        let f = run_script(run);
        let (e, code) = failure_of(f.run().await);
        assert_eq!(code, ErrorCode::InvalidOutput, "{run}: {e}");
        assert!(message(&e).contains(says), "{run}: {e}");
        f.assert_clean(&e);
        // 进程守着协议，只是这次的输出不对：留着；输出目录删掉
        assert!(alive(&f.pid()));
        assert!(f.call_dirs().is_empty(), "{:?}", f.call_dirs());
    }
}

#[tokio::test]
async fn run_timeout_stops_the_process() {
    // 子进程在 `describe` 时起好：`run` 的时钟在那以后才开始走，0.3 秒到了它一定在
    let f = fixture(
        Script {
            describe: concat!(
                "sleep 30 > /dev/null 2>&1 &\n",
                "echo $! > \"$USUYO_CACHE_DIR/child\"\n",
                "reply '{\"protocol\":1,\"version\":\"1.0.0\"}'"
            ),
            run: ":",
            ..Script::default()
        },
        |l| l.run = Duration::from_millis(300),
    );
    let (e, code) = failure_of(f.run().await);
    assert_eq!(code, ErrorCode::Timeout);
    assert!(message(&e).contains("took longer than 300ms"), "{e}");
    f.assert_clean(&e);
    let first = f.pid();
    let child = read(&f.cache().join("child"));
    assert_gone(&first).await;
    assert_gone(&child).await;
    assert!(f.call_dirs().is_empty());

    // 下一次重起一个进程，重新核 describe
    let (_, code) = failure_of(f.run().await);
    assert_eq!(code, ErrorCode::Timeout);
    assert_eq!(f.pids().len(), 2);
    assert_ne!(f.pid(), first);
}

#[tokio::test]
async fn a_flood_of_progress_still_times_out() {
    let f = fixture(
        Script {
            run: "while :; do progress; done",
            ..Script::default()
        },
        |l| l.run = Duration::from_millis(500),
    );
    let (e, code) = failure_of(f.run().await);
    assert_eq!(code, ErrorCode::Timeout);
    assert!(message(&e).contains("took longer than 500ms"), "{e}");
    assert_gone(&f.pid()).await;
}

#[tokio::test]
async fn silence_after_progress_is_stuck() {
    let f = fixture(
        Script {
            run: "progress",
            ..Script::default()
        },
        |l| l.stuck = Duration::from_millis(300),
    );
    // 总时长是 10 秒：是卡住的规则停的，不是总时长
    let (e, code) = failure_of(f.run().await);
    assert_eq!(code, ErrorCode::Timeout);
    assert!(message(&e).contains("no `progress` for 300ms"), "{e}");
    assert_gone(&f.pid()).await;
}

#[tokio::test]
async fn steady_progress_is_not_stuck() {
    // 每 0.25 秒一条，一共 3 秒，比卡住的 2 秒长；一次慢的 fork 也差得远
    let f = fixture(
        Script {
            run: r#"for i in 1 2 3 4 5 6 7 8 9 10 11 12; do progress; sleep 0.25; done
cat "$in0" > "$out/asr.vtt"; reply "$OK""#,
            ..Script::default()
        },
        |l| l.stuck = Duration::from_secs(2),
    );
    f.run().await.unwrap();
}

#[tokio::test]
async fn silence_without_progress_waits_for_the_total() {
    // 没发过 progress 的只受总时长约束（协议 §3.4）
    let f = fixture(
        Script {
            run: r#"sleep 0.6; cat "$in0" > "$out/asr.vtt"; reply "$OK""#,
            ..Script::default()
        },
        |l| l.stuck = Duration::from_millis(200),
    );
    f.run().await.unwrap();
}

#[tokio::test]
async fn cancel_that_is_answered_keeps_the_process() {
    let f = fixture(
        Script {
            run: r#"printf 'partial' > "$out/asr.vtt"; : > "$USUYO_CACHE_DIR/running""#,
            cancel: r#"fail '{"code":-32000,"message":"cancelled","data":{"code":"cancelled","message":"cancelled"}}'"#,
            ..Script::default()
        },
        // 插件要几个 sed 才答得回来：宽限给足，答得快不多花时间
        |l| l.grace = Duration::from_secs(5),
    );
    let cancel = cancel_once_running(&f);
    let (e, code) = failure_of(f.transform.run(f.request(), &cancel).await);
    assert_eq!(code, ErrorCode::Cancelled);
    f.assert_clean(&e);
    assert!(alive(&f.pid()));
    assert!(f.call_dirs().is_empty(), "{:?}", f.call_dirs());

    // 已经取消的令牌：不等进程，直接回
    let (_, code) = failure_of(f.transform.run(f.request(), &cancel).await);
    assert_eq!(code, ErrorCode::Cancelled);
    assert_eq!(f.pids().len(), 1);
}

#[tokio::test]
async fn cancel_that_is_ignored_stops_the_process() {
    let f = run_script(r#": > "$USUYO_CACHE_DIR/running""#);
    let cancel = cancel_once_running(&f);
    let (_, code) = failure_of(f.transform.run(f.request(), &cancel).await);
    assert_eq!(code, ErrorCode::Cancelled);
    assert_gone(&f.pid()).await;
}

#[tokio::test]
async fn idle_process_is_stopped_and_restarted_on_demand() {
    let f = fixture(Script::default(), |l| l.idle = Duration::from_millis(300));
    f.run().await.unwrap();
    let first = f.pid();
    assert_gone(&first).await;
    f.run().await.unwrap();
    assert_eq!(f.pids().len(), 2);
}

#[tokio::test]
async fn busy_process_is_not_idle() {
    // 一次 run 比闲置时长还久：跑着的时候不关
    let f = fixture(
        Script {
            run: r#"sleep 0.6; cat "$in0" > "$out/asr.vtt"; reply "$OK""#,
            ..Script::default()
        },
        |l| l.idle = Duration::from_millis(200),
    );
    f.run().await.unwrap();
    assert_eq!(f.pids().len(), 1);
}

#[tokio::test]
async fn shutdown_escalates_to_the_whole_group() {
    // 不理 stdin 关闭、也不理 SIGTERM：最后 SIGKILL 整组，连子进程
    let f = fixture(
        Script {
            after: "sleep 30 &\necho $! > \"$USUYO_CACHE_DIR/child\"\nwait",
            ..Script::default()
        },
        |_| {},
    );
    std::fs::write(
        f.tmp.path().join("plugins/fake/transform/1.0.0/plugin.sh"),
        read(&f.tmp.path().join("plugins/fake/transform/1.0.0/plugin.sh")).replacen(
            "#!/bin/sh\n",
            "#!/bin/sh\ntrap '' TERM\n",
            1,
        ),
    )
    .unwrap();
    f.run().await.unwrap();
    f.transform.shutdown().await;
    assert_gone(&f.pid()).await;
    assert_gone(&read(&f.cache().join("child"))).await;
}

#[tokio::test]
async fn exit_mid_run_is_a_crash_and_takes_the_group() {
    for run in [
        "exit 3",
        // 留下的子进程拿着 stdout：宿主不等 stdout 关上，看进程退出
        "sleep 30 &\necho $! > \"$USUYO_CACHE_DIR/child\"\nexit 3",
    ] {
        // 等 stdout 关上的话，要到 10 秒的总时长才回 `timeout`
        let f = run_script(run);
        let (e, code) = failure_of(f.run().await);
        assert_eq!(code, ErrorCode::Crashed, "{run}");
        assert!(message(&e).contains("exited during `run`"), "{e}");
        f.assert_clean(&e);
        if let Ok(child) = std::fs::read_to_string(f.cache().join("child")) {
            assert_gone(&child).await;
        }
        // 下一次重起
        let (_, code) = failure_of(f.run().await);
        assert_eq!(code, ErrorCode::Crashed);
        assert_eq!(f.pids().len(), 2);
    }
}

#[tokio::test]
async fn bad_frames_stop_the_process() {
    for (run, says) in [
        ("echo hello", "not a JSON object"),
        // 写完坏消息马上退出：按坏消息算，不按退出
        ("echo hello; exit 0", "not a JSON object"),
        (
            "head -c 100000 /dev/zero | tr '\\0' a; echo",
            "longer than 65536 bytes",
        ),
        (
            r#"printf '{"jsonrpc":"2.0","id":999,"result":{}}\n'"#,
            "not waiting",
        ),
        (
            r#"printf '{"jsonrpc":"2.0","id":5,"method":"ask","params":{}}\n'"#,
            "sent a request",
        ),
        (
            r#"printf '{"jsonrpc":"2.0","method":"progress","params":{}}\n'"#,
            "`progress` has the wrong shape",
        ),
    ] {
        let f = run_script(run);
        let (e, code) = failure_of(f.run().await);
        assert_eq!(code, ErrorCode::InvalidOutput, "{run}: {e}");
        assert!(message(&e).contains(says), "{run}: {e}");
        f.assert_clean(&e);
        assert_gone(&f.pid()).await;
    }
}

#[tokio::test]
async fn stray_output_between_runs_restarts_the_process() {
    let f = run_script(r#"cat "$in0" > "$out/asr.vtt"; reply "$OK"; echo junk"#);
    f.run().await.unwrap();
    // `junk` 在第二次之前读到了就重起；还没读到就是第二次的坏消息，进程也关掉
    if let Err(e) = f.run().await {
        assert!(message(&e).contains("not a JSON object"), "{e}");
    }
    f.run().await.unwrap();
    assert!(f.pids().len() >= 2, "{:?}", f.pids());
}

#[tokio::test]
async fn a_dropped_run_retires_the_process() {
    let f = run_script(
        r#": > "$USUYO_CACHE_DIR/running"
if [ -e "$USUYO_CACHE_DIR/answer" ]; then cat "$in0" > "$out/asr.vtt"; reply "$OK"; fi"#,
    );
    let marker = f.cache().join("running");
    let cancel = CancellationToken::new();
    // 放在堆上：`drop` 才真的丢掉 future（连同它拿着的锁）
    let mut run = Box::pin(f.transform.run(f.request(), &cancel));
    // 插件进了 `run` 以后丢掉 future，不发 `cancel`
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        tokio::select! {
            result = &mut run => panic!("the run should not finish: {result:?}"),
            () = tokio::time::sleep(Duration::from_millis(10)) => {}
        }
        if marker.exists() || Instant::now() > deadline {
            break;
        }
    }
    drop(run);
    let first = f.pid();
    std::fs::write(f.cache().join("answer"), "").unwrap();
    f.run().await.unwrap();
    assert_eq!(f.pids().len(), 2);
    assert_gone(&first).await;
}

#[tokio::test]
async fn stderr_noise_does_not_disturb() {
    let f = run_script(
        r#"i=0; while [ $i -lt 3000 ]; do echo "noise $i {\"jsonrpc\":\"2.0\",\"id\":1}" >&2; i=$((i+1)); done
head -c 50000 /dev/zero | tr '\0' x >&2; echo >&2
if [ -e "$USUYO_CACHE_DIR/fail" ]; then fail '{"code":-32000,"message":"x","data":{"code":"internal","message":"boom"}}'
else cat "$in0" > "$out/asr.vtt"; reply "$OK"; fi"#,
    );
    f.run().await.unwrap();
    std::fs::write(f.cache().join("fail"), "").unwrap();
    let (e, code) = failure_of(f.run().await);
    assert_eq!(code, ErrorCode::Internal);
    let PluginError::Call { stderr, .. } = &e else {
        unreachable!()
    };
    assert_eq!(stderr.len(), 20, "{stderr:?}");
    assert!(stderr.iter().all(|l| l.len() <= 8 * 1024));
    assert!(stderr.iter().any(|l| l.starts_with("xxxx")));
}

#[tokio::test]
async fn plugin_failures_come_through_scrubbed() {
    let f = run_script(
        r#"echo "token $(printf '%s' "$line" | sed -n 's/.*"token":"\([^"]*\)".*/\1/p') out $out" >&2
fail "{\"code\":-32000,\"message\":\"x\",\"data\":{\"code\":\"dependency-missing\",\"message\":\"no model in $USUYO_MODELS_DIR for $out, token $(printf '%s' "$line" | sed -n 's/.*"token":"\([^"]*\)".*/\1/p')\",\"hint\":\"cache $USUYO_CACHE_DIR\",\"needs\":[\"model:fake-model\"]}}""#,
    );
    let mut request = f.request();
    request.provider = Some(ProviderGrant {
        base_url: "http://127.0.0.1:47822/v1".into(),
        token: TOKEN.into(),
        model: "summary".into(),
    });
    let (e, code) = failure_of(f.transform.run(request, &CancellationToken::new()).await);
    assert_eq!(code, ErrorCode::DependencyMissing);
    let PluginError::Call {
        failure, stderr, ..
    } = &e
    else {
        unreachable!()
    };
    assert_eq!(failure.needs[0].as_str(), "model:fake-model");
    assert!(
        failure.message.contains("${MODELS}") && failure.message.contains("${PROC}/out/2"),
        "{}",
        failure.message
    );
    assert!(
        failure.message.contains("[redacted]"),
        "{}",
        failure.message
    );
    assert_eq!(failure.hint.as_deref(), Some("cache ${CACHE}"));
    assert!(
        stderr.iter().any(|l| l.contains("[redacted]")),
        "{stderr:?}"
    );
    f.assert_clean(&e);
}

#[tokio::test]
async fn plugin_errors_are_checked() {
    for (run, expected) in [
        (
            r#"fail '{"code":-32000,"message":"x","data":{"code":"timeout","message":"slow"}}'"#,
            ErrorCode::InvalidOutput,
        ),
        (
            r#"fail '{"code":-32000,"message":"x","data":{"code":"dependency-missing","message":"m"}}'"#,
            ErrorCode::InvalidOutput,
        ),
        (
            r#"fail '{"code":-32000,"message":"x"}'"#,
            ErrorCode::InvalidOutput,
        ),
        (
            r#"fail '{"code":-32602,"message":"bad params"}'"#,
            ErrorCode::InvalidArgs,
        ),
        (
            r#"fail '{"code":7,"message":"?"}'"#,
            ErrorCode::InvalidOutput,
        ),
        (r#"fail '"oops"'"#, ErrorCode::InvalidOutput),
        (
            r#"fail '{"code":-32000,"message":"x","data":{"code":"unsupported-input","message":"not audio"}}'"#,
            ErrorCode::UnsupportedInput,
        ),
    ] {
        let f = run_script(run);
        let (e, code) = failure_of(f.run().await);
        assert_eq!(code, expected, "{run}: {e}");
        f.assert_clean(&e);
        // 回了错误的进程守着协议：留着
        assert!(alive(&f.pid()));
    }
}

#[tokio::test]
async fn oversized_request_is_refused() {
    let f = run_script(RUN_OK);
    let mut request = f.request();
    request
        .options
        .insert("prompt".into(), Value::String("a".repeat(100_000)));
    let result = f.transform.run(request, &CancellationToken::new()).await;
    assert!(
        matches!(&result, Err(PluginError::Unsupported { message, .. }) if message.contains("message limit")),
        "{result:?}"
    );
    assert!(f.call_dirs().is_empty());
    // 进程不受影响
    f.run().await.unwrap();
    assert_eq!(f.pids().len(), 1);
}

#[tokio::test]
async fn runs_one_at_a_time() {
    let f = run_script(
        r#"echo "start $id" >> "$USUYO_CACHE_DIR/log"; sleep 0.2; echo "end $id" >> "$USUYO_CACHE_DIR/log"
cat "$in0" > "$out/asr.vtt"; reply "$OK""#,
    );
    let (a, b) = tokio::join!(f.run(), f.run());
    a.unwrap();
    b.unwrap();
    let log = read(&f.cache().join("log"));
    let lines: Vec<&str> = log.lines().collect();
    assert_eq!(lines.len(), 4);
    assert_eq!(&lines[0][..5], "start");
    assert_eq!(&lines[1][..3], "end");
    assert_eq!(lines[0][6..], lines[1][4..]);
}

#[tokio::test]
async fn cancel_during_describe_drops_the_process() {
    // 取消以后 `describe` 才答回来：没核完的进程不留，下一次重起
    let f = fixture(
        Script {
            describe: r#": > "$USUYO_CACHE_DIR/running"; sleep 0.1; reply '{"protocol":1,"version":"1.0.0"}'"#,
            ..Script::default()
        },
        |_| {},
    );
    let cancel = cancel_once_running(&f);
    let (e, code) = failure_of(f.transform.run(f.request(), &cancel).await);
    assert_eq!(code, ErrorCode::Cancelled);
    f.assert_clean(&e);
    assert_gone(&f.pid()).await;
    f.run().await.unwrap();
    assert_eq!(f.pids().len(), 2);
}

/// `cargo test` 顺带编好的 `examples/kit_transform`，在测试可执行文件的上一级目录旁边。
fn kit_example() -> PathBuf {
    let exe = std::env::current_exe().unwrap();
    let profile = exe.parent().and_then(Path::parent).unwrap();
    let path = profile.join("examples/kit_transform");
    assert!(
        path.is_file(),
        "{} is missing: `cargo test` builds the examples unless it is narrowed with `--test`; add `--example kit_transform`",
        path.display()
    );
    path
}

#[tokio::test]
async fn the_kit_loop_speaks_the_same_protocol() {
    let version = env!("CARGO_PKG_VERSION");
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp.path().join(format!("plugins/fake/transform/{version}"));
    std::fs::create_dir_all(&dir).unwrap();
    let manifest = MANIFEST
        .replace("version: 1.0.0", &format!("version: {version}"))
        .replace("${PLUGIN_DIR}/plugin.sh", "${PLUGIN_DIR}/kit_transform");
    std::fs::write(dir.join("usuyo-plugin.yaml"), manifest).unwrap();
    std::fs::copy(kit_example(), dir.join("kit_transform")).unwrap();
    let input = tmp.path().join("library/audio.wav");
    std::fs::create_dir_all(input.parent().unwrap()).unwrap();
    std::fs::write(&input, "kit-bytes").unwrap();
    let plugin = discover(&tmp.path().join("plugins"))
        .unwrap()
        .transform
        .remove(0);
    let transform = ResidentTransform::new(
        plugin,
        TransformConfig {
            work_dir: tmp.path().join("library/.usuyo/tmp"),
            cache_dir: tmp.path().join("cache"),
            models_dir: tmp.path().join("models"),
            locale: usuyo_plugin::Locale::new("zh-Hans"),
            limits: TransformLimits {
                describe: Duration::from_secs(30),
                stuck: Duration::from_secs(10),
                // 不肯退的话要等满这么久
                stop: Duration::from_secs(30),
                grace: Duration::from_secs(2),
                ..TransformLimits::default()
            },
        },
    );
    let request = |wait: bool| TransformRequest {
        item: RunItem {
            id: "fixture-1".into(),
            kind: "video".into(),
            title: "t".into(),
            lang: None,
        },
        want: vec![Produce::Asr],
        inputs: vec![RunInput {
            kind: ConsumeKind::AudioPcm16k,
            mime: None,
            path: input.clone(),
            snapshot: None,
            image_key: None,
            frame_sec: None,
            role: None,
        }],
        options: Map::from_iter([("wait".to_owned(), Value::Bool(wait))]),
        provider: None,
    };

    assert!(transform.health().await.unwrap().ready);
    let out = transform
        .run(request(false), &CancellationToken::new())
        .await
        .unwrap();
    assert_eq!(read(&out.out_dir().join("asr.vtt")), "kit-bytes");
    assert_eq!(out.result.options["wait"], json!(false));

    // 取消：kit 回 `cancelled`，进程留着，接着能用
    let cancel = CancellationToken::new();
    let trigger = cancel.clone();
    tokio::spawn(async move {
        tokio::time::sleep(Duration::from_millis(100)).await;
        trigger.cancel();
    });
    let (_, code) = failure_of(transform.run(request(true), &cancel).await);
    assert_eq!(code, ErrorCode::Cancelled);
    transform
        .run(request(false), &CancellationToken::new())
        .await
        .unwrap();
    // kit 在 stdin 关上以后自己退出
    let started = Instant::now();
    transform.shutdown().await;
    assert!(started.elapsed() < Duration::from_secs(20));
}
