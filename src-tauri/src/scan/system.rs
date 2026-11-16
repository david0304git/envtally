use crate::models::Item;
use crate::scan::{origin_for, pick_version, Part};
use crate::sh;
use std::time::Duration;

/// 常見命令列工具：程式名、版本參數、一句話說明。找不到就跳過。
const TOOLS: &[(&str, &[&str], &str)] = &[
    ("git", &["--version"], "版本控制系統"),
    ("git-lfs", &["--version"], "Git 的大檔案（LFS）擴充"),
    ("curl", &["--version"], "命令列 HTTP/FTP 傳輸工具"),
    ("wget", &["--version"], "命令列檔案下載工具"),
    ("jq", &["--version"], "命令列 JSON 解析與轉換"),
    ("yq", &["--version"], "命令列 YAML 解析與轉換"),
    ("ffmpeg", &["-version"], "影音轉檔、剪輯與重編碼"),
    ("ffprobe", &["-version"], "讀取影音檔的串流與格式資訊"),
    ("gs", &["--version"], "Ghostscript：PostScript/PDF 轉檔引擎"),
    ("magick", &["-version"], "ImageMagick：影像處理與格式轉換"),
    ("sqlite3", &["--version"], "SQLite 資料庫的命令列工具"),
    ("openssl", &["version"], "TLS 連線與加解密工具組"),
    ("cmake", &["--version"], "跨平台建置系統產生器"),
    ("make", &["--version"], "傳統 make 建置工具"),
    ("tesseract", &["--version"], "OCR 光學字元辨識（圖片轉文字）"),
    ("exiftool", &["-ver"], "讀寫照片／影片的 metadata"),
    ("yt-dlp", &["--version"], "從影音網站下載影片"),
    ("rg", &["--version"], "ripgrep：極快的全文搜尋"),
    ("fd", &["--version"], "快速尋找檔案（find 的現代替代）"),
    ("gpg", &["--version"], "GnuPG：加密、解密與簽章"),
    ("pandoc", &["--version"], "文件格式轉換（Markdown / PDF / docx…）"),
    ("clang", &["--version"], "C / C++ / Objective-C 編譯器前端"),
    ("vim", &["--version"], "終端機文字編輯器"),
    ("zsh", &["--version"], "Z shell（macOS 預設 shell）"),
];

pub fn scan() -> Result<Part, String> {
    let mut part = Part::new();

    for &(program, args, about) in TOOLS {
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
                    .about(about)
            }
            Err(e) => {
                part.warn(format!("{program} 版本讀取失敗：{e}"));
                Item::new("system", origin, program, "未知")
                    .kind("tool")
                    .at(path_str)
                    .about(about)
            }
        };
        part.items.push(item);
    }

    if part.items.is_empty() {
        return Err(String::from("找不到任何已知的命令列工具"));
    }
    Ok(part)
}
