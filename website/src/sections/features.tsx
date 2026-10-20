import type { CSSProperties } from "react"

import { cn } from "@/lib/utils"
import { shot } from "@/lib/site"
import { Reveal, Spotlight } from "@/components/motion"

/**
 * 남은 흔적 말고 앱에 있는 도구. 흰 구역 위에 회색 타일.
 * 중요한 순서대로 크기를 다르게 한다: 제거(넓게)와 복구 기록(옆), 저장공간·시작프로그램(반반), 그림 없는 두 기능은 한 줄.
 */
type Tile = {
  name: string
  text: string
  /** 큰 화면에서 6칸 중 넓이 */
  span: string
  shot: string
  alt: string
  /** 그림 틀의 높이 (기본: 고정 높이). 넓고 낮게 자른 그림은 비율로 정해 아래가 비지 않게 한다 */
  frame?: string
  /** 앱 화면에서 보여 줄 부분: 사이드바를 빼고 내용만 확대한다 (그림 너비 대비 비율이라 화면 너비가 바뀌어도 같은 부분이 보인다) */
  crop: CSSProperties
}

const tiles: Tile[] = [
  {
    name: "프로그램 제거",
    text: "설치된 프로그램과 크기를 보고, 원래 언인스톨러로 지워요. 끝나면 바로 남은 흔적을 찾아 줘요.",
    span: "lg:col-span-4",
    shot: "programs",
    alt: "설치된 프로그램 화면: 프로그램 수, 전체 용량, 언인스톨러가 없는 프로그램 수와 목록",
    // 안내·요약 카드 아래의 목록 부분 (이름·게시자·크기·설치일)
    crop: { width: "128%", marginLeft: "-27%", marginTop: "-41%" },
    frame: "aspect-[5/2]",
  },
  {
    name: "복구 기록",
    text: "정리할 때마다 백업이 남아요. 보관하는 동안 원래 자리로 되돌릴 수 있어요.",
    span: "lg:col-span-2",
    shot: "restore",
    alt: "복구 기록 화면: 정리한 작업마다 되돌리기 단추가 있어요",
    crop: { width: "210%", marginLeft: "-56%", marginTop: "-8%" },
  },
  {
    name: "저장공간 정리",
    text: "프로그램이 다시 만드는 임시 파일만 미리 골라요. 휴지통처럼 되돌릴 수 없는 것은 따로 물어봐요.",
    span: "lg:col-span-3",
    shot: "storage",
    alt: "저장공간 정리 화면",
    crop: { width: "150%", marginLeft: "-30%", marginTop: "-6%" },
  },
  {
    name: "시작프로그램",
    text: "PC를 켤 때 같이 실행되는 프로그램을 끄고 켜요. 꺼 두기만 하고 지우지는 않아요.",
    span: "lg:col-span-3",
    shot: "startup",
    alt: "시작프로그램 화면",
    crop: { width: "150%", marginLeft: "-30%", marginTop: "-6%" },
  },
]

/** 그림 없이 설명하는 두 기능: 한 타일 안에서 반씩 */
const pairs = [
  {
    name: "강제 제거",
    text: "언인스톨러가 없거나 고장 난 프로그램도 등록 정보와 남은 파일을 찾아 백업한 뒤 정리해요. 백신·VPN 같은 보안 프로그램과 장치 드라이버는 다루지 않아요.",
  },
  {
    name: "큰 폴더·큰 파일 찾기",
    text: "어떤 폴더와 파일이 공간을 많이 차지하는지 찾아 크기순으로 보여 줘요. 내 파일일 수 있어서 지우지 않고, 탐색기로 열어 주기만 해요.",
  },
]

function Shot({ t }: { t: Tile }) {
  return (
    <div
      className={cn(
        "mt-auto ml-7 overflow-hidden rounded-tl-[1rem] border-t border-l bg-card transition-transform duration-500 ease-[cubic-bezier(0.22,1,0.36,1)] group-hover/spot:-translate-y-1 sm:ml-8",
        t.frame ?? "h-56 sm:h-72",
      )}
    >
      <img
        src={shot(t.shot)}
        alt={t.alt}
        width={1280}
        height={800}
        loading="lazy"
        decoding="async"
        className="block max-w-none"
        style={t.crop}
      />
    </div>
  )
}

export function Features() {
  return (
    <section id="features" className="py-20 sm:py-28 lg:py-32">
      <div className="frame">
        <Reveal heading className="mx-auto max-w-3xl text-center">
          <h2 className="section-title">제거 말고도 할 수 있는 일</h2>
          <p className="mx-auto mt-5 max-w-[34ch] text-lead leading-snug text-balance text-muted-foreground">
            저장공간 정리, 시작프로그램, 백업 관리도 같이 들어 있어요.
          </p>
        </Reveal>

        <div className="mt-12 grid gap-4 sm:mt-14 sm:gap-5 lg:grid-cols-6">
          {tiles.map((t, i) => (
            <Reveal key={t.name} delay={0.06 * (i % 2)} className={t.span}>
              <Spotlight className="flex h-full flex-col overflow-hidden rounded-[1.75rem] border bg-white/[0.025] transition-colors duration-300 hover:border-white/15">
                <div className="p-7 sm:p-8">
                  <h3 className="text-lead leading-snug font-semibold tracking-tight">{t.name}</h3>
                  <p className="mt-2 max-w-[42ch] text-body text-muted-foreground">{t.text}</p>
                </div>
                <Shot t={t} />
              </Spotlight>
            </Reveal>
          ))}

          <Reveal className="grid gap-8 rounded-[1.75rem] border bg-white/[0.025] p-7 sm:p-8 md:grid-cols-2 md:gap-12 lg:col-span-6">
            {pairs.map((p) => (
              <div key={p.name}>
                <h3 className="text-lead leading-snug font-semibold tracking-tight">{p.name}</h3>
                <p className="mt-2 max-w-[46ch] text-body text-muted-foreground">{p.text}</p>
              </div>
            ))}
          </Reveal>
        </div>
      </div>
    </section>
  )
}
