import type { Lang } from "./i18n";
import { localeOf } from "./i18n";
import type { T } from "./i18nContext";
import type { Ecosystem, Item, Status, Updates } from "./types";

export const ALL_SCOPE = "all";

export interface Row extends Item {
  statusResolved: Status;
  latestResolved: string | null;
  /// 這個分類有沒有線上版本比對（沒有的話「未檢查」是死路，不能那樣寫）
  checkable: boolean;
}

export type SortKey = "name" | "version" | "latest" | "source" | "kind";
export type SortDir = "asc" | "desc";
export type StatusFilter = "all" | "outdated" | "current" | "unknown" | "agent";

export type ViewRow =
  | {
      type: "group";
      id: string;
      label: string;
      blurb: string;
      /// 收合時要看的環境版本（例如 "3.14.7 · 3.13.6"）
      headline: string | null;
      count: number;
      outdated: number;
      collapsed: boolean;
    }
  | { type: "item"; row: Row };

const KIND_KEY: Record<string, string> = {
  runtime: "kind.runtime",
  toolchain: "kind.toolchain",
  formula: "kind.package",
  cask: "kind.cask",
  package: "kind.package",
  gem: "kind.gem",
  sdk: "kind.sdk",
  binary: "kind.binary",
  tool: "kind.tool",
};

/** 回傳 i18n key（呼叫端用 t() 翻） */
export function kindKey(kind: string): string {
  return KIND_KEY[kind] ?? kind;
}

export function kindLabel(kind: string, t: T): string {
  const key = KIND_KEY[kind];
  return key ? t(key) : kind;
}

/** 分類名稱：優先用字典（三語），沒有就用後端給的標籤 */
export function ecoLabel(id: string, label: string, t: T): string {
  const key = `eco.${id}`;
  const translated = t(key);
  return translated === key ? label : translated;
}

export function ecoBlurb(id: string, blurb: string, t: T): string {
  const key = `eco.${id}.blurb`;
  const translated = t(key);
  return translated === key ? blurb : translated;
}

export function statusLabel(
  status: Status,
  latest: string | null,
  checkable: boolean,
  t: T,
): string {
  switch (status) {
    case "outdated":
      return latest ? t("status.outdated", { version: latest }) : t("status.outdatedPlain");
    case "current":
      return t("status.current");
    case "error":
      return t("status.error");
    default:
      return checkable ? t("status.unknown") : t("status.notCheckable");
  }
}

/** 把「版本檢查結果」疊到掃描結果上，得到每列要顯示的狀態。 */
export function resolveRows(
  items: Item[],
  updates: Updates | null,
  checkableEcosystems: ReadonlySet<string>,
): Row[] {
  if (!updates) {
    return items.map((item) => ({
      ...item,
      statusResolved: "unknown" as Status,
      latestResolved: null,
      checkable: checkableEcosystems.has(item.ecosystem),
    }));
  }
  const patches = new Map(updates.patches.map((patch) => [patch.key, patch.latest]));
  const checked = new Set(updates.checkedEcosystems);
  return items.map((item) => {
    const latest = patches.get(item.key) ?? null;
    const statusResolved: Status = latest
      ? "outdated"
      : checked.has(item.ecosystem)
        ? "current"
        : "unknown";
    return {
      ...item,
      statusResolved,
      latestResolved: latest,
      checkable: checkableEcosystems.has(item.ecosystem),
    };
  });
}

function naturalCompare(a: string, b: string): number {
  return a.localeCompare(b, "en", { numeric: true, sensitivity: "base" });
}

export function compareRows(a: Row, b: Row, sort: SortKey): number {
  const byName = () => naturalCompare(a.name, b.name);
  switch (sort) {
    case "version":
      return naturalCompare(a.version, b.version) || byName();
    case "latest":
      return naturalCompare(a.latestResolved ?? "", b.latestResolved ?? "") || byName();
    case "source":
      return naturalCompare(a.source, b.source) || byName();
    case "kind":
      // 用原始 kind 排（不受介面語言影響）
      return naturalCompare(a.kind, b.kind) || byName();
    default:
      return byName();
  }
}

export function sortRows(rows: Row[], sort: SortKey, dir: SortDir): Row[] {
  const sorted = [...rows].sort((a, b) => compareRows(a, b, sort));
  // 第二段（名稱）也一起反轉，維持「反向排序」的直覺
  return dir === "desc" ? sorted.reverse() : sorted;
}

/** 全部檢視時依生態系分組，其餘檢視時只是一個排序好的清單。 */
export function buildView(
  rows: Row[],
  ecosystems: Ecosystem[],
  grouped: boolean,
  sort: SortKey,
  dir: SortDir,
  collapsedGroups: ReadonlySet<string>,
  forceExpand: boolean,
): ViewRow[] {
  if (!grouped) {
    return sortRows(rows, sort, dir).map((row) => ({ type: "item", row }));
  }

  const view: ViewRow[] = [];
  for (const eco of ecosystems) {
    const group = rows.filter((row) => row.ecosystem === eco.id);
    if (group.length === 0) continue;
    const sorted = sortRows(group, sort, dir);
    const collapsed = !forceExpand && collapsedGroups.has(eco.id);
    view.push({
      type: "group",
      id: eco.id,
      label: eco.label,
      blurb: eco.blurb,
      headline: eco.headline,
      count: group.length,
      outdated: group.filter((row) => row.statusResolved === "outdated").length,
      collapsed,
    });
    if (collapsed) continue;
    for (const row of sorted) view.push({ type: "item", row });
  }
  return view;
}

export function formatTime(ms: number, lang: Lang): string {
  return new Intl.DateTimeFormat(localeOf(lang), {
    hour: "2-digit",
    minute: "2-digit",
    second: "2-digit",
    hour12: false,
  }).format(new Date(ms));
}
