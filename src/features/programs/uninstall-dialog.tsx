import { useEffect, useRef, useState } from "react"
import { useQuery, useQueryClient } from "@tanstack/react-query"
import {
  Check,
  ChevronRight,
  CircleAlert,
  ExternalLink,
  Info,
  Loader2,
  RefreshCw,
  RotateCcw,
  ShieldAlert,
  Sparkles,
  Trash2,
  X,
} from "lucide-react"
import { toast } from "sonner"
import { AnimatePresence, motion } from "motion/react"

import { api, storeApi, type InstalledApplication, type UninstallEvent, type UninstallOutcome } from "@/lib/api"
import { cn } from "@/lib/utils"
import { useLastDefined } from "@/hooks/use-last-defined"
import { DrawnCheck } from "@/components/drawn-check"
import { AutoHeight, EASE_OUT_QUINT, focusMarked } from "@/components/motion"
import { Button } from "@/components/ui/button"
import { Dialog, DialogContent, DialogDescription, DialogTitle } from "@/components/ui/dialog"
import { AppIcon } from "./app-icon"
import { useBusy } from "@/lib/busy"

type Phase =
  { step: "confirm" } | { step: "running"; event: UninstallEvent | null } | { step: "done"; outcome: UninstallOutcome }

/** 제거 확인 → 진행 → 결과. 기본 언인스톨러만 실행하고, 파일을 직접 지우지는 않는다. */
export function UninstallDialog({
  app: appProp,
  onClose,
  onRemoved,
  onFindLeftovers,
  onForceUninstall,
}: {
  app: InstalledApplication | null
  onClose: () => void
  onRemoved: (app: InstalledApplication) => void
  onFindLeftovers: (app: InstalledApplication) => void
  /** 제거 명령을 쓸 수 없을 때 강제 제거로 이어 간다 (없으면 그 버튼을 감춘다) */
  onForceUninstall?: (app: InstalledApplication) => void
}) {
  // 닫히는 동안에도 내용이 남아 있게 마지막 대상을 그린다
  const app = useLastDefined(appProp)
  const [phase, setPhase] = useState<Phase>({ step: "confirm" })
  const queryClient = useQueryClient()
  const running = phase.step === "running"
  useBusy(running)
  // 사라지는 중인 화면의 단추도 잠깐 눌리므로, 두 번 눌러도 한 번만 시작한다
  const starting = useRef(false)

  const start = async () => {
    if (!app || starting.current) return
    starting.current = true
    setPhase({ step: "running", event: null })
    let outcome: UninstallOutcome
    try {
      outcome = await api.uninstallApp(app.id, (event) => setPhase({ step: "running", event }))
    } catch (e) {
      outcome = { kind: "failed", message: String(e) }
    } finally {
      starting.current = false
    }
    setPhase({ step: "done", outcome })
    if (outcome.kind === "removed" || outcome.kind === "removedRebootRequired") removedApp.current = app
  }

  // 제거한 프로그램은 창을 닫은 뒤에 목록에서 뺀다 (접히며 사라지는 모습이 창 뒤에서 끝나 버리지 않게)
  const removedApp = useRef<InstalledApplication | null>(null)
  const flushRemoved = () => {
    const removed = removedApp.current
    if (!removed) return
    removedApp.current = null
    onRemoved(removed)
    queryClient.invalidateQueries({ queryKey: ["installed-apps"] })
    queryClient.invalidateQueries({ queryKey: ["recently-removed"] })
  }

  // 언인스톨러를 다시 실행하지 않고, 지금 설치 목록에 아직 있는지만 다시 본다
  const [rechecking, setRechecking] = useState(false)
  const recheck = async () => {
    if (!app) return
    setRechecking(true)
    try {
      const list = await queryClient.fetchQuery({
        queryKey: ["installed-apps"],
        queryFn: api.listInstalledApps,
        staleTime: 0,
      })
      if (!list.some((a) => a.id === app.id)) {
        removedApp.current = app
        setPhase({ step: "done", outcome: { kind: "removed" } })
      } else {
        toast.info("아직 설치 목록에 있어요")
      }
    } catch (e) {
      toast.error("설치 목록을 다시 읽지 못했어요", { description: String(e) })
    } finally {
      setRechecking(false)
    }
  }

  const close = () => {
    if (running) return // 언인스톨러가 도는 동안은 닫지 않는다
    flushRemoved()
    onClose()
    // 닫힘 애니메이션이 끝난 뒤 처음 상태로
    setTimeout(() => setPhase({ step: "confirm" }), 200)
  }

  return (
    <Dialog open={appProp != null} onOpenChange={(open) => !open && close()}>
      <DialogContent
        showCloseButton={!running}
        className="gap-0 overflow-hidden p-0 sm:max-w-md"
        onOpenAutoFocus={focusMarked}
      >
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

            {/* 확인 → 진행 → 결과로 높이가 바뀌어도 창이 한 번에 튀지 않게 */}
            <AutoHeight>
              <AnimatePresence mode="wait" initial={false}>
                <motion.div
                  key={phase.step}
                  initial={{ opacity: 0, y: 6 }}
                  animate={{ opacity: 1, y: 0 }}
                  exit={{ opacity: 0, y: -3, transition: { duration: 0.1 } }}
                  transition={{ duration: 0.25, ease: EASE_OUT_QUINT }}
                >
                  {phase.step === "confirm" && <ConfirmBody app={app} onCancel={close} onStart={start} />}
                  {phase.step === "running" && <RunningBody event={phase.event} />}
                  {phase.step === "done" && (
                    <ResultBody
                      outcome={phase.outcome}
                      onClose={close}
                      onRetry={start}
                      onRecheck={recheck}
                      rechecking={rechecking}
                      onFindLeftovers={() => {
                        close()
                        onFindLeftovers(app)
                      }}
                      onForceUninstall={
                        onForceUninstall &&
                        (() => {
                          close()
                          onForceUninstall(app)
                        })
                      }
                    />
                  )}
                </motion.div>
              </AnimatePresence>
            </AutoHeight>
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
        이 프로그램에 들어 있는 언인스톨러를 실행해요. 언인스톨러 창이 열리면 안내에 따라 진행해 주세요.
      </p>
      <ul className="flex flex-col gap-2 rounded-xl border bg-muted/30 p-3.5 text-xs text-muted-foreground">
        {needsAdmin && (
          <li className="flex gap-2">
            <ShieldAlert className="size-4 shrink-0 text-warning" />
            모든 사용자용 프로그램이라 관리자 권한 요청(UAC)이 뜰 수 있어요.
          </li>
        )}
        <li className="flex gap-2">
          <Check className="size-4 shrink-0 text-primary" />이 단계에서 RobinX는 파일이나 레지스트리를 지우지 않아요.
        </li>
        <li className="flex gap-2">
          <Check className="size-4 shrink-0 text-primary" />
          끝나면 설치 정보를 다시 읽어 제거됐는지 확인해요.
        </li>
        <li className="flex gap-2">
          <Info className="size-4 shrink-0" />
          제거한 프로그램은 RobinX로 되살릴 수 없어요. 다시 쓰려면 새로 설치해야 해요.
        </li>
      </ul>
      <PlannedCommand id={app.id} />
      <div className="mt-2 flex justify-end gap-2">
        <Button variant="outline" onClick={onCancel} data-autofocus>
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

/** 언인스톨러 창을 닫았는데도 프로세스가 남아 기다림이 끝나지 않을 때를 위한 탈출구. */
const STUCK_HINT_AFTER_MS = 8000

function RunningBody({ event }: { event: UninstallEvent | null }) {
  const current = event == null ? 0 : steps.findIndex((s) => s.key === event.kind)
  const [showSkip, setShowSkip] = useState(false)
  const [skipping, setSkipping] = useState(false)

  useEffect(() => {
    const t = setTimeout(() => setShowSkip(true), STUCK_HINT_AFTER_MS)
    return () => clearTimeout(t)
  }, [])

  return (
    <div className="p-5">
      <ol className="flex flex-col gap-1">
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
      <AnimatePresence>
        {showSkip && event?.kind === "running" && (
          <motion.div
            initial={{ opacity: 0, y: 6 }}
            animate={{ opacity: 1, y: 0 }}
            exit={{ opacity: 0 }}
            transition={{ duration: 0.3, ease: EASE_OUT_QUINT }}
            className="mt-3 flex items-center justify-between gap-3 rounded-lg border bg-muted/30 px-3 py-2.5 text-xs text-muted-foreground"
          >
            <span>언인스톨러 창이 이미 닫혔나요?</span>
            <Button
              size="sm"
              variant="outline"
              disabled={skipping}
              onClick={() => {
                setSkipping(true)
                api.stopWaitingUninstall()
              }}
            >
              기다리지 않고 확인
            </Button>
          </motion.div>
        )}
      </AnimatePresence>
    </div>
  )
}

/** 실제로 실행할 명령을 Rust에서 받아 보여 준다. 실행할 수 없는 명령이면 접힌 칸을 두지 않는다. */
function PlannedCommand({ id }: { id: string }) {
  const planned = useQuery({
    queryKey: ["uninstall-command", id],
    queryFn: () => api.uninstallCommandPreview(id),
    retry: false,
  })
  return <CommandPreview command={planned.data ?? null} />
}

function CommandPreview({ command }: { command: string | null }) {
  const [open, setOpen] = useState(false)
  if (!command) return null
  return (
    <div className="text-xs">
      <button
        type="button"
        onClick={() => setOpen((v) => !v)}
        className="flex items-center gap-1 text-muted-foreground hover:text-foreground"
        aria-expanded={open}
      >
        <ChevronRight
          className={cn("size-3.5 transition-transform duration-300 ease-(--ease-out-quint)", open && "rotate-90")}
        />
        실행할 명령 보기
      </button>
      <AnimatePresence initial={false}>
        {open && (
          <motion.div
            initial={{ height: 0, opacity: 0 }}
            animate={{ height: "auto", opacity: 1 }}
            exit={{ height: 0, opacity: 0 }}
            transition={{ duration: 0.25, ease: EASE_OUT_QUINT }}
            className="overflow-hidden"
          >
            <p data-selectable className="mt-2 rounded-lg border bg-muted/40 p-2.5 font-mono leading-5 break-all">
              {command}
            </p>
          </motion.div>
        )}
      </AnimatePresence>
    </div>
  )
}

function ResultBody({
  outcome,
  onClose,
  onRetry,
  onRecheck,
  rechecking,
  onFindLeftovers,
  onForceUninstall,
}: {
  outcome: UninstallOutcome
  onClose: () => void
  onRetry: () => void
  onRecheck: () => void
  rechecking: boolean
  onFindLeftovers: () => void
  onForceUninstall?: () => void
}) {
  const removed = outcome.kind === "removed" || outcome.kind === "removedRebootRequired"
  const view = describe(outcome)
  return (
    <div className="flex flex-col items-center gap-3 p-6 text-center">
      <motion.div
        initial={{ scale: 0.8, opacity: 0 }}
        animate={{ scale: 1, opacity: 1 }}
        transition={{ type: "spring", duration: 0.45, bounce: 0.35, delay: 0.05 }}
        className={cn("flex size-14 items-center justify-center rounded-full", view.toneClass)}
      >
        {view.icon === Check ? <DrawnCheck className="size-7" /> : <view.icon className="size-7" strokeWidth={2.4} />}
      </motion.div>
      <div>
        <p className="text-base font-semibold">{view.title}</p>
        {outcome.kind === "failed" ? (
          // 실패 이유는 고르고 복사할 수 있게 (문제를 알려 줄 때 쓰도록, ErrorState와 같은 모양)
          <p data-selectable className="mt-2 max-w-md font-mono text-xs break-all text-muted-foreground select-text">
            {view.body}
          </p>
        ) : (
          <p className="mt-1 text-sm leading-relaxed text-muted-foreground">{view.body}</p>
        )}
      </div>
      {outcome.kind === "cannotRun" && outcome.command && <CommandPreview command={outcome.command} />}
      {removed && (
        <p className="max-w-md text-xs text-muted-foreground">
          나중에 찾아도 돼요. 잔여물 정리 화면의 "제거한 프로그램"에 남아 있어요.
        </p>
      )}
      <div className="mt-2 flex gap-2">
        {view.retry === "recheck" && (
          // 종료 코드를 모르면 언인스톨러가 아직 열려 있을 수 있다. 다시 실행하지 않고 다시 확인만 한다.
          <Button variant="outline" onClick={onRecheck} disabled={rechecking}>
            <RefreshCw className={cn(rechecking && "animate-spin")} />
            다시 확인
          </Button>
        )}
        {view.retry === true && (
          <Button variant="outline" onClick={onRetry}>
            <RotateCcw />
            다시 시도
          </Button>
        )}
        {outcome.kind === "cannotRun" ? (
          <>
            <Button variant="outline" onClick={onClose}>
              닫기
            </Button>
            <Button variant="outline" onClick={() => storeApi.openSettings().catch((e) => toast.error(String(e)))}>
              <ExternalLink />
              Windows 설정에서 제거
            </Button>
            {onForceUninstall && (
              <Button onClick={onForceUninstall}>
                <Sparkles />
                강제 제거
              </Button>
            )}
          </>
        ) : removed ? (
          <>
            <Button variant="outline" onClick={onClose}>
              닫기
            </Button>
            <Button onClick={onFindLeftovers}>
              <Sparkles />
              남은 흔적 찾기
            </Button>
          </>
        ) : (
          <Button onClick={onClose}>닫기</Button>
        )}
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
        body: "설치 정보에서 사라진 것을 확인했어요. 언인스톨러가 남긴 파일과 레지스트리를 찾아볼까요?",
        retry: false,
      }
    case "removedRebootRequired":
      return {
        icon: RotateCcw,
        toneClass: "bg-chart-3/15 text-warning",
        title: "제거했어요 · 재시작 필요",
        body: "일부 파일은 PC를 다시 시작해야 완전히 지워져요.",
        retry: false,
      }
    case "stillInstalled":
      return {
        icon: CircleAlert,
        toneClass: "bg-chart-3/15 text-warning",
        title: "아직 설치돼 있어요",
        body:
          outcome.exitCode === 1602
            ? "언인스톨러에서 취소한 것 같아요."
            : outcome.exitCode == null
              ? "언인스톨러가 아직 열려 있다면 마저 진행한 뒤 다시 확인해 주세요."
              : `언인스톨러가 끝났지만 프로그램이 그대로 남아 있어요. (종료 코드 ${outcome.exitCode})`,
        retry: outcome.exitCode == null ? ("recheck" as const) : true,
      }
    case "elevationDenied":
      return {
        icon: ShieldAlert,
        toneClass: "bg-chart-3/15 text-warning",
        title: "관리자 권한이 거절됐어요",
        body: "이 프로그램을 지우려면 관리자 권한 요청을 허용해야 해요.",
        retry: true,
      }
    case "cannotRun":
      return {
        icon: CircleAlert,
        toneClass: "bg-chart-3/15 text-warning",
        title: "이 프로그램의 제거 명령은 쓸 수 없어요",
        body: `${outcome.reason}. Windows 설정에서 제거하거나, RobinX가 백업한 뒤 등록 정보와 파일을 정리하는 강제 제거를 쓸 수 있어요.`,
        retry: false,
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
