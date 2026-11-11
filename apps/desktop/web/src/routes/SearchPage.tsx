import { getRouteApi, Link } from "@tanstack/react-router";
import { Input, Label, SearchField } from "react-aria-components";
import { useSearch } from "../api/hooks";
import { ErrorLine } from "../components/ErrorLine";

const route = getRouteApi("/search");

export function SearchPage() {
  const { q } = route.useSearch();
  const navigate = route.useNavigate();
  const results = useSearch(q);

  return (
    <section>
      <h1>搜索</h1>
      <SearchField
        value={q}
        onChange={(value) => void navigate({ search: { q: value }, replace: true })}
        className="field"
      >
        <Label>关键词</Label>
        <Input />
      </SearchField>
      {results.error ? <ErrorLine error={results.error} /> : null}
      {results.data && results.data.length === 0 ? <p className="muted">没有找到</p> : null}
      <ul className="results">
        {results.data?.map((hit) => (
          <li key={hit.id}>
            <Link to="/items/$id" params={{ id: hit.id }}>
              {hit.title}
            </Link>
            <span className="muted">{hit.source}</span>
            <p>{hit.snippet}</p>
          </li>
        ))}
      </ul>
    </section>
  );
}
