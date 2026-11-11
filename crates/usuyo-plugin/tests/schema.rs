//! 提交的 JSON Schema 和类型一致；固定样例按类型读得进来。
//! 改了类型以后用 `USUYO_BLESS=1 cargo test -p usuyo-plugin --test schema` 重写 schema 文件。

use serde_json::Value;
use usuyo_plugin::protocol::{Canonicalized, FetchOutput, ListPage, Media, schemas};

fn dir(name: &str) -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(name)
}

#[test]
fn committed_schemas_match_the_types() {
    let bless = std::env::var_os("USUYO_BLESS").is_some();
    for (name, schema) in schemas() {
        let path = dir("schema").join(name);
        let generated = format!("{}\n", serde_json::to_string_pretty(&schema).unwrap());
        if bless {
            std::fs::write(&path, &generated).unwrap();
            continue;
        }
        let committed = std::fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("{}: {e}; run with USUYO_BLESS=1", path.display()));
        assert_eq!(
            committed, generated,
            "{name} is stale; run with USUYO_BLESS=1"
        );
    }
}

/// 去掉信封的 `ok`，按类型读。
fn body<T: serde::de::DeserializeOwned>(name: &str) -> T {
    let text = std::fs::read_to_string(dir("testdata").join(name)).unwrap();
    let mut value: Value = serde_json::from_str(&text).unwrap();
    assert_eq!(value["ok"], Value::Bool(true));
    value.as_object_mut().unwrap().remove("ok");
    serde_json::from_value(value).unwrap_or_else(|e| panic!("{name}: {e}"))
}

#[test]
fn fixtures_read_as_types() {
    let full: FetchOutput = body("fetch-output.full.json");
    assert_eq!(full.media.len(), 2);
    assert_eq!(full.comments.as_ref().unwrap().as_str(), "comments.json");
    assert_eq!(
        full.published_at.as_ref().unwrap().as_str(),
        "2026-10-01T20:00:00+08:00"
    );
    let minimal: FetchOutput = body("fetch-output.minimal.json");
    assert!(minimal.html.is_none());
    let page: ListPage = body("list-output.json");
    assert_eq!(page.items.len(), 2);
    assert_eq!(page.next_cursor.as_deref(), Some("eyJwbiI6M30"));
}

#[test]
fn canonicalize_shapes() {
    let m: Canonicalized = serde_json::from_str(
        r#"{"match": true, "identity": {"platform": "douyin", "contentId": "1"}, "resolvedUrl": "https://www.iesdouyin.com/share/video/1/?a=b"}"#,
    )
    .unwrap();
    assert!(matches!(
        m,
        Canonicalized::Match {
            resolved_url: Some(_),
            ..
        }
    ));
    let none: Canonicalized = serde_json::from_str(r#"{"match": false}"#).unwrap();
    assert_eq!(none, Canonicalized::NoMatch);
    for bad in [
        r#"{"match": true}"#,
        r#"{"match": false, "identity": {"platform": "web", "url": "https://a.b/"}}"#,
        r#"{"match": true, "identity": {"platform": "file"}}"#,
    ] {
        assert!(serde_json::from_str::<Canonicalized>(bad).is_err(), "{bad}");
    }
}

#[test]
fn media_group_names_one_timeline() {
    let video: Media = serde_json::from_str(
        r#"{"path": "media/video.m4s", "kind": "video", "group": "cid:1176840"}"#,
    )
    .unwrap();
    assert_eq!(video.group.as_ref().unwrap().as_str(), "cid:1176840");
    let none: Media = serde_json::from_str(r#"{"path": "media/a.m4s", "kind": "audio"}"#).unwrap();
    assert!(none.group.is_none());
    assert!(!serde_json::to_string(&none).unwrap().contains("group"));
    for bad in ["", "a b", "line\\n", &"x".repeat(65), "组"] {
        let text = format!(
            r#"{{"path": "media/a.m4s", "kind": "audio", "group": {}}}"#,
            serde_json::to_string(bad).unwrap()
        );
        assert!(serde_json::from_str::<Media>(&text).is_err(), "{bad:?}");
    }
}
