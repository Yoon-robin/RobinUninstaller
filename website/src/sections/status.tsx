import { CircleCheck, CircleDashed, CircleDot } from "lucide-react"

import { REPO_URL } from "@/lib/site"
import { Reveal, SectionHeading } from "@/components/motion"

/** 어디까지 만들었고 어디까지 확인했는지 있는 그대로 보여 준다. 바뀌면 이 표부터 고친다. */
const groups = [
  {
    icon: CircleCheck,
    tone: "text-chart-5",
    title: "만들었고, 테스트 앱으로 확인함",
    items: [
      "프로그램 목록(EXE·MSI·Store 앱)과 기본 언인스톨러 실행",
      "남은 흔적 찾기 → 백업 후 정리 → 복구 기록에서 되돌리기",
      "강제 제거, 시작프로그램 끄기·켜기, 저장공간 정리",
      "되돌릴 수 있는 최적화, 설치 추적(1차)",
      "서명을 확인하는 자동 업데이트",
    ],
  },
  {
    icon: CircleDot,
    tone: "text-chart-1",
    title: "확인하는 중",
    items: ["실제 프로그램을 대상으로 한 흔적 탐지 정확도", "여러 Windows 10·11 환경과 관리자 권한 경로"],
  },
  {
    icon: CircleDashed,
    tone: "text-muted-foreground",
    title: "아직 없음",
    items: [
      "공개 설치 파일 (첫 릴리스 준비 중)",
      "코드 서명 인증서 — 처음 실행할 때 Windows SmartScreen 경고가 나올 수 있어요",
    ],
  },
]

export function Status() {
  return (
    <section id="status" className="relative mx-auto max-w-6xl px-6 pt-32">
      <SectionHeading
        eyebrow="개발 현황"
        title="어디까지 됐는지 그대로 알려 드려요"
        description="지우는 도구라서, 확인하지 않은 것을 됐다고 말하지 않아요. 실제 프로그램을 지우는 시험은 하지 않고, 이 일을 위해 만든 가짜 테스트 앱으로만 확인해요."
      />
      <div className="mt-14 grid gap-4 md:grid-cols-3">
        {groups.map((g, i) => (
          <Reveal key={g.title} delay={i * 0.08} className="h-full">
            <div className="h-full rounded-3xl border border-white/8 bg-card p-6 sm:p-7">
              <h3 className="flex items-center gap-2 font-semibold">
                <g.icon className={`size-5 ${g.tone}`} />
                {g.title}
              </h3>
              <ul className="mt-4 space-y-2.5 text-[0.95rem] leading-relaxed text-muted-foreground">
                {g.items.map((t) => (
                  <li key={t} className="flex gap-2">
                    <span aria-hidden className="mt-2.5 size-1 shrink-0 rounded-full bg-current" />
                    {t}
                  </li>
                ))}
              </ul>
            </div>
          </Reveal>
        ))}
      </div>
      <Reveal className="mt-6 text-center text-sm text-muted-foreground">
        자세한 변경 내용은{" "}
        <a
          href={`${REPO_URL}/commits/main`}
          target="_blank"
          rel="noreferrer"
          className="text-primary underline-offset-4 hover:underline"
        >
          GitHub 커밋 기록
        </a>
        에서 볼 수 있어요.
      </Reveal>
    </section>
  )
}
