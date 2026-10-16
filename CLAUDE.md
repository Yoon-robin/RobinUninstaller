# RobinUninstaller — Claude 작업 규칙

Windows용 언인스톨러 (Revo Uninstaller 같은 프로그램). Tauri 2 + React + shadcn/ui 화면, Rust 엔진.
전체 계획과 단계별 구현 메모는 [PLAN.md](PLAN.md), 새 PC 설정과 현재 진행 상황은 [docs/HANDOFF.md](docs/HANDOFF.md).

## 사용자와 일하는 방식

- **답변은 항상 한국어로.** 코드 주석·커밋 메시지는 지금처럼 쓰면 된다(주석 한국어, 커밋 메시지 영어).
- **도구 설치·환경 설정은 사용자에게 시키지 말고 직접 실행한다.** 사용자가 꼭 해야 하는 것(로그인, UAC "예" 누르기 등)만 부탁한다.
- 작업이 끝나면 이 PC에서 `pnpm check`로 검사하고, 앱을 실제로 띄워 확인한 뒤 커밋·push한다. GitHub CI는 쓰지 않는다(사용자 요청으로 제거함).

## 안전 원칙 (반드시 지킬 것)

- **사용자 PC의 실제 프로그램을 제거하거나 실제 잔여물을 지우는 테스트를 하지 않는다.** 제거·정리 기능은 `crates/fixture`의 가짜 앱 "Fixture Notes"(게시자 "Robin Test Labs")로만 시험하고, 끝나면 `robin-fixture remove`로 정리한다.
- 실제 프로그램 대상으로는 읽기 전용 확인만 한다 (`cargo run -p robin-scanner --example scan -- "이름"`).
- 영구 삭제 기능을 만들지 않는다. 정리는 항상 백업·격리(`crates/recovery`)를 거친다.
- `Revo Uninstaller Pro.zip`(상용 프로그램)과 `docs/private/`(분석 메모)는 **절대 커밋하지 않는다** (.gitignore에 있음). 리포지토리는 공개 상태다.
- 분석한 외부 프로그램의 코드·리소스를 가져오지 않는다. 아이디어만 참고한다.

## 자주 쓰는 명령

```bash
pnpm install
pnpm tauri dev                 # 개발 모드 실행
pnpm check                     # tsc + prettier + cargo fmt + clippy + cargo test
pnpm format                    # 프론트엔드 포맷
cargo run -p robin-fixture -- install [--cancel|--hang|--machine]   # 테스트 앱 (--machine은 관리자로)
cargo run -p robin-fixture -- remove
cargo run -p robin-execution --example uninstall_fixture
cargo run -p robin-scanner --example scan_fixture
cargo run -p robin-recovery --example clean_fixture
cargo run -p robin-startup --example toggle_fixture
```

앱 화면을 자동으로 확인할 때는 `WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS="--remote-debugging-port=9223"`를 주고 `pnpm tauri dev`로 띄운 뒤,
DevTools 프로토콜(`http://127.0.0.1:9223/json`)로 클릭·캡처한다. 마우스를 직접 움직이는 방식은 다른 창을 누를 수 있어 쓰지 않는다.

## 코드 구조

- `src/` React 화면 (`features/programs`, `features/leftovers`, `features/startup`, `features/restore`, `components/`)
- `src-tauri/` Tauri command 계층(얇게 유지) + `worker.rs`(관리자 권한 작업자 모드)
- `crates/core` 공통 모델 · `crates/discovery` 설치 목록·아이콘 · `crates/execution` 언인스톨러 실행
- `crates/scanner` 잔여물 탐지(읽기 전용) · `crates/recovery` 백업·정리·복원 · `crates/startup` 시작프로그램 · `crates/fixture` 테스트 앱

프론트엔드는 경로가 아니라 ID만 넘긴다. 경로·명령은 항상 Rust 쪽이 자기 데이터에서 찾는다.
