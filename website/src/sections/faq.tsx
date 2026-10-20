import { REPO_URL } from "@/lib/site"
import { Reveal } from "@/components/motion"

type Faq = { q: string; a: string }

/** 받기 전에 가장 많이 궁금한 것 (안전·되돌리기·받기·인터넷) */
const core: Faq[] = [
  {
    q: "어디서 받고, 어떻게 확인하나요?",
    a: "GitHub 공식 릴리스에서만 받으세요(이 사이트의 다운로드 단추도 그 파일로 연결돼요). 아직 Microsoft 게시자 서명이 없어 처음 실행할 때 Windows가 경고 창을 띄울 수 있어요. 실행하기 전에 릴리스 페이지의 SHA-256과 받은 파일의 값(PowerShell의 Get-FileHash)이 같은지 확인하고, 출처를 확인할 수 없는 파일은 실행하지 마세요.",
  },
  {
    q: "지운 건 다 되돌릴 수 있나요?",
    a: "아니요. 남은 흔적은 지우기 전에 백업하므로, 백업이 남아 있는 동안 되돌릴 수 있어요. 프로그램 자체의 제거와 임시 파일·휴지통 정리는 되돌릴 수 없어요.",
  },
  {
    q: "내 파일이 지워지지는 않나요?",
    a: "바탕 화면·문서·다운로드 안의 파일은 미리 고르지 않고, 그 폴더 자체는 목록에 넣지 않아요. 고른 항목도 백업한 뒤에 정리해요.",
  },
  {
    q: "인터넷으로 무엇이 나가나요?",
    a: "저절로 하는 건 새 버전이 있는지 확인하는 것뿐이에요(설정에서 끌 수 있어요). 프로그램 메뉴의 '웹에서 검색'을 직접 누르면 그 프로그램 이름과 게시자가 기본 브라우저의 검색어로 들어가요. 설치된 프로그램 목록이나 정리 기록은 어디로도 보내지 않아요.",
  },
]

const more: Faq[] = [
  {
    q: "정리하면 공간이 바로 늘어나나요?",
    a: "임시 파일은 바로 늘어나요. 남은 흔적은 백업으로 옮겨 두기 때문에, 백업을 지우거나 보관 기간이 지나 지워질 때 공간이 생겨요.",
  },
  {
    q: "백업은 어디에 있나요?",
    a: "이 PC 안, 내 계정의 RobinX 폴더(%LOCALAPPDATA%\\RobinX\\backups)에만 있어요.",
  },
  {
    q: "관리자 권한이 필요한가요?",
    a: "평소에는 필요 없어요. 모든 사용자용으로 설치된 프로그램을 정리할 때만 Windows가 권한을 허용할지 물어요.",
  },
  {
    q: "소스 코드를 볼 수 있나요?",
    a: "네, GitHub에 공개돼 있어요(Apache 2.0). 무엇을 어떻게 지우는지 직접 확인할 수 있어요.",
  },
]

function List({ items }: { items: Faq[] }) {
  return (
    <dl className="grid gap-x-12 gap-y-10 sm:grid-cols-2">
      {items.map((f) => (
        <div key={f.q}>
          <dt className="text-lead leading-snug font-semibold tracking-tight">{f.q}</dt>
          <dd className="mt-3 text-body leading-relaxed text-muted-foreground">{f.a}</dd>
        </div>
      ))}
    </dl>
  )
}

/**
 * 질문과 답은 모두 펼쳐 둔다 (눌러야 보이면 찾기 어렵다). 받기 전에 궁금한 네 가지를 먼저, 나머지는 아래에.
 */
export function Faq() {
  return (
    <section id="faq" className="frame grid gap-12 py-20 sm:py-28 lg:grid-cols-12 lg:gap-8 lg:py-32">
      <div className="lg:col-span-4">
        <div className="lg:sticky lg:top-24">
          <Reveal heading>
            <h2 className="section-title">
              자주 묻는
              <br />
              질문
            </h2>
          </Reveal>
          <Reveal>
            <a
              href={`${REPO_URL}/issues`}
              target="_blank"
              rel="noreferrer"
              className="mt-8 inline-block text-body font-medium text-stage underline-offset-4 hover:underline"
            >
              다른 질문은 GitHub에 남겨 주세요
            </a>
          </Reveal>
        </div>
      </div>

      <div className="flex flex-col gap-14 lg:col-span-8">
        <Reveal>
          <List items={core} />
        </Reveal>
        <Reveal>
          <h3 className="mb-8 text-lead font-semibold text-muted-foreground">그 밖의 질문</h3>
          <List items={more} />
        </Reveal>
      </div>
    </section>
  )
}
