use crate::models::Item;
use crate::scan::{first_line, pick_version, Part};
use crate::sh;
use std::time::Duration;

/// `opt/<name>` 是指向「目前生效的那個 keg」的 symlink，
/// 比 `brew list --versions` 的排序可靠（有多版本並存時尤其重要）。
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

pub fn scan() -> Result<Part, String> {
    if !sh::exists("brew") {
        return Err(String::from("找不到 brew（Homebrew 未安裝或不在可搜尋的路徑）"));
    }

    let mut part = Part::new();
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
                    .note("套件管理器本體"),
            );
        }
    }

    let formulae = sh::run(
        "brew",
        &["list", "--versions", "--formula"],
        Duration::from_secs(90),
    );
    match formulae {
        Ok(r) if r.ok() => {
            for line in r.stdout.lines() {
                let mut tokens = line.split_whitespace();
                let Some(name) = tokens.next() else { continue };
                let versions: Vec<&str> = tokens.collect();
                let version = linked_version(&prefix, name)
                    .or_else(|| versions.first().copied().map(str::to_string))
                    .unwrap_or_else(|| String::from("—"));
                let mut item = Item::new("brew", "Homebrew", name, &version)
                    .kind("formula")
                    .at(format!("{prefix}/opt/{name}"))
                    .cmd(format!("brew upgrade {name}"));
                if versions.len() > 1 {
                    let others: Vec<&str> = versions
                        .iter()
                        .copied()
                        .filter(|candidate| *candidate != version.as_str())
                        .collect();

                    if !others.is_empty() {
                        item = item.note(format!("另有 {} 並存", others.join("、")));
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
                        .cmd(format!("brew upgrade --cask {name}")),
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

    Ok(part)
}
