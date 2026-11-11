use tantivy::query::{BooleanQuery, Occur, Query};
use tantivy::schema::{TantivyDocument, Value};
use tantivy::{Score, Searcher};

use crate::collect::{ItemAcc, ItemCollector};
use crate::error::{Error, Result};
use crate::filter::Filter;
use crate::query::{self, Marks};
use crate::snippet::{self, Snippet};
use crate::store::SearchIndex;

const MAX_LIMIT: usize = 200;
const MAX_SEGMENTS_PER_ITEM: usize = 20;

#[derive(Debug, Clone)]
pub struct SearchRequest {
    /// 空白分开的词，每个都要命中；中文、英文、拼音、拼音首字母都行。
    pub query: String,
    pub filter: Filter,
    /// 一页几个条目（1–200）。
    pub limit: usize,
    /// 每个条目带几段最相关的命中（1–20）。
    pub segments_per_item: usize,
    /// 上一页返回的 `next_cursor`。
    pub cursor: Option<String>,
}

impl Default for SearchRequest {
    fn default() -> Self {
        Self {
            query: String::new(),
            filter: Filter::default(),
            limit: 20,
            segments_per_item: 3,
            cursor: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct SearchPage {
    pub items: Vec<ItemHit>,
    /// 还有下一页时给出；传回 `SearchRequest::cursor`。
    /// 游标记的是排序位置，翻页期间索引有写入时，边上的条目可能重复或漏掉一个。
    pub next_cursor: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ItemHit {
    pub item_id: String,
    pub platform: String,
    pub kind: String,
    pub title: String,
    pub author: String,
    /// 条目里最高的段分数。
    pub score: f32,
    /// 最相关的几段，分数从高到低。
    pub segments: Vec<SegmentHit>,
}

#[derive(Debug, Clone)]
pub struct SegmentHit {
    /// 层名；标题、作者、标签、主题那一段是 `"meta"`。
    pub layer: String,
    pub snapshot_id: Option<String>,
    /// 收下时调用方给的定位串，原样返回。
    pub locator: String,
    pub snippet: Snippet,
    pub score: f32,
}

impl SearchIndex {
    /// 搜索。流程分三步：找出命中的段并按条目聚合（`rank_items`）、
    /// 排序翻页、取出文字做高亮（`hydrate`）。
    pub fn search(&self, req: &SearchRequest) -> Result<SearchPage> {
        let parsed = query::parse(&self.fields, &req.query)?;
        let limit = req.limit.clamp(1, MAX_LIMIT);
        let per_item = req.segments_per_item.clamp(1, MAX_SEGMENTS_PER_ITEM);
        let after = req.cursor.as_deref().map(Cursor::parse).transpose()?;

        let mut clauses = vec![(Occur::Must, parsed.query)];
        clauses.extend(
            req.filter
                .to_queries(&self.fields)
                .into_iter()
                .map(|q| (Occur::Must, q)),
        );
        let query = BooleanQuery::new(clauses);

        // 一次搜索只用同一个 searcher，写入重载读者时段地址不会错位。
        let searcher = self.reader.searcher();
        let mut ranked = rank_items(&searcher, &query, per_item)?;
        if let Some(after) = &after {
            ranked.retain(|(id, acc)| after.precedes(acc.best, id));
        }
        let more = ranked.len() > limit;
        ranked.truncate(limit);
        let next_cursor = more
            .then(|| ranked.last().map(|(id, acc)| Cursor::encode(acc.best, id)))
            .flatten();

        let items = ranked
            .into_iter()
            .map(|(id, acc)| self.hydrate(&searcher, id, acc, &parsed.marks))
            .collect::<Result<Vec<_>>>()?;
        Ok(SearchPage { items, next_cursor })
    }

    fn hydrate(
        &self,
        searcher: &Searcher,
        item_id: String,
        acc: ItemAcc,
        marks: &Marks,
    ) -> Result<ItemHit> {
        let f = &self.fields;
        let mut segments = Vec::new();
        let mut head: Option<TantivyDocument> = None;
        for (score, addr) in acc.segments {
            let doc: TantivyDocument = searcher.doc(addr)?;
            segments.push(SegmentHit {
                layer: text_of(&doc, f.layer),
                snapshot_id: opt_text_of(&doc, f.snapshot_id),
                locator: text_of(&doc, f.locator),
                snippet: snippet::make(&text_of(&doc, f.text), marks),
                score,
            });
            head.get_or_insert(doc);
        }
        // 每段都带着条目级字段，取第一段的就行。
        let head = head.unwrap_or_default();
        Ok(ItemHit {
            item_id,
            platform: text_of(&head, f.platform),
            kind: text_of(&head, f.kind),
            title: text_of(&head, f.title),
            author: text_of(&head, f.author),
            score: acc.best,
            segments,
        })
    }
}

/// 召回并聚合，按（分数从高到低，条目 id 从小到大）排好。
/// 以后接向量检索：另一个召回器给出（条目 id，段）的排名，
/// 在这里用倒数排名融合（RRF）合并再排序，后面的翻页和取文字不用改。
fn rank_items(
    searcher: &Searcher,
    query: &dyn Query,
    per_item: usize,
) -> Result<Vec<(String, ItemAcc)>> {
    let items = searcher.search(
        query,
        &ItemCollector {
            segments_per_item: per_item,
        },
    )?;
    let mut ranked: Vec<_> = items.into_iter().collect();
    ranked.sort_by(|a, b| b.1.best.total_cmp(&a.1.best).then_with(|| a.0.cmp(&b.0)));
    Ok(ranked)
}

fn opt_text_of(doc: &TantivyDocument, field: tantivy::schema::Field) -> Option<String> {
    doc.get_first(field)
        .and_then(|v| v.as_str())
        .map(str::to_owned)
}

fn text_of(doc: &TantivyDocument, field: tantivy::schema::Field) -> String {
    opt_text_of(doc, field).unwrap_or_default()
}

/// 翻页游标：上一页最后一个条目的（分数位，条目 id）。
struct Cursor {
    score: Score,
    item_id: String,
}

impl Cursor {
    fn encode(score: Score, item_id: &str) -> String {
        format!("{:08x}.{item_id}", score.to_bits())
    }

    fn parse(s: &str) -> Result<Self> {
        let (bits, item_id) = s.split_once('.').ok_or(Error::BadCursor)?;
        let bits = u32::from_str_radix(bits, 16).map_err(|_| Error::BadCursor)?;
        Ok(Self {
            score: Score::from_bits(bits),
            item_id: item_id.to_owned(),
        })
    }

    /// （score, id）是否排在游标之后。
    fn precedes(&self, score: Score, id: &str) -> bool {
        self.score
            .total_cmp(&score)
            .then_with(|| id.cmp(self.item_id.as_str()))
            .is_gt()
    }
}
