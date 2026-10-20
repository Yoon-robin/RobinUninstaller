#!/usr/bin/env bash
# 웹사이트를 GitHub Pages(gh-pages 브랜치)에 올린다. GitHub Actions는 쓰지 않는다.
#   bash website/scripts/deploy.sh
set -euo pipefail
cd "$(dirname "$0")/.."

# 내려받기 단추·버전·SHA-256을 최신 릴리스에 맞춘다 (가져오지 못하면 저장소의 값을 쓴다)
node scripts/release-info.mjs

# 폴더 자체는 다른 프로그램이 열고 있을 수 있어 안의 내용만 지운다
[ -d dist ] && find dist -mindepth 1 -delete
# Git Bash가 /RobinX/ 를 Windows 경로로 바꾸지 않게 한다
MSYS_NO_PATHCONV=1 MSYS2_ARG_CONV_EXCL="*" BASE_PATH=/RobinX/ pnpm build
touch dist/.nojekyll

cd dist
git init -q -b gh-pages
git add -A
# 커밋 작성자는 원래 저장소 설정을 그대로 쓴다
git -c core.autocrlf=false -c user.name="$(git -C ../.. config user.name)" -c user.email="$(git -C ../.. config user.email)" commit -q -m "Deploy website"
git push -q -f "$(git -C ../.. remote get-url origin)" gh-pages
rm -rf .git
echo "배포함: https://yoon-robin.github.io/RobinX/ (반영까지 1분쯤)"
