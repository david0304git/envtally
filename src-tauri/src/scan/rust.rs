use crate::models::Item;
use crate::scan::{first_line, pick_version, version_key, Part};
use crate::sh;
use std::time::Duration;

fn rustup_home() -> String {
    std::env::var("RUSTUP_HOME").unwrap_or_else(|_| {
        format!(
            "{}/.rustup",
            std::env::var("HOME").unwrap_or_else(|_| String::from("/"))
        )
    })
}

pub fn scan() -> Result<Part, String> {
    let mut part = Part::new();
    let mut found = false;

    let active = sh::run(
        "rustup",
        &["show", "active-toolchain"],
        Duration::from_secs(30),
    )
    .ok()
    .map(|r| r.text().to_string())
    .unwrap_or_default();

    let mut ordered: Vec<(bool, String)> = Vec::new();
    let toolchains_dir = format!("{}/toolchains", rustup_home());
    if let Ok(entries) = std::fs::read_dir(&toolchains_dir) {
        let mut dirs: Vec<_> = entries.flatten().collect();
        dirs.sort_by_key(|e| e.file_name());
        for entry in dirs {
            let name = entry.file_name().to_string_lossy().to_string();
            let rustc = entry.path().join("bin").join("rustc");
            let version = if rustc.is_file() {
                sh::run(
                    &rustc.display().to_string(),
                    &["--version"],
                    Duration::from_secs(20),
                )
                .ok()
                .and_then(|r| pick_version(r.text()))
                .unwrap_or_else(|| String::from("未知"))
            } else {
                String::from("未知")
            };
            let is_default = active.starts_with(&name);
            ordered.push((is_default, version.clone()));

            let mut item = Item::new("rust", "rustup", &name, &version)
                .kind("toolchain")
                .at(entry.path().display().to_string())
                .about("Rust 編譯器與工具鏈（rustc / cargo / rustup），一個 toolchain 就是一套完整版本")
                .cmd(format!("rustup update {name}"))
                .rm(format!("rustup toolchain uninstall {name}"));
            if is_default {
                item = item.note_i18n("預設 toolchain", "note.rust.default", &[]);
            }
            part.items.push(item);
            found = true;
        }
    }

    if !found {
        if let Ok(out) = sh::run("rustc", &["--version"], Duration::from_secs(20)) {
            if let Some(version) = pick_version(out.text()) {
                part.items.push(
                    Item::new("rust", "PATH", "Rust", &version)
                        .kind("toolchain")
                        .about("Rust 編譯器（rustc）")
                        .at(sh::which("rustc").map(|p| p.display().to_string()).unwrap_or_default()),
                );
                found = true;
            }
        }
    }

    // rustup 本體（rustup check 也會回報它自己的更新，沒有這一項就對不到 patch）
    if let Ok(out) = sh::run("rustup", &["--version"], Duration::from_secs(20)) {
        if let Some(version) = pick_version(out.text()) {
            part.items.push(
                Item::new("rust", "rustup", "rustup", &version)
                    .kind("toolchain")
                    .at(sh::which("rustup")
                        .map(|p| p.display().to_string())
                        .unwrap_or_default())
                    .about("Rust 工具鏈管理器本體：安裝/切換 toolchain、更新自己")
                    .cmd("rustup self update")
                    .rm("rustup self uninstall"),
            );
        }
    }

    ordered.sort_by(|a, b| {
        b.0.cmp(&a.0)
            .then_with(|| version_key(&b.1).cmp(&version_key(&a.1)))
    });
    if !ordered.is_empty() {
        part.headline = Some(
            ordered
                .iter()
                .map(|(_, version)| version.clone())
                .collect::<Vec<_>>()
                .join(" · "),
        );
    }

    // cargo install 裝的執行檔
    match sh::run("cargo", &["install", "--list"], Duration::from_secs(60)) {
        Ok(out) if out.ok() => {
            for line in out.stdout.lines() {
                let trimmed = line.trim();
                if trimmed.is_empty() || line.starts_with(' ') || !trimmed.contains(" v") {
                    continue;
                }
                let Some((name, rest)) = trimmed.split_once(" v") else {
                    continue;
                };
                let version = rest.trim_end_matches(':').trim();
                part.items.push(
                    Item::new("rust", "cargo install", name, version)
                        .kind("binary")
                        .cmd(format!("cargo install {name} --force"))
                        .rm(format!("cargo uninstall {name}")),
                );
            }
        }
        Ok(out) => {
            let msg = first_line(out.text());
            if !msg.is_empty() {
                part.warn(format!("cargo install 清單讀取失敗：{msg}"));
            }
        }
        Err(e) => part.warn(format!("cargo install 清單讀取失敗：{e}")),
    }

    if !found && part.items.is_empty() {
        return Ok(Part::absent());
    }
    Ok(part)
}
