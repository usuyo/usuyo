import { FileText, Images, RotateCcw } from "lucide-react";
import { useState } from "react";
import type { TrashEntry } from "../api/endpoints";
import { useItemChange, usePurge, useTrash } from "../api/hooks";
import { ErrorLine } from "../components/ErrorLine";
import { Rail } from "../frame/Rail";
import { Button } from "../ui/Button";
import { askText, clearsText, doneText, introText, trashedText, whatText } from "./words";

/** 回收站：删掉的整条和只留文字移出来的媒体，各自拿回来或立即清除。 */
export function TrashPage() {
  const trash = useTrash();
  const [said, setSaid] = useState({ text: "", token: 0 });
  const say = (text: string) => setSaid((s) => ({ text, token: s.token + 1 }));
  const entries = trash.data?.entries ?? [];

  return (
    <div className="relative h-full overflow-hidden bg-bg">
      <Rail label="主导航" />
      <main className="absolute inset-y-0 right-0 left-rail overflow-y-auto">
        <div className="px-gutter pt-7 pb-10">
          <header className="max-w-[760px]">
            <h1 className="m-0 text-title font-semibold text-text">回收站</h1>
            {trash.data ? <p className="m-0 mt-1 text-note text-sub">{introText(trash.data.trashDays)}</p> : null}
            <p className="m-0 mt-0.5 text-small text-faint">再收下同一条链接，它会从这里回到地图上。</p>
          </header>
          {trash.error ? <ErrorLine error={trash.error} className="mt-4" /> : null}
          <div role="status" className="mt-4 flex min-h-5 items-center text-note font-medium text-text">
            {said.text ? (
              <span key={said.token} className="animate-rise">
                {said.text}
              </span>
            ) : null}
          </div>
          {trash.data && entries.length === 0 ? (
            <div className="animate-rise pt-20 text-center">
              <div className="text-body text-label">回收站里没有东西。</div>
              <div className="mt-1 text-note text-sub">
                {trash.data.trashDays === null ? "删掉的东西会先放在这里。" : `删掉的东西会先在这里放 ${trash.data.trashDays} 天。`}
              </div>
            </div>
          ) : (
            <ul aria-label="回收站里的东西" className="m-0 mt-2 list-none border-t border-border p-0">
              {entries.map((e) => (
                <TrashRow key={e.id} entry={e} onDone={say} />
              ))}
            </ul>
          )}
        </div>
      </main>
    </div>
  );
}

/** 一行：左边是哪一条，右边是什么时候进来、什么时候清除和两个动作；窄了右边那块换到下一行。 */
function TrashRow({ entry, onDone }: { entry: TrashEntry; onDone: (text: string) => void }) {
  const restore = useItemChange(entry.itemId);
  const purge = usePurge();
  const [asking, setAsking] = useState(false);
  const busy = restore.isPending || purge.isPending;
  const media = entry.reason === "keep-text";
  const Icon = media ? Images : FileText;
  const error = restore.error ?? purge.error;

  return (
    <li className="flex min-h-16 animate-rise flex-wrap items-center gap-x-3 gap-y-2 border-b border-border px-3 py-2.5 hover:bg-card">
      <div className="flex min-w-0 flex-[1_1_280px] items-center gap-3">
        <Icon size={16} aria-hidden="true" className="shrink-0 text-sub" />
        <div className="min-w-0">
          <div className="truncate text-body font-medium text-text">{entry.title}</div>
          <div className="mt-0.5 text-small text-sub">{whatText(entry)}</div>
          {error ? <ErrorLine error={error} /> : null}
        </div>
      </div>
      {asking ? (
        <div className="flex min-w-0 flex-[1_1_360px] animate-rise flex-wrap items-center justify-end gap-2">
          <span className="min-w-0 flex-[1_1_220px] text-right text-small text-label">{askText(entry)}</span>
          <div className="flex flex-none gap-2">
            <Button
              variant="strong"
              size="sm"
              isDisabled={busy}
              onPress={() => purge.mutate(entry.id, { onSuccess: () => onDone(doneText(entry, "purge")) })}
            >
              清除，不能恢复
            </Button>
            <Button size="sm" isDisabled={busy} onPress={() => setAsking(false)}>
              算了
            </Button>
          </div>
        </div>
      ) : (
        <div className="ml-auto flex flex-none flex-wrap items-center justify-end gap-x-3 gap-y-2">
          <div className="min-w-[120px] text-small">
            <div className="text-sub">{trashedText(entry)}</div>
            <div className="text-faint">{clearsText(entry)}</div>
          </div>
          <div className="flex min-w-[204px] justify-end gap-2">
            <Button
              size="sm"
              isDisabled={busy}
              aria-label={`${media ? "拿回图和视频：" : "拿回来："}「${entry.title}」`}
              onPress={() =>
                restore.mutate({ kind: media ? "restore-media" : "restore" }, { onSuccess: () => onDone(doneText(entry, "restore")) })
              }
            >
              <RotateCcw size={14} aria-hidden="true" />
              <span>{media ? "拿回图和视频" : "拿回来"}</span>
            </Button>
            <Button size="sm" isDisabled={busy} aria-label={`立即清除「${entry.title}」`} onPress={() => setAsking(true)}>
              立即清除
            </Button>
          </div>
        </div>
      )}
    </li>
  );
}
