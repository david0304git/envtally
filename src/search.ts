/**
 * 搜尋比對。
 *
 * 使用者常常記不住套件的實際拼法（例如 `tripo-cli` 會打成 `TripoCLI`），
 * 所以除了原始字串的包含比對，再加一種「去掉分隔符」的比對：
 * 兩邊都去掉空白、連字號、底線、句點等，再做包含比對。
 */
export function squash(text: string): string {
  return text.toLowerCase().replace(/[\s\-_.·/\\()（）[\]{}]+/g, "");
}

/** 只要任何一個欄位命中就回傳 true；query 為空時一律 true */
export function matchesQuery(
  query: string,
  fields: Array<string | null | undefined>,
): boolean {
  const raw = query.trim().toLowerCase();
  if (!raw) {
    return true;
  }
  const squashed = squash(raw);
  return fields.some((field) => {
    if (!field) {
      return false;
    }
    const text = field.toLowerCase();
    return text.includes(raw) || squash(text).includes(squashed);
  });
}
