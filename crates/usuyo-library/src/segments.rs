//! 按段写的 JSONL（SPEC §1.6）：`<dir>/<4 位本台序号>.jsonl`，每行带本台单调递增的 `n`。
//! 改动记录和发生过的事共用这一套。满 256 KiB 或过了一天（按 UTC 日期）封口。

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use serde_json::Value;

use crate::{Error, Result, time};

const SEAL_BYTES: u64 = 256 * 1024;

#[derive(Debug)]
pub(crate) struct Segments {
    dir: PathBuf,
    next_n: u64,
    seg: u32,
    size: u64,
    /// 当前段第一行的 UTC 日期。
    day: Option<String>,
}

impl Segments {
    pub(crate) fn open(dir: PathBuf) -> Result<Segments> {
        let mut seg = 0;
        for f in segment_files(&dir)? {
            seg = seg.max(segment_number(&f).unwrap_or(0));
        }
        let mut s = Segments {
            dir,
            next_n: 1,
            seg,
            size: 0,
            day: None,
        };
        if seg > 0 {
            let path = s.path(seg);
            let text = fs::read_to_string(&path).map_err(|e| Error::io(&path, e))?;
            s.size = text.len() as u64;
            let lines: Vec<&str> = text.lines().filter(|l| !l.is_empty()).collect();
            if let Some(first) = lines.first() {
                s.day = line_value(&path, first)?["t"]
                    .as_str()
                    .and_then(|t| t.get(..10))
                    .map(str::to_owned);
            }
            if let Some(last) = lines.last() {
                s.next_n = line_value(&path, last)?["n"]
                    .as_u64()
                    .ok_or_else(|| Error::malformed(&path, "last line has no `n`"))?
                    + 1;
            }
        }
        Ok(s)
    }

    fn path(&self, seg: u32) -> PathBuf {
        self.dir.join(format!("{seg:04}.jsonl"))
    }

    /// 写一行：`build` 拿到这行的 `n`，回这行的 JSON 对象。
    pub(crate) fn append(&mut self, build: impl FnOnce(u64) -> Result<Value>) -> Result<Value> {
        let now = time::now();
        let today = now.get(..10).map(str::to_owned);
        if self.seg == 0 || self.size >= SEAL_BYTES || (self.day.is_some() && self.day != today) {
            self.seg += 1;
            self.size = 0;
            self.day = None;
        }
        let value = build(self.next_n)?;
        let mut line = serde_json::to_string(&value).map_err(|e| Error::Json {
            path: self.path(self.seg),
            source: e,
        })?;
        line.push('\n');
        fs::create_dir_all(&self.dir).map_err(|e| Error::io(&self.dir, e))?;
        let path = self.path(self.seg);
        let mut f = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
            .map_err(|e| Error::io(&path, e))?;
        f.write_all(line.as_bytes())
            .and_then(|()| f.sync_data())
            .map_err(|e| Error::io(&path, e))?;
        self.size += line.len() as u64;
        if self.day.is_none() {
            self.day = value["t"]
                .as_str()
                .and_then(|t| t.get(..10))
                .map(str::to_owned)
                .or(today);
        }
        self.next_n += 1;
        Ok(value)
    }
}

fn segment_number(path: &Path) -> Option<u32> {
    path.file_stem()?.to_str()?.parse().ok()
}

/// 一个设备文件夹里的段，按序号排。
pub(crate) fn segment_files(dir: &Path) -> Result<Vec<PathBuf>> {
    let mut out = Vec::new();
    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(out),
        Err(e) => return Err(Error::io(dir, e)),
    };
    for entry in entries {
        let p = entry.map_err(|e| Error::io(dir, e))?.path();
        if p.extension().is_some_and(|x| x == "jsonl") && segment_number(&p).is_some() {
            out.push(p);
        }
    }
    out.sort();
    Ok(out)
}

/// `<root>/<deviceId>/<段>.jsonl` 全部的段。
pub(crate) fn all_segment_files(root: &Path) -> Result<Vec<PathBuf>> {
    let mut out = Vec::new();
    let entries = match fs::read_dir(root) {
        Ok(e) => e,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(out),
        Err(e) => return Err(Error::io(root, e)),
    };
    let mut devices: Vec<PathBuf> = Vec::new();
    for entry in entries {
        let p = entry.map_err(|e| Error::io(root, e))?.path();
        if p.is_dir() {
            devices.push(p);
        }
    }
    devices.sort();
    for d in devices {
        out.extend(segment_files(&d)?);
    }
    Ok(out)
}

pub(crate) fn line_value(path: &Path, line: &str) -> Result<Value> {
    serde_json::from_str(line).map_err(|e| Error::Json {
        path: path.to_owned(),
        source: e,
    })
}
