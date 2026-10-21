/**
 * 말줄임(`truncate`)으로 잘린 글자에 마우스를 올리면 전체 글자를 title로 보여 준다.
 * 화면마다 title을 따로 붙이면 빠뜨리기 쉬워서(잔여물·시작프로그램·큰 프로그램·큰 폴더·일괄 제거 등) 한 곳에서 처리한다.
 * 이미 title이 있는 요소는 건드리지 않고, 잘리지 않았으면 붙였던 title을 뗀다.
 */
export function installAutoTitle() {
  document.addEventListener(
    "pointerover",
    (e) => {
      const el = (e.target as Element | null)?.closest?.<HTMLElement>(".truncate")
      if (!el) return
      const auto = el.dataset.autoTitle === "1"
      if (el.title && !auto) return
      if (el.scrollWidth > el.clientWidth + 1) {
        el.title = (el.textContent ?? "").trim()
        el.dataset.autoTitle = "1"
      } else if (auto) {
        el.removeAttribute("title")
        delete el.dataset.autoTitle
      }
    },
    { passive: true },
  )
}
