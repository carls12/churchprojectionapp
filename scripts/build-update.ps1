param(
  [Parameter(Mandatory = $true)]
  [ValidatePattern('^\d+\.\d+\.\d+$')]
  [string]$Version
)

$ErrorActionPreference = 'Stop'
$root = Split-Path -Parent $PSScriptRoot
Set-Location $root

npm.cmd version $Version --no-git-tag-version --allow-same-version

$tauriPath = Join-Path $root 'src-tauri\tauri.conf.json'
$tauri = Get-Content $tauriPath -Raw -Encoding UTF8 | ConvertFrom-Json
$tauri.version = $Version
$utf8 = New-Object System.Text.UTF8Encoding($false)
[System.IO.File]::WriteAllText($tauriPath, ($tauri | ConvertTo-Json -Depth 20), $utf8)

$cargoPath = Join-Path $root 'src-tauri\Cargo.toml'
$cargo = Get-Content $cargoPath -Raw -Encoding UTF8
$cargo = $cargo -replace '(?m)^(version\s*=\s*")[^"]+("\s*)$', "`${1}$Version`${2}"
[System.IO.File]::WriteAllText($cargoPath, $cargo, $utf8)

$appPath = Join-Path $root 'src\App.tsx'
$app = Get-Content $appPath -Raw -Encoding UTF8
$app = $app -replace 'Version \d+\.\d+\.\d+', "Version $Version"
[System.IO.File]::WriteAllText($appPath, $app, $utf8)

$devShell = 'C:\BuildTools\Common7\Tools\VsDevCmd.bat'
if (-not (Test-Path $devShell)) { throw "Visual Studio Build Tools not found at $devShell" }

cmd.exe /d /s /c "`"$devShell`" -arch=x64 && set PATH=%USERPROFILE%\.cargo\bin;%PATH% && npm.cmd run check && npm.cmd run tauri build -- --bundles nsis"
if ($LASTEXITCODE -ne 0) { throw "Release build failed with exit code $LASTEXITCODE" }

Write-Host "Update installer created for version $Version" -ForegroundColor Green
