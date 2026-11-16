//! 語言執行環境：Java / .NET / PHP / Ruby 各自是一個分類，
//! 這樣在「全部」檢視收合時，每個環境都跟 Python 一樣有自己的版本列。

use crate::models::Item;
use crate::scan::{first_line, origin_for, pick_version, Part};
use crate::sh;
use std::collections::HashMap;
use std::time::Duration;

pub fn java() -> Result<Part, String> {
    let mut part = Part::new();
    let mut found = false;

    // /usr/libexec/java_home -V 把清單印在 stderr
    if let Ok(out) = sh::run("/usr/libexec/java_home", &["-V"], Duration::from_secs(30)) {
        let text = format!("{}\n{}", out.stdout, out.stderr);
        for line in text.lines().map(str::trim) {
            // 25.0.3 (arm64) "Oracle Corporation" - "Java SE 25.0.3" /Library/Java/...
            let Some(first_space) = line.find(' ') else {
                continue;
            };
            let version = &line[..first_space];
            if !version
                .chars()
                .next()
                .map(|c| c.is_ascii_digit())
                .unwrap_or(false)
            {
                continue;
            }
            if version.split('.').count() < 2 {
                continue;
            }
            let arch = line
                .split_once('(')
                .and_then(|(_, rest)| rest.split_once(')'))
                .map(|(arch, _)| arch.trim().to_string());
            let vendor = line
                .split_once('"')
                .and_then(|(_, rest)| rest.split_once('"'))
                .map(|(vendor, _)| vendor.trim().to_string());
            let home = line
                .rsplit('"')
                .next()
                .map(|s| s.trim().to_string())
                .filter(|s| s.starts_with('/'));

            if part.headline.is_none() {
                part.headline = Some(version.to_string());
            }

            let mut item = Item::new(
                "java",
                vendor.as_deref().unwrap_or("Java"),
                "Java",
                version,
            )
            .kind("runtime")
            .about("Java 執行環境（JDK）：跑 Java 程式、也提供編譯工具")
            .note_i18n("由 JDK 安裝器管理，升級需重新安裝", "note.java.managed", &[]);
            if let Some(home) = home {
                item = item.at(home);
            }
            if let Some(arch) = arch {
                item = item.note_i18n(&format!("{arch} 架構"), "note.java.arch", &[&arch]);
            }
            part.items.push(item);
            found = true;
        }
    }

    if !found {
        match sh::run("java", &["-version"], Duration::from_secs(20)) {
            Ok(out) => match pick_version(out.text()) {
                Some(version) => {
                    part.headline = Some(version.clone());
                    part.items.push(
                        Item::new("java", "Java", "Java", &version)
                            .kind("runtime")
                            .about("Java 執行環境（JDK）"),
                    );
                }
                None => part.warn(format!("java -version 輸出無法解析：{}", first_line(out.text()))),
            },
            Err(e) => part.warn(format!("Java 讀取失敗：{e}")),
        }
    }

    if part.items.is_empty() {
        return Ok(Part::absent());
    }
    Ok(part)
}

pub fn dotnet() -> Result<Part, String> {
    if !sh::exists("dotnet") {
        return Ok(Part::absent());
    }

    let mut part = Part::new();

    if let Ok(out) = sh::run("dotnet", &["--list-sdks"], Duration::from_secs(60)) {
        for line in out.stdout.lines() {
            // 9.0.304 [/usr/local/share/dotnet/sdk]
            let mut tokens = line.splitn(2, ' ');
            let Some(version) = tokens.next() else { continue };
            if !version
                .chars()
                .next()
                .map(|c| c.is_ascii_digit())
                .unwrap_or(false)
            {
                continue;
            }
            let path = tokens
                .next()
                .unwrap_or("")
                .trim()
                .trim_start_matches('[')
                .trim_end_matches(']')
                .to_string();
            if part.headline.is_none() {
                part.headline = Some(version.to_string());
            }
            part.items.push(
                Item::new("dotnet", ".NET SDK", ".NET SDK", version)
                    .kind("sdk")
                    .at(path)
                    .about(".NET 的編譯與建置工具鏈（dotnet build / run / publish）"),
            );
        }
    }

    if let Ok(out) = sh::run("dotnet", &["--list-runtimes"], Duration::from_secs(60)) {
        for line in out.stdout.lines() {
            // Microsoft.NETCore.App 9.0.8 [/usr/local/share/dotnet/shared/...]
            let mut tokens = line.split_whitespace();
            let (Some(name), Some(version)) = (tokens.next(), tokens.next()) else {
                continue;
            };
            let path = tokens
                .next()
                .unwrap_or("")
                .trim_start_matches('[')
                .trim_end_matches(']')
                .to_string();
            let about = if name.starts_with("Microsoft.AspNetCore") {
                "ASP.NET Core 執行階段：跑網站與 Web API"
            } else {
                ".NET 執行階段：跑已編譯的 .NET 程式"
            };
            part.items.push(
                Item::new("dotnet", ".NET 執行階段", name, version)
                    .kind("runtime")
                    .at(path)
                    .about(about),
            );
        }
    }

    if part.items.is_empty() {
        return Err(String::from("dotnet 沒有回報任何 SDK 或執行階段"));
    }
    Ok(part)
}

pub fn php() -> Result<Part, String> {
    let Some(path) = sh::which("php") else {
        return Ok(Part::absent());
    };
    let path_str = path.display().to_string();
    let mut part = Part::new();

    match sh::run("php", &["-v"], Duration::from_secs(20)) {
        Ok(out) => match pick_version(out.text()) {
            Some(version) => {
                part.headline = Some(version.clone());
                let origin = origin_for(&path_str);
                let mut item = Item::new("php", origin, "PHP", &version)
                    .kind("runtime")
                    .at(path_str)
                    .about("PHP 直譯器：常用於網站後端與命令列腳本");
                if origin == "Homebrew" {
                    item = item.cmd("brew upgrade php").rm("brew uninstall php");
                }
                part.items.push(item);
            }
            None => part.warn("PHP 版本輸出無法解析".to_string()),
        },
        Err(e) => part.warn(format!("PHP 讀取失敗：{e}")),
    }

    if part.items.is_empty() {
        return Err(String::from("讀不到 PHP 版本"));
    }
    Ok(part)
}

/// 一次把全部 gem 的 summary 抓回來（48 個 gem 只要 0.05 秒）。
fn gem_summaries() -> HashMap<String, String> {
    const SCRIPT: &str = r#"require "rubygems"
Gem::Specification.each { |s| puts s.name + "\t" + s.summary.to_s.gsub(/\s+/, " ") }"#;

    let mut map = HashMap::new();
    let Ok(out) = sh::run("ruby", &["-e", SCRIPT], Duration::from_secs(90)) else {
        return map;
    };
    if !out.ok() {
        return map;
    }
    for line in out.stdout.lines() {
        let Some((name, summary)) = line.split_once('\t') else {
            continue;
        };
        let summary = summary.trim();
        if !name.trim().is_empty() && !summary.is_empty() {
            map.insert(name.trim().to_string(), summary.to_string());
        }
    }
    map
}

pub fn ruby() -> Result<Part, String> {
    let Some(path) = sh::which("ruby") else {
        return Ok(Part::absent());
    };
    let path_str = path.display().to_string();
    let origin = origin_for(&path_str);
    let mut part = Part::new();

    let mut version = String::from("未知");
    if let Ok(out) = sh::run("ruby", &["-v"], Duration::from_secs(20)) {
        let line = first_line(out.text());
        if let Some(rest) = line.strip_prefix("ruby ") {
            version = rest.split_whitespace().next().unwrap_or("未知").to_string();
        }
    }
    part.headline = Some(version.clone());

    let mut ruby_item = Item::new("ruby", origin, "Ruby", &version)
        .kind("runtime")
        .at(path_str)
        .about("Ruby 直譯器");
    if cfg!(target_os = "macos") {
        ruby_item = ruby_item.note_i18n(
            "macOS 內建；要新版請用 brew install ruby，不要直接替換系統 Ruby",
            "note.ruby.builtin",
            &[],
        );
    }
    part.items.push(ruby_item);

    let summaries = gem_summaries();

    match sh::run("gem", &["list", "--local"], Duration::from_secs(90)) {
        Ok(out) if out.ok() => {
            for line in out.stdout.lines() {
                let line = line.trim();
                let Some(open) = line.find('(') else { continue };
                let Some(close) = line.rfind(')') else { continue };
                if close <= open {
                    continue;
                }
                let name = line[..open].trim();
                if name.is_empty() || name.contains(' ') {
                    continue;
                }
                let raw = line[open + 1..close].trim();
                let is_default = raw.starts_with("default:");
                let cleaned = if is_default {
                    raw.trim_start_matches("default:").trim()
                } else {
                    raw
                };
                let version = cleaned.split(',').next().unwrap_or(cleaned).trim();
                let mut item = Item::new("ruby", "Ruby gem", name, version)
                    .kind("gem")
                    .cmd(format!("gem update {name}"));
                if let Some(summary) = summaries.get(name) {
                    item = item.about(summary.clone());
                }
                if is_default {
                    // default gem 動了會拖累系統 Ruby，不提供移除指令
                    item = item.note_i18n(
                        "Ruby 內建 default gem（更新通常需要 sudo，不建議動）",
                        "note.gem.default",
                        &[],
                    );
                } else {
                    item = item.rm(format!("gem uninstall {name}"));
                }
                part.items.push(item);
            }
        }
        Ok(out) => part.warn(format!("gem 清單讀取失敗：{}", first_line(out.text()))),
        Err(e) => part.warn(format!("gem 清單讀取失敗：{e}")),
    }

    Ok(part)
}
