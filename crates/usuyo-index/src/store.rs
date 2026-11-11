use std::fs;
use std::path::Path;
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};

use tantivy::directory::MmapDirectory;
use tantivy::schema::TantivyDocument;
use tantivy::{Index, IndexReader, IndexSettings, IndexWriter, ReloadPolicy, TantivyError, Term};

use crate::doc::IndexDoc;
use crate::error::{Error, Result};
use crate::schema::{self, Fields, META_LAYER};
use crate::stamp::Stamp;

/// 写入用的内存预算（字节）。
const WRITER_HEAP: usize = 50_000_000;

/// 一个磁盘上的搜索索引。写操作经内部锁串行，搜索可以并发。
pub struct SearchIndex {
    pub(crate) index: Index,
    pub(crate) reader: IndexReader,
    pub(crate) fields: Fields,
    writer: Mutex<Option<IndexWriter>>,
    dir: std::path::PathBuf,
    needs_rebuild: AtomicBool,
}

impl SearchIndex {
    /// 打开 `dir` 里的索引，没有就新建。
    ///
    /// 目录为空、版本标记过期或损坏时清空重建一个新索引；
    /// 目录里有别的东西又没有我们的标记，返回 `ForeignDirectory`，不动它。
    /// 新建、被清空、或上次重建没做完时 [`needs_rebuild`](Self::needs_rebuild) 为真。
    pub fn open_or_create(dir: &Path) -> Result<Self> {
        fs::create_dir_all(dir)?;
        let stamp = Stamp::read(dir)?;
        let reuse = match &stamp {
            Some(Ok(s)) if s.is_current() => Some(s.complete),
            Some(_) => {
                clear_dir(dir)?;
                None
            }
            None if fs::read_dir(dir)?.next().is_some() => {
                return Err(Error::ForeignDirectory(dir.to_path_buf()));
            }
            None => None,
        };

        let (schema, fields) = schema::build();
        let (index, needs_rebuild) = match reuse {
            Some(complete) => (Index::open_in_dir(dir)?, !complete),
            None => {
                let index =
                    Index::create(MmapDirectory::open(dir)?, schema, IndexSettings::default())?;
                Stamp::current(false).write(dir)?;
                (index, true)
            }
        };
        schema::register_analyzers(&index);
        let reader = index
            .reader_builder()
            .reload_policy(ReloadPolicy::Manual)
            .try_into()?;
        Ok(Self {
            index,
            reader,
            fields,
            writer: Mutex::new(None),
            dir: dir.to_path_buf(),
            needs_rebuild: AtomicBool::new(needs_rebuild),
        })
    }

    /// 为真时调用方应该 [`rebuild`](Self::rebuild)。
    pub fn needs_rebuild(&self) -> bool {
        self.needs_rebuild.load(Ordering::Relaxed)
    }

    /// 用这条替换同一 `item_id` 的全部旧段。提交（`commit`）后搜索才看得到。
    pub fn upsert(&self, doc: &IndexDoc) -> Result<()> {
        let segments = segment_docs(&self.fields, doc)?;
        self.with_writer(|writer| {
            writer.delete_term(Term::from_field_text(self.fields.item_id, &doc.item_id));
            for segment in segments {
                writer.add_document(segment)?;
            }
            Ok(())
        })
    }

    /// 删掉一条的全部段。提交后生效。
    pub fn delete(&self, item_id: &str) -> Result<()> {
        self.with_writer(|writer| {
            writer.delete_term(Term::from_field_text(self.fields.item_id, item_id));
            Ok(())
        })
    }

    pub fn commit(&self) -> Result<()> {
        self.with_writer(|writer| {
            writer.commit()?;
            Ok(())
        })?;
        self.reader.reload()?;
        Ok(())
    }

    /// 清空后按 `docs` 重新建索引，提交并标记为完整。出错时回滚，旧内容原样保留。
    pub fn rebuild(&self, docs: impl IntoIterator<Item = IndexDoc>) -> Result<()> {
        self.with_writer(|writer| {
            let result = (|| -> Result<()> {
                writer.delete_all_documents()?;
                for doc in docs {
                    for segment in segment_docs(&self.fields, &doc)? {
                        writer.add_document(segment)?;
                    }
                }
                writer.commit()?;
                Ok(())
            })();
            if result.is_err() {
                writer.rollback()?;
            }
            result
        })?;
        self.reader.reload()?;
        Stamp::current(true).write(&self.dir)?;
        self.needs_rebuild.store(false, Ordering::Relaxed);
        Ok(())
    }

    /// 第一次写时才拿写锁，只读的进程不占它。
    fn with_writer<T>(&self, f: impl FnOnce(&mut IndexWriter) -> Result<T>) -> Result<T> {
        let mut guard = self.writer.lock().unwrap_or_else(|p| p.into_inner());
        let writer = match &mut *guard {
            Some(writer) => writer,
            empty => empty.insert(self.index.writer_with_num_threads(1, WRITER_HEAP).map_err(
                |e| match e {
                    TantivyError::LockFailure(..) => Error::Busy,
                    other => other.into(),
                },
            )?),
        };
        f(writer)
    }
}

/// 清掉目录里的全部内容（只在认得出是我们自己的旧索引时调用）。
fn clear_dir(dir: &Path) -> Result<()> {
    for entry in fs::read_dir(dir)? {
        let path = entry?.path();
        if path.is_dir() {
            fs::remove_dir_all(&path)?;
        } else {
            fs::remove_file(&path)?;
        }
    }
    Ok(())
}

/// 把一条拆成每段一个文档；先全部拼好再写，层名不对时不会伤到旧数据。
fn segment_docs(f: &Fields, doc: &IndexDoc) -> Result<Vec<TantivyDocument>> {
    for layer in &doc.layers {
        if layer.layer.is_empty() || layer.layer == META_LAYER {
            return Err(Error::InvalidLayer(layer.layer.clone()));
        }
    }

    let mut out = Vec::new();
    let mut seq = 0u64;
    let mut push = |layer: &str, snapshot_id: Option<&str>, text: &str, locator: &str| {
        if text.trim().is_empty() {
            return;
        }
        let mut d = TantivyDocument::new();
        d.add_text(f.item_id, &doc.item_id);
        d.add_text(f.platform, &doc.platform);
        d.add_text(f.kind, &doc.kind);
        d.add_text(f.title, &doc.title);
        d.add_text(f.author, &doc.author);
        for tag in &doc.tags {
            d.add_text(f.tags, tag);
        }
        for topic in &doc.topics {
            d.add_text(f.topics, topic);
        }
        if let Some(t) = doc.published_at {
            d.add_i64(f.published_at, t);
        }
        d.add_i64(f.added_at, doc.added_at);
        d.add_bool(f.private, doc.private);
        d.add_text(f.layer, layer);
        if let Some(s) = snapshot_id {
            d.add_text(f.snapshot_id, s);
        }
        d.add_text(f.locator, locator);
        d.add_u64(f.seq, seq);
        d.add_text(f.text, text);
        d.add_text(f.words, text);
        d.add_text(f.grams, text);
        d.add_text(f.pinyin, text);
        out.push(d);
        seq += 1;
    };

    // 标题、作者、标签、主题合成一段，让它们也搜得到。
    let meta = std::iter::once(doc.title.as_str())
        .chain(std::iter::once(doc.author.as_str()))
        .chain(doc.tags.iter().map(String::as_str))
        .chain(doc.topics.iter().map(String::as_str))
        .filter(|s| !s.trim().is_empty())
        .collect::<Vec<_>>()
        .join("\n");
    push(META_LAYER, None, &meta, "");

    for layer in &doc.layers {
        for segment in &layer.segments {
            push(
                &layer.layer,
                layer.snapshot_id.as_deref(),
                &segment.text,
                &segment.locator,
            );
        }
    }
    Ok(out)
}
