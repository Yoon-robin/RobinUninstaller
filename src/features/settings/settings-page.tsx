import { useState, type ReactNode } from "react"
import { useQuery, useQueryClient } from "@tanstack/react-query"
import {
  ArchiveRestore,
  Download,
  EyeOff,
  KeyRound,
  Loader2,
  Monitor,
  Moon,
  Palette,
  Sun,
  Trash2,
  TriangleAlert,
  X,
} from "lucide-react"
import { toast } from "sonner"

import { settingsApi, type ScanProgress } from "@/lib/api"
import { ScanProgressView } from "@/components/scan-progress"
import { formatBytes, formatDateTime } from "@/lib/format"
import { cn } from "@/lib/utils"
import { FadeUp, Stagger } from "@/components/motion"
import { useTheme } from "@/components/theme"
import { rovingKeyDown } from "@/features/programs/segmented"
import { Button } from "@/components/ui/button"
import { Dialog, DialogContent, DialogDescription, DialogTitle } from "@/components/ui/dialog"
import { Skeleton } from "@/components/ui/skeleton"
import { Switch } from "@/components/ui/switch"
import { useUpdates } from "@/features/updates/updates"
import { useBusy } from "@/lib/busy"

function Section({
  icon: Icon,
  title,
  description,
  children,
}: {
  icon: typeof Palette
  title: string
  description: string
  children: ReactNode
}) {
  return (
    <section className="overflow-hidden rounded-2xl border bg-card shadow-sm">
      <div className="flex items-start gap-3 border-b px-5 py-4">
        <div className="flex size-9 shrink-0 items-center justify-center rounded-xl bg-primary/12 text-primary">
          <Icon className="size-4.5" />
        </div>
        <div className="min-w-0">
          <h2 className="font-semibold">{title}</h2>
          <p className="mt-0.5 text-xs text-muted-foreground">{description}</p>
        </div>
      </div>
      <div className="px-5 py-4">{children}</div>
    </section>
  )
}

/** 버튼 여러 개 중 하나를 고르는 묶음 */
function Choice<T extends string | number | null>({
  options,
  value,
  onChange,
  disabled,
  label,
}: {
  options: { value: T; label: string; icon?: typeof Sun }[]
  value: T
  onChange: (value: T) => void
  disabled?: boolean
  label: string
}) {
  return (
    <div
      role="radiogroup"
      aria-label={label}
      className="inline-flex flex-wrap gap-1 rounded-xl border bg-muted/40 p-1"
      onKeyDown={(e) =>
        !disabled &&
        rovingKeyDown(
          e,
          options.map((o) => o.value),
          value,
          () => false,
          onChange,
        )
      }
    >
      {options.map((o) => {
        const active = o.value === value
        return (
          <button
            key={String(o.value)}
            type="button"
            role="radio"
            aria-checked={active}
            tabIndex={active ? 0 : -1}
            disabled={disabled}
            onClick={() => !active && onChange(o.value)}
            className={cn(
              "flex h-8 items-center gap-1.5 rounded-lg px-3 text-sm transition-colors disabled:opacity-50",
              active ? "bg-background font-medium shadow-sm" : "text-muted-foreground hover:text-foreground",
            )}
          >
            {o.icon && <o.icon className="size-4" />}
            {o.label}
          </button>
        )
      })}
    </div>
  )
}

export function SettingsPage() {
  const queryClient = useQueryClient()
  const { theme, setTheme } = useTheme()
  const settings = useQuery({ queryKey: ["settings"], queryFn: settingsApi.get })
  const usage = useQuery({ queryKey: ["backup-usage"], queryFn: settingsApi.backupUsage })
  const [pendingRetention, setPendingRetention] = useState<number | null | undefined>(undefined)
  const [confirmPrune, setConfirmPrune] = useState(false)
  const [busy, setBusy] = useState(false)
  useBusy(busy)

  // "지금 정리"로 백업을 지우는 동안: 몇 번째 백업인지, 지운 크기
  const [pruneProgress, setPruneProgress] = useState<ScanProgress | null>(null)

  const refresh = () => {
    for (const key of ["settings", "backup-usage", "backups"]) queryClient.invalidateQueries({ queryKey: [key] })
  }

  const applyRetention = async (days: number | null) => {
    setBusy(true)
    try {
      await settingsApi.setRetention(days)
      toast.success(days == null ? "백업을 계속 보관해요" : `${days}일이 지난 백업은 앱을 켤 때 지워요`)
    } catch (e) {
      toast.error(String(e))
    } finally {
      setBusy(false)
      setPendingRetention(undefined)
      refresh()
    }
  }

  const prune = async () => {
    setBusy(true)
    setPruneProgress(null)
    try {
      const r = await settingsApi.pruneBackups(setPruneProgress)
      if (r.failed > 0) toast.warning(`${r.deleted}개를 지웠어요. ${r.failed}개는 복구 기록에서 하나씩 지워 주세요`)
      else toast.success(`오래된 백업 ${r.deleted}개를 지웠어요`)
    } catch (e) {
      toast.error(String(e))
    } finally {
      setBusy(false)
      setConfirmPrune(false)
      refresh()
    }
  }

  const removeExclusion = async (index: number, location: string) => {
    try {
      await settingsApi.removeExclusion(index, location)
      toast.success("제외 목록에서 뺐어요. 다음 스캔부터 다시 찾아요")
    } catch (e) {
      toast.error(String(e))
    } finally {
      refresh()
    }
  }

  const retention = settings.data?.backupRetentionDays ?? null
  const retentionOptions = [
    { value: null as number | null, label: "계속 보관" },
    ...(settings.data?.retentionChoices ?? []).map((d) => ({
      value: d as number | null,
      label: d === 30 ? "30일 (기본)" : `${d}일`,
    })),
  ]

  return (
    <Stagger className="relative mx-auto flex max-w-3xl min-w-0 flex-col gap-6 px-7 pt-7 pb-10">
      <FadeUp>
        <h1 className="text-2xl font-bold tracking-tight">설정</h1>
        <p className="mt-1 text-sm text-muted-foreground">화면, 백업 보관, 잔여물 스캔에서 뺄 항목을 정해요.</p>
      </FadeUp>

      <FadeUp>
        <Section
          icon={Palette}
          title="화면"
          description="밝기 테마를 고르세요. 시스템을 고르면 Windows 설정을 따라가요."
        >
          <div className="grid gap-4">
            <div>
              <p className="mb-2 text-sm font-medium">테마</p>
              <Choice
                label="테마"
                value={theme}
                onChange={setTheme}
                options={[
                  { value: "dark", label: "어둡게", icon: Moon },
                  { value: "light", label: "밝게", icon: Sun },
                  { value: "system", label: "시스템", icon: Monitor },
                ]}
              />
            </div>
          </div>
        </Section>
      </FadeUp>

      <FadeUp>
        <UpdatesSection />
      </FadeUp>

      <FadeUp>
        <Section
          icon={ArchiveRestore}
          title="백업 보관"
          description="잔여물을 정리할 때 지운 파일과 레지스트리는 모두 백업돼요. 백업을 지우면 되돌릴 수 없어요."
        >
          <div className="flex flex-wrap items-center gap-x-6 gap-y-2 text-sm">
            <span className="text-muted-foreground">
              지금 보관 중{" "}
              {usage.data ? (
                <b className="text-foreground tabular-nums">
                  {usage.data.count}개 · {formatBytes(usage.data.bytes)}
                </b>
              ) : (
                <span className="inline-block h-4 w-24 animate-pulse rounded bg-muted align-middle" />
              )}
            </span>
          </div>
          <div className="mt-4">
            <p className="mb-2 text-sm font-medium">보관 기간</p>
            {settings.isLoading ? (
              <Skeleton className="h-10 w-80 rounded-xl" />
            ) : (
              <Choice
                label="백업 보관 기간"
                value={retention}
                disabled={busy}
                onChange={(days) => (days == null ? applyRetention(null) : setPendingRetention(days))}
                options={retentionOptions}
              />
            )}
            <p className="mt-2 text-xs text-muted-foreground">
              {retention == null
                ? "백업을 직접 지울 때까지 계속 보관해요."
                : `앱을 켤 때 ${retention}일이 지난 백업을 지워요.`}
            </p>
          </div>
          {(usage.data?.expired ?? 0) > 0 && (
            <div className="mt-4 flex flex-wrap items-center gap-3 rounded-xl border border-chart-3/40 bg-chart-3/8 px-4 py-3 text-sm">
              <TriangleAlert className="size-4 shrink-0 text-warning" />
              <span className="mr-auto">보관 기간이 지난 백업이 {usage.data?.expired}개 있어요.</span>
              <Button size="sm" variant="outline" onClick={() => setConfirmPrune(true)} disabled={busy}>
                <Trash2 />
                지금 정리
              </Button>
            </div>
          )}
        </Section>
      </FadeUp>

      <FadeUp>
        <Section
          icon={EyeOff}
          title="잔여물 스캔에서 뺄 항목"
          description='잔여물 결과에서 "다음부터 제외"를 누른 위치예요. 폴더·키를 빼면 그 아래도 모두 빠져요.'
        >
          {settings.isLoading ? (
            <Skeleton className="h-16 w-full rounded-xl" />
          ) : (settings.data?.exclusions ?? []).length === 0 ? (
            <p className="py-4 text-center text-sm text-muted-foreground">
              뺀 항목이 없어요. 잔여물 결과의 항목에 마우스를 올리면 "다음부터 제외" 버튼이 보여요.
            </p>
          ) : (
            <ul className="divide-y rounded-xl border">
              {(settings.data?.exclusions ?? []).map((e, i) => {
                const Icon = e.kind === "path" ? EyeOff : KeyRound
                return (
                  <li key={`${e.location}-${e.valueName}-${i}`} className="flex items-start gap-3 px-4 py-2.5 text-xs">
                    <Icon className="mt-0.5 size-4 shrink-0 text-muted-foreground" />
                    <p data-selectable className="min-w-0 flex-1 font-mono break-all">
                      {e.location}
                      {e.valueName && ` › ${e.valueName}`}
                    </p>
                    <Button
                      variant="ghost"
                      size="icon"
                      className="size-7 shrink-0 text-muted-foreground"
                      aria-label={`${e.location}${e.valueName ? ` › ${e.valueName}` : ""} 제외 목록에서 빼기`}
                      onClick={() => removeExclusion(i, e.location)}
                    >
                      <X />
                    </Button>
                  </li>
                )
              })}
            </ul>
          )}
        </Section>
      </FadeUp>

      <Dialog open={pendingRetention != null} onOpenChange={(o) => !o && !busy && setPendingRetention(undefined)}>
        <DialogContent className="sm:max-w-md">
          <DialogTitle>{pendingRetention}일이 지난 백업을 지울까요?</DialogTitle>
          <DialogDescription className="leading-relaxed">
            앱을 켤 때마다 {pendingRetention}일이 지난 백업을 지워요.{" "}
            <b className="text-foreground">지운 백업으로는 되돌릴 수 없어요.</b> 나중에 "계속 보관"으로 바꿀 수 있어요.
          </DialogDescription>
          <div className="mt-2 flex justify-end gap-2">
            <Button variant="outline" onClick={() => setPendingRetention(undefined)} disabled={busy}>
              취소
            </Button>
            <Button onClick={() => pendingRetention != null && applyRetention(pendingRetention)} disabled={busy}>
              {busy && <Loader2 className="animate-spin" />}
              {pendingRetention}일로 정하기
            </Button>
          </div>
        </DialogContent>
      </Dialog>

      <Dialog open={confirmPrune} onOpenChange={(o) => !o && !busy && setConfirmPrune(false)}>
        <DialogContent className="sm:max-w-md">
          <DialogTitle>오래된 백업 {usage.data?.expired}개를 지울까요?</DialogTitle>
          <DialogDescription className="leading-relaxed">
            {retention}일이 지난 정리 기록과 보관 중인 항목이 영구히 지워져요.{" "}
            <b className="text-foreground">이후에는 되돌릴 수 없어요.</b>
          </DialogDescription>
          {busy && <ScanProgressView work label="오래된 백업을 지우는 중이에요" progress={pruneProgress} />}
          <div className="mt-2 flex justify-end gap-2">
            <Button variant="outline" onClick={() => setConfirmPrune(false)} disabled={busy}>
              취소
            </Button>
            <Button variant="destructive" onClick={prune} disabled={busy}>
              {busy ? <Loader2 className="animate-spin" /> : <Trash2 />}
              영구 삭제
            </Button>
          </div>
        </DialogContent>
      </Dialog>
    </Stagger>
  )
}

function UpdatesSection() {
  const { version, autoCheck, setAutoCheck, lastChecked, checking, checkNow } = useUpdates()
  return (
    <Section
      icon={Download}
      title="업데이트"
      description="새 버전이 나오면 업데이트 창을 띄워 알려 드려요. 설치는 직접 누를 때만 하고, 서명을 확인한 파일만 설치해요."
    >
      <div className="flex flex-wrap items-center gap-x-6 gap-y-3">
        <div className="min-w-0 flex-1 text-sm">
          <p>
            지금 버전 <b className="tabular-nums">{version ?? "…"}</b>
          </p>
          <p className="mt-0.5 text-xs text-muted-foreground">
            {lastChecked ? `마지막 확인 ${formatDateTime(lastChecked)}` : "아직 확인하지 않았어요"}
          </p>
        </div>
        <Button variant="outline" onClick={() => void checkNow(true)} disabled={checking}>
          {checking ? <Loader2 className="animate-spin" /> : <Download />}
          업데이트 확인
        </Button>
      </div>
      <label className="mt-4 flex cursor-pointer items-center justify-between gap-4 rounded-xl border bg-muted/30 px-4 py-3 text-sm">
        <span>
          <span className="block font-medium">앱을 켤 때 확인하기</span>
          <span className="block text-xs text-muted-foreground">
            켜 두면 시작하고 몇 초 뒤, 그 뒤로는 몇 시간마다 새 버전이 있는지 살펴봐요.
          </span>
        </span>
        <Switch checked={autoCheck} onCheckedChange={setAutoCheck} aria-label="앱을 켤 때 업데이트 확인하기" />
      </label>
    </Section>
  )
}
