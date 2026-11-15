import type zh from "../zh-Hans/frame";
import { mirror } from "../messages";

export default mirror<typeof zh>()({
  nav: {
    import: "Import",
    trash: "{trash}",
    plugins: "Fetch Plugins",
    phone: "Phone Connection",
    settings: "Settings",
  },
  theme: {
    system: "Auto",
    dark: "Dark",
    light: "Light",
    switch: "Appearance: {current}. Click to switch to {next}.",
  },
  later: "This page isn't available yet.",
  take: {
    label: "Add Link",
    placeholder: "Paste a link to add it",
    note: "You can also drag links onto the map.",
    adding: "Adding…",
    fetching: "Added. Fetching content…",
    fetched: "Added with content",
    partly: "Added. Some content wasn't fetched.",
    failed: "Link added. Content wasn't fetched.",
    snapshot: "Already saved. Added a new snapshot.",
    show: "Show",
    why: "See Why",
    linksOnly: "Only links can be added.",
  },
});
