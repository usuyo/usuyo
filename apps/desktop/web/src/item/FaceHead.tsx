import { RefreshCw } from "lucide-react";
import { useQueryClient } from "@tanstack/react-query";
import { useEffect, useRef, type ReactNode } from "react";
import {
  Menu,
  MenuItem,
  MenuTrigger,
  Popover,
  ToggleButton,
  ToggleButtonGroup,
  Button as AriaButton,
  type Key,
} from "react-aria-components";
import type { Pending, TransformLayer } from "../api/endpoints";
import { queryKeys, useModelChange, useTransform, useTransforms } from "../api/hooks";
import { ErrorLine } from "../components/ErrorLine";
import { Button } from "../ui/Button";
import { MonoLabel } from "../ui/Chip";
import { FlashLine, useFlash } from "../ui/Flash";
import { copyLabel, FACE_NAME, modelLabel, pendingText, sourceText, type CopyFacet } from "./faces";

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
        <RedoMenu layer={layer} itemId={itemId} busy={busy} current={shown?.source.tool} />
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

/** 再转一次、换一个插件转。内核还没有转换接口时不出现。 */
function RedoMenu({ layer, itemId, busy, current }: { layer: TransformLayer; itemId: string; busy: boolean; current?: string }) {
  const transforms = useTransforms();
  const redo = useTransform(itemId);
  const flash = useFlash(2400);
  const able = transforms.data?.plugins.filter((p) => p.jobs.includes(layer)) ?? [];
  if (able.length === 0) return null;
  // 先放转出这一份的那个工具（再转一次），再放别的（换一个转）。
  const sorted = [...able].sort((a, b) => Number(b.tool === current) - Number(a.tool === current));
  return (
    <span className="flex items-center gap-2">
      <FlashLine flash={flash} className="text-small whitespace-nowrap text-sub" />
      <MenuTrigger>
        <AriaButton
          isDisabled={busy || redo.isPending}
          aria-label={`再转一次${FACE_NAME[layer]}，或换一个转`}
          className="press flex h-ctl-xs cursor-pointer items-center gap-1.5 rounded-sm border-0 bg-transparent px-2 font-sans text-small text-sub outline-none data-[disabled]:cursor-default data-[disabled]:opacity-50 data-[focus-visible]:outline-2 data-[focus-visible]:outline-accent data-[hovered]:bg-chip data-[hovered]:text-text"
        >
          <RefreshCw size={12} aria-hidden="true" />
          <span>再转</span>
        </AriaButton>
        <Popover placement="bottom end" offset={6} className="min-w-[240px] animate-rise rounded-card border border-border2 bg-card p-1.5 shadow-panel">
          <Menu
            aria-label={`用哪个转${FACE_NAME[layer]}`}
            className="outline-none"
            disabledKeys={sorted.filter((p) => p.health !== "ok").map((p) => p.id)}
            onAction={(key) => {
              const plugin = sorted.find((p) => p.id === key);
              if (!plugin) return;
              redo.mutate({ job: layer, plugin: plugin.id }, { onSuccess: () => flash.show("排上了；转好了多一份，原来的留着") });
            }}
          >
            {sorted.map((p, i) => (
              <MenuItem
                key={p.id}
                id={p.id}
                textValue={p.tool}
                className="flex cursor-pointer flex-col rounded-btn px-3 py-2 outline-none data-[disabled]:cursor-default data-[focused]:bg-chip"
              >
                <span className={`text-note ${p.health === "ok" ? "text-text" : "text-sub"}`}>
                  {p.tool === current || (current === undefined && i === 0) ? `用 ${p.tool} 再转一次` : `换成 ${p.tool} 转一份`}
                </span>
                <span className="text-small text-faint">{p.health === "ok" ? p.name : (p.hint ?? "这个插件现在用不了")}</span>
              </MenuItem>
            ))}
          </Menu>
        </Popover>
      </MenuTrigger>
      {redo.error ? <ErrorLine error={redo.error} /> : null}
    </span>
  );
}

/** 还没转出来的那一面：一句实话；等模型的旁边是「下载」，没转出来的是「再试」。 */
export function PendingLine({ pending, itemId }: { pending: Pending; itemId: string }) {
  const transforms = useTransforms();
  const models = useModelChange();
  const retry = useTransform(itemId);
  const m = pending.model;
  const job = pending.layer === "video" ? null : pending.layer;
  const label = m ? modelLabel(m.name, transforms.data) : "";
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
      <span className={`text-note ${pending.state === "failed" ? "text-warn" : "text-label"}`}>{pendingText(pending, label, downloading)}</span>
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
      {pending.state === "failed" && job ? (
        <Button variant="outline" size="xs" isDisabled={retry.isPending} onPress={() => retry.mutate({ job })}>
          再试
        </Button>
      ) : null}
      {models.error ? <ErrorLine error={models.error} /> : null}
      {retry.error ? <ErrorLine error={retry.error} /> : null}
    </div>
  );
}
