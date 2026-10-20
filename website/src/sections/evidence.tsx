import { Reveal } from "@/components/motion"

/** 남은 흔적을 어떤 근거로 나누는지. 앱의 탐지 규칙(crates/scanner)과 같은 말로 쓴다. */
const groups = [
  {
    title: "미리 골라 두는 것",
    tone: "text-primary",
    chip: "bg-primary/10 text-primary",
    label: "연결 정보 일치",
    items: [
      ["프로그램이 설치됐던 폴더", "Windows에 기록된 설치 위치 그대로예요"],
      ["지운 프로그램의 등록 기록", "제거했는데도 Windows에 남은 설치 기록"],
      ["지운 프로그램을 부르는 설정", "PC를 켤 때 자동 실행 같은, 그 폴더를 가리키는 설정"],
    ],
  },
  {
    title: "직접 확인하게 두는 것",
    tone: "text-[oklch(0.5_0.12_60)]",
    chip: "bg-[oklch(0.75_0.14_70/18%)] text-[oklch(0.45_0.11_60)]",
    label: "직접 확인 필요",
    items: [
      ["이름만 같은 것", "이름이 같아도 다른 프로그램 것일 수 있어요"],
      ["내 파일이 섞인 폴더", "문서나 사진이 들어 있으면 미리 고르지 않아요"],
    ],
  },
  {
    title: "건드리지 않는 것",
    tone: "text-muted-foreground",
    chip: "bg-foreground/8 text-muted-foreground",
    label: "보호 규칙으로 제외",
    items: [
      ["Windows와 여러 프로그램이 함께 쓰는 폴더", "처음부터 목록에서 빼요"],
      ["드라이브 전체, RobinX 자신의 파일", "찾아도 목록에 넣지 않아요"],
    ],
  },
]

/** 가운데 제목 아래 세 칸. 각 칸 위의 작은 표시는 앱 화면에 나오는 것과 같은 모양이다. */
export function Evidence() {
  return (
    <section id="evidence" className="py-20 sm:py-28 lg:py-32">
      <div className="frame">
        <Reveal className="mx-auto max-w-3xl text-center">
          <h2 className="section-title">
            왜 남았는지
            <br />
            먼저 보여 줘요
          </h2>
          <p className="mx-auto mt-5 max-w-[38ch] text-lead leading-snug text-balance text-muted-foreground">
            남은 흔적마다 왜 그 프로그램 것으로 봤는지 이유가 붙어요. 확실한 것만 미리 고르고, 애매한 것은 직접 보고
            정하게 남겨 둬요.
          </p>
        </Reveal>

        <div className="mt-14 grid gap-4 sm:mt-16 sm:gap-5 md:grid-cols-3">
          {groups.map((g, gi) => (
            <Reveal key={g.title} delay={0.06 * gi} className="rounded-[1.75rem] bg-muted p-7 sm:p-8">
              <span data-ui className={`inline-flex rounded-full px-2.5 py-1 text-fine font-medium ${g.chip}`}>
                {g.label}
              </span>
              <h3 className={`mt-5 text-lead leading-snug font-semibold tracking-tight ${g.tone}`}>{g.title}</h3>
              <ul className="mt-6 flex flex-col gap-5">
                {g.items.map(([name, note]) => (
                  <li key={name}>
                    <p className="text-body font-semibold">{name}</p>
                    <p className="mt-0.5 text-body text-muted-foreground">{note}</p>
                  </li>
                ))}
              </ul>
            </Reveal>
          ))}
        </div>
      </div>
    </section>
  )
}
