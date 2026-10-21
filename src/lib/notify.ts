import { useEffect, useState } from "react"
import { toast } from "sonner"

/**
 * 실패 알림. 제목에 무엇을 하다 실패했는지(작업과 대상), 설명에 원인을 둔다.
 * `persist`면 사용자가 닫을 때까지 남긴다 (되돌릴 수 없는 작업이 실패했을 때).
 */
export function notifyError(title: string, error: unknown, opts?: { persist?: boolean }) {
  toast.error(title, {
    description: String(error),
    duration: opts?.persist ? Infinity : 8000,
    closeButton: opts?.persist,
  })
}

/** `promise.catch(failed("탐색기를 열지 못했어요"))`처럼 쓴다 */
export const failed = (title: string, opts?: { persist?: boolean }) => (error: unknown) =>
  notifyError(title, error, opts)

/** 복사: 성공하면 잠깐 `copied`가 true(체크 표시), 실패하면 알림 */
export function useCopy() {
  const [copied, setCopied] = useState(false)
  useEffect(() => {
    if (!copied) return
    const t = setTimeout(() => setCopied(false), 1200)
    return () => clearTimeout(t)
  }, [copied])
  const copy = (text: string) =>
    navigator.clipboard.writeText(text).then(() => setCopied(true), failed("복사하지 못했어요"))
  return { copied, copy }
}
