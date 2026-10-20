import { LOGO } from "@/lib/site"
import { Reveal } from "@/components/motion"
import { DownloadButton, DownloadFacts } from "@/components/download"

/** 마지막 구역: 어두운 바탕에 내려받기 하나와 받기 전에 알아야 할 것. */
export function Cta() {
  return (
    <section className="relative overflow-hidden py-24 sm:py-32">
      <div aria-hidden className="horizon pointer-events-none absolute inset-0" />
      <Reveal className="frame relative flex flex-col items-center text-center">
        <img src={LOGO} alt="" width={64} height={64} className="size-14 rounded-[1rem] sm:size-16" />
        <h2 className="section-title mt-7">지금 써 보세요</h2>
        <p className="mt-5 max-w-[32ch] text-lead leading-snug text-balance text-muted-foreground">
          무료이고 소스 코드가 공개돼 있어요. GitHub 공식 릴리스의 파일만 받아 주세요.
        </p>
        <DownloadButton className="mt-9" />
        <DownloadFacts className="mt-5 max-w-[46ch] text-balance text-muted-foreground" linkClassName="text-stage" />
      </Reveal>
    </section>
  )
}
