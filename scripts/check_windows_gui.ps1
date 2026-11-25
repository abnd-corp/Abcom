# Refuse a console-subsystem Windows binary: it opens a terminal whose closing quits the app.
param([Parameter(Mandatory = $true)][string]$BinaryPath)
$ErrorActionPreference = 'Stop'
$bytes = [System.IO.File]::ReadAllBytes((Resolve-Path -LiteralPath $BinaryPath).Path)
$pe = [System.BitConverter]::ToInt32($bytes, 0x3C)
# Subsystem: offset 68 of the optional header, which follows the 24-byte PE signature and file header.
$subsystem = [System.BitConverter]::ToUInt16($bytes, $pe + 24 + 68)
if ($subsystem -ne 2) {
    throw "$BinaryPath uses PE subsystem $subsystem; expected 2 (Windows GUI)"
}
"$BinaryPath : Windows GUI subsystem"
