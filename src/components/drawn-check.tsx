import { motion, useReducedMotion } from "motion/react"

import { EASE_OUT_QUINT } from "@/components/motion"

/**
 * 완료 화면의 체크 표시. 선이 그어지듯 나타난다 (정리·제거가 끝났을 때 한 번 보는 순간이라 조금 눈에 띄어도 된다).
 * 모양은 lucide의 Check와 같다. 움직임 줄이기면 처음부터 다 그려져 있다.
 */
export function DrawnCheck({
  className,
  strokeWidth = 2.4,
  delay = 0.15,
}: {
  className?: string
  strokeWidth?: number
  delay?: number
}) {
  const reduce = useReducedMotion()
  return (
    <svg
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth={strokeWidth}
      strokeLinecap="round"
      strokeLinejoin="round"
      className={className}
      aria-hidden
    >
      <motion.path
        d="M20 6 9 17l-5-5"
        initial={reduce ? false : { pathLength: 0, opacity: 0 }}
        animate={{ pathLength: 1, opacity: 1 }}
        transition={{ duration: 0.4, delay, ease: EASE_OUT_QUINT }}
      />
    </svg>
  )
}
