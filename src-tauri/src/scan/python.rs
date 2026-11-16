use crate::models::Item;
use crate::scan::{first_line, Part};
use crate::sh;
use serde::Deserialize;
use std::path::PathBuf;
use std::time::Duration;

#[derive(Deserialize)]
struct PipPackage {
    name: String,
    version: String,
}

/// 一個可用的 Python 直譯器。
pub struct Interp {
    pub path: PathBuf,
    pub version: String,
    pub prefix: String,
    /// 顯示給使用者看、同時當作 Item 的 source（必須穩定，版本檢查要靠它對回同一批項目）
    pub source: String,
}

fn is_python_bin(name: &str) -> bool {
    if name == "python3" || name == "python" {
        return true;
    }
    match name.strip_prefix("python3.") {
        Some(rest) => !rest.is_empty() && rest.chars().all(|c| c.is_ascii_digit()),
        None => false,
    }
}

fn probe(path: &PathBuf) -> Option<Interp> {
    let exe = path.to_str()?;
    let out = sh::run(
        exe,
        &[
            "-c",
            "import sys;print(sys.version.split()[0]);print(sys.prefix)",
        ],
        Duration::from_secs(20),
    )
    .ok()?;
    if !out.ok() {
        return None;
    }
    let mut lines = out.stdout.lines().map(str::trim).filter(|l| !l.is_empty());
    let version = lines.next()?.to_string();
    let prefix = lines.next().unwrap_or("").to_string();
    if !version.chars().next().map(|c| c.is_ascii_digit()).unwrap_or(false) {
        return None;
    }
    let path_str = path.display().to_string();
    let source = if path_str.starts_with("/usr/bin/") {
        format!("Python {version}（macOS 內建）")
    } else if path_str.starts_with("/opt/homebrew/") {
        format!("Python {version}（Homebrew）")
    } else {
        format!("Python {version}")
    };
    Some(Interp {
        path: path.clone(),
        version,
        prefix,
        source,
    })
}

/// 找出所有 Python 直譯器（去重、最多 4 個）。
pub fn discover() -> Vec<Interp> {
    let mut candidates: Vec<PathBuf> = Vec::new();

    // 先看 PATH 與常見 bin 目錄裡的 python3 / python3.NN
    for dir in sh::search_dirs() {
        let Ok(entries) = std::fs::read_dir(dir) else {
            continue;
        };
        let mut found: Vec<PathBuf> = entries
            .flatten()
            .filter(|e| {
                is_python_bin(&e.file_name().to_string_lossy())
                    && e.path().is_file()
            })
            .map(|e| e.path())
            .collect();
        found.sort();
        candidates.extend(found);
    }

    // python.org 安裝器：多版本都掛在 framework 底下
    let framework = "/Library/Frameworks/Python.framework/Versions";
    if let Ok(entries) = std::fs::read_dir(framework) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if name == "Current" {
                continue;
            }
            let bin = entry.path().join("bin").join("python3");
            if bin.is_file() {
                candidates.push(bin);
            }
        }
    }

    let mut seen: Vec<PathBuf> = Vec::new();
    let mut out: Vec<Interp> = Vec::new();
    for path in candidates {
        if out.len() >= 4 {
            break;
        }
        let canon = std::fs::canonicalize(&path).unwrap_or_else(|_| path.clone());
        if seen.contains(&canon) {
            continue;
        }
        seen.push(canon);
        if let Some(interp) = probe(&path) {
            out.push(interp);
        }
    }
    out
}

pub fn scan() -> Result<Part, String> {
    let interps = discover();
    if interps.is_empty() {
        return Err(String::from("找不到任何 Python 直譯器"));
    }

    let mut part = Part::new();
    for interp in interps {
        part.items.push(
            Item::new("python", &interp.source, "Python", &interp.version)
                .kind("runtime")
                .at(interp.path.display().to_string())
                .note(format!("prefix：{}", interp.prefix)),
        );

        let exe = interp.path.display().to_string();
        let label = interp
            .path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| String::from("python3"));

        match sh::run(
            &exe,
            &["-m", "pip", "list", "--format=json", "--disable-pip-version-check"],
            Duration::from_secs(120),
        ) {
            Ok(out) if out.ok() => match serde_json::from_str::<Vec<PipPackage>>(&out.stdout) {
                Ok(packages) => {
                    for pkg in packages {
                        part.items.push(
                            Item::new("python", &interp.source, &pkg.name, &pkg.version)
                                .kind("package")
                                .cmd(format!("{label} -m pip install -U {}", pkg.name)),
                        );
                    }
                }
                Err(e) => part.warn(format!("{} 的 pip 輸出無法解析：{e}", interp.source)),
            },
            Ok(out) => part.warn(format!(
                "{} 讀不到 pip 清單：{}",
                interp.source,
                first_line(out.text())
            )),
            Err(e) => part.warn(format!("{} 讀不到 pip 清單：{e}", interp.source)),
        }
    }

    Ok(part)
}
