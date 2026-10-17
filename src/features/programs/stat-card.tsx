import type { ReactNode } from "react"
import type { LucideIcon } from "lucide-react"

import { cn } from "@/lib/utils"
import { FadeUp } from "@/components/motion"

const toneClasses = {
  "chart-1": "bg-chart-1/12 text-chart-1",
  "chart-2": "bg-chart-2/12 text-chart-2",
  "chart-3": "bg-chart-3/12 text-chart-3",
  "chart-5": "bg-chart-5/12 text-chart-5",
} as const

const glowClasses = {
  "chart-1": "bg-chart-1",
  "chart-2": "bg-chart-2",
  "chart-3": "bg-chart-3",
  "chart-5": "bg-chart-5",
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
          "relative flex h-full w-full items-start gap-4 overflow-hidden rounded-2xl border bg-card p-5 text-left shadow-sm",
          "transition-[transform,border-color,box-shadow] duration-200 ease-(--ease-out-quint)",
          onClick &&
            "cursor-pointer hover:-translate-y-0.5 hover:border-foreground/15 hover:shadow-lg focus-visible:ring-2 focus-visible:ring-ring focus-visible:outline-none active:translate-y-0 active:scale-[0.99]",
          active && "border-chart-3/60 ring-1 ring-chart-3/40",
        )}
      >
        {/* 카드 오른쪽 위의 은은한 색 번짐 (움직이지 않음) */}
        <span
          aria-hidden
          className={cn(
            "pointer-events-none absolute -top-12 -right-12 size-32 rounded-full opacity-[0.07] blur-2xl",
            glowClasses[tone],
          )}
        />
        <span
          className={cn("relative flex size-10 shrink-0 items-center justify-center rounded-xl", toneClasses[tone])}
        >
          <Icon className="size-5" />
        </span>
        <span className="relative min-w-0">
          <span className="block text-sm text-muted-foreground">{label}</span>
          {value == null ? (
            <span className="mt-1.5 block h-7 w-20 animate-pulse rounded-md bg-muted" />
          ) : (
            <span className="block mt-0.5 text-2xl font-bold tracking-tight tabular-nums">
              {value}
              {unit && <span className="ml-0.5 text-base font-medium text-muted-foreground">{unit}</span>}
            </span>
          )}
          {hint && <span className="block mt-1 truncate text-xs text-muted-foreground">{hint}</span>}
        </span>
      </Tag>
    </FadeUp>
  )
}
