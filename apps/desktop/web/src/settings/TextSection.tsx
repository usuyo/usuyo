import { Link } from "@tanstack/react-router";
import { Switch } from "react-aria-components";
import type { SettingsView } from "../api/endpoints";
import { useChangeSettings, useTransforms } from "../api/hooks";
import { ErrorLine } from "../components/ErrorLine";
import { sizeText } from "../item/faces";
import { modelStateText } from "../plugins/TransformsPanel";
import { Row } from "./Row";

/** 变成文字：收下以后自己转（只在接着电源时），开或关；模型一览，下载和删掉在获取插件页。 */
export function TextSection({ view }: { view: SettingsView }) {
  const change = useChangeSettings();
  const transforms = useTransforms();
  const auto = view.transforms.auto;
  const models = transforms.data?.plugins.flatMap((p) => p.models.map((m) => ({ ...m, plugin: p.name }))) ?? [];
  return (
    <div className="flex flex-col gap-5">
      <Row
        label="自己转"
        sub={
          auto
            ? "收下一条以后自己排上转换，只在接着电源时跑，一次一个；拔了电，跑着的那个跑完就停下等着。你打开的那一条马上转，在电池上也转。"
            : "关着：只在你打开一条时才转那一条。收下的都照样存着，以后打开再转也一样。"
        }
      >
        <Switch
          isSelected={auto}
          isDisabled={change.isPending}
          onChange={(on) => change.mutate({ transforms: { auto: on } })}
          className="group flex h-ctl-sm w-fit cursor-pointer items-center gap-3 font-sans text-note text-label outline-none"
        >
          <span className="flex h-6 w-10 items-center rounded-full border border-border2 bg-chip px-0.5 transition-colors duration-150 group-data-[focus-visible]:outline-2 group-data-[focus-visible]:outline-accent group-data-[selected]:border-label group-data-[selected]:bg-label">
            <span className="size-[18px] rounded-full bg-text transition-transform duration-150 ease-out group-data-[selected]:translate-x-4 group-data-[selected]:bg-bg motion-reduce:transition-none" />
          </span>
          <span>{auto ? "收下以后自己转" : "只在打开时转"}</span>
        </Switch>
      </Row>
      <Row label="模型" sub={<>下载和删掉在<Link to="/plugins" search={{ show: "text" }} className="mx-0.5 text-label underline decoration-border2 underline-offset-4 hover:text-text">获取插件</Link>页的「变成文字」里。</>}>
        {models.length > 0 ? (
          <ul className="m-0 flex list-none flex-col gap-1 p-0">
            {models.map((m) => (
              <li key={m.name} className="flex flex-wrap items-baseline gap-x-2.5 py-1 text-note">
                <span className="break-all text-text">{m.name}</span>
                <span className="mono text-tag text-faint">{sizeText(m.size)}</span>
                <span className="text-small text-sub">{m.plugin} · {modelStateText(m)}</span>
              </li>
            ))}
          </ul>
        ) : (
          <p className="m-0 pt-2 text-note text-sub">{transforms.data ? "装着的转换插件都不用下载模型。" : ""}</p>
        )}
      </Row>
      {transforms.error ? <ErrorLine error={transforms.error} /> : null}
      {change.error ? <ErrorLine error={change.error} /> : null}
    </div>
  );
}
