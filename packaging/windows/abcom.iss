; Build with ISCC /DAppVersion=... /DBinaryPath=... /DOutputPath=... abcom.iss.
#ifndef AppVersion
  #error AppVersion is required
#endif
#ifndef BinaryPath
  #error BinaryPath is required
#endif
#ifndef AppNumericVersion
  #error AppNumericVersion is required
#endif
#ifndef OutputPath
  #error OutputPath is required
#endif
; x86_64 (default, also runs emulated on Windows ARM64) or aarch64 (native ARM64).
#ifndef Arch
  #define Arch "x86_64"
#endif
#if Arch == "aarch64"
  #define WindowsArch "arm64"
#elif Arch == "x86_64"
  #define WindowsArch "x64compatible"
#else
  #error Arch must be x86_64 or aarch64
#endif

[Setup]
AppId=corp.abnd.abcom
AppName=Abcom
AppVersion={#AppVersion}
VersionInfoVersion={#AppNumericVersion}
AppPublisher=Abnd
AppPublisherURL=https://github.com/abnd-corp/Abcom
DefaultDirName={localappdata}\Programs\Abcom
DefaultGroupName=Abcom
PrivilegesRequired=lowest
ArchitecturesAllowed={#WindowsArch}
ArchitecturesInstallIn64BitMode={#WindowsArch}
OutputDir={#OutputPath}
OutputBaseFilename=abcom-windows-{#Arch}-setup
Compression=lzma2
SolidCompression=yes
UninstallDisplayIcon={app}\abcom.exe
CloseApplications=yes
RestartApplications=no
WizardStyle=modern
LicenseFile=..\..\LICENSE

[Files]
Source: "{#BinaryPath}"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{autoprograms}\Abcom"; Filename: "{app}\abcom.exe"; WorkingDir: "{app}"

[Registry]
Root: HKCU; Subkey: "Software\Microsoft\Windows\CurrentVersion\Run"; ValueName: "Abcom"; Flags: uninsdeletevalue

[Run]
Filename: "{app}\abcom.exe"; Description: "Launch Abcom"; Flags: nowait postinstall skipifsilent
