import { Check, Copy } from "lucide-react"

import { useCopy } from "@/lib/notify"
import { cn } from "@/lib/utils"
import { IconTooltip } from "@/components/icon-tooltip"
import { Button } from "@/components/ui/button"

/** 경로·명령 옆에 두는 복사 버튼. 복사하면 잠깐 체크로 바뀌고, 실패하면 알림을 띄운다. */
export function CopyButton({
  text,
  label = "복사",
  className,
  ...rest
}: { text: string; label?: string; className?: string } & Omit<React.ComponentProps<typeof Button>, "onClick">) {
  const { copied, copy } = useCopy()
  return (
    <IconTooltip label={copied ? "복사했어요" : label}>
      <Button
        variant="ghost"
        size="icon"
        className={cn("size-7", className)}
        aria-label={copied ? "복사했어요" : label}
        onClick={(e) => {
          e.stopPropagation()
          void copy(text)
        }}
        {...rest}
      >
        {copied ? <Check className="text-success" /> : <Copy />}
      </Button>
    </IconTooltip>
  )
}
