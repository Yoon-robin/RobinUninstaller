import { Download } from "lucide-react"

import { DOWNLOAD_URL, LOGO } from "@/lib/site"
import { Reveal } from "@/components/motion"

/** 마지막 구역: 어두운 바탕에 내려받기 하나. 처음 실행할 때 뜨는 Windows 경고를 미리 알려 둔다. */
export function Cta() {
  return (
    <section className="bg-night py-20 text-night-foreground sm:py-28">
      <Reveal className="frame flex flex-col items-center text-center">
        <img src={LOGO} alt="" width={64} height={64} className="size-14 rounded-[1rem] sm:size-16" />
        <h2 className="section-title mt-7">지금 써 보세요</h2>
        <p className="mt-5 max-w-[34ch] text-lead leading-snug text-balance text-night-muted">
          무료이고, 소스 코드가 공개돼 있어요. 아직 미리보기라 처음 실행할 때 'Windows의 PC 보호' 창이 뜰 수 있어요.
        </p>
        <a
          href={DOWNLOAD_URL}
          target="_blank"
          rel="noreferrer"
          className="mt-9 inline-flex h-12 items-center gap-2.5 rounded-full bg-night-foreground px-7 text-body font-semibold text-night transition-[transform,opacity] hover:opacity-90 focus-visible:ring-2 focus-visible:ring-night-foreground focus-visible:ring-offset-2 focus-visible:ring-offset-night focus-visible:outline-none active:scale-[0.97]"
        >
          <Download className="size-[1.1rem]" />
          내려받기 (미리보기)
        </a>
      </Reveal>
    </section>
  )
}
