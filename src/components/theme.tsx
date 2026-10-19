import { createContext, useContext, useEffect, useMemo, useState, type MouseEvent, type ReactNode } from "react"
import { flushSync } from "react-dom"
import { invoke } from "@tauri-apps/api/core"
import { getCurrentWindow } from "@tauri-apps/api/window"
import { Moon, Sun } from "lucide-react"
import { AnimatePresence, motion, useReducedMotion } from "motion/react"

import { IconTooltip } from "@/components/icon-tooltip"
import { EASE_OUT_QUINT } from "@/components/motion"

type Theme = "dark" | "light" | "system"
type Resolved = "dark" | "light"
/** 유리(반투명·흐림) 효과: 자동이면 Windows의 "투명 효과" 설정을 따른다 */
export type Transparency = "auto" | "reduced"

const STORAGE_KEY = "robin.theme"
const TRANSPARENCY_KEY = "robin.transparency"

const ThemeContext = createContext<{
  theme: Theme
  resolvedTheme: Resolved
  setTheme: (theme: Theme) => void
  transparency: Transparency
  setTransparency: (value: Transparency) => void
}>({
  theme: "dark",
  resolvedTheme: "dark",
  setTheme: () => {},
  transparency: "auto",
  setTransparency: () => {},
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

function loadTransparency(): Transparency {
  try {
    return localStorage.getItem(TRANSPARENCY_KEY) === "reduced" ? "reduced" : "auto"
  } catch {
    return "auto"
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
  const [transparency, setTransparencyState] = useState<Transparency>(loadTransparency)

  // 설정에서 줄이기를 골랐거나, 자동일 때 Windows가 투명 효과를 끈 상태면 유리 효과를 끈다
  useEffect(() => {
    const mq = matchMedia("(prefers-reduced-transparency: reduce)")
    const apply = () =>
      document.documentElement.classList.toggle(
        "reduce-transparency",
        transparency === "reduced" || (transparency === "auto" && mq.matches),
      )
    apply()
    mq.addEventListener("change", apply)
    return () => mq.removeEventListener("change", apply)
  }, [transparency])

  useEffect(() => {
    const mq = matchMedia("(prefers-color-scheme: light)")
    const onChange = () => setSystem(mq.matches ? "light" : "dark")
    mq.addEventListener("change", onChange)
    return () => mq.removeEventListener("change", onChange)
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
      transparency,
      setTransparency: (next: Transparency) => {
        setTransparencyState(next)
        try {
          localStorage.setItem(TRANSPARENCY_KEY, next)
        } catch {
          // 저장이 막혀 있어도 이번 실행 동안은 바뀐다.
        }
      },
    }),
    [theme, resolvedTheme, transparency],
  )

  return (
    <ThemeContext.Provider value={value}>
      <WindowMaterialSync />
      {children}
    </ThemeContext.Provider>
  )
}

/** Windows 11이면 Mica 배경을 쓰고, 창의 Mica 색조를 앱 테마에 맞춘다. */
function WindowMaterialSync() {
  const { resolvedTheme } = useTheme()

  useEffect(() => {
    invoke<string>("window_material")
      .then((material) => document.documentElement.classList.toggle("mica", material === "mica"))
      .catch(() => {})
  }, [])

  useEffect(() => {
    if (resolvedTheme === "dark" || resolvedTheme === "light") {
      getCurrentWindow()
        .setTheme(resolvedTheme)
        .catch(() => {})
    }
  }, [resolvedTheme])

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
        { duration: 560, easing: "cubic-bezier(0.22, 1, 0.36, 1)", pseudoElement: "::view-transition-new(root)" },
      )
    })
  }

  return (
    <IconTooltip label={isDark ? "라이트 모드로" : "다크 모드로"}>
      <button
        type="button"
        onClick={toggle}
        aria-label={isDark ? "라이트 모드로" : "다크 모드로"}
        className="relative flex size-8 items-center justify-center rounded-md text-muted-foreground transition-colors hover:bg-foreground/8 hover:text-foreground"
      >
        <AnimatePresence mode="popLayout" initial={false}>
          <motion.span
            key={isDark ? "moon" : "sun"}
            initial={{ opacity: 0, rotate: -60, scale: 0.7, filter: "blur(3px)" }}
            animate={{ opacity: 1, rotate: 0, scale: 1, filter: "blur(0px)" }}
            exit={{ opacity: 0, rotate: 60, scale: 0.7, filter: "blur(3px)" }}
            transition={{ duration: 0.35, ease: EASE_OUT_QUINT }}
          >
            {isDark ? <Moon className="size-4" /> : <Sun className="size-4" />}
          </motion.span>
        </AnimatePresence>
      </button>
    </IconTooltip>
  )
}
