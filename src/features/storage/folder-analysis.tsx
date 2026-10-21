import { useSyncExternalStore } from "react"
import { ArrowUp, File, Folder, FolderOpen, Package, Radar, RefreshCw, Square } from "lucide-react"

import { navigate, storageApi, type FolderAnalysis, type ScanProgress } from "@/lib/api"
import { formatAgo, formatBytes, formatCount } from "@/lib/format"
import { cn } from "@/lib/utils"
import { shortenPath } from "@/components/scan-progress"
import { IconTooltip } from "@/components/icon-tooltip"
import { ScanProgressView } from "@/components/scan-progress"
import { InlineError } from "@/components/error-state"
import { Button } from "@/components/ui/button"
import { failed } from "@/lib/notify"

/**
 * 분석 결과와 진행 상태는 화면 밖(모듈)에 둔다. 다른 탭으로 갔다 와도 결과가 남고, 분석 중이었다면 이어서 보인다.
 * (예전에는 탭을 바꾸면 컴포넌트와 함께 사라져서, 뒤에서 끝난 분석 결과를 버리고 처음 화면으로 돌아갔다)
 */
type AnalysisState = {
  result: FolderAnalysis | null
  loading: string | null
  progress: ScanProgress | null
  /** 마지막 분석이 실패한 이유 (카드에 남겨 둔다) */
  error: string | null
  /** 사용자가 마지막 분석을 멈췄는지 (결과는 이전 것을 그대로 둔다) */
  stopped: boolean
  /** 지금 결과를 받은 때 */
  at: number | null
}
let analysisState: AnalysisState = {
  result: null,
  loading: null,
  progress: null,
  error: null,
  stopped: false,
  at: null,
}
const listeners = new Set<() => void>()
function setAnalysis(patch: Partial<AnalysisState>) {
  analysisState = { ...analysisState, ...patch }
  for (const l of listeners) l()
}
function subscribe(listener: () => void) {
  listeners.add(listener)
  return () => listeners.delete(listener)
}

async function load(id?: string, label?: string) {
  setAnalysis({ loading: label ?? "C 드라이브", progress: null, error: null, stopped: false })
  try {
    const next = await storageApi.analyze(id, (progress) => setAnalysis({ progress }))
    // 다른 분석을 시작해 멈춘 결과는 버린다
    // 새 분석이 대신한 결과는 조용히 버리고, 사용자가 멈췄으면 그렇다고 남긴다 (반쯤 센 크기는 보이지 않는다)
    if (next.superseded) return
    if (next.cancelled) setAnalysis({ stopped: true })
    else setAnalysis({ result: next, at: Date.now() })
  } catch (e) {
    setAnalysis({ error: String(e) })
  } finally {
    setAnalysis({ loading: null })
  }
}

/** 한 번에 보여 주는 하위 항목 수 */
const SHOWN = 30

/** 큰 폴더 찾기. 크기를 보여 주고 탐색기로 열어 주기만 한다 (사용자 파일은 지우지 않는다). */
export function FolderAnalysisCard() {
  const { result, loading, progress, error, stopped, at } = useSyncExternalStore(subscribe, () => analysisState)

  const max = result?.items[0]?.bytes ?? 0

  return (
    <section className="overflow-clip rounded-2xl border bg-card shadow-sm">
      <div className="flex flex-wrap items-center gap-3 border-b px-5 py-4">
        <div className="mr-auto min-w-0">
          <h2 className="font-semibold">큰 폴더 찾기</h2>
          <p className="text-xs text-muted-foreground">
            무엇이 공간을 차지하는지 보여 줘요. 여기서는 지우지 않아요. 필요 없는 프로그램은 프로그램 화면에서
            제거하세요.
          </p>
        </div>
        <Button variant="outline" size="sm" onClick={() => navigate("programs")}>
          <Package />
          프로그램 화면
        </Button>
        {loading ? (
          <Button variant="outline" size="sm" onClick={() => storageApi.cancelAnalysis()}>
            <Square />
            멈추기
          </Button>
        ) : (
          !result && (
            <Button size="sm" onClick={() => load()}>
              <Radar />
              분석 시작
            </Button>
          )
        )}
      </div>

      {result && (
        <div className="flex items-center gap-2 border-b bg-muted/30 px-5 py-2">
          <IconTooltip label="상위 폴더">
            <Button
              variant="ghost"
              size="icon"
              className="size-7"
              disabled={!result.parentId || loading != null}
              onClick={() => result.parentId && load(result.parentId, "상위 폴더")}
              aria-label="상위 폴더"
            >
              <ArrowUp />
            </Button>
          </IconTooltip>
          {/* 좁은 창에서 뒤가 잘리면 지금 폴더 이름이 안 보이므로, 앞쪽을 줄이고 끝 폴더를 남긴다 */}
          <p data-selectable className="min-w-0 flex-1 truncate font-mono text-xs" title={result.path}>
            {shortenPath(result.path)}
          </p>
          {at != null && <span className="shrink-0 text-xs text-muted-foreground">{formatAgo(at)}에 셈</span>}
          <IconTooltip label="이 폴더 다시 세기">
            <Button
              variant="ghost"
              size="icon"
              className="size-7"
              disabled={loading != null}
              onClick={() => load(result.id, "이 폴더")}
              aria-label="이 폴더 다시 세기"
            >
              <RefreshCw />
            </Button>
          </IconTooltip>
        </div>
      )}

      {loading && (
        <ScanProgressView
          className={cn("px-5", result ? "border-b py-3" : "py-6")}
          label={`${loading} 크기를 세는 중이에요`}
          progress={progress}
          hint={result ? undefined : "파일이 많으면 1~2분 걸릴 수 있어요. 언제든 멈출 수 있어요."}
        />
      )}
      {stopped && !loading && (
        <p className="border-b bg-muted/30 px-5 py-2 text-xs text-muted-foreground" role="status">
          분석을 멈췄어요.{result ? " 아래는 그 전에 센 결과예요." : ""}
        </p>
      )}
      {error && !loading && (
        <InlineError
          className="border-b px-5 py-3"
          message="폴더 크기를 세지 못했어요"
          error={error}
          onRetry={() => void load()}
        />
      )}
      {loading && !result ? null : !result ? (
        <p className="px-5 py-10 text-center text-sm text-muted-foreground">
          "분석 시작"을 누르면 C 드라이브부터 폴더별 크기를 세요.
        </p>
      ) : result.items.length === 0 ? (
        <p className="px-5 py-10 text-center text-sm text-muted-foreground">비어 있거나 열 수 없는 폴더예요.</p>
      ) : (
        <>
          <ul
            className={cn("divide-y transition-opacity", loading && "pointer-events-none opacity-50")}
            aria-busy={loading != null}
          >
            {result.items.slice(0, SHOWN).map((item) => {
              const pct = max > 0 ? (item.bytes / max) * 100 : 0
              const Icon = item.isDir ? Folder : File
              return (
                <li key={item.id} className="group flex items-center gap-3 px-5 py-2">
                  <button
                    type="button"
                    disabled={!item.isDir}
                    onClick={() => load(item.id, item.name)}
                    className={cn("flex min-w-0 flex-1 items-center gap-3 text-left", item.isDir && "cursor-pointer")}
                  >
                    <Icon className="size-4 shrink-0 text-muted-foreground" />
                    <div className="min-w-0 flex-1">
                      <p className={cn("truncate text-sm", item.isDir && "group-hover:underline")}>{item.name}</p>
                      <div className="mt-1 h-1.5 overflow-hidden rounded-full bg-muted">
                        <div className="forced-fill h-full rounded-full bg-chart-2" style={{ width: `${pct}%` }} />
                      </div>
                    </div>
                    <span
                      className="w-20 shrink-0 text-right text-sm font-medium tabular-nums"
                      title={
                        item.approximate
                          ? "Windows 폴더는 같은 파일이 여러 곳에 연결(하드 링크)돼 있어 실제 차지하는 공간보다 크게 보일 수 있어요"
                          : undefined
                      }
                    >
                      {item.approximate && "약 "}
                      {formatBytes(item.bytes)}
                    </span>
                    <span className="hidden w-24 shrink-0 text-right text-xs text-muted-foreground tabular-nums sm:block">
                      {item.isDir ? `파일 ${formatCount(item.files)}개` : ""}
                      {item.inaccessible > 0 && <span className="block">일부 열 수 없음</span>}
                    </span>
                  </button>
                  <IconTooltip label="탐색기에서 열기">
                    <Button
                      variant="ghost"
                      size="icon"
                      className="size-7 text-muted-foreground"
                      onClick={() =>
                        storageApi
                          .revealFolderItem(item.id)
                          .catch(failed("탐색기에서 열지 못했어요. 이미 지웠거나 옮겼다면 다시 세어 주세요"))
                      }
                      aria-label={`${item.name} 탐색기에서 열기`}
                    >
                      <FolderOpen />
                    </Button>
                  </IconTooltip>
                </li>
              )
            })}
          </ul>
          {result.items.length > SHOWN && (
            <p className="border-t px-5 py-2 text-xs text-muted-foreground">
              큰 것부터 {SHOWN}개만 보여요 (모두 {result.items.length}개)
            </p>
          )}
        </>
      )}
    </section>
  )
}
