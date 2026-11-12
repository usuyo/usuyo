import { useState } from "react";
import type { FollowView } from "../api/endpoints";
import { useFollowChange } from "../api/hooks";
import { ErrorLine } from "../components/ErrorLine";
import { Button } from "../ui/Button";
import { KIND_TEXT, rhythmText, takeText } from "./words";

/** 一个跟着收：跟着什么、收什么、多久一次；停下、接着、不再跟着收（收下的都留着）。 */
export function FollowRow({ follow, topicName }: { follow: FollowView; topicName: (id: string) => string }) {
  const change = useFollowChange();
  const [asking, setAsking] = useState(false);
  return (
    <li className="border-b border-border py-3.5 last:border-b-0">
      <div className="flex flex-wrap items-start gap-x-4 gap-y-2">
        <div className="min-w-0 flex-[1_1_260px]">
          <div className="text-body font-medium text-text">{follow.title}</div>
          <div className="mt-0.5 text-small text-sub">
            {KIND_TEXT[follow.kind]}　{follow.pluginName}
            {follow.url ? <span className="mono text-tag text-faint select-text">　{follow.url}</span> : null}
          </div>
          <div className="mt-1.5 text-note text-label">{takeText(follow.take, topicName)}</div>
          <div className={`mt-0.5 text-small ${follow.on && follow.waiting ? "text-warn" : "text-sub"}`}>{rhythmText(follow)}</div>
        </div>
        <div className="flex items-center gap-1.5">
          <Button variant="outline" size="xs" isDisabled={change.isPending} onPress={() => change.mutate({ kind: "patch", id: follow.id, patch: { on: !follow.on } })}>
            {follow.on ? "停下" : "接着跟"}
          </Button>
          <Button variant="ghost" size="xs" isDisabled={change.isPending} onPress={() => setAsking(true)}>
            不再跟着收
          </Button>
        </div>
      </div>
      {asking ? (
        <div role="status" className="mt-2.5 flex flex-wrap items-center gap-2 rounded-btn bg-chip/60 px-3 py-2">
          <span className="text-note text-text">不再跟着收「{follow.title}」？收下的条目都留着。</span>
          <Button variant="strong" size="xs" isDisabled={change.isPending} onPress={() => change.mutate({ kind: "remove", id: follow.id })}>
            不再跟着收
          </Button>
          <Button variant="quiet" size="xs" onPress={() => setAsking(false)}>
            先留着
          </Button>
        </div>
      ) : null}
      {change.error ? <ErrorLine error={change.error} className="mt-2" /> : null}
    </li>
  );
}
