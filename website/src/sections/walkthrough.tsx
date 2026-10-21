import { useEffect, useRef, useState } from "react"
import { useReducedMotion } from "motion/react"
import { Pause, Play } from "lucide-react"

import { demo } from "@/lib/site"
import { Reveal } from "@/components/motion"

const steps = ["남은 흔적 찾기", "찾은 항목 확인", "백업하고 정리"]

/**
 * 앱에서 남은 흔적을 찾아 정리하는 짧은 녹화. 가짜 응답으로 찍어 실제 PC의 목록은 나오지 않는다
 * (scripts/ui-mock/record-demo.mjs).
 * - 재생·일시정지 단추는 재생 중에도 늘 보인다 (브라우저 기본 단추는 재생하면 숨어서 따로 둔다).
 * - 화면에 보일 때만 재생하고, 벗어나면 멈춘다. 움직임 줄이기를 켠 사람에게는 저절로 재생하지 않는다.
 * - 글자가 중요한 앱 화면이라 잘리지 않게 전체를 보여 준다 (object-contain).
 * - 좁은 화면에서는 창 전체를 줄이면 글자가 4~5px이 되어, 대화창 부분만 찍은 영상을 쓴다.
 */
const NARROW_QUERY = "(max-width: 639px)"
// 2배로 찍은 영상은 3배 무거워서, 고해상도 화면에만 준다 (휴대폰용 영상은 늘 2배)
const wideVideo = () => demo(devicePixelRatio >= 1.5 ? "leftovers@2x.webm" : "leftovers.webm")

function useNarrow() {
  const [narrow, setNarrow] = useState(() => matchMedia(NARROW_QUERY).matches)
  useEffect(() => {
    const mq = matchMedia(NARROW_QUERY)
    const onChange = () => setNarrow(mq.matches)
    mq.addEventListener("change", onChange)
    return () => mq.removeEventListener("change", onChange)
  }, [])
  return narrow
}

export function Walkthrough() {
  const reduce = useReducedMotion()
  const narrow = useNarrow()
  const video = useRef<HTMLVideoElement>(null)
  const [playing, setPlaying] = useState(false)

  useEffect(() => {
    const el = video.current
    if (!el) return
    const sync = () => setPlaying(!el.paused)
    sync()
    el.addEventListener("play", sync)
    el.addEventListener("pause", sync)
    return () => {
      el.removeEventListener("play", sync)
      el.removeEventListener("pause", sync)
    }
  }, [narrow])

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
  }, [reduce, narrow])

  const toggle = () => {
    const el = video.current
    if (!el) return
    if (el.paused) el.play().catch(() => {})
    else el.pause()
  }

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
          <div className="relative">
            <video
              // 폭이 바뀌어 다른 영상으로 갈아 끼울 때 처음부터 다시 읽게 한다
              key={narrow ? "narrow" : "wide"}
              ref={video}
              src={narrow ? demo("leftovers-narrow.webm") : wideVideo()}
              poster={demo(narrow ? "leftovers-narrow-poster.jpg" : "leftovers-poster.jpg")}
              width={narrow ? 784 : 1280}
              height={narrow ? 736 : 800}
              muted
              loop
              playsInline
              preload="metadata"
              onClick={toggle}
              aria-label="RobinX 시연: 남은 흔적 찾기, 찾은 항목 확인, 백업하고 정리, 정리 결과 화면"
              className={
                "window w-full cursor-pointer object-contain p-2 sm:p-3 " +
                (narrow ? "aspect-[784/736]" : "aspect-[16/10]")
              }
            />
            <button
              type="button"
              onClick={toggle}
              aria-label={playing ? "시연 영상 일시정지" : "시연 영상 재생"}
              className="absolute right-4 bottom-4 inline-flex size-11 items-center justify-center rounded-full border border-white/15 bg-background/85 text-foreground transition-colors hover:bg-background focus-visible:ring-2 focus-visible:ring-ring focus-visible:outline-none sm:right-6 sm:bottom-6"
            >
              {playing ? <Pause className="size-4" /> : <Play className="size-4 translate-x-px" />}
            </button>
          </div>
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
