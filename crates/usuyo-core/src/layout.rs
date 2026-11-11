//! 地图上的位置。真正按内容拼的引擎以后来（docs/research/16-piecing-engine.md），
//! 先放一个确定的：关注的主题是一片，成员在它旁边，其余照 id 的哈希放，不会自己挪。

use crate::view::{MapRegion, Topic};

/// 摆一张地图要的条目信息。
#[derive(Debug, Clone)]
pub struct LayoutItem {
    pub id: String,
    pub topic_ids: Vec<String>,
}

/// 摆好的结果：`points` 和传进来的条目一一对应，坐标都是 0–1。
#[derive(Debug, Clone, PartialEq)]
pub struct Placed {
    pub regions: Vec<MapRegion>,
    pub points: Vec<(f64, f64)>,
}

/// 换布局引擎只换这个。
pub trait Layout: Send + Sync {
    fn place(&self, topics: &[Topic], items: &[LayoutItem]) -> Placed;
}

/// 照哈希摆：同样的输入总是同样的位置。
#[derive(Debug, Clone, Copy, Default)]
pub struct HashLayout;

/// 地图大约是宽 1.6 倍于高，半径按宽算，竖直方向要放大。
const ASPECT: f64 = 1.6;

impl Layout for HashLayout {
    fn place(&self, topics: &[Topic], items: &[LayoutItem]) -> Placed {
        let mut regions: Vec<MapRegion> = topics
            .iter()
            .filter(|t| t.followed)
            .map(|t| MapRegion {
                topic_id: t.id.clone(),
                x: 0.15 + 0.7 * unit(&t.id, "x"),
                y: 0.15 + 0.7 * unit(&t.id, "y"),
                r: 0.0,
                count: items.iter().filter(|i| i.topic_ids.contains(&t.id)).count(),
            })
            .collect();
        for r in &mut regions {
            r.r = (0.06 + 0.012 * (r.count as f64).sqrt()).min(0.18);
        }
        let points = items
            .iter()
            .map(|item| {
                let home = item
                    .topic_ids
                    .iter()
                    .find_map(|t| regions.iter().find(|r| &r.topic_id == t));
                match home {
                    Some(r) => {
                        let angle = std::f64::consts::TAU * unit(&item.id, "a");
                        let dist = r.r * 0.85 * unit(&item.id, "d").sqrt();
                        (
                            clamp(r.x + angle.cos() * dist),
                            clamp(r.y + angle.sin() * dist * ASPECT),
                        )
                    }
                    None => (
                        0.04 + 0.92 * unit(&item.id, "x"),
                        0.06 + 0.88 * unit(&item.id, "y"),
                    ),
                }
            })
            .collect();
        Placed { regions, points }
    }
}

fn clamp(v: f64) -> f64 {
    v.clamp(0.01, 0.99)
}

/// FNV-1a 64 位，映到 [0, 1)。
fn unit(id: &str, salt: &str) -> f64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for b in id.bytes().chain([0]).chain(salt.bytes()) {
        h ^= u64::from(b);
        h = h.wrapping_mul(0x0100_0000_01b3);
    }
    (h >> 11) as f64 / (1u64 << 53) as f64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stable_and_near_their_topic() {
        let topics = vec![
            Topic {
                id: "t1".into(),
                name: "咖啡".into(),
                followed: true,
            },
            Topic {
                id: "t2".into(),
                name: "跑步".into(),
                followed: false,
            },
        ];
        let items = vec![
            LayoutItem {
                id: "web-a".into(),
                topic_ids: vec!["t1".into()],
            },
            LayoutItem {
                id: "web-b".into(),
                topic_ids: vec!["t2".into()],
            },
        ];
        let a = HashLayout.place(&topics, &items);
        assert_eq!(a, HashLayout.place(&topics, &items));
        assert_eq!(a.regions.len(), 1, "只有关注的主题是一片");
        let r = &a.regions[0];
        assert_eq!(r.count, 1);
        let (x, y) = a.points[0];
        assert!((x - r.x).abs() <= r.r && (y - r.y).abs() <= r.r * ASPECT);
        for (x, y) in &a.points {
            assert!((0.0..=1.0).contains(x) && (0.0..=1.0).contains(y));
        }
    }
}
