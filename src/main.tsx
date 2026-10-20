import React from "react";
import ReactDOM from "react-dom/client";
import { invoke } from "@tauri-apps/api/core";
import App from "./App";
import { SetupApp, type SetupMode } from "./features/setup/setup-app";
import { beginStartupUpdateCheck } from "./features/updates/updates";
import "./index.css";
import { getCurrentWindow } from "@tauri-apps/api/window";

// setup.exe로 실행됐으면(= robinx.exe 자체 설치) 설치·제거 화면을, 아니면 앱을 그린다. 브라우저(화면 캡처)에서는 앱.
invoke<"app" | SetupMode>("app_mode")
  .catch(() => "app" as const)
  .then((mode) => {
    if (mode === "app") beginStartupUpdateCheck();
    ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
      <React.StrictMode>{mode === "app" ? <App /> : <SetupApp mode={mode} />}</React.StrictMode>,
    );
    hideBoot();
  });

// 창은 숨긴 채 만들어지고, index.html이 시작 화면을 그리면서 바로 보여 준다. 그게 안 됐을 때를 위한 한 번 더.
getCurrentWindow().show().catch(() => {});

// React가 첫 화면(같은 모양의 StartupSplash 또는 설치 화면)을 그리면 index.html의 시작 화면을 걷는다.
// 창 열림 애니메이션(index.html)이 끝난 뒤에 걷어야 커지던 판이 중간에 사라지지 않는다.
function hideBoot() {
  const w = window as Window & { __bootOpened?: boolean };
  if (!w.__bootOpened) {
    window.addEventListener("robin:boot-opened", hideBoot, { once: true });
    return;
  }
  requestAnimationFrame(() => {
    const boot = document.getElementById("boot");
    if (!boot) return;
    boot.style.opacity = "0";
    setTimeout(() => boot.remove(), 300);
  });
}

// 배포판에서는 웹 브라우저 기본 메뉴(뒤로, 새로고침, 검사)를 막는다. 입력칸은 복사·붙여넣기를 위해 둔다.
if (import.meta.env.PROD) {
  window.addEventListener("contextmenu", (e) => {
    const target = e.target as HTMLElement;
    if (!e.defaultPrevented && !target.closest("input, textarea, [data-selectable]")) e.preventDefault();
  });
}
