import { createContext, useCallback, useContext, useEffect, useRef, useState, type ReactNode } from "react"
import { getVersion } from "@tauri-apps/api/app"
import { relaunch } from "@tauri-apps/plugin-process"
import { check, type Update } from "@tauri-apps/plugin-updater"
import { Download, Loader2, RotateCcw } from "lucide-react"
import { toast } from "sonner"

import { formatBytes } from "@/lib/format"
import { Button } from "@/components/ui/button"
import { Dialog, DialogContent, DialogDescription, DialogTitle } from "@/components/ui/dialog"

/**
 * 자동 업데이트 (Tauri updater).
 * - GitHub Releases의 latest.json을 읽어 새 버전이 있는지 본다. 업데이트 파일은 서명을 확인한 뒤에만 설치한다
 *   (공개 키는 tauri.conf.json, 비밀 키는 배포하는 사람의 PC에만 있다).
 * - 앱을 켜고 잠시 뒤 조용히 한 번 확인하고, 새 버전이 있으면 알림만 띄운다. 설치는 사용자가 누를 때만 한다.
 */

const AUTO_KEY = "robin.updates.autoCheck"
const LAST_KEY = "robin.updates.lastChecked"
/** 시작 화면과 첫 목록 읽기가 끝난 뒤에 확인한다 */
const STARTUP_DELAY_MS = 6000

type CheckResult = { kind: "latest" } | { kind: "available"; update: Update } | { kind: "error"; message: string }

const UpdatesContext = createContext<{
  version: string | null
  autoCheck: boolean
  setAutoCheck: (on: boolean) => void
  lastChecked: number | null
  checking: boolean
  /** 직접 누른 확인이면 결과를 알림으로 보여 준다 */
  checkNow: (manual: boolean) => Promise<void>
} | null>(null)

export const useUpdates = () => {
  const ctx = useContext(UpdatesContext)
  if (!ctx) throw new Error("UpdatesProvider 안에서 써야 해요")
  return ctx
}

function readBool(key: string, fallback: boolean) {
  try {
    const v = localStorage.getItem(key)
    return v == null ? fallback : v === "1"
  } catch {
    return fallback
  }
}

function readNumber(key: string) {
  try {
    const v = Number(localStorage.getItem(key))
    return Number.isFinite(v) && v > 0 ? v : null
  } catch {
    return null
  }
}

function friendlyError(e: unknown) {
  const text = String(e)
  // 아직 릴리스가 없거나 latest.json이 없으면 GitHub가 404를 준다
  if (/404|not found|Could not fetch a valid release JSON/i.test(text)) return "아직 배포된 업데이트 정보가 없어요"
  if (/network|dns|connect|timed out|offline/i.test(text)) return "인터넷에 연결할 수 없어요"
  if (/signature/i.test(text)) return "업데이트 파일의 서명이 맞지 않아 설치하지 않았어요"
  return text
}

async function runCheck(): Promise<CheckResult> {
  try {
    const update = await check()
    return update ? { kind: "available", update } : { kind: "latest" }
  } catch (e) {
    return { kind: "error", message: friendlyError(e) }
  }
}

export function UpdatesProvider({ children }: { children: ReactNode }) {
  const [version, setVersion] = useState<string | null>(null)
  const [autoCheck, setAutoCheckState] = useState(() => readBool(AUTO_KEY, true))
  const [lastChecked, setLastChecked] = useState<number | null>(() => readNumber(LAST_KEY))
  const [checking, setChecking] = useState(false)
  const [pending, setPending] = useState<Update | null>(null)
  const [open, setOpen] = useState(false)
  const startedRef = useRef(false)

  useEffect(() => {
    getVersion()
      .then(setVersion)
      .catch(() => setVersion(null))
  }, [])

  const setAutoCheck = (on: boolean) => {
    setAutoCheckState(on)
    try {
      localStorage.setItem(AUTO_KEY, on ? "1" : "0")
    } catch {
      // 저장이 막혀 있으면 이번 실행 동안만
    }
  }

  const checkNow = useCallback(async (manual: boolean) => {
    setChecking(true)
    const result = await runCheck()
    setChecking(false)
    const now = Date.now()
    setLastChecked(now)
    try {
      localStorage.setItem(LAST_KEY, String(now))
    } catch {
      // 무시
    }
    if (result.kind === "available") {
      setPending(result.update)
      if (manual) {
        setOpen(true)
      } else {
        toast(`새 버전 ${result.update.version}이 나왔어요`, {
          description: "지금 설치하거나 나중에 설정에서 설치할 수 있어요.",
          duration: 15000,
          action: { label: "업데이트", onClick: () => setOpen(true) },
        })
      }
    } else if (manual) {
      if (result.kind === "latest") toast.success("최신 버전을 쓰고 있어요")
      else toast.error("업데이트를 확인하지 못했어요", { description: result.message })
    }
  }, [])

  // 앱을 켜고 잠시 뒤 한 번만, 조용히 확인한다
  useEffect(() => {
    if (!autoCheck || startedRef.current) return
    startedRef.current = true
    const t = setTimeout(() => void checkNow(false), STARTUP_DELAY_MS)
    return () => clearTimeout(t)
  }, [autoCheck, checkNow])

  return (
    <UpdatesContext.Provider value={{ version, autoCheck, setAutoCheck, lastChecked, checking, checkNow }}>
      {children}
      <UpdateDialog update={pending} open={open && pending != null} onClose={() => setOpen(false)} />
    </UpdatesContext.Provider>
  )
}

type Stage = { step: "ready" } | { step: "downloading"; done: number; total: number | null } | { step: "installing" }

function UpdateDialog({ update, open, onClose }: { update: Update | null; open: boolean; onClose: () => void }) {
  const [stage, setStage] = useState<Stage>({ step: "ready" })
  const [error, setError] = useState<string | null>(null)
  const busy = stage.step !== "ready"

  useEffect(() => {
    if (open) {
      setStage({ step: "ready" })
      setError(null)
    }
  }, [open])

  const install = async () => {
    if (!update) return
    setError(null)
    let done = 0
    let total: number | null = null
    setStage({ step: "downloading", done: 0, total: null })
    try {
      await update.downloadAndInstall((event) => {
        if (event.event === "Started") {
          total = event.data.contentLength ?? null
          setStage({ step: "downloading", done: 0, total })
        } else if (event.event === "Progress") {
          done += event.data.chunkLength
          setStage({ step: "downloading", done, total })
        } else if (event.event === "Finished") {
          setStage({ step: "installing" })
        }
      })
      // Windows는 설치 프로그램이 앱을 닫고 설치한다. 그렇지 않은 경우를 위해 다시 시작한다.
      await relaunch()
    } catch (e) {
      setError(friendlyError(e))
      setStage({ step: "ready" })
    }
  }

  const percent =
    stage.step === "downloading" && stage.total ? Math.min(100, Math.round((stage.done / stage.total) * 100)) : null

  return (
    <Dialog open={open} onOpenChange={(v) => !v && !busy && onClose()}>
      <DialogContent className="sm:max-w-md" showCloseButton={!busy}>
        <DialogTitle>새 버전 {update?.version}</DialogTitle>
        <DialogDescription className="leading-relaxed">
          지금 쓰는 버전은 {update?.currentVersion}이에요. 업데이트 파일은 RobinX의 서명을 확인한 뒤에만 설치하고, 복구
          기록·설정·백업은 그대로 남아요. 설치하는 동안 앱이 잠깐 닫혔다가 다시 열려요.
        </DialogDescription>
        {update?.body && (
          <div className="max-h-48 overflow-y-auto rounded-xl border bg-muted/30 px-4 py-3 text-sm whitespace-pre-wrap">
            {update.body}
          </div>
        )}
        {stage.step === "downloading" && (
          <div className="flex flex-col gap-2" role="status" aria-live="polite">
            <div className="h-1.5 overflow-hidden rounded-full bg-muted">
              {percent == null ? (
                <div className="scan-sweep h-full w-1/3 rounded-full bg-primary" />
              ) : (
                <div className="h-full rounded-full bg-primary transition-[width]" style={{ width: `${percent}%` }} />
              )}
            </div>
            <p className="text-xs text-muted-foreground tabular-nums">
              받는 중 {formatBytes(stage.done)}
              {stage.total ? ` / ${formatBytes(stage.total)} (${percent}%)` : ""}
            </p>
          </div>
        )}
        {stage.step === "installing" && (
          <p className="flex items-center gap-2 text-sm text-muted-foreground" role="status">
            <Loader2 className="size-4 animate-spin" />
            설치하는 중이에요. 곧 앱이 다시 열려요.
          </p>
        )}
        {error && <p className="rounded-lg bg-destructive/10 px-3 py-2 text-sm text-destructive">{error}</p>}
        <div className="mt-1 flex justify-end gap-2">
          <Button variant="outline" onClick={onClose} disabled={busy}>
            나중에
          </Button>
          <Button onClick={install} disabled={busy || !update}>
            {busy ? <Loader2 className="animate-spin" /> : error ? <RotateCcw /> : <Download />}
            {error ? "다시 시도" : "지금 업데이트"}
          </Button>
        </div>
      </DialogContent>
    </Dialog>
  )
}
