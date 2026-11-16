import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

// Tauri 需要固定 port，且失敗時不要自動換 port（否則 webview 連不到）
export default defineConfig({
  plugins: [react()],
  clearScreen: false,
  server: {
    port: 5173,
    strictPort: true,
    host: "127.0.0.1",
  },
  build: {
    target: "safari15",
    sourcemap: false,
  },
});
