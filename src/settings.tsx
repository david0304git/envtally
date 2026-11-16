import { createContext, useContext } from "react";
import type { Lang } from "./i18n";
import type { ThemeMode } from "./theme";

export interface SettingsValue {
  lang: Lang;
  theme: ThemeMode;
  setLang: (lang: Lang) => void;
  setTheme: (theme: ThemeMode) => void;
}

export const SettingsContext = createContext<SettingsValue | null>(null);

export function useSettings(): SettingsValue {
  const value = useContext(SettingsContext);
  if (!value) throw new Error("useSettings 必須在 SettingsContext 內使用");
  return value;
}
