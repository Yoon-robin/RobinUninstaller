import { useReducedMotion } from "motion/react"

import { demo } from "@/lib/site"
import { Reveal } from "@/components/motion"

/**
 * 앱에서 남은 흔적을 찾아 정리하는 짧은 녹화. 가짜 응답으로 찍어 실제 PC의 목록은 나오지 않는다
 * (scripts/ui-mock/record-demo.mjs). 움직임 줄이기를 켠 사람에게는 자동으로 재생하지 않는다.
 * 앞뒤 구역이 모두 "왼쪽 글, 오른쪽 그림"이라 여기서는 제목을 위에 두고 영상을 화면 폭 전체로 보여 준다.
 */
export function Walkthrough() {
  const reduce = useReducedMotion()
  return (
    <section id="walkthrough" className="frame pt-20 sm:pt-28 lg:pt-32">
      <div className="grid gap-8 lg:grid-cols-12">
        <Reveal className="lg:col-span-6">
          <h2 className="section-title">
            제거한 뒤
            <br />
            정리하는 장면
          </h2>
        </Reveal>
        <Reveal delay={0.08} className="lg:col-span-6 lg:pt-3">
          <p className="max-w-[40ch] text-lead leading-snug">
            방금 지운 프로그램의 남은 흔적을 찾고, 미리 골라 둔 것만 백업한 뒤 정리해요.
          </p>
          <p className="mt-4 max-w-[46ch] text-body text-muted-foreground">
            12초쯤 되는 실제 앱 화면이에요. 화면 속 프로그램(Fixture Notes)과 경로, 크기는 시험용 예시이고, 소리는
            없어요.
          </p>
        </Reveal>
      </div>

      <Reveal delay={0.12} className="mt-12 lg:mt-16">
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
          className="aspect-[16/10] w-full rounded-[1.75rem] bg-night object-cover"
        />
      </Reveal>
    </section>
  )
}
