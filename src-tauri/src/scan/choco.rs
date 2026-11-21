//! Chocolatey：Windows 的套件管理器（概念等同 macOS 的 Homebrew）。
//!
//! `choco list --limit-output` 每行輸出 `名稱|版本`，這裡逐行解析。
//! 舊版 Chocolatey 需要 `--local-only`，所以第一次失敗會退回舊參數。

use crate::models::Item;
use crate::scan::{pick_version, Part};
use crate::sh;
use std::time::Duration;

/// 解析 `choco list --limit-output` 的輸出
fn parse_list(text: &str) -> Vec<(String, String)> {
    let mut out: Vec<(String, String)> = Vec::new();
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let Some((name, version)) = line.split_once('|') else {
            continue;
        };
        let name = name.trim();
        let version = version.trim();
        if name.is_empty() || version.is_empty() {
            continue;
        }
        // 結尾的統計行（例如 "12 packages installed."）不會有 '|'，這行只是保險
        if name.chars().all(|c| c.is_ascii_digit()) {
            continue;
        }
        out.push((name.to_string(), version.to_string()));
    }
    out
}

pub fn scan() -> Result<Part, String> {
    if !sh::exists("choco") {
        return Ok(Part::absent());
    }

    let mut part = Part::new();

    // choco 本體（像 Homebrew 分類會列 brew 自己）
    if let Ok(out) = sh::run("choco", &["--version"], Duration::from_secs(30)) {
        let version = pick_version(out.text()).unwrap_or_else(|| String::from("未知"));
        let mut item = Item::new("choco", "Chocolatey", "choco", &version)
            .kind("toolchain")
            .about("Chocolatey 套件管理器本體：install / upgrade / list")
            .cmd("choco upgrade chocolatey -y");
        if let Some(path) = sh::which("choco") {
            item = item.at(path.display().to_string());
        }
        part.items.push(item);
        part.headline = Some(version);
    }

    let listed = sh::run(
        "choco",
        &["list", "--limit-output"],
        Duration::from_secs(180),
    )
    .or_else(|_| {
        sh::run(
            "choco",
            &["list", "--local-only", "--limit-output"],
            Duration::from_secs(180),
        )
    });

    match listed {
        Ok(out) => {
            for (name, version) in parse_list(&out.stdout) {
                if name.eq_ignore_ascii_case("chocolatey") {
                    continue;
                }
                part.items.push(
                    Item::new("choco", "Chocolatey", &name, &version)
                        .kind("package")
                        .cmd(format!("choco upgrade {name} -y"))
                        .rm(format!("choco uninstall {name} -y")),
                );
            }
        }
        Err(error) => part.warn(format!("choco list 失敗：{error}")),
    }

    if part.items.is_empty() {
        return Ok(Part::absent());
    }
    if part.headline.is_none() {
        part.headline = Some(format!("{} 個套件", part.items.len()));
    }
    Ok(part)
}

#[cfg(test)]
mod tests {
    use super::parse_list;

    #[test]
    fn parses_limit_output() {
        let text = "chocolatey|2.5.1\ngit|2.47.1\npython314|3.14.0\n";
        let parsed = parse_list(text);
        assert_eq!(parsed.len(), 3);
        assert_eq!(parsed[0], (String::from("chocolatey"), String::from("2.5.1")));
        assert_eq!(parsed[2], (String::from("python314"), String::from("3.14.0")));
    }

    #[test]
    fn ignores_noise_lines() {
        let text = "Chocolatey v2.5.1\n\ngit|2.47.1\n2 packages installed.\n";
        let parsed = parse_list(text);
        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed[0].0, "git");
    }
}
