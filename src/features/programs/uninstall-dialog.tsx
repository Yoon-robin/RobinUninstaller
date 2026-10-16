import { useState } from "react"
import { useQueryClient } from "@tanstack/react-query"
import { Check, CircleAlert, Loader2, RotateCcw, ShieldAlert, Trash2, X } from "lucide-react"
import { AnimatePresence, motion } from "motion/react"

import { api, type InstalledApplication, type UninstallEvent, type UninstallOutcome } from "@/lib/api"
import { cn } from "@/lib/utils"
import { EASE_OUT_QUINT } from "@/components/motion"
import { Button } from "@/components/ui/button"
import { Dialog, DialogContent, DialogDescription, DialogTitle } from "@/components/ui/dialog"
import { AppIcon } from "./app-icon"

type Phase =
  | { step: "confirm" }
  | { step: "running"; event: UninstallEvent | null }
  | { step: "done"; outcome: UninstallOutcome }

/** 제거 확인 → 진행 → 결과. 기본 언인스톨러만 실행하고, 파일을 직접 지우지는 않는다. */
export function UninstallDialog({
  app,
  onClose,
  onRemoved,
}: {
  app: InstalledApplication | null
  onClose: () => void
  onRemoved: (app: InstalledApplication) => void
}) {
  const [phase, setPhase] = useState<Phase>({ step: "confirm" })
  const queryClient = useQueryClient()
  const running = phase.step === "running"

  const start = async () => {
    if (!app) return
    setPhase({ step: "running", event: null })
    let outcome: UninstallOutcome
    try {
      outcome = await api.uninstallApp(app.id, (event) => setPhase({ step: "running", event }))
    } catch (e) {
      outcome = { kind: "failed", message: String(e) }
    }
    setPhase({ step: "done", outcome })
    if (outcome.kind === "removed" || outcome.kind === "removedRebootRequired") {
      onRemoved(app)
      queryClient.invalidateQueries({ queryKey: ["installed-apps"] })
    }
  }

  const close = () => {
    if (running) return // 언인스톨러가 도는 동안은 닫지 않는다
    onClose()
    // 닫힘 애니메이션이 끝난 뒤 처음 상태로
    setTimeout(() => setPhase({ step: "confirm" }), 200)
  }

  return (
    <Dialog open={app != null} onOpenChange={(open) => !open && close()}>
      <DialogContent showCloseButton={!running} className="gap-0 overflow-hidden p-0 sm:max-w-md">
        {app && (
          <>
            <div className="flex items-center gap-3 border-b px-5 py-4">
              <AppIcon app={app} className="size-10" />
              <div className="min-w-0">
                <DialogTitle className="truncate text-base">{app.displayName}</DialogTitle>
                <DialogDescription className="truncate">
                  {[app.publisher, app.version].filter(Boolean).join(" · ") || "게시자 정보 없음"}
                </DialogDescription>
              </div>
            </div>

            <AnimatePresence mode="wait" initial={false}>
              <motion.div
                key={phase.step}
                initial={{ opacity: 0, y: 8, filter: "blur(4px)" }}
                animate={{ opacity: 1, y: 0, filter: "blur(0px)" }}
                exit={{ opacity: 0, y: -4, filter: "blur(4px)", transition: { duration: 0.12 } }}
                transition={{ duration: 0.3, ease: EASE_OUT_QUINT }}
              >
                {phase.step === "confirm" && <ConfirmBody app={app} onCancel={close} onStart={start} />}
                {phase.step === "running" && <RunningBody event={phase.event} />}
                {phase.step === "done" && <ResultBody outcome={phase.outcome} onClose={close} onRetry={start} />}
              </motion.div>
            </AnimatePresence>
          </>
        )}
      </DialogContent>
    </Dialog>
  )
}

function ConfirmBody({
  app,
  onCancel,
  onStart,
}: {
  app: InstalledApplication
  onCancel: () => void
  onStart: () => void
}) {
  const needsAdmin = app.scope !== "user"
  return (
    <div className="flex flex-col gap-4 p-5">
      <p className="text-sm leading-relaxed">
        이 프로그램의 <b>기본 언인스톨러</b>를 실행해요. 언인스톨러 창이 열리면 안내에 따라 제거를 진행해 주세요.
      </p>
      <ul className="flex flex-col gap-2 rounded-xl border bg-muted/30 p-3.5 text-xs text-muted-foreground">
        {needsAdmin && (
          <li className="flex gap-2">
            <ShieldAlert className="size-4 shrink-0 text-chart-3" />
            모든 사용자용 프로그램이라 관리자 권한 요청(UAC)이 뜰 수 있어요.
          </li>
        )}
        <li className="flex gap-2">
          <Check className="size-4 shrink-0 text-primary" />
          RobinUninstaller는 파일이나 레지스트리를 직접 지우지 않아요.
        </li>
        <li className="flex gap-2">
          <Check className="size-4 shrink-0 text-primary" />
          끝나면 정말 제거됐는지 다시 확인해요.
        </li>
      </ul>
      <div className="flex justify-end gap-2">
        <Button variant="outline" onClick={onCancel}>
          취소
        </Button>
        <Button variant="destructive" onClick={onStart}>
          <Trash2 />
          제거 시작
        </Button>
      </div>
    </div>
  )
}

const steps = [
  { key: "launching", label: "언인스톨러 실행", hint: "관리자 권한 요청이 뜨면 허용해 주세요" },
  { key: "running", label: "제거 진행 중", hint: "언인스톨러 창의 안내를 따라 주세요" },
  { key: "verifying", label: "제거 확인", hint: "설치 정보를 다시 읽고 있어요" },
] as const

function RunningBody({ event }: { event: UninstallEvent | null }) {
  const current = event == null ? 0 : steps.findIndex((s) => s.key === event.kind)

  return (
    <ol className="flex flex-col gap-1 p-5">
      {steps.map((s, i) => {
        const state = i < current ? "done" : i === current ? "active" : "todo"
        return (
          <li key={s.key} className="flex items-start gap-3 rounded-lg px-2 py-2">
            <span
              className={cn(
                "mt-0.5 flex size-5 shrink-0 items-center justify-center rounded-full border transition-colors duration-300",
                state === "done" && "border-primary bg-primary text-primary-foreground",
                state === "active" && "border-primary text-primary",
                state === "todo" && "text-muted-foreground/50",
              )}
            >
              <AnimatePresence mode="popLayout" initial={false}>
                {state === "done" ? (
                  <motion.span
                    key="done"
                    initial={{ scale: 0.6, opacity: 0 }}
                    animate={{ scale: 1, opacity: 1 }}
                    transition={{ type: "spring", duration: 0.3, bounce: 0.3 }}
                  >
                    <Check className="size-3" strokeWidth={3} />
                  </motion.span>
                ) : state === "active" ? (
                  <motion.span key="active" initial={{ opacity: 0 }} animate={{ opacity: 1 }}>
                    <Loader2 className="size-3 animate-spin" />
                  </motion.span>
                ) : (
                  <span className="size-1.5 rounded-full bg-current" />
                )}
              </AnimatePresence>
            </span>
            <div>
              <p className={cn("text-sm font-medium", state === "todo" && "text-muted-foreground")}>{s.label}</p>
              {state === "active" && <p className="text-xs text-muted-foreground">{s.hint}</p>}
            </div>
          </li>
        )
      })}
    </ol>
  )
}

function ResultBody({
  outcome,
  onClose,
  onRetry,
}: {
  outcome: UninstallOutcome
  onClose: () => void
  onRetry: () => void
}) {
  const view = describe(outcome)
  return (
    <div className="flex flex-col items-center gap-3 p-6 text-center">
      <motion.div
        initial={{ scale: 0.8, opacity: 0 }}
        animate={{ scale: 1, opacity: 1 }}
        transition={{ type: "spring", duration: 0.45, bounce: 0.35, delay: 0.05 }}
        className={cn("flex size-14 items-center justify-center rounded-full", view.toneClass)}
      >
        <view.icon className="size-7" strokeWidth={2.4} />
      </motion.div>
      <div>
        <p className="text-base font-semibold">{view.title}</p>
        <p className="mt-1 text-sm leading-relaxed text-muted-foreground">{view.body}</p>
      </div>
      <div className="mt-2 flex gap-2">
        {view.retry && (
          <Button variant="outline" onClick={onRetry}>
            <RotateCcw />
            다시 시도
          </Button>
        )}
        <Button onClick={onClose}>닫기</Button>
      </div>
    </div>
  )
}

function describe(outcome: UninstallOutcome) {
  switch (outcome.kind) {
    case "removed":
      return {
        icon: Check,
        toneClass: "bg-primary/15 text-primary",
        title: "제거했어요",
        body: "설치 정보에서 사라진 것을 확인했어요. 남은 파일 찾기는 다음 업데이트에서 지원해요.",
        retry: false,
      }
    case "removedRebootRequired":
      return {
        icon: RotateCcw,
        toneClass: "bg-chart-3/15 text-chart-3",
        title: "제거했어요 · 재시작 필요",
        body: "일부 파일은 PC를 다시 시작해야 완전히 지워져요.",
        retry: false,
      }
    case "stillInstalled":
      return {
        icon: CircleAlert,
        toneClass: "bg-chart-3/15 text-chart-3",
        title: "아직 설치되어 있어요",
        body:
          outcome.exitCode === 1602
            ? "언인스톨러에서 취소한 것 같아요."
            : `언인스톨러가 끝났지만 프로그램이 그대로 남아 있어요.${outcome.exitCode != null ? ` (종료 코드 ${outcome.exitCode})` : ""}`,
        retry: true,
      }
    case "elevationDenied":
      return {
        icon: ShieldAlert,
        toneClass: "bg-chart-3/15 text-chart-3",
        title: "관리자 권한이 거절됐어요",
        body: "이 프로그램을 지우려면 관리자 권한 요청을 허용해야 해요.",
        retry: true,
      }
    case "failed":
      return {
        icon: X,
        toneClass: "bg-destructive/15 text-destructive",
        title: "제거하지 못했어요",
        body: outcome.message,
        retry: true,
      }
  }
}
