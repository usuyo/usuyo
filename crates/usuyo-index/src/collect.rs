//! 按条目聚合的收集器：扫完所有命中的段，每个条目留最高分和得分最高的几段。
//! 不设候选上限，长口播命中几百段也不会挤掉别的条目。

use std::collections::HashMap;

use tantivy::collector::{Collector, SegmentCollector};
use tantivy::columnar::StrColumn;
use tantivy::{DocAddress, DocId, Score, SegmentOrdinal, SegmentReader, TantivyError};

pub(crate) struct ItemAcc {
    pub best: Score,
    /// 得分最高的几段，不保证有序。
    pub segments: Vec<(Score, DocAddress)>,
}

impl ItemAcc {
    fn new() -> Self {
        Self {
            best: Score::MIN,
            segments: Vec::new(),
        }
    }

    fn add(&mut self, score: Score, addr: DocAddress, keep: usize) {
        self.best = self.best.max(score);
        self.segments.push((score, addr));
        // 攒到两倍再裁，摊薄排序成本。
        if self.segments.len() > keep * 2 {
            self.trim(keep);
        }
    }

    pub fn trim(&mut self, keep: usize) {
        self.segments
            .sort_by(|a, b| b.0.total_cmp(&a.0).then(a.1.cmp(&b.1)));
        self.segments.truncate(keep);
    }
}

pub(crate) struct ItemCollector {
    /// 每个条目保留几段。
    pub segments_per_item: usize,
}

type Items = HashMap<String, ItemAcc>;

impl Collector for ItemCollector {
    type Fruit = Items;
    type Child = ItemSegmentCollector;

    fn for_segment(
        &self,
        segment_ord: SegmentOrdinal,
        segment: &SegmentReader,
    ) -> tantivy::Result<ItemSegmentCollector> {
        let column = segment
            .fast_fields()
            .str("item_id")
            .map_err(|e| TantivyError::InternalError(format!("item_id fast field: {e}")))?;
        Ok(ItemSegmentCollector {
            segment_ord,
            column,
            keep: self.segments_per_item,
            by_ord: HashMap::new(),
        })
    }

    fn requires_scoring(&self) -> bool {
        true
    }

    fn merge_fruits(&self, fruits: Vec<Result<Items, String>>) -> tantivy::Result<Items> {
        let mut merged = Items::new();
        for fruit in fruits {
            for (item_id, acc) in fruit.map_err(TantivyError::InternalError)? {
                let into = merged.entry(item_id).or_insert_with(ItemAcc::new);
                into.best = into.best.max(acc.best);
                into.segments.extend(acc.segments);
            }
        }
        for acc in merged.values_mut() {
            acc.trim(self.segments_per_item);
        }
        Ok(merged)
    }
}

pub(crate) struct ItemSegmentCollector {
    segment_ord: SegmentOrdinal,
    column: Option<StrColumn>,
    keep: usize,
    by_ord: HashMap<u64, ItemAcc>,
}

impl SegmentCollector for ItemSegmentCollector {
    type Fruit = Result<Items, String>;

    fn collect(&mut self, doc: DocId, score: Score) {
        let Some(column) = &self.column else { return };
        let Some(ord) = column.ords().first(doc) else {
            return;
        };
        let addr = DocAddress::new(self.segment_ord, doc);
        self.by_ord
            .entry(ord)
            .or_insert_with(ItemAcc::new)
            .add(score, addr, self.keep);
    }

    fn harvest(self) -> Self::Fruit {
        let Some(column) = self.column else {
            return Ok(Items::new());
        };
        let mut items = Items::new();
        for (ord, acc) in self.by_ord {
            let mut id = String::new();
            match column.ord_to_str(ord, &mut id) {
                Ok(true) => {}
                Ok(false) => return Err(format!("item_id ordinal {ord} not found")),
                Err(e) => return Err(format!("reading item_id: {e}")),
            }
            items.insert(id, acc);
        }
        Ok(items)
    }
}
