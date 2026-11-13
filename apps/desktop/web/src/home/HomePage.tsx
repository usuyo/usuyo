import { useMap, useTopics } from "../api/hooks";
import { ErrorLine } from "../components/ErrorLine";
import { Rail } from "../frame/Rail";
import { t } from "../i18n";
import { MapScreen } from "./MapScreen";

/** 主页就是地图；资料库空着时也是，地图上只多一句。 */
export function HomePage() {
  const map = useMap();
  const topics = useTopics();
  const error = map.error ?? topics.error;
  if (error) {
    return (
      <div className="relative h-full overflow-hidden bg-bg">
        <Rail label={t("common.nav.home")} />
        <div className="absolute bottom-4 left-[268px] z-10">
          <ErrorLine error={error} />
        </div>
      </div>
    );
  }
  if (!map.data || !topics.data) return <div className="h-full bg-bg" />;
  return <MapScreen map={map.data} topics={topics.data} />;
}
