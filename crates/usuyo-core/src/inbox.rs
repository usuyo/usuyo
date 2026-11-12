//! 收件文件夹（协议 §5.2）：手机经云盘（iCloud 云盘一类）放进按机器目录里的 `inbox/` 的 `.txt`，
//! 里面每个链接收下一条（整段分享文字也行）。放进去本身就是一次确认，不进预览。
//!
//! - 只认 `.txt` 的普通文件；别的文件、子文件夹、点开头的（云盘的占位文件、我们自己的 `.usuyo-*`）都不碰。
//! - 一个文件 10 秒没变才处理，免得读到云盘没传完的。
//! - 收完移到 `.usuyo-done/<日期>/`；有链接没收成的移到 `.usuyo-failed/`，旁边一个 `<名字>.error.json`。
//!   符号链接不读，同样移到 `.usuyo-failed/`。

use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

use crate::engine::{Engine, Via};
use crate::{Error, Result};

/// 多久看一次。
const POLL: Duration = Duration::from_secs(5);
/// 一个文件最多读多少字节。
const MAX_BYTES: u64 = 1 << 20;
/// 一个文件最多收多少条链接。
const MAX_LINKS: usize = 500;

/// 一轮看下来：收了几条链接，处理了几个文件。
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct InboxReport {
    pub taken: usize,
    pub files: usize,
}

/// 一段文字里的全部 http(s) 链接，按出现的顺序（规则同 [`crate::link_in`]）。
fn all_urls(text: &str) -> Vec<url::Url> {
    crate::share_text::link_runs(text)
        .filter_map(crate::link_in)
        .take(MAX_LINKS)
        .collect()
}

fn unique(dir: &Path, name: &str) -> PathBuf {
    let first = dir.join(name);
    if !first.exists() {
        return first;
    }
    (1..)
        .map(|n| dir.join(format!("{n}-{name}")))
        .find(|p| !p.exists())
        .unwrap_or(first)
}

impl Engine {
    /// 收件文件夹在哪。
    pub fn inbox_dir(&self) -> &Path {
        &self.inner.inbox
    }

    pub(crate) async fn run_inbox(&self) {
        loop {
            if let Err(e) = self.scan_inbox().await {
                tracing::warn!(error = %e, "inbox");
            }
            tokio::time::sleep(POLL).await;
        }
    }

    /// 看一轮收件文件夹。
    pub async fn scan_inbox(&self) -> Result<InboxReport> {
        let dir = self.inner.inbox.clone();
        let io = |p: &Path, e| Error::io(p, e);
        std::fs::create_dir_all(&dir).map_err(|e| io(&dir, e))?;
        let mut report = InboxReport::default();
        let mut names: Vec<PathBuf> = std::fs::read_dir(&dir)
            .map_err(|e| io(&dir, e))?
            .filter_map(|e| e.ok().map(|e| e.path()))
            .collect();
        names.sort();
        let settled = SystemTime::now()
            .checked_sub(self.inner.inbox_settle)
            .unwrap_or(SystemTime::UNIX_EPOCH);
        for path in names {
            let Some(name) = path.file_name().and_then(|n| n.to_str()).map(str::to_owned) else {
                continue;
            };
            let is_txt = path
                .extension()
                .and_then(|e| e.to_str())
                .is_some_and(|e| e.eq_ignore_ascii_case("txt"));
            if name.starts_with('.') || !is_txt {
                continue;
            }
            let meta = std::fs::symlink_metadata(&path).map_err(|e| io(&path, e))?;
            if meta.file_type().is_symlink() {
                self.inbox_failed(&path, &name, &["symbolic links are not read".into()])?;
                report.files += 1;
                continue;
            }
            if !meta.is_file() || meta.modified().map_err(|e| io(&path, e))? > settled {
                continue;
            }
            report.files += 1;
            let mut text = String::new();
            {
                use std::io::Read;
                let f = std::fs::File::open(&path).map_err(|e| io(&path, e))?;
                let mut bytes = Vec::new();
                f.take(MAX_BYTES)
                    .read_to_end(&mut bytes)
                    .map_err(|e| io(&path, e))?;
                text.push_str(&String::from_utf8_lossy(&bytes));
            }
            let mut errors = Vec::new();
            let links = all_urls(&text);
            if links.is_empty() {
                errors.push("no http(s) link in the file".to_owned());
            }
            for url in links {
                match self.take(url.as_str(), "link", Via::Mobile).await {
                    Ok(_) => report.taken += 1,
                    // 链接可能带分享参数：原因里只写主机
                    Err(e) => errors.push(format!(
                        "{}: {}",
                        url.host_str().unwrap_or_default(),
                        e.code()
                    )),
                }
            }
            if errors.is_empty() {
                let day = jiff::Zoned::now().date().to_string();
                let done = dir.join(".usuyo-done").join(day);
                std::fs::create_dir_all(&done).map_err(|e| io(&done, e))?;
                let to = unique(&done, &name);
                std::fs::rename(&path, &to).map_err(|e| io(&path, e))?;
            } else {
                self.inbox_failed(&path, &name, &errors)?;
            }
        }
        Ok(report)
    }

    fn inbox_failed(&self, path: &Path, name: &str, errors: &[String]) -> Result<()> {
        let failed = self.inner.inbox.join(".usuyo-failed");
        std::fs::create_dir_all(&failed).map_err(|e| Error::io(&failed, e))?;
        let to = unique(&failed, name);
        std::fs::rename(path, &to).map_err(|e| Error::io(path, e))?;
        let note = to.with_file_name(format!(
            "{}.error.json",
            to.file_name().and_then(|n| n.to_str()).unwrap_or(name)
        ));
        let body = serde_json::json!({ "errors": errors });
        std::fs::write(&note, serde_json::to_vec_pretty(&body)?).map_err(|e| Error::io(&note, e))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_every_link() {
        let got: Vec<String> = all_urls(
            "【笔记】 https://a.test/x?y=1 复制打开\nhttps://b.test/z\n没有链接的一行\nHTTP://C.test/w",
        )
        .into_iter()
        .map(String::from)
        .collect();
        assert_eq!(
            got,
            [
                "https://a.test/x?y=1",
                "https://b.test/z",
                "http://c.test/w"
            ]
        );
    }
}
