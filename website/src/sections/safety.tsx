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
    text: "파일은 격리 폴더로, 레지스트리는 내보내기 파일로 보관한 뒤 치워요. 영구 삭제는 하지 않아요.",
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
        description="RobinUninstaller는 처음부터 이 네 가지 원칙을 지키도록 만들었어요."
      />
      <div className="mt-14 grid gap-4 sm:grid-cols-2">
        {principles.map((p, i) => (
          <Reveal key={p.title} delay={(i % 2) * 0.08}>
            <div className="flex h-full gap-5 rounded-3xl border border-white/8 bg-gradient-to-br from-card/80 to-card/30 p-6 sm:p-7">
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
    </section>
  )
}

const faqs = [
  {
    q: "소스 코드를 볼 수 있나요?",
    a: "네, GitHub에 모두 공개되어 있어요. 무엇을 어떻게 지우는지 코드로 직접 확인할 수 있어요.",
  },
  {
    q: "정말 아무것도 영구 삭제하지 않나요?",
    a: "남은 흔적 정리와 강제 제거는 항상 백업·격리를 거쳐요. 저장공간 정리에서 지우는 캐시처럼 다시 만들어지는 것만 확인 후 지우고, 휴지통 비우기처럼 되돌릴 수 없는 일은 따로 경고해요. 백업은 설정한 기간이 지나면 정리돼요.",
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
    a: "지금은 미리보기 단계라 설치 파일을 준비하고 있어요. 개발 환경이 있다면 GitHub의 안내대로 소스에서 바로 실행해 볼 수 있어요.",
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
        <Accordion type="single" collapsible className="rounded-3xl border border-white/8 bg-card/40 px-6">
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
            이제 마음 놓고 지우세요
          </h2>
          <p className="relative mx-auto mt-4 max-w-xl text-pretty text-muted-foreground sm:text-lg">
            남은 흔적까지 챙기고, 실수는 되돌릴 수 있게. RobinUninstaller의 개발 과정을 GitHub에서 함께 지켜봐 주세요.
          </p>
          <div className="relative mt-8 flex flex-wrap justify-center gap-3">
            <Button asChild size="lg" className="h-12 rounded-xl px-6 text-base shadow-lg shadow-primary/25">
              <a href={REPO_URL} target="_blank" rel="noreferrer">
                <GithubIcon className="size-4.5" />
                GitHub에서 보기
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
          <span className="font-medium text-foreground">RobinUninstaller</span>
          <span>· Windows용 언인스톨러</span>
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
        </div>
      </div>
    </footer>
  )
}
