import { useRef, useState } from "react"
import { motion, useReducedMotion } from "motion/react"
import { Tabs } from "radix-ui"
import { Archive, ScanSearch, Trash2 } from "lucide-react"

import { cn } from "@/lib/utils"
import { Reveal, SectionHeading, easeOutQuint } from "@/components/motion"
import { WindowFrame, shot } from "@/components/window-frame"

const steps = [
  {
    icon: Trash2,
    title: "제거",
    text: "목록에서 프로그램을 골라 제거를 누르면 그 프로그램의 기본 언인스톨러가 실행돼요. 끝날 때까지 기다렸다가 결과를 확인해요.",
  },
  {
    icon: ScanSearch,
    title: "남은 흔적 찾기",
    text: "제거가 끝나면 파일·레지스트리·시작 항목을 훑어 남은 흔적을 찾아요. 항목마다 왜 그 프로그램 것인지 근거가 붙어요.",
  },
  {
    icon: Archive,
    title: "백업하고 정리",
    text: "고른 항목만 백업한 뒤 치워요. 문제가 생기면 복구 기록에서 바로 되돌리면 돼요.",
  },
]

export function How() {
  return (
    <section id="how" className="relative mx-auto max-w-6xl px-6 pt-32">
      <SectionHeading
        eyebrow="사용 방법"
        title="세 단계면 끝나요"
        description="어려운 설정 없이, 평소처럼 제거만 누르면 나머지는 RobinX가 이어서 챙겨요."
      />
      <div className="relative mt-16 grid gap-6 md:grid-cols-3">
        {/* 단계를 잇는 선 */}
        <div
          aria-hidden
          className="absolute top-7 right-[16%] left-[16%] hidden h-px bg-gradient-to-r from-primary/0 via-primary/40 to-primary/0 md:block"
        />
        {steps.map((s, i) => (
          <Reveal key={s.title} delay={i * 0.12} className="relative text-center">
            <div className="relative mx-auto flex size-14 items-center justify-center rounded-2xl border border-primary/30 bg-background shadow-[0_0_40px_-8px] shadow-primary/40">
              <s.icon className="size-6 text-primary" />
              <span className="absolute -top-2 -right-2 flex size-6 items-center justify-center rounded-full bg-primary text-xs font-bold text-primary-foreground">
                {i + 1}
              </span>
            </div>
            <h3 className="mt-6 text-xl font-semibold">{s.title}</h3>
            <p className="mx-auto mt-2.5 max-w-xs text-[0.95rem] leading-relaxed text-pretty text-muted-foreground">
              {s.text}
            </p>
          </Reveal>
        ))}
      </div>
    </section>
  )
}

const screens = [
  { key: "programs", label: "프로그램", caption: "설치된 프로그램을 크기·설치일·형식과 함께 보여 줘요." },
  { key: "programs-grid", label: "카드 보기", caption: "아이콘 위주로 훑어보고 싶을 땐 카드 보기로." },
  { key: "leftovers", label: "남은 흔적", caption: "확실한 흔적은 미리 고르고, 애매한 건 검토하도록 나눠요." },
  {
    key: "restore",
    label: "복구 기록",
    caption: "정리한 흔적은 작업마다 백업이 남아, 보관하는 동안 되돌릴 수 있어요.",
  },
  { key: "startup", label: "시작프로그램", caption: "켜고 끄기만 할 뿐 등록 정보는 지우지 않아요." },
  { key: "storage", label: "저장공간", caption: "다시 만들어지는 캐시만 골라 공간을 확보해요." },
]

export function Gallery() {
  const [active, setActive] = useState(screens[0].key)
  const reduce = useReducedMotion()
  const tabRefs = useRef(new Map<string, HTMLButtonElement>())

  // 좁은 화면에서 탭 목록이 가로로 넘치면, 고른 탭이 보이도록 가운데로 옮긴다
  const select = (key: string) => {
    setActive(key)
    tabRefs.current
      .get(key)
      ?.scrollIntoView({ behavior: reduce ? "auto" : "smooth", block: "nearest", inline: "center" })
  }

  return (
    <section id="screens" className="relative pt-20 sm:pt-28 lg:pt-32">
      <div
        aria-hidden
        className="absolute inset-x-0 top-40 -z-10 mx-auto h-[30rem] max-w-4xl rounded-full bg-[radial-gradient(closest-side,oklch(0.72_0.15_285/0.14),transparent)] blur-2xl"
      />
      <div className="mx-auto max-w-6xl px-6">
        <SectionHeading
          eyebrow="화면"
          title="깔끔하고 차분한 화면"
          description="밝은 테마와 어두운 테마, Windows 11의 Mica 효과를 지원해요."
        />

        {/* Radix Tabs: 방향키로 탭 이동, 탭과 패널이 aria로 연결된다 */}
        <Tabs.Root value={active} onValueChange={select} className="mt-10">
          <Reveal className="relative flex justify-center">
            <Tabs.List
              aria-label="앱 화면"
              className="glass liquid flex max-w-full gap-1 overflow-x-auto rounded-full p-1.5 [scrollbar-width:none] [&::-webkit-scrollbar]:hidden"
            >
              {screens.map((s) => (
                <Tabs.Trigger
                  key={s.key}
                  value={s.key}
                  ref={(el) => {
                    if (el) tabRefs.current.set(s.key, el)
                  }}
                  className={cn(
                    "relative shrink-0 rounded-full px-4 py-2 text-sm transition-colors outline-none",
                    "focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 focus-visible:ring-offset-background",
                    active === s.key
                      ? "font-semibold text-primary-foreground"
                      : "text-muted-foreground hover:text-foreground",
                  )}
                >
                  {active === s.key && (
                    <motion.span
                      layoutId="gallery-tab"
                      className="absolute inset-0 rounded-full bg-primary shadow-[inset_0_1px_0_oklch(1_0_0/35%)]"
                      transition={reduce ? { duration: 0 } : { type: "spring", duration: 0.4, bounce: 0.15 }}
                    />
                  )}
                  <span className="relative">{s.label}</span>
                </Tabs.Trigger>
              ))}
            </Tabs.List>
            {/* 가로로 더 있다는 단서 (좁은 화면) */}
            <span
              aria-hidden
              className="pointer-events-none absolute inset-y-0 right-0 w-10 rounded-r-full bg-gradient-to-l from-background to-transparent sm:hidden"
            />
          </Reveal>

          <Reveal className="mx-auto mt-8 max-w-5xl">
            {screens.map((s) => (
              <Tabs.Content
                key={s.key}
                value={s.key}
                className="outline-none focus-visible:ring-2 focus-visible:ring-ring rounded-2xl"
              >
                <motion.div
                  initial={reduce ? false : { opacity: 0 }}
                  animate={{ opacity: 1 }}
                  transition={{ duration: 0.35, ease: easeOutQuint }}
                >
                  <WindowFrame src={shot(s.key)} alt={`${s.label} 화면`} />
                  <p className="mt-5 text-center text-sm text-muted-foreground">{s.caption}</p>
                </motion.div>
              </Tabs.Content>
            ))}
            <p className="mt-1 text-center text-xs text-muted-foreground">
              실제 개발 버전의 화면이에요. 화면 속 프로그램과 숫자는 설명을 위한 예시예요.
            </p>
          </Reveal>
        </Tabs.Root>
      </div>
    </section>
  )
}
