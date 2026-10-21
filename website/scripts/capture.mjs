// 웹사이트에 넣을 앱 화면을 만든다. 실제 PC 정보가 아니라 **가상의 프로그램**을 가짜 응답으로 넣어 캡처한다.
//
//   (저장소 루트에서) pnpm exec vite --port 1420 --strictPort    # 앱 개발 서버 (pnpm tauri dev 중이면 생략)
//   node website/scripts/capture.mjs                              # → website/public/shots/*.webp
//
// Windows에서는 Edge를 쓴다. 다른 브라우저는 CHROMIUM_PATH로 지정한다.
import { mkdirSync, writeFileSync } from "node:fs"
import { dirname, join } from "node:path"
import { fileURLToPath } from "node:url"

if (!process.env.CHROMIUM_PATH && process.platform === "win32") {
  process.env.CHROMIUM_PATH = "C:\\Program Files (x86)\\Microsoft\\Edge\\Application\\msedge.exe"
}
const { launch, openApp } = await import("../../scripts/ui-mock/mock.mjs")

const out = join(dirname(fileURLToPath(import.meta.url)), "..", "public", "shots")
mkdirSync(out, { recursive: true })

// ── 가상의 프로그램과 아이콘 ─────────────────────────────────────────
const icon = (from, to, glyph) =>
  "data:image/svg+xml;utf8," +
  encodeURIComponent(
    `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 64 64"><defs><linearGradient id="g" x1="0" y1="0" x2="1" y2="1"><stop offset="0" stop-color="${from}"/><stop offset="1" stop-color="${to}"/></linearGradient></defs><rect width="64" height="64" rx="15" fill="url(#g)"/>${glyph}</svg>`,
  )
const g = {
  aperture: `<circle cx="32" cy="32" r="14" fill="none" stroke="#fff" stroke-width="5"/><circle cx="32" cy="32" r="5" fill="#fff"/>`,
  cloud: `<path d="M20 40h24a8 8 0 0 0 0-16 11 11 0 0 0-21-2 7 7 0 0 0-3 18z" fill="#fff"/>`,
  cube: `<path d="M32 14l15 8v20l-15 8-15-8V22z" fill="none" stroke="#fff" stroke-width="4" stroke-linejoin="round"/><path d="M17 22l15 8 15-8M32 30v20" fill="none" stroke="#fff" stroke-width="4"/>`,
  shield: `<path d="M32 13l15 6v11c0 10-7 17-15 21-8-4-15-11-15-21V19z" fill="#fff"/>`,
  note: `<path d="M27 42a6 6 0 1 1-2-4.5V18l18-4v20a6 6 0 1 1-2-4.5V20l-14 3z" fill="#fff"/>`,
  quill: `<path d="M44 14C30 18 22 30 20 46l4 2c3-8 8-15 16-20-6 1-10 4-13 8 3-10 9-17 17-22z" fill="#fff"/>`,
  orbit: `<circle cx="32" cy="32" r="7" fill="#fff"/><ellipse cx="32" cy="32" rx="19" ry="9" fill="none" stroke="#fff" stroke-width="3.5" transform="rotate(-30 32 32)"/>`,
  sync: `<path d="M20 30a12 12 0 0 1 21-7l3-3v10H34l4-4a8 8 0 0 0-14 4zM44 34a12 12 0 0 1-21 7l-3 3V34h10l-4 4a8 8 0 0 0 14-4z" fill="#fff"/>`,
  chat: `<path d="M16 22a6 6 0 0 1 6-6h20a6 6 0 0 1 6 6v12a6 6 0 0 1-6 6H28l-8 7v-7h2a6 6 0 0 1-6-6z" fill="#fff"/>`,
  pin: `<path d="M32 50s14-13 14-24a14 14 0 0 0-28 0c0 11 14 24 14 24z" fill="#fff"/><circle cx="32" cy="26" r="5" fill="#1e293b"/>`,
  code: `<path d="M25 22l-10 10 10 10M39 22l10 10-10 10M35 18l-6 28" fill="none" stroke="#fff" stroke-width="4.5" stroke-linecap="round" stroke-linejoin="round"/>`,
  leaf: `<path d="M46 16C26 16 18 28 18 40c0 4 1 7 3 9 2-12 9-19 19-23-9 6-14 13-16 23 14 0 22-12 22-33z" fill="#fff"/>`,
  type: `<path d="M20 18h24v6h-9v24h-6V24h-9z" fill="#fff"/>`,
  play: `<path d="M26 19l20 13-20 13z" fill="#fff"/>`,
}

const apps = [
  [
    "hklm64:aurora",
    "Aurora Photo Studio",
    "Aurora Labs",
    "msi",
    "machine64",
    3_350_000,
    "20260928",
    icon("#f472b6", "#7c3aed", g.aperture),
  ],
  [
    "hkcu:nimbus",
    "Nimbus Notes",
    "Nimbus Software",
    "exe",
    "user",
    182_000,
    "20261011",
    icon("#38bdf8", "#2563eb", g.cloud),
  ],
  [
    "hklm64:pixelforge",
    "Pixel Forge 3D",
    "Forge Interactive",
    "exe",
    "machine64",
    8_800_000,
    "20260704",
    icon("#fb923c", "#dc2626", g.cube),
  ],
  [
    "hklm64:harbor",
    "Harbor VPN",
    "Harbor Networks",
    "msi",
    "machine64",
    96_000,
    "20260512",
    icon("#34d399", "#047857", g.shield),
  ],
  ["hkcu:lumen", "Lumen Music", "Lumen Audio", "exe", "user", 431_000, "20261002", icon("#a78bfa", "#4f46e5", g.note)],
  [
    "hklm32:quill",
    "Quill Writer",
    "Quill & Co.",
    "exe",
    "machine32",
    265_000,
    "20260319",
    icon("#fbbf24", "#d97706", g.quill),
  ],
  [
    "hkcu:orbit",
    "Orbit Browser",
    "Orbit Foundation",
    "exe",
    "user",
    640_000,
    "20260915",
    icon("#22d3ee", "#0e7490", g.orbit),
  ],
  [
    "hklm64:tidal",
    "Tidal Sync",
    "Tidal Cloud",
    "exe",
    "machine64",
    143_000,
    "20260221",
    icon("#60a5fa", "#1d4ed8", g.sync),
  ],
  ["hkcu:kite", "Kite Chat", "Kite Inc.", "exe", "user", 312_000, "20261014", icon("#4ade80", "#15803d", g.chat)],
  [
    "hklm64:atlas",
    "Atlas Maps",
    "Atlas Geo",
    "msi",
    "machine64",
    1_150_000,
    "20251130",
    icon("#f87171", "#be123c", g.pin),
  ],
  ["hkcu:ember", "Ember Code", "Ember Dev", "exe", "user", 780_000, "20260930", icon("#94a3b8", "#334155", g.code)],
  [
    "hklm32:sprout",
    "Sprout Budget",
    "Sprout Finance",
    "exe",
    "machine32",
    71_000,
    "20250808",
    icon("#86efac", "#16a34a", g.leaf),
    "missing",
  ],
  [
    "hklm64:glyph",
    "Glyph Font Manager",
    "Glyph Type",
    "exe",
    "machine64",
    54_000,
    "20260117",
    icon("#e879f9", "#a21caf", g.type),
  ],
  [
    "hkcu:reel",
    "Reel Video Player",
    "Reel Media",
    "exe",
    "user",
    205_000,
    "20260826",
    icon("#f43f5e", "#9f1239", g.play),
  ],
].map(([id, name, publisher, technology, scope, kb, date, ic, uninstaller = "available"]) => ({
  id,
  displayName: name,
  publisher,
  version: ["2.4.1", "1.9.0", "5.0.3", "3.12.0"][name.length % 4],
  installLocation: `C:\\Program Files\\${publisher}\\${name}`,
  installDate: date,
  estimatedSizeKb: kb,
  displayIcon: "x",
  uninstallCommand: `"C:\\Program Files\\${publisher}\\${name}\\uninstall.exe"`,
  quietUninstallCommand: null,
  productCode: null,
  technology,
  scope,
  uninstaller,
  _icon: ic,
}))
const icons = Object.fromEntries(apps.map((a) => [a.id, a._icon]))
const appList = apps.map(({ _icon, ...a }) => a)

const removed = appList.find((a) => a.id === "hklm64:pixelforge")
const user = "C:\\Users\\me"
const leftovers = {
  appName: removed.displayName,
  protectedSkipped: 3,
  durationMs: 412,
  items: [
    // 엔진 기준과 같게: 등록 정보가 직접 가리키는 것만 "근거 강함", 이름만 맞으면 "검토"
    [
      "folder",
      "C:\\Program Files\\Forge Interactive\\Pixel Forge 3D",
      null,
      214_000_000,
      ["installLocation"],
      85,
      "strong",
    ],
    [
      "registryKey",
      "HKLM\\SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Uninstall\\PixelForge3D",
      null,
      null,
      ["uninstallEntry"],
      95,
      "strong",
    ],
    [
      "folder",
      `${user}\\AppData\\Roaming\\Forge Interactive\\Pixel Forge 3D`,
      null,
      38_400_000,
      ["publisherAndProduct"],
      80,
      "review",
    ],
    [
      "registryKey",
      "HKCU\\SOFTWARE\\Forge Interactive\\Pixel Forge 3D",
      null,
      null,
      ["publisherAndProduct"],
      80,
      "review",
    ],
    [
      "registryValue",
      "HKCU\\SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Run",
      "PixelForgeHelper",
      null,
      ["pointsIntoInstallLocation"],
      80,
      "strong",
      "Pixel Forge 3D 시작 시 자동 실행 항목이에요.",
    ],
    ["folder", `${user}\\AppData\\Local\\Pixel Forge 3D`, null, 1_260_000_000, ["exactName"], 55, "review"],
    [
      "folder",
      `${user}\\Documents\\Pixel Forge Projects`,
      null,
      92_000_000,
      ["partialName", { kind: "containsUserDocuments", count: 14 }],
      30,
      "review",
    ],
  ].map(([kind, location, valueName, sizeBytes, ev, score, confidence, note = null]) => ({
    id: location + (valueName ?? ""),
    kind,
    location,
    valueName,
    sizeBytes,
    evidence: ev.map((e) => (typeof e === "string" ? { kind: e } : e)),
    score,
    confidence,
    note,
  })),
}

const day = 86_400
const now = Math.floor(Date.now() / 1000)
const op = (id, name, ago, items) => ({
  id,
  appName: name,
  createdAt: now - ago,
  items: items.map(([kind, location, status, sizeBytes = null, valueName = null], i) => ({
    id: `${id}-${i}`,
    kind,
    location,
    valueName,
    sizeBytes,
    status,
    error: null,
  })),
})
const backups = [
  op("1790000000003-a", "Pixel Forge 3D", 3600, [
    ["folder", "C:\\Program Files\\Forge Interactive", "removed", 214_000_000],
    ["folder", `${user}\\AppData\\Roaming\\Forge Interactive\\Pixel Forge 3D`, "removed", 38_400_000],
    ["registryKey", "HKCU\\SOFTWARE\\Forge Interactive\\Pixel Forge 3D", "removed"],
    ["registryValue", "HKCU\\SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Run", "removed", null, "PixelForgeHelper"],
  ]),
  op("1790000000002-b", "Quill Writer", 2 * day, [
    ["folder", `${user}\\AppData\\Local\\Quill & Co`, "removed", 12_000_000],
    ["registryKey", "HKLM\\SOFTWARE\\WOW6432Node\\Quill & Co.", "restored"],
  ]),
  op("1790000000001-c", "시작프로그램 정리", 6 * day, [
    ["registryValue", "HKCU\\SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Run", "removed", null, "OldUpdater"],
  ]),
]

const startup = [
  ["run-user:1", "Kite Chat", "runUser", "Kite Inc.", true, false],
  ["run-user:2", "Nimbus Notes", "runUser", "Nimbus Software", true, false],
  ["run-machine:3", "Harbor VPN", "runMachine", "Harbor Networks", true, false, true],
  ["folder-user:4", "Lumen Music.lnk", "folderUser", "Lumen Audio", false, false],
  ["task:5", "Tidal Sync Updater", "task", "Tidal Cloud", true, false],
  ["run-user:6", "OldUpdater", "runUser", null, true, true],
  ["run-user:7", "Orbit Browser Helper", "runUser", "Orbit Foundation", false, false],
].map(([id, name, source, publisher, enabled, missing, needsAdmin = false]) => ({
  id,
  name,
  source,
  location: "HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\Run",
  approvedName: name,
  command: `"C:\\Program Files\\${publisher ?? "Old"}\\${name}.exe" --background`,
  targetPath: `C:\\Program Files\\${publisher ?? "Old"}\\${name}.exe`,
  targetMissing: missing,
  publisher,
  description: null,
  author: null,
  needsAdmin,
  enabled,
  disabledAt: enabled ? null : now - 3 * day,
  changedByUs: !enabled,
}))

// 이름·설명은 crates/storage/src/rules.rs의 실제 규칙과 같게 쓴다. 관리자 항목과 되돌릴 수 없는 항목은 미리 고르지 않는다.
const storage = [
  [
    "user-temp",
    "사용자 임시 파일",
    "프로그램이 설치·실행 중에 잠깐 쓰고 남긴 파일",
    "regenerable",
    false,
    true,
    2_840_000_000,
    18342,
  ],
  [
    "chrome-cache",
    "Chrome 캐시",
    "Chrome이 웹페이지를 빨리 열려고 저장한 파일",
    "regenerable",
    false,
    true,
    1_920_000_000,
    40211,
  ],
  [
    "shader-cache",
    "그래픽 셰이더 캐시",
    "NVIDIA·AMD·Intel·DirectX가 게임용으로 만들어 둔 캐시",
    "regenerable",
    false,
    true,
    412_000_000,
    1290,
  ],
  [
    "windows-temp",
    "Windows 임시 파일",
    "Windows와 서비스가 남긴 임시 파일",
    "regenerable",
    true,
    false,
    1_310_000_000,
    2210,
  ],
  [
    "recycle-bin",
    "휴지통",
    "이미 지운 파일이 모여 있는 곳 (모든 드라이브)",
    "irreversible",
    false,
    false,
    3_700_000_000,
    512,
  ],
].map(([id, name, description, risk, admin, defaultSelected, bytes, files]) => ({
  id,
  name,
  description,
  why: description,
  risk,
  kind: id === "recycle-bin" ? "recycleBin" : "files",
  admin,
  defaultSelected,
  bytes,
  files,
  inaccessible: 0,
  locations: [],
  running: [],
  guidance: null,
}))

const overrides = {
  list_installed_apps: appList,
  measure_app_sizes: {},
  get_app_icon: `(args) => (${JSON.stringify(icons)})[args.id] ?? null`,
  scan_leftovers: leftovers,
  recently_removed: [removed],
  list_backups: backups,
  backup_usage: { count: 3, bytes: 264_400_000, expired: 0 },
  list_startup_items: startup,
  scan_storage: storage,
  disk_usage: { drive: "C:\\", total: 1_000e9, free: 214e9 },
}

const browser = await launch()
// PNG보다 가벼운 WebP로 저장한다. 브라우저의 canvas 인코더를 쓰므로 따로 설치할 것이 없다.
// 글자가 많은 화면이라 품질은 높게 둔다. 2배로 찍어 <name>@2x.webp(2560×1600)와,
// 그것을 줄인 <name>.webp(1280×800)를 만든다 (사이트는 화면 배율에 따라 srcset으로 고른다).
const encoder = await browser.newPage()
const shot = async (name, page) => {
  const png = await page.screenshot()
  const [big, small] = await encoder.evaluate(
    async (src) => {
      const img = new Image()
      img.src = src
      await img.decode()
      const encode = (scale) => {
        const canvas = document.createElement("canvas")
        canvas.width = img.naturalWidth * scale
        canvas.height = img.naturalHeight * scale
        const ctx = canvas.getContext("2d")
        ctx.imageSmoothingQuality = "high"
        ctx.drawImage(img, 0, 0, canvas.width, canvas.height)
        return canvas.toDataURL("image/webp", 0.9).split(",")[1]
      }
      return [encode(1), encode(0.5)]
    },
    `data:image/png;base64,${png.toString("base64")}`,
  )
  writeFileSync(join(out, `${name}@2x.webp`), Buffer.from(big, "base64"))
  writeFileSync(join(out, `${name}.webp`), Buffer.from(small, "base64"))
  console.log("saved", `website/public/shots/${name}.webp, ${name}@2x.webp`)
}
const nav = async (page, name) => {
  await page.getByRole("button", { name, exact: true }).first().click()
  await page.waitForTimeout(1600)
}

// 사이트가 어두운 바탕이라 앱도 어두운 테마로 찍는다 (CAPTURE_SCHEME=light로 바꿀 수 있다)
const page = await openApp(browser, {
  overrides,
  width: 1280,
  height: 800,
  scale: 2,
  scheme: process.env.CAPTURE_SCHEME ?? "dark",
})
await page.waitForTimeout(2500)
await shot("programs", page)

await nav(page, "남은 흔적 정리")
await page
  .getByRole("button", { name: /남은 흔적 찾기/ })
  .first()
  .click()
await page.waitForTimeout(2200)
await shot("leftovers", page)
await page.keyboard.press("Escape")
await page.waitForTimeout(500)

await nav(page, "복구 기록")
await page.waitForTimeout(900)
await shot("restore", page)

await nav(page, "시작프로그램")
await page.waitForTimeout(1200)
await shot("startup", page)

await nav(page, "저장공간")
await page.waitForTimeout(1800)
await shot("storage", page)

await browser.close()
