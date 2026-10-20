import { useEffect, useRef, useState } from "react"
import { Download, Menu } from "lucide-react"

import { LOGO, RELEASE, REPO_URL } from "@/lib/site"
import { GithubIcon } from "@/components/github-icon"
import { Sheet, SheetContent, SheetDescription, SheetTitle, SheetTrigger } from "@/components/ui/sheet"

export const links = [
  { href: "#evidence", label: "남은 흔적" },
  { href: "#features", label: "기능" },
  { href: "#restore", label: "되돌리기" },
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

/**
 * 위에 붙는 막대: 왼쪽 이름, 가운데 둥근 판 안의 링크, 오른쪽 GitHub 아이콘과 다운로드.
 * 맨 위에서는 첫 화면의 빛이 비치도록 투명하고, 내려가면 어두운 바탕을 깔아 글자 위에서도 읽힌다.
 */
export function Nav() {
  const [menuOpen, setMenuOpen] = useState(false)
  const [scrolled, setScrolled] = useState(false)
  const pendingTarget = useRef<string | null>(null)

  useEffect(() => {
    const onScroll = () => setScrolled(window.scrollY > 24)
    onScroll()
    window.addEventListener("scroll", onScroll, { passive: true })
    return () => window.removeEventListener("scroll", onScroll)
  }, [])

  return (
    <div
      className={
        "fixed inset-x-0 top-0 z-30 border-b transition-[background-color,border-color,backdrop-filter] duration-300 " +
        (scrolled ? "border-white/[0.08] bg-background/75 backdrop-blur-xl" : "border-transparent")
      }
    >
      <header className="frame relative flex h-16 items-center gap-3 sm:h-20 sm:gap-6">
        <a href="#top" className="flex items-center gap-2.5 text-body font-semibold tracking-tight">
          <img src={LOGO} alt="" width={28} height={28} className="size-7 rounded-[0.45rem]" />
          RobinX
        </a>
        <nav
          aria-label="페이지 메뉴"
          className="absolute left-1/2 hidden -translate-x-1/2 items-center gap-1 rounded-full border bg-white/[0.03] p-1 md:flex"
        >
          {links.map((l) => (
            <a
              key={l.href}
              href={l.href}
              className="rounded-full px-4 py-1.5 text-fine font-medium text-foreground/75 transition-colors hover:bg-white/[0.06] hover:text-foreground focus-visible:ring-2 focus-visible:ring-ring focus-visible:outline-none"
            >
              {l.label}
            </a>
          ))}
        </nav>
        <a
          href={REPO_URL}
          target="_blank"
          rel="noreferrer"
          aria-label="GitHub에서 소스 보기"
          className="ml-auto hidden size-11 items-center justify-center rounded-full text-foreground/75 transition-colors hover:bg-white/[0.06] hover:text-foreground focus-visible:ring-2 focus-visible:ring-ring focus-visible:outline-none sm:inline-flex"
        >
          <GithubIcon className="size-5" />
        </a>
        <a
          href={RELEASE.downloadUrl}
          className="ml-auto inline-flex h-10 items-center gap-2 rounded-full bg-primary px-4 text-fine font-semibold text-primary-foreground transition-[transform,background-color] hover:bg-primary/90 focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 focus-visible:ring-offset-background focus-visible:outline-none active:scale-[0.97] sm:ml-0 sm:h-11 sm:px-5 sm:text-body"
        >
          <Download className="size-4" />
          다운로드
        </a>

        {/* 좁은 화면: 메뉴 버튼 → 옆에서 나오는 패널 */}
        <Sheet open={menuOpen} onOpenChange={setMenuOpen}>
          <SheetTrigger asChild>
            <button
              type="button"
              aria-label="메뉴 열기"
              className="-mr-2 inline-flex size-11 items-center justify-center rounded-full transition-colors hover:bg-muted focus-visible:ring-2 focus-visible:ring-ring focus-visible:outline-none md:hidden"
            >
              <Menu className="size-5" />
            </button>
          </SheetTrigger>
          <SheetContent
            side="right"
            className="w-72 border-none bg-background p-6"
            onCloseAutoFocus={(e) => {
              const href = pendingTarget.current
              if (!href) return
              pendingTarget.current = null
              e.preventDefault()
              goTo(href)
            }}
          >
            <SheetTitle className="text-lead font-semibold">RobinX</SheetTitle>
            <SheetDescription className="sr-only">페이지 안의 다른 부분으로 이동해요</SheetDescription>
            <nav aria-label="페이지 메뉴" className="mt-6 flex flex-col">
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
                  className="rounded-xl px-3 py-3 text-lead font-medium transition-colors hover:bg-muted focus-visible:ring-2 focus-visible:ring-ring focus-visible:outline-none"
                >
                  {l.label}
                </a>
              ))}
              <a
                href={REPO_URL}
                target="_blank"
                rel="noreferrer"
                className="mt-4 flex items-center gap-2.5 rounded-xl px-3 py-3 text-lead font-medium transition-colors hover:bg-muted focus-visible:ring-2 focus-visible:ring-ring focus-visible:outline-none"
              >
                <GithubIcon className="size-5" />
                GitHub
              </a>
            </nav>
          </SheetContent>
        </Sheet>
      </header>
    </div>
  )
}
