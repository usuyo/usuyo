import type { ReactNode } from "react";
import { useSettings } from "../api/hooks";
import { ErrorLine } from "../components/ErrorLine";
import { Rail } from "../frame/Rail";
import { Appearance } from "./Appearance";
import { Diagnostics } from "./Diagnostics";
import { Network } from "./Network";
import { Storage } from "./Storage";
import { TextSection } from "./TextSection";

const SECTIONS = [
  { id: "appearance", name: "外观" },
  { id: "network", name: "网络" },
  { id: "storage", name: "存储与回收站" },
  { id: "text", name: "变成文字" },
  { id: "diagnostics", name: "诊断" },
  { id: "agent", name: "Agent" },
  { id: "sync", name: "同步" },
] as const;

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
      <Rail label="主导航" />
      <main className="@container absolute inset-y-0 right-0 left-rail overflow-y-auto scroll-smooth px-6 pt-7 pb-24 motion-reduce:scroll-auto lg:px-gutter">
        <h1 className="m-0 text-title font-semibold text-text">设置</h1>
        <p className="m-0 mt-1 text-note text-sub">改了马上生效。会让内容离开这台电脑的，旁边写着。</p>
        <div className="mt-8 grid items-start gap-x-10 @3xl:grid-cols-[160px_minmax(0,1fr)]">
          <nav aria-label="设置的几节" className="sticky top-0 hidden flex-col gap-0.5 @3xl:flex">
            {SECTIONS.map((s) => (
              <a
                key={s.id}
                href={`#${s.id}`}
                className="press flex h-ctl-xs items-center rounded-sm px-2.5 text-note text-sub no-underline hover:bg-chip/60 hover:text-text"
              >
                {s.name}
              </a>
            ))}
          </nav>
          <div className="flex min-w-0 max-w-[760px] flex-col">
            <Section id="appearance" name="外观" sub="只在这台电脑上，不跟着资料库走。">
              <Appearance />
            </Section>
            <Section id="network" name="网络" sub="获取插件去平台取东西时，从哪个出口出去。">
              {gate((v) => (
                // 内核那边的出口变了（别处改的、事件来的）就照新的重新摆。
                <Network key={`${v.network.mode} ${v.network.proxy ?? ""}`} view={v} />
              ))}
            </Section>
            <Section id="storage" name="存储与回收站" sub="资料库就是一个普通文件夹，不开 usuyo 也能读。">
              {gate((v) => <Storage view={v} />)}
            </Section>
            <Section id="text" name="变成文字" sub="把口播、图里的字、画面上的字转出来，都在这台电脑上做。">
              {gate((v) => <TextSection view={v} />)}
            </Section>
            <Section id="diagnostics" name="诊断" sub="出了问题时来这里看。">
              <Diagnostics logs={settings.data?.logs} />
            </Section>
            <Section id="agent" name="Agent" sub="让你自己的 Agent 经 MCP 用资料库。">
              <Later
                lines={[
                  "这里会放：开关、接上的是哪个 Agent、它做过什么。",
                  "Agent 只能搜、读、提议改动；改已有的内容先出改动提议，你确认以后才生效，能整批撤销。不能删除，也不能把内容发出去。",
                  "私密条目和你写的笔记，默认不给 Agent 看。",
                ]}
                state="还没接上：这一版的 usuyo 还没有 MCP 服务。"
              />
            </Section>
            <Section id="sync" name="同步" sub="在你自己的几台电脑之间同步文字。" away>
              <Later
                lines={[
                  "这里会放：接上你自己的 Cloudflare 账号、配第二台电脑、看同步到哪儿了。",
                  "跑在你自己的账号里，没有 usuyo 的服务器；离开这台电脑以前先加密，主密码只有你知道。",
                  "同步文字和机器转出来的文字，视频和图先不同步。",
                ]}
                state="还没接上：不接也能在这台电脑上一直用下去。"
              />
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
        {away ? <span className="rounded-chip border border-border2 px-1.5 text-tag text-label">离开本机</span> : null}
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
