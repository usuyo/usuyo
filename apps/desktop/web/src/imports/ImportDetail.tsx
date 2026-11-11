import { Link } from "@tanstack/react-router";
import { Check } from "lucide-react";
import { useState } from "react";
import { Checkbox } from "react-aria-components";
import type { ImportEntry, ImportView } from "../api/endpoints";
import { useImport, useImportChange, useImportEntries, usePlugins } from "../api/hooks";
import { ErrorLine } from "../components/ErrorLine";
import { dayText } from "../item/format";
import { timeText } from "../plugins/words";
import { Button } from "../ui/Button";
import { NotStored } from "./NotStored";
import { waitingText } from "./words";

const FIRST = 8;
const MORE = 40;

/** 一次导入：在列 → 列完给你看 → 你点了按节奏收 → 收完；随时能停、能接着。 */
export function ImportDetail({ id, onGone }: { id: string; onGone: () => void }) {
  const view = useImport(id);
  const plugins = usePlugins();
  if (view.error) return <ErrorLine error={view.error} />;
  const v = view.data;
  if (!v) return null;
  const plugin = plugins.data?.plugins.find((p) => p.id === v.pluginId);
  const platform = plugin?.platforms[0]?.name;
  const from =
    v.kind === "delivered" ? `交进来的${v.client ? ` · ${v.client}` : ""}` : `${plugin?.name ?? v.pluginId ?? ""} · 从插件列出来的`;

  return (
    <div className="animate-fade">
      <div className="flex items-baseline gap-2.5">
        <h2 className="m-0 min-w-0 truncate text-section font-semibold text-text">{v.title}</h2>
        <span className="mono shrink-0 text-tag text-faint">
          {from} · {dayText(v.createdAt)}
        </span>
      </div>
      {v.state === "listing" ? <Listing v={v} /> : null}
      {v.state === "ready" ? <Ready v={v} platform={platform} /> : null}
      {v.state === "taking" || v.state === "held" || v.state === "done" ? <Running v={v} platform={platform} /> : null}
      <Discard id={v.id} onGone={onGone} />
    </div>
  );
}

/** 在列：一页页列，列到的条目跟着出现。 */
function Listing({ v }: { v: ImportView }) {
  const n = v.preview.new + v.preview.stored + v.preview.trashed;
  return (
    <>
      <p className="m-0 mt-5 text-title font-semibold text-text">在列：列了 {v.pages} 页，{n} 条</p>
      <p className="m-0 mt-1 text-note text-sub">按平台的节奏一页页列，列完给你看一眼，你点了才收。</p>
      {v.waiting ? <p className="m-0 mt-1 text-small text-sub">{waitingText(v.waiting, undefined)}</p> : null}
      <Group v={v} standing="new" title="新的" note="资料库里还没有" />
      <Group v={v} standing="stored" title="已经存着的" note="不动" />
      <Group v={v} standing="trashed" title="在回收站里的" note="你删过的" />
    </>
  );
}

/** 列完了：分成新的、已经存着的、在回收站里的；回收站里的你勾了才拿回来。 */
function Ready({ v, platform }: { v: ImportView; platform: string | undefined }) {
  const change = useImportChange(v.id);
  const [alsoTrashed, setAlsoTrashed] = useState(false);
  const total = v.preview.new + (alsoTrashed ? v.preview.trashed : 0);
  return (
    <>
      <div className="mt-5 flex items-start gap-4">
        <div className="min-w-0 flex-1">
          <p className="m-0 text-title font-semibold text-text">列完了：{v.preview.new + v.preview.stored + v.preview.trashed} 条</p>
          <p className="m-0 mt-1 text-note text-sub">
            这次会收下新的 {v.preview.new} 条{alsoTrashed && v.preview.trashed > 0 ? `，再把回收站里的 ${v.preview.trashed} 条拿回来` : ""}。
            {v.preview.stored > 0 ? `已经存着的 ${v.preview.stored} 条不动。` : ""}
          </p>
          <p className="m-0 mt-1 text-small text-sub">
            收的时候照{platform ? `${platform}的` : "平台的"}节奏一条条来，不用守着；随时能停。
          </p>
        </div>
        <Button
          variant="soft"
          size="md"
          isDisabled={change.isPending || total === 0}
          onPress={() => change.mutate({ kind: "confirm", body: alsoTrashed ? { alsoTrashed: true } : {} })}
        >
          {total > 0 ? `开始收下 ${total} 条` : "没有要收的"}
        </Button>
      </div>
      {change.error ? <ErrorLine error={change.error} className="mt-2" /> : null}
      <Group v={v} standing="new" title="新的" note="资料库里还没有，会收下" />
      <Group
        v={v}
        standing="trashed"
        title="在回收站里的"
        note={alsoTrashed ? "你勾了，会拿回来" : "你删过，在回收站里；不勾就不收"}
        choice={{ on: alsoTrashed, set: setAlsoTrashed }}
      />
      <Group v={v} standing="stored" title="已经存着的" note="已经存着，这次不动" />
      <Group v={v} standing="unresolved" title="认不出的" note="插件认不出这些链接是哪一条，不收" />
    </>
  );
}

/** 按节奏收、停着、收完：只说收下了几条、为什么在等、下一条大概什么时候、哪些没存下。 */
function Running({ v, platform }: { v: ImportView; platform: string | undefined }) {
  const change = useImportChange(v.id);
  return (
    <>
      <div className="mt-5 flex items-start gap-4">
        <div className="min-w-0 flex-1">
          <p className="m-0 text-big font-semibold text-text">
            收下了 <span className="mono">{v.taken}</span> 条
          </p>
          <div className="mt-1.5 flex flex-col gap-0.5 text-note text-sub">
            {v.state === "done" ? <span>这次的都收完了。</span> : null}
            {v.state === "held" ? <span>停着。点「接着导入」从停下的地方接着收。</span> : null}
            {v.state === "taking" && v.waiting ? <span>{waitingText(v.waiting, platform)}</span> : null}
            {v.state === "taking" && !v.waiting && v.nextAt ? <span>下一条大概 {timeText(v.nextAt)}</span> : null}
            {v.skipped > 0 ? <span>{v.skipped} 条轮到时已经在回收站里、或者是私密的，没收。</span> : null}
          </div>
        </div>
        {v.state === "taking" ? (
          <Button variant="outline" size="md" isDisabled={change.isPending} onPress={() => change.mutate({ kind: "hold" })}>
            先停下
          </Button>
        ) : null}
        {v.state === "held" ? (
          <Button variant="soft" size="md" isDisabled={change.isPending} onPress={() => change.mutate({ kind: "resume" })}>
            接着导入
          </Button>
        ) : null}
      </div>
      {change.error ? <ErrorLine error={change.error} className="mt-2" /> : null}
      {v.notStored > 0 ? <NotStored v={v} /> : null}
      <Taken v={v} />
    </>
  );
}

/** 最近收下的几条，点开是条目页。 */
function Taken({ v }: { v: ImportView }) {
  const entries = useImportEntries(v.id, "new", 200, v.taken > 0, v.state === "taking");
  const taken = (entries.data ?? []).filter((e) => e.taken).slice(-6).reverse();
  if (taken.length === 0) return null;
  return (
    <section className="mt-7">
      <h3 className="m-0 text-body font-semibold text-text">最近收下的</h3>
      <ul className="m-0 mt-2 flex list-none flex-col border-t border-border p-0">
        {taken.map((e) => (
          <EntryRow key={e.seq} e={e} />
        ))}
      </ul>
    </section>
  );
}

function EntryRow({ e }: { e: ImportEntry }) {
  const host = URL.canParse(e.url) ? new URL(e.url).host : e.url;
  const body = (
    <>
      <span className="min-w-0 flex-1 truncate text-body text-text">{e.title || e.url}</span>
      <span className="mono shrink-0 text-tag text-faint">
        {e.actedAt ? `${dayText(e.actedAt)} · ` : ""}
        {host}
      </span>
    </>
  );
  return (
    <li className="border-b border-border">
      {e.itemId && (e.taken || e.standing === "stored") ? (
        <Link
          to="/items/$id"
          params={{ id: e.itemId }}
          className="press -mx-2 flex items-center gap-3 rounded-btn px-2 py-2 no-underline hover:bg-chip/60"
        >
          {body}
        </Link>
      ) : (
        <div className="flex items-center gap-3 py-2">{body}</div>
      )}
    </li>
  );
}

/** 预览里的一组。有 `choice` 的那组整组勾选（内核按组收，见 ConfirmImportRequest）。 */
function Group({
  v,
  standing,
  title,
  note,
  choice,
}: {
  v: ImportView;
  standing: ImportEntry["standing"];
  title: string;
  note: string;
  choice?: { on: boolean; set: (on: boolean) => void };
}) {
  const [limit, setLimit] = useState(FIRST);
  const count = standing === "unresolved" ? undefined : v.preview[standing];
  const entries = useImportEntries(v.id, standing, limit, count === undefined || count > 0, v.state === "listing");
  const list = entries.data ?? [];
  if (count === 0 || (count === undefined && list.length === 0)) return null;
  const shown = count ?? list.length;
  return (
    <section className="mt-7">
      <div className="flex items-center gap-2.5">
        <h3 className="m-0 text-body font-semibold text-text">{title}</h3>
        <span className="mono text-tag text-faint">{shown} 条</span>
        <span className="min-w-0 flex-1 truncate text-small text-sub">{note}</span>
        {choice ? (
          <Checkbox
            isSelected={choice.on}
            onChange={choice.set}
            className="group press flex h-ctl-xs cursor-pointer items-center gap-2 rounded-sm border border-border2 px-2.5 text-small text-label outline-none data-[focus-visible]:outline-2 data-[focus-visible]:outline-accent"
          >
            <span className="flex size-3.5 items-center justify-center rounded-[4px] border border-border2 group-data-[selected]:border-label group-data-[selected]:bg-label">
              <Check size={11} strokeWidth={3} aria-hidden="true" className="hidden text-bg group-data-[selected]:block" />
            </span>
            这一组也拿回来
          </Checkbox>
        ) : null}
      </div>
      <ul className="m-0 mt-2 flex list-none flex-col border-t border-border p-0">
        {list.map((e) => (
          <EntryRow key={e.seq} e={e} />
        ))}
      </ul>
      {list.length < shown ? (
        <div className="mt-1.5 flex items-center gap-3 text-small text-faint">
          <span>列出了其中 {list.length} 条</span>
          <Button variant="quiet" size="xs" onPress={() => setLimit(limit + MORE)}>
            再看 {Math.min(MORE, shown - list.length)} 条
          </Button>
        </div>
      ) : null}
    </section>
  );
}

/** 不要这次导入了：清单和还没收的去掉，已经收下的留着。先问一句。 */
function Discard({ id, onGone }: { id: string; onGone: () => void }) {
  const change = useImportChange(id);
  const [asking, setAsking] = useState(false);
  return (
    <div className="mt-10 flex items-center gap-3 border-t border-border pt-4">
      {asking ? (
        <>
          <span className="text-small text-sub">不要这次导入了？清单和还没收的去掉，已经收下的留着。</span>
          <Button variant="outline" size="xs" isDisabled={change.isPending} onPress={() => change.mutate({ kind: "delete" }, { onSuccess: onGone })}>
            不要了
          </Button>
          <Button variant="quiet" size="xs" onPress={() => setAsking(false)}>
            算了
          </Button>
        </>
      ) : (
        <Button variant="quiet" size="xs" onPress={() => setAsking(true)}>
          不要这次导入了
        </Button>
      )}
      {change.error ? <ErrorLine error={change.error} /> : null}
    </div>
  );
}
