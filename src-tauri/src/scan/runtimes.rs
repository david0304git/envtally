use crate::models::Item;
use crate::scan::{first_line, origin_for, pick_version, Part};
use crate::sh;
use std::time::Duration;

pub fn scan() -> Result<Part, String> {
    let mut part = Part::new();
    java(&mut part);
    dotnet(&mut part);
    php(&mut part);
    ruby(&mut part);

    if part.items.is_empty() {
        return Err(String::from("找不到 Java / .NET / PHP / Ruby"));
    }
    Ok(part)
}

fn java(part: &mut Part) {
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
            if !version.chars().next().map(|c| c.is_ascii_digit()).unwrap_or(false) {
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

            let mut item = Item::new(
                "runtimes",
                vendor.as_deref().unwrap_or("Java"),
                "Java",
                version,
            )
            .kind("runtime")
            .note("由 JDK 安裝器管理，升級需重新安裝");
            if let Some(home) = home {
                item = item.at(home);
            }
            if let Some(arch) = arch {
                item = item.note(format!("{arch} 架構"));
            }
            part.items.push(item);
            found = true;
        }
    }

    if !found {
        match sh::run("java", &["-version"], Duration::from_secs(20)) {
            Ok(out) => match pick_version(out.text()) {
                Some(version) => part
                    .items
                    .push(Item::new("runtimes", "Java", "Java", &version).kind("runtime")),
                None => part.warn(format!("java -version 輸出無法解析：{}", first_line(out.text()))),
            },
            Err(e) => part.warn(format!("Java 讀取失敗：{e}")),
        }
    }
}

fn dotnet(part: &mut Part) {
    if !sh::exists("dotnet") {
        return;
    }

    if let Ok(out) = sh::run("dotnet", &["--list-sdks"], Duration::from_secs(60)) {
        for line in out.stdout.lines() {
            // 9.0.304 [/usr/local/share/dotnet/sdk]
            let mut tokens = line.splitn(2, ' ');
            let Some(version) = tokens.next() else { continue };
            if !version.chars().next().map(|c| c.is_ascii_digit()).unwrap_or(false) {
                continue;
            }
            let path = tokens
                .next()
                .unwrap_or("")
                .trim()
                .trim_start_matches('[')
                .trim_end_matches(']')
                .to_string();
            part.items.push(
                Item::new("runtimes", ".NET SDK", ".NET SDK", version)
                    .kind("sdk")
                    .at(path),
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
            part.items.push(
                Item::new("runtimes", ".NET 執行階段", name, version)
                    .kind("runtime")
                    .at(path),
            );
        }
    }
}

fn php(part: &mut Part) {
    let Some(path) = sh::which("php") else {
        return;
    };
    let path_str = path.display().to_string();
    match sh::run("php", &["-v"], Duration::from_secs(20)) {
        Ok(out) => match pick_version(out.text()) {
            Some(version) => part.items.push(
                Item::new("runtimes", origin_for(&path_str), "PHP", &version)
                    .kind("runtime")
                    .at(path_str)
                    .cmd("brew upgrade php"),
            ),
            None => part.warn("PHP 版本輸出無法解析".to_string()),
        },
        Err(e) => part.warn(format!("PHP 讀取失敗：{e}")),
    }
}

fn ruby(part: &mut Part) {
    let Some(path) = sh::which("ruby") else {
        return;
    };
    let path_str = path.display().to_string();
    let origin = origin_for(&path_str);

    let mut version = String::from("未知");
    if let Ok(out) = sh::run("ruby", &["-v"], Duration::from_secs(20)) {
        let line = first_line(out.text());
        if let Some(rest) = line.strip_prefix("ruby ") {
            version = rest
                .split_whitespace()
                .next()
                .unwrap_or("未知")
                .to_string();
        }
    }
    part.items.push(
        Item::new("runtimes", origin, "Ruby", &version)
            .kind("runtime")
            .at(path_str),
    );

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
                let mut item = Item::new("runtimes", "Ruby gem", name, version)
                    .kind("gem")
                    .cmd(format!("gem update {name}"));
                if is_default {
                    item = item.note("Ruby 內建 default gem（更新通常需要 sudo，不建議動）");
                }
                part.items.push(item);
            }
        }
        Ok(out) => part.warn(format!(
            "gem 清單讀取失敗：{}",
            first_line(out.text())
        )),
        Err(e) => part.warn(format!("gem 清單讀取失敗：{e}")),
    }
}
