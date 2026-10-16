import { useEffect, useState, type CSSProperties } from "react"
import { TriangleAlert } from "lucide-react"

import type { InstalledApplication } from "@/lib/api"
import { formatSize } from "@/lib/format"
import { cn } from "@/lib/utils"
import { AppIcon } from "./app-icon"

/** 카드 보기. 처음 나타날 때(보기 전환 직후)만 앞쪽 카드가 차례로 등장한다. */
export function ProgramGrid({
  apps,
  onSelect,
  intro,
}: {
  apps: InstalledApplication[]
  onSelect: (app: InstalledApplication) => void
  intro: boolean
}) {
  const [justMounted, setJustMounted] = useState(true)
  useEffect(() => {
    const t = setTimeout(() => setJustMounted(false), 900)
    return () => clearTimeout(t)
  }, [])
  const animate = intro || justMounted

  return (
    <div className="grid grid-cols-[repeat(auto-fill,minmax(12.5rem,1fr))] gap-3 p-4">
      {apps.map((app, i) => (
        <GridCard key={app.id} app={app} onSelect={onSelect} introIndex={animate && i < 15 ? i : undefined} />
      ))}
    </div>
  )
}

function GridCard({
  app,
  onSelect,
  introIndex,
}: {
  app: InstalledApplication
  onSelect: (app: InstalledApplication) => void
  introIndex?: number
}) {
  return (
    <button
      type="button"
      onClick={() => onSelect(app)}
      style={introIndex == null ? undefined : ({ "--i": introIndex } as CSSProperties)}
      className={cn(
        "group relative flex flex-col items-start gap-3 rounded-xl border bg-background/40 p-4 text-left",
        "transition-[transform,border-color,background-color,box-shadow] duration-200 ease-(--ease-out-quint)",
        "hover:-translate-y-0.5 hover:border-foreground/15 hover:bg-accent/30 hover:shadow-md active:translate-y-0 active:scale-[0.98]",
        "focus-visible:ring-2 focus-visible:ring-ring focus-visible:outline-none",
        introIndex != null && "row-intro",
      )}
    >
      {app.uninstaller === "missing" && (
        <TriangleAlert className="absolute top-3 right-3 size-4 text-chart-3" aria-label="언인스톨러 없음" />
      )}
      <AppIcon
        app={app}
        className="size-12 rounded-xl text-lg transition-transform duration-300 ease-(--ease-out-quint) group-hover:scale-105"
      />
      <div className="min-w-0 self-stretch">
        <p className="line-clamp-2 leading-snug font-medium" title={app.displayName}>
          {app.displayName}
        </p>
        <p className="mt-0.5 truncate text-xs text-muted-foreground">{app.publisher ?? "게시자 정보 없음"}</p>
      </div>
      <div className="mt-auto flex w-full items-center justify-between gap-2 text-xs">
        <span className="text-muted-foreground tabular-nums">{formatSize(app.estimatedSizeKb)}</span>
        <span className="flex gap-1">
          <span className="rounded-md border px-1.5 py-0.5 font-mono text-[10px] font-medium text-muted-foreground">
            {app.technology.toUpperCase()}
          </span>
          {app.scope === "user" && (
            <span className="rounded-md border border-primary/30 px-1.5 py-0.5 text-[10px] text-primary">사용자</span>
          )}
        </span>
      </div>
    </button>
  )
}
