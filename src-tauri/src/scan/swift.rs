//! Swift：Apple 提供的工具鏈（Xcode 或命令列工具）+ macOS SDK。
//! 注意 `swift --version` 第一個版本號是 swift-driver，真正的語言版本在
//! `Apple Swift version 6.4` 之後。

use crate::models::Item;
use crate::scan::Part;
use crate::sh;
use std::time::Duration;

fn parse_swift_version(text: &str) -> Option<String> {
    let marker = "Swift version ";
    let start = text.find(marker)? + marker.len();
    let version = text[start..].split_whitespace().next()?;
    if version
        .chars()
        .next()
        .map(|c| c.is_ascii_digit())
        .unwrap_or(false)
    {
        Some(version.to_string())
    } else {
        None
    }
}

fn parse_target(text: &str) -> Option<String> {
    text.lines().find_map(|line| {
        line.trim()
            .strip_prefix("Target: ")
            .map(|rest| rest.trim().to_string())
            .filter(|rest| !rest.is_empty())
    })
}

pub fn scan() -> Result<Part, String> {
    if !sh::exists("swift") {
        return Ok(Part::absent());
    }

    let mut part = Part::new();
    let out = sh::run("swift", &["--version"], Duration::from_secs(30))?;
    let text = format!("{}\n{}", out.stdout, out.stderr);
    let version = parse_swift_version(&text).unwrap_or_else(|| String::from("未知"));
    let path = sh::which("swift")
        .map(|p| p.display().to_string())
        .unwrap_or_default();

    let dev_dir = sh::run("xcode-select", &["-p"], Duration::from_secs(10))
        .ok()
        .map(|out| out.stdout.trim().to_string())
        .filter(|text| !text.is_empty());
    let source = match dev_dir.as_deref() {
        Some(dir) if dir.contains("CommandLineTools") => "Xcode 命令列工具",
        Some(_) => "Xcode",
        None => "macOS 內建",
    };

    let mut item = Item::new("swift", source, "Swift", &version)
        .kind("toolchain")
        .at(path)
        .about("Swift 編譯器與工具鏈：swift build / run / test（由 Xcode 或命令列工具提供）");
    if let Some(target) = parse_target(&text) {
        item = item.note(target);
    }
    part.items.push(item);
    part.headline = Some(version);

    if let Ok(out) = sh::run("xcrun", &["--show-sdk-version"], Duration::from_secs(15)) {
        let sdk = out.stdout.trim();
        if !sdk.is_empty() {
            part.items.push(
                Item::new("swift", source, "macOS SDK", sdk)
                    .kind("sdk")
                    .about("編譯時使用的 macOS SDK 版本"),
            );
        }
    }

    Ok(part)
}

#[cfg(test)]
mod tests {
    use super::{parse_swift_version, parse_target};

    #[test]
    fn ignores_swift_driver_version() {
        let text = "swift-driver version: 1.168.6 Apple Swift version 6.4 (swiftlang-6.4.0.34.1 clang-2100.3.34.1)\nTarget: arm64-apple-macosx27.0.0\n";
        assert_eq!(parse_swift_version(text).as_deref(), Some("6.4"));
        assert_eq!(parse_target(text).as_deref(), Some("arm64-apple-macosx27.0.0"));
    }

    #[test]
    fn handles_plain_swift_banner() {
        assert_eq!(
            parse_swift_version("Swift version 5.9.2 (swift-5.9.2-RELEASE)").as_deref(),
            Some("5.9.2")
        );
    }
}
