//! 条目 → 索引里的一条。收下、改动和 `reindex` 都只走这一个函数，重建后搜索结果不变。

use usuyo_contract::Manifest;
use usuyo_index::{IndexDoc, LayerText, Segment};

use crate::content::{Content, paragraphs};
use crate::facets::locator;

/// 定位串：`p<n>` 是第 n 段，`c<n>` 是第 n 条评论；转出来的三层见 [`locator`]。
/// 每份转出来的文字（每份平台字幕、每次转录）一层，带 `source`；同一层机器的新的在前，分数一样时先给新的。
pub(crate) fn index_doc(m: &Manifest, c: &Content) -> IndexDoc {
    let mut layers = Vec::new();
    if let Some((snapshot, paras)) = &c.text {
        layers.push(LayerText {
            layer: "text".into(),
            snapshot_id: snapshot.clone(),
            source: None,
            segments: numbered("p", paras.iter().cloned()),
        });
    }
    for t in &c.transcripts {
        layers.push(LayerText {
            layer: t.layer.as_str().into(),
            snapshot_id: Some(t.snapshot.clone()),
            source: Some(t.source.clone()),
            segments: t
                .pieces
                .iter()
                .map(|p| Segment {
                    text: p.text.clone(),
                    locator: locator(&p.spot),
                })
                .collect(),
        });
    }
    if let Some((snapshot, comments)) = &c.comments {
        layers.push(LayerText {
            layer: "comments".into(),
            snapshot_id: Some(snapshot.clone()),
            source: None,
            segments: numbered("c", comments.iter().map(|x| x.text.clone())),
        });
    }
    if let Some(t) = &c.takeaway {
        layers.push(LayerText {
            layer: "takeaway".into(),
            snapshot_id: None,
            source: None,
            segments: vec![Segment {
                text: t.clone(),
                locator: String::new(),
            }],
        });
    }
    let notes = paragraphs(&c.notes);
    if !notes.is_empty() {
        layers.push(LayerText {
            layer: "notes".into(),
            snapshot_id: None,
            source: None,
            segments: numbered("p", notes.into_iter()),
        });
    }
    let platform = m.id.rsplit_once('-').map_or("", |(p, _)| p).to_owned();
    IndexDoc {
        item_id: m.id.clone(),
        platform,
        kind: m.kind.clone(),
        title: m.title.clone(),
        author: m
            .author
            .as_ref()
            .map(|a| a.name.clone())
            .unwrap_or_default(),
        tags: m
            .tags
            .platform
            .iter()
            .map(|t| t.tag.clone())
            .chain(m.tags.user.iter().cloned())
            .collect(),
        topics: m.topics.clone(),
        published_at: m.published_at.as_deref().and_then(seconds),
        added_at: seconds(&m.added_at).unwrap_or_default(),
        private: m.private,
        layers,
    }
}

fn numbered(prefix: &str, texts: impl Iterator<Item = String>) -> Vec<Segment> {
    texts
        .enumerate()
        .map(|(i, text)| Segment {
            text,
            locator: format!("{prefix}{i}"),
        })
        .collect()
}

/// manifest 里的时间由 usuyo 写、schema 校验过；读不了就当没有这个时间。
fn seconds(s: &str) -> Option<i64> {
    usuyo_library::time::parse(s).ok().map(|t| t.as_second())
}
