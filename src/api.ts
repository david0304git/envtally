import { invoke } from "@tauri-apps/api/core";
import type { Inventory, Updates } from "./types";

/** 掃描本機所有生態系（Rust 端只讀執行，不改任何系統狀態） */
export function scanLocal(): Promise<Inventory> {
  return invoke<Inventory>("scan_local");
}

/** 向 Homebrew / npm / PyPI / rustup 檢查是否有新版（會連網，由使用者觸發） */
export function checkUpdates(): Promise<Updates> {
  return invoke<Updates>("check_updates");
}
