import type { RefObject } from "react";
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

const FILTERS: Array<{ id: StatusFilter; label: string }> = [
  { id: "all", label: "全部" },
  { id: "outdated", label: "需更新" },
  { id: "current", label: "已最新" },
  { id: "unknown", label: "未檢查" },
];

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
          placeholder="搜尋名稱、版本、來源"
          spellCheck={false}
          aria-label="搜尋"
        />
        {query.length > 0 && (
          <button type="button" className="search-clear" onClick={() => onQuery("")} aria-label="清除搜尋">
            ×
          </button>
        )}
      </div>

      <div className="seg" role="group" aria-label="狀態篩選">
        {FILTERS.map((entry) => (
          <button
            key={entry.id}
            type="button"
            className={filter === entry.id ? "is-active" : ""}
            onClick={() => onFilter(entry.id)}
          >
            {entry.label}
          </button>
        ))}
      </div>

      {onToggleGroups && (
        <button type="button" className="btn btn--ghost" onClick={onToggleGroups}>
          {allGroupsCollapsed ? "展開全部分類" : "收合全部分類"}
        </button>
      )}

      <span className="count-note">
        {checkedAt === null && <span className="check-hint">版本未檢查 · </span>}
        {shown === total ? `${total} 項` : `${shown} / ${total}`}
      </span>
    </div>
  );
}
