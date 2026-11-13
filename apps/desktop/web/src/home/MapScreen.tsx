import { getRouteApi, Link } from "@tanstack/react-router";
import { Search, X } from "lucide-react";
import { useEffect, useMemo, useRef, useState, type CSSProperties, type DragEvent, type PointerEvent } from "react";
import { Button as AriaButton, Input, SearchField, ToggleButton, ToggleButtonGroup } from "react-aria-components";
import type { MapView, Topic } from "../api/endpoints";
import { useFollow, useSearch, useTake } from "../api/hooks";
import { Rail, RailTopic } from "../frame/Rail";
import { labelBox, type LibraryScene } from "../fog/field";
import { buildPlaces, foldRows, nameSize, textWidth, type FoldRow } from "../fog/places";
import { useFog, useSize } from "../fog/useFog";
import { useUiStore } from "../store/ui";
import { SectionLabel } from "../ui/Chip";
import { Panel } from "./Panel";
import { rich, t } from "../i18n";

const route = getRouteApi("/");
const MAP_X = 248;
const PANEL_W = 372;

type Arrangement = "topic" | "source";

/** 主页：地图在前，搜索浮在上面，右边是面板。 */
export function MapScreen({ map, topics }: { map: MapView; topics: Topic[] }) {
  const { q = "", open } = route.useSearch();
  const navigate = route.useNavigate();
  const root = useRef<HTMLDivElement>(null);
  const canvas = useRef<HTMLCanvasElement>(null);
  const size = useSize(root);
  const [arrangement, setArrangement] = useState<Arrangement>("topic");
  const [hover, setHover] = useState<string | null>(null);
  const [hoverTopic, setHoverTopic] = useState<string | null>(null);
  const [rowHover, setRowHover] = useState<string | null>(null);
  const [railHover, setRailHover] = useState<string | null>(null);
  const [lampN, setLampN] = useState<number | null>(null);
  const landing = useUiStore((s) => s.landing);
  const clearLanding = useUiStore((s) => s.clearLanding);
  const markLanding = useUiStore((s) => s.markLanding);
  const search = useSearch(q);
  const follow = useFollow();
  const take = useTake();

  const followed = useMemo(() => new Set(topics.filter((x) => x.followed).map((x) => x.id)), [topics]);
  const names = useMemo(() => new Map(topics.map((x) => [x.id, x.name])), [topics]);
  const { width, height } = size;
  const built = useMemo(
    () => (width === 0 ? null : buildPlaces(map, names, { width, height, mapX: MAP_X })),
    [map, names, width, height],
  );
  const searching = q.trim() !== "";
  const hits = useMemo(
    () => new Set(searching && search.data ? search.data.map((h) => h.itemId) : []),
    [searching, search.data],
  );
  const panelOpen = open !== undefined || searching;
  const panel = useMemo(
    () => (panelOpen ? { x0: width - PANEL_W - 16, y0: 16, x1: width - 16, y1: height - 16 } : null),
    [panelOpen, width, height],
  );
  const landingSet = useMemo(() => new Set(landing), [landing]);

  const scene = useMemo<LibraryScene | null>(
    () =>
      built && {
        kind: "library",
        frame: { width, height, mapX: MAP_X },
        regions: built.regions,
        sources: built.sources,
        places: built.places,
        arrangement,
        followed,
        hits,
        searching,
        selected: open ?? null,
        rowHover,
        railHover,
        panel,
        landing: landingSet,
      },
    [built, width, height, arrangement, followed, hits, searching, open, rowHover, railHover, panel, landingSet],
  );
  const field = useFog(canvas, scene, { hover: setHover, hoverTopic: setHoverTopic, lampCount: setLampN });

  // 刚收下的一条，名字在它旁边留三秒，之后和别的点一样。
  const landedHere = useMemo(
    () => (built ? built.places.filter((p) => landingSet.has(p.id)) : []),
    [built, landingSet],
  );
  useEffect(() => {
    if (landedHere.length === 0) return undefined;
    const ids = landedHere.map((p) => p.id);
    const id = setTimeout(() => {
      for (const x of ids) clearLanding(x);
    }, 3200);
    return () => clearTimeout(id);
  }, [landedHere, clearLanding]);

  const select = (id: string) => void navigate({ search: (s) => ({ ...s, open: id }) });
  const closeItem = () => void navigate({ search: () => (q === "" ? {} : { q }) });
  const setQuery = (value: string) =>
    void navigate({ search: () => (value === "" ? {} : { q: value }), replace: true });

  const folded = useMemo(() => {
    const rows: FoldRow[] = [];
    if (!built || arrangement !== "topic") return rows;
    for (const r of built.regions) if (followed.has(r.id)) rows.push(...foldRows(r, built.places, { width, height, mapX: MAP_X }));
    return rows;
  }, [built, arrangement, followed, width, height]);
  const foldedIds = useMemo(() => new Set(folded.map((r) => r.place.id)), [folded]);

  function onPointerMove(e: PointerEvent<HTMLDivElement>) {
    const rc = root.current?.getBoundingClientRect();
    if (!rc) return;
    const x = e.clientX - rc.left;
    const y = e.clientY - rc.top;
    const inPanel = panel !== null && x >= panel.x0 && x <= panel.x1 && y >= panel.y0 && y <= panel.y1;
    field.current?.pointer(x < MAP_X || inPanel ? null : { x, y });
  }

  function onPointerDown(e: PointerEvent<HTMLDivElement>) {
    if (e.button !== 0 || !(e.target instanceof Element)) return;
    if (e.target.closest("button, input, a, nav, section, label, textarea")) return;
    if (hover) select(hover);
  }

  function onDrop(e: DragEvent<HTMLDivElement>) {
    e.preventDefault();
    const files = [...e.dataTransfer.files].slice(0, 8);
    const inputs: { input: string; kind: "link" | "file" }[] =
      files.length > 0
        ? files.map((f) => ({ input: f.name, kind: "file" }))
        : [{ input: e.dataTransfer.getData("text/uri-list") || e.dataTransfer.getData("text"), kind: "link" }];
    for (const body of inputs) {
      if (body.input.trim() === "") continue;
      take.mutate(body, { onSuccess: (res) => !res.seen && markLanding(res.id) });
    }
  }

  const scale = (width - MAP_X) / 1192;
  const at = (p: { tx: number; ty: number; sx: number; sy: number }) =>
    arrangement === "source" ? { x: p.sx, y: p.sy } : { x: p.tx, y: p.ty };
  const shownHover = hover ?? rowHover;
  const hovered = shownHover && !foldedIds.has(shownHover) ? built?.places.find((p) => p.id === shownHover) : undefined;
  const right = panel ? panel.x0 - 12 : width - 12;

  let hint = t("home.hint.idle");
  if (hover) hint = t("home.hint.hoverItem");
  else if (searching) hint = hits.size > 0 ? t("home.hint.found") : t("home.hint.noResults");
  else if (hoverTopic) hint = followed.has(hoverTopic) ? t("home.hint.hoverFollowed") : t("home.hint.hoverTopic");

  const railTopics = topics.filter((x) => x.followed);

  return (
    <div
      ref={root}
      className="relative h-full overflow-hidden bg-bg select-none"
      onPointerMove={onPointerMove}
      onPointerLeave={() => field.current?.pointer(null)}
      onPointerDown={onPointerDown}
      onDragOver={(e) => e.preventDefault()}
      onDrop={onDrop}
    >
      <canvas ref={canvas} aria-hidden="true" className="absolute inset-0 size-full" />

      {/* 左栏和搜索放在前面：按 Tab 先到它们，再到地图上的东西。 */}
      <Rail label={t("common.nav.home")}>
        <SectionLabel className="mt-[26px]">{t("home.arrange.title")}</SectionLabel>
        <ToggleButtonGroup
          aria-label={t("home.arrange.title")}
          selectionMode="single"
          disallowEmptySelection
          selectedKeys={[arrangement]}
          onSelectionChange={(keys) => {
            const next = [...keys][0];
            if (next === "topic" || next === "source") setArrangement(next);
          }}
          className="mt-2 flex gap-0.5 rounded-input border border-border p-[3px]"
        >
          {(
            [
              ["topic", t("home.arrange.topic")],
              ["source", t("home.arrange.source")],
            ] as const
          ).map(([id, text]) => (
            <ToggleButton
              key={id}
              id={id}
              className="press h-[38px] flex-1 cursor-pointer rounded-[9px] border-0 bg-transparent text-note font-medium text-sub outline-none data-[focus-visible]:outline-2 data-[focus-visible]:outline-accent data-[selected]:bg-chip data-[selected]:font-semibold data-[selected]:text-text"
            >
              {text}
            </ToggleButton>
          ))}
        </ToggleButtonGroup>
        <div className="relative mt-2 h-[18px] px-1 text-small text-faint">
          <div className={`absolute inset-x-1 top-0 transition-opacity duration-[180ms] ease-out ${arrangement === "topic" ? "opacity-100" : "opacity-0"}`}>
            {t("home.arrange.topicSub")}
          </div>
          <div className={`absolute inset-x-1 top-0 transition-opacity duration-[180ms] ease-out ${arrangement === "source" ? "opacity-100" : "opacity-0"}`}>
            {t("home.arrange.sourceSub")}
          </div>
        </div>
        <SectionLabel className="mt-[26px]">{t("common.followedTopics")}</SectionLabel>
        <div className="mt-1.5 flex flex-col gap-0.5">
          {railTopics.map((topic) => (
            <RailTopic
              key={topic.id}
              name={topic.name}
              active={railHover === topic.id}
              onHoverChange={(on) => setRailHover(on ? topic.id : null)}
              onPress={() => void navigate({ to: "/topics/$id", params: { id: topic.id } })}
            />
          ))}
          {railTopics.length === 0 ? (
            <div className="p-1 text-small text-faint">{t("home.followHint")}</div>
          ) : null}
        </div>
      </Rail>

      <SearchField
        value={q}
        onChange={setQuery}
        aria-label={t("home.search")}
        className="absolute top-4 z-10 flex h-field w-[440px] items-center gap-2.5 rounded-input border border-border bg-hud px-3.5 text-sub"
        style={{ left: MAP_X + (width - MAP_X) / 2 - 220 }}
      >
        <Search size={16} aria-hidden="true" className="shrink-0" />
        <Input
          placeholder={t("home.searchPlaceholder")}
          className="h-ctl min-w-0 flex-1 border-0 bg-transparent p-0 text-body text-text outline-none [&::-webkit-search-cancel-button]:hidden"
        />
        {q ? (
          <AriaButton
            aria-label={t("home.clearSearch")}
            className="press -mr-1.5 flex size-ctl-xs shrink-0 cursor-pointer items-center justify-center rounded-sm border-0 bg-transparent text-sub outline-none data-[focus-visible]:outline-2 data-[focus-visible]:outline-accent data-[hovered]:text-text"
          >
            <X size={14} aria-hidden="true" />
          </AriaButton>
        ) : null}
      </SearchField>

      {/* 主题和来源的名字：换排列时一个淡出，一个淡入。 */}
      {built?.regions.map((r) => {
        const fs = nameSize(r.rad, scale);
        const isH = hoverTopic === r.id;
        const own = followed.has(r.id) || railHover === r.id;
        return (
          <div
            key={r.id}
            className={`pointer-events-none absolute z-[3] flex -translate-1/2 flex-col items-center transition-opacity ${arrangement === "topic" ? "opacity-100 delay-[450ms] duration-[280ms]" : "opacity-0 duration-[280ms]"}`}
            style={{ left: r.x, top: r.y }}
          >
            <span
              className="font-semibold whitespace-nowrap transition-colors duration-150"
              style={{
                fontSize: fs,
                lineHeight: `${Math.round(fs + 6)}px`,
                color: own ? "var(--accent)" : isH ? "color-mix(in srgb, var(--sub) 20%, var(--text))" : "color-mix(in srgb, var(--sub) 70%, var(--text))",
              }}
            >
              {names.get(r.id)}
            </span>
            <span
              className={`text-tiny text-faint transition-[opacity,translate] duration-150 ${isH ? "translate-y-0 opacity-100" : "-translate-y-[3px] opacity-0"}`}
            >
              {t("home.regionCount", { count: r.count })}
            </span>
          </div>
        );
      })}
      {built?.sources.map((s) => (
        <div
          key={s.id}
          className={`pointer-events-none absolute z-[3] -translate-1/2 font-semibold whitespace-nowrap transition-opacity ${arrangement === "source" ? "opacity-100 delay-[450ms] duration-[280ms]" : "opacity-0 duration-[280ms]"}`}
          style={{
            left: s.x,
            top: s.y,
            fontSize: nameSize(s.rad, scale),
            color: "color-mix(in srgb, var(--sub) 70%, var(--text))",
          }}
        >
          {s.name}
        </div>
      ))}

      {/* 「关注」：鼠标停在一片名字上时出现；键盘也能找到。 */}
      {arrangement === "topic" &&
        built?.regions.map((r) => {
          const on = followed.has(r.id);
          const name = names.get(r.id) ?? "";
          const fs = nameSize(r.rad, scale);
          const show = hoverTopic === r.id;
          return (
            <AriaButton
              key={r.id}
              aria-label={on ? t("home.unfollowTopic", { name }) : t("home.followTopic", { name })}
              onPress={() => follow.mutate({ topicId: r.id, followed: !on })}
              className={`press absolute z-[6] h-ctl-xs -translate-y-1/2 cursor-pointer rounded-full border border-border2 bg-bg px-3 text-small font-medium whitespace-nowrap text-label outline-none data-[focus-visible]:opacity-100 data-[focus-visible]:outline-2 data-[focus-visible]:outline-accent ${show ? "opacity-100" : "pointer-events-none opacity-0"} transition-opacity duration-150`}
              style={{ left: r.x + textWidth(name, fs) / 2 + 10, top: r.y }}
            >
              {on ? t("home.unfollow") : t("home.follow")}
            </AriaButton>
          );
        })}

      {/* 关注的主题折成的短列表：点变成行首的圆点。 */}
      {folded.map((row) => {
        const on = rowHover === row.place.id || open === row.place.id;
        return (
          <AriaButton
            key={row.place.id}
            onPress={() => select(row.place.id)}
            onHoverStart={() => setRowHover(row.place.id)}
            onHoverEnd={() => setRowHover((h) => (h === row.place.id ? null : h))}
            className="press absolute z-[4] flex h-[26px] animate-rise cursor-pointer items-center gap-[9px] rounded-sm border-0 px-2.5 text-left outline-none data-[focus-visible]:outline-2 data-[focus-visible]:outline-accent"
            style={{
              left: row.x,
              top: row.y,
              width: row.w,
              animationDelay: `${200 + row.index * 80}ms`,
              background: on ? "color-mix(in srgb, var(--bg), var(--chip))" : "transparent",
            }}
          >
            <span className="size-1.5 shrink-0" />
            <span className={`min-w-0 flex-1 truncate text-note font-medium ${on ? "text-text" : "text-label"}`}>
              {row.place.title}
            </span>
            <span className="shrink-0 text-tiny whitespace-nowrap text-faint">{row.place.platform}</span>
          </AriaButton>
        );
      })}

      {/* 每个点一个看不见的按钮：键盘能一个个走过去，读屏能读出来。 */}
      <div role="group" aria-label={t("home.points")}>
        {built?.places
          .filter((p) => !foldedIds.has(p.id))
          .map((p) => {
            const A = at(p);
            return (
              <AriaButton
                key={p.id}
                aria-label={`${p.title} · ${p.platform}`}
                onPress={() => select(p.id)}
                onFocus={() => setRowHover(p.id)}
                onBlur={() => setRowHover((h) => (h === p.id ? null : h))}
                className="absolute z-[2] size-6 -translate-1/2 cursor-pointer rounded-full border-0 bg-transparent outline-none transition-[left,top] duration-[900ms] ease-[cubic-bezier(0.37,0,0.63,1)] data-[focus-visible]:outline-2 data-[focus-visible]:outline-accent motion-reduce:transition-none"
                style={{ left: A.x, top: A.y }}
              />
            );
          })}
      </div>

      {/* 悬停那一条旁边的一行字，哪边有地方放哪边；不是盖在地图上的卡片。 */}
      {hovered ? (
        <HoverTag key={hovered.id} title={hovered.title} platform={hovered.platform} box={labelBox(at(hovered).x, at(hovered).y, hovered.title, hovered.platform, right)} />
      ) : null}
      {landedHere.map((p) =>
        p.id === shownHover ? null : (
          <div
            key={`land-${p.id}`}
            className="pointer-events-none absolute z-[5] flex h-6 items-center gap-2 px-1.5 whitespace-nowrap"
            style={{ left: at(p).x + 12, top: at(p).y - 12, animation: "fade 250ms ease 900ms both, fade 280ms ease 2900ms reverse forwards" }}
          >
            <span className="text-tag font-medium text-label">{p.title}</span>
            <span className="text-tiny text-faint">{t("home.linkOnly")}</span>
          </div>
        ),
      )}


      {map.points.length === 0 ? (
        <p className="absolute top-1/2 z-[5] m-0 w-max max-w-[calc(100%-var(--spacing-rail)-32px)] -translate-x-1/2 -translate-y-1/2 text-center text-note text-sub" style={{ left: MAP_X + (width - MAP_X) / 2 }}>
          {rich("home.empty", {
            link: <Link to="/plugins" className="mx-0.5 text-label underline decoration-border2 underline-offset-4 hover:text-text" />,
          })}
        </p>
      ) : null}
      <div className="absolute bottom-4 left-[268px] z-10 text-tag whitespace-pre text-sub">{hint}</div>
      <div
        className="absolute right-4 bottom-4 tabular-nums z-10 text-tag whitespace-nowrap text-sub transition-transform duration-[240ms] ease-out"
        style={{ transform: `translateX(${panelOpen ? -(PANEL_W + 16) : 0}px)` }}
      >
        {lampN !== null ? t("home.lamp", { count: lampN }) : ""}
      </div>


      <Panel
        open={panelOpen}
        q={q}
        itemId={open ?? null}
        hits={search.data ?? []}
        searchError={search.error}
        topics={topics}
        hover={shownHover}
        onSelect={select}
        onClose={closeItem}
        onClearQuery={() => setQuery("")}
        onRowHover={setRowHover}
        style={{ width: PANEL_W }}
      />
    </div>
  );
}

function HoverTag({ title, platform, box }: { title: string; platform: string; box: { x0: number; y0: number; side: 1 | -1 } }) {
  const style: CSSProperties & Record<"--from", string> = { left: box.x0, top: box.y0, "--from": `${-4 * box.side}px` };
  return (
    <div
      className="pointer-events-none absolute z-[5] flex h-6 animate-[tag_160ms_cubic-bezier(0.23,1,0.32,1)_both] items-center gap-2 px-1.5 whitespace-nowrap motion-reduce:animate-[fade_160ms_ease_both]"
      style={style}
    >
      <span className="text-tag font-medium text-text">{title}</span>
      <span className="text-tiny text-faint">{platform}</span>
    </div>
  );
}
