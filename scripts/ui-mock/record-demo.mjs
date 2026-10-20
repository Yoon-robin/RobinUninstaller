// 사이트용 짧은 녹화: 방금 제거한 테스트 앱 → 남은 흔적 찾기 → 결과 → 백업하고 정리 → 완료.
// 실제 엔진을 부르지 않고 가짜 데이터(테스트 앱 Fixture Notes, C:\Users\me)만 보여 준다. 실제 PC의 목록은 나오지 않는다.
//
//   pnpm exec vite --port 1420 --strictPort &                 # 개발 서버
//   node node_modules/playwright-core/cli.js install ffmpeg   # 처음 한 번 (녹화에 필요)
//   CHROMIUM_PATH=<chrome.exe> node scripts/ui-mock/record-demo.mjs
//
// 결과: website/public/demo/leftovers.webm (소리 없음, VP9) + leftovers-poster.jpg (결과 화면)
// 앞부분(시작 화면, 프로그램 화면)은 Playwright가 받은 ffmpeg로 잘라 낸다.
import { execFileSync } from "node:child_process"
import { mkdirSync, readdirSync, rmSync } from "node:fs"
import { dirname, join } from "node:path"
import { fileURLToPath } from "node:url"

import { launch, openAppInContext, sample } from "./mock.mjs"

const root = join(dirname(fileURLToPath(import.meta.url)), "..", "..")
const out = join(root, "website", "public", "demo")
const tmp = join(dirname(fileURLToPath(import.meta.url)), "out", "video")
mkdirSync(out, { recursive: true })
rmSync(tmp, { recursive: true, force: true })
mkdirSync(tmp, { recursive: true })

const W = 1280
const H = 800
const me = String.raw`C:\Users\me\AppData`
const fixture = { ...sample.apps[0], installLocation: String.raw`${me}\Local\Robin Test Labs\Fixture Notes` }

const leftover = (id, kind, location, sizeBytes, confidence, evidence, valueName = null) => ({
  id,
  kind,
  location,
  valueName,
  sizeBytes,
  evidence,
  score: confidence === "strong" ? 90 : 40,
  confidence,
  note: null,
})
const report = {
  appName: "Fixture Notes",
  items: [
    leftover("a", "folder", String.raw`${me}\Local\Robin Test Labs\Fixture Notes`, 2_400_000, "strong", [
      { kind: "installLocation" },
    ]),
    leftover("b", "folder", String.raw`${me}\Roaming\Robin Test Labs\Fixture Notes`, 18_000, "strong", [
      { kind: "publisherAndProduct" },
    ]),
    leftover("c", "folder", String.raw`${me}\Local\Fixture Notes`, 31_500_000, "strong", [{ kind: "exactName" }]),
    leftover("d", "registryKey", String.raw`HKCU\Software\Robin Test Labs\Fixture Notes`, null, "strong", [
      { kind: "publisherAndProduct" },
    ]),
    leftover(
      "e",
      "registryValue",
      String.raw`HKCU\Software\Microsoft\Windows\CurrentVersion\Run`,
      null,
      "strong",
      [{ kind: "pointsIntoInstallLocation" }],
      "FixtureNotes",
    ),
    leftover("f", "folder", String.raw`${me}\Roaming\Fixture Notes 2019`, 4_000, "review", [{ kind: "partialName" }]),
  ],
  protectedSkipped: 2,
  durationMs: 640,
}
// 페이지 안에서 돌므로 바깥 변수를 쓰지 않게 항목을 문자열에 넣는다
const operation = `(ids) => ({
  id: "demo-op",
  appName: "Fixture Notes",
  createdAt: 1792400000,
  items: ${JSON.stringify(report.items)}
    .filter((i) => ids.includes(i.id))
    .map((i) => ({ ...i, status: "removed", error: null })),
})`

const handlers = {
  recently_removed: [fixture],
  get_settings: { ...sample.settings, backupRetentionDays: 30 },
  scan_leftovers: `() => new Promise((r) => setTimeout(() => r(${JSON.stringify(report)}), 1600))`,
  clean_leftovers: `(args) => new Promise((r) => setTimeout(() => r((${operation})(args.itemIds)), 1300))`,
}

// openApp과 같지만 녹화하는 컨텍스트에서 연다
const browser = await launch()
const context = await browser.newContext({
  viewport: { width: W, height: H },
  colorScheme: "dark",
  recordVideo: { dir: tmp, size: { width: W, height: H } },
})
// 녹화는 페이지를 만들 때 시작한다. 잔여물 화면이 뜬 시점부터 남기려고 시간을 잰다.
const started = Date.now()
const page = await openAppInContext(context, { overrides: handlers })
const pause = (ms) => page.waitForTimeout(ms)

await page.getByRole("button", { name: "잔여물 정리", exact: true }).first().click()
await pause(1400)
const trimAt = Math.max(0, (Date.now() - started) / 1000 - 0.8)
await page.getByRole("button", { name: "남은 흔적 찾기" }).first().click()
await pause(1600 + 2600)
await page.screenshot({ path: join(out, "leftovers-poster.jpg"), quality: 82, type: "jpeg" })
await page.getByRole("button", { name: /선택 항목 정리/ }).click()
await pause(1700)
await page.getByRole("button", { name: "백업하고 정리" }).click()
await pause(1300 + 3800)

const video = page.video()
await context.close()
await browser.close()
const file = await video.path()
const ffmpegDir = readdirSync(join(process.env.LOCALAPPDATA, "ms-playwright"))
  .filter((d) => d.startsWith("ffmpeg-"))
  .sort()
  .pop()
const ffmpeg = join(process.env.LOCALAPPDATA, "ms-playwright", ffmpegDir, "ffmpeg-win64.exe")
execFileSync(ffmpeg, [
  "-y",
  "-loglevel",
  "error",
  "-ss",
  trimAt.toFixed(2),
  "-i",
  file,
  "-an",
  "-c:v",
  "libvpx-vp9",
  "-b:v",
  "0",
  "-crf",
  "36",
  "-row-mt",
  "1",
  join(out, "leftovers.webm"),
])
console.log(`잘라 낸 앞부분 ${trimAt.toFixed(1)}초`)
console.log("saved website/public/demo/leftovers.webm, leftovers-poster.jpg", readdirSync(out))
