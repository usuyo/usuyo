//! 条目页的几面（feature 003 §7.1）和索引里转出来那三层的定位串（§7.2）。两边都从 [`Content`] 来。

use usuyo_library::{parse_time_fragment, time_fragment};

use crate::content::Content;
use crate::engine::Links;
use crate::transcripts::{Spot, TextLayer, Transcript, remark_of};
use crate::view::{Comment, Facet, FacetSource, Frame, ImageRef, Position, Segment, SourceBy};

/// 一条的几面：画面、图、原文、口播（平台的在前，机器的新的在前）、图里的字、画面上的字、评论。
pub(crate) fn facets(id: &str, c: &Content, links: &Links) -> Vec<Facet> {
    let mut out = Vec::new();
    if let Some(v) = &c.video {
        let mut url = links.url(id, "video");
        // 合起来的那份落地以后地址跟着变，不拿缓存里的原样画面。
        if let Some(run) = &c.play {
            url.push(if url.contains('?') { '&' } else { '?' });
            url.push_str("v=");
            url.push_str(run);
        }
        let frames = c
            .keyframes
            .as_ref()
            .map(|k| {
                k.secs
                    .iter()
                    .map(|&sec| Frame {
                        sec,
                        url: links.url(id, &frame_layer(&k.run_id, sec)),
                    })
                    .collect()
            })
            .unwrap_or_default();
        out.push(Facet::Video {
            key: format!("video:{}", v.snapshot),
            snapshot_id: v.snapshot.clone(),
            duration_sec: v.duration_sec.unwrap_or(0.0),
            url: Some(url),
            frames,
            remark: c
                .keyframes
                .as_ref()
                .filter(|k| k.still)
                .and_then(|_| remark_of("static-picture"))
                .map(str::to_owned),
        });
    }
    let primary = c.primary.as_deref().unwrap_or_default();
    if !c.images.is_empty() {
        let images = c
            .images
            .iter()
            .map(|im| {
                let (width, height) = imagesize::size(&im.path)
                    .map(|s| (s.width as u32, s.height as u32))
                    .unwrap_or((0, 0));
                ImageRef {
                    key: im.key.clone(),
                    url: links.url(id, &format!("image/{}", im.key)),
                    width,
                    height,
                }
            })
            .collect();
        out.push(Facet::Image {
            key: format!("image:{primary}"),
            images,
            remark: c
                .duplicate_images
                .then(|| remark_of("duplicate-images"))
                .flatten()
                .map(str::to_owned),
        });
    }
    if let Some((_, paras)) = &c.text {
        out.push(Facet::Text {
            key: format!("text:{primary}"),
            segments: paras
                .iter()
                .map(|t| Segment {
                    text: t.clone(),
                    position: None,
                })
                .collect(),
        });
    }
    for t in &c.transcripts {
        out.push(transcript_facet(id, t, links));
    }
    if let Some((snapshot, comments)) = &c.comments {
        out.push(Facet::Comments {
            key: format!("comments:{snapshot}"),
            comments: comments
                .iter()
                .map(|x| Comment {
                    author: x.author.clone(),
                    text: x.text.clone(),
                    likes: x.likes,
                })
                .collect(),
        });
    }
    out
}

fn transcript_facet(id: &str, t: &Transcript, links: &Links) -> Facet {
    let source = match &t.machine {
        Some(m) => FacetSource {
            by: SourceBy::Machine,
            tool: m.tool.clone(),
            plugin: m.plugin.clone(),
            run_id: Some(m.run_id.clone()),
            at: Some(m.at.clone()),
            lang: t.lang.clone(),
        },
        None => FacetSource {
            by: SourceBy::Platform,
            tool: None,
            plugin: None,
            run_id: None,
            at: None,
            lang: t.lang.clone(),
        },
    };
    let key = match &t.machine {
        Some(m) => format!("{}:{}", t.layer.as_str(), m.run_id),
        None => format!(
            "asr:platform:{}:{}",
            t.snapshot,
            t.lang.as_deref().unwrap_or("und")
        ),
    };
    let segments = t
        .pieces
        .iter()
        .map(|p| Segment {
            text: p.text.clone(),
            position: Some(position(&p.spot, Some((links, id)))),
        })
        .collect();
    let remark = t.remark.map(str::to_owned);
    match t.layer {
        TextLayer::Asr => Facet::Asr {
            key,
            source,
            segments,
            remark,
        },
        TextLayer::Ocr => Facet::Ocr {
            key,
            source,
            segments,
            remark,
        },
        TextLayer::Frames => Facet::Frames {
            key,
            source,
            segments,
            remark,
        },
    }
}

/// `frame/<runId>/<毫秒>`。
fn frame_layer(run: &str, sec: f64) -> String {
    format!("frame/{run}/{}", (sec * 1000.0).round() as u64)
}

/// 段的位置；给了媒体地址时，画面上的字带那一帧的地址。
pub(crate) fn position(spot: &Spot, media: Option<(&Links, &str)>) -> Position {
    match spot {
        Spot::Time {
            sec,
            end,
            snapshot,
            xywh,
            frame,
        } => Position::Time {
            sec: *sec,
            end_sec: *end,
            snapshot_id: snapshot.clone(),
            xywh: *xywh,
            frame_url: match (media, frame) {
                (Some((links, id)), Some(f)) => Some(links.url(id, f)),
                _ => None,
            },
        },
        Spot::Image { key, xywh } => Position::Image {
            image_key: key.clone(),
            xywh: Some(*xywh),
        },
    }
}

/// 索引里的定位串（feature 003 §7.2）：`t=<起>,<止>`、`img=<imageKey>&xywh=<x,y,w,h>`、
/// `t=<秒>&xywh=<x,y,w,h>`。
pub(crate) fn locator(spot: &Spot) -> String {
    match spot {
        Spot::Time { sec, end, xywh, .. } => {
            let t = time_fragment(*sec, *end);
            match xywh {
                Some(b) => format!("{t}&xywh={}", boxed(b)),
                None => t,
            }
        }
        Spot::Image { key, xywh } => format!("img={key}&xywh={}", boxed(xywh)),
    }
}

fn boxed(b: &[f64; 4]) -> String {
    b.iter()
        .map(|v| {
            let s = format!("{v:.4}");
            s.trim_end_matches('0').trim_end_matches('.').to_owned()
        })
        .collect::<Vec<_>>()
        .join(",")
}

/// 从定位串回位置；认不出（原文、评论的 `p<n>`、`c<n>`）回 `None`。
pub(crate) fn position_of(locator: &str, snapshot: Option<&str>) -> Option<Position> {
    let mut time = None;
    let mut image = None;
    let mut xywh = None;
    for part in locator.split('&') {
        if part.starts_with("t=") {
            time = parse_time_fragment(part);
        } else if let Some(k) = part.strip_prefix("img=") {
            image = Some(k.to_owned());
        } else if let Some(b) = part.strip_prefix("xywh=") {
            let v: Vec<f64> = b.split(',').filter_map(|x| x.parse().ok()).collect();
            xywh = <[f64; 4]>::try_from(v).ok();
        }
    }
    match (time, image) {
        (Some((sec, end)), _) => Some(Position::Time {
            sec,
            end_sec: end,
            snapshot_id: snapshot?.to_owned(),
            xywh,
            frame_url: None,
        }),
        (None, Some(image_key)) => Some(Position::Image { image_key, xywh }),
        (None, None) => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn locators_round_trip() {
        let spots = [
            Spot::Time {
                sec: 102.5,
                end: Some(110.0),
                snapshot: "s".into(),
                xywh: None,
                frame: None,
            },
            Spot::Time {
                sec: 12.04,
                end: None,
                snapshot: "s".into(),
                xywh: Some([0.0736, 0.2542, 0.5504, 0.0542]),
                frame: Some("frame/r/12040".into()),
            },
            Spot::Image {
                key: "3f2a9c0d1e4b5a67".into(),
                xywh: [0.1, 0.25, 0.5, 0.05],
            },
        ];
        let locs: Vec<String> = spots.iter().map(locator).collect();
        assert_eq!(
            locs,
            [
                "t=102.5,110",
                "t=12.04&xywh=0.0736,0.2542,0.5504,0.0542",
                "img=3f2a9c0d1e4b5a67&xywh=0.1,0.25,0.5,0.05"
            ]
        );
        for (spot, loc) in spots.iter().zip(&locs) {
            assert_eq!(position_of(loc, Some("s")), Some(position(spot, None)));
        }
        assert_eq!(position_of("p3", Some("s")), None);
    }
}
