#!/usr/bin/env bash
# 提交前檢查：確認沒有把個人資料（主機名、機型、家目錄路徑、截圖）帶進 repo。
#
# 用法：
#   scripts/check-private-data.sh            # 檢查已追蹤檔案（預設）
#   scripts/check-private-data.sh --history  # 另外掃描完整 git 歷史（較慢）
#
# 自訂字串：把不想出現的字串寫進專案根目錄的 .private-patterns（已在 .gitignore，不會被提交），
# 一行一個。這個檔案本身不要提交。
set -uo pipefail

cd "$(dirname "$0")/.." || exit 1

fail=0

# ── 通用樣式（不包含任何人的真實名稱）────────────────────────
patterns=(
  'MacBook (Air|Pro)'
  'Mac[0-9]+,[0-9]+'
  'Macmini[0-9]+,[0-9]+'
  'iMac[0-9]+,[0-9]+'
  '/Users/[A-Za-z0-9._-]+/(Documents|Desktop|Library)'
)

# ── 本機額外樣式（選用、不進版控）────────────────────────────
if [ -f .private-patterns ]; then
  while IFS= read -r line; do
    [ -z "$line" ] && continue
    case "$line" in \#*) continue ;; esac
    patterns+=("$line")
  done < .private-patterns
fi

echo "檢查已追蹤檔案…"
for pattern in "${patterns[@]}"; do
  # -I 跳過二進位檔；-- 之後只掃描版控中的檔案
  hits=$(git grep -I -n -E -- "$pattern" -- . 2>/dev/null || true)
  if [ -n "$hits" ]; then
    echo "✗ 命中樣式「${pattern}」："
    printf '%s\n' "$hits" | sed 's/^/    /'
    fail=1
  fi
done

# ── 圖片：除了 App 圖示與網站 favicon，不該有截圖 ─────────────
images=$(git ls-files | grep -Ei '\.(png|jpe?g|webp|gif)$' \
  | grep -vE '^(src-tauri/icons/|icon[-_][^/]*\.png$)' || true)
if [ -n "$images" ]; then
  echo "✗ 發現非圖示類圖片（截圖常夾帶主機名與套件清單）："
  printf '%s\n' "$images" | sed 's/^/    /'
  fail=1
fi

# ── 選用：掃描歷史 ───────────────────────────────────────────
if [ "${1:-}" = "--history" ]; then
  echo "掃描 git 歷史…"
  for pattern in "${patterns[@]}"; do
    count=$(git log --all -S "$pattern" --oneline 2>/dev/null | wc -l | tr -d ' ')
    if [ "$count" != "0" ]; then
      echo "✗ 歷史中有 ${count} 個 commit 含「${pattern}」"
      git log --all -S "$pattern" --oneline 2>/dev/null | head -5 | sed 's/^/    /'
      fail=1
    fi
  done
fi

if [ "$fail" = "0" ]; then
  echo "✓ 沒有發現個資樣式"
fi
exit "$fail"
