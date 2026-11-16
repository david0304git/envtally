import { createRoot } from "react-dom/client";
import App from "./App";
import "./styles.css";

const container = document.getElementById("root");
if (!container) throw new Error("找不到 #root");

// 不用 StrictMode：開發模式下它會把 effect 跑兩次，這裡等於整台機器掃兩遍。
createRoot(container).render(<App />);
