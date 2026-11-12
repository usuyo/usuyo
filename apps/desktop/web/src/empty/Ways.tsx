import { Link } from "@tanstack/react-router";
import { X } from "lucide-react";
import { useEffect, useRef, type ReactNode } from "react";
import { useFileReaders, usePlugins, useSites, useVisitors } from "../api/hooks";
import { ErrorLine } from "../components/ErrorLine";
import { Button } from "../ui/Button";
import { waysOf, type Way } from "./words";

const ROW = "press -mx-2.5 grid gap-x-5 gap-y-1 rounded-btn px-2.5 py-2 no-underline hover:bg-chip/60 @lg:grid-cols-[minmax(0,11rem)_minmax(0,1fr)]";

/** 档位：一个中性的小块，只是给你挑的标签。 */
function AccessTag({ children }: { children: ReactNode }) {
  return <span className="inline-flex h-5 items-center rounded-chip bg-chip px-[7px] text-small leading-none text-label">{children}</span>;
}

/**
 * 把存过的一批拿回来的几种路子：平台插件的取法和读别的工具导出的文件的插件，一样大小、照插件给的顺序，
 * 每种写清档位和账号用在哪。不标推荐，不催。
 */
export function Ways({ onClose }: { onClose: () => void }) {
  const plugins = usePlugins();
  const sites = useSites();
  const visitors = useVisitors();
  const readers = useFileReaders();
  const siteName = (site: string) => sites.data?.find((s) => s.site === site)?.name ?? visitors.data?.find((v) => v.site === site)?.name ?? site;
  const groups = waysOf(plugins.data?.plugins ?? [], new Set((readers.data ?? []).map((r) => r.id)), siteName);

  // 打开时焦点进到这一层里，键盘接着往下走就是这几种路子。
  const root = useRef<HTMLElement>(null);
  useEffect(() => root.current?.focus(), []);

  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if (e.key === "Escape") onClose();
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [onClose]);

  return (
    <section
      ref={root}
      tabIndex={-1}
      aria-labelledby="ways-h"
      className="@container outline-none flex max-h-full w-full max-w-[680px] animate-rise flex-col rounded-card border border-border2 bg-hud shadow-panel backdrop-blur-md"
    >
      <header className="flex items-start gap-4 px-5 pt-4 pb-3">
        <div className="min-w-0 flex-1">
          <h2 id="ways-h" className="m-0 text-section font-semibold text-text">
            存过的一批，也能拿回来
          </h2>
          <p className="m-0 mt-0.5 text-note text-sub">在别处存过的，照下面哪一种都行；不拿，一条条收下也一样能用。</p>
        </div>
        <Button variant="ghost" size="xs" square aria-label="收起" onPress={onClose}>
          <X size={15} aria-hidden="true" />
        </Button>
      </header>
      <div className="min-h-0 overflow-y-auto border-t border-border px-5 pt-1 pb-4">
        {plugins.error ? <ErrorLine error={plugins.error} className="mt-3" /> : null}
        {plugins.data && groups.length === 0 ? (
          <p className="m-0 mt-3 text-note text-sub">
            还没有装能列出存过的获取插件。
            <Link to="/plugins" className="ml-1 text-label underline-offset-2 hover:underline">
              去「获取插件」看看
            </Link>
          </p>
        ) : null}
        {groups.map((g) => (
          <section key={g.plugin} aria-label={g.plugin} className="mt-3">
            <h3 className="m-0 text-note font-semibold text-text">{g.plugin}</h3>
            <ul className="m-0 mt-0.5 flex list-none flex-col p-0">
              {g.ways.map((w) => (
                <li key={w.key}>
                  <WayRow way={w} />
                </li>
              ))}
            </ul>
            {g.later.length > 0 ? <p className="m-0 mt-0.5 text-small text-faint">还没做好的：{g.later.join("、")}</p> : null}
          </section>
        ))}
        <section aria-labelledby="ways-files-h" className="mt-4 border-t border-border pt-3">
          <h3 id="ways-files-h" className="m-0 text-note font-semibold text-text">
            别的工具导出的文件
          </h3>
          {readers.error ? <ErrorLine error={readers.error} className="mt-1" /> : null}
          {readers.data && readers.data.length === 0 ? <p className="m-0 mt-1 text-small text-sub">装着的插件里还没有能读文件的。</p> : null}
          <ul className="m-0 mt-0.5 flex list-none flex-col p-0">
            {(readers.data ?? []).map((r) => (
              <li key={r.id}>
                <Link to="/import" search={{ from: "file" }} className={ROW}>
                  <span className="min-w-0">
                    <span className="block text-body text-text">{r.name}</span>
                    <span className="block text-small text-sub">{r.plugin}</span>
                  </span>
                  <span className="min-w-0">
                    <span className="block text-note text-label">{r.reads}</span>
                    <span className="mono mt-0.5 block text-tag text-faint">{r.patterns.join("  ")}</span>
                  </span>
                </Link>
              </li>
            ))}
          </ul>
        </section>
      </div>
    </section>
  );
}

function WayRow({ way }: { way: Way }) {
  return (
    <Link to={way.to} className={ROW}>
      <span className="min-w-0 text-body text-text">{way.name}</span>
      <span className="min-w-0">
        <span className="block text-note text-label">{way.gets.join("；")}</span>
        <span className="mt-1 flex flex-wrap items-center gap-x-2 gap-y-1">
          <AccessTag>{way.access}</AccessTag>
          <span className="text-small text-sub">{way.account}</span>
        </span>
        {way.missing ? <span className="mt-0.5 block text-small text-sub">{way.missing}</span> : null}
      </span>
    </Link>
  );
}
