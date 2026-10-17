import { Check, Settings, type LucideIcon } from "lucide-react"

import { FadeUp, Stagger } from "@/components/motion"
import type { Page } from "@/components/app-sidebar"

type Info = {
  icon: LucideIcon
  title: string
  description: string
  phase: string
  points: string[]
}

const pages: Record<Exclude<Page, "programs" | "leftovers" | "restore" | "startup" | "storage" | "tracking">, Info> = {
  settings: {
    icon: Settings,
    title: "설정",
    description: "테마, 스캔 범위, 백업 보관 기간 같은 옵션을 정할 수 있어요.",
    phase: "MVP 이후",
    points: ["테마와 애니메이션", "백업 보관 위치·기간", "스캔 제외 목록"],
  },
}

export function ComingSoonPage({
  page,
}: {
  page: Exclude<Page, "programs" | "leftovers" | "restore" | "startup" | "storage" | "tracking">
}) {
  const info = pages[page]
  const Icon = info.icon

  return (
    <Stagger
      key={page}
      className="relative mx-auto flex min-h-full max-w-xl flex-col items-center justify-center px-7 py-16 text-center"
    >
      <FadeUp>
        <div className="relative mb-6">
          <div aria-hidden className="absolute inset-0 -z-10 scale-150 rounded-full bg-primary/15 blur-2xl" />
          <div className="flex size-20 items-center justify-center rounded-3xl border bg-card shadow-sm">
            <Icon className="size-9 text-primary" strokeWidth={1.6} />
          </div>
        </div>
      </FadeUp>
      <FadeUp>
        <span className="rounded-full border border-primary/30 bg-primary/10 px-2.5 py-0.5 text-xs font-medium text-primary">
          {info.phase}에서 만나요
        </span>
      </FadeUp>
      <FadeUp>
        <h1 className="mt-4 text-2xl font-bold tracking-tight">{info.title}</h1>
        <p className="mt-2 text-sm leading-relaxed text-muted-foreground">{info.description}</p>
      </FadeUp>
      <FadeUp className="mt-8 w-full">
        <ul className="divide-y rounded-2xl border bg-card text-left text-sm shadow-sm">
          {info.points.map((point) => (
            <li key={point} className="flex items-center gap-3 px-5 py-3.5">
              <span className="flex size-5 shrink-0 items-center justify-center rounded-full bg-primary/12 text-primary">
                <Check className="size-3" strokeWidth={3} />
              </span>
              {point}
            </li>
          ))}
        </ul>
      </FadeUp>
    </Stagger>
  )
}
