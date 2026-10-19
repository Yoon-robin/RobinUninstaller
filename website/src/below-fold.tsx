import { useEffect } from "react"

import { Features } from "@/sections/features"
import { Gallery, How } from "@/sections/how"
import { Faq, FinalCta, Safety } from "@/sections/safety"
import { Status } from "@/sections/status"

/** 첫 화면 아래 구역들. 나중에 받아 와 첫 로딩을 가볍게 한다. */
export default function BelowFold() {
  // 주소에 #faq 같은 앵커가 있으면, 이 구역들이 그려진 뒤에 그 자리로 옮긴다
  useEffect(() => {
    if (location.hash) document.querySelector(location.hash)?.scrollIntoView()
  }, [])

  return (
    <>
      <Features />
      <How />
      <Gallery />
      <Safety />
      <Status />
      <Faq />
      <FinalCta />
    </>
  )
}
