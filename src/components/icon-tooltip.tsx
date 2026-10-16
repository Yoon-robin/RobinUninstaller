import type { ReactElement } from "react"

import { Tooltip, TooltipContent, TooltipTrigger } from "@/components/ui/tooltip"

/** 아이콘 버튼용 툴팁. 첫 표시는 잠깐 기다렸다가 뜨고, 이어서 옮겨 다닐 때는 바로 뜬다(TooltipProvider 설정). */
export function IconTooltip({
  label,
  children,
  side = "bottom",
}: {
  label: string
  children: ReactElement
  side?: "top" | "bottom" | "left" | "right"
}) {
  return (
    <Tooltip>
      <TooltipTrigger asChild>{children}</TooltipTrigger>
      <TooltipContent side={side}>{label}</TooltipContent>
    </Tooltip>
  )
}
