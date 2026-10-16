import { useState } from "react"
import { QueryClient, QueryClientProvider } from "@tanstack/react-query"
import { ChevronRight } from "lucide-react"

import { AppSidebar, type Page } from "@/components/app-sidebar"
import { WindowControls } from "@/components/window-controls"
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
      <TooltipProvider>
        <SidebarProvider className="h-svh">
          <AppSidebar page={page} onNavigate={setPage} />
          <SidebarInset className="h-svh min-w-0 overflow-hidden">
            {/* 제목 표시줄: 빈 곳을 끌면 창이 움직이고, 더블클릭하면 최대화된다. */}
            <header data-tauri-drag-region className="flex h-11 shrink-0 items-center gap-1 border-b pl-2">
              <SidebarTrigger className="text-muted-foreground" />
              <div
                data-tauri-drag-region
                className="pointer-events-none flex items-center gap-1.5 text-sm text-muted-foreground"
              >
                <span>RobinUninstaller</span>
                <ChevronRight className="size-3.5" />
                <span className="font-medium text-foreground">{titles[page]}</span>
              </div>
              <div data-tauri-drag-region className="h-full flex-1" />
              <WindowControls />
            </header>
            <main className="flex-1 overflow-y-auto">{page === "programs" && <ProgramsPage />}</main>
          </SidebarInset>
        </SidebarProvider>
        <Toaster position="bottom-right" />
      </TooltipProvider>
    </QueryClientProvider>
  )
}
