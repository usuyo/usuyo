// 获取插件的标签和停着的原因，换成平常的话。只是给你挑的信息，不拦什么。
import type { Access, FetchMethod, LoginNeed, MethodAccount, MethodCall, OutputKind, PluginPause, TransformJob, VisitorIdentity } from "../api/endpoints";

/** 转换插件做的几样，写在插件卡片上，条目页的「再转」里也用。 */
export const JOB_TEXT: Record<TransformJob, string> = {
  play: "合成能播的视频",
  keyframes: "取关键帧",
  asr: "口播转成文字",
  ocr: "认图里的字",
  frames: "认画面上的字",
};

export const ACCESS_TEXT: Record<Access, string> = {
  "official-export": "读取官方导出",
  "public-web": "不登录，读取公开页面",
  "vision-ui": "读取屏幕并模拟点按",
  "browser-session": "在已登录的网页中读取",
  "session-api": "使用登录状态调用平台接口",
  "private-api": "调用平台未公开的接口",
  "client-hook": "读取或修改平台客户端",
};

/** 每一层是什么意思，一句话。只是给你挑的信息。 */
export const TIER_TEXT: Record<Access, string> = {
  "official-export": "读取平台提供的官方导出或你导出的文件，不另外向平台发送请求。",
  "public-web": "以未登录访客的身份读取公开页面，不使用你的账号。",
  "vision-ui": "只读取屏幕上显示的内容，并像你一样点按；平台看到的是你在使用 App。",
  "browser-session": "在你已登录的网页中读取页面加载的内容，账号不离开该浏览器。",
  "session-api": "使用你的登录状态直接调用平台接口，平台可以看到是你的账号在请求。",
  "private-api": "模拟平台客户端调用未公开的接口。平台通常不允许这种方式，因此默认关闭。",
  "client-hook": "读取或修改平台客户端及其在这台电脑上的数据。这是最高的层级，默认关闭。",
};

/** 「会用到你的 X 账号」：站点名由内核给，插件声明的站点不在 `/sites` 里时写主机名。 */
export function loginText(need: LoginNeed, siteNames: string[]): string {
  const names = siteNames.join("、");
  switch (need) {
    case "none":
      return "不用账号";
    case "optional":
      return names ? `登录了${names}能多取一些` : "登录了能多取一些";
    default:
      return names ? `要登录你的${names}账号` : "要登录";
  }
}

/** 今天的只写时分，别的日子带上日期；十分钟以内的带上秒（照节奏几秒一条时有用）。 */
export function timeText(iso: string, now = new Date()): string {
  const d = new Date(iso);
  const two = (n: number) => n.toString().padStart(2, "0");
  const soon = Math.abs(d.getTime() - now.getTime()) < 10 * 60_000;
  const hm = `${two(d.getHours())}:${two(d.getMinutes())}${soon ? `:${two(d.getSeconds())}` : ""}`;
  const sameDay = d.toDateString() === now.toDateString();
  return sameDay ? hm : `${d.getMonth() + 1} 月 ${d.getDate()} 日 ${hm}`;
}

/**
 * 页面要访客身份时怎么说。`state` 是内核存着的那份的状态：没有就写怎么取，存着的就说换一份；
 * 页面手上没有这个状态时写 `要访客身份`。
 */
export function visitorText(state?: VisitorIdentity["state"]): string {
  if (state === undefined) return "需要访客身份";
  return state === "missing" ? "需要访客身份：在「获取插件」页面中获取访客身份" : "访客身份已失效：更换后重试";
}

const PAUSE_CODE: Record<string, string> = {
  "rate-limited": "平台提示请求过于频繁",
  blocked: "平台阻止了此插件",
  "login-required": "需要重新登录",
  "visitor-required": visitorText(),
  "upstream-changed": "平台已改版，插件无法读取",
};

/** 停着的那一行：为什么、到什么时候。平台挡住的停一小时就自己再试。`platforms` 用来把范围写成平台名。 */
export function pauseText(p: PluginPause, platforms: { id: string; name: string }[], now = new Date()): string {
  if (p.byYou) return "已暂停：点按「立即重试」前不会使用此插件";
  const why = p.hint ?? (p.code ? (PAUSE_CODE[p.code] ?? p.code) : "已暂停");
  // 只有一个平台的插件不用再说是哪个。
  const where = p.scope === "*" || platforms.length <= 1 ? "" : `（${platforms.find((x) => x.id === p.scope)?.name ?? p.scope}）`;
  return p.until ? `${why}${where} · ${timeText(p.until, now)} 自动重试` : `${why}${where} · 需要处理`;
}

/** 一种取法的账号用在哪，一句话。名字用内核给的 `where`（取自清单），没有时用 `names`（站点给人看的名字，没有站点时是平台名）。 */
export function accountText(account: MethodAccount, names: string[]): string {
  const who = account.where ?? names.join("、");
  const text: Record<MethodAccount["kind"], string> = {
    none: "不需要账号",
    visitor: `不需要账号，使用${who || "站点"}的访客身份`,
    "own-browser": `账号只在你的浏览器中使用：脚本在已登录的${who}页面中读取，并把链接发送到 usuyo`,
    "own-phone": `账号只在手机上的${who} App 中使用`,
    "own-app": `账号只在这台电脑上的${who}客户端中使用`,
    developer: `使用你申请的${who}开发者凭据，不登录你的账号`,
    "in-usuyo": `需要在 usuyo 中登录${who}账号`,
  };
  return text[account.kind];
}

/** 取一条时拿得到的几样，换成平常的话。 */
export const OUTPUT_TEXT: Record<OutputKind, string> = {
  text: "正文",
  html: "网页原文",
  images: "图片",
  "media:video": "视频",
  "media:audio": "音频",
  subtitle: "字幕",
  meta: "标题和作者",
  comments: "评论",
};

/** 取得到什么：「取一条：正文、图、视频」「列出：书签、喜欢」「从一条链接列出」「链接交进来」。 */
export function getsText(gets: FetchMethod["gets"]): string[] {
  const list = gets.list ?? [];
  return [
    ...(gets.fetch && gets.fetch.length > 0 ? [`单条：${gets.fetch.map((k) => OUTPUT_TEXT[k]).join("、")}`] : []),
    ...(list.length > 0 ? [`列出：${list.join("、")}`] : gets.link !== undefined ? [] : gets.list ? ["列出已保存的内容"] : []),
    ...(gets.link !== undefined ? [`从链接列出：${gets.link}`] : []),
    ...(gets.delivers === "links" ? ["由浏览器发送链接"] : []),
  ];
}

/** 能列出一批：有 `list`（写死的来源可以是空的），或从一条链接开始。 */
export function canList(m: FetchMethod): boolean {
  return m.gets.list !== undefined || m.gets.link !== undefined;
}

/**
 * 这种取法现在怎样，一句话：用不上的照内核的原因一句句说（停着的也是一条原因，写到什么时候）；用得上的说用得上。
 * 用得上又有暂停的（通用网页按网站停的）另由 `pauseText` 说，这里回 null。没做好的行上有标，也回 null。
 */
export function methodStatus(m: FetchMethod): string | null {
  if (!m.built) return null;
  if (!m.usable) return m.blockers.map((b) => b.hint).join("；");
  return m.pauses.length > 0 ? null : "可用";
}

/** 停着已经写成一条原因（`paused`）的，暂停不再另写一行。 */
export function pausedAsBlocker(m: FetchMethod): boolean {
  return m.blockers.some((b) => b.code === "paused");
}

/** 上一次调用：什么时候，没出错，或那一句错。 */
export function lastCallText(c: MethodCall, now = new Date()): string {
  if (c.ok) return `上次使用：${timeText(c.at, now)}，成功`;
  const why = c.hint ?? (c.code ? (PAUSE_CODE[c.code] ?? c.code) : "未成功");
  return `上次使用：${timeText(c.at, now)}，失败：${why}`;
}

// —— 平台页（一个平台一页）上的话 ——

export const LOGIN_STATE: Record<"missing" | "present" | "expired", string> = {
  missing: "未登录",
  present: "已登录",
  expired: "登录已过期",
};

export const VISITOR_STATE: Record<"missing" | "present" | "expired", string> = {
  missing: "尚未获取",
  present: "已获取",
  expired: "已过期",
};

export const PAGE = {
  title: "获取插件",
  intro: "每个平台一页：添加链接使用的获取方式、导入已保存的内容、账号使用情况和请求频率。usuyo 本身不包含任何平台的插件。",
  navLabel: "平台",
  navPlatforms: "平台",
  navText: "转写与识别",
  textEntry: "转写与识别",
  textEntryNav: "插件和模型",
  textEntryLine: "口播、图片和画面中的文字",
  navNotLoaded: "已安装但未载入",
  navEmpty: "未安装",
  notFound: "已安装的插件中没有此平台。",
  nothingTitle: "尚未安装获取插件。",
  nothingBody: "usuyo 本身不包含平台插件。目前只能在开发环境中安装：在 usuyo 仓库中运行以下命令，它会编译旁边 usuyo-fetch 中的插件并安装到这台电脑；重新启动 usuyo 后即可在此看到。",
  // 左边那一行
  navHeld: "已暂停",
  navPausedUntil: (time: string) => `部分获取方式已暂停，${time} 重试`,
  navPaused: "部分获取方式已暂停",
  navSavedOnly: "仅导入已保存的内容",
  navMixed: "单独设置",
  navOnly: (name: string, usable: boolean) => (usable ? `仅使用${name}` : `仅使用${name}，当前不可用`),
  navAuto: (name: string) => `自动 · ${name}`,
  navAutoNone: "自动 · 无可用方式",
  header: (plugins: string) => `由${plugins}插件获取。不需要账号就能获取的内容，不会使用账号。`,

  // 添加链接
  linkTitle: "添加链接",
  linkSub: "粘贴这个平台的链接时使用的获取方式。",
  linkNoneTitle: "这个平台没有获取单条链接的方式",
  linkNoneSub: "已安装的插件只能导入已保存的内容。粘贴的链接只保存链接本身。",
  wayTitle: "获取方式",
  waySub: "更改后立即生效。",
  wayMixedSub: "当前开关是单独设置的，与下列选项都不对应。选择一项后按它重新设置。",
  wayMixed: "单独设置",
  auto: "自动（推荐）",
  autoSub: "按内核的顺序：不需要账号、层级低的优先，缺少的内容由下一种补充",
  autoLone: (name: string) => `目前只提供${name}`,
  notBuilt: "尚未提供",
  alsoMoves: (name: string, others: string) => `${name}的开关同时作用于${others}。在这里更改获取方式或选回自动，这些平台上的设置也会随之改变。`,
  autoIntro: "按以下顺序获取：先用第一种，获取失败或缺少内容时再用下一种。默认关闭的层级不使用。",
  mixedIntro: "已开启的获取方式按以下顺序使用：",
  noneOn: "没有开启的获取方式。粘贴的链接只保存链接本身。",
  offList: "已关闭：",
  usable: "可用",
  offByYou: "已关闭",
  offByTier: "此层级默认关闭",
  termGets: "获取内容",
  termAccount: "账号",
  termTier: "层级",
  termNeeds: "需要",
  termRisk: "已记录",
  termLast: "上次使用",
  getsUnknown: "插件未说明",
  neverUsed: "尚未使用",
  partTitle: (what: string) => `补充：${what}`,
  partSub: (name: string, account: string) => `${name} · ${account} · 仅在前面的获取方式缺少这些内容时使用`,
  partSwitch: (name: string) => `补充：${name}`,
  switchOn: "开启",
  switchOff: "关闭",

  // 导入已保存的内容
  savedTitle: "导入已保存的内容",
  savedSub: "列出你在这个平台上收藏或点赞的内容，确认后再导入。",
  savedNoneTitle: "这个平台没有导入方式",
  savedNoneSub: "仍可逐条添加链接。",
  importWay: "导入方式",
  kindFile: "读取导出的文件",
  kindScript: "由浏览器脚本发送",
  kindPhone: "通过手机",
  kindLinkList: "从链接列出",
  kindList: "列出已保存的内容",
  steps: (name: string) => `${name}的步骤`,
  goImport: "前往导入",
  stepGetFile: "获取导出文件",
  stepPickFile: "交给 usuyo 读取",
  pickFile: "选择文件",
  stepWhat: "将执行的操作",
  stepPhone: "连接手机",
  stepReady: "本机准备",
  stepLink: "输入链接",
  stepList: (lists: string) => (lists ? `在导入页面选择列表（${lists}）` : "在导入页面选择列表"),
  offByYouSaved: "你已关闭此获取方式，开启后才能使用。",
  offByTierSaved: (tier: string) => `此获取方式默认关闭：${tier}`,
  offSwitch: (name: string) => `${name}：已关闭`,
  risk: (text: string, source: string) => `已记录：${text}（${source}）`,
  readsAlso: (lists: string) => `可读取：${lists}`,
  stepManager: "安装脚本管理器",
  stepManagerBody: "在常用的浏览器中安装 Tampermonkey 或 Violentmonkey 等扩展。脚本在你的浏览器中运行，账号不经过 usuyo。",
  stepScript: "安装脚本",
  stepScriptBody: "在脚本管理器中新建脚本并粘贴全部内容，或将此文件拖入脚本管理器。",
  noScript: "插件未附带脚本文件。",
  scriptMissing: (path: string) => `插件文件夹中没有此脚本文件：${path}。重新安装插件后再试。`,
  reveal: "在访达中显示",
  copyScript: "拷贝脚本",
  copiedScript: "已拷贝脚本",
  copyScriptFailed: "无法拷贝。可在访达中打开文件后拷贝。",
  stepToken: "生成发送令牌",
  tokenBody: "脚本首次发送时需要此令牌。令牌只能向 usuyo 发送内容，无法读取资料库。",
  tokensIssued: (names: string) => `已生成：${names}。已拷贝过的令牌可以继续使用。`,
  tokensNone: "尚未生成。",
  issueToken: "生成令牌",
  tokenName: (plugin: string) => `${plugin}脚本`,
  tokenOnce: "令牌只显示这一次。拷贝到脚本面板中；丢失后可在导入页面撤销并重新生成。",
  copyToken: "拷贝令牌",
  copied: "已拷贝",
  copyFailed: "无法拷贝。可手动选择后拷贝。",
  done: "完成",
  stepPage: "打开你的页面",
  pageBody: (platform: string, pages: string) => `在浏览器中登录${platform}，打开你的${pages}页面并向下滚动。脚本只读取页面已加载的列表，不另外发送请求。`,
  pageFallback: (platform: string) => `在浏览器中登录${platform}，打开收藏或点赞的页面并向下滚动。`,
  // 脚本面板上的按钮名照脚本自己写的引用。
  stepSend: "发送到 usuyo",
  sendBefore: "在脚本面板中点按「交给 usuyo」。发送的内容会出现在",
  sendLink: "导入",
  sendAfter: "页面的记录中，确认后才会导入。没有记录到内容时，点按面板上的「复制诊断」：它只拷贝回复的结构，不包含内容。",

  // 需要的项目
  needsLabel: (name: string) => `${name}需要的项目`,
  needLogin: (name: string, state: string) => `在 usuyo 中登录${name}账号：${state}`,
  needVisitor: (name: string, state: string) => `${name}访客身份：${state}`,
  needPhone: (where: string) => `需要一台已登录${where} App 的手机`,
  goPhone: "前往「手机」页面",
  recheck: "重新检查",
  retryNow: "立即重试",
  needSecret: (label: string) => `填写${label}`,
  needNothing: "无需额外准备。",
  needDone: "已完成：",
  needTodo: "未完成：",
  secretFilled: "已填写，输入新值以替换",
  secretSave: "存储到钥匙串",

  // 账号与访客身份
  accountsTitle: "账号与访客身份",
  accountsSub: "登录在 usuyo 为每个站点单独打开的窗口中进行，与你平时使用的浏览器分开。",
  accountsNone: "这个平台的获取方式不需要在 usuyo 中登录，也不需要访客身份",
  siteTitle: (name: string) => `${name}账号`,
  siteSaved: (day: string) => ` · ${day}保存`,
  users: (names: string) => ` · 使用它的获取方式：${names}`,
  siteNoLoginUrl: "插件未提供登录页面，无法打开登录窗口。",
  login: "登录",
  relogin: "重新登录",
  logout: "退出登录",
  loginOpened: "登录窗口已打开。登录完成后会自动关闭。",
  loginAppOnly: "登录窗口只能在 usuyo 应用中打开，浏览器标签页中不可用。",
  loginFailed: "无法打开登录窗口",
  loginClearFailed: "无法清除这个站点的网页数据",
  visitorTitle: (name: string) => `${name}访客身份`,
  visitorBody: (name: string) => `访客身份不是你的账号：usuyo 打开一次${name}首页（不登录），保存页面提供给访客的身份。`,
  takeVisitor: "获取访客身份",
  replaceVisitor: "更换",
  removeVisitor: "移除",
  visitorOpened: "窗口已打开。获取到访客身份后会自动关闭。",
  visitorAppOnly: "此窗口只能在 usuyo 应用中打开，浏览器标签页中不可用。",
  visitorFailed: "无法打开窗口",
  visitorClearFailed: "无法清除访客窗口的网页数据",

  // 请求频率
  pacingTitle: "请求频率",
  pacingSub: "同一平台的获取方式共用一个请求频率。手动添加的单条链接也按此间隔排队。",
  pacingRow: "当前频率",
  pacingNone: "插件未设定请求频率，使用 usuyo 的默认间隔。",
  pacingBatch: (sec: number, daily?: number) => `每两次请求至少间隔 ${sec} 秒${daily ? `，每天最多导入 ${daily} 次` : ""}。`,
  pacingNoBatch: (sec: number) => `每两次请求至少间隔 ${sec} 秒。这个平台不支持批量导入，只能逐条获取。`,
  pause: "暂停",

  // 关于
  aboutTitle: "关于",
  upstream: (list: string) => `使用的程序：${list}`,
  noUpstream: "不调用其他程序。",
  secretState: (filled: boolean) => (filled ? "已填写" : "未填写"),
  failureLog: "失败日志",
  failureLogSub: "获取失败时，插件和平台的原始回复记录在此，只保存在这台电脑上。",

  // 条目页
  itemPlatformLink: (platform: string) => `查看${platform}的获取方式`,
};
