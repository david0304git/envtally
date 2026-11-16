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
    #[serde(default)]
    summary: String,
}

/// 直接讀已安裝套件的 metadata（Summary 欄位），比 `pip list` 快，
/// 而且順便拿到「這個套件在做什麼」。
const METADATA_SCRIPT: &str = r#"
import json, importlib.metadata as md
seen = {}
for d in md.distributions():
    try:
        name = d.metadata["Name"] or ""
        if not name:
            continue
        key = name.lower()
        if key in seen:
            continue
        seen[key] = {
            "name": name,
            "version": d.version,
            "summary": (d.metadata["Summary"] or "").strip(),
        }
    except Exception:
        continue
print(json.dumps(list(seen.values())))
"#;

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

    for dir in sh::search_dirs() {
        let Ok(entries) = std::fs::read_dir(dir) else {
            continue;
        };
        let mut found: Vec<PathBuf> = entries
            .flatten()
            .filter(|e| is_python_bin(&e.file_name().to_string_lossy()) && e.path().is_file())
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

/// 先試 metadata（含說明），失敗再退回 pip list（沒有說明）。
fn packages(exe: &str) -> Result<Vec<PipPackage>, String> {
    let fast = sh::run(
        exe,
        &["-W", "ignore", "-c", METADATA_SCRIPT],
        Duration::from_secs(120),
    );
    if let Ok(out) = fast {
        if out.ok() {
            if let Ok(list) = serde_json::from_str::<Vec<PipPackage>>(&out.stdout) {
                return Ok(list);
            }
        }
    }

    let out = sh::run(
        exe,
        &["-m", "pip", "list", "--format=json", "--disable-pip-version-check"],
        Duration::from_secs(180),
    )?;
    if !out.ok() {
        return Err(first_line(out.text()));
    }
    serde_json::from_str::<Vec<PipPackage>>(&out.stdout)
        .map_err(|e| format!("pip 輸出無法解析：{e}"))
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
                .about("Python 直譯器本體；下面列出它這個環境裝了哪些套件")
                .note(format!("prefix：{}", interp.prefix)),
        );

        let exe = interp.path.display().to_string();
        let label = interp
            .path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| String::from("python3"));

        match packages(&exe) {
            Ok(packages) => {
                for pkg in packages {
                    let mut item = Item::new("python", &interp.source, &pkg.name, &pkg.version)
                        .kind("package")
                        .cmd(format!("{label} -m pip install -U {}", pkg.name));
                    item = item.about(pkg.summary);
                    part.items.push(item);
                }
            }
            Err(e) => part.warn(format!("{} 讀不到套件清單：{e}", interp.source)),
        }
    }

    Ok(part)
}
