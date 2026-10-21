import { createCn } from "cn/config"

/**
 * 클래스 합치기. 사이트가 따로 정의한 글자 크기(index.css의 `--text-*`)를 알려 줘야
 * 뒤에 오는 글자색(`text-muted-foreground` 등)과 같은 그룹으로 여겨 크기를 지우지 않는다.
 */
export const cn = createCn({
  extend: { classGroups: { "font-size": [{ text: ["fine", "body", "lead", "section", "display"] }] } },
})
