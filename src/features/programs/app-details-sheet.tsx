import type { ReactNode } from "react"
import { FolderOpen, Trash2 } from "lucide-react"
import { toast } from "sonner"

import { api, type InstalledApplication } from "@/lib/api"
import { formatInstallDate, formatSize } from "@/lib/format"
import { Button } from "@/components/ui/button"
import { Separator } from "@/components/ui/separator"
import { Sheet, SheetContent, SheetDescription, SheetHeader, SheetTitle } from "@/components/ui/sheet"
import { AppIcon } from "./app-icon"
import { UninstallerBadge } from "./uninstaller-badge"

const scopeLabels = {
  machine64: "모든 사용자 (64비트)",
  machine32: "모든 사용자 (32비트)",
  user: "현재 사용자",
} as const

export function AppDetailsSheet({
  app,
  onOpenChange,
}: {
  app: InstalledApplication | null
  onOpenChange: (open: boolean) => void
}) {
  const openFolder = async () => {
    if (!app) return
    try {
      await api.openInstallLocation(app.id)
    } catch (e) {
      toast.error(String(e))
    }
  }

  return (
    <Sheet open={app != null} onOpenChange={onOpenChange}>
      <SheetContent className="w-[480px] sm:max-w-[480px]">
        {app && (
          <>
            <SheetHeader className="flex-row items-center gap-3">
              <AppIcon app={app} className="size-12" />
              <div className="min-w-0">
                <SheetTitle className="truncate">{app.displayName}</SheetTitle>
                <SheetDescription className="truncate">
                  {[app.publisher, app.version].filter(Boolean).join(" · ") || "정보 없음"}
                </SheetDescription>
              </div>
            </SheetHeader>

            <div className="flex gap-2 px-4">
              <Button disabled className="flex-1">
                <Trash2 />
                제거 (Phase 2)
              </Button>
              <Button variant="outline" onClick={openFolder} disabled={!app.installLocation}>
                <FolderOpen />
                폴더 열기
              </Button>
            </div>

            <Separator />

            <dl className="grid grid-cols-[7rem_1fr] gap-x-4 gap-y-3 overflow-y-auto px-4 pb-6 text-sm">
              <Row label="언인스톨러">
                <UninstallerBadge status={app.uninstaller} />
              </Row>
              <Row label="설치 형식">{app.technology.toUpperCase()}</Row>
              <Row label="설치 범위">{scopeLabels[app.scope]}</Row>
              <Row label="크기">{formatSize(app.estimatedSizeKb)}</Row>
              <Row label="설치일">{formatInstallDate(app.installDate)}</Row>
              <Row label="설치 폴더" mono>
                {app.installLocation}
              </Row>
              <Row label="제거 명령" mono>
                {app.uninstallCommand}
              </Row>
              <Row label="자동 제거 명령" mono>
                {app.quietUninstallCommand}
              </Row>
              <Row label="MSI 제품 코드" mono>
                {app.productCode}
              </Row>
              <Row label="레지스트리 ID" mono>
                {app.id}
              </Row>
            </dl>
          </>
        )}
      </SheetContent>
    </Sheet>
  )
}

function Row({ label, children, mono }: { label: string; children: ReactNode; mono?: boolean }) {
  const empty = children == null || children === ""
  return (
    <>
      <dt className="text-muted-foreground">{label}</dt>
      <dd className={mono && !empty ? "font-mono text-xs leading-5 break-all" : undefined}>
        {empty ? <span className="text-muted-foreground">—</span> : children}
      </dd>
    </>
  )
}
