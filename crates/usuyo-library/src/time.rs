//! 时间（SPEC §0.5）：usuyo 写的一律 UTC、3 位毫秒、`Z`。

use jiff::{SignedDuration, Timestamp};

use crate::{Error, Result};

/// 现在，照 usuyo 的写法。
pub fn now() -> String {
    format(Timestamp::now())
}

/// 照 usuyo 的写法格式化，毫秒以下截掉。
pub fn format(ts: Timestamp) -> String {
    ts.strftime("%Y-%m-%dT%H:%M:%S%.3fZ").to_string()
}

/// 读 RFC 3339（带偏移）。
pub fn parse(s: &str) -> Result<Timestamp> {
    s.parse::<Timestamp>()
        .map_err(|e| Error::InvalidArgs(format!("bad time `{s}`: {e}")))
}

/// 是不是 usuyo 写法的时间。
pub fn is_canonical(s: &str) -> bool {
    parse(s).is_ok_and(|ts| format(ts) == s)
}

/// 加整数天。
pub fn add_days(s: &str, days: u32) -> Result<String> {
    let ts = parse(s)?;
    let later = ts
        .checked_add(SignedDuration::from_hours(i64::from(days) * 24))
        .map_err(|e| Error::InvalidArgs(format!("time overflow: {e}")))?;
    Ok(format(later))
}

/// 写进 Markdown 给人看的日期：取 `publishedAtOriginal` 的日期部分，没有就取 UTC 日期。
pub fn display_date(original: Option<&str>, utc: Option<&str>) -> Option<String> {
    fn pick(s: &str) -> Option<&str> {
        s.get(..10).filter(|d| d.as_bytes().get(4) == Some(&b'-'))
    }
    original
        .and_then(pick)
        .or_else(|| utc.and_then(pick))
        .map(str::to_owned)
}

/// 媒体位置给人看：向下取整，`102.5` 写 `01:42`，过一小时写 `1:01:42`。
pub fn format_position(sec: f64) -> String {
    let total = if sec.is_finite() && sec > 0.0 {
        sec.floor() as u64
    } else {
        0
    };
    let (h, m, s) = (total / 3600, total / 60 % 60, total % 60);
    if h > 0 {
        format!("{h}:{m:02}:{s:02}")
    } else {
        format!("{m:02}:{s:02}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canonical_form() {
        let ts = parse("2026-11-05T11:00:01.123456+08:00").unwrap();
        assert_eq!(format(ts), "2026-11-05T03:00:01.123Z");
        assert!(is_canonical("2026-11-05T03:00:01.123Z"));
        assert!(!is_canonical("2026-11-05T03:00:01Z"));
        assert_eq!(
            add_days("2026-11-05T03:00:01.123Z", 30).unwrap(),
            "2026-12-05T03:00:01.123Z"
        );
    }

    #[test]
    fn dates_and_positions() {
        assert_eq!(
            display_date(
                Some("2026-10-01T22:00:00+08:00"),
                Some("2026-10-01T14:00:00.000Z")
            ),
            Some("2026-10-01".into())
        );
        assert_eq!(
            display_date(None, Some("2026-09-30T20:00:00.000Z")),
            Some("2026-09-30".into())
        );
        assert_eq!(format_position(102.5), "01:42");
        assert_eq!(format_position(3702.0), "1:01:42");
        assert_eq!(format_position(42.0), "00:42");
    }
}
