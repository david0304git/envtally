import { useMemo, useState } from "react";
import { useT } from "../i18nContext";
import { ALL_SCOPE, ecoBlurb, ecoLabel, type SortDir } from "../rows";
import type { Ecosystem } from "../types";
import EcosystemIcon from "./EcosystemIcon";

type RailSort = "default" | "name" | "count";

interface Props {
  ecosystems: Ecosystem[];
  counts: Map<string, { total: number; outdated: number }>;
  scope: string;
  total: number;
  onSelect: (id: string) => void;
}

export default function EcosystemRail({ ecosystems, counts, scope, total, onSelect }: Props) {
  const t = useT();
  const [query, setQuery] = useState("");
  const [sort, setSort] = useState<RailSort>("default");
  const [dir, setDir] = useState<SortDir>("asc");

  const sorts: Array<{ id: RailSort; label: string }> = [
    { id: "default", label: t("rail.sort.default") },
    { id: "name", label: t("rail.sort.name") },
    { id: "count", label: t("rail.sort.count") },
  ];

  const max = Math.max(1, ...ecosystems.map((eco) => counts.get(eco.id)?.total ?? 0));

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

  const visible = useMemo(() => {
    const needle = query.trim().toLowerCase();
    let list = ecosystems;

    if (needle) {
      // 分類名稱、說明、環境版本都能拿來找（例如打 3.14 可以找到 Python）
      list = list.filter((eco) =>
        [ecoLabel(eco.id, eco.label, t), ecoBlurb(eco.id, eco.blurb, t), eco.headline ?? ""].some(
          (field) => field.toLowerCase().includes(needle),
        ),
      );
    }

    if (sort === "name") {
      list = [...list].sort((a, b) =>
        ecoLabel(a.id, a.label, t).localeCompare(ecoLabel(b.id, b.label, t), "zh-Hant"),
      );
    } else if (sort === "count") {
      const size = (id: string) => counts.get(id)?.total ?? 0;
      list = [...list].sort((a, b) => size(a.id) - size(b.id));
    } else {
      return list;
    }
    return dir === "desc" ? [...list].reverse() : list;
  }, [ecosystems, query, sort, dir, counts, t]);

  return (
    <nav className="rail" aria-label={t("rail.sort")}>
      <div className="rail-tools">
        <div className="rail-search">
          <input
            value={query}
            onChange={(event) => setQuery(event.target.value)}
            placeholder={t("rail.filter")}
            aria-label={t("rail.filter")}
            spellCheck={false}
          />
          {query.length > 0 && (
            <button type="button" onClick={() => setQuery("")} aria-label={t("rail.clear")}>
              ×
            </button>
          )}
        </div>
        <div className="rail-sort" role="group" aria-label={t("rail.sort")}>
          {sorts.map((option) => (
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
        title={t("shortcut.allScope", { mod: navigator.userAgent.includes("Macintosh") ? "⌘" : "Ctrl+" })}
      >
        <span className="rail-label">{t("rail.all")}</span>
        <EcosystemIcon id="all" />
        <span className="rail-count">{total}</span>
        <span className="rail-bar">
          <i style={{ width: "100%" }} />
        </span>
      </button>

      {visible.length === 0 && <p className="rail-empty">{t("rail.empty")}</p>}

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
            title={eco.error ?? ecoBlurb(eco.id, eco.blurb, t)}
          >
            <span className="rail-label">{ecoLabel(eco.id, eco.label, t)}</span>
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
