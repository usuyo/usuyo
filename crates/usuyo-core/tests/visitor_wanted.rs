//! 自动取访客身份（协议 §2.5.1）：要访客身份而它没有、或站点不收存着的那份时，这条先等着、内核说要一份；
//! 交来了马上再试一次，没交来就到点照常再试，这时才说要访客身份。在站点窗口里读的方法：
//! 链接照插件的 `match` 落成导入，记下登录和上一回用它的情况，cookie 不经过内核。
//! 引擎对着假插件（sh 脚本）跑，不碰网络和钥匙串。
#![cfg(unix)]

use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::time::Duration;

use tempfile::TempDir;
use usuyo_core::view::Stored;
use usuyo_core::{DeliveredLink, Engine, EngineConfig, Links, RetryPolicy, Via, WindowSource};
use usuyo_plugin::LoginState;

/// 交来的访客身份里有 `stale` 时回 `visitor-required`（站点不收），别的时候取到。每次调用记一行。
const SCRIPT: &str = r#"#!/bin/sh
c="$USUYO_CACHE_DIR"
mkdir -p "$c"
case "$1" in
  canonicalize)
    id=${2##*/}; id=${id%%\?*}
    printf '{"ok": true, "match": true, "identity": {"platform": "example", "contentId": "%s", "url": "https://www.example.test/n/%s"}}' "$id" "$id"
    ;;
  health)
    echo '{"ok": true, "ready": true, "protocol": 1, "version": "1.0.0"}'
    ;;
  fetch)
    out="$4"
    echo "fetch ${USUYO_VISITOR:+visitor}" >> "$c/calls"
    if grep -q stale "$USUYO_VISITOR" 2>/dev/null; then
      echo '{"ok": false, "error": {"code": "visitor-required", "message": "sent home"}}'; exit 1
    fi
    cat <<JSON
{"ok": true, "identity": {"platform": "example", "contentId": "1"}, "kind": "article", "title": "t",
 "text": "正文", "images": [], "media": [], "platformTags": [], "completeness": "full", "missing": [],
 "evidence": {"fetchedAt": "2026-11-01T03:00:01Z", "access": "public-web"}}
JSON
    ;;
esac
"#;

const MANIFEST: &str = r#"protocol: 1
id: fake/visit
kind: fetch
name: 例子
version: 1.0.0
license: MIT
maintainers: [someone]
platforms: [example]
match: ["www.example.test/n/*"]
sites:
  - site: www.example.test
    name: 例子
    credential: browser-profile
    loginUrl: https://www.example.test/login
visitors:
  - {site: www.example.test, name: 例子, url: "https://www.example.test/"}
methods:
  - id: page
    name: 公开页
    access: public-web
    login: none
    account: visitor
    visitors: [www.example.test]
    capabilities:
      fetch: {outputs: [text, meta]}
  - id: in-usuyo
    name: 在 usuyo 中登录并读取
    access: browser-session
    login: required
    account: in-usuyo
    sites: [www.example.test]
    delivers: links
    script: window.js
    guide: 登录以后读你的收藏
permissions:
  fs: {write: ["${OUT}"]}
run:
  command: ["${PLUGIN_DIR}/plugin.sh"]
"#;

struct Fixture {
    tmp: TempDir,
    library: PathBuf,
}

impl Fixture {
    fn new() -> Fixture {
        let tmp = tempfile::tempdir().unwrap();
        let machine = tmp.path().join("machine");
        let library = Engine::init_library(&tmp.path().join("lib"), &machine).unwrap();
        let dir = machine.join("plugins/fake/visit/1.0.0");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("usuyo-plugin.yaml"), MANIFEST).unwrap();
        std::fs::write(dir.join("plugin.sh"), SCRIPT).unwrap();
        std::fs::write(dir.join("window.js"), "(function (usuyo) {})").unwrap();
        std::fs::set_permissions(
            dir.join("plugin.sh"),
            std::fs::Permissions::from_mode(0o755),
        )
        .unwrap();
        Fixture { tmp, library }
    }

    fn engine(&self, wait: Duration) -> Engine {
        let mut config = EngineConfig::new(&self.library, self.tmp.path().join("machine"));
        config.retry = RetryPolicy {
            base: Duration::ZERO,
            max_attempts: 4,
        };
        config.visitor_wait = wait;
        Engine::open(config).unwrap()
    }

    fn calls(&self) -> Vec<String> {
        std::fs::read_to_string(
            self.tmp
                .path()
                .join("machine/cache/plugins/fake/visit/calls"),
        )
        .unwrap_or_default()
        .lines()
        .map(|l| l.trim_end().to_owned())
        .collect()
    }
}

fn visitor(engine: &Engine, value: &str) {
    let state = serde_json::from_value(serde_json::json!({"cookies": [
        {"name": "vid", "value": value, "domain": ".example.test"}]}))
    .unwrap();
    engine.save_visitor("www.example.test", &state).unwrap();
}

fn wanted(engine: &Engine) -> Vec<(String, bool)> {
    engine
        .visitors_wanted()
        .into_iter()
        .map(|w| (w.site, w.replace))
        .collect()
}

#[tokio::test]
async fn a_missing_visitor_identity_is_asked_for_and_the_item_tried_once_it_arrives() {
    let f = Fixture::new();
    let engine = f.engine(Duration::from_secs(90));
    let t = engine
        .take("https://www.example.test/n/1", "link", Via::Cli)
        .await
        .unwrap();
    engine.run_due().await.unwrap();
    // 没调插件，等着；内核说要一份（不用换）
    assert!(f.calls().is_empty(), "{:?}", f.calls());
    assert_eq!(wanted(&engine), [("www.example.test".to_owned(), false)]);
    // 等着的这一条还不算没取到
    let v = serde_json::to_value(engine.item(&t.id, &Links::default()).unwrap()).unwrap();
    assert!(v.get("missing").is_none(), "{v}");
    // 壳交来一份：马上再试
    visitor(&engine, "good");
    assert!(wanted(&engine).is_empty());
    engine.run_due().await.unwrap();
    assert_eq!(f.calls(), ["fetch visitor"]);
    assert_eq!(
        engine.item(&t.id, &Links::default()).unwrap().stored,
        Stored::Complete
    );
}

#[tokio::test]
async fn a_refused_identity_is_replaced_and_the_item_tried_once_more() {
    let f = Fixture::new();
    let engine = f.engine(Duration::from_secs(90));
    visitor(&engine, "stale");
    let t = engine
        .take("https://www.example.test/n/1", "link", Via::Cli)
        .await
        .unwrap();
    engine.run_due().await.unwrap();
    assert_eq!(f.calls(), ["fetch visitor"]);
    assert_eq!(wanted(&engine), [("www.example.test".to_owned(), true)]);
    visitor(&engine, "good");
    engine.run_due().await.unwrap();
    assert_eq!(f.calls(), ["fetch visitor", "fetch visitor"]);
    assert_eq!(
        engine.item(&t.id, &Links::default()).unwrap().stored,
        Stored::Complete
    );
}

#[tokio::test]
async fn without_a_shell_the_item_waits_once_then_says_what_it_needs() {
    let f = Fixture::new();
    let engine = f.engine(Duration::from_millis(200));
    let t = engine
        .take("https://www.example.test/n/1", "link", Via::Cli)
        .await
        .unwrap();
    engine.run_due().await.unwrap();
    assert_eq!(wanted(&engine).len(), 1);
    tokio::time::sleep(Duration::from_millis(300)).await;
    engine.run_due().await.unwrap();
    assert!(f.calls().is_empty());
    let v = serde_json::to_value(engine.item(&t.id, &Links::default()).unwrap()).unwrap();
    assert!(
        v["missing"]
            .as_str()
            .unwrap()
            .contains("需要获取例子访客身份"),
        "{v}"
    );
    // 歇着：下一条不再等，也不再要
    assert!(wanted(&engine).is_empty());
    let t2 = engine
        .take("https://www.example.test/n/2", "link", Via::Cli)
        .await
        .unwrap();
    engine.run_due().await.unwrap();
    assert!(wanted(&engine).is_empty());
    let v = serde_json::to_value(engine.item(&t2.id, &Links::default()).unwrap()).unwrap();
    assert!(
        v["missing"]
            .as_str()
            .unwrap()
            .contains("需要获取例子访客身份"),
        "{v}"
    );
}

fn link(url: &str) -> DeliveredLink {
    DeliveredLink {
        url: url::Url::parse(url).unwrap(),
        title: Some("标题".into()),
        acted_at: None,
    }
}

#[tokio::test]
async fn links_from_a_site_window_become_imports_and_the_window_use_is_recorded() {
    let f = Fixture::new();
    let engine = f.engine(Duration::from_secs(90));
    let spec = engine.window_spec("fake/visit", "in-usuyo").unwrap();
    assert_eq!(
        (spec.site.as_str(), spec.login_url.as_str()),
        ("www.example.test", "https://www.example.test/login")
    );
    assert_eq!(spec.script, "(function (usuyo) {})");
    assert_eq!(spec.guide.as_deref(), Some("登录以后读你的收藏"));
    // 别的方法不是站点窗口
    assert!(engine.window_spec("fake/visit", "page").is_err());

    let site = || {
        engine
            .sites()
            .unwrap()
            .into_iter()
            .find(|s| s.site == "www.example.test")
            .unwrap()
    };
    assert_eq!(site().login, LoginState::Missing);
    engine.window_signed_in("fake/visit", "in-usuyo").unwrap();
    assert_eq!(site().login, LoginState::Present);

    let handed = engine
        .window_links(
            "fake/visit",
            "in-usuyo",
            vec![
                WindowSource {
                    action: "favorite".into(),
                    links: vec![
                        link("https://www.example.test/n/1?xsec_token=a"),
                        link("https://www.example.test/n/2?xsec_token=b"),
                        link("https://www.example.test/n/1?xsec_token=a"),
                        link("https://other.test/n/3"),
                    ],
                },
                WindowSource {
                    action: "like".into(),
                    links: vec![link("https://www.example.test/n/4?xsec_token=c")],
                },
            ],
        )
        .unwrap();
    assert_eq!((handed.noted, handed.dropped), (3, 1));
    assert_eq!(handed.imports.len(), 2);
    let first = engine.import(&handed.imports[0].id).unwrap();
    let v = serde_json::to_value(&first).unwrap();
    assert_eq!(v["client"], "在 usuyo 中登录并读取", "{v}");
    // 上一回用站点窗口：哪种方法、找到几条
    let used = site().last_use.unwrap();
    assert_eq!(
        (used.method.as_str(), used.method_name.as_str(), used.noted),
        ("in-usuyo", "在 usuyo 中登录并读取", 3)
    );
    // 退出登录：记下的都去掉
    engine.forget_site("www.example.test").unwrap();
    assert_eq!(site().login, LoginState::Missing);
    assert!(site().last_use.is_none());
}
