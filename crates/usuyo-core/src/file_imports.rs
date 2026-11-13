//! 从文件导入（协议 §3.9）：你交给一种读文件的方法的一个文件（别的工具的导出、平台给你的个人数据包，
//! 或浏览器脚本交来的、装着已取到内容的一个文件）。
//!
//! 文件只存一份副本，在按机器目录的 `libraries/<libraryId>/imports/<导入 id>/` 下，不在资料库里，从不执行；
//! 插件经 `USUYO_INPUT` 读它。之后和别的导入一样：列出来先给你看，你确认了才收。
//! 副本在导入删掉、或确认以后排的任务都做完时去掉。

use std::path::{Path, PathBuf};

use usuyo_plugin::FetchPlugin;
use usuyo_plugin::protocol::{FetchInputKind, Method};

use crate::engine::Engine;
use crate::fetch::now_ms;
use crate::jobs::ImportRow;
use crate::view::{Event, FileReaderView, ImportView};
use crate::words::{self, Said};
use crate::{Error, Result};

/// 窗口里交的一个文件最多多大（官方归档带着图和视频，几个 GiB 常见）。
pub const FILE_IMPORT_MAX: u64 = 16 << 30;
/// 浏览器脚本经投递接口交的一个文件最多多大。
pub const HANDOVER_MAX: u64 = 256 << 20;
/// 显示的文件名最多几个字节。
const NAME_MAX: usize = 200;
/// 副本保留的扩展名：只有这几种，别的不带扩展名。
const KEPT_EXTENSIONS: [&str; 5] = ["zip", "json", "html", "csv", "txt"];

/// 一个正在交进来的文件：副本的位置已经备好（空文件，只有自己能读写）。调用方把字节写进 [`Self::path`]，
/// 写完交给 [`Engine::finish_file_import`]；没交出去就丢掉的（写到一半断了、超了大小、请求超时），连文件夹一起去掉。
#[derive(Debug)]
pub struct FileUpload {
    id: String,
    plugin: String,
    method: String,
    name: String,
    dir: PathBuf,
    path: PathBuf,
    kept: bool,
}

impl FileUpload {
    /// 副本：调用方往这里写交来的字节。
    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for FileUpload {
    fn drop(&mut self) {
        if !self.kept
            && let Err(e) = std::fs::remove_dir_all(&self.dir)
        {
            tracing::warn!(dir = %self.dir.display(), error = %e, "unfinished upload not removed");
        }
    }
}

/// 交来的文件名只当显示的字：只留最后一段，去掉控制字符和开头的点，最多 [`NAME_MAX`] 字节；
/// 洗完是空的写「未命名的文件」。它从不进路径。
fn display_name(raw: &str) -> String {
    let last = raw.rsplit(['/', '\\']).next().unwrap_or_default();
    let kept: String = last.chars().filter(|c| !c.is_control()).collect();
    let kept = kept.trim().trim_start_matches('.').trim();
    let mut end = kept.len().min(NAME_MAX);
    while !kept.is_char_boundary(end) {
        end -= 1;
    }
    match kept[..end].trim_end() {
        "" => "未命名的文件".to_owned(),
        name => name.to_owned(),
    }
}

/// 副本的文件名：固定的 `input`，显示名的扩展名在 [`KEPT_EXTENSIONS`] 里时带上它（插件可能按扩展名认格式）。
fn copy_name(display: &str) -> String {
    let ext = display
        .rsplit_once('.')
        .map(|(_, e)| e.to_ascii_lowercase())
        .filter(|e| KEPT_EXTENSIONS.contains(&e.as_str()));
    match ext {
        Some(e) => format!("input.{e}"),
        None => "input".to_owned(),
    }
}

/// 读文件的方法：做了、在插件进程里跑、`list` 写了 `files`。
fn reads_files(m: &Method) -> bool {
    m.callable()
        && m.capabilities
            .list
            .as_ref()
            .is_some_and(|l| !l.files.is_empty())
}

/// 这种方法读出来的条目里只有 `ref` 的，能不能回到它 `fetch <ref>`。
pub(crate) fn fetches_refs(m: &Method) -> bool {
    m.capabilities
        .fetch
        .as_ref()
        .is_some_and(|f| f.input.contains(&FetchInputKind::Ref))
}

impl Engine {
    /// 装着的、能读文件的方法（`GET /file-readers`），照插件和清单的顺序。
    pub fn file_readers(&self) -> Vec<FileReaderView> {
        let mut out = Vec::new();
        for p in self.inner.host.plugins() {
            for m in p.spec.methods.iter().filter(|m| reads_files(m)) {
                let Some(list) = &m.capabilities.list else {
                    continue;
                };
                out.push(FileReaderView {
                    id: format!("{}#{}", p.id(), m.id),
                    name: m.name.clone(),
                    plugin: p.header.name.clone(),
                    // 清单校验过：写了 `files` 就有 `reads`
                    reads: list.reads.clone().unwrap_or_default(),
                    patterns: list.files.clone(),
                    lists: list.sources.iter().map(|s| s.title.clone()).collect(),
                    note: m.risk.as_ref().map(|r| r.text.clone()),
                });
            }
        }
        out
    }

    /// 备好一个交进来的文件的副本：`reader` 是 `<插件 id>#<方法 id>`，那种方法要能读文件、开着、
    /// 这台机器上用得上；`name` 是交来的文件名，洗过才用。
    pub fn prepare_file_import(&self, reader: &str, name: &str) -> Result<FileUpload> {
        let (plugin_id, method) = reader.split_once('#').ok_or_else(|| {
            Error::InvalidArgs(format!("reader `{reader}` is not `<plugin>#<method>`"))
                .hinted(words::NO_READER.said())
        })?;
        let plugin = self.reader_plugin(plugin_id)?;
        let reads = plugin.spec.method(method).is_some_and(reads_files);
        if !reads {
            return Err(Error::UnsupportedInput(format!(
                "{plugin_id} has no method `{method}` that reads files"
            ))
            .hinted(words::NO_READ_METHOD.with(&[("plugin", &plugin.header.name)])));
        }
        self.list_method(&plugin, method)?;
        let name = display_name(name);
        let id = usuyo_library::uuid7();
        let dir = self.inner.import_copies.join(&id);
        let path = dir.join(copy_name(&name));
        // 只是一层、普通的一段：交来的名字不在路径里，这里再核一次
        let inside = path.parent() == Some(dir.as_path())
            && path
                .components()
                .next_back()
                .is_some_and(|c| matches!(c, std::path::Component::Normal(_)));
        if !inside {
            return Err(Error::InvalidArgs(format!(
                "copy path {} is not directly inside its folder",
                path.display()
            )));
        }
        create_private_dir(&dir)?;
        let upload = FileUpload {
            id,
            plugin: plugin_id.to_owned(),
            method: method.to_owned(),
            name,
            dir,
            path,
            kept: false,
        };
        let mut opts = std::fs::OpenOptions::new();
        opts.write(true).create_new(true);
        #[cfg(unix)]
        std::os::unix::fs::OpenOptionsExt::mode(&mut opts, 0o600);
        opts.open(&upload.path)
            .map_err(|e| Error::io(&upload.path, e))?;
        Ok(upload)
    }

    /// 文件写完了：建这次导入，马上列第一页（插件读不了这个文件就在这里回 `invalid-args`，带插件的那句话，
    /// 导入和副本都去掉），其余的页在后台接着列。`client` 是交来的投递端的名字（浏览器脚本经投递接口交的）。
    pub async fn finish_file_import(
        &self,
        mut upload: FileUpload,
        client: Option<&str>,
    ) -> Result<ImportView> {
        let reader = self
            .inner
            .host
            .plugin(&upload.plugin)
            .map(|p| p.header.name.clone())
            .unwrap_or_default();
        let row = ImportRow {
            id: upload.id.clone(),
            kind: "file".into(),
            plugin: Some(upload.plugin.clone()),
            method: Some(upload.method.clone()),
            source: Some(format!("file:{}", upload.name)),
            client: client.map(str::to_owned),
            title: upload.name.clone(),
            action: None,
            created_at: now_ms(),
            cursor: None,
            listed: false,
            pages: 0,
            attempts: 0,
            wait_until: None,
            stop_code: None,
            stop_hint: None,
            held: false,
            confirmed_at: None,
            also_trashed: false,
            skipped: 0,
            file_name: Some(upload.name.clone()),
        };
        self.inner.jobs.insert_import(&row)?;
        // 从这里起副本归这次导入管：删导入时一起去掉
        upload.kept = true;
        if let Err(e) = self.list_page(&row).await {
            self.delete_import(&row.id)?;
            return Err(e);
        }
        let now = self.import_row(&row.id)?;
        if let (Some(code), None) = (&now.stop_code, now.wait_until) {
            let hint = now.stop_hint.as_deref().map_or_else(
                || words::CANT_READ_FILE.with(&[("plugin", &reader)]),
                Said::from_stored,
            );
            self.delete_import(&row.id)?;
            return Err(Error::InvalidArgs(format!(
                "{} {} could not read the file: {code}",
                upload.plugin, upload.method
            ))
            .hinted(hint));
        }
        self.inner.imports_wake.notify_one();
        self.emit(Event::ImportUpdated { id: row.id.clone() });
        self.import(&row.id)
    }

    fn reader_plugin(&self, id: &str) -> Result<FetchPlugin> {
        self.inner.host.plugin(id).cloned().ok_or_else(|| {
            Error::NotFound(format!("plugin {id}")).hinted(words::NOT_INSTALLED.said())
        })
    }

    /// 一次 `file` 类导入的副本：`imports/<导入 id>/input[.扩展名]`。
    pub(crate) fn import_copy(&self, r: &ImportRow) -> Option<PathBuf> {
        let name = r.file_name.as_deref()?;
        Some(self.inner.import_copies.join(&r.id).join(copy_name(name)))
    }

    /// 去掉这次导入的副本（导入删掉了，或确认以后排的任务都做完了）。没有就算了。
    pub(crate) fn drop_import_copy(&self, import: &str) -> Result<()> {
        let dir = self.inner.import_copies.join(import);
        match std::fs::remove_dir_all(&dir) {
            Ok(()) => Ok(()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(e) => Err(Error::io(&dir, e)),
        }
    }

    /// 确认过、排的任务都做完了的 `file` 类导入，副本用不上了。
    pub(crate) fn drop_copy_if_done(&self, import: &str) -> Result<()> {
        let Some(r) = self.inner.jobs.import(import)? else {
            return Ok(());
        };
        if r.kind != "file" || r.confirmed_at.is_none() {
            return Ok(());
        }
        let (live, _) = self.inner.jobs.import_jobs(import)?;
        if live == 0 {
            self.drop_import_copy(import)?;
        }
        Ok(())
    }

    /// 打开时：`imports/` 下没有对应导入的文件夹（交到一半崩了留下的）去掉。
    pub(crate) fn sweep_import_copies(&self) -> Result<()> {
        let dir = &self.inner.import_copies;
        let entries = match std::fs::read_dir(dir) {
            Ok(e) => e,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(()),
            Err(e) => return Err(Error::io(dir, e)),
        };
        for entry in entries {
            let entry = entry.map_err(|e| Error::io(dir, e))?;
            let name = entry.file_name();
            let known = match name.to_str() {
                Some(id) => self.inner.jobs.import(id)?.is_some(),
                None => false,
            };
            if !known {
                let path = entry.path();
                if let Err(e) = std::fs::remove_dir_all(&path) {
                    tracing::warn!(dir = %path.display(), error = %e, "stray import copy not removed");
                }
            }
        }
        Ok(())
    }
}

/// 建只有自己能进的文件夹（unix 上 0700），上级没有就一起建。
fn create_private_dir(dir: &Path) -> Result<()> {
    let mut b = std::fs::DirBuilder::new();
    b.recursive(true);
    #[cfg(unix)]
    std::os::unix::fs::DirBuilderExt::mode(&mut b, 0o700);
    b.create(dir).map_err(|e| Error::io(dir, e))
}

#[cfg(test)]
mod tests {
    use super::{NAME_MAX, copy_name, display_name};

    #[test]
    fn hostile_names_stay_display_text() {
        for (raw, shown) in [
            ("twitter-2026.zip", "twitter-2026.zip"),
            ("..", "未命名的文件"),
            ("../../x", "x"),
            ("a/b", "b"),
            ("C:\\x", "x"),
            ("CON", "CON"),
            ("a:b|c", "a:b|c"),
            ("a\0b\nc\r", "abc"),
            ("", "未命名的文件"),
            ("  .hidden\n", "hidden"),
            ("a/", "未命名的文件"),
        ] {
            assert_eq!(display_name(raw), shown, "{raw:?}");
        }
        let long = "字".repeat(10_000);
        let cut = display_name(&long);
        assert!(cut.len() <= NAME_MAX && cut.chars().all(|c| c == '字'));
    }

    #[test]
    fn copies_have_a_fixed_name() {
        assert_eq!(copy_name("twitter-2026.ZIP"), "input.zip");
        assert_eq!(copy_name("导出.json"), "input.json");
        assert_eq!(copy_name("x.exe"), "input");
        assert_eq!(copy_name("CON"), "input");
        assert_eq!(copy_name("a.b|c"), "input");
    }
}
