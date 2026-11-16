import { invoke } from "@tauri-apps/api/core";
import type { ExportResult, Inventory, Updates } from "./types";

/** 掃描本機所有生態系（Rust 端只讀執行，不改任何系統狀態） */
export function scanLocal(): Promise<Inventory> {
  return invoke<Inventory>("scan_local");
}

/** 向各上游檢查是否有新版（會連網，由使用者觸發） */
export function checkUpdates(): Promise<Updates> {
  return invoke<Updates>("check_updates");
}

/** 把目前盤點結果寫成重建清單（Brewfile / requirements / npm / gems / global.json / REBUILD.md） */
export function exportRebuild(inventory: Inventory, lang: string): Promise<ExportResult> {
  return invoke<ExportResult>("export_rebuild", { inventory, lang });
}

/** 在 Finder 裡選取匯出的資料夾 */
export function revealInFinder(path: string): Promise<void> {
  return invoke<void>("reveal_in_finder", { path });
}
