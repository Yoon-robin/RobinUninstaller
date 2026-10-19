import { useMemo, useState } from "react"
import { useQuery, useQueryClient } from "@tanstack/react-query"
import { Loader2, Package, Trash2 } from "lucide-react"

import { api, canUninstall, navigate, type InstalledApplication } from "@/lib/api"
import { formatSize } from "@/lib/format"
import { Button } from "@/components/ui/button"
import { AppIcon } from "@/features/programs/app-icon"
import { UninstallDialog } from "@/features/programs/uninstall-dialog"
import { LeftoversDialog, type LeftoverTarget } from "@/features/leftovers/leftovers-dialog"
import { measureAppSizes, sizeProgressText, useAppSizeProgress } from "@/features/programs/app-sizes"

const TOP = 5
/** 이보다 작은 프로그램은 공간 확보 추천에 넣지 않는다 (1 GB) */
const MIN_KB = 1024 * 1024

/**
 * 용량이 큰 프로그램. 캐시를 지우는 것보다 안 쓰는 큰 프로그램 하나를 지우는 편이 공간이 훨씬 많이 생기는 경우가 많다.
 * 제거는 프로그램 화면과 같은 흐름(원래 언인스톨러 → 남은 흔적 찾기 → 백업 후 정리)을 그대로 쓴다.
 */
export function BigProgramsCard() {
  const queryClient = useQueryClient()
  // 프로그램 화면과 같은 쿼리를 써서, 이미 읽은 목록과 직접 잰 크기를 다시 계산하지 않는다
  const apps = useQuery({ queryKey: ["installed-apps"], queryFn: api.listInstalledApps })
  const sizes = useQuery({
    queryKey: ["app-sizes", apps.dataUpdatedAt],
    queryFn: measureAppSizes,
    enabled: apps.data != null,
    staleTime: Infinity,
  })
  const sizeProgress = useAppSizeProgress()
  const [target, setTarget] = useState<InstalledApplication | null>(null)
  const [leftovers, setLeftovers] = useState<LeftoverTarget | null>(null)

  const top = useMemo(() => {
    const withSize = (apps.data ?? []).map((a) => {
      const measured = sizes.data?.[a.id]
      return !a.estimatedSizeKb && measured ? { ...a, estimatedSizeKb: measured, sizeMeasured: true } : a
    })
    return withSize
      .filter((a) => (a.estimatedSizeKb ?? 0) >= MIN_KB)
      .sort((a, b) => (b.estimatedSizeKb ?? 0) - (a.estimatedSizeKb ?? 0))
      .slice(0, TOP)
  }, [apps.data, sizes.data])

  if (top.length === 0) return null
  const max = top[0].estimatedSizeKb ?? 1

  return (
    <section className="overflow-clip rounded-2xl border bg-card shadow-sm">
      <div className="flex flex-wrap items-center gap-3 border-b px-5 py-4">
        <div className="mr-auto min-w-0">
          <h2 className="font-semibold">용량이 큰 프로그램</h2>
          <p className="text-xs text-muted-foreground">
            안 쓰는 큰 프로그램을 지우면 캐시 정리보다 공간이 훨씬 많이 생길 수 있어요. 지우고 나면 남은 흔적도 찾아
            드려요.
          </p>
        </div>
        <Button variant="outline" size="sm" onClick={() => navigate("programs")}>
          <Package />
          전체 목록
        </Button>
      </div>
      <ul className="divide-y">
        {top.map((app) => {
          const kb = app.estimatedSizeKb ?? 0
          return (
            <li key={app.id} className="flex items-center gap-3 px-5 py-2.5">
              <AppIcon app={app} className="size-8" />
              <div className="min-w-0 flex-1">
                <p className="truncate text-sm font-medium">{app.displayName}</p>
                <div className="mt-1 h-1.5 overflow-hidden rounded-full bg-muted">
                  <div className="h-full rounded-full bg-chart-2" style={{ width: `${(kb / max) * 100}%` }} />
                </div>
              </div>
              <span className="w-20 shrink-0 text-right text-sm font-semibold tabular-nums">
                {app.sizeMeasured && "약 "}
                {formatSize(kb)}
              </span>
              {canUninstall(app) ? (
                <Button variant="outline" size="sm" onClick={() => setTarget(app)}>
                  <Trash2 />
                  제거
                </Button>
              ) : (
                <Button variant="ghost" size="sm" onClick={() => navigate("programs")}>
                  프로그램 화면에서
                </Button>
              )}
            </li>
          )
        })}
      </ul>
      {sizes.isFetching && (
        <p className="flex items-center gap-2 border-t px-5 py-2.5 text-xs text-muted-foreground" role="status">
          <Loader2 className="size-3.5 animate-spin" />
          크기 기록이 없는 프로그램을 재는 중이에요{sizeProgressText(sizeProgress)}. 순서가 바뀔 수 있어요.
        </p>
      )}

      <UninstallDialog
        app={target}
        onClose={() => setTarget(null)}
        onFindLeftovers={(app) => setLeftovers({ id: app.id, name: app.displayName, removed: true })}
        onRemoved={() => {
          for (const key of ["installed-apps", "storage-scan", "disk-usage", "recently-removed"]) {
            queryClient.invalidateQueries({ queryKey: [key] })
          }
        }}
      />
      <LeftoversDialog target={leftovers} onClose={() => setLeftovers(null)} />
    </section>
  )
}
