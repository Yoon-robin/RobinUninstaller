import type { ReactNode } from "react"
import { Copy, FolderOpen, Info, Sparkles, Trash2 } from "lucide-react"
import { toast } from "sonner"

import { api, canUninstall, type InstalledApplication } from "@/lib/api"
import {
  ContextMenu,
  ContextMenuContent,
  ContextMenuItem,
  ContextMenuLabel,
  ContextMenuSeparator,
  ContextMenuTrigger,
} from "@/components/ui/context-menu"
import {
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuSeparator,
} from "@/components/ui/dropdown-menu"

export type AppActionHandlers = {
  onSelect: (app: InstalledApplication) => void
  onUninstall: (app: InstalledApplication) => void
  onFindLeftovers: (app: InstalledApplication) => void
}

type Action = {
  key: string
  label: string
  icon: typeof Info
  run: () => void
  disabled?: boolean
  destructive?: boolean
  separatorBefore?: boolean
}

function actionsFor(
  app: InstalledApplication,
  { onSelect, onUninstall, onFindLeftovers }: AppActionHandlers,
): Action[] {
  return [
    { key: "info", label: "세부 정보", icon: Info, run: () => onSelect(app) },
    {
      key: "folder",
      label: "설치 폴더 열기",
      icon: FolderOpen,
      disabled: !app.installLocation,
      run: () => api.openInstallLocation(app.id).catch((e) => toast.error(String(e))),
    },
    {
      key: "copy",
      label: "이름 복사",
      icon: Copy,
      run: () => navigator.clipboard.writeText(app.displayName).then(() => toast.success("이름을 복사했어요")),
    },
    { key: "leftovers", label: "흔적 미리 보기", icon: Sparkles, run: () => onFindLeftovers(app) },
    {
      key: "uninstall",
      label: canUninstall(app) ? "제거" : "강제 제거 (언인스톨러 없음)",
      icon: Trash2,
      destructive: true,
      separatorBefore: true,
      run: () => onUninstall(app),
    },
  ]
}

/** '⋯' 버튼 메뉴 내용. */
export function AppDropdownContent({ app, ...handlers }: { app: InstalledApplication } & AppActionHandlers) {
  return (
    <DropdownMenuContent align="end" className="w-52">
      <DropdownMenuLabel className="truncate">{app.displayName}</DropdownMenuLabel>
      <DropdownMenuSeparator />
      {actionsFor(app, handlers).map((a) => [
        a.separatorBefore && <DropdownMenuSeparator key={`${a.key}-sep`} />,
        <DropdownMenuItem
          key={a.key}
          disabled={a.disabled}
          variant={a.destructive ? "destructive" : "default"}
          onClick={a.run}
        >
          <a.icon />
          {a.label}
        </DropdownMenuItem>,
      ])}
    </DropdownMenuContent>
  )
}

/** 오른쪽 클릭 메뉴. '⋯' 메뉴와 같은 동작을 보여 준다. */
export function AppContextMenu({
  app,
  children,
  ...handlers
}: { app: InstalledApplication; children: ReactNode } & AppActionHandlers) {
  return (
    <ContextMenu>
      <ContextMenuTrigger asChild>{children}</ContextMenuTrigger>
      <ContextMenuContent className="w-52">
        <ContextMenuLabel className="truncate">{app.displayName}</ContextMenuLabel>
        <ContextMenuSeparator />
        {actionsFor(app, handlers).map((a) => [
          a.separatorBefore && <ContextMenuSeparator key={`${a.key}-sep`} />,
          <ContextMenuItem
            key={a.key}
            disabled={a.disabled}
            variant={a.destructive ? "destructive" : "default"}
            onClick={a.run}
          >
            <a.icon />
            {a.label}
          </ContextMenuItem>,
        ])}
      </ContextMenuContent>
    </ContextMenu>
  )
}
