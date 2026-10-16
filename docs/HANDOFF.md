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

### 진행 중이던 일: 관리자 권한(UAC) 정리 실제 시험

`robin-fixture install --machine`(관리자로 실행)으로 `HKLM\SOFTWARE\Robin Test Labs`와
`C:\Program Files\Robin Test Labs`를 만들고 앱에서 정리했다.

- 일반 권한 정리: HKCU·AppData 항목 4개 정리, HKLM·Program Files 2개는 "관리자 권한 필요"로 남음 (기대대로)
- 관리자 권한 정리(UAC 허용): **Program Files 폴더는 정리됐지만 `HKLM\SOFTWARE\Robin Test Labs` 키 삭제가 관리자 권한에서도 "접근 거부"로 실패.**
  ACL은 Administrators FullControl로 정상. 원인 미확인.
  - 다음 할 일: `crates/recovery/src/registry.rs`의 오류에 단계(열기/삭제)와 Windows 오류 코드를 담아 다시 UAC로 시험.
  - 의심: 부모 키(`SOFTWARE`)를 `KEY_ALL_ACCESS`로 여는 부분, `RegDeleteTreeW` 호출 방식.
- 그다음: 복구 기록에서 "관리자 권한으로 복원" 시험 → `robin-fixture remove`를 관리자로 실행해 HKLM·Program Files 테스트 흔적 정리.

### 다음 단계 후보

- Phase 5: 강제 제거(언인스톨러 없는 프로그램), 시작프로그램 관리
- 시스템 복원 지점, 서비스 정리(중지·삭제), 오래된 백업 자동 정리
- Phase 6: 설치 추적(스냅샷 비교 → ETW, 커널 드라이버는 쓰지 않음)
- 배포: 코드 서명, NSIS 설치 파일(`pnpm tauri build`)
