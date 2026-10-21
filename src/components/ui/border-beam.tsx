// 출처: Magic UI Border Beam (https://magicui.design/docs/components/border-beam)
// MIT License, Copyright (c) Magic UI

import type { Transition } from "motion/react"

import { cn } from "@/lib/utils"

interface BorderBeamProps {
  /**
   * The size of the border beam.
   */
  size?: number
  /**
   * The duration of the border beam.
   */
  duration?: number
  /**
   * The delay of the border beam.
   */
  delay?: number
  /**
   * The color of the border beam from.
   */
  colorFrom?: string
  /**
   * The color of the border beam to.
   */
  colorTo?: string
  /**
   * The motion transition of the border beam.
   */
  transition?: Transition
  /**
   * The class name of the border beam.
   */
  className?: string
  /**
   * The style of the border beam.
   */
  style?: React.CSSProperties
  /**
   * Whether to reverse the animation direction.
   */
  reverse?: boolean
  /**
   * The initial offset position (0-100).
   */
  initialOffset?: number
  /**
   * The border width of the beam.
   */
  borderWidth?: number
}

export const BorderBeam = ({
  className,
  size = 50,
  delay = 0,
  duration = 6,
  colorFrom = "#ffaa40",
  colorTo = "#9c40ff",
  transition,
  style,
  reverse = false,
  initialOffset = 0,
  borderWidth = 1,
}: BorderBeamProps) => {
  return (
    <div
      className="pointer-events-none absolute inset-0 rounded-[inherit] border-(length:--border-beam-width) border-transparent mask-[linear-gradient(transparent,transparent),linear-gradient(#000,#000)] mask-intersect [mask-clip:padding-box,border-box]"
      style={
        {
          "--border-beam-width": `${borderWidth}px`,
        } as React.CSSProperties
      }
    >
      {/* CSS 애니메이션으로 돈다 (motion의 initial={false}가 첫 등장에서 멈춰 버렸고, 움직임 줄이기에서도 계속 돌았다).
          움직임 줄이기에서는 index.css가 멈춘 모양으로 둔다. */}
      <div
        className={cn(
          "border-beam absolute aspect-square",
          "bg-linear-to-l from-(--color-from) via-(--color-to) to-transparent",
          className
        )}
        style={
          {
            width: size,
            offsetPath: `rect(0 auto auto 0 round ${size}px)`,
            offsetDistance: `${initialOffset}%`,
            "--color-from": colorFrom,
            "--color-to": colorTo,
            "--beam-duration": `${transition?.duration ?? duration}s`,
            "--beam-delay": `${-delay}s`,
            animationDirection: reverse ? "reverse" : undefined,
            ...style,
          } as React.CSSProperties
        }
      />
    </div>
  )
}
