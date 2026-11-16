import { createContext, useContext, useMemo, type ReactNode } from "react";
import { translate, type Lang } from "./i18n";

const LangContext = createContext<Lang>("zh-TW");

export function LangProvider({ lang, children }: { lang: Lang; children: ReactNode }) {
  return <LangContext.Provider value={lang}>{children}</LangContext.Provider>;
}

export function useLang(): Lang {
  return useContext(LangContext);
}

export type T = (key: string, params?: Record<string, string | number>) => string;

export function useT(): T {
  const lang = useContext(LangContext);
  return useMemo<T>(
    () => (key, params) => translate(lang, key, params),
    [lang],
  );
}
