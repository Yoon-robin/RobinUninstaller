import type { MouseEvent, ReactNode } from "react"
import { motion, useReducedMotion } from "motion/react"

import { cn } from "@/lib/utils"

export const easeOutQuint = [0.22, 1, 0.36, 1] as const

/**
 * 구역이 화면에 들어올 때 떠오르게 한다 (`delay`로 순서를 준다).
 * `heading`이면 구역 제목처럼 아래에서 가려져 있다가 걷히듯 나타난다 (제목에만 쓴다).
 * 움직임을 줄이는 설정이면 처음부터 제자리에 있다.
 */
export function Reveal({
  children,
  delay = 0,
  className,
  y = 16,
  as = "div",
  heading = false,
  outerClassName,
}: {
  children: ReactNode
  delay?: number
  className?: string
  y?: number
  as?: "div" | "li"
  heading?: boolean
  /** `heading`일 때 바깥 판의 클래스 (그리드 칸 차지처럼 부모 배치에 쓰는 것) */
  outerClassName?: string
}) {
  const reduce = useReducedMotion()
  const Comp = as === "li" ? motion.li : motion.div
  if (heading) {
    // 걷히는 판(clip-path)을 지켜보면 Chromium이 잘린 영역으로 화면 안인지 따져 좁은 화면에서 끝내 나타나지 않았다.
    // 그래서 잘리지 않은 바깥 판이 화면에 들어왔는지 보고, 안쪽 판만 걷히게 한다.
    return (
      <motion.div
        className={outerClassName}
        initial={reduce ? false : "hidden"}
        whileInView="shown"
        viewport={{ once: true, margin: "-70px" }}
      >
        <motion.div
          className={className}
          variants={{
            hidden: { opacity: 0, y: 28, clipPath: "inset(0% 0% 100% 0%)" },
            shown: { opacity: 1, y: 0, clipPath: "inset(0% 0% -20% 0%)" },
          }}
          transition={{ duration: 0.9, delay, ease: easeOutQuint }}
        >
          {children}
        </motion.div>
      </motion.div>
    )
  }
  return (
    <Comp
      className={className}
      initial={reduce ? false : { opacity: 0, y }}
      whileInView={{ opacity: 1, y: 0 }}
      viewport={{ once: true, margin: "-70px" }}
      transition={{ duration: 0.6, delay, ease: easeOutQuint }}
    >
      {children}
    </Comp>
  )
}

/**
 * 마우스를 따라 판 위에 옅은 빛이 비친다 (기능 타일 등). 마우스가 없는 기기와 움직임 줄이기에서는 아무것도 하지 않는다.
 * 빛은 판 바탕과 내용 사이의 층이라 글자나 그림을 가리지 않는다.
 */
export function Spotlight({ children, className }: { children: ReactNode; className?: string }) {
  const onMove = (e: MouseEvent<HTMLDivElement>) => {
    const r = e.currentTarget.getBoundingClientRect()
    e.currentTarget.style.setProperty("--x", `${e.clientX - r.left}px`)
    e.currentTarget.style.setProperty("--y", `${e.clientY - r.top}px`)
  }
  return (
    <div onMouseMove={onMove} className={cn("group/spot relative isolate", className)}>
      <div
        aria-hidden
        className="pointer-events-none absolute inset-0 -z-10 rounded-[inherit] opacity-0 transition-opacity duration-300 group-hover/spot:opacity-100 motion-reduce:hidden"
        style={{
          background:
            "radial-gradient(420px circle at var(--x, 50%) var(--y, 50%), oklch(0.75 0.13 230 / 9%), transparent 45%)",
        }}
      />
      {children}
    </div>
  )
}
