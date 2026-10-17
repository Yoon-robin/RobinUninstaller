#!/usr/bin/env bash
# Windows PC 없이 엔진 예제를 Wine에서 돌려 보는 검사 (리눅스·클라우드 세션용).
#
# - 대상은 테스트 앱 "Fixture Notes"뿐이고, 전용 Wine 환경(WINEPREFIX)에서만 돌아서 실제 PC에 영향이 없다.
# - 화면(Tauri/WebView2)과 UAC는 Wine에서 확인할 수 없다. 그 부분은 Windows PC에서 확인한다.
# - Wine의 작업 스케줄러는 목록·트리거 읽기가 구현돼 있지 않아 예약 작업 확인은 건너뛴다(ROBIN_SKIP_TASKS).
#
# 처음 한 번: sudo apt-get install -y wine64 gcc-mingw-w64-x86-64 && rustup target add x86_64-pc-windows-gnu
set -uo pipefail
cd "$(dirname "$0")/.."

export WINEDEBUG=-all
export WINEPREFIX="${WINEPREFIX:-$HOME/.wine-robin}"
export ROBIN_SKIP_TASKS=1
TARGET=x86_64-pc-windows-gnu
D="target/$TARGET/debug"

cargo build -q --target "$TARGET" --workspace --exclude robin-uninstaller --examples --bins || exit 1

# 유닛 테스트도 Windows용으로 빌드해 Wine에서 돌린다 (Windows 경로를 쓰는 테스트는 리눅스에서는 실패한다)
echo "유닛 테스트 (Wine)"
CARGO_TARGET_X86_64_PC_WINDOWS_GNU_RUNNER=wine \
  cargo test -q --target "$TARGET" --workspace --exclude robin-uninstaller --lib --bins 2>&1 \
  | grep -E "test result|FAILED|panicked" || exit 1

fixture() { wine "$D/robin-fixture.exe" "$@" >/dev/null 2>&1; }

# 예제 이름과, 그 전에 테스트 앱을 설치할 방법
CASES=(
  "robin-execution uninstall_fixture install"
  "robin-scanner scan_fixture install"
  "robin-recovery clean_fixture install"
  "robin-recovery force_fixture install --broken"
  "robin-startup toggle_fixture install"
  "robin-storage storage_clean_fixture install"
)

failed=0
for case in "${CASES[@]}"; do
  read -r crate example install_args <<<"$case"
  fixture remove
  fixture $install_args
  if out=$(timeout 300 wine "$D/examples/$example.exe" 2>&1); then
    echo "통과  $crate/$example"
  else
    echo "실패  $crate/$example"
    echo "$out" | grep -v '^ \+\(at \|[0-9]\+:\)' | tail -20 | sed 's/^/      /'
    failed=$((failed + 1))
  fi
done
fixture remove

if [ "$failed" -gt 0 ]; then
  echo "실패 $failed건"
  exit 1
fi
echo "모두 통과"
