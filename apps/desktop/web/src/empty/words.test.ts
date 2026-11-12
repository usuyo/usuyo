import { expect, it } from "vitest";
import type { FetchMethod, FetchPlugin } from "../api/endpoints";
import { method as mockMethod } from "../mock/platforms";
import { waysOf } from "./words";

const method = (m: Partial<FetchMethod> & Pick<FetchMethod, "id" | "name">): FetchMethod =>
  mockMethod({ access: "public-web", account: { kind: "none" }, gets: {}, ...m });

const plugin = (methods: FetchMethod[]): FetchPlugin => ({
  id: "someone/site",
  name: "某站",
  version: "0.1.0",
  platforms: [{ id: "site", name: "某站" }],
  access: "public-web",
  login: "none",
  canList: true,
  sources: [],
  sites: [],
  visitors: [],
  settings: [],
  pacing: [],
  pauses: [],
  methods,
});

it("groups the methods that bring a batch in by plugin, in the plugin's own order, with their labels", () => {
  const groups = waysOf(
    [
      plugin([
        method({ id: "page", name: "单条页面", gets: { fetch: ["text"] } }),
        method({ id: "script", name: "浏览器脚本", access: "browser-session", account: { kind: "own-browser", where: "某站" }, gets: { list: ["收藏"], delivers: "links" } }),
        method({ id: "login", name: "收藏夹", access: "session-api", account: { kind: "in-usuyo", sites: ["www.site.test"] }, gets: { list: ["收藏夹"] }, enabled: false }),
        method({ id: "archive", name: "官方归档", access: "official-export", gets: { list: ["收藏"] } }),
        method({ id: "phone", name: "手机", access: "vision-ui", account: { kind: "own-phone", where: "某站" }, gets: { list: ["赞过"] }, built: false }),
      ]),
      plugin([method({ id: "page", name: "单条页面", gets: { fetch: ["text"] } })]),
    ],
    new Set(["someone/site#archive"]),
    (site) => (site === "www.site.test" ? "某站" : site),
  );
  // 只取单条的插件不列；读文件的那一种在文件那一段，不在这里。
  expect(groups).toHaveLength(1);
  const [g] = groups;
  expect(g?.ways.map((w) => w.name)).toEqual(["浏览器脚本", "收藏夹"]);
  expect(g?.ways[0]).toMatchObject({ gets: ["列出：收藏", "链接交进来"], access: "在登录着的网页里读", to: "/import" });
  // 关着的取法去获取插件页打开，账号写成站点给人看的名字。
  expect(g?.ways[1]).toMatchObject({ to: "/plugins", account: "要在 usuyo 里登录你的某站账号" });
  expect(g?.later).toEqual(["手机"]);
});
