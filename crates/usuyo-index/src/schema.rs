use tantivy::Index;
use tantivy::schema::{
    FAST, Field, IndexRecordOption, STORED, STRING, Schema, TextFieldIndexing, TextOptions,
};

use crate::analyzer::{Mode, analyzer};

const WORDS: &str = "usuyo_words";
const GRAMS: &str = "usuyo_grams";
const PINYIN: &str = "usuyo_pinyin";

/// 内部保留的层名：标题、作者、标签、主题合成的一段，放在这一层里。
pub(crate) const META_LAYER: &str = "meta";

/// 索引里一个文档 = 一段文字；条目级字段每段都带一份。
#[derive(Clone)]
pub(crate) struct Fields {
    pub item_id: Field,
    pub platform: Field,
    pub kind: Field,
    pub title: Field,
    pub author: Field,
    pub tags: Field,
    pub topics: Field,
    pub published_at: Field,
    pub added_at: Field,
    pub private: Field,
    pub layer: Field,
    pub snapshot_id: Field,
    pub source: Field,
    pub locator: Field,
    /// 条目内的段序号，和 item_id 一起认一段。
    pub seq: Field,
    /// 原文，只存不索引，给高亮用。
    pub text: Field,
    pub words: Field,
    pub grams: Field,
    pub pinyin: Field,
}

pub(crate) fn build() -> (Schema, Fields) {
    let mut b = Schema::builder();
    let analyzed = |name: &str| {
        TextOptions::default().set_indexing_options(
            TextFieldIndexing::default()
                .set_tokenizer(name)
                .set_index_option(IndexRecordOption::WithFreqs),
        )
    };
    let fields = Fields {
        item_id: b.add_text_field("item_id", STRING | STORED | FAST),
        platform: b.add_text_field("platform", STRING | STORED),
        kind: b.add_text_field("kind", STRING | STORED),
        title: b.add_text_field("title", STORED),
        author: b.add_text_field("author", STORED),
        tags: b.add_text_field("tags", STRING),
        topics: b.add_text_field("topics", STRING),
        published_at: b.add_i64_field("published_at", tantivy::schema::INDEXED),
        added_at: b.add_i64_field("added_at", tantivy::schema::INDEXED),
        private: b.add_bool_field("private", tantivy::schema::INDEXED),
        layer: b.add_text_field("layer", STRING | STORED),
        snapshot_id: b.add_text_field("snapshot_id", STORED),
        source: b.add_text_field("source", STORED),
        locator: b.add_text_field("locator", STORED),
        seq: b.add_u64_field("seq", STORED),
        text: b.add_text_field("text", STORED),
        words: b.add_text_field("words", analyzed(WORDS)),
        grams: b.add_text_field("grams", analyzed(GRAMS)),
        pinyin: b.add_text_field("pinyin", analyzed(PINYIN)),
    };
    (b.build(), fields)
}

pub(crate) fn register_analyzers(index: &Index) {
    let manager = index.tokenizers();
    manager.register(WORDS, analyzer(Mode::Words));
    manager.register(GRAMS, analyzer(Mode::Grams));
    manager.register(PINYIN, analyzer(Mode::Pinyin));
}
