import { REPO_URL } from "@/lib/site"

/** 조용한 한 줄. 위에서 이미 할 말을 다 했으니 마무리만 한다. */
export function Footer() {
  return (
    <footer className="frame mt-28 grid gap-2 pb-8 text-body sm:mt-36 sm:grid-cols-[1fr_auto_1fr] sm:items-baseline sm:gap-6">
      <p className="text-muted-foreground">RobinX · Windows 정리 도구</p>
      <nav aria-label="바닥 링크" className="flex flex-wrap gap-x-6 gap-y-1">
        <a href={REPO_URL} target="_blank" rel="noreferrer" className="hover:underline hover:underline-offset-4">
          GitHub
        </a>
        <a
          href={`${REPO_URL}/issues`}
          target="_blank"
          rel="noreferrer"
          className="hover:underline hover:underline-offset-4"
        >
          의견 남기기
        </a>
        <a
          href={`${REPO_URL}/blob/main/LICENSE`}
          target="_blank"
          rel="noreferrer"
          className="hover:underline hover:underline-offset-4"
        >
          Apache-2.0
        </a>
      </nav>
      <p className="text-muted-foreground sm:text-right">화면 속 프로그램과 숫자는 설명을 위한 예시예요</p>
    </footer>
  )
}
