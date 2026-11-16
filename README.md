# Version Manager

本機環境與套件盤點台。一個 Tauri 桌面應用，把這台 Mac 上「裝了什麼、哪個版本、哪些落後了」集中在一張表裡。

![介面截圖](docs/screenshot.png)

點一列，右側會顯示這個套件**在做什麼**、裝在哪、以及可以複製的更新指令：

<img src="docs/inspector.png" width="320" alt="細節面板" />

## 它做什麼

| 生態系 | 抓什麼 | 怎麼抓 |
|---|---|---|
| Homebrew | formula、cask、Homebrew 本體 | `brew list --versions`，並用 `opt/<name>` 連結判斷**目前生效**的 keg（多版本並存時才不會抓錯） |
| Python | 每個直譯器（最多 4 個，去重）+ 各自的 pip 套件 | 逐一跑 `<python> -m pip list --format=json` |
| Node.js | nvm 下的所有 node 版本、npm、全域套件 | 讀 `~/.nvm/versions/node/*`，`npm ls -g --json` |
| Rust | rustup 的每個 toolchain、`cargo install` 的執行檔 | 讀 `~/.rustup/toolchains/*`，`rustc --version` |
| 語言執行環境 | Java、.NET SDK/執行階段、PHP、Ruby 與 gem | `java_home -V`、`dotnet --list-*`、`php -v`、`gem list --local` |
| 系統工具 | git、curl、jq、ffmpeg、sqlite3 等 24 種命令列工具 | 各自跑 `--version`，取第一個版本號 |

六個生態系並行掃描，實測 **619 項 / 約 1 秒**。

### 套件說明從哪來

每一項都盡量附一句「這是什麼」，來源是**上游自己的 metadata，不是手寫的**：

| 來源 | 抓法 | 覆蓋 |
|---|---|---|
| Homebrew | `brew info --json=v2 --installed` 的 `desc` 欄位 | 154/154 |
| Python | `importlib.metadata` 的 `Summary`（本地讀，比 `pip list` 快） | 384/390 |
| npm 全域 | 各套件自己的 `package.json` → `description` | 5/6 |
| Ruby gem | `Gem::Specification#summary`（一次呼叫抓全部） | 48/48 |
| 執行環境 / 系統工具 | 程式內建的短說明（JDK、.NET、PHP、Ruby、git、ffmpeg…） | 68/68 |

合計 **610/619（98%）**。少數套件上游本來就沒寫 Summary（例如 `orjson`、`safetensors`、`corepack`），面板就只顯示版本相關資訊，不會硬塞一段猜測的文字。

### 版本狀態

顏色只用來表示狀態，不是裝飾：

- **琥珀色**＝有新版（`檢查更新` 按鈕觸發，會連網）
- 灰底小點＝已是最新
- 空心灰點＝還沒檢查過

`檢查更新` 依序問四個來源：`brew outdated --json=v2`、`npm outdated -g --json`、`pip list --outdated`、`rustup check`。**pip 有 300 多個套件，這一步可能要幾十秒到一兩分鐘**，所以做成按鈕而不是開場自動跑。

## 唯讀保證

這個工具不寫入任何系統設定、不執行升級、不改 shell 設定檔。它唯一做的事是**跑查詢指令並解析輸出**。

細節抽屜會給你現成的更新指令（例如 `brew upgrade aom`）方便複製，但**不會替你執行**；要動手請自己貼進終端機。

## 開發

```bash
npm install
npm run app:dev      # 開發模式：Vite + Tauri 視窗（HMR）
npm run app:build    # 打包 .app 到 src-tauri/target/release/bundle/macos/
npm run build        # 只做前端型別檢查 + 打包（不開視窗）
```

Rust 端測試：

```bash
cd src-tauri
cargo test                              # 純函式測試（版本解析、item 唯一性）
cargo test -- --ignored                 # 真的掃這台機器，印出各生態系的項數與耗時
cargo test -- --ignored dump_inventory_json   # 產生 public/preview-inventory.json
```

開發時若要 Cargo / npm 快取留在專案內（避免寫到 `~/.cargo`、`~/.npm`）：

```bash
CARGO_HOME=$PWD/.cargo-home npm run app:dev
npm_config_cache=./.npm-cache npm install
```

## 瀏覽器預覽（不需要開視窗）

`preview.html` 會用 `public/preview-inventory.json`（真實掃描結果）冒充 Tauri 的 `invoke`，所以前端程式碼完全不用改就能在瀏覽器裡看版面：

```bash
npm run dev          # 然後開 http://127.0.0.1:5173/preview.html
# ?checked=1   自動按下「檢查更新」→ 看落後狀態的樣式
# ?selected=1  自動選第一列 → 看右側細節抽屜
```

注意：預覽模式的「有新版」版本號是人工加的（真實版本檢查要連網），只有清單本身是真實資料。

## 移除

`scripts/uninstall.sh` 預設**只列清單、什麼都不刪**，加 `--yes` 才動手：

| 指令 | 會刪什麼 | 回收 |
|---|---|---|
| `bash scripts/uninstall.sh` | 預覽（等於 `npm run app:uninstall`） | — |
| `bash scripts/uninstall.sh --yes` | app 本體 + `~/Library` 快取 | ~4.7 MB |
| `bash scripts/uninstall.sh --with-caches --yes` | 再加建置產物（**保留原始碼**） | ~3.6 GB |
| `bash scripts/uninstall.sh --everything --yes` | 以上全部，並印出最後一行（自己刪專案資料夾） | ~3.7 GB |

### 這個工具在系統上留下什麼

| 位置 | 大小 | 是什麼 | 手動刪除 |
|---|---|---|---|
| `src-tauri/target/release/bundle/macos/Version Manager.app` | 4.4 MB | 打包好的 app | `rm -rf "…/Version Manager.app"` |
| `~/Library/Caches/io.lutszhong.versionmanager` | 8 KB | WKWebView 網路快取 | `rm -rf` |
| `~/Library/WebKit/io.lutszhong.versionmanager` | 332 KB | WKWebView 資料目錄 | `rm -rf` |
| `src-tauri/target` | 3.2 GB | Rust 建置產物（debug + release） | `cargo clean` |
| `.cargo-home` / `.npm-cache` | 173 / 165 MB | 開發時導向專案內的套件快取 | `rm -rf` |
| `node_modules` | 87 MB | 前端相依 | `rm -rf` |
| `.chrome-profile` | 7.9 MB | 無頭 Chrome 截圖用的暫存 profile | `rm -rf` |

**沒有**裝到別的地方：沒有 LaunchAgent／登入項目／開機啟動、沒有改 `~/.zshrc` 之類的 shell 設定、沒有 `npm link` 或 `cargo install` 任何全域指令，也**沒有改動** Homebrew／pip／npm 的套件狀態（掃描全程唯讀）。

刪掉快取後要重建：

```bash
npm install                                    # 重建 node_modules
CARGO_HOME=$PWD/.cargo-home npm run app:build  # 重新下載 crates 並打包（1–2 分鐘）
```

## 檔案結構

```
src/                     前端（React + TypeScript，無 UI 框架）
  App.tsx                狀態與版面組裝
  rows.ts                把版本檢查結果疊到掃描結果上、排序、分組
  components/            SpecPlate / EcosystemRail / Toolbar / InventoryTable / Inspector
  styles.css             全部樣式（設計語彙寫在檔案最上面）
  preview.tsx            瀏覽器預覽入口
src-tauri/
  src/models.rs          Item / Ecosystem / Inventory / Updates
  src/sh.rs              指令執行（自帶 PATH、逾時砍掉、找絕對路徑）
  src/scan/*.rs          每個生態系一個模組 + updates.rs（版本檢查）
  src/lib.rs             Tauri command：scan_local / check_updates
  tauri.conf.json        視窗 1240×820、透明標題列、CSP 只允許自身資源
docs/screenshot.png      README 用截圖
```

## 已知限制（刻意的 MVP 範圍）

- **沒有歷史快照**：不會記錄「上週裝了什麼」，也沒有 diff。
- **不能直接升級**：按鈕只複製指令，不執行。要加的話得先做 dry-run 與逐步確認。
- **只支援 macOS**：PATH 探測、`java_home`、`scutil` 都是 macOS 專屬。
- **`brew list --versions` 之外不做 `brew outdated` 快取**：每次按 `檢查更新` 都重新問。
- **說明是上游原文**：多為英文；只有系統工具與執行環境是我寫的中文短句。
- **`brew info --json=v2 --installed` 偶爾會漏 formula**（例如已停用的 `rubberband`、被改名的 `sdl2`→`sdl2-compat`）；掃描後會再用 `brew list --versions` 對帳補回，並在備註標明「brew info 沒有回報」。
- 從 Finder 直接開 `.app` 時 PATH 很窄，所以 `sh.rs` 會自己組一份 PATH（`/opt/homebrew/bin`、`~/.cargo/bin`、nvm 最新版的 bin…）。
