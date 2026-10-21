import { HardDrive } from "lucide-react"

import { formatBytes } from "@/lib/format"
import { cn } from "@/lib/utils"
import { selectedDrive, useDrives } from "./drive"
import { resetFolderAnalysis } from "./folder-analysis"
import { resetLargeFiles } from "./large-files"

/**
 * 저장공간 화면 위의 드라이브 고르기. 고정 디스크가 둘 이상일 때만 보인다.
 * 디스크 카드·큰 폴더 찾기·큰 파일 찾기가 고른 드라이브를 따른다.
 */
export function DrivePicker() {
  const drives = useDrives()
  const selected = selectedDrive.use()
  const list = drives.data ?? []
  if (list.length < 2) return null
  const current = selected ?? list.find((d) => d.system)?.letter ?? list[0].letter
  const pick = (letter: string) => {
    if (letter === current) return
    selectedDrive.set(letter)
    resetFolderAnalysis()
    resetLargeFiles()
  }
  return (
    <div role="radiogroup" aria-label="드라이브" className="flex flex-wrap gap-2">
      {list.map((d) => {
        const active = d.letter === current
        return (
          <button
            key={d.letter}
            type="button"
            role="radio"
            aria-checked={active}
            onClick={() => pick(d.letter)}
            className={cn(
              "flex cursor-pointer items-center gap-2 rounded-xl border px-3 py-2 text-left text-sm transition-colors",
              active ? "border-primary/50 bg-primary/10" : "bg-card hover:bg-accent/40",
            )}
          >
            <HardDrive className={cn("size-4 shrink-0", active ? "text-primary" : "text-muted-foreground")} />
            <span>
              <span className="font-medium">
                {d.letter}:{d.label ? ` ${d.label}` : ""}
              </span>
              <span className="block text-xs text-muted-foreground tabular-nums">
                {formatBytes(d.free)} 남음 / {formatBytes(d.total)}
                {d.system && " · Windows"}
              </span>
            </span>
          </button>
        )
      })}
    </div>
  )
}
