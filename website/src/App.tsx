import { MotionConfig } from "motion/react"

import { Nav } from "@/sections/nav"
import { Hero } from "@/sections/hero"
import { Evidence } from "@/sections/evidence"
import { Walkthrough } from "@/sections/walkthrough"
import { Features } from "@/sections/features"
import { Restore } from "@/sections/restore"
import { Status } from "@/sections/status"
import { Faq } from "@/sections/faq"
import { Cta } from "@/sections/cta"
import { Footer } from "@/sections/footer"

export default function App() {
  return (
    <MotionConfig reducedMotion="user">
      <Nav />
      <main>
        <Hero />
        <Features />
        <Evidence />
        <Walkthrough />
        <Restore />
        <Status />
        <Faq />
        <Cta />
      </main>
      <Footer />
    </MotionConfig>
  )
}
