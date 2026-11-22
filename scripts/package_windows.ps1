# Build a per-user application installer; no catalogue submission.
param(
    [Parameter(Mandatory = $true)][string]$BinaryPath,
    [string]$OutputPath = "dist/windows",
    [string]$CompilerPath = "${env:ProgramFiles(x86)}\Inno Setup 6\ISCC.exe"
)
$ErrorActionPreference = 'Stop'
$RepoRoot = Split-Path -Parent $PSScriptRoot
if (-not (Test-Path -LiteralPath $CompilerPath)) {
    throw 'Inno Setup 6 is required: https://jrsoftware.org/isinfo.php'
}
$BinaryPath = (Resolve-Path -LiteralPath $BinaryPath).Path
$OutputPath = [System.IO.Path]::GetFullPath($OutputPath)
$Manifest = Get-Content -LiteralPath (Join-Path $RepoRoot 'Cargo.toml') -Raw
$Version = [regex]::Match($Manifest, '(?m)^version\s*=\s*"([^"]+)"').Groups[1].Value
if (-not $Version) { throw 'Package version not found' }
$NumericVersion = ($Version -split '-')[0]
New-Item -ItemType Directory -Path $OutputPath -Force | Out-Null
& $CompilerPath "/DAppVersion=$Version" "/DAppNumericVersion=$NumericVersion" `
    "/DBinaryPath=$BinaryPath" "/DOutputPath=$OutputPath" `
    (Join-Path $RepoRoot 'packaging/windows/abcom.iss')
if ($LASTEXITCODE -ne 0) { throw "ISCC failed: $LASTEXITCODE" }
