import { getRouteApi } from "@tanstack/react-router";
import { useItem } from "../api/hooks";
import { ErrorLine } from "../components/ErrorLine";

const route = getRouteApi("/items/$id");

export function ItemPage() {
  const { id } = route.useParams();
  const item = useItem(id);

  if (item.error) return <ErrorLine error={item.error} />;
  if (!item.data) return null;

  return (
    <article>
      <h1>{item.data.title}</h1>
      <p className="muted">{item.data.source}</p>
      <div className="text-column">{item.data.text}</div>
    </article>
  );
}
