; ==============================================================================
; FamShield Windows Inno Setup Script
; Bundles: Windows Service, Session Helper, Local Admin UI
; Registers: Windows Service (SYSTEM), NRPT DnsPolicyConfig (127.0.0.2:53)
; ==============================================================================

#define MyAppName "FamShield"
#define MyAppVersion "1.0.1"
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
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
DisableProgramGroupPage=yes
; Stealth mode: Do NOT register into Windows Add/Remove Programs (Control Panel / Settings -> Apps)
CreateUninstallRegKey=no
UpdateUninstallLogAppName=no
CloseApplications=no
RestartApplications=no
LicenseFile=..\LICENSE
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
Source: "..\web\dist\*"; DestDir: "{app}\web\dist"; Flags: ignoreversion recursesubdirs createallsubdirs skipifsourcedoesntexist
Source: "..\assets\famshield.ico"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{autoprograms}\FamShield Security"; Filename: "{app}\famshield-session.exe"; IconFilename: "{app}\famshield.ico"
Name: "{autodesktop}\FamShield"; Filename: "{app}\famshield-session.exe"; IconFilename: "{app}\famshield.ico"; Tasks: desktopicon

[Dirs]
Name: "{commonappdata}\FamShield"; Permissions: users-modify

[Registry]
; Clean up any legacy malformed NRPT keys
Root: HKLM; Subkey: "SOFTWARE\Policies\Microsoft\Windows NT\DnsClient\DnsPolicyConfig\{{FamShield-DNS-Filter}"; Flags: deletekey uninsdeletekey
Root: HKLM; Subkey: "SYSTEM\CurrentControlSet\Services\Dnscache\Parameters\DnsPolicyConfig\{{FamShield-DNS-Filter}"; Flags: deletekey uninsdeletekey

; Configure valid RFC/Win32-compliant NRPT rule for namespace '.' to FamShield local DNS Proxy (127.0.0.2:53)
Root: HKLM; Subkey: "SOFTWARE\Policies\Microsoft\Windows NT\DnsClient\DnsPolicyConfig\{{E6B5D7A1-8843-4C49-B944-C5200E481230}"; ValueType: multisz; ValueName: "Name"; ValueData: "."; Flags: uninsdeletekey
Root: HKLM; Subkey: "SOFTWARE\Policies\Microsoft\Windows NT\DnsClient\DnsPolicyConfig\{{E6B5D7A1-8843-4C49-B944-C5200E481230}"; ValueType: string; ValueName: "GenericDNSServers"; ValueData: "127.0.0.2"; Flags: uninsdeletekey
Root: HKLM; Subkey: "SOFTWARE\Policies\Microsoft\Windows NT\DnsClient\DnsPolicyConfig\{{E6B5D7A1-8843-4C49-B944-C5200E481230}"; ValueType: dword; ValueName: "ConfigOptions"; ValueData: 8; Flags: uninsdeletekey
Root: HKLM; Subkey: "SOFTWARE\Policies\Microsoft\Windows NT\DnsClient\DnsPolicyConfig\{{E6B5D7A1-8843-4C49-B944-C5200E481230}"; ValueType: dword; ValueName: "Version"; ValueData: 2; Flags: uninsdeletekey

Root: HKLM; Subkey: "SYSTEM\CurrentControlSet\Services\Dnscache\Parameters\DnsPolicyConfig\{{E6B5D7A1-8843-4C49-B944-C5200E481230}"; ValueType: multisz; ValueName: "Name"; ValueData: "."; Flags: uninsdeletekey
Root: HKLM; Subkey: "SYSTEM\CurrentControlSet\Services\Dnscache\Parameters\DnsPolicyConfig\{{E6B5D7A1-8843-4C49-B944-C5200E481230}"; ValueType: string; ValueName: "GenericDNSServers"; ValueData: "127.0.0.2"; Flags: uninsdeletekey
Root: HKLM; Subkey: "SYSTEM\CurrentControlSet\Services\Dnscache\Parameters\DnsPolicyConfig\{{E6B5D7A1-8843-4C49-B944-C5200E481230}"; ValueType: dword; ValueName: "ConfigOptions"; ValueData: 8; Flags: uninsdeletekey
Root: HKLM; Subkey: "SYSTEM\CurrentControlSet\Services\Dnscache\Parameters\DnsPolicyConfig\{{E6B5D7A1-8843-4C49-B944-C5200E481230}"; ValueType: dword; ValueName: "Version"; ValueData: 2; Flags: uninsdeletekey

; Suppress browser DNS-over-HTTPS (DoH) bypass
Root: HKLM; Subkey: "SOFTWARE\Policies\Google\Chrome"; ValueType: string; ValueName: "DnsOverHttpsMode"; ValueData: "off"; Flags: uninsdeletevalue
Root: HKLM; Subkey: "SOFTWARE\Policies\Microsoft\Edge"; ValueType: string; ValueName: "DnsOverHttpsMode"; ValueData: "off"; Flags: uninsdeletevalue

[Run]
; Install and start Windows Service automatically under LocalSystem
Filename: "{sys}\sc.exe"; Parameters: "create FamShieldService binPath= ""{app}\famshield-service.exe"" start= auto obj= LocalSystem DisplayName= ""FamShield Security Platform"""; Flags: runhidden
Filename: "{sys}\sc.exe"; Parameters: "config FamShieldService binPath= ""{app}\famshield-service.exe"" start= auto obj= LocalSystem DisplayName= ""FamShield Security Platform"""; Flags: runhidden
Filename: "{sys}\sc.exe"; Parameters: "description FamShieldService ""Unified parental control, NUMA DNS proxy and application filtering"""; Flags: runhidden
Filename: "{sys}\sc.exe"; Parameters: "start FamShieldService"; Flags: runhidden
; Configure active network adapters DNS to 127.0.0.2 for 100% reliable interception
Filename: "{sys}\WindowsPowerShell\v1.0\powershell.exe"; Parameters: "-NoProfile -ExecutionPolicy Bypass -Command ""Get-NetAdapter | Where-Object {{ $_.Status -eq 'Up' -and $_.InterfaceAlias -notmatch 'vEthernet|WSL|Loopback' }} | ForEach-Object {{ Set-DnsClientServerAddress -InterfaceIndex $_.InterfaceIndex -ServerAddresses '127.0.0.2' -ErrorAction SilentlyContinue }}; ipconfig /flushdns"""; Flags: runhidden
; Auto-launch tray agent and open local admin setup wizard in default web browser
Filename: "{app}\famshield-session.exe"; Flags: nowait postinstall skipifsilent
Filename: "http://127.0.0.1:38123"; Description: "Mở Bảng Điều Khiển Quản Trị Cục Bộ (Web Admin)"; Flags: shellexec nowait postinstall

[UninstallRun]
; 1. Terminate user session helper and service processes so binary file locks are released
Filename: "{sys}\taskkill.exe"; Parameters: "/F /IM famshield-session.exe /T"; Flags: runhidden; RunOnceId: "KillFamShieldSession"
; 2. Stop and remove Windows Service cleanly
Filename: "{sys}\sc.exe"; Parameters: "stop FamShieldService"; Flags: runhidden; RunOnceId: "StopFamShieldService"
Filename: "{sys}\taskkill.exe"; Parameters: "/F /IM famshield-service.exe /T"; Flags: runhidden; RunOnceId: "KillFamShieldService"
Filename: "{sys}\sc.exe"; Parameters: "delete FamShieldService"; Flags: runhidden; RunOnceId: "DeleteFamShieldService"
; 3. Reset all network adapters DNS back to DHCP and flush cache
Filename: "{sys}\WindowsPowerShell\v1.0\powershell.exe"; Parameters: "-NoProfile -ExecutionPolicy Bypass -Command ""Get-NetAdapter | ForEach-Object {{ Set-DnsClientServerAddress -InterfaceIndex $_.InterfaceIndex -ResetServerAddresses -ErrorAction SilentlyContinue }}; Get-NetAdapter | Where-Object {{ $_.Status -eq 'Up' }} | ForEach-Object {{ netsh interface ip set dns name=$_.Name source=dhcp }}; ipconfig /flushdns"""; Flags: runhidden; RunOnceId: "ResetDnsAddresses"
; 4. Remove browser DoH suppression policies
Filename: "{sys}\WindowsPowerShell\v1.0\powershell.exe"; Parameters: "-NoProfile -ExecutionPolicy Bypass -Command ""Remove-ItemProperty -Path 'HKLM:\SOFTWARE\Policies\Google\Chrome' -Name 'DnsOverHttpsMode' -ErrorAction SilentlyContinue; Remove-ItemProperty -Path 'HKLM:\SOFTWARE\Policies\Microsoft\Edge' -Name 'DnsOverHttpsMode' -ErrorAction SilentlyContinue"""; Flags: runhidden; RunOnceId: "RemoveDohPolicies"
; 5. Remove Code Signing certificate from stores
Filename: "{sys}\certutil.exe"; Parameters: "-delstore Root ""FamShield Root CA"""; Flags: runhidden; RunOnceId: "DelCertRoot"
Filename: "{sys}\certutil.exe"; Parameters: "-delstore TrustedPublisher ""FamShield Root CA"""; Flags: runhidden; RunOnceId: "DelCertPublisher"

[UninstallDelete]
Type: filesandordirs; Name: "{app}\web"
Type: files; Name: "{app}\famshield-service.exe"
Type: files; Name: "{app}\famshield-session.exe"
Type: files; Name: "{app}\*.ico"
Type: files; Name: "{app}\*.cer"
Type: filesandordirs; Name: "{commonappdata}\FamShield"

[Code]
var
  ConfigPage: TInputQueryWizardPage;

// Initialize custom wizard page for Server URL and Enrollment Code configuration
procedure InitializeWizard;
var
  DefaultServerUrl: String;
  DefaultCode: String;
begin
  // Support silent/command-line parameter overrides: /SERVER="https://..." /CODE="123456"
  DefaultServerUrl := ExpandConstant('{param:SERVER|https://famshield.ttcantho.io.vn}');
  DefaultCode := ExpandConstant('{param:CODE|}');

  // Create wizard page right after Destination Directory page
  ConfigPage := CreateInputQueryPage(wpSelectDir,
    'Cấu hình Máy chủ Cập nhật & Ghép nối',
    'Thiết lập địa chỉ máy chủ Cloud Server và mã ghép nối bảo vệ thiết bị',
    'FamShield hỗ trợ cả hai chế độ: Cloud Managed (kết nối máy chủ) và Standalone Free (chạy độc lập).'#13#10 +
    'Vui lòng nhập địa chỉ máy chủ cập nhật (Update / Cloud Server URL) và mã ghép nối bên dưới:');

  ConfigPage.Add('Địa chỉ máy chủ (Server URL / Update Check):', False);
  ConfigPage.Values[0] := DefaultServerUrl;

  ConfigPage.Add('Mã ghép nối thiết bị (Enrollment Code - 6 số, để trống nếu dùng Standalone Free):', False);
  ConfigPage.Values[1] := DefaultCode;
end;

// Validate user input before proceeding to next wizard page
function NextButtonClick(CurPageID: Integer): Boolean;
var
  ServerUrlInput: String;
  CodeInput: String;
  I: Integer;
begin
  Result := True;
  if CurPageID = ConfigPage.ID then
  begin
    ServerUrlInput := Trim(ConfigPage.Values[0]);
    if ServerUrlInput = '' then
    begin
      MsgBox('Vui lòng nhập địa chỉ máy chủ hợp lệ (ví dụ: https://famshield.ttcantho.io.vn hoặc https://cloud.famshield.vn)', mbError, MB_OK);
      Result := False;
      Exit;
    end;

    if (Pos('http://', Lowercase(ServerUrlInput)) <> 1) and (Pos('https://', Lowercase(ServerUrlInput)) <> 1) then
    begin
      MsgBox('Địa chỉ máy chủ phải bắt đầu bằng http:// hoặc https://', mbError, MB_OK);
      Result := False;
      Exit;
    end;

    CodeInput := Trim(ConfigPage.Values[1]);
    if CodeInput <> '' then
    begin
      if Length(CodeInput) <> 6 then
      begin
        MsgBox('Mã ghép nối phải đúng 6 chữ số (hoặc để trống nếu dùng chế độ Standalone Free)', mbError, MB_OK);
        Result := False;
        Exit;
      end;

      for I := 1 to Length(CodeInput) do
      begin
        if (CodeInput[I] < '0') or (CodeInput[I] > '9') then
        begin
          MsgBox('Mã ghép nối chỉ được chứa các ký tự số (0-9)', mbError, MB_OK);
          Result := False;
          Exit;
        end;
      end;
    end;
  end;
end;

// Write bootstrap configuration and server_url.txt after installation files are extracted
procedure CurStepChanged(CurStep: TSetupStep);
var
  DataDir: String;
  ServerUrlVal: String;
  EnrollCodeVal: String;
  BootstrapJson: String;
begin
  if CurStep = ssPostInstall then
  begin
    DataDir := ExpandConstant('{commonappdata}\FamShield');
    ForceDirectories(DataDir);

    ServerUrlVal := Trim(ConfigPage.Values[0]);
    if ServerUrlVal = '' then
      ServerUrlVal := 'https://famshield.ttcantho.io.vn';

    EnrollCodeVal := Trim(ConfigPage.Values[1]);

    // Save server_url.txt for immediate resolution by agent and local admin
    SaveStringToFile(DataDir + '\server_url.txt', ServerUrlVal, False);

    // Save bootstrap.json for auto-enrollment by agent on first start
    BootstrapJson := '{"server_url":"' + ServerUrlVal + '","enrollment_code":"' + EnrollCodeVal + '"}';
    SaveStringToFile(DataDir + '\bootstrap.json', BootstrapJson, False);
  end;
end;

// Clean up existing service and terminate running processes before files extraction
function PrepareToInstall(var NeedsRestart: Boolean): String;
var
  ResultCode: Integer;
begin
  Result := '';

  // 1. Stop existing service
  Exec(ExpandConstant('{sys}\sc.exe'), 'stop FamShieldService', '', SW_HIDE, ewWaitUntilTerminated, ResultCode);

  // 2. Forcefully kill all lingering processes to release file locks on binaries
  Exec(ExpandConstant('{sys}\taskkill.exe'), '/F /IM famshield-session.exe /T', '', SW_HIDE, ewWaitUntilTerminated, ResultCode);
  Exec(ExpandConstant('{sys}\taskkill.exe'), '/F /IM famshield-service.exe /T', '', SW_HIDE, ewWaitUntilTerminated, ResultCode);

  // 3. Remove existing service cleanly so fresh installation can re-register it
  Exec(ExpandConstant('{sys}\sc.exe'), 'delete FamShieldService', '', SW_HIDE, ewWaitUntilTerminated, ResultCode);

  // Wait a moment for Windows SCM and file handles to be released completely
  Sleep(1500);
end;

// Prompt for parent confirmation when uninstaller is launched manually
function InitializeUninstall(): Boolean;
var
  ResultCode: Integer;
begin
  Result := True;
  if not UninstallSilent then
  begin
    // Anti-tamper: Direct execution of unins000.exe requires Master PIN authentication
    if FileExists(ExpandConstant('{app}\famshield-session.exe')) then
    begin
      MsgBox('CẢNH BÁO BẢO MẬT PHỤ HUYNH:' + #13#10#13#10 +
             'Để gỡ cài đặt FamShield, bạn cần xác thực Master PIN.' + #13#10 +
             'FamShield đang khởi chạy cửa sổ xác thực an toàn...',
             mbInformation, MB_OK);
      Exec(ExpandConstant('{app}\famshield-session.exe'), '--uninstall', '', SW_SHOW, ewNoWait, ResultCode);
      Result := False;
      Exit;
    end
    else
    begin
      if MsgBox('CẢNH BÁO BẢO MẬT:' + #13#10#13#10 +
                'Bạn có chắc chắn muốn gỡ bỏ FamShield?' + #13#10 +
                'Toàn bộ cấu hình bảo vệ sẽ bị dừng và cài đặt DNS mạng sẽ được khôi phục về ban đầu.',
                mbConfirmation, MB_YESNO or MB_DEFBUTTON2) <> IDYES then
      begin
        Result := False;
        Exit;
      end;
    end;
  end;
end;
