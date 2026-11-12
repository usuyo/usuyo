import { Link } from "@tanstack/react-router";
import type { Facet, Item, Segment } from "../api/endpoints";
import { MonoLabel } from "../ui/Chip";
import { sourceText } from "./faces";
import { clock, LAYER_NAME, positionText } from "./format";

const LINES = 3;

/** 主页面板里的样子：每个侧面各占一段，只露前几句，全的在条目页。 */
export function FacetPreview({ item }: { item: Item }) {
  if (item.facets.length === 0) {
    return (
      <p className="m-0 mt-[18px] text-body text-sub">
        {item.missing ?? "取到内容以后，这里会有口播和图里的字。"}
      </p>
    );
  }
  return (
    <div className="mt-[18px] flex flex-col gap-4">
      {item.facets.map((f) => (
        <section key={f.key} aria-label={LAYER_NAME[f.layer]} className="border-t border-border pt-2.5">
          <header className="flex flex-wrap items-baseline gap-x-2">
            <h3 className="m-0 text-note font-semibold text-text">{LAYER_NAME[f.layer]}</h3>
            <MonoLabel>{countText(f)}</MonoLabel>
            {"source" in f ? <span className="text-small text-faint">{sourceText(f.layer, f.source)}</span> : null}
            {f.remark ? <span className="text-small text-sub">{f.remark}</span> : null}
          </header>
          <FacetBody facet={f} item={item} />
        </section>
      ))}
    </div>
  );
}

function countText(f: Facet): string {
  switch (f.layer) {
    case "video":
      return clock(f.durationSec);
    case "image":
      return `${f.images.length} 张`;
    case "comments":
      return `${f.comments.length} 条`;
    default:
      return `${f.segments.length} 段`;
  }
}

function FacetBody({ facet, item }: { facet: Facet; item: Item }) {
  if (facet.layer === "video" || facet.layer === "image") {
    const pics = facet.layer === "video" ? facet.frames.map((fr) => ({ k: `${fr.sec}`, url: fr.url, at: clock(fr.sec) })) : facet.images.map((im, i) => ({ k: im.key, url: im.url, at: `图 ${i + 1}` }));
    return (
      <div className="mt-2 flex gap-1.5">
        {pics.slice(0, 4).map((p) => (
          <img key={p.k} src={p.url} alt={p.at} className="h-[46px] w-[78px] rounded-chip border border-border bg-card2 object-cover" />
        ))}
      </div>
    );
  }
  if (facet.layer === "comments") {
    return (
      <ul className="m-0 mt-1.5 flex list-none flex-col gap-1 p-0">
        {facet.comments.slice(0, 2).map((c) => (
          <li key={`${c.author}${c.text}`} className="text-body text-label">
            <span className="text-small text-faint">{c.author}　</span>
            {c.text}
          </li>
        ))}
        <More item={item} rest={facet.comments.length - 2} unit="条" />
      </ul>
    );
  }
  return <Lines segments={facet.segments} item={item} />;
}

function Lines({ segments, item }: { segments: Segment[]; item: Item }) {
  return (
    <div className="mt-1.5 flex flex-col gap-1">
      {segments.slice(0, LINES).map((s) => (
        <div key={`${s.position ? positionText(s.position) : ""}${s.text}`} className="flex items-baseline gap-2.5">
          {s.position ? <span className="mono w-11 shrink-0 text-small leading-[22px] text-faint">{positionText(s.position)}</span> : null}
          <span className="text-body text-label">{s.text}</span>
        </div>
      ))}
      <More item={item} rest={segments.length - LINES} unit="段" />
    </div>
  );
}

function More({ item, rest, unit }: { item: Item; rest: number; unit: string }) {
  if (rest <= 0) return null;
  return (
    <Link to="/items/$id" params={{ id: item.id }} className="w-fit text-small text-sub underline decoration-border2 underline-offset-4 hover:text-text">
      还有 {rest} {unit}，在条目页看
    </Link>
  );
}
