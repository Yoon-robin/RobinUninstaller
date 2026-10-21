import { useState, type ReactNode } from "react"
import { Check, Copy, FolderOpen, Sparkles, Trash2 } from "lucide-react"
import { AnimatePresence, motion } from "motion/react"
import { toast } from "sonner"
import { useLastDefined } from "@/hooks/use-last-defined"

import { api, canUninstall, type InstalledApplication } from "@/lib/api"
import { formatInstallDate, formatSize } from "@/lib/format"
import { FadeUp, Stagger } from "@/components/motion"
import { Button } from "@/components/ui/button"
import { Sheet, SheetContent, SheetDescription, SheetHeader, SheetTitle } from "@/components/ui/sheet"
import { AppIcon } from "./app-icon"
import { UninstallerBadge } from "./uninstaller-badge"

const scopeLabels = {
  machine64: "모든 사용자 · 64비트",
  machine32: "모든 사용자 · 32비트",
  user: "현재 사용자",
} as const

export function AppDetailsSheet({
  app: appProp,
  onOpenChange,
  onUninstall,
  onFindLeftovers,
}: {
  app: InstalledApplication | null
  onOpenChange: (open: boolean) => void
  onUninstall: (app: InstalledApplication) => void
  onFindLeftovers: (app: InstalledApplication) => void
}) {
  // 닫히는 동안에도 내용이 남아 있게 마지막 대상을 그린다
  const app = useLastDefined(appProp)
  const openFolder = async () => {
    if (!app) return
    try {
      await api.openInstallLocation(app.id)
    } catch (e) {
      toast.error(String(e))
    }
  }

  return (
    <Sheet open={appProp != null} onOpenChange={onOpenChange}>
      <SheetContent className="w-[460px] gap-0 sm:max-w-[460px]" onOpenAutoFocus={(e) => e.preventDefault()}>
        {app && (
          <>
            <SheetHeader className="items-center gap-3 border-b px-6 pt-8 pb-6 text-center">
              <div className="rounded-2xl border bg-muted/40 p-3">
                <AppIcon app={app} className="size-12 text-lg" />
              </div>
              <div className="min-w-0">
                <SheetTitle className="text-lg leading-snug">{app.displayName}</SheetTitle>
                <SheetDescription className="mt-1">
                  {[app.publisher, app.version].filter(Boolean).join(" · ") || "게시자 정보 없음"}
                </SheetDescription>
              </div>
              <UninstallerBadge status={app.uninstaller} />
              <div className="mt-2 grid w-full grid-cols-2 gap-2">
                <Button
                  variant="destructive"
                  className="rounded-lg"
                  title={canUninstall(app) ? undefined : "언인스톨러가 없어서 등록 정보와 파일을 직접 정리해요"}
                  onClick={() => onUninstall(app)}
                >
                  <Trash2 />
                  {canUninstall(app) ? "제거" : "강제 제거"}
                </Button>
                <Button variant="outline" className="rounded-lg" onClick={openFolder} disabled={!app.installLocation}>
                  <FolderOpen />
                  폴더 열기
                </Button>
                <Button
                  variant="ghost"
                  className="col-span-2 rounded-lg text-muted-foreground"
                  onClick={() => onFindLeftovers(app)}
                >
                  <Sparkles />
                  흔적 미리 보기
                </Button>
              </div>
            </SheetHeader>

            {/* 패널이 열릴 때 묶음이 차례로 떠오른다. key로 다른 앱을 열 때마다 다시 재생. */}
            <Stagger key={app.id} className="flex flex-col gap-5 overflow-y-auto px-6 py-5">
              <Section title="개요">
                <Field label="크기">
                  {formatSize(app.estimatedSizeKb)}
                  {app.sizeMeasured && <span className="text-muted-foreground"> (설치 폴더를 직접 잰 크기)</span>}
                </Field>
                <Field label="설치일">{formatInstallDate(app.installDate)}</Field>
                <Field label="설치 형식">{app.technology.toUpperCase()}</Field>
                <Field label="설치 범위">{scopeLabels[app.scope]}</Field>
              </Section>

              <Section title="위치와 명령">
                <CodeField label="설치 폴더" value={app.installLocation} />
                <CodeField label="제거 명령" value={app.uninstallCommand} />
                <CodeField label="자동 제거 명령" value={app.quietUninstallCommand} />
              </Section>

              <Section title="식별 정보">
                <CodeField label="MSI 제품 코드" value={app.productCode} />
                <CodeField label="레지스트리 ID" value={app.id} />
              </Section>
            </Stagger>
          </>
        )}
      </SheetContent>
    </Sheet>
  )
}

function Section({ title, children }: { title: string; children: ReactNode }) {
  return (
    <FadeUp>
      <h3 className="mb-2 text-xs font-medium tracking-wide text-muted-foreground">{title}</h3>
      <div className="divide-y rounded-xl border bg-muted/20">{children}</div>
    </FadeUp>
  )
}

function Field({ label, children }: { label: string; children: ReactNode }) {
  return (
    <div className="flex items-center justify-between gap-4 px-4 py-2.5 text-sm">
      <span className="text-muted-foreground">{label}</span>
      <span className="font-medium tabular-nums">{children}</span>
    </div>
  )
}

function CodeField({ label, value }: { label: string; value: string | null }) {
  const [copied, setCopied] = useState(false)
  const copy = async () => {
    if (!value) return
    await navigator.clipboard.writeText(value)
    setCopied(true)
    setTimeout(() => setCopied(false), 1200)
  }

  return (
    <div className="group/code px-4 py-2.5 text-sm">
      <div className="flex items-center justify-between">
        <span className="text-muted-foreground">{label}</span>
        {value && (
          <button
            onClick={copy}
            className={`flex items-center gap-1 rounded-md px-1.5 py-0.5 text-xs text-muted-foreground transition-opacity duration-150 hover:bg-muted hover:text-foreground group-hover/code:opacity-100 ${copied ? "opacity-100" : "opacity-0"}`}
          >
            <AnimatePresence mode="popLayout" initial={false}>
              <motion.span
                key={copied ? "done" : "idle"}
                className="flex items-center gap-1"
                initial={{ opacity: 0, scale: 0.8 }}
                animate={{ opacity: 1, scale: 1 }}
                exit={{ opacity: 0, scale: 0.8 }}
                transition={{ type: "spring", duration: 0.25, bounce: 0 }}
              >
                {copied ? <Check className="size-3 text-primary" /> : <Copy className="size-3" />}
                {copied ? "복사됨" : "복사"}
              </motion.span>
            </AnimatePresence>
          </button>
        )}
      </div>
      {value ? (
        <p data-selectable className="mt-1 font-mono text-xs leading-5 break-all text-foreground/90">
          {value}
        </p>
      ) : (
        <p className="mt-1 text-xs text-muted-foreground/70">기록 없음</p>
      )}
    </div>
  )
}
