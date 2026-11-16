use crate::models::Item;
use crate::scan::{origin_for, pick_version, Part};
use crate::sh;
use std::time::Duration;

/// 只列「有明確版本輸出」的常見命令列工具，找不到就跳過。
const TOOLS: &[(&str, &[&str])] = &[
    ("git", &["--version"]),
    ("git-lfs", &["--version"]),
    ("curl", &["--version"]),
    ("wget", &["--version"]),
    ("jq", &["--version"]),
    ("yq", &["--version"]),
    ("ffmpeg", &["-version"]),
    ("ffprobe", &["-version"]),
    ("gs", &["--version"]),
    ("magick", &["-version"]),
    ("sqlite3", &["--version"]),
    ("openssl", &["version"]),
    ("cmake", &["--version"]),
    ("make", &["--version"]),
    ("tesseract", &["--version"]),
    ("exiftool", &["-ver"]),
    ("yt-dlp", &["--version"]),
    ("rg", &["--version"]),
    ("fd", &["--version"]),
    ("gpg", &["--version"]),
    ("pandoc", &["--version"]),
    ("clang", &["--version"]),
    ("vim", &["--version"]),
    ("zsh", &["--version"]),
];

pub fn scan() -> Result<Part, String> {
    let mut part = Part::new();

    for &(program, args) in TOOLS {
        let Some(path) = sh::which(program) else {
            continue;
        };
        let path_str = path.display().to_string();
        let origin = origin_for(&path_str);

        let item = match sh::run(program, args, Duration::from_secs(15)) {
            Ok(out) => {
                let text = format!("{}\n{}", out.stdout, out.stderr);
                let version = pick_version(&text).unwrap_or_else(|| String::from("未知"));
                Item::new("system", origin, program, &version)
                    .kind("tool")
                    .at(path_str)
            }
            Err(e) => {
                part.warn(format!("{program} 版本讀取失敗：{e}"));
                Item::new("system", origin, program, "未知")
                    .kind("tool")
                    .at(path_str)
            }
        };
        part.items.push(item);
    }

    if part.items.is_empty() {
        return Err(String::from("找不到任何已知的命令列工具"));
    }
    Ok(part)
}
