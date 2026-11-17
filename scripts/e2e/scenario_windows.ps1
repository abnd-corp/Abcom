# Scénario e2e natif Windows, exécuté dans la session interactive de test : lance deux instances isolées de
# l'application réelle et vérifie démarrage, fenêtre, découverte, tray et capture.
# Usage : scenario_windows.ps1 -Binary C:\...\abcom.exe -Out C:\...\resultats
param(
    [Parameter(Mandatory = $true)][string]$Binary,
    [Parameter(Mandatory = $true)][string]$Out
)
$ErrorActionPreference = 'Continue'
New-Item -ItemType Directory -Force $Out | Out-Null
$script:fails = 0

function Check([string]$Name, [scriptblock]$Condition) {
    if (& $Condition) { "PASS $Name" } else { "FAIL $Name"; $script:fails++ }
}

$dataRoot = $env:APPDATA
foreach ($n in 91, 92) {
    if (Test-Path (Join-Path $dataRoot "abcom-$n")) {
        "FAIL données abcom-$n déjà présentes : scénario refusé pour ne rien écraser"
        exit 2
    }
}

# Instances 91 et 92 : dossiers de données et ports distincts de ceux de l'utilisateur.
function Start-Instance([int]$Instance, [string]$Name) {
    $log = Join-Path $Out "$Name.log"
    $line = "/c set ABCOM_INSTANCE=$Instance&& set RUST_LOG=abcom=debug&& `"$Binary`" $Name > `"$log`" 2>&1"
    $shell = Start-Process cmd.exe -ArgumentList $line -WindowStyle Hidden -PassThru
    for ($i = 0; $i -lt 40; $i++) {
        $child = Get-CimInstance Win32_Process -Filter "ParentProcessId = $($shell.Id)" | Where-Object Name -like 'abcom*' | Select-Object -First 1
        if ($child) { return [int]$child.ProcessId }
        Start-Sleep -Milliseconds 250
    }
    return 0
}

$a = Start-Instance 91 'e2e-a'
Start-Sleep 3
$b = Start-Instance 92 'e2e-b'
Start-Sleep 25

$logA = Join-Path $Out 'e2e-a.log'
$logB = Join-Path $Out 'e2e-b.log'
Check 'instance A en vie' { $a -and (Get-Process -Id $a -ErrorAction SilentlyContinue) }
Check 'instance B en vie' { $b -and (Get-Process -Id $b -ErrorAction SilentlyContinue) }
Check 'fenêtre de A' { $a -and (Get-Process -Id $a -ErrorAction SilentlyContinue).MainWindowHandle -ne 0 }
Check 'fenêtre de B' { $b -and (Get-Process -Id $b -ErrorAction SilentlyContinue).MainWindowHandle -ne 0 }
Check 'A découvre B' { Select-String -Path $logA -Pattern 'pair découvert : e2e-b' -Quiet -Encoding UTF8 }
Check 'B découvre A' { Select-String -Path $logB -Pattern 'pair découvert : e2e-a' -Quiet -Encoding UTF8 }
Check 'aucune panique' { -not (Select-String -Path $logA, $logB -Pattern 'panique|panicked' -Quiet -Encoding UTF8) }
Check 'tray créé pour A' { -not (Select-String -Path $logA -Pattern 'icône résidente indisponible' -Quiet -Encoding UTF8) }
Check 'tray créé pour B' { -not (Select-String -Path $logB -Pattern 'icône résidente indisponible' -Quiet -Encoding UTF8) }

try {
    Add-Type -AssemblyName System.Windows.Forms, System.Drawing
    $bounds = [System.Windows.Forms.Screen]::PrimaryScreen.Bounds
    $bitmap = New-Object System.Drawing.Bitmap $bounds.Width, $bounds.Height
    $graphics = [System.Drawing.Graphics]::FromImage($bitmap)
    $graphics.CopyFromScreen($bounds.Location, [System.Drawing.Point]::Empty, $bounds.Size)
    $bitmap.Save((Join-Path $Out 'screen.png'), [System.Drawing.Imaging.ImageFormat]::Png)
    'INFO capture : screen.png'
} catch {
    "INFO capture indisponible : $($_.Exception.Message)"
}

# La croix replie dans le tray sous Windows : l'arrêt du scénario est forcé, sans valeur de test.
foreach ($id in $a, $b) { if ($id) { Stop-Process -Id $id -Force -ErrorAction SilentlyContinue } }
Start-Sleep 2
foreach ($n in 91, 92) {
    $dir = Join-Path $dataRoot "abcom-$n"
    Copy-Item (Join-Path $dir 'logs') (Join-Path $Out "logs-$n") -Recurse -ErrorAction SilentlyContinue
    Remove-Item $dir -Recurse -Force -ErrorAction SilentlyContinue
}

"RESULT échecs=$script:fails"
if ($script:fails -gt 0) { exit 1 }
exit 0
