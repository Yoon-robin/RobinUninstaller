import { useMemo, useState } from "react"
import { useQuery } from "@tanstack/react-query"
import { ExternalLink, Info, RefreshCw, Search, SearchX, Store, X } from "lucide-react"
import { toast } from "sonner"

import { storeApi, type PackageSource, type StoreApp } from "@/lib/api"
import { formatInstallDate } from "@/lib/format"
import { cn } from "@/lib/utils"
import { Highlight } from "@/components/highlight"
import { IconTooltip } from "@/components/icon-tooltip"
import { Button } from "@/components/ui/button"
import { Input } from "@/components/ui/input"
import { Skeleton } from "@/components/ui/skeleton"
import { AppIcon } from "./app-icon"
import { ErrorState } from "@/components/error-state"

/** 목록 열. 좁은 창에서는 설치일·출처를 숨겨 이름 칸이 사라지지 않게 한다 (숨긴 칸에는 `STORE_WIDE_ONLY`). */
const STORE_GRID =
  "grid grid-cols-[minmax(0,1fr)_minmax(0,10rem)_7rem] items-center gap-4 lg:grid-cols-[minmax(0,1fr)_minmax(0,10rem)_6.5rem_5.75rem_7rem]"
const STORE_WIDE_ONLY = "hidden lg:block"

const sourceLabels: Record<PackageSource, { label: string; className: string }> = {
  store: { label: "Store", className: "border-primary/30 text-primary" },
  sideloaded: { label: "직접 배포", className: "text-muted-foreground" },
  developer: { label: "개발자 모드", className: "border-chart-3/40 text-warning" },
}

export function matchesStoreApp(app: StoreApp, query: string) {
  const q = query.trim().toLowerCase()
  return !q || app.displayName.toLowerCase().includes(q) || (app.publisher ?? "").toLowerCase().includes(q)
}

const openSettings = () => storeApi.openSettings().catch((e) => toast.error(String(e)))

/**
 * Microsoft Store·MSIX 앱 목록 (읽기 전용).
 * Store 앱은 Windows가 앱 데이터까지 함께 지우므로 제거는 Windows 설정으로 안내한다.
 */
export function StoreAppsSection({ initialQuery = "" }: { initialQuery?: string }) {
  const apps = useQuery({ queryKey: ["store-apps"], queryFn: storeApi.list })
  const [query, setQuery] = useState(initialQuery)
  const visible = useMemo(() => (apps.data ?? []).filter((a) => matchesStoreApp(a, query)), [apps.data, query])

  return (
    <section className="overflow-clip rounded-2xl border bg-card shadow-sm">
      <div className="sticky top-0 z-10 rounded-t-2xl border-b bg-card">
        <div className="flex flex-wrap items-center gap-3 px-5 pt-4 pb-3">
          <div className="mr-auto flex items-baseline gap-2">
            <h2 className="font-semibold">Microsoft Store 앱</h2>
            <span className="text-sm text-muted-foreground tabular-nums">
              {apps.isLoading ? "불러오는 중…" : `${visible.length}개`}
            </span>
          </div>
          <div className="relative">
            <Search className="pointer-events-none absolute top-1/2 left-3 size-4 -translate-y-1/2 text-muted-foreground" />
            <Input
              value={query}
              onChange={(e) => setQuery(e.target.value)}
              onKeyDown={(e) => {
                if (e.key === "Escape" && query) {
                  e.stopPropagation()
                  setQuery("")
                }
              }}
              placeholder="이름 또는 게시자 검색"
              className="h-9 w-64 rounded-lg bg-background/60 pr-9 pl-9"
            />
            {query && (
              <button
                type="button"
                onClick={() => setQuery("")}
                aria-label="검색어 지우기"
                className="absolute top-1/2 right-2 flex size-5 -translate-y-1/2 items-center justify-center rounded-full text-muted-foreground hover:bg-muted hover:text-foreground"
              >
                <X className="size-3.5" />
              </button>
            )}
          </div>
          <Button variant="outline" className="h-9 rounded-lg" onClick={openSettings}>
            <ExternalLink />
            Windows 설정에서 제거
          </Button>
          <IconTooltip label="새로고침">
            <Button
              variant="outline"
              size="icon"
              className="size-9 rounded-lg"
              onClick={() => apps.refetch()}
              aria-label="새로고침"
            >
              <RefreshCw className={cn(apps.isFetching && "animate-spin")} />
            </Button>
          </IconTooltip>
        </div>

        <p className="flex gap-2 border-t bg-muted/20 px-5 py-2.5 text-xs leading-relaxed text-muted-foreground">
          <Info className="mt-0.5 size-3.5 shrink-0 text-primary" />
          <span>
            현재 사용자에게 설치된 Store·MSIX 앱이에요. Windows 기본 구성 요소는 빼고 보여 줘요. Store 앱은 제거할 때
            Windows가 앱 데이터까지 함께 지우므로 <b className="text-foreground">Windows 설정에서 제거</b>해 주세요.
          </span>
        </p>

        {visible.length > 0 && (
          <div className={cn(STORE_GRID, "border-t bg-muted/30 px-5 py-2 text-xs font-medium text-muted-foreground")}>
            <span>이름</span>
            <span>게시자</span>
            <span className={STORE_WIDE_ONLY}>설치일</span>
            <span className={STORE_WIDE_ONLY}>출처</span>
            <span />
          </div>
        )}
      </div>

      {apps.isError ? (
        <ErrorState
          title="Store 앱 목록을 읽지 못했어요"
          error={apps.error}
          onRetry={() => apps.refetch()}
          retrying={apps.isFetching}
        />
      ) : apps.isLoading ? (
        <div className="divide-y">
          {Array.from({ length: 6 }, (_, i) => (
            <div key={i} className={cn(STORE_GRID, "px-5 py-3")}>
              <div className="flex items-center gap-3">
                <Skeleton className="size-9 rounded-lg" />
                <Skeleton className="h-4 w-40" />
              </div>
              <Skeleton className="h-4 w-24" />
              <Skeleton className={cn(STORE_WIDE_ONLY, "h-4 w-20")} />
              <Skeleton className={cn(STORE_WIDE_ONLY, "h-4 w-14")} />
            </div>
          ))}
        </div>
      ) : visible.length === 0 ? (
        <div className="flex flex-col items-center gap-2 px-5 py-16 text-center text-sm text-muted-foreground">
          {query ? <SearchX className="size-8 opacity-60" /> : <Store className="size-8 opacity-60" />}
          {query ? `"${query}"와 맞는 Store 앱이 없어요` : "설치된 Store 앱이 없어요"}
        </div>
      ) : (
        <ul className="divide-y">
          {visible.map((app) => {
            const source = sourceLabels[app.source]
            return (
              <li key={app.id} className={cn(STORE_GRID, "group px-5 py-2.5 text-sm hover:bg-accent/40")}>
                <div className="flex min-w-0 items-center gap-3">
                  <AppIcon app={app} />
                  <div className="min-w-0">
                    <p className="truncate font-medium" title={app.displayName}>
                      <Highlight text={app.displayName} query={query} />
                    </p>
                    <p className="truncate text-xs text-muted-foreground" title={app.familyName}>
                      {app.version ?? "버전 정보 없음"}
                    </p>
                  </div>
                </div>
                <span className="truncate text-muted-foreground" title={app.publisher ?? undefined}>
                  {app.publisher ? <Highlight text={app.publisher} query={query} /> : "—"}
                </span>
                <span className={cn(STORE_WIDE_ONLY, "text-muted-foreground tabular-nums")}>
                  {formatInstallDate(app.installDate)}
                </span>
                <span className={STORE_WIDE_ONLY}>
                  <span className={cn("rounded-md border px-1.5 py-0.5 text-[11px] font-medium", source.className)}>
                    {source.label}
                  </span>
                </span>
                <Button
                  variant="ghost"
                  size="sm"
                  className="h-7 justify-self-end opacity-0 group-hover:opacity-100 focus-visible:opacity-100"
                  aria-label={`${app.displayName} 설정에서 제거`}
                  onClick={openSettings}
                >
                  설정에서 제거
                </Button>
              </li>
            )
          })}
        </ul>
      )}
    </section>
  )
}
