import type { Ecosystem, Item, Status, Updates } from "./types";

export const ALL_SCOPE = "all";

export interface Row extends Item {
  statusResolved: Status;
  latestResolved: string | null;
  /// 這個分類有沒有線上版本比對（沒有的話「未檢查」是死路，不能那樣寫）
  checkable: boolean;
}

export type SortKey = "name" | "version" | "source";
export type StatusFilter = "all" | "outdated" | "current" | "unknown";

export type ViewRow =
  | {
      type: "group";
      id: string;
      label: string;
      blurb: string;
      count: number;
      outdated: number;
    }
  | { type: "item"; row: Row };

const KIND_LABEL: Record<string, string> = {
  runtime: "執行環境",
  toolchain: "工具鏈",
  formula: "套件",
  cask: "應用程式",
  package: "套件",
  gem: "Gem",
  sdk: "SDK",
  binary: "執行檔",
  tool: "工具",
};

export function kindLabel(kind: string): string {
  return KIND_LABEL[kind] ?? kind;
}

export function statusLabel(status: Status, latest: string | null, checkable: boolean): string {
  switch (status) {
    case "outdated":
      return latest ? `可更新至 ${latest}` : "可更新";
    case "current":
      return "已是最新";
    case "error":
      return "讀取失敗";
    default:
      return checkable
        ? "還沒比對上游版本（按「檢查更新」）"
        : "不檢查這個分類的版本";
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
  if (sort === "version") {
    return naturalCompare(a.version, b.version) || naturalCompare(a.name, b.name);
  }
  if (sort === "source") {
    return naturalCompare(a.source, b.source) || naturalCompare(a.name, b.name);
  }
  return naturalCompare(a.name, b.name);
}

/** 全部檢視時依生態系分組，其餘檢視時只是一個排序好的清單。 */
export function buildView(
  rows: Row[],
  ecosystems: Ecosystem[],
  grouped: boolean,
  sort: SortKey,
): ViewRow[] {
  if (!grouped) {
    return [...rows]
      .sort((a, b) => compareRows(a, b, sort))
      .map((row) => ({ type: "item", row }));
  }

  const view: ViewRow[] = [];
  for (const eco of ecosystems) {
    const group = rows.filter((row) => row.ecosystem === eco.id);
    if (group.length === 0) continue;
    group.sort((a, b) => compareRows(a, b, sort));
    view.push({
      type: "group",
      id: eco.id,
      label: eco.label,
      blurb: eco.blurb,
      count: group.length,
      outdated: group.filter((row) => row.statusResolved === "outdated").length,
    });
    for (const row of group) view.push({ type: "item", row });
  }
  return view;
}

export function formatTime(ms: number): string {
  return new Intl.DateTimeFormat("zh-TW", {
    hour: "2-digit",
    minute: "2-digit",
    second: "2-digit",
    hour12: false,
  }).format(new Date(ms));
}
