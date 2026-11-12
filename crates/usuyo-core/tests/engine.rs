//! 引擎对着假插件（sh 脚本）跑：收下 → 存好 → 搜到、同一链接两次、插件失败后重试、换下一个插件、重建索引。
//! 不碰真网络：假插件不发请求。
#![cfg(unix)]

use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::time::Duration;

use tempfile::TempDir;
use usuyo_core::view::{Event, Facet, HitLayer, Stored};
use usuyo_core::{Engine, EngineConfig, Links, RetryPolicy, Via};

/// 取到的网页：两段正文，一张「图」（不是真的 PNG，量不出尺寸）。
const FETCH_OK: &str = r#"
printf '<html>hi</html>' > "$out/page.html"
mkdir -p "$out/images" && printf '0123456789' > "$out/images/001.png"
cat <<JSON
{"ok": true, "identity": {"platform": "web", "url": "$2"}, "kind": "article",
 "title": "手冲咖啡：三段式注水", "author": {"name": "咖啡日常"},
 "publishedAt": "2026-10-01T20:00:00+08:00",
 "text": "第一段用四十克水闷蒸三十秒。\n\n第二段注水到一百五十克。",
 "html": "page.html", "images": ["images/001.png"], "media": [], "platformTags": [],
 "completeness": "full", "missing": [],
 "evidence": {"fetchedAt": "2026-11-01T03:00:01Z", "access": "public-web"}}
JSON
"#;

const NO_MATCH: &str = r#"if [ "$1" = canonicalize ]; then echo '{"ok": true, "match": false}'; exit 0; fi
if [ "$1" = health ]; then echo '{"ok": true, "ready": true, "protocol": 1, "version": "1.0.0"}'; exit 0; fi"#;

struct Fixture {
    tmp: TempDir,
    library: PathBuf,
}

impl Fixture {
    fn new() -> Fixture {
        let tmp = tempfile::tempdir().unwrap();
        let library =
            Engine::init_library(&tmp.path().join("lib"), &tmp.path().join("machine")).unwrap();
        Fixture { tmp, library }
    }

    fn machine(&self) -> PathBuf {
        self.tmp.path().join("machine")
    }

    /// 装一个假插件：`script` 是 sh 正文，`$1` 是子命令，fetch 时 `$out` 是输出目录。
    fn install(&self, id: &str, platforms: &str, matches: &str, script: &str) {
        let dir = self.machine().join("plugins").join(id).join("1.0.0");
        std::fs::create_dir_all(&dir).unwrap();
        let manifest = format!(
            r#"protocol: 1
id: {id}
kind: fetch
name: 假的
version: 1.0.0
license: MIT
maintainers: [someone]
platforms: [{platforms}]
match: ["{matches}"]
methods:
  - id: page
    name: 公开页
    access: public-web
    login: none
    account: none
    capabilities:
      fetch:
        outputs: [text, html, images, meta]
permissions:
  fs: {{write: ["${{OUT}}"]}}
run:
  command: ["${{PLUGIN_DIR}}/plugin.sh"]
"#
        );
        std::fs::write(dir.join("usuyo-plugin.yaml"), manifest).unwrap();
        let path = dir.join("plugin.sh");
        let body = format!(
            "#!/bin/sh\nout=\"\"\nif [ \"$1\" = fetch ]; then out=\"$4\"; fi\n{NO_MATCH}\n{script}\n"
        );
        std::fs::write(&path, body).unwrap();
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).unwrap();
    }

    fn engine(&self) -> Engine {
        let mut config = EngineConfig::new(&self.library, self.machine());
        config.retry = RetryPolicy {
            base: Duration::ZERO,
            max_attempts: 3,
        };
        Engine::open(config).unwrap()
    }

    fn item_dir(&self, id: &str) -> PathBuf {
        let platform = id.rsplit_once('-').unwrap().0;
        self.library.join("items").join(platform).join(id)
    }
}

fn manifest(dir: &Path) -> serde_json::Value {
    serde_json::from_slice(&std::fs::read(dir.join("manifest.json")).unwrap()).unwrap()
}

fn hit_ids(engine: &Engine, q: &str) -> Vec<String> {
    engine
        .search(q, 20)
        .unwrap()
        .into_iter()
        .map(|h| h.item_id)
        .collect()
}

#[tokio::test]
async fn take_stores_and_finds_it() {
    let f = Fixture::new();
    f.install("fake/web", "web", "*", FETCH_OK);
    let engine = f.engine();
    let mut events = engine.subscribe();

    let taken = engine
        .take(
            "【三段式注水】 https://example.test/pour?utm_source=x 复制打开",
            "link",
            Via::Cli,
        )
        .await
        .unwrap();
    assert!(!taken.seen);
    assert!(taken.id.starts_with("web-"));
    // 马上有条目，只有链接
    let view = engine.item(&taken.id, &Links::default()).unwrap();
    assert_eq!(view.stored, Stored::Link);
    assert_eq!(view.missing, None);
    assert!(matches!(
        events.recv().await.unwrap(),
        Event::ItemAdded { .. }
    ));

    assert_eq!(engine.run_due().await.unwrap(), 1);
    let dir = f.item_dir(&taken.id);
    let m = manifest(&dir);
    let snap = m["primarySnapshot"].as_str().unwrap();
    assert!(dir.join("raw").join(snap).join("page.html").is_file());
    assert!(
        dir.join("raw")
            .join(snap)
            .join("usuyo.fetch.json")
            .is_file()
    );
    let content = std::fs::read_to_string(dir.join("content.md")).unwrap();
    assert!(content.starts_with("# 手冲咖啡：三段式注水\n"), "{content}");
    assert!(content.contains("闷蒸"));
    assert_eq!(m["publishedAt"], "2026-10-01T12:00:00.000Z");
    assert_eq!(m["publishedAtOriginal"], "2026-10-01T20:00:00+08:00");

    let view = engine
        .item(
            &taken.id,
            &Links {
                base: "http://127.0.0.1:1".into(),
                key: Some("k".into()),
            },
        )
        .unwrap();
    assert_eq!(view.stored, Stored::Complete);
    assert_eq!(view.summary.title, "手冲咖啡：三段式注水");
    assert_eq!(view.summary.author.as_deref(), Some("咖啡日常"));
    let Some(Facet::Image { images, .. }) = view.facets.first() else {
        panic!("{:?}", view.facets)
    };
    assert_eq!(
        images[0].url,
        format!(
            "http://127.0.0.1:1/media/{}/image/{}?k=k",
            taken.id, images[0].key
        )
    );
    let Some(Facet::Text { segments, .. }) = view.facets.get(1) else {
        panic!("{:?}", view.facets)
    };
    assert_eq!(segments.len(), 2);
    let media = engine
        .media(&taken.id, &format!("image/{}", images[0].key))
        .unwrap();
    assert_eq!(std::fs::read(media).unwrap(), b"0123456789");
    assert!(engine.media(&taken.id, "notes").is_err(), "只给媒体层");

    let hits = engine.search("闷蒸", 20).unwrap();
    assert_eq!(hits.len(), 1);
    assert_eq!(hits[0].layer, HitLayer::Text);
    let h = &hits[0];
    let marked: Vec<u16> = h.snippet.encode_utf16().collect();
    assert_eq!(
        String::from_utf16(&marked[h.matched[0]..h.matched[1]]).unwrap(),
        "闷蒸"
    );
    assert_eq!(hit_ids(&engine, "shouchong"), [taken.id.as_str()], "拼音");
    assert_eq!(
        engine.search("咖啡日常", 20).unwrap()[0].layer,
        HitLayer::Author
    );

    // 记一句也能搜到，改动经 op 记下
    let v = engine
        .set_note(&taken.id, "下次磨粗一点", Via::Cli, &Links::default())
        .unwrap();
    assert_eq!(v.note.as_deref(), Some("下次磨粗一点"));
    assert_eq!(engine.search("磨粗", 20).unwrap()[0].layer, HitLayer::Note);

    // 删掉：搜不到，读得到 trashedAt；拿回来又搜得到
    let v = engine
        .trash(&taken.id, Via::Cli, &Links::default())
        .unwrap();
    assert!(v.trashed_at.is_some());
    assert!(
        matches!(v.facets.as_slice(), [Facet::Text { .. }]),
        "在回收站里也看得到字，图不经媒体地址给：{:?}",
        v.facets
    );
    assert_eq!(v.note.as_deref(), Some("下次磨粗一点"));
    assert!(hit_ids(&engine, "闷蒸").is_empty());
    let v = engine
        .restore(&taken.id, Via::Cli, &Links::default())
        .unwrap();
    assert!(v.trashed_at.is_none());
    assert_eq!(hit_ids(&engine, "闷蒸"), [taken.id.as_str()]);

    let map = engine.map().unwrap();
    assert_eq!(map.points.len(), 1);
}

#[tokio::test]
async fn passes_plugin_settings() {
    let f = Fixture::new();
    let script = format!(
        r#"grep -q '"allow-private-network":true' "$USUYO_SETTINGS" || exit 3
{FETCH_OK}"#
    );
    f.install("fake/web", "web", "*", &script);
    let mut config = EngineConfig::new(&f.library, f.machine());
    config.plugin_settings.insert(
        "fake/web".into(),
        serde_json::json!({"allow-private-network": true}),
    );
    let engine = Engine::open(config).unwrap();
    let t = engine
        .take("http://127.0.0.1:9/a", "link", Via::Cli)
        .await
        .unwrap();
    engine.run_due().await.unwrap();
    assert_eq!(
        engine.item(&t.id, &Links::default()).unwrap().stored,
        Stored::Complete
    );
}

#[tokio::test]
async fn same_link_twice_is_one_item() {
    let f = Fixture::new();
    f.install("fake/web", "web", "*", FETCH_OK);
    let engine = f.engine();
    let a = engine
        .take("https://example.test/pour", "link", Via::App)
        .await
        .unwrap();
    engine.run_due().await.unwrap();
    let b = engine
        .take("https://EXAMPLE.test/pour#top", "link", Via::App)
        .await
        .unwrap();
    assert_eq!(a.id, b.id);
    assert!(b.seen);
    engine.run_due().await.unwrap();
    let m = manifest(&f.item_dir(&a.id));
    assert_eq!(m["origins"].as_array().unwrap().len(), 2);
    // 第二次取到的和第一次一模一样：不多一份快照
    assert_eq!(m["snapshots"].as_array().unwrap().len(), 1);
    assert_eq!(engine.map().unwrap().points.len(), 1);

    // 在回收站里的再收一次：拿回来，不是第二条
    engine.trash(&a.id, Via::App, &Links::default()).unwrap();
    let c = engine
        .take("https://example.test/pour", "link", Via::App)
        .await
        .unwrap();
    assert_eq!(c.id, a.id);
    assert!(c.seen);
    assert!(
        engine
            .item(&a.id, &Links::default())
            .unwrap()
            .trashed_at
            .is_none()
    );
}

#[tokio::test]
async fn retries_after_a_failing_plugin() {
    let f = Fixture::new();
    // 第一次断网，第二次取到。
    let script = format!(
        r#"if [ ! -f "$USUYO_CACHE_DIR/tried" ]; then
  touch "$USUYO_CACHE_DIR/tried"
  echo '{{"ok": false, "error": {{"code": "network", "message": "offline"}}}}'
  exit 1
fi
{FETCH_OK}"#
    );
    f.install("fake/web", "web", "*", &script);
    let engine = f.engine();
    let mut events = engine.subscribe();
    let t = engine
        .take("https://example.test/pour", "link", Via::Cli)
        .await
        .unwrap();
    engine.run_due().await.unwrap();
    let mut saw_retry = false;
    while let Ok(e) = events.try_recv() {
        if let Event::ItemFetchFailed { code, retry, .. } = e {
            assert_eq!(code, "network");
            saw_retry = retry;
        }
    }
    assert!(saw_retry, "失败过一次，标了会重试");
    let v = engine.item(&t.id, &Links::default()).unwrap();
    assert_eq!(v.stored, Stored::Complete);
}

#[tokio::test]
async fn gives_up_and_says_why() {
    let f = Fixture::new();
    f.install(
        "fake/web",
        "web",
        "*",
        r#"echo '{"ok": false, "error": {"code": "not-found", "message": "gone", "hint": "原帖删了"}}'; exit 1"#,
    );
    let engine = f.engine();
    let t = engine
        .take("https://example.test/gone", "link", Via::Cli)
        .await
        .unwrap();
    engine.run_due().await.unwrap();
    let v = engine.item(&t.id, &Links::default()).unwrap();
    assert_eq!(v.stored, Stored::Link);
    assert_eq!(v.missing.as_deref(), Some("取不到内容：原帖删了"));
}

#[tokio::test]
async fn blocked_plugin_pauses_and_the_next_one_takes_over() {
    let f = Fixture::new();
    f.install(
        "fake/platform",
        "example",
        "example.test/*",
        r#"if [ "$1" = fetch ]; then echo x >> "$USUYO_CACHE_DIR/calls"; fi
echo '{"ok": false, "error": {"code": "blocked", "message": "captcha"}}'; exit 1"#,
    );
    f.install("fake/web", "web", "*", FETCH_OK);
    let engine = f.engine();
    for path in ["a", "b"] {
        let t = engine
            .take(&format!("https://example.test/{path}"), "link", Via::Cli)
            .await
            .unwrap();
        engine.run_due().await.unwrap();
        let v = engine.item(&t.id, &Links::default()).unwrap();
        assert_eq!(v.stored, Stored::Complete, "通用网页插件接着取到了");
    }
    let calls = std::fs::read_to_string(f.machine().join("cache/plugins/fake/platform/calls"))
        .unwrap_or_default();
    assert_eq!(calls.lines().count(), 1, "被拦以后这个平台暂停，不再调它");
}

#[tokio::test]
async fn deleting_the_index_and_reindexing_gives_the_same_results() {
    let f = Fixture::new();
    f.install("fake/web", "web", "*", FETCH_OK);
    let queries = ["闷蒸", "shouchong", "咖啡", "注水 克"];
    let before: Vec<Vec<String>> = {
        let engine = f.engine();
        for p in ["a", "b", "c"] {
            engine
                .take(&format!("https://example.test/{p}"), "link", Via::Cli)
                .await
                .unwrap();
        }
        engine.run_due().await.unwrap();
        queries.iter().map(|q| hit_ids(&engine, q)).collect()
    };
    assert_eq!(before[0].len(), 3);
    let machine_dir = std::fs::read_dir(f.machine().join("libraries"))
        .unwrap()
        .next()
        .unwrap()
        .unwrap()
        .path();
    std::fs::remove_dir_all(machine_dir.join("index")).unwrap();
    let engine = f.engine();
    let after: Vec<Vec<String>> = queries.iter().map(|q| hit_ids(&engine, q)).collect();
    assert_eq!(before, after);
    assert_eq!(engine.reindex().unwrap().items, 3);
    let again: Vec<Vec<String>> = queries.iter().map(|q| hit_ids(&engine, q)).collect();
    assert_eq!(before, again);
}

#[tokio::test]
async fn follow_and_membership_shape_the_map() {
    let f = Fixture::new();
    f.install("fake/web", "web", "*", FETCH_OK);
    let engine = f.engine();
    let topic = engine.create_topic(" 手冲咖啡 ", Via::App).unwrap().id;
    // 同名的不再建一个
    assert_eq!(engine.create_topic("手冲咖啡", Via::App).unwrap().id, topic);
    assert_eq!(engine.topics().unwrap().len(), 1);
    assert!(engine.create_topic("  ", Via::App).is_err());
    assert!(!engine.topics().unwrap()[0].followed);
    let t = engine
        .take("https://example.test/pour", "link", Via::App)
        .await
        .unwrap();
    assert!(engine.map().unwrap().regions.is_empty());
    let v = engine
        .set_topic(&t.id, &topic, true, Via::App, &Links::default())
        .unwrap();
    assert_eq!(v.summary.topic_ids, [topic.as_str()]);
    let followed = engine.follow(&topic, true, Via::App).unwrap();
    assert!(followed.followed);
    let map = engine.map().unwrap();
    assert_eq!(map.regions.len(), 1);
    assert_eq!(map.regions[0].count, 1);
    assert!(
        engine
            .set_topic(&t.id, "no-such-topic", true, Via::App, &Links::default())
            .is_err()
    );
    let v = engine
        .set_topic(&t.id, &topic, false, Via::App, &Links::default())
        .unwrap();
    assert!(v.summary.topic_ids.is_empty());
}

#[tokio::test]
async fn partial_fetch_is_not_complete() {
    let f = Fixture::new();
    let partial = FETCH_OK.replace(
        r#""missing": []"#,
        r#""missing": [], "missingReason": "有 2 张图没下载下来""#,
    );
    f.install("fake/web", "web", "*", &partial);
    let engine = f.engine();
    let t = engine
        .take("https://example.test/some-images", "link", Via::App)
        .await
        .unwrap();
    engine.run_due().await.unwrap();
    let v = engine.item(&t.id, &Links::default()).unwrap();
    assert_eq!(v.stored, Stored::Partial);
    assert_eq!(
        v.missing.as_deref(),
        Some("有些没存下来（有 2 张图没下载下来）")
    );

    // 一张图都没取到：宿主自己从 missing 看出来
    let f = Fixture::new();
    let none = FETCH_OK
        .replace(r#", "images": ["images/001.png"]"#, ", \"images\": []")
        .replace(
            r#""missing": []"#,
            r#""missing": ["images"], "missingReason": "正文里的图片没有下载下来""#,
        )
        .replace(r#""completeness": "full""#, r#""completeness": "partial""#);
    f.install("fake/web", "web", "*", &none);
    let engine = f.engine();
    let t = engine
        .take("https://example.test/no-images", "link", Via::App)
        .await
        .unwrap();
    engine.run_due().await.unwrap();
    let v = engine.item(&t.id, &Links::default()).unwrap();
    assert_eq!(v.stored, Stored::Partial);
    assert!(v.missing.unwrap().contains("图片"), "should name images");

    // 页面本来就没有图，插件也没说缺：不算缺，照都存好了
    let f = Fixture::new();
    let plain = FETCH_OK.replace(r#", "images": ["images/001.png"]"#, ", \"images\": []");
    f.install("fake/web", "web", "*", &plain);
    let engine = f.engine();
    let t = engine
        .take("https://example.test/plain", "link", Via::App)
        .await
        .unwrap();
    engine.run_due().await.unwrap();
    let v = engine.item(&t.id, &Links::default()).unwrap();
    assert_eq!(v.stored, Stored::Complete);
    assert_eq!(v.missing, None);
}

fn machine_dir(f: &Fixture) -> PathBuf {
    std::fs::read_dir(f.machine().join("libraries"))
        .unwrap()
        .next()
        .unwrap()
        .unwrap()
        .path()
}

fn now_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as i64
}

#[tokio::test]
async fn a_rate_limit_pauses_one_site_not_the_whole_web() {
    let f = Fixture::new();
    let script = format!(
        r#"case "$2" in
  *slow.test*) cat <<'JSON'
{{"ok": false, "error": {{"code": "rate-limited", "message": "429", "retryAfter": 99999999999}}}}
JSON
  exit 1;;
esac
{FETCH_OK}"#
    );
    f.install("fake/web", "web", "*", &script);
    let engine = f.engine();
    let slow = engine
        .take("https://slow.test/a", "link", Via::Cli)
        .await
        .unwrap();
    engine.run_due().await.unwrap();
    let other = engine
        .take("https://example.test/b", "link", Via::Cli)
        .await
        .unwrap();
    assert_eq!(engine.run_due().await.unwrap(), 1);
    let v = engine.item(&other.id, &Links::default()).unwrap();
    assert_eq!(v.stored, Stored::Complete, "别的网站照取");

    let db = rusqlite::Connection::open(machine_dir(&f).join("jobs.db")).unwrap();
    let until: i64 = db
        .query_row("SELECT MAX(until_at) FROM pauses", [], |r| r.get(0))
        .unwrap();
    assert!(until <= now_ms() + 3_600_000, "暂停最多一小时：{until}");
    let due: i64 = db
        .query_row(
            "SELECT due_at FROM fetch_jobs WHERE item_id = ?1",
            [&slow.id],
            |r| r.get(0),
        )
        .unwrap();
    assert!(due <= now_ms() + 6 * 3_600_000, "{due}");
}

#[tokio::test]
async fn a_pause_saved_before_the_cap_does_not_hold_the_queue() {
    let f = Fixture::new();
    f.install("fake/web", "web", "*", FETCH_OK);
    let t = {
        let engine = f.engine();
        engine
            .take("https://example.test/pour", "link", Via::Cli)
            .await
            .unwrap()
    };
    let db = rusqlite::Connection::open(machine_dir(&f).join("jobs.db")).unwrap();
    db.execute("UPDATE fetch_jobs SET due_at = 101794421686988", [])
        .unwrap();
    db.execute(
        "INSERT INTO pauses (plugin, method, platform, until_at) VALUES
         ('fake/web', 'page', 'web', 101794421686988), ('fake/web', 'page', 'example.test', NULL)",
        [],
    )
    .unwrap();
    drop(db);
    let engine = f.engine();
    assert_eq!(engine.run_due().await.unwrap(), 1);
    let v = engine.item(&t.id, &Links::default()).unwrap();
    assert_eq!(v.stored, Stored::Complete);
}

#[tokio::test]
async fn one_unreadable_item_does_not_stop_the_rest() {
    let f = Fixture::new();
    f.install("fake/web", "web", "*", FETCH_OK);
    let ids: Vec<String> = {
        let engine = f.engine();
        let mut ids = Vec::new();
        for p in ["a", "b", "c"] {
            let t = engine
                .take(&format!("https://example.test/{p}"), "link", Via::Cli)
                .await
                .unwrap();
            ids.push(t.id);
        }
        engine.run_due().await.unwrap();
        ids
    };
    // 在 Obsidian 里手改坏的 notes.md，和写坏的 manifest.json、回收站项
    std::fs::write(
        f.item_dir(&ids[0]).join("notes.md"),
        "---\ntakeaway: hello\n---\n",
    )
    .unwrap();
    std::fs::write(f.item_dir(&ids[1]).join("manifest.json"), "{").unwrap();
    let trash = f
        .library
        .join(".usuyo/trash/0190a000-0000-7000-8000-000000000000");
    std::fs::create_dir_all(&trash).unwrap();
    std::fs::write(trash.join("entry.json"), "{").unwrap();
    std::fs::remove_dir_all(machine_dir(&f).join("index")).unwrap();

    let engine = f.engine();
    let map = engine.map().unwrap();
    assert_eq!(map.points.len(), 2);
    assert_eq!(
        map.unreadable.iter().map(|u| &u.id).collect::<Vec<_>>(),
        [&ids[1]]
    );
    let r = engine.reindex().unwrap();
    assert_eq!(r.items, 1);
    let mut skipped: Vec<&String> = r.unreadable.iter().map(|u| &u.id).collect();
    skipped.sort();
    let mut expected = vec![&ids[0], &ids[1]];
    expected.sort();
    assert_eq!(skipped, expected);
    engine
        .take("https://example.test/d", "link", Via::Cli)
        .await
        .unwrap();
    engine.run_due().await.unwrap();
    assert_eq!(hit_ids(&engine, "闷蒸").len(), 2, "c 和新收的 d");
}

#[tokio::test]
async fn links_with_a_user_name_or_password_are_not_taken() {
    let f = Fixture::new();
    f.install("fake/web", "web", "*", FETCH_OK);
    let engine = f.engine();
    let e = engine
        .take("https://me:secret@example.test/x", "link", Via::Cli)
        .await
        .unwrap_err();
    assert_eq!(e.code(), "unsupported-input");
}

#[tokio::test]
async fn a_foreign_file_with_the_topic_name_is_left_alone() {
    let f = Fixture::new();
    std::fs::write(f.library.join("topics/咖啡.md"), "随手记的，不是主题\n").unwrap();
    let engine = f.engine();
    engine.create_topic("咖啡", Via::App).unwrap();
    assert_eq!(engine.topics().unwrap().len(), 1);
    assert_eq!(
        std::fs::read_to_string(f.library.join("topics/咖啡.md")).unwrap(),
        "随手记的，不是主题\n"
    );
}

#[cfg(unix)]
#[tokio::test]
async fn jobs_database_is_private() {
    use std::os::unix::fs::PermissionsExt;
    let f = Fixture::new();
    let _engine = f.engine();
    let mode = std::fs::metadata(machine_dir(&f).join("jobs.db"))
        .unwrap()
        .permissions()
        .mode();
    assert_eq!(mode & 0o777, 0o600);
}

#[tokio::test]
async fn blocked_with_no_other_plugin_fails_once_and_forgets_the_link() {
    let f = Fixture::new();
    f.install(
        "fake/web",
        "web",
        "*",
        r#"if [ "$1" = fetch ]; then echo x >> "$USUYO_CACHE_DIR/calls"; fi
echo '{"ok": false, "error": {"code": "blocked", "message": "captcha", "hint": "要验证码"}}'; exit 1"#,
    );
    let engine = f.engine();
    let t = engine
        .take("https://example.test/a?xsec_token=secret", "link", Via::Cli)
        .await
        .unwrap();
    engine.run_due().await.unwrap();
    let v = engine.item(&t.id, &Links::default()).unwrap();
    assert_eq!(v.missing.as_deref(), Some("取不到内容：要验证码"));
    let calls = std::fs::read_to_string(f.machine().join("cache/plugins/fake/web/calls")).unwrap();
    assert_eq!(calls.lines().count(), 1);
    let db = rusqlite::Connection::open(machine_dir(&f).join("jobs.db")).unwrap();
    let url: String = db
        .query_row(
            "SELECT url FROM fetch_jobs WHERE item_id = ?1",
            [&t.id],
            |r| r.get(0),
        )
        .unwrap();
    assert!(!url.contains("secret"));
}

#[tokio::test]
async fn an_untitled_post_is_named_after_its_first_line_not_its_link() {
    let f = Fixture::new();
    let untitled = FETCH_OK
        .replace("\n \"title\": \"手冲咖啡：三段式注水\",", "\n")
        .replace("第一段用四十克水闷蒸三十秒。", "#周末 去山里拍了一段溪水");
    assert!(!untitled.contains("\"title\""), "{untitled}");
    f.install("fake/web", "web", "*", &untitled);
    let engine = f.engine();
    let t = engine
        .take(
            "https://example.test/v/1?share=abc&utm_source=x",
            "link",
            Via::App,
        )
        .await
        .unwrap();
    engine.run_due().await.unwrap();
    let v = engine.item(&t.id, &Links::default()).unwrap();
    assert_eq!(v.summary.title, "#周末 去山里拍了一段溪水");
    let content = std::fs::read_to_string(f.item_dir(&t.id).join("content.md")).unwrap();
    // content.md 的标题行照 Markdown 转义了 `#`
    assert!(
        content.starts_with("# \\#周末 去山里拍了一段溪水\n"),
        "{content}"
    );
}
