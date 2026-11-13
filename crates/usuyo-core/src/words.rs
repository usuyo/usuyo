//! 内核给界面的话：每一句的简体中文和英文并排写在这里（SPEC §0.10 的写法和用词表）。
//! 句子里的变量写 `{name}`。出错的句子先说发生了什么，再说可以怎么做。
//! 带着两种语言走到边界（[`Said`]），由边界按设置的语言挑一种；错误码、层级这类给机器看的不翻译。

use std::fmt::Display;

use serde::Serialize;

/// 界面和内核给的话用哪种语言（按机器的 `settings.json` 的 `language`，`system` 时由系统定）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Lang {
    #[default]
    ZhHans,
    En,
}

impl Lang {
    /// BCP 47 的写法，也是交给插件的 `USUYO_LOCALE`。
    pub fn tag(self) -> &'static str {
        match self {
            Lang::ZhHans => "zh-Hans",
            Lang::En => "en",
        }
    }

    /// 系统的语言标签（`zh-Hans-CN`、`en-US`）：中文开头的用简体中文，别的都用英文。
    pub fn from_system(tag: Option<&str>) -> Lang {
        match tag {
            Some(t) if t.to_ascii_lowercase().starts_with("zh") => Lang::ZhHans,
            _ => Lang::En,
        }
    }
}

/// 表里的一句：两种语言。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Words {
    pub zh: &'static str,
    pub en: &'static str,
}

const fn w(zh: &'static str, en: &'static str) -> Words {
    Words::new(zh, en)
}

impl Words {
    /// 别的 crate 的表用它写一句。
    pub const fn new(zh: &'static str, en: &'static str) -> Words {
        Words { zh, en }
    }

    /// 没有变量的一句（`{trash}`、`{fileManager}` 照系统填）。
    pub fn said(&self) -> Said {
        self.with(&[])
    }

    /// 填好变量；两种语言填的是同一个值。
    pub fn with(&self, vars: &[(&str, &dyn Display)]) -> Said {
        Said {
            zh: fill(Lang::ZhHans, self.zh, vars),
            en: fill(Lang::En, self.en, vars),
        }
    }

    /// 两种语言各填各的值（插件给的一句、按语言写的日期）。
    pub fn with_said(&self, vars: &[(&str, &Said)]) -> Said {
        Said::by(|lang| {
            let picked: Vec<(&str, &dyn Display)> = vars
                .iter()
                .map(|(k, v)| (*k, v.pick_ref(lang) as &dyn Display))
                .collect();
            fill(lang, self.pick(lang), &picked)
        })
    }

    pub fn pick(&self, lang: Lang) -> &'static str {
        match lang {
            Lang::ZhHans => self.zh,
            Lang::En => self.en,
        }
    }
}

fn fill(lang: Lang, text: &str, vars: &[(&str, &dyn Display)]) -> String {
    let mut out = text.to_owned();
    for (name, value) in vars {
        out = out.replace(&format!("{{{name}}}"), &value.to_string());
    }
    out.replace("{trash}", TRASH.pick(lang))
        .replace("{fileManager}", FILE_MANAGER.pick(lang))
}

/// 系统各有叫法的词（SPEC §0.10）。
#[cfg(target_os = "macos")]
const TRASH: Words = w("废纸篓", "Trash");
#[cfg(target_os = "windows")]
const TRASH: Words = w("回收站", "Recycle Bin");
#[cfg(not(any(target_os = "macos", target_os = "windows")))]
const TRASH: Words = w("回收站", "Trash");
#[cfg(target_os = "macos")]
const FILE_MANAGER: Words = w("访达", "Finder");
#[cfg(target_os = "windows")]
const FILE_MANAGER: Words = w("文件资源管理器", "File Explorer");
#[cfg(not(any(target_os = "macos", target_os = "windows")))]
const FILE_MANAGER: Words = w("文件管理器", "the file manager");

/// 一句给人看的话，两种语言都在；插件给的只有一种，两边是同一句。
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Said {
    pub zh: String,
    pub en: String,
}

impl Said {
    /// 插件或平台给的原话：只有一种语言，照原样给。
    pub fn plain(text: impl Into<String>) -> Said {
        let t = text.into();
        Said {
            zh: t.clone(),
            en: t,
        }
    }

    /// 两种语言要分开算的（日期的写法、并列的分隔）。
    pub fn by(f: impl Fn(Lang) -> String) -> Said {
        Said {
            zh: f(Lang::ZhHans),
            en: f(Lang::En),
        }
    }

    pub fn pick(&self, lang: Lang) -> &str {
        self.pick_ref(lang)
    }

    fn pick_ref(&self, lang: Lang) -> &String {
        match lang {
            Lang::ZhHans => &self.zh,
            Lang::En => &self.en,
        }
    }

    /// 存进 `jobs.db` 的写法：宿主自己的话两种语言都存（换了语言，读出来照新的），插件的照原样。
    pub fn to_stored(&self) -> String {
        if self.zh == self.en {
            return self.zh.clone();
        }
        serde_json::json!({ "zh": self.zh, "en": self.en }).to_string()
    }

    /// [`Said::to_stored`] 的反面；不是那种写法的是插件的原话。
    pub fn from_stored(text: &str) -> Said {
        #[derive(serde::Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Pair {
            zh: String,
            en: String,
        }
        match serde_json::from_str::<Pair>(text) {
            Ok(p) if text.starts_with('{') => Said { zh: p.zh, en: p.en },
            _ => Said::plain(text),
        }
    }
}

/// 并列几项：「、」和「, 」。
pub(crate) fn join(lang: Lang, parts: &[&str]) -> String {
    parts.join(match lang {
        Lang::ZhHans => "、",
        Lang::En => ", ",
    })
}

/// 几句接着说：「；」和「; 」。
pub(crate) fn clauses(lang: Lang, parts: &[&str]) -> String {
    parts.join(match lang {
        Lang::ZhHans => "；",
        Lang::En => "; ",
    })
}

// —— 表 ——（按用到它的文件分组）

// fetch.rs：添加一条链接
pub(crate) const NO_LINK: Words = w(
    "这段文字中没有可添加的链接。",
    "This text doesn't contain a link that can be added.",
);
pub(crate) const LINK_WITH_PASSWORD: Words = w(
    "无法添加带用户名或密码的链接。",
    "Links that contain a user name or password can't be added.",
);
pub(crate) const LINK_WITH_PASSWORD_FIRST: Words = w(
    "无法添加带用户名或密码的链接。只检查了第一个链接。",
    "Links that contain a user name or password can't be added. Only the first link was checked.",
);
pub(crate) const PLUGIN_PAUSED: Words = w(
    "此插件已暂停。点按「立即重试」后会再使用。",
    "This plugin is paused. It will be used again after you click Retry Now.",
);
pub(crate) const READER_GONE: Words = w(
    "读取此文件的插件已不再安装。",
    "The plugin that reads this file is no longer installed.",
);
pub(crate) const SENT_FILE_GONE_NAMED: Words = w(
    "找不到发送到 usuyo 的文件「{name}」。",
    "The file \"{name}\" sent to usuyo can't be found.",
);

// engine.rs：条目页的「缺了什么」
pub(crate) const FETCH_FAILED: Words =
    w("无法获取内容：{why}", "Couldn't fetch the content: {why}");
pub(crate) const REASON_UNKNOWN: Words = w("原因未知", "unknown reason");
pub(crate) const PARTLY_SAVED: Words = w(
    "部分内容未保存（{why}）",
    "Some content wasn't saved ({why})",
);
pub(crate) const NOT_SAVED: Words = w("未保存：{what}（{why}）", "Not saved: {what} ({why})");
pub(crate) const OUT_TEXT: Words = w("正文", "post text");
pub(crate) const OUT_HTML: Words = w("网页原文", "web page");
pub(crate) const OUT_IMAGES: Words = w("图片", "images");
pub(crate) const OUT_VIDEO: Words = w("视频", "video");
pub(crate) const OUT_AUDIO: Words = w("音频", "audio");
pub(crate) const OUT_SUBTITLE: Words = w("字幕", "subtitles");
pub(crate) const OUT_META: Words = w("标题和作者", "title and author");
pub(crate) const OUT_COMMENTS: Words = w("评论", "comments");
/// 别的工具导出的文件读出来的条目，平台那一栏的名字。
pub(crate) const PLATFORM_FILE: Words = w("文件", "File");

// imports.rs、file_imports.rs：导入
pub(crate) const SOURCE_EMPTY: Words = w("来源为空或过长。", "The source is empty or too long.");
pub(crate) const NO_SUCH_GROUP: Words = w("预览中没有这一组。", "This group isn't in the preview.");
pub(crate) const STILL_LISTING: Words = w(
    "仍在列出内容。列出完成后才能确认。",
    "Items are still being listed. You can confirm after the list is complete.",
);
pub(crate) const ALREADY_CONFIRMED: Words = w(
    "这次导入已经确认。",
    "This import has already been confirmed.",
);
pub(crate) const PICK_NOT_ALLOWED: Words = w(
    "所选条目中有既不是新条目、也不在{trash}中的条目。",
    "Some selected items are neither new nor in the {trash}.",
);
pub(crate) const NO_LINK_START: Words = w(
    "{method}不能从链接开始列出。",
    "{method} can't start listing from a link.",
);
pub(crate) const PACING_WAIT: Words = w(
    "此平台需要间隔一段时间才能再次请求。可以在{time}之后重试。",
    "This platform needs a pause between requests. Try again after {time}.",
);
pub(crate) const NO_LIST_METHOD: Words = w(
    "{plugin}没有可列出已保存内容的这种获取方式。",
    "{plugin} has no fetch method like this that lists saved items.",
);
pub(crate) const METHOD_BLOCKED: Words = w("{method}：{why}", "{method}: {why}");
pub(crate) const BATCH_OFF: Words = w(
    "此平台只能逐条添加，不支持导入。",
    "Items from this platform can only be added one at a time. Importing isn't supported.",
);
pub(crate) const PLUGIN_GONE: Words =
    w("此插件已不再安装。", "This plugin is no longer installed.");
pub(crate) const SENT_FILE_GONE: Words = w(
    "找不到发送到 usuyo 的文件。",
    "The file sent to usuyo can't be found.",
);
pub(crate) const CANT_READ_FILE: Words =
    w("{plugin}无法读取此文件。", "{plugin} can't read this file.");
pub(crate) const UNFETCHABLE_ENTRIES: Words = w(
    "{plugin}列出的条目中有它自己无法获取的条目。需要更新插件。",
    "{plugin} listed items that it can't fetch itself. The plugin needs an update.",
);
pub(crate) const DELIVERED_BATCH: Words = w("发送到 usuyo 的列表", "List sent to usuyo");
pub(crate) const NO_READER: Words = w(
    "未指定读取此文件的方式。",
    "No method was chosen to read this file.",
);
pub(crate) const NO_READ_METHOD: Words = w(
    "{plugin}没有读取文件的这种方式。",
    "{plugin} has no method like this that reads files.",
);
pub(crate) const NOT_INSTALLED: Words = w("未安装此插件。", "This plugin isn't installed.");

// registry.rs：插件暂停时的一句
pub(crate) const PAUSED_BY_YOU: Words = w(
    "已暂停。点按「立即重试」后会再使用此插件。",
    "Paused. This plugin will be used again after you click Retry Now.",
);
pub(crate) const PAUSED_RATE: Words = w(
    "平台提示请求过于频繁。{time}前不会使用。",
    "The platform reported too many requests. It won't be used until {time}.",
);
pub(crate) const PAUSED_BLOCKED: Words = w(
    "平台阻止了请求。{time}前不会使用。",
    "The platform blocked the request. It won't be used until {time}.",
);
pub(crate) const PAUSED_ERROR: Words = w(
    "上次请求出错。{time}前不会使用。",
    "The last request failed. It won't be used until {time}.",
);
pub(crate) const PAUSED_WAITING: Words = w(
    "已暂停。点按「立即重试」以继续。",
    "Paused. Click Retry Now to continue.",
);
pub(crate) const SCRIPT_MISSING: Words = w(
    "插件文件夹中没有此脚本文件。重新安装插件后再试。",
    "The script file isn't in the plugin's folder. Reinstall the plugin and try again.",
);

// methods.rs：一种获取方式这台电脑上为什么不可用
pub(crate) const BLOCK_NOT_BUILT: Words = w("尚未提供", "Not available yet");
pub(crate) const BLOCK_OFF: Words = w("已关闭此获取方式", "This fetch method is turned off");
pub(crate) const BLOCK_LOGIN_MISSING: Words =
    w("需要在 usuyo 中登录{site}", "Sign in to {site} in usuyo");
pub(crate) const BLOCK_LOGIN_EXPIRED: Words = w(
    "{site}的登录已过期，需要重新登录",
    "Your {site} sign-in has expired. Sign in again",
);
pub(crate) const BLOCK_VISITOR_MISSING: Words =
    w("需要获取{site}访客身份", "Get a {site} visitor identity");
pub(crate) const BLOCK_VISITOR_EXPIRED: Words = w(
    "{site}访客身份已过期，需要重新获取",
    "The {site} visitor identity has expired. Get a new one",
);
pub(crate) const BLOCK_HEALTH_FAILED: Words = w(
    "插件无法完成自检：{why}",
    "The plugin couldn't check itself: {why}",
);
pub(crate) const BLOCK_CHECK_FAILED: Words = w("未通过检查：{name}", "Check failed: {name}");

// models.rs：模型下载失败
pub(crate) const MODEL_NETWORK: Words = w(
    "网络连接失败，或下载中断",
    "The network connection failed or the download was interrupted",
);
pub(crate) const MODEL_REFUSED: Words = w(
    "无法连接下载地址。检查设置中的代理。",
    "Can't reach the download address. Check the proxy in Settings.",
);
pub(crate) const MODEL_MISMATCH: Words = w(
    "下载的文件与插件声明的不一致",
    "The downloaded file doesn't match what the plugin declared",
);
pub(crate) const MODEL_IO: Words = w(
    "无法存储到这台电脑的模型文件夹",
    "Couldn't save to the models folder on this computer",
);
pub(crate) const MODEL_FAILED: Words = w("下载失败", "Download failed");

// transcripts.rs、facets.rs：转写与识别那一面头上的一句
pub(crate) const REMARK_NO_AUDIO: Words = w("没有音轨", "No audio track");
pub(crate) const REMARK_NO_SPEECH: Words = w("未检测到语音", "No speech detected");
pub(crate) const REMARK_STATIC: Words = w("画面几乎不变", "The picture barely changes");
pub(crate) const REMARK_NO_TEXT: Words = w("未识别到文字", "No text recognized");
pub(crate) const REMARK_DUPLICATES: Words = w("有重复的图片", "Some images are duplicates");

// transform.rs：转换插件
pub(crate) const TRANSFORM_WONT_START: Words = w("此插件无法启动", "This plugin can't start");
pub(crate) const TRANSFORM_NEEDS_MODEL: Words = w("需要先下载模型", "Download the model first");
pub(crate) const NO_ASR_PLUGIN: Words = w(
    "这台电脑上没有可转写的插件",
    "No plugin on this computer can transcribe",
);
pub(crate) const NO_OCR_PLUGIN: Words = w(
    "这台电脑上没有可识别文字的插件",
    "No plugin on this computer can recognize text",
);
pub(crate) const NO_AV_PLUGIN: Words = w(
    "这台电脑上没有可处理音视频的插件",
    "No plugin on this computer can process audio and video",
);
pub(crate) const TRANSFORM_TIMEOUT: Words =
    w("处理时间过长，已停止", "Took too long and was stopped");
pub(crate) const TRANSFORM_CRASHED: Words =
    w("转换插件意外退出", "The transform plugin quit unexpectedly");
pub(crate) const TRANSFORM_INVALID: Words = w(
    "转换插件返回的结果无效",
    "The transform plugin returned an invalid result",
);
pub(crate) const TRANSFORM_UNSUPPORTED: Words = w("无法处理此文件", "This file can't be processed");
pub(crate) const TRANSFORM_DEPENDENCY: Words = w(
    "转换插件缺少所需的组件",
    "The transform plugin is missing a component it needs",
);
pub(crate) const TRANSFORM_ERROR: Words =
    w("转换插件出错", "The transform plugin reported an error");

// reveal.rs：在访达中显示
pub(crate) const REVEAL_NO_ANSWER: Words =
    w("{fileManager}没有响应。", "{fileManager} didn't respond.");
pub(crate) const REVEAL_FAILED: Words = w(
    "无法在{fileManager}中显示。",
    "Couldn't show it in {fileManager}.",
);

// phone/：手机那一页
pub(crate) const PHONE_NOT_SERVED: Words = w(
    "此版本的 usuyo 服务不支持手机。",
    "This version of the usuyo service doesn't support phones.",
);
pub(crate) const PHONE_UNSUPPORTED: Words = w(
    "mirroir 只提供 Apple 芯片 Mac 的版本，无法安装在这台电脑上。",
    "mirroir is only available for Mac computers with Apple silicon and can't be installed on this computer.",
);
pub(crate) const PHONE_BUSY: Words = w(
    "正在安装 mirroir。安装完成后再试。",
    "mirroir is being installed. Try again when it's done.",
);
pub(crate) const PHONE_SERVICE_DOWN: Words = w(
    "「mirroir serve」未在这台 Mac 上运行。先按「驱动」一行中的步骤完成登记。",
    "\"mirroir serve\" isn't running on this Mac. Register it first using the steps in the Driver row.",
);
pub(crate) const PHONE_SERVICE_BAD: Words = w(
    "无法识别「mirroir serve」的回复。",
    "The reply from \"mirroir serve\" couldn't be read.",
);
pub(crate) const PHONE_SERVICE_TIMEOUT: Words = w(
    "「mirroir serve」没有及时响应。",
    "\"mirroir serve\" didn't respond in time.",
);
pub(crate) const PHONE_TOOL: Words = w(
    "无法读取这一屏：iPhone 镜像未连接，或尚未授权。",
    "Couldn't read this screen. iPhone Mirroring isn't connected, or access hasn't been granted.",
);
pub(crate) const PHONE_PROGRAM: Words = w(
    "这台 Mac 上的一个系统程序没有响应。",
    "A system program on this Mac didn't respond.",
);
pub(crate) const PHONE_PASTEBOARD: Words = w(
    "无法检查这台 Mac 的剪贴板是否变化。",
    "Couldn't check whether the clipboard on this Mac changed.",
);
pub(crate) const PHONE_OPEN: Words = w("无法打开系统设置。", "Couldn't open System Settings.");
pub(crate) const PHONE_NO_ADB: Words = w(
    "这台电脑上找不到 adb。",
    "adb wasn't found on this computer.",
);
pub(crate) const DRIVER_NOT_ON_RELEASE: Words = w(
    "官方发布中没有这一版的安装包。这台电脑上没有放入任何文件。",
    "The official release has no package for this version. Nothing was added to this computer.",
);
pub(crate) const DRIVER_NO_GITHUB: Words = w(
    "无法连接 GitHub。检查设置中的代理。这台电脑上没有放入任何文件。",
    "Can't connect to GitHub. Check the proxy in Settings. Nothing was added to this computer.",
);
pub(crate) const DRIVER_SHA_MISMATCH: Words = w(
    "下载的安装包与固定的 SHA256 不一致，已丢弃。这台电脑上没有放入任何文件。",
    "The downloaded package doesn't match the pinned SHA256 and was discarded. Nothing was added to this computer.",
);
pub(crate) const DRIVER_IO: Words = w(
    "无法存储到这台电脑的驱动文件夹。没有留下任何文件。",
    "Couldn't save to the driver folder on this computer. No files were left behind.",
);
pub(crate) const DRIVER_WRONG_ARCHIVE: Words = w(
    "下载的安装包中没有所需的程序，已丢弃。这台电脑上没有放入任何文件。",
    "The downloaded package doesn't contain the expected program and was discarded. Nothing was added to this computer.",
);
pub(crate) const DRIVER_NETWORK: Words = w(
    "网络连接失败，或下载中断。这台电脑上没有放入任何文件。",
    "The network connection failed or the download was interrupted. Nothing was added to this computer.",
);
pub(crate) const STEP_REGISTER: Words = w(
    "还有一步需要你在「终端」中完成：让 mirroir 以「mirroir serve」常驻运行。它会在你的登录钥匙串中添加一张只用于自身签名的证书（钥匙串询问 codesign 能否使用时，选择「始终允许」），并登记为登录时启动的后台项目。这两项会更改这台 Mac 的设置，因此 usuyo 不会替你执行。",
    "One more step needs to be done in Terminal: keep mirroir running as \"mirroir serve\". It adds a certificate to your login keychain that it uses only to sign itself (when Keychain asks whether codesign can use it, choose Always Allow), and registers a background item that opens at login. Both change settings on this Mac, so usuyo doesn't do them for you.",
);
pub(crate) const STEP_RENEW: Words = w(
    "正在运行的「mirroir serve」是旧版本。在「终端」中重新登记并重新启动它；授权跟随证书，不需要重新授予。",
    "The running \"mirroir serve\" is an older version. Register it again in Terminal and restart it. Access follows the certificate, so you don't need to grant it again.",
);
pub(crate) const STEP_STOP: Words = w(
    "「mirroir serve」仍在运行，它使用的是自己的副本。在「终端」中停止它；之后可以在访达中删除 ~/Library/LaunchAgents/dev.mirroir.serve.plist 和 ~/.mirroir-mcp/mirroir serve.app，并在「钥匙串访问」中删除证书「mirroir serve local signing」。系统设置中给它的授权由系统保留。",
    "\"mirroir serve\" is still running from its own copy. Stop it in Terminal. Afterward you can delete ~/Library/LaunchAgents/dev.mirroir.serve.plist and ~/.mirroir-mcp/mirroir serve.app in Finder, and delete the certificate \"mirroir serve local signing\" in Keychain Access. The access granted to it in System Settings stays with the system.",
);
pub(crate) const STEP_ADB_STOP: Words = w(
    "adb 服务在这台电脑的后台运行，会一直保留，usuyo 不会停止它。要停止它，在「终端」中运行：",
    "The adb server runs in the background on this computer and keeps running; usuyo doesn't stop it. To stop it, run this in Terminal:",
);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fills_both_languages() {
        let s = w("{name}不可用", "{name} isn't available");
        assert_eq!(
            s.with(&[("name", &"B站")]).pick(Lang::En),
            "B站 isn't available"
        );
        let why = Said {
            zh: "网络不通".into(),
            en: "No network".into(),
        };
        let s = w("无法获取：{why}", "Couldn't fetch: {why}").with_said(&[("why", &why)]);
        assert_eq!(
            (s.zh.as_str(), s.en.as_str()),
            ("无法获取：网络不通", "Couldn't fetch: No network")
        );
    }

    #[test]
    fn stored_hints_keep_both_languages_and_plugin_text_stays() {
        let s = w("网络不通", "No network").said();
        assert_eq!(Said::from_stored(&s.to_stored()), s);
        assert_eq!(Said::from_stored("平台说的话"), Said::plain("平台说的话"));
        assert_eq!(Said::from_stored("{not json"), Said::plain("{not json"));
    }

    #[test]
    fn system_language() {
        assert_eq!(Lang::from_system(Some("zh-Hans-CN")), Lang::ZhHans);
        assert_eq!(Lang::from_system(Some("en-US")), Lang::En);
        assert_eq!(Lang::from_system(None), Lang::En);
    }
}
