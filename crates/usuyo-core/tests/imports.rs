//! 导入、节奏、暂停、收件文件夹：引擎对着假插件（sh 脚本）跑，不碰网络，不碰系统钥匙串。
#![cfg(unix)]

use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::time::Duration;

use tempfile::TempDir;
use usuyo_core::view::ImportState;
use usuyo_core::{DeliveredLink, Engine, EngineConfig, ImportChoice, Links, RetryPolicy, Via};

/// 假的「列出你存过的」：两页，第二页里有一条和第一页重复。
/// `canonicalize` 认 `example.test/<id>`，`fetch` 取到一段正文。
const SCRIPT: &str = r#"
case "$1" in
  canonicalize)
    id=${2##*/}
    printf '{"ok": true, "match": true, "identity": {"platform": "example", "contentId": "%s", "url": "https://example.test/%s"}}' "$id" "$id"
    ;;
  sources)
    echo '{"ok": true, "sources": [{"source": "example:fav", "title": "我的收藏", "action": "favorite"}]}'
    ;;
  list)
    echo "$4" >> "$USUYO_CACHE_DIR/cursors"
    if [ -f "$USUYO_CACHE_DIR/fail-list" ]; then cat "$USUYO_CACHE_DIR/fail-list"; exit 1; fi
    if [ -z "$4" ]; then
      cat <<'JSON'
{"ok": true, "items": [
  {"url": "https://example.test/a", "title": "A", "action": "favorite", "actedAt": "2026-10-30T21:00:00+08:00"},
  {"url": "https://example.test/b", "identity": {"platform": "example", "contentId": "b"}, "title": "B", "action": "favorite"}
], "nextCursor": "p2", "evidence": {"fetchedAt": "2026-11-01T03:00:01Z", "access": "session-api"}}
JSON
    else
      cat <<'JSON'
{"ok": true, "items": [
  {"url": "https://example.test/c", "title": "C", "action": "favorite"},
  {"url": "https://example.test/a", "title": "A again", "action": "favorite"},
  {"url": "https://example.test/d", "title": "D", "action": "favorite"}
], "nextCursor": null, "evidence": {"fetchedAt": "2026-11-01T03:00:01Z", "access": "session-api"}}
JSON
    fi
    ;;
  fetch)
    echo "$2 $6" >> "$USUYO_CACHE_DIR/fetched"
    id=${2##*/}
    cat <<JSON
{"ok": true, "identity": {"platform": "example", "contentId": "$id"}, "kind": "article",
 "title": "标题 $id", "text": "正文 $id 闷蒸", "images": [], "media": [], "platformTags": [],
 "completeness": "full", "missing": [],
 "evidence": {"fetchedAt": "2026-11-01T03:00:01Z", "access": "session-api"}}
JSON
    ;;
esac
"#;

struct Fixture {
    tmp: TempDir,
    library: PathBuf,
}

impl Fixture {
    fn new(pacing: &str) -> Fixture {
        let tmp = tempfile::tempdir().unwrap();
        let machine = tmp.path().join("machine");
        let library = Engine::init_library(&tmp.path().join("lib"), &machine).unwrap();
        let dir = machine.join("plugins/fake/saved/1.0.0");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            dir.join("usuyo-plugin.yaml"),
            format!(
                r#"protocol: 1
id: fake/saved
kind: fetch
name: 例子
version: 1.0.0
license: MIT
maintainers: [someone]
platforms: [example]
match: ["example.test/*"]
access: session-api
login: required
sites:
  - {{site: www.example.test, name: 例子, credential: cookies, loginUrl: "https://www.example.test/", cookies: [sid]}}
capabilities:
  fetch:
    access: public-web
    login: none
    outputs: [text, images, media:video, meta]
  list:
    access: session-api
    login: required
    sites: [www.example.test]
    order: newest-first
    sources:
      - {{source: "example:fav", title: 我的收藏, action: favorite}}
{pacing}
permissions:
  fs: {{write: ["${{OUT}}"]}}
run:
  command: ["${{PLUGIN_DIR}}/plugin.sh"]
"#
            ),
        )
        .unwrap();
        let path = dir.join("plugin.sh");
        std::fs::write(
            &path,
            format!("#!/bin/sh\nout=\"\"\nif [ \"$1\" = fetch ]; then out=\"$4\"; fi\n{SCRIPT}\n"),
        )
        .unwrap();
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).unwrap();
        Fixture { tmp, library }
    }

    fn machine(&self) -> PathBuf {
        self.tmp.path().join("machine")
    }

    fn engine(&self) -> Engine {
        let mut config = EngineConfig::new(&self.library, self.machine());
        config.retry = RetryPolicy {
            base: Duration::ZERO,
            max_attempts: 3,
        };
        config.inbox_settle = Duration::ZERO;
        Engine::open(config).unwrap()
    }

    fn cache(&self, file: &str) -> String {
        std::fs::read_to_string(self.machine().join("cache/plugins/fake/saved").join(file))
            .unwrap_or_default()
    }

    fn set_cache(&self, file: &str, body: &str) {
        let dir = self.machine().join("cache/plugins/fake/saved");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join(file), body).unwrap();
    }

    fn manifest(&self, id: &str) -> serde_json::Value {
        let platform = id.rsplit_once('-').unwrap().0;
        let p = self
            .library
            .join("items")
            .join(platform)
            .join(id)
            .join("manifest.json");
        serde_json::from_slice(&std::fs::read(p).unwrap()).unwrap()
    }
}

fn only_new() -> ImportChoice {
    ImportChoice {
        new: true,
        ..ImportChoice::default()
    }
}

fn both() -> ImportChoice {
    ImportChoice {
        new: true,
        trashed: true,
        ..ImportChoice::default()
    }
}

async fn list_all(engine: &Engine, id: &str) {
    for _ in 0..10 {
        if engine.import(id).unwrap().state != ImportState::Listing {
            return;
        }
        engine.advance_imports().await.unwrap();
    }
    panic!("still listing: {:?}", engine.import(id).unwrap());
}

#[tokio::test]
async fn pages_through_and_previews_without_creating_anything() {
    let f = Fixture::new("");
    let engine = f.engine();
    // b 已经在库里，c 在回收站里
    let b = engine
        .take("https://example.test/b", "link", Via::Cli)
        .await
        .unwrap();
    let c = engine
        .take("https://example.test/c", "link", Via::Cli)
        .await
        .unwrap();
    engine.run_due().await.unwrap();
    engine.trash(&c.id, Via::App, &Links::default()).unwrap();
    let items_before = engine.map().unwrap().points.len();

    let imp = engine
        .start_import("fake/saved", "example:fav", "我的收藏")
        .unwrap();
    assert_eq!(imp.state, ImportState::Listing);
    list_all(&engine, &imp.id).await;
    assert_eq!(f.cache("cursors"), "\np2\n", "第一页不带游标，第二页带");
    let v = engine.import(&imp.id).unwrap();
    assert_eq!(v.state, ImportState::Ready);
    assert_eq!(v.pages, 2);
    assert_eq!(
        (v.preview.new, v.preview.stored, v.preview.trashed),
        (2, 1, 1),
        "{v:?}"
    );
    assert_eq!(
        engine.map().unwrap().points.len(),
        items_before,
        "预览什么都不建"
    );
    let new = engine.import_entries(&imp.id, Some("new"), 0, 50).unwrap();
    assert_eq!(
        new.iter()
            .map(|e| e.title.as_deref().unwrap())
            .collect::<Vec<_>>(),
        ["A", "D"]
    );
    let stored = engine
        .import_entries(&imp.id, Some("stored"), 0, 50)
        .unwrap();
    assert_eq!(stored[0].item_id.as_deref(), Some(b.id.as_str()));

    engine.confirm_import(&imp.id, &only_new()).unwrap();
    assert_eq!(engine.run_due().await.unwrap(), 2);
    let v = engine.import(&imp.id).unwrap();
    assert_eq!(v.state, ImportState::Done);
    assert_eq!((v.taken, v.not_stored, v.skipped), (2, 0, 0));
    assert!(
        engine
            .item(&c.id, &Links::default())
            .unwrap()
            .trashed_at
            .is_some(),
        "回收站里的没拿回来"
    );

    // 收下的条目：来源是导入，你决定收的，不是机器挑的
    let a = &new[0];
    let m = f.manifest(a.item_id.as_deref().unwrap());
    let o = &m["origins"][0];
    assert_eq!(o["entry"], "import");
    assert_eq!(o["takenBy"], "user");
    assert!(o["machinePicked"].is_null());
    assert_eq!(o["importBatch"], imp.id);
    assert_eq!(o["action"], "favorite");
    assert_eq!(o["actedAt"], "2026-10-30T13:00:00.000Z");
    assert_eq!(o["actedAtOriginal"], "2026-10-30T21:00:00+08:00");
    assert_eq!(m["title"], "标题 a");
}

#[tokio::test]
async fn bringing_trashed_ones_back_only_when_asked() {
    let f = Fixture::new("");
    let engine = f.engine();
    let c = engine
        .take("https://example.test/c", "link", Via::Cli)
        .await
        .unwrap();
    engine.run_due().await.unwrap();
    engine.trash(&c.id, Via::App, &Links::default()).unwrap();
    let imp = engine
        .start_import("fake/saved", "example:fav", "")
        .unwrap();
    list_all(&engine, &imp.id).await;
    engine.confirm_import(&imp.id, &both()).unwrap();
    engine.run_due().await.unwrap();
    assert!(
        engine
            .item(&c.id, &Links::default())
            .unwrap()
            .trashed_at
            .is_none()
    );
    assert_eq!(engine.import(&imp.id).unwrap().taken, 4);
}

#[tokio::test]
async fn takes_only_what_you_chose_entry_by_entry() {
    let f = Fixture::new("");
    let engine = f.engine();
    let c = engine
        .take("https://example.test/c", "link", Via::Cli)
        .await
        .unwrap();
    engine.run_due().await.unwrap();
    engine.trash(&c.id, Via::App, &Links::default()).unwrap();
    let imp = engine
        .start_import("fake/saved", "example:fav", "")
        .unwrap();
    list_all(&engine, &imp.id).await;
    let new = engine.import_entries(&imp.id, Some("new"), 0, 50).unwrap();
    let trashed = engine
        .import_entries(&imp.id, Some("trashed"), 0, 50)
        .unwrap();
    let skip_a = new
        .iter()
        .find(|e| e.title.as_deref() == Some("A"))
        .unwrap();

    // 认不出的条目、已经存着的不能出现在例外里：什么都不排
    let stray = ImportChoice {
        except: [9999].into(),
        ..both()
    };
    assert!(engine.confirm_import(&imp.id, &stray).is_err());

    // 新的整组收、A 不收；回收站那组整组不收、C 例外收
    let choice = ImportChoice {
        new: true,
        trashed: false,
        except: [skip_a.seq, trashed[0].seq].into(),
    };
    engine.confirm_import(&imp.id, &choice).unwrap();
    engine.run_due().await.unwrap();
    let after = engine.import_entries(&imp.id, None, 0, 50).unwrap();
    let taken: Vec<&str> = after
        .iter()
        .filter(|e| e.taken)
        .map(|e| e.title.as_deref().unwrap())
        .collect();
    assert_eq!(taken, ["B", "C", "D"], "{after:?}");
    assert!(
        engine
            .item(&c.id, &Links::default())
            .unwrap()
            .trashed_at
            .is_none(),
        "挑了的那条从回收站拿回来"
    );
    assert_eq!(engine.import(&imp.id).unwrap().taken, 3);
}

#[tokio::test]
async fn skipping_the_trashed_group_leaves_it_in_the_trash() {
    let f = Fixture::new("");
    let engine = f.engine();
    let c = engine
        .take("https://example.test/c", "link", Via::Cli)
        .await
        .unwrap();
    engine.run_due().await.unwrap();
    engine.trash(&c.id, Via::App, &Links::default()).unwrap();
    let imp = engine
        .start_import("fake/saved", "example:fav", "")
        .unwrap();
    list_all(&engine, &imp.id).await;
    let none = ImportChoice::default();
    engine.confirm_import(&imp.id, &none).unwrap();
    assert_eq!(engine.run_due().await.unwrap(), 0);
    let v = engine.import(&imp.id).unwrap();
    assert_eq!((v.state, v.taken), (ImportState::Done, 0));
    assert!(
        engine
            .item(&c.id, &Links::default())
            .unwrap()
            .trashed_at
            .is_some()
    );
}

#[tokio::test]
async fn stops_and_resumes_across_a_restart() {
    let f = Fixture::new("");
    let id = {
        let engine = f.engine();
        let imp = engine
            .start_import("fake/saved", "example:fav", "")
            .unwrap();
        // 列到一半停下
        engine.advance_imports().await.unwrap();
        engine.hold_import(&imp.id).unwrap();
        imp.id
    };
    let engine = f.engine();
    assert_eq!(engine.import(&id).unwrap().state, ImportState::Held);
    assert_eq!(engine.advance_imports().await.unwrap().0, 0, "停着的不列");
    engine.resume_import(&id).unwrap();
    list_all(&engine, &id).await;
    assert_eq!(f.cache("cursors"), "\np2\n", "从停的那一页接着列");

    engine.confirm_import(&id, &only_new()).unwrap();
    engine.hold_import(&id).unwrap();
    assert_eq!(engine.run_due().await.unwrap(), 0, "停着的不收");
    drop(engine);
    let engine = f.engine();
    assert_eq!(engine.run_due().await.unwrap(), 0, "重启以后也还停着");
    engine.resume_import(&id).unwrap();
    assert_eq!(engine.run_due().await.unwrap(), 4);
    let v = engine.import(&id).unwrap();
    assert_eq!((v.state, v.taken), (ImportState::Done, 4));
}

#[tokio::test]
async fn takes_at_the_platforms_pace() {
    let f = Fixture::new("pacing:\n  example: {batch: true, interval: 1s, daily: 3}");
    let engine = f.engine();
    let imp = engine
        .start_import("fake/saved", "example:fav", "")
        .unwrap();
    // 两页之间要隔一秒：列完第一页，下一页最早一秒后
    let (_, wake) = engine.advance_imports().await.unwrap();
    assert!(wake.is_some(), "还有一页，下一页按间隔再列");
    list_all(&engine, &imp.id).await;
    engine.confirm_import(&imp.id, &only_new()).unwrap();

    // 两页 list 已经算了 2 次批量调用；今天还能取 1 条
    assert_eq!(engine.run_due().await.unwrap(), 0, "上一次调用刚结束");
    let v = engine.import(&imp.id).unwrap();
    assert_eq!(v.state, ImportState::Taking);
    assert_eq!(v.waiting.as_ref().map(|w| w.code.as_str()), Some("pacing"));
    assert!(v.next_at.is_some());
    tokio::time::sleep(Duration::from_millis(1050)).await;
    assert_eq!(engine.run_due().await.unwrap(), 1);
    tokio::time::sleep(Duration::from_millis(1050)).await;
    assert_eq!(engine.run_due().await.unwrap(), 0, "今天的批量次数用完了");
    let v = engine.import(&imp.id).unwrap();
    assert_eq!(
        v.waiting.as_ref().map(|w| w.code.as_str()),
        Some("daily-limit"),
        "{v:?}"
    );
    assert_eq!(v.taken, 1);

    // 你自己收的一条不算进每天的次数，只守间隔
    let t = engine
        .take("https://example.test/z", "link", Via::App)
        .await
        .unwrap();
    assert_eq!(engine.run_due().await.unwrap(), 1);
    assert_eq!(
        engine.item(&t.id, &Links::default()).unwrap().summary.title,
        "标题 z"
    );
}

#[tokio::test]
async fn a_rate_limited_listing_waits_and_try_now_clears_it() {
    let f = Fixture::new("");
    f.set_cache(
        "fail-list",
        r#"{"ok": false, "error": {"code": "rate-limited", "message": "429", "hint": "慢一点", "retryAfter": 600}}"#,
    );
    let engine = f.engine();
    let imp = engine
        .start_import("fake/saved", "example:fav", "")
        .unwrap();
    let (_, wake) = engine.advance_imports().await.unwrap();
    assert!(wake.unwrap() > usuyo_core_now() + 500_000);
    let v = engine.import(&imp.id).unwrap();
    let w = v.waiting.unwrap();
    assert_eq!(
        (w.code.as_str(), w.hint.as_deref()),
        ("rate-limited", Some("慢一点"))
    );
    assert!(w.until.is_some());
    let plugins = engine.plugins().unwrap();
    let p = &plugins.plugins[0];
    assert_eq!(p.pauses.len(), 1);
    assert_eq!(p.pauses[0].code.as_deref(), Some("rate-limited"));
    assert!(p.pauses[0].until.is_some() && !p.pauses[0].by_you);

    std::fs::remove_file(f.machine().join("cache/plugins/fake/saved/fail-list")).unwrap();
    engine.try_plugin_now("fake/saved").unwrap();
    assert!(engine.plugins().unwrap().plugins[0].pauses.is_empty());
    assert!(
        engine.import(&imp.id).unwrap().waiting.is_none(),
        "现在再试：马上接着列"
    );
    list_all(&engine, &imp.id).await;
    assert_eq!(engine.import(&imp.id).unwrap().state, ImportState::Ready);
}

#[tokio::test]
async fn login_required_stops_until_you_resume() {
    let f = Fixture::new("");
    f.set_cache(
        "fail-list",
        r#"{"ok": false, "error": {"code": "login-required", "message": "401", "hint": "请重新登录例子", "site": "www.example.test"}}"#,
    );
    let engine = f.engine();
    let imp = engine
        .start_import("fake/saved", "example:fav", "")
        .unwrap();
    engine.advance_imports().await.unwrap();
    let v = engine.import(&imp.id).unwrap();
    let w = v.waiting.unwrap();
    assert_eq!(
        (w.code.as_str(), w.hint.as_deref(), w.until),
        ("login-required", Some("请重新登录例子"), None)
    );
    assert_eq!(engine.advance_imports().await.unwrap().0, 0, "不自己反复试");
    std::fs::remove_file(f.machine().join("cache/plugins/fake/saved/fail-list")).unwrap();
    engine.resume_import(&imp.id).unwrap();
    list_all(&engine, &imp.id).await;
}

#[tokio::test]
async fn hold_and_try_now_on_a_plugin() {
    let f = Fixture::new("");
    let engine = f.engine();
    let p = engine.hold_plugin("fake/saved").unwrap();
    assert!(p.pauses[0].by_you && p.pauses[0].until.is_none());
    let imp = engine
        .start_import("fake/saved", "example:fav", "")
        .unwrap();
    engine.advance_imports().await.unwrap();
    assert_eq!(f.cache("cursors"), "", "先别试的插件不调");
    let v = engine.import(&imp.id).unwrap();
    assert_eq!(v.waiting.map(|w| w.code), Some("held".into()));
    engine.try_plugin_now("fake/saved").unwrap();
    list_all(&engine, &imp.id).await;
}

#[tokio::test]
async fn the_registry_shows_labels_sites_and_pacing() {
    let f = Fixture::new("pacing:\n  example: {batch: true, interval: 5s, daily: 500}");
    let engine = f.engine();
    let v = engine.plugins().unwrap();
    assert!(v.not_loaded.is_empty());
    let p = &v.plugins[0];
    let json = serde_json::to_value(p).unwrap();
    assert_eq!(json["access"], "session-api");
    assert_eq!(json["login"], "required");
    assert_eq!(json["canList"], true);
    assert_eq!(json["sites"], serde_json::json!(["www.example.test"]));
    assert_eq!(
        json["pacing"],
        serde_json::json!([{"platform": "example", "batch": true, "intervalSec": 5, "daily": 500}])
    );
    assert_eq!(json["sources"][0]["source"], "example:fav");
    let sites = serde_json::to_value(engine.sites().unwrap()).unwrap();
    assert_eq!(sites[0]["site"], "www.example.test");
    assert_eq!(sites[0]["login"], "missing");
    assert_eq!(sites[0]["cookies"], serde_json::json!(["sid"]));
    let state = serde_json::from_value(serde_json::json!({"cookies": [
        {"name": "sid", "value": "s3cret-value", "domain": ".example.test"}]}))
    .unwrap();
    let s = engine.save_site("www.example.test", &state).unwrap();
    assert_eq!(serde_json::to_value(&s).unwrap()["login"], "present");
    let shown = serde_json::to_string(&engine.sites().unwrap()).unwrap()
        + &serde_json::to_string(&engine.plugins().unwrap()).unwrap();
    assert!(!shown.contains("s3cret-value"));
    let sources = engine.plugin_sources("fake/saved").await.unwrap();
    assert_eq!(sources[0].title, "我的收藏");
}

#[tokio::test]
async fn a_delivered_batch_goes_through_the_same_preview() {
    let f = Fixture::new("");
    let engine = f.engine();
    let b = engine
        .take("https://example.test/b", "link", Via::Cli)
        .await
        .unwrap();
    let link = |u: &str| DeliveredLink {
        url: url::Url::parse(u).unwrap(),
        title: Some(u.into()),
        acted_at: None,
    };
    let imp = engine
        .deliver_batch(
            "小红书收藏夹",
            Some("favorite"),
            vec![
                link("https://example.test/b"),
                link("https://example.test/q"),
                link("https://example.test/q"),
            ],
            Some("脚本"),
        )
        .unwrap();
    assert_eq!(imp.client.as_deref(), Some("脚本"));
    list_all(&engine, &imp.id).await;
    let v = engine.import(&imp.id).unwrap();
    assert_eq!(
        (v.preview.new, v.preview.stored),
        (1, 1),
        "身份由宿主重新算，同一条只算一次"
    );
    engine.confirm_import(&imp.id, &only_new()).unwrap();
    engine.run_due().await.unwrap();
    assert_eq!(engine.import(&imp.id).unwrap().taken, 1);
    assert_eq!(
        engine.item(&b.id, &Links::default()).unwrap().summary.title,
        "标题 b"
    );
}

#[tokio::test]
async fn the_inbox_takes_links_from_txt_files_and_ignores_the_rest() {
    let f = Fixture::new("");
    let engine = f.engine();
    let inbox = engine.inbox_dir().to_owned();
    std::fs::create_dir_all(&inbox).unwrap();
    std::fs::write(
        inbox.join("share.txt"),
        "看看 https://example.test/m1 复制打开\nhttps://example.test/m2\n",
    )
    .unwrap();
    std::fs::write(inbox.join("photo.jpg"), b"\xff\xd8").unwrap();
    std::fs::write(
        inbox.join(".share2.txt.icloud"),
        "https://example.test/nope",
    )
    .unwrap();
    std::fs::write(inbox.join("empty.txt"), "没有链接").unwrap();
    std::os::unix::fs::symlink(f.tmp.path().join("lib"), inbox.join("link.txt")).unwrap();

    let r = engine.scan_inbox().await.unwrap();
    assert_eq!(r.taken, 2);
    assert!(inbox.join("photo.jpg").exists() && inbox.join(".share2.txt.icloud").exists());
    assert!(!inbox.join("share.txt").exists());
    let done: Vec<PathBuf> = std::fs::read_dir(inbox.join(".usuyo-done"))
        .unwrap()
        .flat_map(|d| std::fs::read_dir(d.unwrap().path()).unwrap())
        .map(|e| e.unwrap().path())
        .collect();
    assert_eq!(done.len(), 1);
    assert!(inbox.join(".usuyo-failed/empty.txt").exists());
    assert!(inbox.join(".usuyo-failed/empty.txt.error.json").exists());
    assert!(
        inbox.join(".usuyo-failed/link.txt").exists(),
        "符号链接不读"
    );
    engine.run_due().await.unwrap();
    let hits = engine.search("闷蒸", 10).unwrap();
    assert_eq!(hits.len(), 2);
    let m = f.manifest(&hits[0].item_id);
    assert_eq!(m["origins"][0]["entry"], "mobile");
    assert_eq!(engine.scan_inbox().await.unwrap().taken, 0);
}

fn usuyo_core_now() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as i64
}
