# 자체 설치 프로그램 설계

NSIS 설치 화면 대신 RobinX 디자인의 설치·제거 화면을 쓰기 위해, 설치 프로그램을 직접 만든다 (사용자 결정, 2026-10-19).

## 핵심: robinx.exe가 스스로 설치한다

- 배포하는 `RobinX_<버전>_x64-setup.exe`는 **robinx.exe와 같은 파일**이다. 실행 방법에 따라 설치 화면이 된다.
- 설치는 자기 자신을 설치 폴더에 복사하고, Windows에 등록하고, 바로 가기를 만드는 것이다. 따로 묶어 넣을 파일이 없다.
- 설치·제거 화면은 앱과 같은 React·디자인으로 만든다. 판단과 파일·레지스트리 작업은 `crates/setup`(Rust)이 하고 테스트한다.

### 모드 판단 (`src-tauri/src/main.rs`, 앱을 띄우기 전)

| 조건 | 모드 |
|---|---|
| 인수에 `--uninstall` | 제거 화면 (`/S`가 있으면 화면 없이) |
| 인수에 `/P`, `/S`, `/UPDATE` 중 하나 | 설치 (화면 없이). 자동 업데이트가 이렇게 부른다 |
| 실행 파일 이름에 `setup` 또는 `installer`가 들어 있음 | 설치 화면 |
| 그 밖 | 평소 앱 |

설치된 위치에서 실행된 robinx.exe는 이름이 `robinx.exe`이므로 항상 평소 앱이다.

## 자동 업데이트와의 약속 (tauri-plugin-updater 2.14)

- 업데이터는 받은 파일이 exe면 서명을 확인한 뒤 `"<파일>" /P /UPDATE /R /ARGS <원래 인수>`로 실행하고 앱을 바로 끝낸다.
  (`updater.rs` `install_inner`, `updater_parameters`, `config.rs` `nsis_args`)
- 그래서 설치 모드는 다음을 받아들인다.
  - `/P` 수동 진행(화면 없음, 물어보지 않음), `/S` 조용히, `/UPDATE` 바로 가기를 새로 만들지 않음,
    `/R` 끝나면 앱 다시 실행, `/ARGS …` 다시 실행할 때 넘길 인수, `/NS` 바로 가기 만들지 않음.
- 앱이 끝나기를 기다린다(최대 15초). 그래도 안 끝나면 설치 폴더의 robinx.exe만 끝낸다.
- `latest.json`은 지금처럼 setup.exe와 그 서명(`.sig`)을 가리킨다.

## 설치

- **폴더:** `%LOCALAPPDATA%\Programs\RobinX` (현재 사용자 설치, 관리자 권한 없음).
  - 0.1.0(NSIS)은 `%LOCALAPPDATA%\RobinX`에 설치했는데, 이 폴더는 **백업·설정이 있는 데이터 폴더와 같다.**
    그래서 프로그램 폴더를 분리하고, 옛 위치에서는 `robinx.exe`와 `uninstall.exe` **두 파일만** 지운다. 폴더는 절대 통째로 지우지 않는다.
- **파일:** `robinx.exe`(자기 자신 복사). 임시 이름으로 쓴 뒤 바꿔치기해서, 중간에 실패해도 깨진 실행 파일이 남지 않게 한다.
- **등록 (HKCU, NSIS와 같은 키를 이어 쓴다):**
  - `Software\Microsoft\Windows\CurrentVersion\Uninstall\RobinX`
    `DisplayName`, `DisplayIcon`, `DisplayVersion`, `Publisher`, `InstallLocation`,
    `UninstallString`(`"…\robinx.exe" --uninstall`), `QuietUninstallString`(`… --uninstall /S`),
    `NoModify`=1, `NoRepair`=1, `EstimatedSize`(KB), `MainBinaryName`=`robinx.exe`, `URLInfoAbout`
  - `Software\yoonrobin\RobinX` 기본값 = 설치 폴더
- **바로 가기:** 시작 메뉴 `RobinX.lnk`(AppUserModelID `com.yoonrobin.robinx`), 바탕 화면은 사용자가 고를 때만.
  업데이트(`/UPDATE`) 때는 새로 만들지 않고, 옛 위치를 가리키는 바로 가기만 새 위치로 고친다.
- **기록:** 설치 폴더의 `install.json`에 설치한 파일과 바로 가기 목록을 남긴다. 제거는 이 목록에 있는 것만 지운다.

## 제거

- 앱이 실행 중이면 끝낼지 묻는다 (`/S`면 끝낸다).
- `install.json`에 있는 파일·바로 가기, 등록 키를 지운다. 폴더는 비었을 때만 지운다.
- **백업과 설정(`%LOCALAPPDATA%\RobinX`)은 기본으로 남긴다.** "백업과 설정도 지우기"는 되돌릴 수 없다는 경고와 함께 직접 골라야 한다.
- 실행 중인 자기 자신은 지울 수 없으므로, 임시 폴더에 복사본을 만들어 그 복사본이 제거를 마치고, 끝나면 복사본은 다음 로그인 때 지워지게 한다.

## WebView2

- 설치 화면도 WebView2가 필요하다. Windows 11과 대부분의 Windows 10에는 있다.
- 없으면 화면을 띄우기 전에 Windows 기본 메시지 창으로 알리고, Microsoft 다운로드 페이지를 연다.
- 화면 없는 모드(`/P`, `/S`)는 WebView2 없이도 동작한다.

## 시험

- `crates/setup` 유닛 테스트: 임시 폴더와 시험용 레지스트리 위치(`HKCU\Software\RobinX-SetupTest`)로 설치 → 업그레이드 → 제거,
  옛 NSIS 위치 이전(두 파일만 지우고 백업 폴더는 그대로), 기록에 없는 파일은 지우지 않음.
- 실제 PC: RobinX 자신만 설치·업데이트·제거한다 (다른 프로그램에는 손대지 않음).
