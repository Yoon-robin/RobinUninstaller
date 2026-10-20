// 공유 미리 보기 이미지(1200×630)를 만든다: 로고 + 제목 + 앱 화면.
//   node website/scripts/og.mjs   → website/public/og.png
import { readFileSync } from "node:fs"
import { dirname, join } from "node:path"
import { fileURLToPath, pathToFileURL } from "node:url"

if (!process.env.CHROMIUM_PATH && process.platform === "win32") {
  process.env.CHROMIUM_PATH = String.raw`C:\Program Files (x86)\Microsoft\Edge\Application\msedge.exe`
}
const site = join(dirname(fileURLToPath(import.meta.url)), "..")
const { launch } = await import(pathToFileURL(join(site, "..", "scripts", "ui-mock", "mock.mjs")).href)

const data = (file, type) => `data:${type};base64,${readFileSync(join(site, "public", file)).toString("base64")}`
const logo = data("logo.svg", "image/svg+xml")
const shot = data("shots/leftovers.png", "image/png")
// 빈 페이지에서는 file:// 글꼴을 읽지 못하므로 data URI로 넣는다
const font = `data:font/woff2;base64,${readFileSync(
  join(site, "node_modules", "pretendard", "dist", "web", "variable", "woff2", "PretendardVariable.woff2"),
).toString("base64")}`

// 사이트와 같은 어두운 구성: 남색 바탕 위 파란 빛, 제목(강조 단어는 밝은 파랑), 오른쪽은 창 틀 안의 "남은 흔적" 화면.
const html = `<!doctype html><html><head><style>
@font-face { font-family: P; src: url("${font}") format("woff2"); font-weight: 100 900; }
* { margin: 0; box-sizing: border-box; }
body { width: 1200px; height: 630px; overflow: hidden; font-family: P, sans-serif; color: #f3f5fa; background: radial-gradient(60% 70% at 25% 0%, rgba(59,110,230,.38), transparent 70%), #0b1020; word-break: keep-all; }
.wrap { position: absolute; left: 72px; top: 0; bottom: 0; width: 520px; display: flex; flex-direction: column; justify-content: center; }
.brand { display: flex; align-items: center; gap: 14px; font-size: 30px; font-weight: 700; letter-spacing: -0.02em; }
.brand img { width: 48px; height: 48px; }
h1 em { font-style: normal; color: #7cc8f2; }
h1 { margin-top: 44px; font-size: 88px; line-height: 1.04; font-weight: 700; letter-spacing: -0.035em; }
p { margin-top: 32px; font-size: 27px; line-height: 1.45; color: #b6bdcc; }
.stage { position: absolute; left: 640px; top: 72px; right: 0; bottom: 0; background: #151b2b; border: 1px solid rgba(255,255,255,.12); border-right: 0; border-bottom: 0; border-top-left-radius: 28px; overflow: hidden; box-shadow: 0 0 120px rgba(59,110,230,.35); }
.crop { position: absolute; left: 48px; top: 48px; width: 640px; height: 600px; overflow: hidden; border-top-left-radius: 14px; }
.crop img { position: absolute; width: 1036px; left: -208px; top: -39px; }
</style></head><body>
<div class="wrap">
  <div class="brand"><img src="${logo}">RobinX</div>
  <h1><em>남은 흔적</em>까지<br>정리해요.</h1>
  <p>지운 뒤 남은 파일과 레지스트리를 찾아<br>백업하고 정리하는 무료 Windows 앱</p>
</div>
<div class="stage"><div class="crop"><img src="${shot}"></div></div>
</body></html>`

const browser = await launch()
const page = await browser.newPage({ viewport: { width: 1200, height: 630 } })
await page.setContent(html)
// 폰트와 그림이 다 그려진 뒤에 찍는다
await page.evaluate(async () => {
  await document.fonts.ready
  await Promise.all([...document.images].map((img) => img.decode().catch(() => {})))
})
await page.screenshot({ path: join(site, "public", "og.png") })
await browser.close()
console.log("saved website/public/og.png")
