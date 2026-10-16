import { useQuery } from "@tanstack/react-query"
import { Package } from "lucide-react"

import { api, type InstalledApplication } from "@/lib/api"
import { cn } from "@/lib/utils"

export function AppIcon({ app, className }: { app: InstalledApplication; className?: string }) {
  const icon = useQuery({
    queryKey: ["app-icon", app.id],
    queryFn: () => api.getAppIcon(app.id),
    enabled: app.displayIcon != null,
    staleTime: Infinity,
  })

  return (
    <div className={cn("flex size-8 shrink-0 items-center justify-center rounded-md bg-muted", className)}>
      {icon.data ? (
        <img src={icon.data} alt="" className="size-6" draggable={false} />
      ) : (
        <Package className="size-4 text-muted-foreground" />
      )}
    </div>
  )
}
