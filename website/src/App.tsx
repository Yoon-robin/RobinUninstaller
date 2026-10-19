import { MotionConfig } from "motion/react"

import { Nav } from "@/sections/nav"
import { Hero } from "@/sections/hero"
import { Evidence } from "@/sections/evidence"
import { Tools } from "@/sections/tools"
import { Restore } from "@/sections/restore"
import { Status } from "@/sections/status"
import { Faq } from "@/sections/faq"
import { Footer } from "@/sections/footer"

export default function App() {
  return (
    <MotionConfig reducedMotion="user">
      <Nav />
      <main>
        <Hero />
        <Evidence />
        <Tools />
        <Restore />
        <Status />
        <Faq />
      </main>
      <Footer />
    </MotionConfig>
  )
}
