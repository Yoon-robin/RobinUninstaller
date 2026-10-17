# RobinUninstaller 개발 계획

> 출처: ChatGPT(GPT-6) 상담 결과를 정리하고 검토 의견을 덧붙임 (2026-10-16)
> 대상: Windows 10/11 x64

## 제품 방향

- Revo Uninstaller처럼 **제거 → 잔여물 분석 → 골라서 정리 → 복구** 흐름을 먼저 완성한다.
- 핵심 경쟁력은 기능 개수가 아니라 **"이 파일/레지스트리가 정말 이 프로그램 것인가"를 정확히 판단하고, 왜 그렇게 판단했는지 보여주는 것**이다.
- 원칙: 스캔에서 찾았다고 자동 삭제하지 않는다. 지우면 안 되는 것을 남기는 쪽이 우선이다. 지운 것은 되돌릴 수 있어야 한다.

## 1. 기술 스택 (확정: Tauri 2 + React + shadcn/ui)

Electron처럼 예쁜 웹 UI(shadcn/ui)를 쓰면서도 가볍게 가기 위해 **Tauri 2**를 쓴다. UI는 웹(React)이고, Windows를 실제로 건드리는 엔진은 Rust로 만든다. Tauri는 시스템에 있는 WebView2를 쓰기 때문에 Electron보다 설치 용량과 메모리 사용이 훨씬 작다.

| 영역 | 선택 |
|---|---|
| 앱 셸 | Tauri 2 (WebView2) |
| 프론트엔드 | React + TypeScript + Vite |
| UI | shadcn/ui + Tailwind CSS, lucide 아이콘 |
| 상태 / 데이터 | TanStack Query (엔진 호출 결과 캐시), Zustand (UI 상태) |
| 엔진 (백엔드) | Rust: `windows` 크레이트(Win32/레지스트리/MSI/COM), `winreg` |
| 저장소 | SQLite (`rusqlite`) + JSON 매니페스트 |
| 로깅 | `tracing` |
| 테스트 | `pnpm check`(tsc + fmt + clippy + cargo test)를 로컬에서 실행, 삭제 테스트는 Windows VM에서 |
| 배포 | x64, NSIS 설치형 (Tauri 번들러) |

GPT-6은 C# + WinUI 3를 추천했지만, shadcn을 쓰려면 웹 UI가 필요해서 Tauri로 바꿨다. 덕분에 시스템 코드가 Rust여서 메모리 안전성 면에서도 유리하다. 대신 Rust를 배우는 시간과 Windows API 바인딩 작업이 필요하다.

## 2. 기능 범위

| 기능 | 버전 |
|---|---|
| 설치된 프로그램 목록 (검색·정렬·아이콘·용량·게시자) | MVP |
| 기본 언인스톨러 실행 | MVP |
| 잔여물 스캔 (파일·폴더·레지스트리) | MVP |
| 안전 삭제 + 격리 + 복구 | MVP |
| 강제 제거 (언인스톨러 손상/없음) | v0.2 |
| 시작프로그램 관리 (Run 키, 시작 폴더, 예약 작업) | v0.3 |
| 설치 추적 (설치 전후 스냅샷) | v0.4 |
| 실시간 설치 모니터링, 여러 개 한꺼번에 제거 | v1.0+ |

MVP 흐름: 프로그램 선택 → 정보 수집 → 기본 언인스톨러 실행 → 제거 상태 재확인 → 잔여물 스캔·위험도 분석 → 사용자가 항목별 선택 → 백업 후 삭제 → 기록 저장(복구 가능)

## 3. 아키텍처

```
RobinUninstaller/
├── src/                        # React + shadcn/ui 프론트엔드
│   ├── components/ui/          # shadcn 컴포넌트
│   ├── features/               # programs, scan, restore, startup ...
│   └── lib/api.ts              # Tauri invoke 래퍼 (타입 정의)
├── src-tauri/                  # Tauri 앱 (얇은 command 계층만)
├── crates/                     # Rust 워크스페이스
│   ├── core/                   # 모델, 정책(RiskPolicy, 보호 경로), 공통 타입
│   ├── discovery/              # Registry / MSI / AppX Provider
│   ├── scanner/                # File/Registry Scanner, OwnershipAnalyzer, RiskScorer
│   ├── execution/              # UninstallCoordinator, DeletionPlanner, TransactionJournal
│   ├── recovery/               # Backup, Quarantine, Restore
│   ├── monitor/                # 설치 추적
│   ├── startup/                # 시작프로그램
│   └── elevated-worker/        # 관리자 권한 작업 전용 실행 파일
├── tests/                      # 통합/안전성 테스트
└── docs/
```

- 프론트엔드는 화면만 담당한다. 판단과 시스템 변경은 모두 Rust 엔진이 한다.
- Tauri command는 **읽기 작업**(목록, 스캔, 미리보기)만 직접 처리한다. 삭제·제거처럼 시스템을 바꾸는 작업은 관리자 권한으로 띄운 `elevated-worker`에 Named Pipe로 요청한다.
- Worker는 받은 작업 종류·경로·대상을 **다시 검증**한다. 웹 UI가 보낸 명령이나 경로를 그대로 실행하지 않는다. (WebView가 오염돼도 시스템이 망가지지 않게 하기 위함)
- Tauri capabilities로 프론트엔드가 쓸 수 있는 command를 최소한으로 제한한다.

## 4. 잔여물 탐지와 안전장치

### 4.1 설치 프로그램 탐색
- 레지스트리 Uninstall 키: `HKLM`, `HKLM\...\WOW6432Node`, `HKCU`, 다른 사용자 하이브도 포함. 32/64비트는 `KEY_WOW64_32KEY` / `KEY_WOW64_64KEY`로 명시해서 처리한다.
- MSI: `MsiEnumProductsExW` / `MsiGetProductInfoEx`
- MSIX/AppX: 패키지 관리 API (처음엔 목록만 보여줌)
- 앱 ID는 표시 이름이 아니라 **설치 식별자 + 사용자 SID + 설치 범위**를 조합해서 만든다.

### 4.2 3단계 파이프라인
1. **후보 수집**: 설치 폴더, `%AppData%`, `%LocalAppData%`, `%ProgramData%`, `%ProgramFiles%`, 관련 레지스트리 경로 (C 드라이브 전체를 뒤지지 않는다)
2. **소유권 분석**: 제품 GUID, 게시자, 설치 경로 정확 일치, 다른 프로그램과 공유하는지 확인
3. **위험도 분류**: 높음(80–100, 기본 체크) / 검토 필요(40–79, 기본 해제) / 낮음(0–39, 자동 선택 금지)

점수 예시(초기값이며 테스트하며 조정): 설치 추적으로 확인 +70, MSI 제품코드 일치 +65, 설치 경로 정확 일치 +50, 앱 고유 폴더 +35, 게시자 일치 +15, 이름 일부 일치 +5, 다른 앱이 같은 경로 참조 −80, 공유 구성요소 −100.
**보호 경로나 공유 리소스는 점수와 상관없이 무조건 차단**한다.

### 4.3 안전 삭제 흐름
`Scanner → DeletionCandidate → RiskPolicy 검증 → DeletionPlan → 사용자 승인 → Worker 재검증 → 백업/격리 → 삭제 → 기록 → 복구 매니페스트`

- `std::fs::remove_dir_all` 같은 재귀 삭제를 스캔 결과에 바로 호출하지 않는다.
- Dry Run(미리보기), 파일 격리 보관, 레지스트리 백업(키·값·형식·하이브·뷰)
- 삭제 직전에 경로를 다시 확인하고 심볼릭 링크·정션·리파스 포인트를 검사한다.
- 다른 설치/제거 작업과 동시에 돌리지 않는다. 일부만 지워졌거나 재부팅이 필요한 상태도 기록한다.
- 시스템 복원 지점(`SRSetRestorePointW`)은 보조로만 쓴다. 24시간 안에 이미 만든 지점이 있으면 새로 안 만들 수 있으므로 **자체 백업이 필수**다.

## 5. 로드맵 (1인 개발, 약 12–20주)

| 단계 | 기간 | 내용 | 완료 기준 |
|---|---|---|---|
| Phase 0 | 1주 | Tauri + shadcn 골격, Rust 워크스페이스, SQLite, 로그, CI, 삭제 정책 정의 | 엔진을 UI 없이 단위 테스트할 수 있음 |
| Phase 1 | 2주 | Registry/MSI Provider, 중복 제거, 목록·검색·아이콘 | 테스트 VM의 EXE/MSI 프로그램이 정확히 보임 |
| Phase 2 ✅ | 2주 | 기본 언인스톨러 실행, 권한 상승, 프로세스 감시, 재부팅 상태 | 제거 후 실제 상태를 재확인하고 실패를 정확히 표시 |
| Phase 3 ✅ | 3–4주 | 잔여물 스캐너, 소유권 분석, 위험도, 미리보기 화면 | 공유·무관 파일을 자동 선택하지 않음 |
| Phase 4 ✅ | 2–3주 | 격리, 레지스트리 백업, 트랜잭션 기록, 복원 UI → **MVP 출시** | 삭제 항목을 정확히 복원 |
| Phase 5 | 2–3주 | 강제 제거, 시작프로그램 관리 | 시작 항목 비활성화/원상복구 |
| Phase 6 | 3–5주 | 설치 추적(스냅샷), 베타 | 설치 중 변경사항을 다른 프로세스 변경과 구분 |

MVP 품질 목표: 앱 목록 2초 이내(캐시 먼저 표시), 일반 스캔 10초 이내, 안전 테스트 세트에서 오탐 0건, 테스트 데이터 복원 100%, 스캔 중 UI 멈춤 없음, 잘못된 관리자 요청은 전부 거부.

## 6. 기술적 난제

1. **소유권을 완벽히 알 수 없다**: 예를 들어 `Adobe` 폴더는 여러 제품이 같이 쓴다. 공용 폴더는 통째로 지우지 말고 항목별로 분석하고, 불확실하면 자동 선택하지 않는다.
2. **언인스톨러가 종료됐다고 제거가 끝난 게 아니다**: 자식 프로세스를 띄우고 먼저 종료하는 경우가 있다. 프로세스 종료 대기만으로는 부족하고, 등록 정보 변화·프로세스·파일 잠금·재부팅 필요 여부를 보는 상태 머신이 필요하다.
3. **강제 제거는 위험하다**: 서비스·드라이버·COM·예약 작업이 남는다. 보안 프로그램, 안티치트, VPN 드라이버는 초기 대상에서 빼야 한다.
4. **설치 모니터링이 어렵다**: Windows Update나 백신이 동시에 파일을 바꾼다. 스냅샷 → 이벤트(ETW 등) → 프로세스 연계 순서로 키우고, 커널 드라이버는 만들지 않는다.
5. **백업했다고 다 복원되는 건 아니다**: ACL, 잠긴 파일, 재설치 후 충돌 등이 있다. 복원 전에 현재 상태와 충돌하는지 확인한다.
6. **배포와 신뢰**: 코드 서명, 업데이트 무결성, Windows App SDK 런타임 배포(self-contained면 용량이 커짐)를 고려한다.

## 7. 첫 마일스톤 체크리스트

- [x] Tauri 2 + React + shadcn/ui 앱 골격, Rust 워크스페이스(crates/) 생성
- [x] `InstalledApplication` 구조체와 Provider 트레이트 정의
- [x] HKLM/HKCU 32·64비트 Uninstall 정보 수집
- [x] MSI 정보 수집 및 중복 통합
- [x] 프로그램 목록 화면 + 검색·정렬
- [x] 제거 워크플로 상태 머신
- [x] 읽기 전용 잔여물 스캐너 + 테스트 픽스처
- [ ] 백업·복원 안전성 테스트를 통과한 뒤에 삭제 기능 활성화

## 검토 의견 (Claude)

- 방향에 동의한다. 특히 "삭제 기능은 백업·복원 테스트를 통과한 뒤에 켠다"는 순서가 중요하다.
- **UI 스택**: shadcn/ui를 쓰기 위해 Tauri 2 + React로 정했다. Electron보다 훨씬 가볍고, 엔진을 Rust로 분리하면 UI와 시스템 코드의 경계가 분명해진다.
- 레지스트리/MSI 바인딩은 Rust에서 처음엔 손이 많이 간다. Phase 1에서 `discovery` 크레이트를 먼저 CLI로 만들어 결과를 검증한 뒤 UI에 붙이는 순서를 추천한다.
- 실제 삭제 테스트는 반드시 VM(Hyper-V 체크포인트)에서만 한다.

## Phase 2 구현 메모

- 언인스톨러는 `ShellExecuteExW`로 직접 실행한다(셸 없음). 매니페스트가 관리자 권한을 요구하면 Windows가 UAC를 띄우고, `ERROR_ELEVATION_REQUIRED`면 `runas`로 다시 시도한다.
- MSI는 등록된 명령(`/I`로 기록된 경우가 많음)을 쓰지 않고, 검증된 제품 코드로 `msiexec /x {GUID}`를 만든다.
- NSIS·Inno처럼 자기 복사본을 띄우고 먼저 끝나는 언인스톨러를 위해, Toolhelp 스냅샷으로 (pid, 부모 pid) 쌍을 추적해 자손 프로세스가 모두 끝날 때까지 기다린다.
- 끝나면 레지스트리를 다시 읽어 결과를 정한다: 제거됨 / 재부팅 필요(3010·1641) / 아직 설치됨(취소 1602 등) / UAC 거절 / 실패.
- 언인스톨러가 스스로 권한을 올리므로 `elevated-worker`는 아직 필요 없다. 파일·레지스트리를 직접 지우는 Phase 3~4에서 만든다.
- 테스트는 실제 프로그램 대신 `crates/fixture`(HKCU에만 등록되는 가짜 앱)로 한다.
- 콘솔 호스트·cmd·설문용 브라우저처럼 기다릴 필요 없는 프로세스는 대기에서 뺀다(지우려는 프로그램이 그 브라우저면 예외). 8초가 지나면 "기다리지 않고 확인" 버튼을 보여 준다.

## Phase 3 구현 메모

- `crates/scanner`는 읽기 전용이다. 찾고 근거를 붙이기만 하고, 삭제는 백업·복원이 생기는 Phase 4에서 켠다.
- **찾는 곳:**
  - 파일: 설치 폴더, Program Files(64/32), ProgramData, AppData(Roaming/Local/Local\Programs), 시작 메뉴(사용자/공용)의 1~2단계.
  - 레지스트리: `Software`(HKCU, HKLM 64/32), 남은 Uninstall 키, MSI 등록(압축 GUID), 설치 폴더를 가리키는 Run 값·App Paths·서비스.
- **이름 정규화:** 버전·연도·아키텍처·에디션 꼬리표와 직접 만든 일반 단어 stoplist를 뺀다. 숫자가 다른 이름("Photoshop 2021" vs "2026")은 정확 일치로 보지 않는다.
- **점수:** 가장 강한 근거 하나 + 나머지 근거의 절반(최대 100) + 감점.
  - 약한 근거를 여러 개 모아도 "확실"이 되지 않는다.
  - 다른 설치 프로그램과 공유하면 "확실"이 될 수 없다.
  - 사용자 문서(docx, psd, blend 등)가 들어 있으면 감점한다.
- **보호 규칙:** 드라이브 루트, Windows 폴더, 기본 폴더 자체, 공유 폴더(Common Files, Microsoft Shared, WindowsApps 등)와 그 하위, Microsoft·Classes·Policies 레지스트리 영역(이름 기반 탐색), 상위 키는 점수와 상관없이 제외한다. RobinUninstaller 자신의 데이터도 제외한다.
- **분류:** 확실(80+, 기본 선택) / 검토(30~79) / 낮음(숨김).
- **검증:** `crates/fixture`의 "Fixture Notes"가 흔적 4종과 미끼 3종을 만들고, `cargo run -p robin-scanner --example scan_fixture`가 8가지 기대 결과를 확인한다.

## Phase 4 구현 메모

- `crates/recovery`는 영구 삭제를 하지 않는다.
  - 파일·폴더는 `%LOCALAPPDATA%\RobinUninstaller\backups\<작업 ID>\files`로 옮긴다. 같은 드라이브면 원자적 이동이고, 다른 드라이브면 복사 후 원본을 지운다.
  - 레지스트리는 값 형식과 원본 바이트를 JSON(base64)으로 저장한 **뒤에만** 지운다.
- **지우기 직전 재검사:**
  - 보호 규칙을 다시 적용하고, 백업 저장소와 RobinUninstaller 자신은 건드리지 않는다.
  - Windows 서비스 키는 아직 정리하지 않는다.
  - 정션·심볼릭 링크는 다루지 않는다.
  - 프론트엔드는 마지막 스캔 결과의 항목 ID만 넘길 수 있다.
- **매니페스트:** 항목마다 처리 직후 저장한다(임시 파일 → 바꿔치기). 상태는 removed / needsAdmin / alreadyGone / blocked / failed / restored.
- **복원:** 원래 자리에 같은 이름이 새로 생겼으면 덮어쓰지 않고 실패로 알린다.
- **관리자 권한:** 일반 권한으로 못 지운 항목만 앱 바이너리의 작업자 모드(`--elevated-clean|--elevated-restore <작업 ID> --store <...\RobinUninstaller\backups>`)를 UAC로 띄워 처리한다. 작업자는 기록을 디스크에서 다시 읽고 모든 검사를 처음부터 다시 한다.
- **검증:** `cargo run -p robin-recovery --example clean_fixture`로 정리 → 사라짐 확인 → 복원 → 원본과 바이트 단위 비교, 위험 항목(System32, HKLM\SOFTWARE\Microsoft) 차단을 확인한다.
- **아직 안 한 것:**
  - 시스템 복원 지점
  - 서비스 정리(중지·삭제)
  - 백업 보존 기간 자동 정리
- 레지스트리 삭제는 부모 키를 읽기 권한으로만 열고 DELETE는 지울 키에만 요청한다. `HKLM\SOFTWARE` 같은 하이브 루트는 관리자에게도 DELETE를 주지 않는다. UAC 실행으로 HKLM·Program Files 정리와 복원을 검증했다(2026-10-16).

## Phase 5 구현 메모 — 시작프로그램 관리

- `crates/startup`이 담당한다. 대상: Run 키(HKCU, HKLM 64·32비트), 시작 폴더(사용자, 공용), 로그인·부팅 트리거가 있는 예약 작업.
- **끄기 방식:** 작업 관리자와 같은 `Explorer\StartupApproved\{Run|Run32|StartupFolder}` 바이너리 값을 쓴다.
  - Run 값이나 바로 가기 파일은 그대로 두므로 지우는 것이 아니다.
  - 첫 바이트가 홀수(03)면 꺼짐, 짝수(02·04·06)거나 값이 없으면 켜짐. 4~11바이트는 끈 시각(FILETIME).
- **원상복구:** RobinUninstaller가 끌 때 원래 값(없었으면 "없음")을 `%LOCALAPPDATA%\RobinUninstaller\startup\changes.json`에 **먼저** 기록한다.
  - 다시 켜면 그 값을 그대로 돌려놓거나, 원래 값이 없었으면 값을 지운다.
  - 다른 곳(작업 관리자 등)에서 끈 항목은 기록이 없으므로 작업 관리자와 같은 "켜짐" 값(02)을 쓴다.
- **예약 작업:** 작업 스케줄러 COM API(`ITaskService`)로 모든 폴더를 돌며 로그온(`TASK_TRIGGER_LOGON`)·부팅(`TASK_TRIGGER_BOOT`) 트리거가 있는 작업만 읽는다.
  - `\Microsoft\` 아래(Windows 자체 작업)는 읽지 않는다. 숨김 작업은 포함한다.
  - 표시: 이름, 작업 경로, 첫 번째 "프로그램 시작" 동작의 명령, 만든 이(`RegistrationInfo.Author`), 사용 여부. 게시자·아이콘은 실행 파일에서 읽는다.
  - 끄기·켜기는 작업의 `Enabled` 값만 바꾼다. 원래 값을 같은 `changes.json`에 `previousEnabled`로 **먼저** 기록하고, 켤 때 그 값으로 되돌린 뒤 기록을 지운다. 작업 스케줄러는 끈 시각을 남기지 않아서 우리가 끈 경우에만 기록 시각을 보여 준다.
  - 관리자 표시: 실행 계정(`Principal.UserId`)이 현재 사용자가 아니거나(SYSTEM 등) "가장 높은 권한으로 실행"이면 표시한다. 실제로는 `E_ACCESSDENIED`가 나면 같은 작업자 모드(UAC)로 넘긴다.
- **ID:** 등록 위치 + 이름(소문자)의 FNV-1a 해시 16자리. 예약 작업은 `task` + 작업 경로. 프론트엔드와 관리자 작업자는 이 ID만 주고받고, 엔진이 목록을 다시 읽어 항목을 찾는다.
- **관리자 권한:** HKLM·공용 시작 폴더 항목은 일반 권한으로 쓰기가 거부되면 작업자 모드(`--elevated-startup <on|off> <ID> --store <...\RobinUninstaller\startup>`)를 UAC로 띄운다.
- **표시 정보:** 실행 명령에서 실행 파일을 찾고(경로 없는 `rundll32.exe` 등은 Windows 폴더에서), 바로 가기는 IShellLink로 대상을 읽는다. 게시자·설명은 파일 버전 정보에서 읽는다. 파일이 없으면 "파일 없음"으로 표시한다.
- **검증:** `cargo run -p robin-startup --example toggle_fixture`로 테스트 앱의 Run 값, 시작 폴더 바로 가기, 예약 작업 `\Fixture Notes Updater`를 껐다 켜서 원래 상태(값 없음 / 작업 "사용")로 돌아오는지 확인한다. 앱 화면에서도 같은 흐름을 확인했다(2026-10-17, 예약 작업 포함).
- **관리자 권한 검증(2026-10-17):** 일반 권한으로 띄운 앱에서 `FixtureNotesMachine`(HKLM Run)을 끄기 → UAC → HKLM `StartupApproved\Run`에 03 값, 원래 값(없음)은 사용자 기록에 남음 → 다시 켜기 → UAC → 값이 지워져 원래 상태, 기록도 비워짐.
- **아직 안 한 것:**
  - SYSTEM 예약 작업의 UAC 경로 실제 시험 (`robin-fixture install --machine`이 만드는 `\Fixture Notes Machine Task`)
  - 서비스
  - 파일이 없는 항목 정리(백업 후 Run 값·바로 가기 제거)
