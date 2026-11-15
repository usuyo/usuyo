import { useState } from "react";
import { Form, Input, TextField } from "react-aria-components";
import type { CatalogSource } from "../api/endpoints";
import { useSourceChange } from "../api/hooks";
import { ErrorLine } from "../components/ErrorLine";
import { Button } from "../ui/Button";
import { Card, CardRow } from "../ui/Card";
import { t } from "../i18n";
import { timeText } from "./words";

/** 订阅源：地址、上次读取的时间、插件数，刷新、移除；下面一行添加。 */
export function SourcesCard({ sources }: { sources: CatalogSource[] }) {
  const change = useSourceChange();
  const [address, setAddress] = useState("");
  const busy = change.isPending;
  return (
    <div className="mt-8 max-w-[760px]">
      <Card title={t("plugins.sources.title")} sub={t("plugins.sources.sub")}>
        {sources.map((s) => (
          <CardRow
            key={s.address}
            title={
              <span className="mono break-all text-note">
                {s.address}
                {s.default ? <span className="ml-2 font-sans text-small font-normal text-faint">{t("plugins.sources.default")}</span> : null}
              </span>
            }
            sub={
              <>
                {s.refreshing ? t("plugins.sources.refreshing") : s.refreshedAt ? t("plugins.sources.refreshed", { time: timeText(s.refreshedAt) }) : t("plugins.sources.never")}
                {s.plugins !== undefined ? ` · ${t("plugins.sources.count", { count: s.plugins })}` : null}
                {s.error ? <span className="block text-warn">{s.error}</span> : null}
              </>
            }
            control={
              <>
                <Button variant="outline" size="sm" isDisabled={busy || s.refreshing} onPress={() => change.mutate({ kind: "refresh", address: s.address })}>
                  {t("plugins.sources.refresh")}
                </Button>
                <Button variant="quiet" size="sm" isDisabled={busy} onPress={() => change.mutate({ kind: "remove", address: s.address })}>
                  {t("plugins.sources.remove")}
                </Button>
              </>
            }
          />
        ))}
        <CardRow
          title={t("plugins.sources.addLabel")}
          control={
            sources.length > 1 ? (
              <Button variant="quiet" size="sm" isDisabled={busy} onPress={() => change.mutate({ kind: "refresh" })}>
                {t("plugins.sources.refreshAll")}
              </Button>
            ) : undefined
          }
        >
          <Form
            className="flex flex-wrap items-center gap-2"
            onSubmit={(e) => {
              e.preventDefault();
              const text = address.trim();
              if (text === "") return;
              change.mutate({ kind: "add", address: text }, { onSuccess: () => setAddress("") });
            }}
          >
            <TextField value={address} onChange={setAddress} aria-label={t("plugins.sources.addLabel")} className="min-w-0 flex-[1_1_260px]">
              <Input placeholder={t("plugins.sources.addPlaceholder")} className="mono h-ctl-sm w-full rounded-input border border-border2 bg-transparent px-3 text-note text-text outline-none data-[focused]:border-label" />
            </TextField>
            <Button type="submit" variant="soft" size="sm" isDisabled={busy || address.trim() === ""}>
              {t("plugins.sources.add")}
            </Button>
          </Form>
          {change.error ? <ErrorLine error={change.error} className="mt-2" /> : null}
        </CardRow>
      </Card>
    </div>
  );
}
