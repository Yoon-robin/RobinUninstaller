import { forwardRef, type CSSProperties } from "react"
import { Check, MoreHorizontal, TriangleAlert } from "lucide-react"
import { motion } from "motion/react"

import { needsForce, type InstalledApplication } from "@/lib/api"
import { formatInstallDate, formatSize } from "@/lib/format"
import { cn } from "@/lib/utils"
import { Highlight } from "@/components/highlight"
import { Button } from "@/components/ui/button"
import { DropdownMenu, DropdownMenuTrigger } from "@/components/ui/dropdown-menu"
import { AppContextMenu, AppDropdownContent, type AppActionHandlers } from "./app-actions"
import { AppIcon } from "./app-icon"

/** 목록 열. 좁은 창에서는 설치일·형식을 숨겨 이름 칸이 찌그러지지 않게 한다 (숨긴 칸에는 `WIDE_ONLY`). */
export const LIST_GRID =
  "grid grid-cols-[minmax(0,1fr)_minmax(0,8rem)_6rem_2rem] items-center gap-4 lg:grid-cols-[minmax(0,1fr)_minmax(0,9rem)_6.5rem_5.75rem_5.5rem_2rem]"
export const WIDE_ONLY = "hidden lg:flex"

/** 제거된 항목이 목록에서 접히며 사라지는 모션. 검색·필터로 빠질 때는 쓰지 않는다. */
export const removedExit = {
  opacity: 0,
  height: 0,
  transition: { duration: 0.35, ease: [0.22, 1, 0.36, 1] as const },
}

type Props = {
  app: InstalledApplication
  maxKb: number
  query: string
  removing: boolean
  /** 첫 등장 애니메이션 순서. undefined면 애니메이션 없음. */
  introIndex?: number
  /** 여러 개 고르는 중이면 선택 상태 (undefined면 고르는 중이 아님). 이때 행을 누르면 열지 않고 선택을 바꾼다. */
  picked?: boolean
  onPick?: (app: InstalledApplication) => void
} & AppActionHandlers

export const ProgramRow = forwardRef<HTMLDivElement, Props>(function ProgramRow(
  { app, maxKb, query, removing, introIndex, picked, onPick, ...handlers },
  ref,
) {
  // 큰 프로그램 몇 개가 막대를 독차지하지 않도록 제곱근 비율로 그린다.
  const sizeRatio = app.estimatedSizeKb ? Math.max(0.03, Math.sqrt(app.estimatedSizeKb / maxKb)) : 0
  const introStyle = introIndex == null ? undefined : ({ "--i": introIndex } as CSSProperties)

  return (
    <motion.div ref={ref} exit={removing ? removedExit : undefined} className="overflow-hidden">
      <AppContextMenu app={app} {...handlers}>
        <div
          // 여러 개 제거 중에는 체크 상자처럼, 평소에는 세부 정보를 여는 버튼처럼 읽힌다
          role={picked == null ? "button" : "checkbox"}
          tabIndex={0}
          aria-checked={picked == null ? undefined : picked}
          aria-label={
            picked == null
              ? `${app.displayName}${app.publisher ? `, ${app.publisher}` : ""} 세부 정보 보기`
              : `${app.displayName} 제거 대상으로 고르기`
          }
          onClick={() => (picked == null ? handlers.onSelect(app) : onPick?.(app))}
          onKeyDown={(e) => {
            // 안쪽 버튼·메뉴에서 누른 키는 행 열기로 처리하지 않는다 (메뉴는 포털이라 이벤트가 여기까지 올라온다)
            if (e.target !== e.currentTarget) return
            if (e.key === "Enter" || e.key === " ") {
              e.preventDefault()
              if (picked == null) handlers.onSelect(app)
              else onPick?.(app)
            }
          }}
          style={introStyle}
          className={cn(
            LIST_GRID,
            introIndex != null && "row-intro",
            "group cursor-pointer px-5 py-2.5 text-sm transition-colors outline-none",
            "hover:bg-accent/40 focus-visible:bg-accent/50 data-[state=open]:bg-accent/40",
            picked && "bg-primary/8 hover:bg-primary/12",
          )}
        >
          <div className="flex min-w-0 items-center gap-3">
            {picked != null && (
              <span
                aria-hidden
                className={cn(
                  "flex size-4 shrink-0 items-center justify-center rounded-[4px] border transition-colors",
                  picked ? "border-primary bg-primary text-primary-foreground" : "border-input",
                )}
              >
                {picked && <Check className="size-3" strokeWidth={3} />}
              </span>
            )}
            <AppIcon app={app} />
            <div className="min-w-0">
              <div className="flex items-center gap-2">
                <p className="truncate font-medium" title={app.displayName}>
                  <Highlight text={app.displayName} query={query} />
                </p>
                {needsForce(app) && (
                  <TriangleAlert className="size-3.5 shrink-0 text-chart-3" aria-label="언인스톨러 없음" />
                )}
              </div>
              <p className="truncate text-xs text-muted-foreground">{app.version ?? "버전 정보 없음"}</p>
            </div>
          </div>

          <span className="truncate text-muted-foreground" title={app.publisher ?? undefined}>
            {app.publisher ? <Highlight text={app.publisher} query={query} /> : "—"}
          </span>

          <div className="flex flex-col gap-1">
            <span
              className="tabular-nums"
              title={app.sizeMeasured ? "크기 기록이 없어 설치 폴더를 직접 잰 크기예요" : undefined}
            >
              {formatSize(app.estimatedSizeKb)}
              {app.sizeMeasured && <span className="ml-0.5 text-muted-foreground">*</span>}
            </span>
            {sizeRatio > 0 && (
              <div className="h-1 w-full overflow-hidden rounded-full bg-muted">
                <div
                  className={cn("h-full rounded-full bg-chart-2/70", introIndex != null && "bar-intro")}
                  style={{ width: `${sizeRatio * 100}%` }}
                />
              </div>
            )}
          </div>

          <span className={cn(WIDE_ONLY, "text-muted-foreground tabular-nums")}>
            {formatInstallDate(app.installDate)}
          </span>

          <div className={cn(WIDE_ONLY, "gap-1")}>
            <span className="rounded-md border px-1.5 py-0.5 font-mono text-[10px] font-medium text-muted-foreground">
              {app.technology.toUpperCase()}
            </span>
            {app.scope === "machine32" && (
              <span className="rounded-md border px-1.5 py-0.5 font-mono text-[10px] text-muted-foreground">x86</span>
            )}
            {app.scope === "user" && (
              <span className="rounded-md border border-primary/30 px-1.5 py-0.5 text-[10px] text-primary">사용자</span>
            )}
          </div>

          <div
            onClick={(e) => e.stopPropagation()}
            onContextMenu={(e) => e.stopPropagation()}
            onKeyDown={(e) => e.stopPropagation()}
          >
            <DropdownMenu>
              <DropdownMenuTrigger asChild>
                <Button
                  variant="ghost"
                  size="icon"
                  className="size-8 opacity-0 group-hover:opacity-100 group-focus-visible:opacity-100 focus-visible:opacity-100 data-[state=open]:opacity-100"
                  aria-label="작업"
                >
                  <MoreHorizontal />
                </Button>
              </DropdownMenuTrigger>
              <AppDropdownContent app={app} {...handlers} />
            </DropdownMenu>
          </div>
        </div>
      </AppContextMenu>
    </motion.div>
  )
})
