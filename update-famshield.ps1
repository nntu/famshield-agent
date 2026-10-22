#Requires -Version 5.1
$ErrorActionPreference = 'Stop'

# Auto-elevate to Administrator via UAC prompt if running as standard user
$isAdmin = ([Security.Principal.WindowsPrincipal][Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
if (-not $isAdmin) {
    Write-Host "[*] Dang yeu cau quyen Administrator (UAC)..." -ForegroundColor Yellow
    Start-Process powershell.exe -ArgumentList "-NoProfile -ExecutionPolicy Bypass -File `"$PSCommandPath`"" -Verb RunAs
    exit
}

Write-Host "=======================================================" -ForegroundColor Cyan
Write-Host " FamShield Agent - DNS & Service Auto Updater" -ForegroundColor Cyan
Write-Host "=======================================================" -ForegroundColor Cyan

$serviceName = "FamShieldService"
$installDir = "C:\Program Files\FamShield"
$repoRoot = $PSScriptRoot

Write-Host "[1/6] Dang dung FamShieldService va session agent..." -ForegroundColor Yellow
Stop-Service -Name $serviceName -Force -ErrorAction SilentlyContinue
if (Get-Process -Name "famshield-service" -ErrorAction SilentlyContinue) {
    cmd.exe /c 'taskkill /F /IM famshield-service.exe /T >nul 2>nul'
}
if (Get-Process -Name "famshield-session" -ErrorAction SilentlyContinue) {
    cmd.exe /c 'taskkill /F /IM famshield-session.exe /T >nul 2>nul'
}
Get-Process -Name "famshield-service", "famshield-session" -ErrorAction SilentlyContinue | Stop-Process -Force -ErrorAction SilentlyContinue
Start-Sleep -Milliseconds 1200

Write-Host "[2/6] Don dep khoa NRPT cu gay loi 9572 DNS_ERROR_INVALID_POLICY_TABLE..." -ForegroundColor Yellow
$corruptedKeys = @(
    "HKLM:\SOFTWARE\Policies\Microsoft\Windows NT\DNSClient\DnsPolicyConfig\{FamShield-DNS-Filter}",
    "HKLM:\SYSTEM\CurrentControlSet\Services\Dnscache\Parameters\DnsPolicyConfig\{FamShield-DNS-Filter}"
)
foreach ($key in $corruptedKeys) {
    if (Test-Path $key) {
        Remove-Item -Path $key -Recurse -Force -ErrorAction SilentlyContinue
        Write-Host "      Da xoa khoa loi: $key" -ForegroundColor Green
    }
}

Write-Host "[3/6] Bien dich Rust binaries, Web Admin va cap nhat he thong..." -ForegroundColor Yellow
$serviceSrc = Join-Path $repoRoot "target\release\famshield-service.exe"
$sessionSrc = Join-Path $repoRoot "target\release\famshield-session.exe"

# 1. Compile Rust binaries if cargo is available
if (Get-Command cargo -ErrorAction SilentlyContinue) {
    Write-Host "      Dang bien dich Rust binaries (cargo build --release)..." -ForegroundColor Gray
    & cargo build --release --bin famshield-service --bin famshield-session
    if ($LASTEXITCODE -ne 0) {
        Write-Error "Loi bien dich Rust binaries!"
        exit 1
    }
    Write-Host "      Bien dich Rust binaries thanh cong!" -ForegroundColor Green
}

if (-not (Test-Path $serviceSrc)) {
    Write-Error "Khong tim thay $serviceSrc. Vui long chay build truoc!"
    exit 1
}

Copy-Item -Path $serviceSrc -Destination $installDir -Force
Copy-Item -Path $sessionSrc -Destination $installDir -Force
Write-Host "      Da cap nhat famshield-service.exe va famshield-session.exe" -ForegroundColor Green

# Build Web Admin if pnpm/npm available
$webDir = Join-Path $repoRoot "web"
if (Test-Path (Join-Path $webDir "package.json")) {
    Push-Location $webDir
    try {
        if (Get-Command pnpm -ErrorAction SilentlyContinue) {
            Write-Host "      Dang bien dich giao dien Web Admin (pnpm run build)..." -ForegroundColor Gray
            & pnpm run build | Out-Null
        } elseif (Get-Command npm -ErrorAction SilentlyContinue) {
            Write-Host "      Dang bien dich giao dien Web Admin (npm run build)..." -ForegroundColor Gray
            & npm run build | Out-Null
        }
    } catch {
        Write-Warning "Khong the bien dich Web Admin: $_"
    } finally {
        Pop-Location
    }
}

$webDistSrc = Join-Path $repoRoot "web\dist"
if (Test-Path $webDistSrc) {
    # 1. Update ProgramData\FamShield\web\dist (Uu tien so 1 cua LocalAdminServer)
    $pdWebDist = Join-Path $env:ProgramData "FamShield\web\dist"
    if (-not (Test-Path $pdWebDist)) {
        New-Item -ItemType Directory -Path $pdWebDist -Force | Out-Null
    }
    # Clear old hashed assets
    Get-ChildItem -Path $pdWebDist -Recurse -ErrorAction SilentlyContinue | Remove-Item -Recurse -Force -ErrorAction SilentlyContinue
    Copy-Item -Path (Join-Path $webDistSrc "*") -Destination $pdWebDist -Recurse -Force
    Write-Host "      Da cap nhat $pdWebDist" -ForegroundColor Green

    # 2. Update Program Files\FamShield\web\dist (Fallback du phong)
    $pfWebDist = Join-Path $installDir "web\dist"
    if (-not (Test-Path $pfWebDist)) {
        New-Item -ItemType Directory -Path $pfWebDist -Force | Out-Null
    }
    Get-ChildItem -Path $pfWebDist -Recurse -ErrorAction SilentlyContinue | Remove-Item -Recurse -Force -ErrorAction SilentlyContinue
    Copy-Item -Path (Join-Path $webDistSrc "*") -Destination $pfWebDist -Recurse -Force
    Write-Host "      Da cap nhat $pfWebDist" -ForegroundColor Green
}

Write-Host "[4/6] Khoi dong FamShieldService..." -ForegroundColor Yellow
$svc = Get-Service -Name $serviceName -ErrorAction SilentlyContinue
if ($svc) {
    Start-Service -Name $serviceName -ErrorAction SilentlyContinue
    $svc = Get-Service -Name $serviceName
    Write-Host "      Trang thai service: $($svc.Status)" -ForegroundColor Green
} else {
    Write-Host "      Service FamShield chua duoc cai dat. Vui long chay .\build.ps1 -InstallService" -ForegroundColor Yellow
}

Write-Host "[5/6] Tro DNS tren cac card mang hoat dong ve 127.0.0.2:53..." -ForegroundColor Yellow
$adapters = Get-NetAdapter | Where-Object { $_.Status -eq 'Up' -and $_.InterfaceAlias -notmatch 'vEthernet|WSL|Loopback' }
foreach ($adapter in $adapters) {
    Set-DnsClientServerAddress -InterfaceIndex $adapter.InterfaceIndex -ServerAddresses '127.0.0.2' -ErrorAction SilentlyContinue
    Write-Host "      Card mang [$($adapter.Name)] ($($adapter.InterfaceAlias)) -> DNS: 127.0.0.2" -ForegroundColor Green
}
Clear-DnsClientCache
ipconfig /flushdns | Out-Null

Write-Host "[6/6] Khoi dong famshield-session..." -ForegroundColor Yellow
$sessionExe = Join-Path $installDir "famshield-session.exe"
Start-Process -FilePath $sessionExe

Write-Host ""
Write-Host "=======================================================" -ForegroundColor Green
Write-Host " [OK] DA CAP NHAT THANH CONG!" -ForegroundColor Green
Write-Host " - Windows DNS da duoc tro ve 127.0.0.2:53" -ForegroundColor Green
Write-Host " - Khac phuc triet de loi 9572 DNS_ERROR_INVALID_POLICY_TABLE" -ForegroundColor Green
Write-Host " - Truy cap Web Admin: http://127.0.0.1:38123 (Tab Nhat Ky DNS)" -ForegroundColor Green
Write-Host "=======================================================" -ForegroundColor Green
Write-Host ""
Write-Host "Nhan Enter de hoan tat..."
Read-Host
