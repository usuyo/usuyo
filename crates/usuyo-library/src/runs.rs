//! 转录（SPEC §1.2.2、feature 003 §5）：`derived/runs/<runId>/` 一次转换一个文件夹，
//! 在 `.usuyo/tmp/` 里拼好再整个改名进去，写进去以后不再改。哪次在前按 `runId`（就是时间）算，不另存。

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use usuyo_contract::{FileHash, Keyframes, Ocr, RunRecord};

use crate::fsx::{self, list_files};
use crate::id::{ItemId, is_uuid7};
use crate::item::{ITEM_PATH_MAX, contained_file, is_image_key, read_dirs};
use crate::text::rel_path_fits;
use crate::{Error, Library, Result};

/// 条目文件夹里放转录的地方。
pub(crate) const RUNS: &str = "derived/runs";
/// 关键帧图片在转录文件夹里的子文件夹；只留文字时整个移走。
const KEYFRAMES_DIR: &str = "keyframes";
/// `frame/<runId>/<毫秒>` 里毫秒最多几位（十位够 115 天）。
const MS_DIGITS: usize = 10;

/// 宿主拼好的一次转录：`staged` 是 [`Library::staging_dir`] 给的文件夹，里面只有插件的产出；
/// `files` 是 [`hash_run_products`] 在拿资料库的锁之前算好的；`record` 是调用方照插件的结果写的
/// `run.json`（`id` 由调用方用 [`crate::uuid7`] 取）。
#[derive(Debug, Clone)]
pub struct NewRun {
    pub staged: PathBuf,
    pub files: Vec<FileHash>,
    pub record: RunRecord,
}

/// 同一层的几次转录（同一个 `job`、同样的输入，feature 003 §5.1）：最新的在前，别的照样摆着。
#[derive(Debug, Clone, PartialEq)]
pub struct RunLayer {
    pub job: String,
    /// 输入的 sha256，排好、去重。
    pub inputs: Vec<String>,
    pub runs: Vec<RunRecord>,
}

/// 获取插件存下的一份平台字幕（快照 `media[]` 里 `kind: subtitle` 的），原样，不转。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlatformSubtitle {
    pub snapshot: String,
    /// 快照文件夹里的相对路径，照 `usuyo.fetch.json` 写的。
    pub path: String,
    /// 解析掉符号链接以后的文件，在条目文件夹里。
    pub file: PathBuf,
    /// 那一项的 `lang`，没有就从文件名 `subtitle.<lang>.<扩展名>` 取。
    pub lang: Option<String>,
}

/// 暂存的转录文件夹里每个文件的路径、大小、sha256，按路径排好。不用拿资料库的锁，
/// 写转录前先算；调用方也拿它填 `run.json` 的 `outputs` 和关键帧的哈希。
/// 符号链接和不是普通文件的回 `invalid-args`。
///
/// 调用方：WP6（`usuyo-core` 的转换编排，插件跑完以后）。
pub fn hash_run_products(staged: &Path) -> Result<Vec<FileHash>> {
    fsx::reject_symlinks(staged)?;
    let mut files = Vec::new();
    for rel in list_files(staged)? {
        let path = staged.join(&rel);
        regular_size(&path)?;
        let (size, sha256) = crate::hash::sha256_file(&path)?;
        files.push(FileHash {
            path: rel,
            size,
            sha256,
        });
    }
    Ok(files)
}

/// 普通文件的大小；不是普通文件（命名管道、设备）回 `invalid-args`。
fn regular_size(path: &Path) -> Result<u64> {
    let meta = fs::symlink_metadata(path).map_err(|e| Error::io(path, e))?;
    if meta.is_file() {
        Ok(meta.len())
    } else {
        Err(Error::InvalidArgs(format!(
            "{} is not a regular file",
            path.display()
        )))
    }
}

fn invalid(msg: impl Into<String>) -> Error {
    Error::InvalidArgs(msg.into())
}

/// `subtitle.zh.vtt` → `zh`。
fn lang_from_name(path: &str) -> Option<String> {
    let name = path.rsplit('/').next()?;
    let (lang, _ext) = name.strip_prefix("subtitle.")?.rsplit_once('.')?;
    (!lang.is_empty() && !lang.contains('.')).then(|| lang.to_owned())
}

impl Library {
    /// 写一次转录：核对暂存的文件夹，写 `run.json`，整个改名进 `derived/runs/<runId>/`。
    ///
    /// 核对的：`record.id` 是 UUIDv7、这次还没有；`inputs`、`outputs` 的路径是普通相对路径；
    /// 文件夹里没有符号链接；文件和 `files` 一样（算完哈希以后没变）；声明的产物都在、大小和
    /// sha256 对得上；`keyframes` 产物里列的每帧在 `keyframes/` 下、sha256 对得上；没有声明以外的
    /// 文件；`empty` 的没有产物、有 `reason`，不空的有产物、没有 `reason`。不合的回 `invalid-args`，
    /// 路径太长回 `too-large`，已有同 id 的回 `conflict`，条目不在（进了回收站）回 `not-found`。
    /// 出错时暂存的文件夹删掉，资料库里什么都不留。
    ///
    /// 改名之前 `run.json` 和各产物都已 fsync：中途崩溃时文件夹还在 `.usuyo/tmp/`，
    /// 下次打开资料库时清掉，不会出现没有 `run.json` 的转录。
    ///
    /// 调用方：WP6（`usuyo-core` 的转换编排，一次转换落地时）。
    pub fn write_run(&mut self, id: &ItemId, run: NewRun) -> Result<()> {
        self.writable()?;
        self.check_staged(&run.staged)?;
        let placed = self.place_run(id, &run);
        if placed.is_err()
            && let Err(e) = fsx::remove_dir(&run.staged)
        {
            // 删不掉的下次打开时清（以 UUIDv7 命名的暂存文件夹），这里只报原来的错。
            tracing::warn!(error = %e, "staged run not removed");
        }
        placed
    }

    fn place_run(&self, id: &ItemId, run: &NewRun) -> Result<()> {
        let rec = &run.record;
        self.item(id)?;
        if !is_uuid7(&rec.id) {
            return Err(invalid(format!("bad run id `{}`", rec.id)));
        }
        if let Some(bad) = rec.bad_path() {
            return Err(invalid(format!(
                "run path `{bad}` is not a plain relative path"
            )));
        }
        match (rec.empty, rec.outputs.is_empty(), rec.reason.is_some()) {
            (true, true, true) | (false, false, false) => {}
            (true, false, _) => return Err(invalid("an empty run has outputs")),
            _ => return Err(invalid("`empty`, `outputs` and `reason` disagree")),
        }
        fsx::reject_symlinks(&run.staged)?;

        // 算完哈希以后文件夹不该再变：路径和大小对不上就不收。
        let mut listed = BTreeMap::new();
        for rel in list_files(&run.staged)? {
            let size = regular_size(&run.staged.join(&rel))?;
            listed.insert(rel, size);
        }
        let hashed: BTreeMap<&str, &FileHash> =
            run.files.iter().map(|f| (f.path.as_str(), f)).collect();
        let same = hashed.len() == run.files.len()
            && listed.len() == hashed.len()
            && listed
                .iter()
                .all(|(p, size)| hashed.get(p.as_str()).is_some_and(|f| f.size == *size));
        if !same {
            return Err(invalid("staged run files changed after they were hashed"));
        }

        // 声明了的：各产物，加 `keyframes.json` 里列的每帧（帧不另列在 `outputs` 里）。
        let mut declared: BTreeMap<String, (Option<u64>, String)> = BTreeMap::new();
        let mut declare = |path: &str, size: Option<u64>, sha256: &str| -> Result<()> {
            if path == "run.json" {
                return Err(invalid("`run.json` is written by the host"));
            }
            match declared.insert(path.to_owned(), (size, sha256.to_owned())) {
                Some(_) => Err(invalid(format!("`{path}` is declared twice"))),
                None => Ok(()),
            }
        };
        for o in &rec.outputs {
            declare(&o.path, Some(o.size), &o.sha256)?;
        }
        for o in rec.outputs.iter().filter(|o| o.kind == "keyframes") {
            let path = run.staged.join(&o.path);
            if !hashed.contains_key(o.path.as_str()) {
                return Err(invalid(format!("declared output `{}` is missing", o.path)));
            }
            // 插件写的文件，读不懂算这次转录不合，不算宿主出错。
            let kf: Keyframes = fsx::read_json(&path).map_err(|e| match e {
                Error::Json { .. } | Error::Malformed { .. } => {
                    invalid(format!("`{}` is not a keyframes index: {e}", o.path))
                }
                other => other,
            })?;
            for f in &kf.frames {
                let under = f
                    .path
                    .strip_prefix(KEYFRAMES_DIR)
                    .is_some_and(|rest| rest.starts_with('/'));
                if !under || !usuyo_contract::is_plain_relative(&f.path) {
                    return Err(invalid(format!(
                        "keyframe `{}` is not under `{KEYFRAMES_DIR}/`",
                        f.path
                    )));
                }
                declare(&f.path, None, &f.sha256)?;
            }
        }
        for (path, (size, sha256)) in &declared {
            let f = hashed
                .get(path.as_str())
                .ok_or_else(|| invalid(format!("declared output `{path}` is missing")))?;
            if size.is_some_and(|s| s != f.size) {
                return Err(invalid(format!("`{path}` size differs from the record")));
            }
            if *sha256 != f.sha256 {
                return Err(invalid(format!("`{path}` sha256 differs from the record")));
            }
            let rel = format!("{RUNS}/{}/{path}", rec.id);
            if !rel_path_fits(&rel, ITEM_PATH_MAX) {
                return Err(Error::TooLarge(format!(
                    "path `{rel}` is longer than {ITEM_PATH_MAX} UTF-16 units"
                )));
            }
        }
        if let Some(extra) = hashed.keys().find(|p| !declared.contains_key(**p)) {
            return Err(invalid(format!("`{extra}` is not a declared output")));
        }

        let runs = self.item_dir(id).join(RUNS);
        let dest = runs.join(&rec.id);
        if fs::symlink_metadata(&dest).is_ok() {
            return Err(Error::Conflict(format!("run {} already exists", rec.id)));
        }
        for path in declared.keys() {
            fsx::sync_file(&run.staged.join(path))?;
        }
        fsx::write_json(&run.staged.join("run.json"), rec)?;
        fsx::move_path(&run.staged, &dest)?;
        fsx::sync_dir(&runs);
        Ok(())
    }

    /// 读 `derived/runs/<runId>/run.json`；文件夹名就是 id，路径不合的当写坏了。
    fn read_run(&self, dir: &Path, rid: &str) -> Result<RunRecord> {
        if !is_uuid7(rid) {
            return Err(invalid(format!("bad run id `{rid}`")));
        }
        let path = contained_file(dir, &dir.join(RUNS).join(rid).join("run.json"), || {
            Error::NotFound(format!("run {rid}"))
        })?;
        let rec: RunRecord = fsx::read_json(&path)?;
        if rec.id != rid {
            return Err(Error::malformed(&path, "run id differs from folder name"));
        }
        if let Some(bad) = rec.bad_path() {
            return Err(Error::malformed(
                &path,
                format!("path `{bad}` is not a plain relative path"),
            ));
        }
        Ok(rec)
    }

    /// 一条的全部转录，新的在前（按 id）。读不了的一次跳过、记日志，不挡住别的。
    ///
    /// 调用方：WP7（`usuyo-core` 的条目页和索引文档：每次转录一份）、WP6。
    pub fn runs(&self, id: &ItemId) -> Result<Vec<RunRecord>> {
        self.item(id)?;
        let dir = self.item_dir(id);
        let mut out = Vec::new();
        for d in read_dirs(&dir.join(RUNS))? {
            let name = d.file_name().and_then(|n| n.to_str()).unwrap_or_default();
            match self.read_run(&dir, name) {
                Ok(r) => out.push(r),
                Err(e) => tracing::warn!(item = %id, run = name, error = %e, "run not readable"),
            }
        }
        out.sort_by(|a, b| b.id.cmp(&a.id));
        Ok(out)
    }

    /// 这个 `job` 的转录，新的在前。
    ///
    /// 调用方：WP6（「再转一次」时 `redoOf` 指的那一份）。
    pub fn job_runs(&self, id: &ItemId, job: &str) -> Result<Vec<RunRecord>> {
        let mut runs = self.runs(id)?;
        runs.retain(|r| r.job == job);
        Ok(runs)
    }

    /// 按层分好的转录：同一个 `job`、同样一组输入（按 sha256）算一层，层里新的在前；
    /// 层按各自最新的那次排，新的在前。
    ///
    /// 调用方：WP7（条目页一面有几份时，哪份在前）。
    pub fn run_layers(&self, id: &ItemId) -> Result<Vec<RunLayer>> {
        let mut layers: BTreeMap<(String, Vec<String>), Vec<RunRecord>> = BTreeMap::new();
        for r in self.runs(id)? {
            let inputs: BTreeSet<String> = r.inputs.iter().map(|i| i.sha256.clone()).collect();
            layers
                .entry((r.job.clone(), inputs.into_iter().collect()))
                .or_default()
                .push(r);
        }
        let mut out: Vec<RunLayer> = layers
            .into_iter()
            .map(|((job, inputs), runs)| RunLayer { job, inputs, runs })
            .collect();
        out.sort_by(|a, b| b.runs[0].id.cmp(&a.runs[0].id));
        Ok(out)
    }

    /// 这个 `job` 对现在的输入做好没有（feature 003 §5.1）：有一次这个 `job` 的转录，它的输入的
    /// sha256 盖住了 `needed` 里的每一个。哪个插件、哪个版本做的都算，`empty` 的也算。
    /// 一次 `ocr` 盖几张图时，按张问（`needed` 只放那一张）就知道哪几张还没盖到。
    ///
    /// 调用方：WP6（快照落地以后排任务、启动时补排）。
    pub fn job_done(&self, id: &ItemId, job: &str, needed: &[String]) -> Result<bool> {
        Ok(self.job_runs(id, job)?.iter().any(|r| {
            needed
                .iter()
                .all(|h| r.inputs.iter().any(|i| &i.sha256 == h))
        }))
    }

    /// 一次转录里某种产物（`asr`、`ocr`、`keyframes`、`play`）的文件，解析掉符号链接以后还在条目文件夹里。
    /// 这次没有这种产物、或文件不在了（只留文字移走了）回 `not-found`。
    ///
    /// 调用方：WP7（读 `asr.vtt`、`ocr.json`、`keyframes.json`）。
    pub fn run_output(&self, id: &ItemId, run_id: &str, kind: &str) -> Result<PathBuf> {
        self.item(id)?;
        let dir = self.item_dir(id);
        let rec = self.read_run(&dir, run_id)?;
        self.output_file(&dir, &rec, kind)?
            .ok_or_else(|| Error::NotFound(format!("{kind} of run {run_id}")))
    }

    /// 这次转录有这种产物时它的文件；没有这种产物回 `None`，声明了但文件不在回 `not-found`。
    fn output_file(&self, dir: &Path, rec: &RunRecord, kind: &str) -> Result<Option<PathBuf>> {
        let Some(o) = rec.outputs.iter().find(|o| o.kind == kind) else {
            return Ok(None);
        };
        let p = dir.join(RUNS).join(&rec.id).join(&o.path);
        contained_file(dir, &p, || {
            Error::NotFound(format!("{} of run {}", o.path, rec.id))
        })
        .map(Some)
    }

    /// [`Library::resolve_layer`] 里转录的那几层：`asr`、`ocr`、`frames` 和 `frame/<runId>/<毫秒>`。
    pub(crate) fn resolve_run_layer(
        &self,
        id: &ItemId,
        layer: &str,
        name: &str,
        key: Option<&str>,
        snapshot: Option<&str>,
    ) -> Result<PathBuf> {
        let dir = self.item_dir(id);
        let not_found = || Error::NotFound(format!("{layer} of {id}"));
        let (job, kind) = match name {
            "asr" => ("asr", "asr"),
            "ocr" => ("ocr", "ocr"),
            "frames" => ("frames", "ocr"),
            "frame" => return self.keyframe_file(id, layer, key),
            _ => return Err(invalid(format!("unknown layer `{layer}`"))),
        };
        let from_snapshot =
            |r: &RunRecord| snapshot.is_none_or(|s| r.inputs.iter().any(|i| i.snapshot == s));
        match key {
            Some(rid) if is_uuid7(rid) => {
                let rec = self.read_run(&dir, rid)?;
                if rec.job != job {
                    return Err(not_found());
                }
                self.output_file(&dir, &rec, kind)?.ok_or_else(not_found)
            }
            Some(k) if job == "ocr" && is_image_key(k) => {
                for rec in self.job_runs(id, job)?.iter().filter(|r| from_snapshot(r)) {
                    let Some(p) = self.output_file(&dir, rec, kind)? else {
                        continue;
                    };
                    let ocr: Ocr = fsx::read_json(&p)?;
                    if ocr.images.iter().any(|i| i.image_key.as_deref() == Some(k)) {
                        return Ok(p);
                    }
                }
                Err(not_found())
            }
            Some(_) => Err(invalid(format!("bad key in layer `{layer}`"))),
            None => {
                let newest = self
                    .job_runs(id, job)?
                    .into_iter()
                    .find(|r| from_snapshot(r))
                    .ok_or_else(not_found)?;
                self.output_file(&dir, &newest, kind)?.ok_or_else(not_found)
            }
        }
    }

    /// `frame/<runId>/<毫秒>`：那次 `keyframes` 转录的 `keyframes.json` 里秒数四舍五入到毫秒相等的那一帧。
    fn keyframe_file(&self, id: &ItemId, layer: &str, key: Option<&str>) -> Result<PathBuf> {
        let bad = || invalid(format!("bad frame address `{layer}`"));
        let (rid, ms) = key.and_then(|k| k.split_once('/')).ok_or_else(bad)?;
        if !is_uuid7(rid)
            || ms.is_empty()
            || ms.len() > MS_DIGITS
            || !ms.bytes().all(|c| c.is_ascii_digit())
        {
            return Err(bad());
        }
        let ms: u64 = ms.parse().map_err(|_| bad())?;
        let not_found = || Error::NotFound(format!("{layer} of {id}"));
        let dir = self.item_dir(id);
        let rec = self.read_run(&dir, rid)?;
        if rec.job != "keyframes" {
            return Err(not_found());
        }
        let index = self
            .output_file(&dir, &rec, "keyframes")?
            .ok_or_else(not_found)?;
        let kf: Keyframes = fsx::read_json(&index)?;
        if let Some(bad) = kf.bad_path() {
            return Err(Error::malformed(
                &index,
                format!("path `{bad}` is not a plain relative path"),
            ));
        }
        let frame = kf
            .frames
            .iter()
            .find(|f| (f.sec * 1000.0).round() == ms as f64)
            .ok_or_else(not_found)?;
        contained_file(&dir, &dir.join(RUNS).join(rid).join(&frame.path), not_found)
    }

    /// `video` 先给的那个：最新一次 `play` 转录的产物；给了 `snapshot` 时只看输入在这个快照里的。
    /// 没有这次转录、或产物已经不在了（只留文字移走了）回 `None`，调用方给原样的画面；
    /// 在但指到条目文件夹外回 `not-found`。
    pub(crate) fn play_file(&self, id: &ItemId, snapshot: Option<&str>) -> Result<Option<PathBuf>> {
        let dir = self.item_dir(id);
        let Some((rec, out)) = self.job_runs(id, "play")?.into_iter().find_map(|r| {
            let out = r.outputs.iter().find(|o| o.kind == "play")?.path.clone();
            snapshot
                .is_none_or(|s| r.inputs.iter().any(|i| i.snapshot == s))
                .then_some((r, out))
        }) else {
            return Ok(None);
        };
        let p = dir.join(RUNS).join(&rec.id).join(out);
        match fs::symlink_metadata(&p) {
            Ok(_) => contained_file(&dir, &p, || {
                Error::NotFound(format!("play of run {}", rec.id))
            })
            .map(Some),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(Error::io(&p, e)),
        }
    }

    /// 只留文字时跟着媒体进回收站的转录产物：各次的 `play` 产物和 `keyframes/` 下的图片。
    /// 回从条目文件夹起的相对路径。
    pub(crate) fn run_binaries(&self, id: &ItemId) -> Result<Vec<String>> {
        let dir = self.item_dir(id);
        let mut out = Vec::new();
        for r in self.runs(id)? {
            let base = format!("{RUNS}/{}", r.id);
            for o in r.outputs.iter().filter(|o| o.kind == "play") {
                let rel = format!("{base}/{}", o.path);
                if dir.join(&rel).is_file() {
                    out.push(rel);
                }
            }
            // 只认真的文件夹：在 usuyo 外被换成符号链接的不跟进去，免得把条目外的文件挪进回收站。
            let frames = dir.join(&base).join(KEYFRAMES_DIR);
            match fs::symlink_metadata(&frames) {
                Ok(meta) if meta.is_dir() => {
                    for f in list_files(&frames)? {
                        out.push(format!("{base}/{KEYFRAMES_DIR}/{f}"));
                    }
                }
                Ok(_) => tracing::warn!(item = %id, run = %r.id, "keyframes is not a folder"),
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
                Err(e) => return Err(Error::io(&frames, e)),
            }
        }
        Ok(out)
    }

    /// 各快照里获取插件存下的平台字幕，按快照的先后。文件不在了的跳过、记日志。
    ///
    /// 调用方：WP7（条目页里和机器转的口播并排、索引的 `asr` 层）。
    pub fn platform_subtitles(&self, id: &ItemId) -> Result<Vec<PlatformSubtitle>> {
        let item = self.item(id)?;
        let dir = self.item_dir(id);
        let mut out = Vec::new();
        for s in &item.manifest.snapshots {
            let rec = self.fetch_record(id, &s.id)?;
            for m in rec.media.iter().filter(|m| m.kind == "subtitle") {
                let p = dir.join("raw").join(&s.id).join(&m.path);
                let file = match contained_file(&dir, &p, || Error::NotFound(m.path.clone())) {
                    Ok(f) => f,
                    Err(Error::NotFound(path)) => {
                        tracing::warn!(item = %id, snapshot = %s.id, path, "subtitle file not found");
                        continue;
                    }
                    Err(e) => return Err(e),
                };
                let lang = m
                    .extra
                    .get("lang")
                    .and_then(|v| v.as_str())
                    .filter(|l| !l.is_empty())
                    .map(str::to_owned)
                    .or_else(|| lang_from_name(&m.path));
                out.push(PlatformSubtitle {
                    snapshot: s.id.clone(),
                    path: m.path.clone(),
                    file,
                    lang,
                });
            }
        }
        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn subtitle_language_from_file_name() {
        assert_eq!(
            lang_from_name("media/subtitle.zh.vtt").as_deref(),
            Some("zh")
        );
        assert_eq!(
            lang_from_name("subtitle.zh-Hans.vtt").as_deref(),
            Some("zh-Hans")
        );
        for none in [
            "media/subtitle.vtt",
            "media/captions.zh.vtt",
            "subtitle..vtt",
            "subtitle.a.b.vtt",
        ] {
            assert_eq!(lang_from_name(none), None, "{none}");
        }
    }
}
