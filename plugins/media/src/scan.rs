//! 第一遍扫描的 stderr：`scdet` 报镜头变化，`freezedetect` 报画面停住的区间。
//!
//! 只认以 `[滤镜名 @ 地址]` 开头的行；横幅里的元数据行（标题、注释等文件里写的字）都带缩进，
//! 文件里的字冒充不了这两种输出。`freezedetect` 在停到片尾时只打 `freeze_start`、不打 `freeze_end`，
//! 这种区间按停到片尾算。

/// 一处镜头变化：第几秒，`scdet` 的分数。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SceneChange {
    pub sec: f64,
    pub score: f64,
}

/// 一段停住的画面；`end` 是 `None` 表示停到片尾。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Freeze {
    pub start: f64,
    pub end: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Scan {
    pub scenes: Vec<SceneChange>,
    pub freezes: Vec<Freeze>,
}

enum Event {
    Scene(SceneChange),
    FreezeStart(f64),
    FreezeEnd(f64),
}

/// 这一行是不是 `scdet` 或 `freezedetect` 打的。
pub fn is_scan_line(line: &str) -> bool {
    event(line).is_some()
}

fn number(text: &str) -> Option<f64> {
    let v: f64 = text.trim().parse().ok()?;
    (v.is_finite() && v >= 0.0).then_some(v)
}

fn event(line: &str) -> Option<Event> {
    let rest = line.strip_prefix('[')?;
    let (head, message) = rest.split_once("] ")?;
    let filter = head.split(" @ ").next()?;
    if filter.contains("scdet") {
        let body = message.strip_prefix("lavfi.scd.score: ")?;
        let (score, time) = body.split_once(", lavfi.scd.time: ")?;
        return Some(Event::Scene(SceneChange {
            sec: number(time)?,
            score: number(score)?,
        }));
    }
    if filter.contains("freezedetect") {
        let body = message.strip_prefix("lavfi.freezedetect.")?;
        if let Some(v) = body.strip_prefix("freeze_start: ") {
            return Some(Event::FreezeStart(number(v)?));
        }
        if let Some(v) = body.strip_prefix("freeze_end: ") {
            return Some(Event::FreezeEnd(number(v)?));
        }
    }
    None
}

/// 把留下的行读成镜头变化和停住的区间。
pub fn parse_scan<S: AsRef<str>>(lines: &[S]) -> Scan {
    let mut scan = Scan::default();
    let mut open: Option<f64> = None;
    for line in lines {
        match event(line.as_ref()) {
            Some(Event::Scene(s)) => scan.scenes.push(s),
            // 已经有一段没收尾时又来一个开始：保留先到的。
            Some(Event::FreezeStart(s)) => {
                open.get_or_insert(s);
            }
            Some(Event::FreezeEnd(e)) => {
                if let Some(start) = open.take().filter(|s| *s <= e) {
                    scan.freezes.push(Freeze {
                        start,
                        end: Some(e),
                    });
                }
            }
            None => {}
        }
    }
    if let Some(start) = open {
        scan.freezes.push(Freeze { start, end: None });
    }
    scan
}

impl Scan {
    /// 停住的时间加起来多少秒，不超过 `duration`。
    pub fn frozen_sec(&self, duration: f64) -> f64 {
        let total: f64 = self
            .freezes
            .iter()
            .map(|f| {
                let start = f.start.min(duration);
                let end = f.end.unwrap_or(duration).min(duration);
                (end - start).max(0.0)
            })
            .sum();
        total.min(duration)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const CUTS: &str = "\
[Parsed_freezedetect_3 @ 0x14af0ab10] lavfi.freezedetect.freeze_start: 0
[Parsed_scdet_2 @ 0x14af0a990] lavfi.scd.score: 21.908, lavfi.scd.time: 3
[Parsed_freezedetect_3 @ 0x14af0ab10] lavfi.freezedetect.freeze_duration: 3
[Parsed_freezedetect_3 @ 0x14af0ab10] lavfi.freezedetect.freeze_end: 3
[Parsed_scdet_2 @ 0x14af0a990] lavfi.scd.score: 27.421, lavfi.scd.time: 6
[Parsed_freezedetect_3 @ 0x14af0ab10] lavfi.freezedetect.freeze_start: 6";

    fn lines(text: &str) -> Vec<String> {
        text.lines().map(str::to_owned).collect()
    }

    #[test]
    fn scene_changes_and_a_freeze_that_runs_to_the_end() {
        let scan = parse_scan(&lines(CUTS));
        assert_eq!(
            scan.scenes,
            [
                SceneChange {
                    sec: 3.0,
                    score: 21.908
                },
                SceneChange {
                    sec: 6.0,
                    score: 27.421
                },
            ]
        );
        // 0–3 秒停住一段，6 秒起停到片尾（没有 freeze_end），片长 9 秒。
        assert_eq!(
            scan.freezes,
            [
                Freeze {
                    start: 0.0,
                    end: Some(3.0)
                },
                Freeze {
                    start: 6.0,
                    end: None
                },
            ]
        );
        assert_eq!(scan.frozen_sec(9.0), 6.0);
    }

    #[test]
    fn a_picture_frozen_the_whole_time_has_only_a_start() {
        let scan = parse_scan(&lines(
            "[Parsed_freezedetect_3 @ 0x140604990] lavfi.freezedetect.freeze_start: 0",
        ));
        assert_eq!(
            scan.freezes,
            [Freeze {
                start: 0.0,
                end: None
            }]
        );
        assert_eq!(scan.frozen_sec(5.0), 5.0);
    }

    #[test]
    fn nothing_found() {
        assert_eq!(parse_scan::<&str>(&[]), Scan::default());
        assert_eq!(Scan::default().frozen_sec(7.5), 0.0);
    }

    #[test]
    fn only_filter_lines_count() {
        let hostile = [
            "    title           : lavfi.scd.score: 99, lavfi.scd.time: 1",
            "lavfi.scd.score: 99, lavfi.scd.time: 1",
            "[mov,mp4 @ 0x1] lavfi.scd.score: 99, lavfi.scd.time: 1",
            "[Parsed_scdet_2 @ 0x1] lavfi.scd.score: nan, lavfi.scd.time: 1",
            "[Parsed_scdet_2 @ 0x1] lavfi.scd.score: 5, lavfi.scd.time: -1",
            "[Parsed_scdet_2 @ 0x1] lavfi.scd.score: 5",
            "[Parsed_freezedetect_3 @ 0x1] lavfi.freezedetect.freeze_start: inf",
            "[Parsed_freezedetect_3 @ 0x1] lavfi.freezedetect.freeze_end: 4",
            "[Parsed_scdet_2 @ 0x1] lavfi.scd.score: 12.5, lavfi.scd.time: 2.25",
        ];
        assert!(!is_scan_line(hostile[0]));
        let scan = parse_scan(&hostile);
        assert_eq!(
            scan.scenes,
            [SceneChange {
                sec: 2.25,
                score: 12.5
            }]
        );
        // 没有开始的 freeze_end 不算。
        assert_eq!(scan.freezes, []);
        assert!(is_scan_line(hostile[8]));
    }

    #[test]
    fn frozen_time_is_clamped_to_the_duration() {
        let scan = Scan {
            scenes: vec![],
            freezes: vec![
                Freeze {
                    start: 1.0,
                    end: Some(4.0),
                },
                Freeze {
                    start: 3.0,
                    end: Some(50.0),
                },
            ],
        };
        assert_eq!(scan.frozen_sec(10.0), 10.0);
        let late = Scan {
            scenes: vec![],
            freezes: vec![Freeze {
                start: 12.0,
                end: None,
            }],
        };
        assert_eq!(late.frozen_sec(10.0), 0.0);
    }
}
