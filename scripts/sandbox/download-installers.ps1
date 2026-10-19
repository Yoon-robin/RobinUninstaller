# programs.csv의 설치 파일을 winget으로 "받기만" 한다 (이 PC에는 설치하지 않는다). C2에서 사용자 승인 뒤 실행.
#
#   powershell -ExecutionPolicy Bypass -File scripts\sandbox\download-installers.ps1 [-Only 7zip.7zip,VideoLAN.VLC]
#
# 받은 파일: scripts\sandbox\out\installers\<id>\ (git에 넣지 않는다)
# winget download는 공식 저장소(winget-pkgs)에 적힌 주소에서 받고 해시를 확인한다.

param([string[]]$Only)

$ErrorActionPreference = 'Stop'
# 이 셸에 PATH가 없을 수 있어 앱 실행 별칭을 직접 찾는다
$winget = (Get-Command winget -ErrorAction SilentlyContinue).Source
if (-not $winget) { $winget = Join-Path $env:LOCALAPPDATA 'Microsoft\WindowsApps\winget.exe' }
if (-not (Test-Path $winget)) { throw 'winget(앱 설치 관리자)이 없어요.' }

$dest = Join-Path $PSScriptRoot 'out\installers'
$programs = Import-Csv (Join-Path $PSScriptRoot 'programs.csv')
if ($Only) { $programs = $programs | Where-Object { $Only -contains $_.id } }

foreach ($p in $programs) {
  $dir = Join-Path $dest $p.id
  if (Get-ChildItem $dir -File -ErrorAction SilentlyContinue) {
    Write-Output "이미 있음: $($p.name)"
    continue
  }
  Write-Output "받는 중: $($p.name) ($($p.id))"
  & $winget download --id $p.id --exact --download-directory $dir `
    --accept-package-agreements --accept-source-agreements --disable-interactivity
  if ($LASTEXITCODE -ne 0) { Write-Warning "받지 못함: $($p.id) (종료 코드 $LASTEXITCODE)" }
}
