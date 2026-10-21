import { CircleCheck, CircleDashed, CircleX } from "lucide-react"

import { motion, useReducedMotion } from "motion/react"

import { easeOutQuint, Reveal } from "@/components/motion"
import { cn } from "@/lib/utils"

type Level = "가능" | "백업이 있는 동안" | "안 됨"

const levelStyle: Record<Level, { icon: typeof CircleCheck; tone: string }> = {
  가능: { icon: CircleCheck, tone: "text-sure" },
  "백업이 있는 동안": { icon: CircleDashed, tone: "text-review" },
  "안 됨": { icon: CircleX, tone: "text-muted-foreground" },
}

/** 작업별로 무엇을 되돌릴 수 있는지. 사이트에서 가장 정확해야 하는 부분이라 앱의 동작과 똑같이 쓴다. */
const rows: { task: string; level: Level; note: string }[] = [
  { task: "시작프로그램 끄기", level: "가능", note: "꺼 두기만 해요. 다시 켜면 끄기 전 설정 그대로 돌아와요." },
  {
    task: "남은 흔적 정리",
    level: "백업이 있는 동안",
    note: "원래 자리에 같은 이름이 새로 생겼으면 덮어쓰지 않고, 그 파일은 백업에 남겨 알려 줘요.",
  },
  {
    task: "강제 제거",
    level: "백업이 있는 동안",
    note: "백업해 둔 등록 정보와 파일만 돌아와요. 프로그램이 다시 설치되는 건 아니에요.",
  },
  {
    task: "파일 없는 시작 항목 정리",
    level: "백업이 있는 동안",
    note: "백업해 둔 등록 값과 바로 가기가 돌아와요.",
  },
  {
    task: "프로그램 제거",
    level: "안 됨",
    note: "원래 언인스톨러가 지운 것은 RobinX가 되돌릴 수 없어요. 다시 설치해야 해요.",
  },
  {
    task: "임시 파일·캐시 정리",
    level: "안 됨",
    note: "프로그램이 다시 만드는 것만 지우지만, 지운 파일 자체는 돌아오지 않아요.",
  },
  { task: "휴지통 비우기", level: "안 됨", note: "따로 경고하고, 직접 골라 한 번 더 확인해야 지워요." },
]

/**
 * 작업 × 되돌리기 × 조건의 표. 줄 사이는 선 대신 여백으로 나눈다. 큰 화면에서는 세 칸 표, 좁은 화면에서는 줄마다 쌓는다 (옆으로 밀지 않는다).
 * 되돌릴 수 있는지는 색과 함께 아이콘·글자로 나타낸다.
 */
export function Restore() {
  const reduce = useReducedMotion()
  return (
    <section id="restore" className="py-20 sm:py-28 lg:py-32">
      <div className="frame">
        <Reveal heading className="grid gap-5 lg:grid-cols-12 lg:items-end lg:gap-8">
          <h2 className="section-title lg:col-span-7">되돌릴 수 있는 것, 없는 것</h2>
          <p className="max-w-[34ch] text-lead leading-snug text-muted-foreground lg:col-span-5">
            백업은 직접 지우거나, 보관 기간(처음 설정은 30일)이 지난 뒤 앱을 켤 때 지워져요. 그 뒤로는 되돌릴 수 없어요.
          </p>
        </Reveal>

        <div className="mt-12 sm:mt-14">
          <table className="w-full text-left">
            <caption className="sr-only">작업마다 되돌릴 수 있는지와 조건</caption>
            <thead className="hidden text-fine text-muted-foreground md:table-header-group">
              <tr>
                <th scope="col" className="pb-3 font-medium">
                  작업
                </th>
                <th scope="col" className="pb-3 font-medium">
                  되돌리기
                </th>
                <th scope="col" className="pb-3 font-medium">
                  조건
                </th>
              </tr>
            </thead>
            <tbody>
              {rows.map((r, i) => {
                const { icon: Icon, tone } = levelStyle[r.level]
                return (
                  <motion.tr
                    key={r.task}
                    initial={reduce ? false : { opacity: 0, y: 10 }}
                    whileInView={{ opacity: 1, y: 0 }}
                    viewport={{ once: true, margin: "-40px" }}
                    transition={{ duration: 0.5, delay: 0.06 * i, ease: easeOutQuint }}
                    className="grid grid-cols-[1fr_auto] gap-x-4 gap-y-1 rounded-xl py-4 transition-colors hover:bg-white/[0.025] md:table-row md:py-0"
                  >
                    <th
                      scope="row"
                      className="text-lead leading-snug font-semibold tracking-tight md:w-[26%] md:py-4 md:pr-6 md:align-top"
                    >
                      {r.task}
                    </th>
                    <td className={cn("md:w-[20%] md:py-4 md:pr-6 md:align-top", tone)}>
                      <span className="inline-flex items-center gap-1.5 pt-1 text-body font-semibold whitespace-nowrap">
                        <Icon className="size-4 shrink-0" aria-hidden />
                        {r.level}
                      </span>
                    </td>
                    <td className="col-span-2 text-body text-muted-foreground md:py-4 md:align-top">{r.note}</td>
                  </motion.tr>
                )
              })}
            </tbody>
          </table>
        </div>
      </div>
    </section>
  )
}
