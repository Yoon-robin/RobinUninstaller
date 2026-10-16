import { invoke } from "@tauri-apps/api/core"

// crates/core 의 InstalledApplication 과 맞춰야 한다 (serde camelCase).
export type InstallScope = "machine64" | "machine32" | "user"
export type InstallTechnology = "exe" | "msi"

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
}

export const api = {
  listInstalledApps: () => invoke<InstalledApplication[]>("list_installed_apps"),
}
