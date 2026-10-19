import type { ReactNode } from "react"
import type { LucideIcon } from "lucide-react"
import {
  Archive,
  Boxes,
  HardDrive,
  Hammer,
  Layers,
  Radar,
  Rocket,
  ScanSearch,
  ShieldCheck,
  Undo2,
  FileSearch,
  Monitor,
} from "lucide-react"

import { cn } from "@/lib/utils"
import { Reveal, SectionHeading } from "@/components/motion"
import { shot } from "@/components/window-frame"

const values: { icon: LucideIcon; title: string; detail: string }[] = [
  { icon: Undo2, title: "영구 삭제 없음", detail: "지운 건 모두 백업·격리" },
  { icon: FileSearch, title: "근거 있는 탐지", detail: "왜 그 프로그램 것인지 표시" },
  { icon: ShieldCheck, title: "보호 규칙", detail: "시스템·공유 경로 제외" },
  { icon: Monitor, title: "Windows 10 · 11", detail: "가볍고 빠른 네이티브 앱" },
]

export function ValueStrip() {
  return (
    <section className="relative mx-auto max-w-6xl px-6">
      <Reveal>
        <div className="grid grid-cols-2 gap-px overflow-hidden rounded-2xl border border-white/8 bg-white/8 lg:grid-cols-4">
          {values.map((v) => (
            <div key={v.title} className="flex items-center gap-3.5 bg-background/95 px-5 py-5">
              <span className="flex size-10 shrink-0 items-center justify-center rounded-xl bg-primary/10 text-primary">
                <v.icon className="size-5" />
              </span>
              <span>
                <span className="block text-sm font-semibold">{v.title}</span>
                <span className="block text-xs text-muted-foreground">{v.detail}</span>
              </span>
            </div>
          ))}
        </div>
      </Reveal>
    </section>
  )
}

const tones = {
  teal: "bg-chart-1/12 text-chart-1",
  violet: "bg-chart-2/12 text-chart-2",
  amber: "bg-chart-3/12 text-chart-3",
  rose: "bg-chart-4/12 text-chart-4",
  green: "bg-chart-5/12 text-chart-5",
} as const

function Card({
  icon: Icon,
  tone,
  title,
  children,
  className,
  visual,
  delay = 0,
}: {
  icon: LucideIcon
  tone: keyof typeof tones
  title: string
  children: ReactNode
  className?: string
  visual?: ReactNode
  delay?: number
}) {
  return (
    <Reveal delay={delay} className={cn("h-full", className)}>
      <article className="group relative flex h-full flex-col overflow-hidden glass rounded-3xl p-6 transition-transform duration-300 hover:-translate-y-1 sm:p-7">
        {/* 마우스를 올리면 위쪽 테두리에 빛이 돈다 */}
        <span
          aria-hidden
          className="absolute inset-x-8 top-0 h-px bg-gradient-to-r from-transparent via-primary/60 to-transparent opacity-0 transition-opacity duration-300 group-hover:opacity-100"
        />
        <span className={cn("flex size-11 items-center justify-center rounded-2xl", tones[tone])}>
          <Icon className="size-5.5" />
        </span>
        <h3 className="mt-5 text-lg font-semibold tracking-tight">{title}</h3>
        <p className="mt-2 text-[0.95rem] leading-relaxed text-pretty text-muted-foreground">{children}</p>
        {visual}
      </article>
    </Reveal>
  )
}

/** 카드 아래쪽에 스크린샷 일부를 비스듬히 보여 준다. */
function ShotPeek({ name, x = 0.21, y = 0 }: { name: string; x?: number; y?: number }) {
  return (
    <div className="relative -mx-6 mt-6 -mb-6 h-56 overflow-hidden sm:-mx-7 sm:-mb-7 sm:h-64">
      <img
        src={shot(name)}
        alt=""
        loading="lazy"
        className="absolute top-4 left-6 w-[155%] max-w-none rounded-tl-xl border-t border-l border-white/10 shadow-2xl shadow-black/60 transition-transform duration-500 ease-out group-hover:-translate-y-2 sm:left-7"
        // 사이드바를 건너뛰고 화면의 핵심 부분부터 보이게 옮긴다 (그림 자신의 크기 기준 비율)
        style={{ transform: `translate(${-x * 100}%, ${-y * 100}%)` }}
      />
      <div aria-hidden className="absolute inset-x-0 bottom-0 h-16 bg-gradient-to-t from-card/90 to-transparent" />
    </div>
  )
}

export function Features() {
  return (
    <section id="features" className="relative mx-auto max-w-6xl px-6 pt-32">
      <SectionHeading
        eyebrow="기능"
        title={
          <>
            제거 버튼 하나로 끝나지 않는
            <br className="hidden sm:block" /> 진짜 정리
          </>
        }
        description="기본 언인스톨러가 남기고 간 폴더, 레지스트리, 시작 항목까지. 무엇이 왜 남았는지 보여 주고, 고른 것만 안전하게 치워요."
      />

      <div className="mt-14 grid gap-4 md:grid-cols-6">
        <Card
          className="md:col-span-4"
          icon={ScanSearch}
          tone="teal"
          title="근거와 함께 찾는 남은 흔적"
          visual={<ShotPeek name="leftovers" x={0.2} y={0.05} />}
        >
          게시자·제품 이름, 설치 폴더를 가리키는 값 같은 근거로 점수를 매겨 <b className="text-foreground">확실</b>과{" "}
          <b className="text-foreground">검토 필요</b>로 나눠요. 확실한 것만 미리 골라 두고, 내 문서가 섞인 폴더는 따로
          알려 줘요.
        </Card>
        <Card className="md:col-span-2" icon={Archive} tone="violet" title="백업하고, 언제든 되돌리기" delay={0.08}>
          파일은 격리 폴더로 옮기고 레지스트리는 내보내 둔 뒤 정리해요. 복구 기록에서 항목별로, 혹은 한 번에 원래 자리로
          되돌릴 수 있어요.
        </Card>

        <Card className="md:col-span-2" icon={Hammer} tone="rose" title="언인스톨러가 없어도 강제 제거">
          제거 프로그램이 사라졌거나 망가진 경우에도 설치 정보와 흔적을 찾아 백업 후 정리해요. 보안 프로그램·드라이버는
          제외해요.
        </Card>
        <Card className="md:col-span-2" icon={Layers} tone="amber" title="여러 개를 차례로 제거" delay={0.06}>
          지울 프로그램을 여러 개 골라 두면 하나씩 언인스톨러를 실행하고, 끝날 때마다 남은 흔적을 모아서 보여 줘요.
        </Card>
        <Card className="md:col-span-2" icon={Radar} tone="green" title="설치 추적" delay={0.12}>
          설치 전후를 비교해 새로 생긴 파일과 레지스트리를 기록해 둬요. 나중에 지울 때 가장 확실한 근거가 돼요.
        </Card>

        <Card
          className="md:col-span-3"
          icon={Rocket}
          tone="teal"
          title="시작프로그램 관리"
          visual={<ShotPeek name="startup" />}
        >
          로그인할 때 함께 켜지는 프로그램과 예약 작업을 한눈에 보고 끄거나 다시 켜요. 실행 파일이 사라진 항목도 골라
          정리할 수 있어요.
        </Card>
        <Card
          className="md:col-span-3"
          icon={HardDrive}
          tone="violet"
          title="저장공간 정리"
          delay={0.08}
          visual={<ShotPeek name="storage" />}
        >
          임시 파일·캐시처럼 지워도 다시 만들어지는 것만 골라요. 되돌릴 수 없는 항목은 따로 경고하고 미리 고르지 않아요.
          내 파일은 절대 지우지 않아요.
        </Card>

        <Card className="md:col-span-6" icon={Boxes} tone="amber" title="데스크톱 프로그램과 Store 앱을 한곳에서">
          EXE·MSI 프로그램과 Microsoft Store·MSIX 앱을 함께 보여 줘요. 크기 기록이 없는 프로그램은 설치 폴더를 직접
          재고, 목록은 CSV로 내보낼 수 있어요.
        </Card>
      </div>
    </section>
  )
}
