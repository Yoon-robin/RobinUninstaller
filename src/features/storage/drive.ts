import { useQuery } from "@tanstack/react-query"

import { storageApi } from "@/lib/api"
import { liveStore } from "@/lib/live-store"

/**
 * 저장공간 화면에서 고른 드라이브 문자 ("D"). null이면 시스템 드라이브.
 * 디스크 카드·큰 폴더 찾기·큰 파일 찾기가 함께 따른다. 캐시·임시 파일 정리 규칙은 드라이브와 상관없다.
 */
export const selectedDrive = liveStore<string | null>(null)

/** "D" → "D 드라이브", null → "시스템 드라이브"(이름을 아직 모를 때) */
export const driveName = (letter: string | null, systemLetter?: string) => `${letter ?? systemLetter ?? "C"} 드라이브`

export const DRIVES_KEY = ["drives"]

export function useDrives() {
  return useQuery({ queryKey: DRIVES_KEY, queryFn: storageApi.drives, staleTime: 60_000 })
}

/** 시스템 드라이브 문자 (목록을 아직 못 읽었으면 undefined) */
export function useSystemLetter() {
  return useDrives().data?.find((d) => d.system)?.letter
}
