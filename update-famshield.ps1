#Requires -Version 5.1
<#
.SYNOPSIS
    FamShield Agent - DNS & Service Auto Updater

.DESCRIPTION
    1. Tu dong yeu cau quyen Administrator (UAC).
    2. Thiet lap moi truong build MSVC & phong tranh xung dot link.exe cua MSYS2/Git.
    3. Dung cac tien trinh FamShieldService va famshield-session dang chay.
    4. Don dep cac khoa NRPT bi loi va cau hinh lai NRPT hop le toi 127.0.0.2:53.
    5. Bien dich Rust binaries (famshield-service, famshield-session) & Web Admin.
    6. Cap nhat binaries va file asset Web Admin vao Program Files & ProgramData.
    7. Khoi dong lai FamShieldService, tro DNS adapter va khoi chay famshield-session.

.PARAMETER SkipBuild
    Bo qua buoc bien dich Rust va Web Admin (dung san file binaries trong target\release).

.PARAMETER NoWait
    Khong tam dung cho nhan Enter o cuoi script (phu hop khi chay tu dong).
#>

[CmdletBinding()]
param (
    [switch]$SkipBuild,
    [switch]$NoWait
)

$ErrorActionPreference = 'Stop'

# 0. Auto-elevate to Administrator via UAC prompt if running as standard user
$isAdmin = ([Security.Principal.WindowsPrincipal][Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
if (-not $isAdmin) {
    Write-Host "[*] Dang yeu cau quyen Administrator (UAC)..." -ForegroundColor Yellow
    $argList = "-NoProfile -ExecutionPolicy Bypass -File `"$PSCommandPath`""
    if ($SkipBuild) { $argList += " -SkipBuild" }
    if ($NoWait) { $argList += " -NoWait" }
    Start-Process powershell.exe -ArgumentList $argList -Verb RunAs
    exit
}

Set-Location $PSScriptRoot

Write-Host "=======================================================" -ForegroundColor Cyan
Write-Host " FamShield Agent - DNS & Service Auto Updater" -ForegroundColor Cyan
Write-Host "=======================================================" -ForegroundColor Cyan

$serviceName = "FamShieldService"
$installDir = Join-Path $env:ProgramFiles "FamShield"
$repoRoot = $PSScriptRoot

# Function to ensure MSVC environment and remove path conflicts (e.g. msys64/usr/bin)
function Ensure-BuildEnvironment {
    # 1. Bo sung cac thu muc chua tool cua User vao PATH neu chua co (do UAC -NoProfile co the thieu)
    $userToolDirs = @(
        (Join-Path $env:USERPROFILE ".cargo\bin"),
        (Join-Path $env:LOCALAPPDATA "pnpm"),
        (Join-Path $env:APPDATA "npm")
    )
    foreach ($dir in $userToolDirs) {
        if ((Test-Path $dir) -and ($env:PATH -notlike "*$dir*")) {
            $env:PATH = "$dir;$env:PATH"
        }
    }

    # 2. Loc bo cac duong dan chua GNU link.exe (MSYS2/Git usr/bin) gay xung dot linker
    $sanitized = ($env:PATH -split ';' | Where-Object { 
        $_ -and ($_ -notmatch 'msys64\\usr\\bin') -and ($_ -notmatch 'Git\\usr\\bin') 
    }) -join ';'
    $env:PATH = $sanitized

    # 3. Kiem tra MSVC Environment da san sang chua
    $hasMsvcLink = Get-Command link.exe -ErrorAction SilentlyContinue | Where-Object { $_.Source -like "*MSVC*" }
    if ($env:LIB -and $hasMsvcLink) {
        return
    }

    Write-Host "      Dang thiet lap moi truong MSVC (vcvars64.bat)..." -ForegroundColor Gray
    $candidates = @(
        "C:\Program Files (x86)\Microsoft Visual Studio\18\BuildTools\VC\Auxiliary\Build\vcvars64.bat",
        "C:\Program Files\Microsoft Visual Studio\18\BuildTools\VC\Auxiliary\Build\vcvars64.bat",
        "C:\Program Files\Microsoft Visual Studio\18\Professional\VC\Auxiliary\Build\vcvars64.bat",
        "C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools\VC\Auxiliary\Build\vcvars64.bat",
        "C:\Program Files\Microsoft Visual Studio\2022\Professional\VC\Auxiliary\Build\vcvars64.bat",
        "C:\Program Files\Microsoft Visual Studio\2022\Community\VC\Auxiliary\Build\vcvars64.bat"
    )

    $vcvars = $null
    foreach ($cand in $candidates) {
        if (Test-Path $cand) {
            $vcvars = $cand
            break
        }
    }

    if (-not $vcvars) {
        $vswhere = "${env:ProgramFiles(x86)}\Microsoft Visual Studio\Installer\vswhere.exe"
        if (-not (Test-Path $vswhere)) {
            $vswhere = "${env:ProgramFiles}\Microsoft Visual Studio\Installer\vswhere.exe"
        }
        if (Test-Path $vswhere) {
            $installPath = (& $vswhere -latest -products * -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath 2>$null)
            if ($installPath) {
                $checkBat = Join-Path $installPath.Trim() "VC\Auxiliary\Build\vcvars64.bat"
                if (Test-Path $checkBat) {
                    $vcvars = $checkBat
                }
            }
        }
    }

    if ($vcvars -and (Test-Path $vcvars)) {
        $tempFile = [System.IO.Path]::GetTempFileName() + ".cmd"
        Set-Content -Path $tempFile -Value "@echo off`ncall `"$vcvars`" >nul 2>nul`nset"
        $envLines = & cmd.exe /c $tempFile
        Remove-Item -Path $tempFile -Force -ErrorAction SilentlyContinue

        foreach ($line in $envLines) {
            if ($line -match '^([^=]+)=(.*)$') {
                [System.Environment]::SetEnvironmentVariable($matches[1], $matches[2], [System.EnvironmentVariableTarget]::Process)
            }
        }
    }

    # Luon uu tien toolset MSVC day du neu co
    if (-not $env:VCToolsVersion) {
        $msvc2026Dir = "C:\Program Files (x86)\Microsoft Visual Studio\18\BuildTools\VC\Tools\MSVC\14.51.36231"
        $msvc2022Dir = "C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools\VC\Tools\MSVC\14.44.35207"
        if (Test-Path $msvc2026Dir) {
            $env:VCToolsVersion = "14.51.36231"
        } elseif (Test-Path $msvc2022Dir) {
            $env:VCToolsVersion = "14.44.35207"
        }
    }

    # Prepend MSVC host bin vao dau PATH neu co
    if ($env:VCToolsInstallDir) {
        $msvcHostBin = Join-Path $env:VCToolsInstallDir "bin\HostX64\x64"
        if (Test-Path $msvcHostBin) {
            $env:PATH = "$msvcHostBin;$env:PATH"
        }
    }
}

# [1/6] Stop existing FamShield services and session agents
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

# [2/6] Clean legacy NRPT and configure compliant NRPT routing
Write-Host "[2/6] Don dep NRPT loi va cau hinh DnsPolicyConfig hop le..." -ForegroundColor Yellow
$corruptedKeys = @(
    "HKLM:\SOFTWARE\Policies\Microsoft\Windows NT\DNSClient\DnsPolicyConfig\{FamShield-DNS-Filter}",
    "HKLM:\SYSTEM\CurrentControlSet\Services\Dnscache\Parameters\DnsPolicyConfig\{FamShield-DNS-Filter}"
)
foreach ($key in $corruptedKeys) {
    if (Test-Path $key) {
        Remove-Item -Path $key -Recurse -Force -ErrorAction SilentlyContinue
        Write-Host "      Da xoa khoa cu: $key" -ForegroundColor Green
    }
}

# Configure valid RFC/Win32-compliant NRPT routing for '.' namespace to 127.0.0.2:53
$nrptPaths = @(
    "HKLM:\SOFTWARE\Policies\Microsoft\Windows NT\DnsClient\DnsPolicyConfig\{E6B5D7A1-8843-4C49-B944-C5200E481230}",
    "HKLM:\SYSTEM\CurrentControlSet\Services\Dnscache\Parameters\DnsPolicyConfig\{E6B5D7A1-8843-4C49-B944-C5200E481230}"
)
foreach ($path in $nrptPaths) {
    if (-not (Test-Path $path)) {
        New-Item -Path $path -Force | Out-Null
    }
    Set-ItemProperty -Path $path -Name "Name" -Value @(".") -Type MultiString
    Set-ItemProperty -Path $path -Name "GenericDNSServers" -Value "127.0.0.2" -Type String
}
Write-Host "      Da thiet lap NRPT routing hop le ({E6B5D7A1-8843-4C49-B944-C5200E481230})" -ForegroundColor Green

# [3/6] Build & Copy Binaries and Web Assets
Write-Host "[3/6] Bien dich Rust binaries, Web Admin va cap nhat he thong..." -ForegroundColor Yellow
$serviceSrc = Join-Path $repoRoot "target\release\famshield-service.exe"
$sessionSrc = Join-Path $repoRoot "target\release\famshield-session.exe"

Ensure-BuildEnvironment

if (-not $SkipBuild) {
    # 1. Compile Rust binaries
    if (Get-Command cargo -ErrorAction SilentlyContinue) {
        Write-Host "      Dang bien dich Rust binaries (cargo build --release)..." -ForegroundColor Gray
        & cargo build --release --bin famshield-service --bin famshield-session
        if ($LASTEXITCODE -ne 0) {
            Write-Error "Loi bien dich Rust binaries!"
            exit 1
        }
        Write-Host "      Bien dich Rust binaries thanh cong!" -ForegroundColor Green
    } else {
        Write-Warning "Khong tim thay cargo trong PATH. Bo qua buoc bien dich Rust."
    }

    # 2. Build Web Admin
    $webDir = Join-Path $repoRoot "web"
    if (Test-Path (Join-Path $webDir "package.json")) {
        Push-Location $webDir
        try {
            if (Get-Command pnpm -ErrorAction SilentlyContinue) {
                Write-Host "      Dang bien dich giao dien Web Admin (pnpm run build)..." -ForegroundColor Gray
                & pnpm run build
            } elseif (Get-Command npm -ErrorAction SilentlyContinue) {
                Write-Host "      Dang bien dich giao dien Web Admin (npm run build)..." -ForegroundColor Gray
                & npm run build
            }
        } catch {
            Write-Warning "Khong the bien dich Web Admin: $_"
        } finally {
            Pop-Location
        }
    }
}

if (-not (Test-Path $serviceSrc)) {
    Write-Error "Khong tim thay $serviceSrc. Vui long kiem tra lai qua trinh build!"
    exit 1
}

# Ensure destination directories exist
if (-not (Test-Path $installDir)) {
    New-Item -ItemType Directory -Path $installDir -Force | Out-Null
}

Copy-Item -Path $serviceSrc -Destination $installDir -Force
Copy-Item -Path $sessionSrc -Destination $installDir -Force
Write-Host "      Da cap nhat famshield-service.exe va famshield-session.exe tai $installDir" -ForegroundColor Green

$webDistSrc = Join-Path $repoRoot "web\dist"
if (Test-Path $webDistSrc) {
    # 1. Update ProgramData\FamShield\web\dist (Uu tien so 1 cua LocalAdminServer)
    $pdWebDist = Join-Path $env:ProgramData "FamShield\web\dist"
    if (-not (Test-Path $pdWebDist)) {
        New-Item -ItemType Directory -Path $pdWebDist -Force | Out-Null
    }
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

# [4/6] Restart FamShieldService
Write-Host "[4/6] Khoi dong FamShieldService..." -ForegroundColor Yellow
$svc = Get-Service -Name $serviceName -ErrorAction SilentlyContinue
if ($svc) {
    Start-Service -Name $serviceName -ErrorAction SilentlyContinue
    $svc = Get-Service -Name $serviceName
    Write-Host "      Trang thai service: $($svc.Status)" -ForegroundColor Green
} else {
    Write-Host "      Service FamShield chua duoc cai dat. Vui long chay .\build.ps1 -Action install" -ForegroundColor Yellow
}

# [5/6] Direct DNS on active adapters to 127.0.0.2:53
Write-Host "[5/6] Tro DNS tren cac card mang hoat dong ve 127.0.0.2:53..." -ForegroundColor Yellow
$adapters = Get-NetAdapter -ErrorAction SilentlyContinue | Where-Object { $_.Status -eq 'Up' -and $_.InterfaceAlias -notmatch 'vEthernet|WSL|Loopback' }
foreach ($adapter in $adapters) {
    Set-DnsClientServerAddress -InterfaceIndex $adapter.InterfaceIndex -ServerAddresses '127.0.0.2' -ErrorAction SilentlyContinue
    Write-Host "      Card mang [$($adapter.Name)] ($($adapter.InterfaceAlias)) -> DNS: 127.0.0.2" -ForegroundColor Green
}
Clear-DnsClientCache -ErrorAction SilentlyContinue
ipconfig /flushdns | Out-Null

# [6/6] Launch famshield-session
Write-Host "[6/6] Khoi dong famshield-session..." -ForegroundColor Yellow
$sessionExe = Join-Path $installDir "famshield-session.exe"
if (Test-Path $sessionExe) {
    Start-Process -FilePath $sessionExe
    Write-Host "      Da khoi chay $sessionExe" -ForegroundColor Green
}

Write-Host ""
Write-Host "=======================================================" -ForegroundColor Green
Write-Host " [OK] DA CAP NHAT THANH CONG!" -ForegroundColor Green
Write-Host " - Windows DNS da duoc tro ve 127.0.0.2:53" -ForegroundColor Green
Write-Host " - Khoa NRPT RFC-compliant da duoc dong bo" -ForegroundColor Green
Write-Host " - Truy cap Web Admin: http://127.0.0.1:38123 (Tab Nhat Ky DNS)" -ForegroundColor Green
Write-Host "=======================================================" -ForegroundColor Green
Write-Host ""

if (-not $NoWait) {
    Write-Host "Nhan Enter de hoan tat..."
    Read-Host
}
