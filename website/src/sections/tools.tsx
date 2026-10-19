import { useEffect, useId, useMemo, useState } from "react"
import { AnimatePresence, motion, useReducedMotion } from "motion/react"
import { Search } from "lucide-react"

import { cn } from "@/lib/utils"
import { shot } from "@/lib/site"
import { Reveal, easeOutQuint } from "@/components/motion"

/** 앱의 도구. 스크린샷이 있는 것은 그림을, 없는 것은 한 문장을 크게 보여 준다. */
const tools: { name: string; text: string; shot?: string; alt?: string }[] = [
  {
    name: "프로그램 제거",
    text: "EXE·MSI 프로그램과 Store 앱을 한 목록에서 보고, 원래 언인스톨러로 제거해요. 여러 개를 골라 차례로 지울 수도 있어요.",
    shot: "programs",
    alt: "설치된 프로그램 목록 화면",
  },
  {
    name: "남은 흔적 정리",
    text: "확실한 흔적은 미리 고르고, 애매한 것은 검토하도록 나눠요. 고른 것만 백업한 뒤 치워요.",
    shot: "leftovers",
    alt: "남은 흔적 화면",
  },
  {
    name: "복구 기록",
    text: "정리한 작업마다 백업이 남아요. 항목별로, 혹은 한 번에 원래 자리로 되돌려요.",
    shot: "restore",
    alt: "복구 기록 화면",
  },
  {
    name: "시작프로그램",
    text: "로그인할 때 켜지는 프로그램과 예약 작업을 끄고 켜요. 등록 정보는 지우지 않아요.",
    shot: "startup",
    alt: "시작프로그램 화면",
  },
  {
    name: "저장공간 정리",
    text: "다시 만들어지는 캐시와 임시 파일만 골라요. 되돌릴 수 없는 항목은 따로 경고해요.",
    shot: "storage",
    alt: "저장공간 정리 화면",
  },
  {
    name: "강제 제거",
    text: "언인스톨러가 사라졌거나 망가진 프로그램도 설치 정보와 흔적을 찾아 백업 후 정리해요. 보안 프로그램과 드라이버는 빼요.",
  },
  {
    name: "설치 추적",
    text: "설치 전후를 비교해 새로 생긴 파일과 레지스트리를 기록해요. 나중에 지울 때 강한 근거가 돼요.",
  },
]

export function Tools() {
  const reduce = useReducedMotion()
  const [query, setQuery] = useState("")
  const [active, setActive] = useState(tools[0].name)
  const listId = useId()

  const shown = useMemo(() => {
    const q = query.trim()
    return q ? tools.filter((t) => t.name.includes(q) || t.text.includes(q)) : tools
  }, [query])

  // 검색으로 고른 항목이 목록에서 빠지면 남은 첫 항목을 보여 준다
  const current = shown.find((t) => t.name === active) ?? shown[0] ?? tools[0]

  // 다른 화면을 미리 받아 두어 고를 때 기다리지 않게 한다 (첫 화면 로딩이 끝난 뒤에)
  useEffect(() => {
    const t = setTimeout(() => {
      for (const tool of tools) if (tool.shot) new Image().src = shot(tool.shot)
    }, 2500)
    return () => clearTimeout(t)
  }, [])

  const onKey = (e: React.KeyboardEvent) => {
    if (e.key !== "ArrowDown" && e.key !== "ArrowUp") return
    e.preventDefault()
    const i = shown.findIndex((t) => t.name === current.name)
    const next = shown[(i + (e.key === "ArrowDown" ? 1 : -1) + shown.length) % shown.length]
    if (next) setActive(next.name)
  }

  return (
    <section id="tools" className="mt-20 bg-night py-20 text-night-foreground sm:mt-28 sm:py-24 lg:mt-32 lg:py-28">
      <div className="frame">
        <Reveal>
          <h2 className="section-title max-w-[16ch]">RobinX로 할 수 있는 일</h2>
        </Reveal>

        <div className="mt-14 grid gap-12 lg:grid-cols-12 lg:gap-8">
          {/* 명령 목록: 검색하면 줄어들고, 방향키와 마우스로 고른다 */}
          <div data-ui className="lg:col-span-6" onKeyDown={onKey}>
            <Reveal delay={0.08}>
              <label className="flex h-14 items-center gap-3 rounded-full bg-white/8 px-5 focus-within:ring-2 focus-within:ring-stage">
                <Search className="size-5 text-night-muted" />
                <span className="sr-only">도구 찾기</span>
                <input
                  value={query}
                  onChange={(e) => setQuery(e.target.value)}
                  placeholder="도구 찾기"
                  aria-controls={listId}
                  className="h-full w-full bg-transparent text-lead text-night-foreground outline-none placeholder:text-night-muted"
                />
              </label>
            </Reveal>
            <ul id={listId} className="mt-8 flex flex-col" aria-label="도구">
              {shown.map((t, i) => {
                const on = t.name === current.name
                return (
                  <Reveal as="li" key={t.name} delay={0.12 + i * 0.04} y={16}>
                    <button
                      type="button"
                      aria-pressed={on}
                      onClick={() => setActive(t.name)}
                      onMouseEnter={() => setActive(t.name)}
                      onFocus={() => setActive(t.name)}
                      className={cn(
                        "w-full rounded-[1.25rem] px-5 py-2 text-left transition-colors focus-visible:ring-2 focus-visible:ring-stage focus-visible:outline-none",
                        on ? "bg-stage text-[oklch(0.2_0.04_200)]" : "hover:bg-white/6",
                      )}
                    >
                      <span className="block text-section leading-[1.15] font-semibold tracking-[-0.03em]">
                        {t.name}
                      </span>
                      {on && (
                        <span className="mt-1.5 block max-w-[46ch] text-body leading-snug lg:hidden">{t.text}</span>
                      )}
                      {/* 좁은 화면: 고른 도구의 화면을 바로 아래에 */}
                      {on && t.shot && (
                        <img
                          src={shot(t.shot)}
                          alt={t.alt}
                          width={1280}
                          height={800}
                          loading="lazy"
                          className="mt-4 mb-2 block w-full rounded-[0.75rem] lg:hidden"
                        />
                      )}
                    </button>
                  </Reveal>
                )
              })}
              {shown.length === 0 && <li className="px-5 py-3.5 text-lead text-night-muted">찾는 도구가 없어요</li>}
            </ul>
          </div>

          {/* 고른 도구의 화면. 넓은 화면에서만 옆에 고정해 보여 준다 */}
          <div className="hidden lg:col-span-6 lg:block">
            <div className="sticky top-10">
              <div className="relative aspect-[1280/800] overflow-hidden rounded-[1.25rem] bg-white/6">
                <AnimatePresence initial={false} mode="popLayout">
                  <motion.div
                    key={current.name}
                    initial={reduce ? { opacity: 0 } : { opacity: 0, scale: 1.04 }}
                    animate={{ opacity: 1, scale: 1 }}
                    exit={{ opacity: 0 }}
                    transition={{ duration: 0.28, ease: easeOutQuint }}
                    className="absolute inset-0"
                  >
                    {current.shot ? (
                      <img
                        src={shot(current.shot)}
                        alt={current.alt}
                        width={1280}
                        height={800}
                        className="size-full object-cover"
                      />
                    ) : (
                      <div className="flex size-full items-end p-10">
                        <p className="text-lead text-night-muted">이 도구의 화면은 정리하는 중이에요.</p>
                      </div>
                    )}
                  </motion.div>
                </AnimatePresence>
              </div>
              <p className="mt-6 max-w-[46ch] text-lead leading-snug" aria-live="polite">
                {current.text}
              </p>
            </div>
          </div>
        </div>
      </div>
    </section>
  )
}
