import { RefreshCw } from "lucide-react";
import { useQueryClient } from "@tanstack/react-query";
import { useEffect, useRef, type ReactNode } from "react";
import {
  Menu,
  MenuItem,
  MenuTrigger,
  Popover,
  Text,
  ToggleButton,
  ToggleButtonGroup,
  Button as AriaButton,
  type Key,
} from "react-aria-components";
import type { Pending, TransformJob, TransformLayer, TransformPlugin, TransformsView } from "../api/endpoints";
import { queryKeys, useModelChange, useTransform, useTransforms } from "../api/hooks";
import { ErrorLine } from "../components/ErrorLine";
import { Button } from "../ui/Button";
import { MonoLabel } from "../ui/Chip";
import { FlashLine, useFlash } from "../ui/Flash";
import { JOB_TEXT } from "../plugins/words";
import { copyLabel, FACE_NAME, pendingText, sourceText, type CopyFacet } from "./faces";

/**
 * 一面的头：名字、这一份从哪来；有几份时一排切换（不藏进设置里）；量出来的明显的事；「再转一次」「换一个转」。
 */
export function FaceHead({
  layer,
  copies,
  shown,
  onShow,
  extra,
  itemId,
  busy,
}: {
  layer: TransformLayer;
  copies: CopyFacet[];
  /** 现在摆出来的那一份。 */
  shown: CopyFacet | undefined;
  onShow: (key: string) => void;
  extra?: ReactNode;
  itemId: string;
  /** 这一层正在转：不给再点。 */
  busy: boolean;
}) {
  return (
    <div className="flex flex-col gap-1.5">
      <div className="flex min-h-ctl-xs flex-wrap items-center gap-x-2 gap-y-1">
        <h3 className="m-0 text-note font-semibold text-text">{FACE_NAME[layer]}</h3>
        {shown ? <MonoLabel>{shown.segments.length} 段</MonoLabel> : null}
        <span className="flex-1" />
        <RedoMenu jobs={[layer]} itemId={itemId} busy={busy} label={FACE_NAME[layer]} />
      </div>
      {copies.length > 1 ? (
        <div className="flex flex-wrap items-center gap-1">
          <ToggleButtonGroup
            aria-label={`${FACE_NAME[layer]}的几份`}
            selectionMode="single"
            disallowEmptySelection
            selectedKeys={shown ? [shown.key] : []}
            onSelectionChange={(keys: Set<Key>) => {
              const [k] = [...keys];
              if (typeof k === "string") onShow(k);
            }}
            className="flex flex-wrap gap-1"
          >
            {copies.map((c) => (
              <ToggleButton
                key={c.key}
                id={c.key}
                className="press h-6 cursor-pointer rounded-full border border-border bg-transparent px-2.5 font-sans text-small text-sub outline-none data-[focus-visible]:outline-2 data-[focus-visible]:outline-accent data-[hovered]:text-text data-[selected]:border-label data-[selected]:bg-chip data-[selected]:text-text"
              >
                {copyLabel(c, copies)}
              </ToggleButton>
            ))}
          </ToggleButtonGroup>
          {extra}
        </div>
      ) : null}
      {shown ? (
        <p className="m-0 text-small text-faint">
          {sourceText(layer, shown.source)}
          {shown.remark ? <span className="text-label">　·　{shown.remark}</span> : null}
        </p>
      ) : null}
    </div>
  );
}

/** 一样转换能用哪些插件：`jobs` 里有它的，照 `/transforms` 的顺序；第一个是内核不点名时用的那个。 */
function ableFor(job: TransformJob, transforms: TransformsView | undefined): TransformPlugin[] {
  return transforms?.plugins.filter((p) => p.jobs.includes(job)) ?? [];
}

/** 只差模型没下的插件也能点：内核把这一面挂到「要先下载模型」上，那一句旁边有「下载」。 */
const usable = (p: TransformPlugin) => p.health === "ok" || p.models.some((m) => m.state !== "present");

/**
 * 「再转」：再做一次（不点名插件，内核用它默认的那个），有别的插件能做时「换一个转」。
 * 转出文字的三层做好了多一份、原来的留着；画面的两样换成新做的那份。没装能做它的插件时不出现。
 */
export function RedoMenu({ jobs, itemId, busy, label }: { jobs: TransformJob[]; itemId: string; busy: boolean; label: string }) {
  const transforms = useTransforms();
  const redo = useTransform(itemId);
  const flash = useFlash(2400);
  const choices = jobs.flatMap((job) =>
    ableFor(job, transforms.data).map((p, i) => ({ id: `${job} ${p.id}`, job, plugin: p, again: i === 0 })),
  );
  if (choices.length === 0) return null;
  const text = (c: (typeof choices)[number]) => {
    const name = `「${c.plugin.name}」`;
    if (c.job === "play" || c.job === "keyframes") return c.again ? `重新${JOB_TEXT[c.job]}` : `换成${name}${JOB_TEXT[c.job]}`;
    return c.again ? "再转一次" : `换成${name}转一份`;
  };
  return (
    <span className="flex items-center gap-2">
      <FlashLine flash={flash} className="text-small whitespace-nowrap text-sub" />
      <MenuTrigger>
        <AriaButton
          isDisabled={busy || redo.isPending}
          aria-label={`再转一次${label}，或换一个转`}
          className="press flex h-ctl-xs cursor-pointer items-center gap-1.5 rounded-sm border-0 bg-transparent px-2 font-sans text-small text-sub outline-none data-[disabled]:cursor-default data-[disabled]:opacity-50 data-[focus-visible]:outline-2 data-[focus-visible]:outline-accent data-[hovered]:bg-chip data-[hovered]:text-text"
        >
          <RefreshCw size={12} aria-hidden="true" />
          <span>再转</span>
        </AriaButton>
        <Popover placement="bottom end" offset={6} className="max-w-[min(320px,calc(100vw-32px))] min-w-[240px] animate-rise rounded-card border border-border2 bg-card p-1.5 shadow-panel">
          <Menu
            aria-label={`用哪个转${label}`}
            className="outline-none"
            disabledKeys={choices.filter((c) => !usable(c.plugin)).map((c) => c.id)}
            onAction={(key) => {
              const c = choices.find((x) => x.id === key);
              if (!c) return;
              const kept = c.job === "play" || c.job === "keyframes" ? "排上了；做好了换成新的" : "排上了；转好了多一份，原来的留着";
              redo.mutate(c.again ? { job: c.job } : { job: c.job, plugin: c.plugin.id }, { onSuccess: () => flash.show(kept) });
            }}
          >
            {choices.map((c) => (
              <MenuItem
                key={c.id}
                id={c.id}
                textValue={text(c)}
                className="flex cursor-pointer flex-col rounded-btn px-3 py-2 outline-none data-[disabled]:cursor-default data-[focused]:bg-chip"
              >
                <Text slot="label" className={`text-note ${usable(c.plugin) ? "text-text" : "text-sub"}`}>
                  {text(c)}
                </Text>
                <Text slot="description" className="text-small text-faint">
                  {c.again ? `用「${c.plugin.name}」` : JOB_TEXT[c.job]}
                  {c.plugin.health === "ok" ? "" : `　·　${c.plugin.hint ?? "这个插件现在用不了"}`}
                </Text>
              </MenuItem>
            ))}
          </Menu>
        </Popover>
      </MenuTrigger>
      {redo.error ? <ErrorLine error={redo.error} /> : null}
    </span>
  );
}

/** 还没转出来的那一面：一句实话；等模型的旁边是「下载」和许可，没转出来的是「再试」（这一条没做好的都挪到最前）。 */
export function PendingLine({ pending, itemId }: { pending: Pending; itemId: string }) {
  const transforms = useTransforms();
  const models = useModelChange();
  const retry = useTransform(itemId);
  const m = pending.model;
  const downloading =
    m !== undefined && transforms.data?.plugins.find((p) => p.id === m.plugin)?.models.find((x) => x.name === m.name)?.state === "downloading";
  // 模型下好了，这一条接着转：重拉一次，看到「在转」。
  const queryClient = useQueryClient();
  const was = useRef(downloading);
  useEffect(() => {
    if (was.current && !downloading) void queryClient.invalidateQueries({ queryKey: queryKeys.item(itemId) });
    was.current = downloading;
  }, [downloading, itemId, queryClient]);
  return (
    <div role="status" className="flex flex-wrap items-center gap-x-3 gap-y-1.5 py-2">
      <span className={`text-note ${pending.state === "failed" ? "text-warn" : "text-label"}`}>{pendingText(pending, downloading)}</span>
      {m && pending.state === "needs-model" && !downloading ? (
        <Button
          variant="soft"
          size="xs"
          isDisabled={models.isPending}
          onPress={() => models.mutate({ kind: "download", pluginId: m.plugin, name: m.name })}
        >
          下载
        </Button>
      ) : null}
      {m && pending.state === "needs-model" ? <span className="text-small text-faint">许可：{m.license}</span> : null}
      {pending.state === "failed" ? (
        <Button variant="outline" size="xs" isDisabled={retry.isPending} onPress={() => retry.mutate({ jobs: "pending" })}>
          再试
        </Button>
      ) : null}
      {models.error ? <ErrorLine error={models.error} /> : null}
      {retry.error ? <ErrorLine error={retry.error} /> : null}
    </div>
  );
}
