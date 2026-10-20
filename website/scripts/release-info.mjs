// 최신 GitHub 릴리스의 설치 파일 정보를 src/lib/release.json에 적는다. 빌드 전에 실행한다 (deploy.sh).
// 사이트의 내려받기 단추가 설치 파일로 바로 이어지고, 버전·크기·SHA-256이 실제 릴리스와 맞게 하려는 것이다.
// 가져오지 못하면 있던 파일을 그대로 두고 끝낸다 (사이트는 그 값이나 릴리스 페이지 주소로 만들어진다).
//
//   node website/scripts/release-info.mjs
import { writeFileSync } from "node:fs"
import { dirname, join } from "node:path"
import { fileURLToPath } from "node:url"

const OUT = join(dirname(fileURLToPath(import.meta.url)), "..", "src", "lib", "release.json")
const API = "https://api.github.com/repos/Yoon-robin/RobinX/releases/latest"

try {
  const res = await fetch(API, { headers: { Accept: "application/vnd.github+json" } })
  if (!res.ok) throw new Error(`GitHub API ${res.status}`)
  const release = await res.json()
  const asset = release.assets.find((a) => /_x64-setup\.exe$/.test(a.name))
  if (!asset) throw new Error("설치 파일이 릴리스에 없어요")
  const sha256 = asset.digest?.startsWith("sha256:") ? asset.digest.slice("sha256:".length) : null
  const info = {
    version: release.tag_name.replace(/^v/, ""),
    publishedAt: release.published_at,
    pageUrl: release.html_url,
    file: asset.name,
    url: asset.browser_download_url,
    size: asset.size,
    sha256,
  }
  writeFileSync(OUT, JSON.stringify(info, null, 2) + "\n")
  console.log(`릴리스 정보: v${info.version} ${info.file} (${info.size} 바이트)`)
} catch (e) {
  console.warn(`릴리스 정보를 가져오지 못해 있던 값을 써요: ${e.message}`)
}
