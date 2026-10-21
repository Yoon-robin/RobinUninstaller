import { useEffect, useLayoutEffect, useRef, useState, type ReactNode } from "react"
import { animate, motion, useReducedMotion, type Variants } from "motion/react"

export const EASE_OUT_QUINT = [0.22, 1, 0.36, 1] as const

/** 순서대로 나타나는 묶음. 자식은 <FadeUp>을 쓴다. */
export const staggerContainer: Variants = {
  hidden: {},
  show: { transition: { staggerChildren: 0.025, delayChildren: 0 } },
}

const fadeUpVariants: Variants = {
  // 페이지를 옮길 때마다 보이는 동작이라 짧고 가볍게 (흐림 없이 살짝 떠오르기만)
  hidden: { opacity: 0, y: 4 },
  show: {
    opacity: 1,
    y: 0,
    transition: { duration: 0.2, ease: EASE_OUT_QUINT },
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

/** `id`를 준 숫자가 마지막으로 보여 준 값. 화면을 옮겼다 돌아와도 0부터 다시 세지 않게 한다. */
const lastShown = new Map<string, number>()

/**
 * 숫자가 0에서 목표값까지 올라간다. 처음 한 번만 움직이고, 이후 값이 바뀌면 거기서부터 이어서 움직인다.
 * `id`를 주면 다시 마운트돼도(다른 화면에 다녀오기) 마지막 값에서 시작한다.
 */
export function CountUp({
  value,
  format = formatInteger,
  id,
}: {
  value: number
  format?: (n: number) => string
  id?: string
}) {
  const ref = useRef<HTMLSpanElement>(null)
  const from = useRef(id ? (lastShown.get(id) ?? 0) : 0)
  const reduce = useReducedMotion()

  useEffect(() => {
    const el = ref.current
    if (!el) return
    if (reduce) {
      el.textContent = format(value)
      from.current = value
      if (id) lastShown.set(id, value)
      return
    }
    const controls = animate(from.current, value, {
      duration: 0.9,
      ease: EASE_OUT_QUINT,
      onUpdate: (n) => {
        // 중간에 멈춰도(StrictMode 재실행, 값 변경) 지금 보이는 숫자에서 이어 가도록 매 프레임 기록한다.
        from.current = n
        if (id) lastShown.set(id, n)
        el.textContent = format(n)
      },
    })
    return () => controls.stop()
  }, [value, reduce, format, id])

  return <span ref={ref}>{format(reduce ? value : from.current)}</span>
}

/**
 * 안의 내용이 바뀌어 높이가 달라질 때 한 번에 튀지 않고 짧게 따라간다 (대화상자의 단계 전환 등).
 * 움직임 줄이기면 바로 바뀐다.
 */
export function AutoHeight({ children, className }: { children: ReactNode; className?: string }) {
  const inner = useRef<HTMLDivElement>(null)
  const [height, setHeight] = useState<number | "auto">("auto")
  const reduce = useReducedMotion()

  useLayoutEffect(() => {
    const el = inner.current
    if (!el) return
    const observer = new ResizeObserver(() => setHeight(el.offsetHeight))
    observer.observe(el)
    return () => observer.disconnect()
  }, [])

  return (
    <motion.div
      className={className}
      style={{ overflow: "hidden" }}
      initial={false}
      animate={{ height }}
      transition={{ duration: reduce ? 0 : 0.25, ease: EASE_OUT_QUINT }}
    >
      <div ref={inner}>{children}</div>
    </motion.div>
  )
}

/**
 * 대화상자를 열 때 `data-autofocus`가 붙은 요소(보통 취소 버튼)에 포커스를 준다.
 * 없으면 Radix 기본(첫 번째로 누를 수 있는 요소)을 따른다. `onOpenAutoFocus`에 넘긴다.
 */
export function focusMarked(e: Event) {
  const el = (e.currentTarget as HTMLElement | null)?.querySelector<HTMLElement>("[data-autofocus]")
  if (!el) return
  e.preventDefault()
  el.focus()
}
