import { getRouteApi, Link } from "@tanstack/react-router";
import { ArrowLeft, ExternalLink, Map as MapIcon } from "lucide-react";
import type { Item, Topic } from "../api/endpoints";
import { isGone, useItem, useItemChange, useTopics } from "../api/hooks";
import { ErrorLine } from "../components/ErrorLine";
import { Rail } from "../frame/Rail";
import { openExternal, webUrl } from "../shell";
import { Button } from "../ui/Button";
import { PlatformChip, SectionLabel } from "../ui/Chip";
import { FlashLine, useFlash } from "../ui/Flash";
import { FacetColumn } from "./FacetColumn";
import { dayText, facetOf, GONE_TEXT, STORED_TEXT } from "./format";
import { ItemActions, ItemMeta } from "./ItemDetails";
import { NoteBlock } from "./NoteBlock";
import { usePlayer } from "./usePlayer";
import { Viewer } from "./Viewer";

const route = getRouteApi("/items/$id");

/** 条目页：左边是画面，右边是这一条的几个侧面，各占一块。 */
export function ItemPage() {
  const { id } = route.useParams();
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
              to="/"
              className="press flex h-ctl items-center gap-2.5 rounded-btn px-2.5 text-body text-label no-underline hover:bg-chip"
            >
              <span className="size-[7px] shrink-0 rounded-full bg-accent" aria-hidden="true" />
              <span>{t.name}</span>
            </Link>
          ))}
        </div>
      </Rail>
      <main className="absolute inset-y-0 right-0 left-rail overflow-y-auto px-gutter pt-5 pb-4">
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
        {item.data && topics.data ? <ItemBody key={item.data.id} item={item.data} topics={topics.data} /> : null}
      </main>
    </div>
  );
}

function ItemBody({ item, topics }: { item: Item; topics: Topic[] }) {
  const video = facetOf(item, "video");
  const firstImage = facetOf(item, "image")?.images[0]?.key ?? null;
  const player = usePlayer(video, firstImage, item.stored === "complete" || item.stored === "partial");
  const change = useItemChange(item.id);
  const open = useFlash();
  const trashed = item.trashedAt !== undefined;

  return (
    <>
      <header className="relative mt-2.5 pr-[220px]">
        <div className="flex h-6 items-center gap-2">
          <PlatformChip>{item.platform.name}</PlatformChip>
          <span className="text-note text-sub">{item.author}</span>
        </div>
        <h1 className="m-0 mt-2 text-title font-semibold text-text">{item.title}</h1>
        {trashed ? (
          <div role="status" className="flex h-7 animate-rise items-center gap-3 text-note text-text">
            <span>这条移进了回收站（{dayText(item.trashedAt ?? "")}），30 天内能恢复</span>
            <Button variant="soft" size="sm" className="-my-1" onPress={() => change.mutate({ kind: "restore" })}>
              恢复这条
            </Button>
          </div>
        ) : (
          <p className="m-0 mt-1 text-note text-sub">
            {dayText(item.takenAt)}收下　·　{item.stored === "link" ? `只有链接${item.missing ? "" : "，正在取内容"}` : STORED_TEXT[item.stored]}
          </p>
        )}
        <div className="absolute top-1 right-0 flex flex-col items-end gap-1.5">
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
        className={`mt-6 flex gap-9 transition-opacity duration-200 ${trashed ? "pointer-events-none opacity-40" : "opacity-100"}`}
      >
        <div className="w-[620px] shrink-0">
          <Viewer item={item} player={player} onRetry={() => change.mutate({ kind: "refetch" })} />
          <div className="mt-6 flex items-start gap-6">
            <ItemMeta item={item} topics={topics} />
            <ItemActions item={item} />
          </div>
        </div>
        <section aria-label="这一条的文字" className="flex h-[calc(100vh-192px)] min-h-[560px] min-w-0 flex-1 flex-col">
          <FacetColumn item={item} player={player} />
          <NoteBlock item={item} />
        </section>
      </div>
    </>
  );
}
