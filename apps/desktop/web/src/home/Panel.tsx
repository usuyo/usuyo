import { Link } from "@tanstack/react-router";
import { ChevronLeft, Ellipsis, ExternalLink, X } from "lucide-react";
import { useState, type CSSProperties } from "react";
import { Button as AriaButton, Label, TextArea, TextField } from "react-aria-components";
import type { Item, SearchHit, Topic } from "../api/endpoints";
import { isGone, useItem, useItemChange } from "../api/hooks";
import { ErrorLine } from "../components/ErrorLine";
import { FacetPreview } from "../item/FacetPreview";
import { dayText, GONE_TEXT, LAYER_NAME, positionText, snippetParts, STORED_TEXT } from "../item/format";
import { openExternal, webUrl } from "../shell";
import { Button } from "../ui/Button";
import { PlatformChip, TopicPill } from "../ui/Chip";

interface PanelProps {
  open: boolean;
  q: string;
  itemId: string | null;
  hits: SearchHit[];
  searchError: Error | null;
  topics: Topic[];
  hover: string | null;
  onSelect: (id: string) => void;
  onClose: () => void;
  onClearQuery: () => void;
  onRowHover: (id: string | null) => void;
  style: CSSProperties;
}

/** 右边一栏：有搜索词时是搜到的，点一条是它的详情。关上时最后的样子跟着淡出去。 */
export function Panel(props: PanelProps) {
  const { open, q, itemId } = props;
  const [lastId, setLastId] = useState(itemId);
  if (itemId !== null && itemId !== lastId) setLastId(itemId);
  const hasQ = q.trim() !== "";
  const showDetail = itemId !== null || (!hasQ && lastId !== null);
  return (
    <section
      aria-label={hasQ && itemId === null ? "搜索结果" : "条目详情"}
      aria-hidden={!open}
      inert={!open}
      className={`absolute top-4 right-4 bottom-4 z-[15] overflow-hidden rounded-card border border-border bg-card shadow-panel transition-[opacity,translate] motion-reduce:translate-x-0 ${open ? "visible translate-x-0 opacity-100 duration-[240ms] ease-out" : "invisible translate-x-[18px] opacity-0 duration-[160ms] ease-in"}`}
      style={props.style}
    >
      <div className="h-full overflow-y-auto px-5 pt-3.5 pb-5">
        {showDetail ? (
          <Detail id={itemId ?? lastId ?? ""} {...props} />
        ) : hasQ ? (
          <Results {...props} />
        ) : null}
      </div>
    </section>
  );
}

function Results({ hits, searchError, hover, onSelect, onClearQuery, onRowHover }: PanelProps) {
  return (
    <div className="animate-fade">
      <div className="flex h-ctl-sm items-center gap-2">
        <h2 className="m-0 flex-1 text-section font-semibold text-text">{hits.length ? `搜到 ${hits.length} 条` : "没有搜到"}</h2>
        <Button variant="ghost" size="sm" square aria-label="关掉搜索结果" onPress={onClearQuery}>
          <X size={16} aria-hidden="true" />
        </Button>
      </div>
      <p className="m-0 mt-0.5 text-small text-faint">
        {hits.length ? "左边写的是命中在哪一栏。点一条，在这里看详情" : "试试口播里的一句话、图里的字，或者作者名"}
      </p>
      {searchError ? <ErrorLine error={searchError} className="mt-3" /> : null}
      <ul className="m-0 mt-3.5 flex list-none flex-col gap-0.5 p-0">
        {hits.map((h) => {
          const { pre, mid, post } = snippetParts(h);
          return (
            <li key={h.itemId}>
              <AriaButton
                onPress={() => onSelect(h.itemId)}
                onHoverStart={() => onRowHover(h.itemId)}
                onHoverEnd={() => onRowHover(null)}
                className={`press flex w-full cursor-pointer items-start gap-3 rounded-btn border-0 p-2.5 text-left outline-none data-[focus-visible]:outline-2 data-[focus-visible]:outline-accent ${hover === h.itemId ? "bg-chip" : "bg-transparent"}`}
              >
                <span className="mono w-[84px] shrink-0 pt-px text-tiny leading-[18px] text-faint">
                  {LAYER_NAME[h.layer]}
                  {h.position ? ` ${positionText(h.position)}` : ""}
                </span>
                <span className="flex min-w-0 flex-1 flex-col gap-0.5">
                  <span className="truncate text-note leading-[19px] font-medium text-text">
                    {h.title}
                    {"  ·  "}
                    {h.platform.name}
                  </span>
                  <span className="text-small text-sub">
                    {pre}
                    <span className="font-semibold text-text">{mid}</span>
                    {post}
                  </span>
                </span>
              </AriaButton>
            </li>
          );
        })}
      </ul>
    </div>
  );
}

function Detail({ id, q, topics, onClose }: PanelProps & { id: string }) {
  const item = useItem(id);
  if (isGone(item.error)) {
    return (
      <p role="status" className="m-0 text-small text-sub">
        {GONE_TEXT}
      </p>
    );
  }
  if (item.error) return <ErrorLine error={item.error} />;
  if (!item.data) return null;
  return <DetailBody key={item.data.id} item={item.data} hasQ={q.trim() !== ""} topics={topics} onClose={onClose} />;
}

function DetailBody({ item, hasQ, topics, onClose }: { item: Item; hasQ: boolean; topics: Topic[]; onClose: () => void }) {
  const change = useItemChange(item.id);
  const [draft, setDraft] = useState("");
  const followed = new Set(topics.filter((t) => t.followed).map((t) => t.id));
  const names = new Map(topics.map((t) => [t.id, t.name]));
  return (
    <div className="animate-fade">
      {hasQ ? (
        <Button variant="ghost" size="none" className="-ml-2 mb-1.5 h-ctl-tight gap-1 rounded-sm px-2 text-small" onPress={onClose}>
          <ChevronLeft size={14} aria-hidden="true" />
          <span>搜索结果</span>
        </Button>
      ) : null}
      <div className="flex h-ctl-sm items-center gap-2">
        <PlatformChip>{item.platform.name}</PlatformChip>
        <span className="min-w-0 flex-1 truncate text-note text-sub">{item.author}</span>
        <Button variant="ghost" size="sm" square aria-label="关闭详情" onPress={onClose}>
          <X size={16} aria-hidden="true" />
        </Button>
      </div>
      <h2 className="m-0 mt-2.5 text-panel font-semibold text-text">
        <Link to="/items/$id" params={{ id: item.id }} className="text-inherit no-underline hover:underline hover:decoration-border2 hover:underline-offset-4">
          {item.title}
        </Link>
      </h2>
      <p className="m-0 mt-1.5 text-small text-faint">
        {dayText(item.takenAt)}收下　·　{STORED_TEXT[item.stored]}
      </p>
      {item.topicIds.length ? (
        <div className="mt-3 flex flex-wrap gap-1.5">
          {item.topicIds.map((t) => (
            <TopicPill key={t} followed={followed.has(t)}>
              {names.get(t)}
            </TopicPill>
          ))}
        </div>
      ) : null}
      <FacetPreview item={item} />
      <div className="mt-6 text-tag font-medium text-accent">记一句</div>
      {item.note ? (
        <p className="m-0 mt-1.5 text-body text-text">{item.note}</p>
      ) : (
        <>
          <TextField value={draft} onChange={setDraft} aria-label="记一句" className="mt-2">
            <Label className="sr-only-text">记一句</Label>
            <TextArea
              rows={2}
              placeholder="写一句你自己的话"
              className="block w-full resize-none rounded-btn border border-border2 bg-bg px-3 py-2.5 font-sans text-body text-text outline-none focus:border-label"
            />
          </TextField>
          <Button
            variant="soft"
            size="sm"
            className="mt-2"
            isDisabled={draft.trim() === "" || change.isPending}
            onPress={() => change.mutate({ kind: "note", text: draft })}
          >
            记下
          </Button>
        </>
      )}
      <div className="mt-7 flex gap-2">
        <Button variant="outline" size="md" onPress={() => void openExternal(item.url)} isDisabled={webUrl(item.url) === null}>
          <ExternalLink size={14} aria-hidden="true" />
          <span>打开原帖</span>
        </Button>
        <Link
          to="/items/$id"
          params={{ id: item.id }}
          aria-label="打开条目页"
          className="press flex size-ctl items-center justify-center rounded-btn border border-border2 text-sub hover:text-text"
        >
          <Ellipsis size={16} aria-hidden="true" />
        </Link>
      </div>
    </div>
  );
}
