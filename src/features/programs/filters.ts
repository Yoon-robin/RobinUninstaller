import { needsForce, type InstalledApplication } from "@/lib/api"
import { installDateSortKey } from "@/lib/format"

export type QuickFilter = "all" | "recent" | "large" | "user" | "broken"

export const quickFilters: { key: QuickFilter; label: string }[] = [
  { key: "all", label: "전체" },
  { key: "recent", label: "최근 30일" },
  { key: "large", label: "1 GB 이상" },
  { key: "user", label: "현재 사용자" },
  { key: "broken", label: "언인스톨러 없음" },
]

const ONE_GB_KB = 1024 * 1024
const RECENT_DAYS = 30

export function matchesFilter(app: InstalledApplication, filter: QuickFilter, now = new Date()): boolean {
  switch (filter) {
    case "all":
      return true
    case "recent": {
      const key = installDateSortKey(app.installDate)
      if (!key) return false
      // 설치일은 현지 날짜다. new Date("YYYY-MM-DD")는 UTC 자정으로 읽어 한국 시간 0~9시에 오늘 설치한 것을 뺐다.
      const [y, m, d] = key.split("-").map(Number)
      const today = new Date(now.getFullYear(), now.getMonth(), now.getDate())
      const days = Math.round((today.getTime() - new Date(y, m - 1, d).getTime()) / 86_400_000)
      return days >= 0 && days <= RECENT_DAYS
    }
    case "large":
      return (app.estimatedSizeKb ?? 0) >= ONE_GB_KB
    case "user":
      return app.scope === "user"
    case "broken":
      return needsForce(app)
  }
}

export function matchesQuery(app: InstalledApplication, query: string): boolean {
  const q = query.trim().toLowerCase()
  return !q || app.displayName.toLowerCase().includes(q) || (app.publisher ?? "").toLowerCase().includes(q)
}

export type SortKey = "name" | "size" | "date"

export const sortLabels: Record<SortKey, string> = {
  name: "이름순",
  size: "크기순",
  date: "최근 설치순",
}

export function compareApps(a: InstalledApplication, b: InstalledApplication, key: SortKey) {
  switch (key) {
    case "size":
      return (b.estimatedSizeKb ?? 0) - (a.estimatedSizeKb ?? 0)
    case "date":
      return installDateSortKey(b.installDate).localeCompare(installDateSortKey(a.installDate))
    default:
      // "이름 5"가 "이름 45"보다 앞에 오게 숫자는 수로 비교한다
      return a.displayName.localeCompare(b.displayName, "ko", { numeric: true })
  }
}
