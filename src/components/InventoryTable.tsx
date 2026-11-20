import { useRef } from "react";
import type { KeyboardEvent } from "react";
import { useT } from "../i18nContext";
import { ecoBlurb, ecoLabel, kindLabel, type SortDir, type SortKey, type ViewRow } from "../rows";

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
  hasFilter: boolean;
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
  hasFilter,
}: Props) {
  const t = useT();
  const scrollRef = useRef<HTMLDivElement>(null);

  const columns: Array<{ key: SortKey | null; label: string; className: string }> = [
    { key: "name", label: t("col.name"), className: "h-name" },
    { key: "version", label: t("col.installed"), className: "h-version" },
    { key: "latest", label: t("col.latest"), className: "h-latest" },
    { key: "source", label: t("col.source"), className: "h-source" },
    { key: "kind", label: t("col.kind"), className: "h-kind" },
    { key: null, label: t("col.note"), className: "h-note" },
  ];

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
        {columns.map((column) =>
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
              aria-label={t("sort.aria", { label: column.label })}
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
          <p className="inv-empty-title">
            {scanning ? t("empty.scanning.title") : t("empty.none.title")}
          </p>
          <p className="inv-empty-body">
            {scanning ? t("empty.scanning.body") : t("empty.none.body")}
          </p>
        </div>
      )}

      {hasInventory && view.length === 0 && (
        <div className="inv-empty">
          <p className="inv-empty-title">
            {hasFilter ? t("empty.filtered.title") : t("empty.emptyGroup.title")}
          </p>
          <p className="inv-empty-body">{t("empty.filtered.body")}</p>
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
              title={ecoBlurb(entry.id, entry.blurb, t)}
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
              <span className="group-label">{ecoLabel(entry.id, entry.label, t)}</span>
              {entry.headline ? (
                <span className="group-version">{entry.headline}</span>
              ) : (
                <span className="group-blurb">{ecoBlurb(entry.id, entry.blurb, t)}</span>
              )}
              <span className="group-flags">
                {entry.outdated > 0 && (
                  <span className="group-warn">{t("cell.groupOutdated", { n: entry.outdated })}</span>
                )}
              </span>
              <span className="group-count">{t("cell.groupItems", { n: entry.count })}</span>
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
              {row.agent && (
                <span className="agent-tag" title={t("filter.agent")}>
                  AI
                </span>
              )}
            </span>
            <span className="cell-version">{row.version}</span>
            <span className="cell-latest">
              {row.latestResolved ? (
                row.latestResolved
              ) : row.statusResolved === "current" ? (
                <span className="cell-current">{t("cell.latest")}</span>
              ) : (
                <span className="cell-faint">{t("cell.none")}</span>
              )}
            </span>
            <span className="cell-source" title={row.source}>
              {row.source}
            </span>
            <span className="cell-kind">{kindLabel(row.kind, t)}</span>
            <span className="cell-note" title={row.note ?? undefined}>
              {row.noteKey
                ? t(
                    row.noteKey,
                    Object.fromEntries(row.noteArgs.map((value, index) => [String(index), value])),
                  )
                : row.note}
            </span>
          </div>
        );
      })}
    </div>
  );
}
