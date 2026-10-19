import { REPO_URL } from "@/lib/site"
import { Reveal } from "@/components/motion"

/** 어디까지 만들었고 어디까지 확인했는지 있는 그대로 보여 준다. 바뀌면 이 목록부터 고친다. */
const groups = [
  {
    title: "만들었고, 테스트 앱으로 확인함",
    items: [
      "프로그램 목록(EXE·MSI·Store 앱)과 기본 언인스톨러 실행",
      "남은 흔적 찾기, 백업 후 정리, 복구 기록에서 되돌리기",
      "강제 제거, 시작프로그램, 저장공간 정리",
      "설치 추적(1차)",
      "서명을 확인하는 자동 업데이트",
    ],
  },
  {
    title: "확인하는 중",
    items: ["실제 프로그램을 대상으로 한 흔적 탐지 정확도", "여러 Windows 10·11 환경과 관리자 권한 경로"],
  },
  {
    title: "아직 없음",
    items: ["공개 설치 파일 (첫 릴리스 준비 중)", "코드 서명 인증서. 처음 실행할 때 SmartScreen 경고가 나올 수 있어요"],
  },
]

export function Status() {
  return (
    <section id="status" className="mt-20 bg-muted py-20 sm:mt-28 sm:py-24 lg:mt-32 lg:py-28">
      <div className="frame">
        <div className="grid gap-8 lg:grid-cols-12">
          <Reveal className="lg:col-span-6">
            <h2 className="section-title">어디까지 됐는지</h2>
          </Reveal>
          <Reveal delay={0.08} className="lg:col-span-6 lg:pt-3">
            <p className="max-w-[40ch] text-lead leading-snug">
              지금까지는 직접 만든 가짜 테스트 앱으로만 시험했어요. 실제 프로그램으로 하는 검증은 아직 진행 중이에요.
            </p>
          </Reveal>
        </div>

        <div className="mt-16 grid gap-12 md:grid-cols-3 md:gap-8">
          {groups.map((g, i) => (
            <Reveal key={g.title} delay={0.12 + i * 0.08}>
              <h3 className="text-lead font-semibold">{g.title}</h3>
              <ul className="mt-5 flex flex-col gap-3 text-body">
                {g.items.map((t) => (
                  <li key={t}>{t}</li>
                ))}
              </ul>
            </Reveal>
          ))}
        </div>

        <Reveal className="mt-16">
          <a
            href={`${REPO_URL}/commits/main`}
            target="_blank"
            rel="noreferrer"
            className="text-body font-medium text-primary underline-offset-4 hover:underline"
          >
            GitHub에서 변경 기록 보기
          </a>
        </Reveal>
      </div>
    </section>
  )
}
