use crate::models::{Patch, Updates};
use crate::scan::{first_line, now_ms, pick_version, python, runtimes, upstream, version_key};
use crate::sh;
use serde::Deserialize;
use std::collections::BTreeMap;
use std::time::Duration;

#[derive(Deserialize)]
struct BrewOutdated {
    #[serde(default)]
    formulae: Vec<BrewEntry>,
    #[serde(default)]
    casks: Vec<BrewEntry>,
}

#[derive(Deserialize)]
struct BrewEntry {
    name: String,
    #[serde(default)]
    current_version: Option<String>,
}

#[derive(Deserialize)]
struct NpmRow {
    #[serde(default)]
    current: Option<String>,
    #[serde(default)]
    latest: Option<String>,
}

#[derive(Deserialize)]
struct PipOutdated {
    name: String,
    latest_version: String,
}

struct EcoCheck {
    eco: &'static str,
    patches: Vec<Patch>,
    ok: bool,
    warnings: Vec<String>,
}

impl EcoCheck {
    fn new(eco: &'static str) -> Self {
        EcoCheck {
            eco,
            patches: Vec::new(),
            ok: false,
            warnings: Vec::new(),
        }
    }
}

pub fn check() -> Updates {
    let results = std::thread::scope(|scope| {
        let handles = vec![
            scope.spawn(brew),
            scope.spawn(node),
            scope.spawn(python_check),
            scope.spawn(rustup),
            scope.spawn(java),
            scope.spawn(dotnet),
            scope.spawn(php),
            scope.spawn(ruby),
        ];
        handles
            .into_iter()
            .map(|h| h.join())
            .collect::<Vec<_>>()
    });

    let mut updates = Updates {
        checked_at_ms: now_ms(),
        checked_ecosystems: Vec::new(),
        patches: Vec::new(),
        warnings: Vec::new(),
    };

    for result in results {
        let Ok(check) = result else {
            continue;
        };
        updates.warnings.extend(check.warnings);
        if check.ok {
            updates.checked_ecosystems.push(check.eco.to_string());
            updates.patches.extend(check.patches);
        }
    }
    updates
}

fn brew() -> EcoCheck {
    let mut check = EcoCheck::new("brew");
    if !sh::exists("brew") {
        // Windows 沒有 Homebrew，這不算問題
        if cfg!(target_os = "macos") {
            check
                .warnings
                .push(String::from("找不到 brew，跳過 Homebrew 版本檢查"));
        }
        return check;
    }

    match sh::run("brew", &["outdated", "--json=v2"], Duration::from_secs(300)) {
        Ok(out) if !out.stdout.trim().is_empty() => {
            match serde_json::from_str::<BrewOutdated>(&out.stdout) {
                Ok(data) => {
                    for formula in data.formulae {
                        if let Some(latest) = formula.current_version {
                            check.patches.push(Patch {
                                key: format!("brew|Homebrew|{}", formula.name),
                                latest,
                            });
                        }
                    }
                    for cask in data.casks {
                        if let Some(latest) = cask.current_version {
                            check.patches.push(Patch {
                                key: format!("brew|Homebrew Cask|{}", cask.name),
                                latest,
                            });
                        }
                    }
                    check.ok = true;
                }
                Err(e) => check
                    .warnings
                    .push(format!("brew outdated 輸出無法解析：{e}")),
            }
        }
        Ok(out) => check
            .warnings
            .push(format!("brew outdated 失敗：{}", first_line(out.text()))),
        Err(e) => check
            .warnings
            .push(format!("brew outdated 失敗：{e}")),
    }
    check
}

fn node() -> EcoCheck {
    let mut check = EcoCheck::new("node");
    if !sh::exists("npm") {
        check.warnings.push(String::from("找不到 npm，跳過 npm 版本檢查"));
        return check;
    }

    match sh::run("npm", &["outdated", "-g", "--json"], Duration::from_secs(240)) {
        Ok(out) => {
            let text = out.stdout.trim();
            if text.is_empty() || text == "{}" {
                check.ok = true;
            } else {
                match serde_json::from_str::<BTreeMap<String, NpmRow>>(text) {
                    Ok(rows) => {
                        for (name, row) in rows {
                            if row.current.is_none() {
                                continue;
                            }
                            if let Some(latest) = row.latest {
                                check.patches.push(Patch {
                                    key: format!("node|npm 全域|{name}"),
                                    latest,
                                });
                            }
                        }
                        check.ok = true;
                    }
                    Err(e) => check
                        .warnings
                        .push(format!("npm outdated 輸出無法解析：{e}")),
                }
            }
        }
        Err(e) => check
            .warnings
            .push(format!("npm outdated 失敗：{e}")),
    }
    check
}

fn python_check() -> EcoCheck {
    let mut check = EcoCheck::new("python");
    let interps = python::discover();
    if interps.is_empty() {
        check
            .warnings
            .push(String::from("找不到 Python 直譯器，跳過 pip 版本檢查"));
        return check;
    }

    let mut patches: Vec<Patch> = Vec::new();
    let mut warnings: Vec<String> = Vec::new();
    let mut any_ok = false;

    std::thread::scope(|scope| {
        let handles: Vec<_> = interps
            .iter()
            .map(|interp| {
                let exe = interp.path.display().to_string();
                let source = interp.source.clone();
                scope.spawn(move || {
                    (
                        source,
                        sh::run(
                            &exe,
                            &[
                                "-m",
                                "pip",
                                "list",
                                "--outdated",
                                "--format=json",
                                "--disable-pip-version-check",
                            ],
                            Duration::from_secs(420),
                        ),
                    )
                })
            })
            .collect();

        for handle in handles {
            let Ok((source, result)) = handle.join() else {
                continue;
            };
            match result {
                Ok(out) if out.ok() => match serde_json::from_str::<Vec<PipOutdated>>(&out.stdout) {
                    Ok(rows) => {
                        for row in rows {
                            patches.push(Patch {
                                key: format!("python|{source}|{}", row.name),
                                latest: row.latest_version,
                            });
                        }
                        any_ok = true;
                    }
                    Err(e) => warnings.push(format!("{source} 的 pip outdated 輸出無法解析：{e}")),
                },
                Ok(out) => warnings.push(format!(
                    "{source} 的 pip 檢查失敗：{}",
                    first_line(out.text())
                )),
                Err(e) => warnings.push(format!("{source} 的 pip 檢查失敗：{e}")),
            }
        }
    });

    check.patches = patches;
    check.warnings = warnings;
    check.ok = any_ok;
    check
}

/// 解析 `rustup check` 的輸出。
/// 新舊格式都要吃（新版是小寫 `update available:`，舊版是大寫 `Update available :`）。
/// 回傳 (有沒有辨識出狀態, [(toolchain, 最新版本)])
fn parse_rustup_check(text: &str) -> (bool, Vec<(String, String)>) {
    let lowered = text.to_lowercase();
    let recognised = lowered.contains("up to date") || lowered.contains("update available");

    let mut updates = Vec::new();
    for line in text.lines() {
        let Some((toolchain, rest)) = line.split_once(" - ") else {
            continue;
        };
        if !rest.to_lowercase().contains("update available") {
            continue;
        }
        let Some((_, newer)) = rest.split_once("->") else {
            continue;
        };
        if let Some(latest) = pick_version(newer) {
            updates.push((toolchain.trim().to_string(), latest));
        }
    }

    (recognised, updates)
}

fn rustup() -> EcoCheck {
    let mut check = EcoCheck::new("rust");
    if !sh::exists("rustup") {
        check
            .warnings
            .push(String::from("找不到 rustup，跳過 Rust 版本檢查"));
        return check;
    }

    match sh::run("rustup", &["check"], Duration::from_secs(90)) {
        Ok(out) => {
            let text = format!("{}\n{}", out.stdout, out.stderr);
            let (recognised, updates) = parse_rustup_check(&text);
            if !recognised {
                check.warnings.push(format!(
                    "rustup check 沒有回報狀態：{}",
                    first_line(&text)
                ));
                return check;
            }
            for (toolchain, latest) in updates {
                check.patches.push(Patch {
                    key: format!("rust|rustup|{toolchain}"),
                    latest,
                });
            }
            check.ok = true;
        }
        Err(e) => check.warnings.push(format!("rustup check 失敗：{e}")),
    }
    check
}

/// 版本比較一律走 version_key（數字逐段比，字串不比）
fn is_newer(latest: &str, installed: &str) -> bool {
    !latest.is_empty() && version_key(latest) > version_key(installed)
}

/// Java：只比「同一個 feature 版本的最新修補版」，不會叫你跳大版。
/// 基準是 Adoptium Temurin（Oracle JDK 與它共用上游 OpenJDK 版號）。
/// Adoptium API 的 os / architecture 參數（依執行平台決定）
fn adoptium_platform() -> (&'static str, &'static str) {
    let os = if cfg!(target_os = "macos") {
        "mac"
    } else if cfg!(windows) {
        "windows"
    } else {
        "linux"
    };
    let arch = if cfg!(target_arch = "aarch64") {
        "aarch64"
    } else {
        "x64"
    };
    (os, arch)
}

fn java() -> EcoCheck {
    let mut check = EcoCheck::new("java");
    let Ok(part) = runtimes::java() else {
        check.warnings.push(String::from("找不到 Java，跳過版本檢查"));
        return check;
    };
    let Some(item) = part.items.iter().find(|item| item.kind == "runtime") else {
        check.warnings.push(String::from("Java 項目不存在，跳過版本檢查"));
        return check;
    };

    let installed = item.version.clone();
    let key = item.key.clone();
    let major = installed.split('.').next().unwrap_or_default().to_string();
    if major.is_empty() {
        check.warnings.push(String::from("Java 版本格式無法解析，跳過版本檢查"));
        return check;
    }

    let (os, arch) = adoptium_platform();
    let url = format!(
        "https://api.adoptium.net/v3/assets/latest/{major}/hotspot?os={os}&architecture={arch}&image_type=jdk"
    );
    match upstream::get_json(&url) {
        Ok(value) => {
            let latest = value
                .as_array()
                .and_then(|assets| assets.first())
                .and_then(|asset| asset.pointer("/version/openjdk_version"))
                .and_then(|raw| raw.as_str())
                .map(upstream::strip_build)
                .filter(|text| !text.is_empty());

            match latest {
                Some(latest) => {
                    if is_newer(&latest, &installed) {
                        check.patches.push(Patch { key, latest });
                    }
                    check.ok = true;
                }
                None => check.warnings.push(format!(
                    "Adoptium 沒有 Java {major} 的 {os}/{arch} 建置，無法比對"
                )),
            }
        }
        Err(e) => check
            .warnings
            .push(format!("Java 版本查詢失敗（Adoptium）：{e}")),
    }
    check
}

/// .NET：用官方 releases-index 找同一個 channel，比 SDK 與執行階段。
fn dotnet() -> EcoCheck {
    let mut check = EcoCheck::new("dotnet");
    let Ok(part) = runtimes::dotnet() else {
        check.warnings.push(String::from("找不到 .NET，跳過版本檢查"));
        return check;
    };
    let Some(sdk) = part.items.iter().find(|item| item.kind == "sdk") else {
        check.warnings.push(String::from("找不到 .NET SDK，跳過版本檢查"));
        return check;
    };

    let mut pieces = sdk.version.split('.');
    let channel = format!(
        "{}.{}",
        pieces.next().unwrap_or_default(),
        pieces.next().unwrap_or_default()
    );
    let sdk_key = sdk.key.clone();
    let sdk_version = sdk.version.clone();

    match upstream::get_json(
        "https://dotnetcli.blob.core.windows.net/dotnet/release-metadata/releases-index.json",
    ) {
        Ok(value) => {
            let entry = value
                .get("releases-index")
                .and_then(|list| list.as_array())
                .and_then(|list| {
                    list.iter()
                        .find(|row| row.get("channel-version").and_then(|v| v.as_str()) == Some(&channel))
                });

            let Some(entry) = entry else {
                check
                    .warnings
                    .push(format!(".NET releases-index 找不到 {channel} channel"));
                return check;
            };

            if let Some(latest_sdk) = entry.get("latest-sdk").and_then(|v| v.as_str()) {
                if is_newer(latest_sdk, &sdk_version) {
                    check.patches.push(Patch {
                        key: sdk_key,
                        latest: latest_sdk.to_string(),
                    });
                }
            }

            if let Some(latest_runtime) = entry.get("latest-runtime").and_then(|v| v.as_str()) {
                for item in part.items.iter().filter(|item| item.kind == "runtime") {
                    if is_newer(latest_runtime, &item.version) {
                        check.patches.push(Patch {
                            key: item.key.clone(),
                            latest: latest_runtime.to_string(),
                        });
                    }
                }
            }

            if entry.get("support-phase").and_then(|v| v.as_str()) == Some("eol") {
                let channels = value
                    .get("releases-index")
                    .and_then(|list| list.as_array())
                    .cloned()
                    .unwrap_or_default();
                // support-phase 只有 active/eol；LTS 與否要看 release-type
                let newest_active = |lts_only: bool| {
                    channels
                        .iter()
                        .filter(|row| {
                            row.get("support-phase").and_then(|v| v.as_str()) != Some("eol")
                        })
                        .filter(|row| {
                            !lts_only
                                || row.get("release-type").and_then(|v| v.as_str()) == Some("lts")
                        })
                        .filter_map(|row| row.get("channel-version").and_then(|v| v.as_str()))
                        .max_by(|a, b| version_key(a).cmp(&version_key(b)))
                        .map(str::to_string)
                };
                let target = newest_active(true)
                    .map(|version| format!("{version}（LTS）"))
                    .or_else(|| newest_active(false));
                check.warnings.push(match target {
                    Some(target) => {
                        format!(".NET {channel} 已停止支援（EOL），建議移到 {target}")
                    }
                    None => format!(".NET {channel} 已停止支援（EOL）"),
                });
            }
            check.ok = true;
        }
        Err(e) => check
            .warnings
            .push(format!(".NET 版本查詢失敗（releases-index）：{e}")),
    }
    check
}

/// PHP：本機這個 PHP 是 Homebrew 裝的，所以直接比對 Homebrew 官方 formula 的 stable 版。
fn php() -> EcoCheck {
    let mut check = EcoCheck::new("php");
    let Ok(part) = runtimes::php() else {
        check.warnings.push(String::from("找不到 PHP，跳過版本檢查"));
        return check;
    };
    let Some(item) = part.items.first() else {
        check.warnings.push(String::from("PHP 項目不存在，跳過版本檢查"));
        return check;
    };

    let installed = item.version.clone();
    let key = item.key.clone();

    match upstream::get_json("https://formulae.brew.sh/api/formula/php.json") {
        Ok(value) => {
            let latest = value
                .pointer("/versions/stable")
                .and_then(|v| v.as_str())
                .map(str::to_string);

            match latest {
                Some(latest) => {
                    if is_newer(&latest, &installed) {
                        check.patches.push(Patch { key, latest });
                    }
                    check.ok = true;
                }
                None => check
                    .warnings
                    .push(String::from("Homebrew formula API 沒有回報 PHP 版本")),
            }
        }
        Err(e) => check
            .warnings
            .push(format!("PHP 版本查詢失敗（Homebrew API）：{e}")),
    }
    check
}

/// Ruby：官方 repo 的最新 release tag（macOS 內建的 Ruby 版本很舊，這裡只是告知差距）。
fn ruby() -> EcoCheck {
    let mut check = EcoCheck::new("ruby");
    let Ok(part) = runtimes::ruby() else {
        check.warnings.push(String::from("找不到 Ruby，跳過版本檢查"));
        return check;
    };
    let Some(item) = part.items.iter().find(|item| item.kind == "runtime") else {
        check.warnings.push(String::from("Ruby 項目不存在，跳過版本檢查"));
        return check;
    };

    let installed = item.version.clone();
    let key = item.key.clone();

    match upstream::get_json("https://api.github.com/repos/ruby/ruby/releases/latest") {
        Ok(value) => {
            let latest = value
                .get("tag_name")
                .and_then(|v| v.as_str())
                .map(|tag| tag.trim_start_matches('v').replace('_', "."))
                .filter(|text| !text.is_empty());

            match latest {
                Some(latest) => {
                    if is_newer(&latest, &installed) {
                        check.patches.push(Patch { key, latest });
                    }
                    check.ok = true;
                }
                None => check
                    .warnings
                    .push(String::from("GitHub release 沒有 tag_name")),
            }
        }
        Err(e) => check
            .warnings
            .push(format!("Ruby 版本查詢失敗（GitHub API）：{e}")),
    }
    check
}

#[cfg(test)]
mod tests {
    use super::adoptium_platform;

    #[test]
    fn adoptium_platform_follows_the_host() {
        let (os, arch) = adoptium_platform();
        let expected_os = if cfg!(target_os = "macos") {
            "mac"
        } else if cfg!(windows) {
            "windows"
        } else {
            "linux"
        };
        let expected_arch = if cfg!(target_arch = "aarch64") {
            "aarch64"
        } else {
            "x64"
        };
        assert_eq!(os, expected_os);
        assert_eq!(arch, expected_arch);
    }

    use super::parse_rustup_check;

    #[test]
    fn parses_current_lowercase_rustup_output() {
        // 2026-11 實際的 rustup 1.29.x 輸出（注意是小寫、且含 rustup 自己那一行）
        let text = "stable-aarch64-apple-darwin - update available: 1.99.1 (58cf8f9c3 2026-10-13) -> 1.100.0 (0ef1bc386 2026-11-09)\n\
                    rustup - update available : 1.29.2 -> 1.29.3\n";
        let (recognised, updates) = parse_rustup_check(text);
        assert!(recognised);
        assert_eq!(updates.len(), 2);
        assert_eq!(updates[0].0, "stable-aarch64-apple-darwin");
        assert_eq!(updates[0].1, "1.100.0");
        assert_eq!(updates[1].0, "rustup");
        assert_eq!(updates[1].1, "1.29.3");
    }

    #[test]
    fn parses_legacy_capitalised_output() {
        let text = "stable-aarch64-apple-darwin - Up to date : 1.99.1 (58cf8f9c3 2026-10-13)\n";
        let (recognised, updates) = parse_rustup_check(text);
        assert!(recognised);
        assert!(updates.is_empty());
    }

    #[test]
    fn rejects_unrelated_output() {
        let text = "error: could not create temp file /Users/x/.rustup/tmp/abc_file: Operation not permitted";
        let (recognised, updates) = parse_rustup_check(text);
        assert!(!recognised);
        assert!(updates.is_empty());
    }
}
