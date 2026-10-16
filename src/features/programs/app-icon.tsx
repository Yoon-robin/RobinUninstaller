import { useQuery } from "@tanstack/react-query"

import { api, type InstalledApplication } from "@/lib/api"
import { cn } from "@/lib/utils"

/** 아이콘이 없을 때 쓸 이름 기반 색상 (같은 이름은 항상 같은 색). */
function hueOf(name: string) {
  let h = 0
  for (const ch of name) h = (h * 31 + ch.codePointAt(0)!) % 360
  return h
}

export function AppIcon({ app, className }: { app: InstalledApplication; className?: string }) {
  const icon = useQuery({
    queryKey: ["app-icon", app.id],
    queryFn: () => api.getAppIcon(app.id),
    staleTime: Infinity,
  })

  if (icon.data) {
    return (
      <div className={cn("flex size-9 shrink-0 items-center justify-center", className)}>
        <img src={icon.data} alt="" className="size-[80%]" draggable={false} />
      </div>
    )
  }

  const hue = hueOf(app.displayName)
  const letter = [...app.displayName.trim()][0]?.toUpperCase() ?? "?"
  return (
    <div
      className={cn(
        "flex size-9 shrink-0 items-center justify-center rounded-lg text-sm font-semibold",
        className,
      )}
      style={{
        background: `oklch(0.72 0.12 ${hue} / 16%)`,
        color: `oklch(0.82 0.12 ${hue})`,
      }}
    >
      {letter}
    </div>
  )
}
