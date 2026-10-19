# 가짜 응답으로 화면 캡처하기

Windows 없이(클라우드 세션 등) 화면을 확인하는 도구. 실제 엔진을 부르지 않고 `mock.mjs`의 가짜 데이터를 돌려준다.
실제 동작 확인은 Windows PC에서 `pnpm tauri dev`로 한다 (CLAUDE.md 참고).

```bash
bash scripts/cloud-setup.sh                         # 처음 한 번 (pnpm install 포함)
pnpm exec vite --port 1420 --strictPort &           # 개발 서버
node scripts/ui-mock/shoot.mjs                      # 모든 화면 → scripts/ui-mock/out/*.png (커밋하지 않음)
node scripts/ui-mock/shoot.mjs programs settings    # 일부만
pkill -f "[v]ite --port 1420"                       # 끝나면 서버 끄기 (pkill -f "vite ..."는 자기 셸까지 죽인다)
```

- Chromium 위치는 `CHROMIUM_PATH`로 바꾼다 (기본: 클라우드 컨테이너의 `/opt/pw-browsers/...`).
- 새 기능을 시험할 때는 `openApp(browser, { overrides })`로 응답을 바꾼 스크립트를 따로 만든다.
  응답이 인자에 따라 달라져야 하면 `"(args) => ..."` 문자열로 준다. 예: 제거 진행을 흉내 내려면
  `uninstall_app: "(args) => new Promise(r => setTimeout(() => r({ kind: 'removed' }), 700))"`.
- 앱은 자체 테마 설정을 따르므로 `scheme: "light"`만으로는 밝은 화면이 되지 않는다. 화면 오른쪽 위 테마 버튼을 눌러 바꾼다.
