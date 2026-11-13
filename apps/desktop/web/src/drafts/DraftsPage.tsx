import { Link, useNavigate } from "@tanstack/react-router";
import { PenLine } from "lucide-react";
import { isNotServed } from "../api/client";
import { useDrafts, useNewDraft } from "../api/hooks";
import { ErrorLine, NotServedLine } from "../components/ErrorLine";
import { Rail } from "../frame/Rail";
import { t } from "../i18n";
import { Button } from "../ui/Button";
import { editedText, kindText } from "./words";

/** 成稿：你自己的一篇篇，改过的在前。点进去写；新写一篇先叫「没起名的一篇」，写着写着再起名。 */
export function DraftsPage() {
  const drafts = useDrafts();
  const create = useNewDraft();
  const navigate = useNavigate();
  const missing = isNotServed(drafts.error);
  return (
    <div className="relative h-full overflow-hidden bg-bg">
      <Rail label={t("common.nav.main")} />
      <main className="absolute inset-y-0 right-0 left-rail overflow-y-auto px-6 pt-7 pb-16 lg:px-gutter">
        <header className="flex max-w-[760px] flex-wrap items-start justify-between gap-4">
          <div className="min-w-0">
            <h1 className="m-0 text-title font-semibold text-text">{t("drafts.title")}</h1>
            <p className="m-0 mt-1 text-note text-sub">{t("drafts.intro")}</p>
          </div>
          {missing ? null : (
            <Button
              variant="soft"
              size="md"
              isDisabled={create.isPending}
              onPress={() => create.mutate({ title: "", kind: "article" }, { onSuccess: (d) => void navigate({ to: "/drafts/$id", params: { id: d.id } }) })}
            >
              <PenLine size={15} aria-hidden="true" />
              {t("drafts.new")}
            </Button>
          )}
        </header>
        {missing ? <NotServedLine className="mt-6">{t("drafts.notServed")}</NotServedLine> : null}
        {drafts.error && !missing ? <ErrorLine error={drafts.error} className="mt-6" /> : null}
        {create.error ? <ErrorLine error={create.error} className="mt-3" /> : null}
        {drafts.data && drafts.data.length === 0 ? <p className="m-0 mt-8 text-body text-sub">{t("drafts.empty")}</p> : null}
        <ul className="m-0 mt-6 flex max-w-[760px] list-none flex-col border-t border-border p-0">
          {(drafts.data ?? []).map((d) => (
            <li key={d.id} className="border-b border-border">
              <Link to="/drafts/$id" params={{ id: d.id }} className="press -mx-2 my-1 flex items-baseline gap-4 rounded-btn px-2 py-2.5 no-underline hover:bg-chip/60">
                <span className="min-w-0 flex-1 truncate text-body font-medium text-text">{d.title}</span>
                <span className="shrink-0 text-small text-sub">{kindText(d.kind)}</span>
                <span className="w-24 shrink-0 text-right text-small text-faint">{editedText(d.updatedAt)}</span>
              </Link>
            </li>
          ))}
        </ul>
      </main>
    </div>
  );
}
