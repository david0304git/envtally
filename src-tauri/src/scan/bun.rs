//! Bun：執行環境版本 + 全域安裝的套件（直接讀 Bun 自己的 global node_modules，
//! 不呼叫 `bun pm ls`，避免指令格式變動）。

use crate::models::Item;
use crate::scan::{origin_for, Part};
use crate::sh;
use serde_json::Value;
use std::path::{Path, PathBuf};
use std::time::Duration;

fn bun_global_root() -> PathBuf {
    PathBuf::from(crate::sh::home()).join(".bun/install/global/node_modules")
}

fn read_package(root: &Path, dir: &str) -> Option<(String, String, Option<String>)> {
    let text = std::fs::read_to_string(root.join(dir).join("package.json")).ok()?;
    let value: Value = serde_json::from_str(&text).ok()?;
    let name = value.get("name").and_then(Value::as_str)?.to_string();
    let version = value
        .get("version")
        .and_then(Value::as_str)
        .unwrap_or("—")
        .to_string();
    let description = value
        .get("description")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|text| !text.is_empty())
        .map(str::to_string);
    Some((name, version, description))
}

pub fn scan() -> Result<Part, String> {
    if !sh::exists("bun") {
        return Ok(Part::absent());
    }

    let mut part = Part::new();
    let out = sh::run("bun", &["--version"], Duration::from_secs(20))?;
    let trimmed = out.stdout.trim();
    let version = if trimmed.is_empty() {
        String::from("未知")
    } else {
        trimmed.to_string()
    };
    let path = sh::which("bun")
        .map(|p| p.display().to_string())
        .unwrap_or_default();

    part.headline = Some(version.clone());
    part.items.push(
        Item::new("bun", origin_for(&path), "Bun", &version)
            .kind("runtime")
            .at(path)
            .about("Bun 執行環境（JavaScript / TypeScript，內建套件管理器與打包器）")
            .cmd("bun upgrade"),
    );

    let root = bun_global_root();
    if let Ok(entries) = std::fs::read_dir(&root) {
        let mut dirs: Vec<String> = entries
            .flatten()
            .filter(|entry| entry.path().is_dir())
            .map(|entry| entry.file_name().to_string_lossy().to_string())
            .collect();
        dirs.sort();

        for dir in dirs {
            // 略過 scoped 套件的容器目錄（@scope 本身不是套件）
            if dir.starts_with('@') {
                if let Ok(scoped) = std::fs::read_dir(root.join(&dir)) {
                    let mut inner: Vec<String> = scoped
                        .flatten()
                        .filter(|entry| entry.path().is_dir())
                        .map(|entry| format!("{dir}/{}", entry.file_name().to_string_lossy()))
                        .collect();
                    inner.sort();
                    for name in inner {
                        push_package(&mut part, &root, &name);
                    }
                }
                continue;
            }
            push_package(&mut part, &root, &dir);
        }
    }

    Ok(part)
}

fn push_package(part: &mut Part, root: &Path, dir: &str) {
    let Some((name, version, description)) = read_package(root, dir) else {
        return;
    };
    let mut item = Item::new("bun", "bun 全域", &name, &version)
        .kind("package")
        .cmd(format!("bun remove -g {name}"));
    if let Some(description) = description {
        item = item.about(description);
    }
    part.items.push(item);
}
