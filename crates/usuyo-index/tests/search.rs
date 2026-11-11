use tempfile::TempDir;
use usuyo_index::{
    DateRange, Error, Filter, IndexDoc, ItemHit, LayerText, SearchIndex, SearchRequest, Segment,
    Visibility,
};

fn layer(name: &str, snapshot: Option<&str>, segments: &[(&str, &str)]) -> LayerText {
    LayerText {
        layer: name.into(),
        snapshot_id: snapshot.map(Into::into),
        segments: segments
            .iter()
            .map(|(text, locator)| Segment {
                text: (*text).into(),
                locator: (*locator).into(),
            })
            .collect(),
    }
}

fn doc(id: &str, title: &str, layers: Vec<LayerText>) -> IndexDoc {
    IndexDoc {
        item_id: id.into(),
        platform: "douyin".into(),
        kind: "video".into(),
        title: title.into(),
        author: "咖啡日常".into(),
        added_at: 1_000,
        layers,
        ..IndexDoc::default()
    }
}

fn corpus() -> Vec<IndexDoc> {
    vec![
        IndexDoc {
            tags: vec!["咖啡".into()],
            topics: vec!["topic-coffee".into()],
            published_at: Some(1_700_000_000),
            added_at: 1_700_000_100,
            ..doc(
                "douyin-a",
                "手冲咖啡：三段式注水",
                vec![
                    layer(
                        "asr",
                        Some("snap1"),
                        &[
                            ("第一段 40 克闷蒸 30 秒", "t=42,48"),
                            ("然后绕圈注水到 120 克", "t=50,56"),
                        ],
                    ),
                    layer("notes", None, &[("以后试试 V60 的滤杯", "p=3")]),
                ],
            )
        },
        IndexDoc {
            platform: "web".into(),
            kind: "article".into(),
            tags: vec!["编程".into()],
            published_at: Some(1_600_000_000),
            added_at: 1_600_000_100,
            ..doc(
                "web-b",
                "Rust 并发笔记",
                vec![layer(
                    "text",
                    None,
                    &[
                        ("Tokio 的 async runtime 很适合写网络服务。", "L1"),
                        ("Café 里写代码也行。", "L2"),
                    ],
                )],
            )
        },
        IndexDoc {
            private: true,
            platform: "xhs".into(),
            kind: "note".into(),
            added_at: 1_800_000_000,
            ..doc(
                "xhs-c",
                "私人日记",
                vec![layer("text", None, &[("今天去喝了手冲，很放松。", "L1")])],
            )
        },
    ]
}

fn open_with(docs: Vec<IndexDoc>) -> (TempDir, SearchIndex) {
    let dir = TempDir::new().unwrap();
    let index = SearchIndex::open_or_create(&dir.path().join("index")).unwrap();
    for d in &docs {
        index.upsert(d).unwrap();
    }
    index.commit().unwrap();
    (dir, index)
}

fn ids(index: &SearchIndex, query: &str) -> Vec<String> {
    let req = SearchRequest {
        query: query.into(),
        ..SearchRequest::default()
    };
    index
        .search(&req)
        .unwrap()
        .items
        .into_iter()
        .map(|i| i.item_id)
        .collect()
}

fn search_with(index: &SearchIndex, query: &str, filter: Filter) -> Vec<ItemHit> {
    let req = SearchRequest {
        query: query.into(),
        filter,
        ..SearchRequest::default()
    };
    index.search(&req).unwrap().items
}

#[test]
fn two_character_words() {
    let (_d, index) = open_with(corpus());
    assert_eq!(ids(&index, "闷蒸"), ["douyin-a"]);
    assert_eq!(ids(&index, "注水"), ["douyin-a"]);
    // 不是 jieba 词典里的二字组合也能搜到
    assert_eq!(ids(&index, "蒸 30"), ["douyin-a"]);
    assert_eq!(ids(&index, "段式"), ["douyin-a"]);
    assert_eq!(ids(&index, "不存在的词"), Vec::<String>::new());
}

#[test]
fn mixed_chinese_and_english() {
    let (_d, index) = open_with(corpus());
    assert_eq!(ids(&index, "tokio"), ["web-b"]);
    assert_eq!(ids(&index, "TOKIO 网络"), ["web-b"]);
    assert_eq!(ids(&index, "cafe"), ["web-b"], "折叠重音");
    assert_eq!(ids(&index, "v60 滤杯"), ["douyin-a"]);
    assert_eq!(ids(&index, "v60滤杯"), ["douyin-a"], "同一个词里中英文相连");
}

#[test]
fn pinyin_and_initials() {
    let (_d, index) = open_with(corpus());
    assert_eq!(ids(&index, "shouchong"), ["douyin-a"]);
    assert_eq!(ids(&index, "sc"), ["douyin-a"]);
    assert_eq!(ids(&index, "shouchongkafei"), ["douyin-a"]);
    assert_eq!(ids(&index, "SC"), ["douyin-a"]);
    let bank = doc("bank", "银行", vec![]);
    index.upsert(&bank).unwrap();
    index.commit().unwrap();
    assert_eq!(ids(&index, "yinhang"), ["bank"]);
}

#[test]
fn hit_carries_layer_locator_and_highlight() {
    let (_d, index) = open_with(corpus());
    let hits = search_with(&index, "闷蒸", Filter::default());
    let seg = &hits[0].segments[0];
    assert_eq!(seg.layer, "asr");
    assert_eq!(seg.locator, "t=42,48");
    assert_eq!(seg.snapshot_id.as_deref(), Some("snap1"));
    assert_eq!(hits[0].title, "手冲咖啡：三段式注水");
    let h = &seg.snippet.highlights;
    assert!(!h.is_empty());
    for r in h {
        assert!(
            seg.snippet.text.is_char_boundary(r.start) && seg.snippet.text.is_char_boundary(r.end)
        );
    }
    assert_eq!(&seg.snippet.text[h[0].clone()], "闷蒸");

    // 拼音命中，高亮落在汉字上
    let hits = search_with(&index, "shouchong", Filter::default());
    let meta = hits[0].segments.iter().find(|s| s.layer == "meta").unwrap();
    assert_eq!(
        &meta.snippet.text[meta.snippet.highlights[0].clone()],
        "手冲"
    );
}

#[test]
fn filters() {
    let (_d, index) = open_with(corpus());
    let all = Visibility::IncludePrivate;
    let by = |q: &str, filter: Filter| -> Vec<String> {
        search_with(
            &index,
            q,
            Filter {
                visibility: all,
                ..filter
            },
        )
        .into_iter()
        .map(|i| i.item_id)
        .collect()
    };
    assert_eq!(by("手冲", Filter::default()).len(), 2);
    assert_eq!(
        by(
            "手冲",
            Filter {
                platforms: vec!["xhs".into()],
                ..Filter::default()
            }
        ),
        ["xhs-c"]
    );
    assert_eq!(
        by(
            "手冲",
            Filter {
                kinds: vec!["video".into()],
                ..Filter::default()
            }
        ),
        ["douyin-a"]
    );
    assert_eq!(
        by(
            "注水",
            Filter {
                tags: vec!["咖啡".into()],
                ..Filter::default()
            }
        ),
        ["douyin-a"]
    );
    assert!(
        by(
            "注水",
            Filter {
                tags: vec!["编程".into()],
                ..Filter::default()
            }
        )
        .is_empty()
    );
    assert_eq!(
        by(
            "注水",
            Filter {
                topics: vec!["topic-coffee".into()],
                ..Filter::default()
            }
        ),
        ["douyin-a"]
    );
    // 层：只在口播里找
    assert_eq!(
        by(
            "手冲",
            Filter {
                layers: vec!["notes".into()],
                ..Filter::default()
            }
        )
        .len(),
        0
    );
    assert_eq!(
        by(
            "滤杯",
            Filter {
                layers: vec!["notes".into()],
                ..Filter::default()
            }
        ),
        ["douyin-a"]
    );
    // 层：meta 只在标题等里找
    assert_eq!(
        by(
            "闷蒸",
            Filter {
                layers: vec!["meta".into()],
                ..Filter::default()
            }
        )
        .len(),
        0
    );
    // 发布日期 [from, to)
    let range = DateRange {
        from: Some(1_650_000_000),
        to: None,
    };
    assert_eq!(
        by(
            "手冲",
            Filter {
                published: range,
                ..Filter::default()
            }
        ),
        ["douyin-a"]
    );
    let range = DateRange {
        from: None,
        to: Some(1_650_000_000),
    };
    assert_eq!(
        by(
            "代码",
            Filter {
                published: range,
                ..Filter::default()
            }
        ),
        ["web-b"]
    );
    // 收下日期
    let range = DateRange {
        from: Some(1_700_000_100),
        to: Some(1_700_000_101),
    };
    assert_eq!(
        by(
            "手冲",
            Filter {
                added: range,
                ..Filter::default()
            }
        ),
        ["douyin-a"]
    );
}

#[test]
fn private_is_hidden_by_default() {
    let (_d, index) = open_with(corpus());
    // 默认（Agent 一侧）看不到私密条目
    assert_eq!(ids(&index, "放松"), Vec::<String>::new());
    assert_eq!(ids(&index, "私人日记"), Vec::<String>::new());
    assert_eq!(ids(&index, "手冲"), ["douyin-a"]);

    let with = |visibility| {
        search_with(
            &index,
            "手冲",
            Filter {
                visibility,
                ..Filter::default()
            },
        )
        .into_iter()
        .map(|i| i.item_id)
        .collect::<Vec<_>>()
    };
    assert_eq!(with(Visibility::IncludePrivate).len(), 2);
    assert_eq!(with(Visibility::PrivateOnly), ["xhs-c"]);
}

#[test]
fn upsert_replaces_old_segments() {
    let (_d, index) = open_with(corpus());
    assert_eq!(ids(&index, "闷蒸"), ["douyin-a"]);
    index
        .upsert(&doc(
            "douyin-a",
            "新标题",
            vec![layer("asr", None, &[("改成了冷萃的做法", "t=1,2")])],
        ))
        .unwrap();
    index.commit().unwrap();
    assert_eq!(ids(&index, "闷蒸"), Vec::<String>::new());
    assert_eq!(
        ids(&index, "手冲咖啡"),
        Vec::<String>::new(),
        "旧标题也没了"
    );
    assert_eq!(ids(&index, "冷萃"), ["douyin-a"]);
    assert_eq!(ids(&index, "新标题"), ["douyin-a"]);
}

#[test]
fn bad_layer_keeps_old_data() {
    let (_d, index) = open_with(corpus());
    let bad = doc(
        "douyin-a",
        "x",
        vec![layer("meta", None, &[("冲突的层名", "")])],
    );
    assert!(matches!(index.upsert(&bad), Err(Error::InvalidLayer(_))));
    index.commit().unwrap();
    assert_eq!(ids(&index, "闷蒸"), ["douyin-a"]);
}

#[test]
fn delete_removes_item() {
    let (_d, index) = open_with(corpus());
    index.delete("douyin-a").unwrap();
    index.commit().unwrap();
    assert_eq!(ids(&index, "闷蒸"), Vec::<String>::new());
    assert_eq!(ids(&index, "tokio"), ["web-b"]);
}

fn summary(index: &SearchIndex, query: &str) -> Vec<(String, Vec<(String, String)>)> {
    let req = SearchRequest {
        query: query.into(),
        filter: Filter {
            visibility: Visibility::IncludePrivate,
            ..Filter::default()
        },
        ..SearchRequest::default()
    };
    index
        .search(&req)
        .unwrap()
        .items
        .into_iter()
        .map(|i| {
            (
                i.item_id,
                i.segments
                    .into_iter()
                    .map(|s| (s.layer, s.locator))
                    .collect(),
            )
        })
        .collect()
}

#[test]
fn rebuild_gives_same_results() {
    let (_d, index) = open_with(corpus());
    // 先做一些增删改，让索引里有旧版本和待清的删除
    index
        .upsert(&doc(
            "tmp",
            "临时",
            vec![layer("text", None, &[("临时手冲", "")])],
        ))
        .unwrap();
    index.commit().unwrap();
    index.delete("tmp").unwrap();
    index.commit().unwrap();

    let queries = ["手冲", "注水", "tokio", "sc", "cafe", "放松"];
    let before: Vec<_> = queries.iter().map(|q| summary(&index, q)).collect();

    index.rebuild(corpus()).unwrap();
    let after: Vec<_> = queries.iter().map(|q| summary(&index, q)).collect();
    assert_eq!(before, after);

    // 全新的目录重建也一样
    let (_d2, fresh) = open_with(Vec::new());
    assert!(fresh.needs_rebuild());
    fresh.rebuild(corpus()).unwrap();
    assert!(!fresh.needs_rebuild());
    let from_fresh: Vec<_> = queries.iter().map(|q| summary(&fresh, q)).collect();
    assert_eq!(before, from_fresh);
}

#[test]
fn stamp_controls_rebuild() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("index");
    let index = SearchIndex::open_or_create(&path).unwrap();
    assert!(index.needs_rebuild(), "新建的索引要先重建");
    index.rebuild(corpus()).unwrap();
    drop(index);

    let index = SearchIndex::open_or_create(&path).unwrap();
    assert!(!index.needs_rebuild());
    assert_eq!(ids(&index, "闷蒸"), ["douyin-a"]);
    drop(index);

    // 版本不对：清空重来
    std::fs::write(
        path.join("usuyo-index.json"),
        r#"{"schema":0,"analyzer":0,"complete":true}"#,
    )
    .unwrap();
    let index = SearchIndex::open_or_create(&path).unwrap();
    assert!(index.needs_rebuild());
    assert_eq!(ids(&index, "闷蒸"), Vec::<String>::new());
    drop(index);

    // 不认识的目录不动它
    let other = dir.path().join("other");
    std::fs::create_dir(&other).unwrap();
    std::fs::write(other.join("keep.txt"), "x").unwrap();
    assert!(matches!(
        SearchIndex::open_or_create(&other),
        Err(Error::ForeignDirectory(_))
    ));
    assert!(other.join("keep.txt").exists());
}

#[test]
fn bad_queries_return_errors() {
    let (_d, index) = open_with(corpus());
    let run = |q: &str| {
        index.search(&SearchRequest {
            query: q.into(),
            ..SearchRequest::default()
        })
    };
    assert!(matches!(run(""), Err(Error::EmptyQuery)));
    assert!(matches!(run("   "), Err(Error::EmptyQuery)));
    assert!(matches!(run("!!! ???"), Err(Error::EmptyQuery)));
    assert!(matches!(
        run(&"词 ".repeat(40)),
        Err(Error::QueryTooLong { .. })
    ));
    assert!(matches!(
        run(&"a".repeat(500)),
        Err(Error::QueryTooLong { .. })
    ));
    // 带查询语法的字符不当语法处理，也不 panic
    for q in [
        "\"未闭合",
        "(a",
        "title:手冲",
        "a AND",
        "-手冲",
        "*",
        "手冲\u{0}",
        "𠀀𠀁",
    ] {
        let _ = run(q);
    }
    let bad = SearchRequest {
        query: "手冲".into(),
        cursor: Some("zzz".into()),
        ..SearchRequest::default()
    };
    assert!(matches!(index.search(&bad), Err(Error::BadCursor)));
}

#[test]
fn paging_with_cursor_is_stable_and_complete() {
    let docs: Vec<IndexDoc> = (0..25)
        .map(|i| {
            doc(
                &format!("item-{i:02}"),
                "同一个标题",
                vec![layer("text", None, &[("手冲咖啡的笔记", "")])],
            )
        })
        .collect();
    let (_d, index) = open_with(docs);
    let mut seen = Vec::new();
    let mut cursor = None;
    loop {
        let page = index
            .search(&SearchRequest {
                query: "手冲".into(),
                limit: 10,
                cursor: cursor.clone(),
                ..SearchRequest::default()
            })
            .unwrap();
        seen.extend(page.items.into_iter().map(|i| i.item_id));
        match page.next_cursor {
            Some(c) => cursor = Some(c),
            None => break,
        }
    }
    assert_eq!(seen.len(), 25);
    let mut sorted = seen.clone();
    sorted.sort();
    sorted.dedup();
    assert_eq!(sorted.len(), 25, "没有重复");
}

#[test]
fn long_item_does_not_crowd_out_others() {
    let long: Vec<(String, String)> = (0..600)
        .map(|i| (format!("第{i}段 手冲 的口播"), format!("t={i}")))
        .collect();
    let segs: Vec<(&str, &str)> = long.iter().map(|(a, b)| (a.as_str(), b.as_str())).collect();
    let mut docs = vec![doc("long", "长口播", vec![layer("asr", None, &segs)])];
    docs.push(doc(
        "short",
        "短",
        vec![layer("text", None, &[("手冲", "")])],
    ));
    let (_d, index) = open_with(docs);
    let mut found = ids(&index, "手冲");
    found.sort();
    assert_eq!(found, ["long", "short"]);
    let hits = search_with(&index, "手冲", Filter::default());
    let long_hit = hits.iter().find(|h| h.item_id == "long").unwrap();
    assert_eq!(long_hit.segments.len(), 3);
}

/// 速度和体积的粗测，手动跑：`cargo test -p usuyo-index --release -- --ignored --nocapture`
#[test]
#[ignore]
fn synthetic_speed_and_size() {
    let words = [
        "手冲", "咖啡", "注水", "闷蒸", "滤杯", "rust", "tokio", "搜索", "索引", "拼音", "视频",
        "口播", "笔记", "async", "今天", "我们",
    ];
    let docs: Vec<IndexDoc> = (0..100)
        .map(|d| {
            let segs: Vec<(String, String)> = (0..20)
                .map(|s| {
                    let text: String = (0..30)
                        .map(|k| words[(d * 7 + s * 3 + k * 5) % words.len()])
                        .collect::<Vec<_>>()
                        .join(" 的 ");
                    (text, format!("t={s}"))
                })
                .collect();
            let segs: Vec<(&str, &str)> =
                segs.iter().map(|(a, b)| (a.as_str(), b.as_str())).collect();
            doc(
                &format!("item-{d}"),
                "合成",
                vec![layer("asr", None, &segs)],
            )
        })
        .collect();
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("index");
    let index = SearchIndex::open_or_create(&path).unwrap();
    let t = std::time::Instant::now();
    index.rebuild(docs).unwrap();
    println!("rebuild 2000 段: {:?}", t.elapsed());
    let size: u64 = std::fs::read_dir(&path)
        .unwrap()
        .map(|e| e.unwrap().metadata().unwrap().len())
        .sum();
    println!("索引体积: {} KiB", size / 1024);
    let t = std::time::Instant::now();
    let n = 200;
    for _ in 0..n {
        index
            .search(&SearchRequest {
                query: "手冲 注水".into(),
                ..SearchRequest::default()
            })
            .unwrap();
    }
    println!("单次搜索: {:?}", t.elapsed() / n);
}
