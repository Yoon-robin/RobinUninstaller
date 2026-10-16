# 다른 PC에서 이어서 작업하기

## 1. 새 PC 준비 (Windows 10/11 x64)

필요한 것:

| 도구 | 버전 | 설치 |
|---|---|---|
| Git | 최신 | `winget install Git.Git` |
| Node.js | 24 | `winget install OpenJS.NodeJS` |
| pnpm | 10 | `npm install -g pnpm` |
| Rust | stable (1.99+) | https://rustup.rs 의 `rustup-init.exe` 실행 |
| Visual Studio Build Tools | 2022, "C++를 사용한 데스크톱 개발" | `winget install Microsoft.VisualStudio.2022.BuildTools` 후 C++ 워크로드 선택 |
| WebView2 | Windows 11에는 기본 포함 | |
| GitHub CLI (선택) | | `winget install GitHub.cli` 후 `gh auth login` |

```bash
git clone https://github.com/Yoon-robin/RobinUninstaller.git
cd RobinUninstaller
pnpm install
pnpm check        # 모두 통과해야 정상
pnpm tauri dev    # 앱 실행
```

첫 Rust 빌드는 몇 분 걸린다.

## 2. 리포지토리에 없는 것

- `Revo Uninstaller Pro.zip`, `docs/private/revo-analysis.md`: 상용 프로그램과 그 분석 메모라 공개 리포지토리에 올리지 않았다. 필요하면 원래 PC에서 직접 옮긴다. 분석 결과 중 설계에 반영할 내용은 PLAN.md의 Phase 3·4 메모에 이미 녹아 있다.
- 백업 저장소(`%LOCALAPPDATA%\RobinUninstaller\backups`): PC마다 따로 생긴다.

## 3. 현재 상태 (2026-10-16)

완료: Phase 0~4 (PLAN.md 로드맵 참고)

- 프로그램 목록(레지스트리·MSI, 아이콘, 세부 정보, 필터·검색·카드 보기)
- 기본 언인스톨러 실행 + 자식 프로세스 추적 + 제거 확인
- 잔여물 탐지(근거·점수·보호 규칙, 읽기 전용)
- 백업·격리 후 정리, 복구 기록, 복원, 관리자 권한 작업자 모드

### 관리자 권한(UAC) 정리 실제 시험 — 완료 (2026-10-16)

`robin-fixture install --machine`(관리자로 실행)으로 `HKLM\SOFTWARE\Robin Test Labs`와
`C:\Program Files\Robin Test Labs`를 만들어 앱에서 정리·복원까지 확인했다.

- 일반 권한 정리: HKCU·AppData 항목 4개 정리, HKLM·Program Files 2개는 "관리자 권한 필요" (기대대로)
- 관리자 권한 정리(UAC): 처음엔 HKLM 키 삭제가 "접근 거부"로 실패했다.
  - 원인: `HKLM\SOFTWARE`는 레지스트리 하이브의 루트라 관리자여도 DELETE 권한을 주지 않는데, 부모 키를 DELETE 포함 권한(KEY_ALL_ACCESS)으로 열고 있었다.
  - 수정: 부모는 읽기 권한으로만 열고, DELETE는 지울 키에만 요청한다. 실패 시 단계와 Windows 오류 코드를 기록하고, 하위 키부터 하나씩 지우는 대체 경로를 둔다.
- 수정 후: HKLM 키·Program Files 폴더 정리 성공 → 관리자 권한 복원 성공(값 `Theme=dark`, DWORD `Level=7` 원본과 같음) → 테스트 흔적 전부 정리.
- 이 PC에 남은 테스트 흔적과 테스트 백업은 없다.

### Phase 5 1차: 시작프로그램 화면 (2026-10-16, 클라우드 세션에서 작성)

- `crates/startup` + 시작프로그램 화면을 만들었다. 자세한 내용은 PLAN.md "Phase 5 구현 메모".
- 클라우드(리눅스)에서 만들어서 확인한 범위: Windows 대상 `cargo clippy`(x86_64-pc-windows-msvc), `cargo test`,
  `tsc`·prettier, 가짜 백엔드 응답으로 화면 캡처.
- **Windows PC에서 아직 확인하지 않은 것** (다음에 이 PC에서 할 일):
  1. `pnpm check`
  2. `robin-fixture install` → `cargo run -p robin-startup --example toggle_fixture` → `robin-fixture remove`
  3. `pnpm tauri dev`로 시작프로그램 화면을 띄워 실제 목록·아이콘·게시자가 맞게 보이는지, 작업 관리자 "시작 앱"의 켜짐·꺼짐과 같은지
  4. 관리자 권한 경로: 관리자로 `robin-fixture install --machine` → 앱에서 `FixtureNotesMachine`(모든 사용자)을 끄고 켜기(UAC "예")
     → 관리자로 `robin-fixture remove`. 실제 프로그램의 시작 항목은 끄고 켜지 않는다.

### 다음 단계 후보

- Phase 5 나머지: 강제 제거(언인스톨러 없는 프로그램), 예약 작업 목록
- 시스템 복원 지점, 서비스 정리(중지·삭제), 오래된 백업 자동 정리
- Phase 6: 설치 추적(스냅샷 비교 → ETW, 커널 드라이버는 쓰지 않음)
- 배포: 코드 서명, NSIS 설치 파일(`pnpm tauri build`)
