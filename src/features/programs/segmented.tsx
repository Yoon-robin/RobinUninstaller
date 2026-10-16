import type { LucideIcon } from "lucide-react"
import { motion } from "motion/react"

import { cn } from "@/lib/utils"
import { IconTooltip } from "@/components/icon-tooltip"

/** 선택 표시가 미끄러지듯 옮겨 가는 칩 묶음 (빠른 필터). */
export function FilterChips<K extends string>({
  options,
  value,
  counts,
  onChange,
  layoutId = "filter-chip",
}: {
  options: { key: K; label: string }[]
  value: K
  counts: Record<K, number>
  onChange: (key: K) => void
  /** 한 화면에 칩 묶음이 여럿 떠 있으면 서로 달라야 한다. */
  layoutId?: string
}) {
  return (
    <div className="flex flex-wrap gap-1.5" role="radiogroup" aria-label="빠른 필터">
      {options.map((o) => {
        const active = o.key === value
        const empty = counts[o.key] === 0 && !active
        return (
          <button
            key={o.key}
            type="button"
            role="radio"
            aria-checked={active}
            disabled={empty}
            onClick={() => onChange(o.key)}
            className={cn(
              "relative flex h-7 items-center gap-1.5 rounded-full border px-3 text-xs font-medium transition-colors duration-150",
              active ? "border-transparent text-primary-foreground" : "text-muted-foreground hover:text-foreground",
              empty && "opacity-40",
            )}
          >
            {active && (
              <motion.span
                layoutId={layoutId}
                className="absolute inset-0 rounded-full bg-primary"
                transition={{ type: "spring", duration: 0.35, bounce: 0.15 }}
              />
            )}
            <span className="relative">{o.label}</span>
            <span className={cn("relative tabular-nums", active ? "opacity-80" : "opacity-60")}>{counts[o.key]}</span>
          </button>
        )
      })}
    </div>
  )
}

/** 아이콘 버튼 두세 개짜리 전환 스위치 (목록/카드 보기). */
export function IconSegmented<K extends string>({
  options,
  value,
  onChange,
  layoutId,
}: {
  options: { key: K; icon: LucideIcon; label: string }[]
  value: K
  onChange: (key: K) => void
  layoutId: string
}) {
  return (
    <div className="flex h-9 items-center rounded-lg border bg-background/60 p-0.5" role="radiogroup">
      {options.map((o) => (
        <IconTooltip key={o.key} label={o.label}>
          <button
            type="button"
            role="radio"
            aria-checked={value === o.key}
            aria-label={o.label}
            onClick={() => onChange(o.key)}
            className={cn(
              "relative flex h-full w-8 items-center justify-center rounded-md transition-colors",
              value === o.key ? "text-foreground" : "text-muted-foreground hover:text-foreground",
            )}
          >
            {value === o.key && (
              <motion.span
                layoutId={layoutId}
                className="absolute inset-0 rounded-md bg-muted shadow-sm"
                transition={{ type: "spring", duration: 0.3, bounce: 0.15 }}
              />
            )}
            <o.icon className="relative size-4" />
          </button>
        </IconTooltip>
      ))}
    </div>
  )
}
