import { type FormEvent } from "react";
import { Button, Input, Label, TextField } from "react-aria-components";
import { useTake } from "../api/hooks";
import { ErrorLine } from "../components/ErrorLine";
import { useUiStore } from "../store/ui";

export function TakePage() {
  const draft = useUiStore((state) => state.takeDraft);
  const setDraft = useUiStore((state) => state.setTakeDraft);
  const take = useTake();

  function submit(event: FormEvent) {
    event.preventDefault();
    const url = draft.trim();
    if (url === "") return;
    take.mutate({ url }, { onSuccess: () => setDraft("") });
  }

  return (
    <section>
      <h1>收下</h1>
      <form onSubmit={submit} className="row">
        <TextField value={draft} onChange={setDraft} type="url" className="field">
          <Label>链接</Label>
          <Input placeholder="粘贴一条链接" />
        </TextField>
        <Button type="submit" isPending={take.isPending}>
          收下
        </Button>
      </form>
      {take.isSuccess ? <p role="status">收下了</p> : null}
      {take.error ? <ErrorLine error={take.error} /> : null}
    </section>
  );
}
