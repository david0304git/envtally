import type { Lang } from "./i18n";

type Triple = [zhTW: string, zhCN: string, en: string];

/**
 * 我們自己寫的說明（不是上游 metadata）。
 * 上游的說明一律保持原文（多為英文），這裡只涵蓋我自己撰稿的那幾十項。
 * key 格式：`ecosystem|name`，找不到時退到 `ecosystem|kind` 的通用說明。
 */
const BY_NAME: Record<string, Triple> = {
  "brew|homebrew": [
    "macOS 的套件管理器（本次清單的來源）",
    "macOS 的软件包管理器（本次清单的来源）",
    "macOS package manager — where this inventory comes from",
  ],
  "python|Python": [
    "Python 直譯器本體；下面列出它這個環境裝了哪些套件",
    "Python 解释器本体；下面列出它这个环境装了哪些包",
    "The Python interpreter itself; its installed packages are listed below",
  ],
  "node|Node.js": [
    "JavaScript 執行環境（nvm 安裝的其中一個版本）",
    "JavaScript 运行环境（nvm 安装的其中一个版本）",
    "JavaScript runtime (one of the versions installed by nvm)",
  ],
  "node|npm": [
    "Node.js 的套件管理器本體（全域安裝都用它）",
    "Node.js 的软件包管理器本体（全局安装都用它）",
    "Node.js package manager (used for every global install)",
  ],
  "rust|rustup": [
    "Rust 工具鏈管理器本體：安裝/切換 toolchain、更新自己",
    "Rust 工具链管理器本体：安装/切换 toolchain、更新自己",
    "The rustup toolchain manager itself: install, switch, self-update",
  ],
  "java|Java": [
    "Java 執行環境（JDK）：跑 Java 程式、也提供編譯工具",
    "Java 运行环境（JDK）：运行 Java 程序，也提供编译工具",
    "Java runtime (JDK): runs and compiles Java",
  ],
  "dotnet|.NET SDK": [
    ".NET 的編譯與建置工具鏈（dotnet build / run / publish）",
    ".NET 的编译与构建工具链（dotnet build / run / publish）",
    ".NET build toolchain (dotnet build / run / publish)",
  ],
  "dotnet|Microsoft.NETCore.App": [
    ".NET 執行階段：跑已編譯的 .NET 程式",
    ".NET 运行时：运行已编译的 .NET 程序",
    ".NET runtime: runs compiled .NET apps",
  ],
  "dotnet|Microsoft.AspNetCore.App": [
    "ASP.NET Core 執行階段：跑網站與 Web API",
    "ASP.NET Core 运行时：运行网站与 Web API",
    "ASP.NET Core runtime: serves websites and Web APIs",
  ],
  "php|PHP": [
    "PHP 直譯器：常用於網站後端與命令列腳本",
    "PHP 解释器：常用于网站后端与命令行脚本",
    "PHP interpreter: web backends and CLI scripts",
  ],
  "ruby|Ruby": [
    "Ruby 直譯器：系統工具（如 CocoaPods）會用到",
    "Ruby 解释器：系统工具（如 CocoaPods）会用到",
    "Ruby interpreter, used by tools such as CocoaPods",
  ],
  "go|runtime": [
    "Go 編譯器與工具鏈：go build / test / run",
    "Go 编译器与工具链：go build / test / run",
    "Go compiler and toolchain: go build / test / run",
  ],
  "go|binary": [
    "用 go install 安裝的執行檔",
    "用 go install 安装的可执行文件",
    "A binary installed with go install",
  ],
  "deno|runtime": [
    "Deno 執行環境（TypeScript / JavaScript，預設沙箱權限）",
    "Deno 运行环境（TypeScript / JavaScript，默认沙箱权限）",
    "Deno runtime (TypeScript / JavaScript, sandboxed by default)",
  ],
  "bun|runtime": [
    "Bun 執行環境（JavaScript / TypeScript，內建套件管理器與打包器）",
    "Bun 运行环境（JavaScript / TypeScript，内置包管理器与打包器）",
    "Bun runtime (JavaScript / TypeScript with built-in package manager and bundler)",
  ],
  "bun|package": [
    "bun 全域安裝的套件",
    "bun 全局安装的软件包",
    "Installed globally with bun",
  ],
  "swift|toolchain": [
    "Swift 編譯器與工具鏈：swift build / run / test（由 Xcode 或命令列工具提供）",
    "Swift 编译器与工具链：swift build / run / test（由 Xcode 或命令行工具提供）",
    "Swift compiler and toolchain (provided by Xcode or the command line tools)",
  ],
  "swift|sdk": [
    "編譯時使用的 macOS SDK 版本",
    "编译时使用的 macOS SDK 版本",
    "The macOS SDK version used at compile time",
  ],
  "system|git": ["版本控制系統", "版本控制系统", "Version control system"],
  "system|git-lfs": ["Git 的大檔案（LFS）擴充", "Git 的大文件（LFS）扩展", "Git extension for large files (LFS)"],
  "system|curl": ["命令列 HTTP/FTP 傳輸工具", "命令行 HTTP/FTP 传输工具", "Command-line HTTP/FTP transfer tool"],
  "system|wget": ["命令列檔案下載工具", "命令行文件下载工具", "Command-line file downloader"],
  "system|jq": ["命令列 JSON 解析與轉換", "命令行 JSON 解析与转换", "Command-line JSON processor"],
  "system|yq": ["命令列 YAML 解析與轉換", "命令行 YAML 解析与转换", "Command-line YAML processor"],
  "system|ffmpeg": ["影音轉檔、剪輯與重編碼", "音视频转码、剪辑与重编码", "Audio/video transcoding and editing"],
  "system|ffprobe": ["讀取影音檔的串流與格式資訊", "读取音视频文件的流与格式信息", "Inspects audio/video stream metadata"],
  "system|gs": ["Ghostscript：PostScript/PDF 轉檔引擎", "Ghostscript：PostScript/PDF 转换引擎", "Ghostscript: PostScript/PDF engine"],
  "system|magick": ["ImageMagick：影像處理與格式轉換", "ImageMagick：图像处理与格式转换", "ImageMagick: image processing and conversion"],
  "system|sqlite3": ["SQLite 資料庫的命令列工具", "SQLite 数据库的命令行工具", "SQLite database CLI"],
  "system|openssl": ["TLS 連線與加解密工具組", "TLS 连接与加解密工具组", "TLS and crypto toolkit"],
  "system|cmake": ["跨平台建置系統產生器", "跨平台构建系统生成器", "Cross-platform build system generator"],
  "system|make": ["傳統 make 建置工具", "传统 make 构建工具", "Classic make build tool"],
  "system|tesseract": ["OCR 光學字元辨識（圖片轉文字）", "OCR 光学字符识别（图片转文字）", "OCR: images to text"],
  "system|exiftool": ["讀寫照片／影片的 metadata", "读写照片/视频的 metadata", "Reads and writes photo/video metadata"],
  "system|yt-dlp": ["從影音網站下載影片", "从视频网站下载视频", "Downloads videos from streaming sites"],
  "system|rg": ["ripgrep：極快的全文搜尋", "ripgrep：极快的全文搜索", "ripgrep: very fast full-text search"],
  "system|fd": ["快速尋找檔案（find 的現代替代）", "快速查找文件（find 的现代替代）", "Fast file finder (modern find)"],
  "system|gpg": ["GnuPG：加密、解密與簽章", "GnuPG：加密、解密与签名", "GnuPG: encryption and signing"],
  "system|pandoc": ["文件格式轉換（Markdown / PDF / docx…）", "文档格式转换（Markdown / PDF / docx…）", "Document converter (Markdown / PDF / docx…)"],
  "system|clang": ["C / C++ / Objective-C 編譯器前端", "C / C++ / Objective-C 编译器前端", "C / C++ / Objective-C compiler front end"],
  "system|vim": ["終端機文字編輯器", "终端文本编辑器", "Terminal text editor"],
  "system|zsh": ["Z shell（互動式命令列 shell）", "Z shell（交互式命令行 shell）", "Z shell (interactive command shell)"],
};

/** 名稱會變的項目（toolchain 名稱、Python 版本…）用 ecosystem|kind 當後備。 */
const BY_KIND: Record<string, Triple> = {
  "rust|toolchain": [
    "Rust 編譯器與工具鏈（rustc / cargo / rustup），一個 toolchain 就是一套完整版本",
    "Rust 编译器与工具链（rustc / cargo / rustup），一个 toolchain 就是一套完整版本",
    "A Rust compiler and toolchain (rustc / cargo / rustup); one toolchain is one complete set",
  ],
  "python|runtime": [
    "Python 直譯器本體；下面列出它這個環境裝了哪些套件",
    "Python 解释器本体；下面列出它这个环境装了哪些包",
    "The Python interpreter itself; its installed packages are listed below",
  ],
  "node|runtime": [
    "JavaScript 執行環境（nvm 安裝的其中一個版本）",
    "JavaScript 运行环境（nvm 安装的其中一个版本）",
    "JavaScript runtime (one of the versions installed by nvm)",
  ],
  "java|runtime": [
    "Java 執行環境（JDK）：跑 Java 程式、也提供編譯工具",
    "Java 运行环境（JDK）：运行 Java 程序，也提供编译工具",
    "Java runtime (JDK): runs and compiles Java",
  ],
  "dotnet|sdk": [
    ".NET 的編譯與建置工具鏈（dotnet build / run / publish）",
    ".NET 的编译与构建工具链（dotnet build / run / publish）",
    ".NET build toolchain (dotnet build / run / publish)",
  ],
  "php|runtime": [
    "PHP 直譯器：常用於網站後端與命令列腳本",
    "PHP 解释器：常用于网站后端与命令行脚本",
    "PHP interpreter: web backends and CLI scripts",
  ],
  "ruby|runtime": [
    "Ruby 直譯器：系統工具（如 CocoaPods）會用到",
    "Ruby 解释器：系统工具（如 CocoaPods）会用到",
    "Ruby interpreter, used by tools such as CocoaPods",
  ],
};

const index: Record<Lang, 0 | 1 | 2> = { "zh-TW": 0, "zh-CN": 1, en: 2 };

/** 回傳我們自己寫的說明；沒有就回傳 undefined（呼叫端再退回上游說明）。 */
export function localDescription(
  lang: Lang,
  ecosystem: string,
  name: string,
  kind: string,
): string | undefined {
  const entry = BY_NAME[`${ecosystem}|${name}`] ?? BY_KIND[`${ecosystem}|${kind}`];
  return entry ? entry[index[lang]] : undefined;
}
