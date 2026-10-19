# Windows Sandbox로 실제 프로그램 시험하기

이 PC에서는 실제 프로그램을 지우지 않는다(CLAUDE.md 안전 원칙). 대신 Windows Sandbox에서 실제 무료 프로그램을 설치·제거해
RobinX가 남은 흔적을 얼마나 정확히 고르는지 잰다. Sandbox는 창을 닫으면 안의 내용이 모두 사라지고, 이 PC와는
아래 `shared` 폴더 하나로만 이어진다.

## 준비물 (이 폴더)

| 파일 | 하는 일 |
| --- | --- |
| `programs.csv` | 시험할 무료 프로그램 20개 (winget ID, 설치 방식, 고른 이유) |
| `download-installers.ps1` | 설치 파일을 winget으로 받기만 한다 (이 PC에 설치하지 않음) → `out\installers\` |
| `new-wsb.ps1` | 빌드한 `robinx.exe`를 `out\app\`에 복사하고 `out\robinx-test.wsb`를 만든다 |
| `results-template.md` | 결과 기록 양식 → C2에서 `docs/sandbox-results.md`로 복사해 채운다 |

`out\`은 git에 넣지 않는다 (설치 파일, 빌드 결과, 캡처).

설치 방식이 고르게 섞이도록 골랐다: MSI(wix) 8개, NSIS(nullsoft) 6개, Inno 4개, 그 밖의 exe 2개.
사용자 파일을 만드는 프로그램(ShareX, KeePassXC, Firefox 프로필)은 RobinX가 그것을 **미리 고르지 않는지** 보려고 넣었다.

## 처음 한 번 (사용자 손이 필요함, LOOP C2 🔒)

1. "Windows 샌드박스" 기능 켜기: 관리자 PowerShell에서
   `Enable-WindowsOptionalFeature -Online -FeatureName Containers-DisposableClientVM -All` → 재부팅.
   (Windows Pro 이상. 이 PC는 Windows 11 Pro)
2. 빌드: `pnpm tauri build --no-bundle`
3. 설치 파일 받기: `powershell -ExecutionPolicy Bypass -File scripts\sandbox\download-installers.ps1`
4. 설정 만들기: `powershell -ExecutionPolicy Bypass -File scripts\sandbox\new-wsb.ps1` → `out\robinx-test.wsb` 더블클릭

## Sandbox 안에서 프로그램마다

한 Sandbox에서 4~5개씩 하고 닫는다 (앞 프로그램의 흔적이 쌓이지 않게).

1. `C:\Kit\app\robinx.exe` 실행 (설치하지 않고 바로 실행된다).
2. **설치 추적** → "추적 시작".
3. `C:\Kit\installers\<id>\`의 설치 파일을 기본 옵션으로 설치.
4. "설치 끝남" → 새로 생긴 항목 목록을 캡처해 `C:\Kit\shared\`에 저장 → **"나중에"를 눌러 기록은 저장하지 않는다.**
   저장하면 그 기록이 잔여물 스캔의 근거로 쓰여, 스캐너만의 정확도를 잴 수 없다.
5. 프로그램을 한 번 실행했다가 닫는다 (설정·캐시 파일이 생기게).
6. RobinX **프로그램** 화면에서 그 프로그램을 제거 → 원래 언인스톨러 → 끝나면 "남은 흔적 찾기".
7. 결과를 캡처해 `C:\Kit\shared\`에 저장하고, 양식의 열(미리 고른 항목, 검토 항목, 잘못 고른 항목, 놓친 항목)을 채운다.
   정답은 4번 목록 중 언인스톨러 뒤에도 실제로 남아 있는 것이다.
8. 몇 개만 골라 정리 → **복구 기록**에서 되돌려 원래대로 돌아오는지 본다 (Sandbox라 정리해도 괜찮다).

## 주의

- 시험은 Sandbox 안에서만 한다. 이 PC에서 `download-installers.ps1`는 파일을 받기만 하고 실행하지 않는다.
- 카카오톡처럼 로그인이 필요한 프로그램은 로그인하지 않고 설치·첫 실행 화면까지만 본다.
- Sandbox 안의 RobinX는 업데이트 확인으로 GitHub에 접속할 수 있다. 막으려면 `new-wsb.ps1 -NoNetwork`
  (그 경우 인터넷이 필요한 설치 파일은 실패할 수 있다).
- 스크립트(`.ps1`)는 BOM 있는 UTF-8로 저장한다. BOM이 없으면 Windows PowerShell 5.1이 한글을 깨뜨려 읽는다.
