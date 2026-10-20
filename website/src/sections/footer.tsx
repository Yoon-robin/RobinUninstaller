import { RELEASES_URL, REPO_URL } from "@/lib/site"

const links = [
  { href: REPO_URL, label: "소스" },
  { href: RELEASES_URL, label: "릴리스" },
  { href: `${REPO_URL}/blob/main/SECURITY.md`, label: "보안 문제 알리기" },
  { href: `${REPO_URL}/blob/main/LICENSE`, label: "Apache-2.0" },
  { href: `${REPO_URL}/issues`, label: "의견 남기기" },
]

/** 조용한 마무리. 받기·소스·보안 신고·라이선스를 한 줄에. */
export function Footer() {
  return (
    <footer className="frame grid gap-3 border-t py-8 text-fine sm:grid-cols-[1fr_auto] sm:items-baseline sm:gap-6">
      <p className="text-muted-foreground">RobinX · Windows 정리 도구 · 화면 속 프로그램과 숫자는 예시예요</p>
      <nav aria-label="바닥 링크" className="flex flex-wrap gap-x-6 gap-y-1">
        {links.map((l) => (
          <a
            key={l.label}
            href={l.href}
            target="_blank"
            rel="noreferrer"
            className="inline-flex min-h-11 items-center hover:underline hover:underline-offset-4 sm:min-h-0"
          >
            {l.label}
          </a>
        ))}
      </nav>
    </footer>
  )
}
