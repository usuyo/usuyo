//! 「手机」那一页（docs/platforms/phone.md §4）：这台电脑上 iPhone 镜像那条路和安卓 adb 那条路各自怎样，
//! 装、换新、卸掉手机的驱动 mirroir（mirroir-mcp，Apache-2.0），试通用剪贴板，演练一次只读屏。
//!
//! 只读这台机器：每个外部程序、本机服务的调用都有时限；不碰任何平台。系统的授权只经驱动问，
//! 不读系统的数据库；改系统、钥匙串、登录项的事（`install-agent`、`launchctl`）不替你做，
//! 写成 [`DriverStep`] 交给页面（[`driver`] 模块开头写了为什么）。

mod adb;
mod driver;
mod flows;
mod listener;
mod probe;
mod serve;
#[cfg(test)]
mod tests;

use std::ffi::OsString;
use std::net::{IpAddr, SocketAddr};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
use std::time::{Duration, Instant};

use usuyo_plugin::protocol::Drives;

use crate::engine::Engine;
use crate::models::Net;
use crate::view::{
    ClipboardState, ClipboardTest, DriverState, Event, GrantState, Mirroring, PhoneDriver,
    PhoneDryRun, PhoneGrants, PhoneIos, PhoneSystem, PhoneView, SettingsPane,
};
use crate::{Error, Result};

pub use driver::{Asset, Release};
pub(crate) use serve::Service;

/// 授权给的是这个名字（`install-agent` 做的那个 App），不是 usuyo。
const SERVE_NAME: &str = "mirroir serve";
/// `check_health` 要真截一张图：这么久里只问一次（页面看得见时每 10 秒重拉一次）。
const HEALTH_FOR: Duration = Duration::from_secs(30);

/// 系统设置里两项授权那一栏的地址（mirroir-mcp `PermissionGuidance.swift` 用的同一对）。
const SCREEN_RECORDING_PANE: &str =
    "x-apple.systempreferences:com.apple.preference.security?Privacy_ScreenCapture";
const ACCESSIBILITY_PANE: &str =
    "x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility";

/// 手机那一页读什么、跑什么。出厂的是 [`PhoneConfig::system`]；测试换成假程序、假服务和本机的假 release。
#[derive(Debug, Clone)]
pub struct PhoneConfig {
    /// 驱动放在哪：按机器目录的 `drivers/mirroir-mcp/`。
    pub driver_dir: PathBuf,
    /// 钉住的那一版（`data/phone-driver.json`）。
    pub release: Release,
    /// 这台机器的架构，照 `std::env::consts::ARCH`。
    pub arch: String,
    pub downloads: Downloads,
    /// 「mirroir serve」的 MCP 地址和它的令牌文件。
    pub serve_url: String,
    pub serve_token: PathBuf,
    /// 发令牌之前认连接对面的进程用的 `lsof`、`launchctl`、`id`（[`listener`] 模块开头）。
    pub lsof: PathBuf,
    pub launchctl: PathBuf,
    pub id: PathBuf,
    /// 这一页跑的小程序（`lsof`、`launchctl`、`id`、`sw_vers`、`pgrep`、剪贴板计数、`adb version`）
    /// 每个最多跑多久，到点杀掉。
    pub program_limit: Duration,
    /// `sw_vers`，读 macOS 的版本。
    pub sw_vers: PathBuf,
    /// 有它才算这台 Mac 有 iPhone 镜像；`iphone-mirroring` 打开的也是它。
    pub mirroring_app: PathBuf,
    /// `pgrep`：驱动不在时看 iPhone 镜像开没开。
    pub pgrep: PathBuf,
    /// `open`：打开系统设置的一栏或一个 App。
    pub open: PathBuf,
    /// 打出 Mac 剪贴板 `changeCount` 的程序和参数；只读这个数，不读剪贴板里的字。
    pub pasteboard: Vec<String>,
    /// 找 `adb` 的目录：先 `PATH`（这里是它的值），再这几个。
    pub adb_path: Option<OsString>,
    pub adb_dirs: Vec<PathBuf>,
    /// adb 自己的服务听的地方；看这一页只连它，不起它。
    pub adb_server: SocketAddr,
    /// 试剪贴板：等多久、多久看一次。
    pub clipboard_wait: Duration,
    pub clipboard_every: Duration,
}

/// 驱动的包从哪下。
#[derive(Debug, Clone, Copy)]
pub enum Downloads {
    /// 照这台机器的网络设置，只走 https、只连公网，和下模型同一套。
    System,
    /// 只连本机回环地址、可以是 http（测试的假 release 服务器）；`stall` 秒没收到字节算断。
    Loopback { stall: Duration },
}

impl PhoneConfig {
    /// 这台 Mac 本身：`home` 是你的家目录（找 `~/.mirroir-mcp/serve-token`），`machine_root` 是按机器目录。
    pub fn system(machine_root: &Path, home: &Path) -> Result<PhoneConfig> {
        Ok(PhoneConfig {
            driver_dir: machine_root.join("drivers").join("mirroir-mcp"),
            release: Release::pinned()?,
            arch: std::env::consts::ARCH.to_owned(),
            downloads: Downloads::System,
            serve_url: "http://127.0.0.1:8765/mcp".into(),
            serve_token: home.join(".mirroir-mcp").join("serve-token"),
            lsof: "/usr/sbin/lsof".into(),
            launchctl: "/bin/launchctl".into(),
            id: "/usr/bin/id".into(),
            program_limit: Duration::from_secs(5),
            sw_vers: "/usr/bin/sw_vers".into(),
            mirroring_app: "/System/Applications/iPhone Mirroring.app".into(),
            pgrep: "/usr/bin/pgrep".into(),
            open: "/usr/bin/open".into(),
            pasteboard: [
                "/usr/bin/osascript",
                "-l",
                "JavaScript",
                "-e",
                "ObjC.import('AppKit'); $.NSPasteboard.generalPasteboard.changeCount",
            ]
            .map(String::from)
            .to_vec(),
            adb_path: std::env::var_os("PATH"),
            adb_dirs: vec![
                "/opt/homebrew/bin".into(),
                "/usr/local/bin".into(),
                home.join("Library/Android/sdk/platform-tools"),
            ],
            adb_server: SocketAddr::from(([127, 0, 0, 1], 5037)),
            clipboard_wait: Duration::from_secs(10),
            clipboard_every: Duration::from_millis(500),
        })
    }
}

/// 手机那一页这边出的错；每个配一句给人看的中文。
#[derive(Debug, thiserror::Error)]
pub enum PhoneError {
    #[error("this daemon does not manage phones")]
    NotServed,
    #[error("no pinned driver asset for {0}")]
    Unsupported(String),
    #[error("the driver is being installed")]
    Busy,
    #[error("mirroir serve does not answer: {0}")]
    ServiceDown(String),
    #[error("mirroir serve: {0}")]
    Service(String),
    #[error("mirroir serve timed out")]
    ServiceTimeout,
    #[error("the {tool} tool failed")]
    Tool { tool: &'static str },
    #[error("{program}: {reason}")]
    Program { program: String, reason: String },
    #[error("pasteboard change count: {0}")]
    Pasteboard(String),
    #[error("cannot open {0}")]
    Open(String),
    #[error("no adb on this computer")]
    NoAdb,
}

impl PhoneError {
    pub(crate) fn code(&self) -> &'static str {
        match self {
            PhoneError::NotServed => "not-found",
            PhoneError::Unsupported(_) => "unsupported-input",
            PhoneError::Busy => "conflict",
            PhoneError::ServiceDown(_) | PhoneError::Tool { .. } | PhoneError::NoAdb => {
                "dependency-missing"
            }
            PhoneError::Service(_) => "unavailable",
            PhoneError::ServiceTimeout => "timeout",
            PhoneError::Program { .. } | PhoneError::Pasteboard(_) | PhoneError::Open(_) => {
                "internal"
            }
        }
    }

    pub(crate) fn hint(&self) -> &'static str {
        match self {
            PhoneError::NotServed => "这台电脑上的 usuyo 内核不管手机。",
            PhoneError::Unsupported(_) => "mirroir 只给 Apple 芯片的 Mac 出了包，这台电脑装不了。",
            PhoneError::Busy => "mirroir 正在装，装完再试。",
            PhoneError::ServiceDown(_) => {
                "「mirroir serve」没在这台 Mac 上跑：先照「驱动」那一行把它登记好。"
            }
            PhoneError::Service(_) => "「mirroir serve」回的话看不懂。",
            PhoneError::ServiceTimeout => "「mirroir serve」没有及时回话。",
            PhoneError::Tool { .. } => "读不了这一屏：iPhone 镜像没连上，或者授权还没给。",
            PhoneError::Program { .. } => "这台 Mac 上的一个系统程序没有回话。",
            PhoneError::Pasteboard(_) => "看不了这台 Mac 的剪贴板变没变。",
            PhoneError::Open(_) => "没能打开系统设置。",
            PhoneError::NoAdb => "这台电脑上没找到 adb。",
        }
    }
}

/// 驱动在装、没装上：只在内存里，重启守护进程就没了。
#[derive(Debug, Clone, PartialEq, Eq)]
enum Busy {
    Idle,
    Installing,
    Failed(&'static str),
}

pub(crate) struct Phone {
    config: PhoneConfig,
    net: Net,
    busy: Mutex<Busy>,
    clipboard: Mutex<Option<ClipboardTest>>,
    /// 上一次 `check_health` 截没截到图，和什么时候问的。
    health: Mutex<Option<(Instant, bool)>>,
    /// 这个守护进程的用户，第一次认连接对面的人时问一次。
    uid: tokio::sync::OnceCell<u32>,
}

fn lock<T>(m: &Mutex<T>) -> MutexGuard<'_, T> {
    m.lock().unwrap_or_else(PoisonError::into_inner)
}

impl Phone {
    /// `net` 是下模型那一份（共用设置里的代理）；`Downloads::Loopback` 时换成只连本机的。
    pub(crate) fn new(config: PhoneConfig, net: &Net) -> Result<Phone> {
        let net = match config.downloads {
            Downloads::System => net.clone(),
            Downloads::Loopback { stall } => Net::loopback(stall)?,
        };
        Ok(Phone {
            config,
            net,
            busy: Mutex::new(Busy::Idle),
            clipboard: Mutex::new(None),
            health: Mutex::new(None),
            uid: tokio::sync::OnceCell::new(),
        })
    }

    fn asset(&self) -> Option<&Asset> {
        self.config
            .release
            .assets
            .iter()
            .find(|a| a.arch == self.config.arch)
    }

    /// 连上「mirroir serve」，认出这条连接的对面是驱动，才读令牌；认不出来的当它没在跑，什么都不发。
    async fn service(&self) -> Result<Service, PhoneError> {
        let url = url::Url::parse(&self.config.serve_url)
            .map_err(|e| PhoneError::Service(format!("the serve address: {e}")))?;
        let ip = match url.host() {
            Some(url::Host::Ipv4(ip)) => IpAddr::V4(ip),
            Some(url::Host::Ipv6(ip)) => IpAddr::V6(ip),
            _ => return Err(PhoneError::Service("the serve address is not an IP".into())),
        };
        if !ip.is_loopback() {
            return Err(PhoneError::Service(
                "the serve address is not loopback".into(),
            ));
        }
        let port = url
            .port_or_known_default()
            .ok_or_else(|| PhoneError::Service("the serve address has no port".into()))?;
        // 没有令牌文件时「mirroir serve」从没登记过：不用去连、去认。这里只看它在不在，不读。
        if !self.config.serve_token.is_file() {
            return Err(PhoneError::ServiceDown("no token file".into()));
        }
        let limit = self.config.program_limit;
        let uid = *self
            .uid
            .get_or_try_init(|| listener::me(&self.config.id, limit))
            .await
            .map_err(PhoneError::ServiceDown)?;
        Service::connect(serve::Dial {
            addr: SocketAddr::new(ip, port),
            path: url.path().to_owned(),
            token_file: self.config.serve_token.clone(),
            check: listener::Check {
                lsof: self.config.lsof.clone(),
                launchctl: self.config.launchctl.clone(),
                uid,
                label: driver::AGENT,
                binary: self.config.driver_dir.join(&self.config.release.binary),
                limit,
            },
        })
        .await
    }

    /// 驱动那一行：装没装、能不能换新、没装上的原因、要你做的那一步。
    fn driver_view(&self, served: Option<&str>) -> PhoneDriver {
        let release = &self.config.release;
        let installed = driver::installed(&self.config.driver_dir, &release.binary);
        let offer = self.asset().map(|a| crate::view::DriverOffer {
            version: release.version.clone(),
            size: a.size,
            source: release.page.clone(),
        });
        let busy = lock(&self.busy).clone();
        let binary = self.config.driver_dir.join(&release.binary);
        let (state, version, hint) = match (&busy, &installed) {
            (Busy::Installing, _) => (DriverState::Installing, None, None),
            // 换新没成：装着的那份还在，原因照样给
            (Busy::Failed(h), Some(v)) => {
                (DriverState::Present, Some(v.clone()), Some((*h).to_owned()))
            }
            (Busy::Idle, Some(v)) => (DriverState::Present, Some(v.clone()), None),
            (Busy::Failed(h), None) => (DriverState::Failed, None, Some((*h).to_owned())),
            (Busy::Idle, None) => (DriverState::Absent, None, None),
        };
        let available = match &version {
            Some(v) if *v == release.version => None,
            _ => offer,
        };
        let step = match (state, &version, served) {
            (DriverState::Present, Some(_), None) => Some(driver::register_step(&binary)),
            (DriverState::Present, Some(v), Some(s)) if s != v => Some(driver::renew_step(&binary)),
            (DriverState::Absent | DriverState::Failed, _, Some(_)) => Some(driver::stop_step()),
            _ => None,
        };
        PhoneDriver {
            state,
            version,
            available,
            hint,
            step,
        }
    }

    /// 授权和镜像：经「mirroir serve」自己的 `status`（和必要时的 `check_health`）问，读不到的是 `unknown`。
    /// 驱动不在时只看 iPhone 镜像这个 App 开没开。回 (授权, 镜像, 服务报的版本)。
    async fn reach(&self, has_mirroring: bool) -> Result<(PhoneGrants, Mirroring, Option<String>)> {
        let unknown = |m| {
            (
                PhoneGrants {
                    to: SERVE_NAME.into(),
                    screen: GrantState::Unknown,
                    accessibility: GrantState::Unknown,
                },
                m,
            )
        };
        if !has_mirroring {
            let (g, m) = unknown(Mirroring::NotSetUp);
            return Ok((g, m, None));
        }
        let answer = match self.service().await {
            Ok(mut service) => match service.status().await {
                Ok(a) => Ok((service, a)),
                Err(e) => Err(e),
            },
            Err(e) => Err(e),
        };
        let (mut service, answer) = match answer {
            Ok(a) => a,
            Err(e) => {
                if !matches!(e, PhoneError::ServiceDown(_)) {
                    tracing::warn!(error = %e, "mirroir serve status");
                }
                let running = probe::running(
                    &self.config.pgrep,
                    "iPhone Mirroring",
                    self.config.program_limit,
                )
                .await?;
                let (g, m) = unknown(if running {
                    Mirroring::Away
                } else {
                    Mirroring::NotSetUp
                });
                return Ok((g, m, None));
            }
        };
        let window = serve::window(&answer.status);
        let captured = match window {
            serve::Window::Paused | serve::Window::Gone => self.captures(&mut service).await,
            _ => None,
        };
        let (screen, accessibility) = serve::grants(window, captured);
        Ok((
            PhoneGrants {
                to: SERVE_NAME.into(),
                screen,
                accessibility,
            },
            serve::mirroring(window),
            answer.version,
        ))
    }

    /// `check_health` 截没截到图；问不到是 `None`。`HEALTH_FOR` 里只问一次。
    async fn captures(&self, service: &mut Service) -> Option<bool> {
        if let Some((at, ok)) = *lock(&self.health)
            && at.elapsed() < HEALTH_FOR
        {
            return Some(ok);
        }
        match service.health().await {
            Ok(ok) => {
                *lock(&self.health) = Some((Instant::now(), ok));
                Some(ok)
            }
            Err(e) => {
                tracing::warn!(error = %e, "mirroir serve check_health");
                None
            }
        }
    }
}

/// 装、换新、卸掉。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DriverAction {
    Install,
    Update,
    Remove,
}

impl Engine {
    fn phone(&self) -> Result<&Arc<Phone>> {
        self.inner
            .phone
            .as_ref()
            .ok_or(Error::Phone(PhoneError::NotServed))
    }

    /// `GET /phone`：这台电脑上的样子，现读。
    pub async fn phone_view(&self) -> Result<PhoneView> {
        let p = self.phone()?;
        let has_mirroring = p.config.mirroring_app.is_dir();
        let (macos, reached, android) = tokio::join!(
            probe::macos(&p.config.sw_vers, p.config.program_limit),
            p.reach(has_mirroring),
            adb::android(
                p.config.adb_path.as_deref(),
                &p.config.adb_dirs,
                p.config.adb_server,
                p.config.program_limit,
            ),
        );
        let (grants, mirroring, served) = reached?;
        Ok(PhoneView {
            ios: PhoneIos {
                system: PhoneSystem {
                    macos: macos?,
                    mirroring: has_mirroring,
                },
                driver: p.driver_view(served.as_deref()),
                grants,
                mirroring,
                clipboard: lock(&p.clipboard).clone(),
            },
            android: android?,
        })
    }

    /// 装、换新、卸掉驱动。装和换新马上回（`installing`），在后台下；成没成都发 `phone-updated`。
    pub async fn phone_driver(&self, action: DriverAction) -> Result<PhoneView> {
        let p = Arc::clone(self.phone()?);
        match action {
            DriverAction::Remove => {
                if *lock(&p.busy) == Busy::Installing {
                    return Err(Error::Phone(PhoneError::Busy));
                }
                let dir = p.config.driver_dir.clone();
                tokio::task::spawn_blocking(move || driver::remove(&dir))
                    .await
                    .map_err(|e| Error::Task(e.to_string()))??;
                *lock(&p.busy) = Busy::Idle;
                self.emit(Event::PhoneUpdated);
            }
            DriverAction::Install | DriverAction::Update => {
                let asset = p
                    .asset()
                    .cloned()
                    .ok_or_else(|| PhoneError::Unsupported(p.config.arch.clone()))?;
                let release = p.config.release.clone();
                if driver::installed(&p.config.driver_dir, &release.binary).as_deref()
                    != Some(&release.version)
                {
                    let already = {
                        let mut busy = lock(&p.busy);
                        let already = *busy == Busy::Installing;
                        *busy = Busy::Installing;
                        already
                    };
                    if already {
                        return self.phone_view().await;
                    }
                    self.emit(Event::PhoneUpdated);
                    // 先读好这一页（还在装），再开始下：回的一定是 `installing`。
                    let view = self.phone_view().await;
                    let this = self.clone();
                    let phone = Arc::clone(&p);
                    tokio::spawn(async move {
                        let result =
                            driver::install(&phone.net, &phone.config.driver_dir, &release, &asset)
                                .await;
                        *lock(&phone.busy) = match &result {
                            Ok(()) => {
                                tracing::info!(version = %release.version, "phone driver installed");
                                Busy::Idle
                            }
                            Err(e) => {
                                tracing::warn!(error = %e, "phone driver not installed");
                                Busy::Failed(driver::hint(e))
                            }
                        };
                        this.emit(Event::PhoneUpdated);
                    });
                    return view;
                }
            }
        }
        self.phone_view().await
    }

    /// `POST /phone/android/devices`：你点了「找一下连着的手机」才跑 `adb devices -l`。adb 的服务没在跑时
    /// 它会起来、留在后台；回的页面里 `android.step` 说怎么停掉它。没找到 adb 回 424。
    pub async fn phone_android_devices(&self) -> Result<PhoneView> {
        let p = self.phone()?;
        let android = adb::look(
            p.config.adb_path.as_deref(),
            &p.config.adb_dirs,
            p.config.program_limit,
        )
        .await?;
        let mut view = self.phone_view().await?;
        view.android = android;
        Ok(view)
    }

    /// `POST /phone/open-settings`：打开系统设置的那一栏，或 iPhone 镜像这个 App。不改任何设置。
    pub async fn open_phone_settings(&self, pane: SettingsPane) -> Result<()> {
        let p = self.phone()?;
        let target = match pane {
            SettingsPane::ScreenRecording => SCREEN_RECORDING_PANE.to_owned(),
            SettingsPane::Accessibility => ACCESSIBILITY_PANE.to_owned(),
            SettingsPane::IphoneMirroring => p.config.mirroring_app.display().to_string(),
        };
        Ok(probe::open(&p.config.open, &target).await?)
    }

    /// 试一下通用剪贴板：记下 `changeCount`，在后台每隔一会儿看一次，变了是「到了」，等满了是「没变」。
    /// 马上回（`waiting`）；有结果发 `phone-updated`。只看这个数，剪贴板里的字不读、不存。
    pub async fn phone_clipboard_test(&self) -> Result<PhoneView> {
        let p = Arc::clone(self.phone()?);
        if lock(&p.clipboard)
            .as_ref()
            .is_some_and(|t| t.state == ClipboardState::Waiting)
        {
            return self.phone_view().await;
        }
        let start = probe::change_count(&p.config.pasteboard, p.config.program_limit).await?;
        let test = ClipboardTest {
            state: ClipboardState::Waiting,
            at: jiff::Timestamp::now().to_string(),
            wait_sec: p.config.clipboard_wait.as_secs(),
        };
        *lock(&p.clipboard) = Some(test.clone());
        self.emit(Event::PhoneUpdated);
        // 先读好这一页（在等），再开始看：回的一定是 `waiting`。
        let view = self.phone_view().await;
        let this = self.clone();
        tokio::spawn(async move {
            let deadline = tokio::time::Instant::now() + p.config.clipboard_wait;
            let outcome = loop {
                tokio::time::sleep(p.config.clipboard_every).await;
                match probe::change_count(&p.config.pasteboard, p.config.program_limit).await {
                    Ok(n) if n != start => break Some(ClipboardState::Arrived),
                    Ok(_) if tokio::time::Instant::now() >= deadline => {
                        break Some(ClipboardState::Unchanged);
                    }
                    Ok(_) => {}
                    Err(e) => {
                        tracing::error!(error = %e, "clipboard test stopped");
                        break None;
                    }
                }
            };
            *lock(&p.clipboard) = outcome.map(|state| ClipboardTest { state, ..test });
            this.emit(Event::PhoneUpdated);
        });
        view
    }

    /// 演练一次：经「mirroir serve」读一次这一屏的字（不要截图），对着装着的插件带的流程找最像的那一步。
    /// 什么都不点；读到的字只在这里比，不存、不进日志。
    pub async fn phone_dry_run(&self) -> Result<PhoneDryRun> {
        let p = self.phone()?;
        let texts = p.service().await?.screen_texts().await?;
        let dirs: Vec<PathBuf> = self
            .inner
            .host
            .plugins()
            .iter()
            .filter(|plugin| {
                plugin
                    .spec
                    .methods
                    .iter()
                    .any(|m| m.drives == Some(Drives::Iphone))
            })
            .map(|plugin| plugin.dir.join("flows"))
            .collect();
        let flows = tokio::task::spawn_blocking(move || flows::load(&dirs))
            .await
            .map_err(|e| Error::Task(e.to_string()))?;
        Ok(PhoneDryRun {
            read: texts.len(),
            matched: flows::best(&flows, &texts),
        })
    }
}
