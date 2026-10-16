import { useState } from "react"
import { QueryClient, QueryClientProvider } from "@tanstack/react-query"

import { AppSidebar, type Page } from "@/components/app-sidebar"
import { ProgramsPage } from "@/features/programs/programs-page"
import { Separator } from "@/components/ui/separator"
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
        <SidebarProvider>
          <AppSidebar page={page} onNavigate={setPage} />
          <SidebarInset className="min-w-0">
            <header className="flex h-14 shrink-0 items-center gap-2 border-b px-4">
              <SidebarTrigger className="-ml-1" />
              <Separator orientation="vertical" className="mr-2 data-[orientation=vertical]:h-4" />
              <h1 className="font-semibold">{titles[page]}</h1>
            </header>
            {page === "programs" && <ProgramsPage />}
          </SidebarInset>
        </SidebarProvider>
        <Toaster />
      </TooltipProvider>
    </QueryClientProvider>
  )
}
