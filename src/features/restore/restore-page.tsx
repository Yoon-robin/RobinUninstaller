import { useEffect, useRef, useState } from "react"
import { useQuery, useQueryClient } from "@tanstack/react-query"
import {
  ArchiveRestore,
  ChevronRight,
  FileText,
  Folder,
  FolderOpen,
  HardDrive,
  KeyRound,
  ListTree,
  Loader2,
  RotateCcw,
  ShieldAlert,
  Sparkles,
  Trash2,
  TriangleAlert,
} from "lucide-react"
import { AnimatePresence, motion } from "motion/react"
import { toast } from "sonner"

import {
  navigate,
  restoreFocus,
  countedItems,
  isRestorable,
  recoveryApi,
  settingsApi,
  type ItemStatus,
  type LeftoverKind,
  type Operation,
  type ScanProgress,
} from "@/lib/api"
import { ScanProgressView } from "@/components/scan-progress"
import { formatBytes, formatDateTime } from "@/lib/format"
import { IconTooltip } from "@/components/icon-tooltip"
import { cn } from "@/lib/utils"
import { EASE_OUT_QUINT, FadeUp, Stagger } from "@/components/motion"
import { Button } from "@/components/ui/button"
import { ErrorState, InlineError, StaleBanner } from "@/components/error-state"
import { Dialog, DialogContent, DialogDescription, DialogTitle } from "@/components/ui/dialog"
import { Skeleton } from "@/components/ui/skeleton"
import { rovingKeyDown } from "@/features/programs/segmented"
import { useBusy } from "@/lib/busy"
import { failed, notifyError } from "@/lib/notify"

const statusLabels: Record<ItemStatus, { label: string; tone: string }> = {
  // 곧 처리될 것처럼 읽히지 않게: 지난번 정리가 이 항목에서 멈췄다는 뜻이다
  pending: { label: "중간에 멈춤", tone: "text-muted-foreground" },
  removed: { label: "보관 중", tone: "text-primary" },
  needsAdmin: { label: "관리자 권한 필요", tone: "text-warning" },
  alreadyGone: { label: "이미 없음", tone: "text-muted-foreground" },
  blocked: { label: "보호됨", tone: "text-muted-foreground" },
  failed: { label: "실패", tone: "text-destructive" },
  restored: { label: "복원됨", tone: "text-success" },
  partial: { label: "일부만 정리됨", tone: "text-warning" },
  conflicted: { label: "충돌 있음 · 백업에만 남음", tone: "text-warning" },
}

const kindIcons: Record<LeftoverKind, typeof Folder> = {
  folder: Folder,
  file: FileText,
  registryKey: ListTree,
  registryValue: KeyRound,
}

const formatDate = (seconds: number) => formatDateTime(seconds * 1000)

export function RestorePage() {
  const retentionDays = useQuery({ queryKey: ["settings"], queryFn: settingsApi.get }).data?.backupRetentionDays
  const backups = useQuery({ queryKey: ["backups"], queryFn: recoveryApi.list })
  // 기록이 깨진 백업은 목록에 없지만 파일은 남아 있다. "기록 없음"으로 보이지 않게 따로 알린다
  const unreadable = useQuery({ queryKey: ["backups", "unreadable"], queryFn: recoveryApi.unreadable })
  const [deleting, setDeleting] = useState<Operation | null>(null)
  const [emptying, setEmptying] = useState(false)

  return (
    <Stagger className="relative mx-auto flex max-w-4xl min-w-0 flex-col gap-6 px-7 pt-7 pb-10">
      <FadeUp>
        <h1 className="text-2xl font-bold tracking-tight">복구 기록</h1>
        <p className="mt-1 text-sm text-muted-foreground">
          남은 흔적을 정리할 때 옮긴 파일과 레지스트리가 여기에 보관돼요. 프로그램을 다시 설치한 뒤 되돌리면 예전 설정이
          돌아와요. 제거한 프로그램 자체와 저장공간에서 지운 캐시는 되돌릴 수 없어요.
        </p>
      </FadeUp>

      {unreadable.isError && (
        <FadeUp>
          <InlineError
            className="rounded-2xl border px-4 py-3"
            message="기록을 읽지 못한 백업이 있는지 확인하지 못했어요"
            error={unreadable.error}
            onRetry={() => void unreadable.refetch()}
            retrying={unreadable.isFetching}
          />
        </FadeUp>
      )}
      {(unreadable.data?.length ?? 0) > 0 && (
        <FadeUp>
          <div className="flex flex-wrap items-center gap-3 rounded-2xl border border-chart-3/40 bg-chart-3/8 px-4 py-3 text-sm">
            <TriangleAlert className="size-4 shrink-0 text-warning" />
            <p className="min-w-0 flex-1">
              기록을 읽지 못한 백업이 {unreadable.data?.length}개 있어요. 기록 파일이 손상됐거나 권한이 없어 목록에
              보이지 않지만, 백업 파일은 지우지 않고 그대로 두었어요.
            </p>
            <Button
              variant="outline"
              size="sm"
              onClick={() => recoveryApi.openFolder().catch(failed("백업 폴더를 열지 못했어요"))}
            >
              백업 폴더 열기
            </Button>
          </div>
        </FadeUp>
      )}

      {backups.isError && backups.data && (
        <FadeUp>
          <StaleBanner
            className="rounded-2xl border"
            error={backups.error}
            onRetry={() => void backups.refetch()}
            retrying={backups.isFetching}
          />
        </FadeUp>
      )}
      {backups.isLoading ? (
        <FadeUp>
          <Skeleton className="h-28 w-full rounded-2xl" />
        </FadeUp>
      ) : backups.isError && !backups.data ? (
        <FadeUp>
          <ErrorState
            className="rounded-2xl border border-destructive/40"
            title="복구 기록을 읽지 못했어요"
            hint="백업은 지워지지 않았어요. 백업 폴더(%LOCALAPPDATA%\RobinX\backups)에 그대로 있으니 다시 시도해 보세요."
            error={backups.error}
            onRetry={() => backups.refetch()}
            retrying={backups.isFetching}
          />
        </FadeUp>
      ) : (backups.data ?? []).length === 0 ? (
        <FadeUp>
          <div className="flex flex-col items-center gap-3 rounded-2xl border border-dashed px-6 py-14 text-center">
            <div className="flex size-12 items-center justify-center rounded-2xl bg-muted">
              <ArchiveRestore className="size-6 text-muted-foreground" />
            </div>
            <p className="font-medium">아직 정리한 기록이 없어요</p>
            <p className="text-sm text-muted-foreground">잔여물을 정리하면 되돌릴 수 있도록 여기에 기록돼요.</p>
            <Button variant="outline" size="sm" className="mt-1" onClick={() => navigate("leftovers")}>
              <Sparkles />
              잔여물 정리로 가기
            </Button>
          </div>
        </FadeUp>
      ) : (
        <>
          <FadeUp className="flex flex-col gap-3">
            <HeldSpace ops={backups.data ?? []} />
            <div>
              <Button variant="outline" size="sm" onClick={() => setEmptying(true)}>
                <Trash2 />
                오래된 백업 비우기
              </Button>
            </div>
          </FadeUp>
          {(backups.data ?? []).map((op) => (
            <FadeUp key={op.id}>
              <OperationCard op={op} retentionDays={retentionDays} onDelete={() => setDeleting(op)} />
            </FadeUp>
          ))}
        </>
      )}

      <DeleteBackupDialog op={deleting} onClose={() => setDeleting(null)} />
      <EmptyOldBackupsDialog open={emptying} onClose={() => setEmptying(false)} />
    </Stagger>
  )
}

/** 보관 중인 백업이 차지하는 공간. 정리했다고 공간이 바로 늘지는 않는다는 것을 알려 준다. */
function HeldSpace({ ops }: { ops: Operation[] }) {
  const settings = useQuery({ queryKey: ["settings"], queryFn: settingsApi.get })
  const days = settings.data?.backupRetentionDays
  const held = ops
    .flatMap((op) => op.items)
    .filter((i) => isRestorable(i.status))
    .reduce((s, i) => s + (i.sizeBytes ?? 0), 0)
  if (held === 0) return null
  return (
    <div className="flex items-start gap-3 rounded-2xl border bg-muted/30 px-4 py-3.5 text-sm">
      <HardDrive className="mt-0.5 size-4 shrink-0 text-muted-foreground" />
      <p className="text-muted-foreground">
        백업으로 보관 중인 파일이 <span className="font-semibold text-foreground">{formatBytes(held)}</span>예요. 이
        공간은 백업을 지워야 돌아와요.{" "}
        {!settings.data
          ? ""
          : days == null
            ? "지금은 직접 지울 때까지 보관하도록 설정돼 있어요."
            : `보관 기간 ${days}일이 지난 백업은 자동으로 지워져요.`}
      </p>
    </div>
  )
}

/**
 * 고른 기간보다 오래된 백업을 한 번에 지운다. 기간은 기본 선택 없이 직접 고르고,
 * 지울 개수와 크기를 본 뒤 한 번 더 눌러야 지워진다. 되돌릴 수 없다.
 */
function EmptyOldBackupsDialog({ open, onClose }: { open: boolean; onClose: () => void }) {
  const queryClient = useQueryClient()
  const settings = useQuery({ queryKey: ["settings"], queryFn: settingsApi.get, enabled: open })
  const [days, setDays] = useState<number | null>(null)
  const [busy, setBusy] = useState(false)
  useBusy(busy)
  // 지우는 동안: 몇 번째 백업인지, 지운 크기
  const [progress, setProgress] = useState<ScanProgress | null>(null)
  const preview = useQuery({
    queryKey: ["old-backups", days],
    queryFn: () => settingsApi.oldBackups(days!),
    enabled: open && days != null,
  })

  useEffect(() => {
    if (open) setDays(null)
  }, [open])

  const run = async () => {
    if (days == null) return
    setBusy(true)
    setProgress(null)
    try {
      const r = await settingsApi.deleteOldBackups(days, setProgress)
      if (r.failed > 0) {
        toast.warning(`${r.deleted}개를 지웠어요. ${r.failed}개는 지우지 못했어요`, {
          description: "관리자 권한으로 정리한 백업일 수 있어요. 복구 기록에서 하나씩 지워 주세요.",
        })
      } else {
        toast.success(`오래된 백업 ${r.deleted}개를 지웠어요`)
      }
      onClose()
    } catch (e) {
      notifyError("오래된 백업을 지우지 못했어요", e, { persist: true })
    } finally {
      setBusy(false)
      for (const key of ["backups", "backup-usage", "old-backups"]) queryClient.invalidateQueries({ queryKey: [key] })
    }
  }

  const count = preview.data?.count ?? 0
  return (
    <Dialog open={open} onOpenChange={(v) => !v && !busy && onClose()}>
      <DialogContent className="sm:max-w-md" showCloseButton={!busy}>
        <DialogTitle>오래된 백업 비우기</DialogTitle>
        <DialogDescription className="leading-relaxed">
          고른 기간보다 오래된 정리 기록과 백업을 지워 공간을 돌려받아요. 지운 백업으로는 다시 되돌릴 수 없어요.
          복원하다 충돌이 난 백업은 지우지 않아요.
        </DialogDescription>
        <div
          role="radiogroup"
          aria-label="기간"
          className="flex flex-wrap gap-2"
          onKeyDown={(e) => rovingKeyDown(e, settings.data?.retentionChoices ?? [], days ?? -1, () => busy, setDays)}
        >
          {(settings.data?.retentionChoices ?? []).map((d, i) => (
            <Button
              key={d}
              role="radio"
              aria-checked={days === d}
              // 고른 것(없으면 첫 칸)만 Tab으로 들어오고, 그 안에서는 방향키로 옮긴다
              tabIndex={days === d || (days == null && i === 0) ? 0 : -1}
              variant={days === d ? "default" : "outline"}
              size="sm"
              onClick={() => setDays(d)}
              disabled={busy}
            >
              {d}일 지난 것
            </Button>
          ))}
        </div>
        {settings.isError && (
          <InlineError
            message="설정을 읽지 못해 기간을 보여 줄 수 없어요"
            onRetry={() => void settings.refetch()}
            retrying={settings.isFetching}
          />
        )}
        <p className="min-h-5 text-sm text-muted-foreground" aria-live="polite">
          {days == null
            ? "기간을 먼저 골라 주세요."
            : preview.isError
              ? `지울 백업을 세지 못했어요 (${String(preview.error)})`
              : preview.isLoading
                ? "찾는 중이에요"
                : count === 0
                  ? `${days}일 지난 백업이 없어요.`
                  : `${count}개를 지워요 (${formatBytes(preview.data?.bytes ?? 0)}).`}
        </p>
        {busy && <ScanProgressView work label="오래된 백업을 지우는 중이에요" progress={progress} />}
        <div className="mt-2 flex justify-end gap-2">
          <Button variant="outline" onClick={onClose} disabled={busy}>
            취소
          </Button>
          <Button
            variant="destructive"
            onClick={run}
            disabled={busy || days == null || count === 0 || !preview.isSuccess}
          >
            {busy ? <Loader2 className="animate-spin" /> : <Trash2 />}
            영구 삭제
          </Button>
        </div>
      </DialogContent>
    </Dialog>
  )
}

/** 백업이 언제 저절로 지워지는지. 보관 기간이 지난 백업은 앱을 켤 때 지운다 (충돌이 남은 백업은 지우지 않는다). */
function expiryText(op: Operation, retentionDays: number | null | undefined): { text: string; soon: boolean } | null {
  if (retentionDays == null) return null // "계속 보관"이거나 설정을 아직 못 읽음
  if (op.items.some((i) => i.status === "conflicted")) return { text: "충돌이 있어 저절로 지우지 않아요", soon: false }
  const left = Math.ceil((op.createdAt + retentionDays * 86_400 - Date.now() / 1000) / 86_400)
  if (left <= 0) return { text: "다음에 앱을 켤 때 지워져요", soon: true }
  return { text: `${left}일 뒤 지워져요`, soon: left <= 3 }
}

function OperationCard({
  op,
  retentionDays,
  onDelete,
}: {
  op: Operation
  retentionDays: number | null | undefined
  onDelete: () => void
}) {
  const queryClient = useQueryClient()
  // 정리를 마치고 "복구 기록 보기"로 왔으면 그 기록을 펼치고 보이는 곳으로 옮긴다
  const [focused] = useState(() => restoreFocus.id === op.id)
  const [open, setOpen] = useState(focused)
  const cardRef = useRef<HTMLElement>(null)
  useEffect(() => {
    if (!focused) return
    restoreFocus.id = null
    // 페이지가 들어오는 움직임이 끝난 뒤에 옮긴다
    const t = setTimeout(() => cardRef.current?.scrollIntoView({ block: "start", behavior: "smooth" }), 300)
    return () => clearTimeout(t)
  }, [focused])
  const [busy, setBusy] = useState<string | null>(null)
  useBusy(busy != null)
  // 되돌리는 동안: 지금 항목, 복사한 파일 수 (관리자 권한 작업자는 따로 도는 프로세스라 진행을 알 수 없다)
  const [progress, setProgress] = useState<ScanProgress | null>(null)

  const kept = countedItems(op).filter((i) => isRestorable(i.status)).length
  const expiry = kept > 0 ? expiryText(op, retentionDays) : null
  const restored = op.items.filter((i) => i.status === "restored").length
  const conflicted = op.items.filter((i) => i.status === "conflicted").length
  const keptBytes = op.items.filter((i) => isRestorable(i.status)).reduce((s, i) => s + (i.sizeBytes ?? 0), 0)
  // 관리자 권한이 없어 되돌리지 못한 항목만 관리자 권한으로 다시 시도한다
  const adminRestoreIds = op.items
    .filter((i) => isRestorable(i.status) && i.restoreFailed && i.needsAdmin)
    .map((i) => i.id)

  /** `ids`: 이번에 되돌리려는 항목. 결과는 이 항목들만 센다 (예전에 실패한 항목이 섞이지 않게). */
  const run = async (key: string, ids: string[], action: () => Promise<Operation>) => {
    setBusy(key)
    setProgress(null)
    try {
      const updated = await action()
      const mine = updated.items.filter((i) => ids.includes(i.id))
      const notRestored = mine.filter((i) => isRestorable(i.status) && i.restoreFailed).length
      // 같은 이름이 이미 있어 덮어쓰지 않은 항목은 백업에만 남는다
      const clashed = mine.filter((i) => i.status === "conflicted").length
      if (notRestored > 0)
        toast.error(`${notRestored}개는 되돌리지 못했어요`, {
          description: "항목을 펼쳐 이유를 확인하세요",
          duration: 8000,
        })
      else if (clashed > 0)
        toast.warning(`${clashed}개는 같은 이름이 이미 있어 백업에 남겨 뒀어요`, {
          description: "항목을 펼쳐 백업 폴더에서 직접 꺼낼 수 있어요",
        })
      else toast.success("되돌렸어요")
    } catch (e) {
      notifyError("되돌리지 못했어요", e, { persist: true })
    } finally {
      setBusy(null)
      // 복원하면 프로그램·시작 항목이 다시 생길 수 있다
      for (const key of [
        "backups",
        "backup-usage",
        "old-backups",
        "installed-apps",
        "recently-removed",
        "startup-items",
      ]) {
        queryClient.invalidateQueries({ queryKey: [key] })
      }
    }
  }

  return (
    <section ref={cardRef} className="scroll-mt-4 overflow-hidden rounded-2xl border bg-card shadow-sm">
      <div className="flex flex-wrap items-center gap-3 px-5 py-4 transition-colors has-[>button[aria-expanded]:hover]:bg-accent/30">
        <button
          type="button"
          onClick={() => setOpen((v) => !v)}
          className="flex min-w-0 flex-1 cursor-pointer items-center gap-3 text-left"
          aria-expanded={open}
        >
          <ChevronRight
            className={cn(
              "size-4 shrink-0 text-muted-foreground transition-transform duration-300 ease-(--ease-out-quint)",
              open && "rotate-90",
            )}
          />
          <div className="min-w-0">
            {/* 좁은 창에서 버튼에 밀려 이름이 잘리지 않게 줄을 바꾼다 */}
            <p className="font-semibold break-words">{op.appName}</p>
            <p className="text-xs text-muted-foreground">
              {formatDate(op.createdAt)} · 보관 중 {kept}개{keptBytes > 0 && ` (${formatBytes(keptBytes)})`}
              {restored > 0 && ` · 복원 ${restored}개`}
              {conflicted > 0 && <span className="text-warning">{` · 충돌 ${conflicted}개`}</span>}
              {expiry && <span className={cn(expiry.soon && "text-warning")}>{` · ${expiry.text}`}</span>}
            </p>
          </div>
        </button>
        {adminRestoreIds.length > 0 && (
          <Button
            size="sm"
            variant="outline"
            disabled={busy != null}
            onClick={() => run("admin", adminRestoreIds, () => recoveryApi.restoreElevated(op.id, adminRestoreIds))}
          >
            {busy === "admin" ? <Loader2 className="animate-spin" /> : <ShieldAlert />}
            관리자 권한으로 복원
          </Button>
        )}
        <Button
          size="sm"
          variant="outline"
          disabled={kept === 0 || busy != null}
          onClick={() =>
            run(
              "all",
              op.items.filter((i) => isRestorable(i.status)).map((i) => i.id),
              () => recoveryApi.restore(op.id, undefined, setProgress),
            )
          }
        >
          {busy === "all" ? <Loader2 className="animate-spin" /> : <RotateCcw />}
          정리 되돌리기
        </Button>
        <IconTooltip label="백업 삭제">
          <Button
            size="icon"
            variant="ghost"
            className="size-8 text-muted-foreground hover:text-destructive"
            aria-label="백업 삭제"
            disabled={busy != null}
            onClick={onDelete}
          >
            <Trash2 />
          </Button>
        </IconTooltip>
      </div>
      {busy != null && busy !== "admin" && (
        <ScanProgressView work className="border-t px-5 py-4" label="되돌리는 중이에요" progress={progress} />
      )}

      <AnimatePresence initial={false}>
        {open && (
          <motion.div
            initial={{ height: 0, opacity: 0 }}
            animate={{ height: "auto", opacity: 1 }}
            exit={{ height: 0, opacity: 0 }}
            transition={{ duration: 0.3, ease: EASE_OUT_QUINT }}
            className="overflow-hidden"
          >
            <ul className="divide-y border-t">
              {op.items.map((item) => {
                const Icon = kindIcons[item.kind]
                const s = statusLabels[item.status]
                return (
                  <li key={item.id} className="flex items-start gap-3 px-5 py-3 text-xs">
                    <Icon className="mt-0.5 size-4 shrink-0 text-muted-foreground" />
                    <div className="min-w-0 flex-1">
                      <p data-selectable className="font-mono break-all">
                        {item.location}
                        {item.valueName && ` › ${item.valueName}`}
                      </p>
                      {item.error && (
                        <p className={cn("mt-1", item.status === "conflicted" ? "text-warning" : "text-destructive")}>
                          {item.error}
                        </p>
                      )}
                      {item.status === "conflicted" && (
                        <Button
                          size="sm"
                          variant="outline"
                          className="mt-2 h-7"
                          onClick={() =>
                            recoveryApi.openOperationFolder(op.id).catch(failed("이 기록의 백업 폴더를 열지 못했어요"))
                          }
                        >
                          <FolderOpen />
                          백업 폴더 열기
                        </Button>
                      )}
                    </div>
                    <span className={cn("w-28 shrink-0 text-right font-medium", s.tone)}>{s.label}</span>
                    <span className="flex w-24 shrink-0 justify-end">
                      {isRestorable(item.status) && (
                        <Button
                          size="sm"
                          variant="ghost"
                          className="h-7 shrink-0"
                          disabled={busy != null}
                          onClick={() =>
                            run(item.id, [item.id], () => recoveryApi.restore(op.id, [item.id], setProgress))
                          }
                        >
                          {busy === item.id ? <Loader2 className="animate-spin" /> : <RotateCcw />}
                          되돌리기
                        </Button>
                      )}
                    </span>
                  </li>
                )
              })}
            </ul>
          </motion.div>
        )}
      </AnimatePresence>
    </section>
  )
}

function DeleteBackupDialog({ op, onClose }: { op: Operation | null; onClose: () => void }) {
  const queryClient = useQueryClient()
  const [busy, setBusy] = useState(false)
  useBusy(busy)
  const [progress, setProgress] = useState<ScanProgress | null>(null)
  // 닫히는 애니메이션 동안 op가 null이 돼도 문구가 깨지지 않게 마지막 값을 보여 준다
  const [shown, setShown] = useState(op)
  useEffect(() => {
    if (op) setShown(op)
  }, [op])
  const kept = shown ? countedItems(shown).filter((i) => isRestorable(i.status)).length : 0
  const conflicted = shown ? shown.items.filter((i) => i.status === "conflicted").length : 0
  // 관리자 권한으로 정리한 백업은 일반 권한으로 못 지울 수 있다. 바로 UAC를 띄우지 않고 먼저 알린다.
  const [needsAdmin, setNeedsAdmin] = useState(false)
  useEffect(() => {
    if (op) setNeedsAdmin(false)
  }, [op])

  const confirm = async () => {
    if (!op) return
    setBusy(true)
    setProgress(null)
    try {
      await recoveryApi.deleteBackup(op.id, setProgress, needsAdmin)
      toast.success("백업을 삭제했어요")
      onClose()
    } catch (e) {
      if (String(e) === "needsAdmin") setNeedsAdmin(true)
      else notifyError(`${op.appName} 백업을 지우지 못했어요`, e, { persist: true })
    } finally {
      setBusy(false)
      for (const key of ["backups", "backup-usage", "old-backups"]) queryClient.invalidateQueries({ queryKey: [key] })
    }
  }

  return (
    <Dialog open={op != null} onOpenChange={(open) => !open && !busy && onClose()}>
      {/* 진행 표시가 나타나도 창이 위로 튀지 않게 위쪽에 고정한다 */}
      <DialogContent className="sm:max-w-md" showCloseButton={!busy}>
        <DialogTitle>백업을 완전히 삭제할까요?</DialogTitle>
        <DialogDescription className="leading-relaxed">
          {kept > 0
            ? `${shown?.appName}의 정리 기록과 보관 중인 항목 ${kept}개가 영구히 지워져요.`
            : `${shown?.appName}의 정리 기록을 지워요. 모두 되돌려서 보관 중인 항목은 없어요.`}{" "}
          <b className="text-foreground">이후에는 되돌릴 수 없어요.</b>
          {conflicted > 0 && (
            <span className="mt-2 block rounded-lg bg-chart-3/10 px-3 py-2 text-warning">
              복원할 때 충돌이 난 항목 {conflicted}개는 일부 파일이 이 백업에만 남아 있어요. 삭제하면 그 파일도
              사라져요. 항목을 펼쳐 경로를 먼저 확인하세요.
            </span>
          )}
        </DialogDescription>
        {needsAdmin && (
          <p className="flex gap-2 rounded-lg border bg-muted/30 px-3 py-2.5 text-sm" role="status">
            <ShieldAlert className="mt-0.5 size-4 shrink-0 text-warning" />
            <span>
              일부 파일은 관리자 권한으로 정리한 것이라 관리자 권한이 있어야 지울 수 있어요. 누르면 관리자 권한
              요청(UAC)이 떠요.
            </span>
          </p>
        )}
        {busy && <ScanProgressView work label="백업을 지우는 중이에요" progress={progress} />}
        <div className="mt-2 flex justify-end gap-2">
          <Button variant="outline" onClick={onClose} disabled={busy}>
            취소
          </Button>
          <Button variant="destructive" onClick={confirm} disabled={busy}>
            {busy ? <Loader2 className="animate-spin" /> : needsAdmin ? <ShieldAlert /> : <Trash2 />}
            {needsAdmin ? "관리자 권한으로 지우기" : "영구 삭제"}
          </Button>
        </div>
      </DialogContent>
    </Dialog>
  )
}
