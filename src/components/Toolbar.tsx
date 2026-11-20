import type { RefObject } from "react";
import { useT } from "../i18nContext";
import type { StatusFilter } from "../rows";

interface Props {
  inputRef: RefObject<HTMLInputElement | null>;
  query: string;
  onQuery: (value: string) => void;
  filter: StatusFilter;
  onFilter: (value: StatusFilter) => void;
  shown: number;
  total: number;
  checkedAt: number | null;
  /// 只有「全部」檢視才顯示收合控制；其他檢視傳 null
  onToggleGroups: (() => void) | null;
  allGroupsCollapsed: boolean;
}

export default function Toolbar({
  inputRef,
  query,
  onQuery,
  filter,
  onFilter,
  shown,
  total,
  checkedAt,
  onToggleGroups,
  allGroupsCollapsed,
}: Props) {
  const t = useT();

  const filters: Array<{ id: StatusFilter; label: string }> = [
    { id: "all", label: t("filter.all") },
    { id: "outdated", label: t("filter.outdated") },
    { id: "current", label: t("filter.current") },
    { id: "unknown", label: t("filter.unknown") },
    { id: "agent", label: t("filter.agent") },
  ];

  return (
    <div className="sheet-toolbar">
      <div className="search">
        <svg width="12" height="12" viewBox="0 0 12 12" aria-hidden="true" className="search-icon">
          <circle cx="5" cy="5" r="3.4" fill="none" stroke="currentColor" strokeWidth="1.3" />
          <path d="M7.6 7.6 L10.4 10.4" stroke="currentColor" strokeWidth="1.3" strokeLinecap="round" />
        </svg>
        <input
          ref={inputRef}
          value={query}
          onChange={(event) => onQuery(event.target.value)}
          placeholder={t("search.placeholder")}
          spellCheck={false}
          aria-label={t("search.placeholder")}
        />
        {query.length > 0 && (
          <button
            type="button"
            className="search-clear"
            onClick={() => onQuery("")}
            aria-label={t("search.clear")}
          >
            ×
          </button>
        )}
      </div>

      <div className="seg" role="group" aria-label={t("filter.all")}>
        {filters.map((entry) => (
          <button
            key={entry.id}
            type="button"
            data-filter={entry.id}
            className={filter === entry.id ? "is-active" : ""}
            onClick={() => onFilter(entry.id)}
          >
            {entry.label}
          </button>
        ))}
      </div>

      {onToggleGroups && (
        <button type="button" className="btn btn--ghost" onClick={onToggleGroups}>
          {allGroupsCollapsed ? t("groups.expand") : t("groups.collapse")}
        </button>
      )}

      <span className="count-note">
        {checkedAt === null && <span className="check-hint">{t("count.unchecked")}</span>}
        {shown === total
          ? t("count.items", { n: total })
          : t("count.partial", { shown, total })}
      </span>
    </div>
  );
}
