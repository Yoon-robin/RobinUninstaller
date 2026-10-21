import { useEffect } from "react"

/**
 * 지금 정리·되돌리기·제거처럼 중간에 끊기면 안 되는 작업이 몇 개 돌고 있는지.
 * 창을 닫으려 할 때(X, Alt+F4) 이 값을 보고 한 번 막는다 (App.tsx).
 */
let running = 0

export const isBusy = () => running > 0

/** `active`인 동안 작업 중으로 센다. */
export function useBusy(active: boolean) {
  useEffect(() => {
    if (!active) return
    running += 1
    return () => {
      running -= 1
    }
  }, [active])
}
