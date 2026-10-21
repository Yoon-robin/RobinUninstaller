import type { KeyboardEvent } from "react"
import type { LucideIcon } from "lucide-react"
import { motion } from "motion/react"

import { cn } from "@/lib/utils"
import { IconTooltip } from "@/components/icon-tooltip"

/**
 * 라디오·탭 묶음의 키보드 이동: ←/→(↑/↓)로 앞뒤, Home/End로 처음·끝. 사용 못 하는 항목은 건너뛴다.
 * 고른 항목만 Tab으로 들어오고(나머지는 tabIndex -1), 방향키로 고르면 바로 선택된다.
 */
export function rovingKeyDown<K>(
  e: KeyboardEvent<HTMLElement>,
  keys: K[],
  current: K,
  isDisabled: (key: K) => boolean,
  onChange: (key: K) => void,
) {
  const enabled = keys.filter((k) => !isDisabled(k))
  if (enabled.length === 0) return
  const at = Math.max(0, enabled.indexOf(current))
  let next: K | undefined
  if (e.key === "ArrowRight" || e.key === "ArrowDown") next = enabled[(at + 1) % enabled.length]
  else if (e.key === "ArrowLeft" || e.key === "ArrowUp") next = enabled[(at - 1 + enabled.length) % enabled.length]
  else if (e.key === "Home") next = enabled[0]
  else if (e.key === "End") next = enabled[enabled.length - 1]
  if (next === undefined) return
  e.preventDefault()
  onChange(next)
  // 새로 고른 항목으로 포커스를 옮긴다
  const group = e.currentTarget
  requestAnimationFrame(() => {
    group.querySelector<HTMLElement>('[aria-checked="true"],[aria-selected="true"]')?.focus()
  })
}

/** 선택 표시가 미끄러지듯 옮겨 가는 칩 묶음 (빠른 필터). */
export function FilterChips<K extends string>({
  options,
  value,
  counts,
  onChange,
}: {
  options: { key: K; label: string }[]
  value: K
  counts: Record<K, number>
  onChange: (key: K) => void
}) {
  return (
    <div
      className="flex flex-wrap gap-1.5"
      role="radiogroup"
      aria-label="빠른 필터"
      onKeyDown={(e) =>
        rovingKeyDown(
          e,
          options.map((o) => o.key),
          value,
          (k) => counts[k] === 0 && k !== value,
          onChange,
        )
      }
    >
      {options.map((o) => {
        const active = o.key === value
        const empty = counts[o.key] === 0 && !active
        return (
          <button
            key={o.key}
            type="button"
            role="radio"
            aria-checked={active}
            tabIndex={active ? 0 : -1}
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
                layoutId="filter-chip"
                className="absolute inset-0 rounded-full bg-primary"
                transition={{ type: "spring", duration: 0.3, bounce: 0 }}
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
  label,
}: {
  options: { key: K; icon: LucideIcon; label: string }[]
  value: K
  onChange: (key: K) => void
  layoutId: string
  /** 묶음 이름 (화면 읽기 프로그램용) */
  label: string
}) {
  return (
    <div
      className="flex h-9 items-center rounded-lg border bg-background/60 p-0.5"
      role="radiogroup"
      aria-label={label}
      onKeyDown={(e) =>
        rovingKeyDown(
          e,
          options.map((o) => o.key),
          value,
          () => false,
          onChange,
        )
      }
    >
      {options.map((o) => (
        <IconTooltip key={o.key} label={o.label}>
          <button
            type="button"
            role="radio"
            aria-checked={value === o.key}
            tabIndex={value === o.key ? 0 : -1}
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
                transition={{ type: "spring", duration: 0.3, bounce: 0 }}
              />
            )}
            <o.icon className="relative size-4" />
          </button>
        </IconTooltip>
      ))}
    </div>
  )
}

/** 글자 탭 두세 개짜리 전환 (데스크톱 프로그램 / Store 앱). */
export function TextTabs<K extends string>({
  options,
  value,
  onChange,
  layoutId,
  label,
}: {
  options: { key: K; label: string; count?: number }[]
  value: K
  onChange: (key: K) => void
  layoutId: string
  label: string
}) {
  return (
    <div
      className="flex h-9 items-center rounded-full border bg-muted p-1"
      role="tablist"
      aria-label={label}
      onKeyDown={(e) =>
        rovingKeyDown(
          e,
          options.map((o) => o.key),
          value,
          () => false,
          onChange,
        )
      }
    >
      {options.map((o) => {
        const active = value === o.key
        return (
          <button
            key={o.key}
            type="button"
            role="tab"
            aria-selected={active}
            tabIndex={active ? 0 : -1}
            onClick={() => onChange(o.key)}
            className={cn(
              "relative flex h-full items-center gap-1.5 rounded-full px-3 text-sm transition-colors",
              active ? "font-medium text-foreground" : "text-muted-foreground hover:text-foreground",
            )}
          >
            {active && (
              <motion.span
                layoutId={layoutId}
                className="absolute inset-0 rounded-full bg-background shadow-sm"
                transition={{ type: "spring", duration: 0.3, bounce: 0 }}
              />
            )}
            <span className="relative">{o.label}</span>
            {o.count != null && <span className="relative text-xs tabular-nums opacity-60">{o.count}</span>}
          </button>
        )
      })}
    </div>
  )
}
