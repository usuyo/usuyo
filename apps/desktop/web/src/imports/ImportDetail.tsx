import { Link } from "@tanstack/react-router";
import { Check } from "lucide-react";
import { useState } from "react";
import { Checkbox, ToggleButton, ToggleButtonGroup } from "react-aria-components";
import type { ImportEntry, ImportView, VisitorIdentity } from "../api/endpoints";
import { useImport, useImportChange, useImportEntries, usePlugins, useVisitors } from "../api/hooks";
import { ErrorLine } from "../components/ErrorLine";
import { rich, t } from "../i18n";
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
  const methodName = plugin?.methods.find((m) => m.id === v.method)?.name ?? v.method;
  const visitor = visitors.data?.find((x) => plugin?.visitors.includes(x.site))?.state;
  const pluginName = plugin?.name ?? v.pluginId ?? "";
  const from =
    v.kind === "delivered"
      ? v.client
        ? t("imports.detail.deliveredFrom", { client: v.client })
        : t("imports.detail.delivered")
      : v.kind === "file" && v.file
        ? t("imports.detail.file", { reader: v.file.reader, file: v.file.name })
        : methodName
          ? t("imports.detail.plugin", { plugin: pluginName, method: methodName })
          : pluginName;

  return (
    <div className="animate-fade">
      <div className="flex flex-wrap items-baseline gap-x-2.5 gap-y-0.5">
        <h2 className="m-0 max-w-full min-w-0 truncate text-section font-semibold text-text">{v.title}</h2>
        <span className="mono text-tag text-faint">
          {t("imports.detail.meta", { from, day: dayText(v.createdAt) })}
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
          <p className="m-0 mt-5 text-title font-semibold text-text">{t("imports.detail.listingFile", { count: n })}</p>
          <p className="m-0 mt-1 text-note text-sub">{t("imports.detail.listingFileNote")}</p>
        </>
      ) : (
        <>
          <p className="m-0 mt-5 text-title font-semibold text-text">{t("imports.detail.listingSource", { pages: t("imports.detail.pages", { count: v.pages }), count: n })}</p>
          <p className="m-0 mt-1 text-note text-sub">{t("imports.detail.listingSourceNote")}</p>
        </>
      )}
      {v.waiting ? <p className="m-0 mt-1 text-small text-sub">{waitingText(v.waiting, undefined, visitor)}</p> : null}
      <Group v={v} standing="new" title={t("imports.group.newTitle")} note={t("imports.group.newNote")} />
      <Group v={v} standing="stored" title={t("imports.group.storedTitle")} note={t("imports.group.storedNote")} />
      <Group v={v} standing="trashed" title={t("imports.group.trashedTitle")} note={t("imports.group.trashedNote")} />
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
          <p className="m-0 text-title font-semibold text-text">
            {t(v.kind === "file" ? "imports.detail.readyFile" : "imports.detail.readySource", { count: v.preview.new + v.preview.stored + v.preview.trashed })}
          </p>
          <p className="m-0 mt-1 text-note text-sub">{willText(takeNew, bringBack, v.preview.stored)}</p>
          <p className="m-0 mt-1 text-small text-sub">
            {v.kind === "file"
              ? t("imports.detail.takeNoteFile")
              : platform
                ? t("imports.detail.takeNotePlatform", { platform })
                : t("imports.detail.takeNote")}
          </p>
        </div>
        <Button variant="soft" size="md" isDisabled={change.isPending || total === 0} onPress={confirm}>
          {total > 0 ? t("imports.detail.start", { count: total }) : t("imports.detail.nothing")}
        </Button>
      </div>
      {change.error ? <ErrorLine error={change.error} className="mt-2" /> : null}
      <Group v={v} standing="new" title={t("imports.group.newTitle")} note={t("imports.group.newNote")} choice={{ pick: picks.new, set: set("new") }} />
      <Group
        v={v}
        standing="trashed"
        title={t("imports.group.deletedTitle")}
        note={t("imports.group.deletedNote")}
        choice={{ pick: picks.trashed, set: set("trashed") }}
      />
      <Group v={v} standing="stored" title={t("imports.group.storedTitle")} note={t("imports.group.storedReadyNote")} />
      <Group v={v} standing="unresolved" title={t("imports.group.unresolvedTitle")} note={t("imports.group.unresolvedNote")} />
    </>
  );
}

/** 这次会添加的那一句，和已保存不动的那一句。 */
function willText(takeNew: number, bringBack: number, stored: number): string {
  const will =
    takeNew > 0 && bringBack > 0
      ? t("imports.detail.willBoth", { new: takeNew, back: bringBack })
      : takeNew > 0
        ? t("imports.detail.willNew", { count: takeNew })
        : bringBack > 0
          ? t("imports.detail.willBack", { count: bringBack })
          : t("imports.detail.willNone");
  return stored > 0 ? t("imports.detail.twoSentences", { first: will, second: t("imports.detail.storedKept", { count: stored }) }) : will;
}

/** 按节奏收、停着、收完：只说收下了几条、为什么在等、下一条大概什么时候、哪些没存下。 */
function Running({ v, platform, visitor }: { v: ImportView; platform: string | undefined; visitor: VisitorIdentity["state"] | undefined }) {
  const change = useImportChange(v.id);
  return (
    <>
      <div className="mt-5 flex flex-wrap items-start gap-x-4 gap-y-3">
        <div className="min-w-[200px] flex-1">
          <p className="m-0 text-big font-semibold text-text">{rich("imports.detail.taken", { count: v.taken, n: <span className="mono" /> })}</p>
          <div className="mt-1.5 flex flex-col gap-0.5 text-note text-sub">
            {v.state === "done" ? <span>{t("imports.detail.done")}</span> : null}
            {v.state === "held" ? <span>{t("imports.detail.held")}</span> : null}
            {v.state === "taking" && v.waiting ? <span>{waitingText(v.waiting, platform, visitor)}</span> : null}
            {v.state === "taking" && !v.waiting && v.nextAt ? <span>{t("imports.detail.nextAt", { time: timeText(v.nextAt) })}</span> : null}
            {v.skipped > 0 ? <span>{t("imports.detail.skipped", { count: v.skipped })}</span> : null}
          </div>
        </div>
        {v.state === "taking" ? (
          <Button variant="outline" size="md" isDisabled={change.isPending} onPress={() => change.mutate({ kind: "hold" })}>
            {t("imports.detail.pause")}
          </Button>
        ) : null}
        {v.state === "held" ? (
          <Button variant="soft" size="md" isDisabled={change.isPending} onPress={() => change.mutate({ kind: "resume" })}>
            {t("imports.detail.resume")}
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
      <h3 className="m-0 text-body font-semibold text-text">{t("imports.detail.recent")}</h3>
      <ul className="m-0 mt-2 flex list-none flex-col border-t border-border p-0">
        {taken.map((e) => (
          <EntryRow key={e.seq} e={e} />
        ))}
      </ul>
    </section>
  );
}

function EntryRow({ e, tick }: { e: ImportEntry; tick?: { on: boolean; set: (on: boolean) => void } }) {
  // 从文件导入时，只在文件里的那几条没有链接：只写标题，不写主机。
  const host = e.url === undefined ? "" : URL.canParse(e.url) ? new URL(e.url).host : e.url;
  const name = e.title || e.url || t("imports.detail.untitled");
  if (tick) {
    return (
      <li className="border-b border-border">
        <Checkbox
          isSelected={tick.on}
          onChange={tick.set}
          aria-label={t("imports.detail.addOne", { name })}
          className="group press -mx-2 flex cursor-pointer items-center gap-3 rounded-btn px-2 py-2 outline-none data-[focus-visible]:outline-2 data-[focus-visible]:outline-accent data-[hovered]:bg-chip/60"
        >
          <Tick />
          <span className={`min-w-0 flex-1 truncate text-body ${tick.on ? "text-text" : "text-faint"}`}>{name}</span>
          {host ? <span className="mono shrink-0 text-tag text-faint">{host}</span> : null}
        </Checkbox>
      </li>
    );
  }
  const body = (
    <>
      <span className="min-w-0 flex-1 truncate text-body text-text">{name}</span>
      <span className="mono shrink-0 text-tag text-faint">
        {[e.actedAt ? dayText(e.actedAt) : "", host].filter(Boolean).join(" · ")}
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

const WHOLE = ["take", "skip"] as const;

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
          {pick ? t("imports.group.picked", { picked: picked(pick, shown), count: shown }) : t("imports.group.count", { count: shown })}
        </span>
        {choice && pick ? (
          <div className="ml-auto flex items-center gap-1.5">
            <ToggleButtonGroup
              aria-label={t("imports.group.wholeLabel", { title })}
              selectionMode="single"
              disallowEmptySelection
              selectedKeys={whole}
              onSelectionChange={(keys) => {
                const next = [...keys][0];
                if (next === "take" || next === "skip") choice.set({ take: next === "take", except: new Set() });
              }}
              className="flex gap-0.5 rounded-sm border border-border2 p-[2px]"
            >
              {WHOLE.map((id) => (
                <ToggleButton
                  key={id}
                  id={id}
                  className="press h-[22px] cursor-pointer rounded-[5px] border-0 bg-transparent px-2 text-small text-sub outline-none data-[focus-visible]:outline-2 data-[focus-visible]:outline-accent data-[selected]:bg-chip data-[selected]:font-medium data-[selected]:text-text"
                >
                  {id === "take" ? t("imports.group.takeAll") : t("imports.group.takeNone")}
                </ToggleButton>
              ))}
            </ToggleButtonGroup>
            <Button variant="quiet" size="xs" aria-expanded={open} onPress={() => setOpen(!open)}>
              {open ? t("common.showLess") : t("imports.group.pickEach")}
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
          <span>{pick && open ? t("imports.group.shownPicking", { count: list.length }) : t("imports.group.shown", { count: list.length })}</span>
          <Button variant="quiet" size="xs" onPress={() => setLimit(limit + MORE)}>
            {t("imports.group.more", { count: Math.min(MORE, shown - list.length) })}
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
            {done ? t("imports.discard.confirmDone") : t("imports.discard.confirmActive")}
          </span>
          <Button variant="outline" size="xs" isDisabled={change.isPending} onPress={() => change.mutate({ kind: "delete" }, { onSuccess: onGone })}>
            {done ? t("imports.discard.remove") : t("imports.discard.discard")}
          </Button>
          <Button variant="quiet" size="xs" onPress={() => setAsking(false)}>
            {t("common.cancel")}
          </Button>
        </>
      ) : (
        <Button variant="quiet" size="xs" onPress={() => setAsking(true)}>
          {done ? t("imports.discard.removeRecord") : t("imports.discard.discardImport")}
        </Button>
      )}
      {change.error ? <ErrorLine error={change.error} /> : null}
    </div>
  );
}
