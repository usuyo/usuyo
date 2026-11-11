//! 资料库的整条读写：建库、收下、op 做了再撤销、回收站、清干净。全在临时目录里，不联网。

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use serde_json::{Value, json};
use usuyo_contract::{
    ChangeOp, Changeset, DecisionState, Extra, FetchRecord, Hunk, Identity, MediaFile, Op,
    OpTarget, ProposalState, SCHEMAS, SetChange,
};
use usuyo_library::{
    AddOutcome, Batch, Capture, ItemId, Library, NewOrigin, NewSnapshot, citations,
};

struct Env {
    _dir: tempfile::TempDir,
    root: PathBuf,
    machine: PathBuf,
}

fn env() -> Env {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().join("lib");
    let machine = dir.path().join("machine");
    fs::create_dir_all(&root).unwrap();
    Env {
        _dir: dir,
        root,
        machine,
    }
}

fn user() -> Batch {
    Batch::new("user", "app").unwrap()
}

fn origin(entry: &str) -> NewOrigin {
    NewOrigin {
        entry: entry.into(),
        taken_by: "user".into(),
        ..NewOrigin::default()
    }
}

fn web(url: &str) -> Identity {
    Identity {
        platform: "web".into(),
        content_id: None,
        url: Some(url.into()),
        extra: Extra::new(),
    }
}

/// 假装插件交回了一份：正文、两张图、一段视频。
fn snapshot(lib: &Library, url: &str, text: &str) -> NewSnapshot {
    let staged = lib.staging_dir().unwrap();
    fs::create_dir_all(staged.join("images")).unwrap();
    fs::create_dir_all(staged.join("media")).unwrap();
    fs::write(staged.join("page.html"), format!("<p>{text}</p>")).unwrap();
    fs::write(staged.join("images/001.jpg"), b"jpg1").unwrap();
    fs::write(staged.join("media/video.mp4"), text.as_bytes()).unwrap();
    let mut extra = Extra::new();
    extra.insert("html".into(), json!("page.html"));
    extra.insert("credentialsUpdate".into(), json!({"cookie": "secret"}));
    NewSnapshot {
        plugin: "web".into(),
        plugin_version: Some("0.1.0".into()),
        input_url: Some(url.into()),
        resolved_url: Some(url.into()),
        access: "public-web".into(),
        site: None,
        upstream: None,
        want: vec![
            "text".into(),
            "html".into(),
            "images".into(),
            "media:video".into(),
            "meta".into(),
        ],
        missing_reason: None,
        text_origin: Some("platform".into()),
        files: usuyo_library::hash_staged(&staged).unwrap(),
        staged,
        fetch: FetchRecord {
            identity: Some(web(url)),
            kind: Some("article".into()),
            title: Some("手冲咖啡：三段式注水".into()),
            author: Some(usuyo_contract::Author {
                name: "咖啡日常".into(),
                platform_id: None,
                url: None,
                extra: Extra::new(),
            }),
            published_at: Some("2026-10-01T04:00:00.000Z".into()),
            published_at_original: Some("2026-10-01T12:00:00+08:00".into()),
            text: Some(text.into()),
            text_format: None,
            images: vec!["images/001.jpg".into()],
            media: vec![MediaFile {
                path: "media/video.mp4".into(),
                kind: "video".into(),
                extra: Extra::new(),
            }],
            files: Vec::new(),
            extra,
        },
    }
}

fn capture(lib: &Library, url: &str, text: &str) -> Capture {
    Capture {
        identity: Some(web(url)),
        title_hint: None,
        origin: origin("paste"),
        snapshot: Some(snapshot(lib, url, text)),
        skip_if_private: false,
    }
}

/// 资料库里你看得见的部分（不含改动记录、回收站、本机文件），逐字节。
fn visible(root: &Path) -> BTreeMap<String, Vec<u8>> {
    let mut out = BTreeMap::new();
    walk(root, root, &mut out);
    out.retain(|k, _| {
        !k.starts_with(".usuyo/journal")
            && !k.starts_with(".usuyo/trash")
            && !k.starts_with(".usuyo/devices")
            && !k.starts_with(".usuyo/tmp")
            && k != ".usuyo/lock"
    });
    out
}

fn assert_same(now: &BTreeMap<String, Vec<u8>>, then: &BTreeMap<String, Vec<u8>>) {
    let mut diff = Vec::new();
    for k in now.keys().chain(then.keys()) {
        if now.get(k) != then.get(k) {
            let show = |m: &BTreeMap<String, Vec<u8>>| {
                m.get(k).map(|b| String::from_utf8_lossy(b).into_owned())
            };
            diff.push(format!(
                "{k}\n--- now\n{:?}\n--- before\n{:?}",
                show(now),
                show(then)
            ));
        }
    }
    diff.dedup();
    assert!(diff.is_empty(), "{}", diff.join("\n"));
}

fn walk(root: &Path, dir: &Path, out: &mut BTreeMap<String, Vec<u8>>) {
    for e in fs::read_dir(dir).unwrap() {
        let p = e.unwrap().path();
        if p.is_dir() {
            walk(root, &p, out);
        } else {
            let rel = p
                .strip_prefix(root)
                .unwrap()
                .to_string_lossy()
                .replace('\\', "/");
            out.insert(rel, fs::read(&p).unwrap());
        }
    }
}

fn all_text(root: &Path) -> String {
    let mut out = BTreeMap::new();
    walk(root, root, &mut out);
    out.into_iter()
        .map(|(k, v)| format!("== {k}\n{}\n", String::from_utf8_lossy(&v)))
        .collect()
}

fn validator(name: &str) -> jsonschema::Validator {
    let (_, text) = SCHEMAS.iter().find(|(n, _)| *n == name).unwrap();
    jsonschema::options()
        .should_validate_formats(true)
        .build(&serde_json::from_str(text).unwrap())
        .unwrap()
}

fn assert_valid(name: &str, v: &Value) {
    let errors: Vec<String> = validator(name)
        .iter_errors(v)
        .map(|e| e.to_string())
        .collect();
    assert!(errors.is_empty(), "{name}: {errors:?}\n{v:#}");
}

fn item_target(id: &ItemId) -> OpTarget {
    OpTarget {
        kind: "item".into(),
        id: id.to_string(),
    }
}

#[test]
fn init_builds_the_tree_and_locks() {
    let e = env();
    let lib = Library::init(&e.root, &e.machine).unwrap();
    for d in [
        "items",
        "topics",
        "cards",
        "drafts",
        "canvas",
        ".usuyo/journal",
        ".usuyo/trash",
        ".usuyo/tmp",
        ".usuyo/changes",
    ] {
        assert!(e.root.join(d).is_dir(), "{d}");
    }
    let lj: Value =
        serde_json::from_slice(&fs::read(e.root.join(".usuyo/library.json")).unwrap()).unwrap();
    assert_eq!(lj["formatVersion"], 1);
    assert_eq!(lj["sync"], Value::Null);
    assert!(usuyo_library::is_uuid7(lib.library_id()));
    assert!(lib.machine_dir().join("device.json").is_file());
    assert_eq!(lib.settings().trash_days, Some(30));
    assert!(matches!(
        Library::open(&e.root, &e.machine),
        Err(usuyo_library::Error::Busy)
    ));
    assert!(Library::init(&e.root, &e.machine).is_err());
    let device = lib.device_id().to_owned();
    drop(lib);
    let again = Library::open(&e.root, &e.machine).unwrap();
    assert_eq!(again.device_id(), device);
    assert!(again.previous_path().is_none());
}

#[test]
fn newer_library_opens_read_only() {
    let e = env();
    drop(Library::init(&e.root, &e.machine).unwrap());
    let p = e.root.join(".usuyo/library.json");
    let mut lj: Value = serde_json::from_slice(&fs::read(&p).unwrap()).unwrap();
    lj["formatVersion"] = json!(2);
    fs::write(&p, serde_json::to_vec(&lj).unwrap()).unwrap();
    let mut lib = Library::open(&e.root, &e.machine).unwrap();
    assert_eq!(lib.read_only(), Some("too-new"));
    let err = lib.apply(
        &user(),
        &OpTarget {
            kind: "library".into(),
            id: "settings".into(),
        },
        Op::SetSetting {
            key: "x".into(),
            value: json!(1),
        },
    );
    assert_eq!(err.unwrap_err().code(), "library-version");
}

#[test]
fn add_item_builds_folder_and_second_paste_adds_origin() {
    let e = env();
    let mut lib = Library::init(&e.root, &e.machine).unwrap();
    let url = "https://Example.com:443/post/1#top";
    let out = lib
        .add_item(&user(), capture(&lib, url, "第一段 40 克闷蒸 30 秒"))
        .unwrap();
    let AddOutcome::Created(id) = out else {
        panic!("{out:?}")
    };
    assert!(id.as_str().starts_with("web-"));
    let dir = e.root.join(id.rel_dir());
    let item = lib.item(&id).unwrap();
    let m = &item.manifest;
    let sid = m.primary_snapshot.clone().unwrap();
    assert!(
        dir.join("raw")
            .join(&sid)
            .join("usuyo.fetch.json")
            .is_file()
    );
    assert!(dir.join("raw").join(&sid).join("images/001.jpg").is_file());
    let rec = lib.fetch_record(&id, &sid).unwrap();
    assert!(!rec.extra.contains_key("credentialsUpdate"));
    assert_eq!(rec.files.len(), 3);
    assert_eq!(
        m.identities[0].url.as_deref(),
        Some("https://example.com/post/1")
    );
    assert_eq!(m.title, "手冲咖啡：三段式注水");
    assert_eq!(m.kind, "article");
    assert_eq!(
        m.outputs,
        vec!["html", "images", "media:video", "meta", "text"]
    );
    assert!(m.missing.is_empty());
    assert_eq!(m.completeness, usuyo_contract::Completeness::Full);
    let content = fs::read_to_string(dir.join("content.md")).unwrap();
    assert_eq!(
        content,
        "# 手冲咖啡：三段式注水\n\n咖啡日常 · 2026-10-01 · web · article\n<https://Example.com:443/post/1#top>\n\n---\n\n第一段 40 克闷蒸 30 秒\n"
    );
    assert!(
        fs::metadata(dir.join("content.md"))
            .unwrap()
            .permissions()
            .readonly()
    );
    assert_valid(
        "manifest",
        &serde_json::from_slice(&fs::read(dir.join("manifest.json")).unwrap()).unwrap(),
    );

    // 同一条链接再收一次：不新建，只多一项 origins；文件一样的快照不新建。
    let out = lib
        .add_item(
            &user(),
            capture(&lib, "https://example.com/post/1", "第一段 40 克闷蒸 30 秒"),
        )
        .unwrap();
    assert_eq!(
        out,
        AddOutcome::Existing {
            id: id.clone(),
            restored: false
        }
    );
    let m = lib.item(&id).unwrap().manifest;
    assert_eq!(m.origins.len(), 2);
    assert_eq!(m.snapshots.len(), 1);
    assert_eq!(lib.item_ids().unwrap(), vec![id.clone()]);
    for entry in lib.journal_entries().unwrap() {
        assert_valid("journal-entry", &serde_json::to_value(&entry).unwrap());
    }
    // tmp 里没有留下东西
    assert_eq!(fs::read_dir(e.root.join(".usuyo/tmp")).unwrap().count(), 0);
}

#[test]
fn link_only_and_file_items() {
    let e = env();
    let mut lib = Library::init(&e.root, &e.machine).unwrap();
    let out = lib
        .add_item(
            &user(),
            Capture {
                identity: Some(web("https://example.com/post/1")),
                title_hint: None,
                origin: origin("cli"),
                snapshot: None,
                skip_if_private: false,
            },
        )
        .unwrap();
    let m = lib.item(out.id()).unwrap().manifest;
    assert_eq!(m.completeness, usuyo_contract::Completeness::Link);
    assert_eq!(m.title, "https://example.com/post/1");
    assert_eq!(m.primary_snapshot, None);
    // 和 contract 的 minimal 样例是同一条
    assert_eq!(out.id().as_str(), "web-bw4o2ajgw2apo2ikx3m6p2flxi");
    let f = lib
        .add_item(
            &user(),
            Capture {
                identity: None,
                title_hint: Some("聊天记录".into()),
                origin: origin("drop"),
                snapshot: None,
                skip_if_private: false,
            },
        )
        .unwrap();
    assert_eq!(f.id().platform(), "file");
    assert!(e.root.join("items/file").join(f.id().as_str()).is_dir());
}

#[test]
fn ops_apply_then_undo_restore_identical_bytes() {
    let e = env();
    let mut lib = Library::init(&e.root, &e.machine).unwrap();
    let id = lib
        .add_item(&user(), capture(&lib, "https://example.com/a", "正文"))
        .unwrap()
        .id()
        .clone();
    let sid = lib.item(&id).unwrap().manifest.primary_snapshot.unwrap();
    let other = lib
        .add_item(&user(), capture(&lib, "https://example.com/b", "另一条"))
        .unwrap()
        .id()
        .clone();
    // 先有一份主题和笔记，撤销要回到这里
    let setup = user();
    let topic = lib
        .apply(
            &setup,
            &OpTarget {
                kind: "topic".into(),
                id: String::new(),
            },
            Op::CreateTopic {
                title: "手冲咖啡".into(),
                keywords: vec!["手冲".into()],
                match_: vec!["keywords".into()],
            },
        )
        .unwrap()
        .id;
    lib.apply(
        &setup,
        &OpTarget {
            kind: "topic".into(),
            id: topic.clone(),
        },
        Op::EditTopicPage {
            hunks: vec![Hunk {
                id: "h1".into(),
                before: String::new(),
                after: "第一段。\n\n第二段。".into(),
                anchor: None,
            }],
        },
    )
    .unwrap();
    let card = lib
        .apply(
            &setup,
            &OpTarget {
                kind: "card".into(),
                id: String::new(),
            },
            Op::CreateCard {
                title: String::new(),
                body: "闷蒸比水温要紧，这是一个观点\n".into(),
            },
        )
        .unwrap()
        .id;
    let draft = lib
        .apply(
            &setup,
            &OpTarget {
                kind: "draft".into(),
                id: String::new(),
            },
            Op::CreateDraft {
                title: "三段式".into(),
                body: "开头\n".into(),
                kind: "post".into(),
            },
        )
        .unwrap()
        .id;
    let snap = visible(&e.root);

    let b = user();
    let it = item_target(&id);
    let tt = OpTarget {
        kind: "topic".into(),
        id: topic.clone(),
    };
    let ct = OpTarget {
        kind: "card".into(),
        id: card.clone(),
    };
    let dt = OpTarget {
        kind: "draft".into(),
        id: draft.clone(),
    };
    let ops: Vec<(OpTarget, Op)> = vec![
        (
            it.clone(),
            Op::SetTags(SetChange {
                add: vec!["咖啡".into()],
                remove: vec![],
            }),
        ),
        (
            it.clone(),
            Op::SetTopics(SetChange {
                add: vec![topic.clone()],
                remove: vec![],
            }),
        ),
        (
            it.clone(),
            Op::SetNotSame(SetChange {
                add: vec![other.to_string()],
                remove: vec![],
            }),
        ),
        (
            it.clone(),
            Op::DismissSuggestion {
                tag: "饮品".into()
            },
        ),
        (it.clone(), Op::SetPrivate { value: true }),
        (
            it.clone(),
            Op::SetTakeaway {
                text: Some("闷蒸 30 秒比水温要紧".into()),
            },
        ),
        (
            it.clone(),
            Op::EditNotes {
                hunks: vec![Hunk {
                    id: "n1".into(),
                    before: String::new(),
                    after: "你的笔记，随便写。".into(),
                    anchor: None,
                }],
            },
        ),
        (
            it.clone(),
            Op::AddHighlight {
                id: "h7k2m".into(),
                layer: "asr".into(),
                snapshot: Some(sid.clone()),
                fragment: Some("t=42,48".into()),
                quote: "第一段 40 克".into(),
            },
        ),
        (
            it.clone(),
            Op::AddHighlight {
                id: "h7k2n".into(),
                layer: "text".into(),
                snapshot: None,
                fragment: None,
                quote: "正文".into(),
            },
        ),
        (it.clone(), Op::RemoveHighlight { id: "h7k2m".into() }),
        (tt.clone(), Op::SetFollowed { value: true }),
        (
            tt.clone(),
            Op::EditTopic {
                title: Some("手冲".into()),
                keywords: Some(vec!["闷蒸".into()]),
                match_: None,
            },
        ),
        (
            tt.clone(),
            Op::EditTopicPage {
                hunks: vec![Hunk {
                    id: "h2".into(),
                    before: "第二段。".into(),
                    after: "第二段改了。".into(),
                    anchor: None,
                }],
            },
        ),
        (
            tt.clone(),
            Op::AddCitation {
                anchor: Some("第一段。".into()),
                item: id.to_string(),
                layer: "asr".into(),
                snapshot: Some(sid.clone()),
                fragment: Some("t=42,48".into()),
                quote: "第一段 40 克闷蒸 30 秒".into(),
            },
        ),
        (
            tt.clone(),
            Op::AddCitation {
                anchor: None,
                item: id.to_string(),
                layer: "text".into(),
                snapshot: None,
                fragment: None,
                quote: "正文".into(),
            },
        ),
        (
            tt.clone(),
            Op::Move {
                path: "咖啡".into(),
            },
        ),
        (
            ct.clone(),
            Op::SetTopics(SetChange {
                add: vec![topic.clone()],
                remove: vec![],
            }),
        ),
        (
            ct.clone(),
            Op::EditCard {
                hunks: vec![],
                title: Some("新标题".into()),
            },
        ),
        (
            dt.clone(),
            Op::EditDraft {
                hunks: vec![Hunk {
                    id: "d".into(),
                    before: "开头".into(),
                    after: "开头改了".into(),
                    anchor: None,
                }],
                title: None,
                kind: Some("article".into()),
                from: SetChange {
                    add: vec![topic.clone()],
                    remove: vec![],
                },
            },
        ),
        (
            OpTarget {
                kind: "library".into(),
                id: "settings".into(),
            },
            Op::SetSetting {
                key: "trashDays".into(),
                value: json!(90),
            },
        ),
        (
            OpTarget {
                kind: "topic".into(),
                id: String::new(),
            },
            Op::CreateTopic {
                title: "另一个".into(),
                keywords: vec![],
                match_: vec!["keywords".into()],
            },
        ),
    ];
    for (t, op) in ops {
        lib.apply(&b, &t, op).unwrap();
    }
    assert_ne!(visible(&e.root), snap);
    let w = lib.find::<usuyo_contract::TopicMeta>(&topic).unwrap();
    assert!(w.path.starts_with("topics/咖啡/手冲.md"), "{}", w.path);
    let cites = citations(&w.doc.body);
    assert_eq!(cites.len(), 2);
    assert_eq!(cites[0].column, "口播 00:42");
    assert_eq!(cites[0].author.as_deref(), Some("咖啡日常"));
    assert_eq!(cites[0].date.as_deref(), Some("2026-10-01"));
    assert!(
        w.doc
            .body
            .contains(&format!("第一段。[^{}]", cites[0].label))
    );
    let (notes, _) = lib.notes(&id).unwrap();
    assert_eq!(notes.meta.takeaway.as_deref(), Some("闷蒸 30 秒比水温要紧"));
    let hl = lib.highlights(&id).unwrap();
    assert_eq!(hl.len(), 1);
    assert_eq!(
        hl[0].link.snapshot.as_deref(),
        Some(sid.as_str()),
        "text carries the primary snapshot"
    );
    assert_eq!(lib.settings().trash_days, Some(90));
    for entry in lib.batch_entries(&b.id).unwrap() {
        assert_valid("journal-entry", &serde_json::to_value(&entry).unwrap());
    }

    let applied = visible(&e.root);
    let undo = user();
    let report = lib.undo(&b.id, &undo).unwrap();
    assert!(report.skipped.is_empty(), "{:?}", report.skipped);
    assert_same(&visible(&e.root), &snap);
    // 再撤一次已经撤过的：全部「已经撤过」
    let again = lib.undo(&b.id, &user()).unwrap();
    assert!(again.undone.is_empty());
    assert_eq!(lib.settings().trash_days, Some(30));
    // 撤销的撤销就是重做：回到做完的样子
    let redo = lib.undo(&undo.id, &user()).unwrap();
    assert!(redo.skipped.is_empty(), "{:?}", redo.skipped);
    assert_same(&visible(&e.root), &applied);
}

#[test]
fn undo_lists_what_changed_since() {
    let e = env();
    let mut lib = Library::init(&e.root, &e.machine).unwrap();
    let id = lib
        .add_item(&user(), capture(&lib, "https://example.com/a", "正文"))
        .unwrap()
        .id()
        .clone();
    let b = user();
    lib.apply(
        &b,
        &item_target(&id),
        Op::SetTakeaway {
            text: Some("一".into()),
        },
    )
    .unwrap();
    lib.apply(
        &user(),
        &item_target(&id),
        Op::SetTakeaway {
            text: Some("二".into()),
        },
    )
    .unwrap();
    let r = lib.undo(&b.id, &user()).unwrap();
    assert_eq!(r.undone.len(), 0);
    assert_eq!(r.skipped[0].1, "changed since");
}

#[test]
fn trash_restore_and_keep_text() {
    let e = env();
    let mut lib = Library::init(&e.root, &e.machine).unwrap();
    let id = lib
        .add_item(&user(), capture(&lib, "https://example.com/a", "正文"))
        .unwrap()
        .id()
        .clone();
    let before = visible(&e.root);
    let b = user();
    lib.apply(
        &b,
        &item_target(&id),
        Op::MoveToTrash {
            reason: "delete".into(),
        },
    )
    .unwrap();
    assert!(!e.root.join(id.rel_dir()).exists());
    assert!(
        !e.root.join("items/web").exists(),
        "empty platform folder is pruned"
    );
    let entries = lib.trash_entries().unwrap();
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].target, id.as_str());
    assert!(entries[0].files.contains(&"manifest.json".to_string()));
    assert!(lib.item(&id).is_err());
    let exp = usuyo_library::expires_at(&entries[0].trashed_at, Some(30))
        .unwrap()
        .unwrap();
    assert!(lib.expired(&exp).unwrap().len() == 1);
    assert!(lib.expired(&entries[0].trashed_at).unwrap().is_empty());
    // 撤销进回收站：逐字节回来
    lib.undo(&b.id, &user()).unwrap();
    assert_same(&visible(&e.root), &before);

    // 只留文字
    let b = user();
    lib.apply(&b, &item_target(&id), Op::KeepText {}).unwrap();
    let m = lib.item(&id).unwrap().manifest;
    assert_eq!(m.state, usuyo_contract::ItemState::TextOnly);
    assert!(!m.outputs.contains(&"images".to_string()));
    assert!(!m.outputs.contains(&"media:video".to_string()));
    assert_eq!(m.text_only.as_ref().unwrap().removed.len(), 2);
    let sid = m.primary_snapshot.clone().unwrap();
    assert!(
        !e.root
            .join(id.rel_dir())
            .join("raw")
            .join(&sid)
            .join("images/001.jpg")
            .exists()
    );
    assert!(
        e.root
            .join(id.rel_dir())
            .join("raw")
            .join(&sid)
            .join("page.html")
            .exists()
    );
    lib.undo(&b.id, &user()).unwrap();
    assert_same(&visible(&e.root), &before);

    // 你点「恢复」：回到 normal，stateAt 记恢复的时间
    let b = user();
    lib.apply(
        &b,
        &item_target(&id),
        Op::MoveToTrash {
            reason: "delete".into(),
        },
    )
    .unwrap();
    let tid = lib.trash_entries().unwrap()[0].id.clone();
    lib.apply(
        &user(),
        &OpTarget {
            kind: "trash".into(),
            id: tid.clone(),
        },
        Op::Restore {},
    )
    .unwrap();
    let m = lib.item(&id).unwrap().manifest;
    assert_eq!(m.state, usuyo_contract::ItemState::Normal);
    assert!(m.state_at.is_some());
    assert!(lib.trash_entries().unwrap().is_empty());
}

#[test]
fn recapture_restores_from_trash_but_subscription_skips() {
    let e = env();
    let mut lib = Library::init(&e.root, &e.machine).unwrap();
    let id = lib
        .add_item(&user(), capture(&lib, "https://example.com/a", "正文"))
        .unwrap()
        .id()
        .clone();
    lib.apply(
        &user(),
        &item_target(&id),
        Op::MoveToTrash {
            reason: "delete".into(),
        },
    )
    .unwrap();
    let mut sub = capture(&lib, "https://example.com/a", "正文");
    sub.origin.entry = "subscription".into();
    assert_eq!(
        lib.add_item(&user(), sub).unwrap(),
        AddOutcome::Skipped(id.clone())
    );
    assert_eq!(fs::read_dir(e.root.join(".usuyo/tmp")).unwrap().count(), 0);
    let b = user();
    let out = lib
        .add_item(&b, capture(&lib, "https://example.com/a", "正文"))
        .unwrap();
    assert_eq!(
        out,
        AddOutcome::Existing {
            id: id.clone(),
            restored: true
        }
    );
    assert_eq!(
        lib.item(&id).unwrap().manifest.state,
        usuyo_contract::ItemState::Normal
    );
    let ops: Vec<String> = lib
        .batch_entries(&b.id)
        .unwrap()
        .into_iter()
        .map(|e| e.op)
        .collect();
    assert_eq!(ops, vec!["restore", "add-item"]);
    // 撤销这批：原样放回回收站
    lib.undo(&b.id, &user()).unwrap();
    assert!(lib.item(&id).is_err());
    assert_eq!(lib.trash_entries().unwrap().len(), 1);
}

#[test]
fn private_items_are_skipped_for_agents() {
    let e = env();
    let mut lib = Library::init(&e.root, &e.machine).unwrap();
    let id = lib
        .add_item(&user(), capture(&lib, "https://example.com/a", "正文"))
        .unwrap()
        .id()
        .clone();
    lib.apply(&user(), &item_target(&id), Op::SetPrivate { value: true })
        .unwrap();
    let agent = Batch::new("agent:claude-desktop", "mcp").unwrap();
    let mut c = capture(&lib, "https://example.com/a", "正文");
    c.skip_if_private = true;
    c.origin.entry = "mcp".into();
    assert_eq!(
        lib.add_item(&agent, c).unwrap(),
        AddOutcome::Skipped(id.clone())
    );
    assert_eq!(lib.item(&id).unwrap().manifest.origins.len(), 1);
    // Agent 撤销它自己收下的那批：新建的进回收站，不算删除
    let id2 = lib
        .add_item(&agent, capture(&lib, "https://example.com/b", "b"))
        .unwrap()
        .id()
        .clone();
    lib.undo(&agent.id, &agent).unwrap();
    assert!(lib.item(&id2).is_err());
    assert_eq!(lib.trash_entries().unwrap()[0].reason, "undo");
    assert!(Batch::new("robot", "app").is_err());
}

#[test]
fn purge_cleans_everything_out() {
    let e = env();
    let mut lib = Library::init(&e.root, &e.machine).unwrap();
    let id = lib
        .add_item(
            &user(),
            capture(&lib, "https://example.com/gone", "要删掉的"),
        )
        .unwrap()
        .id()
        .clone();
    let sid = lib.item(&id).unwrap().manifest.primary_snapshot.unwrap();
    let keep = lib
        .add_item(&user(), capture(&lib, "https://example.com/keep", "留着的"))
        .unwrap()
        .id()
        .clone();
    let b = user();
    let topic = lib
        .apply(
            &b,
            &OpTarget {
                kind: "topic".into(),
                id: String::new(),
            },
            Op::CreateTopic {
                title: "主题".into(),
                keywords: vec![],
                match_: vec!["keywords".into()],
            },
        )
        .unwrap()
        .id;
    let tt = OpTarget {
        kind: "topic".into(),
        id: topic.clone(),
    };
    lib.apply(
        &b,
        &tt,
        Op::EditTopicPage {
            hunks: vec![Hunk {
                id: "h".into(),
                before: String::new(),
                after: "我的一段话。".into(),
                anchor: None,
            }],
        },
    )
    .unwrap();
    lib.apply(
        &b,
        &tt,
        Op::AddCitation {
            anchor: Some("我的一段话。".into()),
            item: id.to_string(),
            layer: "asr".into(),
            snapshot: Some(sid.clone()),
            fragment: Some("t=1".into()),
            quote: "原话留着".into(),
        },
    )
    .unwrap();
    lib.apply(
        &b,
        &item_target(&id),
        Op::SetTags(SetChange {
            add: vec!["x".into()],
            remove: vec![],
        }),
    )
    .unwrap();
    lib.apply(
        &b,
        &item_target(&keep),
        Op::SetNotSame(SetChange {
            add: vec![id.to_string()],
            remove: vec![],
        }),
    )
    .unwrap();
    lib.apply(
        &b,
        &item_target(&keep),
        Op::EditNotes {
            hunks: vec![Hunk {
                id: "n".into(),
                before: String::new(),
                after: format!("见 [那条](usuyo://item/{id}/text)"),
                anchor: None,
            }],
        },
    )
    .unwrap();
    lib.record_opened(&id).unwrap();
    lib.record_opened(&keep).unwrap();
    let mut proposal: Changeset = serde_json::from_value(json!({
        "id": usuyo_library::uuid7(), "createdAt": usuyo_library::time::now(), "proposer": "agent:claude-desktop",
        "state": "proposed", "ops": []
    }))
    .unwrap();
    for target in [&id, &keep] {
        proposal.ops.push(ChangeOp {
            id: usuyo_library::uuid7(),
            op: "set-tags".into(),
            target: item_target(target),
            base: None,
            args: json!({"add": ["y"], "remove": []}),
            state: DecisionState::Proposed,
            edited: false,
            extra: Extra::new(),
        });
    }
    proposal.ops.push(ChangeOp {
        id: usuyo_library::uuid7(),
        op: "add-citation".into(),
        target: OpTarget {
            kind: "topic".into(),
            id: topic.clone(),
        },
        base: None,
        args: json!({"anchor": null, "item": id.to_string(), "layer": "text", "quote": "q"}),
        state: DecisionState::Proposed,
        edited: false,
        extra: Extra::new(),
    });
    assert_eq!(proposal.state, ProposalState::Proposed);
    lib.save_proposal(&proposal).unwrap();

    lib.apply(
        &user(),
        &item_target(&id),
        Op::MoveToTrash {
            reason: "delete".into(),
        },
    )
    .unwrap();
    let tid = lib.trash_entries().unwrap()[0].id.clone();
    let applied = lib
        .apply(
            &user(),
            &OpTarget {
                kind: "trash".into(),
                id: tid,
            },
            Op::Purge {},
        )
        .unwrap();
    let report = applied.purged.unwrap();
    assert_eq!(report.forgotten, vec![id.to_string()]);
    assert_eq!(report.rewritten.len(), 2);

    // 库里任何文件都不再出现它的 id
    let text = all_text(&e.root);
    assert!(!text.contains(id.as_str()), "{text}");
    // 原话、标题留着，链接换成 forgotten
    let w = lib.find::<usuyo_contract::TopicMeta>(&topic).unwrap();
    let c = &citations(&w.doc.body)[0];
    assert!(c.forgotten());
    assert_eq!(c.quote, "原话留着");
    assert_eq!(c.title, "手冲咖啡：三段式注水");
    assert!(lib.item(&keep).unwrap().manifest.not_same_as.is_empty());
    assert_eq!(lib.proposals().unwrap()[0].ops.len(), 1);
    assert!(
        lib.journal_entries()
            .unwrap()
            .iter()
            .any(|e| e.is_redacted())
    );
    for entry in lib.journal_entries().unwrap() {
        assert_valid("journal-entry", &serde_json::to_value(&entry).unwrap());
    }
    // 撤不了
    let r = lib.undo(&applied.entry.batch, &user()).unwrap();
    assert!(r.undone.is_empty());
}

#[test]
fn merge_items_and_undo() {
    let e = env();
    let mut lib = Library::init(&e.root, &e.machine).unwrap();
    let a = lib
        .add_item(&user(), capture(&lib, "https://example.com/a", "A 的正文"))
        .unwrap()
        .id()
        .clone();
    let b = lib
        .add_item(&user(), capture(&lib, "https://example.com/b", "B 的正文"))
        .unwrap()
        .id()
        .clone();
    let setup = user();
    lib.apply(
        &setup,
        &item_target(&a),
        Op::EditNotes {
            hunks: vec![Hunk {
                id: "1".into(),
                before: String::new(),
                after: "A 的笔记".into(),
                anchor: None,
            }],
        },
    )
    .unwrap();
    lib.apply(
        &setup,
        &item_target(&a),
        Op::AddHighlight {
            id: "aaaaa".into(),
            layer: "text".into(),
            snapshot: None,
            fragment: None,
            quote: "A".into(),
        },
    )
    .unwrap();
    lib.apply(
        &setup,
        &item_target(&b),
        Op::AddHighlight {
            id: "aaaaa".into(),
            layer: "text".into(),
            snapshot: None,
            fragment: None,
            quote: "B".into(),
        },
    )
    .unwrap();
    let before = visible(&e.root);
    let m = user();
    lib.apply(
        &m,
        &item_target(&a),
        Op::MergeItems {
            into: b.to_string(),
        },
    )
    .unwrap();
    assert!(lib.item(&a).is_err());
    let bm = lib.item(&b).unwrap().manifest;
    assert_eq!(bm.snapshots.len(), 2);
    assert_eq!(bm.origins.len(), 2);
    assert!(bm.aliases.contains(&a.to_string()));
    let hl = lib.highlights(&b).unwrap();
    assert_eq!(hl.len(), 2);
    assert_ne!(hl[0].id, hl[1].id);
    assert_eq!(lib.notes(&b).unwrap().0.body, "A 的笔记\n");
    // 用旧 id 收下：并到 B
    let out = lib
        .add_item(
            &user(),
            Capture {
                identity: Some(web("https://example.com/a")),
                title_hint: None,
                origin: origin("paste"),
                snapshot: None,
                skip_if_private: false,
            },
        )
        .unwrap();
    assert_eq!(out.id(), &b);
    lib.undo(&out_batch(&lib, &b), &user()).unwrap();
    let u = user();
    let r = lib.undo(&m.id, &u).unwrap();
    assert!(r.skipped.is_empty(), "{:?}", r.skipped);
    assert_same(&visible(&e.root), &before);
    // 重做：再并一次
    let r = lib.undo(&u.id, &user()).unwrap();
    assert!(r.skipped.is_empty(), "{:?}", r.skipped);
    assert!(lib.item(&a).is_err());
    assert!(
        lib.item(&b)
            .unwrap()
            .manifest
            .aliases
            .contains(&a.to_string())
    );
}

/// 最后一条 add-item 所在的批。
fn out_batch(lib: &Library, id: &ItemId) -> String {
    lib.journal_entries()
        .unwrap()
        .into_iter()
        .rev()
        .find(|e| {
            e.op == "add-item" && e.targets[0].id.value().map(String::as_str) == Some(id.as_str())
        })
        .unwrap()
        .batch
}

#[test]
fn content_md_changed_outside_goes_to_trash() {
    let e = env();
    let mut lib = Library::init(&e.root, &e.machine).unwrap();
    let id = lib
        .add_item(&user(), capture(&lib, "https://example.com/a", "正文"))
        .unwrap()
        .id()
        .clone();
    let path = e.root.join(id.rel_dir()).join("content.md");
    let original = fs::read(&path).unwrap();
    fs::remove_file(&path).unwrap();
    fs::write(&path, "我手改了").unwrap();
    let h = lib
        .write_content(&id, Some(&usuyo_library::hash_tag(&original)))
        .unwrap();
    assert_eq!(h, usuyo_library::hash_tag(&original));
    assert_eq!(fs::read(&path).unwrap(), original);
    let t = lib.trash_entries().unwrap();
    assert_eq!(t[0].kind, "content");
    assert_eq!(
        fs::read_to_string(
            e.root
                .join(".usuyo/trash")
                .join(&t[0].id)
                .join("content.md")
        )
        .unwrap(),
        "我手改了"
    );
}

#[test]
fn same_title_gets_a_suffix_by_id_order() {
    let e = env();
    let mut lib = Library::init(&e.root, &e.machine).unwrap();
    let b = user();
    let first = lib
        .apply(
            &b,
            &OpTarget {
                kind: "topic".into(),
                id: String::new(),
            },
            Op::CreateTopic {
                title: "Coffee".into(),
                keywords: vec![],
                match_: vec![],
            },
        )
        .unwrap();
    let second = lib
        .apply(
            &b,
            &OpTarget {
                kind: "topic".into(),
                id: String::new(),
            },
            Op::CreateTopic {
                title: "coffee".into(),
                keywords: vec![],
                match_: vec![],
            },
        )
        .unwrap();
    assert_eq!(
        lib.find::<usuyo_contract::TopicMeta>(&first.id)
            .unwrap()
            .path,
        "topics/Coffee.md"
    );
    assert_eq!(
        lib.find::<usuyo_contract::TopicMeta>(&second.id)
            .unwrap()
            .path,
        "topics/coffee 2.md"
    );
    // 一个更小的 id 后来：它用本名，原来那份挪走
    let small = "00000000-0000-7000-8000-000000000000".to_string();
    lib.apply(
        &b,
        &OpTarget {
            kind: "topic".into(),
            id: small.clone(),
        },
        Op::CreateTopic {
            title: "COFFEE".into(),
            keywords: vec![],
            match_: vec![],
        },
    )
    .unwrap();
    assert_eq!(
        lib.find::<usuyo_contract::TopicMeta>(&small).unwrap().path,
        "topics/COFFEE.md"
    );
    let paths: Vec<String> = lib
        .scan::<usuyo_contract::TopicMeta>()
        .unwrap()
        .docs
        .into_iter()
        .map(|w| w.path)
        .collect();
    assert_eq!(paths.len(), 3);
    // 只改大小写的改名
    lib.apply(
        &b,
        &OpTarget {
            kind: "topic".into(),
            id: small.clone(),
        },
        Op::EditTopic {
            title: Some("CoFFEE".into()),
            keywords: None,
            match_: None,
        },
    )
    .unwrap();
    assert_eq!(
        lib.find::<usuyo_contract::TopicMeta>(&small).unwrap().path,
        "topics/CoFFEE.md"
    );
}

#[test]
fn proposals_reject_unproposable_ops() {
    let e = env();
    let mut lib = Library::init(&e.root, &e.machine).unwrap();
    let c: Changeset = serde_json::from_value(json!({
        "id": usuyo_library::uuid7(), "createdAt": usuyo_library::time::now(), "proposer": "agent:x",
        "state": "proposed",
        "ops": [{"id": usuyo_library::uuid7(), "op": "move-to-trash", "target": {"kind": "item", "id": "x"}, "base": null, "args": {"reason": "delete"}, "state": "proposed"}]
    }))
    .unwrap();
    assert!(lib.save_proposal(&c).is_err());
}

#[test]
fn undo_skips_targets_that_went_away_and_keeps_going() {
    let e = env();
    let mut lib = Library::init(&e.root, &e.machine).unwrap();
    let a = lib
        .add_item(&user(), capture(&lib, "https://example.com/a", "a"))
        .unwrap()
        .id()
        .clone();
    let c = lib
        .add_item(&user(), capture(&lib, "https://example.com/c", "c"))
        .unwrap()
        .id()
        .clone();
    let b = user();
    lib.apply(
        &b,
        &item_target(&a),
        Op::SetTags(SetChange {
            add: vec!["x".into()],
            remove: vec![],
        }),
    )
    .unwrap();
    lib.apply(
        &b,
        &item_target(&c),
        Op::SetTags(SetChange {
            add: vec!["y".into()],
            remove: vec![],
        }),
    )
    .unwrap();
    lib.apply(
        &user(),
        &item_target(&a),
        Op::MoveToTrash {
            reason: "delete".into(),
        },
    )
    .unwrap();
    let r = lib.undo(&b.id, &user()).unwrap();
    assert_eq!(r.undone.len(), 1);
    assert_eq!(r.skipped.len(), 1);
    assert!(r.skipped[0].1.starts_with("changed since"));
    assert!(lib.item(&c).unwrap().manifest.tags.user.is_empty());
}

#[test]
fn setting_null_means_forever_and_undo_keeps_it() {
    let e = env();
    let mut lib = Library::init(&e.root, &e.machine).unwrap();
    let t = OpTarget {
        kind: "library".into(),
        id: "settings".into(),
    };
    lib.apply(
        &user(),
        &t,
        Op::SetSetting {
            key: "trashDays".into(),
            value: Value::Null,
        },
    )
    .unwrap();
    assert_eq!(lib.settings().trash_days, None);
    let b = user();
    lib.apply(
        &b,
        &t,
        Op::SetSetting {
            key: "trashDays".into(),
            value: json!(90),
        },
    )
    .unwrap();
    lib.undo(&b.id, &user()).unwrap();
    assert_eq!(lib.settings().trash_days, None);
}

#[test]
fn layers_resolve_to_text_or_files() {
    let e = env();
    let mut lib = Library::init(&e.root, &e.machine).unwrap();
    let id = lib
        .add_item(&user(), capture(&lib, "https://example.com/a", "原文的字"))
        .unwrap()
        .id()
        .clone();
    let sid = lib.item(&id).unwrap().manifest.primary_snapshot.unwrap();
    assert_eq!(
        lib.resolve_layer(&id, "text", None).unwrap(),
        usuyo_library::LayerSource::Text("原文的字".into())
    );
    let usuyo_library::LayerSource::File(video) =
        lib.resolve_layer(&id, "video", Some(&sid)).unwrap()
    else {
        panic!()
    };
    assert!(video.ends_with("media/video.mp4"));
    let key = &lib
        .fetch_record(&id, &sid)
        .unwrap()
        .files
        .iter()
        .find(|f| f.path == "images/001.jpg")
        .unwrap()
        .sha256[..16]
        .to_owned();
    assert!(matches!(
        lib.resolve_layer(&id, &format!("image/{key}"), None)
            .unwrap(),
        usuyo_library::LayerSource::File(_)
    ));
    assert_eq!(
        lib.resolve_layer(&id, "asr", None).unwrap_err().code(),
        "not-found"
    );
    assert_eq!(
        lib.resolve_layer(&id, "ocr/../../x", None)
            .unwrap_err()
            .code(),
        "invalid-args"
    );
}

#[test]
fn fetch_record_paths_cannot_leave_the_snapshot_folder() {
    let e = env();
    let mut lib = Library::init(&e.root, &e.machine).unwrap();

    // 插件交来的路径是绝对路径：不收。
    let mut absolute = capture(&lib, "https://example.com/absolute", "字");
    absolute.snapshot.as_mut().unwrap().fetch.media[0].path = "/etc/hosts".into();
    assert_eq!(
        lib.add_item(&user(), absolute).unwrap_err().code(),
        "invalid-args"
    );

    // 输出目录里带符号链接：不收。
    #[cfg(unix)]
    {
        let linked = capture(&lib, "https://example.com/linked", "字");
        let staged = linked.snapshot.as_ref().unwrap().staged.clone();
        std::os::unix::fs::symlink("/etc/hosts", staged.join("media/link.mp4")).unwrap();
        assert_eq!(
            lib.add_item(&user(), linked).unwrap_err().code(),
            "invalid-args"
        );
    }

    // 收下以后，记录在 usuyo 外被改成指到库外：读的时候算写坏了，不给路径。
    let id = lib
        .add_item(&user(), capture(&lib, "https://example.com/a", "原文的字"))
        .unwrap()
        .id()
        .clone();
    let sid = lib.item(&id).unwrap().manifest.primary_snapshot.unwrap();
    let raw = e
        .root
        .join("items/web")
        .join(id.as_str())
        .join("raw")
        .join(&sid);
    let record = raw.join("usuyo.fetch.json");
    let original = fs::read_to_string(&record).unwrap();
    fs::write(&record, original.replace("media/video.mp4", "/etc/hosts")).unwrap();
    assert_eq!(
        lib.resolve_layer(&id, "video", Some(&sid))
            .unwrap_err()
            .code(),
        "internal"
    );
    fs::write(&record, &original).unwrap();

    // 文件本身被换成指到库外的符号链接：当作没有。
    #[cfg(unix)]
    {
        fs::remove_file(raw.join("media/video.mp4")).unwrap();
        std::os::unix::fs::symlink("/etc/hosts", raw.join("media/video.mp4")).unwrap();
        assert_eq!(
            lib.resolve_layer(&id, "video", Some(&sid))
                .unwrap_err()
                .code(),
            "not-found"
        );
    }
}

#[test]
fn opening_clears_call_folders_left_by_a_crash() {
    let e = env();
    drop(Library::init(&e.root, &e.machine).unwrap());
    let tmp = e.root.join(".usuyo/tmp");
    fs::create_dir_all(tmp.join("usuyo-call-0190-x/out")).unwrap();
    fs::write(tmp.join("usuyo-call-0190-x/out/big.bin"), b"x").unwrap();
    let staged = tmp.join(usuyo_library::uuid7());
    fs::create_dir_all(&staged).unwrap();
    fs::write(tmp.join("migrate-1-2.json"), b"{}").unwrap();
    fs::create_dir_all(tmp.join("not-ours")).unwrap();
    let _lib = Library::open(&e.root, &e.machine).unwrap();
    assert!(!tmp.join("usuyo-call-0190-x").exists());
    assert!(!staged.exists());
    assert!(tmp.join("migrate-1-2.json").is_file(), "迁移计划不动");
    assert!(tmp.join("not-ours").is_dir(), "认不出的不动");
}

#[cfg(unix)]
#[test]
fn machine_folders_are_private() {
    use std::os::unix::fs::PermissionsExt;
    let e = env();
    let lib = Library::init(&e.root, &e.machine).unwrap();
    for dir in [e.machine.as_path(), lib.machine_dir()] {
        let mode = fs::metadata(dir).unwrap().permissions().mode() & 0o777;
        assert_eq!(mode, 0o700, "{}", dir.display());
    }
}

#[test]
fn trash_entry_paths_cannot_leave_their_folders() {
    let e = env();
    let mut lib = Library::init(&e.root, &e.machine).unwrap();
    let id = lib
        .add_item(&user(), capture(&lib, "https://example.com/a", "正文"))
        .unwrap()
        .id()
        .clone();
    lib.apply(&user(), &item_target(&id), Op::KeepText {})
        .unwrap();
    let tid = lib.trash_entries().unwrap()[0].id.clone();
    let path = e.root.join(".usuyo/trash").join(&tid).join("entry.json");
    let good: Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    for (field, bad) in [
        ("files", json!(["../../../outside.txt"])),
        ("files", json!(["/etc/hosts"])),
        ("path", json!("items/web/../../outside")),
    ] {
        let mut v = good.clone();
        v[field] = bad;
        fs::write(&path, serde_json::to_vec(&v).unwrap()).unwrap();
        assert_eq!(lib.trash_entry(&tid).unwrap_err().code(), "internal");
        assert!(
            lib.trash_entries().unwrap().is_empty(),
            "写坏的一项不列，别的照常"
        );
    }
}

#[cfg(unix)]
#[test]
fn layer_paths_are_the_files_actually_served() {
    let e = env();
    let mut lib = Library::init(&e.root, &e.machine).unwrap();
    let id = lib
        .add_item(&user(), capture(&lib, "https://example.com/a", "原文的字"))
        .unwrap()
        .id()
        .clone();
    let sid = lib.item(&id).unwrap().manifest.primary_snapshot.unwrap();
    let raw = e.root.join(id.rel_dir()).join("raw").join(&sid);
    fs::remove_file(raw.join("media/video.mp4")).unwrap();
    std::os::unix::fs::symlink(raw.join("page.html"), raw.join("media/video.mp4")).unwrap();
    let usuyo_library::LayerSource::File(p) = lib.resolve_layer(&id, "video", None).unwrap() else {
        panic!()
    };
    assert_eq!(p, raw.join("page.html").canonicalize().unwrap());
}

#[test]
fn plugin_markdown_with_html_or_other_schemes_is_written_as_plain_text() {
    let e = env();
    let mut lib = Library::init(&e.root, &e.machine).unwrap();
    let content = |lib: &mut Library, url: &str, text: &str| {
        let mut c = capture(lib, url, "x");
        let f = &mut c.snapshot.as_mut().unwrap().fetch;
        f.text = Some(text.into());
        f.text_format = Some("markdown".into());
        let id = lib.add_item(&user(), c).unwrap().id().clone();
        fs::read_to_string(e.root.join(id.rel_dir()).join("content.md")).unwrap()
    };
    let md = content(
        &mut lib,
        "https://example.com/html",
        "## 小标题\n\n<img src=x onerror=alert(1)>\n",
    );
    assert!(md.contains("\\<img"), "{md}");
    let md = content(
        &mut lib,
        "https://example.com/scheme",
        "[打开](obsidian://open?vault=x)\n\n[^1]: file:///etc/hosts\n",
    );
    assert!(md.contains("\\[打开\\]"), "{md}");
    for sneaky in [
        "[x]( obsidian://open)",
        "[x](\nfile:///etc/hosts)",
        "[x](file&colon;///etc/hosts)",
        "[x](&#111;bsidian://open)",
        "> [a]: file:///x",
        "[a]:\nfile:///x",
    ] {
        let md = content(
            &mut lib,
            &format!("https://example.com/{}", sneaky.len()),
            sneaky,
        );
        assert!(md.contains("\\["), "{sneaky}: {md}");
    }
    let md = content(
        &mut lib,
        "https://example.com/clean",
        "## 小标题\n\n**粗** [链接](https://example.com/x) 和 [站内](#a)\n",
    );
    assert!(
        md.contains("## 小标题\n\n**粗** [链接](https://example.com/x)"),
        "{md}"
    );
}

#[test]
fn staged_files_that_change_after_hashing_are_refused() {
    let e = env();
    let mut lib = Library::init(&e.root, &e.machine).unwrap();
    let c = capture(&lib, "https://example.com/a", "字");
    let staged = c.snapshot.as_ref().unwrap().staged.clone();
    fs::write(staged.join("images/002.jpg"), b"late").unwrap();
    assert_eq!(lib.add_item(&user(), c).unwrap_err().code(), "invalid-args");
}
