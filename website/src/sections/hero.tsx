import { useRef } from "react"
import { motion, useReducedMotion, useScroll, useTransform } from "motion/react"
import { ArrowRight, Archive, ShieldCheck, Sparkles } from "lucide-react"

import { REPO_URL } from "@/lib/site"
import { Button } from "@/components/ui/button"
import { GithubIcon } from "@/components/github-icon"
import { easeOutQuint } from "@/components/motion"
import { WindowFrame, shot } from "@/components/window-frame"

const rise = (delay: number) => ({
  initial: { opacity: 0, y: 22, filter: "blur(6px)" },
  animate: { opacity: 1, y: 0, filter: "blur(0px)" },
  transition: { duration: 0.8, delay, ease: easeOutQuint },
})

export function Hero() {
  const ref = useRef<HTMLDivElement>(null)
  const reduce = useReducedMotion()
  const { scrollYProgress } = useScroll({ target: ref, offset: ["start start", "end start"] })
  // 스크롤하면 눕혀 있던 창이 똑바로 선다
  const rotateX = useTransform(scrollYProgress, [0, 0.45], reduce ? [0, 0] : [16, 0])
  const scale = useTransform(scrollYProgress, [0, 0.45], reduce ? [1, 1] : [0.94, 1])
  const glow = useTransform(scrollYProgress, [0, 0.5], [0.55, 0.9])

  return (
    <section id="top" ref={ref} className="relative overflow-hidden pt-36 pb-24 sm:pt-44">
      {/* 배경: 점 격자 + 청록·보라 빛 번짐 */}
      <div
        aria-hidden
        className="bg-dots absolute inset-0 [mask-image:radial-gradient(ellipse_at_top,black_30%,transparent_70%)]"
      />
      <div
        aria-hidden
        className="absolute top-[-12rem] left-1/2 h-[36rem] w-[60rem] -translate-x-1/2 rounded-full bg-[radial-gradient(closest-side,oklch(0.82_0.12_190/0.22),transparent)] blur-2xl"
      />
      <div
        aria-hidden
        className="absolute top-40 left-[65%] h-[24rem] w-[30rem] rounded-full bg-[radial-gradient(closest-side,oklch(0.72_0.15_285/0.18),transparent)] blur-2xl"
      />

      <div className="relative mx-auto max-w-6xl px-6 text-center">
        <motion.a
          {...rise(0.05)}
          href="#safety"
          className="group inline-flex items-center gap-2 rounded-full border border-white/10 bg-white/[0.04] py-1 pr-3 pl-1 text-sm text-muted-foreground backdrop-blur transition-colors hover:border-primary/40 hover:text-foreground"
        >
          <span className="rounded-full bg-primary/15 px-2 py-0.5 text-xs font-semibold text-primary">개발 중</span>
          설치 파일은 아직 없어요 · 안전 원칙 보기
          <ArrowRight className="size-3.5 transition-transform group-hover:translate-x-0.5" />
        </motion.a>

        <motion.h1
          {...rise(0.12)}
          className="mx-auto mt-7 max-w-4xl text-5xl leading-[1.08] font-extrabold tracking-tight text-balance sm:text-6xl lg:text-7xl"
        >
          Windows를
          <br />
          <span className="text-gradient">가볍고 깔끔하게.</span>
        </motion.h1>

        <motion.p
          {...rise(0.2)}
          className="mx-auto mt-6 max-w-2xl text-lg text-pretty text-muted-foreground sm:text-xl"
        >
          프로그램 제거부터 남은 흔적 찾기, 저장공간·시작프로그램 관리까지 한곳에서. 지울 항목은{" "}
          <strong className="font-semibold text-foreground">근거를 보고 직접</strong> 고르고, 남은 흔적은 백업한 뒤
          정리해요. 캐시처럼 되돌릴 수 없는 것은 미리 알려 드려요.
        </motion.p>

        <motion.div {...rise(0.28)} className="mt-9 flex flex-wrap items-center justify-center gap-3">
          <Button asChild size="lg" className="h-12 rounded-xl px-6 text-base shadow-lg shadow-primary/25">
            <a href={REPO_URL} target="_blank" rel="noreferrer">
              <GithubIcon className="size-4.5" />
              개발 현황 보기
            </a>
          </Button>
          <Button asChild size="lg" variant="outline" className="h-12 rounded-xl px-6 text-base">
            <a href="#features">
              기능 살펴보기
              <ArrowRight />
            </a>
          </Button>
        </motion.div>
        <motion.p {...rise(0.34)} className="mt-4 text-sm text-muted-foreground/80">
          Windows 10 · 11 (x64) · 소스 공개 · 개발 버전을 테스트 앱으로 검증하는 중이에요. 첫 릴리스부터 다운로드할 수
          있어요
        </motion.p>
      </div>

      {/* 앱 화면 */}
      <div className="relative mx-auto mt-16 max-w-6xl px-6 [perspective:1600px]">
        <motion.div
          initial={{ opacity: 0, y: 60 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 1.1, delay: 0.35, ease: easeOutQuint }}
        >
          <motion.div style={{ rotateX, scale, transformOrigin: "50% 0%" }} className="relative">
            <motion.div
              aria-hidden
              style={{ opacity: glow }}
              className="absolute -inset-x-10 -top-10 -bottom-4 -z-10 rounded-[3rem] bg-[radial-gradient(closest-side,oklch(0.82_0.12_190/0.35),transparent)] blur-3xl"
            />
            <WindowFrame src={shot("programs")} alt="RobinX 프로그램 목록 화면" eager />

            <FloatingChip
              className="top-[18%] -left-4 hidden sm:flex lg:-left-12"
              delay={1.1}
              icon={<Sparkles className="size-4 text-primary" />}
              title="남은 흔적 7개 발견"
              detail="근거 강함 3 · 검토 필요 4"
            />
            <FloatingChip
              className="top-[46%] -right-4 hidden sm:flex lg:-right-12"
              delay={1.3}
              icon={<Archive className="size-4 text-chart-2" />}
              title="백업 후 정리 완료"
              detail="241 MB · 복구 기록에 보관"
            />
            <FloatingChip
              className="bottom-[10%] left-[8%] hidden md:flex"
              delay={1.5}
              icon={<ShieldCheck className="size-4 text-chart-5" />}
              title="보호 규칙으로 3개 제외"
              detail="Windows 핵심 경로는 건드리지 않아요"
            />
          </motion.div>
        </motion.div>
        {/* 아래쪽을 배경으로 자연스럽게 흐리기 */}
        <div
          aria-hidden
          className="pointer-events-none absolute inset-x-0 bottom-0 h-40 bg-gradient-to-t from-background to-transparent"
        />
      </div>
    </section>
  )
}

function FloatingChip({
  className,
  delay,
  icon,
  title,
  detail,
}: {
  className: string
  delay: number
  icon: React.ReactNode
  title: string
  detail: string
}) {
  const reduce = useReducedMotion()
  return (
    <motion.div
      initial={{ opacity: 0, scale: 0.9, y: 10 }}
      animate={{ opacity: 1, scale: 1, y: 0 }}
      transition={{ duration: 0.6, delay, ease: easeOutQuint }}
      className={`absolute z-10 ${className}`}
    >
      <motion.div
        animate={reduce ? undefined : { y: [0, -6, 0] }}
        transition={{ duration: 5, repeat: Infinity, ease: "easeInOut", delay }}
        className="glass liquid flex items-center gap-3 rounded-2xl px-4 py-3 text-left"
      >
        <span className="flex size-9 items-center justify-center rounded-xl bg-white/5">{icon}</span>
        <span>
          <span className="block text-sm font-semibold">{title}</span>
          <span className="block text-xs text-muted-foreground">{detail}</span>
        </span>
      </motion.div>
    </motion.div>
  )
}
