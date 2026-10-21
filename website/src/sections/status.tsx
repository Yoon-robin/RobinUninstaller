import { RELEASE, REPO_URL } from "@/lib/site"
import { Reveal } from "@/components/motion"

/**
 * 어디까지 만들었고 어디까지 확인했는지 있는 그대로. 버전과 날짜는 빌드할 때 GitHub 릴리스에서 가져온다
 * (scripts/release-info.mjs). 목록이 바뀌면 여기부터 고친다.
 */
const blocks = [
  {
    title: "테스트 앱으로 확인함",
    items: [
      "설치된 프로그램 목록과 제거",
      "남은 흔적 찾기, 백업 후 정리, 복구 기록에서 되돌리기",
      "강제 제거, 시작프로그램, 저장공간 정리",
      "앱 안에서 자동 업데이트 (서명한 파일만 설치)",
    ],
  },
  {
    title: "확인하는 중",
    items: ["실제 프로그램에서 남은 흔적을 얼마나 정확히 찾는지", "여러 PC 환경(Windows 10·11)에서의 동작"],
  },
  {
    title: "알아 둘 것",
    items: [
      "Microsoft 게시자 서명(코드 서명 인증서)이 아직 없어요. 처음 실행할 때 Windows의 경고 창이 뜰 수 있어요.",
      "아직 미리보기예요. 정리하기 전에 목록을 한 번 훑어보세요.",
    ],
  },
]

const released = RELEASE.publishedAt.toLocaleDateString("ko-KR", { year: "numeric", month: "long", day: "numeric" })

export function Status() {
  return (
    <section id="status" className="border-y bg-white/[0.02] py-20 sm:py-24 lg:py-28">
      <div className="frame">
        <div className="grid gap-10 lg:grid-cols-12 lg:gap-8">
          <Reveal heading outerClassName="lg:col-span-5">
            <h2 className="section-title">어디까지 됐는지</h2>
            <p className="mt-5 max-w-[34ch] text-lead leading-snug text-muted-foreground">
              지금까지는 직접 만든 테스트 앱으로 시험했어요. 실제 프로그램으로 하는 검증은 진행 중이에요.
            </p>
          </Reveal>

          {/* 최신 릴리스: 흰 판 하나에 버전·날짜·확인 방법 */}
          <Reveal delay={0.06} className="window rounded-[1.75rem] p-7 sm:p-8 lg:col-span-7">
            <p className="text-section leading-none font-bold tracking-tight tabular-nums">v{RELEASE.version}</p>
            <p className="mt-3 text-body text-muted-foreground">
              최신 미리보기 · {released} 공개 · {RELEASE.file} · {RELEASE.sizeMb} MB
            </p>
            {RELEASE.sha256 && (
              <p className="mt-4 text-fine text-muted-foreground">
                SHA-256 <code className="break-all text-foreground">{RELEASE.sha256}</code>
              </p>
            )}
            <div className="mt-6 flex flex-wrap gap-x-6 gap-y-2 text-body font-medium">
              <a
                href={RELEASE.pageUrl}
                target="_blank"
                rel="noreferrer"
                className="text-stage underline-offset-4 hover:underline"
              >
                릴리스 노트
              </a>
              <a
                href={`${REPO_URL}/commits/main`}
                target="_blank"
                rel="noreferrer"
                className="text-stage underline-offset-4 hover:underline"
              >
                변경 기록
              </a>
            </div>
          </Reveal>
        </div>

        <div className="mt-14 grid gap-10 md:grid-cols-3 md:gap-8">
          {blocks.map((g, i) => (
            <Reveal key={g.title} delay={0.04 * i}>
              <h3 className="text-lead font-semibold">{g.title}</h3>
              <ul className="mt-4 flex list-disc flex-col gap-2.5 pl-5 text-body marker:text-muted-foreground">
                {g.items.map((t) => (
                  <li key={t}>{t}</li>
                ))}
              </ul>
            </Reveal>
          ))}
        </div>
      </div>
    </section>
  )
}
