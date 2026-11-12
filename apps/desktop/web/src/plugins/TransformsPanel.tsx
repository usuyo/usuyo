import { Link } from "@tanstack/react-router";
import { useState } from "react";
import { isNotServed } from "../api/client";
import type { TransformModel, TransformPlugin } from "../api/endpoints";
import { useModelChange, useTransforms } from "../api/hooks";
import { ErrorLine, NotServedLine } from "../components/ErrorLine";
import { sizeText } from "../item/faces";
import { Button } from "../ui/Button";

/** 「变成文字」那一栏：装着的转换插件，各把什么转成什么；模型的大小、许可，下载和删掉。 */
export function TransformsPanel() {
  const transforms = useTransforms();
  return (
    <section aria-label="变成文字" className="min-w-0 animate-fade">
      <h2 className="m-0 text-name font-semibold text-text">变成文字</h2>
      <p className="m-0 mt-1.5 max-w-[640px] text-note text-sub">
        把口播、图里的字、画面上的字转出来的插件。都在这台电脑上跑，不把内容发出去。模型只在你点了「下载」以后才下，放在这台电脑自己的目录里，不进资料库。
      </p>
      {isNotServed(transforms.error) ? (
        <NotServedLine className="mt-5">这台电脑上的 usuyo 内核还没接上转换插件；接上以后，这里列出它们和要下载的模型。</NotServedLine>
      ) : transforms.error ? (
        <ErrorLine error={transforms.error} className="mt-5" />
      ) : null}
      {transforms.data ? (
        <>
          <p className="m-0 mt-3 text-small text-faint">
            {transforms.data.auto ? "收下以后自己转，只在接着电源时；" : "现在只在你打开一条时才转；"}
            <Link to="/settings" hash="text" className="text-label underline decoration-border2 underline-offset-4 hover:text-text">
              在设置里改
            </Link>
          </p>
          <div className="mt-5 grid gap-3 @2xl:grid-cols-2">
            {transforms.data.plugins.map((p) => (
              <TransformCard key={p.id} plugin={p} />
            ))}
          </div>
        </>
      ) : null}
    </section>
  );
}

function TransformCard({ plugin }: { plugin: TransformPlugin }) {
  return (
    <article aria-label={plugin.name} className="flex min-w-0 flex-col rounded-card border border-border bg-card px-4 py-3.5">
      <div className="flex flex-wrap items-baseline gap-x-2">
        <h3 className="m-0 text-section font-semibold text-text">{plugin.name}</h3>
        <span className="text-note text-sub">{plugin.tool}</span>
      </div>
      <p className="m-0 mt-1 text-note text-label">{plugin.turns}</p>
      {plugin.health === "ok" ? (
        plugin.hint ? <p className="m-0 mt-1.5 text-small text-sub">{plugin.hint}</p> : null
      ) : (
        <p className="m-0 mt-1.5 text-small text-warn">{plugin.hint ?? "这个插件现在用不了"}</p>
      )}
      {plugin.bundled?.map((b) => (
        <p key={b.name} className="m-0 mt-1.5 text-small text-sub">
          自带 {b.name}（{b.license}）
        </p>
      ))}
      {plugin.models.length > 0 ? (
        <ul className="m-0 mt-3 flex list-none flex-col gap-2 p-0">
          {plugin.models.map((m) => (
            <ModelRow key={m.name} pluginId={plugin.id} model={m} />
          ))}
        </ul>
      ) : null}
      <div className="flex-1" />
      <div className="mono mt-3 truncate border-t border-border pt-2.5 text-tag text-faint" title={plugin.id}>
        {plugin.id} · {plugin.version}
      </div>
    </article>
  );
}

/** 模型那一行的状态，照实说。 */
export function modelStateText(m: TransformModel): string {
  switch (m.state) {
    case "present":
      return "下好了";
    case "downloading":
      return "在下载";
    case "failed":
      return `没下成：${m.hint ?? "原因没写"}`;
    default:
      return "还没下载";
  }
}

function ModelRow({ pluginId, model }: { pluginId: string; model: TransformModel }) {
  const change = useModelChange();
  const [asking, setAsking] = useState(false);
  return (
    <li className="rounded-btn bg-card2 px-3 py-2.5">
      <div className="flex flex-wrap items-center gap-x-3 gap-y-1.5">
        <div className="min-w-0 flex-[1_1_160px]">
          <div className="text-note font-medium text-text">
            {model.label} <span className="mono text-tag font-normal text-faint">{sizeText(model.size)}</span>
          </div>
          <div className={`text-small ${model.state === "failed" ? "text-warn" : "text-sub"}`}>{modelStateText(model)}</div>
        </div>
        {model.state === "absent" || model.state === "failed" ? (
          <Button variant="soft" size="sm" isDisabled={change.isPending} onPress={() => change.mutate({ kind: "download", pluginId, name: model.name })}>
            {model.state === "failed" ? "再下一次" : "下载"}
          </Button>
        ) : model.state === "present" ? (
          asking ? (
            <span className="flex items-center gap-1.5">
              <Button
                variant="strong"
                size="sm"
                onPress={() => {
                  setAsking(false);
                  change.mutate({ kind: "remove", pluginId, name: model.name });
                }}
              >
                删掉模型
              </Button>
              <Button variant="quiet" size="sm" onPress={() => setAsking(false)}>
                不删了
              </Button>
            </span>
          ) : (
            <Button variant="quiet" size="sm" onPress={() => setAsking(true)}>
              删掉
            </Button>
          )
        ) : null}
      </div>
      {asking ? <p className="m-0 mt-1.5 text-small text-sub">删掉以后，已经转出来的文字都留着；之后要它转的，等你再下载。</p> : null}
      <p className="m-0 mt-1.5 text-small text-faint">
        许可：{model.license}　<span className="mono">{model.name}</span>
      </p>
      {change.error ? <ErrorLine error={change.error} className="mt-1" /> : null}
    </li>
  );
}
