use serde::Serialize;

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
// 掃描階段一律是 Unknown；Current / Outdated 由前端的版本檢查結果推導，
// 保留在 Rust 端是為了之後能改成「掃描時就自帶狀態」。
#[allow(dead_code)]
pub enum Status {
    /// 沒有對上游檢查過版本
    Unknown,
    /// 檢查過，已是最新（或本來就沒有「上游」概念，例如系統內建）
    Current,
    /// 有更新
    Outdated,
    /// 讀取失敗
    Error,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Item {
    /// 全域唯一：ecosystem|source|name|version
    pub id: String,
    /// 給「版本檢查」結果比對用：ecosystem|source|name
    pub key: String,
    pub ecosystem: String,
    pub name: String,
    pub version: String,
    pub source: String,
    /// runtime / toolchain / formula / cask / package / gem / sdk / tool
    pub kind: String,
    pub location: Option<String>,
    pub update_command: Option<String>,
    /// 對應的移除指令（只顯示、不執行）
    pub remove_command: Option<String>,
    pub note: Option<String>,
    /// 這個套件／工具在做什麼（來自上游 metadata，不是我手寫的）
    pub description: Option<String>,
    pub status: Status,
    pub latest: Option<String>,
}

impl Item {
    pub fn new(ecosystem: &str, source: &str, name: &str, version: &str) -> Self {
        let key = format!("{ecosystem}|{source}|{name}");
        Item {
            id: format!("{key}|{version}"),
            key,
            ecosystem: ecosystem.to_string(),
            name: name.to_string(),
            version: version.to_string(),
            source: source.to_string(),
            kind: String::from("package"),
            location: None,
            update_command: None,
            remove_command: None,
            note: None,
            description: None,
            status: Status::Unknown,
            latest: None,
        }
    }

    pub fn kind(mut self, kind: &str) -> Self {
        self.kind = kind.to_string();
        self
    }

    pub fn at(mut self, path: impl Into<String>) -> Self {
        self.location = Some(path.into());
        self
    }

    pub fn cmd(mut self, command: impl Into<String>) -> Self {
        self.update_command = Some(command.into());
        self
    }

    pub fn note(mut self, note: impl Into<String>) -> Self {
        self.note = Some(note.into());
        self
    }

    pub fn rm(mut self, command: impl Into<String>) -> Self {
        self.remove_command = Some(command.into());
        self
    }

    pub fn about(mut self, description: impl Into<String>) -> Self {
        let text = description.into();
        let trimmed = text.trim();
        if !trimmed.is_empty() {
            self.description = Some(trimmed.to_string());
        }
        self
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Ecosystem {
    pub id: String,
    pub label: String,
    pub blurb: String,
    pub items: Vec<Item>,
    pub error: Option<String>,
    pub warnings: Vec<String>,
    pub duration_ms: u64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Inventory {
    pub scanned_at_ms: u64,
    pub host: String,
    pub os: String,
    pub arch: String,
    pub model: String,
    pub ecosystems: Vec<Ecosystem>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Patch {
    /// 對應 Item.key
    pub key: String,
    pub latest: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Updates {
    pub checked_at_ms: u64,
    /// 這幾類真的檢查成功了，前端可以把它們標成「已最新」
    pub checked_ecosystems: Vec<String>,
    pub patches: Vec<Patch>,
    pub warnings: Vec<String>,
}
