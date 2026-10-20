import { ArrowDown } from "lucide-react"
import { motion, useReducedMotion } from "motion/react"

import { LOGO, shot } from "@/lib/site"
import { easeOutQuint } from "@/components/motion"
import { DownloadButton, DownloadFacts } from "@/components/download"

const rise = (delay: number, reduce: boolean | null) => ({
  initial: reduce ? false : { opacity: 0, y: 16 },
  animate: { opacity: 1, y: 0 },
  transition: { duration: 0.6, delay, ease: easeOutQuint },
})

/**
 * 첫 화면: 무슨 앱인지(제목·한 문장) → 받기(단추와 받기 전에 알아야 할 것) → 증거(실제 앱 화면).
 * 그림은 이 앱만의 장면인 "남은 흔적" 창이다: 연결 정보가 맞는 것은 미리 골라 두고, 이름만 비슷한 것은 직접 고르게
 * 나눠 둔 모습. 가짜 응답으로 찍어 실제 PC의 프로그램 이름은 나오지 않는다 (scripts/capture.mjs).
 * 움직임은 이 구역에만 준다.
 */
export function Hero() {
  const reduce = useReducedMotion()
  return (
    <section id="top" className="overflow-hidden">
      <div className="frame flex flex-col items-center pt-12 text-center sm:pt-16 lg:pt-20">
        <motion.img
          {...rise(0, reduce)}
          src={LOGO}
          alt=""
          width={72}
          height={72}
          className="size-16 rounded-[1.1rem] sm:size-[4.5rem]"
        />
        <motion.h1 {...rise(0.05, reduce)} className="display mt-7">
          남은 흔적까지
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
        <motion.div {...rise(0.16, reduce)} className="mt-9 flex flex-wrap items-center justify-center gap-x-7 gap-y-4">
          <DownloadButton />
          <a
            href="#walkthrough"
            className="group inline-flex h-12 items-center gap-1.5 text-body font-medium text-primary hover:underline hover:underline-offset-4"
          >
            작동 장면 보기
            <ArrowDown className="size-4 transition-transform group-hover:translate-y-0.5" />
          </a>
        </motion.div>
        <motion.div {...rise(0.2, reduce)}>
          <DownloadFacts
            className="mt-5 max-w-[46ch] text-balance text-muted-foreground"
            linkClassName="text-primary"
          />
        </motion.div>
      </div>

      {/* 실제 앱 화면. 아래쪽은 다음 구역으로 이어지도록 잘라 둔다 */}
      <motion.figure
        initial={reduce ? false : { opacity: 0, y: 32 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ duration: 0.8, delay: 0.24, ease: easeOutQuint }}
        className="frame mt-12 sm:mt-16"
      >
        <div className="mx-auto max-w-[68rem] overflow-hidden rounded-t-[1.5rem] bg-muted px-3 pt-3 sm:rounded-t-[2rem] sm:px-6 sm:pt-6">
          {/* 좁은 화면에서는 창 전체가 너무 작아 글자가 안 보이므로 "남은 흔적" 창 부분만 잘라 크게 보여 준다 */}
          <div className="overflow-hidden rounded-t-[0.9rem] max-sm:aspect-[766/560] sm:rounded-t-[1.1rem]">
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
        <figcaption className="mx-auto mt-4 max-w-[68rem] text-center text-fine text-muted-foreground">
          실제 앱 화면이에요. 프로그램 이름·경로·크기는 설명을 위한 예시예요.
        </figcaption>
      </motion.figure>
    </section>
  )
}
