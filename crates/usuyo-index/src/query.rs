//! 搜索词 → 查询。空白分开的每个词都要命中（且），
//! 一个词在三处任一命中即可：分词、二字组（兜底）、拼音。

use std::collections::HashSet;

use tantivy::Term;
use tantivy::query::{BooleanQuery, BoostQuery, Occur, Query, TermQuery};
use tantivy::schema::{Field, IndexRecordOption};

use crate::analyzer::{Mode, is_han, tokenize};
use crate::error::{Error, Result};
use crate::schema::Fields;

const MAX_TERMS: usize = 16;
const MAX_TERM_CHARS: usize = 128;
/// 拼音命中的权重：低于原文命中，避免 `sc` 这类缩写压过字面匹配。
const PINYIN_BOOST: f32 = 0.5;

/// 搜索词切出来的词，高亮时用来认原文里的命中。
#[derive(Default)]
pub(crate) struct Marks {
    pub words: HashSet<String>,
    pub grams: HashSet<String>,
    pub pinyin: HashSet<String>,
}

pub(crate) struct ParsedQuery {
    pub query: Box<dyn Query>,
    pub marks: Marks,
}

pub(crate) fn parse(f: &Fields, raw: &str) -> Result<ParsedQuery> {
    let terms: Vec<&str> = raw
        .split_whitespace()
        .map(|t| t.trim_matches('"'))
        .filter(|t| !t.is_empty())
        .collect();
    if terms.len() > MAX_TERMS || terms.iter().any(|t| t.chars().count() > MAX_TERM_CHARS) {
        return Err(Error::QueryTooLong {
            max_terms: MAX_TERMS,
            max_chars: MAX_TERM_CHARS,
        });
    }

    let mut marks = Marks::default();
    let mut required: Vec<(Occur, Box<dyn Query>)> = Vec::new();
    for term in terms {
        let words = unique(tokenize(Mode::WordsQuery, term));
        let grams = unique(tokenize(Mode::GramsQuery, term));
        let pinyin = term
            .is_ascii()
            .then(|| term.to_ascii_lowercase())
            .filter(|t| t.len() >= 2 && t.bytes().all(|b| b.is_ascii_lowercase()));

        let mut alternatives: Vec<(Occur, Box<dyn Query>)> = Vec::new();
        if let Some(q) = all_of(f.words, &words) {
            alternatives.push((Occur::Should, q));
        }
        // 二字组只管汉字部分，拉丁部分仍要在分词里命中。
        let latin: Vec<String> = words
            .iter()
            .filter(|w| !w.chars().any(is_han))
            .cloned()
            .collect();
        if let Some(han) = all_of(f.grams, &grams) {
            let both = [Some(han), all_of(f.words, &latin)]
                .into_iter()
                .flatten()
                .collect();
            alternatives.push((Occur::Should, and(both)));
        }
        if let Some(p) = &pinyin {
            let q = TermQuery::new(
                Term::from_field_text(f.pinyin, p),
                IndexRecordOption::WithFreqs,
            );
            alternatives.push((
                Occur::Should,
                Box::new(BoostQuery::new(Box::new(q), PINYIN_BOOST)),
            ));
        }
        if alternatives.is_empty() {
            continue; // 只有标点之类，没有可搜的词
        }
        marks.words.extend(words);
        marks.grams.extend(grams);
        marks.pinyin.extend(pinyin);
        required.push((Occur::Must, Box::new(BooleanQuery::new(alternatives))));
    }
    if required.is_empty() {
        return Err(Error::EmptyQuery);
    }
    Ok(ParsedQuery {
        query: Box::new(BooleanQuery::new(required)),
        marks,
    })
}

fn unique(tokens: Vec<(String, std::ops::Range<usize>)>) -> Vec<String> {
    let mut seen = HashSet::new();
    tokens
        .into_iter()
        .map(|(t, _)| t)
        .filter(|t| seen.insert(t.clone()))
        .collect()
}

/// 这些词在 `field` 里都要有。
fn all_of(field: Field, tokens: &[String]) -> Option<Box<dyn Query>> {
    let queries: Vec<Box<dyn Query>> = tokens
        .iter()
        .map(|t| {
            Box::new(TermQuery::new(
                Term::from_field_text(field, t),
                IndexRecordOption::WithFreqs,
            )) as Box<dyn Query>
        })
        .collect();
    (!queries.is_empty()).then(|| and(queries))
}

fn and(mut queries: Vec<Box<dyn Query>>) -> Box<dyn Query> {
    if queries.len() == 1 {
        return queries.remove(0);
    }
    Box::new(BooleanQuery::new(
        queries.into_iter().map(|q| (Occur::Must, q)).collect(),
    ))
}
