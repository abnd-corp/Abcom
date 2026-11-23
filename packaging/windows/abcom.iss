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
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
OutputDir={#OutputPath}
OutputBaseFilename=abcom-windows-x86_64-setup
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
