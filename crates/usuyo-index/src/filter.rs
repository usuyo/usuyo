use std::ops::Bound;

use tantivy::Term;
use tantivy::query::{BooleanQuery, ConstScoreQuery, Occur, Query, RangeQuery, TermQuery};
use tantivy::schema::{Field, IndexRecordOption};

use crate::schema::Fields;

/// 谁看得到私密条目。默认只给公开的：Agent 一侧永远不要改这个值。
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Visibility {
    #[default]
    PublicOnly,
    IncludePrivate,
    PrivateOnly,
}

/// Unix 秒的半开区间 `[from, to)`；两端都空表示不限。没有这个时间的条目在限定后不会命中。
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct DateRange {
    pub from: Option<i64>,
    pub to: Option<i64>,
}

/// 同一项里的多个值是「或」，不同项之间是「且」；空表示不限。
#[derive(Debug, Clone, Default)]
pub struct Filter {
    pub platforms: Vec<String>,
    pub kinds: Vec<String>,
    pub tags: Vec<String>,
    pub topics: Vec<String>,
    /// 层名；填 `"meta"` 可只在标题、作者、标签、主题里找。
    pub layers: Vec<String>,
    pub published: DateRange,
    pub added: DateRange,
    pub visibility: Visibility,
}

impl Filter {
    /// 每个筛选项一条不计分的查询，调用方全部 `Must`。
    pub(crate) fn to_queries(&self, f: &Fields) -> Vec<Box<dyn Query>> {
        let mut out = Vec::new();
        for (field, values) in [
            (f.platform, &self.platforms),
            (f.kind, &self.kinds),
            (f.tags, &self.tags),
            (f.topics, &self.topics),
            (f.layer, &self.layers),
        ] {
            if let Some(q) = any_of(field, values) {
                out.push(unscored(q));
            }
        }
        for (field, range) in [(f.published_at, self.published), (f.added_at, self.added)] {
            if range.from.is_some() || range.to.is_some() {
                let lower = match range.from {
                    Some(v) => Bound::Included(Term::from_field_i64(field, v)),
                    None => Bound::Unbounded,
                };
                let upper = match range.to {
                    Some(v) => Bound::Excluded(Term::from_field_i64(field, v)),
                    None => Bound::Unbounded,
                };
                out.push(unscored(Box::new(RangeQuery::new(lower, upper))));
            }
        }
        let private = match self.visibility {
            Visibility::PublicOnly => Some(false),
            Visibility::PrivateOnly => Some(true),
            Visibility::IncludePrivate => None,
        };
        if let Some(value) = private {
            out.push(unscored(Box::new(TermQuery::new(
                Term::from_field_bool(f.private, value),
                IndexRecordOption::Basic,
            ))));
        }
        out
    }
}

fn any_of(field: Field, values: &[String]) -> Option<Box<dyn Query>> {
    let mut terms: Vec<Box<dyn Query>> = values
        .iter()
        .map(|v| {
            Box::new(TermQuery::new(
                Term::from_field_text(field, v),
                IndexRecordOption::Basic,
            )) as Box<dyn Query>
        })
        .collect();
    match terms.len() {
        0 => None,
        1 => terms.pop(),
        _ => Some(Box::new(BooleanQuery::new(
            terms.into_iter().map(|q| (Occur::Should, q)).collect(),
        ))),
    }
}

/// 筛选只决定在不在，不影响排序。
fn unscored(q: Box<dyn Query>) -> Box<dyn Query> {
    Box::new(ConstScoreQuery::new(q, 0.0))
}
