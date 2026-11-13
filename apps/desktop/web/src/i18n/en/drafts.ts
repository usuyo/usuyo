import type zh from "../zh-Hans/drafts";
import { mirror } from "../messages";

export default mirror<typeof zh>()({
  title: "Drafts",
  intro: "Write your own article, post, or script from passages you've quoted. Every quote keeps its source; click it to go back to that moment or image.",
  new: "New Draft",
  notServed: "This version of usuyo doesn't include drafts yet.",
  empty: "No drafts yet. Create a draft and quote passages from topic pages and search results.",
  kind: {
    article: "Article",
    post: "Post",
    script: "Script",
    misc: "Other",
  },
  editedToday: "Edited today",
  editedOn: "Edited {day}",
  page: {
    all: "All Drafts",
    titleLabel: "Title",
    kindLabel: "Type",
    topicsUsed: "Topics quoted: <list></list>",
    empty: "This draft is empty. Start writing, or quote a passage from the right.",
    sources: "Sources",
    sourcesSub: "Added at the end when you export or copy the draft. If a source is deleted, the quote is kept.",
    sourceGone: "Source deleted",
  },
  picker: {
    title: "Quote a Passage",
    sub: "Quotes are added at the end, with their sources.",
    from: "From",
    search: "Search",
    topic: "Quotes in topic pages",
    cite: "Quote “{text}”",
    citeButton: "Quote",
    searchLabel: "Search passages",
    searchPlaceholder: "Search transcripts, text in images, post text",
    noQuotable: "No passages to quote.",
    whichTopic: "Topic",
    topicNotServed: "This version of usuyo doesn't include topic pages yet. Use Search instead.",
    topicEmpty: "This topic page has no quotes yet.",
  },
});
