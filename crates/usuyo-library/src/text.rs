//! 文本规则（SPEC §0.6）：NFC、转义、文件名。

use unicode_normalization::UnicodeNormalization;
use unicode_segmentation::UnicodeSegmentation;

/// NFC。只用于 id、文件名、front matter 键和我们生成的字；平台原文照收到的存。
pub fn nfc(s: &str) -> String {
    s.nfc().collect()
}

/// 查重用的键：NFC 加小写。
pub fn fold(s: &str) -> String {
    nfc(s).to_lowercase()
}

/// YAML 双引号标量。
pub fn yaml_quote(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for c in s.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\t' => out.push_str("\\t"),
            '\r' => out.push_str("\\r"),
            c if c.is_control() => out.push_str(&format!("\\u{:04X}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

/// Markdown 链接文字和原话：转义 `[ ] ( ) < > * _ ` \ #`，换行合成空格。
pub fn md_inline(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut last_space = false;
    for c in s.chars() {
        if c == '\n' || c == '\r' {
            if !last_space {
                out.push(' ');
            }
            last_space = true;
            continue;
        }
        last_space = false;
        if matches!(
            c,
            '[' | ']' | '(' | ')' | '<' | '>' | '*' | '_' | '`' | '\\' | '#'
        ) {
            out.push('\\');
        }
        out.push(c);
    }
    out
}

/// [`md_inline`] 的反向：去掉反斜杠转义。
pub fn md_unescape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars();
    while let Some(c) = chars.next() {
        if c == '\\' {
            if let Some(n) = chars.next() {
                out.push(n);
            }
        } else {
            out.push(c);
        }
    }
    out
}

/// 读到没被转义的 `end` 为止，回 `end` 前后两段。
pub(crate) fn take_escaped<'a>(s: &'a str, end: &str) -> Option<(&'a str, &'a str)> {
    let mut escaped = false;
    for (i, c) in s.char_indices() {
        if escaped {
            escaped = false;
        } else if c == '\\' {
            escaped = true;
        } else if s[i..].starts_with(end) {
            return Some((&s[..i], &s[i + end.len()..]));
        }
    }
    None
}

/// `textFormat: plain` 的正文写进 Markdown：渲染出来仍是纯文字。
pub fn plain_to_markdown(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + s.len() / 8);
    for (i, line) in s.split('\n').enumerate() {
        if i > 0 {
            out.push('\n');
        }
        let digits = line.bytes().take_while(u8::is_ascii_digit).count();
        let numbered = digits > 0 && line.as_bytes().get(digits) == Some(&b'.');
        let first_special = line.starts_with(['-', '+', '=']);
        for (j, c) in line.char_indices() {
            match c {
                '\\' | '`' | '*' | '_' | '[' | ']' | '<' | '>' | '#' | '!' | '|' | '~' => {
                    out.push('\\');
                    out.push(c);
                }
                '&' => out.push_str("&amp;"),
                _ if j == 0 && first_special => {
                    out.push('\\');
                    out.push(c);
                }
                '.' if numbered && j == digits => out.push_str("\\."),
                _ => out.push(c),
            }
        }
    }
    out
}

/// `<…>` 里的链接：空格、`<`、`>` 百分号编码。
pub fn angle_url(url: &str) -> String {
    url.replace(' ', "%20")
        .replace('<', "%3C")
        .replace('>', "%3E")
}

/// 每段文件名不超过的字节数。
pub const SEGMENT_MAX_BYTES: usize = 200;
/// 标题截断时给扩展名（`.md`）和本机后缀（` 2`）留的字节。
const TITLE_RESERVE_BYTES: usize = 11;

const RESERVED: &[&str] = &[
    "con", "prn", "aux", "nul", "com1", "com2", "com3", "com4", "com5", "com6", "com7", "com8",
    "com9", "lpt1", "lpt2", "lpt3", "lpt4", "lpt5", "lpt6", "lpt7", "lpt8", "lpt9",
];

/// 标题来的文件名（不含扩展名）。
pub fn sanitize(title: &str) -> String {
    let mut s = String::new();
    for c in nfc(title).chars() {
        let c = match c {
            '<' => '＜',
            '>' => '＞',
            ':' => '：',
            '"' => '＂',
            '/' => '／',
            '\\' => '＼',
            '|' => '｜',
            '?' => '？',
            '*' => '＊',
            c if c.is_control() => continue,
            c => c,
        };
        s.push(c);
    }
    let mut s = trim_name(&s).to_owned();
    // 按字形截断，留出扩展名和后缀的位置。
    let budget = SEGMENT_MAX_BYTES - TITLE_RESERVE_BYTES;
    if s.len() > budget {
        let mut cut = String::new();
        for g in s.graphemes(true) {
            if cut.len() + g.len() > budget {
                break;
            }
            cut.push_str(g);
        }
        s = trim_name(&cut).to_owned();
    }
    if s.is_empty() {
        s.push('_');
    }
    let stem = s.split('.').next().unwrap_or_default().to_lowercase();
    if RESERVED.contains(&stem.as_str()) {
        s.insert(0, '_');
    }
    s
}

fn trim_name(s: &str) -> &str {
    s.trim_start_matches('.').trim_end_matches([' ', '.'])
}

/// 从库根起的相对路径不超过 200 个 UTF-16 单位。
pub fn rel_path_fits(rel: &str, max_utf16: usize) -> bool {
    rel.encode_utf16().count() <= max_utf16
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sanitize_edge_cases() {
        assert_eq!(sanitize("a/b: c?"), "a／b： c？");
        assert_eq!(sanitize("...hidden. . "), "hidden");
        assert_eq!(sanitize("CON"), "_CON");
        assert_eq!(sanitize("nul.txt"), "_nul.txt");
        assert_eq!(sanitize("console"), "console");
        assert_eq!(sanitize("tab\there\n"), "tabhere");
        assert_eq!(sanitize("# ^ [x]"), "# ^ [x]");
        assert_eq!(sanitize("..."), "_");
        // 约 64 个汉字
        let long = sanitize(&"汉".repeat(100));
        assert_eq!(long.chars().count(), 63);
        assert!(long.len() + ".md".len() + " 2".len() <= SEGMENT_MAX_BYTES);
        // 不在组合字形中间截断
        let emoji = sanitize(&"👨‍👩‍👧".repeat(30));
        assert!(emoji.graphemes(true).all(|g| g == "👨‍👩‍👧"));
        // NFC：分解的 é 合成一个
        assert_eq!(sanitize("e\u{301}"), "é");
    }

    #[test]
    fn escapes() {
        assert_eq!(yaml_quote("a\"b\\c\nd"), "\"a\\\"b\\\\c\\nd\"");
        assert_eq!(md_inline("[x](y)\n*_#"), "\\[x\\]\\(y\\) \\*\\_\\#");
        assert_eq!(md_unescape(&md_inline("a [b] \\ c")), "a [b] \\ c");
        assert_eq!(
            plain_to_markdown("- a\n1. b\n# c & d\n10x"),
            "\\- a\n1\\. b\n\\# c &amp; d\n10x"
        );
        assert_eq!(angle_url("https://a.b/c d<e>"), "https://a.b/c%20d%3Ce%3E");
        assert_eq!(fold("ÀB"), "àb");
    }
}
