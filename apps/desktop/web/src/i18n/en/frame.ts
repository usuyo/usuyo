import type zh from "../zh-Hans/frame";
import { mirror } from "../messages";

export default mirror<typeof zh>()({
  nav: {
    drafts: "Drafts",
    follow: "Subscriptions",
    import: "Import",
    export: "Export",
    changes: "Change History",
    trash: "{trash}",
    plugins: "Fetch Plugins",
    phone: "Phone",
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
    added: "Added",
    snapshot: "Already saved. Added a new snapshot.",
  },
});
