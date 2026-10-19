import { ArrowRight } from "lucide-react"
import { motion, useReducedMotion } from "motion/react"

import { REPO_URL } from "@/lib/site"
import { GithubIcon } from "@/components/github-icon"
import { easeOutQuint } from "@/components/motion"
import { LeftoverDemo } from "@/sections/leftover-demo"

const rise = (delay: number, reduce: boolean | null) => ({
  initial: reduce ? false : { opacity: 0, y: 24 },
  animate: { opacity: 1, y: 0 },
  transition: { duration: 0.8, delay, ease: easeOutQuint },
})

/**
 * 첫 화면: 왼쪽은 제목·한 문장·실행 하나, 오른쪽은 밝은 청록 무대 위의 실제로 움직이는 "남은 흔적" 화면 조각.
 * 무대는 오른쪽과 아래 가장자리로 넘친다.
 */
export function Hero() {
  const reduce = useReducedMotion()
  return (
    <section id="top" className="relative overflow-hidden">
      <div className="frame grid grid-cols-[minmax(0,1fr)] items-center gap-12 pt-10 sm:pt-16 lg:min-h-[calc(100svh-5rem)] lg:grid-cols-12 lg:gap-8 lg:pt-0">
        <div className="lg:col-span-5 lg:pb-24">
          <motion.h1 {...rise(0.05, reduce)} className="display">
            지우기 전에,
            <br />
            근거부터.
          </motion.h1>
          <motion.p {...rise(0.15, reduce)} className="mt-8 max-w-[34ch] text-lead leading-snug">
            RobinX는 프로그램을 지운 뒤 남은 파일과 레지스트리를 찾아, 근거를 보여 주고 백업한 다음 정리하는 무료
            Windows 도구예요.
          </motion.p>
          <motion.div {...rise(0.25, reduce)} className="mt-10 flex flex-wrap items-center gap-x-6 gap-y-4">
            <a
              href={REPO_URL}
              target="_blank"
              rel="noreferrer"
              className="inline-flex h-12 items-center gap-2.5 rounded-full bg-primary px-6 text-body font-semibold text-primary-foreground transition-[transform,background-color] hover:bg-primary/90 focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 focus-visible:outline-none active:scale-[0.97]"
            >
              <GithubIcon className="size-[1.1rem]" />
              GitHub에서 보기
            </a>
            <a
              href="#status"
              className="group inline-flex items-center gap-1.5 text-body font-medium text-primary hover:underline hover:underline-offset-4"
            >
              설치 파일은 언제 나오나요
              <ArrowRight className="size-4 transition-transform group-hover:translate-x-0.5" />
            </a>
          </motion.div>
        </div>

        {/* 무대: 오른쪽·아래로 넘친다. 좁은 화면에서는 양옆으로 넘친다 */}
        <motion.div
          initial={reduce ? false : { opacity: 0, y: 48 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 1, delay: 0.3, ease: easeOutQuint }}
          className="-mx-[clamp(1.25rem,4vw,2.5rem)] min-w-0 lg:col-span-7 lg:mx-0 lg:mr-[calc(-1*(max(0px,(100vw-80rem)/2)+clamp(1.25rem,4vw,2.5rem)))] lg:self-end"
        >
          <div className="overflow-hidden bg-stage pt-8 pl-5 sm:pt-14 sm:pl-14 lg:rounded-tl-[1.75rem]">
            <LeftoverDemo />
          </div>
        </motion.div>
      </div>
    </section>
  )
}
