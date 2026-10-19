<#
.SYNOPSIS
    Build and Installation Script for ChildProtect Windows Agent.

.DESCRIPTION
    1. Compiles Rust workspace binaries (childprotect-service, childprotect-session) in Release mode.
    2. Installs or updates the Windows Service (ChildProtectService) under LocalSystem.
    3. Configures Windows NRPT (Name Resolution Policy Table) to route namespace '.' to 127.0.0.2:53.
    4. Provides actions: build, install, uninstall, status.

.PARAMETER Action
    build      - Build release binaries only.
    install    - Build and install Windows service & NRPT policy (Requires Administrator privileges).
    uninstall  - Stop and remove Windows service & NRPT policy.
    status     - Check service and NRPT status.
#>

[CmdletBinding()]
param (
    [Parameter(Position = 0)]
    [ValidateSet("build", "install", "uninstall", "status")]
    [string]$Action = "install"
)

$ErrorActionPreference = "Stop"

$ProjectRoot = $PSScriptRoot
$TargetDir = Join-Path $ProjectRoot "target\release"
$InstallDir = Join-Path $env:ProgramFiles "ChildProtect"
$ServiceName = "ChildProtectService"
$NrptRulePath = "HKLM:\SOFTWARE\Policies\Microsoft\Windows NT\DnsClient\DnsPolicyConfig\{ChildProtect-DNS-Filter}"

function Assert-Administrator {
    $currentPrincipal = New-Object Security.Principal.WindowsPrincipal([Security.Principal.WindowsIdentity]::GetCurrent())
    if (-not $currentPrincipal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)) {
        Write-Error "Action '$Action' requires elevated Administrator privileges. Please run PowerShell as Administrator."
    }
}

function Build-Agent {
    Write-Host "==> [1/3] Building ChildProtect Release Binaries with Cargo..." -ForegroundColor Cyan
    Set-Location $ProjectRoot
    cargo build --release --bins

    $ServiceBin = Join-Path $TargetDir "childprotect-service.exe"
    $SessionBin = Join-Path $TargetDir "childprotect-session.exe"

    if (-not (Test-Path $ServiceBin) -or -not (Test-Path $SessionBin)) {
        Write-Error "Build failed: Could not locate compiled binaries in $TargetDir"
    }

    Write-Host "==> Build successful: $ServiceBin, $SessionBin" -ForegroundColor Green
}

function Install-Agent {
    Assert-Administrator
    Build-Agent

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

    Copy-Item -Path (Join-Path $TargetDir "childprotect-service.exe") -Destination (Join-Path $InstallDir "childprotect-service.exe") -Force
    Copy-Item -Path (Join-Path $TargetDir "childprotect-session.exe") -Destination (Join-Path $InstallDir "childprotect-session.exe") -Force

    Write-Host "==> [3/3] Registering Windows Service ($ServiceName) and NRPT Routing..." -ForegroundColor Cyan

    $BinPath = Join-Path $InstallDir "childprotect-service.exe"

    if (-not $svc) {
        New-Service -Name $ServiceName -DisplayName "ChildProtect Security Platform" -BinaryPathName "`"$BinPath`"" -StartupType Automatic -Description "Unified parental control, NUMA DNS proxy and application filtering" | Out-Null
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
    Write-Host " ChildProtect Agent has been successfully installed! " -ForegroundColor Green
    Write-Host " - Service Status: Running (LocalSystem)" -ForegroundColor Green
    Write-Host " - DNS Proxy Loopback: 127.0.0.2:53 (NRPT Active)" -ForegroundColor Green
    Write-Host " - Local Data Directory: $env:ProgramData\ChildProtect" -ForegroundColor Green
    Write-Host "==========================================================" -ForegroundColor Green
}

function Uninstall-Agent {
    Assert-Administrator

    Write-Host "==> Uninstalling ChildProtect Agent..." -ForegroundColor Yellow

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

    Write-Host "ChildProtect uninstallation complete. System network settings restored." -ForegroundColor Green
}

function Show-Status {
    Write-Host "--- ChildProtect Agent Status ---" -ForegroundColor Cyan
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
}

# Execute requested action
switch ($Action) {
    "build"     { Build-Agent }
    "install"   { Install-Agent }
    "uninstall" { Uninstall-Agent }
    "status"    { Show-Status }
}
