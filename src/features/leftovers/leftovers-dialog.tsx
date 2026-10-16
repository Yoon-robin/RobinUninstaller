import { useEffect, useMemo, useState, type CSSProperties } from "react"
import { useQuery } from "@tanstack/react-query"
import {
  Copy,
  FileText,
  Folder,
  FolderOpen,
  KeyRound,
  ListTree,
  RefreshCw,
  ShieldCheck,
  Sparkles,
  Trash2,
  TriangleAlert,
} from "lucide-react"
import { AnimatePresence, motion } from "motion/react"
import { toast } from "sonner"

import { scanApi, type Leftover, type LeftoverKind, type ScanReport } from "@/lib/api"
import { formatBytes } from "@/lib/format"
import { cn } from "@/lib/utils"
import { EASE_OUT_QUINT } from "@/components/motion"
import { IconTooltip } from "@/components/icon-tooltip"
import { Button } from "@/components/ui/button"
import { Checkbox } from "@/components/ui/checkbox"
import { Dialog, DialogContent, DialogDescription, DialogTitle } from "@/components/ui/dialog"
import { describeEvidence, kindLabels } from "./evidence"

export type LeftoverTarget = { id: string; name: string; removed: boolean }

/** 남은 흔적 찾기 결과. 지금은 보여 주기만 하고, 삭제는 백업·복원과 함께 다음 단계에서 켠다. */
export function LeftoversDialog({ target, onClose }: { target: LeftoverTarget | null; onClose: () => void }) {
  const scan = useQuery({
    queryKey: ["leftovers", target?.id],
    queryFn: () => scanApi.scanLeftovers(target!.id),
    enabled: target != null,
    staleTime: 0,
    gcTime: 0,
  })

  return (
    <Dialog open={target != null} onOpenChange={(open) => !open && onClose()}>
      <DialogContent className="flex h-[min(720px,88vh)] flex-col gap-0 overflow-hidden p-0 sm:max-w-3xl">
        {target && (
          <>
            <div className="flex items-center gap-3 border-b px-5 py-4 pr-12">
              <div className="flex size-10 items-center justify-center rounded-xl bg-primary/12 text-primary">
                <Sparkles className="size-5" />
              </div>
              <div className="min-w-0 flex-1">
                <DialogTitle className="text-base">남은 흔적</DialogTitle>
                <DialogDescription className="truncate">
                  {target.name}
                  {target.removed
                    ? " · 제거한 뒤 남은 파일과 레지스트리"
                    : " · 설치된 상태라 대부분 지금 쓰고 있는 항목이에요"}
                </DialogDescription>
              </div>
              <IconTooltip label="다시 찾기">
                <Button
                  variant="outline"
                  size="icon"
                  className="size-9 rounded-lg"
                  aria-label="다시 찾기"
                  disabled={scan.isFetching}
                  onClick={() => scan.refetch()}
                >
                  <RefreshCw className={cn(scan.isFetching && "animate-spin")} />
                </Button>
              </IconTooltip>
            </div>

            <AnimatePresence mode="wait" initial={false}>
              {scan.isFetching ? (
                <Scanning key="scanning" />
              ) : scan.isError ? (
                <div key="error" className="flex flex-1 items-center justify-center p-8 text-sm text-destructive">
                  찾지 못했어요: {String(scan.error)}
                </div>
              ) : scan.data ? (
                <Results key={`result-${scan.dataUpdatedAt}`} report={scan.data} onClose={onClose} />
              ) : null}
            </AnimatePresence>
          </>
        )}
      </DialogContent>
    </Dialog>
  )
}

function Scanning() {
  return (
    <motion.div
      initial={{ opacity: 0 }}
      animate={{ opacity: 1 }}
      exit={{ opacity: 0, transition: { duration: 0.12 } }}
      className="flex flex-1 flex-col items-center justify-center gap-4 p-8 text-center"
    >
      <div className="relative h-1 w-56 overflow-hidden rounded-full bg-muted">
        <div className="scan-sweep absolute inset-y-0 w-1/3 rounded-full bg-primary" />
      </div>
      <div>
        <p className="font-medium">남은 흔적을 찾고 있어요</p>
        <p className="mt-1 text-sm text-muted-foreground">설치 폴더, AppData, 레지스트리를 살펴보는 중이에요</p>
      </div>
    </motion.div>
  )
}

function Results({ report, onClose }: { report: ScanReport; onClose: () => void }) {
  const strong = report.items.filter((i) => i.confidence === "strong")
  const review = report.items.filter((i) => i.confidence === "review")
  // 확실한 항목만 미리 선택한다.
  const [selected, setSelected] = useState<Set<string>>(() => new Set(strong.map((i) => i.id)))
  useEffect(() => setSelected(new Set(strong.map((i) => i.id))), [report])

  const selectedBytes = useMemo(
    () => report.items.filter((i) => selected.has(i.id)).reduce((sum, i) => sum + (i.sizeBytes ?? 0), 0),
    [report, selected],
  )
  const toggle = (id: string) =>
    setSelected((s) => {
      const next = new Set(s)
      if (next.has(id)) next.delete(id)
      else next.add(id)
      return next
    })

  return (
    <motion.div
      initial={{ opacity: 0, y: 8 }}
      animate={{ opacity: 1, y: 0 }}
      exit={{ opacity: 0, transition: { duration: 0.12 } }}
      transition={{ duration: 0.3, ease: EASE_OUT_QUINT }}
      className="flex min-h-0 flex-1 flex-col"
    >
      {report.items.length === 0 ? (
        <Clean protectedSkipped={report.protectedSkipped} />
      ) : (
        <>
          <div className="flex flex-wrap items-center gap-2 border-b bg-muted/20 px-5 py-2.5 text-xs">
            <SummaryChip tone="primary" label="확실" value={strong.length} />
            <SummaryChip tone="warn" label="검토 필요" value={review.length} />
            <span className="text-muted-foreground">· {report.durationMs}ms</span>
            {report.protectedSkipped > 0 && (
              <IconTooltip label="Windows 폴더, 여러 프로그램이 함께 쓰는 폴더 등은 점수와 상관없이 건드리지 않아요">
                <span className="ml-auto flex cursor-default items-center gap-1 text-muted-foreground">
                  <ShieldCheck className="size-3.5 text-primary" />
                  보호 규칙으로 {report.protectedSkipped}개 제외
                </span>
              </IconTooltip>
            )}
          </div>

          <div className="min-h-0 flex-1 overflow-y-auto">
            <Group
              title="확실한 흔적"
              hint="강한 근거가 있어 미리 선택했어요"
              items={strong}
              offset={0}
              selected={selected}
              onToggle={toggle}
            />
            <Group
              title="검토가 필요한 항목"
              hint="이름만 비슷하거나 다른 프로그램과 겹칠 수 있어요. 확인한 뒤 직접 선택하세요"
              items={review}
              offset={strong.length}
              selected={selected}
              onToggle={toggle}
            />
          </div>
        </>
      )}

      <div className="flex items-center gap-3 border-t px-5 py-3">
        <p className="mr-auto text-sm text-muted-foreground tabular-nums">
          {selected.size > 0 ? `${selected.size}개 선택 · ${formatBytes(selectedBytes)}` : "선택한 항목 없음"}
        </p>
        <IconTooltip label="지우기 전에 자동으로 백업하고 되돌릴 수 있게 하는 기능과 함께 다음 업데이트에서 켜져요">
          <span tabIndex={0}>
            <Button variant="destructive" disabled>
              <Trash2 />
              선택 항목 지우기
            </Button>
          </span>
        </IconTooltip>
        <Button variant="outline" onClick={onClose}>
          닫기
        </Button>
      </div>
    </motion.div>
  )
}

function SummaryChip({ tone, label, value }: { tone: "primary" | "warn"; label: string; value: number }) {
  return (
    <span
      className={cn(
        "flex items-center gap-1.5 rounded-full px-2.5 py-1 font-medium",
        tone === "primary" ? "bg-primary/12 text-primary" : "bg-chart-3/12 text-chart-3",
      )}
    >
      {label}
      <span className="tabular-nums">{value}</span>
    </span>
  )
}

function Group({
  title,
  hint,
  items,
  offset,
  selected,
  onToggle,
}: {
  title: string
  hint: string
  items: Leftover[]
  offset: number
  selected: Set<string>
  onToggle: (id: string) => void
}) {
  if (items.length === 0) return null
  return (
    <section>
      <div className="sticky top-0 z-10 border-b bg-popover/95 px-5 py-2 backdrop-blur">
        <p className="text-sm font-semibold">
          {title} <span className="font-normal text-muted-foreground tabular-nums">{items.length}</span>
        </p>
        <p className="text-xs text-muted-foreground">{hint}</p>
      </div>
      <ul className="divide-y">
        {items.map((item, i) => (
          <LeftoverRow
            key={item.id}
            item={item}
            index={offset + i}
            checked={selected.has(item.id)}
            onToggle={() => onToggle(item.id)}
          />
        ))}
      </ul>
    </section>
  )
}

const kindIcons: Record<LeftoverKind, typeof Folder> = {
  folder: Folder,
  file: FileText,
  registryKey: ListTree,
  registryValue: KeyRound,
}

function LeftoverRow({
  item,
  index,
  checked,
  onToggle,
}: {
  item: Leftover
  index: number
  checked: boolean
  onToggle: () => void
}) {
  const Icon = kindIcons[item.kind]
  const isFs = item.kind === "folder" || item.kind === "file"
  const fullPath = item.valueName ? `${item.location} › ${item.valueName}` : item.location

  return (
    <li
      className={cn("group flex gap-3 px-5 py-3 transition-colors hover:bg-accent/30", index < 12 && "row-intro")}
      style={index < 12 ? ({ "--i": index } as CSSProperties) : undefined}
    >
      <Checkbox checked={checked} onCheckedChange={onToggle} aria-label={`${fullPath} 선택`} className="mt-0.5" />
      <Icon className="mt-0.5 size-4 shrink-0 text-muted-foreground" aria-label={kindLabels[item.kind]} />
      <div className="min-w-0 flex-1">
        <p data-selectable className="font-mono text-xs leading-5 break-all" title={fullPath}>
          {item.location}
          {item.valueName && (
            <span className="ml-1.5 rounded bg-muted px-1.5 py-0.5 font-sans text-[11px] text-foreground">
              값: {item.valueName}
            </span>
          )}
        </p>
        <div className="mt-1.5 flex flex-wrap gap-1">
          {item.evidence.map((e, i) => {
            const d = describeEvidence(e)
            return (
              <span
                key={i}
                className={cn(
                  "rounded-md border px-1.5 py-0.5 text-[11px]",
                  d.warn ? "border-chart-3/40 bg-chart-3/10 text-chart-3" : "text-muted-foreground",
                )}
              >
                {d.label}
              </span>
            )
          })}
        </div>
        {item.note && (
          <p className="mt-1.5 flex items-start gap-1 text-xs text-chart-3">
            <TriangleAlert className="mt-0.5 size-3 shrink-0" />
            {item.note}
          </p>
        )}
      </div>
      <div className="flex shrink-0 flex-col items-end gap-1">
        <span className="text-xs text-muted-foreground tabular-nums">{isFs ? formatBytes(item.sizeBytes) : ""}</span>
        <div className="flex opacity-0 transition-opacity group-hover:opacity-100 focus-within:opacity-100">
          {isFs && (
            <IconTooltip label="탐색기에서 보기">
              <Button
                variant="ghost"
                size="icon"
                className="size-7"
                aria-label="탐색기에서 보기"
                onClick={() => scanApi.revealLeftover(item.id).catch((e) => toast.error(String(e)))}
              >
                <FolderOpen />
              </Button>
            </IconTooltip>
          )}
          <IconTooltip label="경로 복사">
            <Button
              variant="ghost"
              size="icon"
              className="size-7"
              aria-label="경로 복사"
              onClick={() => navigator.clipboard.writeText(fullPath).then(() => toast.success("경로를 복사했어요"))}
            >
              <Copy />
            </Button>
          </IconTooltip>
        </div>
      </div>
    </li>
  )
}

function Clean({ protectedSkipped }: { protectedSkipped: number }) {
  return (
    <div className="flex flex-1 flex-col items-center justify-center gap-3 p-8 text-center">
      <motion.div
        initial={{ scale: 0.8, opacity: 0 }}
        animate={{ scale: 1, opacity: 1 }}
        transition={{ type: "spring", duration: 0.45, bounce: 0.35 }}
        className="flex size-14 items-center justify-center rounded-full bg-primary/15 text-primary"
      >
        <Sparkles className="size-7" />
      </motion.div>
      <div>
        <p className="text-base font-semibold">깨끗해요</p>
        <p className="mt-1 text-sm text-muted-foreground">
          남은 흔적을 찾지 못했어요.
          {protectedSkipped > 0 && ` (보호 규칙으로 ${protectedSkipped}개는 살펴보지 않았어요)`}
        </p>
      </div>
    </div>
  )
}
