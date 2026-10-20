import { useEffect, useState } from "react"
import { getCurrentWindow } from "@tauri-apps/api/window"

import { cn } from "@/lib/utils"

/**
 * 제목 표시줄을 없앤 창(decorations: false)의 최소화/최대화/닫기 버튼.
 * 자리와 순서는 Windows 사용자에게 익숙하게 오른쪽 위에 두고, 모양은 macOS처럼 둥근 유리 버튼으로 만든다.
 */
export function WindowControls() {
  const [appWindow] = useState(getCurrentWindow)
  const [maximized, setMaximized] = useState(false)

  useEffect(() => {
    const check = () =>
      appWindow
        .isMaximized()
        .then(setMaximized)
        .catch(() => {})
    check()
    const unlisten = appWindow.onResized(check)
    return () => {
      unlisten.then((fn) => fn()).catch(() => {})
    }
  }, [appWindow])

  return (
    <div className="flex items-center gap-1 pr-2.5">
      <CaptionButton label="최소화" onClick={() => appWindow.minimize()}>
        <path d="M0 5h10" />
      </CaptionButton>
      <CaptionButton label={maximized ? "이전 크기로" : "최대화"} onClick={() => appWindow.toggleMaximize()}>
        {maximized ? (
          <>
            <path d="M2.5 2.5V0.5h7v7h-2" />
            <rect x="0.5" y="2.5" width="7" height="7" />
          </>
        ) : (
          <rect x="0.5" y="0.5" width="9" height="9" />
        )}
      </CaptionButton>
      <CaptionButton label="닫기" onClick={() => appWindow.close()} danger>
        <path d="M0.5 0.5l9 9M9.5 0.5l-9 9" />
      </CaptionButton>
    </div>
  )
}

function CaptionButton({
  label,
  onClick,
  danger,
  children,
}: {
  label: string
  onClick: () => void
  danger?: boolean
  children: React.ReactNode
}) {
  return (
    <button
      type="button"
      aria-label={label}
      title={label}
      onClick={onClick}
      className={cn(
        "flex size-7 items-center justify-center rounded-full text-muted-foreground transition-[color,background-color,box-shadow,transform] duration-150",
        "hover:text-foreground active:scale-90",
        danger ? "hover:bg-[#ff5f57] hover:text-white hover:shadow-none" : "hover:bg-foreground/8",
      )}
    >
      <svg viewBox="0 0 10 10" className="size-2.5" fill="none" stroke="currentColor" strokeWidth="1">
        {children}
      </svg>
    </button>
  )
}
