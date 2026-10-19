// 공유 미리 보기 이미지(1200×630)를 만든다: 로고 + 제목 + 앱 화면.
//   node website/scripts/og.mjs   → website/public/og.png
import { readFileSync } from "node:fs"
import { dirname, join } from "node:path"
import { fileURLToPath, pathToFileURL } from "node:url"

if (!process.env.CHROMIUM_PATH && process.platform === "win32") {
  process.env.CHROMIUM_PATH = "C:\Program Files (x86)\Microsoft\Edge\Application\msedge.exe"
}
const site = join(dirname(fileURLToPath(import.meta.url)), "..")
const { launch } = await import(pathToFileURL(join(site, "..", "scripts", "ui-mock", "mock.mjs")).href)

const data = (file, type) => `data:${type};base64,${readFileSync(join(site, "public", file)).toString("base64")}`
const logo = data("logo.svg", "image/svg+xml")
const shot = data("shots/programs.png", "image/png")
const font = pathToFileURL(
  join(site, "node_modules", "pretendard", "dist", "web", "variable", "woff2", "PretendardVariable.woff2"),
).href

const html = `<!doctype html><html><head><style>
@font-face { font-family: P; src: url("${font}") format("woff2"); font-weight: 100 900; }
* { margin: 0; box-sizing: border-box; }
body { width: 1200px; height: 630px; overflow: hidden; font-family: P, sans-serif; color: #f3f4f6;
  background: radial-gradient(60% 80% at 15% 0%, rgba(46,230,240,.18), transparent 60%),
              radial-gradient(50% 70% at 100% 100%, rgba(139,92,246,.22), transparent 60%), #111318; }
.wrap { position: absolute; left: 72px; top: 0; bottom: 0; width: 520px; display: flex; flex-direction: column; justify-content: center; }
.brand { display: flex; align-items: center; gap: 16px; font-size: 34px; font-weight: 800; letter-spacing: -0.02em; }
.brand img { width: 64px; height: 64px; }
h1 { margin-top: 36px; font-size: 64px; line-height: 1.12; font-weight: 800; letter-spacing: -0.03em; word-break: keep-all; }
h1 span { background: linear-gradient(100deg, #3cf0ef, #3aa0f5 50%, #9b5cf6); -webkit-background-clip: text; color: transparent; }
p { margin-top: 24px; font-size: 24px; line-height: 1.5; color: #a1a7b4; word-break: keep-all; }
.shot { position: absolute; left: 640px; top: 96px; width: 760px; border-radius: 20px; padding: 8px;
  background: rgba(36,39,48,.8); border: 1px solid rgba(255,255,255,.12); box-shadow: 0 30px 80px rgba(0,0,0,.6); }
.shot img { display: block; width: 100%; border-radius: 13px; }
</style></head><body>
<div class="wrap">
  <div class="brand"><img src="${logo}">RobinX</div>
  <h1>Windows를<br><span>가볍고 깔끔하게.</span></h1>
  <p>근거를 보고 고르는 제거·정리.<br>남은 흔적은 백업한 뒤 정리해요.</p>
</div>
<div class="shot"><img src="${shot}"></div>
</body></html>`

const browser = await launch()
const page = await browser.newPage({ viewport: { width: 1200, height: 630 } })
await page.setContent(html)
await page.waitForTimeout(800)
await page.screenshot({ path: join(site, "public", "og.png") })
await browser.close()
console.log("saved website/public/og.png")
