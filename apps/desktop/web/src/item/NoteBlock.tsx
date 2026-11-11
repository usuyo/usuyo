import { useState } from "react";
import { Label, TextArea, TextField } from "react-aria-components";
import type { Item } from "../api/endpoints";
import { useItemChange } from "../api/hooks";
import { Button } from "../ui/Button";
import { FlashLine, useFlash } from "../ui/Flash";

/** 记一句：你自己写的话，是你的东西，用强调色。 */
export function NoteBlock({ item }: { item: Item }) {
  const change = useItemChange(item.id);
  const [editing, setEditing] = useState(false);
  const [draft, setDraft] = useState("");
  const flash = useFlash(1200);
  const note = item.note ?? "";
  const showEditor = editing || note === "";

  function save() {
    const text = draft.trim();
    if (text === "" && note === "") return;
    change.mutate(
      { kind: "note", text },
      {
        onSuccess: () => {
          setEditing(false);
          flash.show(text ? "记下了" : "去掉了这一句");
        },
      },
    );
  }

  return (
    <div className="shrink-0 border-t border-border pt-3.5">
      <div className="flex h-5 items-center gap-2.5">
        <span className="text-tag font-medium text-accent">记一句</span>
        <FlashLine flash={flash} className="text-tag text-sub" />
      </div>
      {showEditor ? (
        <>
          <TextField value={draft} onChange={setDraft} aria-label="记一句" className="mt-2">
            <Label className="sr-only-text">记一句</Label>
            <TextArea
              rows={2}
              autoFocus={editing}
              placeholder="写一句你自己的话"
              className="m-0 block w-full resize-none rounded-btn border border-border2 bg-bg px-3 py-2.5 font-sans text-body text-text outline-none focus:border-label"
            />
          </TextField>
          <div className="mt-2 flex gap-2">
            <Button variant="soft" size="sm" onPress={save} isDisabled={change.isPending}>
              记下
            </Button>
            {note ? (
              <Button variant="quiet" size="sm" onPress={() => setEditing(false)}>
                不改了
              </Button>
            ) : null}
          </div>
        </>
      ) : (
        <div className="mt-1 flex items-start gap-3">
          <p className="m-0 min-w-0 flex-1 pt-1.5 text-section leading-6 font-medium text-accent">{note}</p>
          <Button
            variant="outline"
            size="sm"
            onPress={() => {
              setDraft(note);
              setEditing(true);
            }}
          >
            改一下
          </Button>
        </div>
      )}
    </div>
  );
}
