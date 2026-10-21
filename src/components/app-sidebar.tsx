import {
  ArchiveRestore,
  ArrowDownCircle,
  HardDrive,
  Package,
  Rocket,
  Settings,
  ShieldCheck,
  Sparkles,
} from "lucide-react"

import { motion } from "motion/react"

import { Logo } from "@/components/logo"
import { useUpdates } from "@/features/updates/updates"
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

export type Page = "programs" | "leftovers" | "storage" | "startup" | "restore" | "settings"

type Item = { page: Page; label: string; icon: typeof Package; soon?: boolean }

const mainItems: Item[] = [
  { page: "programs", label: "프로그램", icon: Package },
  { page: "leftovers", label: "잔여물 정리", icon: Sparkles },
  { page: "storage", label: "저장공간", icon: HardDrive },
  { page: "startup", label: "시작프로그램", icon: Rocket },
]

const toolItems: Item[] = [
  { page: "restore", label: "복구 기록", icon: ArchiveRestore },
  { page: "settings", label: "설정", icon: Settings },
]

export function AppSidebar({ page, onNavigate }: { page: Page; onNavigate: (page: Page) => void }) {
  const { version, available, openUpdate } = useUpdates()
  const renderItems = (items: Item[]) =>
    items.map((item) => (
      <SidebarMenuItem key={item.page}>
        <SidebarMenuButton
          isActive={page === item.page}
          aria-current={page === item.page ? "page" : undefined}
          onClick={() => onNavigate(item.page)}
          tooltip={item.label}
          // 마우스를 올린 항목은 선택된 항목(진한 알약)보다 옅게
          className="relative isolate h-9 rounded-lg hover:bg-sidebar-accent/45 data-[active=true]:bg-transparent data-[active=true]:font-semibold data-[active=true]:hover:bg-transparent"
        >
          {page === item.page && (
            // 선택 표시(둥근 알약)가 메뉴 사이를 미끄러지듯 옮겨 간다.
            <motion.span
              layoutId="sidebar-active"
              className="absolute inset-0 -z-10 rounded-lg bg-sidebar-accent"
              transition={{ type: "spring", duration: 0.32, bounce: 0 }}
            />
          )}
          <item.icon className={page === item.page ? "text-sidebar-primary" : undefined} />
          <span>{item.label}</span>
        </SidebarMenuButton>
        {item.soon && (
          <SidebarMenuBadge className="rounded-full bg-sidebar-foreground/5 px-1.5 text-[11px] font-normal text-muted-foreground">
            준비 중
          </SidebarMenuBadge>
        )}
      </SidebarMenuItem>
    ))

  return (
    // 창 가장자리에서 떨어져 떠 있는 유리 사이드바 (macOS 스타일)
    <Sidebar collapsible="icon" variant="sidebar">
      <SidebarHeader data-tauri-drag-region className="h-[4.5rem] justify-center">
        {/* 로고와 글자 위에서도 창이 끌리도록 마우스를 아래 끌기 영역으로 넘긴다. 접히면 로고를 아이콘과 같은 선에 둔다. */}
        <div
          data-tauri-drag-region
          className="flex items-center gap-2.5 px-1 group-data-[collapsible=icon]:px-0 [&>*]:pointer-events-none"
        >
          <Logo className="size-8 shrink-0" />
          <div className="grid leading-tight group-data-[collapsible=icon]:hidden">
            <span className="font-semibold tracking-tight">RobinX</span>
            <span className="text-xs text-muted-foreground">{version ? `v${version} · ` : ""}미리보기</span>
          </div>
        </div>
      </SidebarHeader>
      {/* 화면 읽기 프로그램이 앱 메뉴로 찾을 수 있게 */}
      <SidebarContent role="navigation" aria-label="앱 메뉴">
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
      <SidebarFooter className="gap-2 p-3 group-data-[collapsible=icon]:p-2">
        {/* 접힌 사이드바: 카드는 숨기고, 새 버전이 있으면 점이 붙은 아이콘으로 알린다 */}
        {available && (
          <SidebarMenu className="hidden group-data-[collapsible=icon]:flex">
            <SidebarMenuItem>
              <SidebarMenuButton
                onClick={openUpdate}
                tooltip={`새 버전이 나왔어요 (${available})`}
                className="relative h-9 rounded-lg"
              >
                <ArrowDownCircle className="text-primary" />
                <span>새 버전 설치</span>
                <span aria-hidden className="absolute top-1.5 right-1.5 size-2 rounded-full bg-primary" />
              </SidebarMenuButton>
            </SidebarMenuItem>
          </SidebarMenu>
        )}
        <div className="flex flex-col gap-2 group-data-[collapsible=icon]:hidden">
          {available && (
            <button
              type="button"
              onClick={openUpdate}
              className="flex items-start gap-2.5 rounded-xl bg-primary/10 p-3 text-left border transition-colors hover:bg-primary/15"
            >
              <ArrowDownCircle className="mt-0.5 size-4 shrink-0 text-primary" />
              <span className="text-xs leading-relaxed">
                <span className="block font-medium">새 버전이 나왔어요 ({available})</span>
                <span className="block text-muted-foreground">눌러서 설치할 수 있어요. 설정과 백업은 그대로예요.</span>
              </span>
            </button>
          )}
          <div className="flex items-start gap-2.5 rounded-xl bg-sidebar-accent/60 p-3 border">
            <ShieldCheck className="mt-0.5 size-4 shrink-0 text-primary" />
            <div className="text-xs leading-relaxed">
              <p className="font-medium">안전하게 정리해요</p>
              <p className="text-muted-foreground">
                남은 흔적은 백업한 뒤 정리해요. 백업을 보관하는 동안 복구 기록에서 되돌릴 수 있어요.
              </p>
            </div>
          </div>
        </div>
      </SidebarFooter>
    </Sidebar>
  )
}
