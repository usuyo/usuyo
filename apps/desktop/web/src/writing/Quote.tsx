import { Link } from "@tanstack/react-router";
import type { ReactNode } from "react";
import type { Citation, Position, SegmentRef } from "../api/endpoints";
import { t } from "../i18n";
import { citeSource, foundText, jumpOf, whereText } from "./words";

const LINK = "press -mx-2 block rounded-btn px-2 py-1.5 no-underline hover:bg-chip/60";

/**
 * 一处原话：左边是它在哪一层的哪一处，右边是原话和出处。点原话回到条目那一秒、那张图。
 * `itemId` 是 null（来源已删掉）时只摆出来，不能点。
 */
function Slip({
  itemId,
  position,
  where,
  quote,
  source,
  note,
  action,
}: {
  itemId: string | null;
  position: Position | undefined;
  where: string;
  quote: string;
  source: string;
  note?: string | null;
  action?: ReactNode;
}) {
  const body = (
    <span className="grid grid-cols-[minmax(0,1fr)] gap-0.5">
      <span className="text-body text-text select-text">{t("writing.quote", { quote })}</span>
      <span className="truncate text-small text-sub">
        <span className="mono mr-2 text-tag text-faint">{where}</span>
        {source}
      </span>
    </span>
  );
  return (
    <div className="group flex items-start gap-2 border-l-2 border-border2 pl-3">
      <div className="min-w-0 flex-1">
        {itemId === null ? (
          <div className="-mx-2 px-2 py-1.5">{body}</div>
        ) : (
          <Link to="/items/$id" params={{ id: itemId }} search={jumpOf(position)} className={LINK}>
            {body}
          </Link>
        )}
        {note ? <p className="m-0 pb-1 text-small text-sub">{note}</p> : null}
      </div>
      {action ? <div className="flex shrink-0 items-center pt-1.5">{action}</div> : null}
    </div>
  );
}

/** 页里的一处引用：原话、出处，找回得不一样时多一句。 */
export function CitationSlip({ citation, action }: { citation: Citation; action?: ReactNode }) {
  return (
    <Slip
      itemId={citation.found === "gone" ? null : citation.itemId}
      position={citation.position}
      where={whereText(citation.layer, citation.position)}
      quote={citation.quote}
      source={citeSource(citation)}
      note={foundText(citation)}
      action={action}
    />
  );
}

/** 一段还没引用的原话（线头里、角度里的）：出处写平台、标题和作者。 */
export function SegmentSlip({ segment, action, note }: { segment: SegmentRef; action?: ReactNode; note?: string }) {
  return (
    <Slip
      itemId={segment.itemId}
      position={segment.position}
      where={whereText(segment.layer, segment.position)}
      quote={segment.text}
      source={[segment.platform.name, segment.title, segment.author].filter(Boolean).join(" · ")}
      note={note ?? null}
      action={action}
    />
  );
}
