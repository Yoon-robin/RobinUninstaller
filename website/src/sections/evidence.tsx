import { Reveal } from "@/components/motion"

/** 남은 흔적을 어떤 근거로 나누는지. 앱의 탐지 규칙(crates/scanner)과 같은 말로 쓴다. */
const groups = [
  {
    title: "미리 골라 두는 것",
    tone: "text-primary",
    items: [
      ["등록된 설치 폴더", "제거 등록 정보가 직접 가리키는 폴더"],
      ["남은 제거 등록 정보", "제거된 뒤에도 남은 Uninstall 키"],
      ["설치 폴더를 가리키는 값", "시작 항목, App Paths, 서비스"],
    ],
  },
  {
    title: "직접 확인하게 두는 것",
    tone: "text-[oklch(0.5_0.12_60)]",
    items: [
      ["게시자·제품 이름만 일치", "이름이 같아도 다른 프로그램 것일 수 있어요"],
      ["내 문서가 섞인 폴더", "사용자 파일이 있으면 미리 고르지 않아요"],
    ],
  },
  {
    title: "건드리지 않는 것",
    tone: "text-muted-foreground",
    items: [
      ["Windows와 공유 폴더", "보호 규칙으로 처음부터 빼요"],
      ["드라이브 루트, RobinX 자신의 데이터", "찾더라도 목록에 넣지 않아요"],
    ],
  },
]

export function Evidence() {
  return (
    <section id="evidence" className="frame grid gap-12 pt-20 sm:pt-28 lg:grid-cols-12 lg:gap-8 lg:pt-32">
      <div className="lg:col-span-5">
        <Reveal>
          <h2 className="section-title">
            왜 남았는지
            <br />
            먼저 보여 줘요
          </h2>
        </Reveal>
        <Reveal delay={0.08}>
          <p className="mt-8 max-w-[34ch] text-lead leading-snug">
            남은 흔적마다 그 프로그램 것이라는 근거가 붙어요. 근거가 강한 것만 미리 고르고, 애매한 것은 직접 보고
            정하도록 남겨 둬요.
          </p>
        </Reveal>
      </div>

      <div className="flex flex-col gap-14 lg:col-span-7">
        {groups.map((g, gi) => (
          <Reveal key={g.title} delay={0.12 + gi * 0.08}>
            <h3 className={`text-body font-semibold ${g.tone}`}>{g.title}</h3>
            <ul className="mt-5 flex flex-col gap-5">
              {g.items.map(([name, note]) => (
                <li key={name} className="grid gap-1 sm:grid-cols-[1fr_auto] sm:items-baseline sm:gap-6">
                  <span className="text-lead leading-snug font-semibold tracking-tight">{name}</span>
                  <span className="text-body text-muted-foreground sm:text-right">{note}</span>
                </li>
              ))}
            </ul>
          </Reveal>
        ))}
      </div>
    </section>
  )
}
