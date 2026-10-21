import { useLayoutEffect, useRef, useState } from "react"
import { useQueryClient } from "@tanstack/react-query"
import { ArchiveRestore, FileWarning, Trash2 } from "lucide-react"
import { AnimatePresence, motion } from "motion/react"

import { startupApi, type Operation, type StartupEntry } from "@/lib/api"
import { EASE_OUT_QUINT } from "@/components/motion"
import { Button } from "@/components/ui/button"
import { Checkbox } from "@/components/ui/checkbox"
import { Dialog, DialogContent, DialogDescription, DialogTitle } from "@/components/ui/dialog"
import { CleanResult, Cleaning } from "@/features/leftovers/clean-flow"
import { useBusy } from "@/lib/busy"

type Stage = { step: "select" } | { step: "cleaning" } | { step: "done"; op: Operation }

/** 실행 파일이 없는 시작 항목 정리. 예약 작업은 끄기만 할 수 있어서 목록에서 뺀다. */
export function MissingCleanupDialog({
  entries,
  open,
  onClose,
}: {
  entries: StartupEntry[]
  open: boolean
  onClose: () => void
}) {
  const queryClient = useQueryClient()
  const targets = entries.filter((e) => e.targetMissing && e.source !== "task")
  const skippedTasks = entries.filter((e) => e.targetMissing && e.source === "task").length
  const [selected, setSelected] = useState<Set<string>>(new Set())
  const [stage, setStage] = useState<Stage>({ step: "select" })
  useBusy(stage.step === "cleaning")
  const [error, setError] = useState<string | null>(null)
  const [elevating, setElevating] = useState(false)
  useBusy(elevating)

  // 열 때마다 처음 상태로. 현재 사용자 항목만 미리 고르고, 모든 사용자(관리자) 항목은 직접 고르게 한다.
  // 화면에 그리기 전에 바꿔서 지난 결과가 잠깐 보이지 않게 한다.
  useLayoutEffect(() => {
    if (!open) return
    setStage({ step: "select" })
    setError(null)
    setElevating(false)
    setSelected(
      new Set(entries.filter((e) => e.targetMissing && e.source !== "task" && !e.needsAdmin).map((e) => e.id)),
    )
  }, [open])

  const toggle = (id: string) =>
    setSelected((s) => {
      const next = new Set(s)
      if (next.has(id)) next.delete(id)
      else next.add(id)
      return next
    })

  // 사라지는 중인 화면의 단추도 잠깐 눌리므로, 두 번 눌러도 한 번만 정리한다
  const cleaning = useRef(false)
  const run = async () => {
    if (cleaning.current) return
    cleaning.current = true
    setStage({ step: "cleaning" })
    setError(null)
    try {
      // 목록이 그사이 바뀌었으면 지금 있는 항목만 보낸다
      const ids = targets.map((e) => e.id).filter((id) => selected.has(id))
      const op = await startupApi.cleanMissing(ids)
      setStage({ step: "done", op })
    } catch (e) {
      setError(String(e))
      setStage({ step: "select" })
    } finally {
      cleaning.current = false
      queryClient.invalidateQueries({ queryKey: ["startup-items"] })
      queryClient.invalidateQueries({ queryKey: ["backups"] })
    }
  }

  const close = () => {
    if (stage.step === "cleaning" || elevating) return
    queryClient.invalidateQueries({ queryKey: ["startup-items"] })
    onClose()
  }

  return (
    <Dialog open={open} onOpenChange={(o) => !o && close()}>
      <DialogContent
        showCloseButton={stage.step !== "cleaning" && !elevating}
        className="flex max-h-[min(640px,84vh)] flex-col gap-0 overflow-hidden p-0 sm:max-w-xl"
      >
        <div className="flex items-center gap-3 border-b px-5 py-4 pr-12">
          <div className="flex size-10 items-center justify-center rounded-xl bg-chart-3/12 text-warning">
            <FileWarning className="size-5" />
          </div>
          <div className="min-w-0">
            <DialogTitle className="text-base">파일 없는 시작 항목 정리</DialogTitle>
            <DialogDescription>실행 파일이 사라져 로그인할 때 아무것도 실행하지 못하는 항목이에요</DialogDescription>
          </div>
        </div>

        <AnimatePresence mode="wait" initial={false}>
          {stage.step === "cleaning" ? (
            <Cleaning key="cleaning" label="백업하고 정리하는 중이에요" progress={null} />
          ) : stage.step === "done" ? (
            <CleanResult key="done" op={stage.op} onDone={close} onBusyChange={setElevating} />
          ) : (
            <motion.div
              key="select"
              initial={{ opacity: 0, y: 8 }}
              animate={{ opacity: 1, y: 0 }}
              exit={{ opacity: 0, transition: { duration: 0.12 } }}
              transition={{ duration: 0.3, ease: EASE_OUT_QUINT }}
              className="flex min-h-0 flex-1 flex-col"
            >
              <p className="flex gap-2 border-b bg-muted/20 px-5 py-2.5 text-xs leading-relaxed text-muted-foreground">
                <ArchiveRestore className="mt-0.5 size-3.5 shrink-0 text-primary" />
                <span>
                  레지스트리의 자동 실행 값이나 시작 폴더 바로 가기, 꺼 둔 기록을 백업한 뒤 정리해요. 백업을 보관하는
                  동안 복구 기록에서 되돌릴 수 있어요. 드라이브를 뺐거나 네트워크 위치라 잠깐 안 보이는 파일일 수도
                  있으니 이름을 확인하세요. 모든 사용자 항목은 미리 고르지 않았어요.
                  {skippedTasks > 0 && ` 예약 작업 ${skippedTasks}개는 끄기만 할 수 있어 목록에서 뺐어요.`}
                </span>
              </p>
              {targets.length === 0 ? (
                <p className="px-5 py-12 text-center text-sm text-muted-foreground">정리할 항목이 없어요.</p>
              ) : (
                <ul className="min-h-0 flex-1 divide-y overflow-y-auto">
                  {targets.map((e) => (
                    <li key={e.id}>
                      <label className="flex cursor-pointer items-start gap-3 px-5 py-3 hover:bg-accent/40">
                        <Checkbox
                          className="mt-0.5"
                          checked={selected.has(e.id)}
                          onCheckedChange={() => toggle(e.id)}
                        />
                        <div className="min-w-0 text-xs">
                          <p className="text-sm font-medium">{e.name}</p>
                          <p data-selectable className="mt-0.5 font-mono break-all text-muted-foreground">
                            {e.targetPath}
                          </p>
                          <p className="mt-0.5 text-muted-foreground">
                            {e.source.startsWith("folder") ? "시작 폴더 바로 가기" : "레지스트리 자동 실행 값"}
                            {e.needsAdmin && " · 관리자 권한 필요"}
                          </p>
                        </div>
                      </label>
                    </li>
                  ))}
                </ul>
              )}
              <div className="flex items-center gap-3 border-t px-5 py-3">
                <p className="mr-auto text-sm text-muted-foreground tabular-nums">
                  {selected.size > 0 ? `${selected.size}개 선택` : "선택한 항목 없음"}
                </p>
                {error && <p className="text-xs text-destructive">{error}</p>}
                <Button variant="outline" onClick={close}>
                  닫기
                </Button>
                <Button variant="destructive" disabled={selected.size === 0} onClick={run}>
                  <Trash2 />
                  {selected.size > 0 ? `${selected.size}개 백업하고 정리` : "백업하고 정리"}
                </Button>
              </div>
            </motion.div>
          )}
        </AnimatePresence>
      </DialogContent>
    </Dialog>
  )
}
