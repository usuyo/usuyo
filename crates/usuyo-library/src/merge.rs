//! 三方合并（SPEC §1.8、§1.2.1 最后一列），纯函数；同步引擎和读回都用它。

use std::collections::BTreeSet;

use serde_json::{Map, Value};
use similar::{Algorithm, DiffOp, capture_diff_slices};
use usuyo_contract::{ItemState, Manifest, Snapshot};

/// 集合：两边各自加的都留，一边去掉的就去掉；没有底本取并集。结果排好序。
pub fn merge_set(base: Option<&[String]>, ours: &[String], theirs: &[String]) -> Vec<String> {
    let o: BTreeSet<&String> = ours.iter().collect();
    let t: BTreeSet<&String> = theirs.iter().collect();
    let out: BTreeSet<&String> = match base {
        None => o.union(&t).copied().collect(),
        Some(base) => {
            let b: BTreeSet<&String> = base.iter().collect();
            o.union(&t)
                .copied()
                .filter(|x| {
                    let removed = b.contains(x) && (!o.contains(x) || !t.contains(x));
                    !removed
                })
                .collect()
        }
    };
    out.into_iter().cloned().collect()
}

/// 标量：只有两边都改了、改得不一样才合不上（回 `None`）。
pub fn merge_scalar<T: PartialEq + Clone>(base: Option<&T>, ours: &T, theirs: &T) -> Option<T> {
    if ours == theirs {
        return Some(ours.clone());
    }
    match base {
        Some(b) if b == ours => Some(theirs.clone()),
        Some(b) if b == theirs => Some(ours.clone()),
        _ => None,
    }
}

/// 一边相对底本的一处改动：底本的 `[start, end)` 行换成 `lines`。
#[derive(Debug, Clone, PartialEq)]
struct Change<'a> {
    start: usize,
    end: usize,
    lines: Vec<&'a str>,
}

fn changes<'a>(base: &[&'a str], side: &[&'a str]) -> Vec<Change<'a>> {
    let mut out: Vec<Change<'a>> = Vec::new();
    for op in capture_diff_slices(Algorithm::Myers, base, side) {
        let (start, end, new) = match op {
            DiffOp::Equal { .. } => continue,
            DiffOp::Delete {
                old_index, old_len, ..
            } => (old_index, old_index + old_len, Vec::new()),
            DiffOp::Insert {
                old_index,
                new_index,
                new_len,
            } => (
                old_index,
                old_index,
                side[new_index..new_index + new_len].to_vec(),
            ),
            DiffOp::Replace {
                old_index,
                old_len,
                new_index,
                new_len,
            } => (
                old_index,
                old_index + old_len,
                side[new_index..new_index + new_len].to_vec(),
            ),
        };
        // 首尾相接的几处并成一处。
        match out.last_mut() {
            Some(last) if last.end == start => {
                last.end = end;
                last.lines.extend(new);
            }
            _ => out.push(Change {
                start,
                end,
                lines: new,
            }),
        }
    }
    out
}

fn overlaps(a: &Change, b: &Change) -> bool {
    if a.start == a.end && b.start == b.end {
        return false; // 都是插入，另算
    }
    if a.start == a.end {
        return b.start < a.start && a.start < b.end;
    }
    if b.start == b.end {
        return a.start < b.start && b.start < a.end;
    }
    a.start < b.end && b.start < a.end
}

/// 正文按行三方合并。两边在同一处都只是插入时两段都留，`ours_first` 决定先后
/// （按 `deviceId` 排）；改成一样的算一次；改到重叠的行回 `None`，相邻的不算。
pub fn merge_lines(base: &str, ours: &str, theirs: &str, ours_first: bool) -> Option<String> {
    let b: Vec<&str> = base.split_inclusive('\n').collect();
    let o: Vec<&str> = ours.split_inclusive('\n').collect();
    let t: Vec<&str> = theirs.split_inclusive('\n').collect();
    let co = changes(&b, &o);
    let ct = changes(&b, &t);
    // 合在一起按底本位置排，同一处的插入排在替换前面。
    let mut all: Vec<(Change, bool)> = co
        .into_iter()
        .map(|c| (c, true))
        .chain(ct.into_iter().map(|c| (c, false)))
        .collect();
    all.sort_by_key(|(c, mine)| (c.start, c.end != c.start, c.end, *mine != ours_first));
    let mut merged: Vec<(Change, bool)> = Vec::with_capacity(all.len());
    for (c, mine) in all {
        if let Some((prev, prev_mine)) = merged.last()
            && *prev_mine != mine
        {
            if *prev == c {
                continue; // 两边改成一样的
            }
            if overlaps(prev, &c) {
                return None;
            }
        }
        if merged.iter().any(|(p, m)| *m != mine && overlaps(p, &c)) {
            return None;
        }
        merged.push((c, mine));
    }
    let mut out = String::with_capacity(ours.len().max(theirs.len()));
    let mut pos = 0;
    for (c, _) in &merged {
        for line in &b[pos..c.start.max(pos)] {
            out.push_str(line);
        }
        for line in &c.lines {
            out.push_str(line);
        }
        pos = pos.max(c.end);
    }
    for line in &b[pos..] {
        out.push_str(line);
    }
    Some(out)
}

/// front matter 按字段：列表按集合，标量两边都改了才合不上。回合不上的键。
pub fn merge_fields(
    base: Option<&Map<String, Value>>,
    ours: &Map<String, Value>,
    theirs: &Map<String, Value>,
) -> Result<Map<String, Value>, Vec<String>> {
    let keys: BTreeSet<&String> = ours.keys().chain(theirs.keys()).collect();
    let mut out = Map::new();
    let mut conflicts = Vec::new();
    for k in keys {
        let o = ours.get(k).unwrap_or(&Value::Null);
        let t = theirs.get(k).unwrap_or(&Value::Null);
        let b = base.map(|b| b.get(k).unwrap_or(&Value::Null));
        let merged = if o.is_array() || t.is_array() {
            let strings = |v: Option<&Value>| -> Vec<String> {
                v.and_then(Value::as_array)
                    .map(|a| {
                        a.iter()
                            .filter_map(|x| x.as_str().map(str::to_owned))
                            .collect()
                    })
                    .unwrap_or_default()
            };
            let bs = b.map(|b| strings(Some(b)));
            let set = merge_set(bs.as_deref(), &strings(Some(o)), &strings(Some(t)));
            Some(Value::Array(set.into_iter().map(Value::String).collect()))
        } else {
            merge_scalar(b, o, t)
        };
        match merged {
            Some(Value::Null) => {}
            Some(v) => {
                out.insert(k.clone(), v);
            }
            None => conflicts.push(k.clone()),
        }
    }
    if conflicts.is_empty() {
        Ok(out)
    } else {
        Err(conflicts)
    }
}

fn merge_by_id<T: Clone, K: Ord>(
    base: Option<&[T]>,
    ours: &[T],
    theirs: &[T],
    key: impl Fn(&T) -> K,
) -> Vec<T> {
    let bk: Option<BTreeSet<K>> = base.map(|b| b.iter().map(&key).collect());
    let ok: BTreeSet<K> = ours.iter().map(&key).collect();
    let tk: BTreeSet<K> = theirs.iter().map(&key).collect();
    let keep = |k: &K| match &bk {
        None => true,
        Some(b) => !(b.contains(k) && (!ok.contains(k) || !tk.contains(k))),
    };
    let mut out: Vec<T> = ours.iter().filter(|x| keep(&key(x))).cloned().collect();
    out.extend(
        theirs
            .iter()
            .filter(|x| !ok.contains(&key(x)) && keep(&key(x)))
            .cloned(),
    );
    out
}

/// manifest 按 SPEC §1.2.1 最后一列逐字段合并。跟主快照、重算的字段先取 `ours`，
/// 调用方合并后再按主快照重算（`completeness` 已经取了两边高的）。
pub fn merge_manifest(base: Option<&Manifest>, ours: &Manifest, theirs: &Manifest) -> Manifest {
    let mut m = ours.clone();
    let set = |f: fn(&Manifest) -> &Vec<String>| {
        merge_set(base.map(|b| f(b).as_slice()), f(ours), f(theirs))
    };
    m.aliases = set(|m| &m.aliases);
    m.tags.user = set(|m| &m.tags.user);
    m.tags.dismissed = set(|m| &m.tags.dismissed);
    m.topics = set(|m| &m.topics);
    m.same_as = set(|m| &m.same_as);
    m.not_same_as = set(|m| &m.not_same_as);
    m.identities = merge_by_id(
        base.map(|b| b.identities.as_slice()),
        &ours.identities,
        &theirs.identities,
        |i| i.key(),
    );
    m.origins = merge_by_id(
        base.map(|b| b.origins.as_slice()),
        &ours.origins,
        &theirs.origins,
        |o| o.id.clone(),
    );
    m.snapshots = merge_by_id::<Snapshot, String>(
        base.map(|b| b.snapshots.as_slice()),
        &ours.snapshots,
        &theirs.snapshots,
        |s| s.id.clone(),
    );
    for (mine, other) in [
        (&mut m.tags.platform, &theirs.tags.platform),
        (&mut m.tags.engine, &theirs.tags.engine),
    ] {
        for t in other {
            match mine.iter_mut().find(|x| x.tag == t.tag) {
                Some(x) => {
                    if (&t.at, &t.by) > (&x.at, &x.by) {
                        *x = t.clone();
                    }
                }
                None => mine.push(t.clone()),
            }
        }
    }
    if theirs.added_at < m.added_at {
        m.added_at = theirs.added_at.clone();
    }
    m.private = if ours.private == theirs.private {
        ours.private
    } else {
        match base {
            Some(b) if b.private == ours.private => theirs.private,
            Some(b) if b.private == theirs.private => ours.private,
            _ => true,
        }
    };
    let state_changed = |s: &Manifest| base.is_none_or(|b| b.state != s.state);
    let take_theirs_state = match (state_changed(ours), state_changed(theirs)) {
        (false, true) => true,
        (true, true) => rank(theirs.state) > rank(ours.state),
        _ => false,
    };
    if take_theirs_state {
        m.state = theirs.state;
        m.state_at = theirs.state_at.clone();
        m.text_only = theirs.text_only.clone();
    }
    m.completeness = ours.completeness.max(theirs.completeness);
    for (k, v) in &theirs.extra {
        m.extra.entry(k.clone()).or_insert_with(|| v.clone());
    }
    m.sort_arrays();
    m
}

fn rank(s: ItemState) -> u8 {
    match s {
        ItemState::Normal => 0,
        ItemState::TextOnly => 1,
        ItemState::Trashed => 2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn v(xs: &[&str]) -> Vec<String> {
        xs.iter().map(|s| (*s).to_owned()).collect()
    }

    #[test]
    fn sets() {
        assert_eq!(
            merge_set(Some(&v(&["a", "b"])), &v(&["a", "c"]), &v(&["a", "b", "d"])),
            v(&["a", "c", "d"])
        );
        assert_eq!(merge_set(None, &v(&["b"]), &v(&["a"])), v(&["a", "b"]));
    }

    #[test]
    fn scalars() {
        assert_eq!(merge_scalar(Some(&1), &1, &2), Some(2));
        assert_eq!(merge_scalar(Some(&1), &3, &3), Some(3));
        assert_eq!(merge_scalar(Some(&1), &2, &3), None);
        assert_eq!(merge_scalar(None, &2, &3), None);
    }

    #[test]
    fn lines_non_overlapping_and_adjacent_edits_merge() {
        let base = "a\nb\nc\nd\n";
        assert_eq!(
            merge_lines(base, "A\nb\nc\nd\n", "a\nb\nc\nD\n", true).unwrap(),
            "A\nb\nc\nD\n"
        );
        // 相邻的行各改一行不算合不上
        assert_eq!(
            merge_lines(base, "a\nB\nc\nd\n", "a\nb\nC\nd\n", true).unwrap(),
            "a\nB\nC\nd\n"
        );
    }

    #[test]
    fn lines_both_inserted_at_same_place_keep_both_in_device_order() {
        let base = "正文\n";
        let ours = "正文\n[^aaaaa]: 一\n";
        let theirs = "正文\n[^bbbbb]: 二\n";
        assert_eq!(
            merge_lines(base, ours, theirs, true).unwrap(),
            "正文\n[^aaaaa]: 一\n[^bbbbb]: 二\n"
        );
        assert_eq!(
            merge_lines(base, ours, theirs, false).unwrap(),
            "正文\n[^bbbbb]: 二\n[^aaaaa]: 一\n"
        );
    }

    #[test]
    fn lines_identical_edits_count_once_and_overlaps_conflict() {
        let base = "a\nb\n";
        assert_eq!(
            merge_lines(base, "a\nX\n", "a\nX\n", true).unwrap(),
            "a\nX\n"
        );
        assert_eq!(
            merge_lines(base, "a\nb\nnew\n", "a\nb\nnew\n", true).unwrap(),
            "a\nb\nnew\n"
        );
        assert_eq!(merge_lines(base, "a\nX\n", "a\nY\n", true), None);
        assert_eq!(merge_lines(base, "a\n", "a\nY\n", true), None);
        assert_eq!(
            merge_lines(base, base, "a\nb\nc\n", true).unwrap(),
            "a\nb\nc\n"
        );
    }

    #[test]
    fn manifest_columns() {
        let base: Manifest = serde_json::from_str(include_str!(
            "../../usuyo-contract/testdata/manifest.minimal.json"
        ))
        .unwrap();
        let mut ours = base.clone();
        let mut theirs = base.clone();
        ours.tags.user = v(&["a"]);
        ours.state = ItemState::TextOnly;
        ours.completeness = usuyo_contract::Completeness::Partial;
        theirs.tags.user = v(&["b"]);
        theirs.state = ItemState::Trashed;
        theirs.state_at = Some("2026-11-06T00:00:00.000Z".into());
        theirs.added_at = "2026-11-01T00:00:00.000Z".into();
        theirs.private = true;
        let m = merge_manifest(Some(&base), &ours, &theirs);
        assert_eq!(m.tags.user, v(&["a", "b"]));
        assert_eq!(m.state, ItemState::Trashed);
        assert_eq!(m.state_at.as_deref(), Some("2026-11-06T00:00:00.000Z"));
        assert_eq!(m.added_at, "2026-11-01T00:00:00.000Z");
        assert!(m.private);
        assert_eq!(m.completeness, usuyo_contract::Completeness::Partial);
        // 只有一边改了 state：取改了的那边，不比先后
        let m = merge_manifest(Some(&base), &base, &ours);
        assert_eq!(m.state, ItemState::TextOnly);
    }

    #[test]
    fn fields() {
        let b: Map<String, Value> =
            serde_json::from_str(r#"{"title":"t","keywords":["a"]}"#).unwrap();
        let o: Map<String, Value> =
            serde_json::from_str(r#"{"title":"t2","keywords":["a","b"]}"#).unwrap();
        let t: Map<String, Value> =
            serde_json::from_str(r#"{"title":"t","keywords":["c"]}"#).unwrap();
        let m = merge_fields(Some(&b), &o, &t).unwrap();
        assert_eq!(m["title"], "t2");
        assert_eq!(m["keywords"], serde_json::json!(["b", "c"]));
        let t2: Map<String, Value> = serde_json::from_str(r#"{"title":"t3"}"#).unwrap();
        assert_eq!(merge_fields(Some(&b), &o, &t2).unwrap_err(), vec!["title"]);
    }
}
