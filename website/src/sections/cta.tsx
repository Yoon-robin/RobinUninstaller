import { m, useReducedMotion } from "motion/react"

import { LOGO, REPO_URL } from "@/lib/site"
import { easeOutQuint, Reveal } from "@/components/motion"
import { DownloadButton, DownloadFacts } from "@/components/download"
import { GithubIcon } from "@/components/github-icon"

/**
 * 마지막 구역: 아래에서 떠오르는 빛 위에 내려받기와 소스 보기, 받기 전에 알아야 할 것.
 * 구역이 보이면 빛이 지평선 아래에서 천천히 올라온다 (움직임 줄이기면 처음부터 떠 있다).
 */
export function Cta() {
  const reduce = useReducedMotion()
  return (
    <section className="relative overflow-hidden py-24 sm:py-32">
      <m.div
        aria-hidden
        className="horizon pointer-events-none absolute inset-0"
        initial={reduce ? false : { opacity: 0, y: 80 }}
        whileInView={{ opacity: 1, y: 0 }}
        viewport={{ once: true, margin: "-80px" }}
        transition={{ duration: 1.4, ease: easeOutQuint }}
      />
      <Reveal className="frame relative flex flex-col items-center text-center">
        <img
          src={LOGO}
          alt=""
          width={64}
          height={64}
          className="size-14 rounded-[1rem] shadow-[0_0_48px_oklch(0.6_0.17_252/45%)] sm:size-16"
        />
        <h2 className="section-title mt-7">이제 직접 써 보세요</h2>
        <p className="mt-5 max-w-[32ch] text-lead leading-snug text-balance text-muted-foreground">
          무료이고, 무엇을 어떻게 지우는지 소스 코드로 확인할 수 있어요.
        </p>
        <div className="mt-9 flex flex-wrap items-center justify-center gap-3">
          <DownloadButton />
          <a
            href={REPO_URL}
            target="_blank"
            rel="noreferrer"
            className="inline-flex h-12 items-center gap-2 rounded-full border bg-white/[0.03] px-6 text-body font-medium transition-colors hover:bg-white/[0.07] focus-visible:ring-2 focus-visible:ring-ring focus-visible:outline-none"
          >
            <GithubIcon className="size-4" />
            소스 보기
          </a>
        </div>
        <DownloadFacts className="mt-5 max-w-[46ch] text-balance text-muted-foreground" linkClassName="text-stage" />
      </Reveal>
    </section>
  )
}
