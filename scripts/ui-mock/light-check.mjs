// 밝은 테마와 메뉴(유리 효과)를 가짜 응답으로 캡처해 디자인을 확인한다.
//   node scripts/ui-mock/light-check.mjs <출력 폴더>
import { join } from "node:path"

if (!process.env.CHROMIUM_PATH && process.platform === "win32") {
  process.env.CHROMIUM_PATH = "C:\\Program Files (x86)\\Microsoft\\Edge\\Application\\msedge.exe"
}
const { launch, openApp } = await import("./mock.mjs")
const out = process.argv[2] ?? "."

const browser = await launch()
const page = await openApp(browser, { scheme: "light", width: 1280, height: 800 })
await page.waitForTimeout(2500)
await page.screenshot({ path: join(out, "light.png") })

// 첫 프로그램 행에서 오른쪽 클릭 메뉴
const row = page.getByText("Acme Notes", { exact: true }).first()
await row.click({ button: "right" })
await page.waitForTimeout(600)
await page.screenshot({ path: join(out, "light-menu.png") })
await browser.close()
console.log("saved", out)
