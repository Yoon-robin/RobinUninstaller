import type { ReactNode } from "react"
import { motion, useReducedMotion } from "motion/react"

export const easeOutQuint = [0.22, 1, 0.36, 1] as const

/**
 * 구역이 화면에 들어올 때 짧게 떠오르게 한다 (`delay`로 순서를 준다). 첫 화면보다 작고 빠르게:
 * 페이지 전체가 계속 아래에서 떠오르는 느낌이 들지 않게.
 * 움직임을 줄이는 설정이면 처음부터 제자리에 있다.
 */
export function Reveal({
  children,
  delay = 0,
  className,
  y = 12,
  as = "div",
}: {
  children: ReactNode
  delay?: number
  className?: string
  y?: number
  as?: "div" | "li"
}) {
  const reduce = useReducedMotion()
  const Comp = as === "li" ? motion.li : motion.div
  return (
    <Comp
      className={className}
      initial={reduce ? false : { opacity: 0, y }}
      whileInView={{ opacity: 1, y: 0 }}
      viewport={{ once: true, margin: "-60px" }}
      transition={{ duration: 0.5, delay, ease: easeOutQuint }}
    >
      {children}
    </Comp>
  )
}
