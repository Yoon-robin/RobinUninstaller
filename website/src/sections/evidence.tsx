import { Fragment, useEffect, useRef, useState } from "react"
import { CircleCheck, CircleDashed, Folder, KeyRound, ListTree, ShieldCheck, ShieldMinus } from "lucide-react"
import { motion, useInView, useReducedMotion } from "motion/react"

import { cn } from "@/lib/utils"
import { easeOutQuint, Reveal } from "@/components/motion"

type Group = "sure" | "review" | "protected"

/**
 * 남은 흔적을 어떤 근거로 나누는지. 앱의 탐지 규칙(crates/scanner)과 같은 말로 쓴다.
 * 왼쪽은 앱의 "남은 흔적" 창을 HTML로 옮긴 조각(data-ui, 예시 데이터), 오른쪽은 세 갈래 설명.
 * 오른쪽에서 하나를 고르면 왼쪽에서 그 갈래의 항목만 또렷해진다. 화면에 보이는 동안 저절로 차례대로 넘어가고,
 * 직접 고르면 멈춘다 (움직임 줄이기면 저절로 넘어가지 않는다).
 */
const groups: {
  key: Group
  icon: typeof CircleCheck
  tone: string
  chip: string
  label: string
  title: string
  text: string
}[] = [
  {
    key: "sure",
    icon: CircleCheck,
    tone: "text-sure",
    chip: "bg-sure/12 text-sure",
    label: "연결 정보 일치",
    title: "미리 골라 둬요",
    text: "Windows에 기록된 설치 폴더나 남은 제거 등록 정보처럼, 등록 정보가 그 프로그램 것이라고 직접 가리키는 항목이에요.",
  },
  {
    key: "review",
    icon: CircleDashed,
    tone: "text-review",
    chip: "bg-review/12 text-review",
    label: "직접 확인 필요",
    title: "직접 보고 골라요",
    text: "이름만 같거나 다른 프로그램과 함께 쓰는 것 같은 폴더예요. 다른 프로그램 것일 수 있어서 미리 고르지 않아요.",
  },
  {
    key: "protected",
    icon: ShieldMinus,
    tone: "text-muted-foreground",
    chip: "bg-white/[0.07] text-muted-foreground",
    label: "보호 규칙으로 제외",
    title: "건드리지 않아요",
    text: "Windows 폴더, 여러 프로그램이 함께 쓰는 폴더, 문서·바탕 화면 같은 사용자 폴더, RobinX 자신의 파일은 찾아도 목록에 넣지 않아요.",
  },
]

/** 앱 화면과 같은 예시 (website/scripts/capture.mjs의 Pixel Forge 3D) */
const rows: {
  group: Exclude<Group, "protected">
  icon: typeof Folder
  path: string
  value?: string
  reason: string
  size?: string
}[] = [
  {
    group: "sure",
    icon: Folder,
    path: String.raw`C:\Program Files\Forge Interactive\Pixel Forge 3D`,
    reason: "등록된 설치 폴더",
    size: "204 MB",
  },
  {
    group: "sure",
    icon: ListTree,
    path: String.raw`HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall\PixelForge3D`,
    reason: "남은 제거 등록 정보",
  },
  {
    group: "sure",
    icon: KeyRound,
    path: String.raw`HKCU\SOFTWARE\Microsoft\Windows\CurrentVersion\Run`,
    value: "PixelForgeHelper",
    reason: "설치 폴더를 가리킴",
  },
  {
    group: "review",
    icon: Folder,
    path: String.raw`C:\Users\me\AppData\Roaming\Forge Interactive\Pixel Forge 3D`,
    reason: "게시자 › 제품 이름 일치",
    size: "37 MB",
  },
  {
    group: "review",
    icon: Folder,
    path: String.raw`C:\Users\me\AppData\Local\Pixel Forge 3D`,
    reason: "이름 일치",
    size: "1.2 GB",
  },
  {
    group: "review",
    icon: ListTree,
    path: String.raw`HKCU\SOFTWARE\Forge Interactive\Pixel Forge 3D`,
    reason: "게시자 › 제품 이름 일치",
  },
  {
    group: "review",
    icon: Folder,
    path: String.raw`C:\Users\me\Documents\Pixel Forge Projects`,
    reason: "이름 일부 일치 · 직접 만든 문서 14개 포함",
    size: "92 MB",
  },
]

/** 앱의 체크 상자. 확실한 항목은 화면에 들어오면 차례로 체크가 그어진다 */
function Check({ on, delay, animate }: { on: boolean; delay: number; animate: boolean }) {
  return (
    <span
      className={cn(
        "flex size-4 shrink-0 items-center justify-center rounded-[5px] border transition-colors duration-300",
        on ? "border-sure bg-sure" : "border-white/25",
      )}
      style={{ transitionDelay: animate && on ? `${delay}s` : "0s" }}
    >
      <svg viewBox="0 0 24 24" className="size-3" fill="none" stroke="oklch(0.17 0.02 264)" strokeWidth={3.5}>
        <motion.path
          d="M20 6 9 17l-5-5"
          strokeLinecap="round"
          strokeLinejoin="round"
          initial={false}
          animate={{ pathLength: on ? 1 : 0, opacity: on ? 1 : 0 }}
          transition={{ duration: animate ? 0.35 : 0, delay: animate && on ? delay + 0.1 : 0, ease: easeOutQuint }}
        />
      </svg>
    </span>
  )
}

export function Evidence() {
  const reduce = useReducedMotion()
  const panel = useRef<HTMLDivElement>(null)
  const inView = useInView(panel, { amount: 0.4 })
  const seen = useInView(panel, { amount: 0.3, once: true })
  const [active, setActive] = useState<Group>("sure")
  const [touched, setTouched] = useState(false)

  // 보이는 동안 세 갈래를 차례로 보여 준다. 직접 고르면 멈춘다
  useEffect(() => {
    if (reduce || touched || !inView) return
    const order: Group[] = ["sure", "review", "protected"]
    const t = setInterval(() => setActive((g) => order[(order.indexOf(g) + 1) % order.length]), 3600)
    return () => clearInterval(t)
  }, [reduce, touched, inView])

  const pick = (g: Group) => {
    setTouched(true)
    setActive(g)
  }

  return (
    <section id="evidence" className="py-20 sm:py-28 lg:py-32">
      <div className="frame">
        <Reveal heading className="max-w-3xl">
          <h2 className="section-title">
            왜 그 프로그램 것인지
            <br />
            근거를 보여 줘요
          </h2>
          <p className="mt-5 max-w-[40ch] text-lead leading-snug text-muted-foreground">
            찾은 흔적마다 이유가 붙어요. 확실한 것만 미리 고르고, 애매한 건 직접 보고 정하게 남겨 둬요.
          </p>
        </Reveal>

        <div className="mt-12 grid items-start gap-10 sm:mt-14 lg:grid-cols-12 lg:gap-12">
          {/* 앱의 남은 흔적 창을 옮긴 조각 */}
          <Reveal className="lg:col-span-7">
            <div
              ref={panel}
              data-ui
              className="window overflow-hidden rounded-[1.25rem]"
              aria-label="남은 흔적 창 예시"
            >
              <div className="flex flex-wrap items-center gap-2 border-b px-4 py-3 text-[12px] sm:px-5">
                <span className="rounded-full bg-sure/12 px-2 py-0.5 font-medium text-sure">연결 정보 일치 3</span>
                <span className="rounded-full bg-review/12 px-2 py-0.5 font-medium text-review">직접 확인 필요 4</span>
                <span
                  className={cn(
                    "ml-auto inline-flex items-center gap-1 rounded-full px-2 py-0.5 transition-colors duration-300",
                    active === "protected" ? "bg-white/[0.1] text-foreground" : "text-muted-foreground",
                  )}
                >
                  <ShieldCheck className="size-3.5 text-sure" />
                  보호 규칙으로 3개 제외
                </span>
              </div>
              <ul>
                {rows.map((r, i) => {
                  const lit = active === r.group
                  return (
                    <motion.li
                      key={r.path + (r.value ?? "")}
                      initial={reduce ? false : { opacity: 0, x: -8 }}
                      animate={seen ? { opacity: lit ? 1 : 0.5, x: 0 } : undefined}
                      transition={{ duration: 0.45, delay: seen && !touched ? 0.06 * i : 0, ease: easeOutQuint }}
                      className="flex gap-3 border-b border-white/[0.06] px-4 py-3 last:border-b-0 sm:px-5"
                    >
                      <Check on={r.group === "sure" && seen} delay={0.25 + 0.18 * i} animate={!reduce} />
                      <r.icon className="mt-px size-4 shrink-0 text-muted-foreground" aria-hidden />
                      <div className="min-w-0 flex-1">
                        <p className="text-[12px] leading-5 [overflow-wrap:anywhere]">
                          <code>
                            {/* 좁은 화면에서 낱말 중간이 아니라 폴더 구분자 뒤에서 줄이 바뀌게 */}
                            {r.path.split("\\").map((part, j) => (
                              <Fragment key={j}>
                                {j > 0 && (
                                  <>
                                    {"\\"}
                                    <wbr />
                                  </>
                                )}
                                {part}
                              </Fragment>
                            ))}
                          </code>
                          {r.value && (
                            <span className="ml-1.5 rounded bg-white/[0.08] px-1.5 py-0.5 font-sans text-[11px]">
                              값: {r.value}
                            </span>
                          )}
                        </p>
                        <span className="mt-1.5 inline-flex rounded-md border px-1.5 py-0.5 text-[11px] text-muted-foreground">
                          {r.reason}
                        </span>
                      </div>
                      {r.size && (
                        <span className="shrink-0 text-[12px] text-muted-foreground tabular-nums">{r.size}</span>
                      )}
                    </motion.li>
                  )
                })}
              </ul>
              <div className="flex items-center gap-3 border-t px-4 py-3 text-[12px] text-muted-foreground sm:px-5">
                3개 선택 · 204 MB
                <span className="ml-auto rounded-md bg-destructive/15 px-2.5 py-1 font-medium text-destructive">
                  선택 항목 정리
                </span>
              </div>
            </div>
            <p className="mt-3 text-fine text-muted-foreground">
              앱의 남은 흔적 창을 옮긴 예시예요. 설명을 고르면 그 갈래의 항목이 또렷해져요.
            </p>
          </Reveal>

          <div className="flex flex-col gap-2 lg:col-span-5" role="group" aria-label="근거 갈래">
            {groups.map((g, i) => {
              const on = active === g.key
              return (
                <Reveal key={g.key} delay={0.05 * i}>
                  <button
                    type="button"
                    aria-pressed={on}
                    onClick={() => pick(g.key)}
                    onMouseEnter={() => pick(g.key)}
                    className={cn(
                      "relative w-full rounded-[1.25rem] border p-5 text-left transition-colors duration-300 focus-visible:ring-2 focus-visible:ring-ring focus-visible:outline-none sm:p-6",
                      on ? "border-white/15 bg-white/[0.05]" : "border-transparent hover:bg-white/[0.025]",
                    )}
                  >
                    <span data-ui className={`inline-flex rounded-full px-2.5 py-1 text-fine font-medium ${g.chip}`}>
                      {g.label}
                    </span>
                    <span
                      className={cn(
                        "mt-3 flex items-center gap-2 text-lead leading-snug font-semibold tracking-tight",
                        g.tone,
                      )}
                    >
                      <g.icon className="size-5 shrink-0" aria-hidden />
                      {g.title}
                    </span>
                    <span className="mt-2 block text-body text-muted-foreground">{g.text}</span>
                  </button>
                </Reveal>
              )
            })}
            <Reveal>
              <p className="mt-4 px-1 text-body font-medium">
                바탕 화면·문서·다운로드 안의 파일은 이유가 있어도 미리 고르지 않아요. 고른 것도 지우기 전에 백업해요.
              </p>
            </Reveal>
          </div>
        </div>
      </div>
    </section>
  )
}
