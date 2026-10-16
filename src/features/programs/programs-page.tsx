import { useEffect, useMemo, useRef, useState } from "react"
import { useQuery } from "@tanstack/react-query"
import {
  ArrowDownWideNarrow,
  Check,
  HardDrive,
  LayoutGrid,
  List,
  Package,
  RefreshCw,
  Search,
  SearchX,
  TriangleAlert,
  X,
} from "lucide-react"
import { AnimatePresence, motion } from "motion/react"

import { api, type InstalledApplication } from "@/lib/api"
import { formatSize } from "@/lib/format"
import { cn } from "@/lib/utils"
import { IconTooltip } from "@/components/icon-tooltip"
import { CountUp, FadeUp, Stagger, staggerContainer } from "@/components/motion"
import { Button } from "@/components/ui/button"
import { DropdownMenu, DropdownMenuContent, DropdownMenuItem, DropdownMenuTrigger } from "@/components/ui/dropdown-menu"
import { Input } from "@/components/ui/input"
import { Skeleton } from "@/components/ui/skeleton"
import { AppDetailsSheet } from "./app-details-sheet"
import {
  compareApps,
  matchesFilter,
  matchesQuery,
  quickFilters,
  sortLabels,
  type QuickFilter,
  type SortKey,
} from "./filters"
import { ProgramGrid } from "./program-grid"
import { LIST_GRID, ProgramRow } from "./program-row"
import { FilterChips, IconSegmented } from "./segmented"
import { StatCard } from "./stat-card"
import { UninstallDialog } from "./uninstall-dialog"
import { LeftoversDialog, type LeftoverTarget } from "@/features/leftovers/leftovers-dialog"

type ViewMode = "list" | "grid"

const VIEW_KEY = "robin.programs.view"

function loadView(): ViewMode {
  try {
    return localStorage.getItem(VIEW_KEY) === "grid" ? "grid" : "list"
  } catch {
    return "list"
  }
}

const viewOptions = [
  { key: "list" as const, icon: List, label: "목록 보기" },
  { key: "grid" as const, icon: LayoutGrid, label: "카드 보기" },
]

export function ProgramsPage() {
  const [query, setQuery] = useState("")
  const [sort, setSort] = useState<SortKey>("name")
  const [filter, setFilter] = useState<QuickFilter>("all")
  const [view, setViewState] = useState<ViewMode>(loadView)
  const [selected, setSelected] = useState<InstalledApplication | null>(null)
  const [uninstallTarget, setUninstallTarget] = useState<InstalledApplication | null>(null)
  const [leftoversTarget, setLeftoversTarget] = useState<LeftoverTarget | null>(null)
  // 방금 제거된 항목. 목록에서 빠질 때만 접히는 모션을 준다.
  const [removingId, setRemovingId] = useState<string | null>(null)
  const searchRef = useRef<HTMLInputElement>(null)
  const apps = useQuery({ queryKey: ["installed-apps"], queryFn: api.listInstalledApps })

  const setView = (next: ViewMode) => {
    setViewState(next)
    try {
      localStorage.setItem(VIEW_KEY, next)
    } catch {
      // 저장이 막혀 있어도 보기 전환은 그대로 동작한다.
    }
  }

  const all = apps.data ?? []
  const visible = useMemo(
    () => all.filter((a) => matchesFilter(a, filter) && matchesQuery(a, query)).sort((a, b) => compareApps(a, b, sort)),
    [all, query, sort, filter],
  )

  const stats = useMemo(
    () => ({
      total: all.length,
      totalKb: all.reduce((sum, a) => sum + (a.estimatedSizeKb ?? 0), 0),
      maxKb: Math.max(1, ...all.map((a) => a.estimatedSizeKb ?? 0)),
      msi: all.filter((a) => a.technology === "msi").length,
      // 칩 옆 숫자는 검색어까지 반영한다.
      counts: Object.fromEntries(
        quickFilters.map((f) => [f.key, all.filter((a) => matchesFilter(a, f.key) && matchesQuery(a, query)).length]),
      ) as Record<QuickFilter, number>,
      broken: all.filter((a) => matchesFilter(a, "broken")).length,
    }),
    [all, query],
  )

  const loading = apps.isLoading

  // 목록이 처음 나타날 때만 순서대로 등장시킨다. 검색·필터·정렬 때는 움직이지 않는다.
  const [intro, setIntro] = useState(true)
  useEffect(() => {
    if (!apps.data) return
    const t = setTimeout(() => setIntro(false), 1400)
    return () => clearTimeout(t)
  }, [apps.data])

  // Ctrl+F: 검색창으로. 키보드 동작이라 애니메이션 없이 바로.
  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === "f") {
        e.preventDefault()
        searchRef.current?.focus()
        searchRef.current?.select()
      }
    }
    window.addEventListener("keydown", onKey)
    return () => window.removeEventListener("keydown", onKey)
  }, [])

  const handlers = {
    onSelect: setSelected,
    onUninstall: setUninstallTarget,
    onFindLeftovers: (app: InstalledApplication) =>
      setLeftoversTarget({ id: app.id, name: app.displayName, removed: false }),
  }
  const resetFilters = () => {
    setQuery("")
    setFilter("all")
  }

  return (
    <Stagger className="relative mx-auto flex max-w-6xl min-w-0 flex-col gap-6 px-7 pt-7 pb-10">
      <FadeUp>
        <h1 className="text-2xl font-bold tracking-tight">프로그램</h1>
        <p className="mt-1 text-sm text-muted-foreground">
          이 PC에 설치된 프로그램이에요. 항목을 누르면 자세한 정보를, 오른쪽 클릭하면 바로 할 수 있는 작업을 볼 수
          있어요.
        </p>
      </FadeUp>

      <motion.div variants={staggerContainer} className="grid gap-4 sm:grid-cols-3">
        <StatCard
          icon={Package}
          tone="chart-1"
          label="설치된 프로그램"
          value={loading ? null : <CountUp value={stats.total} />}
          unit="개"
          hint={loading ? undefined : `EXE ${stats.total - stats.msi} · MSI ${stats.msi}`}
        />
        <StatCard
          icon={HardDrive}
          tone="chart-2"
          label="전체 예상 용량"
          value={loading ? null : <CountUp value={stats.totalKb} format={formatSize} />}
          hint="설치 프로그램이 기록한 크기 기준"
        />
        <StatCard
          icon={TriangleAlert}
          tone={stats.broken > 0 ? "chart-3" : "chart-5"}
          label="언인스톨러 없음"
          value={loading ? null : <CountUp value={stats.broken} />}
          unit="개"
          hint={
            filter === "broken"
              ? "필터 적용 중 · 다시 누르면 해제"
              : stats.broken > 0
                ? "눌러서 해당 항목만 보기"
                : "모두 정상이에요"
          }
          active={filter === "broken"}
          onClick={
            stats.broken > 0 || filter === "broken"
              ? () => setFilter((f) => (f === "broken" ? "all" : "broken"))
              : undefined
          }
        />
      </motion.div>

      <FadeUp>
        {/* overflow-clip: 모서리는 자르되 스크롤 컨테이너가 되지 않아 안쪽 sticky가 동작한다 */}
        <section className="overflow-clip rounded-2xl border bg-card shadow-sm">
          <div className="sticky top-0 z-10 rounded-t-2xl border-b bg-popover/90 backdrop-blur-xl">
            <div className="flex flex-wrap items-center gap-3 px-5 pt-4 pb-3">
              <div className="mr-auto flex items-baseline gap-2">
                <h2 className="font-semibold">설치된 프로그램</h2>
                <span className="text-sm text-muted-foreground tabular-nums">
                  {loading ? "불러오는 중…" : `${visible.length}개`}
                </span>
              </div>
              <div className="relative">
                <Search className="pointer-events-none absolute top-1/2 left-3 size-4 -translate-y-1/2 text-muted-foreground" />
                <Input
                  ref={searchRef}
                  value={query}
                  onChange={(e) => setQuery(e.target.value)}
                  onKeyDown={(e) => {
                    if (e.key === "Escape" && query) {
                      e.stopPropagation()
                      setQuery("")
                    }
                    if (e.key === "Enter" && visible[0]) setSelected(visible[0])
                  }}
                  placeholder="이름 또는 게시자 검색"
                  className="h-9 w-64 rounded-lg bg-background/60 pr-14 pl-9"
                />
                {query ? (
                  <button
                    type="button"
                    onClick={() => {
                      setQuery("")
                      searchRef.current?.focus()
                    }}
                    aria-label="검색어 지우기"
                    className="absolute top-1/2 right-2 flex size-5 -translate-y-1/2 items-center justify-center rounded-full text-muted-foreground hover:bg-muted hover:text-foreground"
                  >
                    <X className="size-3.5" />
                  </button>
                ) : (
                  <kbd className="pointer-events-none absolute top-1/2 right-2 -translate-y-1/2 rounded border bg-muted/60 px-1.5 font-sans text-[10px] text-muted-foreground">
                    Ctrl F
                  </kbd>
                )}
              </div>
              <DropdownMenu>
                <DropdownMenuTrigger asChild>
                  <Button variant="outline" className="h-9 rounded-lg">
                    <ArrowDownWideNarrow />
                    {sortLabels[sort]}
                  </Button>
                </DropdownMenuTrigger>
                <DropdownMenuContent align="end">
                  {(Object.keys(sortLabels) as SortKey[]).map((key) => (
                    <DropdownMenuItem key={key} onClick={() => setSort(key)}>
                      <Check className={cn(sort !== key && "invisible")} />
                      {sortLabels[key]}
                    </DropdownMenuItem>
                  ))}
                </DropdownMenuContent>
              </DropdownMenu>
              <IconSegmented options={viewOptions} value={view} onChange={setView} layoutId="view-toggle" />
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

            <div className="px-5 pb-3">
              <FilterChips options={quickFilters} value={filter} counts={stats.counts} onChange={setFilter} />
            </div>

            {view === "list" && visible.length > 0 && (
              <div
                className={cn(LIST_GRID, "border-t bg-muted/30 px-5 py-2 text-xs font-medium text-muted-foreground")}
              >
                <span>이름</span>
                <span>게시자</span>
                <span>크기</span>
                <span>설치일</span>
                <span>형식</span>
                <span />
              </div>
            )}
          </div>

          {apps.isError ? (
            <p className="px-5 py-16 text-center text-sm text-destructive">
              목록을 불러오지 못했어요: {String(apps.error)}
            </p>
          ) : loading ? (
            <div className="divide-y">
              {Array.from({ length: 8 }, (_, i) => (
                <div key={i} className={cn(LIST_GRID, "px-5 py-3")}>
                  <div className="flex items-center gap-3">
                    <Skeleton className="size-9 rounded-lg" />
                    <Skeleton className="h-4 w-40" />
                  </div>
                  <Skeleton className="h-4 w-24" />
                  <Skeleton className="h-4 w-20" />
                  <Skeleton className="h-4 w-20" />
                  <Skeleton className="h-5 w-12 rounded-full" />
                  <span />
                </div>
              ))}
            </div>
          ) : visible.length === 0 ? (
            <motion.div
              initial={{ opacity: 0, y: 6 }}
              animate={{ opacity: 1, y: 0 }}
              transition={{ duration: 0.25, ease: [0.22, 1, 0.36, 1] }}
              className="flex flex-col items-center gap-2 px-5 py-16 text-center"
            >
              <div className="mb-1 flex size-12 items-center justify-center rounded-2xl bg-muted">
                <SearchX className="size-6 text-muted-foreground" />
              </div>
              <p className="font-medium">조건에 맞는 프로그램이 없어요</p>
              <p className="text-sm text-muted-foreground">
                {query ? `"${query}"` : "선택한 필터"}에 해당하는 항목이 없어요.
              </p>
              <Button variant="outline" size="sm" className="mt-2 rounded-lg" onClick={resetFilters}>
                검색과 필터 지우기
              </Button>
            </motion.div>
          ) : (
            // 보기를 바꿀 때만 짧게 교차 전환한다. 검색·필터로 항목이 바뀔 때는 움직이지 않는다.
            <AnimatePresence mode="wait" initial={false}>
              <motion.div
                key={view}
                initial={{ opacity: 0, filter: "blur(4px)" }}
                animate={{ opacity: 1, filter: "blur(0px)" }}
                exit={{ opacity: 0, filter: "blur(4px)", transition: { duration: 0.1 } }}
                transition={{ duration: 0.2, ease: [0.22, 1, 0.36, 1] }}
              >
                {view === "list" ? (
                  <div className="divide-y">
                    <AnimatePresence initial={false}>
                      {visible.map((app, i) => (
                        <ProgramRow
                          key={app.id}
                          app={app}
                          maxKb={stats.maxKb}
                          query={query}
                          removing={app.id === removingId}
                          introIndex={intro && i < 16 ? i : undefined}
                          {...handlers}
                        />
                      ))}
                    </AnimatePresence>
                  </div>
                ) : (
                  <ProgramGrid apps={visible} intro={intro} query={query} removingId={removingId} {...handlers} />
                )}
              </motion.div>
            </AnimatePresence>
          )}
        </section>
      </FadeUp>

      <AppDetailsSheet
        app={selected}
        onOpenChange={(open) => !open && setSelected(null)}
        onUninstall={setUninstallTarget}
        onFindLeftovers={handlers.onFindLeftovers}
      />
      <UninstallDialog
        app={uninstallTarget}
        onClose={() => setUninstallTarget(null)}
        onFindLeftovers={(app) => setLeftoversTarget({ id: app.id, name: app.displayName, removed: true })}
        onRemoved={(app) => {
          setRemovingId(app.id)
          setTimeout(() => setRemovingId((id) => (id === app.id ? null : id)), 1500)
          setSelected((current) => (current?.id === app.id ? null : current))
        }}
      />
      <LeftoversDialog target={leftoversTarget} onClose={() => setLeftoversTarget(null)} />
    </Stagger>
  )
}
