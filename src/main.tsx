import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";
import "./index.css";
import { getCurrentWindow } from "@tauri-apps/api/window";

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>,
);

// 창은 숨긴 채 만들어진다(tauri.conf.json). 첫 화면을 그린 뒤에 보여 줘서 흰 화면 깜빡임을 없앤다.
requestAnimationFrame(() => requestAnimationFrame(() => getCurrentWindow().show()));

// 배포판에서는 웹 브라우저 기본 메뉴(뒤로, 새로고침, 검사)를 막는다. 입력칸은 복사·붙여넣기를 위해 둔다.
if (import.meta.env.PROD) {
  window.addEventListener("contextmenu", (e) => {
    const target = e.target as HTMLElement;
    if (!e.defaultPrevented && !target.closest("input, textarea, [data-selectable]")) e.preventDefault();
  });
}
