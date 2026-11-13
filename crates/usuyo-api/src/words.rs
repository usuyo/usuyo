//! 本机接口自己给界面的话：每一句的简体中文和英文并排写（SPEC §0.10）。写出时照内核设置的语言挑（`problem.rs`）。

use usuyo_core::Words;

pub(crate) const UPLOAD_TOO_SLOW: Words = Words::new(
    "发送文件用时过长，未能完成。",
    "Sending the file took too long and didn't finish.",
);
pub(crate) const NAME_THE_METHOD: Words = Words::new(
    "请求中缺少获取方式。",
    "The request doesn't say which fetch method to use.",
);
pub(crate) const BAD_LINK: Words = Words::new(
    "无法使用这个链接：链接需要以 http 或 https 开头、包含网站，且不带用户名和密码。",
    "This link can't be used. It needs to start with http or https, include a website, and contain no user name or password.",
);
pub(crate) const BAD_LINK_FIRST: Words = Words::new(
    "无法使用这个链接：链接需要以 http 或 https 开头、包含网站，且不带用户名和密码。只检查了第一个链接。",
    "This link can't be used. It needs to start with http or https, include a website, and contain no user name or password. Only the first link was checked.",
);
pub(crate) const BAD_IMPORT_REQUEST: Words = Words::new(
    "开始导入的请求无效：需要插件、获取方式和来源。",
    "The request to start an import isn't valid. It needs a plugin, a fetch method, and a source.",
);
pub(crate) const BAD_PREVIEW_QUERY: Words = Words::new(
    "翻看预览的参数无效。",
    "The parameters for browsing the preview aren't valid.",
);
pub(crate) const BAD_CONFIRM_REQUEST: Words = Words::new(
    "确认的请求无效：需要说明是否导入新条目和{trash}中的条目。",
    "The confirmation isn't valid. It needs to say whether to import the new items and the items in the {trash}.",
);
pub(crate) const ONLY_EXTENSIONS_HAND_OVER: Words = Words::new(
    "只有浏览器扩展和脚本的令牌可以发送已获取的内容。",
    "Only browser extension and script tokens can send content they've already fetched.",
);
pub(crate) const BAD_FILE_QUERY: Words = Words::new(
    "请求中缺少读取方式或文件名。",
    "The request needs a method to read the file and the file's name.",
);
pub(crate) const FILE_INCOMPLETE: Words =
    Words::new("文件未完整发送。", "The file wasn't sent completely.");
pub(crate) const FILE_TOO_LARGE: Words = Words::new(
    "文件过大。最大 {max} MiB。",
    "The file is too large. The maximum is {max} MiB.",
);
pub(crate) const FILE_EMPTY: Words = Words::new("文件是空的。", "The file is empty.");
pub(crate) const BAD_LINK_AT: Words =
    Words::new("第 {n} 个链接无法使用。", "Link {n} can't be used.");
