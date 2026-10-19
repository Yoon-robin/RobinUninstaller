import { useEffect, useMemo, useState } from "react"
import { useQuery, useQueryClient } from "@tanstack/react-query"
import {
  Check,
  ChevronRight,
  FolderOpen,
  Info,
  Loader2,
  RefreshCw,
  Settings2,
  ShieldAlert,
  Sparkles,
  TriangleAlert,
} from "lucide-react"
import { AnimatePresence, motion } from "motion/react"
import { toast } from "sonner"

import { storageApi, type CleanEvent, type CleanResult, type StorageCategory } from "@/lib/api"
import { formatBytes } from "@/lib/format"
import { cn } from "@/lib/utils"
import { EASE_OUT_QUINT, FadeUp, Stagger } from "@/components/motion"
import { IconTooltip } from "@/components/icon-tooltip"
import { Button } from "@/components/ui/button"
import { Checkbox } from "@/components/ui/checkbox"
import { Dialog, DialogContent, DialogDescription, DialogTitle } from "@/components/ui/dialog"
import { Skeleton } from "@/components/ui/skeleton"

import { FolderAnalysisCard } from "./folder-analysis"
import { LargeFilesCard } from "./large-files"
import { ErrorState } from "@/components/error-state"

const SCAN_KEY = ["storage-scan"]
const DISK_KEY = ["disk-usage"]

/** 정리할 수 있는지 (실행 중이 아니고, 안내만 하는 항목이 아님) */
const cleanable = (c: StorageCategory) => c.kind !== "info" && c.running.length === 0

export function StoragePage() {
  const queryClient = useQueryClient()
  const scan = useQuery({ queryKey: SCAN_KEY, queryFn: storageApi.scan })
  const disk = useQuery({ queryKey: DISK_KEY, queryFn: storageApi.diskUsage })
  // 사용자가 직접 바꾼 선택. 없으면 기본 선택(다시 만들어지는 항목)을 따른다.
  const [picked, setPicked] = useState<Record<string, boolean>>({})
  const [confirming, setConfirming] = useState(false)

  const items = useMemo(() => (scan.data ?? []).filter((c) => c.kind !== "info"), [scan.data])
  const infos = useMemo(() => (scan.data ?? []).filter((c) => c.kind === "info"), [scan.data])
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
          캐시·임시 파일처럼 지워도 다시 만들어지는 것만 골라 정리해요. 문서·사진·다운로드 같은 내 파일은 지우지 않아요.
        </p>
      </FadeUp>

      <FadeUp>
        <DriveCard used={disk.data ? disk.data.total - disk.data.free : null} disk={disk.data} plan={selectedBytes} />
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

          {scan.isError ? (
            <ErrorState
              title="정리할 수 있는 항목을 찾지 못했어요"
              error={scan.error}
              onRetry={() => scan.refetch()}
              retrying={scan.isFetching}
            />
          ) : scan.isLoading ? (
            <div className="divide-y">
              {Array.from({ length: 5 }, (_, i) => (
                <div key={i} className="flex items-center gap-4 px-5 py-4">
                  <Skeleton className="size-4 rounded" />
                  <Skeleton className="h-4 w-48" />
                  <Skeleton className="ml-auto h-4 w-16" />
                </div>
              ))}
            </div>
          ) : items.length === 0 ? (
            <div className="flex flex-col items-center gap-2 px-5 py-14 text-center">
              <div className="mb-1 flex size-12 items-center justify-center rounded-2xl bg-muted">
                <Sparkles className="size-6 text-primary" />
              </div>
              <p className="font-medium">정리할 게 없어요</p>
              <p className="text-sm text-muted-foreground">캐시와 임시 파일이 이미 깨끗해요.</p>
            </div>
          ) : (
            <ul className="divide-y">
              {items.map((c) => (
                <CategoryRow
                  key={c.id}
                  category={c}
                  checked={isSelected(c)}
                  onCheckedChange={(v) => setPicked((p) => ({ ...p, [c.id]: v }))}
                />
              ))}
            </ul>
          )}

          {items.length > 0 && (
            <div className="sticky bottom-0 flex items-center gap-3 border-t bg-popover/90 px-5 py-3 backdrop-blur-xl">
              <p className="mr-auto text-sm">
                <span className="text-muted-foreground">{selected.length}개 선택 · </span>
                <span className="font-semibold tabular-nums">{formatBytes(selectedBytes)}</span>
                <span className="text-muted-foreground"> 확보 예상</span>
              </p>
              <Button disabled={selected.length === 0} onClick={() => setConfirming(true)}>
                <Sparkles />
                정리하기
              </Button>
            </div>
          )}
        </section>
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
        open={confirming}
        items={selected}
        onClose={() => setConfirming(false)}
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

  return (
    <div className="rounded-2xl border bg-card p-5 shadow-sm">
      <div className="flex flex-wrap items-baseline gap-x-3 gap-y-1">
        <p className="font-semibold">로컬 디스크 ({disk.drive.replace(/\\$/, "")})</p>
        <p className="text-sm text-muted-foreground">
          {formatBytes(disk.total)} 중 <b className="text-foreground tabular-nums">{formatBytes(disk.free)}</b> 남음
        </p>
        {plan > 0 && (
          <p className="ml-auto text-sm text-primary tabular-nums">
            다 정리되면 약 {formatBytes(disk.free + plan)} 남음
          </p>
        )}
      </div>
      <div className="relative mt-3 h-3 overflow-hidden rounded-full bg-muted">
        <motion.div
          className="absolute inset-y-0 left-0 rounded-full bg-chart-2"
          initial={{ width: 0 }}
          animate={{ width: `${usedPct}%` }}
          transition={{ duration: 0.8, ease: EASE_OUT_QUINT }}
        />
        {/* 정리하면 비는 부분: 사용 중 막대의 끝을 밝게 표시 */}
        <motion.div
          className="absolute inset-y-0 bg-primary/70"
          style={{ left: `${usedPct - planPct}%` }}
          animate={{ width: `${planPct}%` }}
          transition={{ duration: 0.4, ease: EASE_OUT_QUINT }}
        />
      </div>
      <div className="mt-2 flex gap-4 text-xs text-muted-foreground">
        <span className="flex items-center gap-1.5">
          <span className="size-2 rounded-full bg-chart-2" />
          사용 중 {formatBytes(used)}
        </span>
        {plan > 0 && (
          <span className="flex items-center gap-1.5">
            <span className="size-2 rounded-full bg-primary" />
            정리 예정 {formatBytes(plan)}
          </span>
        )}
      </div>
    </div>
  )
}

function RiskBadge({ category: c }: { category: StorageCategory }) {
  return c.risk === "regenerable" ? (
    <span className="rounded-full bg-primary/12 px-1.5 py-px text-[10px] font-medium text-primary">
      다시 만들어져요
    </span>
  ) : (
    <span className="rounded-full bg-destructive/12 px-1.5 py-px text-[10px] font-medium text-destructive">
      되돌릴 수 없어요
    </span>
  )
}

function CategoryRow({
  category: c,
  checked,
  onCheckedChange,
}: {
  category: StorageCategory
  checked: boolean
  onCheckedChange: (v: boolean) => void
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
                  <ShieldAlert className="size-3.5 text-chart-3" />
                </IconTooltip>
              )}
            </p>
            <p className="truncate text-xs text-muted-foreground">
              {blocked
                ? `${c.running.join(", ")} 실행 중이라 지금은 정리할 수 없어요. 종료한 뒤 다시 찾아 주세요.`
                : c.description}
            </p>
          </div>
          <span className="shrink-0 text-right text-sm font-semibold tabular-nums">
            {c.bytes > 0 ? formatBytes(c.bytes) : "—"}
            {c.inaccessible > 0 && c.admin && (
              <span className="block text-[10px] font-normal text-muted-foreground">관리자만 볼 수 있어요</span>
            )}
          </span>
          <ChevronRight
            className={cn("size-4 shrink-0 text-muted-foreground transition-transform", open && "rotate-90")}
          />
        </button>
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
                <span className="font-medium">왜 지워도 되나요? </span>
                {c.why}
              </p>
              {c.files > 0 && <p className="text-muted-foreground">파일 {c.files.toLocaleString()}개</p>}
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
                {c.name} <span className="ml-1 font-semibold tabular-nums">{formatBytes(c.bytes)}</span>
              </p>
              <p className="mt-0.5 text-xs text-muted-foreground">
                {c.description}. {c.why}
              </p>
              {c.guidance && (
                <p data-selectable className="mt-1.5 text-xs">
                  {c.guidance}
                </p>
              )}
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
  items,
  onClose,
  onDone,
}: {
  open: boolean
  items: StorageCategory[]
  onClose: () => void
  onDone: () => void
}) {
  const [busy, setBusy] = useState(false)
  const [understood, setUnderstood] = useState(false)
  // 정리가 끝나면 창을 닫지 않고 항목별 결과를 보여 준다
  const [results, setResults] = useState<CleanResult[] | null>(null)
  const [adminError, setAdminError] = useState<string | null>(null)
  // 정리하는 동안의 진행: 지금 항목, 지금까지 센 것, 끝난 항목들
  const [live, setLive] = useState<LiveProgress | null>(null)
  // 되돌릴 수 없는 항목의 확인은 창을 열 때마다 새로 받는다
  useEffect(() => {
    if (open) {
      setUnderstood(false)
      setResults(null)
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
    setBusy(true)
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
      <DialogContent className="sm:max-w-md" showCloseButton={!busy}>
        {busy && live ? (
          <CleanProgressView live={live} items={items} />
        ) : results ? (
          <CleanResultView
            results={results}
            items={items}
            adminError={adminError}
            onClose={() => {
              setUnderstood(false)
              onClose()
            }}
          />
        ) : (
          <>
            <DialogTitle>{formatBytes(total)}를 정리할까요?</DialogTitle>
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
                <ShieldAlert className="size-3.5 text-chart-3" />
                관리자 권한이 필요한 항목이 있어 UAC 창이 떠요. "아니요"를 누르면 그 항목만 빼고 정리해요.
              </p>
            )}
            <div className="mt-1 flex justify-end gap-2">
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

function formatElapsed(ms: number) {
  const s = Math.floor(ms / 1000)
  return s < 60 ? `${s}초` : `${Math.floor(s / 60)}분 ${s % 60}초`
}

/** 파일 경로를 "…\폴더\파일" 정도로 줄인다 */
function shortPath(path: string) {
  const parts = path.split("\\")
  return parts.length > 3 ? `…\\${parts.slice(-2).join("\\")}` : path
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
        {formatElapsed(now - live.startedAt)} 지남 · 지금까지 파일 {doneFiles.toLocaleString()}개,{" "}
        {formatBytes(doneFreed)}
      </DialogDescription>

      <div className="h-1.5 overflow-hidden rounded-full bg-muted" aria-hidden>
        {percent == null || live.stage === "admin" || curCategory?.kind === "recycleBin" ? (
          <div className="scan-sweep h-full w-1/3 rounded-full bg-primary" />
        ) : (
          <div
            className="h-full rounded-full bg-primary transition-[width] duration-300"
            style={{ width: `${percent}%` }}
          />
        )}
      </div>

      <div className="rounded-xl border bg-muted/30 px-3.5 py-3 text-sm" role="status" aria-live="polite">
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
                {formatElapsed(quietFor)} 지남)
              </p>
            ) : (
              <>
                <p className="mt-1 text-xs text-muted-foreground tabular-nums">
                  파일 {cur.deleted.toLocaleString()}개 지움 · {formatBytes(cur.freed)}
                  {cur.failed > 0 && ` · 사용 중 ${cur.failed.toLocaleString()}개 건너뜀`}
                </p>
                <p
                  data-selectable
                  className="mt-1 truncate font-mono text-[11px] text-muted-foreground/80"
                  title={cur.file ?? ""}
                >
                  {cur.file ? shortPath(cur.file) : "파일을 찾는 중"}
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
              <Check
                className={cn(
                  "size-3.5 shrink-0",
                  r.skipped || r.needsAdmin ? "text-muted-foreground" : "text-chart-5",
                )}
              />
              <span className="min-w-0 flex-1 truncate">{nameOf(r.id)}</span>
              <span className="shrink-0 text-muted-foreground tabular-nums">
                {r.needsAdmin
                  ? "관리자 권한 필요"
                  : r.skipped
                    ? "건너뜀"
                    : `${r.deletedFiles.toLocaleString()}개 · ${formatBytes(r.freedBytes)}`}
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
  items,
  adminError,
  onClose,
}: {
  results: CleanResult[]
  items: StorageCategory[]
  adminError: string | null
  onClose: () => void
}) {
  const { freed, failed } = summarize(results)
  return (
    <>
      <DialogTitle className="flex items-center gap-2">
        {freed > 0 ? (
          <>
            <Check className="size-5 text-chart-5" />
            {formatBytes(freed)}를 확보했어요
          </>
        ) : (
          "확보한 공간이 없어요"
        )}
      </DialogTitle>
      <DialogDescription className="leading-relaxed">
        {failed > 0
          ? `지금 쓰고 있는 파일 ${failed.toLocaleString()}개는 건너뛰었어요. 해당 프로그램을 닫고 다시 정리하면 더 비울 수 있어요.`
          : "고른 항목을 정리했어요. 캐시는 프로그램이 필요할 때 다시 만들어요."}
      </DialogDescription>
      <ul className="max-h-64 divide-y overflow-y-auto rounded-xl border text-sm">
        {results.map((r) => {
          const name = items.find((c) => c.id === r.id)?.name ?? r.id
          return (
            <li key={r.id} className="px-3 py-2.5">
              <div className="flex items-center gap-2">
                <span className="min-w-0 flex-1 truncate font-medium">{name}</span>
                <span
                  className={cn("shrink-0 tabular-nums", r.freedBytes > 0 ? "text-chart-5" : "text-muted-foreground")}
                >
                  {r.freedBytes > 0 ? formatBytes(r.freedBytes) : "—"}
                </span>
              </div>
              <p className="mt-0.5 text-xs text-muted-foreground">
                {r.skipped
                  ? `건너뜀: ${r.skipped}`
                  : r.needsAdmin
                    ? "관리자 권한을 받지 못해 건너뛰었어요"
                    : [
                        `파일 ${r.deletedFiles.toLocaleString()}개 지움`,
                        r.failedFiles > 0 ? `사용 중 ${r.failedFiles.toLocaleString()}개 건너뜀` : null,
                      ]
                        .filter(Boolean)
                        .join(" · ")}
              </p>
            </li>
          )
        })}
      </ul>
      {adminError && (
        <p className="rounded-lg bg-chart-3/10 px-3 py-2 text-xs text-chart-3">
          관리자 권한 항목은 정리하지 못했어요: {adminError}
        </p>
      )}
      <div className="mt-1 flex justify-end">
        <Button onClick={onClose}>닫기</Button>
      </div>
    </>
  )
}
