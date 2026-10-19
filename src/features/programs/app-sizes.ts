import { useSyncExternalStore } from "react"

import { api, type ScanProgress } from "@/lib/api"

/**
 * 크기 기록이 없는 프로그램의 설치 폴더 재기. 프로그램 화면과 저장공간 화면이 같은 쿼리(["app-sizes", …])로 쓰므로,
 * 진행 상황도 화면 밖(모듈)에 둬서 어느 화면에서든 같은 숫자를 보여 준다.
 */
let progress: ScanProgress | null = null
const listeners = new Set<() => void>()

function emit(next: ScanProgress | null) {
  progress = next
  for (const l of listeners) l()
}

function subscribe(listener: () => void) {
  listeners.add(listener)
  return () => listeners.delete(listener)
}

/** 쿼리 함수로 쓴다 */
export function measureAppSizes() {
  emit(null)
  return api.measureAppSizes(emit)
}

export const useAppSizeProgress = () => useSyncExternalStore(subscribe, () => progress)

/** "5/14" 같은 짧은 진행 표시 (전체를 모르면 빈 문자열) */
export const sizeProgressText = (p: ScanProgress | null) => (p && p.total > 0 ? ` ${p.done}/${p.total}` : "")
