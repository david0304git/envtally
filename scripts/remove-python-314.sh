#!/usr/bin/env bash
#
# 移除 Python 3.14，把 3.13 設回預設（macOS + python.org 安裝器）
#
#   bash scripts/remove-python-314.sh         # 預覽，什麼都不做
#   bash scripts/remove-python-314.sh --yes   # 執行（會問一次 sudo 密碼）
#
# 只動這幾處：
#   1. /Library/Frameworks/Python.framework/Versions/3.14
#   2. Current 捷徑 → 3.13
#   3. /usr/local/bin 下 3.14 / 3.14t / 3t / 3.10 的捷徑
#   4. python3 / python3-config / python3-intel64 → 3.13
#   5. pkgutil 收據 3.14 與 3.10
# 不會動到：3.13 本身、/usr/bin/python3（Apple 的）、任何 pip 套件。
#
set -uo pipefail

VERSIONS=/Library/Frameworks/Python.framework/Versions
BIN=/usr/local/bin
KEEP=3.13
DROP=3.14
CONFIRM=0
[ "${1:-}" = "--yes" ] && CONFIRM=1

if [ ! -x "$VERSIONS/$KEEP/bin/python3" ]; then
  echo "找不到 $VERSIONS/$KEEP，中止（不會動任何東西）"
  exit 1
fi

echo "保留：$VERSIONS/$KEEP （$("$VERSIONS/$KEEP/bin/python3" -V 2>&1)）"
if [ -d "$VERSIONS/$DROP" ]; then
  echo "移除：$VERSIONS/$DROP （$(du -sh "$VERSIONS/$DROP" 2>/dev/null | cut -f1)）"
else
  echo "移除：$VERSIONS/$DROP （不存在，會跳過）"
fi
echo
echo "預定動作："
echo "  1. 刪除 $VERSIONS/$DROP"
echo "  2. Current → $KEEP"
echo "  3. 移除 $BIN 下 3.14 / 3.14t / 3t / 3.10 的捷徑"
echo "  4. python3 / python3-config / python3-intel64 → $KEEP"
echo "  5. pkgutil --forget 3.14 與 3.10 的收據"
echo

if [ "$CONFIRM" -eq 0 ]; then
  echo "── 這是預覽，什麼都沒有動。"
  echo "   要真的執行：bash scripts/remove-python-314.sh --yes"
  exit 0
fi

echo "需要 sudo 權限（輸入一次密碼即可）"
sudo -v || {
  echo "拿不到 sudo 權限，中止"
  exit 1
}

sudo rm -rf "$VERSIONS/$DROP"
sudo ln -sfn "$KEEP" "$VERSIONS/Current"

for name in "python$DROP" "python$DROP-config" "python$DROP-intel64" \
            "python${DROP}t" "python${DROP}t-config" "python${DROP}t-intel64" \
            python3t python3t-config python3t-intel64 \
            python3.10 python3.10-config python3.10-intel64; do
  sudo rm -f "$BIN/$name"
done

for name in python3 python3-config python3-intel64; do
  if [ -e "$VERSIONS/$KEEP/bin/$name" ]; then
    sudo ln -sfn "../../../Library/Frameworks/Python.framework/Versions/$KEEP/bin/$name" "$BIN/$name"
  fi
done

for pkg in "PythonFramework-$DROP" "PythonDocumentation-$DROP" \
           "PythonApplications-$DROP" "PythonUnixTools-$DROP" \
           PythonFramework-3.10 PythonDocumentation-3.10 \
           PythonApplications-3.10 PythonUnixTools-3.10; do
  sudo pkgutil --forget "org.python.Python.$pkg" >/dev/null 2>&1 && echo "  收據已移除：$pkg"
done

echo
echo "=== 驗證 ==="
ls -l "$VERSIONS"
echo "Current → $(readlink "$VERSIONS/Current")"
echo "python3: $(which python3) （$(python3 -V 2>&1)）"
python3 -m pip -V 2>&1 | head -1 | sed 's/^/pip: /'
echo "斷連結檢查（應該沒有輸出）："
for f in "$BIN"/python3* "$HOME"/.local/bin/python3*; do
  [ -L "$f" ] && [ ! -e "$f" ] && echo "  $f -> $(readlink "$f")"
done
echo
echo "完成。App 重新掃描後應該只剩 3.13.6 與 3.9.6（Apple）兩個直譯器。"
