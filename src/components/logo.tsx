import { useId } from "react"

import { cn } from "@/lib/utils"

/**
 * RobinX 로고: 어두운 타일 위에 R과 X를 합친 마크(청록 → 보라). src-tauri/app-icon.svg 와 같은 모양.
 * R의 윗부분과 고리, X의 두 획이 R의 다리가 된다.
 */
export function Logo({ className }: { className?: string }) {
  // 한 화면에 여러 개가 있어도 그라데이션 ID가 겹치지 않게
  const id = useId().replace(/:/g, "")
  return (
    <svg viewBox="0 0 64 64" className={cn("size-8", className)} aria-hidden>
      <defs>
        <linearGradient id={`${id}-tile`} x1="0" y1="0" x2="0" y2="1">
          <stop offset="0" stopColor="#1b2030" />
          <stop offset="1" stopColor="#0d1018" />
        </linearGradient>
        <linearGradient id={`${id}-rim`} x1="0" y1="0" x2="1" y2="1">
          <stop offset="0" stopColor="#2ee6f0" />
          <stop offset="1" stopColor="#8b5cf6" />
        </linearGradient>
        <linearGradient id={`${id}-ink`} x1="0.1" y1="0" x2="0.9" y2="1">
          <stop offset="0" stopColor="#3cf0ef" />
          <stop offset="0.5" stopColor="#3aa0f5" />
          <stop offset="1" stopColor="#9b5cf6" />
        </linearGradient>
        <linearGradient id={`${id}-leg`} x1="0" y1="0" x2="1" y2="1">
          <stop offset="0" stopColor="#36c8f2" />
          <stop offset="1" stopColor="#a35cf7" />
        </linearGradient>
      </defs>
      <rect x="1" y="1" width="62" height="62" rx="15" fill={`url(#${id}-tile)`} />
      <rect
        x="1.5"
        y="1.5"
        width="61"
        height="61"
        rx="14.5"
        fill="none"
        stroke={`url(#${id}-rim)`}
        strokeOpacity="0.85"
      />
      <path
        fill={`url(#${id}-ink)`}
        d="M13 14 H40 C47.7 14 52.5 18.8 52.5 25.5 C52.5 31.6 48.4 36 42.6 36.6 L37.6 30.8 C40.8 30.4 42.6 28.6 42.6 25.9 C42.6 23.2 40.9 21.6 38.3 21.6 H19.8 Z"
      />
      <path fill={`url(#${id}-leg)`} d="M17.5 28.5 H30 L51.5 50.5 H39 Z" />
      <path fill={`url(#${id}-ink)`} d="M23.5 37.5 L29.6 43.4 L24 50.5 H11.5 Z" />
    </svg>
  )
}
