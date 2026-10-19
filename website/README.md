# RobinX 웹사이트

주소: https://yoon-robin.github.io/RobinUninstaller/

소개용 랜딩 페이지. React + Vite + Tailwind CSS v4 + shadcn/ui + Motion. 앱과 같은 청록 다크 테마와 Pretendard 글꼴을 쓴다.
앱과는 따로 설치하는 별도 프로젝트다 (`website/` 안에서 명령 실행).

```bash
pnpm install
pnpm dev                                   # 개발 서버
pnpm build                                 # dist/ 에 정적 파일
```

배포: 저장소 루트에서 `bash website/scripts/deploy.sh` → 빌드해서 `gh-pages` 브랜치에 올린다 (GitHub Actions 안 씀).

## 앱 화면 (public/shots)

실제 PC의 프로그램 이름이 드러나지 않도록 **가상의 프로그램**을 가짜 응답으로 넣어 캡처한다.
앱 화면이 바뀌면 저장소 루트에서 다시 찍는다.

```bash
pnpm exec vite --port 1420 --strictPort        # 앱 개발 서버 (pnpm tauri dev 중이면 생략)
node website/scripts/capture.mjs               # 서버가 localhost(::1)에만 열리면 UI_MOCK_URL=http://localhost:1420/
```
