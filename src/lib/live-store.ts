import { useSyncExternalStore } from "react"

/**
 * 화면 밖(모듈)에 두는 작은 상태. 진행 중인 작업의 상태를 여기 두면 다른 화면에 다녀와도
 * 컴포넌트가 새로 그려지면서 그대로 이어 보인다 (컴포넌트 상태는 화면을 떠나면 사라진다).
 */
export function liveStore<T>(initial: T) {
  let state = initial
  const listeners = new Set<() => void>()
  const get = () => state
  const set = (next: T | ((prev: T) => T)) => {
    state = typeof next === "function" ? (next as (prev: T) => T)(state) : next
    for (const l of listeners) l()
  }
  const subscribe = (l: () => void) => {
    listeners.add(l)
    return () => listeners.delete(l)
  }
  const use = () => useSyncExternalStore(subscribe, get)
  return { get, set, use }
}
