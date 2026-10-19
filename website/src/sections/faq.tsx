import { useState } from "react"
import { AnimatePresence, motion, useReducedMotion } from "motion/react"

import { cn } from "@/lib/utils"
import { REPO_URL } from "@/lib/site"
import { Reveal } from "@/components/motion"

const faqs = [
  {
    q: "어디서 내려받나요?",
    a: "위의 '내려받기'를 누르면 GitHub에서 설치 파일을 받을 수 있어요. 처음 실행할 때 'Windows의 PC 보호' 창이 뜰 수 있는데, 아직 Microsoft의 유료 게시자 인증을 받지 않은 새 앱이라 그래요. '추가 정보'를 누르고 '실행'을 누르면 설치돼요. 그 뒤로는 새 버전이 나오면 앱이 알려 줘요.",
  },
  {
    q: "지운 건 다 되돌릴 수 있나요?",
    a: "남은 흔적은 지우기 전에 백업해 두므로, 백업이 남아 있는 동안 되돌릴 수 있어요. 임시 파일은 프로그램이 다시 만드는 것만 지우고, 지운 뒤에는 되돌릴 수 없어요.",
  },
  {
    q: "정리하면 공간이 바로 늘어나나요?",
    a: "임시 파일을 지우면 바로 늘어나요. 남은 흔적은 되돌릴 수 있게 백업해 두기 때문에, 백업을 지우거나 30일이 지나 자동으로 지워질 때 공간이 생겨요.",
  },
  {
    q: "백업은 어디에 있나요?",
    a: "이 PC 안, 내 계정의 RobinX 폴더에만 보관돼요. 다른 곳으로 보내지 않아요.",
  },
  {
    q: "인터넷으로 무엇이 나가나요?",
    a: "RobinX가 인터넷에 연결하는 건 새 버전이 나왔는지 확인할 때뿐이에요. 설정에서 끌 수 있어요. 내 PC의 프로그램 목록이나 사용 기록은 보내지 않아요.",
  },
  {
    q: "관리자 권한이 필요한가요?",
    a: "평소에는 그냥 실행돼요. 이 PC의 모든 사용자용으로 설치된 프로그램을 정리할 때만 Windows가 '이 앱이 디바이스를 변경하도록 허용하시겠어요?'라고 물어요.",
  },
  {
    q: "소스 코드를 볼 수 있나요?",
    a: "네, 누구나 볼 수 있게 GitHub에 공개돼 있어요. 무엇을 어떻게 지우는지 직접 확인할 수 있고, 자유롭게 쓰고 고칠 수 있어요(Apache 2.0 라이선스).",
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
            다른 질문은 GitHub에 남겨 주세요
          </a>
        </div>
      </div>
    </section>
  )
}
