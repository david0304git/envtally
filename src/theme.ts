export type ThemeMode = "auto" | "light" | "dark";

const KEY = "version-manager.theme";

export function readThemeMode(): ThemeMode {
  try {
    const value = localStorage.getItem(KEY);
    if (value === "auto" || value === "light" || value === "dark") return value;
  } catch {
    /* 沒有 localStorage（例如沙箱）就用預設值 */
  }
  return "auto";
}

export function saveThemeMode(mode: ThemeMode) {
  try {
    localStorage.setItem(KEY, mode);
  } catch {
    /* 忽略 */
  }
}

export function systemPrefersDark(): boolean {
  return typeof window !== "undefined" && window.matchMedia
    ? window.matchMedia("(prefers-color-scheme: dark)").matches
    : false;
}

/** auto 跟隨系統，其餘直接指定。回傳實際套用的主題。 */
export function applyThemeMode(mode: ThemeMode): "light" | "dark" {
  const resolved = mode === "auto" ? (systemPrefersDark() ? "dark" : "light") : mode;
  document.documentElement.setAttribute("data-theme", resolved);
  return resolved;
}

// ── 語言 ──
import type { Lang } from "./i18n";

const LANG_KEY = "version-manager.lang";

export function readLang(): Lang {
  try {
    const value = localStorage.getItem(LANG_KEY);
    if (value === "zh-TW" || value === "zh-CN" || value === "en") return value;
  } catch {
    /* 忽略 */
  }
  // 第一次開啟：跟隨系統語言
  const nav = typeof navigator !== "undefined" ? navigator.language.toLowerCase() : "zh-tw";
  if (nav.startsWith("zh-cn") || nav.startsWith("zh-hans")) return "zh-CN";
  if (nav.startsWith("en")) return "en";
  return "zh-TW";
}

export function saveLang(lang: Lang) {
  try {
    localStorage.setItem(LANG_KEY, lang);
  } catch {
    /* 忽略 */
  }
}
