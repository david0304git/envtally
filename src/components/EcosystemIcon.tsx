import type { ReactElement } from "react";

/**
 * 左側選單的分類圖案：一律黑色方形底 + 白色線條圖示，
 * 所以顏色不能拿來做識別，造型才是（例如 Java 是咖啡杯）。
 */
const GLYPHS: Record<string, ReactElement> = {
  all: (
    <g fill="currentColor">
      <rect x="2" y="2" width="4.2" height="4.2" rx="1" />
      <rect x="7.8" y="2" width="4.2" height="4.2" rx="1" />
      <rect x="2" y="7.8" width="4.2" height="4.2" rx="1" />
      <rect x="7.8" y="7.8" width="4.2" height="4.2" rx="1" />
    </g>
  ),
  // Homebrew：啤酒杯（杯身 + 把手 + 波浪泡沫）
  brew: (
    <g stroke="currentColor" strokeWidth="1.25" strokeLinecap="round" strokeLinejoin="round">
      <path d="M3.5 5.6h5.2v5.2a1.2 1.2 0 0 1-1.2 1.2H4.7a1.2 1.2 0 0 1-1.2-1.2z" />
      <path d="M8.7 7h.9a1.6 1.6 0 0 1 0 3.2h-.9" />
      <path d="M3.5 5.6c.6-1 .1-1.6.8-2.4M5.1 5.6c.6-1 .1-1.6.8-2.4M6.7 5.6c.6-1 .1-1.6.8-2.4" strokeWidth="1" />
    </g>
  ),
  // Python：盤繞的蛇身（螺旋）
  python: (
    <g stroke="currentColor" strokeWidth="1.25" strokeLinecap="round">
      <path d="M6.9 2.5a4.5 4.5 0 1 0 0 9 3.3 3.3 0 1 0 0-6.6 2.1 2.1 0 1 0 0 4.2" />
    </g>
  ),
  // Node.js：六角形
  node: (
    <path
      d="M7 1.9 11.4 4.4v5.2L7 12.1 2.6 9.6V4.4z"
      stroke="currentColor"
      strokeWidth="1.3"
      strokeLinejoin="round"
    />
  ),
  // Rust：齒輪（本體 + 六個短齒）
  rust: (
    <g stroke="currentColor" strokeWidth="1.5" strokeLinecap="round">
      <circle cx="7" cy="7" r="2.9" />
      <path d="M7 1.9v1.6M7 10.5v1.6M1.9 7h1.6M10.5 7h1.6M3.4 3.4l1.1 1.1M9.5 9.5l1.1 1.1M10.6 3.4l-1.1 1.1M4.5 9.5l-1.1 1.1" strokeWidth="1.2" />
    </g>
  ),
  // Java：咖啡杯（熱氣 + 杯身 + 把手）
  java: (
    <g stroke="currentColor" strokeWidth="1.3" strokeLinecap="round" strokeLinejoin="round">
      <path d="M3 6.6h6v3.6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z" />
      <path d="M9 7.7h1.2a1.6 1.6 0 0 1 0 3.2H9" />
      <path d="M4.7 4.9c0-.9.8-1 .8-2M6.9 4.9c0-.9.8-1 .8-2" />
    </g>
  ),
  // .NET：大括號
  dotnet: (
    <g stroke="currentColor" strokeWidth="1.3" strokeLinecap="round" strokeLinejoin="round">
      <path d="M5.5 2.7c-1.2 0-1.6.6-1.6 1.6v1.5c0 .9-.4 1.2-1.1 1.2.7 0 1.1.3 1.1 1.2v1.5c0 1 .4 1.6 1.6 1.6" />
      <path d="M8.5 2.7c1.2 0 1.6.6 1.6 1.6v1.5c0 .9.4 1.2 1.1 1.2-.7 0-1.1.3-1.1 1.2v1.5c0 1-.4 1.6-1.6 1.6" />
    </g>
  ),
  // PHP：大象（雙耳 + 頭 + 象鼻）
  php: (
    <g stroke="currentColor" strokeWidth="1.15" strokeLinecap="round">
      <circle cx="2.9" cy="5.6" r="1.7" />
      <circle cx="11.1" cy="5.6" r="1.7" />
      <path d="M4.6 5.2h4.8v2.8c0 1.5-1 2.6-2.4 2.6s-2.4-1.1-2.4-2.6z" />
      <path d="M7.9 10.8v1.2" />
    </g>
  ),
  // Ruby：寶石
  ruby: (
    <g stroke="currentColor" strokeWidth="1.2" strokeLinejoin="round">
      <path d="M4.4 3.2h5.2l2.2 3.3L7 11.6 2.2 6.5z" />
      <path d="M2.2 6.5h9.6M5.6 6.5 7 11.6l1.4-5.1" strokeWidth="0.9" />
    </g>
  ),
  // 系統工具：終端機
  system: (
    <g stroke="currentColor" strokeWidth="1.3" strokeLinecap="round" strokeLinejoin="round">
      <rect x="1.9" y="3" width="10.2" height="8" rx="1.6" />
      <path d="M4.3 6.1 5.7 7.3 4.3 8.5M7.5 8.8h2.4" />
    </g>
  ),
};

export default function EcosystemIcon({ id }: { id: string }) {
  return (
    <span className="rail-icon" aria-hidden="true">
      <svg viewBox="0 0 14 14" width="11" height="11" fill="none">
        {GLYPHS[id] ?? GLYPHS.system}
      </svg>
    </span>
  );
}
