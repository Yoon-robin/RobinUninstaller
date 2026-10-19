import { ArchiveRestore, FileLock2, ShieldAlert, Recycle, Check } from "lucide-react"

import { REPO_URL } from "@/lib/site"
import { Reveal, SectionHeading } from "@/components/motion"
import { Button } from "@/components/ui/button"
import { GithubIcon } from "@/components/github-icon"
import { LOGO } from "@/lib/site"
import { Accordion, AccordionContent, AccordionItem, AccordionTrigger } from "@/components/ui/accordion"

const principles = [
  {
    icon: ArchiveRestore,
    title: "남은 흔적은 백업 후 정리",
    text: "파일은 격리 폴더로, 레지스트리는 내보내기 파일로 보관한 뒤 치워요. 백업은 복구 기록에서 직접 지우거나 보관 기간이 지나야 사라져요.",
  },
  {
    icon: Recycle,
    title: "캐시는 다시 만들어지는 것만",
    text: "저장공간 정리는 지워도 프로그램이 다시 만드는 캐시·임시 파일만 대상으로, 확인을 받은 뒤 진행해요.",
  },
  {
    icon: ShieldAlert,
    title: "되돌릴 수 없으면 따로 경고",
    text: "휴지통 비우기처럼 되돌릴 수 없는 항목은 눈에 띄게 표시하고 기본으로 고르지 않아요.",
  },
  {
    icon: FileLock2,
    title: "내 파일은 자동으로 지우지 않아요",
    text: "문서·사진·다운로드는 정리 대상이 아니에요. 내 문서가 섞인 폴더는 검토 필요로 분류해요.",
  },
]

export function Safety() {
  return (
    <section id="safety" className="relative mx-auto max-w-6xl px-6 pt-32">
      <SectionHeading
        eyebrow="안전 원칙"
        title={
          <>
            지우는 프로그램일수록
            <br className="hidden sm:block" /> 더 조심스러워야 하니까
          </>
        }
        description="RobinX는 처음부터 이 네 가지 원칙을 지키도록 만들었어요."
      />
      <div className="mt-14 grid gap-4 sm:grid-cols-2">
        {principles.map((p, i) => (
          <Reveal key={p.title} delay={(i % 2) * 0.08}>
            <div className="flex h-full gap-5 rounded-3xl border border-white/8 bg-card p-6 sm:p-7">
              <span className="flex size-12 shrink-0 items-center justify-center rounded-2xl bg-chart-5/12 text-chart-5">
                <p.icon className="size-6" />
              </span>
              <div>
                <h3 className="flex items-center gap-2 text-lg font-semibold">
                  {p.title}
                  <Check className="size-4 text-chart-5" />
                </h3>
                <p className="mt-2 text-[0.95rem] leading-relaxed text-pretty text-muted-foreground">{p.text}</p>
              </div>
            </div>
          </Reveal>
        ))}
      </div>

      {/* 무엇을 되돌릴 수 있는지: 정리 종류마다 다르므로 한 표로 분명히 보여 준다 */}
      <Reveal className="mt-10">
        <div className="overflow-hidden rounded-3xl border border-white/8 bg-card">
          <div className="border-b border-white/8 px-6 py-4">
            <h3 className="font-semibold">무엇을 되돌릴 수 있나요?</h3>
            <p className="mt-1 text-sm text-muted-foreground">정리하는 종류마다 달라요. 지우기 전에 확인하세요.</p>
          </div>
          {/* 넓은 화면: 표 */}
          <table className="hidden w-full text-left text-sm sm:table">
            <caption className="sr-only">작업별 되돌리기 가능 여부</caption>
            <thead className="text-xs text-muted-foreground">
              <tr>
                <th scope="col" className="px-6 py-3 font-medium">
                  작업
                </th>
                <th scope="col" className="px-6 py-3 font-medium">
                  되돌리기
                </th>
              </tr>
            </thead>
            <tbody className="divide-y divide-white/6">
              {restoreScope.map((r) => (
                <tr key={r.task}>
                  <th scope="row" className="px-6 py-3.5 font-medium">
                    {r.task}
                  </th>
                  <td className="px-6 py-3.5">
                    <RestoreLevel level={r.level} />
                    <span className="text-muted-foreground"> · {r.note}</span>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
          {/* 좁은 화면: 가로로 넘치지 않게 작업마다 한 줄씩 */}
          <ul className="divide-y divide-white/6 sm:hidden">
            {restoreScope.map((r) => (
              <li key={r.task} className="px-5 py-4 text-sm">
                <p className="flex items-start justify-between gap-3 font-medium">
                  {r.task}
                  <RestoreLevel level={r.level} />
                </p>
                <p className="mt-1 text-muted-foreground">{r.note}</p>
              </li>
            ))}
          </ul>
          <p className="border-t border-white/8 px-6 py-3 text-xs text-muted-foreground">
            조건부: 백업을 보관하는 동안, 그리고 원래 자리에 다른 파일이나 값이 새로 생기지 않았을 때만 되돌려요. 겹치면
            덮어쓰지 않고 알려 줘요.
          </p>
        </div>
      </Reveal>
    </section>
  )
}

type Level = "yes" | "conditional" | "no"

const levelLabel: Record<Level, { text: string; className: string }> = {
  yes: { text: "가능", className: "text-chart-5" },
  conditional: { text: "조건부", className: "text-chart-1" },
  no: { text: "불가", className: "text-chart-3" },
}

function RestoreLevel({ level }: { level: Level }) {
  const l = levelLabel[level]
  return <span className={`shrink-0 font-medium ${l.className}`}>{l.text}</span>
}

const restoreScope: { task: string; level: Level; note: string }[] = [
  { task: "시작프로그램 끄기", level: "yes", note: "등록 정보는 그대로 두고 끄기만 해서, 다시 켜면 원래 상태로" },
  { task: "남은 흔적(파일·레지스트리) 정리", level: "conditional", note: "백업을 보관하는 동안 복구 기록에서 복원" },
  { task: "강제 제거", level: "conditional", note: "백업한 흔적만 복원돼요. 프로그램이 다시 실행되는 건 아니에요" },
  {
    task: "최적화 설정 바꾸기",
    level: "conditional",
    note: "끄면 기록해 둔 원래 값으로. 그 사이 다른 곳에서 바꾼 값은 덮어쓰지 않아요",
  },
  { task: "프로그램 제거(원래 언인스톨러)", level: "no", note: "프로그램 자체는 다시 설치해야 해요" },
  { task: "캐시·임시 파일 정리", level: "no", note: "다시 만들어지는 것만 지우지만, 지운 파일은 돌아오지 않아요" },
  { task: "백업 삭제·보관 기간 만료", level: "no", note: "그 백업으로는 더 이상 복원할 수 없어요" },
]

const faqs = [
  {
    q: "소스 코드를 볼 수 있나요?",
    a: "네, GitHub에 모두 공개되어 있고 Apache License 2.0으로 쓰고 고치고 배포할 수 있어요. 무엇을 어떻게 지우는지 코드로 직접 확인할 수 있어요.",
  },
  {
    q: "지운 건 다 되돌릴 수 있나요?",
    a: "남은 흔적 정리와 강제 제거는 항상 백업·격리를 거쳐서, 백업을 보관하는 동안 되돌릴 수 있어요. 저장공간 정리의 캐시·임시 파일은 다시 만들어지는 것만 확인을 받고 지우며, 지운 뒤에는 되돌릴 수 없어요. 휴지통 비우기처럼 되돌릴 수 없는 일은 따로 경고하고 미리 고르지 않아요. 위의 표를 참고하세요.",
  },
  {
    q: "관리자 권한이 필요한가요?",
    a: "평소에는 일반 권한으로 실행돼요. 모든 사용자용으로 설치된 프로그램의 흔적처럼 관리자 권한이 필요한 항목을 정리할 때만 Windows 확인 창(UAC)을 띄워요.",
  },
  {
    q: "어떤 Windows에서 쓸 수 있나요?",
    a: "Windows 10과 11 (64비트)을 지원해요. Windows 11에서는 Mica 배경 효과도 적용돼요.",
  },
  {
    q: "지금 바로 설치할 수 있나요?",
    a: "아직 공개한 설치 파일은 없어요. 첫 릴리스는 GitHub Releases에 올리고, 그 뒤로는 앱이 서명을 확인한 업데이트만 설치해요. 개발 환경이 있다면 GitHub의 안내대로 소스에서 바로 실행해 볼 수 있어요.",
  },
  {
    q: "백업은 어디에 저장되고, 무엇이 PC 밖으로 나가나요?",
    a: "백업과 복구 기록은 내 PC의 %LOCALAPPDATA%\\RobinX 아래에만 저장돼요. RobinX가 인터넷에 연결하는 건 새 버전이 있는지 GitHub에서 확인할 때(설정에서 끌 수 있어요)와 직접 누른 웹 검색(기본 브라우저로 열어요)뿐이에요. 사용 기록이나 프로그램 목록을 보내지 않아요.",
  },
  {
    q: "잘못 정리하면 어떻게 하나요?",
    a: "복구 기록 화면에서 작업을 찾아 '모두 되돌리기'를 누르거나 원하는 항목만 골라 되돌리면 돼요. 원래 자리에 같은 이름이 새로 생겼다면 덮어쓰지 않고 알려 줘요.",
  },
]

export function Faq() {
  return (
    <section id="faq" className="relative mx-auto max-w-3xl px-6 pt-32">
      <SectionHeading eyebrow="FAQ" title="자주 묻는 질문" />
      <Reveal className="mt-12">
        <Accordion type="single" collapsible className="rounded-3xl border border-white/8 bg-card px-6">
          {faqs.map((f) => (
            <AccordionItem key={f.q} value={f.q} className="border-white/8">
              <AccordionTrigger className="py-5 text-base font-semibold hover:no-underline">{f.q}</AccordionTrigger>
              <AccordionContent className="pb-5 text-[0.95rem] leading-relaxed text-muted-foreground">
                {f.a}
              </AccordionContent>
            </AccordionItem>
          ))}
        </Accordion>
      </Reveal>
    </section>
  )
}

export function FinalCta() {
  return (
    <section className="relative mx-auto max-w-6xl px-6 pt-32">
      <Reveal>
        <div className="relative overflow-hidden rounded-[2rem] border border-primary/20 bg-gradient-to-b from-primary/12 via-card/60 to-card/40 px-6 py-16 text-center sm:py-20">
          <div
            aria-hidden
            className="bg-dots absolute inset-0 [mask-image:radial-gradient(ellipse_at_center,black,transparent_70%)]"
          />
          <div
            aria-hidden
            className="absolute -top-32 left-1/2 h-72 w-[40rem] -translate-x-1/2 rounded-full bg-[radial-gradient(closest-side,oklch(0.82_0.12_190/0.3),transparent)] blur-2xl"
          />
          <img src={LOGO} alt="" className="relative mx-auto size-16 rounded-2xl shadow-xl shadow-primary/30" />
          <h2 className="relative mt-6 text-3xl font-bold tracking-tight text-balance sm:text-4xl">
            지우기 전에, 근거부터 확인하세요
          </h2>
          <p className="relative mx-auto mt-4 max-w-xl text-pretty text-muted-foreground sm:text-lg">
            왜 지우는지 보여 주고, 정리한 흔적은 백업해 두어요. RobinX의 개발 과정을 GitHub에서 함께 지켜봐 주세요.
          </p>
          <div className="relative mt-8 flex flex-wrap justify-center gap-3">
            <Button asChild size="lg" className="h-12 rounded-xl px-6 text-base shadow-lg shadow-primary/25">
              <a href={REPO_URL} target="_blank" rel="noreferrer">
                <GithubIcon className="size-4.5" />
                개발 현황 보기
              </a>
            </Button>
            <Button asChild size="lg" variant="outline" className="h-12 rounded-xl px-6 text-base">
              <a href={`${REPO_URL}/issues`} target="_blank" rel="noreferrer">
                의견 남기기
              </a>
            </Button>
          </div>
        </div>
      </Reveal>
    </section>
  )
}

export function Footer() {
  return (
    <footer className="mx-auto mt-24 max-w-6xl border-t border-white/8 px-6 py-10">
      <div className="flex flex-col items-center justify-between gap-4 text-sm text-muted-foreground sm:flex-row">
        <div className="flex items-center gap-2.5">
          <img src={LOGO} alt="" className="size-6 rounded-md" />
          <span className="font-medium text-foreground">RobinX</span>
          <span>· Windows 올인원 관리 도구</span>
        </div>
        <div className="flex items-center gap-5">
          <a href="#features" className="transition-colors hover:text-foreground">
            기능
          </a>
          <a href="#faq" className="transition-colors hover:text-foreground">
            FAQ
          </a>
          <a href={REPO_URL} target="_blank" rel="noreferrer" className="transition-colors hover:text-foreground">
            GitHub
          </a>
          <a
            href={`${REPO_URL}/blob/main/LICENSE`}
            target="_blank"
            rel="noreferrer"
            className="transition-colors hover:text-foreground"
          >
            Apache-2.0
          </a>
        </div>
      </div>
    </footer>
  )
}
