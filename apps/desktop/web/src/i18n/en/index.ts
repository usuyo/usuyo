// 一种语言的全部话，按页面分文件；和简体中文的键、变量一样（`messages.ts`）。
import type zh from "../zh-Hans";
import { mirror } from "../messages";
import common from "./common";
import frame from "./frame";
import home from "./home";
import item from "./item";
import topic from "./topic";
import drafts from "./drafts";
import writing from "./writing";
import follow from "./follow";
import changes from "./changes";
import exportPage from "./export";
import capture from "./capture";
import imports from "./imports";
import plugins from "./plugins";
import phone from "./phone";
import settings from "./settings";
import trash from "./trash";
import shell from "./shell";

export default mirror<typeof zh>()({ common, frame, home, item, topic, drafts, writing, follow, changes, export: exportPage, capture, imports, plugins, phone, settings, trash, shell });
