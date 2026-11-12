//! 获取插件的子命令（协议 §3.2）：读参数，作者照 [`Call`] 分派到每种方法（§2.3.1）。
//!
//! ```no_run
//! use usuyo_plugin_kit::fetch::{Call, USAGE};
//!
//! let args: Vec<String> = std::env::args().skip(1).collect();
//! match Call::parse(&args) {
//!     Some(Call::Fetch { method, .. }) if method == "guest" => { /* … */ }
//!     Some(_) => { /* … */ }
//!     None => {
//!         eprintln!("{USAGE}");
//!         std::process::exit(2);
//!     }
//! }
//! ```

use std::path::PathBuf;

use usuyo_plugin::protocol::{OutputKind, Rfc3339};

/// 读不出来时打到 stderr、退出码 2 的那句。
pub const USAGE: &str = "usage: <plugin> canonicalize <url> | fetch <url-or-ref> --out <dir> --method <id> [--want a,b] | sources --method <id> [--link <url>] | list <source> --method <id> [--cursor <c>] [--since <time>] | health [--probe]";

/// 宿主的一次调用。`canonicalize`、`health` 不分方法。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Call {
    Canonicalize {
        url: String,
    },
    Fetch {
        input: String,
        out: PathBuf,
        method: String,
        /// 只要这几样；没给是清单里这种方法的 `outputs` 全要。
        want: Option<Vec<OutputKind>>,
    },
    Sources {
        method: String,
        /// 来源从这条链接开始（方法的 `list.link`）。
        link: Option<String>,
    },
    List {
        source: String,
        method: String,
        cursor: Option<String>,
        since: Option<Rfc3339>,
    },
    Health {
        probe: bool,
    },
}

impl Call {
    /// 读子命令和参数（不含程序名）。不认识的子命令、参数，缺了要的，都回 `None`。
    pub fn parse(args: &[String]) -> Option<Call> {
        let (command, rest) = args.split_first()?;
        match command.as_str() {
            "canonicalize" => match rest {
                [url] => Some(Call::Canonicalize { url: url.clone() }),
                _ => None,
            },
            "health" => match rest {
                [] => Some(Call::Health { probe: false }),
                [flag] if flag == "--probe" => Some(Call::Health { probe: true }),
                _ => None,
            },
            "fetch" => {
                let (input, rest) = rest.split_first()?;
                let mut f = Flags::read(rest, &["--out", "--method", "--want"])?;
                Some(Call::Fetch {
                    input: input.clone(),
                    out: PathBuf::from(f.take("--out")?),
                    method: f.take("--method")?,
                    want: match f.take("--want") {
                        None => None,
                        Some(list) => Some(
                            list.split(',')
                                .map(OutputKind::parse)
                                .collect::<Option<Vec<_>>>()?,
                        ),
                    },
                })
            }
            "sources" => {
                let mut f = Flags::read(rest, &["--method", "--link"])?;
                Some(Call::Sources {
                    method: f.take("--method")?,
                    link: f.take("--link"),
                })
            }
            "list" => {
                let (source, rest) = rest.split_first()?;
                let mut f = Flags::read(rest, &["--method", "--cursor", "--since"])?;
                Some(Call::List {
                    source: source.clone(),
                    method: f.take("--method")?,
                    cursor: f.take("--cursor"),
                    since: match f.take("--since") {
                        None => None,
                        Some(t) => Some(Rfc3339::try_from(t).ok()?),
                    },
                })
            }
            _ => None,
        }
    }

    /// 这次调用的方法；`canonicalize`、`health` 没有。
    pub fn method(&self) -> Option<&str> {
        match self {
            Call::Fetch { method, .. }
            | Call::Sources { method, .. }
            | Call::List { method, .. } => Some(method),
            Call::Canonicalize { .. } | Call::Health { .. } => None,
        }
    }
}

/// `--名字 值` 一对对；每个名字最多一次。
struct Flags(Vec<(String, String)>);

impl Flags {
    fn read(args: &[String], known: &[&str]) -> Option<Flags> {
        let mut pairs: Vec<(String, String)> = Vec::new();
        let mut it = args.iter();
        while let Some(flag) = it.next() {
            if !known.contains(&flag.as_str()) || pairs.iter().any(|(k, _)| k == flag) {
                return None;
            }
            pairs.push((flag.clone(), it.next()?.clone()));
        }
        Some(Flags(pairs))
    }

    fn take(&mut self, name: &str) -> Option<String> {
        let i = self.0.iter().position(|(k, _)| k == name)?;
        Some(self.0.remove(i).1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(parts: &[&str]) -> Vec<String> {
        parts.iter().map(|s| (*s).to_owned()).collect()
    }

    #[test]
    fn reads_each_call() {
        assert_eq!(
            Call::parse(&args(&[
                "fetch",
                "https://a.test/1",
                "--out",
                "/o",
                "--method",
                "guest",
                "--want",
                "text,media:video"
            ])),
            Some(Call::Fetch {
                input: "https://a.test/1".into(),
                out: PathBuf::from("/o"),
                method: "guest".into(),
                want: Some(vec![OutputKind::Text, OutputKind::Video]),
            })
        );
        assert_eq!(
            Call::parse(&args(&[
                "sources",
                "--method",
                "public",
                "--link",
                "https://a.test/u/1"
            ])),
            Some(Call::Sources {
                method: "public".into(),
                link: Some("https://a.test/u/1".into()),
            })
        );
        let Some(Call::List {
            source,
            method,
            cursor,
            since,
        }) = Call::parse(&args(&[
            "list",
            "a:fav:1",
            "--method",
            "public",
            "--cursor",
            "2",
            "--since",
            "2026-10-31T00:00:00Z",
        ]))
        else {
            panic!("list");
        };
        assert_eq!(
            (source.as_str(), method.as_str(), cursor.as_deref()),
            ("a:fav:1", "public", Some("2"))
        );
        assert!(since.is_some());
        assert_eq!(
            Call::parse(&args(&["health", "--probe"])),
            Some(Call::Health { probe: true })
        );
        assert_eq!(
            Call::parse(&args(&["canonicalize", "https://a.test/"]))
                .and_then(|c| c.method().map(str::to_owned)),
            None
        );
    }

    #[test]
    fn refuses_what_it_does_not_know() {
        // 少了 --method、不认识的参数、重复的参数、坏的 want 和时间
        for bad in [
            &["fetch", "u", "--out", "/o"][..],
            &["fetch", "u", "--out", "/o", "--method", "m", "--nope", "x"],
            &[
                "fetch", "u", "--out", "/o", "--method", "m", "--method", "n",
            ],
            &[
                "fetch", "u", "--out", "/o", "--method", "m", "--want", "video",
            ],
            &["list", "s", "--method", "m", "--since", "yesterday"],
            &["sources"],
            &["health", "--now"],
            &["unfavorite", "u"],
        ] {
            assert_eq!(Call::parse(&args(bad)), None, "{bad:?}");
        }
    }
}
