import { useState } from "react"
import { useQueryClient } from "@tanstack/react-query"
import { ArchiveRestore, Check, Loader2, ShieldAlert, TriangleAlert } from "lucide-react"
import { motion } from "motion/react"

import { countStatus, navigate, recoveryApi, type Leftover, type Operation } from "@/lib/api"
import { formatBytes } from "@/lib/format"
import { cn } from "@/lib/utils"
import { EASE_OUT_QUINT } from "@/components/motion"
import { Button } from "@/components/ui/button"

const panel = {
  initial: { opacity: 0, y: 8 },
  animate: { opacity: 1, y: 0 },
  exit: { opacity: 0, transition: { duration: 0.12 } },
  transition: { duration: 0.3, ease: EASE_OUT_QUINT },
}

/** 지우기 전 확인. 검토 항목을 골랐으면 한 번 더 짚어 준다. */
export function ConfirmClean({
  items,
  onCancel,
  onConfirm,
}: {
  items: Leftover[]
  onCancel: () => void
  onConfirm: () => void
}) {
  const reviewCount = items.filter((i) => i.confidence === "review").length
  const bytes = items.reduce((sum, i) => sum + (i.sizeBytes ?? 0), 0)
  const needsAdmin = items.some((i) => i.location.startsWith("HKLM") || /\\program files/i.test(i.location))

  return (
    <motion.div {...panel} className="flex flex-1 flex-col items-center justify-center gap-5 p-8 text-center">
      <div className="flex size-14 items-center justify-center rounded-full bg-primary/12 text-primary">
        <ArchiveRestore className="size-7" />
      </div>
      <div className="max-w-md">
        <p className="text-base font-semibold">
          {items.length}개 항목을 정리할까요?{" "}
          <span className="font-normal text-muted-foreground">({formatBytes(bytes)})</span>
        </p>
        <p className="mt-2 text-sm leading-relaxed text-muted-foreground">
          바로 지우지 않고 <b className="text-foreground">백업 폴더로 옮겨 보관</b>해요. 레지스트리는 원래 값을 그대로
          기록해 두고요. 문제가 생기면 <b className="text-foreground">복구 기록</b>에서 언제든 되돌릴 수 있어요.
        </p>
      </div>
      <ul className="flex w-full max-w-md flex-col gap-2 text-left text-xs">
        {reviewCount > 0 && (
          <li className="flex gap-2 rounded-lg border border-chart-3/40 bg-chart-3/10 p-3 text-chart-3">
            <TriangleAlert className="size-4 shrink-0" />
            검토가 필요한 항목 {reviewCount}개를 직접 골랐어요. 다른 프로그램이 쓰는 것이 아닌지 한 번 더 확인해 주세요.
          </li>
        )}
        {needsAdmin && (
          <li className="flex gap-2 rounded-lg border bg-muted/30 p-3 text-muted-foreground">
            <ShieldAlert className="size-4 shrink-0" />
            일부 항목은 관리자 권한이 있어야 지울 수 있어요. 먼저 지울 수 있는 것부터 정리하고, 나머지는 따로
            물어볼게요.
          </li>
        )}
      </ul>
      <div className="flex gap-2">
        <Button variant="outline" onClick={onCancel}>
          돌아가기
        </Button>
        <Button variant="destructive" onClick={onConfirm}>
          백업하고 정리
        </Button>
      </div>
    </motion.div>
  )
}

export function Cleaning({ label }: { label: string }) {
  return (
    <motion.div {...panel} className="flex flex-1 flex-col items-center justify-center gap-4 p-8 text-center">
      <Loader2 className="size-8 animate-spin text-primary" />
      <p className="font-medium">{label}</p>
    </motion.div>
  )
}

/** 정리 결과. 관리자 권한이 필요한 항목이 남았으면 이어서 처리할 수 있다. */
export function CleanResult({
  op,
  forceAppId,
  onDone,
  onBusyChange,
}: {
  op: Operation
  /** 강제 제거 중이면 대상 프로그램 ID */
  forceAppId?: string
  onDone: () => void
  /** 관리자 권한 정리(UAC)가 도는 동안 true. 그동안 창을 닫지 못하게 하는 데 쓴다. */
  onBusyChange?: (busy: boolean) => void
}) {
  const queryClient = useQueryClient()
  const [current, setCurrent] = useState(op)
  const [elevating, setElevating] = useState(false)
  const [error, setError] = useState<string | null>(null)

  const removed = countStatus(current, "removed") + countStatus(current, "alreadyGone")
  const needsAdmin = countStatus(current, "needsAdmin")
  const problems = current.items.filter((i) => i.status === "failed" || i.status === "blocked")
  const freed = current.items.filter((i) => i.status === "removed").reduce((s, i) => s + (i.sizeBytes ?? 0), 0)

  const elevate = async () => {
    setElevating(true)
    onBusyChange?.(true)
    setError(null)
    try {
      setCurrent(await recoveryApi.cleanElevated(current.id, forceAppId))
    } catch (e) {
      setError(String(e))
    } finally {
      setElevating(false)
      onBusyChange?.(false)
      // 잔여물·강제 제거·시작 항목 정리가 모두 이 화면을 쓴다
      for (const key of ["backups", "installed-apps", "recently-removed", "startup-items"]) {
        queryClient.invalidateQueries({ queryKey: [key] })
      }
    }
  }

  return (
    <motion.div {...panel} className="flex min-h-0 flex-1 flex-col">
      <div className="flex flex-col items-center gap-3 border-b px-6 pt-8 pb-6 text-center">
        <motion.div
          initial={{ scale: 0.8, opacity: 0 }}
          animate={{ scale: 1, opacity: 1 }}
          transition={{ type: "spring", duration: 0.45, bounce: 0.35 }}
          className={cn(
            "flex size-14 items-center justify-center rounded-full",
            needsAdmin || problems.length ? "bg-chart-3/15 text-chart-3" : "bg-primary/15 text-primary",
          )}
        >
          {needsAdmin || problems.length ? (
            <TriangleAlert className="size-7" />
          ) : (
            <Check className="size-7" strokeWidth={2.4} />
          )}
        </motion.div>
        <div>
          <p className="text-base font-semibold">{removed}개를 정리했어요</p>
          <p className="mt-1 text-sm text-muted-foreground">
            백업 폴더로 옮겼어요{freed > 0 && ` (${formatBytes(freed)})`}. 복구 기록에서 언제든 되돌릴 수 있어요.
          </p>
        </div>
        {needsAdmin > 0 && (
          <div className="mt-1 flex items-center gap-3 rounded-xl border bg-muted/30 px-4 py-3 text-left text-sm">
            <ShieldAlert className="size-5 shrink-0 text-chart-3" />
            <span>
              {needsAdmin}개는 관리자 권한이 있어야 지울 수 있어요.
              {error && <span className="block text-xs text-destructive">{error}</span>}
            </span>
            <Button size="sm" onClick={elevate} disabled={elevating}>
              {elevating ? <Loader2 className="animate-spin" /> : <ShieldAlert />}
              관리자 권한으로 정리
            </Button>
          </div>
        )}
      </div>

      {problems.length > 0 && (
        <div className="min-h-0 flex-1 overflow-y-auto px-5 py-3">
          <p className="mb-2 text-xs font-medium text-muted-foreground">정리하지 못한 항목 {problems.length}개</p>
          <ul className="flex flex-col gap-2">
            {problems.map((i) => (
              <li key={i.id} className="rounded-lg border p-3 text-xs">
                <p data-selectable className="font-mono break-all">
                  {i.location}
                  {i.valueName && ` › ${i.valueName}`}
                </p>
                <p className={cn("mt-1", i.status === "blocked" ? "text-primary" : "text-destructive")}>
                  {i.status === "blocked" ? "안전을 위해 건드리지 않았어요: " : ""}
                  {i.error}
                </p>
              </li>
            ))}
          </ul>
        </div>
      )}

      <div className="mt-auto flex items-center justify-end gap-2 border-t px-5 py-3">
        <Button
          variant="outline"
          disabled={elevating}
          onClick={() => {
            onDone()
            navigate("restore")
          }}
        >
          <ArchiveRestore />
          복구 기록 보기
        </Button>
        <Button onClick={onDone} disabled={elevating}>
          완료
        </Button>
      </div>
    </motion.div>
  )
}
