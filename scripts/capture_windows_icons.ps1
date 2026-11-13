# Native screenshots on disposable Windows CI, never on a personal desktop.
param(
    [Parameter(Mandatory = $true)][string]$BinaryPath,
    [string]$OutputPath = 'dist/visual-icons'
)
$ErrorActionPreference = 'Stop'
if ($env:CI -ne 'true') { throw 'Use a disposable CI runner' }
$BinaryPath = (Resolve-Path -LiteralPath $BinaryPath).Path
New-Item -ItemType Directory -Path $OutputPath -Force | Out-Null
$OutputPath = (Resolve-Path -LiteralPath $OutputPath).Path
Add-Type -AssemblyName System.Windows.Forms
Add-Type -AssemblyName System.Drawing
foreach ($Theme in @('light', 'dark')) {
    foreach ($State in @('normal', 'unread')) {
        $Arguments = @()
        if ($Theme -eq 'dark') { $Arguments += '--dark' }
        if ($State -eq 'unread') { $Arguments += '--unread' }
        $Parameters = @{
            FilePath = $BinaryPath
            PassThru = $true
            RedirectStandardOutput = (Join-Path $OutputPath "$Theme-$State.log")
            RedirectStandardError = (Join-Path $OutputPath "$Theme-$State-errors.log")
        }
        if ($Arguments.Count -gt 0) { $Parameters.ArgumentList = $Arguments }
        $Process = Start-Process @Parameters
        try {
            Start-Sleep -Seconds 4
            $Process.Refresh()
            if ($Process.HasExited) { throw 'Native icon probe exited before capture' }
            if ($Process.MainWindowHandle -eq [IntPtr]::Zero) { throw 'No interactive native window available on this runner' }
            $Bounds = [System.Windows.Forms.SystemInformation]::VirtualScreen
            $Bitmap = New-Object System.Drawing.Bitmap $Bounds.Width, $Bounds.Height
            try {
                $Graphics = [System.Drawing.Graphics]::FromImage($Bitmap)
                try {
                    $Graphics.CopyFromScreen($Bounds.Location, [System.Drawing.Point]::Empty, $Bounds.Size)
                    $Bitmap.Save((Join-Path $OutputPath "$Theme-$State.png"), [System.Drawing.Imaging.ImageFormat]::Png)
                    Write-Output "$Theme-$State desktop captured; native DPI $($Graphics.DpiX)x$($Graphics.DpiY)"
                } finally { $Graphics.Dispose() }
            } finally { $Bitmap.Dispose() }
            if (-not $Process.WaitForExit(15000)) { throw 'Native probe did not exit' }
            if ($Process.ExitCode -ne 0) { throw "Native probe failed: $($Process.ExitCode)" }
        } finally {
            if (-not $Process.HasExited) { $Process.Kill(); $Process.WaitForExit() }
        }
    }
}
