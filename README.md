# RobinX

Windows용 프로그램 제거·정리 도구입니다. 프로그램을 지운 뒤 남은 파일과 레지스트리를 찾아, 항목마다 왜 그 프로그램 것으로 봤는지 근거를 보여 줍니다. 사용자가 고른 항목만 백업한 다음 정리하고, 백업이 남아 있는 동안에는 복구 기록에서 되돌릴 수 있습니다.

저장공간 정리와 시작프로그램 관리도 들어 있습니다. 캐시·임시 파일처럼 다시 만들어지는 것은 확인을 받은 뒤 바로 지우고, 휴지통 비우기처럼 되돌릴 수 없는 작업은 따로 경고합니다.

소개 페이지: https://yoon-robin.github.io/RobinX/
보안 문제는 공개 이슈 대신 [SECURITY.md](SECURITY.md)의 방법으로 알려 주세요.

> 첫 미리보기 버전(v0.1.0)을 [GitHub Releases](https://github.com/Yoon-robin/RobinX/releases/latest)에서 내려받을 수 있습니다. 기능은 테스트용 가짜 앱으로 검증했고, 실제 프로그램을 대상으로 한 검증은 진행 중입니다. 코드 서명 인증서가 없어 처음 실행할 때 SmartScreen 경고가 나올 수 있습니다. 계획은 [PLAN.md](PLAN.md), 다른 PC에서 개발을 이어 가는 방법은 [docs/HANDOFF.md](docs/HANDOFF.md)에 있습니다.

## 기능

- 설치된 프로그램 목록: EXE·MSI 프로그램과 Microsoft Store·MSIX 앱. 크기 기록이 없으면 설치 폴더를 직접 잽니다. Store 앱 제거는 Windows 설정으로 안내합니다. 목록을 CSV로 내보내거나 웹에서 검색할 수 있습니다.
- 제거: 프로그램의 기본 언인스톨러를 실행하고, 끝나면 남은 흔적을 찾습니다. 여러 개를 골라 차례로 지울 수 있습니다.
- 남은 흔적 분류: 근거가 강한 항목만 미리 고릅니다. 시스템 경로와 여러 프로그램이 함께 쓰는 폴더는 보호 규칙으로 뺍니다.
- 백업과 복원: 파일은 격리 폴더로 옮기고 레지스트리는 내보낸 뒤 정리합니다. 영구 삭제는 하지 않습니다.
- 강제 제거: 언인스톨러가 없거나 망가진 프로그램도 백업 후 정리합니다. 보안 프로그램과 드라이버는 대상에서 뺍니다.
- 시작프로그램: Run 키, 시작 폴더, 로그인·부팅 때 실행되는 예약 작업을 끄고 다시 켭니다. 등록 정보는 지우지 않습니다.
- 저장공간 정리: 다시 만들어지는 캐시·임시 파일만 정리하고, 큰 폴더를 찾아 보여 줍니다. 사용자 파일은 지우지 않습니다.
- 설치 추적: 설치 전후를 비교해 새로 생긴 파일과 레지스트리를 기록하고, 나중에 남은 흔적을 찾을 때 근거로 씁니다.

## 기술 스택

- [Tauri 2](https://tauri.app) + React + TypeScript
- [shadcn/ui](https://ui.shadcn.com) + Tailwind CSS
- Rust 엔진 (`windows` 크레이트)

## 지원 환경

Windows 10 / 11 (x64)

## 개발

필요한 것: Node 24 + pnpm, Rust(stable), Visual Studio C++ Build Tools, WebView2

```bash
pnpm install
pnpm tauri dev      # 개발 모드로 실행
pnpm tauri build    # 설치 파일(NSIS) 만들기
pnpm check          # 타입 검사 + fmt + clippy + 테스트 (커밋 전에 실행)
```

### 폴더 구조

- `src/`: React + shadcn/ui 화면
- `src-tauri/`: Tauri 앱 (엔진을 호출하는 얇은 command 계층)
- `crates/core`: 공통 모델
- `crates/discovery`: 설치된 프로그램 탐색 (레지스트리 + MSI, Store·MSIX 앱, 아이콘 추출, 읽기 전용)
- `crates/execution`: 기본 언인스톨러 실행, 자식 프로세스 추적, 제거 결과 확인
- `crates/scanner`: 남은 파일·레지스트리 찾기와 근거·점수 (읽기 전용)
- `crates/recovery`: 백업·격리 후 정리, 작업 기록, 복원 (영구 삭제 없음)
- `crates/startup`: 시작프로그램 목록과 끄기·원상복구 (Run 키, 시작 폴더, 예약 작업)
- `crates/storage`: 저장공간 정리(캐시·임시 파일)와 큰 폴더 분석
- `crates/settings`: 사용자 설정 (백업 보관 기간, 잔여물 스캔 제외 목록)
- `crates/tracker`: 설치 추적: 설치 전후 스냅샷을 비교해 새로 생긴 파일·레지스트리 기록 (읽기 전용)
- `crates/fixture`: 제거·잔여물 테스트용 가짜 앱 "Fixture Notes" (현재 사용자 영역에만 등록)

탐색 결과만 빠르게 보려면: `cargo run -p robin-discovery --example dump`

### 제거 기능 테스트

실제 프로그램을 지우지 말고 테스트 앱으로 확인하세요.

```bash
cargo run -p robin-fixture -- install            # 테스트 앱 등록 (취소 흉내: --cancel)
cargo run -p robin-execution --example uninstall_fixture
cargo run -p robin-scanner --example scan_fixture   # 제거 → 잔여물 스캔 → 기대 결과 확인
cargo run -p robin-recovery --example clean_fixture  # 제거 → 정리 → 복원 → 원본과 비교
cargo run -p robin-fixture -- remove             # 정리
```

## 배포와 자동 업데이트

앱은 GitHub Releases의 `latest.json`으로 새 버전을 확인하고, 사용자가 누를 때만 받아서 설치합니다(Tauri updater). 서명이 맞지 않는 파일은 설치하지 않습니다. 공개 키는 `src-tauri/tauri.conf.json`에, 비밀 키는 배포하는 사람의 PC(`%USERPROFILE%\.tauri\robinx-updater.key`)에만 있습니다.

1. `package.json`, `Cargo.toml`(workspace), `src-tauri/tauri.conf.json`의 버전을 올린다.
2. `bash scripts/release.sh` → 서명된 설치 파일(`target/release/bundle/nsis/`)과 `latest.json`을 만든다.
3. 확인한 뒤 `bash scripts/release.sh --publish` → GitHub Release `v<버전>`에 설치 파일과 `latest.json`을 올린다.

> 비밀 키를 잃어버리면 이미 설치된 앱에 업데이트를 보낼 수 없습니다. 안전한 곳에 따로 백업해 두세요.

## 라이선스

[Apache License 2.0](LICENSE). 함께 들어 있는 외부 코드·글꼴의 라이선스는 [NOTICE](NOTICE)에 있습니다.
