import type { ReactNode } from "react"
import { motion, useReducedMotion } from "motion/react"

export const easeOutQuint = [0.22, 1, 0.36, 1] as const

/**
 * 구역이 화면에 들어올 때 제목 → 글 → 그림 순서로 떠오르게 한다 (`delay`로 순서를 준다).
 * 움직임을 줄이는 설정이면 처음부터 제자리에 있다.
 */
export function Reveal({
  children,
  delay = 0,
  className,
  y = 24,
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
      transition={{ duration: 0.75, delay, ease: easeOutQuint }}
    >
      {children}
    </Comp>
  )
}
