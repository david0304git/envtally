#![allow(dead_code)] // 非 macOS 時整支模組只有 absent 版本

use crate::models::Item;
use crate::scan::{first_line, pick_version, Part};
use crate::sh;
use serde::Deserialize;
use std::collections::HashSet;
use std::time::Duration;

#[derive(Deserialize)]
struct BrewInfo {
    #[serde(default)]
    formulae: Vec<BrewFormula>,
    #[serde(default)]
    casks: Vec<BrewCask>,
}

#[derive(Deserialize)]
struct BrewFormula {
    name: String,
    #[serde(default)]
    desc: Option<String>,
    #[serde(default)]
    installed: Vec<BrewKeg>,
}

#[derive(Deserialize)]
struct BrewKeg {
    #[serde(default)]
    version: Option<String>,
}

#[derive(Deserialize)]
struct BrewCask {
    #[serde(default)]
    token: Option<String>,
    #[serde(default)]
    desc: Option<String>,
    #[serde(default)]
    installed: Option<String>,
}

/// `opt/<name>` 是指向「目前生效的那個 keg」的 symlink，
/// 比版本清單的排序可靠（有多版本並存時尤其重要）。
fn linked_version(prefix: &str, name: &str) -> Option<String> {
    let link = std::path::Path::new(prefix).join("opt").join(name);
    let target = std::fs::read_link(link).ok()?;
    let last = target.file_name()?.to_string_lossy().to_string();
    if last.is_empty() || last == name {
        None
    } else {
        Some(last)
    }
}

/// 主要來源：一次拿到 name + 一行說明 + 已安裝版本。
fn collect_from_json(prefix: &str, part: &mut Part) -> Result<(), String> {
    let out = sh::run(
        "brew",
        &["info", "--json=v2", "--installed"],
        Duration::from_secs(180),
    )?;
    if !out.ok() {
        return Err(first_line(out.text()));
    }
    let info: BrewInfo =
        serde_json::from_str(&out.stdout).map_err(|e| format!("JSON 解析失敗：{e}"))?;

    if info.formulae.is_empty() && info.casks.is_empty() {
        return Err(String::from("brew info 回傳空清單"));
    }

    for formula in info.formulae {
        let versions: Vec<String> = formula
            .installed
            .iter()
            .filter_map(|keg| keg.version.clone())
            .collect();

        // 目前生效的版本：先看 opt 連結，再退回 brew 回報的最新安裝版本
        let version = linked_version(prefix, &formula.name)
            .or_else(|| versions.last().cloned())
            .unwrap_or_else(|| String::from("—"));

        let mut item = Item::new("brew", "Homebrew", &formula.name, &version)
            .kind("formula")
            .at(format!("{prefix}/opt/{}", formula.name))
            .cmd(format!("brew upgrade {}", formula.name))
            .rm(format!("brew uninstall {}", formula.name));

        if let Some(desc) = formula.desc {
            item = item.about(desc);
        }
        if versions.len() > 1 {
            let others: Vec<&str> = versions
                .iter()
                .map(String::as_str)
                .filter(|candidate| *candidate != version.as_str())
                .collect();
            if !others.is_empty() {
                let joined = others.join("、");
                item = item.note_i18n(&format!("另有 {joined} 並存"), "note.brew.multi", &[&joined]);
            }
        }
        part.items.push(item);
    }

    for cask in info.casks {
        let Some(token) = cask.token else { continue };
        let version = cask.installed.unwrap_or_else(|| String::from("—"));
        let mut item = Item::new("brew", "Homebrew Cask", &token, &version)
            .kind("cask")
            .at(format!("/Applications/{token}.app"))
            .cmd(format!("brew upgrade --cask {token}"))
            .rm(format!("brew uninstall --cask {token}"));
        if let Some(desc) = cask.desc {
            item = item.about(desc);
        }
        part.items.push(item);
    }

    Ok(())
}

/// 備援：JSON API 不可用時（離線、brew 太舊）退回純版本清單，只是沒有說明。
fn collect_from_legacy(prefix: &str, part: &mut Part) {
    match sh::run(
        "brew",
        &["list", "--versions", "--formula"],
        Duration::from_secs(90),
    ) {
        Ok(r) if r.ok() => {
            for line in r.stdout.lines() {
                let mut tokens = line.split_whitespace();
                let Some(name) = tokens.next() else { continue };
                let versions: Vec<&str> = tokens.collect();
                let version = linked_version(prefix, name)
                    .or_else(|| versions.first().copied().map(str::to_string))
                    .unwrap_or_else(|| String::from("—"));
                let mut item = Item::new("brew", "Homebrew", name, &version)
                    .kind("formula")
                    .at(format!("{prefix}/opt/{name}"))
                    .cmd(format!("brew upgrade {name}"))
                    .rm(format!("brew uninstall {name}"));
                if versions.len() > 1 {
                    let others: Vec<&str> = versions
                        .into_iter()
                        .filter(|candidate| *candidate != version.as_str())
                        .collect();
                    if !others.is_empty() {
                        let joined = others.join("、");
                        item = item.note_i18n(
                            &format!("另有 {joined} 並存"),
                            "note.brew.multi",
                            &[&joined],
                        );
                    }
                }
                part.items.push(item);
            }
        }
        Ok(r) => part.warn(format!("formula 清單讀取失敗：{}", first_line(r.text()))),
        Err(e) => part.warn(format!("formula 清單讀取失敗：{e}")),
    }

    match sh::run(
        "brew",
        &["list", "--versions", "--cask"],
        Duration::from_secs(60),
    ) {
        Ok(r) if r.ok() => {
            for line in r.stdout.lines() {
                let mut tokens = line.split_whitespace();
                let Some(name) = tokens.next() else { continue };
                let version = tokens.last().unwrap_or("—");
                part.items.push(
                    Item::new("brew", "Homebrew Cask", name, version)
                        .kind("cask")
                        .at(format!("/Applications/{name}.app"))
                        .cmd(format!("brew upgrade --cask {name}"))
                        .rm(format!("brew uninstall --cask {name}")),
                );
            }
        }
        Ok(r) => {
            let msg = first_line(r.text());
            if !msg.is_empty() && !msg.contains("No Cask") {
                part.warn(format!("cask 清單讀取失敗：{msg}"));
            }
        }
        Err(e) => part.warn(format!("cask 清單讀取失敗：{e}")),
    }
}

/// `brew info --installed` 偶爾會漏掉幾筆（已停用、被改名、只有舊 keg），
/// 用 `brew list` 對帳補回來，寧可多一筆有註記的，也不要默默漏掉。
fn reconcile_missing(prefix: &str, part: &mut Part) {
    let known: HashSet<String> = part
        .items
        .iter()
        .filter(|item| item.kind == "formula")
        .map(|item| item.name.clone())
        .collect();

    let Ok(out) = sh::run(
        "brew",
        &["list", "--versions", "--formula"],
        Duration::from_secs(90),
    ) else {
        return;
    };
    if !out.ok() {
        return;
    }

    for line in out.stdout.lines() {
        let mut tokens = line.split_whitespace();
        let Some(name) = tokens.next() else { continue };
        if known.contains(name) {
            continue;
        }
        let versions: Vec<&str> = tokens.collect();
        let version = linked_version(prefix, name)
            .or_else(|| versions.last().copied().map(str::to_string))
            .unwrap_or_else(|| String::from("—"));
        part.items.push(
            Item::new("brew", "Homebrew", name, &version)
                .kind("formula")
                .at(format!("{prefix}/opt/{name}"))
                .cmd(format!("brew upgrade {name}"))
                .rm(format!("brew uninstall {name}"))
                .note_i18n(
                    "brew info 沒有回報此 formula（可能已停用或改名）",
                    "note.brew.unreported",
                    &[],
                ),
        );
    }
}

#[cfg(not(target_os = "macos"))]
pub fn scan() -> Result<Part, String> {
    // Homebrew 只存在於 macOS（Linux 版也不在支援範圍）
    Ok(Part::absent())
}

#[cfg(target_os = "macos")]
pub fn scan() -> Result<Part, String> {
    if !sh::exists("brew") {
        return Ok(Part::absent());
    }

    let mut part = Part::new();

    // brew 自己也是指令，放在它所屬的分類（而不是「全域指令」）
    if let Ok(out) = sh::run("brew", &["--version"], Duration::from_secs(30)) {
        let version = pick_version(out.text()).unwrap_or_else(|| String::from("未知"));
        let mut brew_item = Item::new("brew", "Homebrew", "brew", &version)
            .kind("toolchain")
            .about("Homebrew 套件管理器本體：install / upgrade / list");
        if let Some(path) = sh::which("brew") {
            brew_item = brew_item.at(path.display().to_string());
        }
        part.items.push(brew_item);
    }
    let prefix = sh::run("brew", &["--prefix"], Duration::from_secs(20))
        .ok()
        .map(|r| r.stdout.trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| String::from("/opt/homebrew"));

    if let Ok(r) = sh::run("brew", &["--version"], Duration::from_secs(30)) {
        let line = first_line(r.text());
        if let Some(version) = pick_version(&line) {
            part.items.push(
                Item::new("brew", "Homebrew", "homebrew", &version)
                    .kind("toolchain")
                    .at(format!("{prefix}/bin/brew"))
                    .about("macOS 的套件管理器（本次清單的來源）"),
            );
            part.headline = Some(version);
        }
    }

    if let Err(reason) = collect_from_json(&prefix, &mut part) {
        part.warn(format!("brew info 讀取失敗（{reason}），改用 brew list（沒有說明）"));
        collect_from_legacy(&prefix, &mut part);
    } else {
        reconcile_missing(&prefix, &mut part);
    }

    Ok(part)
}
