/* Envtally 官網：主題切換 + 從 GitHub API 取得最新版本的下載連結 */

const THEME_KEY = "envtally.site.theme";
const REPO = "david0304git/envtally";

/* ── 主題 ─────────────────────────────────────────────── */

function preferredTheme() {
  // ?theme=light|dark 可強制指定（截圖與分享連結用，不寫入 localStorage）
  const forced = new URLSearchParams(location.search).get("theme");
  if (forced === "light" || forced === "dark") return forced;
  const saved = localStorage.getItem(THEME_KEY);
  if (saved === "light" || saved === "dark") return saved;
  return window.matchMedia("(prefers-color-scheme: dark)").matches ? "dark" : "light";
}

function applyTheme(theme) {
  document.documentElement.dataset.theme = theme;
  const button = document.getElementById("theme-toggle");
  if (button) {
    button.setAttribute("aria-label", theme === "dark" ? "切換為淺色" : "切換為深色");
    button.title = button.getAttribute("aria-label");
  }
}

applyTheme(preferredTheme());

document.getElementById("theme-toggle")?.addEventListener("click", () => {
  const next = document.documentElement.dataset.theme === "dark" ? "light" : "dark";
  localStorage.setItem(THEME_KEY, next);
  applyTheme(next);
});

/* 跟隨系統變化（使用者沒有手動選過時） */
window
  .matchMedia("(prefers-color-scheme: dark)")
  .addEventListener("change", () => {
    if (!localStorage.getItem(THEME_KEY)) applyTheme(preferredTheme());
  });

/* ── 最新版本與下載連結 ───────────────────────────────── */

const RELEASES = `https://github.com/${REPO}/releases`;

function setText(id, text) {
  const el = document.getElementById(id);
  if (el) el.textContent = text;
}

function pickAsset(assets, patterns) {
  for (const pattern of patterns) {
    const hit = assets.find((asset) => pattern.test(asset.name));
    if (hit) return hit;
  }
  return null;
}

async function loadLatest() {
  try {
    const response = await fetch(`https://api.github.com/repos/${REPO}/releases/latest`, {
      headers: { Accept: "application/vnd.github+json" },
    });
    if (!response.ok) return;
    const release = await response.json();
    const assets = Array.isArray(release.assets) ? release.assets : [];

    const mac = pickAsset(assets, [/aarch64\.dmg$/i, /\.dmg$/i]);
    const win = pickAsset(assets, [/x64-setup\.exe$/i, /\.exe$/i, /\.msi$/i]);

    if (mac) {
      const link = document.getElementById("dl-mac");
      if (link) link.href = mac.browser_download_url;
    }
    if (win) {
      const link = document.getElementById("dl-win");
      if (link) link.href = win.browser_download_url;
    }

    // 同步版本字串（標題、安裝區、頁尾）
    const tag = release.tag_name || "";
    if (tag) {
      setText("version-label", tag);
      setText("footer-version", tag);
      setText("install-version", `目前版本 ${tag}`);
    }
    // 副檔名提示跟著實際檔案走
    const macKbd = document.querySelector("#dl-mac .kbd");
    const winKbd = document.querySelector("#dl-win .kbd");
    if (mac && macKbd) macKbd.textContent = mac.name.match(/\.(dmg|zip)$/i)?.[0] ?? ".dmg";
    if (win && winKbd) winKbd.textContent = win.name.match(/\.(exe|msi)$/i)?.[0] ?? ".exe";

    setText("install-version", `目前版本 ${tag} · ${assets.length} 個檔案`);
  } catch {
    /* 離線或 API 限流：維持指向 releases 頁面的預設連結 */
    void RELEASES;
  }
}

/* 除錯用：?debug=1 會在頁尾顯示實際視窗寬度 */
if (new URLSearchParams(location.search).get("debug") === "1") {
  const el = document.getElementById("footer-version");
  const paint = () => {
    if (el) el.textContent = `${innerWidth}×${innerHeight} · dpr ${devicePixelRatio}`;
  };
  paint();
  addEventListener("resize", paint);
}

loadLatest();
