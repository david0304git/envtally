import type { ReactElement } from "react";
import { BRAND_GLYPHS } from "./brandGlyphs";

/**
 * 左側選單的分類圖示：一律黑色方形底 + 白色圖形。
 *
 * - 有官方品牌標誌的分類（Homebrew / Python / Node.js / Rust / Java / .NET / PHP / Ruby）
 *   直接使用官方單色向量路徑（見 brandGlyphs.ts）。
 * - `全部` 與 `系統工具` 不是品牌，用自繪的通用圖示。
 */
const LOCAL: Record<string, { viewBox: string; size: number; node: ReactElement }> = {
  all: {
    viewBox: "0 0 14 14",
    size: 11,
    node: (
      <g fill="currentColor">
        <rect x="2" y="2" width="4.2" height="4.2" rx="1" />
        <rect x="7.8" y="2" width="4.2" height="4.2" rx="1" />
        <rect x="2" y="7.8" width="4.2" height="4.2" rx="1" />
        <rect x="7.8" y="7.8" width="4.2" height="4.2" rx="1" />
      </g>
    ),
  },
  cli: {
    viewBox: "0 0 14 14",
    size: 11,
    node: (
      <g>
        <g fill="currentColor">
          <rect x="1.7" y="3" width="6.2" height="1.7" rx="0.85" />
          <rect x="1.7" y="6.15" width="4.8" height="1.7" rx="0.85" />
          <rect x="1.7" y="9.3" width="6.2" height="1.7" rx="0.85" />
        </g>
        <path
          d="M9.7 5.4 11.6 7 9.7 8.6"
          fill="none"
          stroke="currentColor"
          strokeWidth="1.4"
          strokeLinecap="round"
          strokeLinejoin="round"
        />
      </g>
    ),
  },
  system: {
    viewBox: "0 0 14 14",
    size: 11,
    node: (
      <g stroke="currentColor" strokeWidth="1.3" strokeLinecap="round" strokeLinejoin="round">
        <rect x="1.9" y="3" width="10.2" height="8" rx="1.6" />
        <path d="M4.3 6.1 5.7 7.3 4.3 8.5M7.5 8.8h2.4" />
      </g>
    ),
  },
};

export default function EcosystemIcon({ id }: { id: string }) {
  const brand = BRAND_GLYPHS[id];

  if (brand) {
    return (
      <span className="rail-icon" aria-hidden="true">
        <svg viewBox="0 0 24 24" width="13" height="13" fill="currentColor">
          <path d={brand} />
        </svg>
      </span>
    );
  }

  const local = LOCAL[id] ?? LOCAL.system;
  return (
    <span className="rail-icon" aria-hidden="true">
      <svg viewBox={local.viewBox} width={local.size} height={local.size} fill="none">
        {local.node}
      </svg>
    </span>
  );
}
