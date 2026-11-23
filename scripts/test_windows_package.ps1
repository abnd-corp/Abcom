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
        Add-Type -AssemblyName System.Drawing
        $Icon = [System.Drawing.Icon]::ExtractAssociatedIcon((Join-Path $AppDir 'abcom.exe'))
        if ($null -eq $Icon) { throw 'Windows shell could not extract the application icon' }
        try {
            $Bitmap = $Icon.ToBitmap()
            try {
                if ($Bitmap.Width -lt 16 -or $Bitmap.Width -ne $Bitmap.Height) { throw 'Invalid shell icon dimensions' }
                $HasCoral = $false
                for ($Y = 0; $Y -lt $Bitmap.Height; $Y++) {
                    for ($X = 0; $X -lt $Bitmap.Width; $X++) {
                        $Pixel = $Bitmap.GetPixel($X, $Y)
                        if ($Pixel.A -gt 128 -and $Pixel.R -gt 230 -and $Pixel.G -gt 100 -and $Pixel.G -lt 180 -and $Pixel.B -gt 60 -and $Pixel.B -lt 140) { $HasCoral = $true }
                    }
                }
                if (-not $HasCoral) { throw 'Windows shell icon does not contain the Abcom coral mark' }
                Write-Output "Windows shell: Abcom icon extracted ($($Bitmap.Width)x$($Bitmap.Height))"
            } finally { $Bitmap.Dispose() }
        } finally { $Icon.Dispose() }

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
