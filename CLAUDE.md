# RobinX — Claude 작업 규칙

Windows 올인원 관리 도구 (예전 이름 RobinUninstaller). 언인스톨러(Revo Uninstaller 같은)에서 출발해 저장공간·시작프로그램·최적화까지 넓혀 간다. Tauri 2 + React + shadcn/ui 화면, Rust 엔진.
전체 계획과 단계별 구현 메모는 [PLAN.md](PLAN.md), 새 PC 설정과 현재 진행 상황은 [docs/HANDOFF.md](docs/HANDOFF.md).

## 사용자와 일하는 방식

- **답변은 항상 한국어로.** 코드 주석·커밋 메시지는 지금처럼 쓰면 된다(주석 한국어, 커밋 메시지 영어).
- **도구 설치·환경 설정은 사용자에게 시키지 말고 직접 실행한다.** 사용자가 꼭 해야 하는 것(로그인, UAC "예" 누르기 등)만 부탁한다.
- 작업이 끝나면 이 PC에서 `pnpm check`로 검사하고, 앱을 실제로 띄워 확인한 뒤 커밋·push한다. GitHub CI는 쓰지 않는다(사용자 요청으로 제거함).

## 안전 원칙 (반드시 지킬 것)

- **사용자 PC의 실제 프로그램을 제거하거나 실제 잔여물을 지우는 테스트를 하지 않는다.** 제거·정리 기능은 `crates/fixture`의 가짜 앱 "Fixture Notes"(게시자 "Robin Test Labs")로만 시험하고, 끝나면 `robin-fixture remove`로 정리한다.
- 시작프로그램 끄기·켜기도 실제 항목으로는 시험하지 않는다. 테스트 앱 항목(`FixtureNotes`, `Fixture Notes.lnk`)만 쓴다.
- 실제 프로그램 대상으로는 읽기 전용 확인만 한다 (`cargo run -p robin-scanner --example scan -- "이름"`).
- **삭제 원칙 (사용자와 정함, 2026-10-17):**
  - 프로그램 잔여물 정리는 항상 백업·격리(`crates/recovery`)를 거친다. 영구 삭제하지 않는다.
  - 저장공간 정리(`crates/storage`)에서 **다시 만들어지는 것**(캐시·임시 파일)만 사용자가 확인한 뒤 바로 지운다. 격리함은 같은 드라이브라 공간이 늘지 않기 때문이다.
  - **되돌릴 수 없는 것**(메모리 덤프, 휴지통, Windows.old 등)은 따로 경고하고 기본 선택하지 않는다. 사용자가 그 항목을 직접 골라 한 번 더 확인해야 지운다.
  - **사용자 파일**(문서, 다운로드, 바탕 화면, 프로젝트, 게임 데이터 등)은 절대 자동으로 지우지 않는다. 크기를 보여 주고 탐색기로 열어 주는 것까지만 한다.
  - 저장공간 정리도 실제 캐시로는 시험하지 않는다. 디버그 빌드에만 있는 테스트 항목(`Fixture Notes` 캐시)으로 시험한다.
- **업데이트 서명 비밀 키**(`%USERPROFILE%\.tauriobinx-updater.key`)는 저장소에 넣거나 출력·전송하지 않는다. 공개 키만 `tauri.conf.json`에 있다. 릴리스 올리기(`scripts/release.sh --publish`)는 공개 행동이라 사용자에게 먼저 묻는다.
- `Revo Uninstaller Pro.zip`(상용 프로그램)과 `docs/private/`(분석 메모)는 **절대 커밋하지 않는다** (.gitignore에 있음). 리포지토리는 공개 상태다.
- **최적화 원칙:** 바꾸는 설정은 모두 원래 값을 기록해 되돌릴 수 있어야 한다. 레지스트리 클리너·메모리 부스터처럼 효과가 없거나 위험한 기능은 만들지 않고, 보안(Defender·UAC·업데이트)을 끄는 기능도 만들지 않는다. 최적화도 테스트 앱 항목으로만 시험한다.
- 분석한 외부 프로그램의 코드·리소스를 가져오지 않는다. 아이디어만 참고한다.

## 자주 쓰는 명령

```bash
pnpm install
pnpm tauri dev                 # 개발 모드 실행
pnpm check                     # tsc + prettier + cargo fmt + clippy + cargo test
pnpm format                    # 프론트엔드 포맷
cargo run -p robin-fixture -- install [--cancel|--hang|--machine|--broken]   # 테스트 앱 (--machine은 관리자로, --broken은 언인스톨러 없음)
cargo run -p robin-fixture -- remove
cargo run -p robin-execution --example uninstall_fixture
cargo run -p robin-scanner --example scan_fixture
cargo run -p robin-scanner --example size_fixture     # 설치 폴더 크기 재기 (먼저 robin-fixture install)
cargo run -p robin-discovery --example store_apps     # Store·MSIX 앱 목록 (읽기 전용)
cargo run -p robin-recovery --example clean_fixture
cargo run -p robin-recovery --example force_fixture   # install --broken 뒤 강제 제거 → 복원
cargo run -p robin-recovery --example startup_cleanup_fixture   # 파일 없는 시작 항목 정리 → 복원
cargo run -p robin-recovery --example recovery_edge_fixture   # 백업·복원 경계 상황 (덮어쓰기 금지, 조작된 기록 거부 등)
cargo run -p robin-tracker --example track_fixture   # 테스트 앱 설치를 추적 (먼저 cargo build -p robin-fixture)
cargo run -p robin-startup --example list             # 시작 항목 보기 (읽기 전용)
cargo run -p robin-startup --example toggle_fixture   # 테스트 앱 시작 항목만 껐다 켜기
cargo run -p robin-storage --example storage_scan            # 정리할 수 있는 항목 보기 (읽기 전용)
cargo run -p robin-storage --example storage_clean_fixture   # 테스트 앱 캐시만 정리해 보기
cargo run -p robin-tweaks --example tweaks_fixture           # 최적화 적용·되돌리기를 테스트 항목으로만
bash scripts/release.sh [--publish]                         # 서명된 설치 파일 + latest.json (--publish는 GitHub Release에 올림)
```

Windows PC를 쓸 수 없을 때(클라우드 세션 등)는 먼저 `bash scripts/cloud-setup.sh`로 도구를 준비하고,
`scripts/wine-check.sh`로 유닛 테스트와 테스트 앱 예제를 Wine에서 돌린다. 화면은 `scripts/ui-mock/`(가짜 응답 캡처)로 확인한다.
전용 Wine 환경이라 실제 PC에 영향이 없다. 화면·UAC·작업 스케줄러 목록은 Wine에서 확인되지 않으므로 Windows PC에서 따로 확인한다.

앱 화면을 자동으로 확인할 때는 `WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS="--remote-debugging-port=9223"`를 주고 `pnpm tauri dev`로 띄운 뒤,
DevTools 프로토콜(`http://127.0.0.1:9223/json`)로 클릭·캡처한다. 마우스를 직접 움직이는 방식은 다른 창을 누를 수 있어 쓰지 않는다.

## 코드 구조

- `src/` React 화면 (`features/programs`, `features/leftovers`, `features/storage`, `features/startup`, `features/tracking`, `features/settings`, `features/restore`, `components/`)
- `src-tauri/` Tauri command 계층(얇게 유지) + `worker.rs`(관리자 권한 작업자 모드)
- `crates/core` 공통 모델 · `crates/discovery` 설치 목록(Store 앱 포함)·아이콘 · `crates/execution` 언인스톨러 실행
- `crates/scanner` 잔여물 탐지(읽기 전용) · `crates/recovery` 백업·정리·복원 · `crates/fixture` 테스트 앱
- `crates/startup` 시작프로그램 목록·끄기·원상복구 (작업 관리자와 같은 `StartupApproved` 값만 바꾼다)
- `crates/storage` 저장공간 정리(규칙은 `rules.rs`, 사용자 폴더를 넣으면 테스트가 실패한다)와 폴더 크기 분석(읽기 전용)
- `crates/tracker` 설치 추적: 설치 전후 스냅샷 비교(읽기 전용), 기록은 잔여물 스캔의 근거로 쓴다
- `crates/settings` 사용자 설정(백업 보관 기간, 잔여물 스캔 제외 목록)
- `crates/tweaks` 최적화: 항목은 `catalog.rs`에 데이터로 정의(HKCU 공식 설정만, 정책·보안 키 금지는 테스트가 확인). 원래 값을 먼저 기록하고 바꾸며, 되돌릴 때 다른 곳에서 바뀐 값은 덮어쓰지 않는다. 실제 항목은 켜고 끄며 시험하지 않고 `test.fixture`(디버그 전용)로만 시험한다

프론트엔드는 경로가 아니라 ID만 넘긴다. 경로·명령은 항상 Rust 쪽이 자기 데이터에서 찾는다.
