import { Link, Outlet } from "@tanstack/react-router";
import { useHealth } from "../api/hooks";
import { ErrorLine } from "./ErrorLine";

const nav = [
  { to: "/", label: "收下" },
  { to: "/items", label: "存着" },
  { to: "/search", label: "搜索" },
] as const;

export function Layout() {
  const health = useHealth();
  return (
    <div className="app">
      <header className="app-header">
        <nav aria-label="主导航">
          {nav.map((entry) => (
            <Link key={entry.to} to={entry.to} activeOptions={{ exact: entry.to === "/" }}>
              {entry.label}
            </Link>
          ))}
        </nav>
        {health.error ? <ErrorLine error={health.error} /> : null}
      </header>
      <main className="app-main">
        <Outlet />
      </main>
    </div>
  );
}
