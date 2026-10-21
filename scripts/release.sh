#!/usr/bin/env bash
# 설치 파일을 만들고 자동 업데이트용 서명과 latest.json을 준비한다. (Git Bash)
# 설치 파일은 RobinX가 직접 만든 것이다: robinx.exe를 setup 이름으로 복사하면 스스로 설치한다 (docs/INSTALLER.md).
#
#   bash scripts/release.sh            # 빌드 + 서명 + latest.json (올리지 않음)
#   bash scripts/release.sh --publish  # 위 작업 뒤 GitHub Release v<버전>을 만들어 올리고 사이트도 다시 배포한다
#
# 필요한 것:
# - 비밀 키: %USERPROFILE%\.tauri\robinx-updater.key (pnpm tauri signer generate로 만든 것. 저장소에 넣지 않는다)
#   잃어버리면 이미 설치된 앱에 업데이트를 보낼 수 없으니 따로 안전하게 보관한다.
# - 버전: package.json, src-tauri/Cargo.toml, src-tauri/tauri.conf.json 의 버전이 같아야 한다.
set -euo pipefail
cd "$(dirname "$0")/.."

PUBLISH=0
[ "${1:-}" = "--publish" ] && PUBLISH=1

KEY_FILE="$(cygpath -u "$USERPROFILE")/.tauri/robinx-updater.key"
if [ ! -f "$KEY_FILE" ]; then
  echo "비밀 키가 없어요: $KEY_FILE" >&2
  exit 1
fi

ver_conf=$(node -p "require('./src-tauri/tauri.conf.json').version")
ver_pkg=$(node -p "require('./package.json').version")
ver_cargo=$(grep -m1 '^version' src-tauri/Cargo.toml | sed -E 's/.*"(.*)".*/\1/')
if [ "$ver_cargo" = "" ] || grep -q '^version.workspace' src-tauri/Cargo.toml; then
  ver_cargo=$(grep -m1 '^version' Cargo.toml | sed -E 's/.*"(.*)".*/\1/')
fi
if [ "$ver_conf" != "$ver_pkg" ] || [ "$ver_conf" != "$ver_cargo" ]; then
  echo "버전이 서로 달라요: tauri.conf.json=$ver_conf package.json=$ver_pkg Cargo=$ver_cargo" >&2
  exit 1
fi
VERSION="$ver_conf"
TAG="v$VERSION"

# 올릴 때는 올린 커밋 그대로 빌드했는지 확인한다: 커밋하지 않은 변경이 없고, 지금 커밋이 GitHub에 올라가 있어야 한다.
# 태그는 이 커밋에 붙인다 (GitHub 기본 브랜치의 최신 커밋에 붙어 빌드한 코드와 어긋나지 않게).
COMMIT=$(git rev-parse HEAD)
if [ "$PUBLISH" = 1 ]; then
  git diff --quiet HEAD -- || { echo "커밋하지 않은 변경이 있어요. 커밋한 뒤 올려 주세요" >&2; exit 1; }
  git fetch -q origin
  git merge-base --is-ancestor "$COMMIT" "origin/$(git rev-parse --abbrev-ref HEAD)" \
    || { echo "지금 커밋($COMMIT)이 아직 GitHub에 없어요. push한 뒤 올려 주세요" >&2; exit 1; }
fi
echo "버전 $VERSION 빌드 (커밋 ${COMMIT:0:7})"

# NSIS는 쓰지 않는다. 앱만 빌드하고, 그 실행 파일이 곧 설치 파일이다.
pnpm tauri build --no-bundle

OUT_DIR="target/release/bundle"
mkdir -p "$OUT_DIR"
SETUP="$OUT_DIR/RobinX_${VERSION}_x64-setup.exe"
cp target/release/robinx.exe "$SETUP"

# 자동 업데이트 서명 (비밀 키는 화면에 내보내지 않는다)
TAURI_SIGNING_PRIVATE_KEY_PATH="$(cygpath -w "$KEY_FILE")" TAURI_SIGNING_PRIVATE_KEY_PASSWORD="" \
  pnpm tauri signer sign "$(cygpath -w "$SETUP")" >/dev/null
SIG="$SETUP.sig"
[ -f "$SIG" ] || { echo "서명 파일이 없어요: $SIG" >&2; exit 1; }
NAME=$(basename "$SETUP")
# GitHub는 파일 이름의 공백을 점으로 바꾼다
ASSET_NAME="${NAME// /.}"

OUT="target/release/bundle/latest.json"
node -e '
const fs = require("fs")
const [version, sigPath, asset, tag, out] = process.argv.slice(1)
const json = {
  version,
  // 노트가 없으면 비워 둔다 (업데이트 창이 제목만 되풀이하는 빈 상자를 그리지 않게)
  notes: process.env.RELEASE_NOTES || "",
  pub_date: new Date().toISOString(),
  platforms: {
    "windows-x86_64": {
      signature: fs.readFileSync(sigPath, "utf8").trim(),
      url: `https://github.com/Yoon-robin/RobinX/releases/download/${tag}/${asset}`,
    },
  },
}
fs.writeFileSync(out, JSON.stringify(json, null, 2))
' "$VERSION" "$SIG" "$ASSET_NAME" "$TAG" "$OUT"

SHA256=$(sha256sum "$SETUP" | cut -d' ' -f1)
SIZE=$(stat -c %s "$SETUP")

echo "준비됨:"
echo "  $SETUP"
echo "  $SIG"
echo "  $OUT"
echo "  SHA-256 $SHA256 ($SIZE 바이트)"

if [ "$PUBLISH" = 1 ]; then
  # GitHub 릴리스 글에는 받은 파일을 직접 확인할 수 있게 파일 이름·크기·SHA-256과 빌드한 커밋을 붙인다
  NOTES="${RELEASE_NOTES:-RobinX $VERSION}

---
설치 파일 확인
- 파일: $ASSET_NAME ($SIZE 바이트)
- SHA-256: \`$SHA256\`
- 빌드한 커밋: $COMMIT
- 확인 방법 (PowerShell): \`Get-FileHash .\\$ASSET_NAME -Algorithm SHA256\`
- 이 파일은 Windows 게시자 코드 서명이 아직 없어요. 위 SHA-256이 같은지 확인한 뒤 실행해 주세요."
  gh release create "$TAG" "$SETUP" "$OUT" --repo Yoon-robin/RobinX --target "$COMMIT" --title "RobinX $VERSION" --notes "$NOTES"
  echo "올림: https://github.com/Yoon-robin/RobinX/releases/tag/$TAG"

  # 올라간 파일을 다시 받아, 로컬에서 만든 것과 같은지 확인한다
  CHECK_DIR=$(mktemp -d)
  curl -fsSL -o "$CHECK_DIR/latest.json" "https://github.com/Yoon-robin/RobinX/releases/download/$TAG/latest.json"
  curl -fsSL -o "$CHECK_DIR/setup.exe" "https://github.com/Yoon-robin/RobinX/releases/download/$TAG/$ASSET_NAME"
  REMOTE_SHA=$(sha256sum "$CHECK_DIR/setup.exe" | cut -d' ' -f1)
  [ "$REMOTE_SHA" = "$SHA256" ] || { echo "올라간 설치 파일이 로컬 것과 달라요: $REMOTE_SHA" >&2; exit 1; }
  node -e '
const fs = require("fs")
const [remote, sigPath, version] = process.argv.slice(1)
const j = JSON.parse(fs.readFileSync(remote, "utf8"))
const p = j.platforms["windows-x86_64"]
if (j.version !== version) throw new Error(`latest.json 버전이 달라요: ${j.version}`)
if (p.signature.trim() !== fs.readFileSync(sigPath, "utf8").trim()) throw new Error("latest.json 서명이 달라요")
' "$CHECK_DIR/latest.json" "$SIG" "$VERSION"
  rm -rf "$CHECK_DIR"
  echo "확인함: 올라간 설치 파일·latest.json·서명이 로컬과 같아요"
  # 사이트의 받기 단추·버전·SHA-256은 배포할 때 최신 릴리스에서 읽으므로, 사이트도 다시 배포한다
  bash website/scripts/deploy.sh || echo "사이트 배포에 실패했어요. 나중에 bash website/scripts/deploy.sh 로 다시 올리세요" >&2
else
  echo "아직 올리지 않았어요. 올리려면: bash scripts/release.sh --publish"
fi
