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

### 다음 단계 후보

- Phase 5 나머지: 강제 제거(언인스톨러 없는 프로그램), SYSTEM 예약 작업의 UAC 끄기·켜기 시험
- 시스템 복원 지점, 서비스 정리(중지·삭제), 오래된 백업 자동 정리
- Phase 6: 설치 추적(스냅샷 비교 → ETW, 커널 드라이버는 쓰지 않음)
- 배포: 코드 서명, NSIS 설치 파일(`pnpm tauri build`)
