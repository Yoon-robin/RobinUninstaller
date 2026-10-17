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

export const countStatus = (op: Operation, status: ItemStatus) => op.items.filter((i) => i.status === status).length

// crates/startup 의 StartupEntry 와 맞춰야 한다.
export type StartupSource = "runUser" | "runMachine" | "runMachine32" | "folderUser" | "folderCommon" | "task"

export interface StartupEntry {
  id: string
  name: string
  source: StartupSource
  location: string
  approvedName: string
  command: string
  targetPath: string | null
  targetMissing: boolean
  publisher: string | null
  description: string | null
  /** 예약 작업을 만든 이 (작업에만 있음) */
  author: string | null
  /** 바꾸려면 관리자 권한(UAC)이 필요해 보임 */
  needsAdmin: boolean
  enabled: boolean
  /** 끈 시각 (유닉스 초) */
  disabledAt: number | null
  /** RobinUninstaller가 끈 항목이라 원래 값으로 되돌릴 수 있음 */
  changedByUs: boolean
}

/** HKLM·공용 시작 폴더 항목과 다른 계정으로 실행되는 예약 작업은 바꿀 때 관리자 권한(UAC)이 필요하다. */
export const isMachineStartup = (e: StartupEntry) => e.needsAdmin

export const startupApi = {
  list: () => invoke<StartupEntry[]>("list_startup_items"),
  icon: (id: string) => invoke<string | null>("get_startup_icon", { id }),
  setEnabled: (id: string, enabled: boolean) => invoke<StartupEntry>("set_startup_enabled", { id, enabled }),
  reveal: (id: string) => invoke<void>("reveal_startup_item", { id }),
}

/** 다른 화면으로 이동 요청 (App이 받아 처리). */
export function navigate(page: "programs" | "leftovers" | "restore") {
  window.dispatchEvent(new CustomEvent("robin:navigate", { detail: page }))
}
