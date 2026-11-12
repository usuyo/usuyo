//! 从你粘贴、分享进来的一段文字里找链接：App 的分享文字常是「标题 + 链接 + 一句提示」，链接后面紧跟全角标点。
//! 收下、投递、交一批、从链接开始的来源、收件文件夹都经这里，插件拿到的总是一条干净的链接。

/// 一段链接到哪为止：空白、控制字符、非 ASCII、引号、尖括号。
fn run_end(rest: &str) -> usize {
    rest.find(|c: char| {
        c.is_whitespace()
            || c.is_control()
            || !c.is_ascii()
            || matches!(c, '"' | '\'' | '<' | '>' | '`')
    })
    .unwrap_or(rest.len())
}

/// 去掉链接末尾实际上不属于它的 ASCII 标点。
fn trim_tail(run: &str) -> &str {
    run.trim_end_matches(['.', ',', ';', ':', '!', '?', ')', ']', '}'])
}

fn http_with_host(text: &str) -> Option<url::Url> {
    let u = url::Url::parse(text).ok()?;
    (matches!(u.scheme(), "http" | "https") && u.host_str().is_some()).then_some(u)
}

/// 文字里以 `http://`、`https://`（不分大小写）开头的每一段，照出现的顺序，末尾的标点去掉。
pub(crate) fn link_runs(text: &str) -> impl Iterator<Item = &str> {
    // 只改 ASCII 的大小写，字节位置不变
    let lower = text.to_ascii_lowercase();
    let mut pos = 0;
    std::iter::from_fn(move || {
        let start = [lower[pos..].find("https://"), lower[pos..].find("http://")]
            .into_iter()
            .flatten()
            .min()
            .map(|i| pos + i)?;
        let end = start + run_end(&text[start..]);
        pos = end.max(start + 1);
        Some(trim_tail(&text[start..end]))
    })
}

/// 你给的一段文字里的链接：整段就是一条 http(s) 链接就用它；不然取文字里第一段认得出的 http(s) 链接。
/// 没有回 `None`。用户名、密码、长度由调用方再核。
pub fn link_in(text: &str) -> Option<url::Url> {
    let whole = text.trim();
    if !whole.chars().any(char::is_whitespace)
        && let Some(u) = http_with_host(whole)
    {
        return Some(u);
    }
    link_runs(text).find_map(http_with_host)
}

/// 文字里有几段像链接的（拿来在没收下时说「只看第一条」）。
pub fn link_count(text: &str) -> usize {
    link_runs(text).count()
}

#[cfg(test)]
mod tests {
    use super::link_in;

    fn got(text: &str) -> Option<String> {
        link_in(text).map(|u| u.to_string())
    }

    #[test]
    fn share_texts_give_their_link() {
        assert_eq!(
            got("3.05 复制打开抖音，看看【某某的作品】… https://v.douyin.com/iRNBho6u/ 复制此链接，打开Dou音搜索，直接观看视频！").as_deref(),
            Some("https://v.douyin.com/iRNBho6u/")
        );
        assert_eq!(
            got("【标题】 https://b23.tv/AbC12 ").as_deref(),
            Some("https://b23.tv/AbC12")
        );
        assert_eq!(
            got("某某发布了一篇小红书笔记，快来看吧！ http://xhslink.com/a/AbCdEf，复制本条信息，打开【小红书】App查看精彩内容！").as_deref(),
            Some("http://xhslink.com/a/AbCdEf")
        );
    }

    #[test]
    fn links_inside_text_parens_and_twice() {
        assert_eq!(
            got("看这个https://example.com/a?b=1复制打开").as_deref(),
            Some("https://example.com/a?b=1")
        );
        assert_eq!(
            got("（见 (https://a.test/x).）").as_deref(),
            Some("https://a.test/x")
        );
        assert_eq!(
            got("先 https://a.test/1 再 https://b.test/2").as_deref(),
            Some("https://a.test/1"),
            "取第一条"
        );
        assert_eq!(
            got("HTTP://Example.com/x y").as_deref(),
            Some("http://example.com/x")
        );
        assert_eq!(
            got("链接：\"https://a.test/x\"").as_deref(),
            Some("https://a.test/x")
        );
        assert_eq!(
            got("<https://a.test/y>").as_deref(),
            Some("https://a.test/y")
        );
        assert_eq!(
            got("https:// 坏的 https://ok.test/").as_deref(),
            Some("https://ok.test/")
        );
    }

    #[test]
    fn no_link_no_answer() {
        assert!(got("没有链接").is_none());
        assert!(got("https://").is_none());
        assert!(got("ftp://a.test/x").is_none());
        assert!(got("").is_none());
    }
}
