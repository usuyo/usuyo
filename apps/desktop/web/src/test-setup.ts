import { configure } from "@testing-library/react";

// 有的测试渲染整个应用；机器忙的时候第一次渲染超过过默认的 1 秒。只放宽等的上限，测得过的测试不会变慢。
configure({ asyncUtilTimeout: 5000 });
