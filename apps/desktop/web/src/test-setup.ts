import { configure } from "@testing-library/react";
import { beforeEach } from "vitest";
import { setLanguageChoice } from "./i18n";

// 有的测试渲染整个应用；机器忙的时候第一次渲染超过过默认的 1 秒。只放宽等的上限，测得过的测试不会变慢。
configure({ asyncUtilTimeout: 5000 });

// jsdom 的系统语言是 en-US；页面测试照简体中文写，每个测试从简体中文开始，换语言的测试自己换。
beforeEach(() => setLanguageChoice("zh-Hans"));
