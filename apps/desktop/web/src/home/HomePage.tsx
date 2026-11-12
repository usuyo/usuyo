import { useState } from "react";
import { useMap, useTopics } from "../api/hooks";
import { ErrorLine } from "../components/ErrorLine";
import { EmptyPage } from "../empty/EmptyPage";
import { Rail } from "../frame/Rail";
import { MapScreen } from "./MapScreen";

/** 主页：资料库空着时只有雾和名字，有东西了就是地图。 */
export function HomePage() {
  const map = useMap();
  const topics = useTopics();
  // 在空资料库里收下了以后，雾和落下的那几条留着，等你点「去地图看看」再换成地图。
  const [holdEmpty, setHoldEmpty] = useState(false);
  const [wasEmpty, setWasEmpty] = useState(false);
  const empty = map.data !== undefined && map.data.points.length === 0;
  if (empty && !wasEmpty) setWasEmpty(true);

  const error = map.error ?? topics.error;
  if (error) {
    return (
      <div className="relative h-full overflow-hidden bg-bg">
        <Rail label="主页" />
        <div className="absolute bottom-4 left-[268px] z-10">
          <ErrorLine error={error} />
        </div>
      </div>
    );
  }
  if (!map.data || !topics.data) return <div className="h-full bg-bg" />;
  if (empty || (wasEmpty && holdEmpty)) return <EmptyPage onLanded={() => setHoldEmpty(true)} onLeave={() => setHoldEmpty(false)} />;
  return <MapScreen map={map.data} topics={topics.data} />;
}
