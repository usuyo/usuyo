//! 一个插件里的几种方法（协议 §2.3.1）：挑哪一种、换下一种、补缺、开关和默认、这台机器上用不用得上、
//! 凭据只给列了站点的方法、失败日志、从链接开始的来源。引擎对着假插件（sh 脚本）跑，不碰网络和钥匙串。
#![cfg(unix)]

use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::time::Duration;

use tempfile::TempDir;
use usuyo_core::view::{Facet, Stored};
use usuyo_core::{Engine, EngineConfig, Links, RetryPolicy, Via};

/// 每种方法照缓存目录里 `<方法>.sh` 回答（没有就回 `not-found`）。每次调用记一行：方法、`--want`、
/// 交没交凭据和访客身份。参数：`fetch <链接> --out <目录> --method <方法> [--want <几样>]`。
const SCRIPT: &str = r#"#!/bin/sh
c="$USUYO_CACHE_DIR"
case "$1" in
  canonicalize)
    id=${2##*/}
    printf '{"ok": true, "match": true, "identity": {"platform": "example", "contentId": "%s", "url": "https://example.test/%s"}}' "$id" "$id"
    ;;
  health)
    if [ -f "$c/health.json" ]; then cat "$c/health.json"; else echo '{"ok": true, "ready": true, "protocol": 1, "version": "1.0.0"}'; fi
    ;;
  sources)
    echo "sources $3 $5" >> "$c/calls"
    printf '{"ok": true, "sources": [{"source": "example:fav:7", "title": "从 %s", "action": "favorite"}]}' "$5"
    ;;
  list)
    echo "list $4" >> "$c/calls"
    echo '{"ok": true, "items": [{"url": "https://example.test/9", "identity": {"platform": "example", "contentId": "9"}, "title": "t"}], "nextCursor": "2", "evidence": {"fetchedAt": "2026-11-01T03:00:01Z", "access": "public-web"}}'
    ;;
  fetch)
    out="$4"
    echo "$6 ${8:-all} ${USUYO_CREDENTIALS:+cred} ${USUYO_VISITOR:+visitor}" >> "$c/calls"
    if [ -f "$c/$6.sh" ]; then . "$c/$6.sh"; else
      printf '%s\n' '{"ok": false, "error": {"code": "not-found", "message": "gone", "detail": "<html>\n404"}}'; exit 1
    fi
    ;;
esac
"#;

const MANIFEST: &str = r#"protocol: 1
id: fake/multi
kind: fetch
name: 例子
version: 1.0.0
license: MIT
maintainers: [someone]
platforms: [example]
match: ["example.test/*"]
sites:
  - {site: www.example.test, name: 例子, credential: cookies, loginUrl: "https://www.example.test/", cookies: [sid]}
visitors:
  - {site: www.example.test, name: 例子, url: "https://www.example.test/"}
methods:
  - id: session
    name: 登录取视频
    access: session-api
    login: required
    account: in-usuyo
    sites: [www.example.test]
    capabilities:
      fetch: {outputs: [media:video]}
  - id: hooked
    name: 改客户端
    access: client-hook
    login: none
    account: none
    capabilities:
      fetch: {outputs: [text, images, media:video, meta]}
  - id: visitor
    name: 访客页
    access: public-web
    login: none
    account: visitor
    visitors: [www.example.test]
    capabilities:
      fetch: {outputs: [text, meta]}
  - id: guest
    name: 公开接口
    access: public-web
    login: none
    account: none
    capabilities:
      fetch: {outputs: [text, images, meta]}
  - id: tool
    name: 包的工具
    access: public-web
    login: none
    account: none
    capabilities:
      fetch: {outputs: [text]}
  - id: saved
    name: 公开收藏夹
    access: public-web
    login: none
    account: none
    capabilities:
      list:
        order: newest-first
        link: 收藏夹的链接
permissions:
  fs: {write: ["${OUT}"]}
run:
  command: ["${PLUGIN_DIR}/plugin.sh"]
"#;

/// 一份正文（和图），不带视频；`missing` 由调用方给，有缺的才带原因。
fn text_answer(missing: &str) -> String {
    let reason = if missing.is_empty() {
        ""
    } else {
        r#""missingReason": "视频要登录才取得到","#
    };
    format!(
        r#"mkdir -p "$out/images" && printf 'png' > "$out/images/1.png"
cat <<JSON
{{"ok": true, "identity": {{"platform": "example", "contentId": "1"}}, "kind": "video", "title": "三段式注水",
 "text": "第一段闷蒸", "images": ["images/1.png"], "media": [], "platformTags": [],
 "completeness": "partial", "missing": [{missing}], {reason}
 "evidence": {{"fetchedAt": "2026-11-01T03:00:01Z", "access": "public-web"}}}}
JSON"#
    )
}

const VIDEO_ANSWER: &str = r#"mkdir -p "$out/media" && printf 'mp4' > "$out/media/v.mp4"
cat <<JSON
{"ok": true, "identity": {"platform": "example", "contentId": "1"}, "kind": "video",
 "images": [], "media": [{"path": "media/v.mp4", "kind": "video", "mime": "video/mp4"}], "platformTags": [],
 "completeness": "full", "missing": [],
 "evidence": {"fetchedAt": "2026-11-01T03:00:02Z", "access": "session-api", "site": "www.example.test"}}
JSON"#;

const RATE_LIMITED: &str = r#"echo '{"ok": false, "error": {"code": "rate-limited", "message": "429", "retryAfter": 600}}'; exit 1"#;

struct Fixture {
    tmp: TempDir,
    library: PathBuf,
}

impl Fixture {
    fn new() -> Fixture {
        let tmp = tempfile::tempdir().unwrap();
        let machine = tmp.path().join("machine");
        let library = Engine::init_library(&tmp.path().join("lib"), &machine).unwrap();
        let dir = machine.join("plugins/fake/multi/1.0.0");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("usuyo-plugin.yaml"), MANIFEST).unwrap();
        std::fs::write(dir.join("plugin.sh"), SCRIPT).unwrap();
        std::fs::set_permissions(
            dir.join("plugin.sh"),
            std::fs::Permissions::from_mode(0o755),
        )
        .unwrap();
        Fixture { tmp, library }
    }

    fn machine(&self) -> PathBuf {
        self.tmp.path().join("machine")
    }

    fn engine(&self) -> Engine {
        self.engine_retrying_after(Duration::ZERO)
    }

    fn engine_retrying_after(&self, base: Duration) -> Engine {
        let mut config = EngineConfig::new(&self.library, self.machine());
        config.retry = RetryPolicy {
            base,
            max_attempts: 4,
        };
        Engine::open(config).unwrap()
    }

    fn cache(&self) -> PathBuf {
        self.machine().join("cache/plugins/fake/multi")
    }

    fn answer(&self, method: &str, body: &str) {
        std::fs::create_dir_all(self.cache()).unwrap();
        std::fs::write(self.cache().join(format!("{method}.sh")), body).unwrap();
    }

    fn calls(&self) -> Vec<String> {
        std::fs::read_to_string(self.cache().join("calls"))
            .unwrap_or_default()
            .lines()
            .map(|l| l.trim_end().to_owned())
            .collect()
    }

    fn sign_in(&self, engine: &Engine) {
        let state = serde_json::from_value(serde_json::json!({"cookies": [
            {"name": "sid", "value": "s3cret-sid", "domain": ".example.test"}]}))
        .unwrap();
        engine.save_site("www.example.test", &state).unwrap();
        let state = serde_json::from_value(serde_json::json!({"cookies": [
            {"name": "vid", "value": "visitor-vid", "domain": ".example.test"}]}))
        .unwrap();
        engine.save_visitor("www.example.test", &state).unwrap();
    }

    fn failure_log(&self) -> Vec<serde_json::Value> {
        std::fs::read_to_string(self.machine().join("logs/fetch-failures.jsonl"))
            .unwrap_or_default()
            .lines()
            .map(|l| serde_json::from_str(l).unwrap())
            .collect()
    }
}

#[tokio::test]
async fn methods_go_lowest_tier_first_and_account_free_first_and_fall_through() {
    let f = Fixture::new();
    // 包的工具这台机器上缺东西：插件的本机检查只挡它
    std::fs::create_dir_all(f.cache()).unwrap();
    std::fs::write(
        f.cache().join("health.json"),
        r#"{"ok": true, "ready": true, "protocol": 1, "version": "1.0.0", "checks": [{"name": "tool", "ok": false, "message": "没找到它", "methods": ["tool"]}]}"#,
    )
    .unwrap();
    f.answer("session", VIDEO_ANSWER);
    let engine = f.engine();
    f.sign_in(&engine);
    let t = engine
        .take("https://example.test/1?share=abc", "link", Via::Cli)
        .await
        .unwrap();
    assert_eq!(engine.run_due().await.unwrap(), 1);
    // 公开接口 → 访客页 → 登录的；工具被挡，改客户端的默认关着。凭据只给列了站点的那种，访客身份只给列了它的
    assert_eq!(
        f.calls(),
        ["guest all", "visitor all  visitor", "session all cred"]
    );
    let v = engine.item(&t.id, &Links::default()).unwrap();
    assert_eq!(v.stored, Stored::Complete);

    // 每次失败一行，链接只记主机，带平台原样的回答
    let log = f.failure_log();
    assert_eq!(log.len(), 2);
    assert_eq!(
        (
            log[0]["method"].as_str(),
            log[0]["code"].as_str(),
            log[0]["host"].as_str()
        ),
        (Some("guest"), Some("not-found"), Some("example.test"))
    );
    assert_eq!(log[1]["method"], "visitor");
    assert_eq!(log[0]["detail"], "<html>\n404");
    let text = std::fs::read_to_string(f.machine().join("logs/fetch-failures.jsonl")).unwrap();
    assert!(
        !text.contains("share=abc") && !text.contains("s3cret"),
        "{text}"
    );

    // 插件页上每种方法上一次的结果
    let p = &engine.plugins().await.unwrap().plugins[0];
    let m = |id: &str| p.methods.iter().find(|m| m.id == id).unwrap();
    assert!(m("session").last_call.as_ref().unwrap().ok);
    let last = m("guest").last_call.clone().unwrap();
    assert_eq!((last.ok, last.code.as_deref()), (false, Some("not-found")));
    assert!(m("tool").last_call.is_none());
    assert_eq!(m("tool").blockers[0].code, "check-failed");
    assert!(!m("hooked").enabled && !m("hooked").usable);
}

#[tokio::test]
async fn a_rate_limit_waits_and_the_next_round_skips_the_paused_method() {
    let f = Fixture::new();
    f.answer("guest", RATE_LIMITED);
    f.answer("tool", RATE_LIMITED);
    f.answer("visitor", &text_answer(""));
    let engine = f.engine_retrying_after(Duration::from_millis(300));
    f.sign_in(&engine);
    let t = engine
        .take("https://example.test/1", "link", Via::Cli)
        .await
        .unwrap();
    engine.run_due().await.unwrap();
    // 限流不换：这一轮只调了它
    assert_eq!(f.calls(), ["guest all"]);
    assert_eq!(
        engine.item(&t.id, &Links::default()).unwrap().stored,
        Stored::Link
    );
    let p = &engine.plugins().await.unwrap().plugins[0];
    let guest = p.methods.iter().find(|m| m.id == "guest").unwrap();
    assert_eq!(guest.pauses[0].code.as_deref(), Some("rate-limited"));
    assert!(
        p.methods
            .iter()
            .find(|m| m.id == "visitor")
            .unwrap()
            .pauses
            .is_empty()
    );
    // 下一轮（退避以后，不等它的暂停到点）：停着的那种跳过，下一种接着；它也限流，又停一轮
    tokio::time::sleep(Duration::from_millis(400)).await;
    engine.run_due().await.unwrap();
    assert_eq!(f.calls(), ["guest all", "tool all"]);
    tokio::time::sleep(Duration::from_millis(1300)).await;
    engine.run_due().await.unwrap();
    assert_eq!(f.calls(), ["guest all", "tool all", "visitor all  visitor"]);
    assert_eq!(
        engine.item(&t.id, &Links::default()).unwrap().stored,
        Stored::Complete
    );
}

#[tokio::test]
async fn what_the_first_method_misses_is_asked_of_the_next_that_declares_it() {
    let f = Fixture::new();
    f.answer("guest", &text_answer(r#""media:video""#));
    f.answer("session", VIDEO_ANSWER);
    let engine = f.engine();
    f.sign_in(&engine);
    let t = engine
        .take("https://example.test/1", "link", Via::Cli)
        .await
        .unwrap();
    engine.run_due().await.unwrap();
    // 访客页没声明视频，跳过；登录的那种只要视频
    assert_eq!(f.calls(), ["guest all", "session media:video cred"]);
    let v = engine.item(&t.id, &Links::default()).unwrap();
    assert_eq!(v.stored, Stored::Complete);
    // 两份快照：原文和图来自第一份，画面来自第二份
    let manifest: serde_json::Value = serde_json::from_slice(
        &std::fs::read(
            f.library
                .join("items/example")
                .join(&t.id)
                .join("manifest.json"),
        )
        .unwrap(),
    )
    .unwrap();
    let snapshots = manifest["snapshots"].as_array().unwrap();
    assert_eq!(snapshots.len(), 2);
    assert_eq!(snapshots[0]["plugin"], "fake/multi");
    let faces: Vec<&str> = v
        .facets
        .iter()
        .map(|x| match x {
            Facet::Video { .. } => "video",
            Facet::Image { .. } => "image",
            Facet::Text { .. } => "text",
            _ => "other",
        })
        .collect();
    assert!(
        faces.contains(&"video") && faces.contains(&"image") && faces.contains(&"text"),
        "{faces:?}"
    );
    assert!(
        manifest["missing"].as_array().unwrap().is_empty(),
        "{manifest}"
    );
    // 画面来自有画面的那一份（第二次调用），不是主快照
    let with_video = snapshots
        .iter()
        .find(|x| {
            x["outputs"]
                .as_array()
                .unwrap()
                .iter()
                .any(|o| o == "media:video")
        })
        .unwrap()["id"]
        .as_str()
        .unwrap()
        .to_owned();
    assert_ne!(
        manifest["primarySnapshot"].as_str(),
        Some(with_video.as_str())
    );
    let video = v
        .facets
        .iter()
        .find_map(|x| match x {
            Facet::Video { snapshot_id, .. } => Some(snapshot_id.clone()),
            _ => None,
        })
        .unwrap();
    assert_eq!(video, with_video);
}

#[tokio::test]
async fn switches_have_defaults_and_turning_one_on_makes_it_a_candidate() {
    let f = Fixture::new();
    f.answer("hooked", &text_answer(""));
    let engine = f.engine();
    // 没登录、没访客身份：只有不带账号的两种能试
    let t = engine
        .take("https://example.test/1", "link", Via::Cli)
        .await
        .unwrap();
    engine.run_due().await.unwrap();
    assert_eq!(f.calls(), ["guest all", "tool all"]);
    engine.set_method("fake/multi", "hooked", true).unwrap();
    engine.set_method("fake/multi", "tool", false).unwrap();
    engine.refetch(&t.id, &Links::default()).unwrap();
    engine.run_due().await.unwrap();
    assert_eq!(f.calls()[2..], ["guest all", "hooked all"]);
    assert_eq!(
        engine.item(&t.id, &Links::default()).unwrap().stored,
        Stored::Complete
    );
    // 写进按机器的 settings.json；没做的方法、没有的方法开不了
    let settings: serde_json::Value =
        serde_json::from_slice(&std::fs::read(f.machine().join("settings.json")).unwrap()).unwrap();
    assert_eq!(
        settings["plugins"]["fake/multi"]["methods"],
        serde_json::json!({"hooked": {"enabled": true}, "tool": {"enabled": false}})
    );
    assert!(engine.set_method("fake/multi", "nope", true).is_err());
    drop(engine);
    let engine = f.engine();
    let p = &engine.plugins().await.unwrap().plugins[0];
    assert!(p.methods.iter().find(|m| m.id == "hooked").unwrap().enabled);
    assert!(!p.methods.iter().find(|m| m.id == "tool").unwrap().enabled);
}

#[tokio::test]
async fn with_nothing_usable_the_item_says_what_to_do() {
    let f = Fixture::new();
    let engine = f.engine();
    engine.set_method("fake/multi", "guest", false).unwrap();
    engine.set_method("fake/multi", "tool", false).unwrap();
    let t = engine
        .take("https://example.test/1", "link", Via::Cli)
        .await
        .unwrap();
    engine.run_due().await.unwrap();
    assert!(f.calls().is_empty(), "{:?}", f.calls());
    let v = serde_json::to_value(engine.item(&t.id, &Links::default()).unwrap()).unwrap();
    // 你关掉的不拿来当原因：说的是你能去补的那一样
    let missing = v["missing"].as_str().unwrap();
    assert!(missing.contains("访客身份"), "{missing}");
}

#[tokio::test]
async fn a_list_can_start_from_a_link() {
    let f = Fixture::new();
    let engine = f.engine();
    let link = url::Url::parse("https://space.example.test/7/favlist").unwrap();
    let sources = engine
        .plugin_sources("fake/multi", "saved", Some(&link))
        .await
        .unwrap();
    assert_eq!(sources[0].method, "saved");
    assert_eq!(sources[0].source.title, format!("从 {link}"));
    assert_eq!(f.calls(), [format!("sources saved {link}")]);
    // 不能列的方法、没写 `link` 的不接
    assert!(
        engine
            .plugin_sources("fake/multi", "guest", None)
            .await
            .is_err()
    );
    let imp = engine
        .start_import("fake/multi", "saved", "example:fav:7", "")
        .unwrap();
    assert_eq!(serde_json::to_value(&imp).unwrap()["method"], "saved");
    assert!(
        engine
            .start_import("fake/multi", "guest", "example:fav:7", "")
            .is_err()
    );
}

#[tokio::test]
async fn switching_a_method_off_stops_its_import_before_the_next_page() {
    let f = Fixture::new();
    let engine = f.engine();
    let imp = engine
        .start_import("fake/multi", "saved", "example:fav:7", "")
        .unwrap();
    let id = serde_json::to_value(&imp).unwrap()["id"]
        .as_str()
        .unwrap()
        .to_owned();
    // 第一页列了，还有下一页在等
    engine.advance_imports().await.unwrap();
    assert_eq!(f.calls(), ["list saved"]);
    let view = serde_json::to_value(engine.import(&id).unwrap()).unwrap();
    assert_eq!(view["pages"], 1, "{view}");

    engine.set_method("fake/multi", "saved", false).unwrap();
    engine.advance_imports().await.unwrap();
    engine.advance_imports().await.unwrap();
    // 关了以后不再调插件，导入停下并写明原因（没有「到几点再试」）
    assert_eq!(f.calls(), ["list saved"]);
    let view = serde_json::to_value(engine.import(&id).unwrap()).unwrap();
    assert_eq!(view["waiting"]["code"], "unsupported-input", "{view}");
    assert!(view["waiting"]["until"].is_null(), "{view}");
    assert!(view["waiting"]["hint"].is_string(), "{view}");

    // 开回来、点「接着导入」，从停的地方继续
    engine.set_method("fake/multi", "saved", true).unwrap();
    engine.resume_import(&id).unwrap();
    engine.advance_imports().await.unwrap();
    assert_eq!(f.calls().len(), 2);
}
