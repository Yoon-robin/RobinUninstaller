// 모든 화면을 가짜 응답으로 캡처한다: node scripts/ui-mock/shoot.mjs [화면 이름...]
import { mkdirSync } from "node:fs"
import { dirname, join } from "node:path"
import { fileURLToPath } from "node:url"

import { launch, openApp } from "./mock.mjs"

const out = join(dirname(fileURLToPath(import.meta.url)), "out")
mkdirSync(out, { recursive: true })

// [파일 이름, 사이드바 이름, 추가 동작]
const pages = [
  ["programs", "프로그램"],
  ["store-apps", "프로그램", (p) => p.getByRole("tab", { name: /Store 앱/ }).click()],
  ["leftovers", "남은 흔적 정리"],
  ["storage", "저장공간"],
  ["startup", "시작프로그램"],
  ["restore", "복구 기록"],
  ["settings", "설정"],
]
const only = process.argv.slice(2)

const browser = await launch()
const page = await openApp(browser)
for (const [file, nav, action] of pages) {
  if (only.length && !only.includes(file)) continue
  await page.getByRole("button", { name: nav, exact: true }).first().click()
  await page.waitForTimeout(700)
  if (action) {
    await action(page)
    await page.waitForTimeout(700)
  }
  await page.screenshot({ path: join(out, `${file}.png`) })
  console.log("saved", join("scripts/ui-mock/out", `${file}.png`))
}
await browser.close()
