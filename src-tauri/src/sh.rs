use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::sync::OnceLock;
use std::time::Duration;

/// 一條指令的執行結果。
pub struct CmdResult {
    pub code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
}

impl CmdResult {
    pub fn ok(&self) -> bool {
        self.code == Some(0)
    }

    /// stdout 優先，空的就退回 stderr（很多工具把版本印在 stderr）。
    pub fn text(&self) -> &str {
        if self.stdout.trim().is_empty() {
            self.stderr.trim()
        } else {
            self.stdout.trim()
        }
    }
}

/// 家目錄（Windows 沒有 HOME，用 USERPROFILE）
pub fn home() -> String {
    if cfg!(windows) {
        std::env::var("USERPROFILE").unwrap_or_else(|_| String::from("C:\\"))
    } else {
        std::env::var("HOME").unwrap_or_else(|_| String::from("/"))
    }
}

/// PATH 分隔符：Windows 是 ';'，其他是 ':'
pub fn path_separator() -> char {
    if cfg!(windows) {
        ';'
    } else {
        ':'
    }
}

/// nvm 安裝了多個 node 版本時，取號碼最大的那個 bin 目錄。
fn newest_nvm_bin() -> Option<String> {
    let base = format!("{}/.nvm/versions/node", home());
    let mut best: Option<(Vec<u32>, String)> = None;
    for entry in std::fs::read_dir(&base).ok()?.flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        let nums: Vec<u32> = name
            .trim_start_matches('v')
            .split('.')
            .filter_map(|p| p.parse().ok())
            .collect();
        if nums.is_empty() {
            continue;
        }
        let bin = entry.path().join("bin").display().to_string();
        if best.as_ref().map(|(b, _)| nums > *b).unwrap_or(true) {
            best = Some((nums, bin));
        }
    }
    best.map(|(_, bin)| bin)
}

/// 從 Finder 啟動的 GUI app，PATH 只有 /usr/bin:/bin:...，
/// 所以探測時自己組一份夠用的 PATH（順序決定優先權）。
pub fn search_dirs() -> &'static Vec<PathBuf> {
    static DIRS: OnceLock<Vec<PathBuf>> = OnceLock::new();
    DIRS.get_or_init(|| {
        let h = home();
        let mut dirs: Vec<String> = Vec::new();

        if cfg!(windows) {
            // Windows：系統 PATH 優先，再補常見安裝位置
            if let Ok(inherited) = std::env::var("PATH") {
                for p in inherited.split(path_separator()) {
                    if !p.is_empty() {
                        dirs.push(p.to_string());
                    }
                }
            }
            dirs.extend([
                format!("{h}\\.cargo\\bin"),
                format!("{h}\\.bun\\bin"),
                format!("{h}\\.deno\\bin"),
                format!("{h}\\.local\\bin"),
                format!("{h}\\go\\bin"),
                format!("{h}\\scoop\\shims"),
            ]);
            if let Ok(appdata) = std::env::var("APPDATA") {
                dirs.push(format!("{appdata}\\npm"));
            }
            if let Ok(local) = std::env::var("LOCALAPPDATA") {
                dirs.push(format!("{local}\\Microsoft\\WindowsApps"));
                dirs.push(format!("{local}\\Programs\\Microsoft VS Code\\bin"));
            }
            if let Ok(program_data) = std::env::var("PROGRAMDATA") {
                dirs.push(format!("{program_data}\\chocolatey\\bin"));
            }
            dirs.extend([
                String::from("C:\\Program Files\\nodejs"),
                String::from("C:\\Program Files\\Git\\cmd"),
                String::from("C:\\Program Files\\dotnet"),
                String::from("C:\\Program Files\\Java"),
            ]);

            let mut out: Vec<PathBuf> = Vec::new();
            for d in dirs {
                let pb = PathBuf::from(&d);
                if pb.is_dir() && !out.contains(&pb) {
                    out.push(pb);
                }
            }
            return out;
        }

        if let Some(bin) = newest_nvm_bin() {
            dirs.push(bin);
        }
        dirs.extend([
            format!("{h}/.local/bin"),
            format!("{h}/.cargo/bin"),
            format!("{h}/.bun/bin"),
            format!("{h}/.volta/bin"),
            String::from("/opt/homebrew/bin"),
            String::from("/opt/homebrew/sbin"),
            String::from("/usr/local/bin"),
            String::from("/Library/Frameworks/Python.framework/Versions/Current/bin"),
            String::from("/usr/local/share/dotnet"),
            String::from("/usr/bin"),
            String::from("/bin"),
            String::from("/usr/sbin"),
            String::from("/sbin"),
        ]);
        if let Ok(inherited) = std::env::var("PATH") {
            for p in inherited.split(':') {
                if !p.is_empty() {
                    dirs.push(p.to_string());
                }
            }
        }
        let mut out: Vec<PathBuf> = Vec::new();
        for d in dirs {
            let pb = PathBuf::from(&d);
            if pb.is_dir() && !out.contains(&pb) {
                out.push(pb);
            }
        }
        out
    })
}

pub fn path_string() -> &'static str {
    static P: OnceLock<String> = OnceLock::new();
    P.get_or_init(|| {
        let separator = path_separator().to_string();
        search_dirs()
            .iter()
            .map(|p| p.display().to_string())
            .collect::<Vec<_>>()
            .join(&separator)
    })
}

pub fn which(program: &str) -> Option<PathBuf> {
    if program.contains('/') || (cfg!(windows) && program.contains('\\')) {
        let p = PathBuf::from(program);
        return if p.is_file() { Some(p) } else { None };
    }
    for dir in search_dirs() {
        let cand = dir.join(program);
        if cand.is_file() {
            return Some(cand);
        }
        if cfg!(windows) {
            // Windows 的可執行檔常帶副檔名（npm 是 npm.cmd）
            for ext in ["exe", "cmd", "bat"] {
                let with_ext = dir.join(format!("{program}.{ext}"));
                if with_ext.is_file() {
                    return Some(with_ext);
                }
            }
        }
    }
    None
}

pub fn exists(program: &str) -> bool {
    which(program).is_some()
}

/// Windows：從 GUI 程式 spawn 主控台程式時，不指定這個旗標就會閃出黑色視窗。
/// 一次掃描會跑幾十個指令，所以每個 spawn 都要套用。
#[cfg(windows)]
fn hide_console(command: &mut Command) {
    use std::os::windows::process::CommandExt;
    const CREATE_NO_WINDOW: u32 = 0x0800_0000;
    command.creation_flags(CREATE_NO_WINDOW);
}

#[cfg(not(windows))]
fn hide_console(_command: &mut Command) {}

/// 跑一條指令並抓輸出，逾時就砍掉（避免某個指令卡死整個掃描）。
pub fn run(program: &str, args: &[&str], timeout: Duration) -> Result<CmdResult, String> {
    let exe = which(program).ok_or_else(|| format!("找不到指令 {program}"))?;

    let mut command = Command::new(&exe);
    command
        .args(args)
        .env("PATH", path_string())
        .env("HOMEBREW_NO_AUTO_UPDATE", "1")
        .env("HOMEBREW_NO_ENV_HINTS", "1")
        .env("HOMEBREW_NO_ANALYTICS", "1")
        .env("PIP_DISABLE_PIP_VERSION_CHECK", "1")
        .env("NO_COLOR", "1")
        .env("LANG", "en_US.UTF-8")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    hide_console(&mut command);

    let child = command
        .spawn()
        .map_err(|e| format!("{program} 無法啟動：{e}"))?;

    let pid = child.id();
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        let _ = tx.send(child.wait_with_output());
    });

    match rx.recv_timeout(timeout) {
        Ok(Ok(out)) => Ok(CmdResult {
            code: out.status.code(),
            stdout: String::from_utf8_lossy(&out.stdout).to_string(),
            stderr: String::from_utf8_lossy(&out.stderr).to_string(),
        }),
        Ok(Err(e)) => Err(format!("{program} 執行失敗：{e}")),
        Err(_) => {
            if cfg!(windows) {
                let mut kill = Command::new("taskkill");
                kill.args(["/F", "/T", "/PID", &pid.to_string()]);
                hide_console(&mut kill);
                let _ = kill.output();
            } else {
                let _ = Command::new("/bin/kill")
                    .args(["-9", &pid.to_string()])
                    .output();
            }
            Err(format!("{program} 逾時（{} 秒）", timeout.as_secs()))
        }
    }
}
