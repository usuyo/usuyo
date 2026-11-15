//! 获取策略（协议 §2.3.2）：某平台某项功能上指定的方式 ＞ 这个平台的策略 ＞ 总策略。
//! 策略是一张有序的规则表，自上而下第一条匹配的生效；去向是几段方式（一组或一种），按顺序回退，
//! 空的是「不获取」。三个预设是写好的规则表。这里只求值，不碰插件和文件；存在按机器的 `settings.json`
//! 的 `fetchStrategy`。

use std::borrow::Cow;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use usuyo_plugin::protocol::{Access, Account, ListAction, OutputKind};

/// 用户看得懂的一项功能：取一条时的几样从 `outputs` 归并，列清单时按来源的 `action` 分。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Function {
    /// 正文：`text`、`html`、`meta`。
    Text,
    Images,
    /// 视频：`media:video`、`subtitle`。
    Video,
    Audio,
    Comments,
    ListFavorite,
    ListLike,
    ListShare,
    ListView,
    /// 列出存过的，来源没写 `action`（或写的是 `unknown`）。
    List,
}

impl Function {
    pub const ALL: [Function; 10] = [
        Function::Text,
        Function::Images,
        Function::Video,
        Function::Audio,
        Function::Comments,
        Function::ListFavorite,
        Function::ListLike,
        Function::ListShare,
        Function::ListView,
        Function::List,
    ];

    pub fn key(self) -> &'static str {
        match self {
            Function::Text => "text",
            Function::Images => "images",
            Function::Video => "video",
            Function::Audio => "audio",
            Function::Comments => "comments",
            Function::ListFavorite => "list:favorite",
            Function::ListLike => "list:like",
            Function::ListShare => "list:share",
            Function::ListView => "list:view",
            Function::List => "list",
        }
    }

    pub fn parse(key: &str) -> Option<Function> {
        Function::ALL.into_iter().find(|f| f.key() == key)
    }

    pub fn of_output(kind: OutputKind) -> Function {
        match kind {
            OutputKind::Text | OutputKind::Html | OutputKind::Meta => Function::Text,
            OutputKind::Images => Function::Images,
            OutputKind::Video | OutputKind::Subtitle => Function::Video,
            OutputKind::Audio => Function::Audio,
            OutputKind::Comments => Function::Comments,
        }
    }

    pub fn of_action(action: Option<ListAction>) -> Function {
        match action {
            Some(ListAction::Favorite) => Function::ListFavorite,
            Some(ListAction::Like) => Function::ListLike,
            Some(ListAction::Share) => Function::ListShare,
            Some(ListAction::View) => Function::ListView,
            Some(ListAction::Unknown) | None => Function::List,
        }
    }

    pub fn is_list(self) -> bool {
        self >= Function::ListFavorite
    }
}

impl fmt::Display for Function {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.key())
    }
}

impl Serialize for Function {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(self.key())
    }
}

impl<'de> Deserialize<'de> for Function {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let key = String::deserialize(d)?;
        Function::parse(&key)
            .ok_or_else(|| serde::de::Error::custom(format!("`{key}` is not a function")))
    }
}

/// 总策略（或一个平台的策略）选哪一张规则表。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Preset {
    /// 只用账号类型是 `none`、`visitor` 的方式；别的一律不调，哪怕缺内容。
    NoAccountOnly,
    /// 现在的先后：层级低的先，同一层先不带账号的。
    #[default]
    NoAccountFirst,
    /// 用了账号的方式在前，不带账号的垫后。
    AccountFirst,
    /// 你自己的规则表（[`Strategy::rules`]）。
    Custom,
}

/// 去向里的一段：某一种方式（`method`），或一组方式（按账号类型、层级圈出来）：账号类型和层级都对得上的；
/// `accounts` 空是任何账号类型，`access` 空是 `private-api` 以下的几层（`private-api`、`client-hook` 要写明才进来）。
/// 组内照 [`Way`] 的先后。
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Step {
    /// `<插件 id>#<方法 id>`；写了它就不写 `accounts`、`access`。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub method: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub accounts: Vec<Account>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub access: Vec<Access>,
}

const NO_ACCOUNT: [Account; 2] = [Account::None, Account::Visitor];
const WITH_ACCOUNT: [Account; 5] = [
    Account::OwnBrowser,
    Account::OwnPhone,
    Account::OwnApp,
    Account::InUsuyo,
    Account::Developer,
];

impl Step {
    fn admits(&self, w: &Way) -> bool {
        if let Some(method) = &self.method {
            return &w.key == method;
        }
        (self.accounts.is_empty() || self.accounts.contains(&w.account))
            && if self.access.is_empty() {
                w.access < Access::PrivateApi
            } else {
                self.access.contains(&w.access)
            }
    }

    fn group(accounts: &[Account]) -> Step {
        Step {
            accounts: accounts.to_vec(),
            ..Step::default()
        }
    }
}

/// 一条规则：匹配（平台、功能，不写是任意）→ 去向。`use` 空是「不获取」。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Rule {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub platform: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub function: Option<Function>,
    #[serde(rename = "use")]
    pub steps: Vec<Step>,
}

impl Rule {
    fn matches(&self, platform: &str, f: Function) -> bool {
        self.platform.as_deref().is_none_or(|p| p == platform)
            && self.function.is_none_or(|x| x == f)
    }

    fn catch_all(&self) -> bool {
        self.platform.is_none() && self.function.is_none()
    }
}

/// 某平台某项功能上指定的：一种方式（`<插件 id>#<方法 id>`），或「不获取」（`skip`）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Pick {
    Skip,
    Method(String),
}

impl Serialize for Pick {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        match self {
            Pick::Skip => s.serialize_str("skip"),
            Pick::Method(m) => s.serialize_str(m),
        }
    }
}

impl<'de> Deserialize<'de> for Pick {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let text = String::deserialize(d)?;
        if text == "skip" {
            Ok(Pick::Skip)
        } else if text
            .split_once('#')
            .is_some_and(|(p, m)| !p.is_empty() && !m.is_empty())
        {
            Ok(Pick::Method(text))
        } else {
            Err(serde::de::Error::custom(format!(
                "`{text}` is neither `skip` nor `<plugin>#<method>`"
            )))
        }
    }
}

/// 一个平台自己的：另选的预设（不写是跟随总策略）、每项功能上指定的。
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PlatformStrategy {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub preset: Option<Preset>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub functions: BTreeMap<Function, Pick>,
}

impl PlatformStrategy {
    fn is_empty(&self) -> bool {
        self.preset.is_none() && self.functions.is_empty()
    }
}

/// `settings.json` 的 `fetchStrategy`。没写是「优先不使用账号」。
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Strategy {
    #[serde(default)]
    pub preset: Preset,
    /// 自定义的规则表；选了预设时也留着，换回「自定义」时还在。
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub rules: Vec<Rule>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub platforms: BTreeMap<String, PlatformStrategy>,
}

/// 预设各自的规则表。
pub fn preset_rules(preset: Preset) -> Vec<Rule> {
    let all = |steps| {
        vec![Rule {
            platform: None,
            function: None,
            steps,
        }]
    };
    match preset {
        Preset::NoAccountOnly => all(vec![Step::group(&NO_ACCOUNT)]),
        Preset::NoAccountFirst => all(vec![Step::group(&[])]),
        Preset::AccountFirst => all(vec![Step::group(&WITH_ACCOUNT), Step::group(&NO_ACCOUNT)]),
        Preset::Custom => Vec::new(),
    }
}

impl Strategy {
    /// 写进去之前查一遍：自定义的规则表不空、最后一条是兜底（平台、功能都不写）；平台另选的不能是「自定义」。
    /// 去掉空的平台项。
    pub fn checked(mut self) -> Result<Strategy, String> {
        if self.preset == Preset::Custom && self.rules.is_empty() {
            return Err("a custom strategy needs at least one rule".into());
        }
        if let Some(last) = self.rules.last()
            && !last.catch_all()
        {
            return Err("the last rule must match any platform and any function".into());
        }
        if self.rules.len() > 200 {
            return Err("at most 200 rules".into());
        }
        for step in self.rules.iter().flat_map(|r| &r.steps) {
            if step.method.is_some() && (!step.accounts.is_empty() || !step.access.is_empty()) {
                return Err("a step names one method or a group, not both".into());
            }
        }
        for (p, s) in &self.platforms {
            if s.preset == Some(Preset::Custom) {
                return Err(format!(
                    "platform {p}: pick a preset or follow the strategy"
                ));
            }
        }
        self.platforms.retain(|_, s| !s.is_empty());
        Ok(self)
    }

    /// 这个平台照哪张规则表、是从哪来的。
    fn table(&self, platform: &str) -> (Decided, Cow<'_, [Rule]>) {
        match self.platforms.get(platform).and_then(|p| p.preset) {
            Some(p) => (Decided::Platform, Cow::Owned(preset_rules(p))),
            None if self.preset == Preset::Custom => {
                (Decided::Strategy, Cow::Borrowed(&self.rules))
            }
            None => (Decided::Strategy, Cow::Owned(preset_rules(self.preset))),
        }
    }

    /// 这个平台这项功能照什么取：`ways` 是这个平台上装着的、做了的方式，照 [`Way`] 的先后排好，
    /// 用得上用不上都在（指定的那一种用不上时不换别的，没装才回到策略）。只看 `serves` 里有 `f` 的。
    pub fn decide(&self, platform: &str, f: Function, ways: &[Way]) -> Decision {
        let serving = || {
            ways.iter()
                .enumerate()
                .filter(|(_, w)| w.serves.contains(&f))
        };
        match self
            .platforms
            .get(platform)
            .and_then(|p| p.functions.get(&f))
        {
            Some(Pick::Skip) => {
                return Decision {
                    from: Decided::Function,
                    skip: true,
                    order: Vec::new(),
                };
            }
            Some(Pick::Method(key)) => {
                if let Some((i, _)) = serving().find(|(_, w)| &w.key == key) {
                    return Decision {
                        from: Decided::Function,
                        skip: false,
                        order: vec![i],
                    };
                }
            }
            None => {}
        }
        let (from, rules) = self.table(platform);
        let Some(rule) = rules.iter().find(|r| r.matches(platform, f)) else {
            return Decision {
                from,
                skip: true,
                order: Vec::new(),
            };
        };
        let mut order: Vec<usize> = Vec::new();
        for step in &rule.steps {
            for (i, w) in serving() {
                if step.admits(w) && !order.contains(&i) {
                    order.push(i);
                }
            }
        }
        Decision {
            from,
            skip: rule.steps.is_empty(),
            order,
        }
    }
}

/// 求值时的一种方式。`access` 是它在这个平台上的那一层（`perPlatform` 算过的）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Way {
    pub key: String,
    pub account: Account,
    pub access: Access,
    pub serves: BTreeSet<Function>,
}

/// 照哪一层定的：功能上指定的、平台另选的预设、总策略。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Decided {
    Function,
    Platform,
    Strategy,
}

/// 求值的结果：`order` 是 `ways` 里的下标，先后就是试的先后；`skip` 是规则说「不获取」。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Decision {
    pub from: Decided,
    pub skip: bool,
    pub order: Vec<usize>,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn way(key: &str, account: Account, access: Access, serves: &[Function]) -> Way {
        Way {
            key: key.into(),
            account,
            access,
            serves: serves.iter().copied().collect(),
        }
    }

    /// 照 [`Way`] 的先后：层级低的先，同一层先不带账号的。
    fn ways() -> Vec<Way> {
        use Function::*;
        vec![
            way(
                "p/a#export",
                Account::Developer,
                Access::OfficialExport,
                &[Text],
            ),
            way(
                "p/a#guest",
                Account::None,
                Access::PublicWeb,
                &[Text, Images],
            ),
            way(
                "p/a#visitor",
                Account::Visitor,
                Access::PublicWeb,
                &[Text, Video],
            ),
            way(
                "p/a#session",
                Account::InUsuyo,
                Access::SessionApi,
                &[Video, ListFavorite],
            ),
            way(
                "p/a#signed",
                Account::None,
                Access::PrivateApi,
                &[Text, Video],
            ),
        ]
    }

    fn keys(s: &Strategy, platform: &str, f: Function) -> Vec<&'static str> {
        let ways = ways();
        let d = s.decide(platform, f, &ways);
        d.order
            .iter()
            .map(|&i| match ways[i].key.as_str() {
                "p/a#export" => "export",
                "p/a#guest" => "guest",
                "p/a#visitor" => "visitor",
                "p/a#session" => "session",
                _ => "signed",
            })
            .collect()
    }

    fn preset(p: Preset) -> Strategy {
        Strategy {
            preset: p,
            ..Strategy::default()
        }
    }

    #[test]
    fn the_default_is_todays_order_and_leaves_out_the_top_two_tiers() {
        let s = Strategy::default();
        // 层级在前、账号在后：开发者凭据的官方导出排在不带账号的公开页前面，和改成预设以前一样
        assert_eq!(
            keys(&s, "x", Function::Text),
            ["export", "guest", "visitor"]
        );
        assert_eq!(keys(&s, "x", Function::Video), ["visitor", "session"]);
        assert_eq!(keys(&s, "x", Function::ListFavorite), ["session"]);
    }

    #[test]
    fn each_preset_has_its_own_order() {
        let only = preset(Preset::NoAccountOnly);
        assert_eq!(keys(&only, "x", Function::Text), ["guest", "visitor"]);
        assert_eq!(keys(&only, "x", Function::Video), ["visitor"]);
        // 只有带账号的方式能列：这一项一种都不调，不算「不获取」
        let d = only.decide("x", Function::ListFavorite, &ways());
        assert!(d.order.is_empty() && !d.skip);
        let first = preset(Preset::AccountFirst);
        assert_eq!(
            keys(&first, "x", Function::Text),
            ["export", "guest", "visitor"]
        );
        assert_eq!(keys(&first, "x", Function::Video), ["session", "visitor"]);
    }

    #[test]
    fn a_function_pick_beats_the_platform_which_beats_the_strategy() {
        let mut s = preset(Preset::NoAccountOnly);
        s.platforms.insert(
            "x".into(),
            PlatformStrategy {
                preset: Some(Preset::AccountFirst),
                functions: [(Function::Text, Pick::Method("p/a#visitor".into()))].into(),
            },
        );
        let ways = ways();
        let d = s.decide("x", Function::Text, &ways);
        assert_eq!(
            (d.from, keys(&s, "x", Function::Text)),
            (Decided::Function, vec!["visitor"])
        );
        assert_eq!(
            s.decide("x", Function::Video, &ways).from,
            Decided::Platform
        );
        assert_eq!(keys(&s, "x", Function::Video), ["session", "visitor"]);
        // 别的平台照总策略
        assert_eq!(
            s.decide("y", Function::Video, &ways).from,
            Decided::Strategy
        );
        assert_eq!(keys(&s, "y", Function::Video), ["visitor"]);
    }

    #[test]
    fn skip_takes_nothing_and_a_pick_that_is_gone_falls_back_to_the_strategy() {
        let mut s = Strategy::default();
        s.platforms.insert(
            "x".into(),
            PlatformStrategy {
                preset: None,
                functions: [
                    (Function::Video, Pick::Skip),
                    (Function::Text, Pick::Method("p/gone#guest".into())),
                    // 指定的方式不做这一项，也当没装
                    (Function::Images, Pick::Method("p/a#session".into())),
                ]
                .into(),
            },
        );
        let d = s.decide("x", Function::Video, &ways());
        assert!(d.skip && d.order.is_empty());
        assert_eq!(
            s.decide("x", Function::Text, &ways()).from,
            Decided::Strategy
        );
        assert_eq!(
            keys(&s, "x", Function::Text),
            ["export", "guest", "visitor"]
        );
        assert_eq!(keys(&s, "x", Function::Images), ["guest"]);
    }

    #[test]
    fn custom_rules_match_top_down_and_the_last_one_catches_the_rest() {
        let s: Strategy = serde_json::from_value(serde_json::json!({
            "preset": "custom",
            "rules": [
                {"platform": "x", "function": "video", "use": [{"method": "p/a#signed"}, {"accounts": ["in-usuyo"]}]},
                {"function": "comments", "use": []},
                {"platform": "x", "use": [{"access": ["private-api"]}, {"accounts": ["none"]}]},
                {"use": [{"accounts": ["none", "visitor"]}]}
            ]
        }))
        .unwrap();
        let s = s.checked().unwrap();
        assert_eq!(keys(&s, "x", Function::Video), ["signed", "session"]);
        assert_eq!(keys(&s, "x", Function::Text), ["signed", "guest"]);
        assert_eq!(keys(&s, "y", Function::Text), ["guest", "visitor"]);
        let d = s.decide("y", Function::Comments, &ways());
        assert!(d.skip);
        // 兜底不在最后、自定义却没有规则、平台另选「自定义」，都不收
        let mut bad = s.clone();
        bad.rules.pop();
        assert!(bad.checked().is_err());
        assert!(preset(Preset::Custom).checked().is_err());
        let mut bad = Strategy::default();
        bad.platforms.insert(
            "x".into(),
            PlatformStrategy {
                preset: Some(Preset::Custom),
                functions: BTreeMap::new(),
            },
        );
        assert!(bad.checked().is_err());
    }

    #[test]
    fn the_settings_shape_round_trips_and_refuses_unknown_words() {
        let text = r#"{"preset":"account-first","rules":[{"use":[{"accounts":["none"]}]}],"platforms":{"x":{"functions":{"video":"p/a#session","list:like":"skip"}}}}"#;
        let s: Strategy = serde_json::from_str(text).unwrap();
        assert_eq!(serde_json::to_string(&s).unwrap(), text);
        for bad in [
            r#"{"preset":"fastest"}"#,
            r#"{"platforms":{"x":{"functions":{"music":"skip"}}}}"#,
            r#"{"platforms":{"x":{"functions":{"video":"session"}}}}"#,
            r#"{"rules":[{"use":[{"accounts":["none"],"tier":1}]}]}"#,
            r#"{"rules":[{"use":[]}],"extra":1}"#,
        ] {
            assert!(serde_json::from_str::<Strategy>(bad).is_err(), "{bad}");
        }
        assert_eq!(Function::of_output(OutputKind::Subtitle), Function::Video);
        assert!(Function::List.is_list() && !Function::Comments.is_list());
    }
}
