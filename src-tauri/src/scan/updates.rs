use crate::models::{Patch, Updates};
use crate::scan::{first_line, now_ms, pick_version, python};
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
        check.warnings.push(String::from("找不到 brew，跳過 Homebrew 版本檢查"));
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
            if !text.contains("Up to date") && !text.contains("Update available") {
                check
                    .warnings
                    .push(format!("rustup check 沒有回報狀態：{}", first_line(&text)));
                return check;
            }
            for line in text.lines() {
                let Some((toolchain, rest)) = line.split_once(" - ") else {
                    continue;
                };
                if !rest.contains("Update available") {
                    continue;
                }
                let Some((_, newer)) = rest.split_once("->") else {
                    continue;
                };
                if let Some(latest) = pick_version(newer) {
                    check.patches.push(Patch {
                        key: format!("rust|rustup|{}", toolchain.trim()),
                        latest,
                    });
                }
            }
            check.ok = true;
        }
        Err(e) => check.warnings.push(format!("rustup check 失敗：{e}")),
    }
    check
}
