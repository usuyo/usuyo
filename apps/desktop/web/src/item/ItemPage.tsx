import { getRouteApi, Link } from "@tanstack/react-router";
import { ArrowLeft, ExternalLink, Map as MapIcon } from "lucide-react";
import type { Facet, Item, Topic } from "../api/endpoints";
import { useEffect, useRef } from "react";
import type { ItemSearch } from "../router";
import { isGone, useItem, useItemChange, useTopics, useTransform, useTrash } from "../api/hooks";
import { ErrorLine } from "../components/ErrorLine";
import { Rail } from "../frame/Rail";
import { openExternal, webUrl } from "../shell";
import { Button } from "../ui/Button";
import { PlatformChip, SectionLabel } from "../ui/Chip";
import { FlashLine, useFlash } from "../ui/Flash";
import { FacetColumn } from "./FacetColumn";
import { dayText, facetOf, GONE_TEXT, keepText, STORED_TEXT, type MediaKeep } from "./format";
import { ItemActions, ItemMeta } from "./ItemDetails";
import { NoteBlock } from "./NoteBlock";
import { usePlayer } from "./usePlayer";
import { Viewer } from "./Viewer";

const route = getRouteApi("/items/$id");

/** 条目页：画面和这一条的几面并排；窗口窄时画面在上、几面在下。 */
export function ItemPage() {
  const { id } = route.useParams();
  const jump = route.useSearch();
  const item = useItem(id);
  const topics = useTopics();
  const followed = topics.data?.filter((t) => t.followed) ?? [];
  const error = item.error ?? topics.error;
  return (
    <div className="relative h-full overflow-hidden bg-bg">
      <Rail label="主导航">
        <Link
          to="/"
          className="press mt-[22px] flex h-ctl items-center gap-2.5 rounded-btn px-2.5 text-body text-label no-underline hover:bg-chip/60"
        >
          <MapIcon size={16} aria-hidden="true" />
          <span>地图</span>
        </Link>
        <SectionLabel className="mt-[22px]">关注的主题</SectionLabel>
        <div className="mt-1.5 flex flex-col gap-0.5">
          {followed.map((t) => (
            <Link
              key={t.id}
              to="/topics/$id"
              params={{ id: t.id }}
              className="press flex h-ctl items-center gap-2.5 rounded-btn px-2.5 text-body text-label no-underline hover:bg-chip"
            >
              <span className="size-[7px] shrink-0 rounded-full bg-accent" aria-hidden="true" />
              <span>{t.name}</span>
            </Link>
          ))}
        </div>
      </Rail>
      <main className="@container absolute inset-y-0 right-0 left-rail overflow-y-auto px-6 pt-5 pb-6 lg:px-gutter">
        <Link
          to="/"
          search={{ open: id }}
          className="press -ml-2 inline-flex h-ctl-sm items-center gap-2 rounded-btn pr-3 pl-2 text-note text-sub no-underline hover:text-text"
        >
          <ArrowLeft size={16} aria-hidden="true" />
          <span>回到地图</span>
        </Link>
        {isGone(item.error) ? (
          <p role="status" className="m-0 mt-4 text-small text-sub">
            {GONE_TEXT}
          </p>
        ) : error ? (
          <ErrorLine error={error} className="mt-4" />
        ) : null}
        {item.data && topics.data ? <ItemBody key={item.data.id} item={item.data} topics={topics.data} jump={jump} /> : null}
      </main>
    </div>
  );
}

function ItemBody({ item, topics, jump }: { item: Item; topics: Topic[]; jump: ItemSearch }) {
  const video = facetOf(item, "video");
  const firstImage = facetOf(item, "image")?.images[0]?.key ?? null;
  const player = usePlayer(video, firstImage, item.stored === "complete" || item.stored === "partial");
  const change = useItemChange(item.id);
  const transform = useTransform(item.id);
  const trash = useTrash();
  const open = useFlash();
  const trashed = item.trashedAt !== undefined;
  const keep: MediaKeep = {
    ...(trash.data ? { days: trash.data.trashDays } : {}),
    // 只留文字的条目，回收站里已经没有它的任何一项：移出去的媒体到期或被清掉了。
    // 整条在回收站里时，移出去的媒体跟着整条，不单列。
    gone: item.stored === "text-only" && trash.data !== undefined && !trash.data.entries.some((e) => e.itemId === item.id),
  };

  // 从引用跳回来：打开时停到那一秒或那张图，只做一次。
  const { go } = player;
  const jumped = useRef(false);
  useEffect(() => {
    if (jumped.current) return;
    jumped.current = true;
    if (jump.t !== undefined) go({ kind: "time", sec: jump.t, snapshotId: "" });
    else if (jump.img !== undefined) go({ kind: "image", imageKey: jump.img });
  }, [jump, go]);

  // 打开这一条时问一次内核：它还没做好的转换（没转出来的也算）挪到最前，在电池上也转。在回收站里的不问。
  const { mutate: hurry } = transform;
  const hurried = useRef(false);
  useEffect(() => {
    if (hurried.current || trashed) return;
    hurried.current = true;
    hurry({ jobs: "pending" });
  }, [trashed, hurry]);

  // 时间轴上的记号：一秒一个，同一帧上认出的几行只记一次。
  const times = (layer: "asr" | "frames") => [
    ...new Set(
      item.facets.find((f): f is Extract<Facet, { layer: "asr" | "frames" }> => f.layer === layer)?.segments.flatMap((s) => (s.position?.kind === "time" ? [s.position.sec] : [])) ?? [],
    ),
  ];

  return (
    <>
      <header className="mt-2.5 flex flex-wrap items-start gap-x-6 gap-y-3">
        <div className="min-w-0 flex-[1_1_320px]">
          <div className="flex h-6 items-center gap-2">
            <PlatformChip>{item.platform.name}</PlatformChip>
            <span className="truncate text-note text-sub">{item.author}</span>
          </div>
          <h1 className="m-0 mt-2 text-title font-semibold text-text">{item.title}</h1>
          {trashed ? (
            <div role="status" className="flex min-h-7 animate-rise flex-wrap items-center gap-3 text-note text-text">
              <span>
                这条移进了回收站（{dayText(item.trashedAt ?? "")}），{keepText(keep.days)}
              </span>
              <Button variant="soft" size="sm" className="-my-1" onPress={() => change.mutate({ kind: "restore" })}>
                恢复这条
              </Button>
            </div>
          ) : (
            <p className="m-0 mt-1 text-note text-sub">
              {dayText(item.takenAt)}收下　·　
              {item.stored === "link"
                ? `只有链接${item.missing ? "" : "，正在取内容"}`
                : item.stored === "text-only" && keep.gone
                  ? "只留文字，视频和封面已经清掉了"
                  : STORED_TEXT[item.stored]}
            </p>
          )}
        </div>
        <div className="flex flex-col items-end gap-1.5">
          <Button
            variant="outline"
            size="md"
            isDisabled={webUrl(item.url) === null}
            onPress={() => {
              void openExternal(item.url).then((opened) => {
                if (opened) open.show("在浏览器里打开了原帖");
              });
            }}
          >
            <ExternalLink size={14} aria-hidden="true" />
            <span>打开原帖</span>
          </Button>
          <FlashLine flash={open} className="text-tag whitespace-nowrap text-sub" />
        </div>
      </header>

      <div
        inert={trashed}
        className={`mt-6 grid items-start gap-x-9 gap-y-8 transition-opacity duration-200 @[840px]:grid-cols-[minmax(0,5fr)_minmax(0,6fr)] ${trashed ? "pointer-events-none opacity-40" : "opacity-100"}`}
      >
        <div className="flex min-w-0 flex-col gap-6 @[840px]:sticky @[840px]:top-0">
          <div className="w-full max-w-[760px]">
            <Viewer item={item} player={player} keep={keep} speech={times("asr")} screen={times("frames")} onRetry={() => change.mutate({ kind: "refetch" })} />
          </div>
          <div className="flex flex-wrap items-start gap-6">
            <ItemMeta item={item} topics={topics} keep={keep} />
            <ItemActions item={item} keep={keep} />
          </div>
        </div>
        <section aria-label="这一条的文字" className="flex min-w-0 flex-col gap-5">
          <FacetColumn item={item} player={player} keep={keep} />
          <NoteBlock item={item} />
        </section>
      </div>
    </>
  );
}
