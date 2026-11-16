mod models;
mod scan;
mod sh;

use models::{Inventory, Updates};

/// 掃描本機所有生態系（只讀，不改動任何系統狀態）。
#[tauri::command]
async fn scan_local() -> Result<Inventory, String> {
    tauri::async_runtime::spawn_blocking(scan::collect)
        .await
        .map_err(|e| format!("掃描失敗：{e}"))
}

/// 對上游檢查版本（會連網，由使用者按鈕觸發）。
#[tauri::command]
async fn check_updates() -> Result<Updates, String> {
    tauri::async_runtime::spawn_blocking(scan::updates::check)
        .await
        .map_err(|e| format!("版本檢查失敗：{e}"))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![scan_local, check_updates])
        .run(tauri::generate_context!())
        .expect("啟動 Tauri 應用失敗");
}
