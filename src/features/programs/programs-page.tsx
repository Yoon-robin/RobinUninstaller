import { useEffect, useMemo, useState, type ReactNode } from "react"
import { useQuery } from "@tanstack/react-query"
import {
  ArrowDownWideNarrow,
  Check,
  FolderOpen,
  HardDrive,
  Info,
  MoreHorizontal,
  Package,
  RefreshCw,
  Search,
  Trash2,
  TriangleAlert,
  X,
} from "lucide-react"
import { motion } from "motion/react"
import { toast } from "sonner"

import { api, type InstalledApplication } from "@/lib/api"
import { formatInstallDate, formatSize, installDateSortKey } from "@/lib/format"
import { cn } from "@/lib/utils"
import { Badge } from "@/components/ui/badge"
import { Button } from "@/components/ui/button"
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu"
import { Input } from "@/components/ui/input"
import { Skeleton } from "@/components/ui/skeleton"
import { CountUp, FadeUp, Stagger, staggerContainer } from "@/components/motion"
import { AppDetailsSheet } from "./app-details-sheet"
import { AppIcon } from "./app-icon"

type SortKey = "name" | "size" | "date"

const sortLabels: Record<SortKey, string> = {
  name: "이름순",
  size: "크기순",
  date: "최근 설치순",
}

function compare(a: InstalledApplication, b: InstalledApplication, key: SortKey) {
  switch (key) {
    case "size":
      return (b.estimatedSizeKb ?? 0) - (a.estimatedSizeKb ?? 0)
    case "date":
      return installDateSortKey(b.installDate).localeCompare(installDateSortKey(a.installDate))
    default:
      return a.displayName.localeCompare(b.displayName, "ko")
  }
}

const GRID = "grid grid-cols-[minmax(0,1fr)_minmax(0,9rem)_6.5rem_5.75rem_5.5rem_2rem] items-center gap-4"

export function ProgramsPage() {
  const [query, setQuery] = useState("")
  const [sort, setSort] = useState<SortKey>("name")
  const [onlyBroken, setOnlyBroken] = useState(false)
  const [selected, setSelected] = useState<InstalledApplication | null>(null)
  const apps = useQuery({ queryKey: ["installed-apps"], queryFn: api.listInstalledApps })

  const visible = useMemo(() => {
    const q = query.trim().toLowerCase()
    return (apps.data ?? [])
      .filter((a) => !onlyBroken || a.uninstaller === "missing")
      .filter(
        (a) =>
          !q ||
          a.displayName.toLowerCase().includes(q) ||
          (a.publisher ?? "").toLowerCase().includes(q),
      )
      .sort((a, b) => compare(a, b, sort))
  }, [apps.data, query, sort, onlyBroken])

  const stats = useMemo(() => {
    const list = apps.data ?? []
    return {
      total: list.length,
      totalKb: list.reduce((sum, a) => sum + (a.estimatedSizeKb ?? 0), 0),
      maxKb: Math.max(1, ...list.map((a) => a.estimatedSizeKb ?? 0)),
      broken: list.filter((a) => a.uninstaller === "missing").length,
      msi: list.filter((a) => a.technology === "msi").length,
    }
  }, [apps.data])

  const loading = apps.isLoading

  // 목록이 처음 나타날 때만 순서대로 등장시킨다. 검색·필터·정렬 때는 움직이지 않는다.
  const [intro, setIntro] = useState(true)
  useEffect(() => {
    if (!apps.data) return
    const t = setTimeout(() => setIntro(false), 1400)
    return () => clearTimeout(t)
  }, [apps.data])

  return (
    <Stagger className="relative mx-auto flex max-w-6xl min-w-0 flex-col gap-6 px-7 pt-7 pb-10">
      <FadeUp>
        <h1 className="text-2xl font-bold tracking-tight">프로그램</h1>
        <p className="mt-1 text-sm text-muted-foreground">
          이 PC에 설치된 프로그램이에요. 항목을 누르면 자세한 정보를 볼 수 있어요.
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
            onlyBroken
              ? "필터 적용 중 · 다시 누르면 해제"
              : stats.broken > 0
                ? "눌러서 해당 항목만 보기"
                : "모두 정상이에요"
          }
          active={onlyBroken}
          onClick={stats.broken > 0 || onlyBroken ? () => setOnlyBroken((v) => !v) : undefined}
        />
      </motion.div>

      <FadeUp>
      <section className="overflow-hidden rounded-2xl border bg-card shadow-sm">
        <div className="flex flex-wrap items-center gap-3 border-b px-5 py-4">
          <div className="mr-auto flex items-center gap-2">
            <h2 className="font-semibold">설치된 프로그램</h2>
            <span className="text-sm text-muted-foreground tabular-nums">
              {loading ? "불러오는 중…" : `${visible.length}개`}
            </span>
            {onlyBroken && (
              <Badge variant="secondary" className="gap-1">
                언인스톨러 없음
                <button onClick={() => setOnlyBroken(false)} aria-label="필터 해제">
                  <X className="size-3" />
                </button>
              </Badge>
            )}
          </div>
          <div className="relative">
            <Search className="pointer-events-none absolute top-1/2 left-3 size-4 -translate-y-1/2 text-muted-foreground" />
            <Input
              value={query}
              onChange={(e) => setQuery(e.target.value)}
              placeholder="이름 또는 게시자 검색"
              className="h-9 w-64 rounded-lg bg-background/60 pl-9"
            />
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
          <Button
            variant="outline"
            size="icon"
            className="size-9 rounded-lg"
            onClick={() => apps.refetch()}
            aria-label="새로고침"
          >
            <RefreshCw className={cn(apps.isFetching && "animate-spin")} />
          </Button>
        </div>

        <div className={cn(GRID, "border-b bg-muted/30 px-5 py-2.5 text-xs font-medium text-muted-foreground")}>
          <span>이름</span>
          <span>게시자</span>
          <span>크기</span>
          <span>설치일</span>
          <span>형식</span>
          <span />
        </div>

        {apps.isError ? (
          <p className="px-5 py-16 text-center text-sm text-destructive">
            목록을 불러오지 못했어요: {String(apps.error)}
          </p>
        ) : loading ? (
          <div className="divide-y">
            {Array.from({ length: 8 }, (_, i) => (
              <div key={i} className={cn(GRID, "px-5 py-3")}>
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
          <div className="flex flex-col items-center gap-2 px-5 py-16 text-center">
            <Search className="size-8 text-muted-foreground/50" />
            <p className="font-medium">검색 결과가 없어요</p>
            <p className="text-sm text-muted-foreground">다른 이름이나 게시자로 검색해 보세요.</p>
          </div>
        ) : (
          <div className="divide-y">
            {visible.map((app, i) => (
              <ProgramRow
                key={app.id}
                app={app}
                maxKb={stats.maxKb}
                onSelect={setSelected}
                introIndex={intro && i < 16 ? i : undefined}
              />
            ))}
          </div>
        )}
      </section>
      </FadeUp>

      <AppDetailsSheet app={selected} onOpenChange={(open) => !open && setSelected(null)} />
    </Stagger>
  )
}

function ProgramRow({
  app,
  maxKb,
  onSelect,
  introIndex,
}: {
  app: InstalledApplication
  maxKb: number
  onSelect: (app: InstalledApplication) => void
  /** 첫 등장 애니메이션 순서. undefined면 애니메이션 없음. */
  introIndex?: number
}) {
  const introStyle = introIndex == null ? undefined : ({ "--i": introIndex } as React.CSSProperties)
  const openFolder = () => api.openInstallLocation(app.id).catch((e) => toast.error(String(e)))
  // 큰 프로그램 몇 개가 막대를 독차지하지 않도록 제곱근 비율로 그린다.
  const sizeRatio = app.estimatedSizeKb ? Math.max(0.03, Math.sqrt(app.estimatedSizeKb / maxKb)) : 0

  return (
    <div
      role="button"
      tabIndex={0}
      onClick={() => onSelect(app)}
      onKeyDown={(e) => e.key === "Enter" && onSelect(app)}
      style={introStyle}
      className={cn(
        GRID,
        introIndex != null && "row-intro",
        "group cursor-pointer px-5 py-2.5 text-sm transition-colors outline-none hover:bg-accent/40 focus-visible:bg-accent/50",
      )}
    >
      <div className="flex min-w-0 items-center gap-3">
        <AppIcon app={app} />
        <div className="min-w-0">
          <div className="flex items-center gap-2">
            <p className="truncate font-medium" title={app.displayName}>
              {app.displayName}
            </p>
            {app.uninstaller === "missing" && (
              <TriangleAlert className="size-3.5 shrink-0 text-chart-3" aria-label="언인스톨러 없음" />
            )}
          </div>
          <p className="truncate text-xs text-muted-foreground">{app.version ?? "버전 정보 없음"}</p>
        </div>
      </div>

      <span className="truncate text-muted-foreground" title={app.publisher ?? undefined}>
        {app.publisher ?? "—"}
      </span>

      <div className="flex flex-col gap-1">
        <span className="tabular-nums">{formatSize(app.estimatedSizeKb)}</span>
        {sizeRatio > 0 && (
          <div className="h-1 w-full overflow-hidden rounded-full bg-muted">
            <div
              className={cn("h-full rounded-full bg-chart-2/70", introIndex != null && "bar-intro")}
              style={{ width: `${sizeRatio * 100}%` }}
            />
          </div>
        )}
      </div>

      <span className="text-muted-foreground tabular-nums">{formatInstallDate(app.installDate)}</span>

      <div className="flex gap-1">
        <span className="rounded-md border px-1.5 py-0.5 font-mono text-[10px] font-medium text-muted-foreground">
          {app.technology.toUpperCase()}
        </span>
        {app.scope === "machine32" && (
          <span className="rounded-md border px-1.5 py-0.5 font-mono text-[10px] text-muted-foreground">x86</span>
        )}
        {app.scope === "user" && (
          <span className="rounded-md border border-primary/30 px-1.5 py-0.5 text-[10px] text-primary">사용자</span>
        )}
      </div>

      <div onClick={(e) => e.stopPropagation()}>
        <DropdownMenu>
          <DropdownMenuTrigger asChild>
            <Button
              variant="ghost"
              size="icon"
              className="size-8 opacity-0 group-hover:opacity-100 focus-visible:opacity-100 data-[state=open]:opacity-100"
              aria-label="작업"
            >
              <MoreHorizontal />
            </Button>
          </DropdownMenuTrigger>
          <DropdownMenuContent align="end" className="w-52">
            <DropdownMenuLabel className="truncate">{app.displayName}</DropdownMenuLabel>
            <DropdownMenuSeparator />
            <DropdownMenuItem onClick={() => onSelect(app)}>
              <Info />
              세부 정보
            </DropdownMenuItem>
            <DropdownMenuItem onClick={openFolder} disabled={!app.installLocation}>
              <FolderOpen />
              설치 폴더 열기
            </DropdownMenuItem>
            <DropdownMenuSeparator />
            <DropdownMenuItem disabled variant="destructive">
              <Trash2 />
              제거 (곧 지원)
            </DropdownMenuItem>
          </DropdownMenuContent>
        </DropdownMenu>
      </div>
    </div>
  )
}

const toneClasses = {
  "chart-1": "bg-chart-1/12 text-chart-1",
  "chart-2": "bg-chart-2/12 text-chart-2",
  "chart-3": "bg-chart-3/12 text-chart-3",
  "chart-5": "bg-chart-5/12 text-chart-5",
} as const

const glowClasses = {
  "chart-1": "bg-chart-1",
  "chart-2": "bg-chart-2",
  "chart-3": "bg-chart-3",
  "chart-5": "bg-chart-5",
} as const

function StatCard({
  icon: Icon,
  tone,
  label,
  value,
  unit,
  hint,
  active,
  onClick,
}: {
  icon: typeof Package
  tone: keyof typeof toneClasses
  label: string
  value: ReactNode | null
  unit?: string
  hint?: string
  active?: boolean
  onClick?: () => void
}) {
  return (
    <FadeUp className="h-full">
      <div
        role={onClick ? "button" : undefined}
        onClick={onClick}
        className={cn(
          "relative flex h-full items-start gap-4 overflow-hidden rounded-2xl border bg-card p-5 shadow-sm",
          "transition-[transform,border-color,box-shadow] duration-200 ease-(--ease-out-quint)",
          onClick && "cursor-pointer hover:-translate-y-0.5 hover:border-foreground/15 hover:shadow-lg active:translate-y-0 active:scale-[0.99]",
          active && "border-chart-3/60 ring-1 ring-chart-3/40",
        )}
      >
        {/* 카드 오른쪽 위의 은은한 색 번짐 (움직이지 않음) */}
        <div
          aria-hidden
          className={cn("pointer-events-none absolute -top-12 -right-12 size-32 rounded-full opacity-[0.07] blur-2xl", glowClasses[tone])}
        />
        <div className={cn("relative flex size-10 shrink-0 items-center justify-center rounded-xl", toneClasses[tone])}>
          <Icon className="size-5" />
        </div>
        <div className="relative min-w-0">
          <p className="text-sm text-muted-foreground">{label}</p>
          {value == null ? (
            <Skeleton className="mt-1.5 h-7 w-20" />
          ) : (
            <p className="mt-0.5 text-2xl font-bold tracking-tight tabular-nums">
              {value}
              {unit && <span className="ml-0.5 text-base font-medium text-muted-foreground">{unit}</span>}
            </p>
          )}
          {hint && <p className="mt-1 truncate text-xs text-muted-foreground">{hint}</p>}
        </div>
      </div>
    </FadeUp>
  )
}
