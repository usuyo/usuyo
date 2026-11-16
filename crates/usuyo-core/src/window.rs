//! 在 usuyo 的站点窗口里读（协议 §2.3.1，[`Method::reads_in_window`]）：桌面壳照这里给的开一个只给这个站点用的窗口，
//! 用这个 `browser-profile` 站点那份网页数据，打开它的 `loginUrl`，在页面里跑插件的脚本。
//! 脚本记下的链接由壳交到这里，照插件的 `match` 再筛一遍，落成普通的导入，等你在导入页确认。
//! 这个站点的 cookie 不经过内核：内核只记登录好的时间和上一回用它读的情况。

use std::collections::BTreeSet;

use serde::Serialize;
use usuyo_plugin::FetchPlugin;
use usuyo_plugin::protocol::{Credential, Method, Site};

use crate::engine::Engine;
use crate::imports::{BATCH_MAX, DeliveredLink};
use crate::view::Event;
use crate::{Error, Result};

/// `GET /plugins/{id}/methods/{method}/window`：壳开窗口要的。只有给人看的字和脚本，没有凭据。
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WindowView {
    pub plugin: String,
    pub plugin_name: String,
    pub method: String,
    pub method_name: String,
    pub site: String,
    pub site_name: String,
    pub login_url: String,
    /// 除了站点本身，窗口还能去哪些域名（和它们的子域）。
    pub domains: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub guide: Option<String>,
    /// 在页面里跑的脚本全文。
    pub script: String,
}

/// 壳交来的一个来源（收藏、赞过……）的链接。
#[derive(Debug, Clone)]
pub struct WindowSource {
    /// 已经在边界上核对过（`favorite`、`like`……）。
    pub action: String,
    pub links: Vec<DeliveredLink>,
    /// 这个来源前面交过时落成的那次导入：这一回并进去。
    pub import: Option<String>,
}

/// 交完以后：落成了哪几次导入，几条不合插件的 `match` 被丢掉。
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WindowHandover {
    pub imports: Vec<HandedImport>,
    pub noted: u32,
    pub dropped: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HandedImport {
    pub id: String,
    pub action: String,
    pub links: u32,
}

fn count(n: usize) -> u32 {
    u32::try_from(n).unwrap_or(u32::MAX)
}

impl Engine {
    /// 一种在站点窗口里读的方法、它的插件和它用的 `browser-profile` 站点。
    fn window_method(
        &self,
        plugin_id: &str,
        method_id: &str,
    ) -> Result<(FetchPlugin, Method, Site)> {
        let plugin = self
            .loaded()
            .host
            .plugin(plugin_id)
            .cloned()
            .ok_or_else(|| Error::NotFound(format!("plugin {plugin_id}")))?;
        let m = plugin
            .spec
            .method(method_id)
            .filter(|m| m.built && m.reads_in_window())
            .cloned()
            .ok_or_else(|| {
                Error::NotFound(format!(
                    "method {method_id} of {plugin_id} does not read in a usuyo window"
                ))
            })?;
        let site = m
            .sites
            .iter()
            .find_map(|name| {
                plugin
                    .spec
                    .sites
                    .iter()
                    .find(|s| &s.site == name && s.credential == Credential::BrowserProfile)
            })
            .cloned()
            .ok_or_else(|| {
                Error::NotFound(format!("method {method_id} has no browser-profile site"))
            })?;
        Ok((plugin, m, site))
    }

    /// `GET /plugins/{id}/methods/{method}/window`。
    pub fn window_spec(&self, plugin_id: &str, method_id: &str) -> Result<WindowView> {
        let (plugin, m, site) = self.window_method(plugin_id, method_id)?;
        let script = self.method_script_text(plugin_id, method_id)?;
        let login_url = site
            .login_url
            .clone()
            .ok_or_else(|| Error::NotFound(format!("{} has no login page", site.site)))?;
        Ok(WindowView {
            plugin: plugin.id().to_string(),
            plugin_name: plugin.header.name.clone(),
            method: m.id.clone(),
            method_name: m.name.clone(),
            site: site.site.clone(),
            site_name: site.name.clone(),
            login_url,
            domains: site.domains.clone(),
            guide: m.guide.clone(),
            script,
        })
    }

    /// `POST /plugins/{id}/methods/{method}/window/signed-in`：窗口里登录好了，记下时间。
    pub fn window_signed_in(&self, plugin_id: &str, method_id: &str) -> Result<()> {
        let (_, _, site) = self.window_method(plugin_id, method_id)?;
        self.loaded()
            .host
            .profile_signed_in(plugin_id, &site.site)?;
        self.emit(Event::PluginsUpdated);
        Ok(())
    }

    /// `POST /plugins/{id}/methods/{method}/window/links`：窗口里记下的链接。只留合插件 `match` 的、去掉重复的，
    /// 每个来源落成一次导入，等你确认。同一个来源分几次交时带上头一回的导入（`import`），并进那一份；
    /// 那份已经确认了、不在了、不是这种方法这个来源的，另起一份。一回超过 [`BATCH_MAX`] 条也并成一份。
    /// 记下这一回用了站点窗口，条数是交来的这几份导入里一共有几条。
    pub fn window_links(
        &self,
        plugin_id: &str,
        method_id: &str,
        sources: Vec<WindowSource>,
    ) -> Result<WindowHandover> {
        let (plugin, m, site) = self.window_method(plugin_id, method_id)?;
        let mut seen = BTreeSet::new();
        let mut imports = Vec::new();
        let (mut noted, mut dropped) = (0usize, 0usize);
        for source in sources {
            let mut kept = Vec::new();
            for link in source.links {
                if !plugin.spec.matches(&link.url) {
                    dropped += 1;
                    continue;
                }
                if seen.insert(link.url.to_string()) {
                    kept.push(link);
                }
            }
            noted += kept.len();
            let n = kept.len();
            // 交来的那份要是这种方法这个来源的、还在
            let mut target = match &source.import {
                Some(id) => self
                    .inner
                    .jobs
                    .import(id)?
                    .filter(|r| {
                        r.kind == "delivered"
                            && r.client.as_deref() == Some(m.name.as_str())
                            && r.action.as_deref() == Some(source.action.as_str())
                    })
                    .map(|r| r.id),
                None => None,
            };
            while !kept.is_empty() {
                let rest = kept.split_off(kept.len().min(BATCH_MAX));
                let merged = match &target {
                    Some(id) => self.append_links(id, Some(&source.action), kept.clone())?,
                    None => false,
                };
                if !merged {
                    let view = self.deliver_batch(
                        &plugin.header.name,
                        Some(&source.action),
                        kept,
                        Some(&m.name),
                    )?;
                    target = Some(view.id);
                }
                kept = rest;
            }
            if let Some(id) = target {
                imports.push(HandedImport {
                    id,
                    action: source.action.clone(),
                    links: count(n),
                });
            }
        }
        let mut total = 0u32;
        for i in &imports {
            let c = self.inner.jobs.entry_counts(&i.id)?;
            total = total.saturating_add(c.new + c.stored + c.trashed + c.unresolved);
        }
        self.loaded()
            .host
            .profile_used(&site.site, plugin.id().as_str(), &m.id, total)?;
        self.emit(Event::PluginsUpdated);
        for i in &imports {
            self.emit(Event::ImportUpdated { id: i.id.clone() });
        }
        Ok(WindowHandover {
            imports,
            noted: count(noted),
            dropped: count(dropped),
        })
    }
}
