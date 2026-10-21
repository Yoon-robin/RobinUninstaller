import { useEffect, useRef, useState } from "react"
import { useQueryClient } from "@tanstack/react-query"
import {
  Check,
  CircleAlert,
  CircleStop,
  Clock,
  Info,
  Loader2,
  ShieldAlert,
  Sparkles,
  Trash2,
  TriangleAlert,
} from "lucide-react"

import { api, needsForce, type InstalledApplication, type UninstallOutcome } from "@/lib/api"
import { cn } from "@/lib/utils"
import { useLastDefined } from "@/hooks/use-last-defined"
import { focusMarked } from "@/components/motion"
import { Button } from "@/components/ui/button"
import { Dialog, DialogContent, DialogDescription, DialogTitle } from "@/components/ui/dialog"
import { AppIcon } from "./app-icon"
import { useBusy } from "@/lib/busy"

type Status =
  | { kind: "waiting" }
  | { kind: "running"; stage: string }
  | { kind: "done"; outcome: UninstallOutcome }
  | { kind: "skipped" }

const isRemoved = (s: Status) =>
  s.kind === "done" && (s.outcome.kind === "removed" || s.outcome.kind === "removedRebootRequired")

function outcomeLabel(outcome: UninstallOutcome): { text: string; tone: string } {
  switch (outcome.kind) {
    case "removed":
      return { text: "제거됨", tone: "text-primary" }
    case "removedRebootRequired":
      return { text: "제거됨 · 다시 시작 필요", tone: "text-primary" }
    case "stillInstalled":
      return { text: "아직 설치돼 있어요 (취소했거나 실패)", tone: "text-warning" }
    case "elevationDenied":
      return { text: "관리자 권한 요청이 거절됐어요", tone: "text-warning" }
    case "cannotRun":
      return { text: `제거 명령을 쓸 수 없어요: ${outcome.reason}`, tone: "text-warning" }
    case "failed":
      return { text: `실패: ${outcome.message}`, tone: "text-destructive" }
  }
}

/** 언인스톨러 창을 닫았는데도 프로세스가 남아 기다림이 끝나지 않을 때를 위한 탈출구. */
const STUCK_HINT_AFTER_MS = 8000

/**
 * 여러 프로그램을 차례로 제거한다. 언인스톨러는 한 번에 하나씩만 실행된다 (Windows Installer도 동시에 하나만 처리한다).
 * 각 언인스톨러 창의 안내는 사용자가 직접 따른다. 언인스톨러가 없는 프로그램은 강제 제거라 여기서 빼고, 하나씩 하게 한다.
 */
export function BatchUninstallDialog({
  apps: appsProp,
  hidden = false,
  onClose,
  onStarted,
  onRemoved,
  onFindLeftovers,
}: {
  apps: InstalledApplication[] | null
  /** 결과에서 흔적 찾기 창을 연 동안 잠깐 감춘다 (결과는 그대로 남아 있다가 다시 보인다) */
  hidden?: boolean
  onClose: () => void
  /** 제거를 시작했을 때 (이때 고르기 모드를 끝낸다. 확인 창에서 취소하면 고른 것이 그대로 남는다) */
  onStarted?: () => void
  onRemoved: (app: InstalledApplication) => void
  onFindLeftovers: (app: InstalledApplication) => void
}) {
  // 닫히는 동안에도 내용이 남아 있게 마지막 목록을 그린다
  const apps = useLastDefined(appsProp)
  const queryClient = useQueryClient()
  const [phase, setPhase] = useState<"confirm" | "running" | "done">("confirm")
  const [statuses, setStatuses] = useState<Record<string, Status>>({})
  const [current, setCurrent] = useState<string | null>(null)
  const [stuck, setStuck] = useState(false)
  const [skipping, setSkipping] = useState(false)
  const stopAfterCurrent = useRef(false)
  const [stopping, setStopping] = useState(false)

  const runnable = (apps ?? []).filter((a) => !needsForce(a))
  const forced = (apps ?? []).filter(needsForce)
  const needsAdmin = runnable.some((a) => a.scope !== "user")

  // 열 때마다 처음 상태로
  useEffect(() => {
    if (!apps) return
    setPhase("confirm")
    setStatuses({})
    setCurrent(null)
    stopAfterCurrent.current = false
    setStopping(false)
  }, [apps])

  // 같은 프로그램을 오래 기다리면 "기다리지 않고 확인"을 보여 준다
  useEffect(() => {
    setStuck(false)
    setSkipping(false)
    if (!current) return
    const t = setTimeout(() => setStuck(true), STUCK_HINT_AFTER_MS)
    return () => clearTimeout(t)
  }, [current])

  const setStatus = (id: string, status: Status) => setStatuses((s) => ({ ...s, [id]: status }))

  const run = async () => {
    setPhase("running")
    onStarted?.()
    for (const app of runnable) {
      if (stopAfterCurrent.current) {
        setStatus(app.id, { kind: "skipped" })
        continue
      }
      setCurrent(app.id)
      setStatus(app.id, { kind: "running", stage: "언인스톨러 실행 중" })
      let outcome: UninstallOutcome
      try {
        outcome = await api.uninstallApp(app.id, (event) =>
          setStatus(app.id, {
            kind: "running",
            stage:
              event.kind === "launching"
                ? '언인스톨러 실행 중 · 관리자 권한 확인 창(UAC)이 뜨면 "예"를 눌러 주세요'
                : event.kind === "running"
                  ? "제거 진행 중 · 언인스톨러 창의 안내를 따라 주세요"
                  : "제거됐는지 확인하는 중",
          }),
        )
      } catch (e) {
        outcome = { kind: "failed", message: String(e) }
      }
      setStatus(app.id, { kind: "done", outcome })
      if (outcome.kind === "removed" || outcome.kind === "removedRebootRequired") {
        onRemoved(app)
      }
    }
    setCurrent(null)
    setPhase("done")
    // 목록 다시 읽기(와 그에 따른 크기 재기)는 다 끝난 뒤 한 번만
    queryClient.invalidateQueries({ queryKey: ["installed-apps"] })
    queryClient.invalidateQueries({ queryKey: ["recently-removed"] })
  }

  const running = phase === "running"
  useBusy(running)
  const close = () => {
    if (!running) onClose()
  }
  const removed = runnable.filter((a) => statuses[a.id] && isRemoved(statuses[a.id]))
  const finished = runnable.filter((a) => statuses[a.id]?.kind === "done" || statuses[a.id]?.kind === "skipped").length

  return (
    <Dialog open={appsProp != null && !hidden} onOpenChange={(o) => !o && close()}>
      <DialogContent
        showCloseButton={!running}
        className="flex max-h-[min(720px,84vh)] flex-col gap-0 overflow-hidden p-0 sm:max-w-xl"
        onOpenAutoFocus={focusMarked}
      >
        <div className="flex items-center gap-3 border-b px-5 py-4 pr-12">
          <div className="flex size-10 items-center justify-center rounded-xl bg-destructive/12 text-destructive">
            <Trash2 className="size-5" />
          </div>
          <div className="min-w-0">
            <DialogTitle className="text-base">
              {phase === "done" ? "한꺼번에 제거 결과" : `프로그램 ${runnable.length}개 제거`}
            </DialogTitle>
            <DialogDescription>
              {phase === "confirm"
                ? "각 프로그램의 기본 언인스톨러를 하나씩 차례로 실행해요"
                : phase === "running"
                  ? `${Math.min(finished + 1, runnable.length)} / ${runnable.length}번째 · 언인스톨러 창이 열리면 안내에 따라 진행해 주세요`
                  : `${removed.length}개를 제거했어요. 남은 흔적은 지금 찾거나, 나중에 남은 흔적 정리 화면에서 찾을 수 있어요`}
            </DialogDescription>
          </div>
        </div>

        {phase === "confirm" && (
          <ul className="flex flex-col gap-2 border-b bg-muted/20 px-5 py-3 text-xs text-muted-foreground">
            {needsAdmin && (
              <li className="flex gap-2">
                <ShieldAlert className="size-4 shrink-0 text-warning" />
                모든 사용자용 프로그램이 있어 관리자 권한 확인 창(UAC)이 여러 번 뜰 수 있어요.
              </li>
            )}
            <li className="flex gap-2">
              <Check className="size-4 shrink-0 text-primary" />이 단계에서 RobinX는 파일이나 레지스트리를 지우지
              않아요. 남은 흔적은 끝난 뒤 따로 찾아요.
            </li>
            <li className="flex gap-2">
              <Info className="size-4 shrink-0" />
              제거한 프로그램은 RobinX로 되살릴 수 없어요. 다시 쓰려면 새로 설치해야 해요.
            </li>
            {forced.length > 0 && (
              <li className="flex gap-2">
                <TriangleAlert className="size-4 shrink-0 text-warning" />
                언인스톨러가 없는 {forced.length}개({forced.map((a) => a.displayName).join(", ")})는 강제 제거라 여기서
                빼요. 하나씩 제거해 주세요.
              </li>
            )}
          </ul>
        )}

        <ul className="min-h-0 flex-1 divide-y overflow-y-auto">
          {runnable.map((app) => {
            const status = statuses[app.id] ?? { kind: "waiting" }
            return (
              <li
                key={app.id}
                className={cn("flex items-center gap-3 px-5 py-2.5", current === app.id && "bg-primary/6")}
              >
                <AppIcon app={app} />
                <div className="min-w-0 flex-1">
                  <p className="truncate text-sm font-medium">{app.displayName}</p>
                  <p
                    className={cn(
                      // 실패 이유는 줄여 자르지 않고 다 보여 준다
                      status.kind === "done" ? "text-xs break-words" : "truncate text-xs",
                      status.kind === "done" ? outcomeLabel(status.outcome).tone : "text-muted-foreground",
                    )}
                  >
                    {status.kind === "waiting" && (phase === "confirm" ? (app.publisher ?? "") : "차례를 기다리는 중")}
                    {status.kind === "running" && status.stage}
                    {status.kind === "done" && outcomeLabel(status.outcome).text}
                    {status.kind === "skipped" && "멈춰서 실행하지 않았어요"}
                  </p>
                </div>
                <span className="shrink-0">
                  {status.kind === "running" && <Loader2 className="size-4 animate-spin text-primary" />}
                  {status.kind === "waiting" && phase !== "confirm" && (
                    <Clock className="size-4 text-muted-foreground" />
                  )}
                  {status.kind === "done" &&
                    (isRemoved(status) ? (
                      // 다 끝난 뒤에만: 도중에 누르면 창이 닫히고 남은 제거가 보이지 않게 이어진다
                      phase === "done" ? (
                        <Button
                          size="sm"
                          variant="ghost"
                          className="h-7"
                          aria-label={`${app.displayName} 흔적 찾기`}
                          onClick={() => onFindLeftovers(app)}
                        >
                          <Sparkles />
                          흔적 찾기
                        </Button>
                      ) : (
                        <Check className="size-4 text-primary" />
                      )
                    ) : (
                      <CircleAlert className="size-4 text-warning" />
                    ))}
                </span>
              </li>
            )
          })}
        </ul>

        {/* 언인스톨러가 없어 뺀 프로그램: 결과에서도 사라지지 않게 따로 알린다 */}
        {phase !== "confirm" && forced.length > 0 && (
          <p className="flex gap-2 border-t bg-muted/20 px-5 py-3 text-xs text-muted-foreground">
            <TriangleAlert className="size-4 shrink-0 text-warning" />
            <span>
              언인스톨러가 없어 건너뛴 {forced.length}개: {forced.map((a) => a.displayName).join(", ")}. 목록에서 하나씩
              강제 제거해 주세요.
            </span>
          </p>
        )}

        <div className="flex flex-wrap items-center gap-2 border-t px-5 py-3">
          {phase === "confirm" && (
            <>
              <Button variant="outline" className="ml-auto" onClick={close} data-autofocus>
                취소
              </Button>
              <Button variant="destructive" onClick={run} disabled={runnable.length === 0}>
                <Trash2 />
                {runnable.length}개 제거 시작
              </Button>
            </>
          )}
          {phase === "running" && (
            <>
              {stuck && (
                <span className="flex items-center gap-2 text-xs text-muted-foreground">
                  언인스톨러 창이 이미 닫혔나요?
                  <Button
                    variant="ghost"
                    size="sm"
                    disabled={skipping}
                    onClick={() => {
                      setSkipping(true)
                      api.stopWaitingUninstall()
                    }}
                  >
                    기다리지 않고 확인
                  </Button>
                </span>
              )}
              <Button
                variant="outline"
                className="ml-auto"
                disabled={stopping}
                onClick={() => {
                  stopAfterCurrent.current = true
                  setStopping(true)
                }}
              >
                <CircleStop />
                {stopping ? "이번 것만 끝내고 멈춰요" : "이번 것 끝나면 멈추기"}
              </Button>
            </>
          )}
          {phase === "done" && (
            <Button className="ml-auto" onClick={close}>
              완료
            </Button>
          )}
        </div>
      </DialogContent>
    </Dialog>
  )
}
