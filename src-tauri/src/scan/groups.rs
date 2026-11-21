//! 「程式群組」：把同一個程式附帶的多個指令收成一個分類。
//!
//! 例：`R` 與 `Rscript` 都來自 `/Library/Frameworks/R.framework`；
//! `VirtualBox`、`VBoxManage`、`VBoxHeadless`… 都是同一個 VirtualBox 安裝。
//! 這些屬於第三方指令，若全丟進「全域指令」會變成雜物桶。
//!
//! 新增程式時只要在 `RULES` 加一列（名稱或路徑特徵）。

pub struct Rule {
    /// 分類 id（同時對應 i18n 的 `eco.<id>`）
    pub id: &'static str,
    /// 找不到翻譯時的顯示名稱
    pub label: &'static str,
    pub blurb: &'static str,
    /// 精確名稱（不分大小寫）
    names: &'static [&'static str],
    /// 名稱前綴（不分大小寫）
    prefixes: &'static [&'static str],
    /// 實際路徑特徵（解析過連結後的路徑）
    path_hint: Option<&'static str>,
}

pub const RULES: &[Rule] = &[
    Rule {
        id: "r",
        label: "R",
        blurb: "R 統計環境與附帶指令",
        names: &["r", "rscript"],
        prefixes: &[],
        path_hint: Some("r.framework"),
    },
    Rule {
        id: "virtualbox",
        label: "VirtualBox",
        blurb: "Oracle 虛擬機指令",
        names: &["virtualbox", "virtualboxvm"],
        prefixes: &["vbox", "virtualbox"],
        path_hint: Some("/virtualbox/"),
    },
    Rule {
        id: "uv",
        label: "uv",
        blurb: "Python 套件與工具管理器附帶指令",
        names: &["uv", "uvx"],
        prefixes: &[],
        path_hint: None,
    },
];

/// 這個指令屬於哪個程式群組（沒有就回傳 None → 留在「全域指令」）
pub fn owner_of(name: &str, target: &str) -> Option<&'static Rule> {
    let lower_name = name.to_lowercase();
    let lower_target = target.to_lowercase();

    RULES.iter().find(|rule| {
        rule.names.iter().any(|candidate| *candidate == lower_name)
            || rule
                .prefixes
                .iter()
                .any(|prefix| lower_name.starts_with(prefix))
            || rule
                .path_hint
                .map(|hint| lower_target.contains(hint))
                .unwrap_or(false)
    })
}

#[cfg(test)]
mod tests {
    use super::owner_of;

    fn id(name: &str, target: &str) -> Option<&'static str> {
        owner_of(name, target).map(|rule| rule.id)
    }

    #[test]
    fn groups_r_and_its_children() {
        assert_eq!(
            id(
                "R",
                "/Library/Frameworks/R.framework/Versions/4.5-arm64/Resources/bin/R"
            ),
            Some("r")
        );
        assert_eq!(
            id(
                "Rscript",
                "/Library/Frameworks/R.framework/Versions/4.5-arm64/Resources/bin/Rscript"
            ),
            Some("r")
        );
    }

    #[test]
    fn groups_virtualbox_family() {
        for name in [
            "VirtualBox",
            "VirtualBoxVM",
            "VBoxManage",
            "VBoxHeadless",
            "vbox-img",
            "vboximg-mount",
            "vboxwebsrv",
        ] {
            assert_eq!(id(name, "/usr/local/bin/whatever"), Some("virtualbox"), "{name}");
        }
    }

    #[test]
    fn groups_uv() {
        assert_eq!(id("uv", "/Users/x/.local/bin/uv"), Some("uv"));
        assert_eq!(id("uvx", "/Users/x/.local/bin/uvx"), Some("uv"));
    }

    #[test]
    fn leaves_unrelated_commands_alone() {
        assert_eq!(id("dreamina-canvas", "/Users/x/.local/bin/dreamina-canvas"), None);
        assert_eq!(id("docker", "/usr/local/bin/docker"), None);
        // 只有開頭相同、其實無關
        assert_eq!(id("vboxsomethingelse", "/usr/local/bin/x"), Some("virtualbox"));
        assert_eq!(id("uverse", "/opt/tool/bin/uverse"), None);
    }
}
