import { useState } from "react"
import { useQuery, useQueryClient } from "@tanstack/react-query"
import { ArchiveRestore, Loader2, ShieldAlert, TriangleAlert } from "lucide-react"
import { motion } from "motion/react"

import {
  countStatus,
  navigate,
  recoveryApi,
  settingsApi,
  type Leftover,
  type Operation,
  type ScanProgress,
} from "@/lib/api"
import { formatBytes } from "@/lib/format"
import { cn } from "@/lib/utils"
import { DrawnCheck } from "@/components/drawn-check"
import { EASE_OUT_QUINT } from "@/components/motion"
import { Button } from "@/components/ui/button"
import { ScanProgressView } from "@/components/scan-progress"
import { useBusy } from "@/lib/busy"

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
  // 보관 기간이 정해져 있으면 그 기간 동안만 되돌릴 수 있다는 것을 정리 전에 알린다
  const settings = useQuery({ queryKey: ["settings"], queryFn: settingsApi.get })
  const days = settings.data?.backupRetentionDays
  const retentionNote =
    days == null ? "백업은 직접 지울 때까지 보관돼요." : `백업은 ${days}일 동안 보관된 뒤 자동으로 지워져요.`
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
          바로 지우지 않고 백업 폴더로 옮겨 보관해요. 레지스트리는 원래 값을 그대로 기록해 두고요. 문제가 생기면 백업을
          보관하는 동안 복구 기록에서 되돌릴 수 있어요. {retentionNote}
        </p>
        {bytes > 0 && (
          <p className="mt-2 text-sm leading-relaxed text-muted-foreground">
            백업도 공간을 차지해서, 정리한 뒤에도 PC의 남은 공간은 바로 늘지 않아요. 백업을 지우면 그때 돌아와요.
          </p>
        )}
      </div>
      <ul className="flex w-full max-w-md flex-col gap-2 text-left text-xs">
        {reviewCount > 0 && (
          <li className="flex gap-2 rounded-lg border border-chart-3/40 bg-chart-3/10 p-3 text-warning">
            <TriangleAlert className="size-4 shrink-0" />
            직접 확인이 필요한 항목 {reviewCount}개를 골랐어요. 다른 프로그램이 쓰는 것이 아닌지 한 번 더 확인해 주세요.
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

/**
 * 정리하는 동안. `progress`를 넘기면(잔여물·강제 제거) 지금 옮기는 항목, 복사한 파일 수, 몇 번째인지, 걸린 시간을 보여 준다.
 * 다른 드라이브의 폴더는 복사해서 옮기므로 몇 분 걸릴 수 있다.
 */
export function Cleaning({ label, progress }: { label: string; progress?: ScanProgress | null }) {
  if (progress !== undefined) {
    return (
      // 확인·완료 화면처럼 가운데에 같은 폭으로 둔다
      <motion.div {...panel} className="flex flex-1 flex-col items-center justify-center p-8">
        <ScanProgressView
          className="w-full max-w-md"
          label={label}
          progress={progress}
          hint="같은 드라이브의 항목은 금방 옮겨지고, 다른 드라이브의 폴더는 복사해서 옮기느라 오래 걸릴 수 있어요."
        />
      </motion.div>
    )
  }
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
  useBusy(elevating)
  const [error, setError] = useState<string | null>(null)

  const removed = countStatus(current, "removed") + countStatus(current, "alreadyGone")
  // 관리자 권한이 없어 일부만 정리된 항목도 관리자 권한으로 마저 정리할 수 있다
  const needsAdmin =
    countStatus(current, "needsAdmin") +
    current.items.filter((i) => i.status === "partial" && i.error?.includes("관리자")).length
  const problems = current.items.filter(
    (i) => i.status === "failed" || i.status === "blocked" || i.status === "partial",
  )
  // 정리한 파일은 지워진 게 아니라 백업 폴더에 있다. 공간은 백업을 지워야 돌아온다.
  const held = current.items.filter((i) => i.status === "removed").reduce((s, i) => s + (i.sizeBytes ?? 0), 0)
  const settings = useQuery({ queryKey: ["settings"], queryFn: settingsApi.get })
  const days = settings.data?.backupRetentionDays

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
    <motion.div {...panel} className="flex min-h-0 flex-1 flex-col">
      {/* 정리하지 못한 항목이 없으면 확인·진행 화면처럼 세로 가운데에, 있으면 위에 두고 아래에 목록을 보인다 */}
      <div
        className={cn(
          "flex flex-col items-center gap-3 px-6 text-center",
          problems.length > 0 ? "border-b pt-8 pb-6" : "flex-1 justify-center py-8",
        )}
      >
        <motion.div
          initial={{ scale: 0.8, opacity: 0 }}
          animate={{ scale: 1, opacity: 1 }}
          transition={{ type: "spring", duration: 0.45, bounce: 0.35 }}
          className={cn(
            "flex size-14 items-center justify-center rounded-full",
            needsAdmin || problems.length ? "bg-chart-3/15 text-warning" : "bg-primary/15 text-primary",
          )}
        >
          {needsAdmin || problems.length ? <TriangleAlert className="size-7" /> : <DrawnCheck className="size-7" />}
        </motion.div>
        <div>
          <p className="text-base font-semibold">{removed}개를 정리했어요</p>
          <p className="mt-1 text-sm text-muted-foreground">
            백업 폴더로 옮겼어요. 백업을 보관하는 동안 복구 기록에서 되돌릴 수 있어요.
          </p>
          {held > 0 && (
            <p className="mt-2 max-w-md text-sm text-muted-foreground">
              <span className="font-medium text-foreground">{formatBytes(held)}</span>는 백업으로 보관 중이라 PC의 남은
              공간은 아직 그대로예요.{" "}
              {days == null
                ? "복구 기록에서 백업을 지우면 공간이 돌아와요."
                : `${days}일 뒤 백업이 자동으로 지워지거나, 복구 기록에서 먼저 지우면 공간이 돌아와요.`}
            </p>
          )}
        </div>
        {needsAdmin > 0 && (
          <div className="mt-1 flex items-center gap-3 rounded-xl border bg-muted/30 px-4 py-3 text-left text-sm">
            <ShieldAlert className="size-5 shrink-0 text-warning" />
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
                  {i.status === "partial" ? "일부만 정리됐어요 (백업은 온전해요): " : ""}
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
