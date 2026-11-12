//! 插件清单 `usuyo-plugin.yaml`（协议 §2）。
//!
//! 读法：YAML → JSON 值 → 线上形状 `WireManifest`（字段错在哪一层由 serde 报出路径）→
//! 一遍语义检查（§10 `usuyo plugin check` 的那张单子）→ 按种类分好的 `PluginManifest`。

use std::collections::{BTreeMap, BTreeSet};

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::Parsed;
use super::fetch::{OutputKind, SourceInfo};
use super::values::{ByteSize, DurationSpec, PlatformId, PluginId, check_http_url};
use crate::PluginError;

/// 这个版本认的 `protocol`。
pub const SUPPORTED_PROTOCOLS: &[u32] = &[1];

/// 清单文件名（协议 §2.1）。
pub const MANIFEST_FILE: &str = "usuyo-plugin.yaml";

/// 七层取法，从低到高（协议 §2.4）。只是给用户挑选用的标签，宿主不据此拦截。
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "kebab-case")]
pub enum Access {
    OfficialExport,
    PublicWeb,
    VisionUi,
    BrowserSession,
    SessionApi,
    PrivateApi,
    ClientHook,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "kebab-case")]
pub enum Login {
    None,
    Optional,
    Required,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum Credential {
    Cookies,
    BrowserProfile,
    App,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum Interaction {
    #[default]
    Background,
    Foreground,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum ProviderNeed {
    #[default]
    None,
    Optional,
    Required,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "kebab-case")]
pub enum FetchInputKind {
    Url,
    Ref,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum ListOrder {
    NewestFirst,
    Unordered,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum NetworkMode {
    #[default]
    None,
    Allowlist,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum Device {
    Adb,
    IosDevice,
    ScreenCapture,
    Accessibility,
    AudioCapture,
    Clipboard,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum SettingType {
    String,
    Bool,
    Int,
    Enum,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum ExportWrites {
    Files,
    Api,
}

/// 转换插件吃的东西（协议 §2.9）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub enum ConsumeKind {
    #[serde(rename = "media:video")]
    Video,
    #[serde(rename = "media:audio")]
    Audio,
    /// 16 kHz 单声道 WAV，抽音轨插件的中间产物（`produces: audio:pcm16k`），不进资料库。
    #[serde(rename = "audio:pcm16k")]
    AudioPcm16k,
    #[serde(rename = "image")]
    Image,
    #[serde(rename = "html")]
    Html,
    #[serde(rename = "text")]
    Text,
    #[serde(rename = "subtitle")]
    Subtitle,
    #[serde(rename = "derived:asr")]
    Asr,
    #[serde(rename = "derived:ocr")]
    Ocr,
    #[serde(rename = "derived:vision")]
    Vision,
    #[serde(rename = "derived:keyframes")]
    Keyframes,
}

/// 转换插件产出的东西（协议 §2.9）：产物种类，不是路径。路径由宿主定在 `derived/runs/<runId>/` 下。
/// 同一个词表也是 `run` 的 `want` 和结果里每个输出的 `kind`（协议 §4.2）。
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, JsonSchema,
)]
pub enum Produce {
    /// 分开的画面和音轨合起来能播的文件。
    #[serde(rename = "play")]
    Play,
    /// 给转写用的 16 kHz 单声道 WAV；中间产物，不进资料库。
    #[serde(rename = "audio:pcm16k")]
    AudioPcm16k,
    /// 只回流的信息（结果里的 `probe`），不出文件。
    #[serde(rename = "probe")]
    Probe,
    #[serde(rename = "asr")]
    Asr,
    #[serde(rename = "keyframes")]
    Keyframes,
    #[serde(rename = "ocr")]
    Ocr,
    #[serde(rename = "readable")]
    Readable,
    #[serde(rename = "vision")]
    Vision,
    #[serde(rename = "summary")]
    Summary,
    #[serde(rename = "embeddings")]
    Embeddings,
}

impl Produce {
    pub fn as_str(self) -> &'static str {
        match self {
            Produce::Play => "play",
            Produce::AudioPcm16k => "audio:pcm16k",
            Produce::Probe => "probe",
            Produce::Asr => "asr",
            Produce::Keyframes => "keyframes",
            Produce::Ocr => "ocr",
            Produce::Readable => "readable",
            Produce::Vision => "vision",
            Produce::Summary => "summary",
            Produce::Embeddings => "embeddings",
        }
    }

    /// 出不出文件：`probe` 只回信息。
    pub fn writes_files(self) -> bool {
        self != Produce::Probe
    }
}

impl std::fmt::Display for Produce {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct Upstream {
    pub name: String,
    pub url: String,
    pub license: String,
    #[serde(default)]
    pub paid: bool,
}

/// `requires.runtime` 的一项，「名字加版本约束」，例如 `chrome>=136`。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeRequirement {
    pub name: String,
    /// 原样的约束，例如 `>=136`；怎么比由运行时表定（§2.13，还没做）。
    pub constraint: Option<String>,
}

impl RuntimeRequirement {
    fn parse(text: &str) -> Result<Self, String> {
        let end = text
            .find(|c: char| !(c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.')))
            .unwrap_or(text.len());
        let (name, rest) = text.split_at(end);
        let constraint_ok = rest.is_empty()
            || [">=", "<=", "==", "=", ">", "<", "^", "~"]
                .iter()
                .any(|op| {
                    rest.strip_prefix(op)
                        .is_some_and(|v| v.starts_with(|c: char| c.is_ascii_alphanumeric()))
                });
        if name.is_empty() || !constraint_ok {
            return Err(format!("`{text}` is not a runtime like `yt-dlp>=2026.10`"));
        }
        Ok(Self {
            name: name.to_owned(),
            constraint: (!rest.is_empty()).then(|| rest.to_owned()),
        })
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
struct WireRequires {
    #[serde(default)]
    runtime: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct Inject {
    pub host: String,
    pub header: String,
    pub format: String,
}

/// 插件设置（协议 §2.11）。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct Setting {
    pub key: String,
    pub label: String,
    #[serde(rename = "type")]
    pub value_type: SettingType,
    #[serde(default)]
    pub options: Vec<String>,
    #[serde(default)]
    pub default: Option<serde_json::Value>,
    #[serde(default)]
    pub secret: bool,
    #[serde(default)]
    pub inject: Option<Inject>,
}

/// 会用到哪个站点的账号（协议 §2.5）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Site {
    pub site: String,
    pub name: String,
    pub credential: Credential,
    #[serde(default)]
    pub login_url: Option<String>,
    #[serde(default)]
    pub cookies: Vec<String>,
    #[serde(default)]
    pub domains: Vec<String>,
}

/// 访客身份（协议 §2.5.1）：站点给任何没登录的访客的 cookie，不是账号。
/// 和同一主机的账号分开存、分开交。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct Visitor {
    pub site: String,
    pub name: String,
    /// 打开哪一页来取，只能是 https。
    pub url: String,
    /// 这些名字都有了才算取到；空的时候等名字不再变。
    #[serde(default)]
    pub cookies: Vec<String>,
    #[serde(default)]
    pub domains: Vec<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct FsPermissions {
    #[serde(default)]
    pub read: Vec<String>,
    #[serde(default)]
    pub write: Vec<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct NetworkPermissions {
    #[serde(default)]
    pub mode: NetworkMode,
    #[serde(default)]
    pub hosts: Vec<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct Limits {
    #[serde(default)]
    pub memory: Option<ByteSize>,
    /// 转换、导出插件单次 `run` 的总时长；获取插件用能力里的 `timeout`。
    #[serde(default)]
    pub timeout: Option<DurationSpec>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
struct WirePermissions {
    #[serde(default)]
    fs: FsPermissions,
    #[serde(default)]
    network: NetworkPermissions,
    #[serde(default)]
    exec: Option<Vec<String>>,
    #[serde(default)]
    env: Vec<String>,
    #[serde(default)]
    gpu: bool,
    #[serde(default)]
    devices: Vec<Device>,
    #[serde(default)]
    limits: Limits,
}

/// 权限声明（协议 §2.7）。落实归沙箱（§7，还没做）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Permissions {
    pub fs: FsPermissions,
    pub network: NetworkPermissions,
    /// 允许再起的程序名；没写时等于 `requires.runtime` 里的名字。
    pub exec: Vec<String>,
    pub env: Vec<String>,
    pub gpu: bool,
    pub devices: Vec<Device>,
    pub limits: Limits,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct Asset {
    pub name: String,
    pub sha256: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct Install {
    pub assets: BTreeMap<String, Asset>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct Run {
    pub command: Vec<String>,
    /// 支不支持常驻模式（§3.12，宿主还没用它）。
    #[serde(default)]
    pub serve: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct Pacing {
    /// `false` 表示只接受用户一条条收下。
    #[serde(default)]
    pub batch: Option<bool>,
    #[serde(default)]
    pub interval: Option<DurationSpec>,
    #[serde(default)]
    pub daily: Option<u32>,
}

/// 要宿主下载的模型（协议 §2.9）。宿主下，插件不联网；放进 `${MODELS}/<name>/`。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct ModelSpec {
    /// 目录名：小写字母、数字、`.`、`_`、`-`，不以 `.` 开头，一个清单里不重名。
    pub name: String,
    pub url: String,
    /// 同一个 sha256 的别的地址，`url` 不行时按顺序试。
    #[serde(default)]
    pub mirrors: Vec<String>,
    /// 整个下载文件的 sha256。
    pub sha256: String,
    /// 整个下载文件的字节数，下载前给你看。
    pub size: u64,
    pub format: ModelFormat,
    /// 模型权重的许可，下载前给你看。
    pub license: String,
    #[serde(default)]
    pub setting: Option<String>,
    #[serde(default)]
    pub value: Option<String>,
}

/// 下载下来的是什么：单个文件，或宿主解到 `${MODELS}/<name>/` 的压缩包。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub enum ModelFormat {
    #[serde(rename = "file")]
    File,
    #[serde(rename = "tar.bz2")]
    TarBz2,
    #[serde(rename = "tar.gz")]
    TarGz,
    #[serde(rename = "zip")]
    Zip,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(untagged)]
enum WireConsume {
    Kind(ConsumeKind),
    WithMime {
        kind: ConsumeKind,
        mime: Vec<String>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Consume {
    pub kind: ConsumeKind,
    /// 只吃这些格式；空表示都行。
    pub mime: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
struct WirePlatformUse {
    #[serde(default)]
    access: Option<Access>,
    #[serde(default)]
    login: Option<Login>,
    #[serde(default)]
    sites: Option<Vec<String>>,
}

/// 能力声明的线上形状：三种能力共用一个结构，哪些字段属于哪种能力在语义检查里判。
/// `access`、`login`、`sites`、`visitors`、`interaction`、`perPlatform` 挪到了方法上（§2.3.1）：
/// 还写在能力里的清单照旧形状拒掉，说清楚挪到了哪里。
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
struct WireCapability {
    #[serde(default)]
    timeout: Option<DurationSpec>,
    #[serde(default)]
    provider: ProviderNeed,
    // fetch
    #[serde(default)]
    outputs: Option<Vec<OutputKind>>,
    #[serde(default)]
    input: Option<Vec<FetchInputKind>>,
    // list
    #[serde(default)]
    order: Option<ListOrder>,
    #[serde(default)]
    sources: Option<Vec<SourceInfo>>,
    #[serde(default)]
    files: Option<Vec<String>>,
    #[serde(default)]
    link: Option<String>,
    // 旧形状
    #[serde(default)]
    #[schemars(skip)]
    access: Option<serde_json::Value>,
    #[serde(default)]
    #[schemars(skip)]
    login: Option<serde_json::Value>,
    #[serde(default)]
    #[schemars(skip)]
    sites: Option<serde_json::Value>,
    #[serde(default)]
    #[schemars(skip)]
    visitors: Option<serde_json::Value>,
    #[serde(default)]
    #[schemars(skip)]
    interaction: Option<serde_json::Value>,
    #[serde(default)]
    #[schemars(skip)]
    per_platform: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
struct WireCapabilities {
    #[serde(default)]
    fetch: Option<WireCapability>,
    #[serde(default)]
    list: Option<WireCapability>,
    #[serde(default)]
    unfavorite: Option<WireCapability>,
}

/// 账号在哪里被用到（§2.3.1）。顺序就是同一层里挑方法的先后：先不带账号的。
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "kebab-case")]
pub enum Account {
    /// 不用账号。
    None,
    /// 访客身份（§2.5.1），不是账号。
    Visitor,
    /// 只在你自己的浏览器里。
    OwnBrowser,
    /// 只在你手机的 App 里。
    OwnPhone,
    /// 只在你电脑的客户端里。
    OwnApp,
    /// 登录进 usuyo。
    InUsuyo,
    /// 你申请的开发者凭据。
    Developer,
}

/// 不在插件进程里跑的方法交什么给本机接口（§5）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum Delivers {
    Links,
}

/// 记录到的一句情况的出处。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct RiskSource {
    pub label: String,
    pub url: String,
}

/// 记录到的一句情况。只是信息，宿主不拿它拦。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct Risk {
    pub text: String,
    pub source: RiskSource,
}

/// 一种方法的线上形状（§2.3.1）。
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
struct WireMethod {
    id: String,
    name: String,
    #[serde(default = "yes")]
    built: bool,
    access: Access,
    login: Login,
    account: Account,
    #[serde(default)]
    sites: Vec<String>,
    #[serde(default)]
    visitors: Vec<String>,
    #[serde(default)]
    per_platform: BTreeMap<PlatformId, WirePlatformUse>,
    #[serde(default)]
    interaction: Interaction,
    #[serde(default)]
    delivers: Option<Delivers>,
    #[serde(default)]
    capabilities: Option<WireCapabilities>,
    #[serde(default)]
    risk: Option<Risk>,
}

fn yes() -> bool {
    true
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum PluginKindTag {
    Fetch,
    Transform,
    Export,
}

/// 清单的线上形状，也是 `manifest.schema.json` 的来源。种类专属的字段都可选，语义检查再判。
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(rename = "PluginManifest")]
pub(crate) struct WireManifest {
    protocol: u32,
    id: PluginId,
    kind: PluginKindTag,
    name: String,
    #[serde(default)]
    description: Option<String>,
    #[schemars(with = "String")]
    version: semver::Version,
    license: String,
    #[serde(default)]
    homepage: Option<String>,
    maintainers: Vec<String>,
    #[serde(default)]
    upstream: Vec<Upstream>,
    #[serde(default)]
    requires: WireRequires,
    #[serde(default)]
    settings: Vec<Setting>,
    permissions: WirePermissions,
    #[serde(default)]
    install: Option<Install>,
    run: Run,
    // fetch
    #[serde(default)]
    platforms: Option<Vec<PlatformId>>,
    #[serde(default, rename = "match")]
    match_patterns: Option<Vec<String>>,
    #[serde(default)]
    sites: Option<Vec<Site>>,
    #[serde(default)]
    visitors: Option<Vec<Visitor>>,
    #[serde(default)]
    methods: Option<Vec<WireMethod>>,
    #[serde(default)]
    pacing: Option<BTreeMap<PlatformId, Pacing>>,
    // 旧形状：顶层的 `access`、`login`、`capabilities` 由方法代替（§2.3.1），写了就拒。
    #[serde(default)]
    #[schemars(skip)]
    access: Option<serde_json::Value>,
    #[serde(default)]
    #[schemars(skip)]
    login: Option<serde_json::Value>,
    #[serde(default)]
    #[schemars(skip)]
    capabilities: Option<serde_json::Value>,
    // transform
    #[serde(default)]
    consumes: Option<Vec<WireConsume>>,
    #[serde(default)]
    produces: Option<Vec<Produce>>,
    #[serde(default)]
    provider: Option<ProviderNeed>,
    #[serde(default)]
    models: Option<Vec<ModelSpec>>,
    // export
    #[serde(default)]
    target: Option<String>,
    #[serde(default)]
    writes: Option<ExportWrites>,
    #[serde(default)]
    readback: Option<bool>,
}

/// 三类插件共有的部分（协议 §2.2）。
#[derive(Debug, Clone, PartialEq)]
pub struct PluginHeader {
    pub protocol: u32,
    pub id: PluginId,
    pub name: String,
    pub description: Option<String>,
    pub version: semver::Version,
    pub license: String,
    pub homepage: Option<String>,
    pub maintainers: Vec<String>,
    pub upstream: Vec<Upstream>,
    pub runtime: Vec<RuntimeRequirement>,
    pub settings: Vec<Setting>,
    pub permissions: Permissions,
    pub install: Option<Install>,
    pub run: Run,
}

/// 清单：共有部分加按种类分好的那部分。
#[derive(Debug, Clone, PartialEq)]
pub struct PluginManifest {
    pub header: PluginHeader,
    pub kind: PluginKind,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PluginKind {
    Fetch(Box<FetchSpec>),
    Transform(TransformSpec),
    Export(ExportSpec),
}

/// 获取插件的字段（协议 §2.3）。几种取法各是一个方法（§2.3.1）。
#[derive(Debug, Clone, PartialEq)]
pub struct FetchSpec {
    pub platforms: Vec<PlatformId>,
    pub match_patterns: Vec<MatchPattern>,
    pub sites: Vec<Site>,
    pub visitors: Vec<Visitor>,
    /// 照清单里写的顺序；至少一种做了。
    pub methods: Vec<Method>,
    pub pacing: BTreeMap<PlatformId, Pacing>,
}

impl FetchSpec {
    /// 这个插件是不是通用网页一类：运行器把它排在平台插件后面（§3.1）。
    pub fn is_web(&self) -> bool {
        self.platforms.iter().any(|p| p.as_str() == PlatformId::WEB)
    }

    /// 粗筛：链接有没有命中 `match`。最终身份由 canonicalize 判。
    pub fn matches(&self, url: &url::Url) -> bool {
        self.match_patterns.iter().any(|p| p.matches(url))
    }

    pub fn method(&self, id: &str) -> Option<&Method> {
        self.methods.iter().find(|m| m.id == id)
    }

    /// 做了的方法。
    pub fn built(&self) -> impl Iterator<Item = &Method> {
        self.methods.iter().filter(|m| m.built)
    }

    /// 做了的方法里最高的一层（含 `perPlatform`），列表上显示。
    pub fn access(&self) -> Access {
        self.built()
            .flat_map(|m| {
                std::iter::once(m.access).chain(m.per_platform.values().map(|p| p.access))
            })
            .max()
            .unwrap_or(Access::OfficialExport)
    }

    /// 做了的方法里最高的一档 `login`。
    pub fn login(&self) -> Login {
        self.built()
            .flat_map(|m| std::iter::once(m.login).chain(m.per_platform.values().map(|p| p.login)))
            .max()
            .unwrap_or(Login::None)
    }
}

/// 一种方法（§2.3.1）：自己的层级、账号用在哪、交哪些站点的凭据、能做什么。
#[derive(Debug, Clone, PartialEq)]
pub struct Method {
    pub id: String,
    pub name: String,
    /// `false`：声明了、还没做的。照样列出来，开不了，宿主不调。
    pub built: bool,
    pub access: Access,
    pub login: Login,
    pub account: Account,
    /// 宿主只把这几个站点的凭据交给这种方法。
    pub sites: Vec<String>,
    /// 交哪些访客身份；和 `login` 无关。
    pub visitors: Vec<String>,
    pub per_platform: BTreeMap<PlatformId, PlatformUse>,
    pub interaction: Interaction,
    /// 不在插件进程里跑：由浏览器脚本一类把链接交给本机接口。这时没有能力。
    pub delivers: Option<Delivers>,
    pub capabilities: Capabilities,
    pub risk: Option<Risk>,
}

impl Method {
    /// 没写开关时开不开（§2.3.1）：`private-api`、`client-hook` 两层关着，其余开着。
    pub fn on_by_default(&self) -> bool {
        self.access < Access::PrivateApi
    }

    /// 宿主会不会调它：做了、在插件进程里跑。
    pub fn callable(&self) -> bool {
        self.built && self.delivers.is_none()
    }

    /// 挑方法的先后（§2.3.1）：先层级低的，同一层先不带账号的。
    pub fn rank(&self) -> (Access, Account) {
        (self.access, self.account)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Capabilities {
    pub fetch: Option<FetchCapability>,
    pub list: Option<ListCapability>,
    pub unfavorite: Option<CapabilityUse>,
}

/// 每种能力都有的那几项；层级、账号、站点在方法上。
#[derive(Debug, Clone, Default, PartialEq)]
pub struct CapabilityUse {
    pub timeout: Option<DurationSpec>,
    pub provider: ProviderNeed,
}

/// 多平台插件里某个平台走的路不同时的覆盖；没写的项沿用方法本身的。
#[derive(Debug, Clone, PartialEq)]
pub struct PlatformUse {
    pub access: Access,
    pub login: Login,
    pub sites: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FetchCapability {
    pub uses: CapabilityUse,
    pub outputs: BTreeSet<OutputKind>,
    pub input: BTreeSet<FetchInputKind>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ListCapability {
    pub uses: CapabilityUse,
    pub order: ListOrder,
    pub sources: Vec<SourceInfo>,
    /// 导入插件能读的导出文件名 glob（§3.9）。
    pub files: Vec<String>,
    /// 来源从你给的一条链接开始（§2.3.1）：导入页给一个链接框，框上写这句；
    /// `sources` 带 `--link` 时插件把链接换成来源。
    pub link: Option<String>,
}

/// 转换插件的字段（协议 §2.9）。消息见 [`super::transform`]，宿主一侧是 `ResidentTransform`。
#[derive(Debug, Clone, PartialEq)]
pub struct TransformSpec {
    pub consumes: Vec<Consume>,
    pub produces: Vec<Produce>,
    pub provider: ProviderNeed,
    pub models: Vec<ModelSpec>,
}

/// 导出插件的字段（协议 §2.10）。宿主还不跑导出插件，只读清单。
#[derive(Debug, Clone, PartialEq)]
pub struct ExportSpec {
    pub target: String,
    pub writes: ExportWrites,
    pub readback: bool,
}

/// `match` 的一项：`host/path` glob，不含 scheme（协议 §2.3）。
///
/// 主机部分是 `*`（任意主机）、`*.example.com`（子域，不含裸域）或字面主机名；
/// 路径部分可省（任意路径），`*` 匹配任意字符，包括 `/`。查询串和端口不参与。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MatchPattern {
    host: String,
    path: Option<String>,
}

impl MatchPattern {
    fn parse(text: &str) -> Result<Self, String> {
        if text.contains("://") || text.contains('?') || text.contains('#') {
            return Err(format!(
                "`{text}`: write `host/path` without scheme, query or fragment"
            ));
        }
        let (host, path) = match text.split_once('/') {
            Some((h, p)) => (h, Some(p.to_owned())),
            None => (text, None),
        };
        let literal = host.strip_prefix("*.").unwrap_or(host);
        let host_ok = host == "*"
            || (!literal.is_empty()
                && literal.split('.').all(|label| {
                    !label.is_empty()
                        && label
                            .bytes()
                            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
                }));
        if !host_ok {
            return Err(format!(
                "`{text}`: host must be `*`, `*.example.com` or a lowercase host name"
            ));
        }
        Ok(Self {
            host: host.to_owned(),
            path,
        })
    }

    pub fn matches(&self, url: &url::Url) -> bool {
        if !matches!(url.scheme(), "http" | "https") {
            return false;
        }
        let Some(host) = url.host_str() else {
            return false;
        };
        let host_ok = match self.host.strip_prefix("*.") {
            _ if self.host == "*" => true,
            Some(suffix) => host
                .strip_suffix(suffix)
                .is_some_and(|rest| rest.len() > 1 && rest.ends_with('.')),
            None => host == self.host,
        };
        host_ok
            && self.path.as_deref().is_none_or(|pattern| {
                glob_match(
                    pattern.as_bytes(),
                    url.path().trim_start_matches('/').as_bytes(),
                )
            })
    }
}

/// 只认 `*` 的 glob。
fn glob_match(pattern: &[u8], text: &[u8]) -> bool {
    let (mut p, mut t) = (0, 0);
    let mut star: Option<(usize, usize)> = None;
    while t < text.len() {
        if p < pattern.len() && pattern[p] == b'*' {
            star = Some((p, t));
            p += 1;
        } else if p < pattern.len() && pattern[p] == text[t] {
            p += 1;
            t += 1;
        } else if let Some((sp, st)) = star {
            p = sp + 1;
            t = st + 1;
            star = Some((sp, st + 1));
        } else {
            return false;
        }
    }
    pattern[p..].iter().all(|&b| b == b'*')
}

impl PluginManifest {
    /// 读一份 `usuyo-plugin.yaml`。不认识的字段不算错，列在 `unknown_fields` 里。
    pub fn from_yaml(text: &str) -> Result<Parsed<Self>, PluginError> {
        let value: serde_json::Value = serde_yaml_ng::from_str(text).map_err(|e| {
            let field = e
                .location()
                .map_or_else(String::new, |l| format!("line {}", l.line()));
            PluginError::Manifest {
                field,
                message: format!("not valid YAML: {e}"),
            }
        })?;
        let mut unknown_fields = Vec::new();
        let wire: WireManifest = super::deserialize_tracked(value, &mut unknown_fields)
            .map_err(|(field, message)| PluginError::Manifest { field, message })?;
        let value = validate(wire)
            .map_err(|Invalid { field, message }| PluginError::Manifest { field, message })?;
        Ok(Parsed {
            value,
            unknown_fields,
        })
    }

    pub fn fetch(&self) -> Option<&FetchSpec> {
        match &self.kind {
            PluginKind::Fetch(spec) => Some(spec),
            _ => None,
        }
    }
}

struct Invalid {
    field: String,
    message: String,
}

fn invalid(field: impl Into<String>, message: impl Into<String>) -> Invalid {
    Invalid {
        field: field.into(),
        message: message.into(),
    }
}

type Check<T> = Result<T, Invalid>;

/// 不能出现在环境变量名里的字样：凭据只走凭据文件（§2.7）。
/// `AUTH`、`SOCK`：`SSH_AUTH_SOCK` 一类变量交出的是用户的代理和登录态。
const CREDENTIAL_WORDS: [&str; 7] = [
    "TOKEN", "COOKIE", "KEY", "SECRET", "PASSWORD", "AUTH", "SOCK",
];

/// 宿主自己给插件设的变量（§3.2），清单不能声明透传它们；前缀以 `_` 结尾。
const HOST_SET_ENV: [&str; 9] = [
    "HOME",
    "PATH",
    "TMPDIR",
    "TMP",
    "TEMP",
    "SYSTEMROOT",
    "WINDIR",
    "XDG_",
    "USUYO_",
];

/// 能力的 `timeout` 最长算多久：一个插件声明得再长，也不能一直占着取内容的那一个工人。
const CAPABILITY_TIMEOUT_MAX: std::time::Duration = std::time::Duration::from_secs(2 * 3600);

/// 能写在 `permissions.fs` 里的占位符（§2.7）。
const FS_PLACEHOLDERS: [&str; 9] = [
    "${OUT}",
    "${INPUT}",
    "${CACHE}",
    "${PROFILE}",
    "${TARGET}",
    "${MODELS}",
    "${PLUGIN_DIR}",
    "${TMP}",
    "${HOME}",
];

pub(crate) const PLUGIN_DIR_PLACEHOLDER: &str = "${PLUGIN_DIR}";

fn validate(w: WireManifest) -> Check<PluginManifest> {
    if !SUPPORTED_PROTOCOLS.contains(&w.protocol) {
        return Err(invalid(
            "protocol",
            format!(
                "protocol {} is not supported; this host supports {:?}",
                w.protocol, SUPPORTED_PROTOCOLS
            ),
        ));
    }
    non_empty("name", &w.name)?;
    non_empty("license", &w.license)?;
    if w.maintainers.is_empty() {
        return Err(invalid("maintainers", "list at least one GitHub login"));
    }
    for (i, m) in w.maintainers.iter().enumerate() {
        non_empty(&format!("maintainers[{i}]"), m)?;
    }
    if let Some(h) = &w.homepage {
        check_http_url(h).map_err(|e| invalid("homepage", e))?;
    }
    for (i, u) in w.upstream.iter().enumerate() {
        check_http_url(&u.url).map_err(|e| invalid(format!("upstream[{i}].url"), e))?;
        non_empty(&format!("upstream[{i}].license"), &u.license)?;
    }

    let mut runtime = Vec::new();
    for (i, r) in w.requires.runtime.iter().enumerate() {
        let req = RuntimeRequirement::parse(r)
            .map_err(|e| invalid(format!("requires.runtime[{i}]"), e))?;
        if runtime
            .iter()
            .any(|x: &RuntimeRequirement| x.name == req.name)
        {
            return Err(invalid(
                format!("requires.runtime[{i}]"),
                format!("`{}` is listed twice", req.name),
            ));
        }
        runtime.push(req);
    }

    let is_fetch = w.kind == PluginKindTag::Fetch;
    check_settings(&w.settings, is_fetch)?;
    let permissions = check_permissions(w.permissions.clone(), &runtime, &w.settings)?;
    check_run(&w.run, &runtime)?;
    if let Some(install) = &w.install {
        check_install(install)?;
    }

    let header_kind = w.kind;
    let kind = match header_kind {
        PluginKindTag::Fetch => PluginKind::Fetch(Box::new(fetch_spec(&w)?)),
        PluginKindTag::Transform => PluginKind::Transform(transform_spec(&w)?),
        PluginKindTag::Export => {
            let spec = export_spec(&w)?;
            if spec.readback && !permissions.fs.read.iter().any(|p| p == "${TARGET}") {
                return Err(invalid(
                    "permissions.fs.read",
                    "export plugins with `readback: true` must list `${TARGET}`",
                ));
            }
            PluginKind::Export(spec)
        }
    };

    Ok(PluginManifest {
        header: PluginHeader {
            protocol: w.protocol,
            id: w.id,
            name: w.name,
            description: w.description,
            version: w.version,
            license: w.license,
            homepage: w.homepage,
            maintainers: w.maintainers,
            upstream: w.upstream,
            runtime,
            settings: w.settings,
            permissions,
            install: w.install,
            run: w.run,
        },
        kind,
    })
}

fn non_empty(field: &str, value: &str) -> Check<()> {
    if value.trim().is_empty() {
        return Err(invalid(field, "must not be empty"));
    }
    Ok(())
}

fn check_settings(settings: &[Setting], is_fetch: bool) -> Check<()> {
    let mut seen = BTreeSet::new();
    for (i, s) in settings.iter().enumerate() {
        let f = |name: &str| format!("settings[{i}].{name}");
        let key_ok = !s.key.is_empty()
            && s.key.split('-').all(|part| {
                !part.is_empty()
                    && part
                        .bytes()
                        .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit())
            });
        if !key_ok {
            return Err(invalid(
                f("key"),
                format!("`{}` must be lowercase words joined by `-`", s.key),
            ));
        }
        if !seen.insert(s.key.as_str()) {
            return Err(invalid(f("key"), format!("`{}` is declared twice", s.key)));
        }
        non_empty(&f("label"), &s.label)?;
        match (s.value_type, s.options.is_empty()) {
            (SettingType::Enum, true) => {
                return Err(invalid(f("options"), "`enum` settings need options"));
            }
            (SettingType::Enum, false) => {}
            (_, false) => return Err(invalid(f("options"), "only `enum` settings take options")),
            (_, true) => {}
        }
        if let Some(d) = &s.default {
            let ok = match s.value_type {
                SettingType::String => d.is_string(),
                SettingType::Bool => d.is_boolean(),
                SettingType::Int => d.is_i64() || d.is_u64(),
                SettingType::Enum => d.as_str().is_some_and(|v| s.options.iter().any(|o| o == v)),
            };
            if !ok {
                return Err(invalid(
                    f("default"),
                    "does not match `type` (or is not one of `options`)",
                ));
            }
        }
        if s.inject.is_some() && !s.secret {
            return Err(invalid(f("inject"), "only secret settings are injected"));
        }
        if s.secret && !is_fetch && s.inject.is_none() {
            return Err(invalid(
                f("inject"),
                "secret settings of transform and export plugins need `inject`",
            ));
        }
    }
    Ok(())
}

fn check_permissions(
    p: WirePermissions,
    runtime: &[RuntimeRequirement],
    settings: &[Setting],
) -> Check<Permissions> {
    for (side, paths) in [("read", &p.fs.read), ("write", &p.fs.write)] {
        for (i, path) in paths.iter().enumerate() {
            if !FS_PLACEHOLDERS.iter().any(|ph| {
                path.strip_prefix(ph)
                    .is_some_and(|rest| rest.is_empty() || rest.starts_with('/'))
            }) {
                return Err(invalid(
                    format!("permissions.fs.{side}[{i}]"),
                    format!("`{path}` must start with a placeholder such as `${{OUT}}`"),
                ));
            }
        }
    }
    match (p.network.mode, p.network.hosts.is_empty()) {
        (NetworkMode::None, false) => {
            return Err(invalid(
                "permissions.network.hosts",
                "`mode: none` takes no hosts",
            ));
        }
        (NetworkMode::Allowlist, true) => {
            return Err(invalid(
                "permissions.network.hosts",
                "`mode: allowlist` needs hosts",
            ));
        }
        _ => {}
    }
    for (i, h) in p.network.hosts.iter().enumerate() {
        if let Some(key) = h
            .strip_prefix("${setting.")
            .and_then(|r| r.strip_suffix('}'))
        {
            if !settings.iter().any(|s| s.key == key) {
                return Err(invalid(
                    format!("permissions.network.hosts[{i}]"),
                    format!("setting `{key}` is not declared"),
                ));
            }
        } else if h.is_empty() || h.contains('/') || h.contains("${") {
            return Err(invalid(
                format!("permissions.network.hosts[{i}]"),
                format!("`{h}` is not a host, `host:port`, `*` or `${{setting.<key>}}`"),
            ));
        }
    }
    for (i, name) in p.env.iter().enumerate() {
        let ok = name
            .bytes()
            .next()
            .is_some_and(|b| b.is_ascii_alphabetic() || b == b'_')
            && name.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_');
        if !ok {
            return Err(invalid(
                format!("permissions.env[{i}]"),
                format!("`{name}` is not a variable name"),
            ));
        }
        let upper = name.to_ascii_uppercase();
        let host_set = HOST_SET_ENV.iter().any(|h| match h.strip_suffix('_') {
            Some(_) => upper.starts_with(h),
            None => upper == *h,
        });
        if host_set {
            return Err(invalid(
                format!("permissions.env[{i}]"),
                format!("`{name}` is set by the host"),
            ));
        }
        if let Some(word) = CREDENTIAL_WORDS.iter().find(|w| upper.contains(*w)) {
            return Err(invalid(
                format!("permissions.env[{i}]"),
                format!(
                    "`{name}` looks like a credential ({word}); credentials only come through files"
                ),
            ));
        }
    }
    Ok(Permissions {
        exec: p
            .exec
            .unwrap_or_else(|| runtime.iter().map(|r| r.name.clone()).collect()),
        fs: p.fs,
        network: p.network,
        env: p.env,
        gpu: p.gpu,
        devices: p.devices,
        limits: p.limits,
    })
}

fn check_run(run: &Run, runtime: &[RuntimeRequirement]) -> Check<()> {
    let Some(first) = run.command.first() else {
        return Err(invalid("run.command", "must not be empty"));
    };
    if !first.starts_with(PLUGIN_DIR_PLACEHOLDER) && !runtime.iter().any(|r| &r.name == first) {
        return Err(invalid(
            "run.command[0]",
            format!(
                "`{first}` is outside `${{PLUGIN_DIR}}`, so it must be listed in `requires.runtime`"
            ),
        ));
    }
    Ok(())
}

fn check_install(install: &Install) -> Check<()> {
    for (key, asset) in &install.assets {
        let key_ok = key == "any"
            || key.split_once('-').is_some_and(|(os, arch)| {
                matches!(os, "darwin" | "linux" | "windows") && matches!(arch, "amd64" | "arm64")
            });
        if !key_ok {
            return Err(invalid(
                format!("install.assets.{key}"),
                "key must be `any` or `<darwin|linux|windows>-<amd64|arm64>`",
            ));
        }
        if asset.name.is_empty() || asset.name.contains(['/', '\\']) {
            return Err(invalid(
                format!("install.assets.{key}.name"),
                "must be a plain file name",
            ));
        }
        check_sha256(&format!("install.assets.{key}.sha256"), &asset.sha256)?;
    }
    Ok(())
}

fn check_sha256(field: &str, value: &str) -> Check<()> {
    if value.len() != 64
        || !value
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
    {
        return Err(invalid(field, "must be 64 lowercase hex digits"));
    }
    Ok(())
}

/// 种类专属的字段出现在别的种类里就报错，免得写错种类没人发现。
fn reject_foreign(w: &WireManifest) -> Check<()> {
    for (field, set) in [
        ("access", w.access.is_some()),
        ("login", w.login.is_some()),
        ("capabilities", w.capabilities.is_some()),
    ] {
        if set {
            return Err(invalid(
                field,
                "top-level `access`, `login` and `capabilities` were replaced by `methods` (protocol §2.3.1): list each way of fetching as a method with its own `access`, `login`, `account` and `capabilities`",
            ));
        }
    }
    let present: [(&str, bool, PluginKindTag); 13] = [
        ("platforms", w.platforms.is_some(), PluginKindTag::Fetch),
        ("match", w.match_patterns.is_some(), PluginKindTag::Fetch),
        ("sites", w.sites.is_some(), PluginKindTag::Fetch),
        ("visitors", w.visitors.is_some(), PluginKindTag::Fetch),
        ("methods", w.methods.is_some(), PluginKindTag::Fetch),
        ("pacing", w.pacing.is_some(), PluginKindTag::Fetch),
        ("consumes", w.consumes.is_some(), PluginKindTag::Transform),
        ("produces", w.produces.is_some(), PluginKindTag::Transform),
        ("provider", w.provider.is_some(), PluginKindTag::Transform),
        ("models", w.models.is_some(), PluginKindTag::Transform),
        ("target", w.target.is_some(), PluginKindTag::Export),
        ("writes", w.writes.is_some(), PluginKindTag::Export),
        ("readback", w.readback.is_some(), PluginKindTag::Export),
    ];
    for (field, set, owner) in present {
        if set && owner != w.kind {
            let owner = serde_json::to_value(owner).unwrap_or_default();
            return Err(invalid(
                field,
                format!(
                    "only {} plugins have `{field}`",
                    owner.as_str().unwrap_or_default()
                ),
            ));
        }
    }
    Ok(())
}

fn required<'a, T>(field: &str, value: &'a Option<T>) -> Check<&'a T> {
    value.as_ref().ok_or_else(|| invalid(field, "is required"))
}

fn fetch_spec(w: &WireManifest) -> Check<FetchSpec> {
    reject_foreign(w)?;
    let platforms = required("platforms", &w.platforms)?.clone();
    if platforms.is_empty() {
        return Err(invalid("platforms", "list at least one platform"));
    }
    if platforms.iter().collect::<BTreeSet<_>>().len() != platforms.len() {
        return Err(invalid("platforms", "a platform is listed twice"));
    }
    let sites = w.sites.clone().unwrap_or_default();
    check_sites(&sites)?;
    let visitors = w.visitors.clone().unwrap_or_default();
    check_visitors(&visitors)?;

    let wire_methods = required("methods", &w.methods)?;
    let ctx = CapabilityContext {
        platforms: &platforms,
        sites: &sites,
        visitors: &visitors,
    };
    let mut methods: Vec<Method> = Vec::new();
    for (i, m) in wire_methods.iter().enumerate() {
        let method = method(m, &format!("methods[{i}]"), &ctx)?;
        if methods.iter().any(|x| x.id == method.id) {
            return Err(invalid(
                format!("methods[{i}].id"),
                format!("`{}` is declared twice", method.id),
            ));
        }
        methods.push(method);
    }
    if !methods.iter().any(|m| m.built) {
        return Err(invalid(
            "methods",
            "declare at least one method that is built",
        ));
    }
    for s in &sites {
        if !methods.iter().any(|m| {
            m.sites.contains(&s.site) || m.per_platform.values().any(|p| p.sites.contains(&s.site))
        }) {
            return Err(invalid(
                "sites",
                format!("`{}` is not used by any method", s.site),
            ));
        }
    }

    let takes_urls = methods.iter().filter(|m| m.built).any(|m| {
        m.capabilities
            .fetch
            .as_ref()
            .is_some_and(|f| f.input.contains(&FetchInputKind::Url))
    });
    let match_patterns = match (&w.match_patterns, takes_urls) {
        (Some(list), _) if !list.is_empty() => list
            .iter()
            .enumerate()
            .map(|(i, p)| MatchPattern::parse(p).map_err(|e| invalid(format!("match[{i}]"), e)))
            .collect::<Check<Vec<_>>>()?,
        (_, true) => {
            return Err(invalid(
                "match",
                "required when a method's `fetch` takes `url` input",
            ));
        }
        _ => Vec::new(),
    };

    let pacing = w.pacing.clone().unwrap_or_default();
    for key in pacing.keys() {
        if !platforms.contains(key) {
            return Err(invalid(
                format!("pacing.{key}"),
                "is not one of `platforms`",
            ));
        }
    }

    Ok(FetchSpec {
        platforms,
        match_patterns,
        sites,
        visitors,
        methods,
        pacing,
    })
}

/// 方法 id：小写字母、数字、连字符，最多 64 字节；调用时经 `--method` 传。
fn is_method_id(id: &str) -> bool {
    !id.is_empty()
        && id.len() <= 64
        && !id.starts_with('-')
        && id
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
}

fn method(m: &WireMethod, path: &str, ctx: &CapabilityContext<'_>) -> Check<Method> {
    let f = |name: &str| format!("{path}.{name}");
    if !is_method_id(&m.id) {
        return Err(invalid(
            f("id"),
            format!("`{}` must be lowercase letters, digits and `-`", m.id),
        ));
    }
    non_empty(&f("name"), &m.name)?;
    check_method_sites(
        &f("sites"),
        m.built,
        m.account,
        m.login,
        &m.sites,
        ctx.sites,
    )?;
    for name in &m.visitors {
        if !ctx.visitors.iter().any(|v| &v.site == name) {
            return Err(invalid(
                f("visitors"),
                format!("`{name}` is not declared in top-level `visitors`"),
            ));
        }
    }
    if m.built && m.account == Account::Visitor && m.visitors.is_empty() {
        return Err(invalid(
            f("visitors"),
            "`account: visitor` methods name the visitor identities they use",
        ));
    }
    let mut per_platform = BTreeMap::new();
    for (platform, o) in &m.per_platform {
        let p = format!("{path}.perPlatform.{platform}");
        if !ctx.platforms.contains(platform) {
            return Err(invalid(p, "is not one of `platforms`"));
        }
        let pu = PlatformUse {
            access: o.access.unwrap_or(m.access),
            login: o.login.unwrap_or(m.login),
            sites: o.sites.clone().unwrap_or_else(|| m.sites.clone()),
        };
        check_method_sites(
            &format!("{p}.sites"),
            m.built,
            m.account,
            pu.login,
            &pu.sites,
            ctx.sites,
        )?;
        per_platform.insert(platform.clone(), pu);
    }
    if let Some(r) = &m.risk {
        non_empty(&f("risk.text"), &r.text)?;
        non_empty(&f("risk.source.label"), &r.source.label)?;
        check_http_url(&r.source.url).map_err(|e| invalid(f("risk.source.url"), e))?;
    }
    let capabilities = match (&m.capabilities, m.delivers, m.built) {
        (Some(_), Some(_), _) => {
            return Err(invalid(
                f("capabilities"),
                "methods that deliver links run outside the plugin and have no capabilities",
            ));
        }
        (None, None, true) => {
            return Err(invalid(
                f("capabilities"),
                "built methods that run in the plugin need `capabilities`",
            ));
        }
        (None, _, _) => Capabilities::default(),
        (Some(caps), None, _) => capabilities(caps, &f("capabilities"))?,
    };
    if m.built
        && m.delivers.is_none()
        && capabilities.fetch.is_none()
        && capabilities.list.is_none()
    {
        return Err(invalid(
            f("capabilities"),
            "declare `fetch`, `list` or both",
        ));
    }
    Ok(Method {
        id: m.id.clone(),
        name: m.name.clone(),
        built: m.built,
        access: m.access,
        login: m.login,
        account: m.account,
        sites: m.sites.clone(),
        visitors: m.visitors.clone(),
        per_platform,
        interaction: m.interaction,
        delivers: m.delivers,
        capabilities,
        risk: m.risk.clone(),
    })
}

fn capabilities(caps: &WireCapabilities, path: &str) -> Check<Capabilities> {
    let fetch = caps
        .fetch
        .as_ref()
        .map(|c| fetch_capability(c, &format!("{path}.fetch")))
        .transpose()?;
    let list = caps
        .list
        .as_ref()
        .map(|c| list_capability(c, &format!("{path}.list")))
        .transpose()?;
    let unfavorite = caps
        .unfavorite
        .as_ref()
        .map(|c| {
            let p = format!("{path}.unfavorite");
            only_fields(c, &p, &[])?;
            Ok(capability_use(c))
        })
        .transpose()?;
    Ok(Capabilities {
        fetch,
        list,
        unfavorite,
    })
}

fn check_sites(sites: &[Site]) -> Check<()> {
    let mut seen = BTreeSet::new();
    for (i, s) in sites.iter().enumerate() {
        let f = |name: &str| format!("sites[{i}].{name}");
        if !is_host_name(&s.site) {
            return Err(invalid(
                f("site"),
                format!("`{}` is not a lowercase host name", s.site),
            ));
        }
        if !seen.insert(s.site.as_str()) {
            return Err(invalid(
                f("site"),
                format!("`{}` is declared twice", s.site),
            ));
        }
        non_empty(&f("name"), &s.name)?;
        match (&s.login_url, s.credential) {
            (Some(u), _) => {
                check_http_url(u).map_err(|e| invalid(f("loginUrl"), e))?;
            }
            (None, Credential::Cookies | Credential::BrowserProfile) => {
                return Err(invalid(
                    f("loginUrl"),
                    "is required for `cookies` and `browser-profile`",
                ));
            }
            (None, Credential::App) => {}
        }
        check_domains(&format!("sites[{i}].domains"), &s.site, &s.domains)?;
    }
    Ok(())
}

/// `domains` 里的每一项都要和站点同一个可注册域名，不能是公共后缀。
fn check_domains(field: &str, site: &str, domains: &[String]) -> Check<()> {
    let registrable = psl::domain_str(site);
    for (j, d) in domains.iter().enumerate() {
        let bare = d.strip_prefix('.').unwrap_or(d);
        let same = is_host_name(bare)
            && psl::suffix_str(bare) != Some(bare)
            && registrable.is_some()
            && psl::domain_str(bare) == registrable;
        if !same {
            return Err(invalid(
                format!("{field}[{j}]"),
                format!(
                    "`{d}` must share the registrable domain of `{site}` and not be a public suffix"
                ),
            ));
        }
    }
    Ok(())
}

fn check_visitors(visitors: &[Visitor]) -> Check<()> {
    let mut seen = BTreeSet::new();
    for (i, v) in visitors.iter().enumerate() {
        let f = |name: &str| format!("visitors[{i}].{name}");
        if !is_host_name(&v.site) {
            return Err(invalid(
                f("site"),
                format!("`{}` is not a lowercase host name", v.site),
            ));
        }
        if !seen.insert(v.site.as_str()) {
            return Err(invalid(
                f("site"),
                format!("`{}` is declared twice", v.site),
            ));
        }
        non_empty(&f("name"), &v.name)?;
        let url = check_http_url(&v.url).map_err(|e| invalid(f("url"), e))?;
        if url.scheme() != "https" {
            return Err(invalid(f("url"), "must be an https address"));
        }
        check_domains(&f("domains"), &v.site, &v.domains)?;
    }
    Ok(())
}

fn is_host_name(h: &str) -> bool {
    !h.is_empty()
        && h.split('.').all(|label| {
            !label.is_empty()
                && label
                    .bytes()
                    .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
        })
}

struct CapabilityContext<'a> {
    platforms: &'a [PlatformId],
    sites: &'a [Site],
    visitors: &'a [Visitor],
}

/// 能力字段属于哪种能力：`allowed` 之外的专属字段出现了就报错。挪到方法上的字段说清挪到哪里。
fn only_fields(c: &WireCapability, path: &str, allowed: &[&str]) -> Check<()> {
    for (name, set) in [
        ("access", c.access.is_some()),
        ("login", c.login.is_some()),
        ("sites", c.sites.is_some()),
        ("visitors", c.visitors.is_some()),
        ("interaction", c.interaction.is_some()),
        ("perPlatform", c.per_platform.is_some()),
    ] {
        if set {
            return Err(invalid(
                format!("{path}.{name}"),
                format!("`{name}` moved from the capability to its method (protocol §2.3.1)"),
            ));
        }
    }
    let present = [
        ("outputs", c.outputs.is_some()),
        ("input", c.input.is_some()),
        ("order", c.order.is_some()),
        ("sources", c.sources.is_some()),
        ("files", c.files.is_some()),
        ("link", c.link.is_some()),
    ];
    for (name, set) in present {
        if set && !allowed.contains(&name) {
            return Err(invalid(
                format!("{path}.{name}"),
                "does not apply to this capability",
            ));
        }
    }
    Ok(())
}

fn capability_use(c: &WireCapability) -> CapabilityUse {
    CapabilityUse {
        timeout: c.timeout.map(|t| t.at_most(CAPABILITY_TIMEOUT_MAX)),
        provider: c.provider,
    }
}

/// 方法的站点（§2.3.1）：宿主只把这几个站点的凭据交给它。登录进 usuyo 的方法做了就要写；
/// 账号在你手机或电脑客户端里的只能写 `credential: app` 的站点（宿主什么都不交，插件在 `health` 里报登录）；
/// 别的方法宿主拿不到账号，不写站点。
fn check_method_sites(
    path: &str,
    built: bool,
    account: Account,
    login: Login,
    names: &[String],
    declared: &[Site],
) -> Check<()> {
    if login == Login::None && !names.is_empty() {
        return Err(invalid(
            path,
            "`login: none` methods get no account, so no sites",
        ));
    }
    if built && account == Account::InUsuyo && login != Login::None && names.is_empty() {
        return Err(invalid(
            path,
            "methods that log in to usuyo must name their sites",
        ));
    }
    let mut profiles = 0;
    for name in names {
        let Some(site) = declared.iter().find(|s| &s.site == name) else {
            return Err(invalid(
                path,
                format!("`{name}` is not declared in top-level `sites`"),
            ));
        };
        let fits = match account {
            Account::InUsuyo => true,
            Account::OwnPhone | Account::OwnApp => site.credential == Credential::App,
            _ => false,
        };
        if !fits {
            return Err(invalid(
                path,
                format!(
                    "`{name}`: only `account: in-usuyo` methods take login sites (and `own-phone`, `own-app` ones `credential: app` sites)"
                ),
            ));
        }
        if site.credential == Credential::BrowserProfile {
            profiles += 1;
        }
    }
    if profiles > 1 {
        return Err(invalid(
            path,
            "a method may use at most one `browser-profile` site",
        ));
    }
    Ok(())
}

fn fetch_capability(c: &WireCapability, path: &str) -> Check<FetchCapability> {
    only_fields(c, path, &["outputs", "input"])?;
    let uses = capability_use(c);
    let outputs: BTreeSet<_> = c
        .outputs
        .as_ref()
        .ok_or_else(|| invalid(format!("{path}.outputs"), "is required"))?
        .iter()
        .copied()
        .collect();
    if outputs.is_empty() {
        return Err(invalid(
            format!("{path}.outputs"),
            "list at least one output",
        ));
    }
    let input: BTreeSet<_> = match &c.input {
        Some(list) if list.is_empty() => {
            return Err(invalid(
                format!("{path}.input"),
                "list `url`, `ref` or both",
            ));
        }
        Some(list) => list.iter().copied().collect(),
        None => BTreeSet::from([FetchInputKind::Url]),
    };
    Ok(FetchCapability {
        uses,
        outputs,
        input,
    })
}

fn list_capability(c: &WireCapability, path: &str) -> Check<ListCapability> {
    only_fields(c, path, &["order", "sources", "files", "link"])?;
    let uses = capability_use(c);
    let order = c
        .order
        .ok_or_else(|| invalid(format!("{path}.order"), "is required"))?;
    if let Some(label) = &c.link {
        non_empty(&format!("{path}.link"), label)?;
    }
    Ok(ListCapability {
        uses,
        order,
        sources: c.sources.clone().unwrap_or_default(),
        files: c.files.clone().unwrap_or_default(),
        link: c.link.clone(),
    })
}

fn transform_spec(w: &WireManifest) -> Check<TransformSpec> {
    reject_foreign(w)?;
    let consumes: Vec<Consume> = required("consumes", &w.consumes)?
        .iter()
        .map(|c| match c {
            WireConsume::Kind(kind) => Consume {
                kind: *kind,
                mime: Vec::new(),
            },
            WireConsume::WithMime { kind, mime } => Consume {
                kind: *kind,
                mime: mime.clone(),
            },
        })
        .collect();
    if consumes.is_empty() {
        return Err(invalid("consumes", "list at least one input"));
    }
    let produces = required("produces", &w.produces)?.clone();
    if produces.is_empty() {
        return Err(invalid("produces", "list at least one product"));
    }
    if produces.iter().collect::<BTreeSet<_>>().len() != produces.len() {
        return Err(invalid("produces", "a product is listed twice"));
    }
    let provider = *required("provider", &w.provider)?;
    let models = w.models.clone().unwrap_or_default();
    let mut names = BTreeSet::new();
    for (i, m) in models.iter().enumerate() {
        let f = |name: &str| format!("models[{i}].{name}");
        if !is_model_name(&m.name) {
            return Err(invalid(
                f("name"),
                "use lowercase letters, digits, `.`, `_` and `-`, not starting with `.`, at most 100 bytes",
            ));
        }
        if !names.insert(m.name.as_str()) {
            return Err(invalid(
                f("name"),
                format!("model `{}` is listed twice", m.name),
            ));
        }
        check_http_url(&m.url).map_err(|e| invalid(f("url"), e))?;
        let mut urls = BTreeSet::from([m.url.as_str()]);
        for (j, mirror) in m.mirrors.iter().enumerate() {
            check_http_url(mirror).map_err(|e| invalid(format!("models[{i}].mirrors[{j}]"), e))?;
            if !urls.insert(mirror.as_str()) {
                return Err(invalid(
                    format!("models[{i}].mirrors[{j}]"),
                    "repeats `url` or another mirror",
                ));
            }
        }
        check_sha256(&f("sha256"), &m.sha256)?;
        if m.size == 0 {
            return Err(invalid(f("size"), "must be the download size in bytes"));
        }
        non_empty(&f("license"), &m.license)?;
        if let Some(key) = &m.setting
            && !w.settings.iter().any(|s| &s.key == key)
        {
            return Err(invalid(
                format!("models[{i}].setting"),
                format!("setting `{key}` is not declared"),
            ));
        }
    }
    Ok(TransformSpec {
        consumes,
        produces,
        provider,
        models,
    })
}

/// 模型名会成为 `${MODELS}` 下的一个目录名。
fn is_model_name(name: &str) -> bool {
    !name.is_empty()
        && name.len() <= 100
        && !name.starts_with('.')
        && name
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b"._-".contains(&b))
}

fn export_spec(w: &WireManifest) -> Check<ExportSpec> {
    reject_foreign(w)?;
    let target = required("target", &w.target)?.clone();
    non_empty("target", &target)?;
    Ok(ExportSpec {
        target,
        writes: *required("writes", &w.writes)?,
        readback: *required("readback", &w.readback)?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn url(s: &str) -> url::Url {
        url::Url::parse(s).unwrap()
    }

    #[test]
    fn match_patterns() {
        let p = MatchPattern::parse("v.douyin.com/*").unwrap();
        assert!(p.matches(&url("https://v.douyin.com/iRNBho6u/")));
        assert!(p.matches(&url("https://v.douyin.com")));
        assert!(!p.matches(&url("https://www.douyin.com/x")));

        let p = MatchPattern::parse("*.douyin.com/video/*").unwrap();
        assert!(p.matches(&url("https://www.douyin.com/video/123?x=1")));
        assert!(!p.matches(&url("https://douyin.com/video/123")));
        assert!(!p.matches(&url("https://evildouyin.com/video/123")));
        assert!(!p.matches(&url("https://www.douyin.com/user/1")));

        let any = MatchPattern::parse("*").unwrap();
        assert!(any.matches(&url("http://127.0.0.1:8080/a")));
        assert!(!any.matches(&url("ftp://example.com/a")));

        assert!(MatchPattern::parse("https://example.com/*").is_err());
        assert!(MatchPattern::parse("Example.com").is_err());
    }

    #[test]
    fn runtime_requirements() {
        let r = RuntimeRequirement::parse("yt-dlp>=2026.10").unwrap();
        assert_eq!(
            (r.name.as_str(), r.constraint.as_deref()),
            ("yt-dlp", Some(">=2026.10"))
        );
        assert!(
            RuntimeRequirement::parse("node")
                .unwrap()
                .constraint
                .is_none()
        );
        assert!(RuntimeRequirement::parse(">=1").is_err());
        assert!(RuntimeRequirement::parse("chrome>=").is_err());
    }
}
