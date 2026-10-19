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

// 창은 숨긴 채 만들어지고, index.html이 시작 화면을 그리면서 바로 보여 준다. 그게 안 됐을 때를 위한 한 번 더.
getCurrentWindow().show().catch(() => {});

// React가 첫 화면(같은 모양의 StartupSplash)을 그리면 index.html의 시작 화면을 걷는다.
requestAnimationFrame(() => {
  const boot = document.getElementById("boot");
  if (!boot) return;
  boot.style.opacity = "0";
  setTimeout(() => boot.remove(), 300);
});

// 배포판에서는 웹 브라우저 기본 메뉴(뒤로, 새로고침, 검사)를 막는다. 입력칸은 복사·붙여넣기를 위해 둔다.
if (import.meta.env.PROD) {
  window.addEventListener("contextmenu", (e) => {
    const target = e.target as HTMLElement;
    if (!e.defaultPrevented && !target.closest("input, textarea, [data-selectable]")) e.preventDefault();
  });
}
