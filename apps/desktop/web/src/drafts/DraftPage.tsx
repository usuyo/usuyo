import { getRouteApi, Link } from "@tanstack/react-router";
import { ArrowLeft } from "lucide-react";
import { useState } from "react";
import { Input, Label, TextField } from "react-aria-components";
import { isNotServed } from "../api/client";
import type { Draft } from "../api/endpoints";
import { useDraft, useDraftChange } from "../api/hooks";
import { ErrorLine, NotServedLine } from "../components/ErrorLine";
import { Rail } from "../frame/Rail";
import { currentLocale, rich, t } from "../i18n";
import { Choice } from "../ui/Choice";
import { PageText } from "../writing/PageText";
import { citeSource, whereText } from "../writing/words";
import { QuotePicker } from "./QuotePicker";
import { editedText, kindChoices } from "./words";

const route = getRouteApi("/drafts/$id");

/** 一篇成稿：左边写，右边拿原话；底下是出处，按在文中出现的先后排。 */
export function DraftPage() {
  const { id } = route.useParams();
  const draft = useDraft(id);
  return (
    <div className="relative h-full overflow-hidden bg-bg">
      <Rail label={t("common.nav.main")} />
      <main className="@container absolute inset-y-0 right-0 left-rail overflow-y-auto px-6 pt-5 pb-20 lg:px-gutter">
        <Link to="/drafts" className="press -ml-2 inline-flex h-ctl-sm items-center gap-2 rounded-btn pr-3 pl-2 text-note text-sub no-underline hover:text-text">
          <ArrowLeft size={16} aria-hidden="true" />
          <span>{t("drafts.page.all")}</span>
        </Link>
        {isNotServed(draft.error) ? (
          <NotServedLine className="mt-4">{t("drafts.notServed")}</NotServedLine>
        ) : draft.error ? (
          <ErrorLine error={draft.error} className="mt-4" />
        ) : null}
        {draft.data ? <DraftBody key={draft.data.id} draft={draft.data} /> : null}
      </main>
    </div>
  );
}

function DraftBody({ draft }: { draft: Draft }) {
  const change = useDraftChange(draft.id);
  const [title, setTitle] = useState(draft.title);
  const commitTitle = () => {
    if (title.trim() !== "" && title.trim() !== draft.title) change.mutate({ kind: "meta", patch: { title } });
    else setTitle(draft.title);
  };
  // 出处照在文中出现的先后排：编号就是读到它的次序。
  const byLabel = new Map(draft.page.citations.map((c) => [c.label, c]));
  const order = [...new Set(draft.page.blocks.flatMap((b) => b.cites))].flatMap((l) => {
    const c = byLabel.get(l);
    return c ? [c] : [];
  });

  return (
    <div className="mt-3 grid items-start gap-x-10 gap-y-8 @[860px]:grid-cols-[minmax(0,1fr)_minmax(0,320px)]">
      <article className="min-w-0 max-w-[720px]">
        <TextField value={title} onChange={setTitle} aria-label={t("drafts.page.titleLabel")}>
          <Label className="sr-only-text">{t("drafts.page.titleLabel")}</Label>
          <Input
            onBlur={commitTitle}
            onKeyDown={(e) => {
              if (e.key === "Enter") e.currentTarget.blur();
            }}
            className="-mx-1 w-full rounded-sm border-0 border-b border-transparent bg-transparent px-1 py-0.5 font-sans text-title font-semibold text-text outline-none hover:border-border focus:border-border2"
          />
        </TextField>
        <div className="mt-2 flex flex-wrap items-center gap-x-4 gap-y-2">
          <Choice label={t("drafts.page.kindLabel")} options={kindChoices()} value={draft.kind} isDisabled={change.isPending} onChange={(kind) => change.mutate({ kind: "meta", patch: { kind } })} />
          <span className="text-small text-sub">
            {editedText(draft.updatedAt)} <span className="mono ml-2 text-tag text-faint select-text">{draft.page.file}</span>
          </span>
        </div>
        {draft.from.length > 0 ? (
          <p className="m-0 mt-2 text-small text-sub">
            <TopicsUsed topics={draft.from} />
          </p>
        ) : null}
        {change.error ? <ErrorLine error={change.error} className="mt-3" /> : null}
        <div className="mt-7">
          <PageText
            page={draft.page}
            busy={change.isPending}
            empty={t("drafts.page.empty")}
            onEdit={(patch) => change.mutate({ kind: "page", patch })}
          />
        </div>
        {order.length > 0 ? (
          <section aria-labelledby="draft-sources-h" className="mt-12 border-t border-border pt-5">
            <h2 id="draft-sources-h" className="m-0 text-note font-semibold text-text">
              {t("drafts.page.sources")}
            </h2>
            <p className="m-0 mt-0.5 text-small text-sub">{t("drafts.page.sourcesSub")}</p>
            <ol className="m-0 mt-3 flex list-none flex-col gap-2 p-0">
              {order.map((c, i) => (
                <li key={c.label} className="grid grid-cols-[2rem_minmax(0,1fr)] text-small">
                  <span className="mono text-tag text-faint">{i + 1}</span>
                  <span className="text-label">
                    {t("writing.quote", { quote: c.quote })}
                    <span className="ml-2 text-sub">
                      {citeSource(c)} · {whereText(c.layer, c.position)}
                    </span>
                    {c.found === "gone" ? <span className="ml-2 text-faint">{t("drafts.page.sourceGone")}</span> : null}
                  </span>
                </li>
              ))}
            </ol>
          </section>
        ) : null}
      </article>
      <aside className="min-w-0 @[860px]:sticky @[860px]:top-0">
        <QuotePicker busy={change.isPending} onCite={({ itemId, layer, position, text }) => change.mutate({ kind: "cite", body: { segment: { itemId, layer, text, ...(position ? { position } : {}) } } })} />
      </aside>
    </div>
  );
}

/** 「引用的主题：咖啡、胶片和城市」：每个主题是一个链接，按当前语言的并列写法连起来。 */
function TopicsUsed({ topics }: { topics: { id: string; name: string }[] }) {
  const parts = new Intl.ListFormat(currentLocale(), { type: "conjunction" }).formatToParts(topics.map((x) => x.name));
  const list = parts.map((p, i) => {
    if (p.type === "literal") return <span key={`l${i}`}>{p.value}</span>;
    const topic = topics[parts.slice(0, i).filter((q) => q.type === "element").length];
    return topic ? (
      <Link key={topic.id} to="/topics/$id" params={{ id: topic.id }} className="text-label underline decoration-border2 underline-offset-4 hover:text-text">
        {topic.name}
      </Link>
    ) : null;
  });
  return <>{rich("drafts.page.topicsUsed", { list: <span>{list}</span> })}</>;
}
