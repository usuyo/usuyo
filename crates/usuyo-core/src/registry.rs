//! 获取插件的登记：装了哪些、各自的标签、停着没有；登录站点和它们的登录状态。
//! 访客身份（不是账号）和它们有没有存着。
//! 「现在再试」「先别试」、存下和忘掉登录和访客身份、填密钥设置也在这里。

use usuyo_plugin::protocol::{Account, Method, StorageState};
use usuyo_plugin::{DeclaredSite, DeclaredVisitor, FetchPlugin};

use crate::engine::Engine;
use crate::fetch::now_ms;
use crate::jobs::PauseRow;
use crate::methods::blockers;
use crate::view::{
    AccountView, Blocker, Event, GetsView, LastCall, MethodView, NotLoaded, PacingView, PauseView,
    PlatformTierView, PluginView, PluginsView, SettingView, SiteView, SourceView, VisitorView,
};
use crate::{Error, Result};

fn site_view(d: DeclaredSite) -> SiteView {
    SiteView {
        site: d.site.site,
        name: d.site.name,
        login_url: d.site.login_url,
        cookies: d.site.cookies,
        domains: d.site.domains,
        login: d.login,
        captured_at: d.captured_at,
        plugins: d.plugins.into_iter().map(|p| p.to_string()).collect(),
    }
}

fn visitor_view(d: DeclaredVisitor) -> VisitorView {
    VisitorView {
        site: d.visitor.site,
        name: d.visitor.name,
        url: d.visitor.url,
        cookies: d.visitor.cookies,
        domains: d.visitor.domains,
        state: d.state,
        captured_at: d.captured_at,
        expires_at: d.expires.and_then(|t| {
            jiff::Timestamp::from_second(t as i64)
                .ok()
                .map(usuyo_library::time::format)
        }),
        plugins: d.plugins.into_iter().map(|p| p.to_string()).collect(),
    }
}

fn pause_view(x: PauseRow) -> PauseView {
    PauseView {
        method: x.method,
        scope: x.scope,
        by_you: x.held,
        until: x.until.and_then(|t| {
            jiff::Timestamp::from_millisecond(t)
                .ok()
                .map(usuyo_library::time::format)
        }),
        code: x.code,
        hint: x.hint,
    }
}

/// 停着的一句话：为什么、到什么时候。
fn pause_blocker(x: &PauseRow) -> Blocker {
    let hint = match x.until {
        _ if x.held => "你让这个插件先别试了，到你点「现在再试」为止".to_owned(),
        Some(until) => {
            let why = match x.code.as_deref() {
                Some("rate-limited") => "平台限流了",
                Some("blocked") => "平台拦下了请求",
                _ => "上次出了错",
            };
            format!("{why}，{}以前不用它", crate::pace::local_time(until))
        }
        None => "停着，等你点「现在再试」".to_owned(),
    };
    Blocker {
        code: "paused".into(),
        hint,
    }
}

/// 账号在哪里，给人看的名字：登录站点、访客身份照它们在清单里的名字；账号在你自己的浏览器、手机、
/// 电脑客户端里的，有 `credential: app` 的站点就照它，没有就是插件的名字（一个平台一个插件）。
/// 不用账号的和开发者凭据没有。
fn where_of(p: &FetchPlugin, m: &Method) -> Option<String> {
    let site_name = || {
        m.sites
            .iter()
            .filter_map(|s| p.spec.sites.iter().find(|x| &x.site == s))
            .map(|s| s.name.clone())
            .next()
    };
    match m.account {
        Account::None | Account::Developer => None,
        Account::Visitor => m
            .visitors
            .iter()
            .filter_map(|s| p.spec.visitors.iter().find(|x| &x.site == s))
            .map(|v| v.name.clone())
            .next(),
        Account::InUsuyo | Account::OwnBrowser | Account::OwnPhone | Account::OwnApp => {
            Some(site_name().unwrap_or_else(|| p.header.name.clone()))
        }
    }
}

impl Engine {
    /// 装着的获取插件和没加载的。各插件的本机检查过期了的先问一遍（同时问，不带凭据、不碰平台）。
    pub async fn plugins(&self) -> Result<PluginsView> {
        let mut asking = tokio::task::JoinSet::new();
        for p in self.inner.host.plugins() {
            let (this, p) = (self.clone(), p.clone());
            asking.spawn(async move {
                this.fetch_health(&p).await;
            });
        }
        while asking.join_next().await.is_some() {}
        let this = self.clone();
        tokio::task::spawn_blocking(move || {
            let now = now_ms();
            let mut plugins = Vec::new();
            for p in this.inner.host.plugins() {
                plugins.push(this.plugin_view(p, now)?);
            }
            let not_loaded = this
                .inner
                .rejected
                .iter()
                .map(|(dir, reason)| NotLoaded {
                    dir: dir.display().to_string(),
                    reason: reason.clone(),
                })
                .collect();
            Ok(PluginsView {
                plugins,
                not_loaded,
            })
        })
        .await
        .map_err(|e| Error::Task(e.to_string()))?
    }

    /// 一个插件的样子。本机检查用缓存里的（没问过的不算挡）；只读本机文件，不碰钥匙串。
    pub(crate) fn plugin_view(&self, p: &FetchPlugin, now: i64) -> Result<PluginView> {
        let host = &self.inner.host;
        let filled = host.secrets_set(p.id())?;
        let pauses = self.inner.jobs.pauses_of(p.id().as_str(), now)?;
        let calls = self.inner.jobs.method_calls(p.id().as_str())?;
        let sites = host.sites()?;
        let visitors = host.visitors()?;
        let health = self.inner.methods.health_seen(p.id().as_str());
        let pacing = p
            .spec
            .platforms
            .iter()
            .filter_map(|pl| {
                self.inner.pacing.get(pl.as_str()).map(|x| PacingView {
                    platform: pl.as_str().to_owned(),
                    batch: x.batch,
                    interval_sec: x.interval.as_secs(),
                    daily: x.daily,
                })
            })
            .collect();
        let methods = p
            .spec
            .methods
            .iter()
            .map(|m| {
                let enabled = self.inner.methods.enabled(p, m);
                let own: Vec<&PauseRow> = pauses
                    .iter()
                    .filter(|x| x.method == m.id || x.method == "*")
                    .collect();
                let mut blockers = blockers(m, enabled, health.as_ref(), &sites, &visitors);
                // 停着也算用不上；通用网页插件按网站停，一个站停着不挡这种方法，只算整个平台上的
                if m.built
                    && let Some(x) = own.iter().find(|x| {
                        x.scope == "*" || p.spec.platforms.iter().any(|pl| pl.as_str() == x.scope)
                    })
                {
                    blockers.push(pause_blocker(x));
                }
                let where_ = where_of(p, m);
                let account_sites = match m.account {
                    Account::Visitor => m.visitors.clone(),
                    _ => m.sites.clone(),
                };
                let per_platform = m
                    .per_platform
                    .iter()
                    .filter(|(_, u)| u.access != m.access || u.login != m.login)
                    .map(|(pl, u)| PlatformTierView {
                        platform: pl.as_str().to_owned(),
                        access: u.access,
                        login: u.login,
                        enabled: self.inner.methods.enabled_on(p, m, pl.as_str()),
                    })
                    .collect();
                MethodView {
                    id: m.id.clone(),
                    name: m.name.clone(),
                    access: m.access,
                    login: m.login,
                    account: AccountView {
                        kind: m.account,
                        sites: account_sites,
                        where_,
                    },
                    interaction: m.interaction,
                    drives: m.drives,
                    per_platform,
                    gets: GetsView {
                        fetch: m
                            .capabilities
                            .fetch
                            .as_ref()
                            .map(|f| f.outputs.iter().copied().collect()),
                        list: m
                            .capabilities
                            .list
                            .as_ref()
                            .map(|l| l.sources.iter().map(|s| s.title.clone()).collect()),
                        link: m.capabilities.list.as_ref().and_then(|l| l.link.clone()),
                        delivers: m.delivers,
                    },
                    built: m.built,
                    enabled,
                    usable: blockers.is_empty(),
                    blockers,
                    last_call: calls.iter().find(|c| c.method == m.id).map(|c| LastCall {
                        at: jiff::Timestamp::from_millisecond(c.at)
                            .map(usuyo_library::time::format)
                            .unwrap_or_default(),
                        ok: c.code.is_none(),
                        code: c.code.clone(),
                        hint: c.hint.clone(),
                    }),
                    pauses: own.into_iter().cloned().map(pause_view).collect(),
                    risk_note: m.risk.clone(),
                }
            })
            .collect();
        let sources = p
            .spec
            .built()
            .flat_map(|m| {
                m.capabilities
                    .list
                    .iter()
                    .flat_map(|l| l.sources.iter())
                    .map(|s| SourceView {
                        source: s.clone(),
                        method: m.id.clone(),
                    })
            })
            .collect();
        let mut used: Vec<String> = p
            .spec
            .built()
            .flat_map(|m| m.sites.iter().cloned())
            .collect();
        used.sort();
        used.dedup();
        Ok(PluginView {
            id: p.id().to_string(),
            name: p.header.name.clone(),
            description: p.header.description.clone(),
            version: p.header.version.to_string(),
            platforms: p
                .spec
                .platforms
                .iter()
                .map(|pl| self.platform(pl.as_str()))
                .collect(),
            access: p.spec.access(),
            login: p.spec.login(),
            can_list: p.spec.built().any(|m| m.capabilities.list.is_some()),
            sources,
            sites: used,
            visitors: p.spec.visitors.iter().map(|v| v.site.clone()).collect(),
            settings: p
                .header
                .settings
                .iter()
                .map(|s| SettingView {
                    key: s.key.clone(),
                    label: s.label.clone(),
                    value_type: s.value_type,
                    secret: s.secret,
                    filled: s.secret.then(|| filled.contains(&s.key)),
                })
                .collect(),
            pacing,
            pauses: pauses.into_iter().map(pause_view).collect(),
            methods,
        })
    }

    fn plugin_ref(&self, id: &str) -> Result<&FetchPlugin> {
        self.inner
            .host
            .plugin(id)
            .ok_or_else(|| Error::NotFound(format!("plugin {id}")))
    }

    /// 「现在再试」：这个插件的暂停全去掉，排着的任务和导入马上再看。
    pub fn try_plugin_now(&self, id: &str) -> Result<PluginView> {
        let p = self.plugin_ref(id)?;
        self.inner.jobs.clear_pauses(id)?;
        self.inner.methods.forget_health(id);
        // 因为它停着而往后排的任务（它能取的链接）马上再排
        let now = now_ms();
        for job in self.inner.jobs.queued()? {
            if url::Url::parse(&job.url).is_ok_and(|u| p.spec.matches(&u)) {
                self.inner.jobs.due_now(&job.item_id, now)?;
            }
        }
        self.inner.wake.notify_one();
        self.inner.imports_wake.notify_one();
        self.emit(Event::PluginsUpdated);
        self.plugin_view(p, now_ms())
    }

    /// 「先别试」：到你点「现在再试」为止，哪儿都不调它；别的插件照常。
    pub fn hold_plugin(&self, id: &str) -> Result<PluginView> {
        let p = self.plugin_ref(id)?;
        self.inner.jobs.hold(id)?;
        self.emit(Event::PluginsUpdated);
        self.plugin_view(p, now_ms())
    }

    /// 登录站点和登录状态。只读本机的索引，不碰钥匙串。
    pub fn sites(&self) -> Result<Vec<SiteView>> {
        Ok(self
            .inner
            .host
            .sites()?
            .into_iter()
            .map(site_view)
            .collect())
    }

    /// 存下登录窗口读出的 cookie：只留属于这个站点的，存进钥匙串。会阻塞（钥匙串可能弹窗）。
    pub fn save_site(&self, site: &str, state: &StorageState) -> Result<SiteView> {
        let d = self.inner.host.save_site(site, state)?;
        self.inner.wake.notify_one();
        self.inner.imports_wake.notify_one();
        self.emit(Event::PluginsUpdated);
        Ok(site_view(d))
    }

    /// 忘掉一个站点的登录。会阻塞。
    pub fn forget_site(&self, site: &str) -> Result<()> {
        self.inner.host.forget_site(site)?;
        self.emit(Event::PluginsUpdated);
        Ok(())
    }

    /// 访客身份和有没有存着。只读本机的索引，不碰钥匙串。
    pub fn visitors(&self) -> Result<Vec<VisitorView>> {
        Ok(self
            .inner
            .host
            .visitors()?
            .into_iter()
            .map(visitor_view)
            .collect())
    }

    /// 存下访客窗口读出的 cookie：只留属于这个访客身份的，存进钥匙串。会阻塞。
    pub fn save_visitor(&self, site: &str, state: &StorageState) -> Result<VisitorView> {
        let d = self.inner.host.save_visitor(site, state)?;
        self.inner.wake.notify_one();
        self.inner.imports_wake.notify_one();
        self.emit(Event::PluginsUpdated);
        Ok(visitor_view(d))
    }

    /// 去掉一个访客身份。同一主机的账号不动。会阻塞。
    pub fn forget_visitor(&self, site: &str) -> Result<()> {
        self.inner.host.forget_visitor(site)?;
        self.emit(Event::PluginsUpdated);
        Ok(())
    }

    /// 填一个插件的密钥设置（空的就是去掉），存进钥匙串。会阻塞。
    pub fn set_plugin_secret(&self, id: &str, key: &str, value: &str) -> Result<PluginView> {
        let p = self.plugin_ref(id)?;
        self.inner.host.set_secret(p, key, value)?;
        self.emit(Event::PluginsUpdated);
        self.plugin_view(p, now_ms())
    }
}
