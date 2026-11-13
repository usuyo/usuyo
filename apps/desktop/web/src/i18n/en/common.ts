import type zh from "../zh-Hans/common";
import { mirror } from "../messages";

export default mirror<typeof zh>()({
  os: {
    mac: { trash: "Trash", fileManager: "Finder" },
    windows: { trash: "Recycle Bin", fileManager: "File Explorer" },
    linux: { trash: "Trash", fileManager: "file manager" },
  },
  bytes: { one: "{count} byte", other: "{count} bytes" },
  clauseSeparator: "; ",
  cancel: "Cancel",
  done: "Done",
  close: "Close",
  copy: "Copy",
  copied: "Copied",
  copyFailed: "Couldn't copy. Select the text and copy it manually.",
  retry: "Try Again",
  download: "Download",
  delete: "Delete",
  today: "Today",
  yesterday: "Yesterday",
  nav: { main: "Main", home: "Home" },
  map: "Map",
  backToMap: "Back to Map",
  followedTopics: "Followed Topics",
  showLess: "Show Less",
  tag: { autoSelected: "Auto-selected", generated: "Generated" },
  reasonUnknown: "Unknown reason",
  error: {
    daemonDown: "The usuyo service isn't running. Reopen usuyo and try again.",
    notServed: "This version of usuyo doesn't include this feature yet.",
    status: "The usuyo service returned an error ({status}).",
    loadFailed: "Couldn't load.",
  },
});
