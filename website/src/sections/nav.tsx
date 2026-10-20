import { useRef, useState } from "react"
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

/** 평범한 막대: 이름, 링크 몇 개, 내려받기 하나. GitHub는 아이콘 링크로 둔다. 떠 있거나 흐린 유리는 쓰지 않는다. */
export function Nav() {
  const [menuOpen, setMenuOpen] = useState(false)
  const pendingTarget = useRef<string | null>(null)

  return (
    <header className="frame flex h-16 items-center gap-3 sm:h-20 sm:gap-8">
      <a href="#top" className="flex items-center gap-2.5 text-body font-semibold tracking-tight">
        <img src={LOGO} alt="" width={28} height={28} className="size-7 rounded-[0.45rem]" />
        RobinX
      </a>
      <nav aria-label="페이지 메뉴" className="ml-auto hidden items-center gap-7 md:flex">
        {links.map((l) => (
          <a
            key={l.href}
            href={l.href}
            className="text-body text-foreground/80 transition-colors hover:text-foreground focus-visible:rounded-sm focus-visible:ring-2 focus-visible:ring-ring focus-visible:outline-none"
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
        className="ml-auto hidden size-11 items-center justify-center rounded-full text-foreground/80 transition-colors hover:bg-muted hover:text-foreground focus-visible:ring-2 focus-visible:ring-ring focus-visible:outline-none sm:inline-flex md:ml-0"
      >
        <GithubIcon className="size-5" />
      </a>
      <a
        href={RELEASE.downloadUrl}
        className="ml-auto inline-flex h-11 items-center gap-2 rounded-full bg-primary px-5 text-body font-semibold text-primary-foreground transition-[transform,background-color] hover:bg-primary/90 focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 focus-visible:outline-none active:scale-[0.97] sm:ml-0"
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
  )
}
