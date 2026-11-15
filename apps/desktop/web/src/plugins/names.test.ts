import { expect, it } from "vitest";
import type { PluginsView } from "../api/endpoints";
import { method } from "../mock/platforms";
import { localized, type Names } from "./names";

// 目录里的名字（随语言）换上，目录里没有的照清单原样。
const EN: Names = {
  platform: (id, fallback) => (id === "bilibili" ? "Bilibili" : fallback),
  plugin: (id, fallback) => (id === "usuyo-fetch/bilibili" ? "Bilibili" : fallback),
  method: (pluginId, methodId, fallback) => (pluginId === "usuyo-fetch/bilibili" && methodId === "view" ? "Public API" : fallback),
};

it("puts the catalog's names in the interface language on installed plugins, keeping the manifest's where the catalog has none", () => {
  const view: PluginsView = {
    failureLog: "/m/logs/fetch-failures.jsonl",
    notLoaded: [],
    plugins: [
      {
        id: "usuyo-fetch/bilibili",
        name: "B站",
        version: "0.1.0",
        platforms: [{ id: "bilibili", name: "B站" }],
        access: "public-web",
        login: "none",
        canList: false,
        sources: [],
        sites: [],
        visitors: [],
        settings: [],
        pacing: [],
        pauses: [],
        methods: [
          method({ id: "view", name: "公开接口", access: "public-web", account: { kind: "none" }, gets: { fetch: ["meta"] } }),
          method({ id: "other", name: "别的", access: "public-web", account: { kind: "none" }, gets: { fetch: ["meta"] } }),
        ],
      },
    ],
  };
  const out = localized(view, EN).plugins[0];
  expect(out?.name).toBe("Bilibili");
  expect(out?.platforms).toEqual([{ id: "bilibili", name: "Bilibili" }]);
  expect(out?.methods.map((m) => m.name)).toEqual(["Public API", "别的"]);
});
