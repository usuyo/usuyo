import { Link } from "@tanstack/react-router";
import { useState } from "react";
import type { ImportView } from "../api/endpoints";
import { useNotStored } from "../api/hooks";
import { ErrorLine } from "../components/ErrorLine";
import { t } from "../i18n";
import { STORED_TEXT } from "../item/format";
import { Button } from "../ui/Button";

/** 只看最前面这么多条新的；更多的在各自的条目页里写着。 */
const LOOK_AT = 500;

/** 收下了、但内容没存下的：几条、为什么。原因是插件给条目写的，在条目页也看得到。 */
export function NotStored({ v }: { v: ImportView }) {
  const [open, setOpen] = useState(false);
  return (
    <section className="mt-6 rounded-card border border-border bg-card px-4 py-3.5">
      <div className="flex items-center gap-3">
        <p className="m-0 min-w-0 flex-1 text-note text-label">{t("imports.notStored.line", { count: v.notStored })}</p>
        <Button variant="quiet" size="xs" onPress={() => setOpen(!open)} aria-expanded={open}>
          {open ? t("common.showLess") : t("imports.notStored.show")}
        </Button>
      </div>
      {open ? <Reasons v={v} /> : null}
    </section>
  );
}

function Reasons({ v }: { v: ImportView }) {
  const items = useNotStored(v.id, v.taken, LOOK_AT);
  if (items.error) return <ErrorLine error={items.error} className="mt-2" />;
  if (!items.data) return <p className="m-0 mt-2 text-small text-faint">{t("imports.notStored.finding")}</p>;
  return (
    <ul className="m-0 mt-2 flex list-none flex-col p-0">
      {items.data.map((it) => (
        <li key={it.id}>
          <Link to="/items/$id" params={{ id: it.id }} className="press -mx-2 block rounded-btn px-2 py-1.5 no-underline hover:bg-chip/60">
            <span className="block truncate text-body text-text">{it.title}</span>
            <span className="block text-small text-warn">{it.missing ?? STORED_TEXT[it.stored]}</span>
          </Link>
        </li>
      ))}
    </ul>
  );
}
