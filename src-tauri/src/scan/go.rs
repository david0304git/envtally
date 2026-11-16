//! Go：工具鏈版本 + `go install` 裝的執行檔（版本取自 build info 的 `mod` 行）。

use crate::models::Item;
use crate::scan::{origin_for, pick_version, Part};
use crate::sh;
use std::path::Path;
use std::time::Duration;

/// 從 `go version -m <bin>` 的輸出取出模組版本。
/// 格式：`\tpath\t<module>`、`\tmod\t<module>\t<version>\th1:...`
fn parse_module_version(text: &str) -> Option<String> {
    for line in text.lines() {
        let Some(rest) = line.trim_start().strip_prefix("mod\t") else {
            continue;
        };
        let mut fields = rest.split_whitespace();
        let _module = fields.next()?;
        if let Some(version) = fields.next() {
            return Some(version.to_string());
        }
    }
    None
}

pub fn scan() -> Result<Part, String> {
    if !sh::exists("go") {
        return Ok(Part::absent());
    }

    let mut part = Part::new();
    let go_path = sh::which("go")
        .map(|path| path.display().to_string())
        .unwrap_or_default();
    let origin = origin_for(&go_path);

    let version = sh::run("go", &["version"], Duration::from_secs(20))
        .ok()
        .and_then(|out| pick_version(out.text()))
        .unwrap_or_else(|| String::from("未知"));

    let mut item = Item::new("go", origin, "Go", &version)
        .kind("runtime")
        .at(go_path)
        .about("Go 編譯器與工具鏈：go build / test / run");
    if origin == "Homebrew" {
        item = item.cmd("brew upgrade go").rm("brew uninstall go");
    }
    part.items.push(item);
    part.headline = Some(version);

    // GOPATH/bin 底下的執行檔
    if let Ok(out) = sh::run("go", &["env", "GOPATH"], Duration::from_secs(10)) {
        let gopath = out.stdout.trim();
        if !gopath.is_empty() {
            let bin_dir = Path::new(gopath).join("bin");
            if let Ok(entries) = std::fs::read_dir(&bin_dir) {
                let mut files: Vec<_> = entries
                    .flatten()
                    .filter(|entry| entry.path().is_file())
                    .collect();
                files.sort_by_key(|entry| entry.file_name());
                if files.len() > 40 {
                    part.warn(format!(
                        "GOPATH/bin 有 {} 個執行檔，只列出前 40 個",
                        files.len()
                    ));
                }
                for entry in files.into_iter().take(40) {
                    let name = entry.file_name().to_string_lossy().to_string();
                    let path = entry.path().display().to_string();
                    let version =
                        sh::run("go", &["version", "-m", &path], Duration::from_secs(10))
                            .ok()
                            .and_then(|out| parse_module_version(&out.stdout))
                            .unwrap_or_else(|| String::from("—"));
                    part.items.push(
                        Item::new("go", "go install", &name, &version)
                            .kind("binary")
                            .at(path.clone())
                            .about("用 go install 安裝的執行檔")
                            .rm(format!("rm \"{path}\"")),
                    );
                }
            }
        }
    }

    Ok(part)
}

#[cfg(test)]
mod tests {
    use super::parse_module_version;

    #[test]
    fn reads_module_version() {
        let text = "/Users/x/go/bin/foo: go1.23.4\n\tpath\tgithub.com/acme/foo/cmd/foo\n\tmod\tgithub.com/acme/foo\tv1.2.3\th1:abc\n";
        assert_eq!(parse_module_version(text).as_deref(), Some("v1.2.3"));
    }

    #[test]
    fn returns_none_without_mod_line() {
        assert_eq!(parse_module_version("/x/bin/bar: go1.23.4\n"), None);
    }
}
