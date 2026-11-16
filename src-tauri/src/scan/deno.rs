//! Deno：執行環境版本（Deno 沒有「已安裝套件清單」的概念，模組都在快取裡）。

use crate::models::Item;
use crate::scan::{origin_for, Part};
use crate::sh;
use std::time::Duration;

/// `deno --version` 第一行：`deno 2.1.4 (stable, release, aarch64-apple-darwin)`
fn parse_deno_version(text: &str) -> Option<String> {
    let line = text.lines().next()?;
    let rest = line.strip_prefix("deno ")?;
    let version = rest.split_whitespace().next()?;
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

/// 取 `v8 13.0.245.12` / `typescript 5.7.2` 這類「<前綴> <版本>」行。
fn parse_runtime_field(text: &str, prefix: &str) -> Option<String> {
    text.lines().find_map(|line| {
        let rest = line.trim().strip_prefix(prefix)?;
        rest.split_whitespace().next().map(|value| value.to_string())
    })
}

pub fn scan() -> Result<Part, String> {
    if !sh::exists("deno") {
        return Ok(Part::absent());
    }

    let mut part = Part::new();
    let out = sh::run("deno", &["--version"], Duration::from_secs(20))?;
    let text = out.stdout.clone();

    let version = parse_deno_version(&text).unwrap_or_else(|| String::from("未知"));
    let path = sh::which("deno")
        .map(|p| p.display().to_string())
        .unwrap_or_default();

    let mut item = Item::new("deno", origin_for(&path), "Deno", &version)
        .kind("runtime")
        .at(path)
        .about("Deno 執行環境（TypeScript / JavaScript，預設沙箱權限）")
        .cmd("deno upgrade");

    let mut notes: Vec<String> = Vec::new();
    if let Some(v8) = parse_runtime_field(&text, "v8 ") {
        notes.push(format!("V8 {v8}"));
    }
    if let Some(ts) = parse_runtime_field(&text, "typescript ") {
        notes.push(format!("TypeScript {ts}"));
    }
    if !notes.is_empty() {
        item = item.note(notes.join(" · "));
    }

    part.items.push(item);
    part.headline = Some(version);
    Ok(part)
}

#[cfg(test)]
mod tests {
    use super::{parse_deno_version, parse_runtime_field};

    const SAMPLE: &str =
        "deno 2.1.4 (stable, release, aarch64-apple-darwin)\nv8 13.0.245.12\ntypescript 5.7.2\n";

    #[test]
    fn reads_deno_version() {
        assert_eq!(parse_deno_version(SAMPLE).as_deref(), Some("2.1.4"));
    }

    #[test]
    fn reads_bundled_runtimes() {
        assert_eq!(parse_runtime_field(SAMPLE, "v8 ").as_deref(), Some("13.0.245.12"));
        assert_eq!(
            parse_runtime_field(SAMPLE, "typescript ").as_deref(),
            Some("5.7.2")
        );
    }

    #[test]
    fn rejects_junk() {
        assert_eq!(parse_deno_version("error: not found"), None);
    }
}
