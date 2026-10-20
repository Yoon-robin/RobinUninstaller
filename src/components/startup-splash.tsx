import { useEffect, useState } from "react"
import { useQuery } from "@tanstack/react-query"
import { AnimatePresence, motion, useReducedMotion } from "motion/react"

import { api } from "@/lib/api"
import { Logo } from "@/components/logo"
import { WindowControls } from "@/components/window-controls"
import { EASE_OUT_QUINT } from "@/components/motion"

/** 오래 걸릴 때 보여 줄 안내 (몇 초마다 바뀐다) */
const STEPS = ["설치된 프로그램을 읽는 중이에요", "아이콘과 크기 정보를 모으는 중이에요", "거의 다 됐어요"]

/** 이보다 오래 걸리면 시작 화면을 걷고 목록 화면(스켈레톤)으로 넘어간다 */
const MAX_MS = 8000

/**
 * 앱을 처음 열 때 보이는 화면. 첫 화면에 필요한 설치 프로그램 목록을 읽는 동안 로고와 진행 상태를 보여 주고,
 * 다 읽으면 부드럽게 사라진다. 아주 빨리 끝나면 깜빡이지 않게 거의 보이지 않는다.
 */
export function StartupSplash() {
  const apps = useQuery({ queryKey: ["installed-apps"], queryFn: api.listInstalledApps })
  const reduce = useReducedMotion()
  const [timedOut, setTimedOut] = useState(false)
  const [step, setStep] = useState(0)

  useEffect(() => {
    const t = setTimeout(() => setTimedOut(true), MAX_MS)
    const s = setInterval(() => setStep((i) => Math.min(i + 1, STEPS.length - 1)), 2200)
    return () => {
      clearTimeout(t)
      clearInterval(s)
    }
  }, [])

  const visible = apps.isLoading && !timedOut

  return (
    <AnimatePresence>
      {visible && (
        <motion.div
          key="splash"
          role="status"
          aria-live="polite"
          aria-label="RobinX를 준비하는 중이에요"
          className="fixed inset-0 z-[100] flex flex-col bg-background"
          exit={{ opacity: 0, transition: { duration: reduce ? 0 : 0.45, ease: EASE_OUT_QUINT } }}
        >
          {/* 시작 화면에서도 창을 옮기고 닫을 수 있게 */}
          <div data-tauri-drag-region className="flex h-12 shrink-0 items-center justify-end">
            <WindowControls />
          </div>
          <div data-tauri-drag-region className="relative flex flex-1 flex-col items-center justify-center pb-16">
            <motion.div
              initial={reduce ? false : { opacity: 0, scale: 0.86, filter: "blur(8px)" }}
              animate={{ opacity: 1, scale: 1, filter: "blur(0px)" }}
              transition={{ duration: 0.7, ease: EASE_OUT_QUINT }}
              className="relative"
            >
              {!reduce && (
                <motion.span
                  aria-hidden
                  className="absolute inset-0 rounded-[1.4rem] bg-primary/25 blur-xl"
                  animate={{ opacity: [0.35, 0.8, 0.35], scale: [0.95, 1.08, 0.95] }}
                  transition={{ duration: 2.4, repeat: Infinity, ease: "easeInOut" }}
                />
              )}
              <Logo className="relative size-20 drop-shadow-xl" />
            </motion.div>
            <motion.p
              initial={reduce ? false : { opacity: 0, y: 8 }}
              animate={{ opacity: 1, y: 0 }}
              transition={{ duration: 0.6, delay: 0.15, ease: EASE_OUT_QUINT }}
              className="mt-6 text-2xl font-bold tracking-tight"
            >
              RobinX
            </motion.p>
            <div className="mt-6 h-1 w-48 overflow-hidden rounded-full bg-muted">
              <div className="scan-sweep h-full w-1/3 rounded-full bg-primary" />
            </div>
            <div className="mt-4 h-5">
              <AnimatePresence mode="wait" initial={false}>
                <motion.p
                  key={step}
                  initial={reduce ? false : { opacity: 0, y: 4 }}
                  animate={{ opacity: 1, y: 0 }}
                  exit={{ opacity: 0, y: -4 }}
                  transition={{ duration: 0.25 }}
                  className="text-sm text-muted-foreground"
                >
                  {STEPS[step]}
                </motion.p>
              </AnimatePresence>
            </div>
          </div>
        </motion.div>
      )}
    </AnimatePresence>
  )
}
