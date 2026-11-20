//! 全域指令：PATH 上、但不屬於其他分類管理的可執行檔。
//!
//! 排除規則（避免與其他分類重複）：
//! - `*/Cellar/*`：Homebrew formula 已經在 Homebrew 分類列出
//! - `*/.cargo/bin/rustup`：rustup 的墊片，Rust 分類已列 toolchain
//! - `*/Python.framework/*`：Python 分類已列直譯器
//! - `*node_modules*`：npm / bun 全域套件已由 Node.js / Bun 分類列出
//!
//! 系統目錄（/usr/bin、/bin、/usr/sbin、/sbin）整批跳過：那些是 macOS 內建，
//! 其中值得看的已經在「系統工具」分類策展列出。

use crate::models::Item;
use crate::scan::{origin_for, pick_version, Part};
use crate::sh;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

/// 已經在「系統工具」分類策展列出的名稱（不重複列）
const CURATED: &[&str] = &[
    "clang", "cmake", "curl", "ffmpeg", "ffprobe", "git", "gpg", "jq", "make", "openssl",
    "sqlite3", "tesseract", "vim", "zsh",
];

/// 只探測前 N 個（超過就記一則警告）
const MAX_ITEMS: usize = 300;
/// 版本探測的總時間上限（避免少數會卡住的指令拖慢掃描）
const PROBE_DEADLINE_MS: u64 = 1200;
/// 這類指令跑 `--version` 會啟動圖形介面（實測 VirtualBox 會卡 89 秒），不探測
const GUI_LAUNCHERS: &[&str] = &["VirtualBox", "VirtualBoxVM"];
const PROBE_THREADS: usize = 8;

struct Candidate {
    name: String,
    path: PathBuf,
    dir: PathBuf,
    /// 解析過連結的實際路徑，用來判斷是否由其他分類管理
    target: String,
}

/// 這個實際路徑是否已由其他分類管理（不重複列）
fn is_managed(target: &str) -> bool {
    let path = target.replace('\\', "/");
    path.contains("/Cellar/")
        || path.ends_with("/.cargo/bin/rustup")
        || path.contains("/Python.framework/")
        || path.contains("node_modules")
        || path.contains("/.bun/install/")
        || path.contains("/.volta/tools/")
}

#[cfg(unix)]
fn is_executable(path: &Path) -> bool {
    use std::os::unix::fs::PermissionsExt;
    std::fs::metadata(path)
        .map(|meta| meta.is_file() && meta.permissions().mode() & 0o111 != 0)
        .unwrap_or(false)
}

#[cfg(not(unix))]
fn is_executable(path: &Path) -> bool {
    let extension = path
        .extension()
        .map(|ext| ext.to_string_lossy().to_lowercase())
        .unwrap_or_default();
    std::fs::metadata(path)
        .map(|meta| meta.is_file())
        .unwrap_or(false)
        && matches!(extension.as_str(), "exe" | "cmd" | "bat" | "com")
}

/// 從給定的目錄清單收集候選指令（同名只留第一個，與 PATH 優先序一致）
fn collect_from(dirs: &[PathBuf], warnings: &mut Vec<String>) -> Vec<Candidate> {
    let mut seen: Vec<String> = Vec::new();
    let mut out: Vec<Candidate> = Vec::new();
    let mut skipped_managed = 0usize;

    for dir in dirs {
        let Ok(entries) = std::fs::read_dir(dir) else {
            continue;
        };
        let mut names: Vec<String> = entries
            .flatten()
            .map(|entry| entry.file_name().to_string_lossy().to_string())
            .collect();
        names.sort();

        for name in names {
            if name.starts_with('.') || seen.iter().any(|seen_name| *seen_name == name) {
                continue;
            }
            if CURATED.contains(&name.as_str()) {
                continue;
            }

            let path = dir.join(&name);
            let resolved = std::fs::canonicalize(&path).unwrap_or_else(|_| path.clone());
            if !is_executable(&resolved) && !is_executable(&path) {
                continue;
            }

            let target = resolved.display().to_string();
            if is_managed(&target) {
                skipped_managed += 1;
                continue;
            }

            seen.push(name.clone());
            out.push(Candidate {
                name,
                path: path.clone(),
                dir: dir.clone(),
                target,
            });

            if out.len() >= MAX_ITEMS {
                warnings.push(format!(
                    "PATH 上的指令超過 {MAX_ITEMS} 個，只列出前 {MAX_ITEMS} 個"
                ));
                return out;
            }
        }
    }

    let _ = skipped_managed;
    out
}

/// 只掃「使用者與套件管理器」安裝指令的固定目錄。
/// 刻意不繼承整個 PATH —— 否則會把 App 內附的執行檔也算進來。
fn cli_dirs() -> Vec<PathBuf> {
    let home = sh::home();
    let mut dirs: Vec<String> = vec![
        format!("{home}/.local/bin"),
        format!("{home}/.cargo/bin"),
        format!("{home}/.bun/bin"),
        format!("{home}/.deno/bin"),
        format!("{home}/go/bin"),
        format!("{home}/.volta/bin"),
        String::from("/usr/local/bin"),
        String::from("/opt/homebrew/bin"),
        String::from("/opt/homebrew/sbin"),
    ];

    if let Ok(appdata) = std::env::var("APPDATA") {
        dirs.push(format!("{appdata}/npm"));
    }
    if let Ok(profile) = std::env::var("USERPROFILE") {
        dirs.push(format!("{profile}/scoop/shims"));
    }
    if let Ok(program_data) = std::env::var("PROGRAMDATA") {
        dirs.push(format!("{program_data}/chocolatey/bin"));
    }

    let mut out: Vec<PathBuf> = Vec::new();
    for dir in dirs {
        let path = PathBuf::from(&dir);
        if path.is_dir() && !out.contains(&path) {
            out.push(path);
        }
    }
    out
}

fn probe_version(name: &str, path: &Path) -> Option<String> {
    if GUI_LAUNCHERS.contains(&name) {
        return None;
    }
    let program = path.to_string_lossy().to_string();
    let out = sh::run(&program, &["--version"], Duration::from_millis(700)).ok()?;
    let combined = format!("{}\n{}", out.stdout, out.stderr);
    pick_version(&combined)
}

pub fn scan() -> Result<Part, String> {
    let mut warnings: Vec<String> = Vec::new();

    let dirs = cli_dirs();
    let mut candidates = collect_from(&dirs, &mut warnings);
    if candidates.is_empty() {
        return Ok(Part::absent());
    }
    candidates.sort_by(|a, b| a.name.cmp(&b.name));

    // 版本探測：分給幾個執行緒，超過期限就停止（剩下的顯示「—」）
    let deadline = Instant::now() + Duration::from_millis(PROBE_DEADLINE_MS);
    let threads = PROBE_THREADS.min(candidates.len());
    let chunk_size = candidates.len().div_ceil(threads);
    let mut versions: Vec<Option<String>> = vec![None; candidates.len()];

    std::thread::scope(|scope| {
        let handles: Vec<_> = candidates
            .chunks(chunk_size)
            .enumerate()
            .map(|(index, chunk)| {
                scope.spawn(move || {
                    let mut results: Vec<(usize, Option<String>)> = Vec::new();
                    for (offset, candidate) in chunk.iter().enumerate() {
                        let slot = index * chunk_size + offset;
                        if Instant::now() >= deadline {
                            results.push((slot, None));
                            continue;
                        }
                        results.push((slot, probe_version(&candidate.name, &candidate.path)));
                    }
                    results
                })
            })
            .collect();

        for handle in handles {
            if let Ok(results) = handle.join() {
                for (slot, version) in results {
                    if slot < versions.len() {
                        versions[slot] = version;
                    }
                }
            }
        }
    });

    let mut part = Part::new();
    let mut probed = 0usize;
    for (candidate, version) in candidates.iter().zip(versions) {
        let version = match version {
            Some(version) => {
                probed += 1;
                version
            }
            None => String::from("—"),
        };
        let source = origin_for(&candidate.target);
        let mut item = Item::new("cli", source, &candidate.name, &version)
            .kind("cli")
            .at(candidate.path.display().to_string())
            .about("PATH 上的可執行指令");
        if GUI_LAUNCHERS.contains(&candidate.name.as_str()) {
            item = item.note("圖形介面程式；執行 --version 會開啟視窗，因此不探測版本");
        } else if candidate.dir.to_string_lossy().contains(".local/bin") {
            item = item.note("使用者自行安裝（~/.local/bin）");
        }
        part.items.push(item);
    }

    let total = part.items.len();
    if probed < total {
        warnings.push(format!(
            "有 {} 個指令在時間內沒有回報版本（顯示「—」）",
            total - probed
        ));
    }
    for warning in warnings {
        part.warn(warning);
    }
    part.headline = Some(format!("{total} 個指令"));
    Ok(part)
}

#[cfg(test)]
mod tests {
    use super::{collect_from, is_managed, CURATED};
    use std::path::PathBuf;

    #[test]
    fn managed_paths_are_skipped() {
        assert!(is_managed("/opt/homebrew/Cellar/jq/1.7.1/bin/jq"));
        assert!(is_managed("/Users/x/.cargo/bin/rustup"));
        assert!(is_managed("/Library/Frameworks/Python.framework/Versions/3.13/bin/pip3.13"));
        assert!(is_managed("/Users/x/.nvm/versions/node/v24.12.0/lib/node_modules/npm/bin/npx"));
        assert!(is_managed("C:\\Users\\x\\.bun\\install\\global\\node_modules\\foo\\foo.exe"));

        assert!(!is_managed("/usr/local/bin/R"));
        assert!(!is_managed("/opt/homebrew/bin/brew"));
        assert!(!is_managed("/Users/x/.local/bin/uv"));
    }

    #[test]
    fn curated_names_are_not_listed() {
        assert!(CURATED.contains(&"git"));
        assert!(CURATED.contains(&"ffmpeg"));
        assert!(!CURATED.contains(&"VirtualBox"));
    }

    #[cfg(unix)]
    #[test]
    fn collects_executables_and_skips_the_rest() {
        use std::os::unix::fs::PermissionsExt;

        let base = std::env::temp_dir().join(format!("envbench-cli-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&base);
        std::fs::create_dir_all(&base).unwrap();

        // 可執行：會被收進來
        let tool = base.join("mytool");
        std::fs::write(&tool, "#!/bin/sh\necho 1.2.3\n").unwrap();
        std::fs::set_permissions(&tool, std::fs::Permissions::from_mode(0o755)).unwrap();

        // 策展名單內：跳過
        let git = base.join("git");
        std::fs::write(&git, "#!/bin/sh\n").unwrap();
        std::fs::set_permissions(&git, std::fs::Permissions::from_mode(0o755)).unwrap();

        // 不可執行：跳過
        std::fs::write(base.join("notes.txt"), "hello").unwrap();

        // 隱藏檔：跳過
        let hidden = base.join(".hidden");
        std::fs::write(&hidden, "#!/bin/sh\n").unwrap();
        std::fs::set_permissions(&hidden, std::fs::Permissions::from_mode(0o755)).unwrap();

        let dirs = vec![PathBuf::from(&base)];
        let mut warnings = Vec::new();
        let found: Vec<String> = collect_from(&dirs, &mut warnings)
            .into_iter()
            .map(|candidate| candidate.name)
            .collect();

        assert_eq!(found, vec![String::from("mytool")]);
        std::fs::remove_dir_all(&base).unwrap();
    }
}
