; ==============================================================================
; FamShield Windows Inno Setup Script
; Bundles: Windows Service, Session Helper, Local Admin UI
; Registers: Windows Service (SYSTEM), NRPT DnsPolicyConfig (127.0.0.2:53)
; ==============================================================================

#define MyAppName "FamShield"
#define MyAppVersion "0.1.0"
#define MyAppPublisher "FamShield Security Team"
#define MyAppURL "https://famshield.cloud"
#define MyAppExeName "famshield-service.exe"

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
OutputBaseFilename=FamShieldSetup-{#MyAppVersion}
Compression=lzma
SolidCompression=yes
WizardStyle=modern
SetupIconFile=..\assets\famshield.ico

[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"

[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"; Flags: unchecked

[Files]
Source: "..\target\release\famshield-service.exe"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\target\release\famshield-session.exe"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\assets\famshield.ico"; DestDir: "{app}"; Flags: ignoreversion
Source: "certs\FamShieldRoot.cer"; DestDir: "{app}"; Flags: ignoreversion skipifsourcedoesntexist

[Icons]
Name: "{autoprograms}\FamShield Security"; Filename: "{app}\famshield-session.exe"; IconFilename: "{app}\famshield.ico"
Name: "{autodesktop}\FamShield"; Filename: "{app}\famshield-session.exe"; IconFilename: "{app}\famshield.ico"; Tasks: desktopicon

[Dirs]
Name: "{commonappdata}\FamShield"; Permissions: users-modify

[Registry]
; Configure NRPT (Name Resolution Policy Table) to route namespace '.' to FamShield local DNS Proxy (127.0.0.2:53)
Root: HKLM; Subkey: "SOFTWARE\Policies\Microsoft\Windows NT\DnsClient\DnsPolicyConfig\{{FamShield-DNS-Filter}"; ValueType: string; ValueName: "Name"; ValueData: "."; Flags: uninsdeletekey
Root: HKLM; Subkey: "SOFTWARE\Policies\Microsoft\Windows NT\DnsClient\DnsPolicyConfig\{{FamShield-DNS-Filter}"; ValueType: string; ValueName: "GenericDNSServers"; ValueData: "127.0.0.2"; Flags: uninsdeletekey
Root: HKLM; Subkey: "SOFTWARE\Policies\Microsoft\Windows NT\DnsClient\DnsPolicyConfig\{{FamShield-DNS-Filter}"; ValueType: dword; ValueName: "ConfigOptions"; ValueData: 8; Flags: uninsdeletekey
Root: HKLM; Subkey: "SOFTWARE\Policies\Microsoft\Windows NT\DnsClient\DnsPolicyConfig\{{FamShield-DNS-Filter}"; ValueType: dword; ValueName: "Version"; ValueData: 1; Flags: uninsdeletekey

[Run]
; Auto-trust development Code Signing certificate on client machine
Filename: "{sys}\certutil.exe"; Parameters: "-addstore -f Root ""{app}\FamShieldRoot.cer"""; Flags: runhidden; Check: FileExists(ExpandConstant('{app}\FamShieldRoot.cer'))
Filename: "{sys}\certutil.exe"; Parameters: "-addstore -f TrustedPublisher ""{app}\FamShieldRoot.cer"""; Flags: runhidden; Check: FileExists(ExpandConstant('{app}\FamShieldRoot.cer'))
; Install and start Windows Service automatically under LocalSystem
Filename: "{sys}\sc.exe"; Parameters: "create FamShieldService binPath= ""{app}\famshield-service.exe"" start= auto obj= LocalSystem DisplayName= ""FamShield Security Platform"""; Flags: runhidden
Filename: "{sys}\sc.exe"; Parameters: "description FamShieldService ""Unified parental control, NUMA DNS proxy and application filtering"""; Flags: runhidden
Filename: "{sys}\sc.exe"; Parameters: "start FamShieldService"; Flags: runhidden

[UninstallRun]
; Stop and remove Windows Service cleanly
Filename: "{sys}\sc.exe"; Parameters: "stop FamShieldService"; Flags: runhidden
Filename: "{sys}\sc.exe"; Parameters: "delete FamShieldService"; Flags: runhidden

