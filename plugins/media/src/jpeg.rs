//! 读 JPEG 文件头里的宽高（SOF 段），不解码图像。

/// 宽、高；不是 JPEG 或头不完整时 `None`。
pub fn dimensions(bytes: &[u8]) -> Option<(u32, u32)> {
    if bytes.get(..2)? != [0xFF, 0xD8] {
        return None;
    }
    let mut at = 2;
    loop {
        // 段之间可能有填充的 0xFF。
        while *bytes.get(at)? == 0xFF && *bytes.get(at + 1)? == 0xFF {
            at += 1;
        }
        if *bytes.get(at)? != 0xFF {
            return None;
        }
        let marker = *bytes.get(at + 1)?;
        at += 2;
        match marker {
            // 没有长度的标记：TEM、RSTn、SOI。
            0x01 | 0xD0..=0xD7 | 0xD8 => continue,
            // 数据开始或文件结束，还没见到 SOF。
            0xD9 | 0xDA => return None,
            _ => {}
        }
        let len = usize::from(u16::from_be_bytes([*bytes.get(at)?, *bytes.get(at + 1)?]));
        if len < 2 {
            return None;
        }
        // SOF0–SOF15，除去 DHT（C4）、JPG（C8）、DAC（CC）。
        if (0xC0..=0xCF).contains(&marker) && ![0xC4, 0xC8, 0xCC].contains(&marker) {
            let h = u16::from_be_bytes([*bytes.get(at + 3)?, *bytes.get(at + 4)?]);
            let w = u16::from_be_bytes([*bytes.get(at + 5)?, *bytes.get(at + 6)?]);
            return (w > 0 && h > 0).then_some((u32::from(w), u32::from(h)));
        }
        at += len;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn jpeg(app_len: u16, w: u16, h: u16) -> Vec<u8> {
        let mut v = vec![0xFF, 0xD8, 0xFF, 0xE0];
        v.extend_from_slice(&app_len.to_be_bytes());
        v.extend(std::iter::repeat_n(0u8, usize::from(app_len) - 2));
        v.extend_from_slice(&[0xFF, 0xC0, 0, 17, 8]);
        v.extend_from_slice(&h.to_be_bytes());
        v.extend_from_slice(&w.to_be_bytes());
        v.extend_from_slice(&[3, 1, 0x22, 0, 2, 0x11, 1, 3, 0x11, 1]);
        v
    }

    #[test]
    fn reads_size_after_other_segments() {
        assert_eq!(dimensions(&jpeg(16, 960, 540)), Some((960, 540)));
        assert_eq!(dimensions(&jpeg(2, 1, 65535)), Some((1, 65535)));
    }

    #[test]
    fn rejects_everything_else() {
        assert_eq!(dimensions(b""), None);
        assert_eq!(dimensions(b"\x89PNG\r\n\x1a\n"), None);
        assert_eq!(dimensions(&[0xFF, 0xD8]), None);
        let mut cut = jpeg(16, 960, 540);
        cut.truncate(cut.len() - 14);
        assert_eq!(dimensions(&cut), None);
        assert_eq!(dimensions(&jpeg(16, 0, 540)), None);
        // 数据开始前没有 SOF。
        assert_eq!(dimensions(&[0xFF, 0xD8, 0xFF, 0xDA, 0, 2]), None);
    }
}
