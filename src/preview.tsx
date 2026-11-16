/**
 * 瀏覽器預覽入口（npm run dev 後開 http://127.0.0.1:5173/preview.html）
 *
 * 不進 Tauri 也能看真實版面：把 public/preview-inventory.json（由
 * `cargo test -- --ignored dump_inventory_json` 從真實掃描產生）當資料源，
 * 並用假的 __TAURI_INTERNALS__ 冒充 invoke，前端程式碼完全不用改。
 */
import { createRoot } from "react-dom/client";
import "./styles.css";
import type { Inventory, Patch, Updates } from "./types";

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

  const patches: Patch[] = [];
  for (const eco of inventory.ecosystems) {
    for (const item of eco.items.slice(0, 3)) {
      patches.push({ key: item.key, latest: bumpVersion(item.version) });
    }
  }

  const updates: Updates = {
    checkedAtMs: Date.now(),
    checkedEcosystems: inventory.ecosystems.map((eco) => eco.id),
    patches,
    warnings: [],
  };

  window.__TAURI_INTERNALS__ = {
    invoke: async (command: string) => {
      if (command === "scan_local") return inventory;
      if (command === "check_updates") return updates;
      return null;
    },
  };

  const { default: App } = await import("./App");
  createRoot(container).render(<App />);

  const params = new URLSearchParams(window.location.search);

  // ?checked=1 自動觸發一次版本檢查，方便看「需更新」的樣式
  if (params.get("checked") === "1") {
    window.setTimeout(() => {
      const button = Array.from(document.querySelectorAll("button")).find((element) =>
        element.textContent?.includes("檢查更新"),
      );
      button?.click();
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
    window.setTimeout(() => {
      const button = Array.from(document.querySelectorAll<HTMLButtonElement>(".h-sort")).find(
        (element) => element.textContent?.includes(labels[key] ?? key),
      );
      button?.click();
      if (dir === "desc") window.setTimeout(() => button?.click(), 150);
    }, 500);
  }

  // ?railsort=<name|count>:<asc|desc> 代按左側排序，驗證升降冪
  const railSortParam = params.get("railsort");
  if (railSortParam) {
    const [key, dir] = railSortParam.split(":");
    const labels: Record<string, string> = { name: "名稱", count: "件數" };
    window.setTimeout(() => {
      const button = Array.from(document.querySelectorAll<HTMLButtonElement>(".rail-sort button")).find(
        (element) => element.textContent?.includes(labels[key] ?? key),
      );
      button?.click();
      if (dir === "desc") window.setTimeout(() => button?.click(), 150);
    }, 500);
  }

  // ?collapsed=1 自動按下「收合全部分類」，看環境版本總覽
  if (params.get("collapsed") === "1") {
    window.setTimeout(() => {
      const button = Array.from(document.querySelectorAll("button")).find((element) =>
        element.textContent?.includes("收合全部分類"),
      );
      button?.click();
    }, 500);
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
