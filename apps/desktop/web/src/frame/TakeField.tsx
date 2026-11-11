import { Link2 } from "lucide-react";
import { useEffect, useState, type ClipboardEvent, type KeyboardEvent } from "react";
import { Input, Label, TextField } from "react-aria-components";
import { useTake } from "../api/hooks";
import { ErrorLine } from "../components/ErrorLine";
import { useUiStore } from "../store/ui";

/** 左栏顶上的「粘贴链接，收下」：粘贴或回车就收下，下面那行字换成「收下了」一秒。 */
export function TakeField({ note = "也可以把链接拖到地图上" }: { note?: string }) {
  const [value, setValue] = useState("");
  const [toast, setToast] = useState<{ text: string; n: number } | null>(null);
  const take = useTake();
  const markLanding = useUiStore((s) => s.markLanding);

  useEffect(() => {
    if (!toast) return undefined;
    const id = setTimeout(() => setToast(null), 1120);
    return () => clearTimeout(id);
  }, [toast]);

  function submit(raw: string) {
    const input = raw.trim();
    if (input === "") return;
    setValue("");
    take.mutate(
      { input, kind: "link" },
      {
        onSuccess: (res) => {
          if (!res.seen) markLanding(res.id);
          setToast((t) => ({ text: res.seen ? "以前收下过，补了一次快照" : "收下了", n: (t?.n ?? 0) + 1 }));
        },
      },
    );
  }

  return (
    <div>
      <TextField value={value} onChange={setValue} aria-label="收下一条链接" className="mt-[22px]">
        <Label className="flex h-field items-center gap-2 rounded-input border border-border2 bg-bg px-3 text-sub">
          <Link2 size={15} aria-hidden="true" className="shrink-0" />
          <span className="sr-only-text">收下一条链接</span>
          <Input
            placeholder="粘贴链接，收下"
            className="h-ctl min-w-0 flex-1 border-0 bg-transparent p-0 text-note text-text outline-none"
            onPaste={(e: ClipboardEvent<HTMLInputElement>) => {
              const text = e.clipboardData.getData("text");
              if (!text) return;
              e.preventDefault();
              submit(text);
            }}
            onKeyDown={(e: KeyboardEvent<HTMLInputElement>) => {
              if (e.key === "Enter") submit(value);
            }}
          />
        </Label>
      </TextField>
      <div className="relative mt-2 h-[18px] px-1 text-small">
        <div
          className={`absolute inset-x-1 top-0 text-faint transition-opacity ${toast || take.error ? "opacity-0 duration-[120ms]" : "opacity-100 duration-[180ms]"}`}
        >
          {note}
        </div>
        <div
          role="status"
          className={`absolute inset-x-1 top-0 font-medium text-text transition-[opacity,translate] motion-reduce:translate-y-0 ${toast ? "translate-y-0 opacity-100 duration-[120ms]" : "translate-y-[3px] opacity-0 duration-[180ms]"}`}
        >
          {toast?.text}
        </div>
        {take.error && !toast ? (
          <div className="absolute inset-x-1 top-0">
            <ErrorLine error={take.error} />
          </div>
        ) : null}
      </div>
    </div>
  );
}
