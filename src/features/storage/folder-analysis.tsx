import { useState } from "react"
import { ArrowUp, File, Folder, FolderOpen, Loader2, Package, Radar, Square } from "lucide-react"
import { toast } from "sonner"

import { navigate, storageApi, type FolderAnalysis } from "@/lib/api"
import { formatBytes } from "@/lib/format"
import { cn } from "@/lib/utils"
import { IconTooltip } from "@/components/icon-tooltip"
import { Button } from "@/components/ui/button"

/** 큰 폴더 찾기. 크기를 보여 주고 탐색기로 열어 주기만 한다 (사용자 파일은 지우지 않는다). */
export function FolderAnalysisCard() {
  const [result, setResult] = useState<FolderAnalysis | null>(null)
  const [loading, setLoading] = useState<string | null>(null)

  const load = async (id?: string, label?: string) => {
    setLoading(label ?? "드라이브")
    try {
      const next = await storageApi.analyze(id)
      // 다른 분석을 시작해 멈춘 결과는 버린다
      if (!next.cancelled) setResult(next)
    } catch (e) {
      toast.error(String(e))
    } finally {
      setLoading(null)
    }
  }

  const max = result?.items[0]?.bytes ?? 0

  return (
    <section className="overflow-clip rounded-2xl border bg-card shadow-sm">
      <div className="flex flex-wrap items-center gap-3 border-b px-5 py-4">
        <div className="mr-auto min-w-0">
          <h2 className="font-semibold">큰 폴더 찾기</h2>
          <p className="text-xs text-muted-foreground">
            무엇이 공간을 차지하는지 보여 드려요. 여기서는 지우지 않아요. 필요 없는 프로그램은 프로그램 화면에서
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
          <p data-selectable className="min-w-0 flex-1 truncate font-mono text-xs">
            {result.path}
          </p>
        </div>
      )}

      {loading ? (
        <div className="flex items-center justify-center gap-2 px-5 py-14 text-sm text-muted-foreground">
          <Loader2 className="size-4 animate-spin" />
          {loading} 크기를 세는 중이에요. 파일이 많으면 1~2분 걸릴 수 있어요.
        </div>
      ) : !result ? (
        <p className="px-5 py-10 text-center text-sm text-muted-foreground">
          "분석 시작"을 누르면 C 드라이브부터 폴더별 크기를 세요.
        </p>
      ) : result.items.length === 0 ? (
        <p className="px-5 py-10 text-center text-sm text-muted-foreground">비어 있거나 열 수 없는 폴더예요.</p>
      ) : (
        <ul className="divide-y">
          {result.items.slice(0, 30).map((item) => {
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
                      <div className="h-full rounded-full bg-chart-2" style={{ width: `${pct}%` }} />
                    </div>
                  </div>
                  <span className="w-20 shrink-0 text-right text-sm font-medium tabular-nums">
                    {formatBytes(item.bytes)}
                  </span>
                  <span className="hidden w-24 shrink-0 text-right text-xs text-muted-foreground tabular-nums sm:block">
                    {item.isDir ? `파일 ${item.files.toLocaleString()}개` : ""}
                    {item.inaccessible > 0 && <span className="block">일부 열 수 없음</span>}
                  </span>
                </button>
                <IconTooltip label="탐색기에서 열기">
                  <Button
                    variant="ghost"
                    size="icon"
                    className="size-7 text-muted-foreground"
                    onClick={() => storageApi.revealFolderItem(item.id).catch((e) => toast.error(String(e)))}
                    aria-label={`${item.name} 탐색기에서 열기`}
                  >
                    <FolderOpen />
                  </Button>
                </IconTooltip>
              </li>
            )
          })}
        </ul>
      )}
    </section>
  )
}
