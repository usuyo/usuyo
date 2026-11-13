import { PenLine, X } from "lucide-react";
import { useState } from "react";
import { Label, TextArea, TextField } from "react-aria-components";
import type { Citation, PageBlock, PagePatch, WrittenPage } from "../api/endpoints";
import { t } from "../i18n";
import { Button } from "../ui/Button";
import { CitationSlip } from "./Quote";
import { pageAfter, type PageEdit } from "./words";

/**
 * 你写的一页：一段段字，段末带着它引的原话。点「改这段」在原地改，「记下」才交给内核；
 * 引用能从一段里拿掉（原话从页尾去掉，条目不动）。底下「写一段」接在页尾。
 */
export function PageText({
  page,
  busy,
  empty,
  onEdit,
}: {
  page: WrittenPage;
  busy: boolean;
  /** 一段都没有时那一句。 */
  empty: string;
  onEdit: (patch: PagePatch) => void;
}) {
  const [editing, setEditing] = useState<string | null | undefined>(undefined);
  const cites = new Map(page.citations.map((c) => [c.label, c]));
  const apply = (edit: PageEdit) => {
    onEdit(pageAfter(page.blocks, edit));
    setEditing(undefined);
  };

  return (
    <div className="flex flex-col gap-5">
      {page.blocks.length === 0 && editing === undefined ? <p className="m-0 text-body text-sub">{empty}</p> : null}
      {page.blocks.map((b) =>
        editing === b.id ? (
          <Editor key={b.id} initial={b.text} label={b.kind === "heading" ? t("writing.page.editHeading") : t("writing.page.editParagraph")} busy={busy} onSave={(text) => apply({ kind: "text", id: b.id, text })} onCancel={() => setEditing(undefined)} />
        ) : (
          <Block key={b.id} block={b} cites={cites} busy={busy} onEdit={() => setEditing(b.id)} onUncite={(label) => apply({ kind: "uncite", id: b.id, label })} />
        ),
      )}
      {editing === null ? (
        <Editor initial="" label={t("writing.page.newParagraph")} busy={busy} onSave={(text) => apply({ kind: "text", id: null, text })} onCancel={() => setEditing(undefined)} />
      ) : (
        <div>
          <Button variant="outline" size="sm" isDisabled={busy} onPress={() => setEditing(null)}>
            <PenLine size={14} aria-hidden="true" />
            {t("writing.page.addParagraph")}
          </Button>
        </div>
      )}
    </div>
  );
}

function Block({
  block,
  cites,
  busy,
  onEdit,
  onUncite,
}: {
  block: PageBlock;
  cites: Map<string, Citation>;
  busy: boolean;
  onEdit: () => void;
  onUncite: (label: string) => void;
}) {
  const quoted = block.cites.flatMap((l) => {
    const c = cites.get(l);
    return c ? [c] : [];
  });
  return (
    <div className="group/block flex flex-col gap-2">
      {block.text === "" ? (
        <div className="flex justify-end">
          <Button variant="ghost" size="xs" isDisabled={busy} className="opacity-0 group-hover/block:opacity-100 data-[focus-visible]:opacity-100" onPress={onEdit}>
            {t("writing.page.addBeforeQuote")}
          </Button>
        </div>
      ) : (
        <div className="flex items-start gap-3">
          {block.kind === "heading" ? (
            <h3 className="m-0 min-w-0 flex-1 pt-2 text-section font-semibold text-text">{block.text}</h3>
          ) : (
            <p className="m-0 min-w-0 flex-1 text-lede whitespace-pre-wrap text-text select-text">{block.text}</p>
          )}
          <Button
            variant="ghost"
            size="xs"
            isDisabled={busy}
            className="opacity-0 group-hover/block:opacity-100 data-[focus-visible]:opacity-100"
            onPress={onEdit}
          >
            {t("writing.page.edit")}
          </Button>
        </div>
      )}
      {quoted.length > 0 ? (
        <div className="flex flex-col gap-1.5">
          {quoted.map((c) => (
            <CitationSlip
              key={c.label}
              citation={c}
              action={
                <Button
                  variant="ghost"
                  size="xs"
                  square
                  isDisabled={busy}
                  aria-label={t("writing.page.removeQuote", { quote: c.quote })}
                  className="opacity-0 group-hover:opacity-100 data-[focus-visible]:opacity-100"
                  onPress={() => onUncite(c.label)}
                >
                  <X size={13} aria-hidden="true" />
                </Button>
              }
            />
          ))}
        </div>
      ) : null}
    </div>
  );
}

function Editor({ initial, label, busy, onSave, onCancel }: { initial: string; label: string; busy: boolean; onSave: (text: string) => void; onCancel: () => void }) {
  const [text, setText] = useState(initial);
  return (
    <div className="flex flex-col gap-2">
      <TextField value={text} onChange={setText} autoFocus aria-label={label}>
        <Label className="sr-only-text">{label}</Label>
        <TextArea
          rows={Math.max(3, Math.ceil(text.length / 40))}
          onKeyDown={(e) => {
            if (e.key === "Escape") onCancel();
            if (e.key === "Enter" && (e.metaKey || e.ctrlKey)) onSave(text);
          }}
          className="w-full resize-y rounded-input border border-border2 bg-card px-3.5 py-2.5 font-sans text-lede text-text outline-none focus:border-label"
        />
      </TextField>
      <div className="flex items-center gap-2">
        <Button variant="soft" size="sm" isDisabled={busy} onPress={() => onSave(text)}>
          {t("writing.page.save")}
        </Button>
        <Button variant="quiet" size="sm" onPress={onCancel}>
          {t("common.cancel")}
        </Button>
        <span className="text-small text-faint">{t("writing.page.keys")}</span>
      </div>
    </div>
  );
}
