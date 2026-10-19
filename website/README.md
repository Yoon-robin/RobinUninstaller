# RobinX 웹사이트

주소: https://yoon-robin.github.io/RobinX/

소개용 한 페이지. React + Vite + Tailwind CSS v4 + shadcn/ui(메뉴 패널) + Motion, 글꼴은 Pretendard.
앱과는 따로 설치하는 별도 프로젝트다 (`website/` 안에서 명령 실행).

```bash
pnpm install
pnpm dev                                   # 개발 서버
pnpm build                                 # dist/ 에 정적 파일
```

배포: 저장소 루트에서 `bash website/scripts/deploy.sh` → `BASE_PATH=/RobinX/`로 빌드해서 `gh-pages` 브랜치에 올린다 (GitHub Actions 안 씀).

## 디자인 규칙

- 밝은 바탕 하나, 글자색 하나, 강조색(진한 청록) 하나. 밝은 청록은 첫 화면의 무대에만 쓴다. 토큰은 `src/index.css`.
- 글자 크기는 다섯 단계(`text-fine`·`text-body`·`text-lead`·`text-section`·`text-display`)만 쓴다.
- 쓰지 않는 것: 제목 위 작은 꼬리표, 그라데이션, 쉬는 상태의 그림자·빛 번짐, 흐린 유리, 1px 테두리 상자, 아이콘 카드 줄, 고정폭 글꼴(화면 조각 밖).
- 앱 화면 조각(`data-ui`가 붙은 요소, 예: `sections/leftover-demo.tsx`)은 앱의 테두리·글꼴 규칙을 그대로 따른다.
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

## 공유 이미지 (public/og.png)

첫 화면과 같은 구성의 1200×630 그림. 바꾸면 `index.html`의 `og:image` 주소 뒤 `?v=` 숫자를 올려 SNS가 다시 받게 한다.

```bash
node website/scripts/og.mjs                    # Windows는 Edge를 자동으로 찾는다. 다른 곳은 CHROMIUM_PATH
```
