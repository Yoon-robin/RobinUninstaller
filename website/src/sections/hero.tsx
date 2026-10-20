import { ArrowDown, ArrowRight } from "lucide-react"
import { motion, useReducedMotion } from "motion/react"

import { RELEASE, shot } from "@/lib/site"
import { easeOutQuint } from "@/components/motion"
import { DownloadButton, DownloadFacts } from "@/components/download"

const rise = (delay: number, reduce: boolean | null) => ({
  initial: reduce ? false : { opacity: 0, y: 16 },
  animate: { opacity: 1, y: 0 },
  transition: { duration: 0.6, delay, ease: easeOutQuint },
})

/**
 * 첫 화면: 어두운 남색 위에 위쪽에서 퍼지는 파란 빛과 사라지는 격자, 가운데 제목, 그 아래 앱 창.
 * 순서는 무슨 앱인지(제목·한 문장) → 받기(단추와 받기 전에 알아야 할 것) → 증거(실제 앱 화면).
 * 그림은 이 앱만의 장면인 "남은 흔적" 창이다. 가짜 응답으로 찍어 실제 PC의 프로그램 이름은 나오지 않는다
 * (scripts/capture.mjs). 움직임은 이 구역에만 준다.
 */
export function Hero() {
  const reduce = useReducedMotion()
  return (
    <section id="top" className="relative overflow-hidden">
      <div aria-hidden className="glow-top pointer-events-none absolute inset-0" />
      <div aria-hidden className="grid-lines pointer-events-none absolute inset-x-0 top-0 h-[44rem]" />

      <div className="frame relative flex flex-col items-center pt-32 text-center sm:pt-40">
        <motion.a
          {...rise(0, reduce)}
          href={RELEASE.pageUrl}
          target="_blank"
          rel="noreferrer"
          className="group inline-flex items-center gap-2 rounded-full border bg-white/[0.04] py-1 pr-3 pl-1 text-fine text-foreground/85 transition-colors hover:bg-white/[0.07]"
        >
          <span className="rounded-full bg-primary px-2 py-0.5 font-semibold text-primary-foreground">새 버전</span>v
          {RELEASE.version} 공개 · 릴리스 노트
          <ArrowRight className="size-3.5 transition-transform group-hover:translate-x-0.5" />
        </motion.a>
        <motion.h1 {...rise(0.05, reduce)} className="display mt-7">
          <span className="text-stage">남은 흔적</span>까지
          <br />
          정리해요.
        </motion.h1>
        <motion.p
          {...rise(0.1, reduce)}
          className="mt-6 max-w-[30ch] text-lead leading-snug text-balance text-muted-foreground"
        >
          프로그램을 지운 뒤 남은 파일과 레지스트리를 찾아, 근거를 보고 고른 것만 백업한 뒤 정리하는 무료 Windows
          앱이에요.
        </motion.p>
        <motion.div {...rise(0.16, reduce)} className="mt-9 flex flex-wrap items-center justify-center gap-3">
          <DownloadButton />
          <a
            href="#walkthrough"
            className="group inline-flex h-12 items-center gap-1.5 rounded-full border bg-white/[0.03] px-6 text-body font-medium transition-colors hover:bg-white/[0.07] focus-visible:ring-2 focus-visible:ring-ring focus-visible:outline-none"
          >
            작동 장면 보기
            <ArrowDown className="size-4 transition-transform group-hover:translate-y-0.5" />
          </a>
        </motion.div>
        <motion.div {...rise(0.2, reduce)}>
          <DownloadFacts className="mt-5 max-w-[46ch] text-balance text-muted-foreground" linkClassName="text-stage" />
        </motion.div>
      </div>

      {/* 실제 앱 화면을 창 틀에 담아 보여 준다. 아래쪽은 다음 구역으로 이어지도록 잘라 둔다 */}
      <motion.figure
        initial={reduce ? false : { opacity: 0, y: 32 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ duration: 0.8, delay: 0.24, ease: easeOutQuint }}
        className="frame relative mt-14 sm:mt-20"
      >
        <div className="window mx-auto max-w-[68rem] overflow-hidden rounded-[1.25rem] p-2 sm:rounded-[1.6rem] sm:p-3">
          {/* 좁은 화면에서는 창 전체가 너무 작아 글자가 안 보이므로 "남은 흔적" 창 부분만 잘라 크게 보여 준다 */}
          <div className="overflow-hidden rounded-[0.8rem] max-sm:aspect-[766/560] sm:rounded-[1.05rem]">
            <img
              src={shot("leftovers")}
              alt="RobinX의 남은 흔적 창: 설치 정보와 연결된 흔적 3개는 미리 골라 두었고, 직접 확인이 필요한 항목 4개는 고르지 않은 상태예요. 보호 규칙으로 3개를 뺐어요."
              width={1280}
              height={800}
              fetchPriority="high"
              className="block w-full max-sm:mt-[-6.3%] max-sm:ml-[-33.6%] max-sm:w-[167.1%] max-sm:max-w-none"
            />
          </div>
        </div>
        <figcaption className="mx-auto mt-5 max-w-[68rem] text-center text-fine text-muted-foreground">
          실제 앱 화면이에요. 프로그램 이름·경로·크기는 설명을 위한 예시예요.
        </figcaption>
      </motion.figure>
    </section>
  )
}
