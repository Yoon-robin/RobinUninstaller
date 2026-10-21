// 성능 측정: 배포용 빌드(vite build + vite preview) + 가짜 응답 1000개. THROTTLE=4면 CPU를 4배 느리게. 상호작용마다 걸린 시간과 긴 작업(50ms+) 합계.
import { launch, openApp, sample } from "./mock.mjs"

const N = 1000
const apps = Array.from({ length: N }, (_, i) => ({
  ...sample.apps[i % sample.apps.length],
  id: `hkcu:app${i}`,
  displayName: `Program ${String(i).padStart(4, "0")} ${["Studio", "Player", "Notes", "Tools"][i % 4]}`,
  publisher: ["Acme", "Contoso", "Fabrikam", "Northwind"][i % 4],
  estimatedSizeKb: (i * 7919) % 5_000_000,
}))

const browser = await launch()
const page = await openApp(browser, {
  width: 1280,
  height: 860,
  overrides: { list_installed_apps: apps },
})
const cdp = await page.context().newCDPSession(page)
if (process.env.THROTTLE) await cdp.send("Emulation.setCPUThrottlingRate", { rate: Number(process.env.THROTTLE) })
await page.evaluate(() => {
  window.__long = []
  new PerformanceObserver((l) => {
    for (const e of l.getEntries()) window.__long.push({ start: e.startTime, dur: e.duration })
  }).observe({ type: "longtask", buffered: true })
})
await page.reload()
await page.evaluate(() => {
  window.__long = []
  new PerformanceObserver((l) => {
    for (const e of l.getEntries()) window.__long.push({ start: e.startTime, dur: e.duration })
  }).observe({ type: "longtask", buffered: true })
})
await page.waitForSelector("[data-app-row]", { timeout: 60000 })
const load = await page.evaluate(() => ({
  rowsAt: Math.round(performance.now()),
  long: window.__long.reduce((s, e) => s + e.dur, 0),
  maxLong: Math.max(0, ...window.__long.map((e) => e.dur)),
}))
console.log("load: rows visible at", load.rowsAt, "ms; long tasks", Math.round(load.long), "ms (max", Math.round(load.maxLong), ")")
await page.waitForTimeout(3000)

async function measure(name, act, done) {
  await page.evaluate(() => {
    window.__long = []
    window.__t0 = performance.now()
  })
  await act()
  await page.waitForFunction(done, null, { timeout: 30000 }).catch(async () => console.log(name, "TIMEOUT", await page.evaluate(() => [...document.querySelectorAll("h1")].map((h) => h.textContent + ":" + !!h.offsetParent).join(","))))
  const r = await page.evaluate(() => ({
    t: Math.round(performance.now() - window.__t0),
  }))
  await page.waitForTimeout(1200)
  const l = await page.evaluate(() => ({
    long: Math.round(window.__long.reduce((s, e) => s + e.dur, 0)),
    max: Math.round(Math.max(0, ...window.__long.map((e) => e.dur))),
  }))
  console.log(`${name}: ${r.t} ms until done; long tasks total ${l.long} ms (max ${l.max})`)
}

const h1 = (t) => `[...document.querySelectorAll("h1")].some((h) => h.offsetParent && h.textContent === ${JSON.stringify(t)})`
await measure("→ 저장공간", () => page.evaluate(() => [...document.querySelectorAll("[data-sidebar=menu-button]")].find((b) => b.textContent.trim().startsWith('저장공간')).click()), h1("저장공간"))
await measure("→ 복구 기록", () => page.evaluate(() => [...document.querySelectorAll("[data-sidebar=menu-button]")].find((b) => b.textContent.trim().startsWith('복구 기록')).click()), h1("복구 기록"))
await measure("→ 설정", () => page.evaluate(() => [...document.querySelectorAll("[data-sidebar=menu-button]")].find((b) => b.textContent.trim().startsWith('설정')).click()), h1("설정"))
await measure("→ 프로그램", () => page.evaluate(() => [...document.querySelectorAll("[data-sidebar=menu-button]")].find((b) => b.textContent.trim().startsWith('프로그램')).click()), h1("프로그램"))
const search = page.locator("main input:visible").first()
await measure(
  "검색 'Studio'",
  () => search.fill("Studio"),
  `document.querySelectorAll("[data-app-row]").length > 0 && [...document.querySelectorAll("[data-app-row]")].every((r) => r.textContent.includes("Studio"))`,
)
await measure("검색 지우기", () => search.fill(""), `[...document.querySelectorAll("[data-app-row]")].some((r) => r.textContent.includes("Notes"))`)
await measure(
  "자세히 보기 열기",
  () => page.evaluate(() => document.querySelectorAll("[data-app-row]")[2].click()),
  `!!document.querySelector("[role=dialog]")`,
)
await page.keyboard.press("Escape")
// 창을 닫은 뒤 4초 동안은 포커스를 잃으면 마지막 행으로 돌아가므로(스크롤도 따라감) 기다린다
await page.waitForTimeout(4500)
await measure(
  "카드 보기",
  () => page.evaluate(() => [...document.querySelectorAll("[role=radio]")].find((b) => /카드/.test(b.getAttribute("aria-label") ?? b.textContent)).click()),
  `!document.querySelector("[data-app-row]")`,
)
await measure(
  "목록 보기",
  () => page.evaluate(() => [...document.querySelectorAll("[role=radio]")].find((b) => /목록/.test(b.getAttribute("aria-label") ?? b.textContent)).click()),
  `!!document.querySelector("[data-app-row]")`,
)
await measure(
  "스크롤 맨 아래",
  () => page.evaluate(() => (document.querySelector("main").scrollTop = 1e9)),
  `[...document.querySelectorAll("[data-app-row]")].some((r) => r.textContent.includes("0999") || r.textContent.includes("0998"))`,
)
await browser.close()
