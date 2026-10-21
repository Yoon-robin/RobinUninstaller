import { useState } from "react"
import { useQuery } from "@tanstack/react-query"
import { ArrowRight, Check, Sparkles, Trash2, TriangleAlert, type LucideIcon } from "lucide-react"

import { navigate, api, needsForce, scanApi, type InstalledApplication, type RemovedApp } from "@/lib/api"
import { formatDay } from "@/lib/format"
import { FadeUp, Stagger } from "@/components/motion"
import { ErrorState } from "@/components/error-state"
import { Skeleton } from "@/components/ui/skeleton"
import { Button } from "@/components/ui/button"
import { AppIcon } from "@/features/programs/app-icon"
import { LeftoversDialog, type LeftoverTarget } from "./leftovers-dialog"

/** 잔여물 정리: 제거한 프로그램(보관 기간 동안 남는다)과, 언인스톨러가 없어 흔적만 남은 프로그램. */
export function LeftoversPage() {
  const [target, setTarget] = useState<LeftoverTarget | null>(null)
  // 이번에 정리를 마친 앱 (목록에 "정리함" 표시)
  const [cleaned, setCleaned] = useState<Set<string>>(() => new Set())
  const removed = useQuery({ queryKey: ["recently-removed"], queryFn: scanApi.recentlyRemoved })
  const installed = useQuery({ queryKey: ["installed-apps"], queryFn: api.listInstalledApps })
  const broken = (installed.data ?? []).filter(needsForce)

  return (
    <Stagger className="relative mx-auto flex max-w-4xl min-w-0 flex-col gap-6 px-7 pt-7 pb-10">
      <FadeUp>
        <h1 className="text-2xl font-bold tracking-tight">잔여물 정리</h1>
        <p className="mt-1 text-sm text-muted-foreground">언인스톨러가 남기고 간 파일과 레지스트리를 찾아 정리해요.</p>
      </FadeUp>

      <FadeUp>
        <Section
          title="제거한 프로그램"
          hint="백업 보관 기간 동안 여기에 남아 있어요. PC를 다시 시작한 뒤에 찾아도 돼요"
          empty={
            <div className="flex flex-col items-center gap-3 px-6 py-10 text-center">
              <Flow />
              <p className="text-sm text-muted-foreground">
                프로그램을 제거하면 여기에 나타나요. 제거가 끝난 화면에서 바로 찾거나, 다시 시작한 뒤 여기서 찾을 수
                있어요.
              </p>
              <Button variant="outline" size="sm" onClick={() => navigate("programs")}>
                <Trash2 />
                프로그램 화면에서 제거 시작
              </Button>
            </div>
          }
          apps={removed.data ?? []}
          loading={removed.isLoading}
          error={removed.isError ? removed.error : undefined}
          onRetry={() => removed.refetch()}
          cleaned={cleaned}
          action="남은 흔적 찾기"
          icon={Sparkles}
          onPick={(app) => setTarget({ id: app.id, name: app.displayName, removed: true })}
        />
      </FadeUp>

      {broken.length > 0 && (
        <FadeUp>
          <Section
            title="언인스톨러가 없는 프로그램"
            hint="등록 정보만 남았거나 언인스톨러 파일이 사라진 프로그램이에요"
            apps={broken}
            action="강제 제거"
            icon={Trash2}
            cleaned={cleaned}
            warn
            onPick={(app) => setTarget({ id: app.id, name: app.displayName, removed: false, force: true })}
          />
        </FadeUp>
      )}

      <LeftoversDialog
        target={target}
        onClose={() => setTarget(null)}
        onCleaned={(id) => setCleaned((s) => new Set(s).add(id))}
      />
    </Stagger>
  )
}

function Section({
  title,
  hint,
  apps,
  loading,
  error,
  onRetry,
  cleaned,
  action,
  icon: ActionIcon,
  warn,
  empty,
  onPick,
}: {
  title: string
  hint?: string
  apps: (InstalledApplication | RemovedApp)[]
  loading?: boolean
  error?: unknown
  onRetry?: () => void
  cleaned: Set<string>
  action: string
  /** 창 안의 같은 동작과 같은 아이콘 (흔적 찾기 ✦, 강제 제거 휴지통) */
  icon: LucideIcon
  warn?: boolean
  empty?: React.ReactNode
  onPick: (app: InstalledApplication) => void
}) {
  return (
    <section className="overflow-hidden rounded-2xl border bg-card shadow-sm">
      <div className="border-b px-5 py-3.5">
        <h2 className="flex items-center gap-2 font-semibold">
          {warn && <TriangleAlert className="size-4 text-warning" />}
          {title}
          {!loading && !error && (
            <span className="text-sm font-normal text-muted-foreground tabular-nums">{apps.length}</span>
          )}
        </h2>
        {hint && <p className="mt-0.5 text-xs text-muted-foreground">{hint}</p>}
      </div>
      {error ? (
        <ErrorState title="목록을 읽지 못했어요" error={error} onRetry={onRetry} className="py-8" />
      ) : loading ? (
        <div className="divide-y">
          {Array.from({ length: 2 }, (_, i) => (
            <div key={i} className="flex items-center gap-3 px-5 py-3">
              <Skeleton className="size-9 rounded-lg" />
              <Skeleton className="h-4 w-40" />
            </div>
          ))}
        </div>
      ) : apps.length === 0 ? (
        empty
      ) : (
        <ul className="divide-y">
          {apps.map((app) => (
            <li key={app.id} className="flex items-center gap-3 px-5 py-3">
              <AppIcon app={app} />
              <div className="min-w-0 flex-1">
                <p className="flex items-center gap-2 truncate font-medium">
                  {app.displayName}
                  {cleaned.has(app.id) && (
                    <span className="inline-flex shrink-0 items-center gap-1 rounded-md bg-chart-5/12 px-1.5 py-0.5 text-[11px] font-medium text-success">
                      <Check className="size-3" />
                      정리함
                    </span>
                  )}
                </p>
                <p className="truncate text-xs text-muted-foreground">
                  {app.publisher ?? "게시자 정보 없음"}
                  {"removedAt" in app && ` · ${formatDay(app.removedAt * 1000)} 제거`}
                  {"scanned" in app && !app.scanned && !cleaned.has(app.id) && (
                    <span className="font-medium text-primary"> · 아직 찾지 않음</span>
                  )}
                </p>
              </div>
              <Button
                size="sm"
                variant={cleaned.has(app.id) ? "ghost" : "outline"}
                className="rounded-lg"
                onClick={() => onPick(app)}
              >
                <ActionIcon />
                {cleaned.has(app.id) ? "다시 찾기" : action}
              </Button>
            </li>
          ))}
        </ul>
      )}
    </section>
  )
}

/** 정리 흐름 안내: 제거 → 흔적 찾기 → 백업하고 정리 (눌리는 것이 아니라 단계 표시) */
function Flow() {
  const steps = ["기본 언인스톨러로 제거", "남은 흔적 찾기", "백업하고 정리"]
  return (
    <ol className="flex flex-wrap items-center justify-center gap-x-2 gap-y-1 text-xs text-muted-foreground">
      {steps.map((label, i) => (
        <li key={label} className="flex items-center gap-2">
          <span className="flex items-center gap-1.5">
            <span className="flex size-4 items-center justify-center rounded-full border text-[10px] tabular-nums">
              {i + 1}
            </span>
            {label}
          </span>
          {i < steps.length - 1 && <ArrowRight className="size-3 opacity-60" aria-hidden />}
        </li>
      ))}
    </ol>
  )
}
