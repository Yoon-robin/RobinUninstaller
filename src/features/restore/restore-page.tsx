import { useEffect, useState } from "react"
import { useQuery, useQueryClient } from "@tanstack/react-query"
import {
  ArchiveRestore,
  ChevronRight,
  FileText,
  Folder,
  KeyRound,
  ListTree,
  Loader2,
  RotateCcw,
  ShieldAlert,
  Trash2,
} from "lucide-react"
import { AnimatePresence, motion } from "motion/react"
import { toast } from "sonner"

import { isRestorable, recoveryApi, type ItemStatus, type LeftoverKind, type Operation } from "@/lib/api"
import { formatBytes } from "@/lib/format"
import { cn } from "@/lib/utils"
import { EASE_OUT_QUINT, FadeUp, Stagger } from "@/components/motion"
import { Button } from "@/components/ui/button"
import { Dialog, DialogContent, DialogDescription, DialogTitle } from "@/components/ui/dialog"
import { Skeleton } from "@/components/ui/skeleton"

const statusLabels: Record<ItemStatus, { label: string; tone: string }> = {
  pending: { label: "대기", tone: "text-muted-foreground" },
  removed: { label: "보관 중", tone: "text-primary" },
  needsAdmin: { label: "관리자 권한 필요", tone: "text-chart-3" },
  alreadyGone: { label: "이미 없음", tone: "text-muted-foreground" },
  blocked: { label: "보호됨", tone: "text-muted-foreground" },
  failed: { label: "실패", tone: "text-destructive" },
  restored: { label: "복원됨", tone: "text-chart-5" },
  partial: { label: "일부만 정리됨", tone: "text-chart-3" },
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
  const [deleting, setDeleting] = useState<Operation | null>(null)

  return (
    <Stagger className="relative mx-auto flex max-w-4xl min-w-0 flex-col gap-6 px-7 pt-7 pb-10">
      <FadeUp>
        <h1 className="text-2xl font-bold tracking-tight">복구 기록</h1>
        <p className="mt-1 text-sm text-muted-foreground">
          정리할 때 지운 파일과 레지스트리는 모두 여기에 보관돼요. 원래 자리에 같은 이름이 새로 생겼다면 덮어쓰지 않고
          알려 드려요.
        </p>
      </FadeUp>

      {backups.isLoading ? (
        <FadeUp>
          <Skeleton className="h-28 w-full rounded-2xl" />
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
        (backups.data ?? []).map((op) => (
          <FadeUp key={op.id}>
            <OperationCard op={op} onDelete={() => setDeleting(op)} />
          </FadeUp>
        ))
      )}

      <DeleteBackupDialog op={deleting} onClose={() => setDeleting(null)} />
    </Stagger>
  )
}

function OperationCard({ op, onDelete }: { op: Operation; onDelete: () => void }) {
  const queryClient = useQueryClient()
  const [open, setOpen] = useState(false)
  const [busy, setBusy] = useState<string | null>(null)

  const kept = op.items.filter((i) => isRestorable(i.status)).length
  const restored = op.items.filter((i) => i.status === "restored").length
  const keptBytes = op.items.filter((i) => isRestorable(i.status)).reduce((s, i) => s + (i.sizeBytes ?? 0), 0)
  // 관리자 권한이 없어 되돌리지 못한 항목만 관리자 권한으로 다시 시도한다
  const adminRestoreIds = op.items
    .filter((i) => isRestorable(i.status) && i.error?.startsWith("복원 실패") && i.error.includes("관리자"))
    .map((i) => i.id)

  const run = async (key: string, action: () => Promise<Operation>) => {
    setBusy(key)
    try {
      const updated = await action()
      const failed = updated.items.filter((i) => isRestorable(i.status) && i.error?.startsWith("복원 실패")).length
      if (failed > 0) toast.error(`${failed}개는 되돌리지 못했어요. 항목을 펼쳐 이유를 확인하세요`)
      else toast.success("되돌렸어요")
    } catch (e) {
      toast.error(String(e))
    } finally {
      setBusy(null)
      // 복원하면 프로그램·시작 항목이 다시 생길 수 있다
      for (const key of ["backups", "installed-apps", "recently-removed", "startup-items"]) {
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
            </p>
          </div>
        </button>
        {adminRestoreIds.length > 0 && (
          <Button
            size="sm"
            variant="outline"
            disabled={busy != null}
            onClick={() => run("admin", () => recoveryApi.restoreElevated(op.id, adminRestoreIds))}
          >
            {busy === "admin" ? <Loader2 className="animate-spin" /> : <ShieldAlert />}
            관리자 권한으로 복원
          </Button>
        )}
        <Button
          size="sm"
          variant="outline"
          disabled={kept === 0 || busy != null}
          onClick={() => run("all", () => recoveryApi.restore(op.id))}
        >
          {busy === "all" ? <Loader2 className="animate-spin" /> : <RotateCcw />}
          모두 되돌리기
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
                      {item.error && <p className="mt-1 text-destructive">{item.error}</p>}
                    </div>
                    <span className={cn("shrink-0 font-medium", s.tone)}>{s.label}</span>
                    {isRestorable(item.status) && (
                      <Button
                        size="sm"
                        variant="ghost"
                        className="h-7 shrink-0"
                        disabled={busy != null}
                        onClick={() => run(item.id, () => recoveryApi.restore(op.id, [item.id]))}
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
  // 닫히는 애니메이션 동안 op가 null이 돼도 문구가 깨지지 않게 마지막 값을 보여 준다
  const [shown, setShown] = useState(op)
  useEffect(() => {
    if (op) setShown(op)
  }, [op])
  const kept = shown ? shown.items.filter((i) => isRestorable(i.status)).length : 0

  const confirm = async () => {
    if (!op) return
    setBusy(true)
    try {
      await recoveryApi.deleteBackup(op.id)
      toast.success("백업을 삭제했어요")
      onClose()
    } catch (e) {
      toast.error(String(e))
    } finally {
      setBusy(false)
      queryClient.invalidateQueries({ queryKey: ["backups"] })
    }
  }

  return (
    <Dialog open={op != null} onOpenChange={(open) => !open && !busy && onClose()}>
      <DialogContent className="sm:max-w-md">
        <DialogTitle>백업을 완전히 삭제할까요?</DialogTitle>
        <DialogDescription className="leading-relaxed">
          {shown?.appName}의 정리 기록과 보관 중인 항목 {kept}개가 영구히 지워져요.{" "}
          <b className="text-foreground">이후에는 되돌릴 수 없어요.</b>
        </DialogDescription>
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
