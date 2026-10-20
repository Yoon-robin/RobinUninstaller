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

/**
 * 질문과 답을 모두 펼쳐 둔다. 하나씩 골라 보는 방식은 큰 화면에서 오른쪽 아래가 비어 보였고,
 * 답을 보려면 질문마다 눌러야 했다. 제목은 왼쪽에 붙어 있고 목록이 옆에서 지나간다.
 */
export function Faq() {
  return (
    <section id="faq" className="frame grid gap-12 py-20 sm:py-28 lg:grid-cols-12 lg:gap-8 lg:py-32">
      <div className="lg:col-span-4">
        <div className="lg:sticky lg:top-24">
          <Reveal>
            <h2 className="section-title">
              자주 묻는
              <br />
              질문
            </h2>
          </Reveal>
          <Reveal delay={0.08}>
            <a
              href={`${REPO_URL}/issues`}
              target="_blank"
              rel="noreferrer"
              className="mt-8 inline-block text-body font-medium text-primary underline-offset-4 hover:underline"
            >
              다른 질문은 GitHub에 남겨 주세요
            </a>
          </Reveal>
        </div>
      </div>

      <dl className="grid gap-x-12 gap-y-12 sm:grid-cols-2 lg:col-span-8">
        {faqs.map((f, i) => (
          <Reveal key={f.q} delay={0.06 + (i % 2) * 0.06} y={12}>
            <dt className="text-lead leading-snug font-semibold tracking-tight">{f.q}</dt>
            <dd className="mt-3 text-body leading-relaxed text-muted-foreground">{f.a}</dd>
          </Reveal>
        ))}
      </dl>
    </section>
  )
}
