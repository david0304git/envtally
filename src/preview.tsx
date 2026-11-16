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

  // ?selected=1 自動選第一列，方便看右側細節抽屜
  if (params.get("selected") === "1") {
    window.setTimeout(() => {
      document.querySelector<HTMLElement>(".inv-row")?.click();
    }, 700);
  }
}

void bootstrap();
