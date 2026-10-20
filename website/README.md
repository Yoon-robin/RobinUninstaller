# RobinX 웹사이트

주소: https://yoon-robin.github.io/RobinX/

소개용 한 페이지. React + Vite + Tailwind CSS v4 + shadcn/ui(메뉴 패널) + Motion, 글꼴은 Pretendard.
앱과는 따로 설치하는 별도 프로젝트다 (`website/` 안에서 명령 실행).

```bash
pnpm install
pnpm dev                                   # 개발 서버
pnpm build                                 # dist/ 에 정적 파일
```

배포: 저장소 루트에서 `bash website/scripts/deploy.sh` → 최신 릴리스 정보를 가져와(`scripts/release-info.mjs` → `src/lib/release.json`) `BASE_PATH=/RobinX/`로 빌드해서 `gh-pages` 브랜치에 올린다 (GitHub Actions 안 씀).
앱을 새로 릴리스한 뒤에는 사이트도 다시 배포해야 다운로드 단추·버전·SHA-256이 새 릴리스를 가리킨다.

## 디자인 규칙

Apple 제품 페이지 + shadcn (2026-10-20에 다시 잡고, 같은 날 리뷰 명세에 따라 구역마다 모양을 달리함).

- 흰 바탕과 연회색(`--muted`, #F5F5F7) 구역을 번갈아 쓴다: 첫 화면(흰색) → 남은 흔적(회색) → 기능(흰색) → 장면(회색) → 되돌리기(흰색) → 개발 현황(회색) → FAQ(흰색) → 내려받기(어두운 바탕).
- 구역마다 모양을 다르게 해 같은 제목+카드가 되풀이되지 않게 한다: 첫 화면은 가운데 제목과 큰 앱 화면, 남은 흔적은 왼쪽 앱 화면 확대 + 오른쪽 설명, 기능은 크기가 다른 타일, 되돌리기는 표, 개발 현황은 릴리스 판 + 세 목록, FAQ는 왼쪽 제목 + 두 무리.
- 흰 구역 위 타일은 회색, 회색 구역 위 타일은 흰색으로, 선이나 그림자 없이 톤 차이로 나눈다. 표의 줄도 선 대신 여백으로 나눈다.
- 첫 화면 그림은 이 앱만의 장면인 "남은 흔적" 창이다. 좁은 화면에서는 창 부분만 잘라 크게 보여 준다.
- 내려받기 단추는 설치 파일로 바로 이어지고, 옆에 버전·크기·지원 환경·미리보기·서명 상태와 릴리스 노트(SHA-256) 링크를 둔다. 보안 경고를 그냥 넘기라고 쓰지 않는다: 공식 릴리스에서 받고 SHA-256을 확인하라고 쓴다.
- 움직임은 첫 화면에만 눈에 띄게, 나머지 구역은 짧고 작게 떠오른다. 영상은 늘 재생·일시정지 단추가 있고, 화면에 보일 때만 재생하며, 움직임 줄이기면 저절로 재생하지 않는다.
- 글자색 하나, 강조색(진한 청록) 하나. 강조색은 실행 버튼과 링크에만 쓴다.
- 글자 크기는 다섯 단계(`text-fine`·`text-body`·`text-lead`·`text-section`·`text-display`)만 쓴다. 가운데 정렬 문단은 `text-balance`로 줄을 고르게.
- 앱 화면 그림은 밝은 테마로 찍는다 (`scripts/capture.mjs`, `CAPTURE_SCHEME=dark`로 바꿀 수 있음). 타일 안의 그림은 사이드바를 빼고 내용 부분만 확대해 자른다.
- 쓰지 않는 것: 제목 위 작은 꼬리표, 그라데이션, 쉬는 상태의 그림자·빛 번짐, 흐린 유리, 1px 테두리 상자, 아이콘 카드 줄, 고정폭 글꼴(화면 조각 밖).
- 앱 화면 조각(`data-ui`가 붙은 요소, 예: 남은 흔적 구역의 표시)은 앱의 모양을 그대로 따른다.
- 문구는 코드와 대조해 사실만 쓴다. 과장, "X가 아니라 Y" 식 대비, 상투어(한곳에, 올인원)는 쓰지 않는다. 화면 속 숫자는 예시라고 밝힌다.

바꾼 뒤에는 [taste-skill](https://github.com/Leonxlnx/taste-skill)(MIT) 검사기로 확인한다. FAIL이 0개여야 한다 (영상이 없다는 WARN은 괜찮다).

```bash
git clone --depth 1 https://github.com/Leonxlnx/taste-skill.git "$TEMP/taste-skill"   # 처음 한 번 (저장소에 넣지 않음)
pnpm --dir website build                                                            # BASE_PATH 없이
node "$TEMP/taste-skill/skills/taste-skill/scripts/lint.mjs" website/src
(cd website/dist && node "$TEMP/taste-skill/skills/taste-skill/scripts/inspect.mjs" index.html --out "$TEMP/taste-out")
```

검사기는 지금 폴더를 웹 서버의 뿌리로 쓰므로 `website/dist` 안에서 실행해야 한다(밖에서 실행하면 `/assets/...`를 못 찾아 빈 페이지로 검사한다).
1440·768·390px 화면을 `$TEMP/taste-out`에 찍는다. 눈으로도 본다.

## 앱 화면 (public/shots)

실제 PC의 프로그램 이름이 드러나지 않도록 **가상의 프로그램**을 가짜 응답으로 넣어 캡처한다.
앱 화면이 바뀌면 저장소 루트에서 다시 찍는다.

```bash
pnpm exec vite --port 1420 --strictPort        # 앱 개발 서버 (pnpm tauri dev 중이면 생략)
node website/scripts/capture.mjs               # 서버가 localhost(::1)에만 열리면 UI_MOCK_URL=http://localhost:1420/
```

## 앱 녹화 (public/demo)

"제거한 뒤 정리하는 장면" 영상(소리 없음, 반복)과 포스터. 가짜 응답(테스트 앱 Fixture Notes)으로만 찍는다.

```bash
pnpm exec vite --port 1420 --strictPort                      # 앱 개발 서버
node node_modules/playwright-core/cli.js install ffmpeg      # 처음 한 번
CHROMIUM_PATH=<chrome.exe> node scripts/ui-mock/record-demo.mjs   # 앞부분을 잘라 VP9로 다시 인코딩
```

## 공유 이미지 (public/og.png)

첫 화면과 같은 구성의 1200×630 그림. 바꾸면 `index.html`의 `og:image` 주소 뒤 `?v=` 숫자를 올려 SNS가 다시 받게 한다.

```bash
node website/scripts/og.mjs                    # Windows는 Edge를 자동으로 찾는다. 다른 곳은 CHROMIUM_PATH
```
