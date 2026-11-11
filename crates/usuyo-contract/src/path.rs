//! 记录里的相对路径（`usuyo.fetch.json`、`run.json`、`keyframes.json`、回收站的 `entry.json`）。
//! 这些文件来自插件，也可能在 usuyo 外被改过；读的一方拿它们拼路径之前先查。

use std::path::{Component, Path};

/// 只许普通的段：不许空、绝对路径、`.`、`..`、盘符。
pub fn is_plain_relative(p: &str) -> bool {
    !p.is_empty()
        && Path::new(p)
            .components()
            .all(|c| matches!(c, Component::Normal(_)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plain_relative_paths() {
        assert!(is_plain_relative("media/video.m4s"));
        assert!(is_plain_relative("derived/runs/x/keyframes/00000500.jpg"));
        for bad in ["", "/etc/passwd", "../x", "a/../../b", "./a", ".."] {
            assert!(!is_plain_relative(bad), "{bad}");
        }
    }
}
