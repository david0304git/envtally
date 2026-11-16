import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { checkUpdates, scanLocal } from "./api";
import EcosystemRail from "./components/EcosystemRail";
import Inspector from "./components/Inspector";
import InventoryTable from "./components/InventoryTable";
import SpecPlate from "./components/SpecPlate";
import Toolbar from "./components/Toolbar";
import {
  ALL_SCOPE,
  buildView,
  formatTime,
  kindLabel,
  resolveRows,
  type Row,
  type SortKey,
  type StatusFilter,
} from "./rows";
import type { Inventory, Updates } from "./types";

export default function App() {
  const [inventory, setInventory] = useState<Inventory | null>(null);
  const [scanning, setScanning] = useState(true);
  const [scanError, setScanError] = useState<string | null>(null);
  const [updates, setUpdates] = useState<Updates | null>(null);
  const [checking, setChecking] = useState(false);
  const [checkError, setCheckError] = useState<string | null>(null);

  const [scope, setScope] = useState<string>(ALL_SCOPE);
  const [query, setQuery] = useState("");
  const [filter, setFilter] = useState<StatusFilter>("all");
  const [sort, setSort] = useState<SortKey>("name");
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const [showWarnings, setShowWarnings] = useState(false);
  const [collapsedGroups, setCollapsedGroups] = useState<Set<string>>(new Set());

  const searchRef = useRef<HTMLInputElement>(null);

  const scan = useCallback(async () => {
    setScanning(true);
    setScanError(null);
    try {
      const next = await scanLocal();
      setInventory(next);
      // 資料換了一輪，舊的版本檢查結果就對不上 key 了
      setUpdates(null);
      setSelectedId(null);
      setScope(ALL_SCOPE);
    } catch (error) {
      setScanError(error instanceof Error ? error.message : String(error));
    } finally {
      setScanning(false);
    }
  }, []);

  useEffect(() => {
    void scan();
  }, [scan]);

  const runCheck = useCallback(async () => {
    setChecking(true);
    setCheckError(null);
    try {
      setUpdates(await checkUpdates());
    } catch (error) {
      setCheckError(error instanceof Error ? error.message : String(error));
    } finally {
      setChecking(false);
    }
  }, []);

  useEffect(() => {
    const onKeyDown = (event: KeyboardEvent) => {
      const target = event.target as HTMLElement | null;
      const typing = target instanceof HTMLInputElement || target instanceof HTMLSelectElement;

      if ((event.metaKey || event.ctrlKey) && event.key.toLowerCase() === "f") {
        event.preventDefault();
        searchRef.current?.focus();
        searchRef.current?.select();
        return;
      }
      if (event.key === "/" && !typing) {
        event.preventDefault();
        searchRef.current?.focus();
        return;
      }
      if (event.key === "Escape") {
        if (typing) {
          setQuery("");
          searchRef.current?.blur();
        } else {
          setSelectedId(null);
        }
      }
    };
    window.addEventListener("keydown", onKeyDown);
    return () => window.removeEventListener("keydown", onKeyDown);
  }, []);

  const ecosystems = inventory?.ecosystems ?? [];

  const checkableEcosystems = useMemo(
    () =>
      new Set(
        ecosystems.filter((eco) => eco.checkable).map((eco) => eco.id),
      ),
    [ecosystems],
  );

  const allRows = useMemo(() => {
    if (!inventory) return [];
    return resolveRows(
      inventory.ecosystems.flatMap((eco) => eco.items),
      updates,
      checkableEcosystems,
    );
  }, [inventory, updates, checkableEcosystems]);

  const counts = useMemo(() => {
    const map = new Map<string, { total: number; outdated: number }>();
    for (const row of allRows) {
      const entry = map.get(row.ecosystem) ?? { total: 0, outdated: 0 };
      entry.total += 1;
      if (row.statusResolved === "outdated") entry.outdated += 1;
      map.set(row.ecosystem, entry);
    }
    return map;
  }, [allRows]);

  const outdatedTotal = useMemo(
    () => allRows.filter((row) => row.statusResolved === "outdated").length,
    [allRows],
  );

  const visibleRows = useMemo(() => {
    const needle = query.trim().toLowerCase();
    let rows = allRows;
    if (scope !== ALL_SCOPE) {
      rows = rows.filter((row) => row.ecosystem === scope);
    }
    if (needle) {
      rows = rows.filter((row) =>
        [row.name, row.version, row.source, kindLabel(row.kind)].some((field) =>
          field.toLowerCase().includes(needle),
        ),
      );
    }
    if (filter !== "all") {
      rows = rows.filter((row) => row.statusResolved === filter);
    }
    return rows;
  }, [allRows, scope, query, filter]);

  // 有搜尋或篩選時強制展開，否則命中結果會藏在收合的分類裡
  const forceExpand = query.trim() !== "" || filter !== "all";

  const view = useMemo(
    () =>
      buildView(
        visibleRows,
        ecosystems,
        scope === ALL_SCOPE,
        sort,
        collapsedGroups,
        forceExpand,
      ),
    [visibleRows, ecosystems, scope, sort, collapsedGroups, forceExpand],
  );

  const toggleGroup = useCallback((id: string) => {
    setCollapsedGroups((prev) => {
      const next = new Set(prev);
      if (next.has(id)) next.delete(id);
      else next.add(id);
      return next;
    });
  }, []);

  const allGroupsCollapsed =
    ecosystems.length > 0 && ecosystems.every((eco) => collapsedGroups.has(eco.id));

  const toggleAllGroups = useCallback(() => {
    setCollapsedGroups((prev) => {
      const every = ecosystems.length > 0 && ecosystems.every((eco) => prev.has(eco.id));
      return every ? new Set<string>() : new Set(ecosystems.map((eco) => eco.id));
    });
  }, [ecosystems]);

  const selected = useMemo<Row | null>(
    () => allRows.find((row) => row.id === selectedId) ?? null,
    [allRows, selectedId],
  );

  const warnings = useMemo(() => {
    const list: string[] = [];
    for (const eco of ecosystems) {
      if (eco.error) list.push(`${eco.label}：${eco.error}`);
      for (const warning of eco.warnings) list.push(`${eco.label}：${warning}`);
    }
    for (const warning of updates?.warnings ?? []) list.push(warning);
    if (scanError) list.push(`掃描失敗：${scanError}`);
    if (checkError) list.push(`版本檢查失敗：${checkError}`);
    return list;
  }, [ecosystems, updates, scanError, checkError]);

  const scopeTotal = scope === ALL_SCOPE ? allRows.length : (counts.get(scope)?.total ?? 0);

  return (
    <div className="app">
      <header className="chrome" data-tauri-drag-region>
        <span className="chrome-title" data-tauri-drag-region>
          Version Manager
        </span>
        <span className="chrome-spacer" data-tauri-drag-region />
        <div className="chrome-actions">
          <button
            type="button"
            className="btn btn--primary"
            onClick={() => void scan()}
            disabled={scanning}
          >
            {scanning ? "掃描中…" : "重新掃描"}
          </button>
          <button
            type="button"
            className="btn"
            onClick={() => void runCheck()}
            disabled={!inventory || scanning || checking}
            title="連到 Homebrew / npm / PyPI / rustup 查最新版本"
          >
            {checking ? "檢查中…" : "檢查更新"}
          </button>
        </div>
      </header>

      <SpecPlate inventory={inventory} total={allRows.length} outdated={outdatedTotal} />

      <div className={`scan-bar ${scanning ? "is-busy" : "is-idle"}`} role="progressbar" aria-label="正在掃描">
        <i />
      </div>

      <div className={`body ${selected ? "has-inspector" : ""}`}>
        <EcosystemRail
          ecosystems={ecosystems}
          counts={counts}
          scope={scope}
          total={allRows.length}
          onSelect={setScope}
        />

        <section className="sheet">
          <Toolbar
            inputRef={searchRef}
            query={query}
            onQuery={setQuery}
            filter={filter}
            onFilter={setFilter}
            sort={sort}
            onSort={setSort}
            shown={visibleRows.length}
            total={scopeTotal}
            checkedAt={updates?.checkedAtMs ?? null}
            onToggleGroups={scope === ALL_SCOPE ? toggleAllGroups : null}
            allGroupsCollapsed={allGroupsCollapsed}
          />

          {warnings.length > 0 && (
            <div className="warnings">
              <button
                type="button"
                className="warnings-toggle"
                onClick={() => setShowWarnings((value) => !value)}
              >
                {warnings.length} 則讀取警告{showWarnings ? "，收合" : "，展開"}
              </button>
              {showWarnings && (
                <ul>
                  {warnings.map((warning, index) => (
                    <li key={`${index}-${warning}`}>{warning}</li>
                  ))}
                </ul>
              )}
            </div>
          )}

          <InventoryTable
            view={view}
            selectedId={selectedId}
            onSelect={setSelectedId}
            onToggleGroup={toggleGroup}
            scanning={scanning}
            hasInventory={inventory !== null}
            emptyHint={query.trim() || filter !== "all" ? "沒有符合條件的項目" : "這個分類目前是空的"}
          />
        </section>

        {selected && <Inspector row={selected} onClose={() => setSelectedId(null)} />}
      </div>

      <footer className="footer">
        <span>只讀模式，不會改動任何系統狀態</span>
        <span className="spacer" />
        {checking && <span className="footer-busy">正在向 Homebrew / npm / PyPI / rustup 查詢版本…</span>}
        {updates && !checking && (
          <span className="mono">版本檢查 {formatTime(updates.checkedAtMs)}</span>
        )}
        {inventory && <span className="mono">掃描 {formatTime(inventory.scannedAtMs)}</span>}
        <span className="mono">{ecosystems.length} 個生態系</span>
      </footer>
    </div>
  );
}
