//! 第二遍之前的选帧（feature 003 §5.3）：
//!
//! 1. 总有一帧在 `firstSec`；
//! 2. 镜头变化处各取一帧：按分数从高到低挑，和已选的每一帧至少隔 `minGapSec`，
//!    每分钟（第几分钟按时间点整除 60 秒算）最多 `maxPerMinute` 帧，总共最多 `maxFrames` 帧；
//! 3. 选出来的帧数少于 `min(minFrames, 时长 / minFrameEverySec)` 时，按时长均分补到那个数
//!    （补的点离已选的帧不到 0.5 秒就跳过）。
//!
//! 时间一律取整到毫秒再比较。

use crate::options::KeyframeOptions;
use crate::scan::SceneChange;

/// 补帧的点离已选的帧至少多少毫秒。
const PAD_MIN_APART_MS: u64 = 500;
/// 最后一帧离片尾至少多少毫秒：再靠后常常取不到帧。
const TAIL_MARGIN_MS: u64 = 100;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Pick {
    pub ms: u64,
    /// 镜头变化处的分数；总有的那一帧和补的帧为 `None`。
    pub scene: Option<f64>,
}

fn to_ms(sec: f64) -> u64 {
    (sec * 1000.0).round().max(0.0) as u64
}

pub fn select(duration_sec: f64, scenes: &[SceneChange], o: &KeyframeOptions) -> Vec<Pick> {
    let duration_ms = to_ms(duration_sec);
    if duration_ms == 0 {
        return Vec::new();
    }
    let last_ms = duration_ms.saturating_sub(TAIL_MARGIN_MS);
    let max_frames = o.max_frames as usize;
    let gap_ms = to_ms(o.min_gap_sec);
    let first = to_ms(o.first_sec).min(duration_ms / 2);
    let mut picks = vec![Pick {
        ms: first,
        scene: None,
    }];

    let mut candidates: Vec<(u64, f64)> = scenes
        .iter()
        .map(|s| (to_ms(s.sec).min(last_ms), s.score))
        .collect();
    candidates.sort_by(|a, b| b.1.total_cmp(&a.1).then(a.0.cmp(&b.0)));
    for (ms, score) in candidates {
        if picks.len() >= max_frames {
            break;
        }
        let in_minute = picks
            .iter()
            .filter(|p| p.ms / 60_000 == ms / 60_000)
            .count();
        // `minGapSec` 为 0 时同一毫秒也只取一帧（文件名是毫秒，重名的第二次不会写）。
        let apart = picks
            .iter()
            .all(|p| p.ms != ms && p.ms.abs_diff(ms) >= gap_ms);
        if apart && in_minute < o.max_per_minute as usize {
            picks.push(Pick {
                ms,
                scene: Some(score),
            });
        }
    }

    let wanted = f64::from(o.min_frames).min(duration_sec / o.min_frame_every_sec);
    if (picks.len() as f64) < wanted {
        let target = (wanted.ceil() as usize).min(max_frames);
        for i in 0..target {
            if picks.len() >= target {
                break;
            }
            let ms = (duration_sec * (i as f64 + 0.5) / target as f64 * 1000.0).round() as u64;
            let ms = ms.min(last_ms);
            if picks.iter().all(|p| p.ms.abs_diff(ms) >= PAD_MIN_APART_MS) {
                picks.push(Pick { ms, scene: None });
            }
        }
    }
    picks.sort_by_key(|p| p.ms);
    picks
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scenes(list: &[(f64, f64)]) -> Vec<SceneChange> {
        list.iter()
            .map(|&(sec, score)| SceneChange { sec, score })
            .collect()
    }

    fn times(p: &[Pick]) -> Vec<u64> {
        p.iter().map(|p| p.ms).collect()
    }

    #[test]
    fn always_the_first_frame_and_one_per_scene() {
        let p = select(
            63.2,
            &scenes(&[(12.04, 14.2), (30.0, 22.0)]),
            &Default::default(),
        );
        assert_eq!(times(&p), [500, 12_040, 30_000]);
        assert_eq!(p[0].scene, None);
        assert_eq!(p[1].scene, Some(14.2));
    }

    #[test]
    fn no_scene_changes_in_a_short_clip_is_the_first_frame_only() {
        let p = select(7.52, &[], &Default::default());
        assert_eq!(times(&p), [500]);
    }

    #[test]
    fn neighbours_are_at_least_the_gap_apart_and_the_stronger_one_wins() {
        // 12.0 和 13.5 只隔 1.5 秒：留分数高的；2.0 离 0.5 那一帧只有 1.5 秒：丢掉。
        let p = select(
            60.0,
            &scenes(&[(2.0, 50.0), (12.0, 15.0), (13.5, 30.0), (15.5, 11.0)]),
            &Default::default(),
        );
        assert_eq!(times(&p), [500, 13_500, 15_500]);
        for w in p.windows(2) {
            assert!(w[1].ms - w[0].ms >= 2000);
        }
    }

    #[test]
    fn at_most_six_per_minute_keeping_the_strongest() {
        let many: Vec<(f64, f64)> = (0..20)
            .map(|i| (3.0 + 2.5 * f64::from(i), 10.0 + f64::from(i)))
            .collect();
        let p = select(300.0, &scenes(&many), &Default::default());
        let in_first_minute = p.iter().filter(|p| p.ms < 60_000).count();
        assert_eq!(in_first_minute, 6);
        // 0.5 秒那一帧算一帧；其余五帧是分数最高的五处。
        let best: Vec<u64> = (15..20).map(|i| to_ms(3.0 + 2.5 * f64::from(i))).collect();
        let mut kept: Vec<u64> = p.iter().skip(1).map(|p| p.ms).collect();
        kept.sort_unstable();
        assert_eq!(kept, best);
    }

    #[test]
    fn at_most_sixty_frames() {
        // 每 2.1 秒一处，跨 20 分钟：不受每分钟的上限挡也会超过 60。
        let many: Vec<(f64, f64)> = (1..500).map(|i| (2.1 * f64::from(i) + 1.0, 20.0)).collect();
        let o = KeyframeOptions {
            max_per_minute: 600,
            ..Default::default()
        };
        let p = select(1200.0, &scenes(&many), &o);
        assert_eq!(p.len(), 60);
    }

    #[test]
    fn short_picks_are_padded_evenly() {
        // 25 秒、没有镜头变化：需要 min(3, 2.5) 向上取整 = 3 帧。
        let p = select(25.0, &[], &Default::default());
        assert_eq!(times(&p), [500, 4_167, 12_500]);
        // 100 秒以上也是补到 3 帧。
        let p = select(100.0, &scenes(&[(40.0, 20.0)]), &Default::default());
        assert_eq!(p.len(), 3);
        assert!(p.iter().any(|p| p.scene.is_some()));
    }

    #[test]
    fn a_zero_gap_still_never_picks_the_same_millisecond() {
        let o = KeyframeOptions {
            min_gap_sec: 0.0,
            min_frames: 0,
            ..Default::default()
        };
        let p = select(60.0, &scenes(&[(0.5, 40.0), (1.0, 30.0), (1.0, 20.0)]), &o);
        assert_eq!(times(&p), [500, 1_000]);
    }

    #[test]
    fn times_stay_inside_the_clip() {
        let p = select(
            5.0,
            &scenes(&[(5.0, 30.0), (9.0, 12.0)]),
            &Default::default(),
        );
        assert!(p.iter().all(|p| p.ms <= 4_900), "{p:?}");
        assert_eq!(select(0.0, &[], &Default::default()), []);
        let tiny = select(0.4, &[], &Default::default());
        assert_eq!(times(&tiny), [200]);
    }
}
