import { Reveal } from "@/components/motion"
import { cn } from "@/lib/utils"

type Level = "가능" | "백업 있으면" | "불가"

const tone: Record<Level, string> = {
  가능: "text-primary",
  "백업 있으면": "text-[oklch(0.5_0.12_60)]",
  불가: "text-muted-foreground",
}

/** 작업별로 무엇을 되돌릴 수 있는지. 사이트에서 가장 정확해야 하는 부분이라 앱의 동작과 똑같이 쓴다. */
const rows: { task: string; level: Level; note: string }[] = [
  { task: "시작프로그램 끄기", level: "가능", note: "꺼 두기만 하고 지우지 않아요" },
  { task: "남은 흔적 정리", level: "백업 있으면", note: "백업이 남아 있는 동안 되돌릴 수 있어요" },
  { task: "강제 제거", level: "백업 있으면", note: "백업해 둔 파일만 돌아와요" },
  { task: "프로그램 제거", level: "불가", note: "프로그램은 다시 설치해야 해요" },
  { task: "임시 파일 정리", level: "불가", note: "프로그램이 다시 만드는 것만 지워요" },
  { task: "백업을 지운 뒤", level: "불가", note: "그 백업으로는 더 되돌릴 수 없어요" },
]

/** 가운데 제목 아래, 작업마다 되돌릴 수 있는지를 회색 타일 두 줄로. */
export function Restore() {
  return (
    <section id="restore" className="py-20 sm:py-28 lg:py-32">
      <div className="frame">
        <Reveal className="mx-auto max-w-3xl text-center">
          <h2 className="section-title">되돌릴 수 있는 것, 없는 것</h2>
          <p className="mx-auto mt-5 max-w-[36ch] text-lead leading-snug text-balance text-muted-foreground">
            '백업 있으면'은 백업을 지우기 전까지 되돌릴 수 있다는 뜻이에요. 백업은 직접 지우거나 30일이 지나면
            사라져요.
          </p>
        </Reveal>
        <ul className="mx-auto mt-14 grid max-w-5xl gap-4 sm:mt-16 sm:grid-cols-2 sm:gap-5">
          {rows.map((r, i) => (
            <Reveal as="li" key={r.task} delay={0.04 * (i % 2)} y={12} className="rounded-[1.5rem] bg-muted px-7 py-6">
              <p className="flex items-baseline justify-between gap-6">
                <span className="text-lead font-semibold tracking-tight">{r.task}</span>
                <span className={cn("shrink-0 text-body font-semibold", tone[r.level])}>{r.level}</span>
              </p>
              <p className="mt-1 text-body text-muted-foreground">{r.note}</p>
            </Reveal>
          ))}
        </ul>
      </div>
    </section>
  )
}
