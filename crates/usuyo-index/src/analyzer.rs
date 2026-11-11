//! 中英文分词。一个分词器按 `Mode` 切出四种词：
//! - `Words`：jieba 搜索模式的词（索引用），拉丁词整词小写；
//! - `Grams`：汉字单字和相邻二字，保证两字词一定搜得到；
//! - `Pinyin`：汉字串里 1–4 个字的全拼，2–4 个字再加首字母；
//! - `*Query`：搜索词一侧的切法，比索引侧少切一些。
//!
//! 所有词的偏移都指向原文，高亮直接用。

use std::sync::OnceLock;

use jieba_rs::{Jieba, TokenizeMode};
use pinyin::ToPinyinMulti;
use tantivy::tokenizer::{
    AsciiFoldingFilter, LowerCaser, RemoveLongFilter, TextAnalyzer, Token, TokenStream, Tokenizer,
};

/// 分词器规则变了就加一，旧索引会被要求重建。
pub(crate) const ANALYZER_VERSION: u32 = 1;

/// 拼音窗口最长的字数。
const PINYIN_MAX_WINDOW: usize = 4;
/// 超长的词直接丢掉（网址、乱码）。
const MAX_TOKEN_BYTES: usize = 64;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Mode {
    Words,
    WordsQuery,
    Grams,
    GramsQuery,
    Pinyin,
}

pub(crate) fn analyzer(mode: Mode) -> TextAnalyzer {
    TextAnalyzer::builder(UsuyoTokenizer { mode })
        .filter(RemoveLongFilter::limit(MAX_TOKEN_BYTES))
        .filter(LowerCaser)
        .filter(AsciiFoldingFilter)
        .build()
}

/// 用某个分词器切一段文字，返回（小写折叠后的词，原文字节范围）。
pub(crate) fn tokenize(mode: Mode, text: &str) -> Vec<(String, std::ops::Range<usize>)> {
    let mut analyzer = analyzer(mode);
    let mut stream = analyzer.token_stream(text);
    let mut out = Vec::new();
    while stream.advance() {
        let t = stream.token();
        out.push((t.text.clone(), t.offset_from..t.offset_to));
    }
    out
}

fn jieba() -> &'static Jieba {
    static JIEBA: OnceLock<Jieba> = OnceLock::new();
    JIEBA.get_or_init(Jieba::new)
}

pub(crate) fn is_han(c: char) -> bool {
    matches!(c as u32,
        0x3400..=0x4DBF | 0x4E00..=0x9FFF | 0xF900..=0xFAFF | 0x20000..=0x2FA1F)
}

#[derive(Clone)]
struct UsuyoTokenizer {
    mode: Mode,
}

impl Tokenizer for UsuyoTokenizer {
    type TokenStream<'a> = PreparedTokens;

    fn token_stream<'a>(&'a mut self, text: &'a str) -> PreparedTokens {
        PreparedTokens {
            tokens: raw_tokens(self.mode, text),
            next: 0,
        }
    }
}

struct PreparedTokens {
    tokens: Vec<Token>,
    next: usize,
}

impl TokenStream for PreparedTokens {
    fn advance(&mut self) -> bool {
        if self.next < self.tokens.len() {
            self.next += 1;
            true
        } else {
            false
        }
    }

    fn token(&self) -> &Token {
        &self.tokens[self.next - 1]
    }

    fn token_mut(&mut self) -> &mut Token {
        &mut self.tokens[self.next - 1]
    }
}

struct Emitter {
    tokens: Vec<Token>,
}

impl Emitter {
    fn push(&mut self, text: &str, from: usize, to: usize) {
        let position = self.tokens.len();
        self.tokens.push(Token {
            offset_from: from,
            offset_to: to,
            position,
            text: text.to_owned(),
            position_length: 1,
        });
    }
}

fn raw_tokens(mode: Mode, text: &str) -> Vec<Token> {
    let mut out = Emitter { tokens: Vec::new() };
    let mut run_start = 0;
    let mut run_is_han = false;
    let mut in_run = false;
    for (i, c) in text.char_indices() {
        let word_char = c.is_alphanumeric();
        let han = is_han(c);
        // 汉字串和拉丁/数字串在边界处断开。
        if in_run && (!word_char || han != run_is_han) {
            emit_run(&mut out, mode, text, run_start, i, run_is_han);
            in_run = false;
        }
        if word_char && !in_run {
            run_start = i;
            run_is_han = han;
            in_run = true;
        }
    }
    if in_run {
        emit_run(&mut out, mode, text, run_start, text.len(), run_is_han);
    }
    out.tokens
}

fn emit_run(out: &mut Emitter, mode: Mode, text: &str, from: usize, to: usize, han: bool) {
    let run = &text[from..to];
    if !han {
        if matches!(mode, Mode::Words | Mode::WordsQuery) {
            out.push(run, from, to);
        }
        return;
    }
    // 第 k 个字的字节偏移（含末尾），换算 jieba 的字偏移。
    let offsets: Vec<usize> = run
        .char_indices()
        .map(|(i, _)| from + i)
        .chain(std::iter::once(to))
        .collect();
    let chars = offsets.len() - 1;
    match mode {
        Mode::Words | Mode::WordsQuery => {
            let jieba_mode = if mode == Mode::Words {
                TokenizeMode::Search
            } else {
                TokenizeMode::Default
            };
            for t in jieba().tokenize(run, jieba_mode, true) {
                out.push(t.word, offsets[t.start], offsets[t.end]);
            }
        }
        Mode::Grams => {
            for k in 0..chars {
                out.push(
                    &text[offsets[k]..offsets[k + 1]],
                    offsets[k],
                    offsets[k + 1],
                );
                if k + 1 < chars {
                    out.push(
                        &text[offsets[k]..offsets[k + 2]],
                        offsets[k],
                        offsets[k + 2],
                    );
                }
            }
        }
        Mode::GramsQuery => {
            if chars == 1 {
                out.push(run, from, to);
            } else {
                for k in 0..chars - 1 {
                    out.push(
                        &text[offsets[k]..offsets[k + 2]],
                        offsets[k],
                        offsets[k + 2],
                    );
                }
            }
        }
        Mode::Pinyin => emit_pinyin(out, run, &offsets),
    }
}

/// 多音字每个读音都收；一个窗口里的读音组合最多取这么多个。
const PINYIN_MAX_COMBOS: usize = 8;

/// 没有拼音的字（生僻字）当作断点，窗口不跨过它。
fn emit_pinyin(out: &mut Emitter, run: &str, offsets: &[usize]) {
    let readings: Vec<Vec<String>> = run
        .chars()
        .map(|c| {
            let mut all: Vec<String> = c
                .to_pinyin_multi()
                .into_iter()
                .flatten()
                .map(|p| p.plain().replace('ü', "v"))
                .collect();
            all.dedup();
            all
        })
        .collect();
    for start in 0..readings.len() {
        // 每个组合：(全拼, 首字母)
        let mut combos: Vec<(String, String)> = vec![(String::new(), String::new())];
        for (len, options) in readings[start..].iter().take(PINYIN_MAX_WINDOW).enumerate() {
            if options.is_empty() {
                break;
            }
            combos = combos
                .iter()
                .flat_map(|(full, initials)| {
                    options
                        .iter()
                        .map(move |s| (format!("{full}{s}"), format!("{initials}{}", &s[..1])))
                })
                .take(PINYIN_MAX_COMBOS)
                .collect();
            let (from, to) = (offsets[start], offsets[start + len + 1]);
            for (full, initials) in &combos {
                out.push(full, from, to);
                if len >= 1 {
                    out.push(initials, from, to);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn texts(mode: Mode, s: &str) -> Vec<String> {
        tokenize(mode, s).into_iter().map(|(t, _)| t).collect()
    }

    #[test]
    fn words_mixed() {
        let t = texts(Mode::Words, "手冲咖啡用 V60 滤杯，Café");
        assert!(
            t.contains(&"手冲".into()) || t.contains(&"咖啡".into()),
            "{t:?}"
        );
        assert!(t.contains(&"v60".into()));
        assert!(t.contains(&"cafe".into()));
    }

    #[test]
    fn grams_cover_two_chars() {
        assert_eq!(
            texts(Mode::Grams, "手冲咖"),
            ["手", "手冲", "冲", "冲咖", "咖"]
        );
        assert_eq!(texts(Mode::GramsQuery, "手冲咖"), ["手冲", "冲咖"]);
        assert_eq!(texts(Mode::GramsQuery, "手"), ["手"]);
    }

    #[test]
    fn pinyin_windows() {
        let t = texts(Mode::Pinyin, "手冲");
        assert!(t.contains(&"shouchong".into()));
        assert!(t.contains(&"sc".into()));
        assert!(t.contains(&"shou".into()));
    }

    #[test]
    fn offsets_point_into_original_text() {
        let text = "ＡＢ手冲 Café";
        for mode in [Mode::Words, Mode::Grams, Mode::Pinyin] {
            for (_, r) in tokenize(mode, text) {
                assert!(text.is_char_boundary(r.start) && text.is_char_boundary(r.end));
            }
        }
    }
}
