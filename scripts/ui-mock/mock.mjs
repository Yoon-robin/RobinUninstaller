// 앱 화면을 가짜 Tauri 응답으로 띄워 캡처하는 도구 (Windows 없이 화면 확인용, 클라우드 세션 등).
// 실제 엔진은 부르지 않는다. 화면이 부르는 command에 이 파일의 가짜 데이터를 돌려준다.
//
//   pnpm exec vite --port 1420 --strictPort &      # 개발 서버
//   node scripts/ui-mock/shoot.mjs                  # 모든 화면 캡처 → scripts/ui-mock/out/
//
// 새 화면을 시험할 때는 openApp(browser, { overrides: { 명령: 값 또는 "(args) => 값" 문자열 } })로 응답을 바꾼다.
import { chromium } from "playwright-core"

const app = (id, name, publisher, extra = {}) => ({
  id,
  displayName: name,
  publisher,
  version: "1.0.0",
  installLocation: null,
  installDate: "20260101",
  estimatedSizeKb: 50000,
  displayIcon: null,
  uninstallCommand: "uninstall.exe",
  quietUninstallCommand: null,
  productCode: null,
  technology: "exe",
  scope: "user",
  uninstaller: "available",
  ...extra,
})

export const sample = {
  apps: [
    app("hkcu:robinfixtureapp", "Fixture Notes", "Robin Test Labs", { estimatedSizeKb: null }),
    app("hklm64:acme", "Acme Notes", "Acme", { scope: "machine64", technology: "msi", estimatedSizeKb: 1_400_000 }),
    app("hklm32:contoso", "Contoso Player", "Contoso", { scope: "machine32", installDate: "20251203" }),
    app("hkcu:oldtool", "Old Tool", "Nobody", { uninstaller: "missing" }),
  ],
  sizes: { "hkcu:robinfixtureapp": 35806 },
  store: [
    ["Microsoft.WindowsCalculator_8wekyb3d8bbwe", "계산기", "Microsoft Corporation", "store", "2026-03-04"],
    ["Microsoft.WindowsTerminal_8wekyb3d8bbwe", "터미널", "Microsoft Corporation", "store", "2026-08-11"],
    ["Contoso.LineApp_abc", "Contoso Line of Business", "Contoso, Inc.", "sideloaded", "2025-12-01"],
  ].map(([fam, name, publisher, source, date]) => ({
    id: "msix:" + fam.toLowerCase(),
    displayName: name,
    publisher,
    version: "11.2409.0.0",
    installDate: date,
    installLocation: null,
    familyName: fam,
    source,
  })),
  startup: [
    {
      id: "run-user:1",
      name: "FixtureNotes",
      source: "runUser",
      location: "HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\Run",
      approvedName: "FixtureNotes",
      command: '"C:\\Users\\me\\AppData\\Local\\Robin Test Labs\\Fixture Notes\\notes.exe" --tray',
      targetPath: "C:\\Users\\me\\AppData\\Local\\Robin Test Labs\\Fixture Notes\\notes.exe",
      targetMissing: false,
      publisher: "Robin Test Labs",
      description: null,
      author: null,
      needsAdmin: false,
      enabled: true,
      disabledAt: null,
      changedByUs: false,
    },
    {
      id: "folder-user:2",
      // 실제 엔진은 시작 폴더 바로 가기의 확장자를 뺀 이름을 보낸다 (approvedName은 파일 이름 그대로)
      name: "Robin Ghost Tool",
      source: "folderUser",
      location: "C:\\Users\\me\\AppData\\Roaming\\Microsoft\\Windows\\Start Menu\\Programs\\Startup",
      approvedName: "Robin Ghost Tool.lnk",
      command: "C:\\Program Files\\Ghost\\ghost.exe",
      targetPath: "C:\\Program Files\\Ghost\\ghost.exe",
      targetMissing: true,
      publisher: null,
      description: null,
      author: null,
      needsAdmin: false,
      enabled: true,
      disabledAt: null,
      changedByUs: false,
    },
  ],
  settings: { backupRetentionDays: null, retentionChoices: [30, 90, 180], exclusions: [] },
}

/** command 이름 → 응답. 함수면 (args) => 응답. 없으면 list*는 [], 나머지는 null. */
function defaultHandlers() {
  return {
    // 설치 프로그램 화면이 아니라 앱 화면으로 연다 (main.tsx가 가장 먼저 묻는다)
    app_mode: "app",
    list_installed_apps: sample.apps,
    measure_app_sizes: sample.sizes,
    list_store_apps: sample.store,
    list_startup_items: sample.startup,
    get_settings: sample.settings,
    backup_usage: { count: 3, bytes: 52_428_800, expired: 0 },
    disk_usage: { drive: "C:\\", total: 512e9, free: 128e9 },
  }
}

/**
 * 가짜 응답을 넣은 새 페이지를 연다.
 * overrides 값은 JSON으로 넘어가므로, 함수가 필요하면 "(args) => ..." 문자열로 준다.
 */
// scale: 화면 배율 (사이트용 그림은 고해상도 화면에서도 선명하게 2로 찍는다)
export async function openApp(
  browser,
  { scheme = "dark", overrides = {}, width = 1280, height = 860, scale = 1 } = {},
) {
  const page = await browser.newPage({ viewport: { width, height }, colorScheme: scheme, deviceScaleFactor: scale })
  return prepare(page, scheme, overrides)
}

/** openApp과 같지만 이미 만든 컨텍스트(녹화 설정 등)에서 연다. */
export async function openAppInContext(context, { scheme = "dark", overrides = {} } = {}) {
  return prepare(await context.newPage(), scheme, overrides)
}

async function prepare(page, scheme, overrides) {
  const handlers = { ...defaultHandlers(), ...overrides }
  const table = Object.entries(handlers)
    .map(([cmd, v]) =>
      typeof v === "string" && v.trim().startsWith("(")
        ? `${JSON.stringify(cmd)}: ${v}`
        : `${JSON.stringify(cmd)}: () => (${JSON.stringify(v)})`,
    )
    .join(",\n")
  // 앱은 저장된 테마(기본 어두운 테마)를 따르므로, 요청한 테마를 미리 저장해 둔다.
  const init = `
    try { localStorage.setItem("robin.theme", ${JSON.stringify(scheme)}) } catch {}
    const handlers = { ${table} };
    window.__TAURI_INTERNALS__ = {
      metadata: { currentWindow: { label: "main" }, currentWebview: { windowLabel: "main", label: "main" } },
      transformCallback: (cb) => { const id = Math.random(); window["_" + id] = cb; return id },
      invoke: async (cmd, args) => {
        if (cmd in handlers) return structuredClone(await handlers[cmd](args));
        if (cmd.startsWith("list") || cmd === "recently_removed") return [];
        return null;
      },
    };`
  page.on("pageerror", (e) => {
    if (!e.message.includes("unregisterListener")) console.log("pageerror", e.message)
  })
  await page.addInitScript(init)
  await page.goto(process.env.UI_MOCK_URL ?? "http://127.0.0.1:1420/")
  await page.waitForTimeout(1200)
  return page
}

/** 클라우드 컨테이너에는 /opt/pw-browsers에 Chromium이 있다. 다른 곳은 CHROMIUM_PATH로 지정한다. */
export function launch() {
  const executablePath = process.env.CHROMIUM_PATH ?? "/opt/pw-browsers/chromium-1194/chrome-linux/chrome"
  return chromium.launch({ executablePath })
}
