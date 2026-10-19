import { useState } from "react"
import { useQuery, useQueryClient } from "@tanstack/react-query"
import {
  BellOff,
  ChevronRight,
  FlaskConical,
  Gamepad2,
  Loader2,
  MousePointerClick,
  RefreshCw,
  RotateCcw,
  ShieldCheck,
  Undo2,
} from "lucide-react"
import { AnimatePresence, motion } from "motion/react"
import { toast } from "sonner"

import { tweaksApi, type TweakGroup, type TweakView } from "@/lib/api"
import { cn } from "@/lib/utils"
import { EASE_OUT_QUINT, FadeUp, Stagger } from "@/components/motion"
import { Button } from "@/components/ui/button"
import { Dialog, DialogContent, DialogDescription, DialogTitle } from "@/components/ui/dialog"
import { Skeleton } from "@/components/ui/skeleton"
import { Switch } from "@/components/ui/switch"

const groups: { key: TweakGroup; title: string; description: string; icon: typeof ShieldCheck }[] = [
  {
    key: "privacy",
    title: "개인정보",
    description: "맞춤 광고와 기록을 줄여요. 진단 데이터 전송 같은 시스템 정책은 바꾸지 않아요.",
    icon: ShieldCheck,
  },
  {
    key: "suggestions",
    title: "추천·제안 줄이기",
    description: "시작 메뉴, 설정 앱, 잠금 화면에 뜨는 추천과 팁을 줄여요.",
    icon: BellOff,
  },
  { key: "convenience", title: "편의", description: "쓰기 편하게 바꾸는 설정이에요.", icon: MousePointerClick },
  { key: "gaming", title: "게임", description: "게임할 때 백그라운드에서 하는 일을 줄여요.", icon: Gamepad2 },
  {
    key: "test",
    title: "테스트 (개발용)",
    description: "개발 빌드에만 보여요. 테스트 앱의 키만 바꿔요.",
    icon: FlaskConical,
  },
]

const restartHint = {
  none: null,
  explorer: "탐색기를 새로 고치거나(F5) 다시 시작하면 보여요",
  signOut: "다시 로그인하면 반영돼요",
} as const

function formatDate(seconds: number) {
  return new Date(seconds * 1000).toLocaleString("ko-KR", {
    month: "long",
    day: "numeric",
    hour: "2-digit",
    minute: "2-digit",
  })
}

export function TweaksPage() {
  const queryClient = useQueryClient()
  const tweaks = useQuery({ queryKey: ["tweaks"], queryFn: tweaksApi.list })
  const [busy, setBusy] = useState<string | null>(null)
  const [revertAll, setRevertAll] = useState(false)

  const list = tweaks.data ?? []
  const applied = list.filter((t) => t.state === "applied").length
  const ours = list.filter((t) => t.changedByUs)

  const toggle = async (t: TweakView, on: boolean) => {
    setBusy(t.id)
    try {
      if (on) {
        await tweaksApi.apply(t.id)
        toast.success(`${t.title}: 적용했어요`, {
          description: restartHint[t.restart] ?? "원래 값을 기록해 두었어요. 언제든 끌 수 있어요.",
        })
      } else {
        const { leftAsIs } = await tweaksApi.revert(t.id)
        if (leftAsIs.length > 0) {
          toast.warning(`${t.title}: 일부만 되돌렸어요`, {
            description: "그 사이 다른 곳에서 바뀐 값은 덮어쓰지 않았어요.",
          })
        } else {
          toast.success(`${t.title}: 원래대로 되돌렸어요`, { description: restartHint[t.restart] ?? undefined })
        }
      }
    } catch (e) {
      toast.error(String(e))
    } finally {
      setBusy(null)
      queryClient.invalidateQueries({ queryKey: ["tweaks"] })
    }
  }

  return (
    <Stagger className="relative mx-auto flex max-w-4xl min-w-0 flex-col gap-6 px-7 pt-7 pb-10">
      <FadeUp>
        <div className="flex flex-wrap items-end justify-between gap-4">
          <div className="min-w-0">
            <h1 className="text-2xl font-bold tracking-tight">최적화</h1>
            <p className="mt-1 text-sm text-muted-foreground">
              Windows 설정 앱에 있는 설정만 내 계정 범위에서 바꿔요. 바꾸기 전 값을 기록해 두어 끄면 원래대로 돌아가요.
              보안 설정(Defender·UAC·업데이트·방화벽)은 건드리지 않아요.
            </p>
          </div>
          <div className="flex items-center gap-2">
            <Button
              variant="outline"
              size="icon"
              aria-label="새로고침"
              onClick={() => queryClient.invalidateQueries({ queryKey: ["tweaks"] })}
            >
              <RefreshCw className={cn(tweaks.isFetching && "animate-spin")} />
            </Button>
            <Button variant="outline" disabled={ours.length === 0 || busy != null} onClick={() => setRevertAll(true)}>
              <RotateCcw />
              모두 되돌리기
            </Button>
          </div>
        </div>
        {!tweaks.isLoading && (
          <p className="mt-3 text-xs text-muted-foreground">
            {list.length}개 중 {applied}개 적용됨 · RobinX가 바꾼 것 {ours.length}개
          </p>
        )}
      </FadeUp>

      {tweaks.isLoading ? (
        <FadeUp>
          <Skeleton className="h-64 w-full rounded-2xl" />
        </FadeUp>
      ) : tweaks.isError ? (
        <FadeUp>
          <p className="rounded-2xl border border-dashed p-8 text-center text-sm text-muted-foreground">
            최적화 항목을 읽지 못했어요: {String(tweaks.error)}
          </p>
        </FadeUp>
      ) : (
        groups.map((g) => {
          const items = list.filter((t) => t.group === g.key)
          if (items.length === 0) return null
          return (
            <FadeUp key={g.key}>
              <section className="overflow-hidden rounded-2xl border bg-card shadow-sm">
                <header className="flex items-start gap-3 border-b px-5 py-4">
                  <span className="flex size-9 shrink-0 items-center justify-center rounded-xl bg-primary/10 text-primary">
                    <g.icon className="size-4.5" />
                  </span>
                  <div className="min-w-0">
                    <h2 className="font-semibold">{g.title}</h2>
                    <p className="text-xs text-muted-foreground">{g.description}</p>
                  </div>
                </header>
                <ul className="divide-y">
                  {items.map((t) => (
                    <TweakRow key={t.id} tweak={t} busy={busy === t.id} disabled={busy != null} onToggle={toggle} />
                  ))}
                </ul>
              </section>
            </FadeUp>
          )
        })
      )}

      <RevertAllDialog
        open={revertAll}
        items={ours}
        onClose={() => setRevertAll(false)}
        onDone={() => queryClient.invalidateQueries({ queryKey: ["tweaks"] })}
      />
    </Stagger>
  )
}

function TweakRow({
  tweak: t,
  busy,
  disabled,
  onToggle,
}: {
  tweak: TweakView
  busy: boolean
  disabled: boolean
  onToggle: (t: TweakView, on: boolean) => void
}) {
  const [open, setOpen] = useState(false)
  const on = t.state === "applied"
  // RobinX가 바꾸지 않았는데 이미 적용된 상태면, 원래 값을 모르므로 여기서 끄지 않는다
  const lockedOn = on && !t.changedByUs
  const hint = restartHint[t.restart]

  return (
    <li className="px-5 py-4">
      <div className="flex items-start gap-4">
        <div className="min-w-0 flex-1">
          <p className="font-medium">{t.title}</p>
          <p className="mt-0.5 text-sm text-muted-foreground">{t.description}</p>
          <p className="mt-1 text-xs text-muted-foreground/90">{t.why}</p>
          <div className="mt-2 flex flex-wrap items-center gap-x-3 gap-y-1 text-xs">
            {t.changedByUs && t.appliedAt != null && (
              <span className="inline-flex items-center gap-1 text-primary">
                <Undo2 className="size-3.5" />
                RobinX가 {formatDate(t.appliedAt)}에 바꿈 · 끄면 원래 값으로
              </span>
            )}
            {lockedOn && (
              <span className="text-muted-foreground">이미 이렇게 설정돼 있어요 (Windows 설정 또는 다른 프로그램)</span>
            )}
            {t.state === "partial" && <span className="text-chart-3">일부만 적용돼 있어요. 켜면 나머지도 맞춰요</span>}
            {hint && <span className="text-muted-foreground">· {hint}</span>}
            <button
              type="button"
              onClick={() => setOpen((v) => !v)}
              aria-expanded={open}
              className="inline-flex items-center gap-0.5 text-muted-foreground transition-colors hover:text-foreground focus-visible:rounded focus-visible:ring-2 focus-visible:ring-ring focus-visible:outline-none"
            >
              <ChevronRight className={cn("size-3.5 transition-transform", open && "rotate-90")} />
              바꾸는 값 보기
            </button>
          </div>
          <AnimatePresence initial={false}>
            {open && (
              <motion.div
                initial={{ height: 0, opacity: 0 }}
                animate={{ height: "auto", opacity: 1 }}
                exit={{ height: 0, opacity: 0 }}
                transition={{ duration: 0.25, ease: EASE_OUT_QUINT }}
                className="overflow-hidden"
              >
                <div className="mt-3 rounded-xl border bg-muted/30 px-4 py-3 text-xs">
                  <ul className="space-y-1.5 font-mono" data-selectable>
                    {t.changes.map((c) => (
                      <li key={c.location + c.name} className="break-all">
                        {c.location.replace(/ → .*$/, "")} <span className="text-muted-foreground">→</span> {c.name} ={" "}
                        <span className="text-primary">{c.value}</span>
                      </li>
                    ))}
                  </ul>
                  {t.settingsPath && (
                    <p className="mt-2 text-muted-foreground">Windows에서 직접 바꾸려면: {t.settingsPath}</p>
                  )}
                </div>
              </motion.div>
            )}
          </AnimatePresence>
        </div>
        <div className="flex h-6 items-center gap-2 pt-0.5">
          {busy && <Loader2 className="size-4 animate-spin text-muted-foreground" />}
          <Switch
            checked={on}
            disabled={disabled || lockedOn}
            onCheckedChange={(v) => onToggle(t, v)}
            aria-label={`${t.title} ${on ? "끄기" : "켜기"}`}
          />
        </div>
      </div>
    </li>
  )
}

function RevertAllDialog({
  open,
  items,
  onClose,
  onDone,
}: {
  open: boolean
  items: TweakView[]
  onClose: () => void
  onDone: () => void
}) {
  const [busy, setBusy] = useState(false)

  const run = async () => {
    setBusy(true)
    let failed = 0
    let partial = 0
    for (const t of items) {
      try {
        const { leftAsIs } = await tweaksApi.revert(t.id)
        if (leftAsIs.length > 0) partial++
      } catch {
        failed++
      }
    }
    setBusy(false)
    onDone()
    onClose()
    if (failed > 0) toast.error(`${failed}개는 되돌리지 못했어요. 항목을 하나씩 다시 시도해 주세요`)
    else if (partial > 0) toast.warning(`되돌렸어요. ${partial}개는 다른 곳에서 바뀐 값을 그대로 두었어요`)
    else toast.success("모두 원래대로 되돌렸어요")
  }

  return (
    <Dialog open={open} onOpenChange={(v) => !v && !busy && onClose()}>
      <DialogContent className="sm:max-w-md">
        <DialogTitle>RobinX가 바꾼 설정 {items.length}개를 되돌릴까요?</DialogTitle>
        <DialogDescription className="leading-relaxed">
          바꾸기 전 기록해 둔 값으로 돌아가요. 그 사이 Windows 설정 등에서 다시 바꾼 값은 덮어쓰지 않아요.
        </DialogDescription>
        <ul className="max-h-48 divide-y overflow-y-auto rounded-xl border text-sm">
          {items.map((t) => (
            <li key={t.id} className="px-3 py-2">
              {t.title}
            </li>
          ))}
        </ul>
        <div className="mt-1 flex justify-end gap-2">
          <Button variant="outline" onClick={onClose} disabled={busy}>
            취소
          </Button>
          <Button onClick={run} disabled={busy}>
            {busy ? <Loader2 className="animate-spin" /> : <RotateCcw />}
            모두 되돌리기
          </Button>
        </div>
      </DialogContent>
    </Dialog>
  )
}
