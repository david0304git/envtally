use crate::models::Item;
use crate::scan::{first_line, Part};
use crate::sh;
use serde_json::Value;
use std::time::Duration;

pub fn scan() -> Result<Part, String> {
    let mut part = Part::new();
    let mut found_anything = false;

    let nvm_root = format!(
        "{}/.nvm/versions/node",
        std::env::var("HOME").unwrap_or_else(|_| String::from("/"))
    );
    let active_node = sh::which("node")
        .and_then(|p| std::fs::canonicalize(p).ok())
        .map(|p| p.display().to_string());

    let mut versions: Vec<(Vec<u32>, String, std::path::PathBuf)> = Vec::new();
    if let Ok(entries) = std::fs::read_dir(&nvm_root) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            let nums: Vec<u32> = name
                .trim_start_matches('v')
                .split('.')
                .filter_map(|p| p.parse().ok())
                .collect();
            if nums.is_empty() {
                continue;
            }
            versions.push((nums, name, entry.path()));
        }
    }
    versions.sort_by(|a, b| b.0.cmp(&a.0));

    for (_, dir_name, path) in versions {
        found_anything = true;
        let version = dir_name.trim_start_matches('v').to_string();
        let bin = path.join("bin").join("node").display().to_string();
        let is_active = active_node
            .as_ref()
            .map(|active| active == &bin || active.starts_with(&path.display().to_string()))
            .unwrap_or(false);
        let mut item = Item::new("node", "nvm", "Node.js", &version)
            .kind("runtime")
            .at(path.display().to_string())
            .cmd(format!("nvm use {dir_name}"));
        if is_active {
            item = item.note("目前 PATH 使用中");
        }
        part.items.push(item);
    }

    if !found_anything {
        if let Ok(out) = sh::run("node", &["-v"], Duration::from_secs(10)) {
            if let Some(version) = crate::scan::pick_version(out.text()) {
                part.items.push(
                    Item::new("node", "PATH", "Node.js", &version)
                        .kind("runtime")
                        .at(sh::which("node").map(|p| p.display().to_string()).unwrap_or_default()),
                );
                found_anything = true;
            }
        }
    }

    if let Ok(out) = sh::run("npm", &["-v"], Duration::from_secs(20)) {
        if let Some(version) = crate::scan::pick_version(out.text()) {
            part.items.push(
                Item::new("node", "npm 全域", "npm", &version)
                    .kind("toolchain")
                    .at(sh::which("npm").map(|p| p.display().to_string()).unwrap_or_default())
                    .note("Node 的套件管理器本體"),
            );
        }
    }

    match sh::run(
        "npm",
        &["ls", "-g", "--depth=0", "--json"],
        Duration::from_secs(60),
    ) {
        Ok(out) => {
            let json: Option<Value> = serde_json::from_str(&out.stdout).ok();
            match json.as_ref().and_then(|v| v.get("dependencies")) {
                Some(Value::Object(deps)) => {
                    for (name, info) in deps {
                        let version = info
                            .get("version")
                            .and_then(Value::as_str)
                            .unwrap_or("—")
                            .to_string();
                        if name == "npm" {
                            continue;
                        }
                        part.items.push(
                            Item::new("node", "npm 全域", name, &version)
                                .kind("package")
                                .cmd(format!("npm install -g {name}@latest")),
                        );
                    }
                }
                _ => {
                    let msg = first_line(out.text());
                    if !out.ok() && !msg.is_empty() {
                        part.warn(format!("全域 npm 套件讀取失敗：{msg}"));
                    }
                }
            }
        }
        Err(e) => part.warn(format!("全域 npm 套件讀取失敗：{e}")),
    }

    if !found_anything && part.items.is_empty() {
        return Err(String::from("找不到 Node.js（沒有 nvm 目錄也沒有 node）"));
    }
    Ok(part)
}
