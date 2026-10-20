import { useCallback, useEffect, useRef, useState } from "react"
import { QueryClient, QueryClientProvider } from "@tanstack/react-query"
import { ChevronRight } from "lucide-react"
import { MotionConfig, motion, useReducedMotion } from "motion/react"

import { AppSidebar, type Page } from "@/components/app-sidebar"
import { IconTooltip } from "@/components/icon-tooltip"
import { StartupSplash } from "@/components/startup-splash"
import { UpdatesProvider } from "@/features/updates/updates"
import { ThemeProvider, ThemeToggle } from "@/components/theme"
import { WindowControls } from "@/components/window-controls"
import { LeftoversPage } from "@/features/leftovers/leftovers-page"
import { RestorePage } from "@/features/restore/restore-page"
import { SettingsPage } from "@/features/settings/settings-page"
import { StartupPage } from "@/features/startup/startup-page"
import { TrackingPage } from "@/features/tracking/tracking-page"
import { StoragePage } from "@/features/storage/storage-page"
import { ProgramsPage } from "@/features/programs/programs-page"
import { SidebarInset, SidebarProvider, SidebarTrigger } from "@/components/ui/sidebar"
import { Toaster } from "@/components/ui/sonner"
import { TooltipProvider } from "@/components/ui/tooltip"
import { cn } from "@/lib/utils"
import { EASE_OUT_QUINT } from "@/components/motion"

const queryClient = new QueryClient({
  defaultOptions: { queries: { refetchOnWindowFocus: false, staleTime: 60_000 } },
})

const titles: Record<Page, string> = {
  programs: "프로그램",
  leftovers: "잔여물 정리",
  storage: "저장공간",
  startup: "시작프로그램",
  tracking: "설치 추적",
  restore: "복구 기록",
  settings: "설정",
}

export default function App() {
  const [page, setPage] = useState<Page>("programs")
  // 내용이 제목 표시줄 밑으로 지나가기 시작하면 표시줄을 유리처럼 만든다.
  const [scrolled, setScrolled] = useState(false)
  // 시작 화면이 걷히면 앱 화면이 살짝 커지며 자리 잡는다
  const [ready, setReady] = useState(false)
  const onSplashDone = useCallback(() => setReady(true), [])
  const reduce = useReducedMotion()
  useEffect(() => setScrolled(false), [page])
  // 페이지를 옮기면 새 페이지 제목에 포커스를 둔다 (처음 열 때는 건드리지 않는다)
  const firstPage = useRef(true)
  useEffect(() => {
    if (firstPage.current) {
      firstPage.current = false
      return
    }
    requestAnimationFrame(() => {
      // 프로그램 목록은 숨겨 둔 채 남아 있으므로 보이는 제목을 고른다
      const h1 = [...document.querySelectorAll<HTMLElement>("main main h1")].find((el) => el.offsetParent !== null)
      if (!h1) return
      h1.tabIndex = -1
      h1.focus({ preventScroll: true })
    })
  }, [page])

  // 다른 화면에서 보낸 이동 요청 (예: 정리 완료 → 복구 기록)
  useEffect(() => {
    const onNavigate = (e: Event) => setPage((e as CustomEvent<Page>).detail)
    window.addEventListener("robin:navigate", onNavigate)
    return () => window.removeEventListener("robin:navigate", onNavigate)
  }, [])

  return (
    <QueryClientProvider client={queryClient}>
      <ThemeProvider>
        {/* OS의 "애니메이션 효과 줄이기" 설정을 따른다. */}
        <MotionConfig reducedMotion="user">
          {/* 첫 툴팁은 잠깐 기다렸다가, 이어서 옮겨 다닐 때는 바로 뜬다. */}
          <TooltipProvider delayDuration={400} skipDelayDuration={300}>
            <UpdatesProvider ready={ready}>
              <motion.div
                className="h-svh"
                initial={reduce ? false : { opacity: 0, scale: 0.985 }}
                animate={ready || reduce ? { opacity: 1, scale: 1 } : { opacity: 0, scale: 0.985 }}
                transition={{ duration: 0.55, ease: EASE_OUT_QUINT }}
              >
                <SidebarProvider className="h-svh">
                  <AppSidebar page={page} onNavigate={setPage} />
                  <SidebarInset className="relative h-svh min-w-0 overflow-hidden bg-transparent">
                    {/* 제목 표시줄: 빈 곳을 끌면 창이 움직이고, 더블클릭하면 최대화된다. 내용 위에 떠 있다. */}
                    <header
                      data-tauri-drag-region
                      className={cn(
                        // 불투명한 바탕: 스크롤한 내용이 제목 줄 뒤로 비치지 않는다. 스크롤하면 아래 구분선만 보인다.
                        "absolute inset-x-0 top-0 z-20 flex h-12 items-center gap-1 bg-background pl-2 shadow-[0_1px_0_transparent] transition-shadow duration-200",
                        scrolled && "shadow-[0_1px_0_var(--border)]",
                      )}
                    >
                      <IconTooltip label="사이드바 접기/펼치기">
                        <SidebarTrigger className="text-muted-foreground" />
                      </IconTooltip>
                      <div
                        data-tauri-drag-region
                        className="pointer-events-none flex items-center gap-1.5 text-sm text-muted-foreground"
                      >
                        <span>RobinX</span>
                        <ChevronRight className="size-3.5" />
                        {/* 새 제목만 떠오르고 옛 제목은 바로 사라진다. 나가는 애니메이션을 두면 탭을 빠르게 바꿀 때
                          사라지는 중인 제목들이 겹겹이 쌓여 보였다. */}
                        <motion.span
                          key={page}
                          className="font-medium text-foreground"
                          initial={{ opacity: 0, y: 4 }}
                          animate={{ opacity: 1, y: 0 }}
                          transition={{ duration: 0.18, ease: EASE_OUT_QUINT }}
                        >
                          {titles[page]}
                        </motion.span>
                      </div>
                      <div data-tauri-drag-region className="h-full flex-1" />
                      <ThemeToggle />
                      <div className="mx-1.5 h-4 w-px bg-border" />
                      <WindowControls />
                    </header>
                    <main
                      className="relative flex-1 overflow-y-auto pt-12"
                      onScroll={(e) => setScrolled(e.currentTarget.scrollTop > 4)}
                    >
                      {/* 목록 페이지는 숨기기만 해서 스크롤·검색 상태를 유지한다. */}
                      <div hidden={page !== "programs"} className="min-h-full">
                        <ProgramsPage />
                      </div>
                      {page === "leftovers" && <LeftoversPage />}
                      {page === "restore" && <RestorePage />}
                      {page === "storage" && <StoragePage />}
                      {page === "startup" && <StartupPage />}
                      {page === "tracking" && <TrackingPage />}
                      {page === "settings" && <SettingsPage />}
                    </main>
                  </SidebarInset>
                </SidebarProvider>
              </motion.div>
              <Toaster position="bottom-right" />
              <StartupSplash onDone={onSplashDone} />
            </UpdatesProvider>
          </TooltipProvider>
        </MotionConfig>
      </ThemeProvider>
    </QueryClientProvider>
  )
}
