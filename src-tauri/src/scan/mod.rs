pub mod agents;
pub mod brew;
pub mod bun;
pub mod choco;
pub mod cli;
pub mod deno;
pub mod go;
pub mod groups;
pub mod node;
pub mod python;
pub mod runtimes;
pub mod rust;
pub mod swift;
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
    /// 這台機器根本沒有這個環境 → 分類直接隱藏，不當成錯誤
    pub missing: bool,
}

impl Part {
    pub fn new() -> Self {
        Part {
            items: Vec::new(),
            warnings: Vec::new(),
            headline: None,
            missing: false,
        }
    }

    /// 這台機器沒有這個環境（UI 會隱藏整個分類）
    pub fn absent() -> Self {
        Part {
            items: Vec::new(),
            warnings: Vec::new(),
            headline: None,
            missing: true,
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
/// 分組收合時顯示的摘要：有共同版本就顯示版本，沒有才退回指令數。
fn group_headline(items: &[crate::models::Item]) -> String {
    let mut versions: Vec<&str> = Vec::new();
    for item in items {
        let version = item.version.trim();
        if version.is_empty() || version == "—" || version == "未知" {
            continue;
        }
        if !versions.iter().any(|known| *known == version) {
            versions.push(version);
        }
    }
    match versions.len() {
        0 => format!("{} 個指令", items.len()),
        1 => versions[0].to_string(),
        2 | 3 => versions.join(" · "),
        _ => format!("{} 個指令", items.len()),
    }
}

/// 「系統附帶」的標籤：macOS 說 macOS 內建，其他平台說系統內建
pub fn builtin_label() -> &'static str {
    if cfg!(target_os = "macos") {
        "macOS 內建"
    } else {
        "系統內建"
    }
}

pub fn origin_for(path: &str) -> &'static str {
    let lower = path.to_lowercase();
    if lower.starts_with("c:\\program files") {
        return "系統安裝";
    }
    if lower.contains("\\scoop\\") {
        return "Scoop";
    }
    if lower.contains("\\chocolatey\\") {
        return "Chocolatey";
    }
    if path.starts_with("/opt/homebrew/") || path.starts_with("/usr/local/Cellar/") {
        "Homebrew"
    } else if path.starts_with("/Library/Developer/CommandLineTools") {
        "Xcode 命令列工具"
    } else if path.starts_with("/usr/bin/")
        || path.starts_with("/bin/")
        || path.starts_with("/usr/sbin/")
    {
        builtin_label()
    } else if path.starts_with("/Library/Frameworks/Python.framework/") {
        "python.org 安裝"
    } else if path.starts_with("/Library/Frameworks/") {
        "框架安裝"
    } else if path.contains("/.deno/") {
        "Deno 安裝器"
    } else if path.contains("/.bun/") {
        "Bun 安裝器"
    } else if path.starts_with("/usr/local/go/") || path.contains("/go/bin/") {
        "Go 安裝器"
    } else if path.contains("/.nvm/") {
        "nvm"
    } else if path.starts_with("/usr/local/") {
        "其他來源"
    } else {
        "其他來源"
    }
}

/// (id, 名稱, 說明, 掃描函式, 是否有線上版本比對)
fn specs() -> Vec<(&'static str, &'static str, &'static str, ScanFn, bool)> {
    vec![
        ("brew", "Homebrew", "formula 與 cask", brew::scan, true),
        ("choco", "Chocolatey", "Windows 套件管理器", choco::scan, true),
        ("python", "Python", "pip 套件與直譯器", python::scan, true),
        ("node", "Node.js", "nvm 版本與全域 npm", node::scan, true),
        ("rust", "Rust", "rustup toolchain 與 cargo bin", rust::scan, true),
        ("java", "Java", "JDK 執行環境", runtimes::java, true),
        ("dotnet", ".NET", "SDK 與執行階段", runtimes::dotnet, true),
        ("php", "PHP", "直譯器", runtimes::php, true),
        ("ruby", "Ruby", "直譯器與 gem", runtimes::ruby, true),
        ("go", "Go", "工具鏈與 go install 執行檔", go::scan, false),
        ("swift", "Swift", "Swift 工具鏈與 SDK", swift::scan, false),
        ("deno", "Deno", "執行環境", deno::scan, false),
        ("bun", "Bun", "執行環境與全域套件", bun::scan, false),
        ("cli", "全域指令", "第三方指令", cli::scan, false),
        ("system", "系統工具", "macOS 內建的命令列工具", system::scan, false),
    ]
}

/// Windows 的顯示名稱：build >= 22000 就是 Windows 11。
/// 抽成純函式，這樣在 macOS 上也能用單元測試驗證。
#[cfg_attr(not(target_os = "windows"), allow(dead_code))]
fn windows_os_label(version: &str) -> String {
    let build = version
        .split('.')
        .nth(2)
        .and_then(|part| part.parse::<u32>().ok())
        .unwrap_or(0);
    if build >= 22000 {
        format!("Windows 11 ({version})")
    } else {
        format!("Windows {version}")
    }
}

#[cfg(target_os = "windows")]
fn machine() -> (String, String, String, String) {
    let host = std::env::var("COMPUTERNAME").unwrap_or_else(|_| String::from("Windows PC"));
    // ver 的輸出：Microsoft Windows [Version 10.0.22631.3155]
    // build >= 22000 就是 Windows 11
    let os = sh::run("cmd", &["/c", "ver"], Duration::from_secs(10))
        .ok()
        .and_then(|out| pick_version(out.text()))
        .map(|version| windows_os_label(&version))
        .unwrap_or_else(|| String::from("Windows"));
    let arch = match std::env::consts::ARCH {
        "aarch64" => "arm64",
        other => other,
    }
    .to_string();
    let model = sh::run(
        "powershell",
        &[
            "-NoProfile",
            "-Command",
            "(Get-CimInstance Win32_ComputerSystem).Model",
        ],
        Duration::from_secs(15),
    )
    .ok()
    .map(|out| out.stdout.trim().to_string())
    .filter(|text| !text.is_empty() && text != "System Product Name")
    .unwrap_or_else(|| String::from("Windows PC"));
    (host, os, arch, model)
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
fn machine() -> (String, String, String, String) {
    let host = sh::run("hostname", &[], Duration::from_secs(5))
        .ok()
        .map(|out| out.stdout.trim().to_string())
        .filter(|text| !text.is_empty())
        .unwrap_or_else(|| String::from("localhost"));
    let model = String::from("PC");
    let os = sh::run("uname", &["-r"], Duration::from_secs(5))
        .ok()
        .map(|out| format!("Linux {}", out.stdout.trim()))
        .unwrap_or_else(|| String::from("Linux"));
    (
        host,
        os,
        std::env::consts::ARCH.to_string(),
        model,
    )
}

#[cfg(target_os = "macos")]
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
        .map(|r| format!("macOS {}", r.stdout.trim()))
        .filter(|text| text != "macOS ")
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
/// 標記 AI／Agent CLI，並補上對應的更新指令（只顯示、不執行）
fn mark_agent_cli(item: &mut crate::models::Item) {
    if !agents::is_agent_cli(&item.name) {
        return;
    }
    item.agent = true;
    if item.update_command.is_some() {
        return;
    }
}

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
            available: true,
            checkable,
            headline: None,
            error: None,
            warnings: Vec::new(),
            duration_ms,
        };
        match outcome {
            Ok(part) => {
                let mut items = part.items;
                for item in &mut items {
                    mark_agent_cli(item);
                }
                eco.items = items;
                eco.warnings = part.warnings;
                eco.headline = part.headline;
                eco.available = !part.missing;
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

    // 「全域指令」再依程式分組：R、VirtualBox、uv…各自成為一個分類，
    // 剩下的才留在「全域指令」。這樣每個分類都對應一個實際的安裝來源。
    if let Some(position) = ecosystems.iter().position(|eco| eco.id == "cli") {
        let items = std::mem::take(&mut ecosystems[position].items);
        let mut buckets: Vec<(&'static groups::Rule, Vec<crate::models::Item>)> = Vec::new();
        let mut residual: Vec<crate::models::Item> = Vec::new();

        for mut item in items {
            let target = item.location.clone().unwrap_or_default();
            match groups::owner_of(&item.name, &target) {
                Some(rule) => {
                    // 搬家時 ecosystem/key/id 都要跟著改，否則前端分組對不上
                    item.ecosystem = rule.id.to_string();
                    item.key = format!("{}|{}|{}", item.ecosystem, item.source, item.name);
                    item.id = format!("{}|{}", item.key, item.version);
                    match buckets.iter_mut().find(|(known, _)| known.id == rule.id) {
                        Some((_, bucket)) => bucket.push(item),
                        None => buckets.push((rule, vec![item])),
                    }
                }
                None => residual.push(item),
            }
        }

        let duration = ecosystems[position].duration_ms;
        ecosystems[position].items = residual;
        // 「全域指令」是收納用的分類，沒有單一版本可代表 → 不顯示 headline，
        // 收合時改為顯示分類說明（與「系統工具」一致）
        ecosystems[position].headline = None;
        if ecosystems[position].items.is_empty() {
            ecosystems.remove(position);
        }

        // 具體程式排在「全域指令」前面，後者留作收尾（沒有被歸類的）
        buckets.sort_by(|a, b| a.0.label.cmp(b.0.label));
        let insert_at = ecosystems
            .iter()
            .position(|eco| eco.id == "cli")
            .unwrap_or(position.min(ecosystems.len()));

        for (offset, (rule, bucket)) in buckets.into_iter().enumerate() {
            ecosystems.insert(
                (insert_at + offset).min(ecosystems.len()),
                Ecosystem {
                    id: rule.id.to_string(),
                    label: rule.label.to_string(),
                    blurb: rule.blurb.to_string(),
                    headline: Some(group_headline(&bucket)),
                    items: bucket,
                    available: true,
                    checkable: false,
                    error: None,
                    warnings: Vec::new(),
                    duration_ms: duration,
                },
            );
        }
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
    fn group_headline_prefers_the_version() {
        let items = vec![
            crate::models::Item::new("x", "s", "a", "7.2.2"),
            crate::models::Item::new("x", "s", "b", "7.2.2"),
            crate::models::Item::new("x", "s", "c", "—"),
        ];
        assert_eq!(super::group_headline(&items), "7.2.2");

        let mixed = vec![
            crate::models::Item::new("x", "s", "a", "4.5.1"),
            crate::models::Item::new("x", "s", "b", "4.5.2"),
        ];
        assert_eq!(super::group_headline(&mixed), "4.5.1 · 4.5.2");

        let unknown = vec![
            crate::models::Item::new("x", "s", "a", "—"),
            crate::models::Item::new("x", "s", "b", "未知"),
        ];
        assert_eq!(super::group_headline(&unknown), "2 個指令");
    }

    #[test]
    fn builtin_label_matches_the_platform() {
        let label = builtin_label();
        if cfg!(target_os = "macos") {
            assert_eq!(label, "macOS 內建");
        } else {
            assert_eq!(label, "系統內建");
        }
        // /usr/bin 底下的工具要用同一個標籤
        assert_eq!(origin_for("/usr/bin/git"), label);
        assert_eq!(origin_for("/bin/zsh"), label);
    }

    #[test]
    fn windows_os_label_picks_the_right_name() {
        assert_eq!(
            windows_os_label("10.0.22631.3155"),
            "Windows 11 (10.0.22631.3155)"
        );
        assert_eq!(windows_os_label("10.0.19045.1"), "Windows 10.0.19045.1");
        // 非預期格式也不該 panic
        assert_eq!(windows_os_label("odd"), "Windows odd");
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
                "{} ({}) -> {} 項, {} ms, available={}, error={:?}, warnings={:?}",
                eco.label,
                eco.id,
                eco.items.len(),
                eco.duration_ms,
                eco.available,
                eco.error,
                eco.warnings
            );
        }
        assert!(total > 0, "任何生態系都沒抓到東西");
        assert!(!inv.os.is_empty(), "os 名稱是空的");
        assert!(inv.os.chars().any(|c| c.is_ascii_digit()), "os 應含版本號：{}", inv.os);
    }
}
