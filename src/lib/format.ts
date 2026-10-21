const SIZE_UNITS = ["KB", "MB", "GB", "TB"] as const

/** 단위 안에서의 자릿수: KB는 정수, MB는 10 미만만 소수 한 자리, GB·TB는 소수 한 자리 */
const sizeDigits = (value: number, unit: number) => (unit === 0 ? 0 : unit === 1 && value >= 10 ? 0 : 1)

/** KB 단위 크기를 읽기 좋게. 반올림한 값이 1024가 되면 다음 단위로 올린다 ("1024 MB"가 아니라 "1.0 GB"). */
export function formatSize(kb: number | null): string {
  if (kb == null || kb <= 0) return "—"
  let value = kb
  let unit = 0
  while (unit < SIZE_UNITS.length - 1 && Number(value.toFixed(sizeDigits(value, unit))) >= 1024) {
    value /= 1024
    unit++
  }
  return `${value.toFixed(sizeDigits(value, unit))} ${SIZE_UNITS[unit]}`
}

/** 개수·파일 수처럼 세는 수 (1,234) */
export const formatCount = (n: number) => n.toLocaleString("ko-KR")

/** 걸린 시간 ("83초"가 아니라 "1분 23초") */
export function formatDuration(ms: number): string {
  const s = Math.floor(ms / 1000)
  if (s < 60) return `${s}초`
  const m = Math.floor(s / 60)
  if (m < 60) return s % 60 ? `${m}분 ${s % 60}초` : `${m}분`
  return m % 60 ? `${Math.floor(m / 60)}시간 ${m % 60}분` : `${Math.floor(m / 60)}시간`
}

/** 날짜와 시각 (자세히 보기·기록에 쓰는 한 가지 형식: "2026년 10월 21일 오후 01:05") */
export function formatDateTime(date: Date | number): string {
  return new Date(date).toLocaleString("ko-KR", {
    year: "numeric",
    month: "long",
    day: "numeric",
    hour: "2-digit",
    minute: "2-digit",
  })
}

/** 날짜만 ("2026년 10월 21일") */
export function formatDay(date: Date | number): string {
  return new Date(date).toLocaleDateString("ko-KR", { year: "numeric", month: "long", day: "numeric" })
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

export function formatBytes(bytes: number | null): string {
  if (bytes == null) return "—"
  if (bytes === 0) return "0 KB"
  if (bytes < 1024) return "1 KB 미만"
  return formatSize(Math.round(bytes / 1024))
}
