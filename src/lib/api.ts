import { Channel, invoke } from "@tauri-apps/api/core"

// crates/core 의 InstalledApplication 과 맞춰야 한다 (serde camelCase).
export type InstallScope = "machine64" | "machine32" | "user"
export type InstallTechnology = "exe" | "msi"
export type UninstallerStatus = "available" | "missing" | "notRegistered" | "unknown"

export interface InstalledApplication {
  id: string
  displayName: string
  publisher: string | null
  version: string | null
  installLocation: string | null
  installDate: string | null
  estimatedSizeKb: number | null
  displayIcon: string | null
  uninstallCommand: string | null
  quietUninstallCommand: string | null
  productCode: string | null
  technology: InstallTechnology
  scope: InstallScope
  uninstaller: UninstallerStatus
}

// crates/execution 의 UninstallEvent / UninstallOutcome 과 맞춰야 한다.
export type UninstallEvent =
  { kind: "launching"; file: string } | { kind: "running"; processCount: number } | { kind: "verifying" }

export type UninstallOutcome =
  | { kind: "removed" }
  | { kind: "removedRebootRequired" }
  | { kind: "stillInstalled"; exitCode: number | null }
  | { kind: "elevationDenied" }
  | { kind: "failed"; message: string }

export const api = {
  listInstalledApps: () => invoke<InstalledApplication[]>("list_installed_apps"),
  getAppIcon: (id: string) => invoke<string | null>("get_app_icon", { id }),
  openInstallLocation: (id: string) => invoke<void>("open_install_location", { id }),
  uninstallApp: (id: string, onEvent: (event: UninstallEvent) => void) => {
    const channel = new Channel<UninstallEvent>()
    channel.onmessage = onEvent
    return invoke<UninstallOutcome>("uninstall_app", { id, onEvent: channel })
  },
  stopWaitingUninstall: () => invoke<void>("stop_waiting_uninstall"),
}

/** 실제로 실행될 명령 (화면 표시용). crates/execution 의 plan_for 와 같은 규칙. */
export function uninstallCommandPreview(app: InstalledApplication): string | null {
  if (app.technology === "msi" && app.productCode) return `msiexec.exe /x ${app.productCode}`
  return app.uninstallCommand
}

/** 기본 언인스톨러를 실행할 수 있는지. 언인스톨러 파일이 없으면 강제 제거(이후 버전) 대상이다. */
export function canUninstall(app: InstalledApplication) {
  return app.uninstaller === "available" || app.uninstaller === "unknown"
}

// crates/scanner 의 Leftover / ScanReport 와 맞춰야 한다.
export type LeftoverKind = "folder" | "file" | "registryKey" | "registryValue"
export type Confidence = "low" | "review" | "strong"

export type Evidence =
  | { kind: "installLocation" }
  | { kind: "pointsIntoInstallLocation" }
  | { kind: "uninstallEntry" }
  | { kind: "msiProductCode" }
  | { kind: "publisherAndProduct" }
  | { kind: "publisherAndPartialName" }
  | { kind: "exactName" }
  | { kind: "partialName" }
  | { kind: "emptyPublisherFolder" }
  | { kind: "sharedWithOtherApp"; appName: string }
  | { kind: "containsUserDocuments"; count: number }

export interface Leftover {
  id: string
  kind: LeftoverKind
  location: string
  valueName: string | null
  sizeBytes: number | null
  evidence: Evidence[]
  score: number
  confidence: Confidence
  note: string | null
}

export interface ScanReport {
  appName: string
  items: Leftover[]
  protectedSkipped: number
  durationMs: number
}

export const scanApi = {
  scanLeftovers: (id: string) => invoke<ScanReport>("scan_leftovers", { id }),
  recentlyRemoved: () => invoke<InstalledApplication[]>("recently_removed"),
  revealLeftover: (itemId: string) => invoke<void>("reveal_leftover", { itemId }),
}

// crates/recovery 의 Operation 과 맞춰야 한다.
export type ItemStatus = "pending" | "removed" | "needsAdmin" | "alreadyGone" | "blocked" | "failed" | "restored"

export interface OperationItem {
  id: string
  kind: LeftoverKind
  location: string
  valueName: string | null
  sizeBytes: number | null
  status: ItemStatus
  error: string | null
}

export interface Operation {
  id: string
  appName: string
  createdAt: number
  items: OperationItem[]
}

export const recoveryApi = {
  clean: (appName: string, itemIds: string[]) => invoke<Operation>("clean_leftovers", { appName, itemIds }),
  cleanElevated: (opId: string) => invoke<Operation>("clean_leftovers_elevated", { opId }),
  list: () => invoke<Operation[]>("list_backups"),
  restore: (opId: string, itemIds?: string[]) =>
    invoke<Operation>("restore_backup", { opId, itemIds: itemIds ?? null }),
  restoreElevated: (opId: string) => invoke<Operation>("restore_backup_elevated", { opId }),
  deleteBackup: (opId: string) => invoke<void>("delete_backup", { opId }),
}

// crates/startup 의 StartupItem 과 맞춰야 한다.
export type StartupSource = "runUser" | "runMachine" | "runMachine32" | "folderUser" | "folderCommon"

export interface StartupItem {
  id: string
  name: string
  source: StartupSource
  location: string
  command: string
  filePath: string | null
  fileMissing: boolean
  publisher: string | null
  appName: string | null
  enabled: boolean
  needsAdmin: boolean
}

export const startupApi = {
  list: () => invoke<StartupItem[]>("list_startup_items"),
  /** 모든 사용자 항목은 UAC 창이 뜬다. 바뀐 항목을 돌려준다. */
  setEnabled: (id: string, enabled: boolean) => invoke<StartupItem>("set_startup_enabled", { id, enabled }),
  icon: (id: string) => invoke<string | null>("get_startup_icon", { id }),
  reveal: (id: string) => invoke<void>("reveal_startup_item", { id }),
}

export const countStatus = (op: Operation, status: ItemStatus) => op.items.filter((i) => i.status === status).length

/** 다른 화면으로 이동 요청 (App이 받아 처리). */
export function navigate(page: "programs" | "leftovers" | "restore") {
  window.dispatchEvent(new CustomEvent("robin:navigate", { detail: page }))
}
