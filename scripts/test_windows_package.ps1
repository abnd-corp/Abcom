# Native installation smoke test for disposable CI runners.
param([Parameter(Mandatory = $true)][string]$InstallerPath)
$ErrorActionPreference = 'Stop'
if ($env:CI -ne 'true') { throw 'Run this destructive installation test only on a disposable CI runner' }
$InstallerPath = (Resolve-Path -LiteralPath $InstallerPath).Path
$AppDir = Join-Path $env:LOCALAPPDATA 'Programs\Abcom'
$DataDir = Join-Path $env:LOCALAPPDATA 'abcom'
$Sentinel = Join-Path $DataDir 'packaging-test.txt'
$Shortcut = Join-Path ([Environment]::GetFolderPath('Programs')) 'Abcom.lnk'
if (Test-Path $Sentinel) { throw 'Test sentinel already exists' }
New-Item -ItemType Directory -Path $DataDir -Force | Out-Null
Set-Content -LiteralPath $Sentinel -Value 'preserve-me'
try {
    foreach ($Pass in 1..2) {
        $Process = Start-Process -FilePath $InstallerPath -ArgumentList '/VERYSILENT /SUPPRESSMSGBOXES /NORESTART /SP-' -Wait -PassThru
        if ($Process.ExitCode -ne 0) { throw "Installation failed: $($Process.ExitCode)" }
        if (-not (Test-Path (Join-Path $AppDir 'abcom.exe'))) { throw 'Missing executable' }
        if (-not (Test-Path $Shortcut)) { throw 'Missing Start menu shortcut' }
        if ((Get-Content $Sentinel).Trim() -ne 'preserve-me') { throw 'Application data changed' }
    }
    $Process = Start-Process -FilePath (Join-Path $AppDir 'unins000.exe') -ArgumentList '/VERYSILENT /SUPPRESSMSGBOXES /NORESTART' -Wait -PassThru
    if ($Process.ExitCode -ne 0) { throw "Uninstall failed: $($Process.ExitCode)" }
    if (Test-Path (Join-Path $AppDir 'abcom.exe')) { throw 'Executable left behind' }
    if (Test-Path $Shortcut) { throw 'Shortcut left behind' }
    if ((Get-Content $Sentinel).Trim() -ne 'preserve-me') { throw 'Uninstall removed application data' }
} finally {
    Remove-Item -LiteralPath $Sentinel -ErrorAction SilentlyContinue
}
