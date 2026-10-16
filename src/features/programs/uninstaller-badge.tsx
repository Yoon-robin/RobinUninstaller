import { Badge } from "@/components/ui/badge"
import type { UninstallerStatus } from "@/lib/api"

const labels: Record<UninstallerStatus, string> = {
  available: "정상",
  missing: "언인스톨러 없음",
  notRegistered: "제거 명령 없음",
  unknown: "확인 불가",
}

export function UninstallerBadge({ status }: { status: UninstallerStatus }) {
  return (
    <Badge variant={status === "available" ? "secondary" : status === "missing" ? "destructive" : "outline"}>
      {labels[status]}
    </Badge>
  )
}
