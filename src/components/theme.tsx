import { createContext, useContext, useEffect, useMemo, useState, type MouseEvent, type ReactNode } from "react"
import { flushSync } from "react-dom"
import { getCurrentWindow } from "@tauri-apps/api/window"
import { Moon, Sun } from "lucide-react"
import { AnimatePresence, motion, useReducedMotion } from "motion/react"

import { IconTooltip } from "@/components/icon-tooltip"
import { EASE_OUT_QUINT } from "@/components/motion"

type Theme = "dark" | "light" | "system"
type Resolved = "dark" | "light"

const STORAGE_KEY = "robin.theme"

const ThemeContext = createContext<{
  theme: Theme
  resolvedTheme: Resolved
  setTheme: (theme: Theme) => void
}>({
  theme: "dark",
  resolvedTheme: "dark",
  setTheme: () => {},
})

export const useTheme = () => useContext(ThemeContext)

function loadTheme(): Theme {
  try {
    const saved = localStorage.getItem(STORAGE_KEY)
    return saved === "light" || saved === "system" ? saved : "dark"
  } catch {
    return "dark"
  }
}

function systemTheme(): Resolved {
  return matchMedia("(prefers-color-scheme: light)").matches ? "light" : "dark"
}

/** 다크/라이트 테마 상태. html의 class와 color-scheme을 맞추고 선택을 저장한다. */
export function ThemeProvider({ children }: { children: ReactNode }) {
  const [theme, setThemeState] = useState<Theme>(loadTheme)
  const [system, setSystem] = useState<Resolved>(systemTheme)
  const resolvedTheme = theme === "system" ? system : theme

  useEffect(() => {
    const mq = matchMedia("(prefers-color-scheme: light)")
    const onChange = () => setSystem(mq.matches ? "light" : "dark")
    mq.addEventListener("change", onChange)
    // 창이 받은 Windows 밝기 변경도 따라간다 (WebView의 미디어 쿼리가 늦게 바뀌는 경우가 있어서)
    const unlisten = getCurrentWindow()
      .onThemeChanged(({ payload }) => setSystem(payload === "light" ? "light" : "dark"))
      .catch(() => null)
    return () => {
      mq.removeEventListener("change", onChange)
      void unlisten.then((off) => off?.())
    }
  }, [])

  useEffect(() => {
    const root = document.documentElement
    root.classList.toggle("dark", resolvedTheme === "dark")
    root.classList.toggle("light", resolvedTheme === "light")
    root.style.colorScheme = resolvedTheme
  }, [resolvedTheme])

  const value = useMemo(
    () => ({
      theme,
      resolvedTheme,
      setTheme: (next: Theme) => {
        setThemeState(next)
        try {
          localStorage.setItem(STORAGE_KEY, next)
        } catch {
          // 저장이 막혀 있어도 이번 실행 동안은 테마가 바뀐다.
        }
      },
    }),
    [theme, resolvedTheme],
  )

  return (
    <ThemeContext.Provider value={value}>
      <WindowMaterialSync />
      {children}
    </ThemeContext.Provider>
  )
}

/**
 * 창 제목 표시줄·스크롤바 같은 Windows 쪽 색을 앱 테마에 맞춘다.
 * "시스템"이면 창 테마를 고정하지 않는다(null). 고정해 두면 Windows 밝기를 바꿔도 창이 알림을 받지 못해
 * 앱이 따라가지 않았다.
 */
function WindowMaterialSync() {
  const { theme } = useTheme()

  useEffect(() => {
    getCurrentWindow()
      .setTheme(theme === "system" ? null : theme)
      .catch(() => {})
  }, [theme])

  return null
}

/** 다크/라이트 전환 버튼. 누른 자리에서 새 테마가 원형으로 퍼져 나간다. */
export function ThemeToggle() {
  const { resolvedTheme, setTheme } = useTheme()
  const reduce = useReducedMotion()
  const isDark = resolvedTheme !== "light"

  const toggle = (e: MouseEvent<HTMLButtonElement>) => {
    const next = isDark ? "light" : "dark"
    const apply = () => {
      // 뷰 전환 캡처 전에 DOM이 바뀌어 있어야 하므로 class를 바로 바꾼다.
      document.documentElement.classList.toggle("dark", next === "dark")
      document.documentElement.classList.toggle("light", next === "light")
      setTheme(next)
    }

    if (reduce || !document.startViewTransition) {
      apply()
      return
    }

    const { left, top, width, height } = e.currentTarget.getBoundingClientRect()
    const x = left + width / 2
    const y = top + height / 2
    const radius = Math.hypot(Math.max(x, innerWidth - x), Math.max(y, innerHeight - y))

    const transition = document.startViewTransition(() => flushSync(apply))
    transition.ready.then(() => {
      document.documentElement.animate(
        { clipPath: [`circle(0px at ${x}px ${y}px)`, `circle(${radius}px at ${x}px ${y}px)`] },
        { duration: 350, easing: "cubic-bezier(0.22, 1, 0.36, 1)", pseudoElement: "::view-transition-new(root)" },
      )
    })
  }

  return (
    <IconTooltip label={isDark ? "어두운 테마 · 누르면 밝게" : "밝은 테마 · 누르면 어둡게"}>
      <button
        type="button"
        onClick={toggle}
        aria-label={isDark ? "밝은 테마로 바꾸기" : "어두운 테마로 바꾸기"}
        className="relative flex size-8 items-center justify-center rounded-md text-muted-foreground transition-colors hover:bg-foreground/8 hover:text-foreground"
      >
        <AnimatePresence mode="popLayout" initial={false}>
          <motion.span
            key={isDark ? "moon" : "sun"}
            initial={{ opacity: 0, rotate: -45, scale: 0.75 }}
            animate={{ opacity: 1, rotate: 0, scale: 1 }}
            exit={{ opacity: 0, rotate: 45, scale: 0.75 }}
            transition={{ duration: 0.15, ease: EASE_OUT_QUINT }}
          >
            {isDark ? <Moon className="size-4" /> : <Sun className="size-4" />}
          </motion.span>
        </AnimatePresence>
      </button>
    </IconTooltip>
  )
}
