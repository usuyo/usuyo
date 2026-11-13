import type zh from "../zh-Hans/writing";
import { mirror } from "../messages";

export default mirror<typeof zh>()({
  quote: "“{quote}”",
  found: {
    changedTo: "This passage now reads “{now}”",
    changed: "The text here has changed since it was quoted",
    missing: "This sentence is no longer in the original; this is the text as quoted",
    gone: "The source was deleted; this is the text as quoted",
  },
  page: {
    editHeading: "Edit heading",
    editParagraph: "Edit paragraph",
    newParagraph: "New paragraph",
    addParagraph: "Add Paragraph",
    addBeforeQuote: "Add Text Before This Quote",
    edit: "Edit",
    removeQuote: "Remove “{quote}” from this paragraph",
    save: "Save",
    keys: "Press ⌘Return to save or Esc to cancel",
  },
});
