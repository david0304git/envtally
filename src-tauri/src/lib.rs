mod export;
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

/// 把目前盤點結果寫成重建清單（Brewfile / requirements / npm / gems / global.json / REBUILD.md）
#[tauri::command]
async fn export_rebuild(inventory: Inventory, lang: String) -> Result<export::ExportResult, String> {
    tauri::async_runtime::spawn_blocking(move || export::write_bundle(&inventory, &lang))
        .await
        .map_err(|e| format!("匯出失敗：{e}"))?
}

/// 在 Finder 裡選取匯出的資料夾
#[tauri::command]
fn reveal_in_finder(path: String) -> Result<(), String> {
    export::reveal(&path)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            scan_local,
            check_updates,
            export_rebuild,
            reveal_in_finder
        ])
        .run(tauri::generate_context!())
        .expect("啟動 Tauri 應用失敗");
}
