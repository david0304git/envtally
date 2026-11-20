//! 辨識「AI／Agent CLI」。
//!
//! 目的：使用者想知道「我這台電腦裝了哪些 agent CLI」。這類工具可能由 npm、
//! uv、Homebrew 或直接下載安裝，因此不能靠分類判斷，只能靠名稱辨識後
//! **跨分類標記**，再由前端用一個篩選 chip 一次列出。
//!
//! 維護方式：新增工具時只要把名稱（或它的 npm 套件名）加進 `NAMES`；
//! `HINTS` 是關鍵字兜底，避免名單漏更新時完全抓不到。

/// 明確的產品／執行檔／套件名稱（比對前會正規化：小寫、去掉 scope 與分隔符）
const NAMES: &[&str] = &[
    // Anthropic / OpenAI / Google
    "claude",
    "claudecode",
    "anthropic",
    "codex",
    "codexcli",
    "openai",
    "gemini",
    "geminicli",
    // 社群 agent
    "aider",
    "aiderchat",
    "opencode",
    "opencodeai",
    "crush",
    "qwen",
    "qwencode",
    "goose",
    "blockgoose",
    "cursoragent",
    "amazonq",
    "kiro",
    // 本地模型 / LLM 前端
    "ollama",
    "llm",
    "mods",
    "sgpt",
    "shellgpt",
    "aichat",
    // 3D／創作類 agent（本機實測遇到的）
    "tripo",
    "tripocli",
    "dreaminacanvas",
    "dreaminacli",
];

/// SDK／函式庫（名字很像 agent 工具，但其實是給程式呼叫的套件）
const EXCLUDE: &[&str] = &[
    "openai",
    "anthropic",
    "google",
    "genai",
    "langchain",
    "llamaindex",
    "transformers",
    "torch",
    "llama",
];

/// 關鍵字兜底（名稱含這些片段就算）
const HINTS: &[&str] = &[
    "claude",
    "codex",
    "gemini",
    "aider",
    "opencode",
    "crush",
    "ollama",
    "shellgpt",
    "tripo",
    "dreamina",
    "aiagent",
    "agentcli",
];

/// 正規化：小寫、去掉 npm scope、去掉分隔符（- _ . @ /）
fn normalize(name: &str) -> String {
    let lower = name.to_lowercase();
    let without_scope = match lower.rfind('/') {
        Some(index) => &lower[index + 1..],
        None => lower.as_str(),
    };
    without_scope
        .chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .collect()
}

/// 這個名稱是不是 AI／Agent CLI
pub fn is_agent_cli(name: &str) -> bool {
    let key = normalize(name);
    if key.is_empty() {
        return false;
    }
    if EXCLUDE.contains(&key.as_str()) {
        return false;
    }
    NAMES.contains(&key.as_str()) || HINTS.iter().any(|hint| key.contains(hint))
}

#[cfg(test)]
mod tests {
    use super::is_agent_cli;

    #[test]
    fn recognizes_installed_agent_clis() {
        assert!(is_agent_cli("tripo-cli"));
        assert!(is_agent_cli("tripo"));
        assert!(is_agent_cli("dreamina-canvas"));
        assert!(is_agent_cli("claude"));
        assert!(is_agent_cli("@anthropic-ai/claude-code"));
        assert!(is_agent_cli("@openai/codex"));
        assert!(is_agent_cli("@google/gemini-cli"));
        assert!(is_agent_cli("ollama"));
        assert!(is_agent_cli("aider-chat"));
    }

    #[test]
    fn ignores_unrelated_packages() {
        assert!(!is_agent_cli("npm"));
        assert!(!is_agent_cli("corepack"));
        assert!(!is_agent_cli("numpy"));
        assert!(!is_agent_cli("requests"));
        assert!(!is_agent_cli("ffmpeg"));
        assert!(!is_agent_cli(""));
        // 只有片段相同但實際無關
        assert!(!is_agent_cli("llmbench"));
        // SDK／函式庫不算 CLI
        assert!(!is_agent_cli("openai"));
        assert!(!is_agent_cli("anthropic"));
        // 但 OpenAI 的 CLI 套件（@openai/codex）仍要認得
        assert!(is_agent_cli("@openai/codex"));
    }
}
