import { Download } from "lucide-react"

import { RELEASE } from "@/lib/site"
import { cn } from "@/lib/utils"

/**
 * 설치 파일을 바로 받는 단추. 문구는 누르면 일어나는 일(파일을 받음)과 같아야 한다.
 * `tone`: 흰 바탕은 강조색, 어두운 바탕(마지막 구역)은 밝은 단추.
 */
export function DownloadButton({ tone = "primary", className }: { tone?: "primary" | "night"; className?: string }) {
  return (
    <a
      href={RELEASE.downloadUrl}
      className={cn(
        "inline-flex h-12 items-center gap-2.5 rounded-full px-7 text-body font-semibold transition-[transform,background-color,opacity] focus-visible:ring-2 focus-visible:ring-offset-2 focus-visible:outline-none active:scale-[0.97]",
        tone === "primary"
          ? "bg-primary text-primary-foreground hover:bg-primary/90 focus-visible:ring-ring"
          : "bg-night-foreground text-night hover:opacity-90 focus-visible:ring-night-foreground focus-visible:ring-offset-night",
        className,
      )}
    >
      <Download className="size-[1.1rem]" />
      Windows용 무료 다운로드
    </a>
  )
}

/** 받기 전에 알아야 할 것: 버전, 크기, 지원 환경, 미리보기·서명 상태. 릴리스 페이지에서 SHA-256을 확인할 수 있다. */
export function DownloadFacts({ className, linkClassName }: { className?: string; linkClassName?: string }) {
  return (
    <p className={cn("text-fine", className)}>
      <span className="block">
        v{RELEASE.version} · {RELEASE.sizeMb} MB · Windows 10/11 (x64)
      </span>
      <span className="block">
        미리보기 · 게시자 서명 전 {/* 좁은 화면에서 "·"만 줄 앞에 떨어지지 않게 링크와 묶는다 */}
        <span className="whitespace-nowrap">
          ·{" "}
          <a
            href={RELEASE.pageUrl}
            target="_blank"
            rel="noreferrer"
            className={cn("font-medium underline underline-offset-4", linkClassName)}
          >
            릴리스 노트와 SHA-256
          </a>
        </span>
      </span>
    </p>
  )
}
