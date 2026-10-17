import { useEffect, useMemo, useState } from "react"
import { useQuery, useQueryClient } from "@tanstack/react-query"
import {
  ChevronRight,
  CircleStop,
  FileText,
  Folder,
  KeyRound,
  ListTree,
  Loader2,
  Package,
  Play,
  Radar,
  Save,
  Trash2,
} from "lucide-react"
import { AnimatePresence, motion } from "motion/react"
import { toast } from "sonner"

import { api, trackingApi, type ChangeKind, type Trace, type TraceDraft } from "@/lib/api"
import { cn } from "@/lib/utils"
import { EASE_OUT_QUINT, FadeUp, Stagger } from "@/components/motion"
import { Button } from "@/components/ui/button"
import { Checkbox } from "@/components/ui/checkbox"
import { Dialog, DialogContent, DialogDescription, DialogTitle } from "@/components/ui/dialog"
import { Input } from "@/components/ui/input"
import { Skeleton } from "@/components/ui/skeleton"

const kindIcons: Record<ChangeKind, typeof Folder> = {
  folder: Folder,
  file: FileText,
  registryKey: ListTree,
  registryValue: KeyRound,
}

const kindLabels: Record<ChangeKind, string> = {
  folder: "폴더",
  file: "파일",
  registryKey: "레지스트리 키",
  registryValue: "레지스트리 값",
}

function formatTime(seconds: number) {
  return new Date(seconds * 1000).toLocaleString("ko-KR", {
    month: "long",
    day: "numeric",
    hour: "2-digit",
    minute: "2-digit",
  })
}

export function TrackingPage() {
  const queryClient = useQueryClient()
  const status = useQuery({ queryKey: ["tracking-status"], queryFn: trackingApi.status })
  const traces = useQuery({ queryKey: ["traces"], queryFn: trackingApi.list })
  const apps = useQuery({ queryKey: ["installed-apps"], queryFn: api.listInstalledApps })
  const [busy, setBusy] = useState<"start" | "finish" | "cancel" | null>(null)
  const [draft, setDraft] = useState<TraceDraft | null>(null)

  const appNames = useMemo(() => new Map((apps.data ?? []).map((a) => [a.id, a.displayName])), [apps.data])

  const act = async (kind: "start" | "finish" | "cancel") => {
    setBusy(kind)
    try {
      if (kind === "start") {
        await trackingApi.start()
        toast.success("설치 전 상태를 기록했어요", { description: "이제 설치 프로그램을 실행하세요" })
      } else if (kind === "finish") {
        setDraft(await trackingApi.finish())
        // 방금 설치된 프로그램이 목록과 백엔드 캐시에 들어오게 한다
        queryClient.invalidateQueries({ queryKey: ["installed-apps"] })
      } else {
        await trackingApi.cancel()
        toast("추적을 그만뒀어요")
      }
    } catch (e) {
      toast.error(String(e))
    } finally {
      setBusy(null)
      queryClient.invalidateQueries({ queryKey: ["tracking-status"] })
    }
  }

  const startedAt = status.data ?? null

  return (
    <Stagger className="relative mx-auto flex max-w-4xl min-w-0 flex-col gap-6 px-7 pt-7 pb-10">
      <FadeUp>
        <h1 className="text-2xl font-bold tracking-tight">설치 추적</h1>
        <p className="mt-1 text-sm text-muted-foreground">
          설치하기 전과 후를 비교해 새로 생긴 파일·폴더·레지스트리를 기록해 둬요. 나중에 그 프로그램을 제거하면 이
          기록을 가장 강한 근거로 써서 남은 흔적을 빠짐없이 찾아요. 추적은 읽기만 하고 아무것도 바꾸지 않아요.
        </p>
      </FadeUp>

      <FadeUp>
        <section className="relative overflow-hidden rounded-2xl border bg-card p-6 shadow-sm">
          {status.isLoading ? (
            <Skeleton className="h-24 w-full rounded-xl" />
          ) : startedAt == null ? (
            <div className="flex flex-wrap items-center gap-6">
              <div className="flex size-14 shrink-0 items-center justify-center rounded-2xl bg-primary/12 text-primary">
                <Radar className="size-7" />
              </div>
              <ol className="min-w-0 flex-1 space-y-1 text-sm">
                <li>
                  <b>1.</b> 설치 프로그램을 실행하기 <b>전에</b> 추적을 시작해요
                </li>
                <li>
                  <b>2.</b> 평소처럼 설치하고, 설치가 끝나면 "설치 끝남"을 눌러요
                </li>
                <li>
                  <b>3.</b> 새로 생긴 항목을 확인하고, 그 프로그램 것이 아닌 항목은 빼고 저장해요
                </li>
              </ol>
              <Button onClick={() => act("start")} disabled={busy != null}>
                {busy === "start" ? <Loader2 className="animate-spin" /> : <Play />}
                {busy === "start" ? "설치 전 상태 기록 중…" : "추적 시작"}
              </Button>
            </div>
          ) : (
            <div className="flex flex-wrap items-center gap-6">
              <div className="relative flex size-14 shrink-0 items-center justify-center">
                <span className="absolute inset-0 animate-ping rounded-2xl bg-primary/20 [animation-duration:2s]" />
                <span className="relative flex size-14 items-center justify-center rounded-2xl bg-primary/15 text-primary">
                  <Radar className="size-7" />
                </span>
              </div>
              <div className="min-w-0 flex-1">
                <p className="font-semibold">추적 중이에요 · {formatTime(startedAt)}부터</p>
                <p className="mt-1 text-sm text-muted-foreground">
                  지금 설치 프로그램을 실행하세요. 설치가 끝나면 "설치 끝남"을 눌러 주세요. 그동안 다른 프로그램을
                  설치하거나 지우면 기록에 섞일 수 있어요.
                </p>
              </div>
              <div className="flex gap-2">
                <Button variant="outline" onClick={() => act("cancel")} disabled={busy != null}>
                  <CircleStop />
                  그만두기
                </Button>
                <Button onClick={() => act("finish")} disabled={busy != null}>
                  {busy === "finish" ? <Loader2 className="animate-spin" /> : <Package />}
                  {busy === "finish" ? "비교하는 중…" : "설치 끝남"}
                </Button>
              </div>
            </div>
          )}
        </section>
      </FadeUp>

      <FadeUp>
        <h2 className="mb-3 font-semibold">
          저장한 기록 <span className="text-sm font-normal text-muted-foreground">{traces.data?.length ?? 0}개</span>
        </h2>
        {traces.isLoading ? (
          <Skeleton className="h-20 w-full rounded-2xl" />
        ) : (traces.data ?? []).length === 0 ? (
          <div className="rounded-2xl border border-dashed px-6 py-10 text-center text-sm text-muted-foreground">
            아직 저장한 기록이 없어요. 다음에 프로그램을 설치할 때 추적해 보세요.
          </div>
        ) : (
          <div className="flex flex-col gap-3">
            {(traces.data ?? []).map((t) => (
              <TraceCard key={t.id} trace={t} appNames={appNames} />
            ))}
          </div>
        )}
      </FadeUp>

      <ReviewDialog
        draft={draft}
        onClose={() => setDraft(null)}
        onSaved={() => {
          setDraft(null)
          queryClient.invalidateQueries({ queryKey: ["tracking-status"] })
          queryClient.invalidateQueries({ queryKey: ["traces"] })
        }}
      />
    </Stagger>
  )
}

function ChangeLine({ kind, location, valueName }: { kind: ChangeKind; location: string; valueName: string | null }) {
  const Icon = kindIcons[kind]
  return (
    <>
      <Icon className="mt-0.5 size-4 shrink-0 text-muted-foreground" aria-label={kindLabels[kind]} />
      <p data-selectable className="min-w-0 flex-1 font-mono break-all">
        {location}
        {valueName && ` › ${valueName}`}
      </p>
    </>
  )
}

function ReviewDialog({
  draft,
  onClose,
  onSaved,
}: {
  draft: TraceDraft | null
  onClose: () => void
  onSaved: () => void
}) {
  const [keep, setKeep] = useState<Set<number>>(new Set())
  const [linked, setLinked] = useState<Set<string>>(new Set())
  const [name, setName] = useState("")
  const [saving, setSaving] = useState(false)

  useEffect(() => {
    if (!draft) return
    setKeep(new Set(draft.changes.map((_, i) => i)))
    setLinked(new Set(draft.newApps.map((a) => a.id)))
    setName(draft.newApps[0]?.name ?? "")
  }, [draft])

  const toggle = <T,>(set: Set<T>, value: T) => {
    const next = new Set(set)
    if (next.has(value)) next.delete(value)
    else next.add(value)
    return next
  }

  const save = async () => {
    setSaving(true)
    try {
      const trace = await trackingApi.save(name, [...keep], [...linked])
      toast.success(`기록을 저장했어요 · ${trace.name}`, { description: `항목 ${trace.changes.length}개` })
      onSaved()
    } catch (e) {
      toast.error(String(e))
    } finally {
      setSaving(false)
    }
  }

  return (
    <Dialog open={draft != null} onOpenChange={(o) => !o && !saving && onClose()}>
      <DialogContent className="flex h-[min(720px,88vh)] flex-col gap-0 overflow-hidden p-0 sm:max-w-3xl">
        {draft && (
          <>
            <div className="border-b px-5 py-4 pr-12">
              <DialogTitle className="text-base">설치하면서 새로 생긴 항목</DialogTitle>
              <DialogDescription>
                {formatTime(draft.startedAt)} ~ {formatTime(draft.finishedAt)} · 그 프로그램 것이 아닌 항목은 선택을
                풀어 주세요
              </DialogDescription>
            </div>

            <div className="flex flex-col gap-3 border-b bg-muted/20 px-5 py-3 text-sm">
              <label className="flex items-center gap-3">
                <span className="w-28 shrink-0 text-muted-foreground">기록 이름</span>
                <Input
                  value={name}
                  onChange={(e) => setName(e.target.value)}
                  placeholder="예: 메모 앱 2.1 설치"
                  className="h-8"
                />
              </label>
              <div className="flex items-start gap-3">
                <span className="w-28 shrink-0 pt-0.5 text-muted-foreground">연결할 프로그램</span>
                {draft.newApps.length === 0 ? (
                  <p className="text-muted-foreground">
                    새로 등록된 프로그램이 없어요. 기록은 저장되지만 잔여물 스캔에는 쓰이지 않아요.
                  </p>
                ) : (
                  <div className="flex flex-wrap gap-x-4 gap-y-1">
                    {draft.newApps.map((a) => (
                      <label key={a.id} className="flex cursor-pointer items-center gap-2">
                        <Checkbox
                          checked={linked.has(a.id)}
                          onCheckedChange={() => setLinked((s) => toggle(s, a.id))}
                        />
                        {a.name}
                      </label>
                    ))}
                  </div>
                )}
              </div>
            </div>

            {draft.changes.length === 0 ? (
              <p className="flex flex-1 items-center justify-center p-8 text-sm text-muted-foreground">
                새로 생긴 항목을 찾지 못했어요. 설치가 끝났는지 확인해 주세요.
              </p>
            ) : (
              <ul className="min-h-0 flex-1 divide-y overflow-y-auto">
                {draft.changes.map((c, i) => (
                  <li key={i}>
                    <label className="flex cursor-pointer items-start gap-3 px-5 py-2.5 text-xs hover:bg-accent/40">
                      <Checkbox
                        className="mt-0.5"
                        checked={keep.has(i)}
                        onCheckedChange={() => setKeep((s) => toggle(s, i))}
                      />
                      <ChangeLine {...c} />
                    </label>
                  </li>
                ))}
              </ul>
            )}

            <div className="flex items-center gap-3 border-t px-5 py-3">
              <p className="mr-auto text-sm text-muted-foreground tabular-nums">
                {keep.size} / {draft.changes.length}개 선택
              </p>
              <Button variant="outline" onClick={onClose} disabled={saving}>
                나중에
              </Button>
              <Button onClick={save} disabled={saving}>
                {saving ? <Loader2 className="animate-spin" /> : <Save />}
                기록 저장
              </Button>
            </div>
          </>
        )}
      </DialogContent>
    </Dialog>
  )
}

function TraceCard({ trace, appNames }: { trace: Trace; appNames: Map<string, string> }) {
  const queryClient = useQueryClient()
  const [open, setOpen] = useState(false)
  const [confirming, setConfirming] = useState(false)
  const [deleting, setDeleting] = useState(false)
  const linked = trace.appIds.map((id) => appNames.get(id) ?? "제거된 프로그램")

  const remove = async () => {
    if (deleting) return
    setDeleting(true)
    try {
      await trackingApi.delete(trace.id)
      toast.success("기록을 지웠어요")
    } catch (e) {
      toast.error(String(e))
    } finally {
      setConfirming(false)
      setDeleting(false)
      queryClient.invalidateQueries({ queryKey: ["traces"] })
    }
  }

  return (
    <section className="overflow-hidden rounded-2xl border bg-card shadow-sm">
      <div className="flex flex-wrap items-center gap-3 px-5 py-4">
        <button
          type="button"
          onClick={() => setOpen((v) => !v)}
          aria-expanded={open}
          className="flex min-w-0 flex-1 items-center gap-3 text-left"
        >
          <ChevronRight
            className={cn(
              "size-4 shrink-0 text-muted-foreground transition-transform duration-200",
              open && "rotate-90",
            )}
          />
          <div className="min-w-0">
            <p className="truncate font-semibold">{trace.name}</p>
            <p className="text-xs text-muted-foreground">
              {formatTime(trace.finishedAt)} · 항목 {trace.changes.length}개
              {linked.length > 0 ? ` · ${linked.join(", ")}` : " · 연결된 프로그램 없음"}
            </p>
          </div>
        </button>
        {confirming ? (
          <div className="flex items-center gap-2 text-xs">
            <span className="text-muted-foreground">기록만 지워요. 프로그램은 그대로예요</span>
            <Button size="sm" variant="outline" onClick={() => setConfirming(false)}>
              취소
            </Button>
            <Button size="sm" variant="destructive" onClick={remove} disabled={deleting}>
              지우기
            </Button>
          </div>
        ) : (
          <Button
            size="icon"
            variant="ghost"
            className="size-8 text-muted-foreground hover:text-destructive"
            aria-label="기록 지우기"
            onClick={() => setConfirming(true)}
          >
            <Trash2 />
          </Button>
        )}
      </div>
      <AnimatePresence initial={false}>
        {open && (
          <motion.ul
            initial={{ height: 0, opacity: 0 }}
            animate={{ height: "auto", opacity: 1 }}
            exit={{ height: 0, opacity: 0 }}
            transition={{ duration: 0.3, ease: EASE_OUT_QUINT }}
            className="divide-y overflow-hidden border-t"
          >
            {trace.changes.map((c, i) => (
              <li key={i} className="flex items-start gap-3 px-5 py-2.5 text-xs">
                <ChangeLine {...c} />
              </li>
            ))}
          </motion.ul>
        )}
      </AnimatePresence>
    </section>
  )
}
