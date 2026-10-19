import { lazy, Suspense } from "react"
import { MotionConfig } from "motion/react"

import { LiquidGlassFilter } from "@/components/liquid-glass"

import { Nav } from "@/sections/nav"
import { Hero } from "@/sections/hero"
import { ValueStrip } from "@/sections/features"

const BelowFold = lazy(() => import("@/below-fold"))
const Footer = lazy(() => import("@/sections/safety").then((m) => ({ default: m.Footer })))

export default function App() {
  return (
    <MotionConfig reducedMotion="user">
      <LiquidGlassFilter />
      <Nav />
      <main>
        <Hero />
        <ValueStrip />
        <Suspense fallback={<div className="min-h-screen" />}>
          <BelowFold />
        </Suspense>
      </main>
      <Suspense>
        <Footer />
      </Suspense>
    </MotionConfig>
  )
}
