//! 宿主对着假插件（sh 脚本）跑：超时、坏输出、退出码、越界路径、超大输出、事件、取消。
#![cfg(all(unix, feature = "host"))]

use std::collections::BTreeSet;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use tempfile::TempDir;
use tokio::sync::mpsc;
use usuyo_plugin::protocol::{
    Canonicalized, Completeness, ErrorCode, Identity, NotFoundReason, OutputKind, Pause,
    PluginEvent,
};
use usuyo_plugin::{
    CallLimits, CallOptions, CancellationToken, FetchInput, FetchPlugin, FetchRequest, HostConfig,
    ListRequest, PluginError, PluginHost, discover,
};

const MANIFEST: &str = r#"protocol: 1
id: fake/plugin
kind: fetch
name: Fake
version: 1.0.0
license: MIT
maintainers: [someone]
platforms: [example]
match: ["example.com/*"]
access: public-web
login: none
capabilities:
  fetch:
    access: public-web
    login: none
    outputs: [text, html, images, meta]
  list:
    access: public-web
    login: none
    order: newest-first
permissions:
  fs: {write: ["${OUT}"]}
run:
  command: ["${PLUGIN_DIR}/plugin.sh"]
"#;

struct Fixture {
    _tmp: TempDir,
    host: PluginHost,
    work: PathBuf,
}

/// 装一个假插件：`script` 是 sh 的正文，`$1` 是子命令，fetch 时 `$out` 是输出目录。
fn install(script: &str) -> Fixture {
    install_with(script, |_| {})
}

fn install_with(script: &str, tune: impl FnOnce(&mut CallLimits)) -> Fixture {
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp.path().join("plugins/fake/plugin/1.0.0");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("usuyo-plugin.yaml"), MANIFEST).unwrap();
    let path = dir.join("plugin.sh");
    let body =
        format!("#!/bin/sh\nout=\"\"\nif [ \"$1\" = fetch ]; then out=\"$4\"; fi\n{script}\n");
    std::fs::write(&path, body).unwrap();
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).unwrap();

    let found = discover(&tmp.path().join("plugins")).unwrap();
    assert!(found.rejected.is_empty(), "{:?}", found.rejected);
    let mut limits = CallLimits {
        canonicalize: Duration::from_secs(30),
        fetch: Duration::from_secs(30),
        list: Duration::from_secs(30),
        grace: Duration::from_millis(300),
        stdout_bytes: 64 << 10,
        fetch_dir_bytes: 1 << 20,
        ..CallLimits::default()
    };
    tune(&mut limits);
    let work = tmp.path().join("work");
    let config = HostConfig {
        work_dir: work.clone(),
        cache_dir: tmp.path().join("cache"),
        locale: "zh-CN".into(),
        limits,
    };
    Fixture {
        host: PluginHost::new(config, found.fetch),
        work,
        _tmp: tmp,
    }
}

impl Fixture {
    fn plugin(&self) -> &FetchPlugin {
        &self.host.plugins()[0]
    }

    async fn fetch(
        &self,
        want: Option<&[OutputKind]>,
        opts: &CallOptions,
    ) -> Result<usuyo_plugin::FetchedSnapshot, PluginError> {
        let request = FetchRequest {
            input: FetchInput::Url(url::Url::parse("https://example.com/post/1?x=1").unwrap()),
            want: want.map(|w| w.iter().copied().collect()),
            input_file: None,
        };
        self.host.fetch(self.plugin(), request, opts).await
    }

    fn cache(&self) -> PathBuf {
        self.work.parent().unwrap().join("cache/fake/plugin")
    }
}

fn code(result: Result<impl std::fmt::Debug, PluginError>) -> ErrorCode {
    match result {
        Err(PluginError::Call { failure, .. }) => failure.code,
        other => panic!("expected a call failure, got {other:?}"),
    }
}

const OK_FETCH: &str = r#"
printf '<html>hi</html>' > "$out/page.html"
mkdir -p "$out/images" && printf 'png' > "$out/images/001.png"
cat <<'JSON'
{"ok": true, "identity": {"platform": "example", "contentId": "1"}, "kind": "post",
 "title": "标题", "text": "正文", "html": "page.html", "images": ["images/001.png"],
 "media": [], "platformTags": [], "completeness": "partial", "missing": ["images"],
 "evidence": {"fetchedAt": "2026-11-01T03:00:01Z", "access": "public-web"}, "surprise": 1}
JSON
"#;

#[tokio::test]
async fn fetch_succeeds_and_keeps_what_the_plugin_calls_missing() {
    let f = install(&format!(
        "printf '%s\\n' \"$@\" > \"$USUYO_CACHE_DIR/args\"\nenv > \"$USUYO_CACHE_DIR/env\"\n{OK_FETCH}"
    ));
    let snap = f
        .fetch(
            Some(&[OutputKind::Text, OutputKind::Images, OutputKind::Html]),
            &CallOptions::default(),
        )
        .await
        .unwrap();
    // 插件说缺图片，又交来了一张：图片交来一部分、还缺一部分，照插件说的记
    assert_eq!(snap.output.completeness, Completeness::Partial);
    assert_eq!(snap.output.missing, [OutputKind::Images]);
    assert_eq!(
        snap.obtained,
        BTreeSet::from([
            OutputKind::Text,
            OutputKind::Html,
            OutputKind::Images,
            OutputKind::Meta
        ])
    );
    let paths: Vec<_> = snap.files.iter().map(|f| f.path.as_str()).collect();
    assert_eq!(paths, ["images/001.png", "page.html"]);
    assert!(snap.out_dir().join("page.html").is_file());

    let args = std::fs::read_to_string(f.cache().join("args")).unwrap();
    let args: Vec<&str> = args.lines().collect();
    assert_eq!(args[0..2], ["fetch", "https://example.com/post/1?x=1"]);
    assert_eq!(args[2], "--out");
    assert_eq!(args[4..], ["--want", "text,html,images"]);

    let env = std::fs::read_to_string(f.cache().join("env")).unwrap();
    assert!(env.lines().any(|l| l == "USUYO_PROTOCOL=1"), "{env}");
    assert!(env.lines().any(|l| l == "USUYO_LOCALE=zh-CN"));
    assert!(
        env.lines()
            .any(|l| l.starts_with("HOME=") && l.contains("/work/")),
        "{env}"
    );
    assert!(!env.contains("CARGO_"), "host environment leaked: {env}");

    // 丢掉快照，暂存目录一起删掉
    let out = snap.out_dir();
    drop(snap);
    assert!(!out.exists());
}

#[tokio::test]
async fn want_outside_outputs_is_refused_before_running() {
    let f = install("exit 99");
    assert!(matches!(
        f.fetch(Some(&[OutputKind::Video]), &CallOptions::default())
            .await,
        Err(PluginError::Unsupported { .. })
    ));
}

#[tokio::test]
async fn timeout_kills_the_whole_group() {
    let f = install_with(
        "sleep 30 &\necho $! > \"$USUYO_CACHE_DIR/child.pid\"\nsleep 30",
        |l| l.fetch = Duration::from_secs(8),
    );
    let started = Instant::now();
    assert_eq!(
        code(f.fetch(None, &CallOptions::default()).await),
        ErrorCode::Timeout
    );
    assert!(started.elapsed() < Duration::from_secs(20));
    let pid = std::fs::read_to_string(f.cache().join("child.pid")).unwrap();
    let alive = || {
        std::process::Command::new("kill")
            .args(["-0", pid.trim()])
            .stderr(std::process::Stdio::null())
            .status()
            .unwrap()
            .success()
    };
    let deadline = Instant::now() + Duration::from_secs(3);
    while alive() && Instant::now() < deadline {
        std::thread::sleep(Duration::from_millis(50));
    }
    assert!(!alive(), "child of the plugin survived");
}

#[tokio::test]
async fn bad_stdout_and_exit_codes() {
    let cases: [(&str, ErrorCode); 7] = [
        ("echo hello", ErrorCode::Crashed),
        ("exit 3", ErrorCode::Crashed),
        ("echo 'usage' >&2; exit 2", ErrorCode::InvalidArgs),
        ("echo '[1,2]'", ErrorCode::InvalidOutput),
        ("echo '{\"ok\": \"yes\"}'", ErrorCode::InvalidOutput),
        (
            "echo '{\"ok\": false, \"error\": {\"code\": \"timeout\", \"message\": \"x\"}}'; exit 1",
            ErrorCode::InvalidOutput,
        ),
        (
            "echo '{\"ok\": false, \"error\": {\"code\": \"dependency-missing\", \"message\": \"x\"}}'; exit 1",
            ErrorCode::InvalidOutput,
        ),
    ];
    for (script, expected) in cases {
        let f = install(script);
        assert_eq!(
            code(f.fetch(None, &CallOptions::default()).await),
            expected,
            "{script}"
        );
    }
}

#[tokio::test]
async fn plugin_failures_come_through() {
    let f = install(
        "echo 'log line' >&2\necho '{\"ok\": false, \"error\": {\"code\": \"not-found\", \"message\": \"gone\", \"reason\": \"deleted\"}}'\nexit 1",
    );
    match f.fetch(None, &CallOptions::default()).await {
        Err(PluginError::Call {
            failure, stderr, ..
        }) => {
            assert_eq!(failure.code, ErrorCode::NotFound);
            assert_eq!(failure.reason, Some(NotFoundReason::Deleted));
            assert!(failure.code.tries_next());
            assert_eq!(stderr, ["log line"]);
        }
        other => panic!("{other:?}"),
    }
    // 信封为准：退出码 0 也照样是失败
    let f = install(
        "echo '{\"ok\": false, \"error\": {\"code\": \"rate-limited\", \"message\": \"429\", \"retryAfter\": 30}}'",
    );
    assert_eq!(
        code(f.fetch(None, &CallOptions::default()).await),
        ErrorCode::RateLimited
    );
}

fn output_with(html: &str) -> String {
    format!(
        r#"cat <<'JSON'
{{"ok": true, "kind": "post", "text": "x", "html": "{html}", "images": [], "media": [], "platformTags": [],
 "completeness": "full", "missing": [], "evidence": {{"fetchedAt": "2026-11-01T03:00:01Z", "access": "public-web"}}}}
JSON"#
    )
}

#[tokio::test]
async fn output_directory_rules() {
    let cases = [
        ("", output_with("../page.html")),
        ("", output_with("/etc/passwd")),
        ("", output_with("page.html")),
        (
            "printf x > \"$out/page.html\"; ln -s /etc/passwd \"$out/secret\"",
            output_with("page.html"),
        ),
        (
            "printf x > \"$out/page.html\"; ln \"$out/page.html\" \"$out/again.html\"",
            output_with("page.html"),
        ),
        (
            "printf x > \"$out/page.html\"; printf x > \"$out/usuyo.fetch.json\"",
            output_with("page.html"),
        ),
        (
            "printf x > \"$out/page.html\"; printf x > \"$out/.hidden\"",
            output_with("page.html"),
        ),
        (
            "printf x > \"$out/page.html\"; mkfifo \"$out/pipe\"",
            output_with("page.html"),
        ),
        (
            "printf x > \"$out/page.html\"; head -c 2000000 /dev/zero > \"$out/big.bin\"",
            output_with("page.html"),
        ),
        (
            "printf x > \"$out/page.html\"; mv \"$out\" \"$out.moved\"; ln -s / \"$out\"",
            output_with("page.html"),
        ),
    ];
    for (setup, print) in cases {
        let f = install(&format!("{setup}\n{print}"));
        assert_eq!(
            code(f.fetch(None, &CallOptions::default()).await),
            ErrorCode::InvalidOutput,
            "{setup} / {print}"
        );
    }
    let f = install(&format!(
        "printf x > \"$out/page.html\"\nmkdir \"$out/raw\"\n{}",
        output_with("page.html")
    ));
    f.fetch(None, &CallOptions::default()).await.unwrap();
}

#[tokio::test]
async fn oversized_stdout_is_cut_off() {
    let f = install("head -c 200000 /dev/zero | tr '\\0' a; sleep 30");
    let started = Instant::now();
    assert_eq!(
        code(f.fetch(None, &CallOptions::default()).await),
        ErrorCode::InvalidOutput
    );
    assert!(started.elapsed() < Duration::from_secs(20));
}

#[tokio::test]
async fn a_part_the_item_never_had_is_not_missing() {
    // 只有字的帖子：插件说 full、没交图和网页原文。那一条本来就没有，不算缺（协议 §3.7）
    let f = install(
        r#"cat <<'JSON'
{"ok": true, "kind": "post", "text": "x", "images": [], "media": [], "platformTags": [],
 "completeness": "full", "missing": [], "evidence": {"fetchedAt": "2026-11-01T03:00:01Z", "access": "public-web"}}
JSON"#,
    );
    let snap = f.fetch(None, &CallOptions::default()).await.unwrap();
    assert_eq!(snap.output.completeness, Completeness::Full);
    assert!(snap.output.missing.is_empty());
}

#[tokio::test]
async fn missing_is_what_the_plugin_says_among_what_was_asked_for() {
    // 插件说缺图片、网页原文、评论：评论不在这次想要的里头，去掉；图片交来了一部分，照样留着
    let f = install(
        r#"mkdir -p "$out/images" && printf 'png' > "$out/images/001.png"
cat <<'JSON'
{"ok": true, "kind": "post", "text": "x", "images": ["images/001.png"], "media": [], "platformTags": [],
 "completeness": "partial", "missing": ["images", "html", "comments", "html"],
 "evidence": {"fetchedAt": "2026-11-01T03:00:01Z", "access": "public-web"}}
JSON"#,
    );
    let snap = f
        .fetch(
            Some(&[OutputKind::Text, OutputKind::Images, OutputKind::Html]),
            &CallOptions::default(),
        )
        .await
        .unwrap();
    assert_eq!(snap.output.completeness, Completeness::Partial);
    let missing: std::collections::BTreeSet<_> = snap.output.missing.iter().copied().collect();
    assert_eq!(
        missing,
        std::collections::BTreeSet::from([OutputKind::Images, OutputKind::Html])
    );
    assert_eq!(snap.output.missing.len(), 2);
}

#[tokio::test]
async fn stdout_just_over_the_cap_then_exit_is_invalid_output() {
    let f = install("head -c 65600 /dev/zero | tr '\\0' a\nexit 0");
    assert_eq!(
        code(f.fetch(None, &CallOptions::default()).await),
        ErrorCode::InvalidOutput
    );
}

#[tokio::test]
async fn nothing_wanted_arrived_is_unsupported_input() {
    let f = install(
        r#"cat <<'JSON'
{"ok": true, "kind": "post", "images": [], "media": [], "platformTags": [], "completeness": "full",
 "missing": [], "evidence": {"fetchedAt": "2026-11-01T03:00:01Z", "access": "public-web"}}
JSON"#,
    );
    assert_eq!(
        code(f.fetch(None, &CallOptions::default()).await),
        ErrorCode::UnsupportedInput
    );
}

#[tokio::test]
async fn progress_then_silence_times_out() {
    let f = install_with(
        "echo '@usuyo {\"event\":\"progress\",\"done\":1,\"total\":2}' >&2\nsleep 30",
        |l| {
            l.fetch = Duration::from_secs(60);
            l.idle = Duration::from_secs(1);
        },
    );
    let (tx, mut rx) = mpsc::unbounded_channel();
    let opts = CallOptions {
        events: Some(tx),
        ..CallOptions::default()
    };
    let started = Instant::now();
    assert_eq!(code(f.fetch(None, &opts).await), ErrorCode::Timeout);
    assert!(started.elapsed() < Duration::from_secs(20));
    assert!(matches!(
        rx.recv().await,
        Some(PluginEvent::Progress { done: Some(_), .. })
    ));
}

#[tokio::test]
async fn waiting_pauses_the_clock() {
    let f = install_with(
        &format!(
            "echo '@usuyo {{\"event\":\"waiting\",\"reason\":\"user-action\"}}' >&2\nsleep 8\necho '@usuyo {{\"event\":\"progress\"}}' >&2\n{OK_FETCH}"
        ),
        |l| {
            l.fetch = Duration::from_secs(6);
            l.waiting = Duration::from_secs(30);
        },
    );
    f.fetch(None, &CallOptions::default()).await.unwrap();
}

#[tokio::test]
async fn cancel_stops_the_call() {
    let f = install("sleep 30");
    let token = CancellationToken::new();
    let opts = CallOptions {
        cancel: Some(token.clone()),
        ..CallOptions::default()
    };
    tokio::spawn(async move {
        tokio::time::sleep(Duration::from_millis(200)).await;
        token.cancel();
    });
    let result = f.fetch(None, &opts).await;
    assert_eq!(code(result), ErrorCode::Cancelled);
    assert!(!ErrorCode::Cancelled.tries_next());
}

#[tokio::test]
async fn canonicalize_checks_the_platform() {
    let url = url::Url::parse("https://example.com/a").unwrap();
    let f = install(
        r#"echo '{"ok": true, "match": true, "identity": {"platform": "example", "contentId": "42"}}'"#,
    );
    let got = f
        .host
        .canonicalize(f.plugin(), &url, &CallOptions::default())
        .await
        .unwrap();
    assert!(matches!(
        got,
        Canonicalized::Match {
            identity: Identity::Platform { .. },
            ..
        }
    ));

    let f = install(r#"echo '{"ok": true, "match": false}'"#);
    let got = f
        .host
        .canonicalize(f.plugin(), &url, &CallOptions::default())
        .await
        .unwrap();
    assert_eq!(got, Canonicalized::NoMatch);

    let f = install(
        r#"echo '{"ok": true, "match": true, "identity": {"platform": "douyin", "contentId": "42"}}'"#,
    );
    assert_eq!(
        code(
            f.host
                .canonicalize(f.plugin(), &url, &CallOptions::default())
                .await
        ),
        ErrorCode::InvalidOutput
    );
}

#[tokio::test]
async fn list_pages() {
    let f = install(
        r#"printf '%s\n' "$@" > "$USUYO_CACHE_DIR/args"
cat <<'JSON'
{"ok": true, "items": [{"url": "https://example.com/1", "actedAt": "2026-10-31T22:14:00+08:00"}, {"ref": "r/2"}],
 "nextCursor": "c2", "evidence": {"fetchedAt": "2026-11-01T03:00:01Z", "access": "public-web"}}
JSON"#,
    );
    let request = ListRequest {
        source: "example:favorites".into(),
        cursor: Some("c1".into()),
        since: Some("2026-10-31T00:00:00Z".parse().unwrap()),
        input_file: None,
    };
    let page = f
        .host
        .list(f.plugin(), request, &CallOptions::default())
        .await
        .unwrap();
    assert_eq!(page.items.len(), 2);
    assert_eq!(page.next_cursor.as_deref(), Some("c2"));
    let args = std::fs::read_to_string(f.cache().join("args")).unwrap();
    assert_eq!(
        args.lines().collect::<Vec<_>>(),
        [
            "list",
            "example:favorites",
            "--cursor",
            "c1",
            "--since",
            "2026-10-31T00:00:00Z"
        ]
    );

    let f = install(
        r#"echo '{"ok": true, "items": [{"title": "no link"}], "nextCursor": null, "evidence": {"fetchedAt": "2026-11-01T03:00:01Z", "access": "public-web"}}'"#,
    );
    let request = ListRequest {
        source: "example:favorites".into(),
        cursor: None,
        since: None,
        input_file: None,
    };
    assert_eq!(
        code(
            f.host
                .list(f.plugin(), request, &CallOptions::default())
                .await
        ),
        ErrorCode::InvalidOutput
    );
}

#[test]
fn discovery_picks_the_newest_and_explains_rejections() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    let put = |dir: &str, yaml: &str| {
        let d = root.join(dir);
        std::fs::create_dir_all(&d).unwrap();
        std::fs::write(d.join("usuyo-plugin.yaml"), yaml).unwrap();
    };
    put("fake/plugin/1.0.0", MANIFEST);
    put(
        "fake/plugin/1.2.0",
        &MANIFEST.replace("version: 1.0.0", "version: 1.2.0"),
    );
    put("fake/other/1.0.0", MANIFEST);
    put("fake/broken/1.0.0", "protocol: 1\n");
    put("fake/plugin/latest", MANIFEST);
    let found = discover(root).unwrap();
    assert_eq!(found.fetch.len(), 1);
    assert_eq!(found.fetch[0].header.version.to_string(), "1.2.0");
    let mut reasons: Vec<String> = found.rejected.iter().map(|r| r.error.to_string()).collect();
    reasons.sort();
    assert_eq!(reasons.len(), 3, "{reasons:?}");
    assert!(
        reasons
            .iter()
            .any(|r| r.contains("does not match its install directory"))
    );
    assert!(reasons.iter().any(|r| r.contains("is not a version")));
}

#[test]
fn candidates_put_web_last() {
    let tmp = tempfile::tempdir().unwrap();
    let put = |dir: &str, yaml: &str| {
        let d = tmp.path().join(dir);
        std::fs::create_dir_all(&d).unwrap();
        std::fs::write(d.join("usuyo-plugin.yaml"), yaml).unwrap();
    };
    let web = std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../plugins/web/usuyo-plugin.yaml"),
    )
    .unwrap();
    put("builtin/web/0.0.0", &web);
    put("fake/plugin/1.0.0", MANIFEST);
    let found = discover(tmp.path()).unwrap();
    // discover 按目录名排：builtin 在前
    assert_eq!(found.fetch[0].id().as_str(), "builtin/web");
    let host = PluginHost::new(
        HostConfig {
            work_dir: tmp.path().join("work"),
            cache_dir: tmp.path().join("cache"),
            locale: "zh-CN".into(),
            limits: CallLimits::default(),
        },
        found.fetch,
    );
    let ids = |u: &str| -> Vec<String> {
        host.candidates(&url::Url::parse(u).unwrap())
            .iter()
            .map(|p| p.id().to_string())
            .collect()
    };
    assert_eq!(ids("https://example.com/x"), ["fake/plugin", "builtin/web"]);
    assert_eq!(ids("https://other.org/x"), ["builtin/web"]);
}

#[tokio::test]
async fn retry_after_has_a_ceiling() {
    let f = install(
        r#"cat <<'JSON'
{"ok": false, "error": {"code": "rate-limited", "message": "429", "retryAfter": 18446744073709551615}}
JSON
exit 1"#,
    );
    match f.fetch(None, &CallOptions::default()).await {
        Err(PluginError::Call { failure, .. }) => {
            let pause = failure.pause();
            assert!(
                matches!(pause, Some(Pause::For(d)) if d <= Duration::from_secs(3600)),
                "{pause:?}"
            );
        }
        other => panic!("{other:?}"),
    }
}

#[tokio::test]
async fn one_file_in_two_roles_is_refused() {
    for roles in [
        r#""html": "comments.json", "images": [], "media": []"#,
        r#""images": ["comments.json"], "media": []"#,
        r#""images": [], "media": [{"path": "comments.json", "kind": "other"}]"#,
    ] {
        let f = install(&format!(
            r#"printf '[]' > "$out/comments.json"
cat <<'JSON'
{{"ok": true, "kind": "post", "text": "x", {roles}, "comments": "comments.json",
 "platformTags": [], "completeness": "full", "missing": [],
 "evidence": {{"fetchedAt": "2026-11-01T03:00:01Z", "access": "public-web"}}}}
JSON"#
        ));
        assert_eq!(
            code(f.fetch(None, &CallOptions::default()).await),
            ErrorCode::InvalidOutput,
            "{roles}"
        );
    }
}

#[tokio::test]
async fn waiting_has_one_budget_per_call() {
    let f = install_with(
        r#"while true; do
  echo '@usuyo {"event":"waiting","reason":"user-action"}' >&2
  sleep 1
  echo '@usuyo {"event":"progress"}' >&2
done"#,
        |l| {
            l.fetch = Duration::from_secs(60);
            l.waiting = Duration::from_secs(3);
        },
    );
    // 每次 waiting 都不到 3 秒，加起来过了 3 秒就算超时
    let started = Instant::now();
    let result = tokio::time::timeout(
        Duration::from_secs(20),
        f.fetch(None, &CallOptions::default()),
    )
    .await
    .expect("waiting and progress in turns kept the call alive");
    assert_eq!(code(result), ErrorCode::Timeout);
    assert!(started.elapsed() < Duration::from_secs(10));
}

#[tokio::test]
async fn envelope_strings_are_bounded() {
    let long = "标".repeat(5000);
    let f = install_with(
        &format!(
            r#"printf x > "$out/page.html"
cat <<'JSON'
{{"ok": true, "kind": "post", "title": "{long}\nsecond", "author": {{"name": "{long}"}},
 "text": "a\u0007b\nc\u001b[31m", "html": "page.html", "images": [], "media": [],
 "platformTags": ["{long}", "ok"], "missingReason": "{long}",
 "completeness": "full", "missing": [],
 "evidence": {{"fetchedAt": "2026-11-01T03:00:01Z", "access": "public-web", "site": "{long}"}}}}
JSON"#
        ),
        |l| l.stdout_bytes = 1 << 20,
    );
    let snap = f.fetch(None, &CallOptions::default()).await.unwrap();
    let o = &snap.output;
    let title = o.title.as_deref().unwrap();
    assert!(
        title.len() <= 1024 && !title.contains('\n'),
        "{}",
        title.len()
    );
    assert!(o.author.as_ref().unwrap().name.len() <= 1024);
    assert!(o.missing_reason.as_deref().unwrap().len() <= 1024);
    assert!(o.evidence.site.as_deref().unwrap().len() <= 1024);
    assert!(o.platform_tags.iter().all(|t| t.len() <= 100));
    assert_eq!(o.platform_tags[1], "ok");
    assert_eq!(
        o.text.as_deref(),
        Some("a\u{7}b\nc\u{1b}[31m"),
        "正文照原样"
    );

    let f = install(&format!(
        r#"cat <<'JSON'
{{"ok": false, "error": {{"code": "network", "message": "one\nforged line\r", "hint": "{long}"}}}}
JSON
exit 1"#
    ));
    match f.fetch(None, &CallOptions::default()).await {
        Err(PluginError::Call { failure, .. }) => {
            assert!(
                !failure.message.contains(['\n', '\r']),
                "{}",
                failure.message
            );
            assert!(failure.hint.unwrap().len() <= 1024);
        }
        other => panic!("{other:?}"),
    }
}

#[tokio::test]
async fn too_many_output_files_are_refused() {
    let f = install(&format!(
        "printf x > \"$out/page.html\"\nmkdir \"$out/many\" && (cd \"$out/many\" && seq 1 10001 | xargs touch)\n{}",
        output_with("page.html")
    ));
    assert_eq!(
        code(f.fetch(None, &CallOptions::default()).await),
        ErrorCode::InvalidOutput
    );
}

#[tokio::test]
async fn over_long_identities_are_refused() {
    let long = "x".repeat(5000);
    for identity in [
        format!(r#"{{"platform": "example", "contentId": "{long}"}}"#),
        format!(
            r#"{{"platform": "example", "contentId": "1", "url": "https://example.com/{long}"}}"#
        ),
        r#"{"platform": "example", "contentId": "1\n2"}"#.to_owned(),
    ] {
        let f = install_with(
            &format!(
                r#"printf x > "$out/page.html"
cat <<'JSON'
{{"ok": true, "identity": {identity}, "kind": "post", "text": "x", "html": "page.html", "images": [],
 "media": [], "platformTags": [], "completeness": "full", "missing": [],
 "evidence": {{"fetchedAt": "2026-11-01T03:00:01Z", "access": "public-web"}}}}
JSON"#
            ),
            |l| l.stdout_bytes = 1 << 20,
        );
        assert_eq!(
            code(f.fetch(None, &CallOptions::default()).await),
            ErrorCode::InvalidOutput,
            "{}",
            &identity[..60.min(identity.len())]
        );
    }
}

/// 要联网的插件拿到宿主定好的代理（`USUYO_PROXY`），不联网的拿不到；`HTTP_PROXY` 一类不给。
#[tokio::test]
async fn networked_plugins_get_the_proxy_and_others_do_not() {
    let tmp = tempfile::tempdir().unwrap();
    for (id, network) in [
        (
            "fake/online",
            "\n  network: {mode: allowlist, hosts: [\"example.com\"]}",
        ),
        ("fake/offline", ""),
    ] {
        let dir = tmp.path().join("plugins").join(id).join("1.0.0");
        std::fs::create_dir_all(&dir).unwrap();
        let manifest = MANIFEST
            .replace("id: fake/plugin", &format!("id: {id}"))
            .replace(
                "  fs: {write: [\"${OUT}\"]}",
                &format!("  fs: {{write: [\"${{OUT}}\"]}}{network}"),
            );
        std::fs::write(dir.join("usuyo-plugin.yaml"), manifest).unwrap();
        let path = dir.join("plugin.sh");
        std::fs::write(
            &path,
            "#!/bin/sh\nenv > \"$USUYO_CACHE_DIR/env\"\necho '{\"ok\": true, \"match\": false}'\n",
        )
        .unwrap();
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).unwrap();
    }
    let found = discover(&tmp.path().join("plugins")).unwrap();
    assert!(found.rejected.is_empty(), "{:?}", found.rejected);
    let host = PluginHost::new(
        HostConfig {
            work_dir: tmp.path().join("work"),
            cache_dir: tmp.path().join("cache"),
            locale: "zh-CN".into(),
            limits: CallLimits::default(),
        },
        found.fetch,
    )
    .with_proxy(usuyo_plugin::ProxySetting::parse("socks5://127.0.0.1:7979").unwrap());
    let link = url::Url::parse("https://example.com/post/1").unwrap();
    for p in host.plugins() {
        host.canonicalize(p, &link, &CallOptions::default())
            .await
            .unwrap();
    }
    let env =
        |id: &str| std::fs::read_to_string(tmp.path().join("cache").join(id).join("env")).unwrap();
    let online = env("fake/online");
    assert!(
        online
            .lines()
            .any(|l| l == "USUYO_PROXY=socks5://127.0.0.1:7979"),
        "{online}"
    );
    assert!(!env("fake/offline").contains("USUYO_PROXY"));
    for e in [online, env("fake/offline")] {
        assert!(
            !e.lines()
                .any(|l| l.to_ascii_lowercase().starts_with("http_proxy=")
                    || l.to_ascii_lowercase().starts_with("https_proxy=")),
            "{e}"
        );
    }
}
