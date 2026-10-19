#!/usr/bin/env bash
# 설치 파일을 만들고 자동 업데이트용 서명과 latest.json을 준비한다. (Git Bash)
# 설치 파일은 RobinX가 직접 만든 것이다: robinx.exe를 setup 이름으로 복사하면 스스로 설치한다 (docs/INSTALLER.md).
#
#   bash scripts/release.sh            # 빌드 + 서명 + latest.json (올리지 않음)
#   bash scripts/release.sh --publish  # 위 작업 뒤 GitHub Release v<버전>을 만들어 올린다
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
echo "버전 $VERSION 빌드"

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
  notes: process.env.RELEASE_NOTES || `RobinX ${version}`,
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

echo "준비됨:"
echo "  $SETUP"
echo "  $SIG"
echo "  $OUT"

if [ "$PUBLISH" = 1 ]; then
  gh release create "$TAG" "$SETUP" "$OUT" --repo Yoon-robin/RobinX --title "RobinX $VERSION" --notes "${RELEASE_NOTES:-RobinX $VERSION}"
  echo "올림: https://github.com/Yoon-robin/RobinX/releases/tag/$TAG"
else
  echo "아직 올리지 않았어요. 올리려면: bash scripts/release.sh --publish"
fi
