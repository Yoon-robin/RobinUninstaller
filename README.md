# RobinUninstaller

Windows용 언인스톨러. 프로그램을 제거한 뒤 남은 파일과 레지스트리를 찾아 **왜 그 프로그램 것인지 근거와 함께** 보여주고, 사용자가 고른 것만 백업 후 정리합니다. 지운 것은 언제든 되돌릴 수 있습니다.

> 개발 초기 단계입니다. 자세한 계획은 [PLAN.md](PLAN.md)를 보세요.

## 주요 기능 (계획)

- 설치된 프로그램 목록 (EXE / MSI / MSIX)
- 기본 언인스톨러 실행 후 잔여물 스캔
- 위험도 기반 잔여물 분류, 공유 파일·시스템 경로 보호
- 격리 보관과 레지스트리 백업, 원클릭 복원
- 강제 제거, 시작프로그램 관리, 설치 추적 (MVP 이후)

## 기술 스택

- [Tauri 2](https://tauri.app) + React + TypeScript
- [shadcn/ui](https://ui.shadcn.com) + Tailwind CSS
- Rust 엔진 (`windows` 크레이트), SQLite

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

- `src/` — React + shadcn/ui 화면
- `src-tauri/` — Tauri 앱 (엔진을 호출하는 얇은 command 계층)
- `crates/core` — 공통 모델
- `crates/discovery` — 설치된 프로그램 탐색 (레지스트리 + MSI, 아이콘 추출, 읽기 전용)
- `crates/execution` — 기본 언인스톨러 실행, 자식 프로세스 추적, 제거 결과 확인
- `crates/fixture` — 제거 테스트용 가짜 앱 (현재 사용자 영역에만 등록)

탐색 결과만 빠르게 보려면: `cargo run -p robin-discovery --example dump`

### 제거 기능 테스트

실제 프로그램을 지우지 말고 테스트 앱으로 확인하세요.

```bash
cargo run -p robin-fixture -- install            # 테스트 앱 등록 (취소 흉내: --cancel)
cargo run -p robin-execution --example uninstall_fixture
cargo run -p robin-fixture -- remove             # 정리
```
