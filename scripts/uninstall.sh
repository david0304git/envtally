#!/usr/bin/env bash
#
# Version Manager — 移除工具
#
#   bash scripts/uninstall.sh                        # 預覽：只列出會刪什麼（預設）
#   bash scripts/uninstall.sh --yes                  # 真的移除 app 與它的系統快取（約 5 MB）
#   bash scripts/uninstall.sh --with-caches --yes    # 另外清掉建置快取（約 3.6 GB，保留原始碼）
#   bash scripts/uninstall.sh --everything --yes     # 以上全部，並印出刪除專案資料夾的指令
#
# 移除的東西全部是可再生的：app 用 npm run app:build 重建、快取用 npm install 重建。
# 唯一不可再生的是這個資料夾裡的原始碼與 git 記錄，腳本不會刪它（除非你自己執行最後那行）。
#
set -uo pipefail

PROJECT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
BUNDLE_ID="io.lutszhong.versionmanager"
APP="$PROJECT_DIR/src-tauri/target/release/bundle/macos/Version Manager.app"

WITH_CACHES=0
EVERYTHING=0
CONFIRM=0

for arg in "$@"; do
  case "$arg" in
    --yes | -y) CONFIRM=1 ;;
    --with-caches) WITH_CACHES=1 ;;
    --everything)
      WITH_CACHES=1
      EVERYTHING=1
      ;;
    -h | --help)
      sed -n '2,14p' "$0"
      exit 0
      ;;
    *)
      echo "未知參數：$arg"
      echo "用法：bash scripts/uninstall.sh [--with-caches|--everything] [--yes]"
      exit 2
      ;;
  esac
done

targets=(
  "$APP"
  "$HOME/Library/Caches/$BUNDLE_ID"
  "$HOME/Library/WebKit/$BUNDLE_ID"
  "$HOME/Library/Application Support/$BUNDLE_ID"
  "$HOME/Library/HTTPStorages/$BUNDLE_ID"
  "$HOME/Library/HTTPStorages/$BUNDLE_ID.binarycookies"
  "$HOME/Library/Preferences/$BUNDLE_ID.plist"
  "$HOME/Library/Saved Application State/$BUNDLE_ID.savedState"
  # 開發模式（npm run app:dev）用產品名當目錄名，可能出現這兩種
  "$HOME/Library/Caches/version-manager"
  "$HOME/Library/WebKit/version-manager"
)

if [ "$WITH_CACHES" -eq 1 ]; then
  targets+=(
    "$PROJECT_DIR/src-tauri/target"
    "$PROJECT_DIR/.cargo-home"
    "$PROJECT_DIR/.npm-cache"
    "$PROJECT_DIR/node_modules"
    "$PROJECT_DIR/.chrome-profile"
    "$PROJECT_DIR/dist"
  )
fi

# 只允許刪這兩個範圍，避免任何變數意外展開成空字串或怪路徑
assert_safe() {
  local path="$1"
  [ -n "$path" ] || return 1
  case "$path" in
    "$HOME"/Library/* | "$PROJECT_DIR"/*) return 0 ;;
    *)
      echo "  拒絕刪除非預期路徑：$path"
      return 1
      ;;
  esac
}

human() {
  du -sh "$1" 2>/dev/null | cut -f1
}

echo "專案：$PROJECT_DIR"
echo "模式：$([ "$WITH_CACHES" -eq 1 ] && echo "app + 建置快取" || echo "只移除 app")"
echo
echo "會處理這些路徑："
existing=0
for path in "${targets[@]}"; do
  if [ -e "$path" ]; then
    existing=$((existing + 1))
    printf '  %-8s %s\n' "$(human "$path")" "${path/#$HOME/~}"
  else
    printf '  %-8s %s（不存在）\n' "--" "${path/#$HOME/~}"
  fi
done

if [ "$existing" -eq 0 ]; then
  echo
  echo "沒有找到任何要刪的東西（可能已經清乾淨了）。"
  exit 0
fi

if pgrep -f "$APP/Contents/MacOS/version-manager" >/dev/null 2>&1; then
  echo
  echo "注意：app 正在執行，會先結束它。"
  if [ "$CONFIRM" -eq 1 ]; then
    pkill -f "$APP/Contents/MacOS/version-manager" || true
    sleep 1
  fi
fi

echo
if [ "$CONFIRM" -eq 0 ]; then
  echo "── 這是預覽，什麼都沒有刪。"
  echo "   要真的執行，請加 --yes："
  echo "     bash scripts/uninstall.sh$([ "$WITH_CACHES" -eq 1 ] && echo " --with-caches") --yes"
  exit 0
fi

failed=0
for path in "${targets[@]}"; do
  [ -e "$path" ] || continue
  if ! assert_safe "$path"; then
    failed=1
    continue
  fi
  if rm -rf "$path"; then
    printf '  已刪除  %s\n' "${path/#$HOME/~}"
  else
    printf '  失敗    %s\n' "${path/#$HOME/~}"
    failed=1
  fi
done

# LaunchServices 會留一筆指向已刪 .app 的註記，macOS 自己會慢慢清掉，不影響使用。
echo
echo "提示：LaunchServices 可能還留著一筆已刪除 app 的註記，那是正常現象、不佔空間。"

if [ "$EVERYTHING" -eq 1 ]; then
  echo
  echo "最後一步（腳本本身在專案裡跑，所以留給你執行）："
  echo "  rm -rf \"$PROJECT_DIR\""
fi

echo
if [ "$failed" -eq 0 ]; then
  echo "完成。"
else
  echo "有項目沒刪成功，請看上面的輸出。"
  exit 1
fi
