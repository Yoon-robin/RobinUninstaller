import { useState, type ReactNode } from "react"
import { Check, Copy, FolderOpen, Trash2 } from "lucide-react"
import { toast } from "sonner"

import { api, type InstalledApplication } from "@/lib/api"
import { formatInstallDate, formatSize } from "@/lib/format"
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
  app,
  onOpenChange,
}: {
  app: InstalledApplication | null
  onOpenChange: (open: boolean) => void
}) {
  const openFolder = async () => {
    if (!app) return
    try {
      await api.openInstallLocation(app.id)
    } catch (e) {
      toast.error(String(e))
    }
  }

  return (
    <Sheet open={app != null} onOpenChange={onOpenChange}>
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
                <Button disabled className="rounded-lg">
                  <Trash2 />
                  제거 (곧 지원)
                </Button>
                <Button variant="outline" className="rounded-lg" onClick={openFolder} disabled={!app.installLocation}>
                  <FolderOpen />
                  폴더 열기
                </Button>
              </div>
            </SheetHeader>

            <div className="flex flex-col gap-5 overflow-y-auto px-6 py-5">
              <Section title="개요">
                <Field label="크기">{formatSize(app.estimatedSizeKb)}</Field>
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
            </div>
          </>
        )}
      </SheetContent>
    </Sheet>
  )
}

function Section({ title, children }: { title: string; children: ReactNode }) {
  return (
    <section>
      <h3 className="mb-2 text-xs font-medium tracking-wide text-muted-foreground">{title}</h3>
      <div className="divide-y rounded-xl border bg-muted/20">{children}</div>
    </section>
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
            className="flex items-center gap-1 rounded-md px-1.5 py-0.5 text-xs text-muted-foreground opacity-0 transition hover:bg-muted hover:text-foreground group-hover/code:opacity-100"
          >
            {copied ? <Check className="size-3" /> : <Copy className="size-3" />}
            {copied ? "복사됨" : "복사"}
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
