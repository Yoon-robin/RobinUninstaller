import { useSyncExternalStore } from "react"
import {
  FileArchive,
  FileVideo,
  File,
  FolderOpen,
  Disc3,
  Package,
  Loader2,
  ScanSearch,
  Square,
  RefreshCw,
} from "lucide-react"
import { toast } from "sonner"

import { storageApi, type LargeFiles } from "@/lib/api"
import { formatBytes } from "@/lib/format"
import { IconTooltip } from "@/components/icon-tooltip"
import { Button } from "@/components/ui/button"

/**
 * 큰 파일 찾기. 공간을 많이 차지하는 파일을 보여 주고 탐색기로 열어 줄 뿐, 아무것도 지우지 않는다.
 * 사용자 파일은 자동으로 지우지 않는다는 원칙(CLAUDE.md "삭제 원칙")에 따라, 지울지는 사용자가 탐색기에서 정한다.
 * 결과는 화면 밖(모듈)에 둬서 다른 탭에 다녀와도 남는다.
 */
type State = { result: LargeFiles | null; loading: boolean }
let state: State = { result: null, loading: false }
const listeners = new Set<() => void>()
function set(patch: Partial<State>) {
  state = { ...state, ...patch }
  for (const l of listeners) l()
}
function subscribe(listener: () => void) {
  listeners.add(listener)
  return () => listeners.delete(listener)
}

async function find() {
  set({ loading: true })
  try {
    const next = await storageApi.findLargeFiles()
    if (!next.cancelled) set({ result: next })
  } catch (e) {
    toast.error(String(e))
  } finally {
    set({ loading: false })
  }
}

/** 확장자로 대강의 종류 (설명용) */
function kindOf(name: string): { label: string; icon: typeof File } {
  const ext = name.toLowerCase().split(".").pop() ?? ""
  if (["mp4", "mkv", "avi", "mov", "wmv", "webm", "m4v"].includes(ext)) return { label: "동영상", icon: FileVideo }
  if (["iso", "img", "vhd", "vhdx", "vmdk", "wim"].includes(ext)) return { label: "디스크 이미지", icon: Disc3 }
  if (["zip", "7z", "rar", "tar", "gz", "xz"].includes(ext)) return { label: "압축 파일", icon: FileArchive }
  if (["exe", "msi", "msix", "appx"].includes(ext)) return { label: "설치 파일", icon: Package }
  return { label: ext ? ext.toUpperCase() : "파일", icon: File }
}

function formatDate(seconds: number | null) {
  if (!seconds) return ""
  return new Date(seconds * 1000).toLocaleDateString("ko-KR", { year: "numeric", month: "short", day: "numeric" })
}

/** "C:\Users\me\Videos\a.mp4" → "C:\Users\me\Videos" */
function folderOf(path: string) {
  const i = path.lastIndexOf("\\")
  return i > 0 ? path.slice(0, i) : path
}

export function LargeFilesCard() {
  const { result, loading } = useSyncExternalStore(subscribe, () => state)
  const max = result?.items[0]?.bytes ?? 0
  const total = result?.items.reduce((s, f) => s + f.bytes, 0) ?? 0

  return (
    <section className="overflow-clip rounded-2xl border bg-card shadow-sm">
      <div className="flex flex-wrap items-center gap-3 border-b px-5 py-4">
        <div className="mr-auto min-w-0">
          <h2 className="font-semibold">큰 파일 찾기</h2>
          <p className="text-xs text-muted-foreground">
            200 MB가 넘는 파일을 큰 것부터 보여 드려요. 여기서는 지우지 않아요. 필요 없는 파일은 탐색기에서 직접
            지우세요.
          </p>
        </div>
        {loading ? (
          <Button variant="outline" size="sm" onClick={() => storageApi.cancelLargeFiles()}>
            <Square />
            멈추기
          </Button>
        ) : result ? (
          <IconTooltip label="다시 찾기">
            <Button variant="ghost" size="icon" className="size-8" onClick={find} aria-label="다시 찾기">
              <RefreshCw />
            </Button>
          </IconTooltip>
        ) : (
          <Button size="sm" onClick={find}>
            <ScanSearch />
            찾기
          </Button>
        )}
      </div>

      {loading ? (
        <div className="flex items-center justify-center gap-2 px-5 py-14 text-sm text-muted-foreground" role="status">
          <Loader2 className="size-4 animate-spin" />C 드라이브를 훑는 중이에요. 파일이 많으면 1~2분 걸릴 수 있어요.
        </div>
      ) : !result ? (
        <p className="px-5 py-10 text-center text-sm text-muted-foreground">
          "찾기"를 누르면 Windows 폴더를 뺀 C 드라이브 전체에서 큰 파일을 찾아요.
        </p>
      ) : result.items.length === 0 ? (
        <p className="px-5 py-10 text-center text-sm text-muted-foreground">200 MB가 넘는 파일이 없어요.</p>
      ) : (
        <>
          <p className="border-b bg-muted/30 px-5 py-2 text-xs text-muted-foreground">
            {result.items.length}개 · 모두 {formatBytes(total)}
          </p>
          <ul className="divide-y">
            {result.items.map((f) => {
              const kind = kindOf(f.name)
              const pct = max > 0 ? (f.bytes / max) * 100 : 0
              return (
                <li key={f.id} className="flex items-center gap-3 px-5 py-2">
                  <kind.icon className="size-4 shrink-0 text-muted-foreground" />
                  <div className="min-w-0 flex-1">
                    <p className="truncate text-sm" title={f.path}>
                      {f.name}
                    </p>
                    <p data-selectable className="truncate text-xs text-muted-foreground" title={f.path}>
                      {folderOf(f.path)}
                    </p>
                    <div className="mt-1 h-1.5 overflow-hidden rounded-full bg-muted">
                      <div className="h-full rounded-full bg-chart-2" style={{ width: `${pct}%` }} />
                    </div>
                  </div>
                  <span className="hidden w-24 shrink-0 text-right text-xs text-muted-foreground sm:block">
                    {kind.label}
                    <span className="block tabular-nums">{formatDate(f.modified)}</span>
                  </span>
                  <span className="w-20 shrink-0 text-right text-sm font-medium tabular-nums">
                    {formatBytes(f.bytes)}
                  </span>
                  <IconTooltip label="탐색기에서 보기">
                    <Button
                      variant="ghost"
                      size="icon"
                      className="size-7 text-muted-foreground"
                      onClick={() => storageApi.revealFolderItem(f.id).catch((e) => toast.error(String(e)))}
                      aria-label={`${f.name} 탐색기에서 보기`}
                    >
                      <FolderOpen />
                    </Button>
                  </IconTooltip>
                </li>
              )
            })}
          </ul>
        </>
      )}
    </section>
  )
}
