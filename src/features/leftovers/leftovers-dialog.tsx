import { useEffect, useMemo, useState, type CSSProperties } from "react"
import { useQuery } from "@tanstack/react-query"
import {
  Copy,
  EyeOff,
  FileText,
  Folder,
  FolderOpen,
  KeyRound,
  ListTree,
  Loader2,
  RefreshCw,
  ShieldCheck,
  ShieldX,
  Sparkles,
  Trash2,
  TriangleAlert,
  ScanSearch,
} from "lucide-react"
import { AnimatePresence, motion } from "motion/react"
import { toast } from "sonner"

import {
  forceApi,
  recoveryApi,
  scanApi,
  settingsApi,
  type ForceCheck,
  type Leftover,
  type LeftoverKind,
  type Operation,
  type ScanReport,
} from "@/lib/api"
import { useQueryClient } from "@tanstack/react-query"
import { CleanResult, Cleaning, ConfirmClean } from "./clean-flow"
import { formatBytes } from "@/lib/format"
import { cn } from "@/lib/utils"
import { EASE_OUT_QUINT } from "@/components/motion"
import { IconTooltip } from "@/components/icon-tooltip"
import { BorderBeam } from "@/components/ui/border-beam"
import { Button } from "@/components/ui/button"
import { Checkbox } from "@/components/ui/checkbox"
import { Dialog, DialogContent, DialogDescription, DialogTitle } from "@/components/ui/dialog"
import { describeEvidence, kindLabels } from "./evidence"

/** `force`: 언인스톨러가 없는 프로그램의 강제 제거. 등록 정보와 파일을 같은 백업 흐름으로 정리한다. */
export type LeftoverTarget = { id: string; name: string; removed: boolean; force?: boolean }

/** 남은 흔적 찾기 결과. 고른 항목은 백업·격리한 뒤 정리한다. */
export function LeftoversDialog({ target, onClose }: { target: LeftoverTarget | null; onClose: () => void }) {
  const force = target?.force === true
  // 강제 제거는 먼저 대상에서 빼야 하는 프로그램인지, 실행 중인지 확인한다
  const check = useQuery({
    queryKey: ["force-check", target?.id],
    queryFn: () => forceApi.check(target!.id),
    enabled: force,
    staleTime: 0,
    gcTime: 0,
  })
  const ready = !force || (check.data != null && check.data.blocked == null && check.data.running.length === 0)
  // 확인·정리·결과 단계에서는 다시 찾기를 막고, 정리 중에는 창도 닫지 못하게 한다
  const [locked, setLocked] = useState<"none" | "stage" | "busy">("none")
  useEffect(() => setLocked("none"), [target])
  const scan = useQuery({
    queryKey: ["leftovers", target?.id],
    queryFn: () => scanApi.scanLeftovers(target!.id),
    enabled: target != null && ready,
    staleTime: 0,
    gcTime: 0,
  })

  return (
    <Dialog open={target != null} onOpenChange={(open) => !open && locked !== "busy" && onClose()}>
      <DialogContent
        showCloseButton={locked !== "busy"}
        className="flex h-[min(720px,88vh)] flex-col gap-0 overflow-hidden p-0 sm:max-w-3xl"
      >
        {target && (
          <>
            <div className="flex items-center gap-3 border-b px-5 py-4 pr-12">
              <div
                className={cn(
                  "flex size-10 items-center justify-center rounded-xl",
                  force ? "bg-destructive/12 text-destructive" : "bg-primary/12 text-primary",
                )}
              >
                {force ? <Trash2 className="size-5" /> : <Sparkles className="size-5" />}
              </div>
              <div className="min-w-0 flex-1">
                <DialogTitle className="text-base">{force ? "강제 제거" : "남은 흔적"}</DialogTitle>
                <DialogDescription className="truncate">
                  {target.name}
                  {force
                    ? " · 언인스톨러 없이 등록 정보와 파일을 직접 정리해요"
                    : target.removed
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
                  disabled={scan.isFetching || !ready || locked !== "none"}
                  onClick={() => scan.refetch()}
                >
                  <RefreshCw className={cn(scan.isFetching && "animate-spin")} />
                </Button>
              </IconTooltip>
            </div>

            <AnimatePresence mode="wait" initial={false}>
              {!ready ? (
                <ForceGate
                  key="gate"
                  check={check.data}
                  error={check.error}
                  onRetry={() => check.refetch()}
                  onClose={onClose}
                />
              ) : scan.isFetching ? (
                <Scanning key="scanning" />
              ) : scan.isError ? (
                <div key="error" className="flex flex-1 items-center justify-center p-8 text-sm text-destructive">
                  찾지 못했어요: {String(scan.error)}
                </div>
              ) : scan.data ? (
                <Results
                  key={`result-${scan.dataUpdatedAt}`}
                  report={scan.data}
                  appId={target.id}
                  appName={target.name}
                  forceAppId={force ? target.id : undefined}
                  onClose={onClose}
                  onLockChange={setLocked}
                />
              ) : null}
            </AnimatePresence>
          </>
        )}
      </DialogContent>
    </Dialog>
  )
}

/** 강제 제거 전 확인 화면: 확인 중 / 대상 아님 / 실행 중이라 닫아 달라고 안내. */
function ForceGate({
  check,
  error,
  onRetry,
  onClose,
}: {
  check: ForceCheck | undefined
  error: unknown
  onRetry: () => void
  onClose: () => void
}) {
  const panel = "flex flex-1 flex-col items-center justify-center gap-4 p-8 text-center"
  const motionProps = {
    initial: { opacity: 0 },
    animate: { opacity: 1 },
    exit: { opacity: 0, transition: { duration: 0.12 } },
  }

  if (error) {
    return (
      <motion.div {...motionProps} className={panel}>
        <p className="text-sm text-destructive">확인하지 못했어요: {String(error)}</p>
        <div className="flex gap-2">
          <Button variant="outline" onClick={onClose}>
            닫기
          </Button>
          <Button onClick={onRetry}>
            <RefreshCw />
            다시 확인
          </Button>
        </div>
      </motion.div>
    )
  }
  if (!check) {
    return (
      <motion.div {...motionProps} className={panel}>
        <Loader2 className="size-7 animate-spin text-primary" />
        <p className="font-medium">강제로 지워도 되는지 확인하고 있어요</p>
      </motion.div>
    )
  }
  if (check.blocked) {
    return (
      <motion.div {...motionProps} className={panel}>
        <div className="flex size-14 items-center justify-center rounded-full bg-chart-3/15 text-chart-3">
          <ShieldX className="size-7" />
        </div>
        <div className="max-w-md">
          <p className="text-base font-semibold">이 프로그램은 강제 제거하지 않아요</p>
          <p className="mt-2 text-sm leading-relaxed text-muted-foreground">{check.blocked}</p>
        </div>
        <Button variant="outline" onClick={onClose}>
          닫기
        </Button>
      </motion.div>
    )
  }
  return (
    <motion.div {...motionProps} className={panel}>
      <div className="flex size-14 items-center justify-center rounded-full bg-chart-3/15 text-chart-3">
        <TriangleAlert className="size-7" />
      </div>
      <div className="max-w-md">
        <p className="text-base font-semibold">프로그램이 실행 중이에요</p>
        <p className="mt-2 text-sm leading-relaxed text-muted-foreground">
          실행 중인 파일은 옮길 수 없어요. 아래 프로그램을 직접 닫은 뒤 다시 확인해 주세요. 작업 중인 내용이 있을 수
          있어 RobinX가 대신 끄지는 않아요.
        </p>
      </div>
      <ul className="flex w-full max-w-md flex-col gap-1.5 text-left">
        {check.running.map((p) => (
          <li
            key={p.pid}
            data-selectable
            className="rounded-lg border bg-muted/30 px-3 py-2 font-mono text-xs break-all"
          >
            {p.path}
          </li>
        ))}
      </ul>
      <div className="flex gap-2">
        <Button variant="outline" onClick={onClose}>
          닫기
        </Button>
        <Button onClick={onRetry}>
          <RefreshCw />
          다시 확인
        </Button>
      </div>
    </motion.div>
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
      {/* 돋보기 타일 테두리를 따라 빛이 돈다 */}
      <div className="relative flex size-16 items-center justify-center rounded-2xl bg-primary/10 text-primary">
        <ScanSearch className="size-7" />
        <BorderBeam size={36} duration={2.4} borderWidth={1.5} colorFrom="var(--chart-1)" colorTo="var(--chart-2)" />
      </div>
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

type Stage = { step: "review" } | { step: "confirm" } | { step: "cleaning" } | { step: "done"; op: Operation }

function Results({
  report,
  appId,
  appName,
  forceAppId,
  onClose,
  onLockChange,
}: {
  report: ScanReport
  appId: string
  appName: string
  forceAppId?: string
  onClose: () => void
  onLockChange: (lock: "none" | "stage" | "busy") => void
}) {
  const queryClient = useQueryClient()
  const [stage, setStage] = useState<Stage>({ step: "review" })
  const [cleanError, setCleanError] = useState<string | null>(null)
  useEffect(() => {
    onLockChange(stage.step === "review" ? "none" : stage.step === "cleaning" ? "busy" : "stage")
  }, [stage.step, onLockChange])
  const strong = report.items.filter((i) => i.confidence === "strong")
  const review = report.items.filter((i) => i.confidence === "review")
  // 근거가 강한 항목(등록 정보가 직접 가리키는 위치 등)만 미리 선택한다.
  const [selected, setSelected] = useState<Set<string>>(() => new Set(strong.map((i) => i.id)))
  useEffect(() => setSelected(new Set(strong.map((i) => i.id))), [report])

  const selectedBytes = useMemo(
    () => report.items.filter((i) => selected.has(i.id)).reduce((sum, i) => sum + (i.sizeBytes ?? 0), 0),
    [report, selected],
  )
  const selectedItems = report.items.filter((i) => selected.has(i.id))

  const runClean = async () => {
    setStage({ step: "cleaning" })
    setCleanError(null)
    try {
      const op = await recoveryApi.clean(appId, appName, [...selected], forceAppId)
      setStage({ step: "done", op })
    } catch (e) {
      setCleanError(String(e))
      setStage({ step: "review" })
    } finally {
      for (const key of ["backups", "installed-apps", "recently-removed", "startup-items"]) {
        queryClient.invalidateQueries({ queryKey: [key] })
      }
    }
  }

  if (stage.step === "confirm") {
    return <ConfirmClean items={selectedItems} onCancel={() => setStage({ step: "review" })} onConfirm={runClean} />
  }
  if (stage.step === "cleaning") {
    return <Cleaning label="백업하고 정리하는 중이에요" />
  }
  if (stage.step === "done") {
    return (
      <CleanResult
        op={stage.op}
        forceAppId={forceAppId}
        onDone={onClose}
        onBusyChange={(busy) => onLockChange(busy ? "busy" : "stage")}
      />
    )
  }

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
          {forceAppId && (
            <div className="flex gap-2 border-b bg-destructive/5 px-5 py-2.5 text-xs leading-relaxed text-muted-foreground">
              <ShieldCheck className="mt-0.5 size-3.5 shrink-0 text-primary" />
              <span>
                <b className="text-foreground">프로그램 등록 정보</b>와 설치 폴더를 지우면 목록에서 사라져요. 모두
                백업돼 복구 기록에서 되돌릴 수 있어요. Windows 서비스는 건드리지 않아요.
              </span>
            </div>
          )}
          <div className="flex flex-wrap items-center gap-2 border-b bg-muted/20 px-5 py-2.5 text-xs">
            <SummaryChip tone="primary" label="근거 강함" value={strong.length} />
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
              title="근거가 강한 흔적"
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
        {cleanError && <p className="text-xs text-destructive">{cleanError}</p>}
        <Button variant="destructive" disabled={selected.size === 0} onClick={() => setStage({ step: "confirm" })}>
          <Trash2 />
          선택 항목 정리
        </Button>
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
          <IconTooltip label="다음부터 제외 (설정에서 되돌릴 수 있어요)">
            <Button
              variant="ghost"
              size="icon"
              className="size-7"
              aria-label="다음부터 제외"
              onClick={() =>
                settingsApi
                  .addExclusion(item.id)
                  .then((added) =>
                    toast.success(added ? "다음 스캔부터 이 위치는 빼요" : "이미 제외 목록에 있어요", {
                      description: "설정 › 잔여물 스캔에서 뺄 항목에서 되돌릴 수 있어요",
                    }),
                  )
                  .catch((e) => toast.error(String(e)))
              }
            >
              <EyeOff />
            </Button>
          </IconTooltip>
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
