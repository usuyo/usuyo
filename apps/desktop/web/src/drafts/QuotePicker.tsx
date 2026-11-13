import { Search } from "lucide-react";
import { useState } from "react";
import { Input, Label, TextField } from "react-aria-components";
import { isNotServed } from "../api/client";
import type { SegmentRef } from "../api/endpoints";
import { useSearch, useTopicPage, useTopics } from "../api/hooks";
import { ErrorLine, NotServedLine } from "../components/ErrorLine";
import { t } from "../i18n";
import { Button } from "../ui/Button";
import { Choice } from "../ui/Choice";
import { CitationSlip, SegmentSlip } from "../writing/Quote";
import { quotableOf } from "./words";

type From = "search" | "topic";

/**
 * 拿原话：搜一句（口播、图里的字、作者写的都搜），或者从一个主题页里引过的拿。点「引进来」接在这一篇的末尾，出处跟着。
 */
export function QuotePicker({ busy, onCite }: { busy: boolean; onCite: (segment: Pick<SegmentRef, "itemId" | "layer" | "position" | "text">) => void }) {
  const [from, setFrom] = useState<From>("search");
  return (
    <section aria-labelledby="picker-h" className="flex min-h-0 flex-col rounded-card border border-border bg-card px-4 pt-3.5 pb-3">
      <h2 id="picker-h" className="m-0 text-section font-semibold text-text">
        {t("drafts.picker.title")}
      </h2>
      <p className="m-0 mt-0.5 text-small text-sub">{t("drafts.picker.sub")}</p>
      <div className="mt-3">
        <Choice<From>
          label={t("drafts.picker.from")}
          options={[
            { id: "search", text: t("drafts.picker.search") },
            { id: "topic", text: t("drafts.picker.topic") },
          ]}
          value={from}
          onChange={setFrom}
        />
      </div>
      <div className="mt-3 min-h-0">{from === "search" ? <SearchQuotes busy={busy} onCite={onCite} /> : <TopicQuotes busy={busy} onCite={onCite} />}</div>
    </section>
  );
}

function CiteButton({ text, busy, onPress }: { text: string; busy: boolean; onPress: () => void }) {
  return (
    <Button variant="ghost" size="xs" isDisabled={busy} aria-label={t("drafts.picker.cite", { text })} onPress={onPress}>
      {t("drafts.picker.citeButton")}
    </Button>
  );
}

function SearchQuotes({ busy, onCite }: { busy: boolean; onCite: (s: SegmentRef) => void }) {
  const [q, setQ] = useState("");
  const hits = useSearch(q);
  const quotable = quotableOf(hits.data ?? []);
  return (
    <div className="flex flex-col gap-2">
      <TextField value={q} onChange={setQ} aria-label={t("drafts.picker.searchLabel")}>
        <Label className="flex h-ctl-sm items-center gap-2 rounded-input border border-border2 px-3 text-sub focus-within:border-label">
          <Search size={14} aria-hidden="true" className="shrink-0" />
          <span className="sr-only-text">{t("drafts.picker.searchLabel")}</span>
          <Input placeholder={t("drafts.picker.searchPlaceholder")} className="h-full min-w-0 flex-1 border-0 bg-transparent p-0 font-sans text-note text-text outline-none" />
        </Label>
      </TextField>
      {hits.error ? <ErrorLine error={hits.error} /> : null}
      {q.trim() !== "" && hits.data && quotable.length === 0 ? <p className="m-0 text-small text-sub">{t("drafts.picker.noQuotable")}</p> : null}
      <div className="flex flex-col gap-1.5">
        {quotable.map((s) => (
          <SegmentSlip key={`${s.itemId}:${s.layer}:${s.text}`} segment={s} action={<CiteButton text={s.text} busy={busy} onPress={() => onCite(s)} />} />
        ))}
      </div>
    </div>
  );
}

function TopicQuotes({ busy, onCite }: { busy: boolean; onCite: (s: Pick<SegmentRef, "itemId" | "layer" | "position" | "text">) => void }) {
  const topics = useTopics();
  const list = [...(topics.data ?? [])].sort((a, b) => Number(b.followed) - Number(a.followed));
  const [topicId, setTopicId] = useState<string | undefined>(undefined);
  const chosen = topicId ?? list[0]?.id;
  return (
    <div className="flex flex-col gap-3">
      {list.length > 0 ? <Choice label={t("drafts.picker.whichTopic")} options={list.map((x) => ({ id: x.id, text: x.name }))} value={chosen} onChange={setTopicId} /> : null}
      {chosen ? <TopicCitations topicId={chosen} busy={busy} onCite={onCite} /> : null}
    </div>
  );
}

function TopicCitations({ topicId, busy, onCite }: { topicId: string; busy: boolean; onCite: (s: Pick<SegmentRef, "itemId" | "layer" | "position" | "text">) => void }) {
  const page = useTopicPage(topicId);
  if (isNotServed(page.error)) return <NotServedLine>{t("drafts.picker.topicNotServed")}</NotServedLine>;
  if (page.error) return <ErrorLine error={page.error} />;
  const cites = page.data?.page.citations.filter((c) => c.itemId !== null) ?? [];
  if (page.data && cites.length === 0) return <p className="m-0 text-small text-sub">{t("drafts.picker.topicEmpty")}</p>;
  return (
    <div className="flex flex-col gap-1.5">
      {cites.map((c) => (
        <CitationSlip
          key={c.label}
          citation={c}
          action={
            <CiteButton
              text={c.quote}
              busy={busy}
              onPress={() => c.itemId !== null && onCite({ itemId: c.itemId, layer: c.layer, text: c.quote, ...(c.position ? { position: c.position } : {}) })}
            />
          }
        />
      ))}
    </div>
  );
}
