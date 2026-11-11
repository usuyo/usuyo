//! front matter 的 YAML 子集（SPEC §0.6）：同一个对象每台写出一样的字节。
//!
//! 值只有三种：双引号字符串、`true`（`false` 不写）、一行一项的字符串列表。
//! 认识的键照 `KEYS` 的顺序，不认识的按键名排在后面；列表按字符串排；空列表不写。

use std::path::Path;

use serde_json::{Map, Value};
use usuyo_contract::FrontMatter;

use crate::text::yaml_quote;
use crate::{Error, Result};

/// 对象写成 front matter 的行（不含 `---`）；没有要写的键时是空串。
pub fn render_meta<T: FrontMatter>(meta: &T) -> Result<String> {
    let value = serde_json::to_value(meta).map_err(|e| Error::InvalidArgs(e.to_string()))?;
    let Value::Object(map) = value else {
        return Err(Error::InvalidArgs("front matter must be an object".into()));
    };
    render_map(&map, T::KEYS)
}

/// 同 [`render_meta`]，对已经是 JSON 对象的。
pub fn render_map(map: &Map<String, Value>, keys: &[&str]) -> Result<String> {
    let mut order: Vec<&String> = Vec::with_capacity(map.len());
    for k in keys {
        if let Some((key, _)) = map.get_key_value(*k) {
            order.push(key);
        }
    }
    // serde_json 的 Map 默认按键名排，其余的直接按这个顺序。
    order.extend(map.keys().filter(|k| !keys.contains(&k.as_str())));
    let mut out = String::new();
    for key in order {
        if !is_key(key) {
            return Err(Error::InvalidArgs(format!("bad front matter key `{key}`")));
        }
        match &map[key] {
            Value::Null | Value::Bool(false) => {}
            Value::Bool(true) => out.push_str(&format!("{key}: true\n")),
            Value::String(s) => out.push_str(&format!("{key}: {}\n", yaml_quote(s))),
            Value::Array(items) if items.is_empty() => {}
            Value::Array(items) => {
                let mut strings = Vec::with_capacity(items.len());
                for item in items {
                    match item {
                        Value::String(s) => strings.push(s.as_str()),
                        _ => {
                            return Err(Error::InvalidArgs(format!(
                                "front matter list `{key}` must hold strings"
                            )));
                        }
                    }
                }
                strings.sort_unstable();
                strings.dedup();
                out.push_str(&format!("{key}:\n"));
                for s in strings {
                    out.push_str(&format!("  - {}\n", yaml_quote(s)));
                }
            }
            Value::Number(_) | Value::Object(_) => {
                return Err(Error::InvalidArgs(format!(
                    "front matter `{key}` must be a string, true or a list"
                )));
            }
        }
    }
    Ok(out)
}

fn is_key(k: &str) -> bool {
    let mut chars = k.chars();
    chars.next().is_some_and(|c| c.is_ascii_alphabetic())
        && chars.all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
}

/// 读 front matter 的行（不含 `---`）。不合这套写法的就是写坏了。
pub fn parse_map(path: &Path, text: &str) -> Result<Map<String, Value>> {
    let bad =
        |line: usize, why: &str| Error::malformed(path, format!("front matter line {line}: {why}"));
    let mut map = Map::new();
    let mut list: Option<(String, Vec<Value>)> = None;
    for (i, line) in text.lines().enumerate() {
        let n = i + 1;
        if let Some(item) = line.strip_prefix("  - ") {
            let (_, items) = list
                .as_mut()
                .ok_or_else(|| bad(n, "list item without key"))?;
            items.push(Value::String(
                unquote(item).ok_or_else(|| bad(n, "bad string"))?,
            ));
            continue;
        }
        if let Some((k, items)) = list.take() {
            if items.is_empty() {
                return Err(bad(n, "empty list"));
            }
            map.insert(k, Value::Array(items));
        }
        let (key, rest) = line
            .split_once(':')
            .ok_or_else(|| bad(n, "expected `key: value`"))?;
        if !is_key(key) {
            return Err(bad(n, "bad key"));
        }
        if map.contains_key(key) {
            return Err(bad(n, "duplicate key"));
        }
        let value = match rest {
            "" => {
                list = Some((key.to_owned(), Vec::new()));
                continue;
            }
            " true" => Value::Bool(true),
            " false" => Value::Bool(false),
            r => match r.strip_prefix(' ').and_then(unquote) {
                Some(s) => Value::String(s),
                None => {
                    return Err(bad(
                        n,
                        "values must be double-quoted strings, true or a list",
                    ));
                }
            },
        };
        map.insert(key.to_owned(), value);
    }
    if let Some((k, items)) = list.take() {
        if items.is_empty() {
            return Err(bad(text.lines().count(), "empty list"));
        }
        map.insert(k, Value::Array(items));
    }
    Ok(map)
}

/// 双引号标量；只认我们写出的那几种转义。
fn unquote(s: &str) -> Option<String> {
    let inner = s.strip_prefix('"')?.strip_suffix('"')?;
    let mut out = String::with_capacity(inner.len());
    let mut chars = inner.chars();
    while let Some(c) = chars.next() {
        match c {
            '"' => return None,
            '\\' => match chars.next()? {
                '\\' => out.push('\\'),
                '"' => out.push('"'),
                'n' => out.push('\n'),
                't' => out.push('\t'),
                'r' => out.push('\r'),
                'u' => {
                    let hex: String = chars.by_ref().take(4).collect();
                    let code = u32::from_str_radix(&hex, 16).ok()?;
                    out.push(char::from_u32(code)?);
                }
                _ => return None,
            },
            c => out.push(c),
        }
    }
    Some(out)
}

/// 拆开 `---\n…\n---\n` 和正文；没有 front matter 时第一项是 `None`。
pub fn split(path: &Path, text: &str) -> Result<(Option<String>, String)> {
    let Some(rest) = text.strip_prefix("---\n") else {
        return Ok((None, text.to_owned()));
    };
    let mut offset = 0;
    for line in rest.split_inclusive('\n') {
        if line == "---\n" || line == "---" {
            let fm = rest[..offset].to_owned();
            let body = rest[offset + line.len()..].to_owned();
            return Ok((Some(fm), body));
        }
        offset += line.len();
    }
    Err(Error::malformed(path, "front matter is not closed"))
}

/// 拼回整份：有键才写 `---` 块。
pub fn join(meta_lines: &str, body: &str) -> String {
    // 正文自己以 `---` 开头时，空的 front matter 也要写出来，不然读回时会被当成 front matter。
    if meta_lines.is_empty() && !(body.starts_with("---\n") || body == "---") {
        body.to_owned()
    } else {
        format!("---\n{meta_lines}---\n{body}")
    }
}

/// 读成类型；时间字段要是 usuyo 的写法（SPEC §0.5）。
pub fn parse_meta<T: FrontMatter>(path: &Path, lines: Option<&str>) -> Result<T> {
    let map = match lines {
        Some(l) => parse_map(path, l)?,
        None => Map::new(),
    };
    if let Some(Value::String(t)) = map.get("createdAt")
        && !crate::time::is_canonical(t)
    {
        return Err(Error::malformed(
            path,
            format!("createdAt `{t}` is not UTC ms"),
        ));
    }
    serde_json::from_value(Value::Object(map)).map_err(|e| Error::malformed(path, e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use usuyo_contract::{NotesMeta, TopicMeta};

    const TOPIC: &str = "---\nid: \"0193f2a4-7c1e-7a3b-9d2e-5f0a1b2c3d4e\"\ntitle: \"手冲咖啡\"\ncreatedAt: \"2026-11-05T03:00:01.123Z\"\nfollowed: true\nkeywords:\n  - \"手冲\"\n  - \"闷蒸\"\nmatch:\n  - \"keywords\"\n---\n闷蒸决定后面三段注水均不均匀。[^k3f7q]\n\n[^k3f7q]: 「第一段 40 克闷蒸 30 秒」[手冲咖啡：三段式注水 · 口播 00:42](usuyo://item/douyin-4plrcxk343onmyxibmrr672kga/asr@0193f2b0-1d2e-7f3a-8b4c-5d6e7f8a9b0c#t=42,48) · 咖啡日常 · 2026-10-01\n";

    #[test]
    fn spec_topic_round_trips_byte_for_byte() {
        let p = Path::new("t.md");
        let (fm, body) = split(p, TOPIC).unwrap();
        let meta: TopicMeta = parse_meta(p, fm.as_deref()).unwrap();
        assert!(meta.followed);
        assert_eq!(meta.keywords, vec!["手冲", "闷蒸"]);
        assert_eq!(join(&render_meta(&meta).unwrap(), &body), TOPIC);
    }

    #[test]
    fn unknown_keys_sort_after_known_and_survive() {
        let p = Path::new("t.md");
        let text = "---\nid: \"0193f2a4-7c1e-7a3b-9d2e-5f0a1b2c3d4e\"\ntitle: \"t\"\ncreatedAt: \"2026-11-05T03:00:01.123Z\"\naaa: \"1\"\nzzz:\n  - \"b\"\n---\n";
        let (fm, body) = split(p, text).unwrap();
        let meta: TopicMeta = parse_meta(p, fm.as_deref()).unwrap();
        assert_eq!(meta.extra.len(), 2);
        assert_eq!(join(&render_meta(&meta).unwrap(), &body), text);
    }

    #[test]
    fn false_and_empty_are_omitted_and_lists_sorted() {
        let mut meta: TopicMeta = serde_json::from_value(serde_json::json!({
            "id": "0193f2a4-7c1e-7a3b-9d2e-5f0a1b2c3d4e", "title": "引号\"和\\反斜杠\n换行",
            "createdAt": "2026-11-05T03:00:01.123Z", "followed": false,
            "keywords": ["b", "a", "no"], "match": []
        }))
        .unwrap();
        let out = render_meta(&meta).unwrap();
        assert_eq!(
            out,
            "id: \"0193f2a4-7c1e-7a3b-9d2e-5f0a1b2c3d4e\"\ntitle: \"引号\\\"和\\\\反斜杠\\n换行\"\ncreatedAt: \"2026-11-05T03:00:01.123Z\"\nkeywords:\n  - \"a\"\n  - \"b\"\n  - \"no\"\n"
        );
        let back: TopicMeta = parse_meta(Path::new("x"), Some(&out)).unwrap();
        meta.keywords.sort();
        assert_eq!(back, meta);
    }

    #[test]
    fn notes_without_takeaway_have_no_front_matter() {
        assert_eq!(render_meta(&NotesMeta::default()).unwrap(), "");
        let (fm, body) = split(Path::new("n"), "你的笔记\n").unwrap();
        assert!(fm.is_none());
        assert_eq!(body, "你的笔记\n");
    }

    #[test]
    fn body_starting_with_a_rule_survives() {
        let p = Path::new("n.md");
        let out = join(
            &render_meta(&NotesMeta::default()).unwrap(),
            "---\n分隔线下面\n",
        );
        assert_eq!(out, "---\n---\n---\n分隔线下面\n");
        let (fm, body) = split(p, &out).unwrap();
        let meta: NotesMeta = parse_meta(p, fm.as_deref()).unwrap();
        assert_eq!(meta, NotesMeta::default());
        assert_eq!(body, "---\n分隔线下面\n");
        assert_eq!(join(&render_meta(&meta).unwrap(), &body), out);
    }

    #[test]
    fn rejects_what_usuyo_would_not_write() {
        let p = Path::new("x");
        for bad in [
            "title: 手冲",
            "title: 'x'",
            "n: 3",
            "a: \"1\"\na: \"2\"",
            "list:\n  - x",
            "  - \"orphan\"",
            "k: \"bad \\q escape\"",
            "x: {a: 1}",
        ] {
            assert!(parse_map(p, bad).is_err(), "{bad}");
        }
        assert!(parse_meta::<TopicMeta>(p, Some("id: \"0193f2a4-7c1e-7a3b-9d2e-5f0a1b2c3d4e\"\ntitle: \"t\"\ncreatedAt: \"2026-11-05 03:00\"\n")).is_err());
        assert!(split(p, "---\nid: \"x\"\n").is_err());
    }
}
