/**
 * 瀏覽器預覽入口（npm run dev 後開 http://127.0.0.1:5173/preview.html）
 *
 * 不進 Tauri 也能看真實版面：把 public/preview-inventory.json（由
 * `cargo test -- --ignored dump_inventory_json` 從真實掃描產生）當資料源，
 * 並用假的 __TAURI_INTERNALS__ 冒充 invoke，前端程式碼完全不用改。
 */
import { createRoot } from "react-dom/client";
import "./styles.css";
import type { Inventory, Updates } from "./types";

type Invoke = (command: string) => Promise<unknown>;

declare global {
  interface Window {
    __TAURI_INTERNALS__?: { invoke: Invoke };
  }
}

/** 2.3.1 -> 2.3.2，用來生成假的「有新版」標記 */
function bumpVersion(version: string): string {
  const parts = version.split(".");
  const index = parts.length - 1;
  const last = Number.parseInt(parts[index] ?? "", 10);
  if (Number.isNaN(last)) return `${version}.1`;
  parts[index] = String(last + 1);
  return parts.join(".");
}

async function bootstrap() {
  const container = document.getElementById("root");
  if (!container) throw new Error("找不到 #root");

  const response = await fetch("/preview-inventory.json");
  if (!response.ok) {
    throw new Error(
      "找不到 preview-inventory.json，先在 src-tauri 跑：cargo test -- --ignored dump_inventory_json",
    );
  }
  const inventory = (await response.json()) as Inventory;

  // 有 public/preview-updates.json 就用真實檢查結果，否則退回「前 3 筆 +1」的假資料
  const realUpdates = await fetch("/preview-updates.json")
    .then((response) => (response.ok ? (response.json() as Promise<Updates>) : null))
    .catch(() => null);

  const updates: Updates = realUpdates ?? {
    checkedAtMs: Date.now(),
    checkedEcosystems: inventory.ecosystems
      .filter((eco) => eco.checkable)
      .map((eco) => eco.id),
    patches: inventory.ecosystems.flatMap((eco) =>
      eco.items.slice(0, 3).map((item) => ({ key: item.key, latest: bumpVersion(item.version) })),
    ),
    warnings: [],
  };

  window.__TAURI_INTERNALS__ = {
    invoke: async (command: string) => {
      if (command === "scan_local") return inventory;
      if (command === "check_updates") return updates;
      if (command === "export_rebuild") {
        return {
          folder: "/tmp/version-manager-preview/export",
          files: ["Brewfile", "requirements-Python-3.13.6.txt", "REBUILD.md"],
          itemCount: 0,
        };
      }
      if (command === "reveal_in_finder") return null;
      return null;
    },
  };

  const { default: App } = await import("./App");
  createRoot(container).render(<App />);

  const params = new URLSearchParams(window.location.search);

  // ?theme=light|dark|auto / ?lang=zh-TW|zh-CN|en 直接寫進 localStorage（App 啟動時會讀）
  const themeParam = params.get("theme");
  if (themeParam === "light" || themeParam === "dark" || themeParam === "auto") {
    localStorage.setItem("envbench.theme", themeParam);
  }
  const langParam = params.get("lang");
  if (langParam === "zh-TW" || langParam === "zh-CN" || langParam === "en") {
    localStorage.setItem("envbench.lang", langParam);
  }

  // ?checked=1 自動觸發一次版本檢查（用 title 裡的 ⌘U 找按鈕，不受語言影響）
  if (params.get("checked") === "1") {
    window.setTimeout(() => {
      document.querySelector<HTMLButtonElement>('[data-action="check"]')?.click();
    }, 400);
  }

  // ?sort=<name|version|latest|source|kind>:<asc|desc> 代按表頭，驗證升降冪
  const sortParam = params.get("sort");
  if (sortParam) {
    const [key, dir] = sortParam.split(":");
    const labels: Record<string, string> = {
      name: "名稱",
      version: "已安裝",
      latest: "最新",
      source: "來源",
      kind: "類型",
    };
    const order = ["name", "version", "latest", "source", "kind"];
    window.setTimeout(() => {
      const columns = Array.from(document.querySelectorAll<HTMLButtonElement>(".h-sort"));
      const button = columns[Math.max(0, order.indexOf(key))];
      void labels; // 保留 labels 供除錯時對照
      button?.click();
      if (dir === "desc") window.setTimeout(() => button?.click(), 150);
    }, 500);
  }

  // ?railsort=<name|count>:<asc|desc> 代按左側排序，驗證升降冪
  const railSortParam = params.get("railsort");
  if (railSortParam) {
    const [key, dir] = railSortParam.split(":");
    window.setTimeout(() => {
      const buttons = Array.from(document.querySelectorAll<HTMLButtonElement>(".rail-sort button"));
      const button = buttons[key === "count" ? 2 : 1];
      button?.click();
      if (dir === "desc") window.setTimeout(() => button?.click(), 150);
    }, 500);
  }

  // ?settings=1 打開設定面板
  if (params.get("settings") === "1") {
    window.setTimeout(() => {
      document.querySelector<HTMLButtonElement>(".btn--icon")?.click();
    }, 500);
  }

  // ?collapsed=1 自動按下「收合全部分類」，看環境版本總覽
  if (params.get("collapsed") === "1") {
    window.setTimeout(() => {
      document.querySelector<HTMLButtonElement>(".sheet-toolbar .btn--ghost")?.click();
    }, 500);
  }

  // ?warnings=1 直接打開右上角的警告面板
  if (params.get("warnings") === "1") {
    setTimeout(() => {
      document.querySelector<HTMLButtonElement>('[data-action="warnings"]')?.click();
    }, 400);
  }

  // ?filter=<all|outdated|current|unknown|agent> 預先切換篩選 chip
  const previewFilter = params.get("filter");
  if (previewFilter) {
    setTimeout(() => {
      document
        .querySelector<HTMLButtonElement>(`.seg button[data-filter="${previewFilter}"]`)
        ?.click();
    }, 250);
  }

  // ?q=<關鍵字> 預先填入搜尋框
  const previewQuery = params.get("q");
  if (previewQuery) {
    setTimeout(() => {
      const input = document.querySelector<HTMLInputElement>(".search input");
      if (!input) return;
      const setter = Object.getOwnPropertyDescriptor(
        HTMLInputElement.prototype,
        "value",
      )?.set;
      setter?.call(input, previewQuery);
      input.dispatchEvent(new Event("input", { bubbles: true }));
    }, 250);
  }

  // ?focus=<ecoId> 直接把左側分類切到某一個（例如 focus=cli）
  const focusEco = params.get("focus");
  if (focusEco) {
    setTimeout(() => {
      document
        .querySelector<HTMLButtonElement>(`.rail-item[data-eco="${focusEco}"]`)
        ?.click();
    }, 300);
  }

  // ?selected=1 自動選第一列；?select=<名稱> 選指定那一列，方便看細節抽屜
  const wantedSelect = params.get("select");
  if (params.get("selected") === "1" || wantedSelect) {
    window.setTimeout(() => {
      const rows = Array.from(document.querySelectorAll<HTMLElement>(".inv-row"));
      const target =
        wantedSelect === null
          ? rows[0]
          : rows.find(
              (row) =>
                row.querySelector(".cell-name")?.textContent?.trim().toLowerCase() ===
                wantedSelect.toLowerCase(),
            );
      target?.click();
      target?.scrollIntoView({ block: "center" });
    }, 700);
  }
}

void bootstrap();
