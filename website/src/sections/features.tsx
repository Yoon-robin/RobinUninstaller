import type { CSSProperties } from "react"

import { cn } from "@/lib/utils"
import { shot } from "@/lib/site"
import { Reveal } from "@/components/motion"

/**
 * 앱의 기능을 Apple 제품 페이지의 타일(벤토)로 보여 준다. 회색 구역 위에 흰 타일.
 * 그림은 앱 화면의 오른쪽 내용 부분만 확대해 잘라 보여 준다 (사이드바는 빼고).
 */
type Tile = {
  name: string
  text: string
  /** 타일 넓이 (큰 화면에서 6칸 중) */
  span: string
  shot?: string
  alt?: string
  /** 그림을 얼마나 확대하고 어디서부터 보여 줄지 */
  crop?: CSSProperties
}

const tiles: Tile[] = [
  {
    name: "남은 흔적 정리",
    text: "제거한 뒤 남은 파일과 레지스트리를 찾아요. 확실한 것은 미리 고르고, 애매한 것은 직접 보고 고르게 나눠요. 고른 것만 백업한 뒤 지워요.",
    span: "lg:col-span-4",
    shot: "leftovers",
    alt: "남은 흔적 화면: 미리 고른 항목과 직접 확인할 항목이 나뉘어 있어요",
    crop: { width: "160%", marginLeft: "-32%", marginTop: "-6%" },
  },
  {
    name: "복구 기록",
    text: "정리할 때마다 백업이 남아서, 원래 자리로 되돌릴 수 있어요.",
    span: "lg:col-span-2",
    shot: "restore",
    alt: "복구 기록 화면",
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
  {
    name: "강제 제거",
    text: "제거 기능이 없거나 고장 난 프로그램도 남은 파일을 찾아 백업한 뒤 정리해요. 백신 같은 보안 프로그램과 장치 드라이버는 다루지 않아요.",
    span: "lg:col-span-3",
  },
  {
    name: "설치 추적",
    text: "새 프로그램을 설치할 때 무엇이 새로 생겼는지 기록해 둬요. 나중에 지울 때 그 기록으로 남은 흔적을 더 정확히 찾아요.",
    span: "lg:col-span-3",
  },
]

export function Features() {
  return (
    <section id="features" className="bg-muted py-20 sm:py-28 lg:py-32">
      <div className="frame">
        <Reveal className="mx-auto max-w-3xl text-center">
          <h2 className="section-title">RobinX로 할 수 있는 일</h2>
          <p className="mx-auto mt-5 max-w-[40ch] text-lead leading-snug text-balance text-muted-foreground">
            프로그램 제거 말고도 PC를 정리할 때 쓰는 도구가 들어 있어요.
          </p>
        </Reveal>

        <div className="mt-14 grid gap-4 sm:mt-16 sm:gap-5 lg:grid-cols-6">
          {tiles.map((t, i) => (
            <Reveal
              key={t.name}
              delay={0.04 * (i % 3)}
              className={cn("flex flex-col overflow-hidden rounded-[1.75rem] bg-background", t.span)}
            >
              <div className="p-7 sm:p-8">
                <h3 className="text-lead leading-snug text-balance font-semibold tracking-tight">{t.name}</h3>
                <p className="mt-2 max-w-[46ch] text-body text-muted-foreground">{t.text}</p>
              </div>
              {t.shot && (
                <div className="mt-auto ml-7 h-56 overflow-hidden rounded-tl-[1rem] bg-muted sm:ml-8 sm:h-72">
                  <img
                    src={shot(t.shot)}
                    alt={t.alt}
                    width={1280}
                    height={800}
                    loading="lazy"
                    className="block max-w-none"
                    style={t.crop}
                  />
                </div>
              )}
            </Reveal>
          ))}
        </div>
      </div>
    </section>
  )
}
