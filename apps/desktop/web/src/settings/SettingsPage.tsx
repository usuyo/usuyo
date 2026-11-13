import type { ReactNode } from "react";
import { useSettings } from "../api/hooks";
import { ErrorLine } from "../components/ErrorLine";
import { Rail } from "../frame/Rail";
import { Appearance } from "./Appearance";
import { Diagnostics } from "./Diagnostics";
import { General } from "./General";
import { Network } from "./Network";
import { Storage } from "./Storage";
import { TextSection } from "./TextSection";
import { WindowSection } from "./WindowSection";
import { t, type MessageKey } from "../i18n";

const SECTIONS = [
  { id: "general", name: "settings.general.name" },
  { id: "appearance", name: "settings.appearance.name" },
  { id: "window", name: "settings.window.name" },
  { id: "network", name: "settings.network.name" },
  { id: "storage", name: "settings.storage.name" },
  { id: "text", name: "settings.text.name" },
  { id: "diagnostics", name: "settings.diagnostics.name" },
  { id: "agent", name: "settings.agent.name" },
  { id: "sync", name: "settings.sync.name" },
] as const satisfies { id: string; name: MessageKey }[];

/**
 * 设置：一页从上到下，左边是目录。改了马上生效，没有「保存」。
 * 外观只在这台电脑的窗口里；别的经内核。
 */
export function SettingsPage() {
  const settings = useSettings();
  const gate = (node: (v: NonNullable<typeof settings.data>) => ReactNode) =>
    settings.data ? node(settings.data) : settings.error ? <ErrorLine error={settings.error} /> : null;

  return (
    <div className="relative h-full overflow-hidden bg-bg">
      <Rail label={t("common.nav.main")} />
      <main className="@container absolute inset-y-0 right-0 left-rail overflow-y-auto scroll-smooth px-6 pt-7 pb-24 motion-reduce:scroll-auto lg:px-gutter">
        <h1 className="m-0 text-title font-semibold text-text">{t("settings.title")}</h1>
        <p className="m-0 mt-1 text-note text-sub">{t("settings.intro")}</p>
        <div className="mt-8 grid items-start gap-x-10 @3xl:grid-cols-[160px_minmax(0,1fr)]">
          <nav aria-label={t("settings.sectionsLabel")} className="sticky top-0 hidden flex-col gap-0.5 @3xl:flex">
            {SECTIONS.map((s) => (
              <a
                key={s.id}
                href={`#${s.id}`}
                className="press flex h-ctl-xs items-center rounded-sm px-2.5 text-note text-sub no-underline hover:bg-chip/60 hover:text-text"
              >
                {t(s.name)}
              </a>
            ))}
          </nav>
          <div className="flex min-w-0 max-w-[760px] flex-col">
            <Section id="general" name={t("settings.general.name")} sub={t("settings.general.sub")}>
              {gate((v) => <General view={v} />)}
            </Section>
            <Section id="appearance" name={t("settings.appearance.name")} sub={t("settings.appearance.sub")}>
              <Appearance />
            </Section>
            <Section id="window" name={t("settings.window.name")} sub={t("settings.window.sub")}>
              {gate((v) => <WindowSection view={v} />)}
            </Section>
            <Section id="network" name={t("settings.network.name")} sub={t("settings.network.sub")}>
              {gate((v) => (
                // 内核那边的出口变了（别处改的、事件来的）就照新的重新摆。
                <Network key={`${v.network.mode} ${v.network.proxy ?? ""}`} view={v} />
              ))}
            </Section>
            <Section id="storage" name={t("settings.storage.name")} sub={t("settings.storage.sub")}>
              {gate((v) => <Storage view={v} />)}
            </Section>
            <Section id="text" name={t("settings.text.name")} sub={t("settings.text.sub")}>
              {gate((v) => <TextSection view={v} />)}
            </Section>
            <Section id="diagnostics" name={t("settings.diagnostics.name")} sub={t("settings.diagnostics.sub")}>
              <Diagnostics logs={settings.data?.logs} />
            </Section>
            <Section id="agent" name={t("settings.agent.name")} sub={t("settings.agent.sub")}>
              <Later lines={[t("settings.agent.plan"), t("settings.agent.limits"), t("settings.agent.privacy")]} state={t("settings.agent.state")} />
            </Section>
            <Section id="sync" name={t("settings.sync.name")} sub={t("settings.sync.sub")} away>
              <Later lines={[t("settings.sync.plan"), t("settings.sync.privacy"), t("settings.sync.scope")]} state={t("settings.sync.state")} />
            </Section>
          </div>
        </div>
      </main>
    </div>
  );
}

function Section({ id, name, sub, away, children }: { id: string; name: string; sub: string; away?: boolean; children: ReactNode }) {
  return (
    <section id={id} aria-labelledby={`${id}-h`} className="scroll-mt-6 border-t border-border pt-5 pb-9 first:border-t-0 first:pt-0">
      <div className="flex flex-wrap items-baseline gap-x-2.5">
        <h2 id={`${id}-h`} className="m-0 text-section font-semibold text-text">
          {name}
        </h2>
        {away ? <span className="rounded-chip border border-border2 px-1.5 text-tag text-label">{t("settings.away")}</span> : null}
      </div>
      <p className="m-0 mt-1 text-note text-sub">{sub}</p>
      <div className="mt-4">{children}</div>
    </section>
  );
}

/** 还没做的一节：说清楚以后放什么、现在怎样，不放假按钮。 */
function Later({ lines, state }: { lines: string[]; state: string }) {
  return (
    <div className="rounded-card border border-dashed border-border2 px-4 py-3.5">
      <ul className="m-0 flex list-none flex-col gap-1.5 p-0">
        {lines.map((l) => (
          <li key={l} className="text-note text-label">
            {l}
          </li>
        ))}
      </ul>
      <p className="m-0 mt-3 text-small text-sub">{state}</p>
    </div>
  );
}
