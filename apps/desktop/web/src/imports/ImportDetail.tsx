import { Link } from "@tanstack/react-router";
import { Check } from "lucide-react";
import { useState } from "react";
import { Checkbox, ToggleButton, ToggleButtonGroup } from "react-aria-components";
import type { ImportEntry, ImportView, VisitorIdentity } from "../api/endpoints";
import { useImport, useImportChange, useImportEntries, usePlugins, useVisitors } from "../api/hooks";
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
  const visitors = useVisitors();
  if (view.error) return <ErrorLine error={view.error} />;
  const v = view.data;
  if (!v) return null;
  const plugin = plugins.data?.plugins.find((p) => p.id === v.pluginId);
  const platform = plugin?.platforms[0]?.name;
  const visitor = visitors.data?.find((x) => plugin?.visitors.includes(x.site))?.state;
  const from =
    v.kind === "delivered"
      ? `交进来的${v.client ? ` · ${v.client}` : ""}`
      : v.kind === "file" && v.file
        ? `${v.file.reader} · 读「${v.file.name}」`
        : `${plugin?.name ?? v.pluginId ?? ""} · 从插件列出来的`;

  return (
    <div className="animate-fade">
      <div className="flex flex-wrap items-baseline gap-x-2.5 gap-y-0.5">
        <h2 className="m-0 max-w-full min-w-0 truncate text-section font-semibold text-text">{v.title}</h2>
        <span className="mono text-tag text-faint">
          {from} · {dayText(v.createdAt)}
        </span>
      </div>
      {v.state === "listing" ? <Listing v={v} visitor={visitor} /> : null}
      {v.state === "ready" ? <Ready v={v} platform={platform} /> : null}
      {v.state === "taking" || v.state === "held" || v.state === "done" ? <Running v={v} platform={platform} visitor={visitor} /> : null}
      <Discard id={v.id} done={v.state === "done"} onGone={onGone} />
    </div>
  );
}

/** 在列：一页页列，列到的条目跟着出现。 */
function Listing({ v, visitor }: { v: ImportView; visitor: VisitorIdentity["state"] | undefined }) {
  const n = v.preview.new + v.preview.stored + v.preview.trashed;
  return (
    <>
      {v.kind === "file" ? (
        <>
          <p className="m-0 mt-5 text-title font-semibold text-text">在读：读出 {n} 条</p>
          <p className="m-0 mt-1 text-note text-sub">在这台电脑上读这个文件的副本，不连平台；读完给你看一眼，你点了才收。</p>
        </>
      ) : (
        <>
          <p className="m-0 mt-5 text-title font-semibold text-text">在列：列了 {v.pages} 页，{n} 条</p>
          <p className="m-0 mt-1 text-note text-sub">按平台的节奏一页页列，列完给你看一眼，你点了才收。</p>
        </>
      )}
      {v.waiting ? <p className="m-0 mt-1 text-small text-sub">{waitingText(v.waiting, undefined, visitor)}</p> : null}
      <Group v={v} standing="new" title="新的" note="资料库里还没有" />
      <Group v={v} standing="stored" title="已经存着的" note="不动" />
      <Group v={v} standing="trashed" title="在回收站里的" note="你删过的" />
    </>
  );
}

/** 两组可以挑的：整组一个选择，展开以后逐条改；`except` 是和整组选得不一样的条目。 */
type Pick = { take: boolean; except: Set<number> };
type Choosable = "new" | "trashed";

function picked(pick: Pick, count: number): number {
  return pick.take ? count - pick.except.size : pick.except.size;
}

/** 列完了：分成新的、在回收站里的、已经存着的。新的、回收站里的默认都收，整组或逐条改。 */
function Ready({ v, platform }: { v: ImportView; platform: string | undefined }) {
  const change = useImportChange(v.id);
  const [picks, setPicks] = useState<Record<Choosable, Pick>>({
    new: { take: true, except: new Set() },
    trashed: { take: true, except: new Set() },
  });
  const takeNew = picked(picks.new, v.preview.new);
  const bringBack = picked(picks.trashed, v.preview.trashed);
  const total = takeNew + bringBack;
  const set = (standing: Choosable) => (pick: Pick) => setPicks((p) => ({ ...p, [standing]: pick }));
  const confirm = () =>
    change.mutate({
      kind: "confirm",
      body: { new: picks.new.take, trashed: picks.trashed.take, except: [...picks.new.except, ...picks.trashed.except] },
    });
  return (
    <>
      <div className="mt-5 flex flex-wrap items-start gap-x-4 gap-y-3">
        <div className="min-w-[240px] flex-1">
          <p className="m-0 text-title font-semibold text-text">{v.kind === "file" ? "读完了" : "列完了"}：{v.preview.new + v.preview.stored + v.preview.trashed} 条</p>
          <p className="m-0 mt-1 text-note text-sub">
            {total > 0
              ? `这次会收下${takeNew > 0 ? `新的 ${takeNew} 条` : ""}${takeNew > 0 && bringBack > 0 ? "，" : ""}${bringBack > 0 ? `从回收站拿回 ${bringBack} 条` : ""}。`
              : "这次一条都不收。"}
            {v.preview.stored > 0 ? `已经存着的 ${v.preview.stored} 条不动。` : ""}
          </p>
          <p className="m-0 mt-1 text-small text-sub">
            收的时候照{platform ? `${platform}的` : "平台的"}节奏一条条来，不用守着；随时能停。
          </p>
        </div>
        <Button variant="soft" size="md" isDisabled={change.isPending || total === 0} onPress={confirm}>
          {total > 0 ? `开始收下 ${total} 条` : "没有要收的"}
        </Button>
      </div>
      {change.error ? <ErrorLine error={change.error} className="mt-2" /> : null}
      <Group v={v} standing="new" title="新的" note="资料库里还没有" choice={{ pick: picks.new, set: set("new") }} />
      <Group
        v={v}
        standing="trashed"
        title="你删过的"
        note="在回收站里；收下就从回收站拿回来，不想要就点「全不收」"
        choice={{ pick: picks.trashed, set: set("trashed") }}
      />
      <Group v={v} standing="stored" title="已经存着的" note="已经存着，这次不动" />
      <Group v={v} standing="unresolved" title="认不出的" note="插件认不出这些链接是哪一条，不收" />
    </>
  );
}

/** 按节奏收、停着、收完：只说收下了几条、为什么在等、下一条大概什么时候、哪些没存下。 */
function Running({ v, platform, visitor }: { v: ImportView; platform: string | undefined; visitor: VisitorIdentity["state"] | undefined }) {
  const change = useImportChange(v.id);
  return (
    <>
      <div className="mt-5 flex flex-wrap items-start gap-x-4 gap-y-3">
        <div className="min-w-[200px] flex-1">
          <p className="m-0 text-big font-semibold text-text">
            收下了 <span className="mono">{v.taken}</span> 条
          </p>
          <div className="mt-1.5 flex flex-col gap-0.5 text-note text-sub">
            {v.state === "done" ? <span>这次的都收完了。</span> : null}
            {v.state === "held" ? <span>停着。点「接着导入」从停下的地方接着收。</span> : null}
            {v.state === "taking" && v.waiting ? <span>{waitingText(v.waiting, platform, visitor)}</span> : null}
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

function EntryRow({ e, tick }: { e: ImportEntry; tick?: { on: boolean; set: (on: boolean) => void } }) {
  const host = URL.canParse(e.url) ? new URL(e.url).host : e.url;
  if (tick) {
    return (
      <li className="border-b border-border">
        <Checkbox
          isSelected={tick.on}
          onChange={tick.set}
          aria-label={`收下 ${e.title || e.url}`}
          className="group press -mx-2 flex cursor-pointer items-center gap-3 rounded-btn px-2 py-2 outline-none data-[focus-visible]:outline-2 data-[focus-visible]:outline-accent data-[hovered]:bg-chip/60"
        >
          <Tick />
          <span className={`min-w-0 flex-1 truncate text-body ${tick.on ? "text-text" : "text-faint"}`}>{e.title || e.url}</span>
          <span className="mono shrink-0 text-tag text-faint">{host}</span>
        </Checkbox>
      </li>
    );
  }
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

function Tick() {
  return (
    <span className="flex size-3.5 shrink-0 items-center justify-center rounded-[4px] border border-border2 group-data-[selected]:border-label group-data-[selected]:bg-label">
      <Check size={11} strokeWidth={3} aria-hidden="true" className="hidden text-bg group-data-[selected]:block" />
    </span>
  );
}

const WHOLE = [
  { id: "take", text: "全收" },
  { id: "skip", text: "全不收" },
] as const;

/** 预览里的一组。有 `choice` 的那组可以挑：收起时整组全收或全不收，展开以后逐条勾。 */
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
  choice?: { pick: Pick; set: (pick: Pick) => void };
}) {
  const [limit, setLimit] = useState(FIRST);
  const [open, setOpen] = useState(false);
  const count = standing === "unresolved" ? undefined : v.preview[standing];
  const entries = useImportEntries(v.id, standing, limit, count === undefined || count > 0, v.state === "listing");
  const list = entries.data ?? [];
  if (count === 0 || (count === undefined && list.length === 0)) return null;
  const shown = count ?? list.length;
  const pick = choice?.pick;
  const whole = pick && pick.except.size === 0 ? [pick.take ? "take" : "skip"] : [];
  const tick = (e: ImportEntry) =>
    choice && pick && open
      ? {
          on: pick.take !== pick.except.has(e.seq),
          set: (on: boolean) => {
            const except = new Set(pick.except);
            if (on === pick.take) except.delete(e.seq);
            else except.add(e.seq);
            choice.set({ take: pick.take, except });
          },
        }
      : undefined;
  return (
    <section className="mt-7">
      <div className="flex flex-wrap items-center gap-x-2.5 gap-y-1.5">
        <h3 className="m-0 text-body font-semibold text-text">{title}</h3>
        <span className="mono text-tag text-faint">
          {pick ? `收 ${picked(pick, shown)} / ${shown} 条` : `${shown} 条`}
        </span>
        {choice && pick ? (
          <div className="ml-auto flex items-center gap-1.5">
            <ToggleButtonGroup
              aria-label={`${title}：整组`}
              selectionMode="single"
              disallowEmptySelection
              selectedKeys={whole}
              onSelectionChange={(keys) => {
                const next = [...keys][0];
                if (next === "take" || next === "skip") choice.set({ take: next === "take", except: new Set() });
              }}
              className="flex gap-0.5 rounded-sm border border-border2 p-[2px]"
            >
              {WHOLE.map(({ id, text }) => (
                <ToggleButton
                  key={id}
                  id={id}
                  className="press h-[22px] cursor-pointer rounded-[5px] border-0 bg-transparent px-2 text-small text-sub outline-none data-[focus-visible]:outline-2 data-[focus-visible]:outline-accent data-[selected]:bg-chip data-[selected]:font-medium data-[selected]:text-text"
                >
                  {text}
                </ToggleButton>
              ))}
            </ToggleButtonGroup>
            <Button variant="quiet" size="xs" aria-expanded={open} onPress={() => setOpen(!open)}>
              {open ? "收起" : "逐条挑"}
            </Button>
          </div>
        ) : null}
        <span className="basis-full text-small text-sub">{note}</span>
      </div>
      <ul className="m-0 mt-2 flex list-none flex-col border-t border-border p-0">
        {list.map((e) => (
          <EntryRow key={e.seq} e={e} tick={tick(e)} />
        ))}
      </ul>
      {list.length < shown ? (
        <div className="mt-1.5 flex flex-wrap items-center gap-x-3 text-small text-faint">
          <span>{pick && open ? `列出了其中 ${list.length} 条，没列出的跟着整组` : `列出了其中 ${list.length} 条`}</span>
          <Button variant="quiet" size="xs" onPress={() => setLimit(limit + MORE)}>
            再看 {Math.min(MORE, shown - list.length)} 条
          </Button>
        </div>
      ) : null}
    </section>
  );
}

/**
 * 收完之前是「不要这次导入了」：清单和还没收的去掉，已经收下的留着。
 * 收完以后只剩一条记录，就是「去掉这条记录」。都先问一句。
 */
function Discard({ id, done, onGone }: { id: string; done: boolean; onGone: () => void }) {
  const change = useImportChange(id);
  const [asking, setAsking] = useState(false);
  return (
    <div className="mt-10 flex flex-wrap items-center gap-x-3 gap-y-2 border-t border-border pt-4">
      {asking ? (
        <>
          <span className="text-small text-sub">
            {done ? "去掉这条导入记录？收下的条目都留着。" : "不要这次导入了？清单和还没收的去掉，已经收下的留着。"}
          </span>
          <Button variant="outline" size="xs" isDisabled={change.isPending} onPress={() => change.mutate({ kind: "delete" }, { onSuccess: onGone })}>
            {done ? "去掉" : "不要了"}
          </Button>
          <Button variant="quiet" size="xs" onPress={() => setAsking(false)}>
            算了
          </Button>
        </>
      ) : (
        <Button variant="quiet" size="xs" onPress={() => setAsking(true)}>
          {done ? "去掉这条记录" : "不要这次导入了"}
        </Button>
      )}
      {change.error ? <ErrorLine error={change.error} /> : null}
    </div>
  );
}
