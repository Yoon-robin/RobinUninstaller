import { useEffect, useState } from "react"
import { Loader2 } from "lucide-react"

import type { ScanProgress } from "@/lib/api"
import { formatBytes, formatCount, formatDuration } from "@/lib/format"
import { cn } from "@/lib/utils"

/** 긴 경로는 끝의 폴더 몇 개만 보여 준다 (어디를 보고 있는지는 끝부분이 알려 준다) */
export function shortenPath(path: string, keep = 3) {
  const parts = path.split("\\").filter(Boolean)
  return parts.length > keep + 1 ? `${parts[0]}\\…\\${parts.slice(-keep).join("\\")}` : path
}

/** 마운트된 뒤 흐른 초 (1초마다) */
function useElapsed() {
  const [seconds, setSeconds] = useState(0)
  useEffect(() => {
    const started = Date.now()
    const t = setInterval(() => setSeconds(Math.floor((Date.now() - started) / 1000)), 1000)
    return () => clearInterval(t)
  }, [])
  return seconds
}

/**
 * 오래 걸리는 탐색의 진행 상황. 멈춘 것처럼 보이지 않게 지금 보는 곳, 센 파일 수, 흐른 시간을 계속 바꿔 보여 준다.
 * 전체 수를 알면 막대가 차오르고, 모르면 막대가 오간다.
 */
export function ScanProgressView({
  label,
  progress,
  unit = "개",
  hint,
  showBytes,
  work = true,
  className,
}: {
  /** 무엇을 하는 중인지 ("정리할 수 있는 항목을 찾는 중이에요") */
  label: string
  progress: ScanProgress | null
  /** done/total의 단위 */
  unit?: string
  /** 오래 걸릴 수 있다는 안내 등 */
  hint?: string
  /** 센 크기도 보여 줄지 (저장공간 찾기처럼 정리할 크기와 헷갈릴 수 있으면 끈다) */
  showBytes?: boolean
  /** 찾기가 아니라 옮기기·되돌리기·지우기 중이면 true (문구가 "다루는 곳 / 처리한 파일"로 바뀐다) */
  work?: boolean
  className?: string
}) {
  const elapsed = useElapsed()
  const p = progress
  const percent = p && p.total > 0 ? Math.min(100, Math.round((p.done / p.total) * 100)) : null
  const current = p?.current ? (p.current.includes("\\") ? shortenPath(p.current) : p.current) : null

  return (
    // 읽어 주는 것은 단계 이름뿐이다. 1초마다 바뀌는 시간·개수·경로까지 알리면 화면 읽기 프로그램이 쉬지 않고 읽는다.
    <div className={cn("flex flex-col gap-2.5", className)}>
      <div className="flex items-center gap-2 text-sm">
        <Loader2 className="size-4 shrink-0 animate-spin text-primary" />
        <span className="font-medium" role="status">
          {label}
        </span>
        <span className="ml-auto shrink-0 text-xs text-muted-foreground tabular-nums">
          {p && p.total > 0 && `${formatCount(p.done)} / ${formatCount(p.total)}${unit} · `}
          {formatDuration(elapsed * 1000)}
        </span>
      </div>
      <div className="h-1.5 overflow-hidden rounded-full bg-muted">
        {percent == null ? (
          <div className="scan-sweep h-full w-1/3 rounded-full bg-primary" />
        ) : (
          <div
            className="forced-fill h-full w-full origin-left rounded-full bg-primary transition-transform duration-200 ease-(--ease-out-quint)"
            style={{ transform: `scaleX(${Math.max(percent, 3) / 100})` }}
          />
        )}
      </div>
      <div className="flex min-w-0 items-center gap-3 text-xs text-muted-foreground">
        <span className="min-w-0 flex-1 truncate" title={p?.current}>
          {current ? (
            <>
              {work ? "지금 다루는 곳" : "지금 보는 곳"}: {current}
            </>
          ) : (
            "준비하는 중"
          )}
        </span>
        {p && p.files > 0 && (
          <span className="shrink-0 tabular-nums">
            {work ? "처리한 파일" : "살펴본 파일"} {formatCount(p.files)}개{showBytes && ` · ${formatBytes(p.bytes)}`}
          </span>
        )}
      </div>
      {hint && <p className="text-xs text-muted-foreground">{hint}</p>}
    </div>
  )
}
