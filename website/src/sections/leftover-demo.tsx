import { useMemo, useState } from "react"
import { AnimatePresence, motion, useReducedMotion } from "motion/react"
import { Archive, Check, FileText, Folder, KeyRound, Loader2, RotateCcw, ShieldCheck, Sparkles } from "lucide-react"

import { cn } from "@/lib/utils"

/**
 * 첫 화면의 "남은 흔적" 화면 조각. 앱의 어두운 화면을 그대로 옮긴 HTML이고, 실제로 눌러 볼 수 있다:
 * 항목을 고르면 합계가 바뀌고, "백업하고 정리"를 누르면 복구 기록으로 옮겨졌다가 "되돌리기"로 돌아온다.
 * 데이터는 설명용 예시 프로그램(Pixel Forge 3D)이다. 이 페이지에서는 아무것도 지우지 않는다.
 */

type Kind = "folder" | "registry" | "value"
type Item = { id: string; kind: Kind; path: string; reason: string; mb: number | null; strong: boolean }

const ITEMS: Item[] = [
  {
    id: "a",
    kind: "folder",
    path: "C:\\Program Files\\Forge Interactive\\Pixel Forge 3D",
    reason: "등록된 설치 폴더",
    mb: 204,
    strong: true,
  },
  {
    id: "b",
    kind: "registry",
    path: "HKLM\\SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Uninstall\\PixelForge3D",
    reason: "남은 제거 등록 정보",
    mb: null,
    strong: true,
  },
  {
    id: "c",
    kind: "value",
    path: "HKCU\\…\\CurrentVersion\\Run  값: PixelForgeHelper",
    reason: "설치 폴더를 가리킴",
    mb: null,
    strong: true,
  },
  {
    id: "d",
    kind: "folder",
    path: "C:\\Users\\me\\AppData\\Roaming\\Forge Interactive\\Pixel Forge 3D",
    reason: "게시자 › 제품 이름 일치",
    mb: 37,
    strong: false,
  },
  {
    id: "e",
    kind: "folder",
    path: "C:\\Users\\me\\AppData\\Local\\Pixel Forge 3D",
    reason: "게시자 › 제품 이름 일치",
    mb: 1229,
    strong: false,
  },
]

const icon = { folder: Folder, registry: FileText, value: KeyRound }

function size(mb: number) {
  return mb >= 1000 ? `${(mb / 1024).toFixed(1)} GB` : `${mb} MB`
}

type Phase = "pick" | "cleaning" | "done"

export function LeftoverDemo() {
  const reduce = useReducedMotion()
  const [picked, setPicked] = useState(() => new Set(ITEMS.filter((i) => i.strong).map((i) => i.id)))
  const [phase, setPhase] = useState<Phase>("pick")
  const [cleaned, setCleaned] = useState<Item[]>([])

  const total = useMemo(() => ITEMS.filter((i) => picked.has(i.id)).reduce((s, i) => s + (i.mb ?? 0), 0), [picked])

  const toggle = (id: string) =>
    setPicked((p) => {
      const n = new Set(p)
      if (n.has(id)) n.delete(id)
      else n.add(id)
      return n
    })

  const clean = () => {
    setPhase("cleaning")
    setTimeout(
      () => {
        setCleaned(ITEMS.filter((i) => picked.has(i.id)))
        setPhase("done")
      },
      reduce ? 0 : 900,
    )
  }

  const undo = () => {
    setCleaned([])
    setPhase("pick")
  }

  const visible = ITEMS.filter((i) => !cleaned.some((c) => c.id === i.id))
  const groups = [
    { title: "설치 정보와 연결된 흔적", note: "근거가 강해 미리 골라 뒀어요", items: visible.filter((i) => i.strong) },
    {
      title: "직접 확인이 필요한 항목",
      note: "이름만 비슷하거나 다른 프로그램과 겹칠 수 있어요",
      items: visible.filter((i) => !i.strong),
    },
  ]

  return (
    <div
      data-ui
      role="group"
      aria-label="남은 흔적 화면 예시 (눌러 볼 수 있어요. 실제로 지우지는 않아요)"
      className="w-[min(40rem,calc(100vw-1.25rem))] max-w-none rounded-tl-xl bg-[oklch(0.225_0.004_260)] text-[0.875rem] text-[oklch(0.96_0.002_260)]"
    >
      {/* 머리 */}
      <div className="flex items-center gap-3 px-5 pt-5 pb-4">
        <span className="flex size-10 items-center justify-center rounded-xl bg-[oklch(0.3_0.03_200)] text-[oklch(0.82_0.12_190)]">
          <Sparkles className="size-5" />
        </span>
        <div className="min-w-0">
          <p className="font-semibold">남은 흔적</p>
          <p className="truncate text-[0.8125rem] text-[oklch(0.72_0.006_260)]">
            Pixel Forge 3D · 제거한 뒤 남은 파일과 레지스트리
          </p>
        </div>
      </div>
      <div className="flex items-center gap-2 bg-white/[0.035] px-5 py-2.5 text-[0.8125rem]">
        <span className="shrink-0 rounded-full whitespace-nowrap bg-[oklch(0.82_0.12_190/15%)] px-2.5 py-0.5 font-medium text-[oklch(0.82_0.12_190)]">
          연결 정보 일치 {ITEMS.filter((i) => i.strong).length}
        </span>
        <span className="shrink-0 rounded-full whitespace-nowrap bg-[oklch(0.83_0.14_75/15%)] px-2.5 py-0.5 font-medium text-[oklch(0.83_0.14_75)]">
          직접 확인 필요 {ITEMS.filter((i) => !i.strong).length}
        </span>
        <span className="ml-auto hidden items-center gap-1.5 whitespace-nowrap text-[oklch(0.72_0.006_260)] sm:flex">
          <ShieldCheck className="size-3.5 text-[oklch(0.75_0.14_150)]" />
          보호 규칙으로 3개 제외
        </span>
      </div>

      {/* 목록 */}
      <div className="min-h-[19rem]">
        <AnimatePresence initial={false}>
          {phase === "done" && (
            <motion.div
              key="done"
              initial={reduce ? false : { opacity: 0, height: 0 }}
              animate={{ opacity: 1, height: "auto" }}
              exit={{ opacity: 0, height: 0 }}
              transition={{ duration: 0.4, ease: [0.22, 1, 0.36, 1] }}
              className="overflow-hidden"
            >
              <div className="m-4 flex items-start gap-3 rounded-xl bg-[oklch(0.75_0.14_150/12%)] p-4" role="status">
                <Archive className="mt-0.5 size-4 shrink-0 text-[oklch(0.75_0.14_150)]" />
                <div className="min-w-0 flex-1">
                  <p className="font-medium">{cleaned.length}개를 백업하고 정리했어요</p>
                  <p className="mt-0.5 text-[0.8125rem] text-[oklch(0.72_0.006_260)]">
                    파일은 격리 폴더로, 레지스트리는 내보낸 파일로 복구 기록에 남아 있어요.
                  </p>
                </div>
                <button
                  type="button"
                  onClick={undo}
                  className="inline-flex h-8 shrink-0 items-center gap-1.5 rounded-lg bg-white/10 px-3 text-[0.8125rem] font-medium transition-colors hover:bg-white/15 focus-visible:ring-2 focus-visible:ring-[oklch(0.82_0.12_190)] focus-visible:outline-none active:scale-[0.97]"
                >
                  <RotateCcw className="size-3.5" />
                  되돌리기
                </button>
              </div>
            </motion.div>
          )}
        </AnimatePresence>

        {groups.map(
          (g) =>
            g.items.length > 0 && (
              <div key={g.title}>
                <div className="px-5 pt-4 pb-1.5">
                  <p className="text-[0.8125rem] font-semibold">
                    {g.title} <span className="text-[oklch(0.72_0.006_260)]">{g.items.length}</span>
                  </p>
                  <p className="text-[0.8125rem] text-[oklch(0.72_0.006_260)]">{g.note}</p>
                </div>
                <ul>
                  <AnimatePresence initial={false}>
                    {g.items.map((item) => {
                      const Icon = icon[item.kind]
                      const on = picked.has(item.id)
                      return (
                        <motion.li
                          key={item.id}
                          layout={!reduce}
                          exit={reduce ? undefined : { opacity: 0, x: 24, transition: { duration: 0.3 } }}
                          className=""
                        >
                          <label
                            className={cn(
                              "flex cursor-pointer items-start gap-3 px-5 py-3 transition-colors hover:bg-white/[0.04]",
                              phase !== "pick" && "pointer-events-none",
                            )}
                          >
                            <input
                              type="checkbox"
                              checked={on}
                              onChange={() => toggle(item.id)}
                              disabled={phase !== "pick"}
                              className="peer sr-only"
                            />
                            <span
                              aria-hidden
                              className={cn(
                                "mt-0.5 flex size-4 shrink-0 items-center justify-center rounded-[0.3rem] transition-colors peer-focus-visible:ring-2 peer-focus-visible:ring-[oklch(0.82_0.12_190)]",
                                on ? "bg-[oklch(0.82_0.12_190)] text-[oklch(0.2_0.04_200)]" : "bg-white/12",
                              )}
                            >
                              {on && <Check className="size-3" strokeWidth={3} />}
                            </span>
                            <Icon className="mt-0.5 size-4 shrink-0 text-[oklch(0.72_0.006_260)]" />
                            <span className="min-w-0 flex-1">
                              <span className="block truncate text-[0.8125rem]">{item.path}</span>
                              <span className="mt-1.5 inline-block rounded-md bg-white/8 px-2 py-0.5 text-[0.8125rem] text-[oklch(0.8_0.006_260)]">
                                {item.reason}
                              </span>
                            </span>
                            <span className="shrink-0 text-[0.8125rem] text-[oklch(0.72_0.006_260)] tabular-nums">
                              {item.mb == null ? "" : size(item.mb)}
                            </span>
                          </label>
                        </motion.li>
                      )
                    })}
                  </AnimatePresence>
                </ul>
              </div>
            ),
        )}
      </div>

      {/* 바닥 */}
      <div className="flex items-center gap-3 bg-white/[0.035] px-5 py-4">
        <p className="text-[0.8125rem] text-[oklch(0.72_0.006_260)] tabular-nums" aria-live="polite">
          {phase === "done" ? "복구 기록에서 언제든 되돌릴 수 있어요" : `${picked.size}개 선택 · ${size(total)}`}
        </p>
        <button
          type="button"
          onClick={clean}
          disabled={phase !== "pick" || picked.size === 0}
          className="ml-auto inline-flex h-9 items-center gap-2 rounded-lg bg-[oklch(0.82_0.12_190)] px-4 text-[0.8125rem] font-semibold text-[oklch(0.2_0.04_200)] transition-[transform,opacity] hover:opacity-90 focus-visible:ring-2 focus-visible:ring-white focus-visible:outline-none active:scale-[0.97] disabled:opacity-40"
        >
          {phase === "cleaning" ? <Loader2 className="size-4 animate-spin" /> : <Archive className="size-4" />}
          {phase === "cleaning" ? "백업하는 중" : "백업하고 정리"}
        </button>
      </div>
    </div>
  )
}
