import { useState } from "react"
import { QueryClient, QueryClientProvider } from "@tanstack/react-query"
import { ChevronRight } from "lucide-react"
import { AnimatePresence, MotionConfig, motion } from "motion/react"

import { AppSidebar, type Page } from "@/components/app-sidebar"
import { IconTooltip } from "@/components/icon-tooltip"
import { ThemeProvider, ThemeToggle } from "@/components/theme"
import { WindowControls } from "@/components/window-controls"
import { ComingSoonPage } from "@/features/coming-soon/coming-soon-page"
import { LeftoversPage } from "@/features/leftovers/leftovers-page"
import { ProgramsPage } from "@/features/programs/programs-page"
import { SidebarInset, SidebarProvider, SidebarTrigger } from "@/components/ui/sidebar"
import { Toaster } from "@/components/ui/sonner"
import { TooltipProvider } from "@/components/ui/tooltip"

const queryClient = new QueryClient({
  defaultOptions: { queries: { refetchOnWindowFocus: false, staleTime: 60_000 } },
})

const titles: Record<Page, string> = {
  programs: "프로그램",
  leftovers: "잔여물 정리",
  startup: "시작프로그램",
  tracking: "설치 추적",
  restore: "복구 기록",
  settings: "설정",
}

export default function App() {
  const [page, setPage] = useState<Page>("programs")

  return (
    <QueryClientProvider client={queryClient}>
      <ThemeProvider>
        {/* OS의 "애니메이션 효과 줄이기" 설정을 따른다. */}
        <MotionConfig reducedMotion="user">
          {/* 첫 툴팁은 잠깐 기다렸다가, 이어서 옮겨 다닐 때는 바로 뜬다. */}
          <TooltipProvider delayDuration={400} skipDelayDuration={300}>
            <SidebarProvider className="h-svh">
              <AppSidebar page={page} onNavigate={setPage} />
              <SidebarInset className="h-svh min-w-0 overflow-hidden">
                {/* 제목 표시줄: 빈 곳을 끌면 창이 움직이고, 더블클릭하면 최대화된다. */}
                <header data-tauri-drag-region className="flex h-11 shrink-0 items-center gap-1 border-b pl-2">
                  <IconTooltip label="사이드바 접기/펼치기">
                    <SidebarTrigger className="text-muted-foreground" />
                  </IconTooltip>
                  <div
                    data-tauri-drag-region
                    className="pointer-events-none flex items-center gap-1.5 text-sm text-muted-foreground"
                  >
                    <span>RobinUninstaller</span>
                    <ChevronRight className="size-3.5" />
                    <AnimatePresence mode="popLayout" initial={false}>
                      <motion.span
                        key={page}
                        className="font-medium text-foreground"
                        initial={{ opacity: 0, y: 6, filter: "blur(3px)" }}
                        animate={{ opacity: 1, y: 0, filter: "blur(0px)" }}
                        exit={{ opacity: 0, y: -4, filter: "blur(3px)", transition: { duration: 0.12 } }}
                        transition={{ type: "spring", duration: 0.35, bounce: 0 }}
                      >
                        {titles[page]}
                      </motion.span>
                    </AnimatePresence>
                  </div>
                  <div data-tauri-drag-region className="h-full flex-1" />
                  <ThemeToggle />
                  <div className="mx-1.5 h-4 w-px bg-border" />
                  <WindowControls />
                </header>
                <main className="relative flex-1 overflow-y-auto">
                  <div
                    aria-hidden
                    className="pointer-events-none absolute inset-x-0 top-0 h-72 bg-[radial-gradient(60%_100%_at_30%_0%,oklch(0.82_0.12_190/9%),transparent)]"
                  />
                  {/* 목록 페이지는 숨기기만 해서 스크롤·검색 상태를 유지한다. */}
                  <div hidden={page !== "programs"} className="min-h-full">
                    <ProgramsPage />
                  </div>
                  {page === "leftovers" && <LeftoversPage />}
                  {page !== "programs" && page !== "leftovers" && <ComingSoonPage page={page} />}
                </main>
              </SidebarInset>
            </SidebarProvider>
            <Toaster position="bottom-right" />
          </TooltipProvider>
        </MotionConfig>
      </ThemeProvider>
    </QueryClientProvider>
  )
}
