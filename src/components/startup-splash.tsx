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
/** 목록을 금방 읽어도 이만큼은 보여 준다 (너무 빨리 휙 지나가지 않게) */
const MIN_MS = 1300

/**
 * 앱을 처음 열 때 보이는 화면. index.html의 시작 화면(창 열림 애니메이션)을 같은 자리·모양으로 이어받아,
 * 첫 화면에 필요한 설치 프로그램 목록을 읽는 동안 보여 준다. 빨리 읽어도 MIN_MS는 보여 주고,
 * 사라질 때는 살짝 커지며 흐려지고 그 뒤로 앱 화면이 자리 잡는다(`onDone`).
 */
export function StartupSplash({ onDone }: { onDone?: () => void }) {
  const apps = useQuery({ queryKey: ["installed-apps"], queryFn: api.listInstalledApps })
  const reduce = useReducedMotion()
  const [timedOut, setTimedOut] = useState(false)
  const [minElapsed, setMinElapsed] = useState(false)
  const [step, setStep] = useState(0)

  useEffect(() => {
    const m = setTimeout(() => setMinElapsed(true), reduce ? 0 : MIN_MS)
    return () => clearTimeout(m)
  }, [reduce])

  useEffect(() => {
    const t = setTimeout(() => setTimedOut(true), MAX_MS)
    const s = setInterval(() => setStep((i) => Math.min(i + 1, STEPS.length - 1)), 2200)
    return () => {
      clearTimeout(t)
      clearInterval(s)
    }
  }, [])

  const visible = (apps.isLoading || !minElapsed) && !timedOut
  useEffect(() => {
    if (!visible) onDone?.()
  }, [visible, onDone])

  return (
    <AnimatePresence>
      {visible && (
        <motion.div
          key="splash"
          role="status"
          aria-live="polite"
          aria-label="RobinX를 준비하는 중이에요"
          className="fixed inset-0 z-[100] flex flex-col bg-background"
          exit={
            reduce
              ? { opacity: 0, transition: { duration: 0 } }
              : { opacity: 0, scale: 1.04, transition: { duration: 0.5, ease: EASE_OUT_QUINT } }
          }
        >
          {/* 시작 화면에서도 창을 옮기고 닫을 수 있게 */}
          <div data-tauri-drag-region className="flex h-12 shrink-0 items-center justify-end">
            <WindowControls />
          </div>
          <div data-tauri-drag-region className="relative flex flex-1 flex-col items-center justify-center pb-16">
            {/* 로고와 이름은 index.html의 시작 화면과 같은 자리에 그대로 둔다 (다시 나타나면 튀어 보인다) */}
            <Logo className="relative size-20 drop-shadow-xl" />
            <p className="mt-6 text-2xl font-bold tracking-tight">RobinX</p>
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
