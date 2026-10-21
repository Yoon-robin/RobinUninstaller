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
  type ScanProgress,
} from "@/lib/api"
import { useQueryClient } from "@tanstack/react-query"
import { CleanResult, Cleaning, ConfirmClean } from "./clean-flow"
import { formatBytes } from "@/lib/format"
import { cn } from "@/lib/utils"
import { useLastDefined } from "@/hooks/use-last-defined"
import { EASE_OUT_QUINT } from "@/components/motion"
import { IconTooltip } from "@/components/icon-tooltip"
import { BorderBeam } from "@/components/ui/border-beam"
import { Button } from "@/components/ui/button"
import { Checkbox } from "@/components/ui/checkbox"
import { Dialog, DialogContent, DialogDescription, DialogTitle } from "@/components/ui/dialog"
import { describeEvidence, kindLabels } from "./evidence"
import { ErrorState } from "@/components/error-state"

/** `force`: 언인스톨러가 없는 프로그램의 강제 제거. 등록 정보와 파일을 같은 백업 흐름으로 정리한다. */
export type LeftoverTarget = { id: string; name: string; removed: boolean; force?: boolean }

/** 남은 흔적 찾기 결과. 고른 항목은 백업·격리한 뒤 정리한다. */
export function LeftoversDialog({
  target: targetProp,
  onClose,
}: {
  target: LeftoverTarget | null
  onClose: () => void
}) {
  // 닫히는 동안에도 내용이 남아 있게 마지막 대상을 그린다. 찾기는 열려 있을 때만 한다.
  const open = targetProp != null
  const target = useLastDefined(targetProp)
  const force = target?.force === true
  // 강제 제거는 먼저 대상에서 빼야 하는 프로그램인지, 실행 중인지 확인한다
  const check = useQuery({
    queryKey: ["force-check", target?.id],
    queryFn: () => forceApi.check(target!.id),
    enabled: open && force,
    staleTime: 0,
    gcTime: 0,
  })
  const ready = !force || (check.data != null && check.data.blocked == null && check.data.running.length === 0)
  // 확인·정리·결과 단계에서는 다시 찾기를 막고, 정리 중에는 창도 닫지 못하게 한다
  const [locked, setLocked] = useState<"none" | "stage" | "busy">("none")
  useEffect(() => setLocked("none"), [targetProp])
  const scan = useQuery({
    queryKey: ["leftovers", target?.id],
    queryFn: () => scanApi.scanLeftovers(target!.id),
    enabled: open && ready,
    staleTime: 0,
    gcTime: 0,
  })

  return (
    <Dialog open={open} onOpenChange={(next) => !next && locked !== "busy" && onClose()}>
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
                  checking={check.isFetching}
                  onRetry={() => check.refetch()}
                  onClose={onClose}
                />
              ) : scan.isFetching ? (
                <Scanning key="scanning" />
              ) : scan.isError ? (
                <div key="error" className="flex flex-1 items-center justify-center">
                  <ErrorState
                    title="남은 흔적을 찾지 못했어요"
                    hint="아무것도 지우지 않았어요. 다시 찾아보거나 창을 닫아 주세요."
                    error={scan.error}
                    onRetry={() => scan.refetch()}
                    retrying={scan.isFetching}
                  />
                </div>
              ) : scan.data ? (
                <Results
                  key={`result-${scan.dataUpdatedAt}`}
                  report={scan.data}
                  appId={target.id}
                  appName={target.name}
                  forceAppId={force ? target.id : undefined}
                  preview={!force && !target.removed}
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
  checking,
  onRetry,
  onClose,
}: {
  check: ForceCheck | undefined
  error: unknown
  /** 다시 확인하는 중 */
  checking: boolean
  onRetry: () => void
  onClose: () => void
}) {
  // "다시 확인"을 한 번이라도 눌렀으면, 아직 실행 중일 때 그렇다고 알려 준다 (눌러도 화면이 그대로라 확인됐는지 모르니까)
  const [rechecked, setRechecked] = useState(false)
  const retry = () => {
    setRechecked(true)
    onRetry()
  }
  const panel = "flex flex-1 flex-col items-center justify-center gap-4 p-8 text-center"
  const motionProps = {
    initial: { opacity: 0 },
    animate: { opacity: 1 },
    exit: { opacity: 0, transition: { duration: 0.12 } },
  }
  const retryButton = (
    <Button onClick={retry} disabled={checking}>
      {checking ? <Loader2 className="animate-spin" /> : <RefreshCw />}
      다시 확인
    </Button>
  )

  if (error) {
    return (
      <motion.div {...motionProps} className="flex flex-1 flex-col items-center justify-center p-8">
        <ErrorState title="강제 제거해도 되는지 확인하지 못했어요" error={error} onRetry={retry} retrying={checking} />
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
  // 같은 실행 파일이 여러 개 돌면(Electron 앱 등) 한 줄로 묶는다
  const running = [
    ...check.running.reduce((m, p) => m.set(p.path, (m.get(p.path) ?? 0) + 1), new Map<string, number>()),
  ]
  return (
    <motion.div {...motionProps} className="flex min-h-0 flex-1 flex-col">
      {/* 목록이 길어도 아래 버튼은 늘 보이게, 위쪽만 스크롤한다 */}
      <div className="flex min-h-0 flex-1 flex-col items-center gap-4 overflow-y-auto p-8 text-center">
        <div className="mt-auto flex size-14 shrink-0 items-center justify-center rounded-full bg-chart-3/15 text-chart-3">
          <TriangleAlert className="size-7" />
        </div>
        <div className="max-w-md">
          <p className="text-base font-semibold">프로그램이 실행 중이에요</p>
          <p className="mt-2 text-sm leading-relaxed text-muted-foreground">
            실행 중인 파일은 옮길 수 없어요. 아래 프로그램을 직접 닫은 뒤 다시 확인해 주세요. 작업 중인 내용이 있을 수
            있어 RobinX가 대신 끄지는 않아요.
          </p>
        </div>
        <ul className="mb-auto flex w-full max-w-md flex-col gap-1.5 text-left">
          {running.map(([path, count]) => (
            <li key={path} className="flex items-start gap-2 rounded-lg border bg-muted/30 px-3 py-2 text-xs">
              <span data-selectable className="min-w-0 flex-1 font-mono break-all">
                {path}
              </span>
              {count > 1 && <span className="shrink-0 text-muted-foreground tabular-nums">×{count}</span>}
            </li>
          ))}
        </ul>
      </div>
      <div className="flex items-center gap-2 border-t px-5 py-3">
        {rechecked && !checking && (
          <p className="text-xs text-muted-foreground" role="status">
            방금 다시 확인했어요. 아직 실행 중이에요.
          </p>
        )}
        <Button variant="outline" className="ml-auto" onClick={onClose}>
          닫기
        </Button>
        {retryButton}
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
  preview,
  onClose,
  onLockChange,
}: {
  report: ScanReport
  appId: string
  appName: string
  forceAppId?: string
  /** 아직 설치된 프로그램: 보여 주기만 하고 정리하지 않는다 (설치 폴더·등록 정보가 지금 쓰는 것이라) */
  preview: boolean
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
  // 근거가 강한 항목(등록 정보가 직접 가리키는 위치 등)만 미리 선택한다. 미리 보기는 고르지 않는다.
  const initial = () => new Set(preview ? [] : strong.map((i) => i.id))
  const [selected, setSelected] = useState<Set<string>>(initial)
  // "다음부터 제외"를 누른 항목. 이번 정리에서도 빼서, 제외한 것이 그대로 정리되지 않게 한다.
  const [excluded, setExcluded] = useState<Set<string>>(() => new Set())
  useEffect(() => {
    setSelected(initial())
    setExcluded(new Set())
  }, [report])

  const selectedBytes = useMemo(
    () => report.items.filter((i) => selected.has(i.id)).reduce((sum, i) => sum + (i.sizeBytes ?? 0), 0),
    [report, selected],
  )
  const selectedItems = report.items.filter((i) => selected.has(i.id))
  // 정리하는 동안의 진행 (지금 옮기는 항목, 복사한 파일 수)
  const [cleanProgress, setCleanProgress] = useState<ScanProgress | null>(null)

  const runClean = async () => {
    setStage({ step: "cleaning" })
    setCleanError(null)
    setCleanProgress(null)
    try {
      const op = await recoveryApi.clean(appId, appName, [...selected], forceAppId, setCleanProgress)
      setStage({ step: "done", op })
    } catch (e) {
      setCleanError(String(e))
      setStage({ step: "review" })
    } finally {
      for (const key of [
        "backups",
        "backup-usage",
        "old-backups",
        "installed-apps",
        "recently-removed",
        "startup-items",
      ]) {
        queryClient.invalidateQueries({ queryKey: [key] })
      }
    }
  }

  if (stage.step === "confirm") {
    return <ConfirmClean items={selectedItems} onCancel={() => setStage({ step: "review" })} onConfirm={runClean} />
  }
  if (stage.step === "cleaning") {
    return <Cleaning label="백업하고 정리하는 중이에요" progress={cleanProgress} />
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

  const exclude = (id: string) =>
    settingsApi
      .addExclusion(id)
      .then((added) => {
        setExcluded((s) => new Set(s).add(id))
        setSelected((s) => {
          const next = new Set(s)
          next.delete(id)
          return next
        })
        // 설정 화면의 제외 목록이 바로 바뀌게
        queryClient.invalidateQueries({ queryKey: ["settings"] })
        toast.success(added ? "이번 정리와 다음 찾기에서 이 위치는 빼요" : "이미 제외 목록에 있어요", {
          description: "되돌리려면 설정 › 잔여물 스캔에서 뺄 항목을 보세요",
        })
      })
      .catch((e) => toast.error(String(e)))

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
                프로그램 등록 정보와 설치 폴더를 지우면 목록에서 사라져요. 백업해 두므로 보관하는 동안 복구 기록에서
                되돌릴 수 있어요. Windows 서비스는 건드리지 않아요.
              </span>
            </div>
          )}
          <div className="flex flex-wrap items-center gap-2 border-b bg-muted/20 px-5 py-2.5 text-xs">
            <SummaryChip tone="primary" label="연결 정보 일치" value={strong.length} />
            <SummaryChip tone="warn" label="직접 확인 필요" value={review.length} />
            <span className="text-muted-foreground" title={`찾는 데 ${report.durationMs}ms 걸렸어요`}>
              · {report.durationMs < 1000 ? "1초 안에 찾음" : `${Math.round(report.durationMs / 1000)}초 걸림`}
            </span>
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
              title="설치 정보와 연결된 흔적"
              hint={
                preview
                  ? "설치 등록 정보가 직접 가리키는 위치예요"
                  : "설치 등록 정보가 직접 가리키는 위치라 미리 골라 두었어요. 안전을 보장하는 표시는 아니니 한 번 훑어보세요"
              }
              items={strong}
              offset={0}
              selected={selected}
              excluded={excluded}
              onToggle={preview ? undefined : toggle}
              onExclude={preview ? undefined : exclude}
            />
            <Group
              title="직접 확인이 필요한 항목"
              hint={
                preview
                  ? "이름만 비슷하거나 다른 프로그램과 겹칠 수 있어요"
                  : "이름만 비슷하거나 다른 프로그램과 겹칠 수 있어요. 확인한 뒤 직접 선택하세요"
              }
              items={review}
              offset={strong.length}
              selected={selected}
              excluded={excluded}
              onToggle={preview ? undefined : toggle}
              onExclude={preview ? undefined : exclude}
            />
          </div>
        </>
      )}

      <div className="flex items-center gap-3 border-t px-5 py-3">
        {preview ? (
          <p className="mr-auto text-sm text-muted-foreground">
            설치된 상태라 보여 주기만 해요. 프로그램을 제거한 뒤 남은 흔적을 정리할 수 있어요.
          </p>
        ) : (
          <>
            <p className="mr-auto text-sm text-muted-foreground tabular-nums">
              {selected.size > 0 ? `${selected.size}개 선택 · ${formatBytes(selectedBytes)}` : "선택한 항목 없음"}
            </p>
            {cleanError && <p className="text-xs text-destructive">{cleanError}</p>}
            <Button variant="destructive" disabled={selected.size === 0} onClick={() => setStage({ step: "confirm" })}>
              <Trash2 />
              선택 항목 정리
            </Button>
          </>
        )}
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
  excluded,
  onToggle,
  onExclude,
}: {
  title: string
  hint: string
  items: Leftover[]
  offset: number
  selected: Set<string>
  excluded: Set<string>
  /** 없으면 고를 수 없다 (미리 보기) */
  onToggle?: (id: string) => void
  /** 없으면 제외할 수 없다 (미리 보기: 지금 쓰는 설치 폴더를 빼 버릴 수 있어서) */
  onExclude?: (id: string) => void
}) {
  if (items.length === 0) return null
  return (
    <section>
      <div className="sticky top-0 z-10 border-b bg-popover px-5 py-2">
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
            excluded={excluded.has(item.id)}
            onToggle={onToggle && (() => onToggle(item.id))}
            onExclude={onExclude && (() => onExclude(item.id))}
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
  excluded,
  onToggle,
  onExclude,
}: {
  item: Leftover
  index: number
  checked: boolean
  excluded: boolean
  onToggle?: () => void
  onExclude?: () => void
}) {
  const Icon = kindIcons[item.kind]
  const isFs = item.kind === "folder" || item.kind === "file"
  const fullPath = item.valueName ? `${item.location} › ${item.valueName}` : item.location

  return (
    <li
      className={cn("group flex gap-3 px-5 py-3 transition-colors hover:bg-accent/30", index < 12 && "row-intro")}
      style={index < 12 ? ({ "--i": index } as CSSProperties) : undefined}
    >
      {onToggle && (
        <Checkbox
          checked={checked}
          disabled={excluded}
          onCheckedChange={onToggle}
          aria-label={`${fullPath} 선택`}
          className="mt-0.5"
        />
      )}
      <Icon className="mt-0.5 size-4 shrink-0 text-muted-foreground" aria-label={kindLabels[item.kind]} />
      <div className={cn("min-w-0 flex-1", excluded && "opacity-50")}>
        <p data-selectable className="font-mono text-xs leading-5 break-all" title={fullPath}>
          {item.location}
          {item.valueName && (
            <span className="ml-1.5 rounded bg-muted px-1.5 py-0.5 font-sans text-[11px] text-foreground">
              값: {item.valueName}
            </span>
          )}
        </p>
        <div className="mt-1.5 flex flex-wrap gap-1">
          {excluded && (
            <span className="rounded-md border border-foreground/20 px-1.5 py-0.5 text-[11px] font-medium">
              제외함 · 이번에 정리하지 않아요
            </span>
          )}
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
          {onExclude && !excluded && (
            <IconTooltip label="다음부터 제외 (설정에서 되돌릴 수 있어요)">
              <Button variant="ghost" size="icon" className="size-7" aria-label="다음부터 제외" onClick={onExclude}>
                <EyeOff />
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
        <p className="text-base font-semibold">남은 흔적을 찾지 못했어요</p>
        <p className="mt-1 text-sm text-muted-foreground">
          설치 폴더, AppData, 레지스트리에서 이 프로그램과 이어진 항목이 없었어요.
          {protectedSkipped > 0 && ` 보호 규칙으로 ${protectedSkipped}개는 살펴보지 않았어요.`}
        </p>
      </div>
    </div>
  )
}
