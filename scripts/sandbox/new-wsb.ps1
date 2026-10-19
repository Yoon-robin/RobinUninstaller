# Windows Sandbox 설정 파일(.wsb)을 만든다. 만들기만 하고 Sandbox를 켜지는 않는다.
#
#   powershell -ExecutionPolicy Bypass -File scripts\sandbox\new-wsb.ps1 [-NoNetwork]
#
# 결과: scripts\sandbox\out\robinx-test.wsb (더블클릭하면 Sandbox가 열린다)
# Sandbox 안에서 보이는 폴더:
#   C:\Kit\app         빌드한 robinx.exe (읽기 전용)   ← pnpm tauri build --no-bundle 결과를 복사
#   C:\Kit\installers  시험할 설치 파일 (읽기 전용)     ← download-installers.ps1로 미리 받아 둔다
#   C:\Kit\shared      결과 캡처·메모를 꺼내 오는 곳 (쓰기 가능, 이 폴더만 PC와 이어진다)
# .wsb는 상대 경로를 받지 않아서, 이 저장소의 실제 위치로 경로를 채워 만든다.

param([switch]$NoNetwork)

$ErrorActionPreference = 'Stop'
$root = (Resolve-Path (Join-Path $PSScriptRoot '..\..')).Path
$out = Join-Path $PSScriptRoot 'out'
$app = Join-Path $out 'app'
$installers = Join-Path $out 'installers'
$shared = Join-Path $out 'shared'
New-Item -ItemType Directory -Force -Path $app, $installers, $shared | Out-Null

$exe = Join-Path $root 'target\release\robinx.exe'
if (Test-Path $exe) {
  Copy-Item $exe (Join-Path $app 'robinx.exe') -Force
} else {
  Write-Warning "target\release\robinx.exe가 없어요. 먼저 pnpm tauri build --no-bundle 로 빌드하세요."
}
if (-not (Get-ChildItem $installers -File -ErrorAction SilentlyContinue)) {
  Write-Warning "설치 파일이 없어요. download-installers.ps1로 먼저 받으세요."
}

function Esc([string]$s) { [System.Security.SecurityElement]::Escape($s) }
function Mapped([string]$host_, [string]$inside, [bool]$readOnly) {
  @"
    <MappedFolder>
      <HostFolder>$(Esc $host_)</HostFolder>
      <SandboxFolder>$(Esc $inside)</SandboxFolder>
      <ReadOnly>$($readOnly.ToString().ToLower())</ReadOnly>
    </MappedFolder>
"@
}

$networking = if ($NoNetwork) { 'Disable' } else { 'Default' }
$wsb = @"
<Configuration>
  <VGpu>Default</VGpu>
  <Networking>$networking</Networking>
  <MemoryInMB>4096</MemoryInMB>
  <ClipboardRedirection>Enable</ClipboardRedirection>
  <MappedFolders>
$(Mapped $app 'C:\Kit\app' $true)
$(Mapped $installers 'C:\Kit\installers' $true)
$(Mapped $shared 'C:\Kit\shared' $false)
  </MappedFolders>
  <LogonCommand>
    <Command>explorer.exe C:\Kit</Command>
  </LogonCommand>
</Configuration>
"@

$target = Join-Path $out 'robinx-test.wsb'
# Sandbox는 BOM 없는 UTF-8도 읽지만, 경로에 한글이 있을 수 있어 BOM을 붙인다
[System.IO.File]::WriteAllText($target, $wsb, (New-Object System.Text.UTF8Encoding($true)))
Write-Output "만듦: $target"
