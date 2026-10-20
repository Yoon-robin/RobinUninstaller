import { forwardRef, useEffect, useState, type CSSProperties } from "react"
import { TriangleAlert } from "lucide-react"
import { AnimatePresence, motion } from "motion/react"

import { needsForce, type InstalledApplication } from "@/lib/api"
import { formatSize } from "@/lib/format"
import { cn } from "@/lib/utils"
import { Highlight } from "@/components/highlight"
import { AppContextMenu, type AppActionHandlers } from "./app-actions"
import { AppIcon } from "./app-icon"

/** 카드 보기. 처음 나타날 때(보기 전환 직후)만 앞쪽 카드가 차례로 등장한다. */
export function ProgramGrid({
  apps,
  intro,
  query,
  removingId,
  ...handlers
}: {
  apps: InstalledApplication[]
  intro: boolean
  query: string
  removingId: string | null
} & AppActionHandlers) {
  const [justMounted, setJustMounted] = useState(true)
  useEffect(() => {
    const t = setTimeout(() => setJustMounted(false), 900)
    return () => clearTimeout(t)
  }, [])
  const animate = intro || justMounted

  return (
    <div className="grid grid-cols-[repeat(auto-fill,minmax(12.5rem,1fr))] gap-3 p-4">
      <AnimatePresence initial={false}>
        {apps.map((app, i) => (
          <GridCard
            key={app.id}
            app={app}
            query={query}
            removing={app.id === removingId}
            introIndex={animate && i < 15 ? i : undefined}
            {...handlers}
          />
        ))}
      </AnimatePresence>
    </div>
  )
}

type CardProps = {
  app: InstalledApplication
  query: string
  removing: boolean
  introIndex?: number
} & AppActionHandlers

const GridCard = forwardRef<HTMLDivElement, CardProps>(function GridCard(
  { app, query, removing, introIndex, ...handlers },
  ref,
) {
  return (
    <motion.div
      ref={ref}
      exit={
        removing ? { opacity: 0, scale: 0.94, transition: { duration: 0.25, ease: [0.22, 1, 0.36, 1] } } : undefined
      }
    >
      <AppContextMenu app={app} {...handlers}>
        <button
          type="button"
          onClick={() => handlers.onSelect(app)}
          style={introIndex == null ? undefined : ({ "--i": introIndex } as CSSProperties)}
          className={cn(
            "group relative flex h-full w-full flex-col items-start gap-3 rounded-xl border bg-background/40 p-4 text-left",
            "transition-[transform,border-color,background-color,box-shadow] duration-200 ease-(--ease-out-quint)",
            "hover:-translate-y-0.5 hover:border-foreground/15 hover:bg-accent/30 hover:shadow-md active:translate-y-0 active:scale-[0.98]",
            "focus-visible:ring-2 focus-visible:ring-ring focus-visible:outline-none data-[state=open]:border-foreground/15",
            introIndex != null && "row-intro",
          )}
        >
          {needsForce(app) && (
            <TriangleAlert className="absolute top-3 right-3 size-4 text-chart-3" aria-label="언인스톨러 없음" />
          )}
          <AppIcon
            app={app}
            className="size-12 rounded-xl text-lg transition-transform duration-300 ease-(--ease-out-quint) group-hover:scale-105"
          />
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
              <span className="rounded-md border px-1.5 py-0.5 font-mono text-[10px] font-medium text-muted-foreground">
                {app.technology.toUpperCase()}
              </span>
              {app.scope === "user" && (
                <span className="rounded-md border border-primary/30 px-1.5 py-0.5 text-[10px] text-primary">
                  사용자
                </span>
              )}
            </span>
          </div>
        </button>
      </AppContextMenu>
    </motion.div>
  )
})
