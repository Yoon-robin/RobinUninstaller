import { useEffect, useRef, type ReactNode } from "react"
import { animate, motion, useReducedMotion, type Variants } from "motion/react"

export const EASE_OUT_QUINT = [0.22, 1, 0.36, 1] as const

/** 순서대로 나타나는 묶음. 자식은 <FadeUp>을 쓴다. */
export const staggerContainer: Variants = {
  hidden: {},
  show: { transition: { staggerChildren: 0.04, delayChildren: 0.02 } },
}

const fadeUpVariants: Variants = {
  // 페이지를 옮길 때마다 보이는 동작이라 짧고 가볍게 (흐림 없이 살짝 떠오르기만)
  hidden: { opacity: 0, y: 6 },
  show: {
    opacity: 1,
    y: 0,
    transition: { duration: 0.24, ease: EASE_OUT_QUINT },
  },
}

export function Stagger({ children, className }: { children: ReactNode; className?: string }) {
  return (
    <motion.div className={className} variants={staggerContainer} initial="hidden" animate="show">
      {children}
    </motion.div>
  )
}

export function FadeUp({ children, className }: { children: ReactNode; className?: string }) {
  return (
    <motion.div className={className} variants={fadeUpVariants}>
      {children}
    </motion.div>
  )
}

const formatInteger = (n: number) => Math.round(n).toLocaleString()

/** 숫자가 0에서 목표값까지 올라간다. 처음 한 번만 움직이고, 이후 값이 바뀌면 거기서부터 이어서 움직인다. */
export function CountUp({ value, format = formatInteger }: { value: number; format?: (n: number) => string }) {
  const ref = useRef<HTMLSpanElement>(null)
  const from = useRef(0)
  const reduce = useReducedMotion()

  useEffect(() => {
    const el = ref.current
    if (!el) return
    if (reduce) {
      el.textContent = format(value)
      from.current = value
      return
    }
    const controls = animate(from.current, value, {
      duration: 0.9,
      ease: EASE_OUT_QUINT,
      onUpdate: (n) => {
        // 중간에 멈춰도(StrictMode 재실행, 값 변경) 지금 보이는 숫자에서 이어 가도록 매 프레임 기록한다.
        from.current = n
        el.textContent = format(n)
      },
    })
    return () => controls.stop()
  }, [value, reduce, format])

  return <span ref={ref}>{format(reduce ? value : from.current)}</span>
}
