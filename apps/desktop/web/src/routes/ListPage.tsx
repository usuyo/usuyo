import { useRef } from "react";
import { useVirtualizer } from "@tanstack/react-virtual";
import { Link } from "@tanstack/react-router";
import { useItems } from "../api/hooks";
import { ErrorLine } from "../components/ErrorLine";

const ROW_HEIGHT = 56;

export function ListPage() {
  const items = useItems();
  const scrollRef = useRef<HTMLDivElement>(null);
  const rows = items.data ?? [];
  const virtualizer = useVirtualizer({
    count: rows.length,
    getScrollElement: () => scrollRef.current,
    estimateSize: () => ROW_HEIGHT,
  });

  return (
    <section>
      <h1>存着</h1>
      {items.error ? <ErrorLine error={items.error} /> : null}
      {items.data && rows.length === 0 ? <p className="muted">还没有收下的内容</p> : null}
      <div ref={scrollRef} className="list-scroll">
        <ul style={{ height: virtualizer.getTotalSize() }} className="list">
          {virtualizer.getVirtualItems().map((row) => {
            const item = rows[row.index];
            if (!item) return null;
            return (
              <li
                key={item.id}
                style={{ height: row.size, transform: `translateY(${row.start}px)` }}
                className="list-row"
              >
                <Link to="/items/$id" params={{ id: item.id }}>
                  {item.title}
                </Link>
                <span className="muted">{item.source}</span>
              </li>
            );
          })}
        </ul>
      </div>
    </section>
  );
}
