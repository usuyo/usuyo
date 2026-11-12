//! 子进程的时间上限：按媒体时长算，时长不明时用固定的一小时。

use std::time::Duration;

/// 时长不明时的上限。
const UNKNOWN: Duration = Duration::from_secs(3600);
/// 再长的媒体也不超过这个。
const CEILING: Duration = Duration::from_secs(6 * 3600);

/// `base_sec` 加上每秒媒体 `per_media_sec` 秒。
pub fn timeout(duration_sec: Option<f64>, base_sec: f64, per_media_sec: f64) -> Duration {
    match duration_sec {
        Some(d) => Duration::try_from_secs_f64(base_sec + d * per_media_sec)
            .map_or(CEILING, |t| t.min(CEILING)),
        None => UNKNOWN,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scales_with_the_media() {
        assert_eq!(timeout(Some(100.0), 60.0, 0.5), Duration::from_secs(110));
        assert_eq!(timeout(Some(1e12), 60.0, 2.0), CEILING);
        assert_eq!(timeout(None, 60.0, 2.0), UNKNOWN);
        assert_eq!(timeout(Some(f64::NAN), 60.0, 2.0), CEILING);
    }
}
