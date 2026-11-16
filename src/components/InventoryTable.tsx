import { useRef } from "react";
import type { KeyboardEvent } from "react";
import { kindLabel, type SortDir, type SortKey, type ViewRow } from "../rows";

const COLUMNS: Array<{ key: SortKey | null; label: string; className: string }> = [
  { key: "name", label: "名稱", className: "h-name" },
  { key: "version", label: "已安裝", className: "h-version" },
  { key: "latest", label: "最新", className: "h-latest" },
  { key: "source", label: "來源", className: "h-source" },
  { key: "kind", label: "類型", className: "h-kind" },
  { key: null, label: "備註", className: "h-note" },
];

interface Props {
  view: ViewRow[];
  selectedId: string | null;
  onSelect: (id: string) => void;
  onToggleGroup: (id: string) => void;
  sort: SortKey;
  sortDir: SortDir;
  onSort: (key: SortKey) => void;
  scanning: boolean;
  hasInventory: boolean;
  emptyHint: string;
}

export default function InventoryTable({
  view,
  selectedId,
  onSelect,
  onToggleGroup,
  sort,
  sortDir,
  onSort,
  scanning,
  hasInventory,
  emptyHint,
}: Props) {
  const scrollRef = useRef<HTMLDivElement>(null);

  // 用 DOM 順序移動焦點，比維護一份索引狀態更不容易出錯
  const moveFocus = (event: KeyboardEvent<HTMLDivElement>, delta: number) => {
    const rows = Array.from(scrollRef.current?.querySelectorAll<HTMLElement>(".inv-row") ?? []);
    const index = rows.indexOf(event.currentTarget);
    const next = rows[index + delta];
    if (next) next.focus();
  };

  return (
    <div className="inv-scroll" ref={scrollRef}>
      <div className="inv-head" role="row">
        <span className="h-status" aria-hidden="true" />
        {COLUMNS.map((column) =>
          column.key === null ? (
            <span key={column.className} className={column.className}>
              {column.label}
            </span>
          ) : (
            <button
              key={column.className}
              type="button"
              className={`${column.className} h-sort ${sort === column.key ? "is-active" : ""}`}
              onClick={() => onSort(column.key as SortKey)}
              aria-label={`依${column.label}排序，再按一次反向`}
            >
              {column.label}
              {sort === column.key && (
                <span className="h-arrow" aria-hidden="true">
                  {sortDir === "asc" ? "↑" : "↓"}
                </span>
              )}
            </button>
          ),
        )}
      </div>

      {!hasInventory && (
        <div className="inv-empty">
          <p className="inv-empty-title">{scanning ? "正在讀取這台 Mac 的環境…" : "還沒有資料"}</p>
          <p className="inv-empty-body">
            {scanning ? "依套件數量不同，通常需要 2–10 秒。" : "按右上角「重新掃描」再試一次。"}
          </p>
        </div>
      )}

      {hasInventory && view.length === 0 && (
        <div className="inv-empty">
          <p className="inv-empty-title">{emptyHint}</p>
          <p className="inv-empty-body">換個關鍵字，或把狀態篩選切回「全部」。</p>
        </div>
      )}

      {view.map((entry) => {
        if (entry.type === "group") {
          return (
            <button
              key={`group-${entry.id}`}
              type="button"
              className="group-row"
              onClick={() => onToggleGroup(entry.id)}
              aria-expanded={!entry.collapsed}
              title={entry.blurb}
            >
              <svg
                className={`group-caret ${entry.collapsed ? "" : "is-open"}`}
                width="10"
                height="10"
                viewBox="0 0 10 10"
                aria-hidden="true"
              >
                <path
                  d="M3 1.8 L7 5 L3 8.2"
                  fill="none"
                  stroke="currentColor"
                  strokeWidth="1.5"
                  strokeLinecap="round"
                  strokeLinejoin="round"
                />
              </svg>
              <span className="group-label">{entry.label}</span>
              {entry.headline ? (
                <span className="group-version">{entry.headline}</span>
              ) : (
                <span className="group-blurb">{entry.blurb}</span>
              )}
              <span className="group-flags">
                {entry.outdated > 0 && <span className="group-warn">{entry.outdated} 需更新</span>}
              </span>
              <span className="group-count">{entry.count} 項</span>
            </button>
          );
        }

        const { row } = entry;
        const classes = ["inv-row"];
        if (row.statusResolved === "outdated") classes.push("is-outdated");
        if (row.id === selectedId) classes.push("is-selected");

        return (
          <div
            key={row.id}
            className={classes.join(" ")}
            role="row"
            tabIndex={0}
            onClick={() => onSelect(row.id)}
            onKeyDown={(event) => {
              if (event.key === "Enter" || event.key === " ") {
                event.preventDefault();
                onSelect(row.id);
                return;
              }
              if (event.key === "ArrowDown") {
                event.preventDefault();
                moveFocus(event, 1);
              }
              if (event.key === "ArrowUp") {
                event.preventDefault();
                moveFocus(event, -1);
              }
            }}
          >
            <span className="cell-status">
              <i className={`dot dot--${row.statusResolved}`} />
            </span>
            <span
              className="cell-name"
              title={row.description ? `${row.name}\n${row.description}` : row.name}
            >
              {row.name}
            </span>
            <span className="cell-version">{row.version}</span>
            <span className="cell-latest">
              {row.latestResolved ? (
                row.latestResolved
              ) : row.statusResolved === "current" ? (
                <span className="cell-current">已最新</span>
              ) : (
                <span className="cell-faint">—</span>
              )}
            </span>
            <span className="cell-source" title={row.source}>
              {row.source}
            </span>
            <span className="cell-kind">{kindLabel(row.kind)}</span>
            <span className="cell-note" title={row.note ?? undefined}>
              {row.note}
            </span>
          </div>
        );
      })}
    </div>
  );
}
