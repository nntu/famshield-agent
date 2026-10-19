; ==============================================================================
; ChildProtect Windows Inno Setup Script
; Bundles: Windows Service, Session Helper, Local Admin UI
; Registers: Windows Service (SYSTEM), NRPT DnsPolicyConfig (127.0.0.2:53)
; ==============================================================================

#define MyAppName "ChildProtect"
#define MyAppVersion "0.1.0"
#define MyAppPublisher "ChildProtect Security Team"
#define MyAppURL "https://childprotect.cloud"
#define MyAppExeName "childprotect-service.exe"

[Setup]
AppId={{D377B812-421E-4E3E-81BA-8D2930219EF4}
AppName={#MyAppName}
AppVersion={#MyAppVersion}
AppPublisher={#MyAppPublisher}
AppPublisherURL={#MyAppURL}
AppSupportURL={#MyAppURL}
AppUpdatesURL={#MyAppURL}
DefaultDirName={autopf}\{#MyAppName}
DisableProgramGroupPage=yes
LicenseFile=..\README.md
PrivilegesRequired=admin
OutputBaseFilename=ChildProtectSetup-{#MyAppVersion}
Compression=lzma
SolidCompression=yes
WizardStyle=modern

[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"

[Files]
; Placeholders for built binaries
; Source: "..\target\release\childprotect-service.exe"; DestDir: "{app}"; Flags: ignoreversion
; Source: "..\target\release\childprotect-session.exe"; DestDir: "{app}"; Flags: ignoreversion

[Run]
; Install and start Windows Service automatically
Filename: "{sys}\sc.exe"; Parameters: "create ChildProtectService binPath= ""{app}\childprotect-service.exe"" start= auto obj= LocalSystem"; Flags: runhidden
Filename: "{sys}\sc.exe"; Parameters: "start ChildProtectService"; Flags: runhidden

[UninstallRun]
; Stop and remove Windows Service cleanly
Filename: "{sys}\sc.exe"; Parameters: "stop ChildProtectService"; Flags: runhidden
Filename: "{sys}\sc.exe"; Parameters: "delete ChildProtectService"; Flags: runhidden

[Registry]
; Clean up NRPT DnsPolicyConfig upon uninstallation
Root: HKLM; Subkey: "SOFTWARE\Policies\Microsoft\Windows NT\DnsClient\DnsPolicyConfig\{ChildProtect-DNS-Filter}"; Flags: uninsdeletekey
