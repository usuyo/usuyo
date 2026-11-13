//! 从文件导入（协议 §3.9）：引擎对着假的读文件插件（sh 脚本）跑，不碰网络。
#![cfg(unix)]

use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::time::Duration;

use tempfile::TempDir;
use usuyo_core::view::{ImportKind, ImportState};
use usuyo_core::{Engine, EngineConfig, ImportChoice, RetryPolicy};

/// 假的读文件插件：`list` 读 `USUYO_INPUT`，交一条只有 `ref` 的（两次，去重）、一条带 `ref` 和身份的、
/// 一条只有链接的；`fetch <ref>` 从同一个文件里读，`fetch <链接>` 照常取。文件里有 `broken` 就说读不了。
const SCRIPT: &str = r#"
case "$1" in
  health)
    echo '{"ok": true, "ready": true, "protocol": 1, "version": "1.0.0"}'
    ;;
  canonicalize)
    id=${2##*/}
    printf '{"ok": true, "match": true, "identity": {"platform": "example", "contentId": "%s", "url": "https://example.test/%s"}}' "$id" "$id"
    ;;
  list)
    echo "$USUYO_INPUT" >> "$USUYO_CACHE_DIR/inputs"
    if grep -q broken "$USUYO_INPUT"; then
      echo '{"ok": false, "error": {"code": "unsupported-input", "message": "not an export", "hint": "这不是例子的导出文件"}}'
      exit 1
    fi
    cat <<'JSON'
{"ok": true, "items": [
  {"ref": "chat/1", "title": "聊天 1"},
  {"ref": "chat/1", "title": "聊天 1 又一次"},
  {"ref": "post/p", "identity": {"platform": "example", "contentId": "p"}, "title": "P", "action": "like"},
  {"url": "https://example.test/u", "title": "U", "action": "like"}
], "nextCursor": null, "evidence": {"fetchedAt": "2026-11-01T03:00:01Z", "access": "official-export"}}
JSON
    ;;
  fetch)
    echo "$2 $6 ${USUYO_INPUT:-none}" >> "$USUYO_CACHE_DIR/fetched"
    body=$(head -c 12 "${USUYO_INPUT:-/dev/null}")
    cat <<JSON
{"ok": true, "kind": "article",
 "title": "标题 $2", "text": "从文件里读的 $2 $body", "images": [], "media": [], "platformTags": [],
 "completeness": "full", "missing": [],
 "evidence": {"fetchedAt": "2026-11-01T03:00:01Z", "access": "official-export"}}
JSON
    ;;
esac
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
        let dir = machine.join("plugins/fake/reader/1.0.0");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            dir.join("usuyo-plugin.yaml"),
            r#"protocol: 1
id: fake/reader
kind: fetch
name: 例子
version: 1.0.0
license: MIT
maintainers: [someone]
platforms: [example]
match: ["example.test/*"]
methods:
  - id: page
    name: 公开页
    access: public-web
    login: none
    account: none
    capabilities:
      fetch:
        outputs: [text, meta]
  - id: export
    name: 个人数据下载
    access: official-export
    login: none
    account: none
    risk:
      text: 包里有没有收藏没核
      source: {label: 例子, url: "https://example.test/notes"}
    capabilities:
      list:
        order: unordered
        files: ["*.json"]
        reads: 例子设置里「下载你的数据」给的 JSON
        sources:
          - {source: "example:liked", title: 喜欢, action: like}
      fetch:
        input: [ref]
        outputs: [text, meta]
pacing:
  example: {batch: true, interval: 1h, daily: 1}
permissions:
  fs: {read: ["${INPUT}"], write: ["${OUT}"]}
run:
  command: ["${PLUGIN_DIR}/plugin.sh"]
"#,
        )
        .unwrap();
        let path = dir.join("plugin.sh");
        std::fs::write(&path, format!("#!/bin/sh\n{SCRIPT}\n")).unwrap();
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
        Engine::open(config).unwrap()
    }

    fn cache(&self, file: &str) -> String {
        std::fs::read_to_string(self.machine().join("cache/plugins/fake/reader").join(file))
            .unwrap_or_default()
    }

    /// 按机器目录里这个库的 `imports/`。
    fn copies(&self) -> PathBuf {
        let libs = self.machine().join("libraries");
        let lib = std::fs::read_dir(&libs).unwrap().next().unwrap().unwrap();
        lib.path().join("imports")
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

fn dir_entries(p: &Path) -> usize {
    std::fs::read_dir(p).map(|d| d.count()).unwrap_or(0)
}

async fn upload(
    engine: &Engine,
    name: &str,
    body: &str,
) -> usuyo_core::Result<usuyo_core::view::ImportView> {
    let up = engine.prepare_file_import("fake/reader#export", name)?;
    std::fs::write(up.path(), body).unwrap();
    engine.finish_file_import(up, None).await
}

#[tokio::test]
async fn the_reader_is_listed_from_its_manifest() {
    let f = Fixture::new();
    let engine = f.engine();
    let readers = serde_json::to_value(engine.file_readers()).unwrap();
    assert_eq!(
        readers,
        serde_json::json!([{
            "id": "fake/reader#export",
            "name": "个人数据下载",
            "plugin": "例子",
            "reads": "例子设置里「下载你的数据」给的 JSON",
            "patterns": ["*.json"],
            "lists": ["喜欢"],
            "note": "包里有没有收藏没核"
        }])
    );
}

#[tokio::test]
async fn a_file_is_read_previewed_and_taken_through_ref_and_link() {
    let f = Fixture::new();
    let engine = f.engine();
    let v = upload(&engine, "../../我的导出.json", "hello export file")
        .await
        .unwrap();
    assert_eq!(v.kind, ImportKind::File);
    assert_eq!(
        v.state,
        ImportState::Ready,
        "第一页在交的时候就列了，只有一页"
    );
    let file = v.file.as_ref().unwrap();
    assert_eq!(
        (file.name.as_str(), file.reader.as_str()),
        ("我的导出.json", "例子")
    );
    assert_eq!(v.source.as_deref(), Some("file:我的导出.json"));
    let copy = f.copies().join(&v.id).join("input.json");
    assert_eq!(std::fs::read_to_string(&copy).unwrap(), "hello export file");
    assert_eq!(
        f.cache("inputs").trim(),
        copy.to_str().unwrap(),
        "插件读的是副本"
    );

    let entries = engine.import_entries(&v.id, None, 0, 50).unwrap();
    assert_eq!(entries.len(), 3, "同一个 ref 只记一次: {entries:?}");
    assert!(entries[0].url.is_none(), "只有 ref 的没有链接");
    assert!(entries.iter().all(|e| e.standing == "new"));

    // 几天以后才确认：重新打开以后副本还在
    drop(engine);
    let engine = f.engine();
    assert!(copy.is_file(), "打开库时不清掉还在等确认的副本");

    engine
        .confirm_import(
            &v.id,
            &ImportChoice {
                new: true,
                ..ImportChoice::default()
            },
        )
        .unwrap();
    // 读文件的两条不守平台的节奏（每天一次、间隔一小时）：链接那条用掉今天的一次，它们照样取
    assert_eq!(engine.run_due().await.unwrap(), 3);
    let fetched = f.cache("fetched");
    let copy_s = copy.to_str().unwrap();
    assert!(
        fetched.contains(&format!("chat/1 export {copy_s}")),
        "只有 ref 的回到读文件的方法取，带着副本: {fetched}"
    );
    assert!(
        fetched.contains(&format!("post/p export {copy_s}")),
        "{fetched}"
    );

    let entries = engine.import_entries(&v.id, None, 0, 50).unwrap();
    let chat = entries
        .iter()
        .find(|e| e.title.as_deref() == Some("聊天 1"))
        .unwrap();
    let chat_id = chat.item_id.clone().unwrap();
    assert!(chat_id.starts_with("file-"), "{chat_id}");
    let m = f.manifest(&chat_id);
    assert_eq!(m["origins"][0]["entry"], "import");
    assert_eq!(m["origins"][0]["importBatch"], v.id);
    let snap = &m["snapshots"][0];
    assert_eq!(snap["plugin"], "fake/reader");
    assert!(snap.get("inputUrl").is_none_or(|u| u.is_null()), "{snap}");
    let hits = engine.search("从文件里读的", 10).unwrap();
    assert!(hits.len() >= 2, "{hits:?}");

    let p = entries
        .iter()
        .find(|e| e.title.as_deref() == Some("P"))
        .unwrap();
    assert!(p.item_id.as_deref().unwrap().starts_with("example-"));

    // 链接那条走平台的方法，取到了；都做完以后副本去掉
    assert!(
        fetched.contains("https://example.test/u page none"),
        "{fetched}"
    );
    let done = engine.import(&v.id).unwrap();
    assert_eq!(done.state, ImportState::Done, "{done:?}");
    assert_eq!(done.taken, 3);
    assert!(!copy.exists(), "收完了副本去掉");
}

#[tokio::test]
async fn an_unreadable_file_is_refused_and_leaves_nothing() {
    let f = Fixture::new();
    let engine = f.engine();
    let e = upload(&engine, "x.json", "broken").await.unwrap_err();
    assert_eq!(e.code(), "invalid-args");
    assert_eq!(
        e.hint(),
        Some(usuyo_core::Said::plain("这不是例子的导出文件"))
    );
    assert!(engine.imports().unwrap().is_empty());
    assert_eq!(dir_entries(&f.copies()), 0);
}

#[tokio::test]
async fn an_upload_dropped_halfway_leaves_nothing() {
    let f = Fixture::new();
    let engine = f.engine();
    let up = engine
        .prepare_file_import("fake/reader#export", "a.json")
        .unwrap();
    std::fs::write(up.path(), "half").unwrap();
    drop(up);
    assert_eq!(dir_entries(&f.copies()), 0);
    // 不读文件的方法、不认识的插件
    let e = engine
        .prepare_file_import("fake/reader#page", "a.json")
        .unwrap_err();
    assert_eq!(e.code(), "unsupported-input");
    assert!(e.hint().is_some());
    assert_eq!(
        engine
            .prepare_file_import("nobody/here#x", "a.json")
            .unwrap_err()
            .code(),
        "not-found"
    );
    assert_eq!(
        engine
            .prepare_file_import("fake/reader#export", "a.json")
            .map(|_| ())
            .map_err(|e| e.code()),
        Ok(())
    );
}

#[tokio::test]
async fn hostile_names_never_reach_the_path() {
    let f = Fixture::new();
    let engine = f.engine();
    let long = "x".repeat(10_000);
    for raw in [
        "..",
        "../../x",
        "a/b",
        "C:\\x",
        "CON",
        "a:b|c.zip",
        "a\0b.json",
        "a\nb\r",
        "",
        &long,
    ] {
        let v = upload(&engine, raw, "fine").await.unwrap();
        let dir = f.copies().join(&v.id);
        let files: Vec<String> = std::fs::read_dir(&dir)
            .unwrap()
            .map(|e| e.unwrap().file_name().into_string().unwrap())
            .collect();
        assert_eq!(files.len(), 1, "{raw:?}: {files:?}");
        assert!(
            files[0] == "input" || files[0].starts_with("input."),
            "{raw:?}: {files:?}"
        );
        let shown = &v.file.as_ref().unwrap().name;
        assert!(
            !shown.is_empty()
                && shown.len() <= 200
                && !shown.contains(['/', '\\', '\0', '\n', '\r']),
            "{raw:?} shows {shown:?}"
        );
    }
    assert_eq!(dir_entries(&f.copies()), 10, "一次交一个文件夹");
}

#[tokio::test]
async fn reading_back_waits_while_the_reader_is_held_and_never_touches_stored_items() {
    let f = Fixture::new();
    let engine = f.engine();
    // P 已经在库里：清单里说是它的那一条算「已经存着」，不收，也不碰它
    let p = engine
        .take("https://example.test/p", "link", usuyo_core::Via::Cli)
        .await
        .unwrap();
    engine.run_due().await.unwrap();
    let before = f.manifest(&p.id);
    let v = upload(&engine, "a.json", "hello").await.unwrap();
    let stored = engine.import_entries(&v.id, Some("stored"), 0, 50).unwrap();
    assert_eq!(stored.len(), 1);
    assert_eq!(stored[0].item_id.as_deref(), Some(p.id.as_str()));

    engine
        .confirm_import(
            &v.id,
            &ImportChoice {
                new: true,
                ..ImportChoice::default()
            },
        )
        .unwrap();
    engine.hold_plugin("fake/reader").unwrap();
    engine.run_due().await.unwrap();
    assert!(
        !f.cache("fetched").contains("chat/1"),
        "先别试的时候不回去读：{}",
        f.cache("fetched")
    );
    assert_eq!(engine.import(&v.id).unwrap().state, ImportState::Taking);

    engine.try_plugin_now("fake/reader").unwrap();
    engine.run_due().await.unwrap();
    assert!(f.cache("fetched").contains("chat/1"));
    assert_eq!(
        f.manifest(&p.id)["snapshots"],
        before["snapshots"],
        "存着的没动"
    );
}
