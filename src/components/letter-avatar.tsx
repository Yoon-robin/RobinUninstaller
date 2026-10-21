import type { CSSProperties } from "react"

import { cn } from "@/lib/utils"

/** 이름에서 고른 색상 (같은 이름은 항상 같은 색). */
function hueOf(name: string) {
  let h = 0
  for (const ch of name) h = (h * 31 + ch.codePointAt(0)!) % 360
  return h
}

/**
 * 아이콘이 없는 프로그램·시작 항목의 첫 글자 아이콘.
 * 바탕은 같은 옅은 색이고, 글자는 밝은 테마에서 진하게, 어두운 테마에서 밝게 써서 두 테마 모두 3:1 넘게 읽힌다.
 */
export function LetterAvatar({ name, className }: { name: string; className?: string }) {
  const letter = [...name.trim()][0]?.toUpperCase() ?? "?"
  return (
    <div
      className={cn(
        "flex size-9 shrink-0 items-center justify-center rounded-lg bg-[oklch(0.72_0.12_var(--hue)/16%)] text-sm font-semibold",
        "text-[oklch(0.46_0.13_var(--hue))] dark:text-[oklch(0.82_0.12_var(--hue))]",
        className,
      )}
      style={{ "--hue": hueOf(name) } as CSSProperties}
    >
      {letter}
    </div>
  )
}
