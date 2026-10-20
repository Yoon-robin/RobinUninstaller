import { useEffect, useState, type ReactNode } from "react"
import { invoke } from "@tauri-apps/api/core"
import { getCurrentWindow } from "@tauri-apps/api/window"
import { Check, Loader2, RotateCcw, TriangleAlert } from "lucide-react"
import { AnimatePresence, MotionConfig, motion, useReducedMotion } from "motion/react"

import { cn } from "@/lib/utils"
import { Logo } from "@/components/logo"
import { EASE_OUT_QUINT } from "@/components/motion"
import { ThemeProvider } from "@/components/theme"
import { Button } from "@/components/ui/button"
import { Checkbox } from "@/components/ui/checkbox"

/**
 * 자체 설치·제거 화면 (docs/INSTALLER.md). 배포하는 setup.exe가 곧 robinx.exe라서, 앱과 같은 창·디자인으로 그린다.
 * 실제 작업은 src-tauri/src/setup.rs → crates/setup 이 한다. 이 화면은 고르고, 기다리고, 결과를 보여 줄 뿐이다.
 */

export type SetupMode = "setup" | "uninstall"

interface Existing {
  dir: string
  version: string | null
  legacy: boolean
}

interface SetupInfo {
  mode: SetupMode
  version: string
  existing: Existing | null
  installDir: string
  running: boolean
  dataExists: boolean
}

interface InstallReport {
  dir: string
  exe: string
  migrated: string[]
  warnings: string[]
}

interface UninstallReport {
  removed: string[]
  warnings: string[]
  keptData: string | null
}

const setupApi = {
  info: () => invoke<SetupInfo>("setup_info"),
  install: (desktopShortcut: boolean) => invoke<InstallReport>("run_install", { desktopShortcut }),
  uninstall: (deleteData: boolean) => invoke<UninstallReport>("run_uninstall", { deleteData }),
  launch: () => invoke<void>("launch_installed"),
  finish: () => invoke<void>("finish_setup"),
}

type Step<R> =
  { kind: "ready" } | { kind: "working" } | { kind: "done"; report: R } | { kind: "error"; message: string }

export function SetupApp({ mode }: { mode: SetupMode }) {
  const [info, setInfo] = useState<SetupInfo | null>(null)
  const [loadError, setLoadError] = useState<string | null>(null)

  useEffect(() => {
    setupApi
      .info()
      .then(setInfo)
      .catch((e) => setLoadError(String(e)))
  }, [])

  return (
    <ThemeProvider>
      <MotionConfig reducedMotion="user">
        <div className="relative flex h-screen flex-col overflow-hidden bg-background text-foreground select-none">
          <TitleBar title={mode === "setup" ? "RobinX 설치" : "RobinX 제거"} />
          <main className="relative flex min-h-0 flex-1 flex-col px-10 pt-2 pb-8">
            {loadError ? (
              <Problem message={loadError} />
            ) : !info ? (
              <div className="flex flex-1 items-center justify-center">
                <Loader2 className="size-6 animate-spin text-muted-foreground" />
              </div>
            ) : mode === "setup" ? (
              <InstallFlow info={info} />
            ) : (
              <UninstallFlow info={info} />
            )}
          </main>
        </div>
      </MotionConfig>
    </ThemeProvider>
  )
}

/** 제목 표시줄: 끌어서 옮기고, 최소화·닫기만 있다. 닫기는 제거 후 임시 파일 정리까지 맡긴다. */
function TitleBar({ title }: { title: string }) {
  const win = getCurrentWindow()
  return (
    <div data-tauri-drag-region className="relative flex h-11 shrink-0 items-center justify-between pl-4">
      <span data-tauri-drag-region className="text-xs font-medium text-muted-foreground">
        {title}
      </span>
      <div className="flex items-center gap-1 pr-2.5">
        <CaptionButton label="최소화" onClick={() => win.minimize()}>
          <path d="M0 5h10" />
        </CaptionButton>
        <CaptionButton label="닫기" danger onClick={() => setupApi.finish()}>
          <path d="M0.5 0.5l9 9M9.5 0.5l-9 9" />
        </CaptionButton>
      </div>
    </div>
  )
}

function CaptionButton({
  label,
  onClick,
  danger,
  children,
}: {
  label: string
  onClick: () => void
  danger?: boolean
  children: ReactNode
}) {
  return (
    <button
      type="button"
      aria-label={label}
      title={label}
      onClick={onClick}
      className={cn(
        "flex size-7 items-center justify-center rounded-full text-muted-foreground transition-[color,background-color,transform] duration-150 active:scale-90",
        danger ? "hover:bg-[#ff5f57] hover:text-white" : "hover:bg-foreground/8 hover:text-foreground",
      )}
    >
      <svg viewBox="0 0 10 10" className="size-2.5" fill="none" stroke="currentColor" strokeWidth="1">
        {children}
      </svg>
    </button>
  )
}

/** 화면 바뀔 때 부드럽게 바꿔 끼운다 */
function Screen({ id, children }: { id: string; children: ReactNode }) {
  const reduce = useReducedMotion()
  return (
    <AnimatePresence mode="wait" initial={false}>
      <motion.section
        key={id}
        initial={reduce ? false : { opacity: 0, y: 12 }}
        animate={{ opacity: 1, y: 0 }}
        exit={reduce ? undefined : { opacity: 0, y: -8 }}
        transition={{ duration: 0.35, ease: EASE_OUT_QUINT }}
        className="flex min-h-0 flex-1 flex-col"
      >
        {children}
      </motion.section>
    </AnimatePresence>
  )
}

function Hero({ title, children }: { title: string; children?: ReactNode }) {
  return (
    <div className="flex items-center gap-5">
      <Logo className="size-16 shrink-0 drop-shadow-xl" />
      <div className="min-w-0">
        <h1 className="text-2xl font-bold tracking-tight">{title}</h1>
        {children && <div className="mt-1 text-sm text-muted-foreground">{children}</div>}
      </div>
    </div>
  )
}

function Note({ tone = "muted", children }: { tone?: "muted" | "warn" | "danger"; children: ReactNode }) {
  return (
    <div
      className={cn(
        "flex items-start gap-2.5 rounded-xl px-3.5 py-3 text-sm",
        tone === "muted" && "bg-muted/50 text-muted-foreground",
        tone === "warn" && "bg-chart-3/12 text-chart-3",
        tone === "danger" && "bg-destructive/12 text-destructive",
      )}
    >
      {tone !== "muted" && <TriangleAlert className="mt-0.5 size-4 shrink-0" />}
      <div>{children}</div>
    </div>
  )
}

function Working({ label }: { label: string }) {
  const reduce = useReducedMotion()
  return (
    <div className="flex flex-1 flex-col items-center justify-center gap-6" role="status" aria-live="polite">
      <motion.div
        animate={reduce ? undefined : { scale: [1, 1.06, 1] }}
        transition={{ duration: 1.6, repeat: Infinity, ease: "easeInOut" }}
      >
        <Logo className="size-20 drop-shadow-xl" />
      </motion.div>
      <div className="h-1 w-56 overflow-hidden rounded-full bg-muted">
        <div className="scan-sweep h-full w-1/3 rounded-full bg-primary" />
      </div>
      <p className="text-sm text-muted-foreground">{label}</p>
    </div>
  )
}

function Problem({ message, onRetry }: { message: string; onRetry?: () => void }) {
  return (
    <div className="flex flex-1 flex-col justify-between">
      <div className="mt-4 flex flex-col gap-4">
        <Hero title="문제가 생겼어요" />
        <Note tone="danger">
          <span data-selectable className="break-all select-text">
            {message}
          </span>
        </Note>
        <p className="text-sm text-muted-foreground">
          아무것도 바뀌지 않았거나 일부만 바뀌었을 수 있어요. 다시 시도해도 같으면 GitHub에 알려 주세요.
        </p>
      </div>
      <div className="flex justify-end gap-2">
        <Button variant="outline" onClick={() => setupApi.finish()}>
          닫기
        </Button>
        {onRetry && (
          <Button onClick={onRetry}>
            <RotateCcw />
            다시 시도
          </Button>
        )}
      </div>
    </div>
  )
}

function InstallFlow({ info }: { info: SetupInfo }) {
  const [step, setStep] = useState<Step<InstallReport>>({ kind: "ready" })
  const [desktop, setDesktop] = useState(!info.existing)
  const [openAfter, setOpenAfter] = useState(true)
  const old = info.existing

  const run = async () => {
    setStep({ kind: "working" })
    try {
      setStep({ kind: "done", report: await setupApi.install(desktop) })
    } catch (e) {
      setStep({ kind: "error", message: String(e) })
    }
  }

  const finish = async () => {
    if (openAfter) await setupApi.launch().catch(() => {})
    await setupApi.finish()
  }

  if (step.kind === "working")
    return (
      <Screen id="working">
        <Working label="설치하는 중이에요" />
      </Screen>
    )
  if (step.kind === "error")
    return (
      <Screen id="error">
        <Problem message={step.message} onRetry={run} />
      </Screen>
    )
  if (step.kind === "done")
    return (
      <Screen id="done">
        <div className="flex flex-1 flex-col justify-between">
          <div className="mt-4 flex flex-col gap-5">
            <div className="flex items-center gap-5">
              <span className="flex size-16 items-center justify-center rounded-2xl bg-primary/15 text-primary">
                <Check className="size-8" strokeWidth={2.4} />
              </span>
              <div>
                <h1 className="text-2xl font-bold tracking-tight">설치했어요</h1>
                <p className="mt-1 text-sm text-muted-foreground">
                  시작 메뉴{desktop ? "와 바탕 화면" : ""}에서 RobinX를 찾을 수 있어요.
                </p>
              </div>
            </div>
            {step.report.warnings.length > 0 && (
              <Note tone="warn">
                설치는 끝났지만 확인할 것이 있어요.
                <ul className="mt-1 list-disc pl-4 text-xs">
                  {step.report.warnings.map((w) => (
                    <li key={w}>{w}</li>
                  ))}
                </ul>
              </Note>
            )}
            <label className="flex cursor-pointer items-center gap-2.5 text-sm">
              <Checkbox checked={openAfter} onCheckedChange={(v) => setOpenAfter(v === true)} />
              RobinX 바로 열기
            </label>
          </div>
          <div className="flex justify-end">
            <Button size="lg" onClick={finish}>
              완료
            </Button>
          </div>
        </div>
      </Screen>
    )

  return (
    <Screen id="ready">
      <div className="flex flex-1 flex-col justify-between">
        <div className="mt-4 flex flex-col gap-5">
          <Hero title="RobinX 설치">
            프로그램을 지워도 남는 파일과 설정 찌꺼기를 찾아 주는 무료 Windows 앱이에요.
            <span className="mt-1 block text-xs">버전 {info.version}</span>
          </Hero>
          {old && (
            <Note>
              {old.version === info.version
                ? `이미 같은 버전(${info.version})이 설치돼 있어요. 다시 설치해요.`
                : old.version
                  ? `설치된 버전 ${old.version} → 새 버전 ${info.version}.`
                  : "설치된 RobinX를 새 버전으로 바꿔요."}{" "}
              백업과 설정은 그대로 남아요.
            </Note>
          )}
          {info.running && <Note tone="warn">RobinX가 열려 있어요. 설치를 누르면 닫고 바꿔요.</Note>}
          <label className="flex cursor-pointer items-center gap-2.5 text-sm">
            <Checkbox checked={desktop} onCheckedChange={(v) => setDesktop(v === true)} />
            바탕 화면에 바로 가기 만들기
          </label>
          <p className="text-xs text-muted-foreground">
            내 계정에만 설치해서 관리자 권한이 필요 없어요.
            <span data-selectable className="mt-0.5 block truncate select-text" title={info.installDir}>
              {info.installDir}
            </span>
          </p>
        </div>
        <div className="flex justify-end gap-2">
          <Button variant="outline" size="lg" onClick={() => setupApi.finish()}>
            취소
          </Button>
          <Button size="lg" onClick={run}>
            {old ? (old.version === info.version ? "다시 설치" : "업데이트") : "설치"}
          </Button>
        </div>
      </div>
    </Screen>
  )
}

function UninstallFlow({ info }: { info: SetupInfo }) {
  const [step, setStep] = useState<Step<UninstallReport>>({ kind: "ready" })
  const [deleteData, setDeleteData] = useState(false)

  const run = async () => {
    setStep({ kind: "working" })
    try {
      setStep({ kind: "done", report: await setupApi.uninstall(deleteData) })
    } catch (e) {
      setStep({ kind: "error", message: String(e) })
    }
  }

  if (!info.existing && step.kind === "ready")
    return (
      <Screen id="none">
        <div className="flex flex-1 flex-col justify-between">
          <div className="mt-4">
            <Hero title="설치된 RobinX가 없어요">이미 제거됐거나 다른 방법으로 설치했어요.</Hero>
          </div>
          <div className="flex justify-end">
            <Button size="lg" onClick={() => setupApi.finish()}>
              닫기
            </Button>
          </div>
        </div>
      </Screen>
    )
  if (step.kind === "working")
    return (
      <Screen id="working">
        <Working label="제거하는 중이에요" />
      </Screen>
    )
  if (step.kind === "error")
    return (
      <Screen id="error">
        <Problem message={step.message} onRetry={run} />
      </Screen>
    )
  if (step.kind === "done")
    return (
      <Screen id="done">
        <div className="flex flex-1 flex-col justify-between">
          <div className="mt-4 flex flex-col gap-5">
            <div className="flex items-center gap-5">
              <span className="flex size-16 items-center justify-center rounded-2xl bg-primary/15 text-primary">
                <Check className="size-8" strokeWidth={2.4} />
              </span>
              <div>
                <h1 className="text-2xl font-bold tracking-tight">제거했어요</h1>
                <p className="mt-1 text-sm text-muted-foreground">RobinX를 써 주셔서 고마워요.</p>
              </div>
            </div>
            {step.report.keptData && <Note>백업과 설정은 남겨 뒀어요. 다시 설치하면 그대로 이어서 쓸 수 있어요.</Note>}
            {step.report.warnings.length > 0 && (
              <Note tone="warn">
                일부는 지우지 못했어요.
                <ul className="mt-1 list-disc pl-4 text-xs">
                  {step.report.warnings.map((w) => (
                    <li key={w}>{w}</li>
                  ))}
                </ul>
              </Note>
            )}
          </div>
          <div className="flex justify-end">
            <Button size="lg" onClick={() => setupApi.finish()}>
              닫기
            </Button>
          </div>
        </div>
      </Screen>
    )

  return (
    <Screen id="ready">
      <div className="flex flex-1 flex-col justify-between">
        <div className="mt-4 flex flex-col gap-5">
          <Hero title="RobinX 제거">
            RobinX{info.existing?.version ? ` ${info.existing.version}` : ""} 프로그램과 바로 가기, Windows 등록 정보를
            지워요.
          </Hero>
          {info.running && <Note tone="warn">RobinX가 열려 있어요. 제거를 누르면 닫고 지워요.</Note>}
          {info.dataExists && (
            <div className="flex flex-col gap-2">
              <label className="flex cursor-pointer items-center gap-2.5 text-sm">
                <Checkbox checked={deleteData} onCheckedChange={(v) => setDeleteData(v === true)} />
                백업과 설정도 지우기
              </label>
              {deleteData ? (
                <Note tone="danger">정리한 흔적의 백업까지 함께 사라져서, 복구 기록에서 더는 되돌릴 수 없어요.</Note>
              ) : (
                <p className="pl-7 text-xs text-muted-foreground">
                  고르지 않으면 남겨 둬요. 다시 설치하면 그대로 이어서 쓸 수 있어요.
                </p>
              )}
            </div>
          )}
        </div>
        <div className="flex justify-end gap-2">
          <Button variant="outline" size="lg" onClick={() => setupApi.finish()}>
            취소
          </Button>
          <Button variant="destructive" size="lg" onClick={run}>
            제거
          </Button>
        </div>
      </div>
    </Screen>
  )
}
