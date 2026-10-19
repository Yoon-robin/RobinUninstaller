import { useQuery } from "@tanstack/react-query"
import { EyeOff, Loader2, ShieldAlert } from "lucide-react"

import { startupApi } from "@/lib/api"
import { Button } from "@/components/ui/button"

/**
 * 일반 사용자에게 안 보이는 예약 작업(SYSTEM 등 다른 계정으로 도는 것). 관리자 권한(UAC)으로 읽어 보여 주기만 한다.
 * 켜고 끄기는 아직 없다 (LOOP F1 "읽기 전용부터"). 결과는 쿼리 캐시에 남아 다른 탭에 다녀와도 그대로다.
 */
export function HiddenTasksCard() {
  // 누를 때만 UAC를 띄운다
  const hidden = useQuery({
    queryKey: ["startup-hidden-tasks"],
    queryFn: startupApi.hiddenTasks,
    enabled: false,
    staleTime: Infinity,
    retry: false,
  })
  const tasks = hidden.data

  return (
    <section className="overflow-clip rounded-2xl border bg-card shadow-sm">
      <div className="flex flex-wrap items-center gap-3 px-5 py-4">
        <div className="mr-auto min-w-0">
          <h2 className="flex items-center gap-2 font-semibold">
            <EyeOff className="size-4 text-muted-foreground" />
            숨은 예약 작업
          </h2>
          <p className="text-xs text-muted-foreground">
            SYSTEM처럼 다른 계정으로 도는 예약 작업은 관리자 권한이 있어야 보여요. 여기서는 읽기만 하고 켜거나 끄지
            않아요.
          </p>
        </div>
        <Button variant="outline" size="sm" onClick={() => void hidden.refetch()} disabled={hidden.isFetching}>
          {hidden.isFetching ? <Loader2 className="animate-spin" /> : <ShieldAlert className="text-chart-3" />}
          {tasks ? "다시 읽기" : "관리자 권한으로 읽기"}
        </Button>
      </div>

      {hidden.isFetching && (
        <p className="border-t px-5 py-3 text-xs text-muted-foreground" role="status">
          UAC 창에서 "예"를 누르면 따로 실행된 RobinX가 작업 목록을 읽어 와요.
        </p>
      )}
      {hidden.isError && !hidden.isFetching && (
        <p className="border-t px-5 py-3 text-sm text-destructive">{String(hidden.error)}</p>
      )}
      {tasks && !hidden.isFetching && (
        <div className="border-t">
          {tasks.length === 0 ? (
            <p className="px-5 py-6 text-center text-sm text-muted-foreground">
              관리자 권한으로만 보이는 예약 작업이 없어요. 로그인·부팅 때 실행되는 작업은 위 목록에 모두 있어요.
            </p>
          ) : (
            <ul className="divide-y">
              {tasks.map((t) => (
                <li key={t.id} className="flex items-start gap-3 px-5 py-3 text-sm">
                  <div className="min-w-0 flex-1">
                    <p className="truncate font-medium">{t.name}</p>
                    <p data-selectable className="truncate font-mono text-xs text-muted-foreground" title={t.command}>
                      {t.command || "프로그램 실행 동작 없음"}
                    </p>
                    <p className="truncate text-xs text-muted-foreground">
                      {t.approvedName}
                      {t.author && ` · 만든 이 ${t.author}`}
                      {t.targetMissing && <span className="text-chart-3"> · 실행 파일이 없어요</span>}
                    </p>
                  </div>
                  <span className="shrink-0 text-xs text-muted-foreground">{t.enabled ? "켜짐" : "꺼짐"}</span>
                </li>
              ))}
            </ul>
          )}
        </div>
      )}
    </section>
  )
}
