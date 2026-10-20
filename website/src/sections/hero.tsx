import { ArrowRight, Download } from "lucide-react"
import { motion, useReducedMotion } from "motion/react"

import { DOWNLOAD_URL, LOGO, REPO_URL, shot } from "@/lib/site"
import { GithubIcon } from "@/components/github-icon"
import { easeOutQuint } from "@/components/motion"

const rise = (delay: number, reduce: boolean | null) => ({
  initial: reduce ? false : { opacity: 0, y: 20 },
  animate: { opacity: 1, y: 0 },
  transition: { duration: 0.8, delay, ease: easeOutQuint },
})

/**
 * 첫 화면: Apple 제품 페이지처럼 가운데에 앱 아이콘·제목·한 문장·실행 둘, 그 아래 실제 앱 화면을 크게.
 * 앱 화면은 가짜 응답으로 찍은 것이라 실제 PC의 프로그램 이름은 나오지 않는다 (scripts/capture.mjs).
 */
export function Hero() {
  const reduce = useReducedMotion()
  return (
    <section id="top" className="overflow-hidden">
      <div className="frame flex flex-col items-center pt-14 text-center sm:pt-20 lg:pt-24">
        <motion.img
          {...rise(0, reduce)}
          src={LOGO}
          alt=""
          width={72}
          height={72}
          className="size-16 rounded-[1.1rem] sm:size-[4.5rem]"
        />
        <motion.h1 {...rise(0.06, reduce)} className="display mt-8">
          남은 흔적까지
          <br />
          정리해요.
        </motion.h1>
        <motion.p {...rise(0.14, reduce)} className="mt-6 max-w-[32ch] text-lead leading-snug text-balance text-muted-foreground">
          제거한 뒤에도 남는 파일과 레지스트리를 찾아, 백업해 두고 정리하는 무료 Windows 앱이에요.
        </motion.p>
        <motion.div
          {...rise(0.22, reduce)}
          className="mt-9 flex flex-wrap items-center justify-center gap-x-7 gap-y-4"
        >
          <a
            href={DOWNLOAD_URL}
            target="_blank"
            rel="noreferrer"
            className="inline-flex h-12 items-center gap-2.5 rounded-full bg-primary px-7 text-body font-semibold text-primary-foreground transition-[transform,background-color] hover:bg-primary/90 focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 focus-visible:outline-none active:scale-[0.97]"
          >
            <Download className="size-[1.1rem]" />
            내려받기 (미리보기)
          </a>
          <a
            href={REPO_URL}
            target="_blank"
            rel="noreferrer"
            className="group inline-flex items-center gap-1.5 text-body font-medium text-primary hover:underline hover:underline-offset-4"
          >
            <GithubIcon className="size-4" />
            소스 보기
            <ArrowRight className="size-4 transition-transform group-hover:translate-x-0.5" />
          </a>
        </motion.div>
        <motion.p {...rise(0.28, reduce)} className="mt-4 text-fine text-muted-foreground">
          Windows용 · 무료 · 소스 공개
        </motion.p>
      </div>

      {/* 실제 앱 화면. 아래쪽은 다음 구역으로 이어지도록 잘라 둔다 */}
      <motion.div
        initial={reduce ? false : { opacity: 0, y: 40 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ duration: 1, delay: 0.32, ease: easeOutQuint }}
        className="frame mt-14 sm:mt-20"
      >
        <div className="mx-auto max-w-[68rem] overflow-hidden rounded-t-[1.5rem] bg-muted px-3 pt-3 sm:rounded-t-[2rem] sm:px-6 sm:pt-6">
          <img
            src={shot("programs")}
            alt="RobinX의 설치된 프로그램 화면: 프로그램 수, 전체 용량, 언인스톨러가 없는 프로그램 수와 목록"
            width={1280}
            height={800}
            className="block w-full rounded-t-[0.9rem] sm:rounded-t-[1.1rem]"
          />
        </div>
      </motion.div>
    </section>
  )
}
