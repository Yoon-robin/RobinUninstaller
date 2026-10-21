import { useEffect, useMemo, useState } from "react"
import { useQuery, useQueryClient } from "@tanstack/react-query"
import {
  Check,
  Copy,
  ChevronRight,
  CircleSlash,
  FolderOpen,
  Info,
  Loader2,
  RefreshCw,
  Settings2,
  ShieldAlert,
  Sparkles,
  TriangleAlert,
} from "lucide-react"
import { DrawnCheck } from "@/components/drawn-check"
import { AnimatePresence, motion } from "motion/react"
import { toast } from "sonner"

import { storageApi, type CleanEvent, type CleanResult, type ScanProgress, type StorageCategory } from "@/lib/api"
import { formatBytes, formatCount, formatDuration } from "@/lib/format"
import { cn } from "@/lib/utils"
import { EASE_OUT_QUINT, FadeUp, Stagger } from "@/components/motion"
import { IconTooltip } from "@/components/icon-tooltip"
import { Button } from "@/components/ui/button"
import { Checkbox } from "@/components/ui/checkbox"
import { Dialog, DialogContent, DialogDescription, DialogTitle } from "@/components/ui/dialog"
import { Skeleton } from "@/components/ui/skeleton"
import { ScanProgressView, shortenPath } from "@/components/scan-progress"

import { FolderAnalysisCard } from "./folder-analysis"
import { LargeFilesCard } from "./large-files"
import { BigProgramsCard } from "./big-programs"
import { ErrorState, StaleBanner, InlineError } from "@/components/error-state"
import { useBusy } from "@/lib/busy"

const SCAN_KEY = ["storage-scan"]
const DISK_KEY = ["disk-usage"]

/** 정리할 수 있는지 (실행 중이 아니고, 안내만 하는 항목이 아님) */
const cleanable = (c: StorageCategory) => c.kind !== "info" && c.running.length === 0

/** 실행 파일 이름 → 사람이 아는 이름 */
const APP_NAMES: Record<string, string> = {
  "chrome.exe": "Chrome",
  "msedge.exe": "Edge",
  "brave.exe": "Brave",
  "whale.exe": "웨일",
  "firefox.exe": "Firefox",
  "discord.exe": "Discord",
  "code.exe": "VS Code",
  "slack.exe": "Slack",
  "spotify.exe": "Spotify",
  "steam.exe": "Steam",
  "steamwebhelper.exe": "Steam",
  "pip.exe": "pip",
  "cargo.exe": "Cargo",
  "rustc.exe": "Rust 빌드",
  "java.exe": "Java(Gradle)",
  "studio64.exe": "Android Studio",
  "dotnet.exe": ".NET",
  "devenv.exe": "Visual Studio",
}
const runningNames = (exes: string[]) => [...new Set(exes.map((e) => APP_NAMES[e] ?? e))].join(", ")

export function StoragePage() {
  const queryClient = useQueryClient()
  // 처음 찾을 때는 다 셀 때까지 기다리지 않고, 찾은 항목부터 크기 순으로 보여 준다
  const [partial, setPartial] = useState<StorageCategory[]>([])
  // 지금 재는 항목과 센 파일 수 (멈춘 것처럼 보이지 않게)
  const [progress, setProgress] = useState<ScanProgress | null>(null)
  const scan = useQuery({
    queryKey: SCAN_KEY,
    queryFn: () => {
      setPartial([])
      setProgress(null)
      return storageApi.scan((item) => setPartial((p) => [...p, item]), setProgress)
    },
  })
  // 찾는 동안은 찾은 순서대로 아래에 붙이고(누르려던 줄이 밀리지 않게), 다 찾으면 한 번 크기순으로 정렬한다
  const data = useMemo(
    () => (scan.data ? [...scan.data].sort((a, b) => b.bytes - a.bytes) : partial.length > 0 ? partial : undefined),
    [scan.data, partial],
  )
  const finding = scan.isLoading
  const disk = useQuery({ queryKey: DISK_KEY, queryFn: storageApi.diskUsage })
  // 처음부터 찾지 못했을 때만 오류 화면. 이미 찾은 목록이 있으면 그대로 두고 위에 띠만 보인다.
  const scanFailed = scan.isError && scan.data == null
  const scanStale = scan.isError && scan.data != null
  // 사용자가 직접 바꾼 선택. 없으면 기본 선택(다시 만들어지는 항목)을 따른다.
  const [picked, setPicked] = useState<Record<string, boolean>>({})
  // 확인 창을 연 순간의 항목. 그 뒤에 다시 찾기가 끝나도 창 안의 목록·합계가 바뀌지 않는다 (보지 못한 항목이 끼어들지 않게)
  const [confirming, setConfirming] = useState<StorageCategory[]>([])
  const [confirmOpen, setConfirmOpen] = useState(false)

  const items = useMemo(() => (data ?? []).filter((c) => c.kind !== "info"), [data])
  const infos = useMemo(() => (data ?? []).filter((c) => c.kind === "info"), [data])
  const isSelected = (c: StorageCategory) => cleanable(c) && (picked[c.id] ?? c.defaultSelected)
  const selected = items.filter(isSelected)
  const selectedBytes = selected.reduce((s, c) => s + c.bytes, 0)

  const refresh = () => {
    queryClient.invalidateQueries({ queryKey: SCAN_KEY })
    queryClient.invalidateQueries({ queryKey: DISK_KEY })
  }

  return (
    <Stagger className="relative mx-auto flex max-w-5xl min-w-0 flex-col gap-6 px-7 pt-7 pb-10">
      <FadeUp>
        <h1 className="text-2xl font-bold tracking-tight">저장공간</h1>
        <p className="mt-1 text-sm text-muted-foreground">
          캐시·임시 파일처럼 지워도 다시 만들어지는 것을 미리 골라 두고, 휴지통처럼 되돌릴 수 없는 항목은 한 번 더
          확인한 뒤 지워요. 문서·사진·다운로드 같은 내 파일은 지우지 않아요.
        </p>
      </FadeUp>

      <FadeUp>
        {disk.isError && !disk.data ? (
          <div className="rounded-2xl border bg-card px-5 py-4 shadow-sm">
            <InlineError
              message="디스크 사용량을 읽지 못했어요"
              error={disk.error}
              onRetry={() => void disk.refetch()}
              retrying={disk.isFetching}
            />
          </div>
        ) : (
          <DriveCard used={disk.data ? disk.data.total - disk.data.free : null} disk={disk.data} plan={selectedBytes} />
        )}
      </FadeUp>

      <FadeUp>
        <section className="overflow-clip rounded-2xl border bg-card shadow-sm">
          <div className="flex items-center gap-3 border-b px-5 py-4">
            <div className="mr-auto">
              <h2 className="font-semibold">정리할 수 있는 항목</h2>
              <p className="text-xs text-muted-foreground">다시 만들어지는 항목만 미리 골라 두었어요.</p>
            </div>
            <IconTooltip label="다시 찾기">
              <Button
                variant="outline"
                size="icon"
                className="size-9 rounded-lg"
                onClick={refresh}
                aria-label="다시 찾기"
              >
                <RefreshCw className={cn(scan.isFetching && "animate-spin")} />
              </Button>
            </IconTooltip>
          </div>

          {scanStale && (
            <StaleBanner error={scan.error} onRetry={() => void scan.refetch()} retrying={scan.isFetching} />
          )}
          {scanFailed ? (
            <ErrorState
              title="정리할 수 있는 항목을 찾지 못했어요"
              error={scan.error}
              onRetry={() => scan.refetch()}
              retrying={scan.isFetching}
            />
          ) : finding && items.length === 0 ? (
            <ScanProgressView
              className="px-5 py-6"
              label="정리할 수 있는 항목을 찾는 중이에요"
              progress={progress}
              showBytes={false}
            />
          ) : items.length === 0 ? (
            <div className="flex flex-col items-center gap-2 px-5 py-14 text-center">
              <div className="mb-1 flex size-12 items-center justify-center rounded-2xl bg-muted">
                <Sparkles className="size-6 text-primary" />
              </div>
              <p className="font-medium">정리할 게 없어요</p>
              <p className="text-sm text-muted-foreground">
                지금은 정리할 만한 캐시와 임시 파일을 찾지 못했어요. 아래 "용량이 큰 프로그램"과 "큰 폴더 찾기"에서
                공간을 많이 쓰는 곳을 볼 수 있어요.
              </p>
            </div>
          ) : (
            <ul className="divide-y">
              {/* 찾은 항목이 계속 늘어나므로 진행 표시는 목록 위에 둔다 (아래에 두면 화면 밖으로 밀려난다) */}
              {scan.isFetching && (
                <li className="bg-muted/20 px-5 py-4">
                  <ScanProgressView
                    label={finding ? "나머지 항목을 찾는 중이에요" : "다시 찾는 중이에요"}
                    progress={progress}
                    showBytes={false}
                  />
                </li>
              )}
              {items.map((c) => (
                <CategoryRow
                  key={c.id}
                  category={c}
                  checked={isSelected(c)}
                  onCheckedChange={(v) => setPicked((p) => ({ ...p, [c.id]: v }))}
                  onRecheck={refresh}
                  rechecking={scan.isFetching}
                />
              ))}
            </ul>
          )}

          {items.length > 0 && (
            <div className="sticky bottom-0 flex items-center gap-3 border-t bg-card px-5 py-3">
              <p className="mr-auto text-sm">
                <span className="text-muted-foreground">{selected.length}개 선택 · </span>
                <span className="font-semibold tabular-nums">{formatBytes(selectedBytes)}</span>
                <span className="text-muted-foreground"> 확보 예상</span>
              </p>
              {/* 다시 찾는 동안에는 목록이 곧 바뀌므로 정리를 시작하지 않는다 */}
              <Button
                disabled={selected.length === 0 || scan.isFetching}
                onClick={() => {
                  setConfirming(selected)
                  setConfirmOpen(true)
                }}
              >
                <Sparkles />
                정리하기
              </Button>
            </div>
          )}
        </section>
      </FadeUp>

      <FadeUp>
        <BigProgramsCard />
      </FadeUp>

      {infos.length > 0 && (
        <FadeUp>
          <InfoSection items={infos} />
        </FadeUp>
      )}

      <FadeUp>
        <FolderAnalysisCard />
      </FadeUp>

      <FadeUp>
        <LargeFilesCard />
      </FadeUp>

      <CleanDialog
        open={confirmOpen}
        items={confirming}
        onClose={() => setConfirmOpen(false)}
        onDone={() => {
          setPicked({})
          refresh()
        }}
      />
    </Stagger>
  )
}

function DriveCard({
  disk,
  used,
  plan,
}: {
  disk: { drive: string; total: number; free: number } | undefined
  used: number | null
  plan: number
}) {
  if (!disk || used == null) return <Skeleton className="h-28 w-full rounded-2xl" />
  const usedPct = (used / disk.total) * 100
  const planPct = Math.min((plan / disk.total) * 100, usedPct)
  // 남은 공간이 10% 아래면 경고색으로 (Windows 업데이트·임시 파일이 실패하기 시작하는 즈음)
  const low = disk.free / disk.total < 0.1

  return (
    <div className="rounded-2xl border bg-card p-5 shadow-sm">
      <div className="flex flex-wrap items-baseline gap-x-3 gap-y-1">
        <p className="font-semibold">로컬 디스크 ({disk.drive.replace(/\\$/, "")})</p>
        <p className="text-sm text-muted-foreground">
          {formatBytes(disk.total)} 중{" "}
          <b className={cn("tabular-nums", low ? "text-warning" : "text-foreground")}>{formatBytes(disk.free)}</b> 남음
          {low && <span className="ml-1.5 font-medium text-warning">· 거의 꽉 찼어요</span>}
        </p>
        {plan > 0 && (
          <p className="ml-auto text-sm text-primary tabular-nums">
            다 정리되면 약 {formatBytes(disk.free + plan)} 남음
          </p>
        )}
      </div>
      <div className="relative mt-3 h-3 overflow-hidden rounded-full bg-muted">
        <motion.div
          className={cn(
            "forced-fill-used absolute inset-y-0 left-0 overflow-hidden rounded-full",
            low ? "bg-chart-3" : "bg-chart-2",
          )}
          initial={{ width: 0 }}
          animate={{ width: `${usedPct}%` }}
          transition={{ duration: 0.8, ease: EASE_OUT_QUINT }}
        >
          {/* 정리하면 비는 부분: 사용 중 막대 안쪽 끝에 붙여 둔다 (따로 두면 처음에 막대와 떨어져 떠 보였다).
              폭은 사용 중 막대에 대한 비율이고, 오른쪽 끝을 기준으로 늘고 준다. */}
          <div
            className="forced-fill absolute inset-y-0 right-0 w-full origin-right bg-primary/70 transition-transform duration-300 ease-(--ease-out-quint)"
            style={{ transform: `scaleX(${usedPct > 0 ? planPct / usedPct : 0})` }}
          />
        </motion.div>
      </div>
      <div className="mt-2 flex gap-4 text-xs text-muted-foreground">
        <span className="flex items-center gap-1.5">
          <span className="forced-fill-used size-2 rounded-full bg-chart-2" />
          사용 중 {formatBytes(used)}
        </span>
        {plan > 0 && (
          <span className="flex items-center gap-1.5">
            <span className="forced-fill size-2 rounded-full bg-primary" />
            정리 예정 {formatBytes(plan)}
          </span>
        )}
      </div>
    </div>
  )
}

function RiskBadge({ category: c }: { category: StorageCategory }) {
  return c.risk === "regenerable" ? (
    <span className="rounded-full bg-primary/12 px-1.5 py-px text-[11px] font-medium text-primary">
      다시 만들어져요
    </span>
  ) : (
    <span className="rounded-full bg-destructive/12 px-1.5 py-px text-[11px] font-medium text-destructive">
      되돌릴 수 없어요
    </span>
  )
}

function CategoryRow({
  category: c,
  checked,
  onCheckedChange,
  onRecheck,
  rechecking,
}: {
  category: StorageCategory
  checked: boolean
  onCheckedChange: (v: boolean) => void
  /** 실행 중이라 막힌 항목: 프로그램을 닫은 뒤 다시 확인 */
  onRecheck: () => void
  rechecking: boolean
}) {
  const [open, setOpen] = useState(false)
  const blocked = c.running.length > 0
  const reveal = () => storageApi.revealCategory(c.id).catch((e) => toast.error(String(e)))

  return (
    <li className={cn(blocked && "bg-muted/20")}>
      <div className="flex items-center gap-4 px-5 py-3">
        <Checkbox
          checked={checked}
          disabled={blocked}
          onCheckedChange={(v) => onCheckedChange(v === true)}
          aria-label={`${c.name} 선택`}
        />
        <button
          type="button"
          onClick={() => setOpen((v) => !v)}
          aria-expanded={open}
          className="flex min-w-0 flex-1 items-center gap-2 text-left"
        >
          <div className="min-w-0 flex-1">
            <p className="flex flex-wrap items-center gap-1.5 font-medium">
              {c.name}
              <RiskBadge category={c} />
              {c.admin && (
                <IconTooltip label="관리자 권한이 필요해요 (UAC 창이 떠요)" side="top">
                  <ShieldAlert className="size-3.5 text-warning" />
                </IconTooltip>
              )}
            </p>
            <p className="truncate text-xs text-muted-foreground">
              {blocked ? `${runningNames(c.running)} 실행 중 · 닫으면 정리할 수 있어요` : c.description}
            </p>
          </div>
          <span className="shrink-0 text-right text-sm font-semibold tabular-nums">
            {c.bytes > 0 ? formatBytes(c.bytes) : "—"}
            {c.inaccessible > 0 && c.admin && (
              <span className="block text-[11px] font-normal text-muted-foreground">일부만 셌어요</span>
            )}
          </span>
          <ChevronRight
            className={cn(
              "size-4 shrink-0 text-muted-foreground transition-transform duration-300 ease-(--ease-out-quint)",
              open && "rotate-90",
            )}
          />
        </button>
        {blocked && (
          <Button variant="outline" size="sm" onClick={onRecheck} disabled={rechecking}>
            <RefreshCw className={cn(rechecking && "animate-spin")} />
            다시 확인
          </Button>
        )}
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
            <div className="mx-5 mb-3 ml-13 space-y-2 rounded-xl border bg-muted/30 px-4 py-3 text-xs">
              <p>
                <span className="font-medium">
                  {c.risk === "irreversible" ? "지우면 어떻게 되나요? " : "왜 지워도 되나요? "}
                </span>
                {c.why}
              </p>
              {c.files > 0 && <p className="text-muted-foreground">파일 {formatCount(c.files)}개</p>}
              {c.locations.length > 0 && (
                <ul data-selectable className="space-y-0.5 font-mono break-all text-muted-foreground">
                  {c.locations.map((l) => (
                    <li key={l}>{l}</li>
                  ))}
                </ul>
              )}
              {c.locations.length > 0 && (
                <Button size="sm" variant="outline" className="h-7" onClick={reveal}>
                  <FolderOpen />
                  위치 열기
                </Button>
              )}
            </div>
          </motion.div>
        )}
      </AnimatePresence>
    </li>
  )
}

/** 안내 문장. `…`로 감싼 명령은 코드 모양과 복사 버튼으로 보여 준다 (문장에 묻혀 고르기 어렵지 않게). */
function Guidance({ text }: { text: string }) {
  const parts = text.split("`")
  return (
    <p data-selectable className="mt-1.5 text-xs leading-6">
      {parts.map((part, i) => (i % 2 === 1 ? <Command key={i} command={part} /> : part))}
    </p>
  )
}

function Command({ command }: { command: string }) {
  const [copied, setCopied] = useState(false)
  return (
    <span className="mx-0.5 inline-flex items-center gap-1 rounded-md border bg-muted/60 py-0.5 pr-0.5 pl-1.5 align-middle">
      <code className="font-mono text-[11px]">{command}</code>
      <IconTooltip label={copied ? "복사했어요" : "명령 복사"}>
        <button
          type="button"
          aria-label={copied ? "명령을 복사했어요" : `${command} 복사`}
          className="flex size-5 items-center justify-center rounded text-muted-foreground hover:bg-muted hover:text-foreground"
          onClick={() =>
            navigator.clipboard.writeText(command).then(
              () => {
                setCopied(true)
                setTimeout(() => setCopied(false), 1200)
              },
              (e) => toast.error(String(e)),
            )
          }
        >
          {copied ? <Check className="size-3 text-success" /> : <Copy className="size-3" />}
        </button>
      </IconTooltip>
    </span>
  )
}

function InfoSection({ items }: { items: StorageCategory[] }) {
  return (
    <section className="rounded-2xl border bg-card shadow-sm">
      <div className="border-b px-5 py-4">
        <h2 className="font-semibold">직접 정리할 수 있는 큰 항목</h2>
        <p className="text-xs text-muted-foreground">
          지우면 되돌릴 수 없거나 Windows 설정에서 해야 해서, 여기서는 크기와 방법만 알려 드려요.
        </p>
      </div>
      <ul className="divide-y">
        {items.map((c) => (
          <li key={c.id} className="flex items-start gap-4 px-5 py-4">
            <Info className="mt-0.5 size-4 shrink-0 text-muted-foreground" />
            <div className="min-w-0 flex-1 text-sm">
              <p className="font-medium">
                {c.name}{" "}
                <span className="ml-1 font-semibold tabular-nums">
                  {c.bytes > 0 ? (
                    formatBytes(c.bytes)
                  ) : (
                    <span className="font-normal text-muted-foreground">
                      {c.admin ? "크기를 세려면 관리자 권한이 필요해요" : "크기를 세지 못했어요"}
                    </span>
                  )}
                </span>
              </p>
              <p className="mt-0.5 text-xs text-muted-foreground">
                {c.description}. {c.why}
              </p>
              {c.guidance && <Guidance text={c.guidance} />}
            </div>
            {c.id === "hibernation" ? null : c.id === "windows-old" ? (
              <Button size="sm" variant="outline" onClick={() => storageApi.openSettings()}>
                <Settings2 />
                저장소 설정 열기
              </Button>
            ) : (
              <Button
                size="sm"
                variant="outline"
                onClick={() => storageApi.revealCategory(c.id).catch((e) => toast.error(String(e)))}
              >
                <FolderOpen />
                위치 열기
              </Button>
            )}
          </li>
        ))}
      </ul>
    </section>
  )
}

function summarize(results: CleanResult[]) {
  const freed = results.reduce((s, r) => s + r.freedBytes, 0)
  const failed = results.reduce((s, r) => s + r.failedFiles, 0)
  const skipped = results.filter((r) => r.skipped)
  return { freed, failed, skipped }
}

function CleanDialog({
  open,
  items: selectedItems,
  onClose,
  onDone,
}: {
  open: boolean
  items: StorageCategory[]
  onClose: () => void
  onDone: () => void
}) {
  const [busy, setBusy] = useState(false)
  useBusy(busy)
  // 정리를 시작할 때 고른 항목. 끝나면 화면이 선택을 비우고 다시 찾으므로, 결과는 이 목록으로 보여 준다
  const [ranItems, setRanItems] = useState<StorageCategory[] | null>(null)
  const items = ranItems ?? selectedItems
  const [understood, setUnderstood] = useState(false)
  // 정리가 끝나면 창을 닫지 않고 항목별 결과를 보여 준다
  const [results, setResults] = useState<CleanResult[] | null>(null)
  // 정리 전후 시스템 드라이브의 남은 공간 차이 (실제로 늘어난 공간)
  const [measured, setMeasured] = useState<{ drive: string; gained: number } | null>(null)
  const [adminError, setAdminError] = useState<string | null>(null)
  // 정리하는 동안의 진행: 지금 항목, 지금까지 센 것, 끝난 항목들
  const [live, setLive] = useState<LiveProgress | null>(null)
  // 되돌릴 수 없는 항목의 확인은 창을 열 때마다 새로 받는다
  useEffect(() => {
    if (open) {
      setRanItems(null)
      setUnderstood(false)
      setResults(null)
      setMeasured(null)
      setAdminError(null)
      setLive(null)
    }
  }, [open])
  const irreversible = items.filter((c) => c.risk === "irreversible")
  const needsAdmin = items.some((c) => c.admin)
  // 되돌릴 수 없는 항목은 체크해야 지우므로, 합계도 실제로 지울 것만 센다
  const total = items.filter((c) => c.risk !== "irreversible" || understood).reduce((s, c) => s + c.bytes, 0)
  // 되돌릴 수 없는 항목만 골랐다면 확인 체크 없이는 정리할 게 없다
  const onlyIrreversible = irreversible.length === items.length

  const run = async () => {
    if (busy) return
    // 먼저 잠근다: 남은 공간을 재는 동안 두 번 눌러 정리가 두 번 시작되지 않게
    setBusy(true)
    setRanItems(selectedItems)
    const before = await storageApi.diskUsage().catch(() => null)
    const allow = irreversible.length > 0 && understood
    setLive({ startedAt: Date.now(), stage: "normal", current: null, done: [] })
    const onEvent = (e: CleanEvent) =>
      setLive((l) => {
        if (!l) return l
        if (e.kind === "started")
          return {
            ...l,
            current: {
              id: e.id,
              index: e.index,
              total: e.total,
              freed: 0,
              deleted: 0,
              failed: 0,
              file: null,
              since: Date.now(),
            },
          }
        if (e.kind === "progress" && l.current?.id === e.id)
          return {
            ...l,
            current: {
              ...l.current,
              freed: e.freedBytes,
              deleted: e.deletedFiles,
              failed: e.failedFiles,
              file: e.current,
            },
          }
        if (e.kind === "finished") return { ...l, current: null, done: [...l.done, e.result] }
        return l
      })
    try {
      const results = await storageApi.clean(
        items.map((c) => c.id),
        allow,
        onEvent,
      )
      const adminIds = results.filter((r) => r.needsAdmin).map((r) => r.id)
      let all = results.filter((r) => !r.needsAdmin)
      if (adminIds.length > 0) {
        // 관리자 권한 작업자는 따로 도는 프로세스라 중간 소식이 없다. 걸린 시간만 보여 준다.
        setLive((l) => (l ? { ...l, stage: "admin", current: null } : l))
        try {
          all = all.concat(await storageApi.cleanElevated(adminIds, allow))
        } catch (e) {
          setAdminError(String(e))
          // 관리자 권한을 받지 못한 항목도 결과 목록에 "건너뜀"으로 남긴다
          all = all.concat(results.filter((r) => r.needsAdmin))
        }
      }
      setResults(all)
      const after = await storageApi.diskUsage().catch(() => null)
      setMeasured(before && after ? { drive: after.drive, gained: after.free - before.free } : null)
      onDone()
    } catch (e) {
      toast.error(String(e))
    } finally {
      setBusy(false)
      setUnderstood(false)
    }
  }

  return (
    <Dialog
      open={open}
      onOpenChange={(v) => {
        if (!v && !busy) {
          setUnderstood(false)
          onClose()
        }
      }}
    >
      {/* 안쪽 칸이 긴 경로 때문에 대화상자 밖으로 넓어지지 않게 (grid 칸은 기본으로 내용보다 줄지 않는다) */}
      <DialogContent className="sm:max-w-md *:min-w-0" showCloseButton={!busy}>
        {busy && live ? (
          <CleanProgressView live={live} items={items.filter((c) => c.risk !== "irreversible" || understood)} />
        ) : results ? (
          <CleanResultView
            results={results}
            measured={measured}
            items={items}
            adminError={adminError}
            onClose={() => {
              setUnderstood(false)
              onClose()
            }}
          />
        ) : (
          <>
            <DialogTitle>
              {onlyIrreversible && !understood
                ? "되돌릴 수 없는 항목을 지울까요?"
                : `${formatBytes(total)} 정리할까요?`}
            </DialogTitle>
            <DialogDescription className="leading-relaxed">
              아래 항목의 파일을 바로 지워요. 지금 쓰고 있는 파일은 건너뛰어요.
            </DialogDescription>
            <ul className="max-h-56 divide-y overflow-y-auto rounded-xl border text-sm">
              {items.map((c) => (
                <li
                  key={c.id}
                  className={cn(
                    "flex items-center gap-2 px-3 py-2",
                    c.risk === "irreversible" && !understood && "text-muted-foreground",
                  )}
                >
                  <span
                    className={cn(
                      "min-w-0 flex-1 truncate",
                      c.risk === "irreversible" && !understood && "line-through decoration-muted-foreground/60",
                    )}
                  >
                    {c.name}
                  </span>
                  <RiskBadge category={c} />
                  <span className="w-16 shrink-0 text-right tabular-nums">{formatBytes(c.bytes)}</span>
                </li>
              ))}
            </ul>
            {irreversible.length > 0 && (
              <label className="flex cursor-pointer items-start gap-2.5 rounded-xl border border-destructive/40 bg-destructive/8 p-3 text-sm">
                <Checkbox checked={understood} onCheckedChange={(v) => setUnderstood(v === true)} className="mt-0.5" />
                <span>
                  <span className="flex items-center gap-1.5 font-medium text-destructive">
                    <TriangleAlert className="size-4" />
                    되돌릴 수 없는 항목이 있어요
                  </span>
                  <span className="mt-1 block text-muted-foreground">
                    지우면 복구할 수 없어요: {irreversible.map((c) => c.name).join(", ")}. 확인했으면 체크해 주세요.{" "}
                    {onlyIrreversible ? "체크해야 정리할 수 있어요." : "체크하지 않으면 이 항목만 빼고 정리해요."}
                  </span>
                </span>
              </label>
            )}
            {needsAdmin && (
              <p className="flex items-center gap-1.5 text-xs text-muted-foreground">
                <ShieldAlert className="size-3.5 text-warning" />
                관리자 권한이 필요한 항목이 있어 UAC 창이 떠요. "아니요"를 누르면 그 항목만 빼고 정리해요.
              </p>
            )}
            <div className="mt-2 flex justify-end gap-2">
              <Button
                variant="outline"
                onClick={() => {
                  setUnderstood(false)
                  onClose()
                }}
                disabled={busy}
              >
                취소
              </Button>
              <Button onClick={run} disabled={busy || (onlyIrreversible && !understood)}>
                {busy ? <Loader2 className="animate-spin" /> : <Sparkles />}
                정리하기
              </Button>
            </div>
          </>
        )}
      </DialogContent>
    </Dialog>
  )
}

interface LiveProgress {
  startedAt: number
  /** normal: 이 앱에서 정리 중, admin: 관리자 권한 작업자가 정리 중 */
  stage: "normal" | "admin"
  current: {
    id: string
    index: number
    total: number
    freed: number
    deleted: number
    failed: number
    /** 방금 지운 파일 */
    file: string | null
    since: number
  } | null
  done: CleanResult[]
}

function useNow(intervalMs: number) {
  const [now, setNow] = useState(() => Date.now())
  useEffect(() => {
    const t = setInterval(() => setNow(Date.now()), intervalMs)
    return () => clearInterval(t)
  }, [intervalMs])
  return now
}

/**
 * 정리하는 동안의 화면. 지금 항목, 지운 파일 수와 크기, 방금 지운 파일, 걸린 시간, 끝난 항목 기록을 보여 줘
 * 오래 걸려도 멈춘 것처럼 보이지 않게 한다. 휴지통과 관리자 권한 정리는 중간 진행을 알 수 없어 시간만 센다.
 */
function CleanProgressView({ live, items }: { live: LiveProgress; items: StorageCategory[] }) {
  const now = useNow(500)
  const nameOf = (id: string) => items.find((c) => c.id === id)?.name ?? id
  const cur = live.current
  const curCategory = cur ? items.find((c) => c.id === cur.id) : undefined
  const doneFreed = live.done.reduce((s, r) => s + r.freedBytes, 0) + (cur?.freed ?? 0)
  const doneFiles = live.done.reduce((s, r) => s + r.deletedFiles, 0) + (cur?.deleted ?? 0)
  const target = items.reduce((s, c) => s + c.bytes, 0)
  // 스캔 때 잰 크기 기준의 대략적인 진행. 끝나기 전에는 100%를 넘기지 않는다.
  const percent = target > 0 ? Math.min(99, Math.round((doneFreed / target) * 100)) : null
  const quietFor = cur ? now - cur.since : 0

  return (
    <>
      <DialogTitle className="flex items-center gap-2">
        <Loader2 className="size-5 animate-spin text-primary" />
        정리하는 중이에요
      </DialogTitle>
      <DialogDescription className="tabular-nums">
        {formatDuration(now - live.startedAt)} 지남 · 지금까지 파일 {formatCount(doneFiles)}개, {formatBytes(doneFreed)}
      </DialogDescription>

      <div className="h-1.5 overflow-hidden rounded-full bg-muted" aria-hidden>
        {percent == null || live.stage === "admin" || curCategory?.kind === "recycleBin" ? (
          <div className="scan-sweep h-full w-1/3 rounded-full bg-primary" />
        ) : (
          <div
            className="forced-fill h-full rounded-full bg-primary transition-[width] duration-300"
            style={{ width: `${percent}%` }}
          />
        )}
      </div>

      {/* 읽어 주는 것은 지금 정리하는 항목 이름뿐이다 (파일 수·경로·시간은 계속 바뀌어 화면에만 보인다) */}
      <p className="sr-only" role="status">
        {live.stage === "admin"
          ? "관리자 권한으로 정리하는 중"
          : cur
            ? `${nameOf(cur.id)} 정리 중, ${cur.total}개 중 ${cur.index + 1}번째`
            : "다음 항목을 준비하는 중"}
      </p>
      <div className="rounded-xl border bg-muted/30 px-3.5 py-3 text-sm">
        {live.stage === "admin" ? (
          <>
            <p className="font-medium">관리자 권한으로 정리하는 중</p>
            <p className="mt-1 text-xs text-muted-foreground">
              UAC 창에서 허용하면 따로 실행된 RobinX가 정리해요. 끝나면 결과가 여기에 나와요.
            </p>
          </>
        ) : cur ? (
          <>
            <p className="flex items-baseline justify-between gap-3">
              <span className="min-w-0 truncate font-medium">{nameOf(cur.id)}</span>
              <span className="shrink-0 text-xs text-muted-foreground tabular-nums">
                {cur.index + 1} / {cur.total}
              </span>
            </p>
            {curCategory?.kind === "recycleBin" ? (
              <p className="mt-1 text-xs text-muted-foreground">
                Windows가 휴지통을 한 번에 비워서 중간 진행은 보이지 않아요.
                {curCategory.bytes > 1e9 ? ` ${formatBytes(curCategory.bytes)}라 몇 분 걸릴 수 있어요.` : ""} (
                {formatDuration(quietFor)} 지남)
              </p>
            ) : (
              <>
                <p className="mt-1 text-xs text-muted-foreground tabular-nums">
                  파일 {formatCount(cur.deleted)}개 지움 · {formatBytes(cur.freed)}
                  {cur.failed > 0 && ` · 사용 중 ${formatCount(cur.failed)}개 건너뜀`}
                </p>
                <p
                  data-selectable
                  className="mt-1 truncate font-mono text-[11px] text-muted-foreground/80"
                  title={cur.file ?? ""}
                >
                  {cur.file ? shortenPath(cur.file) : "파일을 찾는 중"}
                </p>
              </>
            )}
          </>
        ) : (
          <p className="text-muted-foreground">다음 항목을 준비하는 중</p>
        )}
      </div>

      {live.done.length > 0 && (
        <ul className="max-h-40 divide-y overflow-y-auto rounded-xl border text-xs" aria-label="끝난 항목">
          {live.done.map((r) => (
            <li key={r.id} className="flex items-center gap-2 px-3 py-2">
              {r.needsAdmin ? (
                <ShieldAlert className="size-3.5 shrink-0 text-warning" />
              ) : r.skipped ? (
                <CircleSlash className="size-3.5 shrink-0 text-muted-foreground" />
              ) : (
                <Check className="size-3.5 shrink-0 text-success" />
              )}
              <span className="min-w-0 flex-1 truncate">{nameOf(r.id)}</span>
              <span className="shrink-0 text-muted-foreground tabular-nums">
                {r.needsAdmin
                  ? "관리자 권한 필요"
                  : r.skipped
                    ? "건너뜀"
                    : `${formatCount(r.deletedFiles)}개 · ${formatBytes(r.freedBytes)}`}
              </span>
            </li>
          ))}
        </ul>
      )}
      <p className="text-xs text-muted-foreground">정리하는 동안에는 창을 닫을 수 없어요.</p>
    </>
  )
}

/** 정리 결과: 항목마다 확보한 공간, 지운 파일 수, 사용 중이라 건너뛴 파일, 건너뛴 이유 */
function CleanResultView({
  results,
  measured,
  items,
  adminError,
  onClose,
}: {
  results: CleanResult[]
  measured: { drive: string; gained: number } | null
  items: StorageCategory[]
  adminError: string | null
  onClose: () => void
}) {
  const { freed, failed } = summarize(results)
  const drive = measured?.drive.replace(/\\$/, "") ?? "C:"
  return (
    <>
      <DialogTitle className="flex items-center gap-2">
        {freed > 0 ? (
          <>
            <DrawnCheck className="size-5 text-success" />
            {formatBytes(freed)} 지웠어요
          </>
        ) : (
          "지운 파일이 없어요"
        )}
      </DialogTitle>
      {measured && freed > 0 && (
        <p className="rounded-xl bg-muted/50 px-3.5 py-2.5 text-sm">
          {measured.gained > 0 ? (
            <>
              {drive} 드라이브 남은 공간이 실제로{" "}
              <span className="font-semibold text-success tabular-nums">{formatBytes(measured.gained)}</span> 늘었어요.
            </>
          ) : (
            `${drive} 드라이브 남은 공간은 거의 그대로예요.`
          )}{" "}
          <span className="text-muted-foreground">
            지운 양과 다를 수 있어요. 휴지통은 다른 드라이브에도 있고, 그사이 다른 프로그램이 파일을 쓰기도 해요.
          </span>
        </p>
      )}
      <DialogDescription className="leading-relaxed">
        {failed > 0
          ? `지금 쓰고 있는 파일 ${formatCount(failed)}개는 건너뛰었어요. 해당 프로그램을 닫고 다시 정리하면 더 비울 수 있어요.`
          : freed > 0
            ? "고른 항목을 정리했어요. 캐시는 프로그램이 필요할 때 다시 만들어요."
            : "아래에서 건너뛴 이유를 확인해 주세요."}
      </DialogDescription>
      <ul className="max-h-64 divide-y overflow-y-auto rounded-xl border text-sm">
        {results.map((r) => {
          const name = items.find((c) => c.id === r.id)?.name ?? r.id
          return (
            <li key={r.id} className="px-3 py-2.5">
              <div className="flex items-center gap-2">
                <span className="min-w-0 flex-1 truncate font-medium">{name}</span>
                <span
                  className={cn("shrink-0 tabular-nums", r.freedBytes > 0 ? "text-success" : "text-muted-foreground")}
                >
                  {r.freedBytes > 0 ? formatBytes(r.freedBytes) : "—"}
                </span>
              </div>
              <p className="mt-0.5 text-xs text-muted-foreground">
                {r.needsAdmin
                  ? "관리자 권한을 받지 못해 건너뛰었어요"
                  : r.skipped
                    ? r.skipped
                    : [
                        `파일 ${formatCount(r.deletedFiles)}개 지움`,
                        r.failedFiles > 0 ? `사용 중 ${formatCount(r.failedFiles)}개 건너뜀` : null,
                      ]
                        .filter(Boolean)
                        .join(" · ")}
              </p>
            </li>
          )
        })}
      </ul>
      {adminError && (
        <p className="rounded-lg bg-chart-3/10 px-3 py-2 text-xs text-warning">
          관리자 권한 항목은 정리하지 못했어요: {adminError}
        </p>
      )}
      <div className="mt-1 flex justify-end">
        <Button onClick={onClose}>닫기</Button>
      </div>
    </>
  )
}
