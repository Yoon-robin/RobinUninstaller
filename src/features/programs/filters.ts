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

/** 이름 비교. 비교할 때마다 locale 옵션으로 localeCompare를 부르면 매번 Collator를 새로 만들어 1000개 정렬이 느렸다. */
const nameCollator = new Intl.Collator("ko", { numeric: true })

export function compareApps(a: InstalledApplication, b: InstalledApplication, key: SortKey) {
  switch (key) {
    case "size":
      return (b.estimatedSizeKb ?? 0) - (a.estimatedSizeKb ?? 0)
    case "date":
      return installDateSortKey(b.installDate).localeCompare(installDateSortKey(a.installDate))
    default:
      // "이름 5"가 "이름 45"보다 앞에 오게 숫자는 수로 비교한다
      return nameCollator.compare(a.displayName, b.displayName)
  }
}

/** 정렬한 새 배열. 설치일 순서는 날짜 키를 한 번만 만든다 (비교마다 정규식을 돌리지 않게). */
export function sortApps(apps: InstalledApplication[], key: SortKey) {
  if (key !== "date") return [...apps].sort((a, b) => compareApps(a, b, key))
  const keys = new Map(apps.map((a) => [a.id, installDateSortKey(a.installDate)]))
  return [...apps].sort((a, b) => keys.get(b.id)!.localeCompare(keys.get(a.id)!))
}
