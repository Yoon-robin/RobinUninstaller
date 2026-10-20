import release from "./release.json"

export const REPO_URL = "https://github.com/Yoon-robin/RobinX"
/** 가장 최근 릴리스 페이지 (릴리스 노트, 파일별 SHA-256) */
export const RELEASES_URL = `${REPO_URL}/releases/latest`

/**
 * 최신 릴리스의 설치 파일. 빌드할 때 scripts/release-info.mjs가 GitHub에서 가져와 release.json에 적는다.
 * 그 값이 없으면 릴리스 페이지로 보낸다.
 */
export const RELEASE = {
  version: release.version as string,
  /** 설치 파일을 바로 받는 주소 */
  downloadUrl: (release.url as string | undefined) ?? RELEASES_URL,
  pageUrl: (release.pageUrl as string | undefined) ?? RELEASES_URL,
  file: release.file as string,
  sizeMb: Math.round(((release.size as number) / 1024 / 1024) * 10) / 10,
  sha256: release.sha256 as string | null,
  publishedAt: new Date(release.publishedAt as string),
}

export const LOGO = `${import.meta.env.BASE_URL}logo.png`
export const shot = (name: string) => `${import.meta.env.BASE_URL}shots/${name}.png`
/** 앱 녹화 (public/demo, scripts/ui-mock/record-demo.mjs로 만든다) */
export const demo = (file: string) => `${import.meta.env.BASE_URL}demo/${file}`
