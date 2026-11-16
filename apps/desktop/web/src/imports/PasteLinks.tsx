import { useState } from "react";
import { Label, TextArea, TextField } from "react-aria-components";
import { usePasteLinks } from "../api/hooks";
import { ErrorLine } from "../components/ErrorLine";
import { t } from "../i18n";
import { Button } from "../ui/Button";

/**
 * 粘贴链接：一段文字里的每条链接（一行一条、夹在分享文字里、挤在一起的都算）交给内核，落成一次导入。
 * 认哪条是哪个平台的、哪些认不出、哪些已经存着，都由内核照插件的 `match` 算；这里不认任何平台。
 */
export function PasteLinks({ onStarted }: { onStarted: (id: string) => void }) {
  const [text, setText] = useState("");
  const paste = usePasteLinks();
  return (
    <section aria-labelledby="paste-links-h">
      <h2 id="paste-links-h" className="m-0 text-section font-semibold text-text">
        {t("imports.paste.title")}
      </h2>
      <p className="m-0 mt-1 text-small text-sub">{t("imports.paste.note")}</p>
      <TextField
        value={text}
        onChange={(v) => {
          setText(v);
          paste.reset();
        }}
        className="mt-3.5 flex flex-col gap-1"
      >
        <Label className="text-small text-label">{t("imports.paste.label")}</Label>
        <TextArea
          rows={8}
          placeholder={t("imports.paste.placeholder")}
          className="block w-full resize-y rounded-btn border border-border2 bg-bg px-3 py-2.5 font-sans text-note text-text outline-none focus:border-label"
        />
      </TextField>
      <div className="mt-3 flex flex-wrap items-center gap-3">
        <Button
          variant="soft"
          size="sm"
          isDisabled={text.trim() === "" || paste.isPending}
          onPress={() => paste.mutate({ text }, { onSuccess: (v) => onStarted(v.id) })}
        >
          {t("imports.paste.find")}
        </Button>
        <span className="text-small text-sub">{t("imports.paste.after")}</span>
      </div>
      {paste.error ? <ErrorLine error={paste.error} className="mt-2" /> : null}
    </section>
  );
}
