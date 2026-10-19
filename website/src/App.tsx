import { MotionConfig } from "motion/react"

import { LiquidGlassFilter } from "@/components/liquid-glass"

import { Nav } from "@/sections/nav"
import { Hero } from "@/sections/hero"
import { Features, ValueStrip } from "@/sections/features"
import { Gallery, How } from "@/sections/how"
import { Faq, FinalCta, Footer, Safety } from "@/sections/safety"

export default function App() {
  return (
    <MotionConfig reducedMotion="user">
      <LiquidGlassFilter />
      <Nav />
      <main>
        <Hero />
        <ValueStrip />
        <Features />
        <How />
        <Gallery />
        <Safety />
        <Faq />
        <FinalCta />
      </main>
      <Footer />
    </MotionConfig>
  )
}
