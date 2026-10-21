import { useQuery } from "@tanstack/react-query"

import { api, storeApi } from "@/lib/api"
import { cn } from "@/lib/utils"
import { LetterAvatar } from "@/components/letter-avatar"

/** 프로그램·Store 앱 아이콘. Store 앱은 ID가 `msix:`로 시작한다. */
export function AppIcon({ app, className }: { app: { id: string; displayName: string }; className?: string }) {
  const icon = useQuery({
    queryKey: ["app-icon", app.id],
    queryFn: () => (app.id.startsWith("msix:") ? storeApi.icon(app.id) : api.getAppIcon(app.id)),
    staleTime: Infinity,
  })

  if (icon.data) {
    return (
      <div className={cn("flex size-9 shrink-0 items-center justify-center", className)}>
        <img src={icon.data} alt="" className="size-[80%]" draggable={false} />
      </div>
    )
  }

  return <LetterAvatar name={app.displayName} className={className} />
}
