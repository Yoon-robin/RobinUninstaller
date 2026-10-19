import { useEffect, useRef, useState } from "react"
import { motion } from "motion/react"
import { Menu } from "lucide-react"

import { cn } from "@/lib/utils"
import { LOGO, REPO_URL } from "@/lib/site"
import { Button } from "@/components/ui/button"
import { GithubIcon } from "@/components/github-icon"
import { easeOutQuint } from "@/components/motion"
import { Sheet, SheetContent, SheetDescription, SheetTitle, SheetTrigger } from "@/components/ui/sheet"

const links = [
  { href: "#features", label: "기능" },
  { href: "#how", label: "사용 방법" },
  { href: "#screens", label: "화면" },
  { href: "#safety", label: "안전 원칙" },
  { href: "#status", label: "개발 현황" },
  { href: "#faq", label: "FAQ" },
]

/** 앵커로 이동하고 그 구역에 포커스를 둔다 (스크린리더·키보드 사용자가 이어서 읽도록) */
function goTo(href: string) {
  const el = document.querySelector<HTMLElement>(href)
  if (!el) return
  history.pushState(null, "", href)
  el.scrollIntoView({ behavior: matchMedia("(prefers-reduced-motion: reduce)").matches ? "auto" : "smooth" })
  if (!el.hasAttribute("tabindex")) el.setAttribute("tabindex", "-1")
  el.focus({ preventScroll: true })
}

export function Nav() {
  const [scrolled, setScrolled] = useState(false)
  const [menuOpen, setMenuOpen] = useState(false)
  const pendingTarget = useRef<string | null>(null)
  useEffect(() => {
    const onScroll = () => setScrolled(window.scrollY > 12)
    onScroll()
    window.addEventListener("scroll", onScroll, { passive: true })
    return () => window.removeEventListener("scroll", onScroll)
  }, [])

  return (
    <motion.header
      initial={{ y: -24, opacity: 0 }}
      animate={{ y: 0, opacity: 1 }}
      transition={{ duration: 0.6, ease: easeOutQuint }}
      className="fixed inset-x-0 top-0 z-50 flex justify-center px-4 pt-3"
    >
      <nav
        className={cn(
          "flex w-full max-w-6xl items-center gap-6 rounded-2xl border px-4 py-2.5 transition-[background-color,border-color,box-shadow] duration-300",
          scrolled ? "glass border-transparent" : "border-transparent bg-transparent",
        )}
      >
        <a href="#top" className="flex items-center gap-2.5 font-semibold tracking-tight">
          <img src={LOGO} alt="" className="size-7 rounded-lg" />
          RobinX
        </a>
        <div className="ml-4 hidden items-center gap-1 md:flex">
          {links.map((l) => (
            <a
              key={l.href}
              href={l.href}
              className="rounded-lg px-3 py-1.5 text-sm text-muted-foreground transition-colors hover:bg-white/5 hover:text-foreground focus-visible:ring-2 focus-visible:ring-ring focus-visible:outline-none"
            >
              {l.label}
            </a>
          ))}
        </div>
        <Button asChild variant="outline" size="lg" className="ml-auto rounded-xl px-3 sm:px-3.5">
          <a href={REPO_URL} target="_blank" rel="noreferrer" aria-label="GitHub에서 보기">
            <GithubIcon className="size-4" />
            <span className="hidden sm:inline">GitHub</span>
          </a>
        </Button>
        {/* 좁은 화면: 메뉴 버튼 → 옆에서 나오는 패널 */}
        <Sheet open={menuOpen} onOpenChange={setMenuOpen}>
          <SheetTrigger asChild>
            <Button variant="outline" size="icon-lg" className="rounded-xl md:hidden" aria-label="메뉴 열기">
              <Menu />
            </Button>
          </SheetTrigger>
          <SheetContent
            side="right"
            className="w-72 border-white/10 bg-background/95 p-6 backdrop-blur-xl"
            onCloseAutoFocus={(e) => {
              const href = pendingTarget.current
              if (!href) return
              pendingTarget.current = null
              e.preventDefault()
              goTo(href)
            }}
          >
            <SheetTitle className="flex items-center gap-2.5">
              <img src={LOGO} alt="" className="size-7 rounded-lg" />
              RobinX
            </SheetTitle>
            <SheetDescription className="sr-only">페이지 안의 다른 부분으로 이동해요</SheetDescription>
            <nav aria-label="페이지 메뉴" className="mt-4 flex flex-col gap-1">
              {links.map((l) => (
                <a
                  key={l.href}
                  href={l.href}
                  onClick={(e) => {
                    // 패널이 닫히면서 포커스가 메뉴 버튼으로 돌아가지 않도록, 닫힌 뒤 목적지로 옮긴다
                    e.preventDefault()
                    setMenuOpen(false)
                    pendingTarget.current = l.href
                  }}
                  className="rounded-xl px-3 py-3 text-base text-muted-foreground transition-colors hover:bg-white/5 hover:text-foreground focus-visible:ring-2 focus-visible:ring-ring focus-visible:outline-none"
                >
                  {l.label}
                </a>
              ))}
            </nav>
          </SheetContent>
        </Sheet>
      </nav>
    </motion.header>
  )
}
