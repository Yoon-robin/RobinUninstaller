import { useMemo, useRef, useState } from "react"
import { useQuery, useQueryClient } from "@tanstack/react-query"
import {
  ChevronRight,
  FileWarning,
  FolderOpen,
  Loader2,
  Power,
  PowerOff,
  RefreshCw,
  Rocket,
  Search,
  SearchX,
  ShieldAlert,
  X,
} from "lucide-react"
import { AnimatePresence, LayoutGroup, motion } from "motion/react"
import { toast } from "sonner"

import { isMachineStartup, startupApi, type StartupEntry, type StartupSource } from "@/lib/api"
import { cn } from "@/lib/utils"
import { CountUp, EASE_OUT_QUINT, FadeUp, Stagger, staggerContainer } from "@/components/motion"
import { Highlight } from "@/components/highlight"
import { IconTooltip } from "@/components/icon-tooltip"
import { Button } from "@/components/ui/button"
import { Input } from "@/components/ui/input"
import { Skeleton } from "@/components/ui/skeleton"
import { Switch } from "@/components/ui/switch"
import { FilterChips } from "@/features/programs/segmented"
import { StatCard } from "@/features/programs/stat-card"

const QUERY_KEY = ["startup-items"]

const sourceLabels: Record<StartupSource, { kind: string; scope: string }> = {
  runUser: { kind: "레지스트리", scope: "현재 사용자" },
  runMachine: { kind: "레지스트리", scope: "모든 사용자" },
  runMachine32: { kind: "레지스트리", scope: "모든 사용자 · 32비트" },
  folderUser: { kind: "시작 폴더", scope: "현재 사용자" },
  folderCommon: { kind: "시작 폴더", scope: "모든 사용자" },
  task: { kind: "예약 작업", scope: "로그인·부팅 때 실행" },
}

type Filter = "all" | "on" | "off" | "missing" | "task"

const quickFilters: { key: Filter; label: string }[] = [
  { key: "all", label: "전체" },
  { key: "on", label: "켜짐" },
  { key: "off", label: "꺼짐" },
  { key: "missing", label: "파일 없음" },
  { key: "task", label: "예약 작업" },
]

const LIST_GRID = "grid grid-cols-[minmax(0,1fr)_10rem_6.5rem_2.5rem] items-center gap-4"

function matches(e: StartupEntry, filter: Filter) {
  if (filter === "on") return e.enabled
  if (filter === "off") return !e.enabled
  if (filter === "missing") return e.targetMissing
  if (filter === "task") return e.source === "task"
  return true
}

function formatAgo(seconds: number) {
  const days = Math.floor((Date.now() / 1000 - seconds) / 86400)
  if (days <= 0) return "오늘 끔"
  if (days < 30) return `${days}일 전에 끔`
  if (days < 365) return `${Math.floor(days / 30)}개월 전에 끔`
  return `${Math.floor(days / 365)}년 전에 끔`
}

function formatDate(seconds: number) {
  return new Date(seconds * 1000).toLocaleString("ko-KR", {
    year: "numeric",
    month: "long",
    day: "numeric",
    hour: "2-digit",
    minute: "2-digit",
  })
}

export function StartupPage() {
  const items = useQuery({ queryKey: QUERY_KEY, queryFn: startupApi.list })
  const [filter, setFilter] = useState<Filter>("all")
  const [query, setQuery] = useState("")
  const searchRef = useRef<HTMLInputElement>(null)
  const loading = items.isLoading

  const all = useMemo(() => items.data ?? [], [items.data])
  const counts = useMemo(
    () =>
      Object.fromEntries(quickFilters.map((f) => [f.key, all.filter((e) => matches(e, f.key)).length])) as Record<
        Filter,
        number
      >,
    [all],
  )
  const visible = useMemo(() => {
    const q = query.trim().toLowerCase()
    return all
      .filter((e) => matches(e, filter))
      .filter(
        (e) =>
          !q ||
          e.name.toLowerCase().includes(q) ||
          (e.publisher ?? "").toLowerCase().includes(q) ||
          (e.author ?? "").toLowerCase().includes(q) ||
          (e.description ?? "").toLowerCase().includes(q),
      )
  }, [all, filter, query])

  const toggleFilter = (key: Filter) => setFilter((f) => (f === key ? "all" : key))

  return (
    <Stagger className="relative mx-auto flex max-w-6xl min-w-0 flex-col gap-6 px-7 pt-7 pb-10">
      <FadeUp>
        <h1 className="text-2xl font-bold tracking-tight">시작프로그램</h1>
        <p className="mt-1 text-sm text-muted-foreground">
          Windows에 로그인하거나 켤 때 함께 실행되는 프로그램과 예약 작업이에요. 꺼도 프로그램이나 등록 정보는 지우지
          않아서 언제든 다시 켤 수 있어요.
        </p>
      </FadeUp>

      <motion.div variants={staggerContainer} className="grid gap-4 sm:grid-cols-3">
        <StatCard
          icon={Power}
          tone="chart-1"
          label="켜진 항목"
          value={loading ? null : <CountUp value={counts.on} />}
          unit="개"
          hint={filter === "on" ? "필터 적용 중 · 다시 누르면 해제" : "로그인할 때 실행돼요"}
          active={filter === "on"}
          onClick={() => toggleFilter("on")}
        />
        <StatCard
          icon={PowerOff}
          tone="chart-2"
          label="꺼진 항목"
          value={loading ? null : <CountUp value={counts.off} />}
          unit="개"
          hint={filter === "off" ? "필터 적용 중 · 다시 누르면 해제" : "다시 켜면 원래대로 돌아가요"}
          active={filter === "off"}
          onClick={() => toggleFilter("off")}
        />
        <StatCard
          icon={FileWarning}
          tone={counts.missing > 0 ? "chart-3" : "chart-5"}
          label="파일 없음"
          value={loading ? null : <CountUp value={counts.missing} />}
          unit="개"
          hint={
            filter === "missing"
              ? "필터 적용 중 · 다시 누르면 해제"
              : counts.missing > 0
                ? "실행할 파일이 사라진 항목이에요"
                : "모두 정상이에요"
          }
          active={filter === "missing"}
          onClick={counts.missing > 0 || filter === "missing" ? () => toggleFilter("missing") : undefined}
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
                  ref={searchRef}
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
                    onClick={() => {
                      setQuery("")
                      searchRef.current?.focus()
                    }}
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
              {/* 프로그램 화면의 필터 칩과 선택 표시 애니메이션이 섞이지 않게 묶는다 */}
              <LayoutGroup id="startup">
                <FilterChips options={quickFilters} value={filter} counts={counts} onChange={setFilter} />
              </LayoutGroup>
            </div>

            {visible.length > 0 && (
              <div
                className={cn(LIST_GRID, "border-t bg-muted/30 px-5 py-2 text-xs font-medium text-muted-foreground")}
              >
                <span>이름</span>
                <span>등록 위치</span>
                <span>상태</span>
                <span className="sr-only">켜기·끄기</span>
              </div>
            )}
          </div>

          {items.isError ? (
            <p className="px-5 py-16 text-center text-sm text-destructive">
              시작 항목을 읽지 못했어요: {String(items.error)}
            </p>
          ) : loading ? (
            <div className="divide-y">
              {Array.from({ length: 6 }, (_, i) => (
                <div key={i} className={cn(LIST_GRID, "px-5 py-3")}>
                  <div className="flex items-center gap-3">
                    <Skeleton className="size-9 rounded-lg" />
                    <Skeleton className="h-4 w-40" />
                  </div>
                  <Skeleton className="h-4 w-28" />
                  <Skeleton className="h-4 w-16" />
                  <Skeleton className="h-5 w-9 rounded-full" />
                </div>
              ))}
            </div>
          ) : visible.length === 0 ? (
            <div className="flex flex-col items-center gap-2 px-5 py-16 text-center">
              <div className="mb-1 flex size-12 items-center justify-center rounded-2xl bg-muted">
                {all.length === 0 ? (
                  <Rocket className="size-6 text-muted-foreground" />
                ) : (
                  <SearchX className="size-6 text-muted-foreground" />
                )}
              </div>
              <p className="font-medium">
                {all.length === 0 ? "로그인할 때 실행되는 프로그램이 없어요" : "조건에 맞는 항목이 없어요"}
              </p>
              {all.length > 0 && (
                <Button
                  variant="outline"
                  size="sm"
                  className="mt-2 rounded-lg"
                  onClick={() => {
                    setFilter("all")
                    setQuery("")
                  }}
                >
                  필터 초기화
                </Button>
              )}
            </div>
          ) : (
            <ul className="divide-y">
              {visible.map((entry) => (
                <StartupRow key={entry.id} entry={entry} query={query.trim()} />
              ))}
            </ul>
          )}
        </section>
      </FadeUp>
    </Stagger>
  )
}

function hueOf(name: string) {
  let h = 0
  for (const ch of name) h = (h * 31 + ch.codePointAt(0)!) % 360
  return h
}

function StartupIcon({ entry }: { entry: StartupEntry }) {
  const icon = useQuery({
    queryKey: ["startup-icon", entry.id],
    queryFn: () => startupApi.icon(entry.id),
    staleTime: Infinity,
  })

  if (icon.data) {
    return (
      <div className="flex size-9 shrink-0 items-center justify-center">
        <img src={icon.data} alt="" className="size-[80%]" draggable={false} />
      </div>
    )
  }
  const hue = hueOf(entry.name)
  return (
    <div
      className="flex size-9 shrink-0 items-center justify-center rounded-lg text-sm font-semibold"
      style={{ background: `oklch(0.72 0.12 ${hue} / 16%)`, color: `oklch(0.82 0.12 ${hue})` }}
    >
      {[...entry.name.trim()][0]?.toUpperCase() ?? "?"}
    </div>
  )
}

function StartupRow({ entry, query }: { entry: StartupEntry; query: string }) {
  const queryClient = useQueryClient()
  const [open, setOpen] = useState(false)
  const [busy, setBusy] = useState(false)
  const machine = isMachineStartup(entry)
  const source = sourceLabels[entry.source]
  const isTask = entry.source === "task"
  const subtitle = [entry.publisher ?? entry.author, entry.description !== entry.name ? entry.description : null]
    .filter(Boolean)
    .join(" · ")

  const toggle = async (enabled: boolean) => {
    setBusy(true)
    try {
      const updated = await startupApi.setEnabled(entry.id, enabled)
      queryClient.setQueryData<StartupEntry[]>(QUERY_KEY, (old) => old?.map((e) => (e.id === updated.id ? updated : e)))
      toast.success(enabled ? `다시 켰어요 · ${entry.name}` : `껐어요 · ${entry.name}`, {
        description: enabled ? "다음 로그인부터 다시 실행돼요" : "다음 로그인부터 실행되지 않아요",
      })
    } catch (e) {
      toast.error(String(e))
    } finally {
      setBusy(false)
    }
  }

  const reveal = () => startupApi.reveal(entry.id).catch((e) => toast.error(String(e)))
  const inFolder = entry.source === "folderUser" || entry.source === "folderCommon"

  return (
    <li className={cn(!entry.enabled && "bg-muted/20")}>
      <div className={cn(LIST_GRID, "px-5 py-2.5")}>
        <button
          type="button"
          onClick={() => setOpen((v) => !v)}
          aria-expanded={open}
          className="flex min-w-0 items-center gap-3 text-left"
        >
          <ChevronRight
            className={cn(
              "-ml-1 size-4 shrink-0 text-muted-foreground transition-transform duration-200",
              open && "rotate-90",
            )}
          />
          <div className={cn("transition-opacity", !entry.enabled && "opacity-50 grayscale")}>
            <StartupIcon entry={entry} />
          </div>
          <div className="min-w-0">
            <p className="flex items-center gap-2 font-medium">
              <span className="truncate">
                <Highlight text={entry.name} query={query} />
              </span>
              {entry.targetMissing && (
                <span className="shrink-0 rounded-full bg-chart-3/12 px-1.5 py-px text-[10px] font-medium text-chart-3">
                  파일 없음
                </span>
              )}
            </p>
            <p className="truncate text-xs text-muted-foreground">
              {subtitle ? <Highlight text={subtitle} query={query} /> : "게시자 정보 없음"}
            </p>
          </div>
        </button>

        <div className="min-w-0 text-xs">
          <p className="truncate">{source.kind}</p>
          <p className="flex items-center gap-1 truncate text-muted-foreground">
            {source.scope}
            {machine && (
              <IconTooltip label="바꾸려면 관리자 권한이 필요해요" side="top">
                <ShieldAlert className="size-3.5 shrink-0 text-chart-3" />
              </IconTooltip>
            )}
          </p>
        </div>

        <div className="text-xs">
          {entry.enabled ? (
            <span className="font-medium text-primary">켜짐</span>
          ) : (
            <>
              <p className="font-medium text-muted-foreground">꺼짐</p>
              {entry.disabledAt && <p className="text-muted-foreground/80">{formatAgo(entry.disabledAt)}</p>}
            </>
          )}
        </div>

        <div className="flex justify-end">
          {busy ? (
            <Loader2 className="size-4 animate-spin text-muted-foreground" />
          ) : (
            <Switch
              checked={entry.enabled}
              onCheckedChange={toggle}
              aria-label={`${entry.name} ${entry.enabled ? "끄기" : "켜기"}`}
            />
          )}
        </div>
      </div>

      <AnimatePresence initial={false}>
        {open && (
          <motion.div
            initial={{ height: 0, opacity: 0 }}
            animate={{ height: "auto", opacity: 1 }}
            exit={{ height: 0, opacity: 0 }}
            transition={{ duration: 0.3, ease: EASE_OUT_QUINT }}
            className="overflow-hidden"
          >
            <dl className="mx-5 mb-3 grid grid-cols-[6rem_minmax(0,1fr)] gap-x-4 gap-y-2 rounded-xl border bg-muted/30 px-4 py-3 text-xs">
              <dt className="text-muted-foreground">실행 명령</dt>
              <dd data-selectable className="font-mono break-all">
                {entry.command}
              </dd>
              <dt className="text-muted-foreground">{isTask ? "작업 경로" : "등록 위치"}</dt>
              <dd data-selectable className="font-mono break-all">
                {entry.location}
                {" › "}
                {entry.approvedName}
              </dd>
              {entry.author && (
                <>
                  <dt className="text-muted-foreground">만든 이</dt>
                  <dd data-selectable className="break-all">
                    {entry.author}
                  </dd>
                </>
              )}
              {entry.targetPath && (
                <>
                  <dt className="text-muted-foreground">실행 파일</dt>
                  <dd data-selectable className="font-mono break-all">
                    {entry.targetPath}
                    {entry.targetMissing && <span className="ml-2 font-sans text-chart-3">(파일이 없어요)</span>}
                  </dd>
                </>
              )}
              {!entry.enabled && (
                <>
                  <dt className="text-muted-foreground">꺼진 상태</dt>
                  <dd>
                    {entry.disabledAt ? `${formatDate(entry.disabledAt)}에 끔` : "꺼져 있어요"}
                    {entry.changedByUs
                      ? " · RobinUninstaller가 끔 (다시 켜면 원래 값으로 되돌려요)"
                      : isTask
                        ? " · 작업 스케줄러 등 다른 곳에서 끔"
                        : " · 작업 관리자 등 다른 곳에서 끔"}
                  </dd>
                </>
              )}
              <dd className="col-span-2 mt-1 flex gap-2">
                <Button
                  size="sm"
                  variant="outline"
                  className="h-7"
                  onClick={reveal}
                  disabled={!inFolder && (entry.targetMissing || !entry.targetPath)}
                >
                  <FolderOpen />
                  {inFolder ? "바로 가기 위치 열기" : "파일 위치 열기"}
                </Button>
              </dd>
            </dl>
          </motion.div>
        )}
      </AnimatePresence>
    </li>
  )
}
