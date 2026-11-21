import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { checkUpdates, exportRebuild, revealInFinder, scanLocal } from "./api";
import EcosystemRail from "./components/EcosystemRail";
import Inspector from "./components/Inspector";
import InventoryTable from "./components/InventoryTable";
import SpecPlate from "./components/SpecPlate";
import Toolbar from "./components/Toolbar";
import { LANGS, type Lang } from "./i18n";
import { LangProvider, useT } from "./i18nContext";
import {
  ALL_SCOPE,
  buildView,
  ecoLabel,
  formatTime,
  kindKey,
  resolveRows,
  type Row,
  type SortDir,
  type SortKey,
  type StatusFilter,
} from "./rows";
import { matchesQuery } from "./search";
import { SettingsContext, useSettings, type SettingsValue } from "./settings";
import type { ExportResult, Inventory, Updates } from "./types";
import {
  applyThemeMode,
  readLang,
  readThemeMode,
  saveLang,
  saveThemeMode,
  type ThemeMode,
} from "./theme";

export default function App() {
  const [lang, setLangState] = useState<Lang>(readLang);
  const [theme, setThemeState] = useState<ThemeMode>(readThemeMode);

  const setLang = useCallback((next: Lang) => {
    setLangState(next);
    saveLang(next);
  }, []);
  const setTheme = useCallback((next: ThemeMode) => {
    setThemeState(next);
    saveThemeMode(next);
  }, []);

  useEffect(() => {
    document.documentElement.lang = lang;
  }, [lang]);

  useEffect(() => {
    applyThemeMode(theme);
    if (theme !== "auto" || !window.matchMedia) return;
    const media = window.matchMedia("(prefers-color-scheme: dark)");
    const onChange = () => applyThemeMode("auto");
    media.addEventListener("change", onChange);
    return () => media.removeEventListener("change", onChange);
  }, [theme]);

  const settings = useMemo<SettingsValue>(
    () => ({ lang, theme, setLang, setTheme }),
    [lang, theme, setLang, setTheme],
  );

  return (
    <SettingsContext.Provider value={settings}>
      <LangProvider lang={lang}>
        <Workspace />
      </LangProvider>
    </SettingsContext.Provider>
  );
}

function Workspace() {
  const t = useT();
  // macOS 顯示 ⌘，其他平台顯示 Ctrl+
  const modLabel = useMemo(
    () => (navigator.userAgent.includes("Macintosh") ? "⌘" : "Ctrl+"),
    [],
  );
  const { lang, theme, setLang, setTheme } = useSettings();

  const [inventory, setInventory] = useState<Inventory | null>(null);
  // 第一次載入後預設收合；之後重新掃描保留使用者展開的狀態
  const collapsedInitialized = useRef(false);
  const [scanning, setScanning] = useState(true);
  const [scanError, setScanError] = useState<string | null>(null);
  const [updates, setUpdates] = useState<Updates | null>(null);
  const [checking, setChecking] = useState(false);
  const [checkError, setCheckError] = useState<string | null>(null);

  const [scope, setScope] = useState<string>(ALL_SCOPE);
  const [query, setQuery] = useState("");
  const [filter, setFilter] = useState<StatusFilter>("all");
  const [sort, setSort] = useState<SortKey>("name");
  const [sortDir, setSortDir] = useState<SortDir>("asc");
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const [collapsedGroups, setCollapsedGroups] = useState<Set<string>>(new Set());

  const [settingsOpen, setSettingsOpen] = useState(false);
  const [warnOpen, setWarnOpen] = useState(false);
  const warnRef = useRef<HTMLDivElement | null>(null);
  const [exporting, setExporting] = useState(false);
  const [toast, setToast] = useState<
    { kind: "ok"; result: ExportResult } | { kind: "error"; message: string } | null
  >(null);

  const searchRef = useRef<HTMLInputElement>(null);
  const settingsRef = useRef<HTMLDivElement>(null);

  const scan = useCallback(async () => {
    setScanning(true);
    setScanError(null);
    try {
      const next = await scanLocal();
      setInventory(next);
      if (!collapsedInitialized.current) {
        collapsedInitialized.current = true;
        setCollapsedGroups(
          new Set(next.ecosystems.filter((eco) => eco.available).map((eco) => eco.id)),
        );
      }
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

  const runExport = useCallback(async () => {
    if (!inventory || exporting) return;
    setExporting(true);
    try {
      const result = await exportRebuild(inventory, lang);
      setToast({ kind: "ok", result });
    } catch (error) {
      setToast({ kind: "error", message: error instanceof Error ? error.message : String(error) });
    } finally {
      setExporting(false);
    }
  }, [inventory, exporting, lang]);

  useEffect(() => {
    if (!toast) return;
    const timer = window.setTimeout(() => setToast(null), 15000);
    return () => window.clearTimeout(timer);
  }, [toast]);

  useEffect(() => {
    if (!warnOpen) return;
    const closeWarn = (event: MouseEvent) => {
      if (!warnRef.current?.contains(event.target as Node)) setWarnOpen(false);
    };
    document.addEventListener("mousedown", closeWarn);
    return () => document.removeEventListener("mousedown", closeWarn);
  }, [warnOpen]);

  useEffect(() => {
    if (!settingsOpen) return;
    const onPointerDown = (event: PointerEvent) => {
      if (!settingsRef.current?.contains(event.target as Node)) setSettingsOpen(false);
    };
    document.addEventListener("pointerdown", onPointerDown);
    return () => document.removeEventListener("pointerdown", onPointerDown);
  }, [settingsOpen]);

  const allEcosystems = inventory?.ecosystems ?? [];
  const ecosystems = useMemo(
    () => allEcosystems.filter((eco) => eco.available),
    [allEcosystems],
  );

  const scopeOrder = useMemo(
    () => [ALL_SCOPE, ...ecosystems.map((eco) => eco.id)],
    [ecosystems],
  );

  const cycleScope = useCallback(
    (delta: number) => {
      setScope((current) => {
        const index = scopeOrder.indexOf(current);
        const next = (index + delta + scopeOrder.length) % scopeOrder.length;
        return scopeOrder[next] ?? ALL_SCOPE;
      });
    },
    [scopeOrder],
  );

  useEffect(() => {
    const onKeyDown = (event: KeyboardEvent) => {
      const target = event.target as HTMLElement | null;
      const typing = target instanceof HTMLInputElement || target instanceof HTMLSelectElement;
      const meta = event.metaKey || event.ctrlKey;

      if (meta && event.key.toLowerCase() === "r") {
        event.preventDefault();
        void scan();
        return;
      }
      if (meta && event.key.toLowerCase() === "u") {
        event.preventDefault();
        void runCheck();
        return;
      }
      if (meta && event.key.toLowerCase() === "e") {
        event.preventDefault();
        void runExport();
        return;
      }
      if (meta && (event.key === "[" || event.key === "]")) {
        event.preventDefault();
        cycleScope(event.key === "]" ? 1 : -1);
        return;
      }
      if (meta && event.key === "0") {
        event.preventDefault();
        setScope(ALL_SCOPE);
        return;
      }
      if (meta && event.key.toLowerCase() === "f") {
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
        if (warnOpen) {
          setWarnOpen(false);
          return;
        }
        if (settingsOpen) {
          setSettingsOpen(false);
          return;
        }
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
  }, [scan, runCheck, runExport, cycleScope, settingsOpen, warnOpen]);

  const checkableEcosystems = useMemo(
    () => new Set(ecosystems.filter((eco) => eco.checkable).map((eco) => eco.id)),
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
    const needle = query.trim();
    let rows = allRows;
    if (scope !== ALL_SCOPE) {
      rows = rows.filter((row) => row.ecosystem === scope);
    }
    if (needle) {
      rows = rows.filter((row) =>
        matchesQuery(needle, [
          row.name,
          row.version,
          row.source,
          row.location,
          row.description,
          t(kindKey(row.kind)),
        ]),
      );
    }
    if (filter === "agent") {
      rows = rows.filter((row) => row.agent);
    } else if (filter !== "all") {
      rows = rows.filter((row) => row.statusResolved === filter);
    }
    return rows;
  }, [allRows, scope, query, filter, t]);

  // 有搜尋或篩選時強制展開，否則命中結果會藏在收合的分類裡
  const forceExpand = query.trim() !== "" || filter !== "all";

  // 同一欄再按一次就反向；換欄位則從遞增開始
  const handleSort = useCallback((key: SortKey) => {
    // 收合狀態下排序看不到效果，所以排序時一併展開全部分類
    setCollapsedGroups(new Set());
    setSort((current) => {
      if (current === key) {
        setSortDir((dir) => (dir === "asc" ? "desc" : "asc"));
        return current;
      }
      setSortDir("asc");
      return key;
    });
  }, []);

  const view = useMemo(
    () =>
      buildView(
        visibleRows,
        ecosystems,
        scope === ALL_SCOPE,
        sort,
        sortDir,
        collapsedGroups,
        forceExpand,
      ),
    [visibleRows, ecosystems, scope, sort, sortDir, collapsedGroups, forceExpand],
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
      const label = ecoLabel(eco.id, eco.label, t);
      if (eco.error) list.push(`${label}：${eco.error}`);
      for (const warning of eco.warnings) list.push(`${label}：${warning}`);
    }
    for (const warning of updates?.warnings ?? []) list.push(warning);
    if (scanError) list.push(`scan: ${scanError}`);
    if (checkError) list.push(`check: ${checkError}`);
    return list;
  }, [ecosystems, updates, scanError, checkError, t]);

  const scopeTotal = scope === ALL_SCOPE ? allRows.length : (counts.get(scope)?.total ?? 0);

  return (
    <div className="app">
      <header className="chrome" data-tauri-drag-region>
        <span className="chrome-title" data-tauri-drag-region>
          Envbench
        </span>
        <span className="chrome-spacer" data-tauri-drag-region />
        <div className="chrome-actions">
          <button
            type="button"
            className="btn"
            onClick={() => void runExport()}
            disabled={!inventory || exporting}
            data-action="export"
            title={`${t("export.tooltip")} · ${t("shortcut.export", { mod: modLabel })}`}
          >
            {exporting ? t("chrome.exporting") : t("chrome.export")}
          </button>
          <button
            type="button"
            className="btn btn--primary"
            data-action="scan"
            onClick={() => void scan()}
            disabled={scanning}
            title={t("shortcut.scan", { mod: modLabel })}
          >
            {scanning ? t("chrome.scanning") : t("chrome.scan")}
          </button>
          <button
            type="button"
            className="btn"
            data-action="check"
            onClick={() => void runCheck()}
            disabled={!inventory || scanning || checking}
            title={t("shortcut.check", { mod: modLabel })}
          >
            {checking ? t("chrome.checking") : t("chrome.check")}
          </button>
          <div className="settings-wrap" ref={warnRef}>
            <button
              type="button"
              className={`btn btn--icon bell${warnings.length > 0 ? " has-warnings" : ""}`}
              data-action="warnings"
              onClick={() => setWarnOpen((open) => !open)}
              aria-label={t("warn.aria", { n: warnings.length })}
              aria-expanded={warnOpen}
              title={t("warn.aria", { n: warnings.length })}
            >
              <svg width="13" height="13" viewBox="0 0 14 14" aria-hidden="true" fill="none">
                <path
                  d="M3.3 9.7V7.1a3.7 3.7 0 0 1 7.4 0v2.6l.9 1.1H2.4z"
                  stroke="currentColor"
                  strokeWidth="1.2"
                  strokeLinejoin="round"
                />
                <path
                  d="M5.9 12.1a1.2 1.2 0 0 0 2.2 0"
                  stroke="currentColor"
                  strokeWidth="1.2"
                  strokeLinecap="round"
                />
              </svg>
              {warnings.length > 0 && <span className="bell-badge">{warnings.length}</span>}
            </button>
            {warnOpen && (
              <div className="settings warn-pop" role="dialog" aria-label={t("warn.title")}>
                <p className="settings-title">
                  {t("warn.title")}
                  {warnings.length > 0 && <span className="warn-count">{warnings.length}</span>}
                </p>
                {warnings.length === 0 ? (
                  <p className="warn-empty">{t("warn.empty")}</p>
                ) : (
                  <ul className="warn-list">
                    {warnings.map((warning, index) => (
                      <li key={`${index}-${warning}`}>{warning}</li>
                    ))}
                  </ul>
                )}
              </div>
            )}
          </div>
          <div className="settings-wrap" ref={settingsRef}>
            <button
              type="button"
              className="btn btn--icon"
              onClick={() => setSettingsOpen((open) => !open)}
              aria-label={t("chrome.settings")}
              aria-expanded={settingsOpen}
              title={t("chrome.settings")}
            >
              {/* 齒輪：厚齒環 + 8 個短齒（比放射狀線條更不像太陽） */}
              <svg width="13" height="13" viewBox="0 0 14 14" aria-hidden="true">
                <g fill="currentColor">
                  {[0, 45, 90, 135, 180, 225, 270, 315].map((angle) => (
                    <rect
                      key={angle}
                      x="6.15"
                      y="0.5"
                      width="1.7"
                      height="2.2"
                      rx="0.5"
                      transform={`rotate(${angle} 7 7)`}
                    />
                  ))}
                  <circle cx="7" cy="7" r="3.9" fill="none" stroke="currentColor" strokeWidth="1.9" />
                </g>
              </svg>
            </button>
            {settingsOpen && (
              <div className="settings" role="dialog" aria-label={t("settings.title")}>
                <p className="settings-title">{t("settings.title")}</p>
                <div className="settings-row">
                  <span className="settings-label">{t("settings.appearance")}</span>
                  <div className="seg seg--sm">
                    {(["auto", "light", "dark"] as ThemeMode[]).map((mode) => (
                      <button
                        key={mode}
                        type="button"
                        className={theme === mode ? "is-active" : ""}
                        onClick={() => setTheme(mode)}
                      >
                        {t(`settings.${mode}`)}
                      </button>
                    ))}
                  </div>
                </div>
                <div className="settings-row">
                  <span className="settings-label">{t("settings.language")}</span>
                  <div className="seg seg--sm">
                    {LANGS.map((option) => (
                      <button
                        key={option.id}
                        type="button"
                        className={lang === option.id ? "is-active" : ""}
                        onClick={() => setLang(option.id)}
                      >
                        {option.label}
                      </button>
                    ))}
                  </div>
                </div>
              </div>
            )}
          </div>
        </div>
      </header>

      <SpecPlate inventory={inventory} total={allRows.length} outdated={outdatedTotal} />

      <div
        className={`scan-bar ${scanning ? "is-busy" : "is-idle"}`}
        role="progressbar"
        aria-label={t("chrome.scanning")}
      >
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
            shown={visibleRows.length}
            total={scopeTotal}
            checkedAt={updates?.checkedAtMs ?? null}
            onToggleGroups={scope === ALL_SCOPE ? toggleAllGroups : null}
            allGroupsCollapsed={allGroupsCollapsed}
          />

          <InventoryTable
            view={view}
            selectedId={selectedId}
            onSelect={setSelectedId}
            onToggleGroup={toggleGroup}
            sort={sort}
            sortDir={sortDir}
            onSort={handleSort}
            scanning={scanning}
            hasInventory={inventory !== null}
            hasFilter={query.trim() !== "" || filter !== "all"}
          />
        </section>

        {selected && <Inspector row={selected} onClose={() => setSelectedId(null)} />}
      </div>

      <footer className="footer">
        <span className="spacer" />
        {checking && <span className="footer-busy">{t("footer.checking")}</span>}
        {updates && !checking && (
          <span className="mono">
            {t("footer.checkAt", { time: formatTime(updates.checkedAtMs, lang) })}
          </span>
        )}
        {inventory && (
          <span className="mono">
            {t("footer.scanAt", { time: formatTime(inventory.scannedAtMs, lang) })}
          </span>
        )}
        <span className="mono">{t("footer.ecosystems", { n: ecosystems.length })}</span>
      </footer>

      {toast && (
        <div className={`toast ${toast.kind === "ok" ? "" : "is-error"}`} role="status">
          {toast.kind === "ok" ? (
            <>
              <span className="toast-text">
                {t("export.done", { path: toast.result.folder })}
              </span>
              <span className="toast-files mono">{toast.result.files.join(" · ")}</span>
              <button
                type="button"
                className="btn btn--tiny"
                onClick={() => void revealInFinder(toast.result.folder)}
              >
                {t("export.reveal")}
              </button>
            </>
          ) : (
            <span className="toast-text">{t("export.failed", { message: toast.message })}</span>
          )}
          <button
            type="button"
            className="toast-close"
            onClick={() => setToast(null)}
            aria-label={t("insp.close")}
          >
            ×
          </button>
        </div>
      )}
    </div>
  );
}
