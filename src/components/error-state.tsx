import { RefreshCw, TriangleAlert } from "lucide-react"

import { cn } from "@/lib/utils"
import { Button } from "@/components/ui/button"

/**
 * 읽기·찾기에 실패했을 때 공통 화면. "없음"과 구별되게 무엇이 실패했는지, 원인 문구, 다시 시도 버튼을 보여 준다.
 * 원인 문구는 복사할 수 있게 둔다(문제를 알려 줄 때 쓰도록).
 */
export function ErrorState({
  title,
  error,
  hint = "잠시 뒤 다시 시도해 보세요. 계속되면 아래 내용을 알려 주세요.",
  onRetry,
  retrying,
  className,
}: {
  title: string
  error: unknown
  hint?: string
  onRetry?: () => void
  retrying?: boolean
  className?: string
}) {
  return (
    <div role="alert" className={cn("flex flex-col items-center gap-3 px-6 py-12 text-center", className)}>
      <span className="flex size-11 items-center justify-center rounded-2xl bg-destructive/10 text-destructive">
        <TriangleAlert className="size-5" />
      </span>
      <p className="font-medium">{title}</p>
      <p className="max-w-md text-sm text-muted-foreground">{hint}</p>
      <p data-selectable className="max-w-md font-mono text-xs break-all text-muted-foreground">
        {String(error)}
      </p>
      {onRetry && (
        <Button variant="outline" size="sm" onClick={onRetry} disabled={retrying}>
          <RefreshCw className={cn(retrying && "animate-spin")} />
          다시 시도
        </Button>
      )}
    </div>
  )
}

/** 설정 구역처럼 좁은 자리에 쓰는 한 줄 오류. 읽지 못한 것을 "없음"으로 보이지 않게 한다. */
export function InlineError({
  message,
  error,
  onRetry,
  retrying,
  className,
}: {
  message: string
  error?: unknown
  onRetry?: () => void
  retrying?: boolean
  className?: string
}) {
  return (
    <div role="alert" className={cn("flex flex-wrap items-center gap-2 text-sm", className)}>
      <TriangleAlert className="size-4 shrink-0 text-destructive" />
      <span className="min-w-0 flex-1">
        {message}
        {error != null && (
          <span data-selectable className="ml-1 text-xs text-muted-foreground">
            ({String(error)})
          </span>
        )}
      </span>
      {onRetry && (
        <Button variant="outline" size="sm" onClick={onRetry} disabled={retrying}>
          <RefreshCw className={cn(retrying && "animate-spin")} />
          다시 시도
        </Button>
      )}
    </div>
  )
}

/** 이미 보여 준 목록을 새로 읽지 못했을 때: 목록은 그대로 두고 위에 띠만 둔다. */
export function StaleBanner({
  error,
  onRetry,
  retrying,
  className,
}: {
  error: unknown
  onRetry: () => void
  retrying?: boolean
  className?: string
}) {
  return (
    <div
      role="status"
      className={cn(
        "flex flex-wrap items-center gap-3 border-b border-chart-3/40 bg-chart-3/8 px-5 py-2.5 text-sm",
        className,
      )}
    >
      <TriangleAlert className="size-4 shrink-0 text-warning" />
      <span className="min-w-0 flex-1">
        새로 읽지 못해 마지막으로 읽은 목록을 보여 주고 있어요.
        <span data-selectable className="ml-1 text-xs text-muted-foreground">
          ({String(error)})
        </span>
      </span>
      <Button variant="outline" size="sm" onClick={onRetry} disabled={retrying}>
        <RefreshCw className={cn(retrying && "animate-spin")} />
        다시 시도
      </Button>
    </div>
  )
}
