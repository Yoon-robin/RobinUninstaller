import { useState } from "react"
import { AnimatePresence, motion, useReducedMotion } from "motion/react"

import { cn } from "@/lib/utils"
import { REPO_URL } from "@/lib/site"
import { Reveal } from "@/components/motion"

const faqs = [
  {
    q: "지금 설치할 수 있나요?",
    a: "아직 공개한 설치 파일은 없어요. 첫 릴리스는 GitHub Releases에 올리고, 그 뒤로는 앱이 서명을 확인한 업데이트만 설치해요. 개발 환경이 있다면 소스에서 바로 실행해 볼 수 있어요.",
  },
  {
    q: "지운 건 다 되돌릴 수 있나요?",
    a: "남은 흔적 정리와 강제 제거는 항상 백업을 거쳐서, 백업을 보관하는 동안 되돌릴 수 있어요. 캐시·임시 파일 정리는 다시 만들어지는 것만 확인을 받고 지우며, 지운 뒤에는 돌아오지 않아요.",
  },
  {
    q: "정리하면 공간이 바로 늘어나나요?",
    a: "캐시·임시 파일 정리는 바로 늘어나요. 남은 흔적은 되돌릴 수 있도록 같은 드라이브에 백업해 두기 때문에, 복구 기록에서 백업을 지우거나 보관 기간(처음에는 30일)이 지나야 공간이 돌아와요.",
  },
  {
    q: "백업은 어디에 있나요?",
    a: "내 PC의 %LOCALAPPDATA%\\RobinX 아래에만 저장돼요. 파일은 격리 폴더로 옮기고, 레지스트리는 내보낸 파일로 보관해요.",
  },
  {
    q: "인터넷으로 무엇이 나가나요?",
    a: "RobinX가 직접 연결하는 건 새 버전이 있는지 GitHub에서 확인할 때뿐이에요(설정에서 끌 수 있어요). 웹 검색은 기본 브라우저로 열어요. 사용 기록이나 프로그램 목록은 보내지 않아요.",
  },
  {
    q: "관리자 권한이 필요한가요?",
    a: "평소에는 일반 권한으로 실행돼요. 모든 사용자용으로 설치된 프로그램의 흔적처럼 관리자 권한이 필요한 항목을 정리할 때만 Windows 확인 창을 띄워요.",
  },
  {
    q: "소스 코드를 볼 수 있나요?",
    a: "네, GitHub에 모두 공개되어 있고 Apache License 2.0으로 쓰고 고치고 배포할 수 있어요. 무엇을 어떻게 지우는지 코드로 직접 확인할 수 있어요.",
  },
]

/** 왼쪽은 질문 목록, 오른쪽은 고른 질문의 답. 좁은 화면에서는 질문 바로 아래에 답이 열린다. */
export function Faq() {
  const [open, setOpen] = useState(0)
  const reduce = useReducedMotion()

  return (
    <section id="faq" className="frame grid gap-12 pt-20 sm:pt-28 lg:grid-cols-12 lg:gap-8 lg:pt-32">
      <div className="lg:col-span-5">
        <Reveal>
          <h2 className="section-title">자주 묻는 질문</h2>
        </Reveal>
        <ul className="mt-10 flex flex-col">
          {faqs.map((f, i) => {
            const on = i === open
            return (
              <Reveal as="li" key={f.q} delay={0.08 + i * 0.04} y={12}>
                <button
                  type="button"
                  aria-expanded={on}
                  onClick={() => setOpen(i)}
                  onMouseEnter={() => matchMedia("(min-width: 1024px)").matches && setOpen(i)}
                  className={cn(
                    "flex w-full items-center gap-4 rounded-xl py-3 text-left text-body transition-colors focus-visible:ring-2 focus-visible:ring-ring focus-visible:outline-none",
                    on ? "font-semibold text-foreground" : "text-muted-foreground hover:text-foreground",
                  )}
                >
                  <span
                    aria-hidden
                    className={cn(
                      "size-2 shrink-0 rounded-full transition-colors",
                      on ? "bg-primary" : "bg-foreground/25",
                    )}
                  />
                  {f.q}
                </button>
                {/* 좁은 화면: 질문 바로 아래에 답 */}
                <AnimatePresence initial={false}>
                  {on && (
                    <motion.p
                      initial={reduce ? false : { opacity: 0, height: 0 }}
                      animate={{ opacity: 1, height: "auto" }}
                      exit={{ opacity: 0, height: 0 }}
                      transition={{ duration: 0.25 }}
                      className="overflow-hidden pb-3 pl-6 text-body lg:hidden"
                    >
                      {f.a}
                    </motion.p>
                  )}
                </AnimatePresence>
              </Reveal>
            )
          })}
        </ul>
      </div>

      {/* 넓은 화면: 답이 큰 글자로 */}
      <div className="hidden lg:col-span-6 lg:col-start-7 lg:block">
        <div className="sticky top-16 pt-28">
          <AnimatePresence mode="wait" initial={false}>
            <motion.p
              key={open}
              initial={{ opacity: 0 }}
              animate={{ opacity: 1 }}
              exit={{ opacity: 0 }}
              transition={{ duration: 0.15 }}
              className="max-w-[30ch] text-lead leading-[1.4] tracking-tight"
              aria-live="polite"
            >
              {faqs[open].a}
            </motion.p>
          </AnimatePresence>
          <a
            href={`${REPO_URL}/issues`}
            target="_blank"
            rel="noreferrer"
            className="mt-10 inline-block text-body font-medium text-primary underline-offset-4 hover:underline"
          >
            다른 질문은 GitHub 이슈로 남겨 주세요
          </a>
        </div>
      </div>
    </section>
  )
}
