import { useEffect, useState } from "react"
import { getCurrentWindow } from "@tauri-apps/api/window"

import { cn } from "@/lib/utils"
import { IconTooltip } from "@/components/icon-tooltip"

/**
 * 제목 표시줄을 없앤 창(decorations: false)의 최소화/최대화/닫기 버튼.
 * Windows 창 버튼처럼 제목 줄 높이를 다 채우고 오른쪽 모서리까지 붙인다. 최대화한 창에서 마우스를
 * 오른쪽 위 끝으로 밀어 두고 눌러도 닫히고, 닫기 위에서는 Windows와 같은 빨강이 된다.
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
    <div className="flex h-full items-stretch">
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
    <IconTooltip label={label}>
      <button
        type="button"
        aria-label={label}
        onClick={onClick}
        className={cn(
          "flex w-[46px] items-center justify-center text-muted-foreground transition-colors duration-150",
          "hover:text-foreground",
          // 흰 아이콘과 대비 5.6:1 (Windows 닫기 빨강)
          danger
            ? "hover:bg-[#c42b1c] hover:text-white active:bg-[#c42b1c]/85"
            : "hover:bg-foreground/8 active:bg-foreground/12",
        )}
      >
        <svg viewBox="0 0 10 10" className="size-2.5" fill="none" stroke="currentColor" strokeWidth="1">
          {children}
        </svg>
      </button>
    </IconTooltip>
  )
}
