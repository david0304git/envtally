# Envbench

本機環境與套件盤點台。一個 Tauri 桌面應用，把這台 Mac 上「裝了什麼、哪個版本、哪些落後了」集中在一張表裡。

![介面截圖](docs/screenshot.png)

點一列，右側會顯示這個套件**在做什麼**、裝在哪，以及可以直接複製的**更新／移除指令**：

<img src="docs/inspector.png" width="320" alt="細節面板" />

## 它做什麼

| 生態系 | 抓什麼 | 怎麼抓 |
|---|---|---|
| Homebrew | formula、cask、Homebrew 本體 | `brew info --json=v2 --installed`（一次拿到說明＋已安裝版本），再用 `opt/<name>` 連結判斷**目前生效**的 keg；漏掉的 formula 用 `brew list` 對帳補回 |
| Python | 每個直譯器（最多 4 個，去重）+ 各自的 pip 套件 | 逐一跑 `<python> -c "import importlib.metadata…"` 直接讀本地 metadata（版本＋Summary，比 `pip list` 快） |
| Node.js | nvm 下的所有 node 版本、npm、全域套件 | 讀 `~/.nvm/versions/node/*`，`npm ls -g --json` |
| Rust | rustup 的每個 toolchain、`cargo install` 的執行檔 | 讀 `~/.rustup/toolchains/*`，`rustc --version` |
| Java | JDK 本體 | `java_home -V` |
| .NET | SDK 與執行階段 | `dotnet --list-sdks`、`dotnet --list-runtimes` |
| PHP | 直譯器 | `php -v` |
| Ruby | 直譯器與 gem（49 項） | `ruby -v`、`gem list --local` |
| Go | 工具鏈版本 + `$GOPATH/bin` 的執行檔 | `go version`、`go env GOPATH`、`go version -m`（從 build info 取模組版本） |
| Swift | Swift 工具鏈 + macOS SDK | `swift --version`、`xcrun --show-sdk-version`（注意跳過 swift-driver 的版本號） |
| Deno | 執行環境（含內建 V8 / TypeScript 版本） | `deno --version` |
| Bun | 執行環境 + 全域套件 | `bun --version`、讀 `~/.bun/install/global/node_modules/*/package.json` |
| 系統工具 | git、curl、jq、ffmpeg、sqlite3 等 24 種命令列工具 | 各自跑 `--version`，取第一個版本號 |

十三個分類並行掃描（沒安裝的自動隱藏），實測 **619 項 / 約 1 秒**。

### 左側分類選單

- **篩選分類**：打關鍵字過濾分類，分類名稱、說明、環境版本都會比對（例如打 `3.14` 會留下 Python）
- **排序**：`預設`（套件管理器 → 語言 → 工具）／`名稱`／`件數`。**按一次遞增、再按一次遞減**，按鈕上會出現 `↑` / `↓`
- **圖示**：分類名稱**右方**直接顯示官方標誌的黑色單色版（沒有底色方塊、不做雙色）。Homebrew / Python / Node.js / Rust / Java / .NET / PHP / Ruby 的路徑取自 [Simple Icons](https://simpleicons.org)（圖示檔 CC0-1.0）；`全部` 與 `系統工具` 不是品牌，用自繪通用圖示。各標誌為其擁有者的商標，此處僅用於辨識本機安裝的環境。

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

### 環境版本總覽（收合分類）

「全部」檢視的分類列可以整列點下去收合；收合後右邊直接給這組的**環境版本**，不用點進去看子套件：

![收合後的環境版本總覽](docs/overview.png)

| 分類 | 收合時顯示什麼 |
|---|---|
| Homebrew | `7.0.1`（Homebrew 本體版本） |
| Python | `3.14.7 · 3.13.6 · 3.12.12 · 3.9.6`（PATH 上的預設直譯器排第一） |
| Node.js | `24.12.0 · 22.19.0`（nvm 目前使用中的排第一） |
| Rust | `1.99.1`（預設 toolchain） |
| Java | `25.0.3` |
| .NET | `9.0.304`（SDK） |
| PHP | `8.4.11` |
| Ruby | `2.6.10p210`（gem 掛在這個分類下） |
| 系統工具 | 沒有單一版本，改顯示分類說明 |

工具列有「收合全部分類／展開全部分類」一次切換。**有搜尋關鍵字或狀態篩選時會強制展開**，免得命中結果藏在收合的分類裡。

### 表格排序

表頭 `名稱` / `已安裝` / `最新` / `來源` / `類型` 都可以點：

- 點別的欄位 → 該欄**遞增**排序
- 再點同一欄 → 切換成**遞減**（箭頭從 `↑` 變 `↓`，各欄位都有指示）
- 「全部」檢視時，排序在每個分類內部各自套用

### 沒安裝的分類會自動隱藏

掃描器先確認工具在不在：不在就回傳「這台沒有」，前端**整個分類不顯示**，也不會變成警告。底部狀態列會註明：

```
未安裝：Go · Deno · Bun        10 個生態系
```

所以在只有 Python 的機器上，畫面就是乾淨的兩個分類（Python + 系統工具），不會出現一堆「找不到 X」。分類清單目前有 13 個（Homebrew / Python / Node.js / Rust / Java / .NET / PHP / Ruby / Go / Swift / Deno / Bun / 系統工具）。

### 版本狀態

面板版本下方那行，講的是「**有沒有跟上游比對過版本**」，不是套件本身：

| 顯示 | 意思 |
|---|---|
| 還沒比對上游版本（按「檢查更新」） | 這個分類可以比對，但你還沒按過（或上次那個來源查詢失敗） |
| 已是最新 | 問過了，沒有新版 |
| 可更新至 X | 問過了，上游有 X（琥珀色） |
| 不檢查這個分類的版本 | 只有 `系統工具`（git/ffmpeg 這類）沒有上游版本可對，這行不會變 |

### 線上比對來源

八個分類都會去問上游（只有 `系統工具` 不比對）：

| 分類 | 來源 | 比對邏輯 |
|---|---|---|
| Homebrew | `brew outdated --json=v2` | 整個 Cellar |
| Python | `pip list --outdated --format=json` | 每個直譯器各自的套件 |
| Node.js | `npm outdated -g --json` | 全域套件 |
| Rust | `rustup check` | toolchain |
| Java | `api.adoptium.net/v3/assets/latest/{major}/hotspot` | **只比同一個 feature 版本的最新修補版**（不會叫你跳大版）；基準是 Adoptium Temurin |
| .NET | `dotnetcli.blob.core.windows.net/.../releases-index.json` | 同 channel 的 `latest-sdk` 與 `latest-runtime`，並回報 **EOL** |
| PHP | `formulae.brew.sh/api/formula/php.json` | Homebrew 官方 formula 的 stable 版 |
| Ruby | `api.github.com/repos/ruby/ruby/releases/latest` | 官方最新 release tag |

全部用系統內建的 `curl` 抓（沒有額外 TLS/HTTP 相依），逾時 25 秒，抓不到只會在警告列出現一行，不會讓整個檢查失敗。

表格用顏色表示同一件事：**琥珀色**＝有新版、灰綠小點＝已最新、空心灰點＝還沒比對。**pip 有 300 多個套件，檢查可能要幾十秒到一兩分鐘**，所以做成按鈕而不是開場自動跑。

## 唯讀保證

這個工具不寫入任何系統設定、不執行升級、不改 shell 設定檔。它唯一做的事是**跑查詢指令並解析輸出**。

細節抽屜會給你現成的更新與移除指令（例如 `brew upgrade aom` / `brew uninstall aom`）方便複製，但**不會替你執行**；要動手請自己貼進終端機。

### 更新／移除指令對照

| 來源 | 更新 | 移除 |
|---|---|---|
| Homebrew formula | `brew upgrade <name>` | `brew uninstall <name>` |
| Homebrew cask | `brew upgrade --cask <token>` | `brew uninstall --cask <token>` |
| pip 套件 | `python3.13 -m pip install -U <name>` | `python3.13 -m pip uninstall <name>` |
| npm 全域 | `npm install -g <name>@latest` | `npm uninstall -g <name>` |
| Node（nvm） | `nvm use vX.Y.Z` | `nvm uninstall vX.Y.Z` |
| Rust toolchain | `rustup update <toolchain>` | `rustup toolchain uninstall <toolchain>` |
| cargo 執行檔 | `cargo install <name> --force` | `cargo uninstall <name>` |
| Ruby gem | `gem update <name>` | `gem uninstall <name>` |

**刻意不給移除指令**的項目（共 54 項）：直譯器／工具鏈本體（Python、npm、Homebrew 自己、.NET SDK、JDK）、Ruby default gem、macOS 內建或 Xcode 附的命令列工具。這些移掉會弄壞系統或整個環境，與其給一條危險的指令，不如不給。

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

## 外觀與語言

右上角齒輪：**外觀**（自動／淺色／深色）與**語言**（繁體中文／简体中文／English）。設定存在 localStorage，下次開啟沿用；「自動」會跟隨 macOS 的深色模式。

![深色模式 + English](docs/dark-en.png)

介面全部三語；**上游套件說明**維持原文（多為英文），**讀取警告**也保留原始語言，因為內容常含指令的原始輸出。

## 匯出重建清單

右上角「匯出清單」（`⌘E`）會在 `~/Downloads/Envbench/export-<時間>/` 產生一包可直接使用的檔案：

| 檔案 | 內容 |
|---|---|
| `Brewfile` | Homebrew formula / cask → `brew bundle install --file=Brewfile` |
| `requirements-Python-*.txt` | 每個 Python 直譯器一份 → `pip install -r …` |
| `npm-globals.sh` | 全域 npm 套件 → `sh npm-globals.sh` |
| `gems.sh` | Ruby gems → `sh gems.sh` |
| `global.json` | .NET SDK 版本（`rollForward: latestPatch`） |
| `REBUILD.md` | 上面全部的說明與指令，語言跟隨介面 |

資料直接取自畫面上這份盤點結果（不是重新掃描），所以匯出的就是你看到的。匯出後可從提示條按「在 Finder 顯示」。

## 鍵盤快捷鍵

| 快捷鍵 | 動作 |
|---|---|
| `⌘R` | 重新掃描 |
| `⌘U` | 檢查更新 |
| `⌘E` | 匯出重建清單 |
| `⌘F` 或 `/` | 搜尋 |
| `⌘[` / `⌘]` | 上一個／下一個分類 |
| `⌘0` | 全部 |
| `Esc` | 清除選取／關閉面板 |
| `↑` `↓` | 在表格列之間移動焦點 |

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
| `src-tauri/target/release/bundle/macos/Envbench.app` | 4.4 MB | 打包好的 app | `rm -rf "…/Envbench.app"` |
| `~/Library/Caches/io.lutszhong.envbench` | 8 KB | WKWebView 網路快取 | `rm -rf` |
| `~/Library/WebKit/io.lutszhong.envbench` | 332 KB | WKWebView 資料目錄 | `rm -rf` |
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
  i18n.ts / i18nDesc.ts  三語字典（介面字串 + 我自己撰稿的說明）
  i18nContext.tsx        語言 context 與 useT()
  theme.ts / settings.tsx 外觀（自動/淺/深）與語言的持久化
src-tauri/
  src/models.rs          Item / Ecosystem / Inventory / Updates
  src/sh.rs              指令執行（自帶 PATH、逾時砍掉、找絕對路徑）
  src/scan/*.rs          每個生態系一個模組 + updates.rs（版本檢查）
  src/export.rs          匯出重建清單（Brewfile / requirements / npm / gems / global.json / REBUILD.md）
  src/lib.rs             Tauri command：scan_local / check_updates / export_rebuild / reveal_in_finder
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
