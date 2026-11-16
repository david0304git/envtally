pub mod brew;
pub mod node;
pub mod python;
pub mod runtimes;
pub mod rust;
pub mod system;
pub mod upstream;
pub mod updates;

use crate::models::{Ecosystem, Inventory, Item};
use crate::sh;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

/// 單一生態系的掃描結果（部分失敗只記 warning，不讓整個生態系消失）。
pub struct Part {
    pub items: Vec<Item>,
    pub warnings: Vec<String>,
    /// 這個生態系的「環境版本」摘要（收合分類時要看的東西）
    pub headline: Option<String>,
}

impl Part {
    pub fn new() -> Self {
        Part {
            items: Vec::new(),
            warnings: Vec::new(),
            headline: None,
        }
    }

    pub fn warn(&mut self, msg: impl Into<String>) {
        self.warnings.push(msg.into());
    }
}

impl Default for Part {
    fn default() -> Self {
        Self::new()
    }
}

pub type ScanFn = fn() -> Result<Part, String>;

pub fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

/// 把版本號拆成可比較的數字序列（"3.14.7" -> [3, 14, 7]）。
pub fn version_key(version: &str) -> Vec<u64> {
    version
        .trim_start_matches('v')
        .split(|c: char| !c.is_ascii_digit())
        .filter_map(|part| part.parse::<u64>().ok())
        .collect()
}

pub fn first_line(text: &str) -> String {
    text.lines()
        .map(str::trim)
        .find(|l| !l.is_empty())
        .unwrap_or("")
        .to_string()
}

/// 從任意版本輸出裡挖出第一個像版本號的字串（至少兩段數字）。
pub fn pick_version(text: &str) -> Option<String> {
    let bytes = text.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i].is_ascii_digit() {
            let start = i;
            while i < bytes.len() && (bytes[i].is_ascii_digit() || bytes[i] == b'.') {
                i += 1;
            }
            let cand = text[start..i].trim_matches('.');
            if cand.split('.').count() >= 2 && !cand.is_empty() && cand.len() <= 24 {
                return Some(cand.to_string());
            }
        } else {
            i += 1;
        }
    }
    None
}

/// 依執行檔位置判斷來源（表格「來源」欄用）。
pub fn origin_for(path: &str) -> &'static str {
    if path.starts_with("/opt/homebrew/") || path.starts_with("/usr/local/Cellar/") {
        "Homebrew"
    } else if path.starts_with("/Library/Developer/CommandLineTools") {
        "Xcode 命令列工具"
    } else if path.starts_with("/usr/bin/")
        || path.starts_with("/bin/")
        || path.starts_with("/usr/sbin/")
    {
        "macOS 內建"
    } else if path.starts_with("/Library/Frameworks/") {
        "python.org 安裝"
    } else if path.contains("/.nvm/") {
        "nvm"
    } else if path.starts_with("/usr/local/") {
        "Homebrew（Intel 路徑）"
    } else {
        "其他來源"
    }
}

/// (id, 名稱, 說明, 掃描函式, 是否有線上版本比對)
fn specs() -> Vec<(&'static str, &'static str, &'static str, ScanFn, bool)> {
    vec![
        ("brew", "Homebrew", "formula 與 cask", brew::scan, true),
        ("python", "Python", "pip 套件與直譯器", python::scan, true),
        ("node", "Node.js", "nvm 版本與全域 npm", node::scan, true),
        ("rust", "Rust", "rustup toolchain 與 cargo bin", rust::scan, true),
        ("java", "Java", "JDK 執行環境", runtimes::java, true),
        ("dotnet", ".NET", "SDK 與執行階段", runtimes::dotnet, true),
        ("php", "PHP", "直譯器", runtimes::php, true),
        ("ruby", "Ruby", "直譯器與 gem", runtimes::ruby, true),
        ("system", "系統工具", "命令列工具版本", system::scan, false),
    ]
}

fn machine() -> (String, String, String, String) {
    let host = sh::run("scutil", &["--get", "ComputerName"], Duration::from_secs(5))
        .ok()
        .map(|r| r.stdout.trim().to_string())
        .filter(|s| !s.is_empty())
        .or_else(|| {
            sh::run("hostname", &[], Duration::from_secs(5))
                .ok()
                .map(|r| r.stdout.trim().trim_end_matches(".local").to_string())
                .filter(|s| !s.is_empty())
        })
        .unwrap_or_else(|| String::from("這台 Mac"));

    let os = sh::run("sw_vers", &["-productVersion"], Duration::from_secs(5))
        .ok()
        .map(|r| r.stdout.trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| String::from("macOS"));

    let arch = match std::env::consts::ARCH {
        "aarch64" => "arm64",
        other => other,
    }
    .to_string();

    let model = sh::run("sysctl", &["-n", "hw.model"], Duration::from_secs(5))
        .ok()
        .map(|r| r.stdout.trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| String::from("Mac"));

    (host, os, arch, model)
}

/// 並行掃描所有生態系。
pub fn collect() -> Inventory {
    let started = Instant::now();
    let specs = specs();

    let results = std::thread::scope(|scope| {
        let handles: Vec<_> = specs
            .into_iter()
            .map(|(id, label, blurb, f, checkable)| {
                scope.spawn(move || {
                    let t = Instant::now();
                    let outcome = f();
                    (
                        id,
                        label,
                        blurb,
                        checkable,
                        outcome,
                        t.elapsed().as_millis() as u64,
                    )
                })
            })
            .collect();
        handles
            .into_iter()
            .map(|h| h.join())
            .collect::<Vec<_>>()
    });

    let mut ecosystems: Vec<Ecosystem> = Vec::new();
    for result in results {
        let Ok((id, label, blurb, checkable, outcome, duration_ms)) = result else {
            continue;
        };
        let mut eco = Ecosystem {
            id: id.to_string(),
            label: label.to_string(),
            blurb: blurb.to_string(),
            items: Vec::new(),
            checkable,
            headline: None,
            error: None,
            warnings: Vec::new(),
            duration_ms,
        };
        match outcome {
            Ok(part) => {
                eco.items = part.items;
                eco.warnings = part.warnings;
                eco.headline = part.headline;
            }
            Err(err) => eco.error = Some(err),
        }
        eco.items.sort_by(|a, b| {
            a.name
                .to_lowercase()
                .cmp(&b.name.to_lowercase())
                .then_with(|| a.version.cmp(&b.version))
        });
        eco.items.dedup_by(|a, b| a.id == b.id);
        ecosystems.push(eco);
    }

    let (host, os, arch, model) = machine();
    let _ = started;

    Inventory {
        scanned_at_ms: now_ms(),
        host,
        os,
        arch,
        model,
        ecosystems,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pick_version_handles_common_formats() {
        assert_eq!(pick_version("git version 2.54.0 (Apple Git-157)").as_deref(), Some("2.54.0"));
        assert_eq!(pick_version("curl 8.7.1 (x86_64-apple-darwin26.0)").as_deref(), Some("8.7.1"));
        assert_eq!(pick_version("jq-1.7.1-apple").as_deref(), Some("1.7.1"));
        assert_eq!(pick_version("3.54.0 2026-04-09 12:25:13 8fa").as_deref(), Some("3.54.0"));
        assert_eq!(
            pick_version("PHP 8.4.11 (cli) (built: Jul 29 2025 15:30:21)").as_deref(),
            Some("8.4.11")
        );
        assert_eq!(pick_version("no version here").as_deref(), None);
    }

    #[test]
    fn item_ids_are_unique_and_patchable() {
        let a = Item::new("python", "Python 3.13.6", "numpy", "2.3.1");
        let b = Item::new("python", "Python 3.14.0", "numpy", "2.3.1");
        assert_ne!(a.id, b.id);
        assert_ne!(a.key, b.key);
        assert!(a.id.starts_with(&a.key));
    }

    /// 真的去問上游有沒有新版（會連網，pip 那段最慢）：cargo test -- --ignored real_check_updates --nocapture
    #[test]
    #[ignore = "需要網路"]
    fn real_check_updates() {
        let updates = updates::check();
        println!("檢查成功的生態系: {:?}", updates.checked_ecosystems);
        println!("可更新項目: {}", updates.patches.len());
        for patch in updates.patches.iter().take(4) {
            println!("  樣本 {} -> {}", patch.key, patch.latest);
        }
        for ecosystem in ["rust", "java", "dotnet", "php", "ruby"] {
            for patch in updates
                .patches
                .iter()
                .filter(|patch| patch.key.starts_with(ecosystem))
            {
                println!("  [{ecosystem}] {} -> {}", patch.key, patch.latest);
            }
        }
        for warning in &updates.warnings {
            println!("  警告: {warning}");
        }
        assert!(!updates.checked_ecosystems.is_empty(), "沒有任何生態系檢查成功");
    }

    /// 把真實掃描結果寫成前端預覽用的資料：cargo test -- --ignored dump_inventory_json
    #[test]
    #[ignore = "產生預覽資料"]
    fn dump_inventory_json() {
        let inventory = collect();
        let json = serde_json::to_string(&inventory).expect("序列化失敗");
        let dir = std::path::Path::new("../public");
        std::fs::create_dir_all(dir).expect("建立 public/ 失敗");
        std::fs::write(dir.join("preview-inventory.json"), json).expect("寫入失敗");
        println!("寫出 public/preview-inventory.json");
    }

    /// 需要真實本機環境，平常不跑：cargo test -- --ignored
    #[test]
    #[ignore = "產生預覽資料（會連網）"]
    fn dump_updates_json() {
        let updates = updates::check();
        let json = serde_json::to_string(&updates).expect("序列化失敗");
        let dir = std::path::Path::new("../public");
        std::fs::create_dir_all(dir).expect("建立 public/ 失敗");
        std::fs::write(dir.join("preview-updates.json"), json).expect("寫入失敗");
        println!(
            "寫出 public/preview-updates.json（{} 筆可更新）",
            updates.patches.len()
        );
    }

    #[test]
    #[ignore = "寫檔案（工作區內）"]
    fn real_export_bundle() {
        let inventory = collect();
        let base = std::path::Path::new("target/export-test");
        for lang in ["zh-TW", "en"] {
            let result =
                crate::export::write_bundle_at(&inventory, lang, base).expect("匯出失敗");
            println!("[{lang}] {} → {:?}", result.folder, result.files);
        }
    }

    #[test]
    #[ignore = "需要本機環境"]
    fn real_scan_finds_items() {
        let inv = collect();
        assert!(!inv.ecosystems.is_empty());
        let total: usize = inv.ecosystems.iter().map(|e| e.items.len()).sum();
        for eco in &inv.ecosystems {
            println!(
                "{} ({}) -> {} 項, {} ms, error={:?}, warnings={:?}",
                eco.label,
                eco.id,
                eco.items.len(),
                eco.duration_ms,
                eco.error,
                eco.warnings
            );
        }
        assert!(total > 0, "任何生態系都沒抓到東西");
        assert!(inv.os.chars().next().map(|c| c.is_ascii_digit()).unwrap_or(false));
    }
}
