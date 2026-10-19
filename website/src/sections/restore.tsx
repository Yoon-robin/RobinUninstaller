import { shot } from "@/lib/site"
import { Reveal } from "@/components/motion"
import { cn } from "@/lib/utils"

type Level = "가능" | "조건부" | "불가"

const tone: Record<Level, string> = {
  가능: "text-primary",
  조건부: "text-[oklch(0.5_0.12_60)]",
  불가: "text-muted-foreground",
}

/** 작업별로 무엇을 되돌릴 수 있는지. 사이트에서 가장 정확해야 하는 부분이라 앱의 동작과 똑같이 쓴다. */
const rows: { task: string; level: Level; note: string }[] = [
  { task: "시작프로그램 끄기", level: "가능", note: "등록 정보는 그대로 두고 끄기만 해요" },
  { task: "남은 흔적 정리", level: "조건부", note: "백업을 보관하는 동안 복구 기록에서" },
  { task: "강제 제거", level: "조건부", note: "백업한 흔적만 돌아와요" },
  { task: "최적화", level: "조건부", note: "그 사이 다른 곳에서 바꾼 값은 덮어쓰지 않아요" },
  { task: "프로그램 제거", level: "불가", note: "프로그램은 다시 설치해야 해요" },
  { task: "캐시·임시 파일 정리", level: "불가", note: "다시 만들어지는 것만 지워요" },
  { task: "백업 삭제·기간 만료", level: "불가", note: "그 백업으로는 더 복원할 수 없어요" },
]

export function Restore() {
  return (
    <section id="restore" className="frame grid gap-14 pt-20 sm:pt-28 lg:grid-cols-12 lg:gap-8 lg:pt-32">
      <div className="lg:col-span-5">
        <Reveal>
          <h2 className="section-title">
            되돌릴 수 있는 것,
            <br />
            없는 것
          </h2>
        </Reveal>
        <Reveal delay={0.08}>
          <p className="mt-8 max-w-[36ch] text-lead leading-snug">
            지우는 프로그램이라 이 목록이 가장 중요해요. 조건부는 백업이 남아 있을 때만 된다는 뜻이에요.
          </p>
        </Reveal>
        <ul className="mt-12 flex flex-col">
          {rows.map((r, i) => (
            <Reveal as="li" key={r.task} delay={0.12 + i * 0.04} y={12} className="py-3">
              <p className="flex items-baseline justify-between gap-6">
                <span className="text-lead font-semibold">{r.task}</span>
                <span className={cn("shrink-0 text-body font-semibold", tone[r.level])}>{r.level}</span>
              </p>
              <p className="text-body text-muted-foreground">{r.note}</p>
            </Reveal>
          ))}
        </ul>
      </div>

      <Reveal delay={0.16} className="lg:col-span-7 lg:pt-4">
        <div className="lg:sticky lg:top-10">
          <img
            src={shot("restore")}
            alt="복구 기록 화면: 작업마다 백업된 항목과 되돌리기 버튼이 있어요"
            width={1280}
            height={800}
            loading="lazy"
            className="w-full rounded-[1.25rem] bg-night"
          />
          <p className="mt-5 max-w-[52ch] text-body text-muted-foreground">
            복구 기록. 원래 자리에 같은 이름이 새로 생겼으면 덮어쓰지 않고 알려 줘요. 백업은 직접 지우거나 정해 둔 보관
            기간이 지나야 사라져요.
          </p>
        </div>
      </Reveal>
    </section>
  )
}
