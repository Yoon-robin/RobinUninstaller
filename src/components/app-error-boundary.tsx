import { Component, type ErrorInfo, type ReactNode } from "react"
import { getCurrentWindow } from "@tauri-apps/api/window"
import { Copy, RefreshCw, TriangleAlert, X } from "lucide-react"

import { Button } from "@/components/ui/button"

/**
 * 화면을 그리다 오류가 나면 창 전체가 빈 화면이 되지 않게, 무슨 일이 있었는지와 다시 불러오기를 보여 준다.
 * 진행 중이던 작업(정리·되돌리기)은 엔진에서 계속되고 기록에 남으므로, 다시 불러온 뒤 복구 기록에서 확인할 수 있다.
 */
export class AppErrorBoundary extends Component<{ children: ReactNode }, { error: Error | null }> {
  state = { error: null as Error | null }

  static getDerivedStateFromError(error: Error) {
    return { error }
  }

  componentDidCatch(error: Error, info: ErrorInfo) {
    console.error("화면 오류", error, info.componentStack)
  }

  render() {
    const { error } = this.state
    if (!error) return this.props.children
    const detail = `${error.name}: ${error.message}\n${error.stack ?? ""}`
    return (
      // 창 틀을 앱이 직접 그리므로, 이 화면에서도 끌어서 옮기고 닫을 수 있게 한다
      <div
        data-tauri-drag-region
        role="alert"
        className="flex h-svh flex-col items-center justify-center gap-4 bg-background p-8 text-center text-foreground"
      >
        <span className="flex size-12 items-center justify-center rounded-2xl bg-destructive/10 text-destructive">
          <TriangleAlert className="size-6" />
        </span>
        <div className="max-w-md">
          <p className="text-base font-semibold">화면을 그리다 문제가 생겼어요</p>
          <p className="mt-2 text-sm text-muted-foreground">
            다시 불러오면 대부분 풀려요. 하던 정리나 되돌리기는 복구 기록에서 확인할 수 있어요. 계속되면 아래 내용을
            복사해 GitHub 이슈에 남겨 주세요.
          </p>
        </div>
        <p
          data-selectable
          className="max-h-32 max-w-lg overflow-y-auto font-mono text-xs break-all whitespace-pre-wrap text-muted-foreground"
        >
          {error.name}: {error.message}
        </p>
        <div className="flex gap-2">
          <Button
            variant="outline"
            onClick={() =>
              getCurrentWindow()
                .close()
                .catch(() => {})
            }
          >
            <X />
            닫기
          </Button>
          <Button variant="outline" onClick={() => navigator.clipboard.writeText(detail).catch(() => {})}>
            <Copy />
            오류 복사
          </Button>
          <Button onClick={() => location.reload()}>
            <RefreshCw />
            다시 불러오기
          </Button>
        </div>
      </div>
    )
  }
}
