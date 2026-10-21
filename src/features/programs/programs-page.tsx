import { useEffect, useMemo, useRef, useState } from "react"
import { keepPreviousData, useQuery } from "@tanstack/react-query"
import {
  ArrowDownWideNarrow,
  Check,
  CircleCheck,
  MousePointerClick,
  FileDown,
  HardDrive,
  LayoutGrid,
  List,
  ListChecks,
  Package,
  RefreshCw,
  Search,
  SearchX,
  Trash2,
  TriangleAlert,
  X,
} from "lucide-react"
import { AnimatePresence, motion, useReducedMotion } from "motion/react"
import { toast } from "sonner"

import { api, storeApi, canUninstall, needsForce, type InstalledApplication } from "@/lib/api"
import { formatSize } from "@/lib/format"
import { useAppReady } from "@/lib/app-ready"
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
import { LIST_GRID, ProgramRow, WIDE_ONLY } from "./program-row"
import { FilterChips, IconSegmented, TextTabs } from "./segmented"
import { StatCard } from "./stat-card"
import { measureAppSizes, sizeProgressText, useAppSizeProgress } from "./app-sizes"
import { UninstallDialog } from "./uninstall-dialog"
import { BatchUninstallDialog } from "./batch-uninstall-dialog"
import { StoreAppsSection, matchesStoreApp } from "./store-apps-section"
import { LeftoversDialog, type LeftoverTarget } from "@/features/leftovers/leftovers-dialog"
import { ErrorState } from "@/components/error-state"

type ViewMode = "list" | "grid"

const VIEW_KEY = "robin.programs.view"
const INTRO_KEY = "robin.programs.introDismissed"

function loadIntroDismissed() {
  try {
    return localStorage.getItem(INTRO_KEY) === "1"
  } catch {
    return false
  }
}

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

/** 요약 카드가 한 번 등장했는지 (탭을 오갈 때 다시 틀지 않게, 화면 밖에 둔다) */
let statsShownOnce = false

export function ProgramsPage() {
  const [query, setQuery] = useState("")
  const [sort, setSort] = useState<SortKey>("name")
  const [filter, setFilter] = useState<QuickFilter>("all")
  const [view, setViewState] = useState<ViewMode>(loadView)
  const [introDismissed, setIntroDismissed] = useState(loadIntroDismissed)
  const dismissIntro = () => {
    setIntroDismissed(true)
    try {
      localStorage.setItem(INTRO_KEY, "1")
    } catch {
      // 저장이 막혀 있으면 이번 실행 동안만 숨긴다
    }
  }
  const [selected, setSelected] = useState<InstalledApplication | null>(null)
  const [uninstallTarget, setUninstallTarget] = useState<InstalledApplication | null>(null)
  const [leftoversTarget, setLeftoversTarget] = useState<LeftoverTarget | null>(null)
  // 방금 제거된 항목. 목록에서 빠질 때만 접히는 모션을 준다.
  const [removingId, setRemovingId] = useState<string | null>(null)
  // 여러 개 골라 한꺼번에 제거: 고르는 중이면 Set (아니면 null)
  const [picked, setPicked] = useState<Set<string> | null>(null)
  const [batch, setBatch] = useState<InstalledApplication[] | null>(null)
  // 결과에서 흔적 찾기를 연 동안 결과 창을 감춘다 (흔적 창을 닫으면 결과로 돌아온다)
  const [batchHidden, setBatchHidden] = useState(false)
  // 데스크톱 프로그램 / Microsoft Store 앱
  const [tab, setTab] = useState<"desktop" | "store">("desktop")
  // 데스크톱 목록에서 못 찾은 이름을 Store 앱 탭으로 넘겨 이어서 찾는다
  const [storeQuery, setStoreQuery] = useState("")
  const searchRef = useRef<HTMLInputElement>(null)
  const apps = useQuery({ queryKey: ["installed-apps"], queryFn: api.listInstalledApps })
  const storeApps = useQuery({ queryKey: ["store-apps"], queryFn: storeApi.list })
  // 크기 기록이 없는 프로그램은 설치 폴더를 직접 잰다 (목록을 다시 읽을 때마다)
  const sizes = useQuery({
    queryKey: ["app-sizes", apps.dataUpdatedAt],
    queryFn: measureAppSizes,
    enabled: apps.data != null,
    staleTime: Infinity,
    // 목록을 다시 읽어 다시 재는 동안 앞서 잰 크기를 그대로 보여 준다 (크기·정렬이 잠깐 사라지지 않게)
    placeholderData: keepPreviousData,
  })
  const sizeProgress = useAppSizeProgress()

  const setView = (next: ViewMode) => {
    setViewState(next)
    try {
      localStorage.setItem(VIEW_KEY, next)
    } catch {
      // 저장이 막혀 있어도 보기 전환은 그대로 동작한다.
    }
  }

  const all = useMemo(
    () =>
      (apps.data ?? []).map((a) => {
        const measured = sizes.data?.[a.id]
        return !a.estimatedSizeKb && measured ? { ...a, estimatedSizeKb: measured, sizeMeasured: true } : a
      }),
    [apps.data, sizes.data],
  )
  const visible = useMemo(
    () => all.filter((a) => matchesFilter(a, filter) && matchesQuery(a, query)).sort((a, b) => compareApps(a, b, sort)),
    [all, query, sort, filter],
  )
  // 같은 검색어로 Store 앱 탭에서 찾은 수 (데스크톱 목록에 없을 때 알려 준다)
  const storeMatches = useMemo(
    () => (query.trim() ? (storeApps.data ?? []).filter((a) => matchesStoreApp(a, query)).length : 0),
    [storeApps.data, query],
  )
  // 여러 개 제거로 고른 프로그램 중 아직 목록에 있는 것 (새로고침으로 사라진 것은 수에서도 뺀다)
  const pickedApps = useMemo(() => (picked ? all.filter((a) => picked.has(a.id)) : []), [all, picked])
  const pickedRunnable = pickedApps.filter((a) => !needsForce(a)).length
  const hiddenPicked = picked ? pickedApps.filter((a) => !visible.some((v) => v.id === a.id)).length : 0
  // Esc: 고르기 끝내기 (창이나 메뉴가 열려 있으면 그쪽이 먼저 받는다)
  useEffect(() => {
    if (!picked) return
    const onKey = (e: KeyboardEvent) => {
      if (e.key !== "Escape" || e.defaultPrevented) return
      if ((e.target as HTMLElement | null)?.closest?.("input, textarea, [role=dialog], [role=menu]")) return
      setPicked(null)
    }
    window.addEventListener("keydown", onKey)
    return () => window.removeEventListener("keydown", onKey)
  }, [picked])

  const stats = useMemo(
    () => ({
      total: all.length,
      totalKb: all.reduce((sum, a) => sum + (a.estimatedSizeKb ?? 0), 0),
      maxKb: Math.max(1, ...all.map((a) => a.estimatedSizeKb ?? 0)),
      msi: all.filter((a) => a.technology === "msi").length,
      measured: all.filter((a) => a.sizeMeasured).length,
      // 칩 옆 숫자는 검색어까지 반영한다.
      counts: Object.fromEntries(
        quickFilters.map((f) => [f.key, all.filter((a) => matchesFilter(a, f.key) && matchesQuery(a, query)).length]),
      ) as Record<QuickFilter, number>,
      broken: all.filter((a) => matchesFilter(a, "broken")).length,
    }),
    [all, query],
  )

  const loading = apps.isLoading
  // 목록을 다 읽어 숫자를 믿을 수 있는지 (읽는 중·못 읽음이면 "0개"나 초록 표시를 하지 않는다)
  const known = !loading && !apps.isError
  const noneInstalled = known && (apps.data?.length ?? 0) === 0
  const reduceMotion = useReducedMotion()

  // 목록이 처음 나타날 때만 순서대로 등장시킨다. 검색·필터·정렬 때는 움직이지 않는다.
  // 시작 화면이 걷힌 뒤에 시작한다 (그 뒤에서 끝나 버리지 않게).
  const ready = useAppReady()
  const [introDone, setIntroDone] = useState(false)
  const intro = ready && !introDone
  useEffect(() => {
    if (!apps.data || !ready) return
    const t = setTimeout(() => setIntroDone(true), 1400)
    return () => clearTimeout(t)
  }, [apps.data, ready])

  // Ctrl+F: 검색창으로. 키보드 동작이라 애니메이션 없이 바로.
  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      // 다른 화면에서는 이 화면이 숨겨진 채 남아 있으므로, 검색창이 보일 때만 가로챈다
      if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === "f" && searchRef.current?.offsetParent) {
        e.preventDefault()
        searchRef.current?.focus()
        searchRef.current?.select()
      }
    }
    window.addEventListener("keydown", onKey)
    return () => window.removeEventListener("keydown", onKey)
  }, [])

  // 언인스톨러가 있으면 실행하고, 없으면 강제 제거(확인 → 스캔 → 백업 정리)로 간다.
  const startUninstall = (app: InstalledApplication) => {
    if (canUninstall(app)) {
      setUninstallTarget(app)
    } else {
      setSelected(null)
      setLeftoversTarget({ id: app.id, name: app.displayName, removed: false, force: true })
    }
  }
  const handlers = {
    onSelect: setSelected,
    onUninstall: startUninstall,
    onFindLeftovers: (app: InstalledApplication) =>
      setLeftoversTarget({ id: app.id, name: app.displayName, removed: false }),
  }
  const markRemoved = (app: InstalledApplication) => {
    setRemovingId(app.id)
    setTimeout(() => setRemovingId((id) => (id === app.id ? null : id)), 1500)
    setSelected((current) => (current?.id === app.id ? null : current))
  }
  // 키보드: 목록은 Tab 한 번으로 들어가고 ↑/↓/Home/End로 행을 옮긴다.
  const listRef = useRef<HTMLDivElement>(null)
  const [rowFocus, setRowFocus] = useState(0)
  const moveRowFocus = (e: React.KeyboardEvent) => {
    const rows = [...(listRef.current?.querySelectorAll<HTMLElement>("[data-app-row]") ?? [])]
    const at = rows.indexOf(e.target as HTMLElement)
    if (at < 0) return
    const next =
      e.key === "ArrowDown"
        ? at + 1
        : e.key === "ArrowUp"
          ? at - 1
          : e.key === "Home"
            ? 0
            : e.key === "End"
              ? rows.length - 1
              : null
    if (next == null) return
    e.preventDefault()
    rows[Math.max(0, Math.min(next, rows.length - 1))]?.focus()
  }
  // 제거·흔적 창을 닫았을 때 포커스가 갈 곳을 잃었으면(행이 사라짐 등) 목록의 같은 자리(없으면 다음) 행으로 돌린다
  const dialogOpen = selected != null || uninstallTarget != null || leftoversTarget != null || batch != null
  // 창을 닫은 시각. 제거한 행은 닫은 뒤 조금 있다가 목록에서 빠지므로 몇 초 동안은 다시 확인한다.
  const closedAt = useRef(0)
  useEffect(() => {
    if (dialogOpen) {
      closedAt.current = -1
      return
    }
    if (closedAt.current === -1) closedAt.current = Date.now()
    // 처음 화면이 뜰 때는 건드리지 않는다. 창을 닫은 뒤 잠깐 동안만.
    if (closedAt.current === 0 || Date.now() - closedAt.current > 4000) return
    // 창은 닫히는 애니메이션이 끝난 뒤에야 포커스를 놓으므로 몇 번 나눠 확인한다
    const refocus = () => {
      if (document.activeElement && document.activeElement !== document.body) return
      const rows = listRef.current?.querySelectorAll<HTMLElement>("[data-app-row]")
      if (rows?.length) rows[Math.min(rowFocus, rows.length - 1)].focus()
    }
    const timers = [60, 350, 700].map((ms) => setTimeout(refocus, ms))
    return () => timers.forEach(clearTimeout)
  }, [dialogOpen, visible.length, rowFocus])
  const togglePick = (app: InstalledApplication) =>
    setPicked((p) => {
      if (!p) return p
      const next = new Set(p)
      if (next.has(app.id)) next.delete(app.id)
      else next.add(app.id)
      return next
    })
  const exportList = async () => {
    try {
      const name = await api.exportAppList()
      if (name) toast.success(`${name} 파일로 저장했어요`)
    } catch (e) {
      toast.error(String(e))
    }
  }
  const resetFilters = () => {
    setQuery("")
    setFilter("all")
  }

  return (
    <Stagger className="relative mx-auto flex max-w-6xl min-w-0 flex-col gap-6 px-7 pt-7 pb-10">
      <FadeUp className="flex flex-wrap items-end gap-4">
        <div className="mr-auto">
          <h1 className="text-2xl font-bold tracking-tight">프로그램</h1>
          <p className="mt-1 text-sm text-muted-foreground">
            이 PC에 설치된 프로그램이에요. 항목을 누르면 자세한 정보를, 오른쪽 클릭하면 바로 할 수 있는 작업을 볼 수
            있어요.
          </p>
        </div>
        <TextTabs
          label="프로그램 종류"
          layoutId="programs-tab"
          value={tab}
          onChange={(next) => {
            setTab(next)
            setPicked(null)
          }}
          options={[
            { key: "desktop", label: "데스크톱 프로그램", count: apps.data?.length },
            { key: "store", label: "Store 앱", count: storeApps.data?.length },
          ]}
        />
      </FadeUp>

      {/* 오른쪽 클릭 메뉴가 있는 데스크톱 목록에서만. 닫으면 아래가 튀지 않게 접히며 사라진다 (간격 gap-6만큼 함께 줄인다). */}
      <AnimatePresence initial={false}>
        {!introDismissed && tab === "desktop" && (
          <motion.div
            key="intro"
            exit={reduceMotion ? { opacity: 0 } : { opacity: 0, height: 0, marginBottom: -24 }}
            transition={{ duration: 0.22, ease: [0.22, 1, 0.36, 1] }}
            className="overflow-hidden"
          >
            <div className="flex items-start gap-3 rounded-2xl border border-primary/30 bg-primary/8 px-5 py-4 text-sm">
              <span className="mt-0.5 flex size-7 shrink-0 items-center justify-center rounded-lg bg-primary/15 text-primary">
                <MousePointerClick className="size-4" />
              </span>
              <div className="min-w-0 flex-1">
                <p className="font-medium">프로그램 메뉴에서 제거해요</p>
                <p className="mt-1 text-muted-foreground">
                  지울 프로그램을 오른쪽 클릭하거나 <b className="text-foreground">⋯</b>를 눌러{" "}
                  <b className="text-foreground">제거</b>를 고르면, 끝난 뒤 남은 흔적을 찾아 백업하고 정리할 수 있어요.
                  항목을 누르면 열리는 자세히 보기에도 같은 버튼이 있어요. 지우기 전에 무엇이 남을지 보려면{" "}
                  <b className="whitespace-nowrap text-foreground">흔적 미리 보기</b>를 고르세요.
                </p>
              </div>
              <Button variant="ghost" size="sm" onClick={dismissIntro}>
                알겠어요
              </Button>
            </div>
          </motion.div>
        )}
      </AnimatePresence>

      {tab === "store" ? (
        <FadeUp>
          <StoreAppsSection query={storeQuery} onQueryChange={setStoreQuery} searchRef={searchRef} />
        </FadeUp>
      ) : (
        <>
          <motion.div
            variants={staggerContainer}
            // 처음 한 번만 차례로 등장한다 (Store 탭에서 돌아올 때는 바로)
            initial={statsShownOnce ? false : "hidden"}
            animate="show"
            onAnimationComplete={() => (statsShownOnce = true)}
            className="grid gap-4 sm:grid-cols-3"
          >
            <StatCard
              icon={Package}
              tone="chart-1"
              label="설치된 프로그램"
              value={
                loading || !ready ? null : apps.isError ? "—" : <CountUp id="programs-total" value={stats.total} />
              }
              unit="개"
              hint={loading || apps.isError ? undefined : `EXE ${stats.total - stats.msi} · MSI ${stats.msi}`}
            />
            <StatCard
              icon={HardDrive}
              tone="chart-2"
              label="전체 예상 용량"
              value={
                loading || !ready ? null : apps.isError ? (
                  "—"
                ) : (
                  <CountUp id="programs-size" value={stats.totalKb} format={formatSize} />
                )
              }
              hint={
                loading || apps.isError
                  ? undefined
                  : sizes.isFetching
                    ? `크기를 재는 중${sizeProgressText(sizeProgress)}`
                    : stats.measured > 0
                      ? `기록한 크기 + 직접 잰 ${stats.measured}개`
                      : "설치 프로그램이 기록한 크기 기준"
              }
            />
            <StatCard
              // 다 읽은 뒤 0개일 때만 초록 체크. 읽는 중이거나 못 읽었으면 아무 색도 아니다.
              icon={known && stats.broken === 0 ? CircleCheck : TriangleAlert}
              tone={!known ? "muted" : stats.broken > 0 ? "chart-3" : "chart-5"}
              label="언인스톨러 없음"
              value={
                loading || !ready ? null : apps.isError ? "—" : <CountUp id="programs-broken" value={stats.broken} />
              }
              unit="개"
              hint={
                !known || noneInstalled
                  ? undefined
                  : filter === "broken"
                    ? "필터 적용 중 · 다시 누르면 해제"
                    : stats.broken > 0
                      ? "눌러서 해당 항목만 보기"
                      : "모든 프로그램에 언인스톨러가 있어요"
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
            <section className="@container overflow-clip rounded-2xl border bg-card shadow-sm">
              <div className="sticky top-0 z-10 rounded-t-2xl border-b bg-card">
                <div className="flex flex-wrap items-center gap-3 px-5 pt-4 pb-3">
                  <div className="mr-auto flex items-baseline gap-2">
                    <h2 className="font-semibold">설치된 프로그램</h2>
                    <span className="text-sm text-muted-foreground tabular-nums">
                      {loading ? "불러오는 중…" : `${visible.length}개`}
                    </span>
                  </div>
                  {/* 좁은 창에서는 버튼 묶음이 통째로 다음 줄로 가고 검색창이 줄어든다 (버튼 하나만 떨어지지 않게) */}
                  <div className="flex flex-1 items-center justify-end gap-3">
                    <div className="relative max-w-64 min-w-40 flex-1">
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
                        className="h-9 w-full rounded-lg bg-background/60 pr-8 pl-9 lg:pr-14"
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
                        <kbd className="pointer-events-none absolute top-1/2 right-2 hidden -translate-y-1/2 rounded border bg-muted/60 px-1.5 font-sans text-[11px] text-muted-foreground lg:block">
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
                          <DropdownMenuItem
                            key={key}
                            role="menuitemradio"
                            aria-checked={sort === key}
                            onClick={() => setSort(key)}
                          >
                            <Check aria-hidden className={cn(sort !== key && "invisible")} />
                            {sortLabels[key]}
                          </DropdownMenuItem>
                        ))}
                      </DropdownMenuContent>
                    </DropdownMenu>
                    <Button
                      variant={picked ? "default" : "outline"}
                      className="h-9 rounded-lg"
                      onClick={() => setPicked((p) => (p ? null : new Set()))}
                      aria-pressed={picked != null}
                    >
                      <ListChecks />
                      {picked ? "고르기 끝내기" : "여러 개 제거"}
                    </Button>
                    {/* 고르는 중에는 목록으로만 보여 주지만, 버튼은 자리를 지켜 화면이 튀지 않게 한다 */}
                    <div className={cn(picked && "pointer-events-none opacity-50")} aria-disabled={picked != null}>
                      <IconSegmented
                        options={viewOptions}
                        value={picked ? "list" : view}
                        onChange={setView}
                        layoutId="view-toggle"
                        label="보기 방식"
                      />
                    </div>
                    <IconTooltip
                      label={sizes.isFetching ? "크기를 다 잰 뒤에 내보낼 수 있어요" : "목록 내보내기 (CSV)"}
                    >
                      {/* 꺼진 버튼은 마우스 이벤트를 받지 않아 툴팁이 뜨지 않으므로 바깥 칸이 받는다 */}
                      <span tabIndex={loading || sizes.isFetching ? 0 : -1} className="inline-flex rounded-lg">
                        <Button
                          variant="outline"
                          size="icon"
                          className="size-9 rounded-lg"
                          disabled={loading || sizes.isFetching}
                          onClick={exportList}
                          aria-label="목록 내보내기"
                        >
                          <FileDown />
                        </Button>
                      </span>
                    </IconTooltip>
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
                </div>

                <div className="px-5 pb-3">
                  <FilterChips options={quickFilters} value={filter} counts={stats.counts} onChange={setFilter} />
                </div>

                {(view === "list" || picked) && visible.length > 0 && (
                  <div
                    className={cn(
                      LIST_GRID,
                      "border-t bg-muted/30 px-5 py-2 text-xs font-medium text-muted-foreground",
                    )}
                  >
                    <span>이름</span>
                    <span>게시자</span>
                    <span>크기</span>
                    <span className={WIDE_ONLY}>설치일</span>
                    <span className={WIDE_ONLY}>형식</span>
                    <span />
                  </div>
                )}
              </div>

              {apps.isError ? (
                <ErrorState
                  title="설치된 프로그램 목록을 읽지 못했어요"
                  error={apps.error}
                  onRetry={() => apps.refetch()}
                  retrying={apps.isFetching}
                />
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
                      <Skeleton className={cn(WIDE_ONLY, "h-4 w-20")} />
                      <Skeleton className={cn(WIDE_ONLY, "h-5 w-12 rounded-full")} />
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
                  <p className="font-medium">
                    {noneInstalled ? "설치된 프로그램을 찾지 못했어요" : "조건에 맞는 프로그램이 없어요"}
                  </p>
                  <p className="text-sm text-muted-foreground">
                    {noneInstalled
                      ? "Windows에 등록된 프로그램이 없어요. 새로 설치했다면 새로고침해 보세요."
                      : query
                        ? `"${query}" 검색 결과가 없어요.`
                        : "선택한 필터에 해당하는 프로그램이 없어요."}
                  </p>
                  <div className="mt-2 flex flex-wrap justify-center gap-2">
                    {storeMatches > 0 && (
                      <Button
                        size="sm"
                        className="rounded-lg"
                        onClick={() => {
                          setStoreQuery(query)
                          setTab("store")
                        }}
                      >
                        Store 앱에서 {storeMatches}개 보기
                      </Button>
                    )}
                    {!noneInstalled && (
                      <Button variant="outline" size="sm" className="rounded-lg" onClick={resetFilters}>
                        검색과 필터 지우기
                      </Button>
                    )}
                  </div>
                </motion.div>
              ) : (
                // 보기를 바꿀 때만 짧게 교차 전환한다. 검색·필터로 항목이 바뀔 때는 움직이지 않는다.
                <AnimatePresence mode="wait" initial={false}>
                  <motion.div
                    key={view}
                    initial={{ opacity: 0 }}
                    animate={{ opacity: 1 }}
                    exit={{ opacity: 0, transition: { duration: 0.08 } }}
                    transition={{ duration: 0.14, ease: [0.22, 1, 0.36, 1] }}
                  >
                    {view === "list" || picked ? (
                      <div className="divide-y" ref={listRef} onKeyDown={moveRowFocus}>
                        <AnimatePresence initial={false}>
                          {visible.map((app, i) => (
                            <ProgramRow
                              key={app.id}
                              app={app}
                              maxKb={stats.maxKb}
                              query={query}
                              removing={app.id === removingId}
                              introIndex={intro && i < 16 ? i : undefined}
                              picked={picked ? picked.has(app.id) : undefined}
                              onPick={togglePick}
                              tabbable={i === Math.min(rowFocus, visible.length - 1)}
                              onRowFocus={() => setRowFocus(i)}
                              {...handlers}
                            />
                          ))}
                        </AnimatePresence>
                      </div>
                    ) : (
                      <ProgramGrid
                        apps={visible}
                        intro={intro}
                        animateOnMount={false}
                        query={query}
                        removingId={removingId}
                        {...handlers}
                      />
                    )}
                  </motion.div>
                </AnimatePresence>
              )}
            </section>
          </FadeUp>
        </>
      )}

      <AppDetailsSheet
        app={selected}
        onOpenChange={(open) => !open && setSelected(null)}
        onUninstall={startUninstall}
        onFindLeftovers={handlers.onFindLeftovers}
      />
      <UninstallDialog
        app={uninstallTarget}
        onClose={() => setUninstallTarget(null)}
        onFindLeftovers={(app) => setLeftoversTarget({ id: app.id, name: app.displayName, removed: true })}
        onForceUninstall={(app) =>
          setLeftoversTarget({ id: app.id, name: app.displayName, removed: false, force: true })
        }
        onRemoved={markRemoved}
      />
      <LeftoversDialog
        target={leftoversTarget}
        onClose={() => {
          setLeftoversTarget(null)
          setBatchHidden(false)
        }}
      />
      <BatchUninstallDialog
        apps={batch}
        hidden={batchHidden}
        onClose={() => setBatch(null)}
        onStarted={() => setPicked(null)}
        onRemoved={markRemoved}
        onFindLeftovers={(app) => {
          setBatchHidden(true)
          setLeftoversTarget({ id: app.id, name: app.displayName, removed: true })
        }}
      />

      {/* 고르는 중이면 아래에 떠 있는 작업 막대 */}
      <AnimatePresence>
        {picked && (
          <motion.div
            initial={{ opacity: 0, y: 16 }}
            animate={{ opacity: 1, y: 0 }}
            exit={{ opacity: 0, y: 16, transition: { duration: 0.15 } }}
            transition={{ type: "spring", duration: 0.35, bounce: 0 }}
            // 알림 위치를 옮기는 기준 (App.tsx useToastsOutOfTheWay)
            data-pick-bar
            className="sticky bottom-4 z-20 mx-auto flex w-full max-w-xl items-center gap-3 rounded-2xl border bg-popover px-4 py-3 shadow-[var(--shadow-floating)]"
          >
            <p className="mr-auto text-sm tabular-nums">
              {pickedApps.length > 0 ? `${pickedApps.length}개 선택` : "제거할 프로그램을 눌러 고르세요"}
              {/* 검색·필터로 가려진 선택이 있으면 알린다 (보이지 않는 것까지 지우지 않게) */}
              {hiddenPicked > 0 && (
                <span className="block text-xs text-muted-foreground">
                  그중 {hiddenPicked}개는 지금 목록에 안 보여요
                </span>
              )}
              {pickedApps.length > 0 && pickedRunnable === 0 && (
                <span className="block text-xs text-warning">언인스톨러가 없는 프로그램은 하나씩 강제 제거해요</span>
              )}
            </p>
            <Button
              variant="ghost"
              size="sm"
              onClick={() => setPicked(new Set(visible.map((a) => a.id)))}
              disabled={visible.length === 0}
            >
              보이는 것 모두
            </Button>
            <Button variant="ghost" size="sm" onClick={() => setPicked(new Set())} disabled={pickedApps.length === 0}>
              선택 해제
            </Button>
            <Button
              variant="destructive"
              size="sm"
              disabled={pickedRunnable === 0}
              onClick={() => setBatch(pickedApps)}
            >
              <Trash2 />
              선택한 프로그램 제거
            </Button>
          </motion.div>
        )}
      </AnimatePresence>
    </Stagger>
  )
}
