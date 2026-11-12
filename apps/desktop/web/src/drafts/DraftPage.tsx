import { getRouteApi, Link } from "@tanstack/react-router";
import { ArrowLeft } from "lucide-react";
import { useState } from "react";
import { Input, Label, TextField } from "react-aria-components";
import { isNotServed } from "../api/client";
import type { Draft } from "../api/endpoints";
import { useDraft, useDraftChange } from "../api/hooks";
import { ErrorLine, NotServedLine } from "../components/ErrorLine";
import { Rail } from "../frame/Rail";
import { Choice } from "../ui/Choice";
import { PageText } from "../writing/PageText";
import { citeSource, whereText } from "../writing/words";
import { QuotePicker } from "./QuotePicker";
import { editedText, KIND_CHOICES } from "./words";

const route = getRouteApi("/drafts/$id");

/** 一篇成稿：左边写，右边拿原话；底下是出处，按在文中出现的先后排。 */
export function DraftPage() {
  const { id } = route.useParams();
  const draft = useDraft(id);
  return (
    <div className="relative h-full overflow-hidden bg-bg">
      <Rail label="主导航" />
      <main className="@container absolute inset-y-0 right-0 left-rail overflow-y-auto px-6 pt-5 pb-20 lg:px-gutter">
        <Link to="/drafts" className="press -ml-2 inline-flex h-ctl-sm items-center gap-2 rounded-btn pr-3 pl-2 text-note text-sub no-underline hover:text-text">
          <ArrowLeft size={16} aria-hidden="true" />
          <span>所有成稿</span>
        </Link>
        {isNotServed(draft.error) ? (
          <NotServedLine className="mt-4">这台电脑上的 usuyo 内核还没有成稿；以后在这里把引过的原话拼成你自己的一篇。</NotServedLine>
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
        <TextField value={title} onChange={setTitle} aria-label="标题">
          <Label className="sr-only-text">标题</Label>
          <Input
            onBlur={commitTitle}
            onKeyDown={(e) => {
              if (e.key === "Enter") e.currentTarget.blur();
            }}
            className="-mx-1 w-full rounded-sm border-0 border-b border-transparent bg-transparent px-1 py-0.5 font-sans text-title font-semibold text-text outline-none hover:border-border focus:border-border2"
          />
        </TextField>
        <div className="mt-2 flex flex-wrap items-center gap-x-4 gap-y-2">
          <Choice label="这一篇是" options={KIND_CHOICES} value={draft.kind} isDisabled={change.isPending} onChange={(kind) => change.mutate({ kind: "meta", patch: { kind } })} />
          <span className="text-small text-sub">
            {editedText(draft.updatedAt)}　<span className="mono text-tag text-faint select-text">{draft.page.file}</span>
          </span>
        </div>
        {draft.from.length > 0 ? (
          <p className="m-0 mt-2 text-small text-sub">
            用到的主题：
            {draft.from.map((t, i) => (
              <span key={t.id}>
                {i > 0 ? "、" : ""}
                <Link to="/topics/$id" params={{ id: t.id }} className="text-label underline decoration-border2 underline-offset-4 hover:text-text">
                  {t.name}
                </Link>
              </span>
            ))}
          </p>
        ) : null}
        {change.error ? <ErrorLine error={change.error} className="mt-3" /> : null}
        <div className="mt-7">
          <PageText
            page={draft.page}
            busy={change.isPending}
            empty="还是空的。先写一句，或者从右边引一段原话进来。"
            onEdit={(patch) => change.mutate({ kind: "page", patch })}
          />
        </div>
        {order.length > 0 ? (
          <section aria-labelledby="draft-sources-h" className="mt-12 border-t border-border pt-5">
            <h2 id="draft-sources-h" className="m-0 text-note font-semibold text-text">
              出处
            </h2>
            <p className="m-0 mt-0.5 text-small text-sub">导出和复制时跟着这一篇走，排在页尾；来源删掉了，原话照样留着。</p>
            <ol className="m-0 mt-3 flex list-none flex-col gap-2 p-0">
              {order.map((c, i) => (
                <li key={c.label} className="grid grid-cols-[2rem_minmax(0,1fr)] text-small">
                  <span className="mono text-tag text-faint">{i + 1}</span>
                  <span className="text-label">
                    「{c.quote}」<span className="text-sub">　{citeSource(c)}　{whereText(c.layer, c.position)}</span>
                    {c.found === "gone" ? <span className="text-faint">　来源已删掉</span> : null}
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
