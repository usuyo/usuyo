// 左栏、还没提供的页面、左上角添加链接的那一栏。
export default {
  nav: {
    import: "导入",
    trash: "{trash}",
    plugins: "获取插件",
    phone: "手机连接",
    settings: "设置",
  },
  theme: {
    system: "自动",
    dark: "深色",
    light: "浅色",
    switch: "外观：{current}。点按切换为{next}。",
  },
  later: "此页面尚未提供。",
  take: {
    label: "添加链接",
    placeholder: "粘贴链接以添加",
    note: "也可以将链接拖到地图上。",
    adding: "正在添加…",
    fetching: "已添加，正在获取内容…",
    fetched: "已添加并获取内容",
    partly: "已添加，部分内容未获取到",
    failed: "已添加链接，未获取到内容",
    snapshot: "已保存过，已添加新的快照",
    show: "查看",
    why: "查看原因",
    linksOnly: "只能添加链接。",
  },
} as const;
