import { cn } from "@/lib/utils"

export const shot = (name: string) => `${import.meta.env.BASE_URL}shots/${name}.png`

/** 앱 스크린샷을 감싸는 창 테두리. 스크린샷 자체에 제목 표시줄이 있어 테두리와 그림자만 준다. */
export function WindowFrame({
  src,
  alt,
  className,
  eager,
}: {
  src: string
  alt: string
  className?: string
  eager?: boolean
}) {
  return (
    <div className={cn("glass overflow-hidden rounded-2xl p-1.5", className)}>
      <img
        src={src}
        alt={alt}
        width={1280}
        height={800}
        loading={eager ? "eager" : "lazy"}
        decoding="async"
        className="block h-auto w-full rounded-[0.75rem] select-none"
        draggable={false}
      />
    </div>
  )
}
