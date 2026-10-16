import { ArchiveRestore, Package, Radar, Rocket, Settings, Trash2 } from "lucide-react"

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

export type Page = "programs" | "leftovers" | "startup" | "tracking" | "restore" | "settings"

const mainItems: { page: Page; label: string; icon: typeof Package; soon?: boolean }[] = [
  { page: "programs", label: "프로그램", icon: Package },
  { page: "leftovers", label: "잔여물 정리", icon: Trash2, soon: true },
  { page: "startup", label: "시작프로그램", icon: Rocket, soon: true },
  { page: "tracking", label: "설치 추적", icon: Radar, soon: true },
]

const toolItems: typeof mainItems = [
  { page: "restore", label: "복구 기록", icon: ArchiveRestore, soon: true },
  { page: "settings", label: "설정", icon: Settings, soon: true },
]

export function AppSidebar({ page, onNavigate }: { page: Page; onNavigate: (page: Page) => void }) {
  const renderItems = (items: typeof mainItems) =>
    items.map((item) => (
      <SidebarMenuItem key={item.page}>
        <SidebarMenuButton
          isActive={page === item.page}
          disabled={item.soon}
          onClick={() => onNavigate(item.page)}
          tooltip={item.label}
        >
          <item.icon />
          <span>{item.label}</span>
        </SidebarMenuButton>
        {item.soon && <SidebarMenuBadge className="text-muted-foreground">준비 중</SidebarMenuBadge>}
      </SidebarMenuItem>
    ))

  return (
    <Sidebar collapsible="icon">
      <SidebarHeader>
        <SidebarMenu>
          <SidebarMenuItem>
            <SidebarMenuButton size="lg" className="pointer-events-none">
              <div className="flex aspect-square size-8 items-center justify-center rounded-lg bg-primary text-primary-foreground">
                <Trash2 className="size-4" />
              </div>
              <div className="grid flex-1 text-left leading-tight">
                <span className="truncate font-semibold">RobinUninstaller</span>
                <span className="truncate text-xs text-muted-foreground">v0.1.0</span>
              </div>
            </SidebarMenuButton>
          </SidebarMenuItem>
        </SidebarMenu>
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
      <SidebarFooter className="px-4 pb-3 text-xs text-muted-foreground group-data-[collapsible=icon]:hidden">
        읽기 전용 · 아직 아무것도 지우지 않아요
      </SidebarFooter>
    </Sidebar>
  )
}
