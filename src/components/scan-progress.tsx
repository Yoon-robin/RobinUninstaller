import { useEffect, useState } from "react"
import { Loader2 } from "lucide-react"

import type { ScanProgress } from "@/lib/api"
import { formatBytes } from "@/lib/format"
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

const elapsedText = (s: number) => (s < 60 ? `${s}초` : `${Math.floor(s / 60)}분 ${s % 60}초`)

/**
 * 오래 걸리는 탐색의 진행 상황. 멈춘 것처럼 보이지 않게 지금 보는 곳, 센 파일 수, 흐른 시간을 계속 바꿔 보여 준다.
 * 전체 수를 알면 막대가 차오르고, 모르면 막대가 오간다.
 */
export function ScanProgressView({
  label,
  progress,
  unit = "개",
  hint,
  showBytes = true,
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
  className?: string
}) {
  const elapsed = useElapsed()
  const p = progress
  const percent = p && p.total > 0 ? Math.min(100, Math.round((p.done / p.total) * 100)) : null
  const current = p?.current ? (p.current.includes("\\") ? shortenPath(p.current) : p.current) : null

  return (
    <div className={cn("flex flex-col gap-2.5", className)} role="status" aria-live="polite">
      <div className="flex items-center gap-2 text-sm">
        <Loader2 className="size-4 shrink-0 animate-spin text-primary" />
        <span className="font-medium">{label}</span>
        <span className="ml-auto shrink-0 text-xs text-muted-foreground tabular-nums">
          {p && p.total > 0 && `${p.done.toLocaleString("ko-KR")} / ${p.total.toLocaleString("ko-KR")}${unit} · `}
          {elapsedText(elapsed)}
        </span>
      </div>
      <div className="h-1.5 overflow-hidden rounded-full bg-muted">
        {percent == null ? (
          <div className="scan-sweep h-full w-1/3 rounded-full bg-primary" />
        ) : (
          <div
            className="h-full rounded-full bg-primary transition-[width] duration-300"
            style={{ width: `${Math.max(percent, 3)}%` }}
          />
        )}
      </div>
      <div className="flex min-w-0 items-center gap-3 text-xs text-muted-foreground">
        <span className="min-w-0 flex-1 truncate" title={p?.current}>
          {current ? <>지금 보는 곳: {current}</> : "준비하는 중"}
        </span>
        {p && p.files > 0 && (
          <span className="shrink-0 tabular-nums">
            살펴본 파일 {p.files.toLocaleString("ko-KR")}개{showBytes && ` · ${formatBytes(p.bytes)}`}
          </span>
        )}
      </div>
      {hint && <p className="text-xs text-muted-foreground">{hint}</p>}
    </div>
  )
}
