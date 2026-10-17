# 비슷한 프로젝트 조사 (2026-10-17)

언인스톨러와 저장공간 정리·분석 도구를 조사해 우리 앱에 가져올 점을 모았다.
라이선스·별 수·최근 활동은 2026-10-17 기준 GitHub에서 확인했다.

> **라이선스 원칙:** 우리 리포지토리는 MIT 공개다. GPL 프로젝트(BleachBit, WinDirStat, MangoDisk, dupeGuru, Krokiet)는 코드·규칙 파일을 가져오지 않고 아이디어만 참고한다. MIT·Apache-2.0 프로젝트는 고지를 지키면 크레이트로 쓰거나 코드를 참고할 수 있다.

## 1. 언인스톨러

| 프로젝트 | 라이선스 / 기술 | 참고할 점 |
|---|---|---|
| [Bulk Crap Uninstaller](https://github.com/BCUninstaller/Bulk-Crap-Uninstaller) | Apache-2.0 / C# / 별 약 2.2만 | 아래 별도 정리. 가장 참고 가치가 크다. 유지보수자를 구하는 중이라 미래는 불확실하다. |
| [UniGetUI](https://github.com/Devolutions/UniGetUI) | MIT / C# / 별 약 2.7만 | winget·Scoop·Chocolatey·pip·npm·cargo를 하나로 묶는 추상화. 설치 항목에 winget ID를 붙이는 근거로 쓸 수 있다. |
| [UninstallView](https://www.nirsoft.net/utils/uninstall_view.html) | 프리웨어 | QuietUninstallString 노출, 오프라인 레지스트리 읽기, 키 수정 시간 표시, CSV/JSON 내보내기. |
| [BHUninstaller](https://github.com/wpexpertinbd/BHUninstaller) | MIT / 소규모 | 전부 휴지통으로 보내고 파일마다 왜 매칭됐는지 보여 준다. 우리 원칙과 같다. |
| Revo / IObit / Geek (상용) | 아이디어만 | Revo: Hunter Mode(창·트레이 아이콘을 끌어다 놓아 제거), 강제 제거, 일괄 제거 시 복원 지점 1회. Geek: "지난 실행 이후 새로 설치·변경된 프로그램 강조". IObit: 번들웨어 오탐과 자체 번들 설치로 신뢰를 잃음 → **따라 하지 말 것**. |

- Rust/Tauri로 만든 의미 있는 규모의 Windows 언인스톨러는 찾지 못했다. 이 자리가 비어 있다.
- "revo-uninstaller-windows" 같은 이름의 별 0개 저장소는 상용 제품을 사칭한 미끼일 가능성이 높다. 참고하지 않는다.

### BCU에서 배울 것 (소스 확인)

- **신뢰도 점수:** 근거마다 고정 점수(언인스톨 키 +20, 회사명 일치 +4·불일치 −2, 다른 앱이 쓰는 폴더 −7, Store 앱 −10, 실행 파일 있음 −4, 경로 깊이마다 −2 등)를 더하고, 등급 구간(Bad 5 / Questionable 7 / Good 9 / VeryGood 12)으로 나눈다. 우리 scanner의 점수 체계와 비교해 볼 만하다.
- **이름 매칭:** Sift4 편집거리. 4자 이하는 매칭하지 않는다. 비슷한 이름의 다른 앱이 있으면 가장 가까운 것만 남기고 나머지는 감점한다.
- **잔여물 범위:** 방화벽 규칙, COM, AppCompatFlags, UserAssist, EventLog 소스, Prefetch, WER, 바로 가기까지 본다.
- **고아 폴더:** Program Files에서 어느 설치 항목에도 속하지 않는 폴더를 찾는다. 강제 제거와 저장공간 정리 양쪽에 쓸 수 있다.
- **언인스톨러 찾기:** UninstallString이 없으면 설치 폴더에서 `unins*`, `uninstall*` 실행 파일을 찾는다.
- **조용한 제거:** Inno `/VERYSILENT /SUPPRESSMSGBOXES /NORESTART`, MSI `/qn`, QuietUninstallString 우선. NSIS 버튼 자동 클릭은 불안정하므로 따라 하지 않는다.
- **플랫폼 앱:** Store 앱(`Remove-AppxPackage`, IsProtected는 보호), Steam 라이브러리와 appId별 셰이더 캐시.

## 2. 저장공간 정리·분석

| 프로젝트 | 라이선스 / 기술 | 참고할 점 |
|---|---|---|
| [MangoDisk](https://github.com/harry0703/MangoDisk) | GPL-3.0 / **Rust + Tauri** | 우리와 구조·철학이 가장 비슷하다. 규칙을 TOML로 선언(위험도, 기본 선택, 꺼져 있어야 할 프로세스)하고 빌드 때 검증한다. 기본은 읽기 전용 스캔. DELETE 권한을 미리 확인해 관리자 필요 여부를 표시한다. 삭제 뒤 다시 스캔해 실제 확보량을 기록한다. |
| [BleachBit](https://github.com/bleachbit/bleachbit) | GPL-3.0 / Python | 선언형 규칙(CleanerML). 해당 프로그램이 실행 중이면 건너뛴다. 옵션마다 경고 문구를 붙인다. Windows Update 정리는 서비스를 멈췄다가 다시 켠다. |
| Windows 디스크 정리 | 내장 | `HKLM\...\Explorer\VolumeCaches` 레지스트리를 읽으면 다른 앱이 등록한 정리 대상까지 찾을 수 있다. 미리보기(크기 계산)와 실행이 분리돼 있다. |
| DISM | 내장 | `/AnalyzeComponentStore`가 "정리 권장"이라고 할 때만 `/StartComponentCleanup`을 제안한다. `/ResetBase`는 업데이트 제거를 막으므로 제공하지 않는다. WinSxS를 직접 지우면 안 된다. |
| [Czkawka](https://github.com/qarmin/czkawka) | 코어 MIT, Krokiet GUI GPL / Rust | 중복 파일: 크기 → 앞부분 해시 → 전체 해시(blake3). 크기·수정 시간 기준 해시 캐시. 빈 폴더·큰 파일·깨진 파일. `trash` 크레이트로 휴지통 삭제. |
| [fclones](https://github.com/pkolaczk/fclones) | MIT / Rust | 찾기와 지우기를 두 단계로 분리. SSD/HDD에 따라 병렬도 조정. |
| [dust](https://github.com/bootandy/dust), [dua-cli](https://github.com/Byron/dua-cli), [diskus](https://github.com/sharkdp/diskus) | Apache-2.0 / MIT / Rust | rayon 병렬 탐색, 할당 크기 계산(`filesize`), 표시 → 일괄 처리 흐름. `jwalk`는 보관(archived)됐으므로 쓰지 않는다. |
| WizTree / WinDirStat / SpaceSniffer | 프리웨어 / GPL | WizTree는 관리자 권한으로 NTFS `$MFT`를 직접 읽어 매우 빠르다(`ntfs` 크레이트, Apache-2.0). 트리맵·선버스트, 확장자별 색. |
| [kondo](https://github.com/tbillington/kondo), [npkill](https://github.com/voidcosmos/npkill), [cargo-sweep](https://github.com/holmgr/cargo-sweep) | MIT | 개발 산출물: 마커 파일이 있을 때만(`Cargo.toml` → `target`, `package.json` → `node_modules`), 오래된 프로젝트만. 앱 실행에 필요한 `node_modules`는 경고하고 기본 해제. |

### 스캔할 때 꼭 지킬 것

- 정션·심볼릭 링크는 따라가지 않는다.
- OneDrive 등 클라우드 파일(오프라인 속성)은 열지 않는다. 열면 다운로드가 시작된다.
- 하드링크는 한 번만 센다(WinSxS가 실제보다 커 보이는 이유).
- 실행 직전에 크기·수정 시간이 스캔 때와 같은지 다시 확인한다.

## 3. 이 PC 측정 (읽기 전용, 2026-10-17)

C: 928 GB 중 169 GB 남음.

| 위치 | 크기 | 메모 |
|---|---|---|
| WSL (`AppData\Local\wsl`, `Packages`) | 약 19–23 GB | 가상 디스크(vhdx)는 안에서 지워도 줄지 않는다. 압축(compact)이 필요하다. |
| Android SDK·에뮬레이터 (`.android`, `AppData\Local\Android`) | 약 21 GB | 안 쓰는 에뮬레이터 이미지(AVD)일 수 있다. |
| 사용자 Temp | 9.3 GB | 24시간 넘은 파일만 대상. |
| CapCut | 8.1 GB | 캐시 여부 확인 필요. |
| Roblox | 6.3 GB (옛 버전 폴더 2.2 GB) | 현재 버전 외 폴더는 재설치 시 다시 받는다. |
| NVIDIA 셰이더 캐시 | 4.3 GB | 자동으로 다시 만들어진다(첫 실행 때 잠깐 끊김). |
| Gradle 캐시 | 2.7 GB | 다시 받을 수 있다. |
| 메모리 덤프 `MEMORY.DMP` | 1.8 GB | 관리자 권한 필요. |
| 바탕 화면 | 44.6 GB | 프로젝트 빌드 산출물(`target`, `node_modules`)이 섞여 있을 수 있다. |

## 4. 우리 앱에 가져올 것 (우선순위)

1. **저장공간 정리 화면** — 카테고리 카드(예상 확보량, 위험도 배지, 관리자 방패), "왜 안전한가" 근거, 안전 항목만 기본 선택. 규칙은 우리가 직접 선언형으로 작성.
2. **큰 폴더 분석** — 병렬 탐색, top-N 목록, 이후 트리맵.
3. **개발 산출물 찾기** — 마커 파일 기준, 오래된 프로젝트만.
4. **BCU식 점수 보강** — 비슷한 이름 감점, 다른 설치 앱 사용 중 감점.
5. **강제 제거** — 언인스톨러 탐색 → 없으면 격리 기반 제거.
6. **Program Files 고아 폴더** — 강제 제거·저장공간 정리 공용.
7. **조용한 제거와 일괄 제거** — 복원 지점은 묶음당 1번.
8. **Store/MSIX 앱** — IsProtected는 보호.
9. **새로 설치·변경된 프로그램 강조** — 설치 추적 스냅샷과 연결.
10. **Hunter Mode** — 창 → 프로세스 → 설치 항목.
