import { Bot, FileInput, RefreshCw, User } from "lucide-react";
import { useState } from "react";
import type { ChangeActor, ChangeBatch, ChangeOp } from "../api/endpoints";
import { useChangeBatch } from "../api/hooks";
import { ErrorLine } from "../components/ErrorLine";
import { Button } from "../ui/Button";
import { actorText, hm, opText, stateText } from "./words";

function ActorIcon({ by }: { by: ChangeActor }) {
  const Icon = by.kind === "agent" ? Bot : by.kind === "readback" ? FileInput : by.kind === "sync" ? RefreshCw : User;
  return <Icon size={14} aria-hidden="true" className="shrink-0 text-sub" />;
}

/** 改之前和改之后，一行一样；没有的那一边不写。 */
function Diff({ op }: { op: ChangeOp }) {
  if (op.before === undefined && op.after === undefined) return null;
  return (
    <dl className="m-0 mt-1.5 grid grid-cols-[2.5rem_minmax(0,1fr)] gap-x-2 gap-y-1 text-note">
      {op.before !== undefined ? (
        <>
          <dt className="mono text-tag leading-5 text-faint">之前</dt>
          <dd className="m-0 text-sub select-text">{op.before === "" ? "（没有）" : op.before}</dd>
        </>
      ) : null}
      {op.after !== undefined && op.op !== "add-to-topic" ? (
        <>
          <dt className="mono text-tag leading-5 text-faint">之后</dt>
          <dd className="m-0 text-text select-text">{op.after}</dd>
        </>
      ) : null}
    </dl>
  );
}

const DECISION_TEXT: Record<NonNullable<ChangeOp["decision"]>, string> = { open: "", accepted: "接受了", declined: "不要了" };

/**
 * 一批改动：谁、几点、经什么、做了哪几处。提议用虚线框着，一处处或整批接受、不要；生效了的整批撤销，撤销过的能重做。
 * 撤销会连带别的事时，先问一句。
 */
export function BatchCard({ batch }: { batch: ChangeBatch }) {
  const act = useChangeBatch(batch.id);
  const [asking, setAsking] = useState(false);
  const proposed = batch.state === "proposed";
  const quiet = batch.state === "undone" || batch.state === "declined";
  return (
    <article
      aria-label={`${hm(batch.at)} ${actorText(batch.by)}`}
      className={`rounded-card border px-4 pt-3 pb-3 ${proposed ? "border-dashed border-border2" : "border-border"} ${quiet ? "bg-transparent" : "bg-card"}`}
    >
      <header className="flex flex-wrap items-center gap-x-2.5 gap-y-1">
        <span className="mono text-tag text-faint">{hm(batch.at)}</span>
        <ActorIcon by={batch.by} />
        <span className="text-note font-medium text-text">{actorText(batch.by)}</span>
        <span className="text-small text-sub">{batch.by.device}　{batch.via}</span>
        <span className={`ml-auto text-small ${proposed ? "text-label" : "text-faint"}`}>{stateText(batch)}</span>
      </header>
      <ul className={`m-0 mt-2 flex list-none flex-col gap-2.5 p-0 ${quiet ? "opacity-60" : ""}`}>
        {batch.ops.map((o) => (
          <li key={o.id} className="border-t border-border pt-2 first:border-t-0 first:pt-0">
            <div className="flex flex-wrap items-baseline gap-x-3 gap-y-1">
              <span className="min-w-0 flex-1 text-body text-label">{opText(o)}</span>
              {o.decision === "open" ? (
                <span className="flex gap-1.5">
                  <Button variant="outline" size="xs" isDisabled={act.isPending} onPress={() => act.mutate({ kind: "accept", ops: [o.id] })}>
                    接受这处
                  </Button>
                  <Button variant="ghost" size="xs" isDisabled={act.isPending} onPress={() => act.mutate({ kind: "decline", ops: [o.id] })}>
                    不要这处
                  </Button>
                </span>
              ) : o.decision ? (
                <span className="text-small text-faint">{DECISION_TEXT[o.decision]}</span>
              ) : null}
            </div>
            <Diff op={o} />
          </li>
        ))}
      </ul>
      {asking && batch.undoNote ? (
        <div role="status" className="mt-3 flex flex-wrap items-center gap-2 rounded-btn bg-chip/60 px-3 py-2">
          <span className="text-note text-text">{batch.undoNote}</span>
          <Button variant="strong" size="xs" isDisabled={act.isPending} onPress={() => act.mutate({ kind: "undo" }, { onSuccess: () => setAsking(false) })}>
            一起撤
          </Button>
          <Button variant="quiet" size="xs" onPress={() => setAsking(false)}>
            先不撤
          </Button>
        </div>
      ) : null}
      <footer className="mt-3 flex flex-wrap items-center gap-2">
        {proposed ? (
          <>
            <Button variant="soft" size="xs" isDisabled={act.isPending} onPress={() => act.mutate({ kind: "accept" })}>
              还没定的都接受
            </Button>
            <Button variant="ghost" size="xs" isDisabled={act.isPending} onPress={() => act.mutate({ kind: "decline" })}>
              还没定的都不要
            </Button>
            <span className="text-small text-faint">一处处定也行；放着不定，它一直在这里。</span>
          </>
        ) : batch.state === "applied" ? (
          <Button variant="ghost" size="xs" isDisabled={act.isPending || asking} onPress={() => (batch.undoNote ? setAsking(true) : act.mutate({ kind: "undo" }))}>
            撤销这一批
          </Button>
        ) : batch.state === "undone" ? (
          <Button variant="ghost" size="xs" isDisabled={act.isPending} onPress={() => act.mutate({ kind: "redo" })}>
            重做这一批
          </Button>
        ) : null}
      </footer>
      {act.error ? <ErrorLine error={act.error} className="mt-2" /> : null}
    </article>
  );
}
