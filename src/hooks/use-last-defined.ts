import { useRef } from "react"

/**
 * 값이 null이 돼도 마지막으로 있던 값을 돌려준다.
 * 대화상자·시트는 부모가 대상을 null로 바꿔 닫는데, 닫히는 애니메이션 동안에도 내용을 그대로 그리려고 쓴다.
 */
export function useLastDefined<T>(value: T | null | undefined): T | null {
  const last = useRef<T | null>(value ?? null)
  if (value != null) last.current = value
  return last.current
}
