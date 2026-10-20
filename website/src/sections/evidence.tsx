import { CircleCheck, CircleDashed, ShieldMinus } from "lucide-react"

import { shot } from "@/lib/site"
import { Reveal } from "@/components/motion"

/**
 * 남은 흔적을 어떤 근거로 나누는지. 앱의 탐지 규칙(crates/scanner)과 같은 말로 쓴다.
 * 왼쪽은 앱의 "남은 흔적" 창을 그대로 확대한 그림, 오른쪽은 세 갈래 설명. 색만으로 나누지 않도록 아이콘과 이름을 함께 쓴다.
 */
const groups = [
  {
    icon: CircleCheck,
    tone: "text-primary",
    chip: "bg-primary/10 text-primary",
    label: "연결 정보 일치",
    title: "미리 골라 둬요",
    text: "Windows에 기록된 설치 폴더, 지운 프로그램의 등록 기록, 그 폴더를 가리키는 자동 실행 설정처럼 등록 정보가 직접 가리키는 것.",
  },
  {
    icon: CircleDashed,
    tone: "text-[oklch(0.45_0.11_60)]",
    chip: "bg-[oklch(0.75_0.14_70/18%)] text-[oklch(0.45_0.11_60)]",
    label: "직접 확인 필요",
    title: "직접 보고 골라요",
    text: "이름만 같은 폴더, 다른 프로그램과 함께 쓰는 것 같은 폴더, 문서·사진이 섞인 폴더. 미리 고르지 않아요.",
  },
  {
    icon: ShieldMinus,
    tone: "text-muted-foreground",
    chip: "bg-foreground/8 text-muted-foreground",
    label: "보호 규칙으로 제외",
    title: "건드리지 않아요",
    text: "Windows 폴더, 여러 프로그램이 함께 쓰는 폴더, 드라이브 전체, 문서·바탕 화면 같은 사용자 폴더 자체, RobinX 자신의 파일.",
  },
]

export function Evidence() {
  return (
    <section id="evidence" className="bg-muted py-20 sm:py-28 lg:py-32">
      <div className="frame">
        <Reveal className="max-w-3xl">
          <h2 className="section-title">
            왜 남았는지
            <br />
            먼저 보여 줘요
          </h2>
          <p className="mt-5 max-w-[40ch] text-lead leading-snug text-muted-foreground">
            찾은 흔적마다 그 프로그램 것으로 본 이유가 붙어요. 확실한 것만 미리 고르고, 나머지는 직접 보고 정해요.
          </p>
        </Reveal>

        <div className="mt-12 grid items-start gap-10 sm:mt-14 lg:grid-cols-12 lg:gap-12">
          {/* 앱의 남은 흔적 창만 잘라 크게. 항목마다 붙은 이유(등록된 설치 폴더 등)가 읽히는 크기 */}
          <Reveal className="lg:col-span-7">
            <figure>
              <div className="aspect-[766/702] overflow-hidden rounded-[1.5rem] bg-background">
                <img
                  src={shot("leftovers")}
                  alt="남은 흔적 창 확대: 항목마다 '등록된 설치 폴더', '남은 제거 등록 정보', '설치 폴더를 가리킴', '게시자 › 제품 이름 일치' 같은 이유가 붙어 있어요"
                  width={1280}
                  height={800}
                  loading="lazy"
                  decoding="async"
                  className="block max-w-none"
                  style={{ width: "167.1%", marginLeft: "-33.6%", marginTop: "-6.3%" }}
                />
              </div>
              <figcaption className="mt-3 text-fine text-muted-foreground">
                앱 화면 그대로예요. 프로그램 이름과 경로는 예시예요.
              </figcaption>
            </figure>
          </Reveal>

          <div className="flex flex-col gap-9 lg:col-span-5 lg:pt-2">
            {groups.map((g, i) => (
              <Reveal key={g.label} delay={0.05 * i}>
                <span data-ui className={`inline-flex rounded-full px-2.5 py-1 text-fine font-medium ${g.chip}`}>
                  {g.label}
                </span>
                <h3
                  className={`mt-3 flex items-center gap-2 text-lead leading-snug font-semibold tracking-tight ${g.tone}`}
                >
                  <g.icon className="size-5 shrink-0" aria-hidden />
                  {g.title}
                </h3>
                <p className="mt-2 text-body text-muted-foreground">{g.text}</p>
              </Reveal>
            ))}
            <Reveal>
              <p className="text-body font-medium">
                바탕 화면·문서·다운로드 안의 파일은 이유가 있어도 미리 고르지 않아요. 고른 것도 지우기 전에 백업해요.
              </p>
            </Reveal>
          </div>
        </div>
      </div>
    </section>
  )
}
