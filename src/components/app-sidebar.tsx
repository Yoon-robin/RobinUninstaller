import { ArchiveRestore, HardDrive, Package, Radar, Rocket, Settings, ShieldCheck, Sparkles } from "lucide-react"

import { motion } from "motion/react"

import { Logo } from "@/components/logo"
import {
  Sidebar,
  SidebarContent,
  SidebarFooter,
  SidebarGroup,
  SidebarGroupContent,
  SidebarGroupLabel,
  SidebarHeader,
  SidebarMenu,
  SidebarMenuBadge,
  SidebarMenuButton,
  SidebarMenuItem,
} from "@/components/ui/sidebar"

export type Page = "programs" | "leftovers" | "storage" | "startup" | "tracking" | "restore" | "settings"

type Item = { page: Page; label: string; icon: typeof Package; soon?: boolean }

const mainItems: Item[] = [
  { page: "programs", label: "프로그램", icon: Package },
  { page: "leftovers", label: "잔여물 정리", icon: Sparkles },
  { page: "storage", label: "저장공간", icon: HardDrive },
  { page: "startup", label: "시작프로그램", icon: Rocket },
  { page: "tracking", label: "설치 추적", icon: Radar },
]

const toolItems: Item[] = [
  { page: "restore", label: "복구 기록", icon: ArchiveRestore },
  { page: "settings", label: "설정", icon: Settings },
]

export function AppSidebar({ page, onNavigate }: { page: Page; onNavigate: (page: Page) => void }) {
  const renderItems = (items: Item[]) =>
    items.map((item) => (
      <SidebarMenuItem key={item.page}>
        <SidebarMenuButton
          isActive={page === item.page}
          onClick={() => onNavigate(item.page)}
          tooltip={item.label}
          className="relative isolate h-9 rounded-[0.625rem] data-[active=true]:bg-transparent data-[active=true]:font-semibold"
        >
          {page === item.page && (
            // 선택 표시(둥근 알약)가 메뉴 사이를 미끄러지듯 옮겨 간다.
            <motion.span
              layoutId="sidebar-active"
              className="absolute inset-0 -z-10 rounded-[0.625rem] bg-sidebar-accent shadow-[inset_0_0_0_1px_var(--glass-edge)]"
              transition={{ type: "spring", duration: 0.4, bounce: 0.12 }}
            />
          )}
          <item.icon className={page === item.page ? "text-sidebar-primary" : undefined} />
          <span>{item.label}</span>
        </SidebarMenuButton>
        {item.soon && (
          <SidebarMenuBadge className="rounded-full bg-sidebar-foreground/5 px-1.5 text-[10px] font-normal text-muted-foreground">
            준비 중
          </SidebarMenuBadge>
        )}
      </SidebarMenuItem>
    ))

  return (
    // 창 가장자리에서 떨어져 떠 있는 유리 사이드바 (macOS 스타일)
    <Sidebar collapsible="icon" variant="floating">
      <SidebarHeader data-tauri-drag-region className="h-[4.5rem] justify-center">
        <div data-tauri-drag-region className="flex items-center gap-2.5 px-1">
          <Logo className="size-8 shrink-0" />
          <div className="grid leading-tight group-data-[collapsible=icon]:hidden">
            <span className="font-semibold tracking-tight">RobinX</span>
            <span className="text-xs text-muted-foreground">v0.1.0 · 미리보기</span>
          </div>
        </div>
      </SidebarHeader>
      <SidebarContent>
        <SidebarGroup>
          <SidebarGroupLabel>관리</SidebarGroupLabel>
          <SidebarGroupContent>
            <SidebarMenu>{renderItems(mainItems)}</SidebarMenu>
          </SidebarGroupContent>
        </SidebarGroup>
        <SidebarGroup>
          <SidebarGroupLabel>도구</SidebarGroupLabel>
          <SidebarGroupContent>
            <SidebarMenu>{renderItems(toolItems)}</SidebarMenu>
          </SidebarGroupContent>
        </SidebarGroup>
      </SidebarContent>
      <SidebarFooter className="p-3 group-data-[collapsible=icon]:hidden">
        <div className="flex items-start gap-2.5 rounded-xl bg-sidebar-accent/60 p-3 shadow-[inset_0_0_0_1px_var(--glass-edge)]">
          <ShieldCheck className="mt-0.5 size-4 shrink-0 text-primary" />
          <div className="text-xs leading-relaxed">
            <p className="font-medium">안전하게 정리해요</p>
            <p className="text-muted-foreground">지운 것은 모두 백업돼 복구 기록에서 되돌릴 수 있어요.</p>
          </div>
        </div>
      </SidebarFooter>
    </Sidebar>
  )
}
