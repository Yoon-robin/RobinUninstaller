export function formatSize(kb: number | null): string {
  if (kb == null || kb <= 0) return "—"
  if (kb < 1024) return `${kb} KB`
  const mb = kb / 1024
  if (mb < 1024) return `${mb.toFixed(mb < 10 ? 1 : 0)} MB`
  return `${(mb / 1024).toFixed(1)} GB`
}

/**
 * 레지스트리 InstallDate를 YYYY-MM-DD로 맞춘다.
 * 대부분 YYYYMMDD지만 일부 설치 프로그램은 M/D/YYYY로 기록한다. 둘 다 아니면 그대로 둔다.
 */
export function formatInstallDate(raw: string | null): string {
  if (!raw) return "—"
  const ymd = /^(\d{4})(\d{2})(\d{2})$/.exec(raw)
  if (ymd) return `${ymd[1]}-${ymd[2]}-${ymd[3]}`
  const mdy = /^(\d{1,2})\/(\d{1,2})\/(\d{4})$/.exec(raw)
  if (mdy) return `${mdy[3]}-${mdy[1].padStart(2, "0")}-${mdy[2].padStart(2, "0")}`
  return raw
}

/** 설치일 정렬용 키. 알 수 없는 형식은 빈 문자열(맨 뒤). */
export function installDateSortKey(raw: string | null): string {
  const formatted = formatInstallDate(raw)
  return /^\d{4}-\d{2}-\d{2}$/.test(formatted) ? formatted : ""
}
