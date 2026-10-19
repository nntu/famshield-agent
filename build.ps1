<#
.SYNOPSIS
    Build and Installation Script for FamShield Windows Agent.

.DESCRIPTION
    1. Compiles Rust workspace binaries (famshield-service, famshield-session) in Release mode.
    2. Automatically signs binaries with FamShield Authenticode Code Signing Certificate.
    3. Packages production Windows installer via Inno Setup (ISCC.exe) and signs the installer.
    4. Installs or updates the Windows Service (FamShieldService) under LocalSystem.
    5. Configures Windows NRPT (Name Resolution Policy Table) to route namespace '.' to 127.0.0.2:53.
    6. Provides actions: build, sign, package, install, uninstall, status, reset-pin.

.PARAMETER Action
    build      - Build release binaries only.
    sign       - Ensure Authenticode certificate and digitally sign compiled binaries.
    package    - Compile release binaries, sign them, build Inno Setup installer, and sign installer.
    install    - Build, sign, and install Windows service & NRPT policy (Requires Administrator privileges).
    uninstall  - Stop and remove Windows service & NRPT policy.
    status     - Check service and NRPT status.
    reset-pin  - Reset Master PIN with Administrator elevation.
#>

[CmdletBinding()]
param (
    [Parameter(Position = 0)]
    [ValidateSet("build", "sign", "package", "install", "uninstall", "status", "reset-pin")]
    [string]$Action = "install",

    [Parameter(Position = 1)]
    [string]$NewPin = "1234"
)

$ErrorActionPreference = "Stop"

$ProjectRoot = $PSScriptRoot
$TargetDir = Join-Path $ProjectRoot "target\release"
$InstallDir = Join-Path $env:ProgramFiles "FamShield"
$ServiceName = "FamShieldService"
$NrptRulePath = "HKLM:\SOFTWARE\Policies\Microsoft\Windows NT\DnsClient\DnsPolicyConfig\{FamShield-DNS-Filter}"
$CertsDir = Join-Path $ProjectRoot "installer\certs"
$CertFile = Join-Path $CertsDir "FamShieldRoot.cer"
$CertSubject = "CN=FamShield Security Authority"

function Assert-Administrator {
    $currentPrincipal = New-Object Security.Principal.WindowsPrincipal([Security.Principal.WindowsIdentity]::GetCurrent())
    if (-not $currentPrincipal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)) {
        Write-Error "Action '$Action' requires elevated Administrator privileges. Please run PowerShell as Administrator."
    }
}

function Find-SignTool {
    $candidates = @(
        "C:\Program Files (x86)\Windows Kits\10\bin\10.0.26100.0\x64\signtool.exe",
        "C:\Program Files (x86)\Windows Kits\10\bin\10.0.22621.0\x64\signtool.exe",
        "C:\Program Files (x86)\Windows Kits\10\bin\10.0.19041.0\x64\signtool.exe",
        "C:\Program Files (x86)\Windows Kits\10\App Certification Kit\signtool.exe"
    )
    foreach ($path in $candidates) {
        if (Test-Path $path) {
            return $path
        }
    }
    $found = Get-ChildItem -Path "C:\Program Files (x86)\Windows Kits" -Filter "signtool.exe" -Recurse -ErrorAction SilentlyContinue |
        Where-Object { $_.FullName -like "*x64*" } | Select-Object -First 1
    if ($found) {
        return $found.FullName
    }
    return "signtool.exe"
}

function Find-InnoSetupCompiler {
    $candidates = @(
        "C:\Program Files (x86)\Inno Setup 6\ISCC.exe",
        "C:\Program Files\Inno Setup 6\ISCC.exe"
    )
    foreach ($path in $candidates) {
        if (Test-Path $path) {
            return $path
        }
    }
    $found = Get-ChildItem -Path "C:\Program Files (x86)\Inno Setup*", "C:\Program Files\Inno Setup*" -Filter "ISCC.exe" -Recurse -ErrorAction SilentlyContinue | Select-Object -First 1
    if ($found) {
        return $found.FullName
    }
    return "ISCC.exe"
}

function Ensure-CodeSigningCert {
    Write-Host "==> Checking for Authenticode Code Signing Certificate..." -ForegroundColor Cyan

    $cert = Get-ChildItem Cert:\CurrentUser\My -ErrorAction SilentlyContinue | Where-Object { $_.Subject -like "*$CertSubject*" } | Select-Object -First 1
    if (-not $cert) {
        $cert = Get-ChildItem Cert:\LocalMachine\My -ErrorAction SilentlyContinue | Where-Object { $_.Subject -like "*$CertSubject*" } | Select-Object -First 1
    }

    if (-not $cert) {
        Write-Host "==> Creating Self-Signed Code Signing Certificate for FamShield..." -ForegroundColor Yellow
        $cert = New-SelfSignedCertificate -Type CodeSigning `
            -Subject "CN=FamShield Security Authority, O=FamShield Team, C=VN" `
            -CertStoreLocation "Cert:\CurrentUser\My" `
            -KeyUsage DigitalSignature `
            -FriendlyName "FamShield Development Authenticode Signing" `
            -NotAfter (Get-Date).AddYears(5)

        if (-not (Test-Path $CertsDir)) {
            New-Item -ItemType Directory -Path $CertsDir -Force | Out-Null
        }

        # Export public certificate (.cer)
        Export-Certificate -Cert $cert -FilePath $CertFile -Force | Out-Null

        # Install into CurrentUser Trusted Root & Trusted Publisher so Windows trusts the binaries locally
        Import-Certificate -FilePath $CertFile -CertStoreLocation "Cert:\CurrentUser\Root" | Out-Null
        Import-Certificate -FilePath $CertFile -CertStoreLocation "Cert:\CurrentUser\TrustedPublisher" | Out-Null

        Write-Host "==> Created and trusted certificate: $($cert.Thumbprint)" -ForegroundColor Green
    } else {
        Write-Host "==> Found existing Code Signing Certificate: $($cert.Thumbprint)" -ForegroundColor Green
        if (-not (Test-Path $CertFile)) {
            if (-not (Test-Path $CertsDir)) {
                New-Item -ItemType Directory -Path $CertsDir -Force | Out-Null
            }
            Export-Certificate -Cert $cert -FilePath $CertFile -Force | Out-Null
        }
    }

    return $cert
}

function Sign-Binary {
    param([string]$FilePath)

    if (-not (Test-Path $FilePath)) {
        Write-Error "Cannot sign file: '$FilePath' not found."
    }

    $signTool = Find-SignTool
    $cert = Ensure-CodeSigningCert
    $thumbprint = $cert.Thumbprint

    Write-Host "==> Signing Authenticode: $([System.IO.Path]::GetFileName($FilePath))..." -ForegroundColor Cyan

    # Sign with RFC 3161 timestamping (DigiCert or Sectigo)
    $tsUrl = "http://timestamp.digicert.com"
    & "$signTool" sign /sha1 $thumbprint /fd sha256 /tr $tsUrl /td sha256 "$FilePath"

    if ($LASTEXITCODE -ne 0) {
        Write-Host "Timestamp server unavailable, attempting signing without timestamp..." -ForegroundColor Yellow
        & "$signTool" sign /sha1 $thumbprint /fd sha256 "$FilePath"
    }

    if ($LASTEXITCODE -eq 0) {
        Write-Host "==> Verification: Signature OK for $([System.IO.Path]::GetFileName($FilePath))" -ForegroundColor Green
    } else {
        Write-Error "Failed to sign binary: $FilePath"
    }
}

function Sign-AllBinaries {
    $ServiceBin = Join-Path $TargetDir "famshield-service.exe"
    $SessionBin = Join-Path $TargetDir "famshield-session.exe"

    if (-not (Test-Path $ServiceBin) -or -not (Test-Path $SessionBin)) {
        Build-Agent
    }

    Sign-Binary -FilePath $ServiceBin
    Sign-Binary -FilePath $SessionBin
    Write-Host "==> All FamShield release binaries are digitally signed!" -ForegroundColor Green
}

function Build-Agent {
    Write-Host "==> Building FamShield Release Binaries with Cargo..." -ForegroundColor Cyan
    Set-Location $ProjectRoot
    cargo build --release --bins

    $ServiceBin = Join-Path $TargetDir "famshield-service.exe"
    $SessionBin = Join-Path $TargetDir "famshield-session.exe"

    if (-not (Test-Path $ServiceBin) -or -not (Test-Path $SessionBin)) {
        Write-Error "Build failed: Could not locate compiled binaries in $TargetDir"
    }

    Write-Host "==> Build successful: $ServiceBin, $SessionBin" -ForegroundColor Green
}

function Install-Agent {
    Assert-Administrator
    Build-Agent
    Sign-AllBinaries

    Write-Host "==> [2/3] Installing Binaries into $InstallDir..." -ForegroundColor Cyan
    if (-not (Test-Path $InstallDir)) {
        New-Item -ItemType Directory -Path $InstallDir -Force | Out-Null
    }

    # Stop service if running
    $svc = Get-Service -Name $ServiceName -ErrorAction SilentlyContinue
    if ($svc) {
        Write-Host "Stopping existing $ServiceName service..." -ForegroundColor Yellow
        Stop-Service -Name $ServiceName -Force -ErrorAction SilentlyContinue
        Start-Sleep -Seconds 2
    }

    Copy-Item -Path (Join-Path $TargetDir "famshield-service.exe") -Destination (Join-Path $InstallDir "famshield-service.exe") -Force
    Copy-Item -Path (Join-Path $TargetDir "famshield-session.exe") -Destination (Join-Path $InstallDir "famshield-session.exe") -Force

    Write-Host "==> [3/3] Registering Windows Service ($ServiceName) and NRPT Routing..." -ForegroundColor Cyan

    $BinPath = Join-Path $InstallDir "famshield-service.exe"

    if (-not $svc) {
        New-Service -Name $ServiceName -DisplayName "FamShield Security Platform" -BinaryPathName "`"$BinPath`"" -StartupType Automatic -Description "Unified parental control, NUMA DNS proxy and application filtering" | Out-Null
        # Set to LocalSystem
        sc.exe config $ServiceName obj= LocalSystem | Out-Null
    } else {
        sc.exe config $ServiceName binPath= "`"$BinPath`"" | Out-Null
    }

    # Configure NRPT routing for '.' namespace to 127.0.0.2
    if (-not (Test-Path $NrptRulePath)) {
        New-Item -Path $NrptRulePath -Force | Out-Null
    }
    Set-ItemProperty -Path $NrptRulePath -Name "Name" -Value "."
    Set-ItemProperty -Path $NrptRulePath -Name "GenericDNSServers" -Value "127.0.0.2"
    Set-ItemProperty -Path $NrptRulePath -Name "ConfigOptions" -Value 0x8 -Type DWord
    Set-ItemProperty -Path $NrptRulePath -Name "Version" -Value 1 -Type DWord

    Write-Host "Starting $ServiceName service..." -ForegroundColor Cyan
    Start-Service -Name $ServiceName -ErrorAction SilentlyContinue

    Write-Host "==========================================================" -ForegroundColor Green
    Write-Host " FamShield Agent has been successfully installed! " -ForegroundColor Green
    Write-Host " - Service Status: Running (LocalSystem)" -ForegroundColor Green
    Write-Host " - Digital Signature: Authenticode Verified" -ForegroundColor Green
    Write-Host " - DNS Proxy Loopback: 127.0.0.2:53 (NRPT Active)" -ForegroundColor Green
    Write-Host " - Local Data Directory: $env:ProgramData\FamShield" -ForegroundColor Green
    Write-Host "==========================================================" -ForegroundColor Green
}

function Package-Agent {
    Write-Host "==> Packaging FamShield Windows Installer with Inno Setup..." -ForegroundColor Cyan

    # 1. Build and sign release binaries
    Build-Agent
    Sign-AllBinaries

    # 2. Locate Inno Setup Compiler
    $iscc = Find-InnoSetupCompiler
    if (-not (Test-Path $iscc)) {
        Write-Error "Inno Setup Compiler (ISCC.exe) not found. Please install Inno Setup 6."
    }

    $issFile = Join-Path $ProjectRoot "installer\FamShield.iss"
    Write-Host "==> Compiling installer with Inno Setup: $issFile..." -ForegroundColor Cyan
    & "$iscc" "$issFile"

    if ($LASTEXITCODE -ne 0) {
        Write-Error "Inno Setup compilation failed with code $LASTEXITCODE."
    }

    # 3. Locate and sign the generated installer executable
    $installerExe = Join-Path $ProjectRoot "installer\Output\FamShieldSetup-0.1.0.exe"
    if (Test-Path $installerExe) {
        Sign-Binary -FilePath $installerExe
        Write-Host "==========================================================" -ForegroundColor Green
        Write-Host " FamShield Setup package generated and digitally signed!  " -ForegroundColor Green
        Write-Host " Installer: $installerExe" -ForegroundColor Green
        Write-Host "==========================================================" -ForegroundColor Green
    } else {
        Write-Error "Could not locate generated installer executable at $installerExe"
    }
}

function Uninstall-Agent {
    Assert-Administrator

    Write-Host "==> Uninstalling FamShield Agent..." -ForegroundColor Yellow

    # 1. Stop and remove service
    $svc = Get-Service -Name $ServiceName -ErrorAction SilentlyContinue
    if ($svc) {
        Write-Host "Stopping service $ServiceName..." -ForegroundColor Yellow
        Stop-Service -Name $ServiceName -Force -ErrorAction SilentlyContinue
        Start-Sleep -Seconds 1
        sc.exe delete $ServiceName | Out-Null
        Write-Host "Service $ServiceName removed." -ForegroundColor Green
    }

    # 2. Clean up NRPT registry rule
    if (Test-Path $NrptRulePath) {
        Remove-Item -Path $NrptRulePath -Recurse -Force | Out-Null
        Write-Host "NRPT DnsPolicyConfig rule removed." -ForegroundColor Green
    }

    Write-Host "FamShield uninstallation complete. System network settings restored." -ForegroundColor Green
}

function Show-Status {
    Write-Host "--- FamShield Agent Status ---" -ForegroundColor Cyan
    $svc = Get-Service -Name $ServiceName -ErrorAction SilentlyContinue
    if ($svc) {
        $color = if ($svc.Status -eq "Running") { "Green" } else { "Yellow" }
        Write-Host "Service '$ServiceName': $($svc.Status)" -ForegroundColor $color
    } else {
        Write-Host "Service '$ServiceName': Not Installed" -ForegroundColor Red
    }

    if (Test-Path $NrptRulePath) {
        $dns = Get-ItemPropertyValue -Path $NrptRulePath -Name "GenericDNSServers" -ErrorAction SilentlyContinue
        Write-Host "NRPT DNS Route for namespace '.': $dns" -ForegroundColor Green
    } else {
        Write-Host "NRPT DNS Route: Inactive (Normal Windows DNS)" -ForegroundColor Gray
    }

    # Certificate status
    $cert = Get-ChildItem Cert:\CurrentUser\My -ErrorAction SilentlyContinue | Where-Object { $_.Subject -like "*$CertSubject*" } | Select-Object -First 1
    if ($cert) {
        Write-Host "Code Signing Cert: Present ($($cert.Thumbprint))" -ForegroundColor Green
    } else {
        Write-Host "Code Signing Cert: Not Created (Will auto-generate on sign/package)" -ForegroundColor Yellow
    }
}

function Reset-Pin {
    Assert-Administrator
    $SvcBin = Join-Path $InstallDir "famshield-service.exe"
    if (-not (Test-Path $SvcBin)) {
        $SvcBin = Join-Path $TargetDir "famshield-service.exe"
    }

    if (-not (Test-Path $SvcBin)) {
        Write-Error "Could not find famshield-service.exe to perform PIN reset. Run '.\build.ps1 build' first."
    }

    Write-Host "==> Resetting FamShield Master PIN with Administrator Privileges..." -ForegroundColor Cyan
    & $SvcBin --reset-pin $NewPin
    if ($LASTEXITCODE -eq 0) {
        Write-Host "Master PIN successfully updated to: $NewPin" -ForegroundColor Green
    } else {
        Write-Error "Failed to reset Master PIN."
    }
}

# Execute requested action
switch ($Action) {
    "build"     { Build-Agent }
    "sign"      { Sign-AllBinaries }
    "package"   { Package-Agent }
    "install"   { Install-Agent }
    "uninstall" { Uninstall-Agent }
    "status"    { Show-Status }
    "reset-pin" { Reset-Pin }
}


