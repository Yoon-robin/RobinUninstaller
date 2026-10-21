import { useCallback, useEffect, useLayoutEffect, useRef, useState } from "react"
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
import { StoragePage } from "@/features/storage/storage-page"
import { ProgramsPage } from "@/features/programs/programs-page"
import { SidebarInset, SidebarProvider, SidebarTrigger } from "@/components/ui/sidebar"
import { Toaster } from "@/components/ui/sonner"
import { TooltipProvider } from "@/components/ui/tooltip"
import { cn } from "@/lib/utils"
import { EASE_OUT_QUINT } from "@/components/motion"
import { isBusy } from "@/lib/busy"
import { getCurrentWindow } from "@tauri-apps/api/window"
import { toast } from "sonner"

// retry: false — 모두 이 PC를 읽는 작업이라 다시 해도 결과가 같다. 기본값(3번 다시)이면 실패한 스캔을
// 오류를 보여 주기 전에 몇 초씩 세 번 더 돈다.
// refetchOnReconnect: false — 인터넷과 상관없는 작업이다. 켜 두면 연결이 끊겼다 이어질 때 남은 흔적 찾기가
// 다시 돌아, 확인 화면이나 정리 중인 창이 처음으로 돌아가고 고른 것을 잃었다.
const queryClient = new QueryClient({
  defaultOptions: {
    queries: { refetchOnWindowFocus: false, refetchOnReconnect: false, staleTime: 60_000, retry: false },
  },
})

const titles: Record<Page, string> = {
  programs: "프로그램",
  leftovers: "잔여물 정리",
  storage: "저장공간",
  startup: "시작프로그램",
  restore: "복구 기록",
  settings: "설정",
}

export default function App() {
  const [page, setPage] = useState<Page>("programs")
  // 내용을 내리면 제목 표시줄 아래에 구분선을 그린다.
  const [scrolled, setScrolled] = useState(false)
  // 시작 화면이 걷히면 앱 화면이 살짝 커지며 자리 잡는다
  const [ready, setReady] = useState(false)
  const onSplashDone = useCallback(() => setReady(true), [])
  const reduce = useReducedMotion()
  // 사이드바를 접었는지 기억한다 (다음에 켜도 그대로)
  const [sidebarOpen, setSidebarOpen] = useState(() => {
    try {
      return localStorage.getItem("robin.sidebar") !== "closed"
    } catch {
      return true
    }
  })
  const changeSidebar = useCallback((open: boolean) => {
    setSidebarOpen(open)
    try {
      localStorage.setItem("robin.sidebar", open ? "open" : "closed")
    } catch {
      // 저장하지 못해도 이번 실행에서는 그대로 쓴다
    }
  }, [])

  // 페이지마다 스크롤 위치를 따로 기억한다. 모든 페이지가 <main> 하나를 같이 써서, 그대로 두면 프로그램 목록을
  // 내린 채 설정으로 가면 설정이 중간부터 열린다. 처음 여는 페이지는 맨 위에서 시작한다.
  const main = useRef<HTMLElement>(null)
  // 제목 줄이 시작하는 곳(사이드바 오른쪽 끝). 대화상자 덮개가 이 띠만 비워 둔다 (index.css .below-titlebar).
  const titlebar = useRef<HTMLElement>(null)
  useLayoutEffect(() => {
    const el = titlebar.current
    if (!el) return
    const update = () =>
      document.documentElement.style.setProperty("--titlebar-left", `${el.getBoundingClientRect().left}px`)
    update()
    const observer = new ResizeObserver(update)
    observer.observe(el)
    return () => observer.disconnect()
  }, [])
  const positions = useRef<Partial<Record<Page, number>>>({})
  const restoring = useRef(false)
  useLayoutEffect(() => {
    const el = main.current
    if (!el) return
    const target = positions.current[page] ?? 0
    restoring.current = true
    el.scrollTop = target
    setScrolled(el.scrollTop > 4)
    // 새로 그린 페이지는 내용이 늦게 차서 처음에는 그만큼 내려가지 못할 수 있다. 잠깐 동안 다시 맞춘다.
    let frame = 0
    const started = performance.now()
    const retry = () => {
      if (el.scrollTop < target - 1 && performance.now() - started < 600) {
        el.scrollTop = target
        frame = requestAnimationFrame(retry)
      } else {
        restoring.current = false
        setScrolled(el.scrollTop > 4)
      }
    }
    frame = requestAnimationFrame(retry)
    // 사용자가 그 사이 직접 굴리면 그만둔다
    const stop = () => {
      cancelAnimationFrame(frame)
      restoring.current = false
    }
    el.addEventListener("wheel", stop, { passive: true, once: true })
    el.addEventListener("keydown", stop, { once: true })
    return () => {
      cancelAnimationFrame(frame)
      restoring.current = false
      el.removeEventListener("wheel", stop)
      el.removeEventListener("keydown", stop)
    }
  }, [page])
  // 정리·되돌리기·제거 중에 창을 닫으면(X, Alt+F4) 한 번 막고 묻는다. 그래도 닫겠다면 그때 닫는다.
  useEffect(() => {
    const win = getCurrentWindow()
    const unlisten = win.onCloseRequested((e) => {
      if (!isBusy()) return
      e.preventDefault()
      toast.warning("작업이 끝난 뒤에 닫아 주세요", {
        id: "close-while-busy",
        description: "정리·되돌리기·제거가 진행 중이에요. 지금 닫으면 중간에 멈출 수 있어요.",
        action: { label: "그래도 닫기", onClick: () => void win.destroy() },
      })
    })
    return () => {
      unlisten.then((off) => off()).catch(() => {})
    }
  }, [])
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
                <SidebarProvider className="h-svh" open={sidebarOpen} onOpenChange={changeSidebar}>
                  <AppSidebar page={page} onNavigate={setPage} />
                  <SidebarInset className="relative h-svh min-w-0 overflow-hidden bg-transparent">
                    {/* 제목 표시줄: 빈 곳을 끌면 창이 움직이고, 더블클릭하면 최대화된다. 스크롤하는 내용 위쪽에 고정돼 있다. */}
                    <header
                      data-tauri-drag-region
                      // 대화상자·시트는 이 줄 아래에만 덮인다. 열려 있는 동안에도 창을 끌고 줄이고 닫을 수 있게
                      // (모달이 body에 거는 pointer-events: none을 이 줄만 되돌린다).
                      data-app-titlebar
                      ref={titlebar}
                      className={cn(
                        "pointer-events-auto",
                        // 불투명한 바탕: 스크롤한 내용이 제목 줄 뒤로 비치지 않는다. 스크롤하면 아래 구분선만 보인다.
                        "relative z-20 flex h-12 shrink-0 items-center gap-1 bg-background pl-2 shadow-[0_1px_0_transparent] transition-shadow duration-200",
                        scrolled && "shadow-[0_1px_0_var(--border)]",
                      )}
                    >
                      <IconTooltip label="사이드바 접기/펼치기">
                        <SidebarTrigger className="size-8 rounded-md text-muted-foreground hover:bg-foreground/8 hover:text-foreground" />
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
                      <div className="ml-1.5 h-4 w-px bg-border" />
                      <WindowControls />
                    </header>
                    {/* 스크롤은 제목 줄 아래에서 시작한다 (제목 줄이 스크롤바 위쪽을 가리지 않게) */}
                    <main
                      ref={main}
                      className="relative min-h-0 flex-1 scroll-pt-4 scroll-pb-4 overflow-y-auto"
                      onScroll={(e) => {
                        const top = e.currentTarget.scrollTop
                        if (!restoring.current) positions.current[page] = top
                        setScrolled(top > 4)
                      }}
                    >
                      {/* 목록 페이지는 숨기기만 해서 스크롤·검색 상태를 유지한다. */}
                      <div hidden={page !== "programs"} className="min-h-full">
                        <ProgramsPage />
                      </div>
                      {page === "leftovers" && <LeftoversPage />}
                      {page === "restore" && <RestorePage />}
                      {page === "storage" && <StoragePage />}
                      {page === "startup" && <StartupPage />}
                      {page === "settings" && <SettingsPage />}
                    </main>
                  </SidebarInset>
                </SidebarProvider>
              </motion.div>
              <Toaster position="bottom-right" containerAriaLabel="알림" />
              <StartupSplash onDone={onSplashDone} />
            </UpdatesProvider>
          </TooltipProvider>
        </MotionConfig>
      </ThemeProvider>
    </QueryClientProvider>
  )
}
