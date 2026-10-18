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

### Phase 5 시작프로그램 관리 — 완료 (2026-10-17)

- Run 키(HKCU, HKLM 64·32비트)와 시작 폴더(사용자·공용) 목록, 게시자·설명·아이콘, "파일 없음" 표시.
- 끄기·다시 켜기는 작업 관리자와 같은 `StartupApproved` 값만 바꾼다. RobinUninstaller가 끈 항목은 원래 값을 기록해 두었다가 그대로 되돌린다.
- 테스트 앱 항목으로 엔진 예제와 앱 화면 모두에서 끄기 → 원상복구(값 없음)를 확인했다. 테스트 흔적은 정리했다.
- 관리자 권한 항목도 실제로 시험했다(2026-10-17): `robin-fixture install --machine`으로 만든 `FixtureNotesMachine`(HKLM Run)을 일반 권한 앱에서 끄기·다시 켜기 → 두 번 모두 UAC 후 성공, 원래 상태(값 없음)로 돌아옴. 테스트 흔적은 관리자 권한으로 정리했다.
- 로그인·부팅 때 실행되는 예약 작업(`\Microsoft\` 아래 제외)도 같은 목록에 나오고 "예약 작업" 필터가 있다. 끄기는 작업의 "사용" 여부만 바꾸고 원래 값을 기록해 되돌린다.
  - 테스트 앱 `install`이 사용자 로그온 작업 `\Fixture Notes Updater`를 만들고 `remove`가 지운다. 엔진 예제와 앱 화면에서 끄기 → 다시 켜기(작업 "준비", 기록 비워짐)를 확인했다(2026-10-17).
  - `install --machine`은 SYSTEM 부팅 작업 `\Fixture Notes Machine Task`(없는 파일 실행)도 만든다. 이 작업의 UAC 경로는 아직 시험하지 않았다.
- UAC 창에서 "아니요"를 누르는 경우는 아직 화면에서 확인하지 않았다 (코드상 "관리자 권한 요청이 거절됐어요" 알림).

### 새 PC에서 겪은 것 (2026-10-17)

- Git for Windows 기본값(`core.autocrlf=true`) 때문에 CRLF로 받아져 prettier 검사가 실패했다. `.gitattributes`로 LF를 고정했다.
- pnpm은 Git Bash에서 실행되지 않을 수 있다. PowerShell에서 실행한다.

### 저장공간 정리 — 1차 완료 (2026-10-17)

- 삭제 원칙을 사용자와 정했다 (CLAUDE.md "삭제 원칙"): 다시 만들어지는 캐시·임시 파일만 확인 후 바로 삭제, 되돌릴 수 없는 것은 따로 경고하고 기본 선택 안 함, 사용자 파일은 절대 자동 삭제 안 함.
- 화면: 드라이브 용량 막대(정리 후 예상), 정리 항목 목록(위험도·관리자·실행 중 표시, "왜 지워도 되나요?"), 직접 정리할 큰 항목 안내(WSL, Windows.old), 큰 폴더 찾기(C: 전체 약 3.5초, 읽기 전용).
- 테스트 앱 캐시(디버그 빌드 전용 항목)로 앱 화면에서 정리를 확인했다. 실제 캐시는 지우지 않았다. 되돌릴 수 없는 항목(휴지통)은 확인 창의 경고까지만 확인했다.
- **아직 안 함:** 관리자 항목(Windows 임시 파일, 메모리 덤프)의 UAC 정리 실제 시험(테스트 항목이 없음), 정리 기록, Windows Update 캐시·DISM·개발 산출물(node_modules/target) 찾기, 트리맵.

### Phase 5 강제 제거 — 작성 (2026-10-17, 클라우드 세션, 브랜치 `claude/force-uninstall`)

- 언인스톨러 없는 프로그램에 "강제 제거"를 켰다. 자세한 내용은 PLAN.md "Phase 5 구현 메모 — 강제 제거".
- 클라우드(리눅스)에서 확인한 범위: Windows 대상 `cargo clippy`, 유닛 테스트, `tsc`·prettier, 가짜 백엔드 응답으로 화면 캡처
  (실행 중 안내 → 다시 확인 → 스캔 결과, VPN 이름 프로그램 차단).
- **Windows PC에서 아직 확인하지 않은 것:**
  1. `pnpm check`
  2. `robin-fixture install --broken` → `cargo run -p robin-recovery --example force_fixture` → `robin-fixture remove`
     (Wine에서는 통과함. 실제 Windows에서 한 번 더)
  3. `install --broken` 뒤 앱에서 Fixture Notes "강제 제거" → 목록에서 사라짐 → 잔여물 정리 화면의 "제거한 프로그램"에 보임 → 복구 기록에서 되돌리면 다시 목록에 보임
  4. 실행 중 안내: 설치 폴더의 프로그램을 실행할 수 없으니(가짜 exe), 확인하려면 테스트 앱에 실제로 도는 실행 파일을 두는 방법을 찾아야 한다
- 확인이 끝나면 `main`에 합친다.

### 클라우드에서 Wine으로 검사하기 (2026-10-17)

- `scripts/wine-check.sh`: Windows용(x86_64-pc-windows-gnu)으로 빌드해 Wine에서 유닛 테스트 전체와 테스트 앱 예제 6개
  (uninstall, scan, clean, force, toggle, storage_clean)를 돌린다. 2026-10-17 기준 모두 통과.
- Wine에서 안 되는 것: 앱 화면(WebView2), UAC, 작업 스케줄러 목록·트리거 읽기(`E_NOTIMPL`) → 예약 작업 확인은 건너뛴다.
- 테스트 앱이 바로 가기·예약 작업을 PowerShell 대신 COM으로 직접 만든다. Wine의 PowerShell은 아무것도 안 하고 성공을 돌려줘서
  흔적이 조용히 빠지는 문제가 있었다. Windows PC에서 `robin-fixture install`/`install --machine`이 여전히 잘 되는지 확인할 것.

### 파일 없는 시작 항목 정리 — 작성 (2026-10-17, 클라우드, 브랜치 `claude/force-uninstall`)

- 시작프로그램 화면에 "파일 없는 항목 정리" 버튼과 확인 창을 추가했다. 백업 후 정리, 복구 기록에서 되돌리기.
- Wine에서 `startup_cleanup_fixture` 통과, 화면은 가짜 응답으로 캡처 확인. **Windows에서 확인할 것:** 앱에서 테스트 앱의
  `RobinGhostTool`·`Robin Ghost Tool` 정리 → 복구 기록에서 되돌리기. 실제 프로그램의 시작 항목은 정리하지 않는다.

### Phase 6 설치 추적 1차 — 작성 (2026-10-17, 클라우드, 브랜치 `claude/force-uninstall`)

- 설치 추적 화면(추적 시작 → 설치 끝남 → 항목 고르고 저장, 기록 목록)과 `crates/tracker`, 잔여물 스캔의 `TrackedInstall` 근거.
  자세한 내용은 PLAN.md "Phase 6 구현 메모".
- Wine에서 `track_fixture` 통과(스냅샷 3초 이내), 화면은 가짜 응답으로 캡처 확인.
- **Windows에서 확인할 것:** 실제 PC에서 스냅샷 시간(Program Files·AppData가 크면 오래 걸릴 수 있음), 추적 중 다른 프로그램 변경이
  얼마나 섞이는지, 앱에서 테스트 앱 설치를 추적 → 저장 → 테스트 앱 제거 후 잔여물 스캔에 "설치 추적 기록에 있음" 근거가 보이는지.

### 코드 리뷰 수정 — 작성 (2026-10-17, 클라우드, 브랜치 `claude/force-uninstall`)

- 리뷰에서 나온 버그를 고쳤다 (PLAN.md "코드 리뷰 수정"). Wine 검사 모두 통과, 화면 변경은 tsc·prettier로 확인.
- **Windows에서 꼭 확인할 것 (UAC 경로가 바뀜):**
  1. `install --machine` 후 HKLM·Program Files 항목 "관리자 권한으로 정리" → 복구 기록 "관리자 권한으로 복원"(이제 고른 항목만, `--items`)
  2. 관리자 작업자 허용 목록 때문에 정상 항목이 "Windows 설정 영역이라…"로 막히지 않는지 (예: 설치 추적 기록의 `HKLM\SOFTWARE\Classes` 항목은 막히는 게 정상)
  3. 저장공간 "Windows 임시 파일"(관리자) 정리가 여전히 되는지 (핸들 기반 삭제로 바뀜)
  4. 언인스톨러 대기: 실제 NSIS·Inno 언인스톨러가 아니라 테스트 앱으로 `uninstall_fixture`
  5. 1번에서 관리자 권한으로 정리한 Program Files 백업을 복구 기록에서 "영구 삭제" → UAC 후 지워지는지 (`--elevated-delete`)

### 설정 화면 — 작성 (2026-10-17, 클라우드, 브랜치 `claude/settings`)

- 테마, 백업 보관 기간·사용량·지금 정리, 잔여물 스캔 제외 목록 (PLAN.md "설정 화면").
- Wine에서 유닛 테스트(설정 저장·제외 판정·기간 계산) 통과, 화면은 가짜 응답으로 캡처 확인.
- **Windows에서 확인할 것:** 테스트 앱 잔여물에서 "다음부터 제외" → 다시 찾으면 빠지는지 → 설정에서 빼면 다시 보이는지.
  보관 기간은 테스트 백업으로만 시험한다(실제 백업이 지워질 수 있음).

### 여러 개 한꺼번에 제거 — 작성 (2026-10-17, 클라우드, 브랜치 `claude/batch-uninstall`)

- 화면만 바뀜(PLAN.md "여러 개 한꺼번에 제거"). 가짜 응답으로 고르기·확인·진행·결과 화면 캡처 확인.
- **Windows에서 확인할 것:** 테스트 앱 항목은 하나뿐이라 실제 PC에서는 "Fixture Notes" 하나만 골라 시험한다(실제 프로그램은 고르지 않는다).
  `install` → 제거됨 + 흔적 찾기, `install --cancel` → "아직 설치돼 있어요", `install --broken` → 확인 창에서 빠지는지. 끝나면 `robin-fixture remove`.

### Microsoft Store·MSIX 앱 목록 — 작성 (2026-10-17, 클라우드, 브랜치 `claude/store-apps`)

- 읽기 전용 목록과 "Windows 설정에서 제거" 안내(PLAN.md "Microsoft Store·MSIX 앱 목록"). 순수 함수 테스트는 Linux·Wine에서 통과.
  Wine에는 Store 패키지가 없어 빈 목록이 나오는 것(오류 없음)까지만 확인했다. 화면은 가짜 응답으로 캡처 확인.
- **Windows에서 확인할 것 (읽기 전용):** `cargo run -p robin-discovery --example store_apps`로 계산기·터미널 같은 앱이 이름·게시자·날짜와 함께 나오는지,
  Windows 구성 요소(ShellExperienceHost 등)와 프레임워크(VCLibs 등)가 빠졌는지, 아이콘 개수가 대부분 잡히는지. 앱 화면의 "Store 앱" 탭과 설정 열기 버튼.

### 설치 폴더 크기 직접 재기 — 작성 (2026-10-17, 클라우드, 브랜치 `claude/install-sizes`)

- 크기 기록이 없는 프로그램만 설치 폴더를 재서 보여 준다(PLAN.md "설치 폴더 크기 직접 재기"). Wine에서 `size_fixture` 통과, 화면은 가짜 응답으로 캡처 확인.
- **Windows에서 확인할 것 (읽기 전용):** `robin-fixture install` → `cargo run -p robin-scanner --example size_fixture` → `robin-fixture remove`.
  출력되는 실제 대상 목록에 이상한 위치(너무 넓은 폴더)가 없는지, 앱 화면에서 `*` 표시된 크기가 탐색기 속성의 크기와 비슷한지.

### 목록 내보내기·웹 검색 — 작성 (2026-10-18, 클라우드, 브랜치 `claude/export-list`)

- PLAN.md "목록 내보내기와 웹 검색". CSV 만들기·주소 만들기는 유닛 테스트 통과(Linux·Wine), 화면은 가짜 응답으로 캡처 확인.
- **Windows에서 확인할 것:** 내보내기 버튼 → 저장 창이 뜨는지, 저장한 CSV를 Excel로 열었을 때 한글·열이 맞는지. 메뉴의 "웹에서 검색"이 기본 브라우저로 열리는지.

### 다음 단계 후보

- Phase 5 나머지: SYSTEM 예약 작업의 UAC 끄기·켜기 시험
- 시스템 복원 지점, 서비스 정리(중지·삭제)
- Store 앱 직접 제거: 테스트용 MSIX(자체 서명, 개발자 모드)를 fixture에 먼저 만든다
- Phase 6 다음: 설치 추적을 ETW로 (커널 드라이버는 쓰지 않음)
- 배포: 코드 서명, NSIS 설치 파일(`pnpm tauri build`)
