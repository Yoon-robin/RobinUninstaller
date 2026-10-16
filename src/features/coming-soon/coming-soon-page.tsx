import { ArchiveRestore, Check, Radar, Rocket, Settings, Trash2, type LucideIcon } from "lucide-react"

import { FadeUp, Stagger } from "@/components/motion"
import type { Page } from "@/components/app-sidebar"

type Info = {
  icon: LucideIcon
  title: string
  description: string
  phase: string
  points: string[]
}

const pages: Record<Exclude<Page, "programs">, Info> = {
  leftovers: {
    icon: Trash2,
    title: "잔여물 정리",
    description: "프로그램을 지운 뒤 남은 파일과 레지스트리를 찾아, 왜 그 프로그램 것인지 근거와 함께 보여 줘요.",
    phase: "Phase 3",
    points: ["설치 폴더·AppData·레지스트리 스캔", "확실한 것만 미리 선택, 애매한 건 검토", "지우기 전에 자동 백업"],
  },
  startup: {
    icon: Rocket,
    title: "시작프로그램",
    description: "Windows가 켜질 때 함께 실행되는 프로그램을 확인하고 끄거나 다시 켤 수 있어요.",
    phase: "Phase 5",
    points: ["Run 키·시작 폴더·예약 작업", "끈 항목은 언제든 원상 복구", "게시자와 파일 위치 표시"],
  },
  tracking: {
    icon: Radar,
    title: "설치 추적",
    description: "설치하는 동안 무엇이 바뀌었는지 기록해 두었다가, 나중에 흔적 없이 깨끗하게 지워요.",
    phase: "Phase 6",
    points: ["설치 전후 스냅샷 비교", "다른 프로그램의 변경과 구분", "추적 기록 기반 완전 제거"],
  },
  restore: {
    icon: ArchiveRestore,
    title: "복구 기록",
    description: "RobinUninstaller가 지운 모든 항목의 기록과 백업이에요. 실수로 지운 것도 되돌릴 수 있어요.",
    phase: "Phase 4",
    points: ["삭제한 파일은 격리 보관", "레지스트리 값까지 백업", "항목별 또는 한 번에 복원"],
  },
  settings: {
    icon: Settings,
    title: "설정",
    description: "테마, 스캔 범위, 백업 보관 기간 같은 옵션을 정할 수 있어요.",
    phase: "MVP 이후",
    points: ["테마와 애니메이션", "백업 보관 위치·기간", "스캔 제외 목록"],
  },
}

export function ComingSoonPage({ page }: { page: Exclude<Page, "programs"> }) {
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
