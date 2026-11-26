//! 少量 HTTP 查詢：直接用系統的 curl，不引入 TLS／HTTP 相依套件。

use crate::scan::first_line;
use crate::sh;
use std::time::Duration;

pub const TIMEOUT_SECS: u64 = 25;

/// GET 一個 JSON 端點並解析（失敗回傳可讀訊息，不要 panic）。
pub fn get_json(url: &str) -> Result<serde_json::Value, String> {
    let out = sh::run(
        "curl",
        &[
            "-sS",
            "-L",
            "--max-time",
            &TIMEOUT_SECS.to_string(),
            "-H",
            "Accept: application/json",
            "-H",
            "User-Agent: envtally",
            url,
        ],
        Duration::from_secs(TIMEOUT_SECS + 10),
    )?;

    if !out.ok() {
        let message = first_line(&out.stderr);
        return Err(if message.is_empty() {
            format!("HTTP {}", out.code.unwrap_or(-1))
        } else {
            message
        });
    }

    serde_json::from_str(&out.stdout).map_err(|e| format!("JSON 解析失敗：{e}"))
}

/// 去掉版本字串的建置資訊（"25.0.5+8-LTS" -> "25.0.5"）。
pub fn strip_build(version: &str) -> String {
    version
        .split('+')
        .next()
        .unwrap_or(version)
        .trim()
        .to_string()
}
