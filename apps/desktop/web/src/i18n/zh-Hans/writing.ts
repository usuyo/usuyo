// 主题页和文稿共用的话：你写的一页、引用和原话。
export default {
  quote: "「{quote}」",
  found: {
    changedTo: "此处现在为「{now}」",
    changed: "此处的文字已与引用时不同",
    missing: "原文中已找不到这句话，这里显示的是引用时的文字",
    gone: "来源已删除，这里显示的是引用时的文字",
  },
  page: {
    editHeading: "编辑小标题",
    editParagraph: "编辑段落",
    newParagraph: "新段落",
    addParagraph: "添加段落",
    addBeforeQuote: "在此引用前添加文字",
    edit: "编辑",
    removeQuote: "从此段中移除「{quote}」",
    save: "保存",
    keys: "按 ⌘Return 保存，按 Esc 取消",
  },
} as const;
