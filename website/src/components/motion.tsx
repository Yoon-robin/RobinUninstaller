import type { ReactNode } from "react"
import { motion, useReducedMotion } from "motion/react"

export const easeOutQuint = [0.22, 1, 0.36, 1] as const

/** 화면에 들어올 때 아래에서 살짝 떠오른다. */
export function Reveal({
  children,
  delay = 0,
  className,
  y = 18,
}: {
  children: ReactNode
  delay?: number
  className?: string
  y?: number
}) {
  const reduce = useReducedMotion()
  return (
    <motion.div
      className={className}
      initial={reduce ? false : { opacity: 0, y }}
      whileInView={{ opacity: 1, y: 0 }}
      viewport={{ once: true, margin: "-80px" }}
      transition={{ duration: 0.7, delay, ease: easeOutQuint }}
    >
      {children}
    </motion.div>
  )
}

/** 섹션 머리말 (작은 꼬리표 + 제목 + 설명) */
export function SectionHeading({
  eyebrow,
  title,
  description,
}: {
  eyebrow: string
  title: ReactNode
  description?: ReactNode
}) {
  return (
    <Reveal className="mx-auto max-w-2xl text-center">
      <p className="text-sm font-semibold tracking-wide text-primary">{eyebrow}</p>
      <h2 className="mt-3 text-3xl font-bold tracking-tight text-balance sm:text-4xl">{title}</h2>
      {description && <p className="mt-4 text-base text-pretty text-muted-foreground sm:text-lg">{description}</p>}
    </Reveal>
  )
}
