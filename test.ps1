<#
.SYNOPSIS
    High-Performance Test Runner for FamShield Agent Workspace.

.DESCRIPTION
    Tối ưu hóa quy trình kiểm thử Rust Workspace:
    - Chạy song song đa luồng xuyên module (Cross-crate Concurrency) với cargo-nextest.
    - Chế độ chỉ test các module bị thay đổi (-Changed) qua git diff (Fast Inner Loop).
    - Hỗ trợ test nhanh từng module cụ thể (-Target <crate_or_alias>).
    - Tự động fallback sang cargo test tiêu chuẩn nếu chưa cài đặt nextest.
    - Đo lường và hiển thị thời gian chạy chi tiết.

.PARAMETER Target
    Tên crate hoặc tên viết tắt (core, storage, dns, windows, service, session, local-admin, sync, updater).

.PARAMETER Changed
    Chỉ chạy test các crates có mã nguồn bị thay đổi trong git (staged hoặc unstaged).

.PARAMETER All
    Chạy toàn bộ test suites của cả workspace (Final Handover Quality Gate).

.PARAMETER Filter
    Chuỗi lọc tên test case cụ thể (tương đương test filter trong cargo/nextest).

.PARAMETER Engine
    Chọn engine test: 'auto' (mặc định), 'nextest', hoặc 'cargo'.

.EXAMPLE
    .\test.ps1 -Changed                  # Siêu nhanh: chỉ test các module vừa sửa code
    .\test.ps1 -Target dns               # Test nhanh crate famshield-dns
    .\test.ps1 -Target local-admin totp  # Test hàm totp trong crate local-admin
    .\test.ps1 -All                      # Chạy toàn bộ test suites song song
#>

[CmdletBinding(DefaultParameterSetName = "Default")]
param (
    [Parameter(Position = 0, ParameterSetName = "Target")]
    [Parameter(Position = 0, ParameterSetName = "Default")]
    [Alias("p", "crate", "module", "m")]
    [string]$Target = "",

    [Parameter(Position = 1, ParameterSetName = "Target")]
    [Parameter(Position = 1, ParameterSetName = "Default")]
    [Alias("f")]
    [string]$Filter = "",

    [Parameter(ParameterSetName = "Changed")]
    [switch]$Changed,

    [Parameter(ParameterSetName = "All")]
    [switch]$All,

    [Parameter()]
    [ValidateSet("auto", "nextest", "cargo")]
    [string]$Engine = "auto"
)

$ErrorActionPreference = "Stop"
$ProjectRoot = $PSScriptRoot

# Bản đồ tên rút gọn sang tên đầy đủ của Crate
$CrateMap = @{
    "core"        = "famshield-core"
    "storage"     = "famshield-storage"
    "dns"         = "famshield-dns"
    "windows"     = "famshield-windows"
    "service"     = "famshield-service"
    "session"     = "famshield-session"
    "local-admin" = "famshield-local-admin"
    "local_admin" = "famshield-local-admin"
    "admin"       = "famshield-local-admin"
    "sync"        = "famshield-sync"
    "updater"     = "famshield-updater"
}

# Bản đồ đường dẫn thư mục sang tên Crate
$PathToCrate = @{
    "agent/crates/core"        = "famshield-core"
    "agent/crates/storage"     = "famshield-storage"
    "agent/crates/dns"         = "famshield-dns"
    "agent/crates/windows"     = "famshield-windows"
    "agent/crates/service"     = "famshield-service"
    "agent/crates/session"     = "famshield-session"
    "agent/crates/local_admin" = "famshield-local-admin"
    "agent/crates/sync"        = "famshield-sync"
    "agent/crates/updater"     = "famshield-updater"
}

function Resolve-CrateName {
    param([string]$name)
    if ([string]::IsNullOrWhiteSpace($name)) { return "" }
    $lower = $name.ToLower().Trim()
    if ($CrateMap.ContainsKey($lower)) {
        return $CrateMap[$lower]
    }
    if ($lower.StartsWith("famshield-")) {
        return $lower
    }
    return "famshield-$lower"
}

function Detect-ChangedCrates {
    $found = @{}
    try {
        $statusOutput = & git status --porcelain 2>$null
        if ($statusOutput) {
            foreach ($line in $statusOutput) {
                if ($line.Length -gt 3) {
                    $file = $line.Substring(3).Trim().Replace("\", "/")
                    foreach ($dir in $PathToCrate.Keys) {
                        if ($file.StartsWith($dir)) {
                            $crate = $PathToCrate[$dir]
                            $found[$crate] = $true
                            break
                        }
                    }
                }
            }
        }
    } catch {
        Write-Warning "Khong the truy van git status de phat hien cac module thay doi."
    }
    return @($found.Keys)
}

function Test-HasNextest {
    try {
        $v = & cargo nextest --version 2>$null
        return ($LASTEXITCODE -eq 0 -and $v -match "cargo-nextest")
    } catch {
        return $false
    }
}

# 1. Xác định Test Engine
$useNextest = $false
if ($Engine -eq "nextest") {
    if (-not (Test-HasNextest)) {
        Write-Error "cargo-nextest chua duoc cai dat. Cai bang: cargo install cargo-nextest --locked"
    }
    $useNextest = $true
} elseif ($Engine -eq "auto") {
    $useNextest = Test-HasNextest
}

$engineLabel = if ($useNextest) { "cargo-nextest (Song song sieu toc)" } else { "cargo test (Tuan tu)" }

Write-Host "=================================================================" -ForegroundColor Cyan
Write-Host "  FamShield Fast Test Runner" -ForegroundColor Green
Write-Host "  Engine: $engineLabel" -ForegroundColor Yellow
Write-Host "=================================================================" -ForegroundColor Cyan

$startTime = [System.Diagnostics.Stopwatch]::StartNew()

# 2. Xác định phạm vi kiểm thử (Scope)
$cratesToTest = @()

if ($All.IsPresent) {
    Write-Host "==> Che do: FULL WORKSPACE (Tat ca modules)" -ForegroundColor Magenta
} elseif ($Changed.IsPresent) {
    Write-Host "==> Che do: CHANGED MODULES (Tu dong phat hien ma thay doi)" -ForegroundColor Magenta
    $cratesToTest = Detect-ChangedCrates
    if ($cratesToTest.Count -eq 0) {
        Write-Host "==> Khong co module nao co thay doi ma nguon! (Tat ca deu sach)" -ForegroundColor Green
        return
    }
    Write-Host "==> Phat hien thay doi tai: $($cratesToTest -join ', ')" -ForegroundColor Yellow
} elseif (-not [string]::IsNullOrWhiteSpace($Target)) {
    $resolved = Resolve-CrateName $Target
    $cratesToTest = @($resolved)
    Write-Host "==> Che do: TARGETED MODULE -> $resolved" -ForegroundColor Magenta
} else {
    # Mặc định: kiểm tra xem có module nào thay đổi không
    $detected = Detect-ChangedCrates
    if ($detected.Count -gt 0) {
        Write-Host "==> Mac dinh: Phat hien $($detected.Count) module dang duoc sua doi -> Chi test cac module nay:" -ForegroundColor Magenta
        Write-Host "    $($detected -join ', ')" -ForegroundColor Yellow
        Write-Host "    (De test toan bo workspace, vui long dung: .\test.ps1 -All)" -ForegroundColor DarkGray
        $cratesToTest = $detected
    } else {
        Write-Host "==> Khong co module nao dang thay doi do dang. Chay test toan bo workspace..." -ForegroundColor Magenta
        $All = $true
    }
}

# 3. Thực thi lệnh Test
$exitCode = 0

if ($useNextest) {
    $cmdArgs = @("nextest", "run")
    
    if (-not $All.IsPresent -and $cratesToTest.Count -gt 0) {
        foreach ($c in $cratesToTest) {
            $cmdArgs += "-p"
            $cmdArgs += $c
        }
    } else {
        $cmdArgs += "--workspace"
    }

    if (-not [string]::IsNullOrWhiteSpace($Filter)) {
        $cmdArgs += "-E"
        $cmdArgs += "test($Filter)"
    }

    Write-Host "==> Thuc thi: cargo $($cmdArgs -join ' ')" -ForegroundColor Cyan
    & cargo @cmdArgs
    $exitCode = $LASTEXITCODE
} else {
    $cmdArgs = @("test")
    
    if (-not $All.IsPresent -and $cratesToTest.Count -gt 0) {
        foreach ($c in $cratesToTest) {
            $cmdArgs += "-p"
            $cmdArgs += $c
        }
    } else {
        $cmdArgs += "--workspace"
    }

    if (-not [string]::IsNullOrWhiteSpace($Filter)) {
        $cmdArgs += "--"
        $cmdArgs += $Filter
    }

    Write-Host "==> Thuc thi: cargo $($cmdArgs -join ' ')" -ForegroundColor Cyan
    & cargo @cmdArgs
    $exitCode = $LASTEXITCODE
}

$startTime.Stop()
$elapsed = [Math]::Round($startTime.Elapsed.TotalSeconds, 2)

Write-Host "-----------------------------------------------------------------" -ForegroundColor Cyan
if ($exitCode -eq 0) {
    Write-Host "  TEST PASSED THANH CONG! (Thoi gian: $elapsed s)" -ForegroundColor Green
} else {
    Write-Host "  TEST FAILED! Ma loi: $exitCode (Thoi gian: $elapsed s)" -ForegroundColor Red
}
Write-Host "=================================================================" -ForegroundColor Cyan

exit $exitCode
