//! 获取插件的登记：装了哪些、各自的标签、停着没有；登录站点和它们的登录状态。
//! 「现在再试」「先别试」、存下和忘掉登录、填密钥设置也在这里。

use usuyo_plugin::protocol::StorageState;
use usuyo_plugin::{DeclaredSite, FetchPlugin};

use crate::engine::Engine;
use crate::fetch::now_ms;
use crate::view::{
    Event, NotLoaded, PacingView, PauseView, PluginView, PluginsView, SettingView, SiteView,
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

impl Engine {
    /// 装着的获取插件和没加载的。只读本机文件，不碰钥匙串。
    pub fn plugins(&self) -> Result<PluginsView> {
        let host = &self.inner.host;
        let now = now_ms();
        let mut plugins = Vec::new();
        for p in host.plugins() {
            plugins.push(self.plugin_view(p, now)?);
        }
        let not_loaded = self
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
    }

    fn plugin_view(&self, p: &FetchPlugin, now: i64) -> Result<PluginView> {
        let filled = self.inner.host.secrets_set(p.id())?;
        let pauses = self
            .inner
            .jobs
            .pauses_of(p.id().as_str(), now)?
            .into_iter()
            .map(|x| PauseView {
                scope: x.scope,
                by_you: x.held,
                until: x.until.and_then(|t| {
                    jiff::Timestamp::from_millisecond(t)
                        .ok()
                        .map(usuyo_library::time::format)
                }),
                code: x.code,
                hint: x.hint,
            })
            .collect();
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
        let mut sites: Vec<String> = p.spec.sites.iter().map(|s| s.site.clone()).collect();
        sites.dedup();
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
            access: p.spec.access,
            login: p.spec.login,
            can_list: p.spec.capabilities.list.is_some(),
            sources: p
                .spec
                .capabilities
                .list
                .as_ref()
                .map(|l| l.sources.clone())
                .unwrap_or_default(),
            sites,
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
            pauses,
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

    /// 填一个插件的密钥设置（空的就是去掉），存进钥匙串。会阻塞。
    pub fn set_plugin_secret(&self, id: &str, key: &str, value: &str) -> Result<PluginView> {
        let p = self.plugin_ref(id)?;
        self.inner.host.set_secret(p, key, value)?;
        self.emit(Event::PluginsUpdated);
        self.plugin_view(p, now_ms())
    }
}
