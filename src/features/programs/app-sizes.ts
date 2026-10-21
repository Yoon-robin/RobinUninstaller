import { useSyncExternalStore } from "react"

import { api, type InstalledApplication, type ScanProgress } from "@/lib/api"

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

/**
 * 크기 쿼리 키. 설치 목록의 ID·설치 위치가 같으면 같은 키라 다시 재지 않는다
 * (예전에는 목록을 다시 읽은 시각을 키로 써서, 같은 목록을 다시 읽을 때마다 모두 다시 쟀다).
 */
export function appSizesKey(apps: InstalledApplication[] | undefined) {
  let hash = 5381
  for (const a of apps ?? []) {
    const s = a.id + "|" + (a.installLocation ?? "")
    for (let i = 0; i < s.length; i++) hash = ((hash << 5) + hash + s.charCodeAt(i)) | 0
  }
  return ["app-sizes", apps?.length ?? 0, hash] as const
}

/** 쿼리 함수로 쓴다 */
export function measureAppSizes() {
  emit(null)
  return api.measureAppSizes(emit)
}

export const useAppSizeProgress = () => useSyncExternalStore(subscribe, () => progress)

/** "5/14" 같은 짧은 진행 표시 (전체를 모르면 빈 문자열) */
export const sizeProgressText = (p: ScanProgress | null) => (p && p.total > 0 ? ` ${p.done} / ${p.total}` : "")
