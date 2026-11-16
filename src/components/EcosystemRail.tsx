import { useMemo, useState } from "react";
import { ALL_SCOPE, type SortDir } from "../rows";
import type { Ecosystem } from "../types";
import EcosystemIcon from "./EcosystemIcon";

type RailSort = "default" | "name" | "count";

const SORTS: Array<{ id: RailSort; label: string }> = [
  { id: "default", label: "預設" },
  { id: "name", label: "名稱" },
  { id: "count", label: "件數" },
];

interface Props {
  ecosystems: Ecosystem[];
  counts: Map<string, { total: number; outdated: number }>;
  scope: string;
  total: number;
  onSelect: (id: string) => void;
}

export default function EcosystemRail({ ecosystems, counts, scope, total, onSelect }: Props) {
  const [query, setQuery] = useState("");
  const [sort, setSort] = useState<RailSort>("default");
  const [dir, setDir] = useState<SortDir>("asc");

  // 同一選項再按一次就反向；換選項從遞增開始
  const chooseSort = (id: RailSort) => {
    if (id === "default") {
      setSort("default");
      setDir("asc");
      return;
    }
    if (id === sort) {
      setDir((current) => (current === "asc" ? "desc" : "asc"));
      return;
    }
    setSort(id);
    setDir("asc");
  };

  const max = Math.max(1, ...ecosystems.map((eco) => counts.get(eco.id)?.total ?? 0));

  const visible = useMemo(() => {
    const needle = query.trim().toLowerCase();
    let list = ecosystems;

    if (needle) {
      // 分類名稱、說明、環境版本都能拿來找（例如打 3.14 可以找到 Python）
      list = list.filter((eco) =>
        [eco.label, eco.blurb, eco.headline ?? ""].some((field) =>
          field.toLowerCase().includes(needle),
        ),
      );
    }

    if (sort === "name") {
      list = [...list].sort((a, b) => a.label.localeCompare(b.label, "zh-Hant"));
    } else if (sort === "count") {
      const size = (id: string) => counts.get(id)?.total ?? 0;
      list = [...list].sort((a, b) => size(a.id) - size(b.id));
    } else {
      return list;
    }
    return dir === "desc" ? [...list].reverse() : list;
  }, [ecosystems, query, sort, dir, counts]);

  return (
    <nav className="rail" aria-label="分類">
      <div className="rail-tools">
        <div className="rail-search">
          <input
            value={query}
            onChange={(event) => setQuery(event.target.value)}
            placeholder="篩選分類"
            aria-label="篩選分類"
            spellCheck={false}
          />
          {query.length > 0 && (
            <button type="button" onClick={() => setQuery("")} aria-label="清除篩選">
              ×
            </button>
          )}
        </div>
        <div className="rail-sort" role="group" aria-label="分類排序">
          {SORTS.map((option) => (
            <button
              key={option.id}
              type="button"
              className={sort === option.id ? "is-active" : ""}
              onClick={() => chooseSort(option.id)}
            >
              {option.label}
              {sort === option.id && option.id !== "default" && (
                <span className="rail-sort-arrow" aria-hidden="true">
                  {dir === "asc" ? "↑" : "↓"}
                </span>
              )}
            </button>
          ))}
        </div>
      </div>

      <button
        type="button"
        className={`rail-item ${scope === ALL_SCOPE ? "is-active" : ""}`}
        onClick={() => onSelect(ALL_SCOPE)}
      >
        <span className="rail-label">全部</span>
        <EcosystemIcon id="all" />
        <span className="rail-count">{total}</span>
        <span className="rail-bar">
          <i style={{ width: "100%" }} />
        </span>
      </button>

      {visible.length === 0 && <p className="rail-empty">沒有符合的分類</p>}

      {visible.map((eco) => {
        const stat = counts.get(eco.id) ?? { total: 0, outdated: 0 };
        // 條長代表件數佔比，讓「哪個分類是大戶」一眼看得出來
        const width = Math.max(2, Math.round((stat.total / max) * 100));
        return (
          <button
            key={eco.id}
            type="button"
            className={`rail-item ${scope === eco.id ? "is-active" : ""}`}
            onClick={() => onSelect(eco.id)}
            title={eco.error ?? eco.blurb}
          >
            <span className="rail-label">{eco.label}</span>
            <EcosystemIcon id={eco.id} />
            <span className="rail-count">
              {stat.total}
              {stat.outdated > 0 && <span className="rail-warn">{stat.outdated}</span>}
            </span>
            <span className="rail-bar">
              <i style={{ width: `${width}%` }} />
            </span>
          </button>
        );
      })}
    </nav>
  );
}
