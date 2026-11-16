import { ALL_SCOPE } from "../rows";
import type { Ecosystem } from "../types";

interface Props {
  ecosystems: Ecosystem[];
  counts: Map<string, { total: number; outdated: number }>;
  scope: string;
  total: number;
  onSelect: (id: string) => void;
}

export default function EcosystemRail({ ecosystems, counts, scope, total, onSelect }: Props) {
  const max = Math.max(1, ...ecosystems.map((eco) => counts.get(eco.id)?.total ?? 0));

  return (
    <nav className="rail" aria-label="生態系">
      <button
        type="button"
        className={`rail-item ${scope === ALL_SCOPE ? "is-active" : ""}`}
        onClick={() => onSelect(ALL_SCOPE)}
      >
        <span className="rail-label">全部</span>
        <span className="rail-count">{total}</span>
        <span className="rail-bar">
          <i style={{ width: "100%" }} />
        </span>
      </button>

      {ecosystems.map((eco) => {
        const stat = counts.get(eco.id) ?? { total: 0, outdated: 0 };
        // 條長代表件數佔比，讓「哪個生態系是大戶」一眼看得出來
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
