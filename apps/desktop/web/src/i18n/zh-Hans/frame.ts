// 左栏、还没提供的页面、左上角添加链接的那一栏。
export default {
  nav: {
    drafts: "文稿",
    follow: "订阅",
    import: "导入",
    export: "导出",
    changes: "更改记录",
    trash: "{trash}",
    plugins: "获取插件",
    phone: "手机",
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
    added: "已添加",
    snapshot: "此链接已保存过，已添加新的快照",
  },
} as const;
