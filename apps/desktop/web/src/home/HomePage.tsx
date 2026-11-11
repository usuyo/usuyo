import { useEffect, useState } from "react";
import { useMap, useTopics } from "../api/hooks";
import { ErrorLine } from "../components/ErrorLine";
import { EmptyPage } from "../empty/EmptyPage";
import { Rail } from "../frame/Rail";
import { MapScreen } from "./MapScreen";

/** 主页：资料库空着时只有雾和名字，有东西了就是地图。 */
export function HomePage() {
  const map = useMap();
  const topics = useTopics();
  // 在空资料库里刚收下的那一条，等它落完再换成地图。
  const [holdEmpty, setHoldEmpty] = useState(false);
  useEffect(() => {
    if (!holdEmpty) return undefined;
    const id = setTimeout(() => setHoldEmpty(false), 3200);
    return () => clearTimeout(id);
  }, [holdEmpty]);
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
  if (empty || (wasEmpty && holdEmpty)) return <EmptyPage onLanded={() => setHoldEmpty(true)} />;
  return <MapScreen map={map.data} topics={topics.data} />;
}
