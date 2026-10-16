import { useMemo, useState } from "react"
import { useQuery, useQueryClient } from "@tanstack/react-query"
import {
  Copy,
  FileWarning,
  FolderOpen,
  MoreHorizontal,
  Power,
  RefreshCw,
  Rocket,
  Search,
  SearchX,
  ShieldAlert,
  X,
} from "lucide-react"
import { motion } from "motion/react"
import { toast } from "sonner"

import { startupApi, type StartupItem, type StartupSource } from "@/lib/api"
import { cn } from "@/lib/utils"
import { Highlight } from "@/components/highlight"
import { IconTooltip } from "@/components/icon-tooltip"
import { CountUp, FadeUp, Stagger, staggerContainer } from "@/components/motion"
import { Button } from "@/components/ui/button"
import { DropdownMenu, DropdownMenuContent, DropdownMenuItem, DropdownMenuTrigger } from "@/components/ui/dropdown-menu"
import { Input } from "@/components/ui/input"
import { Skeleton } from "@/components/ui/skeleton"
import { Switch } from "@/components/ui/switch"
import { FilterChips } from "@/features/programs/segmented"
import { StatCard } from "@/features/programs/stat-card"

const GRID = "grid grid-cols-[minmax(0,1fr)_minmax(0,9rem)_minmax(0,9rem)_4.5rem] items-center gap-4"

const sourceLabels: Record<StartupSource, { where: string; who: string }> = {
  runUser: { where: "레지스트리", who: "현재 사용자" },
  runMachine: { where: "레지스트리", who: "모든 사용자" },
  runMachine32: { where: "레지스트리 (32비트)", who: "모든 사용자" },
  folderUser: { where: "시작 폴더", who: "현재 사용자" },
  folderCommon: { where: "시작 폴더", who: "모든 사용자" },
}

type Filter = "all" | "enabled" | "disabled" | "missing"

const filters: { key: Filter; label: string }[] = [
  { key: "all", label: "전체" },
  { key: "enabled", label: "켜짐" },
  { key: "disabled", label: "꺼짐" },
  { key: "missing", label: "파일 없음" },
]

function matchesFilter(item: StartupItem, filter: Filter) {
  if (filter === "enabled") return item.enabled
  if (filter === "disabled") return !item.enabled
  if (filter === "missing") return item.fileMissing
  return true
}

function matchesQuery(item: StartupItem, query: string) {
  const q = query.trim().toLowerCase()
  if (!q) return true
  return [item.name, item.command, item.publisher, item.appName].some((v) => v?.toLowerCase().includes(q))
}

export function StartupPage() {
  const queryClient = useQueryClient()
  const items = useQuery({ queryKey: ["startup-items"], queryFn: startupApi.list })
  const [query, setQuery] = useState("")
  const [filter, setFilter] = useState<Filter>("all")
  const [busyId, setBusyId] = useState<string | null>(null)

  const all = items.data ?? []
  const visible = useMemo(
    () => all.filter((i) => matchesFilter(i, filter) && matchesQuery(i, query)),
    [all, filter, query],
  )
  const counts = useMemo(
    () =>
      Object.fromEntries(
        filters.map((f) => [f.key, all.filter((i) => matchesFilter(i, f.key) && matchesQuery(i, query)).length]),
      ) as Record<Filter, number>,
    [all, query],
  )
  const enabledCount = all.filter((i) => i.enabled).length
  const missingCount = all.filter((i) => i.fileMissing).length
  const loading = items.isLoading

  const toggle = async (item: StartupItem, enabled: boolean) => {
    setBusyId(item.id)
    try {
      const updated = await startupApi.setEnabled(item.id, enabled)
      queryClient.setQueryData<StartupItem[]>(["startup-items"], (old) =>
        old?.map((i) => (i.id === updated.id ? updated : i)),
      )
      toast.success(
        updated.enabled
          ? `${updated.name}: 다음 로그인부터 다시 실행돼요`
          : `${updated.name}: 다음 로그인부터 실행되지 않아요`,
      )
    } catch (e) {
      toast.error(String(e))
    } finally {
      setBusyId(null)
    }
  }

  return (
    <Stagger className="relative mx-auto flex max-w-6xl min-w-0 flex-col gap-6 px-7 pt-7 pb-10">
      <FadeUp>
        <h1 className="text-2xl font-bold tracking-tight">시작프로그램</h1>
        <p className="mt-1 text-sm text-muted-foreground">
          Windows에 로그인할 때 함께 실행되는 프로그램이에요. 끄기는 작업 관리자와 같은 방식이라 항목을 지우지 않고,
          언제든 다시 켤 수 있어요.
        </p>
      </FadeUp>

      <motion.div variants={staggerContainer} className="grid gap-4 sm:grid-cols-3">
        <StatCard
          icon={Rocket}
          tone="chart-1"
          label="시작 항목"
          value={loading ? null : <CountUp value={all.length} />}
          unit="개"
          hint="Run 키 · 시작 폴더"
        />
        <StatCard
          icon={Power}
          tone="chart-2"
          label="켜진 항목"
          value={loading ? null : <CountUp value={enabledCount} />}
          unit="개"
          hint={loading ? undefined : `꺼진 항목 ${all.length - enabledCount}개`}
        />
        <StatCard
          icon={FileWarning}
          tone={missingCount > 0 ? "chart-3" : "chart-5"}
          label="파일 없음"
          value={loading ? null : <CountUp value={missingCount} />}
          unit="개"
          hint={
            filter === "missing"
              ? "필터 적용 중 · 다시 누르면 해제"
              : missingCount > 0
                ? "지워진 프로그램이 남긴 항목일 수 있어요"
                : "모두 정상이에요"
          }
          active={filter === "missing"}
          onClick={
            missingCount > 0 || filter === "missing"
              ? () => setFilter((f) => (f === "missing" ? "all" : "missing"))
              : undefined
          }
        />
      </motion.div>

      <FadeUp>
        <section className="overflow-clip rounded-2xl border bg-card shadow-sm">
          <div className="sticky top-0 z-10 rounded-t-2xl border-b bg-popover/90 backdrop-blur-xl">
            <div className="flex flex-wrap items-center gap-3 px-5 pt-4 pb-3">
              <div className="mr-auto flex items-baseline gap-2">
                <h2 className="font-semibold">시작 항목</h2>
                <span className="text-sm text-muted-foreground tabular-nums">
                  {loading ? "불러오는 중…" : `${visible.length}개`}
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
                  placeholder="이름, 명령, 게시자 검색"
                  className="h-9 w-64 rounded-lg bg-background/60 pr-8 pl-9"
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
              <IconTooltip label="새로고침">
                <Button
                  variant="outline"
                  size="icon"
                  className="size-9 rounded-lg"
                  onClick={() => items.refetch()}
                  aria-label="새로고침"
                >
                  <RefreshCw className={cn(items.isFetching && "animate-spin")} />
                </Button>
              </IconTooltip>
            </div>
            <div className="px-5 pb-3">
              <FilterChips
                options={filters}
                value={filter}
                counts={counts}
                onChange={setFilter}
                layoutId="startup-filter-chip"
              />
            </div>
            {visible.length > 0 && (
              <div className={cn(GRID, "border-t bg-muted/30 px-5 py-2 text-xs font-medium text-muted-foreground")}>
                <span>이름</span>
                <span>프로그램</span>
                <span>등록 위치</span>
                <span className="text-right">사용</span>
              </div>
            )}
          </div>

          {items.isError ? (
            <p className="px-5 py-16 text-center text-sm text-destructive">
              목록을 불러오지 못했어요: {String(items.error)}
            </p>
          ) : loading ? (
            <div className="divide-y">
              {Array.from({ length: 6 }, (_, i) => (
                <div key={i} className={cn(GRID, "px-5 py-3")}>
                  <div className="flex items-center gap-3">
                    <Skeleton className="size-9 rounded-lg" />
                    <Skeleton className="h-4 w-40" />
                  </div>
                  <Skeleton className="h-4 w-24" />
                  <Skeleton className="h-4 w-20" />
                  <Skeleton className="ml-auto h-5 w-9 rounded-full" />
                </div>
              ))}
            </div>
          ) : visible.length === 0 ? (
            <div className="flex flex-col items-center gap-2 px-5 py-16 text-center">
              <div className="mb-1 flex size-12 items-center justify-center rounded-2xl bg-muted">
                <SearchX className="size-6 text-muted-foreground" />
              </div>
              <p className="font-medium">{all.length === 0 ? "시작 항목이 없어요" : "조건에 맞는 항목이 없어요"}</p>
              {all.length > 0 && (
                <Button
                  variant="outline"
                  size="sm"
                  className="mt-2 rounded-lg"
                  onClick={() => {
                    setQuery("")
                    setFilter("all")
                  }}
                >
                  검색과 필터 지우기
                </Button>
              )}
            </div>
          ) : (
            <div className="divide-y">
              {visible.map((item) => (
                <StartupRow
                  key={item.id}
                  item={item}
                  query={query}
                  busy={busyId === item.id}
                  locked={busyId != null}
                  onToggle={(enabled) => toggle(item, enabled)}
                />
              ))}
            </div>
          )}
        </section>
      </FadeUp>
    </Stagger>
  )
}

function StartupRow({
  item,
  query,
  busy,
  locked,
  onToggle,
}: {
  item: StartupItem
  query: string
  busy: boolean
  locked: boolean
  onToggle: (enabled: boolean) => void
}) {
  const source = sourceLabels[item.source]
  const owner = item.appName ?? item.publisher

  return (
    <div className={cn(GRID, "group px-5 py-2.5 text-sm transition-colors hover:bg-accent/40")}>
      <div className={cn("flex min-w-0 items-center gap-3", !item.enabled && "opacity-55")}>
        <StartupIcon item={item} />
        <div className="min-w-0">
          <div className="flex items-center gap-2">
            <p className="truncate font-medium">
              <Highlight text={item.name} query={query} />
            </p>
            {item.fileMissing && (
              <span className="shrink-0 rounded-full bg-chart-3/12 px-1.5 py-px text-[10px] font-medium text-chart-3">
                파일 없음
              </span>
            )}
          </div>
          <p data-selectable className="truncate font-mono text-xs text-muted-foreground" title={item.command}>
            {item.command || "(명령 없음)"}
          </p>
        </div>
      </div>
      <div className={cn("min-w-0", !item.enabled && "opacity-55")}>
        <p className="truncate" title={owner ?? undefined}>
          {owner ? <Highlight text={owner} query={query} /> : <span className="text-muted-foreground">알 수 없음</span>}
        </p>
        {item.appName && item.publisher && (
          <p className="truncate text-xs text-muted-foreground" title={item.publisher}>
            {item.publisher}
          </p>
        )}
      </div>
      <div className="min-w-0 text-xs" title={item.location}>
        <p className="truncate">{source.where}</p>
        <p className="truncate text-muted-foreground">{source.who}</p>
      </div>
      <div className="flex items-center justify-end gap-1">
        <DropdownMenu>
          <DropdownMenuTrigger asChild>
            <Button
              variant="ghost"
              size="icon"
              className="size-7 text-muted-foreground opacity-0 group-hover:opacity-100 focus-visible:opacity-100 data-[state=open]:opacity-100"
              aria-label="더 보기"
            >
              <MoreHorizontal />
            </Button>
          </DropdownMenuTrigger>
          <DropdownMenuContent align="end">
            <DropdownMenuItem
              disabled={item.fileMissing}
              onClick={() => startupApi.reveal(item.id).catch((e) => toast.error(String(e)))}
            >
              <FolderOpen />
              파일 위치 열기
            </DropdownMenuItem>
            <DropdownMenuItem
              onClick={() => navigator.clipboard.writeText(item.command).then(() => toast.success("명령을 복사했어요"))}
            >
              <Copy />
              명령 복사
            </DropdownMenuItem>
          </DropdownMenuContent>
        </DropdownMenu>
        {item.needsAdmin ? (
          <IconTooltip label="모든 사용자 항목이라 관리자 권한을 요청해요" side="left">
            <ShieldAlert className="size-3.5 shrink-0 text-muted-foreground" />
          </IconTooltip>
        ) : (
          <span className="w-3.5" />
        )}
        <Switch
          checked={item.enabled}
          disabled={locked}
          onCheckedChange={onToggle}
          aria-label={`${item.name} ${item.enabled ? "끄기" : "켜기"}`}
          className={cn(busy && "animate-pulse")}
        />
      </div>
    </div>
  )
}

/** 아이콘이 없을 때 쓸 이름 기반 색상 (같은 이름은 항상 같은 색). */
function hueOf(name: string) {
  let h = 0
  for (const ch of name) h = (h * 31 + ch.codePointAt(0)!) % 360
  return h
}

function StartupIcon({ item }: { item: StartupItem }) {
  const icon = useQuery({
    queryKey: ["startup-icon", item.id, item.filePath],
    queryFn: () => startupApi.icon(item.id),
    staleTime: Infinity,
    enabled: !!item.filePath && !item.fileMissing,
  })

  if (icon.data) {
    return (
      <div className="flex size-9 shrink-0 items-center justify-center">
        <img src={icon.data} alt="" className="size-[80%]" draggable={false} />
      </div>
    )
  }
  const hue = hueOf(item.name)
  return (
    <div
      className="flex size-9 shrink-0 items-center justify-center rounded-lg text-sm font-semibold"
      style={{ background: `oklch(0.72 0.12 ${hue} / 16%)`, color: `oklch(0.82 0.12 ${hue})` }}
    >
      {[...item.name.trim()][0]?.toUpperCase() ?? "?"}
    </div>
  )
}
