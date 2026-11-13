import { Link } from "@tanstack/react-router";
import { ArrowLeft } from "lucide-react";
import { t } from "../i18n";
import { Rail } from "./Rail";

/** 左栏底下那几页还没做，先说一句。 */
export function LaterPage() {
  return (
    <div className="relative h-full overflow-hidden bg-bg">
      <Rail label={t("common.nav.main")} />
      <main className="absolute inset-y-0 right-0 left-rail px-gutter pt-5">
        <Link
          to="/"
          className="press -ml-2 inline-flex h-ctl-sm items-center gap-2 rounded-btn pr-3 pl-2 text-note text-sub no-underline hover:text-text"
        >
          <ArrowLeft size={16} aria-hidden="true" />
          <span>{t("common.backToMap")}</span>
        </Link>
        <p className="mt-6 text-body text-label">{t("frame.later")}</p>
      </main>
    </div>
  );
}
