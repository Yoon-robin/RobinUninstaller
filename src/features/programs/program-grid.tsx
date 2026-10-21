import { memo, useLayoutEffect, useRef, useState, type CSSProperties } from "react"
import { useVirtualizer } from "@tanstack/react-virtual"
import { TriangleAlert } from "lucide-react"

import { needsForce, type InstalledApplication } from "@/lib/api"
import { formatSize } from "@/lib/format"
import { cn } from "@/lib/utils"
import { Highlight } from "@/components/highlight"
import { AppContextMenu, type AppActionHandlers } from "./app-actions"
import { AppIcon } from "./app-icon"

/** 카드 최소 폭(12.5rem)·간격(gap-3)·안쪽 여백(p-4). 열 수를 직접 계산해 줄 단위로 가상화한다. */
const CARD_MIN = 200
const GAP = 12
const PAD = 16
/** 카드 한 줄 높이 추정 (실제 높이는 그린 뒤 잰다) */
const ROW_ESTIMATE = 178

/**
 * 카드 보기. 보이는 줄(과 앞뒤 몇 줄)만 그린다 — 1000개를 다 그리면 보기를 바꾸는 데 4초 걸렸다.
 * 처음 화면이 뜰 때만 앞쪽 카드가 차례로 등장한다.
 */
export function ProgramGrid({
  apps,
  active,
  intro,
  query,
  removingId,
  ...handlers
}: {
  apps: InstalledApplication[]
  /** 프로그램 화면이 보이는지 (숨어 있으면 가상 목록을 멈춘다) */
  active: boolean
  intro: boolean
  query: string
  removingId: string | null
} & AppActionHandlers) {
  const ref = useRef<HTMLDivElement>(null)
  const [width, setWidth] = useState(0)
  const [scrollMargin, setScrollMargin] = useState(0)
  // 너비가 바뀌면 열 수를, 위치가 바뀌면 스크롤 기준(목록이 <main> 안에서 얼마나 아래에 있는지)을 다시 잰다
  useLayoutEffect(() => {
    const el = ref.current
    const main = el?.closest("main")
    if (!el || !main) return
    const measure = () => {
      if (!el.offsetParent) return
      setWidth(el.clientWidth)
      setScrollMargin(Math.round(el.getBoundingClientRect().top - main.getBoundingClientRect().top + main.scrollTop))
    }
    measure()
    const observer = new ResizeObserver(measure)
    observer.observe(el)
    return () => observer.disconnect()
  }, [])

  const cols = Math.max(1, Math.floor((width - PAD * 2 + GAP) / (CARD_MIN + GAP)))
  const rows = Math.ceil(apps.length / cols)
  const virtualizer = useVirtualizer({
    count: rows,
    getScrollElement: () => ref.current?.closest("main") ?? null,
    estimateSize: () => ROW_ESTIMATE,
    overscan: 3,
    scrollMargin,
    enabled: active && width > 0,
    measureElement: (el) => el.getBoundingClientRect().height || ROW_ESTIMATE,
  })

  return (
    <div
      ref={ref}
      className="relative"
      style={{ height: width > 0 ? virtualizer.getTotalSize() + PAD * 2 : undefined }}
    >
      {virtualizer.getVirtualItems().map((row) => {
        const start = row.index * cols
        return (
          <div
            key={row.key}
            data-index={row.index}
            ref={virtualizer.measureElement}
            className="absolute inset-x-0 top-0 grid gap-3 px-4 pb-3"
            style={{
              gridTemplateColumns: `repeat(${cols}, minmax(0, 1fr))`,
              transform: `translateY(${row.start - scrollMargin + PAD}px)`,
            }}
          >
            {apps.slice(start, start + cols).map((app, i) => (
              <GridCard
                key={app.id}
                app={app}
                query={query}
                removing={app.id === removingId}
                // 처음 등장은 앞 두 줄까지만
                introIndex={intro && row.index < 2 ? start + i : undefined}
                {...handlers}
              />
            ))}
          </div>
        )
      })}
    </div>
  )
}

type CardProps = {
  app: InstalledApplication
  query: string
  removing: boolean
  introIndex?: number
} & AppActionHandlers

const GridCard = memo(function GridCard({ app, query, removing, introIndex, ...handlers }: CardProps) {
  return (
    <div className={cn("transition-opacity duration-150", removing && "opacity-40")}>
      <AppContextMenu app={app} {...handlers}>
        <button
          type="button"
          onClick={() => handlers.onSelect(app)}
          style={introIndex == null ? undefined : ({ "--i": introIndex } as CSSProperties)}
          className={cn(
            "group relative flex h-full w-full cursor-pointer flex-col items-start gap-3 rounded-xl border bg-background/40 p-4 text-left",
            "transition-[transform,border-color,background-color] duration-150 ease-(--ease-out-quint)",
            "hover:-translate-y-0.5 hover:border-foreground/15 hover:bg-accent/30 active:translate-y-0 active:scale-[0.98] motion-reduce:hover:translate-y-0 motion-reduce:active:scale-100",
            "focus-visible:ring-2 focus-visible:ring-ring focus-visible:outline-none data-[state=open]:border-foreground/15",
            introIndex != null && "row-intro",
          )}
        >
          {needsForce(app) && (
            <TriangleAlert className="absolute top-3 right-3 size-4 text-warning" aria-label="언인스톨러 없음" />
          )}
          <AppIcon app={app} className="size-12 rounded-xl text-lg" />
          <div className="min-w-0 self-stretch">
            <p className="line-clamp-2 leading-snug font-medium" title={app.displayName}>
              <Highlight text={app.displayName} query={query} />
            </p>
            <p className="mt-0.5 truncate text-xs text-muted-foreground">
              {app.publisher ? <Highlight text={app.publisher} query={query} /> : "게시자 정보 없음"}
            </p>
          </div>
          <div className="mt-auto flex w-full items-center justify-between gap-2 text-xs">
            <span
              className="text-muted-foreground tabular-nums"
              title={app.sizeMeasured ? "크기 기록이 없어 설치 폴더를 직접 잰 크기예요" : undefined}
            >
              {formatSize(app.estimatedSizeKb)}
              {app.sizeMeasured && "*"}
            </span>
            <span className="flex gap-1">
              <span className="rounded-md border px-1.5 py-0.5 font-mono text-[11px] font-medium text-muted-foreground">
                {app.technology.toUpperCase()}
              </span>
              {app.scope === "user" && (
                <span className="rounded-md border border-primary/30 px-1.5 py-0.5 text-[11px] text-primary">
                  사용자
                </span>
              )}
            </span>
          </div>
        </button>
      </AppContextMenu>
    </div>
  )
})
