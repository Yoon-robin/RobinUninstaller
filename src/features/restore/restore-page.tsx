import { useEffect, useState } from "react"
import { useQuery, useQueryClient } from "@tanstack/react-query"
import {
  ArchiveRestore,
  ChevronRight,
  HardDrive,
  FileText,
  Folder,
  FolderOpen,
  KeyRound,
  ListTree,
  Loader2,
  RotateCcw,
  ShieldAlert,
  Trash2,
  TriangleAlert,
} from "lucide-react"
import { AnimatePresence, motion } from "motion/react"
import { toast } from "sonner"

import {
  isRestorable,
  recoveryApi,
  settingsApi,
  type ItemStatus,
  type LeftoverKind,
  type Operation,
  type ScanProgress,
} from "@/lib/api"
import { ScanProgressView } from "@/components/scan-progress"
import { formatBytes } from "@/lib/format"
import { cn } from "@/lib/utils"
import { EASE_OUT_QUINT, FadeUp, Stagger } from "@/components/motion"
import { Button } from "@/components/ui/button"
import { ErrorState } from "@/components/error-state"
import { Dialog, DialogContent, DialogDescription, DialogTitle } from "@/components/ui/dialog"
import { Skeleton } from "@/components/ui/skeleton"
import { useBusy } from "@/lib/busy"

const statusLabels: Record<ItemStatus, { label: string; tone: string }> = {
  pending: { label: "대기", tone: "text-muted-foreground" },
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

function formatDate(seconds: number) {
  return new Date(seconds * 1000).toLocaleString("ko-KR", {
    year: "numeric",
    month: "long",
    day: "numeric",
    hour: "2-digit",
    minute: "2-digit",
  })
}

export function RestorePage() {
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
          남은 흔적을 정리할 때 옮긴 파일과 레지스트리가 여기에 보관돼요. 원래 자리에 같은 이름이 새로 생겼다면 덮어쓰지
          않고 알려 드려요. 제거한 프로그램 자체나 저장공간에서 지운 캐시는 여기서 되돌릴 수 없어요.
        </p>
      </FadeUp>

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
              onClick={() => recoveryApi.openFolder().catch((e) => toast.error(String(e)))}
            >
              백업 폴더 열기
            </Button>
          </div>
        </FadeUp>
      )}

      {backups.isLoading ? (
        <FadeUp>
          <Skeleton className="h-28 w-full rounded-2xl" />
        </FadeUp>
      ) : backups.isError ? (
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
              <OperationCard op={op} onDelete={() => setDeleting(op)} />
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
        {days == null
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
      toast.error("백업을 지우지 못했어요", { description: String(e) })
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
        <div role="radiogroup" aria-label="기간" className="flex flex-wrap gap-2">
          {(settings.data?.retentionChoices ?? []).map((d) => (
            <Button
              key={d}
              role="radio"
              aria-checked={days === d}
              variant={days === d ? "default" : "outline"}
              size="sm"
              onClick={() => setDays(d)}
              disabled={busy}
            >
              {d}일 지난 것
            </Button>
          ))}
        </div>
        <p className="min-h-5 text-sm text-muted-foreground" aria-live="polite">
          {days == null
            ? "기간을 먼저 골라 주세요."
            : preview.isLoading
              ? "찾는 중이에요"
              : count === 0
                ? `${days}일 지난 백업이 없어요.`
                : `${count}개, ${formatBytes(preview.data?.bytes ?? 0)}를 지워요.`}
        </p>
        {busy && <ScanProgressView label="오래된 백업을 지우는 중이에요" progress={progress} />}
        <div className="flex justify-end gap-2">
          <Button variant="outline" onClick={onClose} disabled={busy}>
            취소
          </Button>
          <Button variant="destructive" onClick={run} disabled={busy || days == null || count === 0}>
            {busy ? <Loader2 className="animate-spin" /> : <Trash2 />}
            영구 삭제
          </Button>
        </div>
      </DialogContent>
    </Dialog>
  )
}

function OperationCard({ op, onDelete }: { op: Operation; onDelete: () => void }) {
  const queryClient = useQueryClient()
  const [open, setOpen] = useState(false)
  const [busy, setBusy] = useState<string | null>(null)
  useBusy(busy != null)
  // 되돌리는 동안: 지금 항목, 복사한 파일 수 (관리자 권한 작업자는 따로 도는 프로세스라 진행을 알 수 없다)
  const [progress, setProgress] = useState<ScanProgress | null>(null)

  const kept = op.items.filter((i) => isRestorable(i.status)).length
  const restored = op.items.filter((i) => i.status === "restored").length
  const conflicted = op.items.filter((i) => i.status === "conflicted").length
  const keptBytes = op.items.filter((i) => isRestorable(i.status)).reduce((s, i) => s + (i.sizeBytes ?? 0), 0)
  // 관리자 권한이 없어 되돌리지 못한 항목만 관리자 권한으로 다시 시도한다
  const adminRestoreIds = op.items
    .filter((i) => isRestorable(i.status) && i.error?.startsWith("복원 실패") && i.error.includes("관리자"))
    .map((i) => i.id)

  /** `ids`: 이번에 되돌리려는 항목. 결과는 이 항목들만 센다 (예전에 실패한 항목이 섞이지 않게). */
  const run = async (key: string, ids: string[], action: () => Promise<Operation>) => {
    setBusy(key)
    setProgress(null)
    try {
      const updated = await action()
      const mine = updated.items.filter((i) => ids.includes(i.id))
      const failed = mine.filter((i) => isRestorable(i.status) && i.error?.startsWith("복원 실패")).length
      // 같은 이름이 이미 있어 덮어쓰지 않은 항목은 백업에만 남는다
      const clashed = mine.filter((i) => i.status === "conflicted").length
      if (failed > 0) toast.error(`${failed}개는 되돌리지 못했어요. 항목을 펼쳐 이유를 확인하세요`)
      else if (clashed > 0)
        toast.warning(`${clashed}개는 같은 이름이 이미 있어 백업에 남겨 뒀어요`, {
          description: "항목을 펼쳐 백업 폴더에서 직접 꺼낼 수 있어요",
        })
      else toast.success("되돌렸어요")
    } catch (e) {
      toast.error(String(e))
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
    <section className="overflow-hidden rounded-2xl border bg-card shadow-sm">
      <div className="flex flex-wrap items-center gap-3 px-5 py-4">
        <button
          type="button"
          onClick={() => setOpen((v) => !v)}
          className="flex min-w-0 flex-1 items-center gap-3 text-left"
          aria-expanded={open}
        >
          <ChevronRight
            className={cn(
              "size-4 shrink-0 text-muted-foreground transition-transform duration-200",
              open && "rotate-90",
            )}
          />
          <div className="min-w-0">
            <p className="truncate font-semibold">{op.appName}</p>
            <p className="text-xs text-muted-foreground">
              {formatDate(op.createdAt)} · 보관 중 {kept}개{keptBytes > 0 && ` (${formatBytes(keptBytes)})`}
              {restored > 0 && ` · 복원 ${restored}개`}
              {conflicted > 0 && <span className="text-warning">{` · 충돌 ${conflicted}개`}</span>}
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
      </div>
      {busy != null && busy !== "admin" && (
        <ScanProgressView className="border-t px-5 py-4" label="되돌리는 중이에요" progress={progress} />
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
                          onClick={() => recoveryApi.openOperationFolder(op.id).catch((e) => toast.error(String(e)))}
                        >
                          <FolderOpen />
                          백업 폴더 열기
                        </Button>
                      )}
                    </div>
                    <span className={cn("shrink-0 font-medium", s.tone)}>{s.label}</span>
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
  const kept = shown ? shown.items.filter((i) => isRestorable(i.status)).length : 0
  const conflicted = shown ? shown.items.filter((i) => i.status === "conflicted").length : 0

  const confirm = async () => {
    if (!op) return
    setBusy(true)
    setProgress(null)
    try {
      await recoveryApi.deleteBackup(op.id, setProgress)
      toast.success("백업을 삭제했어요")
      onClose()
    } catch (e) {
      toast.error(String(e))
    } finally {
      setBusy(false)
      for (const key of ["backups", "backup-usage", "old-backups"]) queryClient.invalidateQueries({ queryKey: [key] })
    }
  }

  return (
    <Dialog open={op != null} onOpenChange={(open) => !open && !busy && onClose()}>
      <DialogContent className="sm:max-w-md">
        <DialogTitle>백업을 완전히 삭제할까요?</DialogTitle>
        <DialogDescription className="leading-relaxed">
          {shown?.appName}의 정리 기록과 보관 중인 항목 {kept}개가 영구히 지워져요.{" "}
          <b className="text-foreground">이후에는 되돌릴 수 없어요.</b>
          {conflicted > 0 && (
            <span className="mt-2 block rounded-lg bg-chart-3/10 px-3 py-2 text-warning">
              복원할 때 충돌이 난 항목 {conflicted}개는 일부 파일이 이 백업에만 남아 있어요. 삭제하면 그 파일도
              사라져요. 항목을 펼쳐 경로를 먼저 확인하세요.
            </span>
          )}
        </DialogDescription>
        {busy && <ScanProgressView label="백업을 지우는 중이에요" progress={progress} />}
        <div className="mt-2 flex justify-end gap-2">
          <Button variant="outline" onClick={onClose} disabled={busy}>
            취소
          </Button>
          <Button variant="destructive" onClick={confirm} disabled={busy}>
            {busy ? <Loader2 className="animate-spin" /> : <Trash2 />}
            영구 삭제
          </Button>
        </div>
      </DialogContent>
    </Dialog>
  )
}
