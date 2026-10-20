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
Get-Process -Name "famshield-service", "famshield-session" -ErrorAction SilentlyContinue | Stop-Process -Force -ErrorAction SilentlyContinue
Start-Sleep -Milliseconds 800

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

Write-Host "[3/6] Sao chep binary moi nhat vao $installDir..." -ForegroundColor Yellow
$serviceSrc = Join-Path $repoRoot "target\release\famshield-service.exe"
$sessionSrc = Join-Path $repoRoot "target\release\famshield-session.exe"

if (-not (Test-Path $serviceSrc)) {
    Write-Error "Khong tim thay $serviceSrc. Vui long chay build truoc!"
    exit 1
}

Copy-Item -Path $serviceSrc -Destination $installDir -Force
Copy-Item -Path $sessionSrc -Destination $installDir -Force
Write-Host "      Da cap nhat famshield-service.exe va famshield-session.exe" -ForegroundColor Green

Write-Host "[4/6] Khoi dong FamShieldService..." -ForegroundColor Yellow
Start-Service -Name $serviceName
$svc = Get-Service -Name $serviceName
Write-Host "      Trang thai service: $($svc.Status)" -ForegroundColor Green

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
