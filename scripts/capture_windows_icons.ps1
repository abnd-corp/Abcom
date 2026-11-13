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
Add-Type -AssemblyName UIAutomationClient
Add-Type -AssemblyName UIAutomationTypes
Add-Type @'
using System;
using System.Runtime.InteropServices;
public static class VisualMouse {
    [DllImport("user32.dll")] public static extern bool SetCursorPos(int x, int y);
    [DllImport("user32.dll")] public static extern void mouse_event(uint flags, uint dx, uint dy, uint data, UIntPtr extra);
}
'@
$Shell = New-Object -ComObject Shell.Application
$Shell.MinimizeAll()
function Show-NotificationIcons {
    $Root = [System.Windows.Automation.AutomationElement]::RootElement
    $Scope = [System.Windows.Automation.TreeScope]::Descendants
    $NameProperty = [System.Windows.Automation.AutomationElement]::NameProperty
    $Existing = $Root.FindFirst($Scope, (New-Object System.Windows.Automation.PropertyCondition $NameProperty, 'Abcom'))
    if ($null -ne $Existing -and -not $Existing.Current.IsOffscreen) { return }
    foreach ($Name in @('Show hidden icons', 'Hidden icon menu', 'Notification Chevron')) {
        $Button = $Root.FindFirst($Scope, (New-Object System.Windows.Automation.PropertyCondition $NameProperty, $Name))
        if ($null -ne $Button -and -not $Button.Current.IsOffscreen) {
            $Rect = $Button.Current.BoundingRectangle
            [VisualMouse]::SetCursorPos([int]($Rect.X + $Rect.Width / 2), [int]($Rect.Y + $Rect.Height / 2)) | Out-Null
            [VisualMouse]::mouse_event(2, 0, 0, 0, [UIntPtr]::Zero)
            [VisualMouse]::mouse_event(4, 0, 0, 0, [UIntPtr]::Zero)
            [VisualMouse]::SetCursorPos(0, 0) | Out-Null
            Start-Sleep -Milliseconds 500
            Write-Output "Notification overflow opened: $Name"
            return
        }
    }
    throw 'Notification overflow button unavailable; tray badge cannot be certified'
}
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
            Show-NotificationIcons
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

$Shell.UndoMinimizeAll()
