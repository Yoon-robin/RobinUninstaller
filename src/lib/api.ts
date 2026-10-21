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
  /** 화면 전용: 크기 기록이 없어 설치 폴더를 직접 잰 값으로 채웠는지 */
  sizeMeasured?: boolean
}

// crates/core 의 StoreApp 과 맞춰야 한다.
export type PackageSource = "store" | "sideloaded" | "developer"

export interface StoreApp {
  id: string
  displayName: string
  publisher: string | null
  version: string | null
  installDate: string | null
  installLocation: string | null
  familyName: string
  source: PackageSource
}

export const storeApi = {
  list: () => invoke<StoreApp[]>("list_store_apps"),
  icon: (id: string) => invoke<string | null>("get_store_app_icon", { id }),
  /** Windows 설정의 "설치된 앱" 화면. Store 앱 제거는 거기서 한다. */
  openSettings: () => invoke<void>("open_apps_settings"),
}

// crates/execution 의 UninstallEvent / UninstallOutcome 과 맞춰야 한다.
export type UninstallEvent =
  { kind: "launching"; file: string } | { kind: "running"; processCount: number } | { kind: "verifying" }

export type UninstallOutcome =
  | { kind: "removed" }
  | { kind: "removedRebootRequired" }
  | { kind: "stillInstalled"; exitCode: number | null }
  | { kind: "elevationDenied" }
  | { kind: "cannotRun"; reason: string; command: string | null }
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
  /** 크기 기록이 없는 프로그램의 설치 폴더 크기 (앱 ID → KB) */
  measureAppSizes: (onProgress?: (p: ScanProgress) => void) =>
    invoke<Record<string, number>>("measure_app_sizes", { onProgress: progressChannel(onProgress) }),
  /** 프로그램 목록(데스크톱 + Store 앱)을 CSV로 저장. 저장 창을 취소하면 null, 저장하면 파일 이름. */
  exportAppList: () => invoke<string | null>("export_app_list"),
  /** 프로그램 이름으로 웹 검색 (기본 브라우저) */
  searchWeb: (id: string) => invoke<void>("search_app_web", { id }),
}

/** 실제로 실행될 명령 (화면 표시용). crates/execution 의 plan_for 와 같은 규칙. */
export function uninstallCommandPreview(app: InstalledApplication): string | null {
  if (app.technology === "msi" && app.productCode) return `msiexec.exe /x ${app.productCode}`
  return app.uninstallCommand
}

/** 기본 언인스톨러를 실행할 수 있는지. 언인스톨러 파일이 없으면 강제 제거 대상이다. */
export function canUninstall(app: InstalledApplication) {
  return app.uninstaller === "available" || app.uninstaller === "unknown"
}

/** 기본 언인스톨러를 쓸 수 없어 강제 제거로 가야 하는 프로그램. 필터·경고 표시·잔여물 화면이 함께 쓴다. */
export function needsForce(app: InstalledApplication) {
  return !canUninstall(app)
}

// crates/execution 의 ForceCheck 와 맞춰야 한다.
export interface ForceCheck {
  /** 강제 제거하면 안 되는 이유 (보안 프로그램·드라이버 등) */
  blocked: string | null
  /** 설치 폴더 안에서 실행 중인 프로세스 */
  running: { pid: number; path: string }[]
}

export const forceApi = {
  check: (id: string) => invoke<ForceCheck>("force_uninstall_check", { id }),
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
  | { kind: "inUserFolder" }
  | { kind: "sharedWithOtherApp"; appName: string }
  | { kind: "containsUserDocuments"; count: number }
  | { kind: "tooLargeToCheck" }

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
export type ItemStatus =
  | "pending"
  | "removed"
  | "needsAdmin"
  | "alreadyGone"
  | "blocked"
  | "failed"
  | "restored"
  /** 일부만 정리됨: 백업은 온전하고 원래 자리에 일부가 남음 (다시 정리하거나 되돌릴 수 있다) */
  | "partial"
  /** 되돌렸지만 내용이 다른 파일이 있어 일부는 덮어쓰지 않음. 그 파일은 백업에만 있다 */
  | "conflicted"

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
  /** `forceAppId`: 강제 제거 중이면 대상 프로그램 ID (등록 정보가 사라지면 "제거한 프로그램"으로 옮긴다). */
  /** `appId`: 스캔한 프로그램. 엔진은 같은 프로그램의 마지막 스캔 결과에 있는 항목만 정리한다. */
  /** `onProgress`: 지금 옮기는 항목, 복사한 파일 수 (다른 드라이브는 복사라 오래 걸릴 수 있다) */
  clean: (
    appId: string,
    appName: string,
    itemIds: string[],
    forceAppId?: string,
    onProgress?: (p: ScanProgress) => void,
  ) =>
    invoke<Operation>("clean_leftovers", {
      appId,
      appName,
      itemIds,
      forceAppId: forceAppId ?? null,
      onProgress: progressChannel(onProgress),
    }),
  cleanElevated: (opId: string, forceAppId?: string) =>
    invoke<Operation>("clean_leftovers_elevated", { opId, forceAppId: forceAppId ?? null }),
  list: () => invoke<Operation[]>("list_backups"),
  /** 기록을 읽지 못한 백업 폴더 이름 (손상·권한 문제). 백업은 지우지 않는다. */
  unreadable: () => invoke<string[]>("unreadable_backups"),
  openFolder: () => invoke<void>("open_backup_folder"),
  /** 한 작업의 백업 폴더 (충돌로 백업에만 남은 파일을 꺼낼 때) */
  openOperationFolder: (opId: string) => invoke<void>("open_backup_operation", { opId }),
  restore: (opId: string, itemIds?: string[], onProgress?: (p: ScanProgress) => void) =>
    invoke<Operation>("restore_backup", { opId, itemIds: itemIds ?? null, onProgress: progressChannel(onProgress) }),
  /** 관리자 권한으로 되돌린다. `itemIds`가 없으면 되돌릴 수 있는 항목 전부. */
  restoreElevated: (opId: string, itemIds?: string[]) =>
    invoke<Operation>("restore_backup_elevated", { opId, itemIds: itemIds ?? null }),
  deleteBackup: (opId: string, onProgress?: (p: ScanProgress) => void) =>
    invoke<void>("delete_backup", { opId, onProgress: progressChannel(onProgress) }),
}

export const countStatus = (op: Operation, status: ItemStatus) => op.items.filter((i) => i.status === status).length

/** 복구 기록에서 되돌릴 수 있는 항목 (정리했거나 일부만 정리한 항목) */
export const isRestorable = (status: ItemStatus) => status === "removed" || status === "partial"

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
  /** RobinX가 끈 항목이라 원래 값으로 되돌릴 수 있음 */
  changedByUs: boolean
}

/** HKLM·공용 시작 폴더 항목과 다른 계정으로 실행되는 예약 작업은 바꿀 때 관리자 권한(UAC)이 필요하다. */
export const isMachineStartup = (e: StartupEntry) => e.needsAdmin

export const startupApi = {
  list: () => invoke<StartupEntry[]>("list_startup_items"),
  /** 관리자 권한(UAC)으로만 보이는 예약 작업. 읽기만 한다. */
  hiddenTasks: () => invoke<StartupEntry[]>("list_hidden_startup_tasks"),
  icon: (id: string) => invoke<string | null>("get_startup_icon", { id }),
  setEnabled: (id: string, enabled: boolean) => invoke<StartupEntry>("set_startup_enabled", { id, enabled }),
  reveal: (id: string) => invoke<void>("reveal_startup_item", { id }),
  /** 실행 파일이 없는 항목을 백업 후 정리한다. 관리자 항목은 `recoveryApi.cleanElevated`로 이어서. */
  cleanMissing: (ids: string[]) => invoke<Operation>("clean_missing_startup", { ids }),
}

// crates/storage 의 Category / CleanResult / FolderEntry 와 맞춰야 한다.
export type StorageRisk = "regenerable" | "irreversible"
export type StorageKind = "files" | "recycleBin" | "info"

export interface StorageCategory {
  id: string
  name: string
  description: string
  why: string
  risk: StorageRisk
  kind: StorageKind
  admin: boolean
  defaultSelected: boolean
  bytes: number
  files: number
  /** 권한이 없어 크기를 다 세지 못한 폴더 수 */
  inaccessible: number
  locations: string[]
  /** 실행 중이라 지금은 정리할 수 없게 만드는 프로그램 */
  running: string[]
  guidance: string | null
}

export interface CleanResult {
  id: string
  freedBytes: number
  deletedFiles: number
  failedFiles: number
  skipped: string | null
  needsAdmin: boolean
}

/** 정리하는 동안 오는 소식 (crates/storage 의 CleanEvent) */
export type CleanEvent =
  | { kind: "started"; id: string; index: number; total: number }
  | { kind: "progress"; id: string; freedBytes: number; deletedFiles: number; failedFiles: number; current: string }
  | { kind: "finished"; result: CleanResult }

export interface DiskUsage {
  drive: string
  total: number
  free: number
}

export interface FolderItem {
  id: string
  name: string
  path: string
  isDir: boolean
  bytes: number
  files: number
  inaccessible: number
  /** Windows 폴더 안: 하드 링크 때문에 실제보다 크게 보일 수 있음 */
  approximate: boolean
}

export interface LargeFile {
  id: string
  name: string
  path: string
  bytes: number
  /** 마지막으로 고친 때 (유닉스 초) */
  modified: number | null
}

export interface LargeFiles {
  root: string
  minBytes: number
  items: LargeFile[]
  cancelled: boolean
}

export interface FolderAnalysis {
  id: string
  path: string
  parentId: string | null
  items: FolderItem[]
  cancelled: boolean
}

/** 오래 걸리는 탐색의 진행 상황 (Rust `ProgressSnapshot`). 0.15초마다, 바뀐 것만 온다. */
export interface ScanProgress {
  /** 지금까지 살펴본 파일 수와 크기 */
  files: number
  bytes: number
  /** 끝낸 단위(항목·폴더) 수와 전체 수. 전체를 모르면 0 */
  done: number
  total: number
  /** 지금 살펴보는 곳 (항목 이름이나 폴더 경로) */
  current: string
}

function progressChannel(onProgress?: (p: ScanProgress) => void) {
  const channel = new Channel<ScanProgress>()
  if (onProgress) channel.onmessage = onProgress
  return channel
}

export const storageApi = {
  diskUsage: () => invoke<DiskUsage>("disk_usage"),
  /** 정리할 수 있는 항목. `onItem`은 항목을 하나 찾을 때마다 (다 찾기 전에 먼저 보여 주려고) */
  scan: (onItem?: (item: StorageCategory) => void, onProgress?: (p: ScanProgress) => void) => {
    const channel = new Channel<StorageCategory>()
    if (onItem) channel.onmessage = onItem
    return invoke<StorageCategory[]>("scan_storage", { onItem: channel, onProgress: progressChannel(onProgress) })
  },
  clean: (ids: string[], allowIrreversible: boolean, onEvent: (event: CleanEvent) => void) => {
    const channel = new Channel<CleanEvent>()
    channel.onmessage = onEvent
    return invoke<CleanResult[]>("clean_storage", { ids, allowIrreversible, onEvent: channel })
  },
  cleanElevated: (ids: string[], allowIrreversible: boolean) =>
    invoke<CleanResult[]>("clean_storage_elevated", { ids, allowIrreversible }),
  revealCategory: (id: string) => invoke<void>("reveal_storage_category", { id }),
  openSettings: () => invoke<void>("open_storage_settings"),
  analyze: (id?: string, onProgress?: (p: ScanProgress) => void) =>
    invoke<FolderAnalysis>("analyze_folder", { id: id ?? null, onProgress: progressChannel(onProgress) }),
  cancelAnalysis: () => invoke<void>("cancel_folder_analysis"),
  revealFolderItem: (id: string) => invoke<void>("reveal_folder_item", { id }),
  /** 시스템 드라이브의 큰 파일 (읽기 전용) */
  findLargeFiles: (onProgress?: (p: ScanProgress) => void) =>
    invoke<LargeFiles>("find_large_files", { onProgress: progressChannel(onProgress) }),
  cancelLargeFiles: () => invoke<void>("cancel_large_files"),
}

/** 다른 화면으로 이동 요청 (App이 받아 처리). */
export function navigate(page: "programs" | "leftovers" | "restore") {
  window.dispatchEvent(new CustomEvent("robin:navigate", { detail: page }))
}

// src-tauri/src/settings.rs 와 crates/settings 에 맞춰야 한다.
export type ExclusionKind = "path" | "registry"

export interface Exclusion {
  kind: ExclusionKind
  location: string
  valueName: string | null
  addedAt: number
}

export interface Settings {
  /** 이 기간(일)보다 오래된 백업을 지운다. null이면 계속 보관. 처음에는 30 */
  backupRetentionDays: number | null
  retentionChoices: number[]
  exclusions: Exclusion[]
}

export interface BackupUsage {
  count: number
  bytes: number
  /** 지금 보관 기간 기준으로 지울 수 있는 백업 수 */
  expired: number
}

export const settingsApi = {
  get: () => invoke<Settings>("get_settings"),
  setRetention: (days: number | null) => invoke<void>("set_backup_retention", { days }),
  /** 마지막 스캔 결과의 항목을 다음 스캔부터 뺀다. 이미 빠져 있으면 false */
  addExclusion: (itemId: string) => invoke<boolean>("add_exclusion", { itemId }),
  removeExclusion: (index: number, location: string) => invoke<void>("remove_exclusion", { index, location }),
  backupUsage: () => invoke<BackupUsage>("backup_usage"),
  pruneBackups: (onProgress?: (p: ScanProgress) => void) =>
    invoke<{ deleted: number; failed: number }>("prune_backups", { onProgress: progressChannel(onProgress) }),
  /** `days`일보다 오래된 백업 (읽기 전용). days는 retentionChoices 중 하나 */
  oldBackups: (days: number) => invoke<{ count: number; bytes: number }>("old_backups", { days }),
  deleteOldBackups: (days: number, onProgress?: (p: ScanProgress) => void) =>
    invoke<{ deleted: number; failed: number }>("delete_old_backups", {
      days,
      onProgress: progressChannel(onProgress),
    }),
}
