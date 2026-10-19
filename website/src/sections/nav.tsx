import { useEffect, useState } from "react"
import { motion } from "motion/react"

import { cn } from "@/lib/utils"
import { LOGO, REPO_URL } from "@/lib/site"
import { Button } from "@/components/ui/button"
import { GithubIcon } from "@/components/github-icon"
import { easeOutQuint } from "@/components/motion"

const links = [
  { href: "#features", label: "기능" },
  { href: "#how", label: "사용 방법" },
  { href: "#screens", label: "화면" },
  { href: "#safety", label: "안전 원칙" },
  { href: "#faq", label: "FAQ" },
]

export function Nav() {
  const [scrolled, setScrolled] = useState(false)
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
          scrolled
            ? "border-white/10 bg-background/70 shadow-lg shadow-black/20 backdrop-blur-xl"
            : "border-transparent bg-transparent",
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
              className="rounded-lg px-3 py-1.5 text-sm text-muted-foreground transition-colors hover:bg-white/5 hover:text-foreground"
            >
              {l.label}
            </a>
          ))}
        </div>
        <Button asChild variant="outline" size="lg" className="ml-auto rounded-xl px-3.5">
          <a href={REPO_URL} target="_blank" rel="noreferrer">
            <GithubIcon className="size-4" />
            GitHub
          </a>
        </Button>
      </nav>
    </motion.header>
  )
}
