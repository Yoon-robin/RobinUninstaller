import { useEffect, useState, type RefObject } from "react"
import { Search, X } from "lucide-react"

import { cn } from "@/lib/utils"
import { Input } from "@/components/ui/input"

/**
 * 목록 화면의 검색창 (프로그램·Store 앱·시작프로그램 공용).
 * - 한글을 조합하는 동안("ㅋ", "칼")에는 목록을 거르지 않고, 글자가 완성되면 거른다 (목록이 깜빡이지 않게).
 * - Esc는 검색어를 지우고, X도 지운 뒤 검색창에 포커스를 남긴다.
 * - Ctrl+F: 이 검색창이 보이고 대화상자가 열려 있지 않을 때 검색창으로 간다.
 * - Enter: `onEnter`가 있으면 부른다 (첫 결과 열기 등).
 */
export function SearchInput({
  value,
  onChange,
  onEnter,
  inputRef,
  placeholder = "이름 또는 게시자 검색",
  className,
  inputClassName,
}: {
  value: string
  onChange: (value: string) => void
  onEnter?: () => void
  inputRef: RefObject<HTMLInputElement | null>
  placeholder?: string
  className?: string
  inputClassName?: string
}) {
  // 화면에 보이는 글자 (조합 중인 글자 포함). 바깥 값은 조합이 끝난 글자만 받는다.
  const [text, setText] = useState(value)
  useEffect(() => setText(value), [value])

  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      const input = inputRef.current
      // 다른 화면·탭의 숨은 검색창은 가로채지 않는다
      if (!(e.ctrlKey || e.metaKey) || e.key.toLowerCase() !== "f" || !input?.offsetParent) return
      if (document.querySelector('[role="dialog"]')) return
      e.preventDefault()
      input.focus()
      input.select()
    }
    window.addEventListener("keydown", onKey)
    return () => window.removeEventListener("keydown", onKey)
  }, [inputRef])

  const clear = () => {
    setText("")
    onChange("")
    inputRef.current?.focus()
  }

  return (
    <div className={cn("relative", className)}>
      <Search className="pointer-events-none absolute top-1/2 left-3 size-4 -translate-y-1/2 text-muted-foreground" />
      <Input
        ref={inputRef}
        value={text}
        onChange={(e) => {
          setText(e.target.value)
          if (!(e.nativeEvent as InputEvent).isComposing) onChange(e.target.value)
        }}
        onCompositionEnd={(e) => onChange(e.currentTarget.value)}
        onKeyDown={(e) => {
          if (e.nativeEvent.isComposing) return
          if (e.key === "Escape" && text) {
            e.stopPropagation()
            clear()
          }
          if (e.key === "Enter") onEnter?.()
        }}
        placeholder={placeholder}
        className={cn("h-9 w-full rounded-lg bg-background/60 pr-9 pl-9 lg:pr-14", inputClassName)}
      />
      {text ? (
        <button
          type="button"
          onClick={clear}
          aria-label="검색어 지우기"
          className="absolute top-1/2 right-2 flex size-5 -translate-y-1/2 items-center justify-center rounded-full text-muted-foreground hover:bg-muted hover:text-foreground"
        >
          <X className="size-3.5" />
        </button>
      ) : (
        <kbd className="pointer-events-none absolute top-1/2 right-2 hidden -translate-y-1/2 rounded border bg-muted/60 px-1.5 font-sans text-[11px] text-muted-foreground lg:block">
          Ctrl F
        </kbd>
      )}
    </div>
  )
}
