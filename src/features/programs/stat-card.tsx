import type { ReactNode } from "react"
import type { LucideIcon } from "lucide-react"

import { cn } from "@/lib/utils"
import { FadeUp } from "@/components/motion"

const toneClasses = {
  "chart-1": "bg-chart-1/12 text-chart-1",
  "chart-2": "bg-chart-2/12 text-chart-2",
  "chart-3": "bg-chart-3/12 text-warning",
  "chart-5": "bg-chart-5/12 text-success",
} as const

export function StatCard({
  icon: Icon,
  tone,
  label,
  value,
  unit,
  hint,
  active,
  onClick,
}: {
  icon: LucideIcon
  tone: keyof typeof toneClasses
  label: string
  value: ReactNode | null
  unit?: string
  hint?: string
  active?: boolean
  onClick?: () => void
}) {
  const Tag = onClick ? "button" : "div"
  return (
    <FadeUp className="h-full">
      <Tag
        type={onClick ? "button" : undefined}
        onClick={onClick}
        aria-pressed={onClick ? active : undefined}
        className={cn(
          "relative flex h-full w-full items-start gap-3 overflow-hidden rounded-2xl border bg-card p-4 text-left shadow-sm lg:gap-4 lg:p-5",
          "transition-[transform,border-color,box-shadow] duration-200 ease-(--ease-out-quint)",
          onClick &&
            "cursor-pointer hover:border-foreground/20 focus-visible:ring-2 focus-visible:ring-ring focus-visible:outline-none active:scale-[0.99] motion-reduce:active:scale-100",
          active && "border-chart-3/60 ring-1 ring-chart-3/40",
        )}
      >
        <span
          className={cn(
            "relative flex size-8 shrink-0 items-center justify-center rounded-xl lg:size-10",
            toneClasses[tone],
          )}
        >
          <Icon className="size-4 lg:size-5" />
        </span>
        <span className="relative min-w-0">
          <span className="block truncate text-sm text-muted-foreground">{label}</span>
          {value == null ? (
            <span className="mt-1.5 block h-7 w-20 animate-pulse rounded-md bg-muted" />
          ) : (
            <span className="block mt-0.5 text-2xl font-bold tracking-tight tabular-nums">
              {value}
              {unit && <span className="ml-0.5 text-base font-medium text-muted-foreground">{unit}</span>}
            </span>
          )}
          {hint && <span className="mt-1 block text-xs text-pretty text-muted-foreground">{hint}</span>}
        </span>
      </Tag>
    </FadeUp>
  )
}
