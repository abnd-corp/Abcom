# Refuse a Windows binary built for the console subsystem: it would open a terminal beside the
# window, and closing that terminal would quit the application.
param([Parameter(Mandatory = $true)][string]$BinaryPath)
$ErrorActionPreference = 'Stop'
$bytes = [System.IO.File]::ReadAllBytes((Resolve-Path -LiteralPath $BinaryPath).Path)
$pe = [System.BitConverter]::ToInt32($bytes, 0x3C)
# Optional header starts 24 bytes after the PE signature; Subsystem is at offset 68 in it.
$subsystem = [System.BitConverter]::ToUInt16($bytes, $pe + 24 + 68)
if ($subsystem -ne 2) {
    throw "$BinaryPath uses PE subsystem $subsystem; expected 2 (Windows GUI)"
}
"$BinaryPath : Windows GUI subsystem"
