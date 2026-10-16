import { cn } from "@/lib/utils"

/** RobinUninstaller 로고: 로빈 알 색(청록) 바탕에 휴지통. src-tauri/app-icon.svg 와 같은 모양. */
export function Logo({ className }: { className?: string }) {
  return (
    <svg viewBox="0 0 64 64" className={cn("size-8", className)} aria-hidden>
      <defs>
        <linearGradient id="logo-bg" x1="0" y1="0" x2="1" y2="1">
          <stop offset="0" stopColor="#5eead4" />
          <stop offset="1" stopColor="#0891b2" />
        </linearGradient>
      </defs>
      <rect width="64" height="64" rx="16" fill="url(#logo-bg)" />
      <g fill="none" stroke="#042f2e" strokeWidth="4" strokeLinecap="round" strokeLinejoin="round">
        <path d="M18 22h28" />
        <path d="M27 22v-4h10v4" />
        <path d="M21 22l2 24a3 3 0 0 0 3 3h12a3 3 0 0 0 3-3l2-24" />
        <path d="M29 30v11M35 30v11" />
      </g>
    </svg>
  )
}
