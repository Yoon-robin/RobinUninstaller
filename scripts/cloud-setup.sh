#!/usr/bin/env bash
# 클라우드(리눅스) 세션에서 검사 도구를 준비한다. 여러 번 실행해도 된다.
#
# - Wine + mingw: scripts/wine-check.sh (Windows용 빌드를 Wine에서 테스트)
# - Rust 대상: x86_64-pc-windows-gnu(빌드·Wine), x86_64-pc-windows-msvc(clippy만)
# - pnpm 의존성: 화면 검사(tsc, prettier)와 scripts/ui-mock(가짜 응답 캡처)
set -euo pipefail
cd "$(dirname "$0")/.."

SUDO=""
[ "$(id -u)" -ne 0 ] && SUDO="sudo"

need=()
command -v wine >/dev/null || need+=(wine64)
command -v x86_64-w64-mingw32-gcc >/dev/null || need+=(gcc-mingw-w64-x86-64)
if [ ${#need[@]} -gt 0 ]; then
  echo "설치: ${need[*]}"
  $SUDO apt-get update -qq
  DEBIAN_FRONTEND=noninteractive $SUDO apt-get install -y -qq "${need[@]}"
fi

rustup target add x86_64-pc-windows-gnu x86_64-pc-windows-msvc >/dev/null

command -v pnpm >/dev/null || npm install -g pnpm@10 >/dev/null
pnpm install --frozen-lockfile

# Wine 환경을 미리 만들어 둔다 (첫 실행이 느리다)
export WINEDEBUG=-all WINEPREFIX="${WINEPREFIX:-$HOME/.wine-robin}"
[ -d "$WINEPREFIX" ] || wineboot -i >/dev/null 2>&1 || true

echo "준비 완료. 검사: bash scripts/wine-check.sh, 화면 캡처: scripts/ui-mock/README.md"
