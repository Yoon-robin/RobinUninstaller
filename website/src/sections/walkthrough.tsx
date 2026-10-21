import { useEffect, useRef } from "react"
import { useReducedMotion } from "motion/react"

import { demo } from "@/lib/site"
import { Reveal } from "@/components/motion"

const steps = ["남은 흔적 찾기", "찾은 항목 확인", "백업하고 정리"]

/**
 * 앱에서 남은 흔적을 찾아 정리하는 짧은 녹화. 가짜 응답으로 찍어 실제 PC의 목록은 나오지 않는다
 * (scripts/ui-mock/record-demo.mjs).
 * - 재생·일시정지 단추는 늘 보인다 (키보드·터치로도 멈출 수 있게).
 * - 화면에 보일 때만 재생하고, 벗어나면 멈춘다. 움직임 줄이기를 켠 사람에게는 저절로 재생하지 않는다.
 * - 글자가 중요한 앱 화면이라 잘리지 않게 전체를 보여 준다 (object-contain).
 */
export function Walkthrough() {
  const reduce = useReducedMotion()
  const video = useRef<HTMLVideoElement>(null)

  useEffect(() => {
    const el = video.current
    if (!el || reduce) return
    // 사용자가 직접 멈췄으면 다시 보일 때도 저절로 틀지 않는다
    let pausedByUser = false
    const onPause = () => {
      if (el.dataset.auto !== "1") pausedByUser = true
      delete el.dataset.auto
    }
    const onPlay = () => {
      pausedByUser = false
    }
    el.addEventListener("pause", onPause)
    el.addEventListener("play", onPlay)
    const io = new IntersectionObserver(
      ([entry]) => {
        if (entry.isIntersecting) {
          if (!pausedByUser) el.play().catch(() => {})
        } else if (!el.paused) {
          el.dataset.auto = "1"
          el.pause()
        }
      },
      { threshold: 0.4 },
    )
    io.observe(el)
    return () => {
      io.disconnect()
      el.removeEventListener("pause", onPause)
      el.removeEventListener("play", onPlay)
    }
  }, [reduce])

  return (
    <section id="walkthrough" className="border-y bg-white/[0.02] py-20 sm:py-24 lg:py-28">
      <div className="frame">
        <Reveal heading className="mx-auto max-w-3xl text-center">
          <h2 className="section-title">정리하는 모습</h2>
          <p className="mx-auto mt-5 max-w-[34ch] text-lead leading-snug text-balance text-muted-foreground">
            방금 지운 프로그램의 남은 흔적을 찾고, 미리 골라 둔 것만 백업한 다음 정리해요.
          </p>
        </Reveal>

        <Reveal className="mx-auto mt-12 max-w-[68rem] sm:mt-14">
          <video
            ref={video}
            src={demo("leftovers.webm")}
            poster={demo("leftovers-poster.jpg")}
            width={1280}
            height={800}
            muted
            loop
            playsInline
            controls
            preload="metadata"
            aria-label="RobinX 시연: 남은 흔적 찾기, 찾은 항목 확인, 백업하고 정리, 정리 결과 화면"
            className="window aspect-[16/10] w-full rounded-[1.5rem] object-contain p-2 sm:p-3"
          />
          <ol className="mt-6 flex flex-col items-center justify-center gap-x-3 gap-y-2 text-body font-medium sm:flex-row sm:flex-wrap">
            {steps.map((s, i) => (
              <li key={s} className="flex items-center gap-3">
                {i > 0 && (
                  <span aria-hidden className="hidden text-muted-foreground sm:inline">
                    →
                  </span>
                )}
                <span>
                  <span className="mr-1.5 text-stage tabular-nums">{i + 1}</span>
                  {s}
                </span>
              </li>
            ))}
          </ol>
          <p className="mt-3 text-center text-fine text-muted-foreground">
            소리 없는 실제 앱 화면이에요. 화면 속 프로그램(Fixture Notes)과 경로, 크기는 시험용 예시예요.
          </p>
        </Reveal>
      </div>
    </section>
  )
}
