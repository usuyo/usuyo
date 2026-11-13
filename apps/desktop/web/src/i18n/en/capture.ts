import type zh from "../zh-Hans/capture";
import { mirror } from "../messages";

export default mirror<typeof zh>()({
  dialog: "Add Link",
  linkLabel: "Share link",
  placeholder: "Paste a share link and press Return",
  added: "Added",
  seen: "Already in your library; new snapshot saved",
  line: "{platform} · {title}",
  lineSeen: "{platform} · {title} · Added {day}",
  openAnytime: "Open this window anytime",
  keyReturn: "Return",
  add: "Add",
  seenNote: "Saved content is unchanged. The new snapshot is saved separately.",
  openItem: "Open Item",
  addAnother: "Add Another Link",
  shortcut: "Press <keys>⌥ Space</keys> to open this window",
});
