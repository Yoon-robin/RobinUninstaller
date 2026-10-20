import { useReducedMotion } from "motion/react"

import { demo } from "@/lib/site"
import { Reveal } from "@/components/motion"

/**
 * 앱에서 남은 흔적을 찾아 정리하는 짧은 녹화. 가짜 응답으로 찍어 실제 PC의 목록은 나오지 않는다
 * (scripts/ui-mock/record-demo.mjs). 움직임 줄이기를 켠 사람에게는 자동으로 재생하지 않는다.
 */
export function Walkthrough() {
  const reduce = useReducedMotion()
  return (
    <section id="walkthrough" className="frame grid gap-14 pt-20 sm:pt-28 lg:grid-cols-12 lg:gap-8 lg:pt-32">
      <div className="lg:col-span-5">
        <Reveal>
          <h2 className="section-title">
            제거한 뒤
            <br />
            정리하는 장면
          </h2>
        </Reveal>
        <Reveal delay={0.08}>
          <p className="mt-8 max-w-[36ch] text-lead leading-snug">
            방금 지운 프로그램의 남은 흔적을 찾고, 미리 골라 둔 것만 백업한 뒤 정리해요. 12초쯤 되는 실제 앱 화면이에요.
          </p>
        </Reveal>
        <Reveal delay={0.12}>
          <p className="mt-6 max-w-[40ch] text-body text-muted-foreground">
            화면 속 프로그램(Fixture Notes)과 경로, 크기는 시험용 예시예요. 소리는 없어요.
          </p>
        </Reveal>
      </div>

      <Reveal delay={0.16} className="lg:col-span-7 lg:pt-4">
        <video
          src={demo("leftovers.webm")}
          poster={demo("leftovers-poster.jpg")}
          width={1280}
          height={800}
          muted
          loop
          playsInline
          autoPlay={!reduce}
          controls={!!reduce}
          preload="metadata"
          aria-label="남은 흔적 찾기 → 찾은 항목 확인 → 백업하고 정리 → 5개를 정리했어요 화면으로 이어지는 앱 녹화"
          className="w-full rounded-[1.25rem] bg-night"
        />
      </Reveal>
    </section>
  )
}
