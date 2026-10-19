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
git clone https://github.com/Yoon-robin/RobinX.git
cd RobinX
pnpm install
pnpm check        # 모두 통과해야 정상
pnpm tauri dev    # 앱 실행
```

첫 Rust 빌드는 몇 분 걸린다.

## 2. 리포지토리에 없는 것

- `Revo Uninstaller Pro.zip`, `docs/private/revo-analysis.md`: 상용 프로그램과 그 분석 메모라 공개 리포지토리에 올리지 않았다. 필요하면 원래 PC에서 직접 옮긴다. 분석 결과 중 설계에 반영할 내용은 PLAN.md의 Phase 3·4 메모에 이미 녹아 있다.
- 백업 저장소(`%LOCALAPPDATA%\RobinX\backups`): PC마다 따로 생긴다.

## 3. 지금 상태 (2026-10-19) — 새 세션은 여기부터

모든 작업은 `main`에 들어 있다. 원격의 `claude/*` 브랜치는 GitHub에서 지워도 된다 (클라우드 세션에서는 원격 브랜치 삭제가 막혀 있다).
`claude/phase5-startup`만 `main`의 조상이 아닌데, 시작프로그램 1차 작업의 옛 사본이고 내용은 `main`에 다른 커밋으로 들어가 있다.

### 만든 기능

- 프로그램 목록: 레지스트리·MSI, 아이콘, 세부 정보, 필터·검색·카드 보기, 크기 기록이 없으면 설치 폴더를 직접 잼, CSV 내보내기, 웹에서 검색
- Microsoft Store·MSIX 앱 목록 (읽기 전용, 제거는 Windows 설정으로 안내)
- 제거: 기본 언인스톨러 실행·자식 프로세스 추적·제거 확인, 여러 개 차례로 제거, 언인스톨러 없는 프로그램 강제 제거
- 잔여물: 근거·점수·보호 규칙으로 찾기(읽기 전용) → 백업·격리 후 정리 → 복구 기록에서 복원, 관리자 권한 작업자 모드(UAC)
- 시작프로그램: Run 키·시작 폴더·예약 작업 끄기·원상복구, 파일 없는 항목 정리
- 저장공간: 다시 만들어지는 캐시만 정리, 큰 폴더 찾기
- 설치 추적: 설치 전후 스냅샷 비교 기록 → 잔여물 스캔의 근거
- 설정: 테마, 투명 효과(자동/줄이기), 백업 보관 기간, 잔여물 스캔 제외 목록
- 최적화: HKCU 공식 설정 24개(개인정보·작업 표시줄·추천 줄이기·편의·게임, 그중 선택지 항목 4개), 원래 값 기록 후 적용·되돌리기 (`crates/tweaks`)

### Windows PC 확인 (2026-10-19)

**확인 끝남:**
- `pnpm check` 통과(테스트 93개), `pnpm tauri dev`로 7개 화면 모두 오류 없이 뜸
- 테스트 앱 예제 10개 모두 통과: uninstall · scan(11) · clean(9) · force(10) · startup_cleanup(12) · recovery_edge(10) · toggle · storage_clean · size · track(13)
- 앱 화면에서 테스트 앱으로:
  - 강제 제거 → 목록에서 사라짐 → 잔여물 정리 화면에 보임 → 복구 기록에서 되돌리면 다시 보임
  - 파일 없는 시작 항목 정리 → 복원
  - 여러 개 제거(테스트 앱 하나만) → 결과 화면
  - 잔여물 "다음부터 제외" → 다시 찾으면 빠짐 → 설정에서 빼면 다시 보임
  - 설치 추적: 시작 3초, 끝남 2초, 테스트 앱 항목 11개만 잡힘(다른 프로그램 섞이지 않음) → 잔여물 근거 "설치 추적 기록에 있음" 11개
  - 백업 영구 삭제(일반 권한), 추적 기록 지우기
- 읽기 전용: `store_apps` 출력 정상(40개, 아이콘 35개, 구성 요소·프레임워크 빠짐). `size_fixture`의 실제 대상 목록에 너무 넓은 폴더 없음
- 고친 것: Store 목록에 앱이 아닌 기반 패키지(`WinAppRuntime.*`, `Winget.Source`, `StorePurchaseApp`)가 보이던 것

**주의할 점 (이번에 겪음):**
- `cargo build -p robin-fixture --examples`는 테스트 앱 **실행 파일을 만들지 않는다**. 반드시 `cargo build -p robin-fixture`. 옛 실행 파일로 돌리면 새 흔적(바로 가기·유령 항목)이 안 생겨 예제가 실패한다.
- "파일 없는 시작 항목 정리" 창은 실제 PC의 파일 없는 항목(예: Authenticator, ITCS Connect)도 기본 선택한다. 시험할 때는 테스트 앱 항목(`Robin Ghost Tool`, `RobinGhostTool`)만 남기고 선택을 푼다.
- 이 PC의 Git Bash에는 pnpm 경로(`%APPDATA%
pm`)가 PATH에 없을 때가 있다: `export PATH="$(cygpath -u "$APPDATA")/npm:$PATH"`

**UAC 경로 확인 (2026-10-19, 사용자가 UAC 허용):**
- 관리자 권한 정리: HKLM 키·Program Files 폴더 → 7개 모두 정리
- 고른 항목만 관리자 권한 복원(`--items`): HKLM 키만 복원(값 `Theme=dark`, `Level=7` 그대로), 나머지는 보관 중으로 남음
- 백업 영구 삭제(`--elevated-delete`): 관리자 권한으로 격리한 Program Files 폴더가 든 백업 → 삭제됨
- SYSTEM 예약 작업 끄기·켜기: UAC 후 `Disabled` ↔ `Ready` 확인. 일반 권한으로는 "Access is denied"라 UAC 경로가 실제로 쓰였다.
- **알게 된 것:** 관리자가 기본 권한(SDDL 없음)으로 등록한 SYSTEM 작업은 일반 사용자에게 **보이지 않아** 시작프로그램 목록에 안 나온다.
  테스트 앱은 실제 프로그램 다수처럼 로그인 사용자 읽기(`(A;;FR;;;AU)`)를 주도록 바꿨다. 이런 작업까지 보려면 "관리자 권한으로 모든 작업 보기" 같은 기능이 필요하다(다음 작업 후보).

**안전·보안 강화 뒤 UAC 다시 확인 (2026-10-19, 사용자가 UAC 허용):**
- 작업자 저장소 검사를 "문자열 끝"에서 "`사용자 폴더\계정\AppData\Local\RobinX\<sub>` 그대로 + 경로에 링크 없음"으로 바꾼 뒤 확인했다.
- 테스트 앱 `install --machine` → 앱에서 제거 → 남은 흔적: HKLM·Program Files 항목은 이름만 맞아 이제 "검토"(미리 선택 안 됨) → 직접 골라 정리
  → 일반 권한 3개 정리 + "관리자 권한으로 정리"(UAC) 2개 정리 → HKLM 키·Program Files 폴더 없어짐 확인
- 복구 기록 "모두 되돌리기"(일반 3개) → "관리자 권한으로 복원"(UAC) → HKLM 키·Program Files 폴더 돌아옴, 복원 5개
- 조작한 저장소 경로로 작업자 직접 실행(관리자): 문서 폴더 아래·다른 드라이브·`..` 포함·다른 하위 폴더 → **4개 모두 종료 코드 2로 거부**, 실제 백업은 그대로
- 정리: `robin-fixture remove`(관리자), 테스트 백업은 앱에서 삭제

**아직 남은 것 (사용자 손이 필요함):**
1. UAC 창에서 "아니요"를 눌렀을 때 알림("관리자 권한 요청이 거절됐어요") — 시험 중 세 번 모두 "예"가 눌려 확인 못 함
2. 저장 창(CSV 내보내기 → Excel로 열기), "웹에서 검색", "Windows 설정에서 제거" 버튼 (외부 창·브라우저가 열려서 사용자가 봐야 함)
3. 저장공간 "Windows 임시 파일"(관리자) 정리는 실제 파일이라 시험하지 않았다. 테스트 전용 관리자 항목을 먼저 만들어야 한다.
4. 강제 제거의 "실행 중 안내": 실제로 도는 테스트 실행 파일이 필요하다.

### 클라우드(리눅스) 세션에서 시작하기

```bash
bash scripts/cloud-setup.sh      # Wine·mingw·Rust 대상·pnpm 의존성 (여러 번 실행해도 됨)
bash scripts/wine-check.sh       # 유닛 테스트 + 테스트 앱 예제 10개를 Wine에서
cargo clippy --workspace --all-targets --target x86_64-pc-windows-msvc -- -D warnings
pnpm exec tsc --noEmit -p . && pnpm exec prettier --check src
```

- 리눅스용 `cargo test`는 `--exclude robinx`로 돌린다 (Tauri 앱은 GTK가 필요하다).
- 화면은 `scripts/ui-mock/README.md`대로 가짜 응답으로 캡처해서 확인한다.
- Wine에서 안 되는 것: 앱 화면(WebView2), UAC, 작업 스케줄러 목록, Store 패키지(빈 목록), 저장 창.

### 다음 작업 후보

- 위 "아직 남은 것"
- 시작프로그램: 일반 사용자에게 안 보이는 SYSTEM 예약 작업을 관리자 권한으로 읽어 보여 주기
- Store 앱 직접 제거: 테스트용 MSIX(자체 서명 또는 개발자 모드 등록)를 `crates/fixture`에 먼저 만든다
- 시스템 복원 지점(제거 전 선택), 서비스 정리(중지·삭제)
- 설치 추적을 ETW로 (커널 드라이버는 쓰지 않음)
- 배포: 코드 서명, NSIS 설치 파일(`pnpm tauri build`)

## 4. 작업 기록 (오래된 것부터)

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
- 끄기·다시 켜기는 작업 관리자와 같은 `StartupApproved` 값만 바꾼다. RobinX가 끈 항목은 원래 값을 기록해 두었다가 그대로 되돌린다.
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

- `scripts/wine-check.sh`: Windows용(x86_64-pc-windows-gnu)으로 빌드해 Wine에서 유닛 테스트 전체와 테스트 앱 예제를 돌린다.
  처음엔 6개였고 지금은 10개다(2026-10-19 기준 모두 통과). 준비는 `scripts/cloud-setup.sh`.
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

### 최적화 1차 — 완료 (2026-10-19)

- optimizerNXT(GPL-3.0)는 코드 없이 구조 아이디어만 참고 (분석 메모는 `docs/private/optimizernxt-notes.md`, 커밋 안 함).
- `crates/tweaks`: 항목은 `catalog.rs`에 데이터로. 현재 사용자(HKCU) 공식 설정만, 정책(`\Policies`)·Defender·SmartScreen·업데이트·방화벽·Run 키 등은 테스트로 금지.
  - 적용: 지금 값을 읽어 `%LOCALAPPDATA%\RobinX	weaks\changes.json`에 **먼저** 기록 → 쓰기. 쓰다 실패하면 이번에 쓴 값을 되돌림. 다시 적용해도 첫 기록 유지.
  - 되돌리기: 기록한 원래 값(종류까지)으로. 지금 값이 우리가 쓴 값과 다르면(다른 곳에서 바꿈) 덮어쓰지 않고 알림. 적용하며 만든 키는 위쪽까지 비어 있으면 지움.
  - 상태: 적용됨/일부/아님 (읽기 전용). RobinX가 바꾸지 않았는데 이미 적용된 항목은 원래 값을 모르므로 화면에서 끌 수 없게 둠.
- 화면: 사이드바 "최적화", 묶음별 카드, 항목마다 설명·이유·"바꾸는 값 보기"(레지스트리 경로·값, 설정 앱 위치), 다시 시작 안내, "모두 되돌리기".
- **확인:** `tweaks_fixture` 17개 통과(값 없던 경우·원래 값 REG_EXPAND_SZ 보존·다른 곳에서 바뀐 값 보존·만든 키 정리). 실제 앱에서 테스트 항목만 켜고 끄기 → 값 쓰기·기록·되돌리기·키 정리 확인. 실제 항목은 상태 읽기만 했다.
- **아직:** 실제 항목의 반영(탐색기 새로 고침 등)은 사용자가 직접 켜 보며 확인해야 한다. 관리자 범위(HKLM) 항목은 없음.

### 최적화 2차 — 완료 (2026-10-19)

- RyTuneX·optimizerDuck·WinMemoryCleaner(모두 GPL 계열)는 아이디어만 참고 (메모 `docs/private/optimizer-3repos-notes.md`, 커밋 안 함).
- **다시 읽어 확인:** 적용·되돌리기 직후 값을 다시 읽어 다르면 이번에 바꾼 값을 되돌리고 오류로 알린다.
- **바로 반영:** 적용·되돌리기 뒤 `WM_SETTINGCHANGE`(전체, "TraySettings", "ImmersiveColorSet")와 `SHChangeNotify(SHCNE_ASSOCCHANGED)`를 보낸다(`notify.rs`). 다시 로그인해야 하는 항목은 보내지 않는다.
- **선택지 항목:** `Tweak.options`/`default_option`. 처음 고를 때 모든 선택지의 값을 원래 값으로 기록하고, 선택지를 바꾸면 기록의 `option`만 갱신(원래 값은 처음 것). 값이 하나도 없으면 Windows 기본 선택지로 표시.
  항목: 작업 표시줄 아이콘 위치(`TaskbarAl`), 단추 합치기(`TaskbarGlomLevel`·`MMTaskbarGlomLevel`), 검색(`SearchboxTaskbarMode`), 탐색기 시작 위치(`LaunchTo`).
- **추가 항목:** 작업 보기 단추, 시계 초, 작업 끝내기, 숨김 파일, 탐색기 간격, 클립보드 기록, 탐색기 동기화 알림.
- **확인:** `tweaks_fixture` 28개 통과(선택지 기본값·고르기·바꾸기·되돌리기 포함). 실제 앱은 26개 항목 상태 읽기만, 테스트 선택지 항목만 바꿔 봄 → 값 쓰기·되돌리기·키 정리 확인.
