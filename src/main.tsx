import { createRoot } from "react-dom/client";
import App from "./App";
import "./styles.css";
import { applyThemeMode, readLang, readThemeMode } from "./theme";

const container = document.getElementById("root");
if (!container) throw new Error("找不到 #root");

// 先套主題與語言，避免載入瞬間閃一下淺色
applyThemeMode(readThemeMode());
document.documentElement.lang = readLang();

// 不用 StrictMode：開發模式下它會把 effect 跑兩次，這裡等於整台機器掃兩遍。
createRoot(container).render(<App />);
