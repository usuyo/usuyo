//! 演练模式用的流程（docs/platforms/phone.md §6）：流程是数据，由驱动 iPhone 的获取插件带在
//! `${PLUGIN_DIR}/flows/*.json` 里，宿主不认识任何平台。一个文件一个流程：
//!
//! ```json
//! {"name": "给人看的流程名", "steps": [
//!   {"expect": ["这一屏一定有的字"], "tap": ["要点的字"], "variants": [{"expect": [], "tap": []}]}
//! ]}
//! ```
//!
//! `allow`、`next` 这些跑流程才用的字段这里不看。比的时候不分大小写，屏上一段字里含着要找的字就算找到。

use std::path::{Path, PathBuf};

use serde::Deserialize;

use crate::view::{DryRunMatch, DryRunTarget, ExpectCount};

/// 一个流程文件最多多大。
const FLOW_CAP: u64 = 256 << 10;

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct Flow {
    pub name: String,
    pub steps: Vec<Step>,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct Step {
    pub expect: Vec<String>,
    #[serde(default)]
    pub tap: Vec<String>,
    #[serde(default)]
    pub variants: Vec<Variant>,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct Variant {
    pub expect: Vec<String>,
    #[serde(default)]
    pub tap: Vec<String>,
}

/// 这几个目录里的 `*.json`，按文件名排；读不了、不是流程的记一笔跳过。
pub(crate) fn load(dirs: &[PathBuf]) -> Vec<Flow> {
    let mut flows = Vec::new();
    for dir in dirs {
        let entries = match std::fs::read_dir(dir) {
            Ok(e) => e,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => continue,
            Err(e) => {
                tracing::warn!(dir = %dir.display(), error = %e, "flows folder not readable");
                continue;
            }
        };
        let mut files: Vec<PathBuf> = entries
            .filter_map(|e| e.ok().map(|e| e.path()))
            .filter(|p| p.extension().is_some_and(|x| x == "json") && p.is_file())
            .collect();
        files.sort();
        for path in files {
            match read(&path) {
                Ok(f) => flows.push(f),
                Err(e) => tracing::warn!(path = %path.display(), error = %e, "flow file skipped"),
            }
        }
    }
    flows
}

fn read(path: &Path) -> Result<Flow, String> {
    let len = std::fs::metadata(path).map_err(|e| e.to_string())?.len();
    if len > FLOW_CAP {
        return Err(format!("larger than {FLOW_CAP} bytes"));
    }
    let bytes = std::fs::read(path).map_err(|e| e.to_string())?;
    serde_json::from_slice(&bytes).map_err(|e| e.to_string())
}

fn fold(s: &str) -> String {
    s.trim().to_lowercase()
}

/// 屏上 `texts` 里有没有一段含着 `label`。
fn on_screen(texts: &[String], label: &str) -> bool {
    let want = fold(label);
    !want.is_empty() && texts.iter().any(|t| t.contains(&want))
}

/// 最像的那一步：该有的字对上得最多的；一样多时比例高的、排前的。至少对上一段、过半才算像。
pub(crate) fn best(flows: &[Flow], screen: &[String]) -> Option<DryRunMatch> {
    let texts: Vec<String> = screen.iter().map(|t| fold(t)).collect();
    let mut best: Option<(usize, usize, DryRunMatch)> = None;
    for flow in flows {
        for (i, step) in flow.steps.iter().enumerate() {
            let candidates = std::iter::once((&step.expect, &step.tap))
                .chain(step.variants.iter().map(|v| (&v.expect, &v.tap)));
            for (expect, tap) in candidates {
                let total = expect.len();
                let matched = expect.iter().filter(|e| on_screen(&texts, e)).count();
                if matched == 0 || matched * 2 < total {
                    continue;
                }
                let better = best.as_ref().is_none_or(|(m, t, _)| {
                    matched > *m || (matched == *m && matched * t > *m * total)
                });
                if better {
                    best = Some((
                        matched,
                        total,
                        DryRunMatch {
                            flow: flow.name.clone(),
                            step: i + 1,
                            expect: ExpectCount { matched, total },
                            targets: tap
                                .iter()
                                .map(|label| DryRunTarget {
                                    label: label.clone(),
                                    found: on_screen(&texts, label),
                                })
                                .collect(),
                        },
                    ));
                }
            }
        }
    }
    best.map(|(_, _, m)| m)
}
