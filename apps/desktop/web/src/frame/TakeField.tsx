import { Link } from "@tanstack/react-router";
import { Link2 } from "lucide-react";
import { useEffect, useState, type ClipboardEvent, type KeyboardEvent } from "react";
import { Input, Label, TextField } from "react-aria-components";
import { ApiError } from "../api/client";
import { useFetchingItem, useTake } from "../api/hooks";
import { ErrorLine } from "../components/ErrorLine";
import { t } from "../i18n";
import { useUiStore } from "../store/ui";

/** 取到了的那一句停多久，再换回下面那行说明；取不到的、出错的一直留着，到下一次添加为止。 */
const DONE_MS = 8000;

/**
 * 左栏顶上添加链接的那一栏：粘贴或回车就添加。下面那一行跟着这一条走：正在添加、正在获取、取到了（点「查看」打开）、
 * 没取到（点「查看原因」去条目页，那里有原因和「重试」）。拖到地图上的链接也在这一行说。
 */
export function TakeField({ note = t("frame.take.note") }: { note?: string }) {
  const [value, setValue] = useState("");
  const take = useTake();
  const markLanding = useUiStore((s) => s.markLanding);
  const reportTake = useUiStore((s) => s.reportTake);

  function submit(raw: string) {
    const input = raw.trim();
    if (input === "") return;
    setValue("");
    take.mutate(
      { input, kind: "link" },
      {
        onSuccess: (res) => {
          if (!res.seen) markLanding(res.id);
          reportTake({ result: res });
        },
        onError: (error) => reportTake({ error }),
      },
    );
  }

  return (
    <div>
      <TextField value={value} onChange={setValue} aria-label={t("frame.take.label")} className="mt-[22px]">
        <Label className="flex h-field items-center gap-2 rounded-input border border-border2 bg-bg px-3 text-sub">
          <Link2 size={15} aria-hidden="true" className="shrink-0" />
          <span className="sr-only-text">{t("frame.take.label")}</span>
          <Input
            placeholder={t("frame.take.placeholder")}
            className="h-ctl min-w-0 flex-1 border-0 bg-transparent p-0 text-note text-text outline-none"
            onPaste={(e: ClipboardEvent<HTMLInputElement>) => {
              const text = e.clipboardData.getData("text");
              if (!text) return;
              e.preventDefault();
              submit(text);
            }}
            onKeyDown={(e: KeyboardEvent<HTMLInputElement>) => {
              if (e.key === "Enter") submit(value);
            }}
          />
        </Label>
      </TextField>
      <div className="mt-2 min-h-[18px] px-1 text-small">
        {take.isPending ? (
          <p role="status" className="m-0 text-label">
            {t("frame.take.adding")}
          </p>
        ) : (
          <TakeStatus note={note} />
        )}
      </div>
    </div>
  );
}

function TakeStatus({ note }: { note: string }) {
  const last = useUiStore((s) => s.lastTake);
  const clear = useUiStore((s) => s.clearTake);
  const result = last && "result" in last ? last.result : null;
  const item = useFetchingItem(result && !result.seen ? result.id : null);
  const data = item.data;
  const fetching = result !== null && !result.seen && (data === undefined || (data.stored === "link" && data.missing === undefined));
  const failed = data !== undefined && data.stored === "link" && data.missing !== undefined;
  const done = result !== null && !fetching && !failed;

  // 每添加一次（`n` 变了）重新计时。
  const n = last?.n;
  useEffect(() => {
    if (!done || n === undefined) return undefined;
    const id = setTimeout(clear, DONE_MS);
    return () => clearTimeout(id);
  }, [done, clear, n]);

  // 内核不收的（例如拖进来的是文件）没带一句话时，说只能添加链接，不报状态码。
  if (last && "error" in last) {
    return last.error instanceof ApiError && last.error.code === "unsupported-input" && last.error.hint === undefined ? (
      <p role="status" className="m-0 text-sub">
        {t("frame.take.linksOnly")}
      </p>
    ) : (
      <ErrorLine error={last.error} />
    );
  }
  if (item.error) return <ErrorLine error={item.error} />;
  if (!result) return <p className="m-0 text-faint">{note}</p>;
  const text = result.seen
    ? t("frame.take.snapshot")
    : fetching
      ? t("frame.take.fetching")
      : failed
        ? t("frame.take.failed")
        : data?.stored === "partial"
          ? t("frame.take.partly")
          : t("frame.take.fetched");
  return (
    <p role="status" title={result.title} className={`m-0 flex min-w-0 items-baseline gap-1.5 ${failed ? "text-warn" : "text-label"}`}>
      <span className="min-w-0">{text}</span>
      {/* 没取到的去条目页：那里写着原因，有「重试」和这个平台的获取方式。 */}
      {failed ? (
        <Link to="/items/$id" params={{ id: result.id }} className="shrink-0 text-text underline decoration-border2 underline-offset-4 hover:decoration-text">
          {t("frame.take.why")}
        </Link>
      ) : (
        <Link to="/" search={{ open: result.id }} className="shrink-0 text-text underline decoration-border2 underline-offset-4 hover:decoration-text">
          {t("frame.take.show")}
        </Link>
      )}
    </p>
  );
}
