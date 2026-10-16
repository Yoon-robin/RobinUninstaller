import { useState } from "react"
import { useQuery } from "@tanstack/react-query"
import { ArchiveRestore, ArrowRight, Sparkles, Trash2, TriangleAlert } from "lucide-react"

import { api, scanApi, type InstalledApplication } from "@/lib/api"
import { FadeUp, Stagger } from "@/components/motion"
import { Button } from "@/components/ui/button"
import { AppIcon } from "@/features/programs/app-icon"
import { LeftoversDialog, type LeftoverTarget } from "./leftovers-dialog"

/** 잔여물 정리: 이번 실행에서 제거한 프로그램과, 언인스톨러가 없어 흔적만 남은 프로그램. */
export function LeftoversPage() {
  const [target, setTarget] = useState<LeftoverTarget | null>(null)
  const removed = useQuery({ queryKey: ["recently-removed"], queryFn: scanApi.recentlyRemoved })
  const installed = useQuery({ queryKey: ["installed-apps"], queryFn: api.listInstalledApps })
  const broken = (installed.data ?? []).filter((a) => a.uninstaller === "missing")

  return (
    <Stagger className="relative mx-auto flex max-w-4xl min-w-0 flex-col gap-6 px-7 pt-7 pb-10">
      <FadeUp>
        <h1 className="text-2xl font-bold tracking-tight">잔여물 정리</h1>
        <p className="mt-1 text-sm text-muted-foreground">
          언인스톨러가 남기고 간 파일과 레지스트리를 찾아 정리해요. 정리한 항목은 모두 백업돼 복구 기록에서 되돌릴 수
          있어요.
        </p>
      </FadeUp>

      <FadeUp>
        <Section
          title="방금 제거한 프로그램"
          empty={
            <div className="flex flex-col items-center gap-3 px-6 py-10 text-center">
              <Flow />
              <p className="text-sm text-muted-foreground">
                프로그램을 제거하면 여기에 나타나요. 제거가 끝난 화면에서 바로 찾을 수도 있어요.
              </p>
            </div>
          }
          apps={removed.data ?? []}
          action="남은 흔적 찾기"
          onPick={(app) => setTarget({ id: app.id, name: app.displayName, removed: true })}
        />
      </FadeUp>

      {broken.length > 0 && (
        <FadeUp>
          <Section
            title="언인스톨러가 없는 프로그램"
            hint="등록 정보만 남았거나 언인스톨러 파일이 사라진 프로그램이에요"
            apps={broken}
            action="흔적 찾기"
            warn
            onPick={(app) => setTarget({ id: app.id, name: app.displayName, removed: true })}
          />
        </FadeUp>
      )}

      <LeftoversDialog target={target} onClose={() => setTarget(null)} />
    </Stagger>
  )
}

function Section({
  title,
  hint,
  apps,
  action,
  warn,
  empty,
  onPick,
}: {
  title: string
  hint?: string
  apps: InstalledApplication[]
  action: string
  warn?: boolean
  empty?: React.ReactNode
  onPick: (app: InstalledApplication) => void
}) {
  return (
    <section className="overflow-hidden rounded-2xl border bg-card shadow-sm">
      <div className="border-b px-5 py-3.5">
        <h2 className="flex items-center gap-2 font-semibold">
          {warn && <TriangleAlert className="size-4 text-chart-3" />}
          {title}
          <span className="text-sm font-normal text-muted-foreground tabular-nums">{apps.length}</span>
        </h2>
        {hint && <p className="mt-0.5 text-xs text-muted-foreground">{hint}</p>}
      </div>
      {apps.length === 0 ? (
        empty
      ) : (
        <ul className="divide-y">
          {apps.map((app) => (
            <li key={app.id} className="flex items-center gap-3 px-5 py-3">
              <AppIcon app={app} />
              <div className="min-w-0 flex-1">
                <p className="truncate font-medium">{app.displayName}</p>
                <p className="truncate text-xs text-muted-foreground">{app.publisher ?? "게시자 정보 없음"}</p>
              </div>
              <Button size="sm" variant="outline" className="rounded-lg" onClick={() => onPick(app)}>
                <Sparkles />
                {action}
              </Button>
            </li>
          ))}
        </ul>
      )}
    </section>
  )
}

/** 정리 흐름 안내: 제거 → 흔적 찾기 → 백업하고 정리 */
function Flow() {
  const steps = [
    { icon: Trash2, label: "기본 언인스톨러로 제거" },
    { icon: Sparkles, label: "남은 흔적 찾기" },
    { icon: ArchiveRestore, label: "백업하고 정리" },
  ]
  return (
    <div className="flex flex-wrap items-center justify-center gap-2 text-xs">
      {steps.map((s, i) => (
        <div key={s.label} className="flex items-center gap-2">
          <span className="flex items-center gap-1.5 rounded-full bg-primary/10 px-3 py-1.5 text-primary">
            <s.icon className="size-3.5" />
            {s.label}
          </span>
          {i < steps.length - 1 && <ArrowRight className="size-3.5 text-muted-foreground" />}
        </div>
      ))}
    </div>
  )
}
